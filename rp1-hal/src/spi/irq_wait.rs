//! SPI0's dedicated interrupt and synchronous controller adapter.

use super::*;
use crate::{
    driver_irq::{self, CoreState, Instance, Limit, Managed, PerCore, Source},
    timer::WaitTimer,
};
use rp1_rt::interrupt::{Binding, BlockIrq, Slot};

const SPI: u8 = 0;
const SPI_IRQ: u8 = crate::pac::Interrupt::SPI0 as u8;
type Line<const CORE: u8> = Binding<BlockIrq<{ rp1_proto::BlockId::Spi0 as u16 }>, SPI_IRQ, CORE>;

/// Capability carried by SPI0 parts when the declaration reserves its IRQ for the HAL.
pub struct Irq<const N: u8, const CORE: u8>(PhantomData<core::cell::Cell<()>>);

impl<const CORE: u8, P: CompletePins<SPI>> Parts<SPI, P, Irq<SPI, CORE>> {
    /// # Safety
    /// Only the generated resource constructor may consume this line. It must
    /// install the matching SPI0 handler on the same core.
    #[doc(hidden)]
    pub unsafe fn __new(token: Token<SPI>, pins: Pins<SPI, P>, line: Line<CORE>) -> Self {
        StateSlots::bind::<CORE>(line.into_line());
        Self {
            token,
            pins,
            irq: Irq(PhantomData),
        }
    }
}

static PROC0: CoreState<SpiSource, SPI_IRQ, 0> = Slot::new(Managed::new(SpiSource { mask: 0 }));
static PROC1: CoreState<SpiSource, SPI_IRQ, 1> = Slot::new(Managed::new(SpiSource { mask: 0 }));
type StateSlots = PerCore<SpiSource, SPI_IRQ>;

impl Instance<SPI_IRQ> for SpiSource {
    const PROC0: &'static CoreState<Self, SPI_IRQ, 0> = &PROC0;
    const PROC1: &'static CoreState<Self, SPI_IRQ, 1> = &PROC1;
}

/// # Safety
/// Called only by the generated SPI handler on CORE for instance N.
/// Returns the serviced device bits for the parity fixture.
#[doc(hidden)]
#[inline(always)]
pub unsafe fn __on_interrupt<const N: u8, const CORE: u8>() -> u32 {
    const { assert!(N == SPI) };
    // A stale NVIC entry can arrive while init recovery holds SPI in reset.
    if read(RESETS_DONE1) & reset_bit(N) == 0 {
        return 0;
    }
    StateSlots::on_interrupt::<CORE, _>(&mut Mmio { base: base(N) })
}

impl From<driver_irq::Error> for Error {
    fn from(error: driver_irq::Error) -> Self {
        match error {
            driver_irq::Error::Busy => Self::Busy,
            driver_irq::Error::IrqUnavailable => Self::IrqUnavailable,
        }
    }
}

#[derive(Default)]
struct SpiSource {
    mask: u32,
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

/// Synchronous SPI0 access using the FIFO/error IRQ, a dedicated wait alarm and WFE.
pub struct Blocking<'a, P: CompletePins<0>, const CORE: u8> {
    spi: &'a mut Spi<0, P, Controller, Irq<0, CORE>>,
    timer: &'a mut WaitTimer,
    config: WaitConfig,
}

impl<P: CompletePins<0>, const CORE: u8> Spi<0, P, Controller, Irq<0, CORE>> {
    /// Borrow the controller for the normal IRQ-backed synchronous operations.
    /// Each operation checks thread context and IRQ delivery before accessing the bus.
    pub fn blocking<'a>(&'a mut self, timer: &'a mut WaitTimer) -> Blocking<'a, P, CORE> {
        Blocking {
            spi: self,
            timer,
            config: WaitConfig::new(),
        }
    }
}

impl<const SCK: u8, const MOSI: u8, const MISO: u8, const CORE: u8>
    Spi<0, PinTuple<SCK, MOSI, MISO>, Controller, Irq<0, CORE>>
where
    Pin<SCK>: Sck<0>,
    Pin<MOSI>: Mosi<0>,
    Pin<MISO>: Miso<0>,
{
    /// Wait for wire idle or confirm a previously requested stop, then return
    /// the parts. On failure the error retains the controller and its pins.
    pub fn free_until(
        mut self,
        timer: &mut WaitTimer,
        end: Deadline,
    ) -> Result<Parts<0, PinTuple<SCK, MOSI, MISO>, Irq<0, CORE>>, crate::FreeError<Self, Error>>
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

impl<P: CompletePins<0>, const CORE: u8> Blocking<'_, P, CORE> {
    pub fn with_wait_config(mut self, config: WaitConfig) -> Self {
        self.config = config;
        self
    }

    adapter_config!(spi);

    fn check_context(&mut self) -> Result<(), Error> {
        self.timer.thread_context()?;
        Ok(StateSlots::check::<CORE>()?)
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
        let result = flush::<CORE>(
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
                match wait_for::<CORE>(self.timer, &mut io, int::RXFI, end) {
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

fn flush<const CORE: u8>(
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
        wait_for::<CORE>(timer, io, int::TXEI, end)?
    } else {
        io.now_low()
    };
    // Anchor the first check to TX empty so waiter retirement does not extend
    // the estimated frame. Start a bit early and let BUSY decide when to return.
    let frame = frame_delay(bits - 1, hz).ticks();
    let bit = frame_delay(1, hz);
    let mut first = true;
    loop {
        error(io.read(RISR))?;
        if io.read(SR) & IDLE == sr::TFE {
            return Ok(());
        }
        let now = io.now();
        end.check(now)?;
        let delay = if core::mem::take(&mut first) {
            tail_delay(frame, empty, now.ticks() as u32)
        } else {
            bit
        };
        timer.delay_until((now + delay).min(end.instant()))?;
    }
}

fn tail_delay(frame: u32, empty: u32, now: u32) -> Duration {
    Duration::from_ticks(frame.saturating_sub(now.wrapping_sub(empty)))
}

fn frame_delay(bits: u8, hz: u32) -> Duration {
    Duration::from_ticks((u32::from(bits) * 1_000_000).div_ceil(hz) + 1)
}

#[rp1_rt::isram]
fn wait_for<const CORE: u8>(
    timer: &mut WaitTimer,
    io: &mut Mmio,
    mask: u32,
    end: Deadline,
) -> Result<u32, Error> {
    let mask = mask | ERRORS;
    StateSlots::wait_for::<CORE, _, _>(io, timer, Limit::Absolute(end), mask, |io| ready(io, mask))
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

impl<W: Word, P: CompletePins<0>, const CORE: u8> Bus<W> for Blocking<'_, P, CORE> {
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

spi_bus!([P: CompletePins<0>, const CORE: u8], Blocking<'_, P, CORE>);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spi::tests::Model;

    #[test]
    fn stray_irq_is_masked_without_waking_or_disarming_the_wait() {
        let mut io = Model::new(4);
        let mut source = SpiSource::default();
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
        let mut source = SpiSource::default();
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
    fn tx_empty_timestamp_excludes_waiter_retirement_across_wrap() {
        let mut io = Model::new(4);
        let mut clock = Model::new(4);
        io.risr = int::TXEI;
        let empty = ready(&mut io, int::TXEI).unwrap().unwrap();
        assert_eq!(empty, clock.now_low());
        let before_wrap = u32::MAX - 1;
        assert_eq!(tail_delay(8, before_wrap, 2).ticks(), 4);
        assert_eq!(tail_delay(8, before_wrap, 7).ticks(), 0);
    }
}
