//! PL011 interrupt and thread-mode adapters.

use super::*;
use crate::{
    driver_irq::{self, Limit, Source},
    timer::{Instant, WaitTimer},
};

/// Capability carried by UART parts when the declaration reserves the IRQ for the HAL.
#[derive(Clone, Copy)]
// A split half can be put in its core's Slot for try I/O.
pub struct Irq<const N: u8, const CORE: u8>(PhantomData<core::cell::Cell<()>>);

driver_irq::device_irq!(State, Registers, wait_device,
    (base: usize, wait: &mut WaitTimer, end: Deadline, operation: Operation<'_>) -> Result<usize, Error>;
    0 => Uart0, UART0;
    1 => Uart1, UART1;
    2 => Uart2, UART2;
    3 => Uart3, UART3;
    4 => Uart4, UART4;
);

/// # Safety
/// Called only by the generated UART handler on CORE for instance N.
/// Returns the serviced device bits for the parity fixture.
#[doc(hidden)]
#[inline(always)]
pub unsafe fn __on_interrupt<const N: u8, const CORE: u8>() -> u32 {
    service_irq::<N, CORE, _>(&mut Mmio)
}

impl From<driver_irq::Error> for Error {
    fn from(error: driver_irq::Error) -> Self {
        match error {
            driver_irq::Error::Busy => Self::Busy,
            driver_irq::Error::IrqUnavailable => Self::IrqUnavailable,
        }
    }
}

struct State {
    base: usize,
    reset_bit: u32,
    source: u32,
}

impl State {
    const fn new(instance: u8) -> Self {
        Self {
            base: base(instance),
            reset_bit: reset_bit(instance),
            source: 0,
        }
    }
}

impl<R: Registers> Source<R> for State {
    type Error = Error;
    const INLINE_CHECK: bool = true;

    fn is_armed(&self) -> bool {
        self.source != 0
    }

    // Return true when a post-arm probe is needed.
    fn arm(&mut self, io: &mut R, source: u32) -> Result<bool, Error> {
        if self.source != 0 {
            return Ok(false);
        }
        if io.read(self.base + IMSC) != 0 {
            return Err(Error::IrqUnavailable);
        }
        self.source = source;
        io.write(self.base + IMSC, source);
        Ok(true)
    }

    fn service(&mut self, io: &mut R) -> u32 {
        // A stale NVIC entry can arrive after the driver asserted reset.
        if self.source == 0 && io.read(RESETS_DONE1) & self.reset_bit == 0 {
            return 0;
        }
        let asserted = io.read(self.base + MIS);
        let owned = asserted & self.source;
        // Retire the whole waiting direction on a hit. Mask other asserted
        // sources too, so an accidental raw unmask cannot hold the line high.
        let clear = (if owned != 0 { self.source } else { 0 }) | (asserted & !owned);
        if clear != 0 {
            let imsc = io.read(self.base + IMSC);
            io.write(self.base + IMSC, imsc & !clear);
        }
        if owned != 0 {
            self.source = 0;
        }
        owned
    }

    fn retire(&mut self, io: &mut R) {
        if self.source != 0 {
            let imsc = io.read(self.base + IMSC);
            io.write(self.base + IMSC, imsc & !self.source);
            self.source = 0;
        }
    }
}

// All pin routes and adapter shapes share the same waiter and peripheral code.
enum Operation<'a> {
    Read(&'a mut u8),
    Write(&'a mut TxState, &'a [u8]),
    TxRoom,
}

impl Operation<'_> {
    // The initial probe and the wake probe share FIFO access code.
    #[inline(never)]
    fn probe(&mut self, base: usize, io: &mut Mmio) -> Option<Result<usize, Error>> {
        match self {
            Self::Read(first) => read_byte(io, base).transpose().map(|value| {
                **first = value?;
                Ok(1)
            }),
            Self::Write(tx, bytes) => tx.enqueue(base, bytes).map(Ok),
            Self::TxRoom => {
                let flags = io.read(base + FR);
                (flags & FR_TXFF == 0).then_some(Ok(flags as usize))
            }
        }
    }
}

