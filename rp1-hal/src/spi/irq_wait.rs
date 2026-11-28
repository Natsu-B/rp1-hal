//! SPI interrupts and synchronous controller adapter.

use super::*;
use crate::{
    driver_irq::{self, Limit, Source},
    timer::WaitTimer,
};

/// Capability carried by SPI parts when the declaration reserves the IRQ for the HAL.
pub struct Irq<const N: u8, const CORE: u8>(PhantomData<core::cell::Cell<()>>);

driver_irq::device_irq!(SpiSource, Io, wait_device,
    (timer: &mut WaitTimer, io: &mut Mmio, mask: u32, end: Deadline) -> Result<u32, Error>;
    0 => Spi0, SPI0;
    1 => Spi1, SPI1;
    3 => Spi3, SPI3;
);

/// # Safety
/// Called only by the generated SPI handler on CORE for instance N.
/// Returns the serviced device bits for the parity fixture.
#[doc(hidden)]
#[inline(always)]
pub unsafe fn __on_interrupt<const N: u8, const CORE: u8>() -> u32 {
    // A pending NVIC entry can outlive controller init recovery.
    if read(RESETS_DONE1) & reset_bit(N) == 0 {
        return 0;
    }
    service_irq::<N, CORE, _>(&mut Mmio::new(N))
}

impl From<driver_irq::Error> for Error {
    fn from(error: driver_irq::Error) -> Self {
        match error {
            driver_irq::Error::Busy => Self::Busy,
            driver_irq::Error::IrqUnavailable => Self::IrqUnavailable,
        }
    }
}

struct SpiSource {
    mask: u32,
}

impl SpiSource {
    const fn new(_instance: u8) -> Self {
        Self { mask: 0 }
    }
}

impl<I: Io> Source<I> for SpiSource {
    type Error = Error;

    fn is_armed(&self) -> bool {
        self.mask != 0
    }

    fn arm(&mut self, io: &mut I, mask: u32) -> Result<bool, Error> {
        if self.mask != 0 {
            return Ok(false);
        }
        if io.read(IMR) != 0 {
            return Err(Error::IrqUnavailable);
        }
        self.mask = mask;
        enable_interrupt(io, mask);
        Ok(true)
    }

    fn retire(&mut self, io: &mut I) {
        if self.mask != 0 {
            io.write(IMR, 0);
            self.mask = 0;
        }
        // Even when the ISR masked the source, retire the RP1 wrapper.
        let wrap = io.read(IRQ_WRAP);
        io.write(IRQ_WRAP, wrap & !irq_wrap::IRQ_EN);
        io.read(ISR);
    }

    fn service(&mut self, io: &mut I) -> u32 {
        let asserted = io.read(ISR);
        let pending = asserted & self.mask;
        if asserted != 0 {
            // Silence a stray level as well. If only a stray source fired,
            // keep the current wait's sources armed.
            io.write(IMR, if pending == 0 { self.mask } else { 0 });
        }
        if pending != 0 {
            self.mask = 0;
        }
        // Error bits stay latched, and FIFO readiness persists until thread I/O.
        // Do not acknowledge errors or touch the FIFO in the ISR.
        pending
    }
}

/// Synchronous SPI controller access using the FIFO/error IRQ, a dedicated wait alarm and WFE.
pub struct Blocking<'a, const N: u8, P: CompletePins<N>, const CORE: u8> {
    spi: &'a mut Spi<N, P, Controller, Irq<N, CORE>>,
    timer: &'a mut WaitTimer,
    config: WaitConfig,
}

impl<P: CompletePins<N>, const CORE: u8, const N: u8> Spi<N, P, Controller, Irq<N, CORE>> {
    /// Borrow the controller for the normal IRQ-backed synchronous operations.
    /// Each operation checks thread context and IRQ delivery before accessing the bus.
    pub fn blocking<'a>(&'a mut self, timer: &'a mut WaitTimer) -> Blocking<'a, N, P, CORE> {
        Blocking {
            spi: self,
            timer,
            config: WaitConfig::new(),
        }
    }
}

impl<const SCK: u8, const MOSI: u8, const MISO: u8, const CORE: u8, const N: u8>
    Spi<N, PinTuple<SCK, MOSI, MISO>, Controller, Irq<N, CORE>>
