//! SPI master on the RP1 DesignWare APB SSI blocks (DOC: RP1 peripherals §3.6, Linux
//! spi-dw-core). STATIC: a same-core wait timer enables IRQ/WFE waits through
//! [`Spi::blocking`]. [`Spi::polling`] selects polling. Both implement [`SpiBus`]
//! and support [`Device`] with GPIO chip select.
//!
//! Instances (DOC §3.6.1): SPI0/1/2/3/5/6/8 are masters, SPI4/7 slave-only. Only the bank-0
//! masters SPI0, 1, 2, 3, 5 are exposed; SPI6/8 are not on bank 0. The serial clock is clk_sys
//! divided by an even BAUDR (2..=65534): `sclk = Clocks::sys().raw() / div`, so 100 MHz at most at
//! 200 MHz. clk_sys belongs to RP1 (clock authority) and is not voted on through SCMI. The
//! reset is RESETS CTRL1 bit 10 + n, DONE1 the same bit (rp1-regdb resets.toml).
//!
//! The controller's own chip-select outputs are not routed: SER selects CS0 internally because
//! the DW SSI only shifts with a slave selected, and [`Device`] drives a GPIO as chip select.
use crate::{
    clocks::Clocks,
    gpio::{Alternate, Output, Pin, Unconfigured},
    read,
    timer::{self, Deadline, Duration, Instant, Timer, WaitConfig},
    write,
};
use core::marker::PhantomData;
use rp1_rt::interrupt::ThreadGuard;

pub use crate::Try;
use embedded_hal::spi::{
    ErrorKind, ErrorType, Mode, Operation, Phase, Polarity, SpiBus, SpiDevice,
};
pub use irq_wait::{__on_interrupt, Blocking, Irq};

use crate::pac::dw_ssi::{RegisterBlock, ctrlr0, int, irq_wrap, sr};
macro_rules! offsets {
    ($($name:ident = $field:ident),* $(,)?) => {$(const $name: usize = core::mem::offset_of!(RegisterBlock, $field);)*};
}
offsets! {
    CTRLR0 = ctrlr0, SSIENR = ssienr, SER = ser, BAUDR = baudr, TXFTLR = txftlr, RXFTLR = rxftlr,
    RXFLR = rxflr, SR = sr, IMR = imr, ISR = isr, RISR = risr, ICR = icr, DMACR = dmacr, DR = dr,
    RX_SAMPLE_DLY = rx_sample_dly, IRQ_WRAP = irq_wrap,
}
/// Default deadline for a whole bus call or device transaction. At the slowest
/// SCLK (~3 kHz), it covers about ten 32-bit frames. Use `*_until` or
/// [`WaitConfig`] for longer transfers.
const TIMEOUT_US: u32 = 100_000;
const RESETS_SET1: usize = 0x4001_4000 + 0x2000 + 4;
const RESETS_CLR1: usize = 0x4001_4000 + 0x3000 + 4;
const RESETS_DONE1: usize = 0x4001_4000 + 0x1c;
const fn reset_bit(n: u8) -> u32 {
    1 << (10 + n)
}
const fn base(n: u8) -> usize {
    0x4005_0000 + n as usize * 0x4000
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    /// The operation's absolute deadline elapsed.
    Timeout,
    /// Frequency, frame size or word type out of range.
    Invalid,
    /// The clock tree no longer matches the `Clocks` value, or the block is not a DW SSI.
    Precondition,
    /// RX/TX FIFO overflow or underflow flagged by the controller.
    Overrun,
    /// The controller has not stopped.
    Busy,
    /// The deadline exceeds the supported comparison horizon.
    InvalidDeadline,
    /// Synchronous operations require unmasked thread context.
    WrongContext,
    /// The managed device IRQ is unavailable.
    IrqUnavailable,
    /// The dedicated deadline alarm could not be retired.
    AlarmUnavailable,
    /// The controller reported a multi-master/chip-select conflict.
    ChipSelectFault,
}
impl From<timer::Error> for Error {
    fn from(error: timer::Error) -> Self {
        match error {
            timer::Error::Timeout => Self::Timeout,
            timer::Error::InvalidDeadline => Self::InvalidDeadline,
            timer::Error::IrqUnavailable => Self::IrqUnavailable,
            timer::Error::AlarmUnavailable => Self::AlarmUnavailable,
            timer::Error::WrongContext | timer::Error::Masked => Self::WrongContext,
        }
    }
}

impl embedded_hal::spi::Error for Error {
    fn kind(&self) -> ErrorKind {
        match self {
            Self::Overrun => ErrorKind::Overrun,
            Self::ChipSelectFault => ErrorKind::ChipSelectFault,
            _ => ErrorKind::Other,
        }
    }
}

mod sealed {
    pub trait Pin {}
    impl<const N: u8> Pin for crate::gpio::Pin<N> {}
    pub trait CompletePins<const N: u8> {}
    pub trait Bus {}
    impl<const N: u8, P: super::CompletePins<N>, I> Bus for super::Polling<'_, N, P, I> {}
    impl<B: Bus + ?Sized> Bus for &mut B {}
    impl<P: super::CompletePins<N>, const CORE: u8, const N: u8> Bus
        for super::Blocking<'_, N, P, CORE>
    {
    }
    /// Lets a [`super::Bus::admit`] call through; only `spi` can make one, so code outside
    /// this crate cannot call any [`super::Bus`] method.
    pub struct Key(pub(in crate::spi) ());
    /// Proof that [`super::Bus::admit`] passed for words `W` (frame size and clk_sys); the
    /// unchecked operations take it. Only `admit` makes one.
    pub struct Admitted<W>(
        pub(in crate::spi) super::Deadline,
        pub(in crate::spi) core::marker::PhantomData<W>,
    );
    impl<W> Clone for Admitted<W> {
        fn clone(&self) -> Self {
            *self
        }
    }
    impl<W> Copy for Admitted<W> {}
    pub trait Word {}
    impl Word for u8 {}
    impl Word for u16 {}
    impl Word for u32 {}
}
/// SCLK route of controller N; `FUNC` is the GPIO function select.
pub trait Sck<const N: u8>: sealed::Pin {
    const FUNC: u8;
}
/// MOSI (SIO0, TXD) route of controller N.
pub trait Mosi<const N: u8>: sealed::Pin {
    const FUNC: u8;
}
/// MISO (SIO1, RXD) route of controller N.
pub trait Miso<const N: u8>: sealed::Pin {
    const FUNC: u8;
}
/// Outputs: input enable, 8 mA, Schmitt, fast slew. MISO: input enable, pull-down, Schmitt; also
/// the pad state while the mux switches (output disabled).
const OUT_PAD: u32 = 0x63;
const IN_PAD: u32 = 0x46;