fn read_flags<const N: u8>() -> u32 {
    Mmio.read(base(N) + FR)
}

#[rp1_rt::isram]
fn wait_device<const CORE: u8>(
    state: &IrqState<CORE>,
    base: usize,
    wait: &mut WaitTimer,
    end: Deadline,
    mut operation: Operation<'_>,
) -> Result<usize, Error> {
    let source = match operation {
        Operation::Read(_) => RX_INTERRUPTS,
        Operation::Write(..) | Operation::TxRoom => int::TX,
    };
    driver_irq::wait_for(state, &mut Mmio, wait, Limit::Absolute(end), source, |io| {
        operation.probe(base, io)
    })
}

/// Synchronous UART I/O using its device IRQ, the dedicated wait alarm and WFE.
/// Release this borrow before splitting or freeing the UART.
pub struct Blocking<'a, const N: u8, const CORE: u8> {
    tx: &'a mut TxState,
    rx: &'a RxState,
    wait: &'a mut WaitTimer,
    timeout: Option<MicrosDurationU32>,
}

/// Synchronous UART transmit and wire-idle waits on a split half.
pub struct BlockingTx<'a, const N: u8, const CORE: u8> {
    tx: &'a mut TxState,
    wait: &'a mut WaitTimer,
    timeout: Option<MicrosDurationU32>,
}

/// Synchronous UART receive waits on a split half.
pub struct BlockingRx<'a, const N: u8, const CORE: u8> {
    rx: &'a RxState,
    wait: &'a mut WaitTimer,
    timeout: Option<MicrosDurationU32>,
}

macro_rules! adapter_timeout {
    ($($adapter:ident),+) => {$(
        impl<const CORE: u8, const N: u8> $adapter<'_, N, CORE> {
            /// Override the default timeout for trait calls on this borrow.
            /// Explicit `*_until` calls use their supplied deadline.
            pub fn with_timeout(mut self, timeout: MicrosDurationU32) -> Self {
                self.timeout = Some(timeout);
                self
            }
        }

        impl<const CORE: u8, const N: u8> embedded_io::ErrorType for $adapter<'_, N, CORE> {
            type Error = Error;
        }
    )+};
}

adapter_timeout!(Blocking, BlockingTx, BlockingRx);

impl<const PIN: u8, const CORE: u8, const N: u8> Rx<N, PIN, Irq<N, CORE>> {
    /// Borrow this half for synchronous IRQ-backed reads.
    pub fn blocking<'a>(&'a mut self, wait: &'a mut WaitTimer) -> BlockingRx<'a, N, CORE> {
        BlockingRx {
            rx: &self.state,
            wait,
            timeout: None,
        }
    }
}

impl<const PIN: u8, const CORE: u8, const N: u8> Tx<N, PIN, Irq<N, CORE>> {
    /// Borrow this half for synchronous IRQ-backed writes and flush.
    pub fn blocking<'a>(&'a mut self, wait: &'a mut WaitTimer) -> BlockingTx<'a, N, CORE> {
        BlockingTx {
            tx: &mut self.state,
            wait,
            timeout: None,
        }
    }
}

impl<const TX: u8, const RX: u8, const CORE: u8, const N: u8>
    Uart<N, PinTuple<TX, RX>, Irq<N, CORE>>
where
    Pin<TX>: TxPin<N>,
    Pin<RX>: RxPin<N>,
{
    /// Borrow the UART for synchronous IRQ-backed I/O.
    pub fn blocking<'a>(&'a mut self, wait: &'a mut WaitTimer) -> Blocking<'a, N, CORE> {
        Blocking {
            tx: &mut self.halves.0.state,
            rx: &self.halves.1.state,
            wait,
            timeout: None,
        }
    }
}

