//! UART1's shared PL011 interrupt and thread-mode adapters.

use super::*;
use crate::{
    driver_irq::{self, CoreState, Instance, Limit, Managed, PerCore, Source},
    timer::{Instant, WaitTimer},
};
use rp1_rt::interrupt::{Binding, BlockIrq, Slot};

const UART_IRQ: u8 = rp1_pac::Interrupt::UART1 as u8;
type Line<const CORE: u8> = Binding<BlockIrq<{ rp1_proto::BlockId::Uart1 as u16 }>, UART_IRQ, CORE>;
const UART: u8 = 1;

/// Capability carried by UART1 parts when their declaration reserves the IRQ for the HAL.
#[derive(Clone, Copy)]
// Like the pins and binding, a split half can be put in its core's Slot for try I/O.
pub struct Irq<const N: u8, const CORE: u8>(PhantomData<core::cell::Cell<()>>);

impl<const CORE: u8, P: CompletePins<UART>> Parts<UART, P, Irq<UART, CORE>> {
    /// # Safety
    /// Only the generated resource constructor may consume this line. It must
    /// install the matching UART1 handler on the same core.
    #[doc(hidden)]
    pub unsafe fn __new(token: Token<UART>, pins: Pins<UART, P>, line: Line<CORE>) -> Self {
        StateSlots::bind::<CORE>(line.into_line());
        Self {
            token,
            pins,
            irq: Irq(PhantomData),
        }
    }
}

static PROC0: CoreState<State, UART_IRQ, 0> = Slot::new(Managed::new(State::new()));
static PROC1: CoreState<State, UART_IRQ, 1> = Slot::new(Managed::new(State::new()));
type StateSlots = PerCore<State, UART_IRQ>;

impl Instance<UART_IRQ> for State {
    const PROC0: &'static CoreState<Self, UART_IRQ, 0> = &PROC0;
    const PROC1: &'static CoreState<Self, UART_IRQ, 1> = &PROC1;
}

/// # Safety
/// Called only by the generated UART handler on CORE for instance N.
/// Returns the serviced PL011 bits for the parity fixture.
#[doc(hidden)]
#[inline(always)]
pub unsafe fn __on_interrupt<const N: u8, const CORE: u8>() -> u32 {
    const { assert!(N == UART) };
    StateSlots::on_interrupt::<CORE, _>(&mut Mmio)
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
struct State {
    source: u32,
}

impl State {
    const fn new() -> Self {
        Self { source: 0 }
    }
}

impl<R: Registers> Source<R> for State {
    type Error = Error;

    fn is_armed(&self) -> bool {
        self.source != 0
    }

    // Return true when a post-arm probe is needed.
    fn arm(&mut self, io: &mut R, source: u32) -> Result<bool, Error> {
        if self.source != 0 {
            return Ok(false);
        }
        if io.read(base(UART) + IMSC) != 0 {
            return Err(Error::IrqUnavailable);
        }
        self.source = source;
        io.write(base(UART) + IMSC, source);
        Ok(true)
    }

    fn service(&mut self, io: &mut R) -> u32 {
        // A stale NVIC entry can arrive after the driver asserted reset.
        if self.source == 0 && io.read(RESETS_DONE1) & reset_bit(UART) == 0 {
            return 0;
        }
        let asserted = io.read(base(UART) + MIS);
        let owned = asserted & self.source;
        // Retire the whole waiting direction on a hit. Mask other asserted
        // sources too, so an accidental raw unmask cannot hold the line high.
        let clear = (if owned != 0 { self.source } else { 0 }) | (asserted & !owned);
        if clear != 0 {
            let imsc = io.read(base(UART) + IMSC);
            io.write(base(UART) + IMSC, imsc & !clear);
        }
        if owned != 0 {
            self.source = 0;
        }
        owned
    }

    fn retire(&mut self, io: &mut R) {
        if self.source != 0 {
            let imsc = io.read(base(UART) + IMSC);
            io.write(base(UART) + IMSC, imsc & !self.source);
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
    fn probe(&mut self, io: &mut Mmio) -> Option<Result<usize, Error>> {
        match self {
            Self::Read(first) => read_byte(io, UART).transpose().map(|value| {
                **first = value?;
                Ok(1)
            }),
            Self::Write(tx, bytes) => tx.enqueue(UART, bytes).map(Ok),
            Self::TxRoom => {
                let flags = io.read(base(UART) + FR);
                (flags & FR_TXFF == 0).then_some(Ok(flags as usize))
            }
        }
    }
}

fn read_flags() -> u32 {
    Mmio.read(base(UART) + FR)
}

#[rp1_rt::isram]
fn wait_for<const CORE: u8>(
    wait: &mut WaitTimer,
    end: Deadline,
    mut operation: Operation<'_>,
) -> Result<usize, Error> {
    let source = match operation {
        Operation::Read(_) => RX_INTERRUPTS,
        Operation::Write(..) | Operation::TxRoom => int::TX,
    };
    StateSlots::wait_for::<CORE, _, _>(&mut Mmio, wait, Limit::Absolute(end), source, |io| {
        operation.probe(io)
    })
}

/// Synchronous UART1 I/O using its device IRQ, the dedicated wait alarm and WFE.
/// Release this borrow before splitting or freeing the UART.
pub struct Blocking<'a, const CORE: u8> {
    tx: &'a mut TxState,
    rx: &'a RxState,
    wait: &'a mut WaitTimer,
    timeout: Option<MicrosDurationU32>,
}

/// Synchronous UART1 transmit and wire-idle waits on a split half.
pub struct BlockingTx<'a, const CORE: u8> {
    tx: &'a mut TxState,
    wait: &'a mut WaitTimer,
    timeout: Option<MicrosDurationU32>,
}

/// Synchronous UART1 receive waits on a split half.
pub struct BlockingRx<'a, const CORE: u8> {
    rx: &'a RxState,
    wait: &'a mut WaitTimer,
    timeout: Option<MicrosDurationU32>,
}

