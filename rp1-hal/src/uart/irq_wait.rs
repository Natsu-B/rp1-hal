//! UART1's shared PL011 interrupt and thread-mode adapters.

use super::*;
use crate::timer::{Instant, WaitTimer};
use rp1_rt::interrupt::{Binding, BlockIrq, Line as NvicLine, Priority, Slot};

const UART_IRQ: u8 = 42;
type Line<const CORE: u8> = Binding<BlockIrq<{ rp1_proto::BlockId::Uart1 as u16 }>, UART_IRQ, CORE>;
const UART: u8 = 1;
const CEILING: u8 = 4;
const PRIORITY: Priority = Priority::new(CEILING).unwrap();

/// Capability carried by UART1 parts when their declaration reserves the IRQ for the HAL.
#[derive(Clone, Copy)]
// Like the pins and binding, a split half can be put in its core's Slot for try I/O.
pub struct Irq<const CORE: u8>(PhantomData<core::cell::Cell<()>>);

impl<const CORE: u8, P: CompletePins<UART>> Parts<UART, P, Irq<CORE>> {
    /// # Safety
    /// Only the generated resource constructor may consume this line. It must
    /// install the matching UART1 handler on the same core.
    #[doc(hidden)]
    pub unsafe fn __new(token: Token<UART>, pins: Pins<UART, P>, line: Line<CORE>) -> Self {
        let stored =
            with_state::<CORE, _>(|state| state.line = Delivery::Unstarted(line.into_line()));
        debug_assert!(stored.is_some());
        Self {
            token,
            pins,
            irq: Irq(PhantomData),
        }
    }
}

// Ordinary DSRAM statics have one physical copy per core. Slot's core parameter
// also checks the route in the generated handler and resource constructor.
static PROC0: Slot<State, 0, CEILING> = Slot::new(State::new());
static PROC1: Slot<State, 1, CEILING> = Slot::new(State::new());

#[inline(always)]
fn with_state<const CORE: u8, T>(f: impl FnOnce(&mut State) -> T) -> Option<T> {
    const { assert!(CORE < 2) };
    if CORE == 0 {
        PROC0.try_with(f)
    } else {
        PROC1.try_with(f)
    }
}

/// # Safety
/// Called only by the generated UART1 handler on CORE.
/// Returns the serviced PL011 bits for the parity fixture.
#[doc(hidden)]
#[inline(always)]
pub unsafe fn __on_interrupt<const CORE: u8>() -> u32 {
    let pending = with_state::<CORE, _>(|state| {
        // A stale NVIC entry can arrive after the driver asserted reset.
        if !state.armed && Mmio.read(RESETS_DONE1) & reset_bit(UART) == 0 {
            return 0;
        }
        state.service(&mut Mmio)
    })
    .unwrap_or(0);
    if pending != 0 {
        rp1_rt::send_event();
    }
    pending
}

#[derive(Default)]
enum Delivery {
    #[default]
    Unavailable,
    Unstarted(NvicLine<UART_IRQ>),
    Started(NvicLine<UART_IRQ>),
}

#[derive(Default)]
struct State {
    line: Delivery,
    source: u32,
    armed: bool,
}

impl State {
    const fn new() -> Self {
        Self {
            line: Delivery::Unavailable,
            source: 0,
            armed: false,
        }
    }

    // The thread has checked admission before borrowing this slot. Only its
    // first arm may enable the NVIC line. Later removal is an error.
    fn check_line(&self) -> Result<(), Error> {
        match &self.line {
            Delivery::Unstarted(_) => Ok(()),
            Delivery::Started(line) if line.is_enabled() => Ok(()),
            _ => Err(Error::IrqUnavailable),
        }
    }

    fn start_line(&mut self) {
        if matches!(self.line, Delivery::Unstarted(_))
            && let Delivery::Unstarted(mut line) = core::mem::take(&mut self.line)
        {
            line.set_priority(PRIORITY);
            line.enable();
            self.line = Delivery::Started(line);
        }
    }

    // Return true when a post-arm probe is needed.
    fn arm(&mut self, io: &mut impl Registers, source: u32) -> Result<bool, Error> {
        if self.armed {
            return Ok(false);
        }
        if io.read(base(UART) + IMSC) != 0 {
            return Err(Error::IrqUnavailable);
        }
        self.source = source;
        self.armed = true;
        io.write(base(UART) + IMSC, source);
        Ok(true)
    }

    fn service(&mut self, io: &mut impl Registers) -> u32 {
        let asserted = io.read(base(UART) + MIS);
        let owned = if self.armed {
            asserted & self.source
        } else {
            0
        };
        // Retire the whole waiting direction on a hit. Mask other asserted
        // sources too, so an accidental raw unmask cannot hold the line high.
        let clear = (if owned != 0 { self.source } else { 0 }) | (asserted & !owned);
        if clear != 0 {
            let imsc = io.read(base(UART) + IMSC);
            io.write(base(UART) + IMSC, imsc & !clear);
        }
        if owned != 0 {
            self.armed = false;
        }
        owned
    }

    fn retire(&mut self, io: &mut impl Registers) {
        if self.armed {
            let imsc = io.read(base(UART) + IMSC);
            io.write(base(UART) + IMSC, imsc & !self.source);
            self.armed = false;
        }
        self.source = 0;
    }
}

struct Active<const CORE: u8>;

impl<const CORE: u8> Drop for Active<CORE> {
    fn drop(&mut self) {
        // Only this thread and the priority-4 handler borrow the slot. A drop
        // in the thread cannot run inside another borrow.
        let retired = with_state::<CORE, _>(|state| state.retire(&mut Mmio));
        debug_assert!(retired.is_some());
    }
}