impl<const CORE: u8, const N: u8> BlockingRx<'_, N, CORE> {
    /// Read one byte or return a receive error. An empty slice returns zero.
    /// A timeout retires the wait. Later reads can still receive incoming bytes.
    pub fn read_until(&mut self, bytes: &mut [u8], end: Deadline) -> Result<usize, Error> {
        let Some(first) = bytes.first_mut() else {
            return Ok(0);
        };
        if self.rx.resetting {
            return Err(Error::Busy);
        }
        wait_for_irq::<N, CORE>(base(N), self.wait, end, Operation::Read(first))
    }

    /// Read the whole slice with one deadline. On failure the received prefix remains in it.
    pub fn read_exact_until(&mut self, bytes: &mut [u8], end: Deadline) -> Result<(), Error> {
        for byte in bytes {
            self.read_until(core::slice::from_mut(byte), end)?;
        }
        Ok(())
    }
}

impl<const CORE: u8, const N: u8> BlockingTx<'_, N, CORE> {
    /// Wait for space then queue up to one FIFO of bytes. A timeout queues no bytes.
    pub fn write_until(&mut self, bytes: &[u8], end: Deadline) -> Result<usize, Error> {
        if bytes.is_empty() {
            return Ok(0);
        }
        if self.tx.resetting {
            return Err(Error::Busy);
        }
        wait_for_irq::<N, CORE>(base(N), self.wait, end, Operation::Write(self.tx, bytes))
    }

    /// Queue the whole slice with one deadline. On failure an accepted prefix still transmits.
    pub fn write_all_until(&mut self, mut bytes: &[u8], end: Deadline) -> Result<(), Error> {
        while !bytes.is_empty() {
            let written = self.write_until(bytes, end)?;
            bytes = &bytes[written..];
        }
        Ok(())
    }

    /// Wait for the last stop bit to leave the wire. The FIFO IRQ reports space.
    /// A timer alarm checks BUSY after the estimated FIFO tail and then in
    /// single-bit intervals. After timeout, queued bytes continue to transmit.
    /// Retry flush or rejoin the halves to reset the UART.
    pub fn flush_until(&mut self, end: Deadline) -> Result<(), Error> {
        if self.tx.resetting {
            return Err(Error::Busy);
        }
        // A full FIFO guarantees a later crossing of the TX watermark.
        let flags = wait_for_irq::<N, CORE>(base(N), self.wait, end, Operation::TxRoom)? as u32;
        if flags & FR_BUSY == 0 {
            return Ok(());
        }
        let bit_us = self.tx.divisor.div_ceil(DIVISOR_PER_BIT_US);
        let now = Timer::new().now();
        let tail_us = self
            .tx
            .remaining_ticks(now.ticks() as u32)
            .min(self.tx.frame_ticks(if flags & FR_TXFE == 0 {
                FIFO_SIZE as u8
            } else {
                0
            }));
        let first = now.ticks().saturating_add(u64::from(tail_us));
        let mut context = self.wait.thread_context()?;
        flush_tail(
            flags,
            first,
            bit_us,
            end,
            || Timer::new().now(),
            |next| {
                context.delay_until(next)?;
                Ok(read_flags::<N>())
            },
        )
    }

    fn tx_deadline(&self) -> Result<Deadline, Error> {
        default_end(
            self.timeout
                .unwrap_or(MicrosDurationU32::micros(self.tx.bound_us)),
        )
    }
}

fn flush_tail(
    mut flags: u32,
    first: u64,
    bit_us: u32,
    end: Deadline,
    mut now: impl FnMut() -> Instant,
    mut wake: impl FnMut(Instant) -> Result<u32, Error>,
) -> Result<(), Error> {
    loop {
        if flags & FR_BUSY == 0 {
            return Ok(());
        }
        let now = now();
        if now >= end.instant() {
            return Err(Error::Timeout);
        }
        // Every alarm is capped by the same absolute deadline.
        // Once the first deadline passes, only single-bit sleeps remain.
        let bit = now.ticks().saturating_add(u64::from(bit_us));
        let next = first.max(bit).min(end.instant().ticks());
        flags = wake(Instant::from_ticks(next))?;
    }
}