macro_rules! adapter_timeout {
    ($($adapter:ident),+) => {$(
        impl<const CORE: u8> $adapter<'_, CORE> {
            /// Override the default timeout for trait calls on this borrow.
            /// Explicit `*_until` calls use their supplied deadline.
            pub fn with_timeout(mut self, timeout: MicrosDurationU32) -> Self {
                self.timeout = Some(timeout);
                self
            }
        }

        impl<const CORE: u8> embedded_io::ErrorType for $adapter<'_, CORE> {
            type Error = Error;
        }
    )+};
}

adapter_timeout!(Blocking, BlockingTx, BlockingRx);

impl<const PIN: u8, const CORE: u8> Rx<UART, PIN, Irq<UART, CORE>> {
    /// Borrow this half for synchronous IRQ-backed reads.
    pub fn blocking<'a>(&'a mut self, wait: &'a mut WaitTimer) -> BlockingRx<'a, CORE> {
        BlockingRx {
            rx: &self.state,
            wait,
            timeout: None,
        }
    }
}

impl<const PIN: u8, const CORE: u8> Tx<UART, PIN, Irq<UART, CORE>> {
    /// Borrow this half for synchronous IRQ-backed writes and flush.
    pub fn blocking<'a>(&'a mut self, wait: &'a mut WaitTimer) -> BlockingTx<'a, CORE> {
        BlockingTx {
            tx: &mut self.state,
            wait,
            timeout: None,
        }
    }
}

impl<const TX: u8, const RX: u8, const CORE: u8> Uart<UART, PinTuple<TX, RX>, Irq<UART, CORE>>
where
    Pin<TX>: TxPin<UART>,
    Pin<RX>: RxPin<UART>,
{
    /// Borrow the UART for synchronous IRQ-backed I/O.
    pub fn blocking<'a>(&'a mut self, wait: &'a mut WaitTimer) -> Blocking<'a, CORE> {
        Blocking {
            tx: &mut self.halves.0.state,
            rx: &self.halves.1.state,
            wait,
            timeout: None,
        }
    }
}

impl<const CORE: u8> BlockingRx<'_, CORE> {
    /// Read one byte or return a receive error. An empty slice returns zero.
    /// A timeout retires the wait. Later reads can still receive incoming bytes.
    pub fn read_until(&mut self, bytes: &mut [u8], end: Deadline) -> Result<usize, Error> {
        let Some(first) = bytes.first_mut() else {
            return Ok(0);
        };
        if self.rx.resetting {
            return Err(Error::Busy);
        }
        wait_for::<CORE>(self.wait, end, Operation::Read(first))
    }

    /// Read the whole slice with one deadline. On failure the received prefix remains in it.
    pub fn read_exact_until(&mut self, bytes: &mut [u8], end: Deadline) -> Result<(), Error> {
        for byte in bytes {
            self.read_until(core::slice::from_mut(byte), end)?;
        }
        Ok(())
    }
}

impl<const CORE: u8> BlockingTx<'_, CORE> {
    /// Wait for space then queue up to one FIFO of bytes. A timeout queues no bytes.
    pub fn write_until(&mut self, bytes: &[u8], end: Deadline) -> Result<usize, Error> {
        if bytes.is_empty() {
            return Ok(0);
        }
        if self.tx.resetting {
            return Err(Error::Busy);
        }
        wait_for::<CORE>(self.wait, end, Operation::Write(self.tx, bytes))
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
        let flags = wait_for::<CORE>(self.wait, end, Operation::TxRoom)? as u32;
        if flags & FR_BUSY == 0 {
            return Ok(());
        }
        let queued = self.tx.pending(flags, Mmio.read(base(UART) + RIS));
        let bit_us = self.tx.divisor.div_ceil(DIVISOR_PER_BIT_US);
        let now = Timer::new().now();
        let tail_us = self
            .tx
            .remaining_ticks(now.ticks() as u32)
            .min(self.tx.frame_ticks(queued));
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
                Ok(read_flags())
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

impl<const CORE: u8> Blocking<'_, CORE> {
    fn rx(&mut self) -> BlockingRx<'_, CORE> {
        BlockingRx {
            rx: self.rx,
            wait: self.wait,
            timeout: self.timeout,
        }
    }

    fn tx(&mut self) -> BlockingTx<'_, CORE> {
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

