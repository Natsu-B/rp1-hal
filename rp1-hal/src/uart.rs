//! PL011 UARTs 0..4 on their bank-0 pins (DOC: RP1 peripherals §3.5). The baud divisors come
//! from clk_uart as reported by [`Clocks`]. UART5 has no bank-0 pins. The `log!` sink (UART0 on
//! GPIO14/15) keeps its dedicated bounded boot path through [`crate::logging::Logger`].
//!
//! STATIC: declare a UART and `wait_timer` on the same core for IRQ/WFE
//! waits through [`Uart::blocking`]. Use [`Uart::polling`] for explicit polling
//! and `try_*` for immediate I/O. The wait timer serializes the split halves.
//! UART0 is available to the driver when the declaration leaves out the log sink.
//!
//! | UART | Hardware evidence |
//! |---|---|
//! | UART0 | HW, PL011 loopback with RX, RT and TX IRQs and SHM reporting |
//! | UART1 | HW, IRQ waits and exact data with an FT230X peer |
//! | UART2 | HW, PL011 loopback with RX, RT and TX IRQs, deadline and re-arm |
//! | UART3 | HW, PL011 loopback with RX, RT and TX IRQs, deadline and re-arm |
//! | UART4 | HW, PL011 loopback with RX, RT and TX IRQs, deadline and re-arm |
use crate::{
    clocks::Clocks,
    gpio::{Alternate, Pin},
    linux_blocks::{Hardware, Mmio},
    pac::pl011::{RegisterBlock, int},
    scmi::Registers,
    timer::{self, Deadline, Timer},
};
use core::{fmt, marker::PhantomData};
use fugit::{HertzU32, MicrosDurationU32};
use rp1_rt::interrupt::{ThreadGuard, free};

mod irq_wait;
mod traits;
pub use irq_wait::{__on_interrupt, Blocking, BlockingRx, BlockingTx, Irq};

macro_rules! offsets {
    ($($name:ident = $field:ident),* $(,)?) => {$(const $name: usize = core::mem::offset_of!(RegisterBlock, $field);)*};
}
offsets! {
    DR = dr, ECR = rsr_ecr, FR = fr, IBRD = ibrd, FBRD = fbrd, LCR_H = lcr_h, CR = cr,
    IFLS = ifls, IMSC = imsc, RIS = ris, MIS = mis, ICR = icr,
}
/// DR bits 8..=11: the receive status of the character read with them.
const DR_FE: u32 = 1 << 8;
const DR_PE: u32 = 1 << 9;
const DR_BE: u32 = 1 << 10;
const DR_OE: u32 = 1 << 11;
const FR_BUSY: u32 = 1 << 3;
const FR_RXFE: u32 = 1 << 4;
const FR_TXFF: u32 = 1 << 5;
const FR_TXFE: u32 = 1 << 7;
const FIFO_SIZE: usize = 32;
/// RX and TX watermarks at half of their 32-byte FIFOs.
const IFLS_HALF: u32 = (2 << 3) | 2;
const TX_WATERMARK: u8 = 16;
// Clocks admits clk_uart at the XOSC rate. The PL011 bit period is
// 16 * divisor / clk_uart; the stored divisor is scaled by 64.
const DIVISOR_PER_BIT_US: u32 = crate::clocks::XOSC_HZ / 250_000;
/// 8 data bits, FIFOs enabled, no parity, one stop bit.
const LCR_H_8N1_FIFO: u32 = 0x70;
/// UARTEN, TXE, RXE.
const CR_ENABLE: u32 = 0x301;
const CR_LBE: u32 = 1 << 7;
/// RESETS CTRL1 through its set and clear aliases: one bit per UART, so asserting or releasing
/// this UART's reset leaves the sibling UARTs (possibly Linux's) and every other block alone.
const RESETS: usize = 0x4001_4000;
const RESETS_SET1: usize = RESETS + 0x2000 + 4;
const RESETS_CLR1: usize = RESETS + 0x3000 + 4;
const RESETS_DONE1: usize = RESETS + 0x1c;
const fn reset_bit(n: u8) -> u32 {
    1 << (26 + n)
}
/// Bound of the two waits of the bring-up (reset done, transmitter idle).
const START_TIMEOUT_US: u32 = 10_000;
/// RX wait used by [`Polling`] until changed.
pub const DEFAULT_READ_TIMEOUT: MicrosDurationU32 = MicrosDurationU32::millis(100);
/// Bit times for a full TX FIFO to leave the line: 32 queued characters and the one being
/// shifted, 10 bits each at 8N1.
const FULL_FIFO_BITS: u32 = 330;
const RX_ERRORS: u32 = int::FE | int::PE | int::BE | int::OE;
const RX_INTERRUPTS: u32 = int::RX | int::RT | RX_ERRORS;

/// Bound of the TX waits (room in the FIFO, drain) at `baud`. Without flow control the
/// transmitter drains at the baud rate whatever the peer does, so the bound follows from it:
/// twice a full FIFO, plus 1 ms for a core stalled by the other core or a handler. A bound
/// set by the caller for reads would cut `free_polling` short and truncate transmission on reset.
fn tx_bound_us(baud: HertzU32) -> u32 {
    let full_fifo_us = (FULL_FIFO_BITS * 1_000_000).div_ceil(baud.raw());
    2 * full_fifo_us + 1_000
}

mod sealed {
    pub trait CompletePins<const N: u8> {}
    pub trait Pin {}
    impl<const N: u8> Pin for crate::gpio::Pin<N> {}
}
/// TX route of UART N; `FUNC` is the GPIO function select (`crate::pinmux`).
pub trait TxPin<const N: u8>: sealed::Pin {
    const FUNC: u8;
}
/// RX route of UART N.
pub trait RxPin<const N: u8>: sealed::Pin {
    const FUNC: u8;
}
/// Pad while the mux switches (output disabled) and for RX: input enable, pull-up (line idle),
/// Schmitt. TX: input enable, 8 mA, Schmitt.
const IDLE_PAD: u32 = 0x4a;
const TX_PAD: u32 = 0x62;

const fn base(n: u8) -> usize {
    0x4003_0000 + n as usize * 0x4000
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    /// The RX FIFO was full and one or more received characters before this one were lost.
    /// The character the overrun was reported with is discarded too.
    Overrun,
    /// The line was held low for longer than a character.
    Break,
    /// The parity bit did not match (never with the 8N1 this driver configures).
    Parity,
    /// No valid stop bit.
    Framing,
    /// The reset, the transmitter or the peer kept the driver waiting too long.
    Timeout,
    /// The baud rate has no divisor at clk_uart.
    Invalid,
    /// The clock tree is not the one the [`Clocks`] value describes.
    Precondition,
    /// The transmitter has not drained, or reset has not been confirmed.
    Busy,
    /// The absolute deadline is beyond the maximum comparator horizon.
    InvalidDeadline,
    /// A synchronous operation was called outside thread mode.
    WrongContext,
    /// Interrupts were masked at the synchronous call boundary.
    Masked,
    /// The UART line is unavailable, or raw UART masks are enabled when a managed wait arms.
    IrqUnavailable,
    /// The dedicated alarm could not be retired.
    AlarmUnavailable,
}

impl From<timer::Error> for Error {
    fn from(error: timer::Error) -> Self {
        match error {
            timer::Error::Timeout => Self::Timeout,
            timer::Error::InvalidDeadline => Self::InvalidDeadline,
            timer::Error::WrongContext => Self::WrongContext,
            timer::Error::Masked => Self::Masked,
            timer::Error::IrqUnavailable => Self::IrqUnavailable,
            timer::Error::AlarmUnavailable => Self::AlarmUnavailable,
        }
    }
}