where
    Pin<SCK>: Sck<N>,
    Pin<MOSI>: Mosi<N>,
    Pin<MISO>: Miso<N>,
{
    /// Wait for wire idle or confirm a previously requested stop, then return
    /// the parts. On failure the error retains the controller and its pins.
    pub fn free_until(
        mut self,
        timer: &mut WaitTimer,
        end: Deadline,
    ) -> Result<Parts<N, PinTuple<SCK, MOSI, MISO>, Irq<N, CORE>>, crate::FreeError<Self, Error>>
    {
        let stopping = self.state == State::Stopping;
        let mut bus = self.blocking(timer);
        let result = if stopping {
            bus.cancel_until(end)
        } else {
            bus.flush_until(end)
        };
        match result {
            Ok(()) => self.try_free(),
            Err(error) => Err(crate::FreeError {
                driver: self,
                error,
            }),
        }
    }
}

impl<P: CompletePins<N>, const CORE: u8, const N: u8> Blocking<'_, N, P, CORE> {
    pub fn with_wait_config(mut self, config: WaitConfig) -> Self {
        self.config = config;
        self
    }

    adapter_config!(spi);

    fn check_context(&mut self) -> Result<(), Error> {
        self.timer.thread_context()?;
        Ok(check_irq::<N, CORE>()?)
    }

    fn reconfigure(&mut self, cr0: u32, div: u32) -> Result<(), Error> {
        self.spi.check_state(u8::MAX)?;
        if self.spi.outstanding != 0 {
            return Err(Error::Busy);
        }
        self.check_context()?;
        let end = default_deadline(self.config)?;
        let mut io = self.spi.io();
        if !io.sys_current_strict(&self.spi.clocks) {
            return Err(Error::Precondition);
        }
        self.flush_admitted(end)?;
        setup(&mut io, cr0, div, self.spi.rsd);
        self.spi.cr0 = cr0;
        self.spi.div = div;
        Ok(())
    }

    /// Exchange frames before the absolute deadline. On failure some frames
    /// may have reached the wire. Use `try_recover` before starting another call.
    pub fn transfer_until<W: Word>(
        &mut self,
        read: &mut [W],
        write: &[W],
        end: Deadline,
    ) -> Result<(), Error> {
        let ok = self.admit(sealed::Key(()), Some(end))?;
        self.exchange(ok, read, write)
    }

    /// Wait for TX FIFO empty and then physical wire idle.
    pub fn flush_until(&mut self, end: Deadline) -> Result<(), Error> {
        self.spi.check_state(u8::MAX)?;
        self.check_context()?;
        end.check(Timer::new().now())?;
        admit(&mut self.spi.io(), &self.spi.clocks)?;
        self.flush_admitted(end)
    }

    /// Request stop and, if necessary, wait until the supplied cleanup
    /// deadline to confirm it. A failed stop retains the owner.
    pub fn cancel_until(&mut self, end: Deadline) -> Result<(), Error> {
        self.timer.thread_context()?;
        end.check(Timer::new().now())?;
        let mut io = self.spi.io();
        let frame = frame_delay(self.spi.bits, self.spi.frequency().raw());
        let mut result = self.spi.cancel_with(&mut io);
        while result == Err(Error::Busy) {
            let now = io.now();
            end.check(now)?;
            self.timer.delay_until((now + frame).min(end.instant()))?;
            result = stop(&mut io);
        }
        result
    }

    fn flush_admitted(&mut self, end: Deadline) -> Result<(), Error> {
        let mut io = self.spi.io();
        let result = flush::<CORE, N>(
            self.timer,
            &mut io,
            end,
            self.spi.bits,
            self.spi.frequency().raw(),
        );
        if result.is_err() {
            let _ = self.spi.cancel_with(&mut io);
        }
        result
    }

    fn exchange_admitted(
        &mut self,
        mut transfer: Exchange<'_>,
        end: Deadline,
    ) -> Result<(), Error> {
        if transfer.len == 0 {
            return Ok(());
        }
        let mut io = self.spi.io();
        let mut watermark_changed = false;
        let result = (|| {
            loop {
                let remaining = end.check(io.now());
                if transfer.step(&mut io, remaining.is_ok())? {
                    return Ok(());
                }
                remaining?;
                // Wait for an RX batch or the remaining tail. TX-empty
                // cannot complete an exchange because the last frame is still shifting.
                let in_flight = transfer.sent - transfer.received;
                let watermark = in_flight.min(transfer.depth.div_ceil(2)) as u32 - 1;
                io.write(RXFTLR, watermark);
                watermark_changed = watermark != 0;
                match wait_for_irq::<N, CORE>(self.timer, &mut io, int::RXFI, end) {
                    Ok(_) => {}
                    Err(Error::Timeout) => {
                        // Drain a final ready batch only. No new frame may start.
                        return if transfer.step(&mut io, false)? {
                            Ok(())
                        } else {
                            Err(Error::Timeout)
                        };
                    }
                    Err(error) => return Err(error),
                }
            }
        })();
        if watermark_changed {
            io.write(RXFTLR, 0);
        }
        if result.is_err() {
            let _ = self.spi.cancel_with(&mut io);
        }
        result
    }
}