impl<const CORE: u8> embedded_io::ReadReady for BlockingRx<'_, CORE> {
    fn read_ready(&mut self) -> Result<bool, Error> {
        self.rx.ready_to_read(UART)
    }
}

impl<const CORE: u8> embedded_io::WriteReady for BlockingTx<'_, CORE> {
    fn write_ready(&mut self) -> Result<bool, Error> {
        self.tx.ready_to_write(UART)
    }
}

impl<const CORE: u8> embedded_io::ReadReady for Blocking<'_, CORE> {
    fn read_ready(&mut self) -> Result<bool, Error> {
        self.rx.ready_to_read(UART)
    }
}

impl<const CORE: u8> embedded_io::WriteReady for Blocking<'_, CORE> {
    fn write_ready(&mut self) -> Result<bool, Error> {
        self.tx.ready_to_write(UART)
    }
}

impl<const CORE: u8> embedded_io::Read for BlockingRx<'_, CORE> {
    fn read(&mut self, bytes: &mut [u8]) -> Result<usize, Error> {
        if bytes.is_empty() {
            return Ok(0);
        }
        let end = default_end(self.timeout.unwrap_or(DEFAULT_READ_TIMEOUT))?;
        self.read_until(bytes, end)
    }
}

impl<const CORE: u8> embedded_io::Write for BlockingTx<'_, CORE> {
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

impl<const CORE: u8> embedded_io::Read for Blocking<'_, CORE> {
    fn read(&mut self, bytes: &mut [u8]) -> Result<usize, Error> {
        self.rx().read(bytes)
    }
}

impl<const CORE: u8> embedded_io::Write for Blocking<'_, CORE> {
    fn write(&mut self, bytes: &[u8]) -> Result<usize, Error> {
        self.tx().write(bytes)
    }

    fn flush(&mut self) -> Result<(), Error> {
        self.tx().flush()
    }
}

macro_rules! impl_fmt_write {
    ($($adapter:ident),+) => {$(
        impl<const CORE: u8> fmt::Write for $adapter<'_, CORE> {
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
    struct Pl011 {
        mask: u32,
        asserted: u32,
        assert_on_unmask: u32,
        writes: usize,
    }

    impl Registers for Pl011 {
        fn read(&mut self, address: usize) -> u32 {
            if address == RESETS_DONE1 {
                return reset_bit(UART);
            }
            match address - base(UART) {
                IMSC => self.mask,
                MIS => self.asserted & self.mask,
                _ => panic!("unexpected register"),
            }
        }
        fn write(&mut self, address: usize, value: u32) {
            assert_eq!(address, base(UART) + IMSC);
            self.writes += 1;
            self.mask = value;
            if value != 0 {
                self.asserted |= self.assert_on_unmask;
            }
        }
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
                let mut io = Pl011::default();
                let mut state = State::default();
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
        let mut io = Pl011::default();
        let mut state = State::default();
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
            ..Pl011::default()
        };
        let mut state = State::default();
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
    fn latched_tx_watermark_caps_the_queued_tail() {
        let mut tx = TxState {
            bound_us: 5_000,
            divisor: 1736,
            earliest_end: 0,
            queued: 32,
            resetting: false,
        };
        tx.record_write(FR_BUSY, int::TX, 2, 10);
        assert_eq!(tx.queued, TX_WATERMARK + 2);
        tx.record_write(FR_BUSY, 0, 20, 20);
        assert_eq!(tx.queued, FIFO_SIZE as u8);
        tx.record_write(FR_BUSY | FR_TXFE, int::TX, 1, 2000);
        assert_eq!(tx.queued, 1);
    }

    #[test]
    fn flush_hint_decays_with_time_and_across_wrap() {
        let mut tx = TxState {
            bound_us: 5_000,
            divisor: 1736,
            earliest_end: 0,
            queued: FIFO_SIZE as u8,
            resetting: false,
        };
        let before = u32::MAX - 100;
        tx.record_write(FR_BUSY | FR_TXFF, 0, 0, before);
        tx.record_write(FR_BUSY, int::TX, 2, before.wrapping_add(16 * 87));
        assert_eq!(tx.queued, TX_WATERMARK + 2);
        let after = before.wrapping_add(2_777 + 173);
        assert_eq!(tx.earliest_end, after);
        assert_eq!(tx.remaining_ticks(after.wrapping_sub(7)), 7);
        assert_eq!(tx.remaining_ticks(after.wrapping_add(1)), 0);
    }

    #[test]
    fn foreign_mask_is_preserved() {
        let mut io = Pl011 {
            mask: int::TX,
            ..Pl011::default()
        };
        let mut state = State::default();
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