/// A frame word: the low `frame_bits` bits are shifted, MSB first.
pub trait Word: sealed::Word + Copy + 'static {
    const BITS: u8;
    fn to_u32(self) -> u32;
    fn from_u32(v: u32) -> Self;
    #[doc(hidden)]
    fn rx(words: &mut [Self]) -> Rx<'_>;
    #[doc(hidden)]
    fn tx(words: &[Self]) -> Tx<'_>;
}
/// Receive buffer of any word type: one non-generic transfer loop serves all of them (the
/// shared-SRAM image is small).
#[doc(hidden)]
pub enum Rx<'a> {
    U8(&'a mut [u8]),
    U16(&'a mut [u16]),
    U32(&'a mut [u32]),
}
#[doc(hidden)]
pub enum Tx<'a> {
    U8(&'a [u8]),
    U16(&'a [u16]),
    U32(&'a [u32]),
}
impl Rx<'_> {
    fn len(&self) -> usize {
        match self {
            Self::U8(w) => w.len(),
            Self::U16(w) => w.len(),
            Self::U32(w) => w.len(),
        }
    }
    #[inline(always)]
    fn set(&mut self, i: usize, v: u32) {
        match self {
            Self::U8(w) => w.get_mut(i).map(|w| *w = v as u8),
            Self::U16(w) => w.get_mut(i).map(|w| *w = v as u16),
            Self::U32(w) => w.get_mut(i).map(|w| *w = v),
        };
    }
}
impl Tx<'_> {
    fn len(&self) -> usize {
        match self {
            Self::U8(w) => w.len(),
            Self::U16(w) => w.len(),
            Self::U32(w) => w.len(),
        }
    }
    #[inline(always)]
    fn get(&self, i: usize) -> u32 {
        match self {
            Self::U8(w) => w.get(i).map_or(0, |w| u32::from(*w)),
            Self::U16(w) => w.get(i).map_or(0, |w| u32::from(*w)),
            Self::U32(w) => w.get(i).copied().unwrap_or(0),
        }
    }
}
macro_rules! word {
    ($($t:ty: $v:ident),*) => {$(impl Word for $t {
        const BITS: u8 = <$t>::BITS as u8;
        #[inline(always)]
        fn to_u32(self) -> u32 { self as u32 }
        #[inline(always)]
        fn from_u32(v: u32) -> Self { v as $t }
        fn rx(words: &mut [Self]) -> Rx<'_> { Rx::$v(words) }
        fn tx(words: &[Self]) -> Tx<'_> { Tx::$v(words) }
    })*};
}
word!(u8: U8, u16: U16, u32: U32);

/// [`Parts::init`] settings: `Config::new(hz, mode)` plus the `with_*` setters (the struct is
/// `#[non_exhaustive]`, so fields can be added without breaking callers).
///
/// ```
/// use rp1_hal::{fugit::RateExtU32, spi::Config};
/// let c = Config::new(10.MHz(), embedded_hal::spi::MODE_3)
///     .with_frame_bits(16)
///     .with_rx_sample_dly(2);
/// assert_eq!((c.frequency.raw(), c.frame_bits, c.rx_sample_dly), (10_000_000, 16, 2));
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Config {
    /// Upper bound for SCLK; the rate used is the highest `clk_sys / even divider` not above it.
    pub frequency: fugit::HertzU32,
    pub mode: Mode,
    /// 4..=16, or 4..=32 when the instance has the 32-bit frame field. Default 8.
    pub frame_bits: u8,
    /// RX_SAMPLE_DLY: clk_sys cycles by which MISO sampling is delayed (0 = none, the
    /// default). Written as given, without a range check or read-back, as Linux spi-dw-core
    /// does (`rx-sample-delay-ns`). The usable maximum is the DW_apb_ssi synthesis parameter
    /// SSI_RX_DLY_SR_DEPTH, which is not known for RP1, and what the block does with a larger
    /// value is not verified (UNVERIFIED: recalled from the DesignWare databook as "no delay").
    /// STATIC: only 0 is hardware-tested; an external device at high SCLK may need a delay or
    /// a lower rate.
    pub rx_sample_dly: u8,
}
impl Config {
    pub const fn new(frequency: fugit::HertzU32, mode: Mode) -> Self {
        Self {
            frequency,
            mode,
            frame_bits: 8,
            rx_sample_dly: 0,
        }
    }
    pub const fn with_frequency(mut self, frequency: fugit::HertzU32) -> Self {
        self.frequency = frequency;
        self
    }
    pub const fn with_mode(mut self, mode: Mode) -> Self {
        self.mode = mode;
        self
    }
    pub const fn with_frame_bits(mut self, frame_bits: u8) -> Self {
        self.frame_bits = frame_bits;
        self
    }
    /// See [`Config::rx_sample_dly`] (the upper limit is unverified).
    pub const fn with_rx_sample_dly(mut self, cycles: u8) -> Self {
        self.rx_sample_dly = cycles;
        self
    }
}

/// Even BAUDR divider for at most `hz` from `sys_hz` (as Linux dw_spi: round up, then to even).
fn divider(sys_hz: u32, hz: u32) -> Result<u32, Error> {
    if hz == 0 || sys_hz == 0 {
        return Err(Error::Invalid);
    }
    let div = sys_hz.div_ceil(hz).max(2).next_multiple_of(2);
    if div > 0xfffe {
        return Err(Error::Invalid);
    }
    Ok(div)
}

pub struct Token<const N: u8> {
    _private: core::marker::PhantomData<core::cell::Cell<()>>,
}
/// The SPI controller role.
pub struct Controller {
    _private: (),
}

/// Marker for a controller whose IRQ is available to the application.
pub struct NoIrq;

/// A configured controller. Use [`Spi::blocking`] on SPI0, or select [`Spi::polling`].
/// Each `SpiBus` call checks once that clk_sys still matches, before any
/// FIFO access. This includes a whole `transfer_in_place` buffer. A [`Device`]
/// checks once for an entire transaction.
///
/// Dropping the driver leaves the controller enabled and the pins muxed. Recover
/// the token with [`Spi::try_free`] when you need to initialize it again.
pub struct Spi<const N: u8, P: CompletePins<N>, Role = Controller, I = NoIrq> {
    pins: P::Active,
    clocks: Clocks,
    depth: u16,
    outstanding: u16,
    dfs32: bool,
    /// Frame size, as encoded in `cr0`.
    bits: u8,
    cr0: u32,
    div: u32,
    rsd: u32,
    state: State,
    _role: PhantomData<Role>,
    irq: I,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    Idle,
    Transfer,
    Stopping,
}

/// The initialization error retains the block and its pins during recovery.
pub type InitError<const N: u8, P, I = NoIrq> =
    crate::InitError<Parts<N, P, I>, Error, crate::Recovery<Parts<N, P, I>>>;

/// Explicit polling access to the SPI bus.
/// Bus calls and reconfiguration require unmasked thread context. Use the
/// controller's native `try_*` operations from an interrupt handler.
pub struct Polling<'a, const N: u8, P: CompletePins<N>, I = NoIrq> {
    spi: &'a mut Spi<N, P, Controller, I>,
    config: WaitConfig,
}

impl<const N: u8> Token<N> {
    /// # Safety
    /// Generated resources only: sole owner of SPI N, its reset and IRQ, with Linux's node disabled.
    ///
    /// Any other N is a compile-time error (SPI4/7 are slave-only, SPI6/8 not on bank 0):
    ///
    /// ```compile_fail,E0080
    /// let _ = unsafe { rp1_hal::spi::Token::<4>::new() };
    /// ```
    pub const unsafe fn new() -> Self {
        const { assert!(matches!(N, 0 | 1 | 2 | 3 | 5)) };
        Self {
            _private: core::marker::PhantomData,
        }
    }
}

pin_builder!(Pin, []);
pin_builder!(Pin, [], sck: SCK => Sck);
pin_builder!(Pin, [sck: SCK], mosi: MOSI => Mosi);
pin_builder!(Pin, [sck: SCK, mosi: MOSI], miso: MISO => Miso);
complete_pins!(SCK: Sck, MOSI: Mosi, MISO: Miso; PinTuple<SCK, MOSI, MISO, Alternate>);

/// A declared SPI N with its pins, as the generated resources hand it out.
pub struct Parts<const N: u8, P: CompletePins<N>, I = NoIrq> {
    token: Token<N>,
    pins: Pins<N, P>,
    irq: I,
}
impl<const N: u8, P: CompletePins<N>> Parts<N, P> {
    /// Assemble the owned block and the routed pin bundle.
    pub fn new(token: Token<N>, pins: Pins<N, P>) -> Self {
        Self {
            token,
            pins,
            irq: NoIrq,
        }
    }

    /// Return the block and pins for manual assembly.
    pub fn into_parts(self) -> (Token<N>, Pins<N, P>) {
        (self.token, self.pins)
    }
}

/// SCK, MOSI and MISO.
type PinTuple<const SCK: u8, const MOSI: u8, const MISO: u8, M = Unconfigured> =
    (Pin<SCK, M>, Pin<MOSI, M>, Pin<MISO, M>);

impl<const N: u8, const SCK: u8, const MOSI: u8, const MISO: u8, I>
    Parts<N, PinTuple<SCK, MOSI, MISO>, I>
where
    Pin<SCK>: Sck<N>,
    Pin<MOSI>: Mosi<N>,
    Pin<MISO>: Miso<N>,
{
    /// Checks clk_sys and the configuration, releases only this controller's reset,
    /// detects FIFO depth and frame width, then routes the typed pins. GPIO bank 0 and
    /// the raw microsecond timer must already run. [`Clocks::wait_from_proc0`] provides
    /// the clock description for proc1.
    ///
    /// A stale clock, invalid frequency or frame size outside 4–32 bits fails before
    /// reset is released. The probe checks the instance's frame limit with SSI
    /// disabled. Pins are muxed only on success.
    ///
    /// Retry with 8-bit frames if the hardware rejects the desired frame width:
    ///
    /// ```no_run
    /// # use rp1_hal::{clocks::Clocks, fugit::RateExtU32, gpio::Pin, spi};
    /// # type Pins = (Pin<11>, Pin<10>, Pin<9>);
    /// fn start(parts: spi::Parts<0, Pins>, clocks: &Clocks)
    ///     -> Result<spi::Spi<0, Pins>, spi::InitError<0, Pins>>
    /// {
    ///     let config = spi::Config::new(1.MHz(), embedded_hal::spi::MODE_0);
    ///     match parts.init(clocks, config.with_frame_bits(32)) {
    ///         Ok(bus) => Ok(bus),
    ///         Err(error) => error.try_into_parts()?.init(clocks, config.with_frame_bits(8)),
    ///     }
    /// }
    /// ```
    pub fn init(
        self,
        clocks: &Clocks,
        config: Config,
    ) -> Result<
        Spi<N, PinTuple<SCK, MOSI, MISO>, Controller, I>,
        InitError<N, PinTuple<SCK, MOSI, MISO>, I>,
    > {
        self.init_on(&mut Mmio::new(N), clocks, config, |(sck, mosi, miso)| {
            (
                sck.into_alternate(<Pin<SCK> as Sck<N>>::FUNC, IN_PAD, OUT_PAD),
                mosi.into_alternate(<Pin<MOSI> as Mosi<N>>::FUNC, IN_PAD, OUT_PAD),
                miso.into_alternate(<Pin<MISO> as Miso<N>>::FUNC, IN_PAD, IN_PAD),
            )
        })
    }

    fn init_on(
        self,
        io: &mut impl Io,
        clocks: &Clocks,
        config: Config,
        route: impl FnOnce(PinTuple<SCK, MOSI, MISO>) -> PinTuple<SCK, MOSI, MISO, Alternate>,
    ) -> Result<
        Spi<N, PinTuple<SCK, MOSI, MISO>, Controller, I>,
        InitError<N, PinTuple<SCK, MOSI, MISO>, I>,
    > {
        if !io.sys_current_strict(clocks) {
            return Err(InitError::new(Error::Precondition, self));
        }
        let div = match divider(clocks.sys().raw(), config.frequency.raw())
            .and_then(|div| control(config.mode, config.frame_bits, true).map(|_| div))
        {
            Ok(div) => div,
            Err(error) => return Err(InitError::new(error, self)),
        };
        if let Err(error) = io.release_reset(reset_bit(N)) {
            return Err(InitError::recovering(error, self));
        }
        mask_interrupts(io);
        let Some((depth, dfs32)) = probe(io) else {
            return Err(InitError::recovering(Error::Precondition, self));
        };
        // If width detection rejects the configuration, probe has left SSI disabled
        // with reset released, the same safe state as try_free.
        let cr0 = match control(config.mode, config.frame_bits, dfs32) {
            Ok(cr0) => cr0,
            Err(error) => return Err(InitError::new(error, self)),
        };
        let rsd = config.rx_sample_dly.into();
        setup(io, cr0, div, rsd);
        Ok(Spi {
            pins: route(self.pins.0),
            clocks: *clocks,
            depth,
            dfs32,
            bits: config.frame_bits,
            cr0,
            div,
            rsd,
            state: State::Idle,
            outstanding: 0,
            _role: PhantomData,
            irq: self.irq,
        })
    }
}

impl<const N: u8, P: CompletePins<N>, I> InitError<N, P, I> {
    /// Request SPI reset and return the parts once reset is confirmed.
    pub fn try_recover(self) -> Result<Parts<N, P, I>, Self> {
        self.recover(|| {
            write(RESETS_SET1, reset_bit(N));
            read(RESETS_DONE1) & reset_bit(N) == 0
        })
    }
}

/// Depth of the FIFO behind threshold register `reg` (0..depth-1 read back, as Linux's TXFTLR
/// probe); 1 if nothing sticks.
fn fifo_depth(io: &mut impl Io, reg: usize) -> u32 {
    let mut depth = 1;
    while depth < 256 {
        io.write(reg, depth);
        if io.read(reg) != depth {
            break;
        }
        depth += 1;
    }
    io.write(reg, 0);
    depth
}

/// Frames that may be in flight (the shallower of the TX and RX FIFOs, which DW_apb_ssi sizes
/// separately) and whether CTRLR0 has the 32-bit DFS field. `None` if the block does not behave
/// like a DW SSI (no FIFO).
fn probe(io: &mut impl Io) -> Option<(u16, bool)> {
    io.write(SSIENR, 0);
    if io.read(SSIENR) & 1 != 0 || io.read(SR) & sr::BUSY != 0 {
        return None;
    }
    let depth = fifo_depth(io, TXFTLR).min(fifo_depth(io, RXFTLR));
    io.write(CTRLR0, u32::MAX);
    let dfs32 = io.read(CTRLR0) & ctrlr0::DFS_MASK == 0;
    io.write(CTRLR0, 0);
    (depth > 1).then_some((depth as u16, dfs32))
}

/// CTRLR0: Motorola SPI, transmit and receive, mode, frame size (SRL off).
fn control(mode: Mode, bits: u8, dfs32: bool) -> Result<u32, Error> {
    if bits < 4 || bits > if dfs32 { 32 } else { 16 } {
        return Err(Error::Invalid);
    }
    let dfs = u32::from(bits - 1);
    let mut cr0 = if dfs32 {
        dfs << ctrlr0::DFS32_SHIFT
    } else {
        dfs
    };
    if mode.phase == Phase::CaptureOnSecondTransition {
        cr0 |= ctrlr0::SCPH;
    }
    if mode.polarity == Polarity::IdleHigh {
        cr0 |= ctrlr0::SCPOL;
    }
    Ok(cr0)
}

/// Disabled -> configured with DMA off, CS0 selected -> enabled (FIFOs flushed). The interrupt
/// mask is the owner's and stays as it is.
fn setup(io: &mut impl Io, cr0: u32, div: u32, rsd: u32) {
    io.write(SSIENR, 0);
    io.write(DMACR, 0);
    io.write(CTRLR0, cr0);
    io.write(BAUDR, div);
    io.write(RX_SAMPLE_DLY, rsd);
    io.write(SER, 1);
    io.read(ICR);
    io.write(SSIENR, 1);
}

/// Masks every controller interrupt and closes the RP1 wrapper.
fn mask_interrupts(io: &mut impl Io) {
    io.write(IMR, 0);
    let wrap = io.read(IRQ_WRAP);
    io.write(IRQ_WRAP, wrap & !irq_wrap::IRQ_EN);
}
/// [`Spi::enable_interrupt`]: the controller mask, then the RP1 wrapper.
fn enable_interrupt(io: &mut impl Io, mask: u32) {
    let unmasked = io.read(IMR) | mask;
    io.write(IMR, unmasked);
    let wrap = io.read(IRQ_WRAP);
    io.write(IRQ_WRAP, wrap | irq_wrap::IRQ_EN);
}

trait Io {
    fn read(&mut self, offset: usize) -> u32;
    fn write(&mut self, offset: usize, value: u32);
    fn now(&mut self) -> Instant;
    fn now_low(&mut self) -> u32 {
        self.now().ticks() as u32
    }
    /// clk_sys still matches `clocks`, per bus call ([`Clocks::sys_current`]).
    fn sys_current(&mut self, clocks: &Clocks) -> bool;
    /// clk_sys still matches `clocks`, at init and reconfiguration
    /// ([`Clocks::sys_current_strict`]).
    fn sys_current_strict(&mut self, clocks: &Clocks) -> bool;
    /// Releases the controller reset `bit` (RESETS CTRL1) and waits for DONE1.
    fn release_reset(&mut self, bit: u32) -> Result<(), Error>;
}
struct Mmio {
    base: usize,
}
impl Mmio {
    const fn new(n: u8) -> Self {
        Self { base: base(n) }
    }
}
impl Io for Mmio {
    #[inline(always)]
    fn read(&mut self, offset: usize) -> u32 {
        read(self.base + offset)
    }
    #[inline(always)]
    fn write(&mut self, offset: usize, value: u32) {
        if offset == DR {
            // FIFO pushes need no barrier each; the next register read orders them.
            unsafe { ((self.base + offset) as *mut u32).write_volatile(value) }
        } else {
            write(self.base + offset, value);
        }
    }
    #[inline(always)]
    fn now(&mut self) -> Instant {
        Timer::new().now()
    }
    #[inline(always)]
    fn now_low(&mut self) -> u32 {
        Timer::new().now_32().ticks()
    }
    #[inline(always)]
    fn sys_current(&mut self, clocks: &Clocks) -> bool {
        clocks.sys_current()
    }
    fn sys_current_strict(&mut self, clocks: &Clocks) -> bool {
        clocks.sys_current_strict()
    }
    fn release_reset(&mut self, bit: u32) -> Result<(), Error> {
        write(RESETS_CLR1, bit);
        crate::poll(RESETS_DONE1, bit, bit).map_err(|_| Error::Timeout)
    }
}

const FIFO_ERRORS: u32 = int::RXOI | int::RXUI | int::TXOI;
const ERRORS: u32 = FIFO_ERRORS | int::MSTI;
const IDLE: u32 = sr::BUSY | sr::TFE;

fn error(status: u32) -> Result<(), Error> {
    if status & FIFO_ERRORS != 0 {
        Err(Error::Overrun)
    } else if status & int::MSTI != 0 {
        Err(Error::ChipSelectFault)
    } else {
        Ok(())
    }
}

/// Frames beyond `write` send zero and frames beyond `read` are discarded. Each
/// step drains and fills at most one FIFO, with at most `depth` frames in flight, so neither FIFO can overflow.
struct Exchange<'a> {
    read: Rx<'a>,
    write: Tx<'a>,
    sent: usize,
    received: usize,
    len: usize,
    mask: u32,
    depth: usize,
}