fn flush<const CORE: u8, const N: u8>(
    timer: &mut WaitTimer,
    io: &mut Mmio,
    end: Deadline,
    bits: u8,
    hz: u32,
) -> Result<(), Error> {
    error(io.read(RISR))?;
    let status = io.read(SR);
    if status & IDLE == sr::TFE {
        return Ok(());
    }
    let empty = if status & sr::TFE == 0 {
        wait_for_irq::<N, CORE>(timer, io, int::TXEI, end)?
    } else {
        io.now_low()
    };
    if wire_idle(io)? {
        return Ok(());
    }
    flush_tail(timer, io, end, bits, hz, empty)
}

#[rp1_rt::isram]
fn flush_tail(
    timer: &mut WaitTimer,
    io: &mut Mmio,
    end: Deadline,
    bits: u8,
    hz: u32,
    empty: u32,
) -> Result<(), Error> {
    let mut context = timer.thread_context()?;
    // BUSY has no IRQ. Check near the end of the last frame, then use
    // short rechecks to bound late detection if it is still busy.
    let frame = first_tail_ticks(bits, hz);
    let step = frame_delay(1, hz).min(Duration::from_ticks(3));
    let mut first = true;
    loop {
        let now = io.now();
        end.check(now)?;
        let delay = if core::mem::take(&mut first) {
            tail_delay(frame, empty, now.ticks() as u32)
        } else {
            step
        };
        if delay.ticks() != 0 {
            context.delay_accepted(Deadline::at((now + delay).min(end.instant())))?;
        }
        if wire_idle(io)? {
            return Ok(());
        }
    }
}

fn wire_idle(io: &mut impl Io) -> Result<bool, Error> {
    error(io.read(RISR))?;
    Ok(io.read(SR) & IDLE == sr::TFE)
}

fn tail_delay(frame: u32, empty: u32, now: u32) -> Duration {
    Duration::from_ticks(frame.saturating_sub(now.wrapping_sub(empty)))
}

fn first_tail_ticks(bits: u8, hz: u32) -> u32 {
    // Budget two SCLK periods around the last payload after TX empty.
    // Lead the alarm by 3 µs for TX IRQ handling and the wake-to-probe cost.
    frame_delay(bits + 2, hz).ticks().saturating_sub(3)
}

fn frame_delay(bits: u8, hz: u32) -> Duration {
    Duration::from_ticks((u32::from(bits) * 1_000_000).div_ceil(hz) + 1)
}

#[rp1_rt::isram]
fn wait_device<const CORE: u8>(
    state: &IrqState<CORE>,
    timer: &mut WaitTimer,
    io: &mut Mmio,
    mask: u32,
    end: Deadline,
) -> Result<u32, Error> {
    let mask = mask | ERRORS;
    driver_irq::wait_for(state, io, timer, Limit::Absolute(end), mask, |io| {
        ready(io, mask)
    })
}

fn ready(io: &mut impl Io, mask: u32) -> Option<Result<u32, Error>> {
    let status = io.read(RISR);
    (status & mask != 0).then(|| {
        error(status)?;
        // RX batches only need the ready indication.
        Ok(if mask & int::TXEI != 0 {
            io.now_low()
        } else {
            0
        })
    })
}

impl<W: Word, P: CompletePins<N>, const CORE: u8, const N: u8> Bus<W> for Blocking<'_, N, P, CORE> {
    fn admit(
        &mut self,
        _: sealed::Key,
        end: Option<Deadline>,
    ) -> Result<sealed::Admitted<W>, Error> {
        self.spi.check_state(W::BITS)?;
        self.check_context()?;
        let end = end.map_or_else(|| default_deadline(self.config), Ok)?;
        end.check(Timer::new().now())?;
        self.spi.check(W::BITS)?;
        Ok(sealed::Admitted(end, PhantomData))
    }

    fn exchange(
        &mut self,
        ok: sealed::Admitted<W>,
        read: &mut [W],
        write: &[W],
    ) -> Result<(), Error> {
        self.exchange_admitted(
            Exchange::new(
                W::rx(read),
                W::tx(write),
                self.spi.bits,
                self.spi.fifo_depth(),
            ),
            ok.0,
        )
    }

    fn exchange_in_place(&mut self, ok: sealed::Admitted<W>, words: &mut [W]) -> Result<(), Error> {
        in_place(words, |read, write| self.exchange(ok, read, write))
    }

    fn wait_idle(&mut self, ok: sealed::Admitted<W>) -> Result<(), Error> {
        self.flush_admitted(ok.0)
    }

    fn delay(&mut self, ok: sealed::Admitted<W>, ns: u32) -> Result<(), Error> {
        let target = delay_end(Timer::new().now(), ok.0, ns)?;
        self.timer.delay_until(target.min(ok.0.instant()))?;
        if target > ok.0.instant() {
            return Err(Error::Timeout);
        }
        Ok(())
    }

    fn abort(&mut self, _: sealed::Admitted<W>) {
        if self.spi.state != State::Stopping {
            let _ = self.spi.try_cancel();
        }
    }
}