impl From<rp1_rt::interrupt::ContextError> for Error {
    fn from(error: rp1_rt::interrupt::ContextError) -> Self {
        timer::Error::from(error).into()
    }
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl core::error::Error for Error {}

/// An overrun is invalid data. The data read before it is good, but one or more
/// characters between that data and the next one read are missing.
impl embedded_io::Error for Error {
    fn kind(&self) -> embedded_io::ErrorKind {
        use embedded_io::ErrorKind;
        match self {
            Self::Break | Self::Parity | Self::Framing | Self::Overrun => ErrorKind::InvalidData,
            Self::Timeout => ErrorKind::TimedOut,
            Self::Invalid | Self::InvalidDeadline => ErrorKind::InvalidInput,
            _ => ErrorKind::Other,
        }
    }
}
/// The receive status in a DR value. A break also fails the stop-bit check, so it is looked
/// at before framing; overrun first, as the one that means lost data.
fn receive_error(data: u32) -> Option<Error> {
    [
        (DR_OE, Error::Overrun),
        (DR_BE, Error::Break),
        (DR_PE, Error::Parity),
        (DR_FE, Error::Framing),
    ]
    .into_iter()
    .find_map(|(bit, error)| (data & bit != 0).then_some(error))
}

pub struct Token<const N: u8> {
    _private: core::marker::PhantomData<core::cell::Cell<()>>,
}
/// A raw UART without a generated wait IRQ.
#[derive(Clone, Copy)]
pub struct Serial {
    _private: (),
}

/// A configured UART that owns its TX and RX pins.
/// Use [`Uart::blocking`] for UART1 synchronous I/O, or [`Uart::polling`] for explicit polling.
/// Immediate byte operations are available on all instances.
///
/// ```compile_fail,E0277
/// use rp1_hal::{gpio::Pin, uart::Uart};
/// fn raw_is_not_a_stream(raw: &mut Uart<1, (Pin<0>, Pin<1>)>) {
///     let _ = embedded_io::Write::write_all(raw, b"x");
/// }
/// ```
pub struct Uart<const N: u8, P: CompletePins<N>, I = Serial> {
    halves: P::Active<I>,
}

/// The initialization error retains the block and its pins during recovery.
pub type InitError<const N: u8, P, I = Serial> =
    crate::InitError<Parts<N, P, I>, Error, crate::Recovery<Parts<N, P, I>>>;

/// Explicit polling access for a UART or either half.
/// Synchronous operations require unmasked thread mode. In an ISR use the
/// driver's bounded `try_*` methods.
pub struct Polling<'a, T>(&'a mut T);

impl<T> Polling<'_, T> {
    fn driver(&self) -> &T {
        self.0
    }
}

/// 8N1 UART configuration with FIFOs enabled.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Config {
    baud: HertzU32,
}
impl Config {
    /// Select a baud rate for 8N1 with FIFOs enabled.
    pub const fn new(baud: HertzU32) -> Self {
        Self { baud }
    }
}

impl From<HertzU32> for Config {
    fn from(baud: HertzU32) -> Self {
        Self::new(baud)
    }
}

/// The transmitter and routed TX pin of UART N.
///
/// Can be held in an [`rp1_rt::interrupt::Slot`] to share with the owning core's handler.
/// Both halves stay on the owning core.
pub struct Tx<const N: u8, const PIN: u8, I = Serial> {
    pin: Pin<PIN, Alternate>,
    irq: I,
    state: TxState,
    #[cfg(test)]
    cts: Option<crate::gpio::ErasedPin>,
}

/// The receiver and routed RX pin of UART N.
///
/// Can be held in an [`rp1_rt::interrupt::Slot`] to share with the owning core's handler.
/// Both halves stay on the owning core.
pub struct Rx<const N: u8, const PIN: u8, I = Serial> {
    pin: Pin<PIN, Alternate>,
    _irq: I,
    state: RxState,
}

struct TxState {
    bound_us: u32,
    divisor: u32,
    earliest_end: u32,
    queued: u8,
    resetting: bool,
}

struct RxState {
    timeout: MicrosDurationU32,
    resetting: bool,
}
impl<const N: u8> Token<N> {
    /// # Safety
    /// Generated resources only: sole owner of UART N, with Linux's node disabled.
    ///
    /// UART5 has no bank-0 pins and is a compile-time error:
    ///
    /// ```compile_fail,E0080
    /// let _ = unsafe { rp1_hal::uart::Token::<5>::new() };
    /// ```
    pub const unsafe fn new() -> Self {
        const { assert!(N < 5) };
        Self {
            _private: core::marker::PhantomData,
        }
    }
}

pin_builder!(Pin, []);
pin_builder!(Pin, [], tx: TX => TxPin);
pin_builder!(Pin, [tx: TX], rx: RX => RxPin);
/// The required pin routes for this peripheral.
pub trait CompletePins<const N: u8>: sealed::CompletePins<N> {
    #[doc(hidden)]
    type Active<I>;
}

impl<const N: u8, const TX: u8, const RX: u8> sealed::CompletePins<N> for PinTuple<TX, RX>
where
    Pin<TX>: TxPin<N>,
    Pin<RX>: RxPin<N>,
{
}

impl<const N: u8, const TX: u8, const RX: u8> CompletePins<N> for PinTuple<TX, RX>
where
    Pin<TX>: TxPin<N>,
    Pin<RX>: RxPin<N>,
{
    type Active<I> = (Tx<N, TX, I>, Rx<N, RX, I>);
}

type PinTuple<const TX: u8, const RX: u8> = (Pin<TX>, Pin<RX>);

/// A declared UART N with its bank-0 TX and RX routes.
pub struct Parts<const N: u8, P: CompletePins<N>, I = Serial> {
    token: Token<N>,
    pins: Pins<N, P>,
    irq: I,
}
impl<const N: u8, P: CompletePins<N>> Parts<N, P> {
    /// Assemble the owned block and the routed pin bundle.
    ///
    /// ```no_run
    /// use rp1_hal::{clocks::Clocks, fugit::RateExtU32, gpio::Pin, uart};
    ///
    /// fn setup(token: uart::Token<2>, tx: Pin<4>, rx: Pin<5>, clocks: &Clocks)
    ///     -> Result<(), uart::InitError<2, (Pin<4>, Pin<5>)>>
    /// {
    ///     let pins = uart::Pins::<2>::builder().tx(tx).rx(rx);
    ///     let uart = uart::Parts::new(token, pins).init(clocks, 115_200.Hz())?;
    ///     let (_tx, _rx) = uart.split();
    ///     Ok(())
    /// }
    /// ```
    pub fn new(token: Token<N>, pins: Pins<N, P>) -> Self {
        Self {
            token,
            pins,
            irq: Serial { _private: () },
        }
    }

    /// Return the block and routed pins for manual assembly.
    pub fn into_parts(self) -> (Token<N>, Pins<N, P>) {
        (self.token, self.pins)
    }
}
impl<const N: u8, const TX: u8, const RX: u8, I: Copy> Parts<N, PinTuple<TX, RX>, I>
where
    Pin<TX>: TxPin<N>,
    Pin<RX>: RxPin<N>,
{
    /// Check clk_uart and the baud rate, release only this UART's reset and configure 8N1.
    /// The pins are muxed only after setup succeeds.
    ///
    /// ```no_run
    /// # use rp1_hal::{clocks::Clocks, fugit::RateExtU32, gpio::Pin, uart::Parts};
    /// fn hello(uart2: Parts<2, (Pin<4>, Pin<5>)>, clocks: &Clocks) {
    ///     let mut uart = uart2.init(clocks, 115_200.Hz()).unwrap();
    ///     embedded_io::Write::write_all(&mut uart.polling(), b"hello").unwrap();
    /// }
    /// ```
    ///
    /// Retry with another baud rate. If reset has not been confirmed, the
    /// initialization error still owns the parts for a later recovery attempt.
    ///
    /// ```no_run
    /// # use rp1_hal::{clocks::Clocks, fugit::RateExtU32, gpio::Pin, uart};
    /// # type Pins = (Pin<4>, Pin<5>);
    /// fn start(parts: uart::Parts<2, Pins>, clocks: &Clocks)
    ///     -> Result<uart::Uart<2, Pins>, uart::InitError<2, Pins>>
    /// {
    ///     match parts.init(clocks, 1_000_000.Hz()) {
    ///         Ok(uart) => Ok(uart),
    ///         Err(error) => {
    ///             let parts = match error.try_into_parts() {
    ///                 Ok(parts) => parts,
    ///                 Err(recovery) => recovery.try_recover()?,
    ///             };
    ///             parts.init(clocks, 115_200.Hz())
    ///         }
    ///     }
    /// }
    /// ```
    pub fn init(
        self,
        clocks: &Clocks,
        config: impl Into<Config>,
    ) -> Result<Uart<N, PinTuple<TX, RX>, I>, InitError<N, PinTuple<TX, RX>, I>> {
        self.init_with_route(clocks, config.into(), false)
    }

    /// Initialize PL011 internal loopback with both pads disconnected.
    /// Free and reinitialize the UART to connect it to the pads.
    pub fn init_loopback(
        self,
        clocks: &Clocks,
        config: impl Into<Config>,
    ) -> Result<Uart<N, PinTuple<TX, RX>, I>, InitError<N, PinTuple<TX, RX>, I>> {
        self.init_with_route(clocks, config.into(), true)
    }

    fn init_with_route(
        self,
        clocks: &Clocks,
        config: Config,
        loopback: bool,
    ) -> Result<Uart<N, PinTuple<TX, RX>, I>, InitError<N, PinTuple<TX, RX>, I>> {
        if !clocks.still_current() {
            return Err(InitError::new(Error::Precondition, self));
        }
        let uart = self.init_on(&mut Mmio, clocks.uart(), config, |tx, rx| {
            if loopback {
                const DISCONNECTED: u8 = 31;
                (
                    tx.into_alternate(DISCONNECTED, 0, 0x80),
                    rx.into_alternate(DISCONNECTED, 0, 0x80),
                )
            } else {
                (
                    tx.into_alternate(<Pin<TX> as TxPin<N>>::FUNC, IDLE_PAD, TX_PAD),
                    rx.into_alternate(<Pin<RX> as RxPin<N>>::FUNC, IDLE_PAD, IDLE_PAD),
                )
            }
        })?;
        if loopback {
            Mmio.write(base(N) + CR, CR_ENABLE | CR_LBE);
        }
        Ok(uart)
    }

    fn init_on(
        self,
        io: &mut impl Hardware,
        clock: HertzU32,
        config: Config,
        route: impl FnOnce(Pin<TX>, Pin<RX>) -> (Pin<TX, Alternate>, Pin<RX, Alternate>),
    ) -> Result<Uart<N, PinTuple<TX, RX>, I>, InitError<N, PinTuple<TX, RX>, I>> {
        debug_assert!(N != 1 || clock.raw() == crate::clocks::XOSC_HZ);
        match start::<true>(io, N, clock, config.baud) {
            Ok((integer, fraction)) => {
                let (tx, rx) = self.pins.0;
                let (tx, rx) = route(tx, rx);
                Ok(Uart::join(
                    Tx {
                        pin: tx,
                        irq: self.irq,
                        state: TxState {
                            bound_us: tx_bound_us(config.baud),
                            divisor: 64 * integer + fraction,
                            earliest_end: 0,
                            queued: 0,
                            resetting: false,
                        },
                        #[cfg(test)]
                        cts: self.pins.1,
                    },
                    Rx {
                        pin: rx,
                        _irq: self.irq,
                        state: RxState {
                            timeout: DEFAULT_READ_TIMEOUT,
                            resetting: false,
                        },
                    },
                ))
            }
            Err(Error::Timeout) => Err(InitError::recovering(Error::Timeout, self)),
            Err(error) => Err(InitError::new(error, self)),
        }
    }
}