impl<const CORE: u8, const N: u8> Blocking<'_, N, CORE> {
    fn rx(&mut self) -> BlockingRx<'_, N, CORE> {
        BlockingRx {
            rx: self.rx,
            wait: self.wait,
            timeout: self.timeout,
        }
    }

    fn tx(&mut self) -> BlockingTx<'_, N, CORE> {
        BlockingTx {
            tx: self.tx,
            wait: self.wait,
            timeout: self.timeout,
        }
    }

    /// See [`BlockingRx::read_until`].
    pub fn read_until(&mut self, bytes: &mut [u8], end: Deadline) -> Result<usize, Error> {
        self.rx().read_until(bytes, end)
    }

    /// See [`BlockingRx::read_exact_until`].
    pub fn read_exact_until(&mut self, bytes: &mut [u8], end: Deadline) -> Result<(), Error> {
        self.rx().read_exact_until(bytes, end)
    }

    /// See [`BlockingTx::write_until`].
    pub fn write_until(&mut self, bytes: &[u8], end: Deadline) -> Result<usize, Error> {
        self.tx().write_until(bytes, end)
    }

    /// See [`BlockingTx::write_all_until`].
    pub fn write_all_until(&mut self, bytes: &[u8], end: Deadline) -> Result<(), Error> {
        self.tx().write_all_until(bytes, end)
    }

    /// See [`BlockingTx::flush_until`].
    pub fn flush_until(&mut self, end: Deadline) -> Result<(), Error> {
        self.tx().flush_until(end)
    }
}

impl<const CORE: u8, const N: u8> embedded_io::ReadReady for BlockingRx<'_, N, CORE> {
    fn read_ready(&mut self) -> Result<bool, Error> {
        self.rx.ready_to_read(N)
    }
}

impl<const CORE: u8, const N: u8> embedded_io::WriteReady for BlockingTx<'_, N, CORE> {
    fn write_ready(&mut self) -> Result<bool, Error> {
        self.tx.ready_to_write(N)
    }
}

impl<const CORE: u8, const N: u8> embedded_io::ReadReady for Blocking<'_, N, CORE> {
    fn read_ready(&mut self) -> Result<bool, Error> {
        self.rx.ready_to_read(N)
    }
}

impl<const CORE: u8, const N: u8> embedded_io::WriteReady for Blocking<'_, N, CORE> {
    fn write_ready(&mut self) -> Result<bool, Error> {
        self.tx.ready_to_write(N)
    }
}

impl<const CORE: u8, const N: u8> embedded_io::Read for BlockingRx<'_, N, CORE> {
    fn read(&mut self, bytes: &mut [u8]) -> Result<usize, Error> {
        if bytes.is_empty() {
            return Ok(0);
        }
        let end = default_end(self.timeout.unwrap_or(DEFAULT_READ_TIMEOUT))?;
        self.read_until(bytes, end)
    }
}

impl<const CORE: u8, const N: u8> embedded_io::Write for BlockingTx<'_, N, CORE> {
    fn write(&mut self, bytes: &[u8]) -> Result<usize, Error> {
        if bytes.is_empty() {
            return Ok(0);
        }
        self.write_until(bytes, self.tx_deadline()?)
    }

    fn flush(&mut self) -> Result<(), Error> {
        self.flush_until(self.tx_deadline()?)
    }
}

impl<const CORE: u8, const N: u8> embedded_io::Read for Blocking<'_, N, CORE> {
    fn read(&mut self, bytes: &mut [u8]) -> Result<usize, Error> {
        self.rx().read(bytes)
    }
}