spi_bus!([P: CompletePins<N>, const CORE: u8, const N: u8], Blocking<'_, N, P, CORE>);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spi::tests::Model;

    #[test]
    fn instance_routes_match_register_base_and_clock() {
        use crate::pac;

        assert_eq!(IRQ_ROUTES, &[(0, 16, 19), (1, 17, 20), (3, 19, 22)]);
        for (n, address, clock) in [
            (0, pac::SPI0::PTR, pac::SPI0::CLOCK_DEPENDENCY),
            (1, pac::SPI1::PTR, pac::SPI1::CLOCK_DEPENDENCY),
            (2, pac::SPI2::PTR, pac::SPI2::CLOCK_DEPENDENCY),
            (3, pac::SPI3::PTR, pac::SPI3::CLOCK_DEPENDENCY),
            (5, pac::SPI5::PTR, pac::SPI5::CLOCK_DEPENDENCY),
        ] {
            assert_eq!(Mmio::new(n).base, address as usize, "instance {n}");
            assert_eq!(clock, "ClockId::Sys", "instance {n}");
        }
    }

    #[test]
    fn stray_irq_is_masked_without_waking_or_disarming_the_wait() {
        let mut io = Model::new(4);
        let mut source = SpiSource::new(0);
        io.imr = int::TXEI;
        io.risr = int::TXEI;
        assert_eq!(source.service(&mut io), 0);
        assert_eq!(io.imr, 0);

        assert_eq!(source.arm(&mut io, int::RXFI | ERRORS), Ok(true));
        io.imr |= int::TXEI;
        assert_eq!(source.service(&mut io), 0);
        assert_eq!(io.imr, int::RXFI | ERRORS);
        io.risr = int::RXFI;
        assert_eq!(source.service(&mut io), int::RXFI);
    }

    #[test]
    fn source_retires_and_rearms_without_stale_completion() {
        let mut io = Model::new(4);
        let mut source = SpiSource::new(0);
        assert_eq!(source.arm(&mut io, int::TXEI | ERRORS), Ok(true));
        io.risr = int::TXEI;
        assert_eq!(source.service(&mut io), int::TXEI);
        // A second entry before retirement cannot publish another completion.
        assert_eq!(source.service(&mut io), 0);
        source.retire(&mut io);
        assert_eq!(source.service(&mut io), 0);
        assert_eq!(source.arm(&mut io, int::RXFI | ERRORS), Ok(true));
        assert_eq!(source.service(&mut io), 0);
        io.risr = int::RXOI;
        assert_eq!(source.service(&mut io), int::RXOI);
        assert_eq!(
            ready(&mut io, int::RXFI | ERRORS),
            Some(Err(Error::Overrun))
        );
        source.retire(&mut io);
        io.read(ICR);
        assert_eq!(source.arm(&mut io, int::RXFI | ERRORS), Ok(true));
        assert_eq!(source.service(&mut io), 0);
        io.risr = int::RXFI;
        assert_eq!(source.service(&mut io), int::RXFI);
    }

    #[test]
    fn tx_empty_probe_samples_the_clock() {
        let mut io = Model::new(4);
        let mut clock = Model::new(4);
        io.risr = int::TXEI;
        let empty = ready(&mut io, int::TXEI).unwrap().unwrap();
        assert_eq!(empty, clock.now_low());
    }

    #[test]
    fn first_tail_delay_accounts_for_wake_lead_and_elapsed_ticks() {
        assert_eq!(first_tail_ticks(8, 100_000), 98);
        assert_eq!(first_tail_ticks(8, 1_000_000), 8);
        let before_wrap = u32::MAX - 1;
        let first = first_tail_ticks(8, 1_000_000);
        assert_eq!(tail_delay(first, before_wrap, 2).ticks(), 4);
        assert_eq!(tail_delay(first, before_wrap, 7).ticks(), 0);
    }
}