impl<'a> Exchange<'a> {
    fn new(read: Rx<'a>, write: Tx<'a>, bits: u8, depth: usize) -> Self {
        let len = read.len().max(write.len());
        Self {
            read,
            write,
            sent: 0,
            received: 0,
            len,
            mask: word_mask(bits),
            depth,
        }
    }

    #[rp1_rt::isram]
    fn step(&mut self, io: &mut impl Io, can_send: bool) -> Result<bool, Error> {
        error(io.read(RISR))?;
        let ready = (io.read(RXFLR) as usize).min(self.sent - self.received);
        for _ in 0..ready {
            self.read.set(self.received, io.read(DR) & self.mask);
            self.received += 1;
        }
        if can_send {
            let room = (self.depth - (self.sent - self.received)).min(self.len - self.sent);
            for _ in 0..room {
                io.write(DR, self.write.get(self.sent) & self.mask);
                self.sent += 1;
            }
        }
        error(io.read(RISR))?;
        Ok(self.received == self.len)
    }
}

fn word_mask(bits: u8) -> u32 {
    u32::MAX >> (32 - u32::from(bits))
}

fn poll_exchange(io: &mut impl Io, mut exchange: Exchange<'_>, end: Deadline) -> Result<(), Error> {
    if exchange.len == 0 {
        return Ok(());
    }
    let mut remaining = end.check(io.now());
    remaining?;
    loop {
        if exchange.step(io, remaining.is_ok())? {
            return Ok(());
        }
        remaining?;
        remaining = end.check(io.now());
    }
}

/// The check each public bus call makes once, before any FIFO access: `Precondition` if
/// clk_sys no longer matches `clocks` (SCLK would not be [`Spi::frequency`]).
#[inline(always)]
fn admit(io: &mut impl Io, clocks: &Clocks) -> Result<(), Error> {
    if io.sys_current(clocks) {
        Ok(())
    } else {
        Err(Error::Precondition)
    }
}

fn run_polling(io: &mut impl Io, transfer: Exchange<'_>, end: Deadline) -> Result<(), Error> {
    if transfer.len == 0 {
        return Ok(());
    }
    end.check(io.now())?;
    // The polling loop owns the FIFO and preserves the raw mask on success.
    let interrupts = io.read(IMR);
    io.write(IMR, 0);
    let result = poll_exchange(io, transfer, end);
    if result.is_ok() {
        io.write(IMR, interrupts);
    }
    result
}

fn default_deadline(config: WaitConfig) -> Result<Deadline, Error> {
    Deadline::after(config.timeout_or(Duration::from_ticks(TIMEOUT_US))).map_err(Error::from)
}

#[inline(never)]
fn poll_idle(io: &mut impl Io, end: Deadline) -> Result<(), Error> {
    loop {
        let remaining = end.check(io.now());
        error(io.read(RISR))?;
        if io.read(SR) & IDLE == sr::TFE {
            return Ok(());
        }
        remaining?;
    }
}

/// Masks the interrupts and disables the controller. A frame on the wire may be truncated.
fn stop(io: &mut impl Io) -> Result<(), Error> {
    mask_interrupts(io);
    io.write(SSIENR, 0);
    if io.read(SSIENR) & 1 == 0 && io.read(SR) & sr::BUSY == 0 {
        Ok(())
    } else {
        Err(Error::Busy)
    }
}

/// A new CTRLR0 / BAUDR: the strict clk_sys check first (`Precondition`, nothing written),
/// then wait for idle and set the controller up again.
fn reconfigure(
    io: &mut impl Io,
    clocks: &Clocks,
    cr0: u32,
    div: u32,
    rsd: u32,
) -> Result<(), Error> {
    if !io.sys_current_strict(clocks) {
        return Err(Error::Precondition);
    }
    let end = Deadline::at(io.now() + Duration::from_ticks(TIMEOUT_US));
    poll_idle(io, end)?;
    setup(io, cr0, div, rsd);
    Ok(())
}

impl<const N: u8, P: CompletePins<N>, I> Spi<N, P, Controller, I> {
    fn io(&self) -> Mmio {
        Mmio::new(N)
    }
    /// Selects the synchronous polling engine.
    pub fn polling(&mut self) -> Polling<'_, N, P, I> {
        Polling {
            spi: self,
            config: WaitConfig::new(),
        }
    }

    /// SCLK actually used: clk_sys / BAUDR.
    pub fn frequency(&self) -> fugit::HertzU32 {
        fugit::HertzU32::from_raw(self.clocks.sys().raw() / self.div)
    }
    /// BAUDR divider in use.
    pub fn divider(&self) -> u32 {
        self.div
    }
    /// Frame size, 4..=16 (4..=32 with [`Spi::has_32_bit_frames`]). `SpiBus<W>` needs
    /// `frame_bits <= W::BITS`.
    pub fn frame_bits(&self) -> u8 {
        self.bits
    }
    /// FIFO depth in frames (detected at init).
    pub fn fifo_depth(&self) -> usize {
        usize::from(self.depth)
    }
    pub fn has_32_bit_frames(&self) -> bool {
        self.dfs32
    }
    /// Once per public call, before any FIFO access.
    fn check(&self, word_bits: u8) -> Result<(), Error> {
        self.check_exchange(word_bits)?;
        admit(&mut self.io(), &self.clocks)
    }

    fn check_exchange(&self, word_bits: u8) -> Result<(), Error> {
        self.check_state(word_bits)?;
        if self.outstanding != 0 {
            return Err(Error::Busy);
        }
        Ok(())
    }

    fn check_state(&self, word_bits: u8) -> Result<(), Error> {
        if self.state != State::Idle {
            return Err(Error::Busy);
        }
        if self.frame_bits() > word_bits {
            return Err(Error::Invalid);
        }
        Ok(())
    }

    /// One bounded FIFO step. While the transfer exists, it owns both buffers.
    /// Neither the IRQ handler nor DMA sees the buffers.
    pub fn try_start_transfer<'a, W: Word>(
        &'a mut self,
        read: &'a mut [W],
        write: &'a [W],
    ) -> Result<Transfer<'a, N, P, I>, Error> {
        self.start_with(&mut self.io(), read, write)
    }

    fn start_with<'a, W: Word>(
        &'a mut self,
        io: &mut impl Io,
        read: &'a mut [W],
        write: &'a [W],
    ) -> Result<Transfer<'a, N, P, I>, Error> {
        self.check_exchange(W::BITS)?;
        admit(io, &self.clocks)?;
        mask_interrupts(io);
        self.state = State::Transfer;
        let exchange = Exchange::new(W::rx(read), W::tx(write), self.bits, self.fifo_depth());
        let mut transfer = Transfer {
            spi: self,
            exchange,
            report: None,
        };
        transfer.complete_with(io)?;
        Ok(transfer)
    }

    /// Queue one frame, or return `WouldBlock` if either FIFO has no capacity.
    pub fn try_write_word<W: Word>(&mut self, word: W) -> Result<Try<()>, Error> {
        self.check_state(W::BITS)?;
        let mut io = self.io();
        admit(&mut io, &self.clocks)?;
        self.check_error(&mut io)?;
        if self.outstanding == self.depth || io.read(SR) & sr::TFNF == 0 {
            return Ok(Try::WouldBlock);
        }
        io.write(DR, word.to_u32() & word_mask(self.bits));
        self.outstanding += 1;
        Ok(Try::Ready(()))
    }

    /// Read one frame from a preceding [`Self::try_write_word`].
    pub fn try_read_word<W: Word>(&mut self) -> Result<Try<W>, Error> {
        self.check_state(W::BITS)?;
        let mut io = self.io();
        admit(&mut io, &self.clocks)?;
        self.check_error(&mut io)?;
        if self.outstanding == 0 || io.read(SR) & sr::RFNE == 0 {
            return Ok(Try::WouldBlock);
        }
        let word = W::from_u32(io.read(DR) & word_mask(self.bits));
        self.outstanding -= 1;
        Ok(Try::Ready(word))
    }

    fn check_error(&mut self, io: &mut impl Io) -> Result<(), Error> {
        error(io.read(RISR)).inspect_err(|_| {
            let _ = self.cancel_with(io);
        })
    }

    /// Explicitly poll for all frames under one absolute deadline.
    pub fn transfer_polling<W: Word>(
        &mut self,
        read: &mut [W],
        write: &[W],
        end: Deadline,
    ) -> Result<(), Error> {
        self.polling().transfer_until(read, write, end)
    }

    /// Explicitly poll for physical wire idle under the supplied deadline.
    pub fn flush_polling(&mut self, end: Deadline) -> Result<(), Error> {
        self.polling().flush_until(end)
    }

    /// Clears the overflow, underflow and multi-master interrupts. TXEI and RXFI follow the
    /// FIFO levels instead.
    pub fn clear_interrupt(&mut self) {
        if self.state == State::Idle {
            self.io().read(ICR);
        }
    }
}
impl<const N: u8, P: CompletePins<N>> Spi<N, P> {
    /// Unmasks the [`int`] bits in `mask` and opens the RP1 wrapper
    /// that routes the controller interrupt to the cores; the NVIC line comes from the generated
    /// `irq` resources. Polling transfers mask every interrupt while they run and restore the
    /// mask afterwards.
    ///
    /// The masking is per exchange: between the operations of a `Device` transaction a handler
    /// can run with CS asserted. Mixing interrupt-driven and polling use of one controller is
    /// unsupported.
    pub fn enable_interrupt(&mut self, mask: u32) {
        if self.state == State::Idle {
            enable_interrupt(&mut self.io(), mask);
        }
    }
    pub fn disable_interrupt(&mut self, mask: u32) {
        if self.state == State::Idle {
            let mut io = self.io();
            let masked = io.read(IMR) & !mask;
            io.write(IMR, masked);
        }
    }
    /// Pending unmasked interrupts (ISR).
    pub fn interrupt_status(&self) -> u32 {
        if self.state != State::Idle {
            0
        } else {
            self.io().read(ISR)
        }
    }
}