impl<const N: u8, P: CompletePins<N>, I> InitError<N, P, I> {
    /// Request UART reset and return the parts once reset is confirmed.
    pub fn try_recover(self) -> Result<Parts<N, P, I>, Self> {
        self.recover(|| try_reset(&mut Mmio, N))
    }
}

/// Releases UART `n`'s reset and configures it for 8N1 at `baud`. Nothing is written unless
/// the divisors are valid. LCR_H is only legal to change with no character in progress. Out of
/// reset (cold, or after [`Uart::try_free`] / [`Uart::try_cancel`]) UARTEN has been clear since, so none
/// can be; a UART an earlier image left running is only waited for until its transmitter is idle.
pub(crate) fn start<const SET_WATERMARKS: bool>(
    io: &mut impl Hardware,
    n: u8,
    clock: HertzU32,
    baud: HertzU32,
) -> Result<(u32, u32), Error> {
    let (integer, fraction) = divisors(clock, baud)?;
    io.write(RESETS_CLR1, reset_bit(n));
    wait(
        io,
        RESETS_DONE1,
        reset_bit(n),
        reset_bit(n),
        START_TIMEOUT_US,
    )?;
    let base = base(n);
    wait(io, base + FR, FR_BUSY, 0, START_TIMEOUT_US)?;
    io.write(base + CR, 0);
    io.write(base + IMSC, 0);
    io.write(base + ICR, 0x7ff);
    io.write(base + ECR, 0);
    if SET_WATERMARKS {
        // Reset value, written so the wait does not depend on the previous configuration.
        io.write(base + IFLS, IFLS_HALF);
    }
    io.write(base + IBRD, integer);
    io.write(base + FBRD, fraction);
    io.write(base + LCR_H, LCR_H_8N1_FIFO);
    io.write(base + CR, CR_ENABLE);
    Ok((integer, fraction))
}
/// Waits for `address & mask == value` until more than `timeout_us` of the 1 MHz timer have
/// passed, however many polls that takes.
fn wait(
    io: &mut impl Hardware,
    address: usize,
    mask: u32,
    value: u32,
    timeout_us: u32,
) -> Result<(), Error> {
    // The timer wraps every 2^32 us: summing the steps between polls in 64 bits lets every
    // `timeout_us` expire.
    let mut last = io.now_us();
    let mut elapsed_us = 0u64;
    loop {
        if io.read(address) & mask == value {
            return Ok(());
        }
        let now = io.now_us();
        elapsed_us += u64::from(now.wrapping_sub(last));
        last = now;
        if elapsed_us > u64::from(timeout_us) {
            // One more look: the deadline may have passed while this core was stalled.
            return if io.read(address) & mask == value {
                Ok(())
            } else {
                Err(Error::Timeout)
            };
        }
    }
}
/// Service at most one FIFO of TX data even if the peripheral drains during the call.
fn enqueue(io: &mut impl Registers, n: u8, bytes: &[u8]) -> usize {
    let mut count = 0;
    for &byte in bytes.iter().take(FIFO_SIZE) {
        if io.read(base(n) + FR) & FR_TXFF != 0 {
            break;
        }
        io.write(base(n) + DR, u32::from(byte));
        count += 1;
    }
    count
}

/// Pops the RX FIFO, which must not be empty.
fn take(io: &mut impl Registers, n: u8) -> Result<u8, Error> {
    let data = io.read(base(n) + DR);
    match receive_error(data) {
        Some(error) => {
            io.write(base(n) + ICR, (data >> 1) & RX_ERRORS);
            io.write(base(n) + ECR, 0);
            Err(error)
        }
        None => Ok(data as u8),
    }
}
fn read_byte(io: &mut impl Registers, n: u8) -> Result<Option<u8>, Error> {
    if io.read(base(n) + FR) & FR_RXFE == 0 {
        return take(io, n).map(Some);
    }

    // OE can be asserted without being copied to a FIFO entry if the peer stops.
    // Read RIS before rechecking FIFO state so a newly arrived DR entry keeps its error bits.
    let errors = io.read(base(n) + RIS) & RX_ERRORS;
    if errors == 0 {
        return Ok(None);
    }
    if io.read(base(n) + FR) & FR_RXFE == 0 {
        return take(io, n).map(Some);
    }
    io.write(base(n) + ICR, errors);
    io.write(base(n) + ECR, 0);
    receive_error(errors << 1).map_or(Ok(None), Err)
}

/// Polling release: nothing is written unless the transmitter is idle in time.
fn release(io: &mut impl Hardware, n: u8, timeout_us: u32) -> Result<(), Error> {
    wait(io, base(n) + FR, FR_BUSY, 0, timeout_us)?;
    Ok(())
}
/// Asserts only this UART's reset. The PL011 forbids changing LCR_H (and with it FEN, the
/// only register way to empty the FIFOs) while a character is still being sent or received,
/// even after UARTEN is cleared (DOC: ARM DDI0183, UARTCR and UARTLCR_H), and BUSY says
/// nothing about reception. The reset ends any character at once and returns every register
/// to its reset value: FIFOs empty, interrupts masked (IMSC = 0), UART disabled (DOC: ARM
/// DDI0183, register reset values). [`start`] releases it again.
fn hold_in_reset(io: &mut impl Hardware, n: u8) {
    io.write(RESETS_SET1, reset_bit(n));
}
fn try_reset(io: &mut impl Hardware, n: u8) -> bool {
    hold_in_reset(io, n);
    io.read(RESETS_DONE1) & reset_bit(n) == 0
}

fn mask_interrupts(n: u8, bits: u32, on: bool) {
    // PL011 has one IMSC register shared by TX and RX, with no set/clear aliases.
    free(|_| set_mask(&mut Mmio, n, bits, on));
}

fn set_mask(io: &mut impl Hardware, n: u8, bits: u32, on: bool) {
    let imsc = io.read(base(n) + IMSC);
    io.write(base(n) + IMSC, if on { imsc | bits } else { imsc & !bits });
}
/// Returns the integer and fractional PL011 divisors for the requested baud rate.
pub fn divisors(clock: HertzU32, baud: HertzU32) -> Result<(u32, u32), Error> {
    let baud = u64::from(baud.raw());
    if baud == 0 {
        return Err(Error::Invalid);
    }
    let scaled = (u64::from(clock.raw()) * 4 + baud / 2) / baud;
    if !(64..=(65535 * 64 + 63)).contains(&scaled) {
        return Err(Error::Invalid);
    }
    Ok(((scaled / 64) as u32, (scaled % 64) as u32))
}
impl TxState {
    // Bound the FIFO tail at the next flush. The latched TX watermark stays
    // asserted until the FIFO is refilled above half. The shifter is separate.
    fn pending(&self, flags: u32, ris: u32) -> u8 {
        if flags & FR_TXFE != 0 {
            0
        } else if ris & int::TX != 0 {
            self.queued.min(TX_WATERMARK)
        } else {
            self.queued
        }
    }