impl<const CORE: u8, const N: u8> embedded_io::Write for Blocking<'_, N, CORE> {
    fn write(&mut self, bytes: &[u8]) -> Result<usize, Error> {
        self.tx().write(bytes)
    }

    fn flush(&mut self) -> Result<(), Error> {
        self.tx().flush()
    }
}

macro_rules! impl_fmt_write {
    ($($adapter:ident),+) => {$(
        impl<const CORE: u8, const N: u8> fmt::Write for $adapter<'_, N, CORE> {
            fn write_str(&mut self, s: &str) -> fmt::Result {
                embedded_io::Write::write_all(self, s.as_bytes()).map_err(|_| fmt::Error)
            }
        }
    )+};
}

impl_fmt_write!(Blocking, BlockingTx);
#[cfg(test)]
mod tests {
    use super::*;
    use core::cell::Cell;

    #[derive(Default)]
    struct Pl011<const N: u8 = 1> {
        mask: u32,
        asserted: u32,
        assert_on_unmask: u32,
        writes: usize,
    }

    impl<const N: u8> Registers for Pl011<N> {
        fn read(&mut self, address: usize) -> u32 {
            if address == RESETS_DONE1 {
                return reset_bit(N);
            }
            match address - base(N) {
                IMSC => self.mask,
                MIS => self.asserted & self.mask,
                _ => panic!("unexpected register"),
            }
        }
        fn write(&mut self, address: usize, value: u32) {
            assert_eq!(address, base(N) + IMSC);
            self.writes += 1;
            self.mask = value;
            if value != 0 {
                self.asserted |= self.assert_on_unmask;
            }
        }
    }

    #[test]
    fn instance_routes_match_register_base_and_clock() {
        use crate::pac;

        assert_eq!(
            IRQ_ROUTES,
            &[(0, 0, 25), (1, 1, 42), (2, 2, 43), (3, 3, 44), (4, 4, 45)]
        );
        for (n, address, clock) in [
            (0, pac::UART0::PTR, pac::UART0::CLOCK_DEPENDENCY),
            (1, pac::UART1::PTR, pac::UART1::CLOCK_DEPENDENCY),
            (2, pac::UART2::PTR, pac::UART2::CLOCK_DEPENDENCY),
            (3, pac::UART3::PTR, pac::UART3::CLOCK_DEPENDENCY),
            (4, pac::UART4::PTR, pac::UART4::CLOCK_DEPENDENCY),
        ] {
            assert_eq!(base(n), address as usize, "instance {n}");
            assert_eq!(clock, "uartclk and apb_pclk", "instance {n}");
        }
    }

    #[test]
    fn each_source_only_services_its_own_pl011() {
        fn exercise<const N: u8>() {
            let mut io = Pl011::<N>::default();
            let mut state = State::new(N);
            state.arm(&mut io, RX_INTERRUPTS).unwrap();
            io.asserted = int::RT;
            assert_eq!(state.service(&mut io), int::RT);
            assert_eq!(io.mask, 0);
        }

        exercise::<0>();
        exercise::<1>();
        exercise::<2>();
        exercise::<3>();
        exercise::<4>();
    }

    #[test]
    fn source_assertion_before_during_and_after_unmask_is_seen() {
        for source in [
            int::RX,
            int::RT,
            int::TX,
            int::OE,
            int::FE,
            int::PE,
            int::BE,
        ] {
            for phase in 0..3 {
                let mut io = Pl011::<1>::default();
                let mut state = State::new(1);
                let mask = if source == int::TX {
                    source
                } else {
                    RX_INTERRUPTS
                };
                if phase == 0 {
                    io.asserted = source;
                } else if phase == 1 {
                    io.assert_on_unmask = source;
                }
                assert_eq!(state.arm(&mut io, mask), Ok(true));
                if phase == 2 {
                    assert_eq!(state.service(&mut io), 0);
                    io.asserted = source;
                }
                assert_eq!(state.service(&mut io), source);
                assert_eq!((io.mask, io.asserted), (0, source));
                assert_eq!(state.service(&mut io), 0);
                state.retire(&mut io);
                assert_eq!(io.writes, 2);
            }
        }
    }