impl<const N: u8, P: CompletePins<N>, I> Spi<N, P, Controller, I> {
    /// Mask interrupts and request stop. Unsent and unread FIFO frames are discarded.
    /// Disabling can truncate a frame on the wire. On success, SCK has stopped toggling.
    /// Collect the parts with [`Self::try_free`] once stopped.
    pub fn try_cancel(&mut self) -> Result<(), Error> {
        self.cancel_with(&mut self.io())
    }

    fn cancel_with(&mut self, io: &mut impl Io) -> Result<(), Error> {
        self.state = State::Stopping;
        self.outstanding = 0;
        stop(io)
    }

    /// Restore the original configuration once stop is confirmed. A cancelled
    /// transfer may have sent a prefix. RX and TX FIFOs are discarded on stop.
    pub fn try_recover(&mut self) -> Result<(), Error> {
        self.recover_with(&mut self.io())
    }

    fn recover_with(&mut self, io: &mut impl Io) -> Result<(), Error> {
        if self.state == State::Stopping {
            stop(io)?;
            if !io.sys_current_strict(&self.clocks) {
                return Err(Error::Precondition);
            }
            setup(io, self.cr0, self.div, self.rsd);
            self.state = State::Idle;
        }
        Ok(())
    }
}

impl<const N: u8, const SCK: u8, const MOSI: u8, const MISO: u8, I>
    Spi<N, PinTuple<SCK, MOSI, MISO>, Controller, I>
where
    Pin<SCK>: Sck<N>,
    Pin<MOSI>: Mosi<N>,
    Pin<MISO>: Miso<N>,
{
    /// Return the parts after checking wire idle and disabling the controller.
    /// If the bus is still busy, [`Error::Busy`] returns the unchanged driver
    /// (or [`Error::Timeout`] from [`Self::free_polling`]). Retry or call
    /// [`Self::try_cancel`] before trying again. If stop has already been
    /// requested, `try_free` retries disable until it can confirm stop.
    /// The pins stay muxed and reset stays released. The managed IRQ remains
    /// reserved for these parts. When using a raw IRQ, disable and unpend it
    /// first or ensure its handler accepts the controller being gone.
    pub fn try_free(
        self,
    ) -> Result<Parts<N, PinTuple<SCK, MOSI, MISO>, I>, crate::FreeError<Self, Error>> {
        self.free_with(false)
    }

    /// Poll for wire idle for up to 100 ms before disabling. On failure the driver is returned.
    pub fn free_polling(
        self,
    ) -> Result<Parts<N, PinTuple<SCK, MOSI, MISO>, I>, crate::FreeError<Self, Error>> {
        if let Err(error) = poll_context() {
            return Err(crate::FreeError {
                driver: self,
                error,
            });
        }
        self.free_with(true)
    }

    fn free_with(
        mut self,
        polling: bool,
    ) -> Result<Parts<N, PinTuple<SCK, MOSI, MISO>, I>, crate::FreeError<Self, Error>> {
        let idle = if self.state == State::Transfer || self.outstanding != 0 {
            Err(Error::Busy)
        } else if self.state == State::Stopping {
            Ok(())
        } else if polling {
            default_deadline(WaitConfig::new()).and_then(|end| poll_idle(&mut self.io(), end))
        } else if self.io().read(SR) & (sr::BUSY | sr::TFE) == sr::TFE {
            Ok(())
        } else {
            Err(Error::Busy)
        };
        if let Err(error) = idle.and_then(|()| self.try_cancel()) {
            return Err(crate::FreeError {
                driver: self,
                error,
            });
        }
        let (sck, mosi, miso) = self.pins;
        Ok(Parts {
            token: Token {
                _private: PhantomData,
            },
            pins: Pins::from_required((sck.release(), mosi.release(), miso.release())),
            irq: self.irq,
        })
    }
}

#[cfg(feature = "eh-nb")]
impl<const N: u8, P: CompletePins<N>, I> ErrorType for Spi<N, P, Controller, I> {
    type Error = Error;
}

#[cfg(feature = "eh-nb")]
impl<W: Word, const N: u8, P: CompletePins<N>, I> embedded_hal_nb::spi::FullDuplex<W>
    for Spi<N, P, Controller, I>
{
    fn read(&mut self) -> nb::Result<W, Error> {
        match self.try_read_word().map_err(nb::Error::Other)? {
            Try::Ready(word) => Ok(word),
            Try::WouldBlock => Err(nb::Error::WouldBlock),
        }
    }

    fn write(&mut self, word: W) -> nb::Result<(), Error> {
        self.check_state(W::BITS).map_err(nb::Error::Other)?;
        // The trait associates a read with the immediately preceding write.
        if self.outstanding != 0 {
            return Err(nb::Error::WouldBlock);
        }
        match self.try_write_word(word).map_err(nb::Error::Other)? {
            Try::Ready(()) => Ok(()),
            Try::WouldBlock => Err(nb::Error::WouldBlock),
        }
    }
}

/// Progress of a CPU-driven transfer. A failed or cancelled transfer may have
/// sent part of its transmit buffer and updated part of its receive buffer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TransferReport {
    pub sent: usize,
    pub received: usize,
}

#[must_use = "dropping the handle requests stop"]
pub struct Transfer<'a, const N: u8, P: CompletePins<N>, I = NoIrq> {
    spi: &'a mut Spi<N, P, Controller, I>,
    exchange: Exchange<'a>,
    report: Option<TransferReport>,
}

impl<const N: u8, P: CompletePins<N>, I> Transfer<'_, N, P, I> {
    pub fn progress(&self) -> TransferReport {
        TransferReport {
            sent: self.exchange.sent,
            received: self.exchange.received,
        }
    }

    /// Service at most one FIFO and return completion when all RX is collected.
    pub fn try_complete(&mut self) -> Result<Try<TransferReport>, Error> {
        self.complete_with(&mut self.spi.io())
    }

    fn complete_with(&mut self, io: &mut impl Io) -> Result<Try<TransferReport>, Error> {
        if let Some(report) = self.report {
            return Ok(Try::Ready(report));
        }
        if self.spi.state == State::Stopping {
            return Err(Error::Busy);
        }
        match self.exchange.step(io, true) {
            Ok(true) => {
                let report = self.progress();
                self.report = Some(report);
                self.spi.state = State::Idle;
                Ok(Try::Ready(report))
            }
            Ok(false) => Ok(Try::WouldBlock),
            Err(error) => {
                let _ = self.spi.cancel_with(io);
                Err(error)
            }
        }
    }

    pub fn try_cancel(&mut self) -> Result<(), Error> {
        if self.report.is_some() {
            Ok(())
        } else {
            self.spi.try_cancel()
        }
    }
}

impl<const N: u8, P: CompletePins<N>, I> Drop for Transfer<'_, N, P, I> {
    fn drop(&mut self) {
        if self.spi.state == State::Transfer {
            let _ = self.spi.try_cancel();
        }
    }
}

fn poll_context() -> Result<(), Error> {
    ThreadGuard::new()
        .map(|_| ())
        .map_err(timer::Error::from)
        .map_err(Error::from)
}

fn in_place<W: Word>(
    words: &mut [W],
    mut transfer: impl FnMut(&mut [W], &[W]) -> Result<(), Error>,
) -> Result<(), Error> {
    let mut tx = [W::from_u32(0); 32];
    // One admission and deadline for all 32-frame chunks.
    for chunk in words.chunks_mut(tx.len()) {
        let tx = &mut tx[..chunk.len()];
        tx.copy_from_slice(chunk);
        transfer(chunk, tx)?;
    }
    Ok(())
}

// Both adapters expose the same configuration calls. Their reconfiguration
// methods use the selected engine to reach wire idle.
macro_rules! adapter_config {
    ($spi:ident) => {
        /// SCLK actually used.
        pub fn frequency(&self) -> fugit::HertzU32 {
            self.$spi.frequency()
        }

        pub fn divider(&self) -> u32 {
            self.$spi.divider()
        }

        pub fn fifo_depth(&self) -> usize {
            self.$spi.fifo_depth()
        }

        pub fn has_32_bit_frames(&self) -> bool {
            self.$spi.has_32_bit_frames()
        }

        /// Sets SCLK to at most `frequency` and returns the rate used.
        pub fn set_frequency(
            &mut self,
            frequency: fugit::HertzU32,
        ) -> Result<fugit::HertzU32, Error> {
            let div = divider(self.$spi.clocks.sys().raw(), frequency.raw())?;
            self.reconfigure(self.$spi.cr0, div)?;
            Ok(self.$spi.frequency())
        }

        pub fn set_mode(&mut self, mode: Mode) -> Result<(), Error> {
            let cr0 =
                control(mode, self.$spi.bits, self.$spi.dfs32)? | (self.$spi.cr0 & ctrlr0::SRL);
            self.reconfigure(cr0, self.$spi.div)
        }

        pub fn set_frame_bits(&mut self, bits: u8) -> Result<(), Error> {
            let cr0 =
                control(self.$spi.mode(), bits, self.$spi.dfs32)? | (self.$spi.cr0 & ctrlr0::SRL);
            self.reconfigure(cr0, self.$spi.div)?;
            self.$spi.bits = bits;
            Ok(())
        }

        /// Internal loopback feeds transmitted frames back into RX.
        pub fn set_loopback(&mut self, on: bool) -> Result<(), Error> {
            let cr0 = (self.$spi.cr0 & !ctrlr0::SRL) | if on { ctrlr0::SRL } else { 0 };
            self.reconfigure(cr0, self.$spi.div)
        }

        pub fn try_recover(&mut self) -> Result<(), Error> {
            self.$spi.try_recover()
        }
    };
}

impl<const N: u8, P: CompletePins<N>, I> Spi<N, P, Controller, I> {
    fn mode(&self) -> Mode {
        Mode {
            phase: if self.cr0 & ctrlr0::SCPH != 0 {
                Phase::CaptureOnSecondTransition
            } else {
                Phase::CaptureOnFirstTransition
            },
            polarity: if self.cr0 & ctrlr0::SCPOL != 0 {
                Polarity::IdleHigh
            } else {
                Polarity::IdleLow
            },
        }
    }
}