    fn frame_ticks(&self, count: u8) -> u32 {
        // Round down only after multiplying the frames. This is a lower
        // bound on the time it takes to transmit those complete frames.
        u32::from(count) * 10 * self.divisor / DIVISOR_PER_BIT_US
    }

    fn remaining_ticks(&self, now: u32) -> u32 {
        (self.earliest_end.wrapping_sub(now) as i32).max(0) as u32
    }

    fn record_write(&mut self, flags: u32, ris: u32, count: usize, now: u32) {
        let previous = self.pending(flags, ris);
        self.queued = (previous + count as u8).min(FIFO_SIZE as u8);

        // Bound the earliest possible end as well as the FIFO depth. A depth
        // seen on the last write can be stale by the time flush is called.
        let remaining = if flags & FR_TXFF != 0 {
            self.frame_ticks(FIFO_SIZE as u8)
        } else if flags & FR_TXFE != 0 {
            0
        } else {
            self.remaining_ticks(now)
        };
        self.earliest_end = now
            .wrapping_add(remaining)
            .wrapping_add(self.frame_ticks(count as u8));
    }

    fn enqueue(&mut self, n: u8, bytes: &[u8]) -> Option<usize> {
        // Timestamp before FR so a full FIFO guarantees 32 complete frames
        // after that tick. Observe RIS before any refill clears its TX bit.
        let sample = (n == 1).then(|| {
            let now = Timer::new().now_32().ticks();
            let flags = Mmio.read(base(n) + FR);
            (now, flags, Mmio.read(base(n) + RIS))
        });
        let count = enqueue(&mut Mmio, n, bytes);
        if let Some((now, flags, ris)) = sample {
            self.record_write(flags, ris, count, now);
        }
        (count != 0).then_some(count)
    }
}

impl<const N: u8, const PIN: u8, I> Tx<N, PIN, I> {
    /// Selects the synchronous polling engine.
    pub fn polling(&mut self) -> Polling<'_, Self> {
        Polling(self)
    }

    /// Queues one byte if the FIFO has room. Returns [`Error::Busy`] after a stop request.
    pub fn try_write_byte(&mut self, byte: u8) -> Result<bool, Error> {
        if self.state.resetting {
            return Err(Error::Busy);
        }
        let flags = Mmio.read(base(N) + FR);
        if flags & FR_TXFF != 0 {
            return Ok(false);
        }
        if N == 1 {
            let now = Timer::new().now_32().ticks();
            self.state
                .record_write(flags, Mmio.read(base(N) + RIS), 1, now);
        }
        Mmio.write(base(N) + DR, u32::from(byte));
        Ok(true)
    }

    /// Whether all queued bytes have left the wire. Returns [`Error::Busy`] after a stop request.
    pub fn try_flush(&mut self) -> Result<bool, Error> {
        if self.state.resetting {
            return Err(Error::Busy);
        }
        Ok(Mmio.read(base(N) + FR) & FR_BUSY == 0)
    }

    /// Clears the TX interrupt bit selected by `mask`.
    pub fn clear_interrupt(&mut self, mask: u32) {
        if !self.state.resetting {
            clear_interrupts(&mut Mmio, N, int::TX, mask);
        }
    }
}

impl<const N: u8, const PIN: u8> Tx<N, PIN, Serial> {
    /// Unmasks the TX FIFO threshold interrupt. The NVIC line is `irq.UARTn`.
    pub fn enable_interrupt(&mut self) {
        if !self.state.resetting {
            mask_interrupts(N, int::TX, true);
        }
    }

    /// Masks the TX interrupt.
    pub fn disable_interrupt(&mut self) {
        if !self.state.resetting {
            mask_interrupts(N, int::TX, false);
        }
    }

    /// Pending unmasked TX interrupt, as a [PL011 interrupt bit](int).
    pub fn interrupt_status(&self) -> u32 {
        if self.state.resetting {
            0
        } else {
            pending_interrupts(&mut Mmio, N, int::TX)
        }
    }
}

impl<const N: u8, const PIN: u8, I> Rx<N, PIN, I> {
    /// Selects the synchronous polling engine.
    pub fn polling(&mut self) -> Polling<'_, Self> {
        Polling(self)
    }

    /// Pops one character or returns `None` if the FIFO is empty. A character received
    /// with an error is discarded and the error is returned.
    pub fn try_read_byte(&mut self) -> Result<Option<u8>, Error> {
        if self.state.resetting {
            return Err(Error::Busy);
        }
        read_byte(&mut Mmio, N)
    }

    /// Clears the receive interrupt bits selected by `mask`.
    /// Receive error interrupts clear only through the PL011 interrupt-clear register
    /// (UARTICR, DOC: ARM DDI0183 §2.8.5). Draining the FIFO also clears RX and RT.
    pub fn clear_interrupt(&mut self, mask: u32) {
        if !self.state.resetting {
            clear_interrupts(&mut Mmio, N, RX_INTERRUPTS, mask);
        }
    }
}

impl<const N: u8, const PIN: u8> Rx<N, PIN, Serial> {
    /// Unmasks the RX FIFO threshold, receive timeout and receive error interrupts.
    /// Receive timeout lets a short message below the threshold raise an interrupt.
    /// Use [`Rx::clear_interrupt`] to acknowledge receive errors. The NVIC line is `irq.UARTn`.
    pub fn enable_interrupt(&mut self) {
        if !self.state.resetting {
            mask_interrupts(N, RX_INTERRUPTS, true);
        }
    }

    /// Masks the RX FIFO threshold, receive timeout and receive error interrupts.
    pub fn disable_interrupt(&mut self) {
        if !self.state.resetting {
            mask_interrupts(N, RX_INTERRUPTS, false);
        }
    }

    /// Pending unmasked receive interrupts, as [PL011 interrupt bits](int).
    pub fn interrupt_status(&self) -> u32 {
        if self.state.resetting {
            0
        } else {
            pending_interrupts(&mut Mmio, N, RX_INTERRUPTS)
        }
    }
}

fn pending_interrupts(io: &mut impl Registers, n: u8, direction: u32) -> u32 {
    io.read(base(n) + MIS) & direction
}

fn clear_interrupts(io: &mut impl Registers, n: u8, direction: u32, mask: u32) {
    io.write(base(n) + ICR, direction & mask);
}