    #[test]
    fn timed_out_source_cannot_wake_next_direction_but_incoming_data_is_retained() {
        let mut io = Pl011::<1>::default();
        let mut state = State::new(1);
        assert_eq!(state.arm(&mut io, RX_INTERRUPTS), Ok(true));
        // The deadline alarm ended the operation without a UART IRQ.
        state.retire(&mut io);
        io.asserted = int::RT;
        assert_eq!(state.service(&mut io), 0);
        assert_eq!(state.arm(&mut io, int::TX), Ok(true));
        assert_eq!(state.service(&mut io), 0);
        assert_eq!(io.mask, int::TX);
        state.retire(&mut io);
        assert_eq!(state.arm(&mut io, RX_INTERRUPTS), Ok(true));
        assert_eq!(state.service(&mut io), int::RT);
    }

    #[test]
    fn stray_assertions_are_masked_without_retiring_an_unrelated_wait() {
        let mut io = Pl011 {
            mask: int::RX | int::TX,
            asserted: int::RX | int::TX,
            ..Pl011::<1>::default()
        };
        let mut state = State::new(1);
        assert_eq!(state.service(&mut io), 0);
        assert_eq!(io.mask, 0);

        state.arm(&mut io, RX_INTERRUPTS).unwrap();
        io.mask |= int::TX;
        io.asserted = int::TX;
        assert_eq!(state.service(&mut io), 0);
        assert_eq!(io.mask, RX_INTERRUPTS);
        assert_ne!(state.source, 0);

        io.mask |= int::TX;
        io.asserted |= int::RX;
        assert_eq!(state.service(&mut io), int::RX);
        assert_eq!(io.mask, 0);
        assert_eq!(state.source, 0);
    }

    #[test]
    fn flush_hint_decays_with_time_and_across_wrap() {
        let mut tx = TxState {
            bound_us: 5_000,
            divisor: 1736,
            earliest_end: 0,
            resetting: false,
        };
        let before = u32::MAX - 100;
        tx.record_write(FR_BUSY | FR_TXFF, 0, before);
        tx.record_write(FR_BUSY, 2, before.wrapping_add(16 * 87));
        let after = before.wrapping_add(2_777 + 173);
        assert_eq!(tx.earliest_end, after);
        assert_eq!(tx.remaining_ticks(after.wrapping_sub(7)), 7);
        assert_eq!(tx.remaining_ticks(after.wrapping_add(1)), 0);

        tx.record_write(FR_BUSY | FR_TXFE, 1, before);
        assert_eq!(tx.earliest_end, before.wrapping_add(86));
    }

    #[test]
    fn foreign_mask_is_preserved() {
        let mut io = Pl011 {
            mask: int::TX,
            ..Pl011::<1>::default()
        };
        let mut state = State::new(1);
        assert_eq!(
            state.arm(&mut io, RX_INTERRUPTS),
            Err(Error::IrqUnavailable)
        );
        state.retire(&mut io);
        assert_eq!((io.mask, io.writes), (int::TX, 0));
    }

    #[test]
    fn physical_flush_needs_wire_idle_and_can_finish_on_the_deadline() {
        for idle in [false, true] {
            let now = Cell::new(100);
            let mut wakes = 0;
            let end = Deadline::at(Instant::from_ticks(135));
            let got = flush_tail(
                FR_BUSY,
                120,
                8,
                end,
                || Instant::from_ticks(now.get()),
                |next| {
                    wakes += 1;
                    assert_eq!(next.ticks(), [120, 128, 135][wakes - 1]);
                    now.set(next.ticks());
                    Ok(FR_TXFE | if wakes == 3 && idle { 0 } else { FR_BUSY })
                },
            );
            assert_eq!(wakes, 3);
            assert_eq!(got, if idle { Ok(()) } else { Err(Error::Timeout) });
        }
    }
}