impl<const N: u8, P: CompletePins<N>, I> Polling<'_, N, P, I> {
    // Named field for use by both adapters' shared configuration methods.
    pub fn with_wait_config(mut self, config: WaitConfig) -> Self {
        self.config = config;
        self
    }

    adapter_config!(spi);

    fn reconfigure(&mut self, cr0: u32, div: u32) -> Result<(), Error> {
        self.spi.check_state(u8::MAX)?;
        if self.spi.outstanding != 0 {
            return Err(Error::Busy);
        }
        poll_context()?;
        reconfigure(&mut self.spi.io(), &self.spi.clocks, cr0, div, self.spi.rsd)?;
        self.spi.cr0 = cr0;
        self.spi.div = div;
        Ok(())
    }

    pub fn transfer_until<W: Word>(
        &mut self,
        read: &mut [W],
        write: &[W],
        end: Deadline,
    ) -> Result<(), Error> {
        let ok = self.admit(sealed::Key(()), Some(end))?;
        self.exchange(ok, read, write)
    }

    pub fn flush_until(&mut self, end: Deadline) -> Result<(), Error> {
        self.check_flush(end)?;
        self.flush_admitted(end)
    }

    fn check_flush(&self, end: Deadline) -> Result<(), Error> {
        self.spi.check_state(u8::MAX)?;
        poll_context()?;
        end.check(Timer::new().now())?;
        admit(&mut self.spi.io(), &self.spi.clocks)
    }

    fn flush_admitted(&mut self, end: Deadline) -> Result<(), Error> {
        let mut io = self.spi.io();
        let result = poll_idle(&mut io, end);
        if result.is_err() {
            let _ = self.spi.cancel_with(&mut io);
        }
        result
    }
}

/// The bus side of [`Device`]. Only the HAL's adapters and their mutable
/// references implement it, and admission is required before CS moves.
///
/// ```compile_fail,E0061
/// fn skip_the_check<B: rp1_hal::spi::Bus<u8>>(bus: &mut B, rx: &mut [u8], tx: &[u8]) {
///     let _ = bus.exchange(rx, tx);
/// }
/// ```
///
/// ```compile_fail,E0603
/// let _ = rp1_hal::spi::sealed::Key(());
/// ```
pub trait Bus<W: Word>: sealed::Bus {
    #[doc(hidden)]
    fn admit(
        &mut self,
        key: sealed::Key,
        end: Option<Deadline>,
    ) -> Result<sealed::Admitted<W>, Error>;
    #[doc(hidden)]
    fn exchange(
        &mut self,
        ok: sealed::Admitted<W>,
        read: &mut [W],
        write: &[W],
    ) -> Result<(), Error>;
    #[doc(hidden)]
    fn exchange_in_place(&mut self, ok: sealed::Admitted<W>, words: &mut [W]) -> Result<(), Error>;
    #[doc(hidden)]
    fn wait_idle(&mut self, ok: sealed::Admitted<W>) -> Result<(), Error>;
    #[doc(hidden)]
    fn delay(&mut self, ok: sealed::Admitted<W>, ns: u32) -> Result<(), Error>;
    #[doc(hidden)]
    fn abort(&mut self, ok: sealed::Admitted<W>);
}

fn delay_end(now: Instant, end: Deadline, ns: u32) -> Result<Instant, Error> {
    end.check(now)?;
    if ns == 0 {
        return Ok(now);
    }
    // Round up to microseconds and allow for the current counter phase.
    let ticks = now
        .ticks()
        .checked_add(u64::from(ns.div_ceil(1000)) + 1)
        .ok_or(Error::InvalidDeadline)?;
    Ok(Instant::from_ticks(ticks))
}