impl<const N: u8, const TX: u8, const RX: u8, I> Uart<N, PinTuple<TX, RX>, I>
where
    Pin<TX>: TxPin<N>,
    Pin<RX>: RxPin<N>,
{
    /// Splits the UART into independently owned halves. Dropping a half leaves the UART running.
    ///
    /// For a custom handler, put the receive half in a slot for its owning core
    /// before enabling the NVIC line.
    ///
    /// ```no_run
    /// use rp1_hal::{gpio::Pin, pac::pl011::int};
    /// use rp1_hal::uart::{Rx, Tx, Uart};
    /// use rp1_rt::interrupt::Slot;
    ///
    /// static RX: Slot<Option<Rx<2, 5>>, 0> = Slot::empty();
    /// # mod rp1 {
    /// #     pub mod __irq_proc0 {
    /// #         pub trait Handler {}
    /// #         pub trait Declared {}
    /// #         pub enum UART2 {}
    /// #         impl Declared for UART2 {}
    /// #     }
    /// # }
    ///
    /// fn install_rx(uart: Uart<2, (Pin<4>, Pin<5>)>) -> Tx<2, 4> {
    ///     let (tx, mut rx) = uart.split();
    ///     rx.enable_interrupt();
    ///     RX.put(rx).ok().unwrap();
    ///     tx
    /// }
    ///
    /// #[rp1_rt::interrupt(proc0)]
    /// fn UART2() {
    ///     RX.with(|rx| {
    ///         let errors = rx.interrupt_status() & (int::FE | int::PE | int::BE | int::OE);
    ///         rx.clear_interrupt(errors);
    ///         // Drain the FIFO with rx.try_read_byte(), handling receive errors.
    ///         // Stop after a bounded number of bytes so the ISR stays short.
    ///     });
    /// }
    /// # fn main() {}
    /// ```
    pub fn split(self) -> (Tx<N, TX, I>, Rx<N, RX, I>) {
        self.halves
    }

    /// Reunites the halves of UART N without changing its configuration.
    pub fn join(tx: Tx<N, TX, I>, rx: Rx<N, RX, I>) -> Self {
        Self { halves: (tx, rx) }
    }

    /// Selects the synchronous polling engine.
    pub fn polling(&mut self) -> Polling<'_, Self> {
        Polling(self)
    }

    /// Feed transmitted data back into RX. Change this while RX is quiescent
    /// and the transmitter is idle. The GPIO routing stays as configured.
    pub fn set_loopback(&mut self, on: bool) -> Result<(), Error> {
        let mut io = Mmio;
        let base = base(N);
        if self.halves.0.state.resetting || io.read(base + FR) & FR_BUSY != 0 {
            return Err(Error::Busy);
        }
        let cr = io.read(base + CR);
        io.write(base + CR, (cr & !CR_LBE) | if on { CR_LBE } else { 0 });
        Ok(())
    }

    /// See [`Tx::try_write_byte`].
    pub fn try_write_byte(&mut self, byte: u8) -> Result<bool, Error> {
        self.halves.0.try_write_byte(byte)
    }

    /// See [`Rx::try_read_byte`].
    pub fn try_read_byte(&mut self) -> Result<Option<u8>, Error> {
        self.halves.1.try_read_byte()
    }

    /// See [`Tx::try_flush`].
    pub fn try_flush(&mut self) -> Result<bool, Error> {
        self.halves.0.try_flush()
    }

    /// Clears the [PL011 interrupt bits](int) in `mask`. Receive error interrupts clear
    /// only through UARTICR (see [`Rx::clear_interrupt`]). RX and RT also clear by draining
    /// the FIFO, TX by refilling it.
    pub fn clear_interrupt(&mut self, mask: u32) {
        if !self.halves.0.state.resetting {
            Mmio.write(base(N) + ICR, mask);
        }
    }
}

impl<const N: u8, const TX: u8, const RX: u8> Uart<N, PinTuple<TX, RX>, Serial>
where
    Pin<TX>: TxPin<N>,
    Pin<RX>: RxPin<N>,
{
    /// Unmasks the same receive interrupt sources as [`Rx::enable_interrupt`].
    pub fn enable_rx_interrupt(&mut self) {
        self.halves.1.enable_interrupt();
    }

    /// Masks the same receive interrupt sources as [`Rx::disable_interrupt`].
    pub fn disable_rx_interrupt(&mut self) {
        self.halves.1.disable_interrupt();
    }

    /// See [`Tx::enable_interrupt`].
    pub fn enable_tx_interrupt(&mut self) {
        self.halves.0.enable_interrupt();
    }

    /// See [`Tx::disable_interrupt`].
    pub fn disable_tx_interrupt(&mut self) {
        self.halves.0.disable_interrupt();
    }

    /// Pending unmasked interrupts (MIS), as [PL011 interrupt bits](int).
    pub fn interrupt_status(&self) -> u32 {
        if self.halves.0.state.resetting {
            0
        } else {
            Mmio.read(base(N) + MIS)
        }
    }
}
impl<const N: u8, const TX: u8, const RX: u8, I> Uart<N, PinTuple<TX, RX>, I>
where
    Pin<TX>: TxPin<N>,
    Pin<RX>: RxPin<N>,
{
    /// Return the parts once the transmitter is idle and UART reset is confirmed.
    /// Unread or incoming characters are discarded.
    /// A busy transmitter returns the unchanged UART with [`Error::Busy`] (or
    /// [`Error::Timeout`] from [`Self::free_polling`]). Retry or request cancellation.
    /// If reset has already been requested, retry `try_free` to confirm it.
    ///
    /// With a separately owned NVIC line, disable and unpend it first, or use a
    /// handler that accepts the UART being gone. Reset masks the device sources
    /// but leaves NVIC pending state intact. The generated UART1 handler already
    /// ignores a UART held in reset.
    ///
    /// ```no_run
    /// # use rp1_hal::{FreeError, gpio::Pin, uart::{Error, Parts, Uart}};
    /// # type Pins = (Pin<4>, Pin<5>);
    /// fn cancel_and_release(mut uart: Uart<2, Pins>)
    ///     -> Result<Parts<2, Pins>, FreeError<Uart<2, Pins>, Error>>
    /// {
    ///     let _ = uart.try_cancel(); // try_free checks reset again and retains the UART if pending.
    ///     uart.try_free()
    /// }
    /// ```
    pub fn try_free(self) -> Result<Parts<N, PinTuple<TX, RX>, I>, crate::FreeError<Self, Error>> {
        self.try_free_with(&mut Mmio, false)
    }

    /// Poll for wire idle for twice the time needed to send a full FIFO and the byte
    /// being shifted, plus 1 ms. The driver is returned on timeout.
    pub fn free_polling(
        self,
    ) -> Result<Parts<N, PinTuple<TX, RX>, I>, crate::FreeError<Self, Error>> {
        if let Err(error) = ThreadGuard::new() {
            return Err(crate::FreeError {
                driver: self,
                error: error.into(),
            });
        }
        self.try_free_with(&mut Mmio, true)
    }

    fn try_free_with(
        self,
        io: &mut impl Hardware,
        polling: bool,
    ) -> Result<Parts<N, PinTuple<TX, RX>, I>, crate::FreeError<Self, Error>> {
        let mut uart = self;
        let idle = if uart.halves.0.state.resetting {
            Ok(())
        } else if polling {
            release(io, N, uart.halves.0.state.bound_us)
        } else if io.read(base(N) + FR) & FR_BUSY == 0 {
            Ok(())
        } else {
            Err(Error::Busy)
        };
        match idle.and_then(|()| uart.cancel_with(io)) {
            Ok(()) => Ok(uart.into_parts()),
            Err(error) => Err(crate::FreeError {
                driver: uart,
                error,
            }),
        }
    }

    /// Request reset even while transmission is in progress. A truncated frame may
    /// look like a framing error to the peer. Once requested, use [`Self::try_free`]
    /// to collect the parts. The TX pin remains muxed to UART.
    /// Queued TX characters are discarded, as are unread RX characters. The TX pin
    /// is expected to be idle (high) while held in reset (HYP, 2026-11-27).
    pub fn try_cancel(&mut self) -> Result<(), Error> {
        self.cancel_with(&mut Mmio)
    }

    fn cancel_with(&mut self, io: &mut impl Hardware) -> Result<(), Error> {
        self.halves.0.state.resetting = true;
        self.halves.1.state.resetting = true;
        if try_reset(io, N) {
            Ok(())
        } else {
            Err(Error::Busy)
        }
    }

    fn into_parts(self) -> Parts<N, PinTuple<TX, RX>, I> {
        let (tx, rx) = self.halves;
        Parts {
            token: Token {
                _private: PhantomData,
            },
            pins: Pins(
                (tx.pin.release(), rx.pin.release()),
                #[cfg(test)]
                tx.cts,
            ),
            irq: tx.irq,
        }
    }
}

impl<const N: u8, const PIN: u8, I> Polling<'_, Rx<N, PIN, I>> {
    /// Sets the receive wait, initially [`DEFAULT_READ_TIMEOUT`]. The setting persists on the
    /// receiver across `polling()` borrows.
    pub fn set_read_timeout(&mut self, timeout: MicrosDurationU32) {
        self.0.state.timeout = timeout;
    }
}

/// Formatting stops at the first timeout. Bytes already queued continue to transmit.
/// Import only one of `fmt::Write` and `embedded_io::Write` when calling `write!`.
impl<T> fmt::Write for Polling<'_, T>
where
    Self: embedded_io::Write<Error = Error>,
{
    fn write_str(&mut self, s: &str) -> fmt::Result {
        embedded_io::Write::write_all(self, s.as_bytes()).map_err(|_| fmt::Error)
    }
}
impl<T> embedded_io::ErrorType for Polling<'_, T> {
    type Error = Error;
}
// A caller-supplied deadline is admitted before the first UART access. Once admitted,
// a bounded final probe can still win at the end of the operation.
fn default_end(timeout: MicrosDurationU32) -> Result<Deadline, Error> {
    ThreadGuard::new()?;
    Ok(Deadline::after(timeout)?)
}

fn poll<T>(end: Deadline, mut probe: impl FnMut() -> Option<T>) -> Result<T, Error> {
    ThreadGuard::new()?;
    poll_with(end, &mut (), |_| Timer::new().now(), |_| probe())
}