// All pin routes and adapter shapes share the same waiter and peripheral code.
enum Operation<'a> {
    Read(&'a mut u8),
    Write(&'a mut TxState, &'a [u8]),
    TxRoom,
}

impl Operation<'_> {
    fn probe(&mut self) -> Option<Result<usize, Error>> {
        match self {
            Self::Read(first) => read_byte(&mut Mmio, UART).transpose().map(|value| {
                **first = value?;
                Ok(1)
            }),
            Self::Write(tx, bytes) => tx.enqueue(UART, bytes).map(Ok),
            Self::TxRoom => {
                let flags = read_flags();
                (flags & FR_TXFF == 0).then_some(Ok(flags as usize))
            }
        }
    }
}

fn read_flags() -> u32 {
    Mmio.read(base(UART) + FR)
}

// The timer waiter checks the deadline and probes after arming and waking.
// Register UART from inside that checked probe to keep expired calls side-effect free.
#[inline(never)]
fn wait_for<const CORE: u8>(
    wait: &mut WaitTimer,
    end: Deadline,
    mut operation: Operation<'_>,
) -> Result<usize, Error> {
    let mut context = wait.thread_context()?;
    with_state::<CORE, _>(|state| state.check_line()).ok_or(Error::Busy)??;
    let source = match operation {
        Operation::Read(_) => RX_INTERRUPTS,
        Operation::Write(..) | Operation::TxRoom => int::TX,
    };
    let mut active = None;
    let mut post_alarm_probe = true;
    context.wait_until(end, || {
        if active.is_some() && !core::mem::take(&mut post_alarm_probe) {
            let fired = with_state::<CORE, _>(|state| !state.armed).unwrap_or(false);
            if !fired && Timer::new().now() < end.instant() {
                // Unrelated events do not turn an armed UART wait into polling.
                return None;
            }
        }
        probe_and_arm(&mut operation, Operation::probe, || {
            if Timer::new().now() >= end.instant() {
                return Ok(false);
            }
            let armed = with_state::<CORE, _>(|state| {
                let armed = state.arm(&mut Mmio, source)?;
                if armed {
                    state.start_line();
                }
                Ok::<_, Error>(armed)
            })
            .ok_or(Error::Busy)??;
            if armed && active.is_none() {
                active = Some(Active::<CORE>);
            }
            Ok(armed)
        })
    })?
}

// Pass the probe's input explicitly so the waiter can borrow it on both probes.
fn probe_and_arm<T, S>(
    state: &mut S,
    mut probe: impl FnMut(&mut S) -> Option<Result<T, Error>>,
    arm: impl FnOnce() -> Result<bool, Error>,
) -> Option<Result<T, Error>> {
    if let Some(ready) = probe(state) {
        return Some(ready);
    }
    match arm() {
        Ok(true) => probe(state),
        Ok(false) => None,
        Err(error) => Some(Err(error)),
    }
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

impl<const PIN: u8, const CORE: u8> Rx<UART, PIN, Irq<CORE>> {
    /// Borrow this half for synchronous IRQ-backed reads.
    pub fn blocking<'a>(&'a mut self, wait: &'a mut WaitTimer) -> BlockingRx<'a, CORE> {
        BlockingRx {
            rx: &self.state,
            wait,
            timeout: None,
        }
    }
}

impl<const PIN: u8, const CORE: u8> Tx<UART, PIN, Irq<CORE>> {
    /// Borrow this half for synchronous IRQ-backed writes and flush.
    pub fn blocking<'a>(&'a mut self, wait: &'a mut WaitTimer) -> BlockingTx<'a, CORE> {
        BlockingTx {
            tx: &mut self.state,
            wait,
            timeout: None,
        }
    }
}

impl<const TX: u8, const RX: u8, const CORE: u8> Uart<UART, PinTuple<TX, RX>, Irq<CORE>>
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
    fn data_on_either_side_of_arm_is_consumed_once() {
        for phase in 0..3 {
            let ready = Cell::new(phase == 0);
            let arms = Cell::new(0);
            let read = |_: &mut ()| ready.replace(false).then_some(Ok(42));
            let arm = || {
                arms.set(arms.get() + 1);
                if phase == 1 {
                    ready.set(true);
                }
                Ok(true)
            };
            let first = probe_and_arm(&mut (), read, arm);
            if phase == 2 {
                assert_eq!(first, None);
                ready.set(true);
                assert_eq!(probe_and_arm(&mut (), read, arm), Some(Ok(42)));
            } else {
                assert_eq!(first, Some(Ok(42)));
            }
            assert_eq!(arms.get(), u8::from(phase != 0));
            assert!(!ready.get());
        }
    }

    #[test]
    fn deadline_probe_preserves_completion_without_starting_a_new_wait() {
        // A UART IRQ was never delivered. The deadline alarm gets one final
        // observation; it can see retained data or finish with Timeout.
        assert_eq!(
            probe_and_arm(&mut (), |_| Some(Ok(42)), || panic!("rearmed at deadline")),
            Some(Ok(42))
        );
        assert_eq!(
            probe_and_arm(&mut (), |_| None::<Result<u8, Error>>, || Ok(false)),
            None
        );
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
        assert!(state.armed);

        io.mask |= int::TX;
        io.asserted |= int::RX;
        assert_eq!(state.service(&mut io), int::RX);
        assert_eq!(io.mask, 0);
        assert!(!state.armed);
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