impl<W: Word, const N: u8, P: CompletePins<N>, I> Bus<W> for Polling<'_, N, P, I> {
    fn admit(
        &mut self,
        _: sealed::Key,
        end: Option<Deadline>,
    ) -> Result<sealed::Admitted<W>, Error> {
        self.spi.check_state(W::BITS)?;
        poll_context()?;
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
        let io = &mut self.spi.io();
        let transfer = Exchange::new(
            W::rx(read),
            W::tx(write),
            self.spi.bits,
            self.spi.fifo_depth(),
        );
        let result = run_polling(io, transfer, ok.0);
        if result.is_err() {
            let _ = self.spi.cancel_with(io);
        }
        result
    }

    fn exchange_in_place(&mut self, ok: sealed::Admitted<W>, words: &mut [W]) -> Result<(), Error> {
        in_place(words, |read, write| self.exchange(ok, read, write))
    }

    fn wait_idle(&mut self, ok: sealed::Admitted<W>) -> Result<(), Error> {
        self.flush_admitted(ok.0)
    }

    fn delay(&mut self, ok: sealed::Admitted<W>, ns: u32) -> Result<(), Error> {
        let timer = Timer::new();
        let target = delay_end(timer.now(), ok.0, ns)?;
        while timer.now() < target.min(ok.0.instant()) {
            core::hint::spin_loop();
        }
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

impl<W: Word, B: Bus<W> + ?Sized> Bus<W> for &mut B {
    fn admit(
        &mut self,
        key: sealed::Key,
        end: Option<Deadline>,
    ) -> Result<sealed::Admitted<W>, Error> {
        (**self).admit(key, end)
    }
    fn exchange(
        &mut self,
        ok: sealed::Admitted<W>,
        read: &mut [W],
        write: &[W],
    ) -> Result<(), Error> {
        (**self).exchange(ok, read, write)
    }
    fn exchange_in_place(&mut self, ok: sealed::Admitted<W>, words: &mut [W]) -> Result<(), Error> {
        (**self).exchange_in_place(ok, words)
    }
    fn wait_idle(&mut self, ok: sealed::Admitted<W>) -> Result<(), Error> {
        (**self).wait_idle(ok)
    }
    fn delay(&mut self, ok: sealed::Admitted<W>, ns: u32) -> Result<(), Error> {
        (**self).delay(ok, ns)
    }
    fn abort(&mut self, ok: sealed::Admitted<W>) {
        (**self).abort(ok);
    }
}

macro_rules! spi_bus {
    ([$($generic:tt)*], $adapter:ty) => {
        impl<$($generic)*> ErrorType for $adapter {
            type Error = Error;
        }

        impl<W: Word, $($generic)*> SpiBus<W> for $adapter {
            fn read(&mut self, words: &mut [W]) -> Result<(), Error> {
                self.transfer(words, &[])
            }
            fn write(&mut self, words: &[W]) -> Result<(), Error> {
                self.transfer(&mut [], words)
            }
            fn transfer(&mut self, read: &mut [W], write: &[W]) -> Result<(), Error> {
                let ok = self.admit(sealed::Key(()), None)?;
                self.exchange(ok, read, write)
            }
            fn transfer_in_place(&mut self, words: &mut [W]) -> Result<(), Error> {
                let ok = self.admit(sealed::Key(()), None)?;
                self.exchange_in_place(ok, words)
            }
            fn flush(&mut self) -> Result<(), Error> {
                self.flush_until(default_deadline(self.config)?)
            }
        }
    };
}

spi_bus!([const N: u8, P: CompletePins<N>, I], Polling<'_, N, P, I>);

/// A bus with an active-low GPIO chip select for one [`SpiDevice`].
///
/// ```no_run
/// # use rp1_hal::{fugit::RateExtU32, gpio::Pin, spi::{Controller, Device, Error, Irq, Spi}, timer::WaitTimer};
/// use embedded_hal::spi::SpiDevice;
/// # type Spi0 = Spi<0, (Pin<11>, Pin<10>, Pin<9>), Controller, Irq<0, 0>>;
/// fn lend(spi: &mut Spi0, cs: Pin<8>, wait: &mut WaitTimer) -> Result<Pin<8>, Error> {
///     let mut dev = Device::new(spi.blocking(wait), cs);
///     dev.write(&[1u8, 2])?;
///     let (_, cs) = dev.free();
///     spi.blocking(wait).set_frequency(1.MHz())?;
///     // The chip select comes back as `Pin<8>` and can make a new device.
///     let mut dev = Device::new(spi.blocking(wait), cs);
///     dev.write(&[3u8])?;
///     Ok(dev.free().1)
/// }
/// ```
pub struct Device<B, const CS: u8> {
    bus: B,
    cs: Pin<CS, Output>,
}

impl<B, const CS: u8> Device<B, CS> {
    /// Drives `cs` high (deselected) and owns the bus exclusively.
    pub fn new(bus: B, cs: Pin<CS>) -> Self {
        Self {
            bus,
            cs: cs.into_output(true),
        }
    }

    pub fn bus_mut(&mut self) -> &mut B {
        &mut self.bus
    }

    /// Returns the bus and the chip select, which stays driven high (deselected).
    /// The pin is handed back unconfigured, so it can go to [`Device::new`]
    /// again or be reconfigured.
    pub fn free(self) -> (B, Pin<CS>) {
        (self.bus, self.cs.release())
    }

    /// Uses one deadline for all operations, including wire flush and CS delays.
    pub fn transaction_until<W: Word>(
        &mut self,
        operations: &mut [Operation<'_, W>],
        end: Deadline,
    ) -> Result<(), Error>
    where
        B: Bus<W>,
    {
        self.run(operations, Some(end))
    }

    fn run<W: Word>(
        &mut self,
        operations: &mut [Operation<'_, W>],
        end: Option<Deadline>,
    ) -> Result<(), Error>
    where
        B: Bus<W>,
    {
        use embedded_hal::digital::OutputPin;
        let cs = &mut self.cs;
        transaction(
            &mut self.bus,
            |high| {
                let _ = if high { cs.set_high() } else { cs.set_low() };
            },
            operations,
            end,
        )
    }
}

impl<B, const CS: u8> ErrorType for Device<B, CS> {
    type Error = Error;
}

impl<W: Word, B: Bus<W>, const CS: u8> SpiDevice<W> for Device<B, CS> {
    /// Admits once before CS moves, then executes the operations in order.
    /// Delays flush first. Errors stop the bus and release CS.
    fn transaction(&mut self, operations: &mut [Operation<'_, W>]) -> Result<(), Error> {
        self.run(operations, None)
    }
}

fn transaction<W: Word, B: Bus<W>>(
    bus: &mut B,
    cs: impl FnMut(bool),
    operations: &mut [Operation<'_, W>],
    end: Option<Deadline>,
) -> Result<(), Error> {
    let ok = bus.admit(sealed::Key(()), end)?;
    let mut selected = Selected {
        bus,
        cs,
        ok,
        complete: false,
    };
    (selected.cs)(false);
    for op in operations {
        match op {
            Operation::Read(r) => selected.bus.exchange(ok, r, &[]),
            Operation::Write(w) => selected.bus.exchange(ok, &mut [], w),
            Operation::Transfer(r, w) => selected.bus.exchange(ok, r, w),
            Operation::TransferInPlace(w) => selected.bus.exchange_in_place(ok, w),
            Operation::DelayNs(ns) => selected
                .bus
                .wait_idle(ok)
                .and_then(|()| selected.bus.delay(ok, *ns)),
        }?;
    }
    selected.bus.wait_idle(ok)?;
    selected.complete = true;
    Ok(())
}

struct Selected<'a, W: Word, B: Bus<W>, F: FnMut(bool)> {
    bus: &'a mut B,
    cs: F,
    ok: sealed::Admitted<W>,
    complete: bool,
}

impl<W: Word, B: Bus<W>, F: FnMut(bool)> Drop for Selected<'_, W, B, F> {
    fn drop(&mut self) {
        if !self.complete {
            self.bus.abort(self.ok);
        }
        (self.cs)(true);
    }
}

mod irq_wait;

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use fugit::RateExtU32;
    /// Model timer step per `now()`.
    const STEP_US: u32 = 1000;
    /// A bounded wait that started reading the timer at `start` gave up at the first look
    /// past [`TIMEOUT_US`], whatever the model's step.
    fn assert_timed_out(tick: u64, start: u32) {
        let waited = (tick - u64::from(start)) as u32;
        assert!(
            (TIMEOUT_US..=TIMEOUT_US + STEP_US).contains(&waited),
            "{waited}"
        );
    }
    use std::{collections::VecDeque, vec::Vec};
    /// Loopback DW SSI: every RXFLR read shifts the whole TX FIFO into the RX FIFO. After the
    /// last push SR keeps BUSY for `hold_busy` more polls (the last frame still on the wire
    /// although RX already has it).
    pub(super) struct Model {
        tx: VecDeque<u32>,
        rx: VecDeque<u32>,
        depth: usize,
        rx_depth: usize,
        pub(super) tick: u64,
        stalled: bool,
        hold_busy: u32,
        busy_left: u32,
        pub(super) risr: u32,
        pushed: Vec<u32>,
        txftlr: u32,
        rxftlr: u32,
        cr0: u32,
        dfs32: bool,
        writes: Vec<(usize, u32)>,
        /// clk_sys as the clock checks see it.
        sys: Option<u32>,
        /// clk_sys drifted in a way only the strict check sees (CLK_SYS_CTRL, PLL_SYS PWR or
        /// FBDIV_FRAC): the fast check passes, the strict one fails.
        strict_stale: bool,
        /// Fast (per bus call) and strict (init, reconfiguration) clock checks made.
        fast: u32,
        strict: u32,
        /// Reset bits released.
        resets: Vec<u32>,
        pub(super) imr: u32,
        pub(super) wrap: u32,
        stuck_enabled: bool,
    }
    impl Model {
        pub(super) fn new(depth: usize) -> Self {
            Self {
                tx: VecDeque::new(),
                rx: VecDeque::new(),
                depth,
                rx_depth: depth,
                tick: u64::from(u32::MAX - 50),
                stalled: false,
                hold_busy: 0,
                busy_left: 0,
                risr: 0,
                pushed: Vec::new(),
                txftlr: 0,
                rxftlr: 0,
                cr0: 0,
                dfs32: true,
                writes: Vec::new(),
                sys: Some(200_000_000),
                strict_stale: false,
                fast: 0,
                strict: 0,
                resets: Vec::new(),
                imr: 0,
                wrap: 0,
                stuck_enabled: false,
            }
        }
    }
    const CLOCKS: crate::clocks::Clocks = crate::clocks::Clocks::for_test(200_000_000, 50_000_000);
    impl Io for Model {
        fn read(&mut self, offset: usize) -> u32 {
            match offset {
                RXFLR => {
                    if !self.stalled {
                        self.rx.extend(self.tx.drain(..));
                        assert!(self.rx.len() <= self.rx_depth, "RX overflow");
                    }
                    self.rx.len() as u32
                }
                DR => self.rx.pop_front().expect("RX underflow"),
                RISR => self.risr,
                ISR => self.risr & self.imr,
                ICR => core::mem::take(&mut self.risr),
                SSIENR => u32::from(self.stuck_enabled),
                SR => {
                    if self.stalled {
                        sr::BUSY
                    } else if self.busy_left > 0 {
                        self.busy_left -= 1;
                        sr::BUSY | sr::TFE
                    } else {
                        sr::TFE
                    }
                }
                TXFTLR => self.txftlr,
                RXFTLR => self.rxftlr,
                CTRLR0 => self.cr0,
                IMR => self.imr,
                IRQ_WRAP => self.wrap,
                _ => 0,
            }
        }
        fn write(&mut self, offset: usize, value: u32) {
            self.writes.push((offset, value));
            match offset {
                DR => {
                    self.tx.push_back(value);
                    self.pushed.push(value);
                    self.busy_left = self.hold_busy;
                    assert!(self.tx.len() <= self.depth, "TX overflow");
                }
                SSIENR if value == 0 && !self.stuck_enabled => {
                    self.tx.clear();
                    self.rx.clear();
                }
                IMR => self.imr = value,
                IRQ_WRAP => self.wrap = value,
                TXFTLR => self.txftlr = value.min(self.depth as u32 - 1),
                RXFTLR => self.rxftlr = value.min(self.rx_depth as u32 - 1),
                CTRLR0 => {
                    self.cr0 = if self.dfs32 {
                        value & !ctrlr0::DFS_MASK
                    } else {
                        value & 0xffff
                    }
                }
                _ => {}
            }
        }
        fn now(&mut self) -> Instant {
            self.tick += u64::from(STEP_US);
            Instant::from_ticks(self.tick)
        }
        fn sys_current(&mut self, clocks: &Clocks) -> bool {
            self.fast += 1;
            self.sys == Some(clocks.sys().raw())
        }
        fn sys_current_strict(&mut self, clocks: &Clocks) -> bool {
            self.strict += 1;
            !self.strict_stale && self.sys == Some(clocks.sys().raw())
        }
        fn release_reset(&mut self, bit: u32) -> Result<(), Error> {
            self.resets.push(bit);
            Ok(())
        }
    }

    fn end(m: &Model) -> Deadline {
        Deadline::at(Instant::from_ticks(m.tick + u64::from(TIMEOUT_US)))
    }

    fn exchange(
        m: &mut Model,
        read: Rx<'_>,
        write: Tx<'_>,
        mask: u32,
        depth: usize,
    ) -> Result<(), Error> {
        let end = end(m);
        poll_exchange(
            m,
            Exchange::new(read, write, mask.count_ones() as u8, depth),
            end,
        )
    }

    fn wait_idle(m: &mut Model) -> Result<(), Error> {
        let end = end(m);
        poll_idle(m, end)
    }

    /// `free` used to disable at once, cutting a frame still on the wire.
    #[test]
    fn free_waits_for_the_wire_and_refuses_a_busy_controller() {
        let stopped = [(IMR, 0), (IRQ_WRAP, 0x8), (SSIENR, 0)];
        let mut m = Model::new(8);
        (m.imr, m.wrap, m.busy_left) = (int::RXFI, 0x9, 3);
        assert_eq!(wait_idle(&mut m).and_then(|()| stop(&mut m)), Ok(()));
        assert_eq!((m.busy_left, m.writes.as_slice()), (0, &stopped[..]));

        let mut m = Model::new(8);
        (m.imr, m.wrap, m.stalled) = (int::RXFI, 0x9, true);
        assert_eq!(
            wait_idle(&mut m).and_then(|()| stop(&mut m)),
            Err(Error::Timeout)
        );
        assert!(m.writes.is_empty());
        assert_timed_out(m.tick, u32::MAX - 50);
        // `try_cancel` requests stop without waiting for wire idle.
        let before = m.tick;
        assert_eq!(stop(&mut m), Err(Error::Busy));
        assert_eq!((m.tick, m.writes.as_slice()), (before, &stopped[..]));
    }

    #[test]
    fn cancel_gates_interrupts_and_polling_until_release() {
        let mut model = Model::new(8);
        let mut bus = init_model(
            unsafe { Token::new() },
            &mut model,
            pins(),
            Config::new(1.MHz(), embedded_hal::spi::MODE_0),
            &mut false,
        )
        .unwrap();
        model.stalled = true;
        assert_eq!(bus.cancel_with(&mut model), Err(Error::Busy));
        bus.enable_interrupt(int::RXFI);
        bus.disable_interrupt(int::RXFI);
        bus.clear_interrupt();
        assert_eq!(bus.interrupt_status(), 0);
        assert_eq!(bus.polling().write(&[0u8]), Err(Error::Busy));
        assert_eq!(bus.polling().set_frequency(2.MHz()), Err(Error::Busy));

        model.stalled = false;
        assert_eq!(bus.cancel_with(&mut model), Ok(()));
        assert_eq!(bus.polling().write(&[0u8]), Err(Error::Busy));
    }

    #[test]
    fn bounded_transfer_cancel_recover_clears_old_fifo_and_fault() {
        let mut model = Model::new(4);
        let config = Config::new(1.MHz(), embedded_hal::spi::MODE_0);
        let mut bus = init_model(
            unsafe { Token::new() },
            &mut model,
            pins(),
            config,
            &mut false,
        )
        .unwrap();
        let mut rx = [0u8; 10];
        let mut first = bus.start_with(&mut model, &mut rx, &[9; 10]).unwrap();
        assert_eq!(
            first.progress(),
            TransferReport {
                sent: 4,
                received: 0
            }
        );
        model.stalled = true;
        model.risr = int::RXOI;
        assert_eq!(first.complete_with(&mut model), Err(Error::Overrun));
        drop(first);
        assert_eq!(bus.recover_with(&mut model), Err(Error::Busy));
        assert_eq!(bus.check_state(8), Err(Error::Busy));
        model.stalled = false;
        bus.recover_with(&mut model).unwrap();
        let mut next = bus.start_with(&mut model, &mut rx, &[3; 10]).unwrap();
        assert_eq!(next.complete_with(&mut model), Ok(Try::WouldBlock));
        assert_eq!(next.complete_with(&mut model), Ok(Try::WouldBlock));
        assert_eq!(
            next.complete_with(&mut model),
            Ok(Try::Ready(TransferReport {
                sent: 10,
                received: 10
            }))
        );
        assert_eq!(
            next.complete_with(&mut model),
            Ok(Try::Ready(next.progress()))
        );
        drop(next);
        assert_eq!(rx, [3; 10]);
    }

    #[test]
    fn expired_or_unreachable_poll_deadline_never_touches_fifo() {
        for (delta, error) in [(0, Error::Timeout), (1u64 << 32, Error::InvalidDeadline)] {
            let mut io = Model::new(4);
            let now = io.now().ticks();
            let end = Deadline::at(Instant::from_ticks(now + delta));
            let mut received = [0; 8];
            let exchange = Exchange::new(Rx::U8(&mut received), Tx::U8(&[0; 8]), 8, 4);
            assert_eq!(run_polling(&mut io, exchange, end), Err(error));
            assert!(io.writes.is_empty());
        }
    }

    #[test]
    fn dividers_round_down_the_rate() {
        let rate = |hz| 200_000_000 / divider(200_000_000, hz).unwrap();
        assert_eq!(divider(200_000_000, 1_000_000), Ok(200));
        assert_eq!(divider(200_000_000, 10_000_000), Ok(20));
        assert_eq!(divider(200_000_000, 30_000_000), Ok(8));
        assert_eq!(rate(30_000_000), 25_000_000);
        assert_eq!(divider(200_000_000, 200_000_000), Ok(2));
        assert_eq!(divider(200_000_000, 3_000_000), Ok(68));
        assert!(rate(3_000_000) <= 3_000_000);
        assert_eq!(divider(200_000_000, 3_052), Ok(65532));
        assert_eq!(divider(200_000_000, 3_051), Err(Error::Invalid));
        assert_eq!(divider(200_000_000, 0), Err(Error::Invalid));
        assert_eq!(divider(50_000_000, 1_000_000), Ok(50));
    }

    #[test]
    fn control_word_follows_the_pssi_layout() {
        use embedded_hal::spi::{MODE_0, MODE_3};
        assert_eq!(control(MODE_0, 8, false), Ok(7));
        assert_eq!(control(MODE_3, 16, false), Ok(15 | (1 << 6) | (1 << 7)));
        assert_eq!(control(MODE_0, 32, true), Ok(31 << 16));
        assert_eq!(control(MODE_0, 17, false), Err(Error::Invalid));
        assert_eq!(control(MODE_0, 3, true), Err(Error::Invalid));
        // SRL is CTRLR0 bit 11 (Linux DW_PSSI_CTRLR0_SRL).
        assert_eq!(ctrlr0::SRL, 0x800);
        assert_eq!(reset_bit(0), 1 << 10);
        assert_eq!(reset_bit(5), 1 << 15);
        assert_eq!(RESETS_CLR1, 0x4001_7004);
        assert_eq!(RESETS_DONE1, 0x4001_401c);
        assert_eq!(base(5), 0x4006_4000);
    }

    #[test]
    fn probe_detects_depth_and_frame_field() {
        let mut m = Model::new(64);
        assert_eq!(probe(&mut m), Some((64, true)));
        let mut m = Model::new(8);
        m.dfs32 = false;
        assert_eq!(probe(&mut m), Some((8, false)));
        let mut m = Model::new(1);
        assert_eq!(probe(&mut m), None);
        // Review 6 #6: a shallower RX FIFO bounds the frames in flight.
        let mut m = Model::new(64);
        m.rx_depth = 16;
        assert_eq!(probe(&mut m), Some((16, true)));
        let mut m = Model::new(8);
        m.rx_depth = 32;
        assert_eq!(probe(&mut m), Some((8, true)));
        // With that bound a 16-deep RX FIFO never overflows behind a 64-deep TX FIFO.
        let mut m = Model::new(64);
        m.rx_depth = 16;
        let (depth, _) = probe(&mut m).unwrap();
        let tx: Vec<u8> = (0..100).collect();
        let mut rx = [0u8; 100];
        exchange(
            &mut m,
            Rx::U8(&mut rx),
            Tx::U8(&tx),
            0xff,
            usize::from(depth),
        )
        .unwrap();
        assert_eq!(rx[..], tx[..]);
    }

    #[test]
    fn exchange_bounds_frames_in_flight_and_pads() {
        let mut m = Model::new(4);
        let tx: Vec<u16> = (0..37).map(|i| 0xf000 | i).collect();
        let mut rx = [0u16; 40];
        exchange(&mut m, Rx::U16(&mut rx), Tx::U16(&tx), 0xfff, 4).unwrap();
        for i in 0..37 {
            assert_eq!(rx[i], tx[i] & 0xfff);
        }
        assert_eq!(rx[37..], [0, 0, 0]);
        assert_eq!(m.pushed.len(), 40);
        // Longer write than read: received frames beyond `read` are drained and dropped.
        let mut m = Model::new(2);
        let mut rx = [0u8; 1];
        exchange(&mut m, Rx::U8(&mut rx), Tx::U8(&[1, 2, 3, 4, 5]), 0xff, 2).unwrap();
        assert_eq!(rx, [1]);
        assert!(m.rx.is_empty() && m.tx.is_empty());
        let mut m = Model::new(2);
        exchange(&mut m, Rx::U32(&mut []), Tx::U32(&[]), 0xff, 2).unwrap();
        assert!(m.pushed.is_empty());
    }

    #[test]
    fn stalls_and_overruns_are_errors() {
        let mut m = Model::new(4);
        m.stalled = true;
        assert_eq!(
            exchange(&mut m, Rx::U8(&mut [0; 8]), Tx::U8(&[0; 8]), 0xff, 4),
            Err(Error::Timeout)
        );
        assert_eq!(m.pushed.len(), 4);
        assert_timed_out(m.tick, u32::MAX - 50);
        let mut m = Model::new(4);
        m.risr = int::RXOI;
        assert_eq!(
            exchange(&mut m, Rx::U8(&mut [0; 8]), Tx::U8(&[0; 8]), 0xff, 4),
            Err(Error::Overrun)
        );
        let mut m = Model::new(4);
        m.stalled = true;
        assert_eq!(wait_idle(&mut m), Err(Error::Timeout));
        m.stalled = false;
        assert_eq!(wait_idle(&mut m), Ok(()));
    }

    #[test]
    fn setup_selects_cs0_with_dma_off() {
        let mut m = Model::new(4);
        setup(&mut m, 7, 20, 3);
        assert_eq!(
            m.writes,
            [
                (SSIENR, 0),
                (DMACR, 0),
                (CTRLR0, 7),
                (BAUDR, 20),
                (RX_SAMPLE_DLY, 3),
                (SER, 1),
                (SSIENR, 1)
            ]
        );
        assert_eq!(RX_SAMPLE_DLY, 0xf0);
        assert_eq!(
            Config::new(1.MHz(), embedded_hal::spi::MODE_0).rx_sample_dly,
            0
        );
        use embedded_hal::spi::Error as _;
        assert_eq!(Error::Overrun.kind(), ErrorKind::Overrun);
    }

    /// Review 6 (Codex #1): clk_sys elsewhere (e.g. XOSC) or not decodable: `Precondition`
    /// from the admission, before DR or any other register is touched.
    #[test]
    fn stale_sys_clock_fails_before_any_fifo_access() {
        for sys in [Some(50_000_000), None] {
            let mut m = Model::new(4);
            m.sys = sys;
            assert_eq!(admit(&mut m, &CLOCKS), Err(Error::Precondition));
            assert!(m.writes.is_empty() && m.pushed.is_empty(), "{sys:?}");
        }
        let mut m = Model::new(4);
        assert_eq!(admit(&mut m, &CLOCKS), Ok(()));
        let mut rx = [0; 4];
        let deadline = end(&m);
        run_polling(
            &mut m,
            Exchange::new(Rx::U16(&mut rx), Tx::U16(&[1, 2, 3, 0x1ff]), 8, 4),
            deadline,
        )
        .unwrap();
        assert_eq!(rx, [1, 2, 3, 0xff]);
        // An exchange error retires the source. Explicit recovery restores the configuration.
        let mut m = Model::new(4);
        let config = Config::new(10.MHz(), embedded_hal::spi::MODE_0).with_rx_sample_dly(2);
        let mut spi =
            init_model(unsafe { Token::new() }, &mut m, pins(), config, &mut false).unwrap();
        m.risr = int::RXOI;
        let end = end(&m);
        assert_eq!(
            run_polling(
                &mut m,
                Exchange::new(Rx::U8(&mut [0; 2]), Tx::U8(&[1, 2]), 8, 4),
                end
            ),
            Err(Error::Overrun)
        );
        spi.cancel_with(&mut m).unwrap();
        assert_eq!(spi.check_state(8), Err(Error::Busy));
        spi.recover_with(&mut m).unwrap();
        assert_eq!(spi.check_state(8), Ok(()));
        assert!(
            m.writes
                .ends_with(&[(RX_SAMPLE_DLY, 2), (SER, 1), (SSIENR, 1)])
        );
        assert_eq!(m.risr, 0);
    }

    #[test]
    fn interrupts_are_off_after_init_and_during_transfers() {
        let interrupt_writes = |m: &Model| -> Vec<(usize, u32)> {
            let writes = m.writes.iter().copied();
            writes
                .filter(|(o, _)| *o == IMR || *o == IRQ_WRAP)
                .collect()
        };
        // Wrapper bit 3 stands for bits the driver must keep.
        let mut m = Model::new(4);
        (m.imr, m.wrap) = (0x3f, 0x9);
        let config = Config::new(10.MHz(), embedded_hal::spi::MODE_0);
        let mut spi =
            init_model(unsafe { Token::new() }, &mut m, pins(), config, &mut false).expect("init");
        assert_eq!((m.imr, m.wrap), (0, 0x8));
        m.writes.clear();
        enable_interrupt(&mut m, int::RXFI | int::RXOI);
        assert_eq!(
            interrupt_writes(&m),
            [(IMR, int::RXFI | int::RXOI), (IRQ_WRAP, 0x9)]
        );
        let end = end(&m);
        run_polling(
            &mut m,
            Exchange::new(Rx::U8(&mut [0; 2]), Tx::U8(&[1, 2]), 8, 4),
            end,
        )
        .unwrap();
        assert!(interrupt_writes(&m).ends_with(&[(IMR, 0), (IMR, int::RXFI | int::RXOI)]));

        m.risr = int::RXOI;
        m.writes.clear();
        assert_eq!(
            run_polling(
                &mut m,
                Exchange::new(Rx::U8(&mut [0; 2]), Tx::U8(&[1, 2]), 8, 4),
                end,
            ),
            Err(Error::Overrun)
        );
        spi.cancel_with(&mut m).unwrap();
        spi.recover_with(&mut m).unwrap();
        assert_eq!(m.imr, 0);
        assert_eq!(m.wrap, 0x8);
    }

    fn pins() -> PinTuple<11, 10, 9> {
        unsafe { (Pin::new(), Pin::new(), Pin::new()) }
    }
    /// `init_on` over `m` with a route that only records that it ran.
    fn init_model(
        token: Token<0>,
        m: &mut Model,
        pins: PinTuple<11, 10, 9>,
        config: Config,
        routed: &mut bool,
    ) -> Result<Spi<0, PinTuple<11, 10, 9>>, InitError<0, PinTuple<11, 10, 9>>> {
        Parts::new(token, Pins::from_required(pins)).init_on(
            m,
            &CLOCKS,
            config,
            |(sck, mosi, miso)| {
                *routed = true;
                (sck.assume(), mosi.assume(), miso.assume())
            },
        )
    }

    /// Review 7: a stale clk_sys or a configuration no instance takes fails before the reset
    /// is released, without a register write and without routing the pins; the token and the
    /// pins come back and a retry with them succeeds.
    #[test]
    fn init_failure_leaves_reset_and_pinmux_untouched_and_retries() {
        let config = Config::new(10.MHz(), embedded_hal::spi::MODE_0);
        let mut token = unsafe { Token::<0>::new() };
        let mut pins = pins();
        for (sys, config, want) in [
            (Some(50_000_000), config, Error::Precondition),
            (None, config, Error::Precondition),
            (
                Some(200_000_000),
                config.with_frequency(0.Hz()),
                Error::Invalid,
            ),
            (
                Some(200_000_000),
                config.with_frequency(3_051.Hz()),
                Error::Invalid,
            ),
            (Some(200_000_000), config.with_frame_bits(3), Error::Invalid),
            (
                Some(200_000_000),
                config.with_frame_bits(33),
                Error::Invalid,
            ),
        ] {
            let mut m = Model::new(64);
            m.sys = sys;
            let mut routed = false;
            let e = init_model(token, &mut m, pins, config, &mut routed)
                .err()
                .expect("init must fail");
            let error = *e.error();
            let (t, pins_back) = e.try_into_parts().unwrap().into_parts();
            assert_eq!(error, want, "{sys:?} {config:?}");
            assert!(m.resets.is_empty() && m.writes.is_empty() && !routed);
            (token, pins) = (t, pins_back.into_pins());
        }
        // A 16-bit instance asked for 32-bit frames: the reset is released and the block
        // probed, but the pins are still not routed; the retry with 16 bits succeeds.
        let mut m = Model::new(8);
        m.dfs32 = false;
        let mut routed = false;
        let e = init_model(token, &mut m, pins, config.with_frame_bits(32), &mut routed)
            .err()
            .expect("32 bits on a 16-bit instance");
        assert_eq!(*e.error(), Error::Invalid);
        assert_eq!((m.resets.as_slice(), routed), (&[reset_bit(0)][..], false));
        let config = config.with_frame_bits(16).with_rx_sample_dly(3);
        let (token, pins) = e.try_into_parts().unwrap().into_parts();
        let spi = init_model(token, &mut m, pins.into_pins(), config, &mut routed).expect("retry");
        assert!(routed);
        assert_eq!(
            (spi.fifo_depth(), spi.has_32_bit_frames(), spi.frame_bits()),
            (8, false, 16)
        );
        assert_eq!((spi.divider(), spi.frequency().raw()), (20, 10_000_000));
        assert!(m.writes.ends_with(&[
            (CTRLR0, 15),
            (BAUDR, 20),
            (RX_SAMPLE_DLY, 3),
            (SER, 1),
            (SSIENR, 1)
        ]));
        // One strict clock check per init, never the fast one.
        assert_eq!((m.strict, m.fast), (2, 0));
    }

    #[test]
    fn init_with_unconfirmed_disable_keeps_the_pins() {
        let config = Config::new(1.MHz(), embedded_hal::spi::MODE_0);
        let mut model = Model::new(8);
        model.stuck_enabled = true;
        let mut routed = false;
        let error = init_model(
            unsafe { Token::new() },
            &mut model,
            pins(),
            config,
            &mut routed,
        )
        .err()
        .expect("disable is stuck");
        assert_eq!(*error.error(), Error::Precondition);
        assert!(!routed);
        assert!(
            model
                .writes
                .iter()
                .all(|(reg, _)| matches!(*reg, IMR | IRQ_WRAP | SSIENR))
        );
        let error = error.try_into_parts().err().expect("pins are still owned");
        let error = error
            .recover(|| false)
            .err()
            .expect("reset is not confirmed");
        assert!(error.recover(|| true).is_ok());
    }

    /// Review 8: init and reconfiguration use the strict check only. clk_sys drifted in a way
    /// only it sees (the fast check passes): init fails before the reset is released, without
    /// a register write or pin routing, and a reconfiguration writes nothing.
    #[test]
    fn init_and_reconfigure_use_the_strict_check() {
        let config = Config::new(10.MHz(), embedded_hal::spi::MODE_0);
        let mut m = Model::new(64);
        m.strict_stale = true;
        assert!(m.sys_current(&CLOCKS));
        m.fast = 0;
        let mut routed = false;
        let e = init_model(unsafe { Token::new() }, &mut m, pins(), config, &mut routed)
            .err()
            .expect("strict check must fail");
        assert_eq!(*e.error(), Error::Precondition);
        assert!(m.resets.is_empty() && m.writes.is_empty() && !routed);
        assert_eq!((m.strict, m.fast), (1, 0));

        let mut m = Model::new(64);
        m.strict_stale = true;
        assert_eq!(
            reconfigure(&mut m, &CLOCKS, 15, 20, 0),
            Err(Error::Precondition)
        );
        assert!(m.writes.is_empty());
        assert_eq!((m.strict, m.fast), (1, 0));
        m.strict_stale = false;
        assert_eq!(reconfigure(&mut m, &CLOCKS, 15, 20, 3), Ok(()));
        assert!(m.writes.ends_with(&[
            (CTRLR0, 15),
            (BAUDR, 20),
            (RX_SAMPLE_DLY, 3),
            (SER, 1),
            (SSIENR, 1)
        ]));
        assert_eq!((m.strict, m.fast), (2, 0));
    }

    /// The `Bus` and `SpiBus` polling impls reject words
    /// narrower than the frame with `Invalid` before any register access and before CS moves.
    /// (On the host the controller is not mapped: any access would fault.)
    #[test]
    fn spi_bus_rejects_narrow_words_before_cs() {
        let mut m = Model::new(64);
        let config = Config::new(10.MHz(), embedded_hal::spi::MODE_0).with_frame_bits(16);
        let mut spi =
            init_model(unsafe { Token::new() }, &mut m, pins(), config, &mut false).expect("init");
        let mut spi = spi.polling();
        let log = core::cell::RefCell::new(Vec::new());
        let mut cs = |high| log.borrow_mut().push(high);
        let mut ops = [Operation::<u8>::Write(&[1, 2]), Operation::DelayNs(10)];
        assert_eq!(
            transaction(&mut spi, &mut cs, &mut ops, None),
            Err(Error::Invalid)
        );
        assert_eq!(
            transaction(&mut &mut spi, &mut cs, &mut ops, None),
            Err(Error::Invalid)
        );
        assert!(log.borrow().is_empty(), "CS moved");
        assert_eq!(SpiBus::<u8>::write(&mut spi, &[1]), Err(Error::Invalid));
        assert_eq!(
            SpiBus::<u8>::transfer_in_place(&mut spi, &mut [1]),
            Err(Error::Invalid)
        );
    }

    #[test]
    fn flush_waits_for_the_last_frame_after_rx_is_drained() {
        let mut m = Model::new(4);
        m.hold_busy = 3;
        let mut rx = [0u8; 6];
        exchange(
            &mut m,
            Rx::U8(&mut rx),
            Tx::U8(&[1, 2, 3, 4, 5, 6]),
            0xff,
            4,
        )
        .unwrap();
        // All RX collected, the shifter still busy.
        assert!(m.rx.is_empty() && m.busy_left == 3);
        wait_idle(&mut m).unwrap();
        assert_eq!(m.busy_left, 0);
    }

    /// Codex review 10 #1: `Device::free` hands back the chip select in the state
    /// `Device::new` takes (type-level: `new` writes registers, so it is not called here).
    #[test]
    fn freed_chip_select_makes_a_new_device() {
        fn renew<B>(dev: Device<B, 8>) -> Device<B, 8> {
            let (bus, cs) = dev.free();
            Device::new(bus, cs)
        }
        fn free_twice<B>(dev: Device<B, 8>) -> (B, Pin<8>) {
            renew(dev).free()
        }
        let _ = free_twice::<()>;
    }

    /// A bus over [`Model`] that records each call in a shared event log; `wait_idle` is
    /// logged as "flush" only once the model reports idle. `admit` is the driver's own
    /// admission over the model's clk_sys.
    struct LogBus<'a> {
        m: Model,
        log: &'a core::cell::RefCell<Vec<String>>,
        fail_op: Option<(usize, Error)>,
        fail_flush: Option<Error>,
        ops: usize,
    }
    use std::string::{String, ToString};
    impl sealed::Bus for LogBus<'_> {}
    impl LogBus<'_> {
        fn op(&mut self, name: &str, read: &mut [u8], write: &[u8]) -> Result<(), Error> {
            let n = self.ops;
            self.ops += 1;
            if let Some((at, e)) = self.fail_op
                && at == n
            {
                self.log.borrow_mut().push(std::format!("{name} err"));
                return Err(e);
            }
            exchange(&mut self.m, Rx::U8(read), Tx::U8(write), 0xff, 4)?;
            // The last frame is still being shifted out.
            assert!(self.m.busy_left > 0);
            self.log.borrow_mut().push(name.to_string());
            Ok(())
        }
    }
    impl Bus<u8> for LogBus<'_> {
        fn admit(
            &mut self,
            _: sealed::Key,
            deadline: Option<Deadline>,
        ) -> Result<sealed::Admitted<u8>, Error> {
            let r = admit(&mut self.m, &CLOCKS);
            let name = if r.is_ok() { "admit" } else { "admit err" };
            self.log.borrow_mut().push(name.to_string());
            r.map(|()| sealed::Admitted(deadline.unwrap_or_else(|| end(&self.m)), PhantomData))
        }
        fn exchange(
            &mut self,
            _: sealed::Admitted<u8>,
            read: &mut [u8],
            write: &[u8],
        ) -> Result<(), Error> {
            let name = match (read.is_empty(), write.is_empty()) {
                (false, true) => "read",
                (true, false) => "write",
                _ => "transfer",
            };
            self.op(name, read, write)
        }
        fn exchange_in_place(
            &mut self,
            _: sealed::Admitted<u8>,
            words: &mut [u8],
        ) -> Result<(), Error> {
            let tx = words.to_vec();
            self.op("in_place", words, &tx)
        }
        fn wait_idle(&mut self, _: sealed::Admitted<u8>) -> Result<(), Error> {
            if let Some(e) = self.fail_flush {
                self.log.borrow_mut().push("flush err".to_string());
                return Err(e);
            }
            wait_idle(&mut self.m)?;
            assert_eq!(self.m.busy_left, 0);
            self.log.borrow_mut().push("flush".to_string());
            Ok(())
        }
        fn delay(&mut self, _: sealed::Admitted<u8>, ns: u32) -> Result<(), Error> {
            self.log.borrow_mut().push(std::format!("delay {ns}"));
            Ok(())
        }

        fn abort(&mut self, _: sealed::Admitted<u8>) {
            self.log.borrow_mut().push("abort".to_string());
        }
    }
    /// (result, event log, (fast, strict) clock checks) of one transaction over a model whose
    /// clk_sys reads as `sys` to the fast check. The strict check always fails there, so a
    /// transaction that used it could not pass.
    fn logged_with(
        sys: Option<u32>,
        fail_op: Option<(usize, Error)>,
        fail_flush: Option<Error>,
        ops: &mut [Operation<'_, u8>],
    ) -> (Result<(), Error>, Vec<String>, (u32, u32)) {
        let log = core::cell::RefCell::new(Vec::new());
        let mut m = Model::new(4);
        m.hold_busy = 3;
        m.sys = sys;
        m.strict_stale = true;
        let mut bus = LogBus {
            m,
            log: &log,
            fail_op,
            fail_flush,
            ops: 0,
        };
        let r = transaction(
            &mut bus,
            |high| {
                log.borrow_mut()
                    .push(if high { "cs high" } else { "cs low" }.to_string())
            },
            ops,
            None,
        );
        let checks = (bus.m.fast, bus.m.strict);
        drop(bus);
        (r, log.into_inner(), checks)
    }
    fn logged(
        fail_op: Option<(usize, Error)>,
        fail_flush: Option<Error>,
        ops: &mut [Operation<'_, u8>],
    ) -> (Result<(), Error>, Vec<String>) {
        let (r, log, checks) = logged_with(Some(200_000_000), fail_op, fail_flush, ops);
        assert_eq!(checks, (1, 0), "one fast clock check per transaction");
        (r, log)
    }

    /// Review 6 (Codex #2): admission, CS low -> operations -> flush -> CS high, a flush before
    /// each delay. On an operation error, request stop and release CS, keeping the error.
    #[test]
    fn device_transaction_orders_cs_flush_and_delay() {
        let mut read = [0u8; 2];
        let mut buf = [5u8, 6];
        let (r, log) = logged(
            None,
            None,
            &mut [
                Operation::Write(&[1, 2]),
                Operation::DelayNs(1500),
                Operation::Read(&mut read),
                Operation::TransferInPlace(&mut buf),
                Operation::Transfer(&mut [0; 2], &[3, 4]),
            ],
        );
        assert_eq!(r, Ok(()));
        assert_eq!(
            log,
            [
                "admit",
                "cs low",
                "write",
                "flush",
                "delay 1500",
                "read",
                "in_place",
                "transfer",
                "flush",
                "cs high"
            ]
        );
        assert_eq!(buf, [5, 6]);
        // An operation fails: skip further operations, request stop, and release CS.
        let (r, log) = logged(
            Some((1, Error::Overrun)),
            Some(Error::Timeout),
            &mut [
                Operation::Write(&[1]),
                Operation::Write(&[2]),
                Operation::DelayNs(10),
                Operation::Write(&[3]),
            ],
        );
        assert_eq!(r, Err(Error::Overrun));
        assert_eq!(
            log,
            ["admit", "cs low", "write", "write err", "abort", "cs high"]
        );
        // The flush before a delay fails: no delay, CS still released, that error returned.
        let (r, log) = logged(
            None,
            Some(Error::Timeout),
            &mut [
                Operation::Write(&[1]),
                Operation::DelayNs(10),
                Operation::Write(&[2]),
            ],
        );
        assert_eq!(r, Err(Error::Timeout));
        assert_eq!(
            log,
            ["admit", "cs low", "write", "flush err", "abort", "cs high"]
        );
        // Only the final flush fails.
        let (r, log) = logged(None, Some(Error::Timeout), &mut [Operation::Write(&[1])]);
        assert_eq!(r, Err(Error::Timeout));
        assert_eq!(
            log,
            ["admit", "cs low", "write", "flush err", "abort", "cs high"]
        );
        // No operations: still a framed, flushed CS pulse.
        let (r, log) = logged(None, None, &mut []);
        assert_eq!(r, Ok(()));
        assert_eq!(log, ["admit", "cs low", "flush", "cs high"]);
    }

    /// Review 7: a stale clk_sys fails the transaction before CS is asserted: no CS edge, no
    /// operation, no flush, no register access.
    #[test]
    fn stale_clock_transaction_never_moves_cs() {
        for sys in [Some(50_000_000), None] {
            let (r, log, checks) = logged_with(
                sys,
                None,
                None,
                &mut [Operation::Write(&[1, 2]), Operation::Read(&mut [0; 2])],
            );
            assert_eq!(r, Err(Error::Precondition));
            assert_eq!(log, ["admit err"]);
            assert_eq!(checks, (1, 0));
        }
    }
}