fn poll_with<T, S>(
    end: Deadline,
    state: &mut S,
    mut now: impl FnMut(&mut S) -> timer::Instant,
    mut probe: impl FnMut(&mut S) -> Option<T>,
) -> Result<T, Error> {
    end.check(now(state))?;
    loop {
        let elapsed = now(state) >= end.instant();
        if let Some(ready) = probe(state) {
            return Ok(ready);
        }
        if elapsed {
            return Err(Error::Timeout);
        }
    }
}

impl<const N: u8, const PIN: u8, I> Tx<N, PIN, I> {
    /// Poll for space, then queue up to one FIFO of bytes.
    pub fn write_polling(&mut self, bytes: &[u8], end: Deadline) -> Result<usize, Error> {
        if bytes.is_empty() {
            return Ok(0);
        }
        if self.state.resetting {
            return Err(Error::Busy);
        }
        poll(end, || self.state.enqueue(N, bytes))
    }

    /// Poll for wire idle. A timeout leaves queued bytes on their way to the peer.
    pub fn flush_polling(&mut self, end: Deadline) -> Result<(), Error> {
        if self.state.resetting {
            return Err(Error::Busy);
        }
        poll(end, || {
            (Mmio.read(base(N) + FR) & FR_BUSY == 0).then_some(())
        })
    }
}

impl<const N: u8, const PIN: u8, I> Rx<N, PIN, I> {
    /// Poll for one byte, retaining each byte's receive status until it is read.
    pub fn read_polling(&mut self, bytes: &mut [u8], end: Deadline) -> Result<usize, Error> {
        let Some(first) = bytes.first_mut() else {
            return Ok(0);
        };
        *first = poll(end, || self.try_read_byte().transpose())??;
        Ok(1)
    }
}

impl<const N: u8, const TX: u8, const RX: u8, I> Uart<N, PinTuple<TX, RX>, I>
where
    Pin<TX>: TxPin<N>,
    Pin<RX>: RxPin<N>,
{
    /// See [`Rx::read_polling`].
    pub fn read_polling(&mut self, bytes: &mut [u8], end: Deadline) -> Result<usize, Error> {
        self.halves.1.read_polling(bytes, end)
    }
    /// See [`Tx::write_polling`].
    pub fn write_polling(&mut self, bytes: &[u8], end: Deadline) -> Result<usize, Error> {
        self.halves.0.write_polling(bytes, end)
    }
    /// See [`Tx::flush_polling`].
    pub fn flush_polling(&mut self, end: Deadline) -> Result<(), Error> {
        self.halves.0.flush_polling(end)
    }
}

impl<const N: u8, const PIN: u8, I> embedded_io::Write for Polling<'_, Tx<N, PIN, I>> {
    /// Queues up to one FIFO of bytes, waiting for space for the first byte only.
    /// On timeout nothing was queued, so the same slice can be retried.
    fn write(&mut self, b: &[u8]) -> Result<usize, Error> {
        if b.is_empty() {
            return Ok(0);
        }
        let end = default_end(MicrosDurationU32::micros(self.0.state.bound_us))?;
        self.0.write_polling(b, end)
    }

    /// Waits for wire idle. Transmission continues on timeout.
    fn flush(&mut self) -> Result<(), Error> {
        let end = default_end(MicrosDurationU32::micros(self.0.state.bound_us))?;
        self.0.flush_polling(end)
    }
}

/// Reads at most one byte per call. Receive errors are reported as in [`Rx::try_read_byte`].
impl<const N: u8, const PIN: u8, I> embedded_io::Read for Polling<'_, Rx<N, PIN, I>> {
    fn read(&mut self, b: &mut [u8]) -> Result<usize, Error> {
        if b.is_empty() {
            return Ok(0);
        }
        let end = default_end(self.0.state.timeout)?;
        self.0.read_polling(b, end)
    }
}
impl<const N: u8, const TX: u8, const RX: u8, I> Polling<'_, Uart<N, PinTuple<TX, RX>, I>>
where
    Pin<TX>: TxPin<N>,
    Pin<RX>: RxPin<N>,
{
    /// Sets the receive wait, initially [`DEFAULT_READ_TIMEOUT`]. The setting persists on the
    /// receiver across `polling()` borrows.
    pub fn set_read_timeout(&mut self, timeout: MicrosDurationU32) {
        self.0.halves.1.polling().set_read_timeout(timeout);
    }
}
impl<const N: u8, const TX: u8, const RX: u8, I> embedded_io::Write
    for Polling<'_, Uart<N, PinTuple<TX, RX>, I>>
where
    Pin<TX>: TxPin<N>,
    Pin<RX>: RxPin<N>,
{
    fn write(&mut self, b: &[u8]) -> Result<usize, Error> {
        self.0.halves.0.polling().write(b)
    }
    fn flush(&mut self) -> Result<(), Error> {
        self.0.halves.0.polling().flush()
    }
}
impl<const N: u8, const TX: u8, const RX: u8, I> embedded_io::Read
    for Polling<'_, Uart<N, PinTuple<TX, RX>, I>>
where
    Pin<TX>: TxPin<N>,
    Pin<RX>: RxPin<N>,
{
    fn read(&mut self, b: &mut [u8]) -> Result<usize, Error> {
        self.0.halves.1.polling().read(b)
    }
}

#[cfg(test)]
mod optional_pin;

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use fugit::RateExtU32;
    use std::{collections::BTreeMap, collections::VecDeque, vec::Vec};

    const N: u8 = 2;
    /// UART `N`'s registers. FR is derived: BUSY and TXFF from the test's switches, RXFE from
    /// the queue of received DR values. A character in progress (`busy` for TX, `receiving`
    /// for RX) outlives clearing UARTEN, as on the PL011, and makes an LCR_H write panic; only
    /// asserting UART `N`'s reset ends it and empties the FIFOs. While the reset is held the
    /// block ignores writes and reads as its reset values, and every such access is counted,
    /// so a test sees a driver that keeps using a UART it released. DONE1 drops on the
    /// assert and returns on the release unless `reset_stuck`.
    pub(super) struct Fake {
        regs: BTreeMap<usize, u32>,
        writes: Vec<(usize, u32)>,
        rx: VecDeque<u32>,
        busy: bool,
        /// `busy` clears once `elapsed_us` reaches this time.
        drain_at: Option<u64>,
        receiving: bool,
        in_reset: bool,
        reset_stuck: bool,
        reset_assert_stuck: bool,
        accesses_in_reset: usize,
        /// Free TX FIFO slots, and what was pushed.
        tx_room: usize,
        tx: Vec<u8>,
        /// A DR value that enters the RX FIFO once `elapsed_us` reaches the time.
        rx_later: Option<(u64, u32)>,
        now: u32,
        /// Timer step per read, and the time read so far.
        step_us: u32,
        elapsed_us: u64,
    }
    impl Fake {
        pub(super) fn new() -> Self {
            Self {
                regs: [(RESETS_DONE1, reset_bit(N))].into(),
                writes: Vec::new(),
                rx: VecDeque::new(),
                busy: false,
                drain_at: None,
                receiving: false,
                in_reset: false,
                reset_stuck: false,
                reset_assert_stuck: false,
                accesses_in_reset: 0,
                tx_room: 32,
                tx: Vec::new(),
                rx_later: None,
                // The 32-bit timer wraps during every bounded wait.
                now: u32::MAX - 500,
                step_us: 100,
                elapsed_us: 0,
            }
        }
    }
    impl Registers for Fake {
        fn read(&mut self, address: usize) -> u32 {
            let uart = base(N)..base(N) + 0x1000;
            if self.in_reset && uart.contains(&address) {
                self.accesses_in_reset += 1;
                // FR resets to TXFE | RXFE, every other register to 0 (DOC: ARM DDI0183).
                return if address == base(N) + FR { 0x90 } else { 0 };
            }
            if address == base(N) + FR {
                if self.drain_at.is_some_and(|at| at <= self.elapsed_us) {
                    self.busy = false;
                }
                if let Some((_, data)) = self.rx_later.take_if(|(at, _)| *at <= self.elapsed_us) {
                    self.rx.push_back(data);
                }
                return if self.busy { FR_BUSY } else { 0 }
                    | if self.tx_room == 0 { FR_TXFF } else { 0 }
                    | if self.rx.is_empty() { FR_RXFE } else { 0 };
            }
            if address == base(N) + DR {
                return self.rx.pop_front().expect("RX underflow");
            }
            self.regs.get(&address).copied().unwrap_or(0)
        }
        fn write(&mut self, address: usize, value: u32) {
            if self.in_reset && (base(N)..base(N) + 0x1000).contains(&address) {
                self.accesses_in_reset += 1;
                return;
            }
            if address == base(N) + DR {
                self.tx_room = self.tx_room.checked_sub(1).expect("TX overflow");
                self.tx.push(value as u8);
                return;
            }
            if address == base(N) + LCR_H {
                assert!(!self.busy && !self.receiving, "LCR_H changed mid-character");
            }
            self.writes.push((address, value));
            if address == RESETS_SET1 && value & reset_bit(N) != 0 {
                if self.reset_assert_stuck {
                    return;
                }
                (self.busy, self.receiving, self.tx_room) = (false, false, 32);
                self.rx.clear();
                self.rx_later = None;
                self.in_reset = true;
                self.regs.clear();
                self.regs.insert(RESETS_DONE1, 0);
                return;
            }
            if address == RESETS_CLR1 && value & reset_bit(N) != 0 {
                self.in_reset = false;
                let done = if self.reset_stuck { 0 } else { reset_bit(N) };
                self.regs.insert(RESETS_DONE1, done);
                return;
            }
            self.regs.insert(address, value);
        }
    }
    impl Hardware for Fake {
        fn now_us(&mut self) -> u32 {
            self.now = self.now.wrapping_add(self.step_us);
            self.elapsed_us += u64::from(self.step_us);
            self.now
        }
    }

    #[test]
    fn register_offsets_and_instances_match_the_pac() {
        assert_eq!([DR, ECR, FR, IBRD, FBRD], [0, 4, 0x18, 0x24, 0x28]);
        assert_eq!([LCR_H, CR, IMSC, MIS, ICR], [0x2c, 0x30, 0x38, 0x40, 0x44]);
        assert_eq!(crate::pac::UART0::PTR as usize, base(0));
        assert_eq!(crate::pac::UART4::PTR as usize, base(4));
        // CTRL1 (+4) through RESETS' SET (+0x2000) and CLR (+0x3000) aliases.
        let ctrl1 = crate::pac::RESETS::PTR as usize + 4;
        assert_eq!([RESETS_SET1, RESETS_CLR1], [ctrl1 + 0x2000, ctrl1 + 0x3000]);
    }
    #[test]
    fn start_releases_the_reset_then_programs_8n1_and_enables() {
        let mut f = Fake::new();
        start::<true>(&mut f, N, 50_000_000.Hz(), 115_200.Hz()).unwrap();
        let b = base(N);
        assert_eq!(
            f.writes,
            [
                (RESETS_CLR1, 1 << 28),
                (b + CR, 0),
                (b + IMSC, 0),
                (b + ICR, 0x7ff),
                (b + ECR, 0),
                (b + IFLS, IFLS_HALF),
                (b + IBRD, 27),
                (b + FBRD, 8),
                (b + LCR_H, 0x70),
                (b + CR, 0x301),
            ]
        );
        let mut f = Fake::new();
        assert_eq!(
            start::<true>(&mut f, N, 50_000_000.Hz(), 0.Hz()),
            Err(Error::Invalid)
        );
        assert!(f.writes.is_empty());
    }
    #[test]
    fn a_reset_or_transmitter_that_stays_busy_times_out_before_configuration() {
        let mut f = Fake::new();
        hold_in_reset(&mut f, N);
        f.writes.clear();
        f.reset_stuck = true;
        assert_eq!(
            start::<true>(&mut f, N, 50_000_000.Hz(), 115_200.Hz()),
            Err(Error::Timeout)
        );
        assert_eq!(f.writes, [(RESETS_CLR1, 1 << 28)]);
        let mut f = Fake::new();
        f.busy = true;
        assert_eq!(
            start::<true>(&mut f, N, 50_000_000.Hz(), 115_200.Hz()),
            Err(Error::Timeout)
        );
        assert!((10_000..10_500).contains(&f.elapsed_us));
        assert_eq!(f.writes, [(RESETS_CLR1, 1 << 28)]);
    }
    #[test]
    fn receive_errors_are_told_apart_and_replace_their_character() {
        let mut f = Fake::new();
        assert_eq!(read_byte(&mut f, N), Ok(None));
        f.rx.extend([
            0x41,
            DR_FE | 0x42,
            DR_PE | 0x43,
            DR_BE | DR_FE,
            DR_OE | 0x44,
            0x45,
        ]);
        let mut got = Vec::new();
        while !f.rx.is_empty() {
            got.push(read_byte(&mut f, N));
        }
        assert_eq!(
            got,
            [
                Ok(Some(0x41)),
                Err(Error::Framing),
                Err(Error::Parity),
                Err(Error::Break),
                Err(Error::Overrun),
                Ok(Some(0x45)),
            ]
        );
        // Each error also clears the sticky copy in RSR.
        let acknowledged: Vec<u32> = f
            .writes
            .iter()
            .filter_map(|&(address, mask)| (address == base(N) + ICR).then_some(mask))
            .collect();
        assert_eq!(acknowledged, [int::FE, int::PE, int::BE | int::FE, int::OE]);
    }
    #[test]
    fn error_kinds_for_embedded_io() {
        use embedded_io::{Error as _, ErrorKind};
        for error in [Error::Break, Error::Parity, Error::Framing] {
            assert_eq!(error.kind(), ErrorKind::InvalidData);
        }
        assert_eq!(Error::Timeout.kind(), ErrorKind::TimedOut);
        assert_eq!(Error::Invalid.kind(), ErrorKind::InvalidInput);
        assert_eq!(Error::Overrun.kind(), ErrorKind::InvalidData);
        assert_eq!(Error::Precondition.kind(), ErrorKind::Other);
    }
    fn model_poll<T>(
        io: &mut Fake,
        timeout: u32,
        probe: impl FnMut(&mut Fake) -> Option<T>,
    ) -> Result<T, Error> {
        let end = Deadline::at(timer::Instant::from_ticks(
            io.elapsed_us + u64::from(timeout),
        ));
        poll_with(
            end,
            io,
            |io| {
                io.now_us();
                timer::Instant::from_ticks(io.elapsed_us)
            },
            probe,
        )
    }

    /// embedded-io `write`: a full FIFO ends the call with the count of the bytes queued, and
    /// only a call that queued nothing times out.
    #[test]
    fn a_write_reports_the_prefix_it_queued() {
        let mut f = Fake::new();
        f.tx_room = 3;
        let send = |f: &mut Fake, b: &[u8]| {
            model_poll(f, 1_000, |f| {
                let count = enqueue(f, N, b);
                (count != 0).then_some(count)
            })
        };
        assert_eq!(send(&mut f, b"hello"), Ok(3));
        assert_eq!(f.tx, b"hel");
        assert_eq!(send(&mut f, b"lo"), Err(Error::Timeout));
        assert_eq!(f.tx, b"hel");
        f.tx_room = 32;
        assert_eq!(send(&mut f, b"lo"), Ok(2));
        assert_eq!(enqueue(&mut f, N, b""), 0);
        assert_eq!(f.tx, b"hello");
    }

    #[test]
    fn absolute_polling_has_a_final_probe_and_checks_admission_before_uart_access() {
        let mut f = Fake::new();
        f.step_us = 1;
        for (now, end, expected) in [
            (0, 0, Err(Error::Timeout)),
            (0, 1, Err(Error::Timeout)),
            (0, u64::from(u32::MAX), Err(Error::InvalidDeadline)),
            (u64::from(u32::MAX) - 2, u64::from(u32::MAX) + 2, Ok(42)),
            (u64::from(u32::MAX) + 1, 100, Err(Error::Timeout)),
        ] {
            f.elapsed_us = now;
            let deadline = Deadline::at(timer::Instant::from_ticks(end));
            assert_eq!(
                poll_with(
                    deadline,
                    &mut f,
                    |f| {
                        f.now_us();
                        timer::Instant::from_ticks(f.elapsed_us)
                    },
                    |f| {
                        assert_eq!(expected, Ok(42));
                        (f.elapsed_us == end).then_some(42)
                    }
                ),
                expected
            );
        }
    }

    /// The bound is time on the timer, not polls: the same wait ends after the same time
    /// whether a poll takes 1 us or 300 us, and across the timer's wrap.
    #[test]
    fn bootstrap_and_release_polling_preserve_their_timer_bound_across_wrap() {
        for (step_us, timeout_us) in [(1, 2_000), (300, 2_000), (7, 0), (1 << 30, u32::MAX)] {
            let mut io = Fake::new();
            (io.step_us, io.tx_room, io.busy) = (step_us, 0, true);
            for (mask, value) in [(FR_TXFF, 0), (FR_RXFE, 0), (FR_BUSY, 0)] {
                let started = io.elapsed_us;
                assert_eq!(
                    wait(&mut io, base(N) + FR, mask, value, timeout_us),
                    Err(Error::Timeout)
                );
                // Each wait reads the timer once to start, then runs just past the timeout.
                let elapsed = io.elapsed_us - started - u64::from(step_us);
                assert!(elapsed > u64::from(timeout_us));
                assert!(elapsed <= u64::from(timeout_us) + u64::from(step_us));
            }
        }
    }

    #[test]
    fn overrun_is_retained_when_the_peer_stops_without_another_character() {
        let mut io = Fake::new();
        io.regs.insert(base(N) + RIS, int::OE);
        io.rx.push_back(0x41);
        assert_eq!(read_byte(&mut io, N), Ok(Some(0x41)));
        assert!(io.writes.is_empty());
        assert_eq!(read_byte(&mut io, N), Err(Error::Overrun));
        assert_eq!(io.writes, [(base(N) + ICR, int::OE), (base(N) + ECR, 0)]);
    }

    #[test]
    fn a_read_waits_for_a_byte_that_arrives_in_time() {
        let mut f = Fake::new();
        f.rx_later = Some((40_000, 0x55));
        let receive = |f: &mut Fake| read_byte(f, N).transpose();
        assert_eq!(model_poll(&mut f, 30_000, receive), Err(Error::Timeout));
        assert_eq!(model_poll(&mut f, 30_000, receive), Ok(Ok(0x55)));
        f.rx_later = Some((0, DR_FE));
        assert_eq!(model_poll(&mut f, 30_000, receive), Ok(Err(Error::Framing)));
    }
    /// A transmitter that does not drain in time is not cut off by `free_polling`: nothing is written
    /// and the error is reported, so the caller still has the UART. A reset does not wait for TX.
    #[test]
    fn free_polling_refuses_a_busy_transmitter_and_cancel_does_not_wait() {
        let stopped = [(RESETS_SET1, 1 << 28)];
        let mut f = Fake::new();
        f.busy = true;
        let tx = Tx {
            pin: unsafe { Pin::<4>::new() }.assume(),
            irq: Serial { _private: () },
            state: TxState {
                bound_us: 5_000,
                divisor: 1736,
                earliest_end: 0,
                queued: 0,
                resetting: false,
            },
            cts: None,
        };
        let rx = Rx {
            pin: unsafe { Pin::<5>::new() }.assume(),
            _irq: Serial { _private: () },
            state: RxState {
                timeout: DEFAULT_READ_TIMEOUT,
                resetting: false,
            },
        };
        let uart = match Uart::<N, (Pin<4>, Pin<5>)>::join(tx, rx).try_free_with(&mut f, true) {
            Err(e) => {
                assert_eq!(e.error, Error::Timeout);
                e.driver
            }
            Ok(_) => panic!("released a busy UART"),
        };
        assert!(f.writes.is_empty());
        let (tx, rx) = uart.split();
        f.busy = false;
        let _: Parts<N, (Pin<4>, Pin<5>)> = Uart::join(tx, rx).try_free_with(&mut f, true).unwrap();
        assert_eq!(f.writes, stopped);

        let mut f = Fake::new();
        f.busy = true;
        hold_in_reset(&mut f, N);
        assert_eq!((f.writes.as_slice(), f.elapsed_us), (&stopped[..], 0));
    }
    #[test]
    fn release_keeps_both_halves_while_reset_is_unconfirmed() {
        let mut io = Fake::new();
        io.reset_assert_stuck = true;
        let tx = Tx {
            pin: unsafe { Pin::<4>::new() }.assume(),
            irq: Serial { _private: () },
            state: TxState {
                bound_us: 5_000,
                divisor: 1736,
                earliest_end: 0,
                queued: 0,
                resetting: false,
            },
            cts: None,
        };
        let rx = Rx {
            pin: unsafe { Pin::<5>::new() }.assume(),
            _irq: Serial { _private: () },
            state: RxState {
                timeout: DEFAULT_READ_TIMEOUT,
                resetting: false,
            },
        };
        let error = Uart::<N, (Pin<4>, Pin<5>)>::join(tx, rx)
            .try_free_with(&mut io, false)
            .err()
            .expect("reset is still pending");
        let (mut uart, error) = error.into_parts();
        assert_eq!(error, Error::Busy);
        assert_eq!(uart.try_write_byte(0x41), Err(Error::Busy));
        assert_eq!(uart.try_flush(), Err(Error::Busy));
        let (mut tx, mut rx) = uart.split();
        assert_eq!(tx.try_write_byte(0x41), Err(Error::Busy));
        assert_eq!(tx.try_flush(), Err(Error::Busy));
        assert_eq!(rx.try_read_byte(), Err(Error::Busy));
        io.reset_assert_stuck = false;
        assert!(Uart::join(tx, rx).try_free_with(&mut io, false).is_ok());
    }

    /// `free` (TX drained, a character arriving) and `abort` (one being sent, one arriving)
    /// never touch LCR_H under a live character, and a bring-up afterwards finds nothing left
    /// to send or receive.
    #[test]
    fn release_ends_characters_in_progress_and_discards_the_fifos() {
        for forced in [false, true] {
            let mut f = Fake::new();
            (f.receiving, f.busy, f.tx_room) = (true, forced, 30);
            f.rx.extend([0x41, 0x42]);
            if forced {
                hold_in_reset(&mut f, N);
            } else {
                assert_eq!(release(&mut f, N, 5_000), Ok(()));
                hold_in_reset(&mut f, N);
            }
            start::<true>(&mut f, N, 50_000_000.Hz(), 115_200.Hz()).unwrap();
            assert_eq!((read_byte(&mut f, N), f.tx_room), (Ok(None), 32));
            assert_eq!(f.accesses_in_reset, 0);
        }
    }
    /// The fake catches a driver that keeps using a released UART: the write is lost.
    #[test]
    fn a_uart_held_in_reset_ignores_writes_and_reads_as_reset() {
        let mut f = Fake::new();
        hold_in_reset(&mut f, N);
        assert_eq!(enqueue(&mut f, N, b"x"), 1);
        assert_eq!(read_byte(&mut f, N), Ok(None));
        assert_eq!((f.tx.len(), f.accesses_in_reset), (0, 4));
    }
    #[test]
    fn interrupt_mask_updates_keep_the_other_bits() {
        let mut f = Fake::new();
        let b = base(N);
        set_mask(&mut f, N, RX_INTERRUPTS, true);
        set_mask(&mut f, N, int::TX, true);
        assert_eq!(f.regs[&(b + IMSC)], RX_INTERRUPTS | int::TX);
        f.regs.insert(b + MIS, int::TX | int::OE | int::RX);
        assert_eq!(
            pending_interrupts(&mut f, N, RX_INTERRUPTS),
            int::OE | int::RX
        );
        assert_eq!(pending_interrupts(&mut f, N, int::TX), int::TX);
        clear_interrupts(&mut f, N, RX_INTERRUPTS, int::TX | int::OE);
        clear_interrupts(&mut f, N, int::TX, u32::MAX);
        assert_eq!(f.writes[2..], [(b + ICR, int::OE), (b + ICR, int::TX)]);
        set_mask(&mut f, N, RX_INTERRUPTS, false);
        assert_eq!(f.regs[&(b + IMSC)], int::TX);
    }
    /// The TX bound covers a full FIFO at the slowest baud rate with a divisor and does not
    /// depend on the read timeout.
    #[test]
    fn the_tx_bound_follows_the_baud_rate() {
        assert_eq!(tx_bound_us(115_200.Hz()), 2 * 2_865 + 1_000);
        assert_eq!(tx_bound_us(1.MHz()), 2 * 330 + 1_000);
        assert_eq!(tx_bound_us(300.Hz()), 2 * 1_100_000 + 1_000);
        let slowest = 50_000_000 * 4 / (65535 * 64 + 63) + 1;
        assert!(divisors(50_000_000.Hz(), slowest.Hz()).is_ok());
        assert!(tx_bound_us(slowest.Hz()) > 2 * 330 * 1_000_000 / slowest);
        // A full FIFO at 115200 baud drains within the bound.
        let mut f = Fake::new();
        f.busy = true;
        f.drain_at = Some(2_865);
        assert_eq!(release(&mut f, N, tx_bound_us(115_200.Hz())), Ok(()));
    }
    #[test]
    fn baud_divisor_and_invalid_inputs() {
        assert_eq!(divisors(50_000_000.Hz(), 115200.Hz()), Ok((27, 8)));
        assert_eq!(divisors(0.Hz(), 115200.Hz()), Err(Error::Invalid));
        assert_eq!(divisors(50_000_000.Hz(), 0.Hz()), Err(Error::Invalid));
        assert_eq!(divisors(50_000_000.Hz(), 1.Hz()), Err(Error::Invalid));
    }
}
