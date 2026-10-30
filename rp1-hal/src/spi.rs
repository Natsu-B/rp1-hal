//! Polling SPI master on the RP1 DesignWare APB SSI blocks (DOC: RP1 peripherals §3.6, Linux
//! spi-dw-core). embedded-hal 1.0 [`SpiBus`] on [`Spi`], [`SpiDevice`] on [`Device`] with a GPIO
//! chip select.
//!
//! Instances (DOC §3.6.1): SPI0/1/2/3/5/6/8 are masters, SPI4/7 slave-only. Only the bank-0
//! masters SPI0, 1, 2, 3, 5 are exposed; SPI6/8 are not on bank 0. The serial clock is clk_sys
//! divided by an even BAUDR (2..=65534): `sclk = Clocks::sys_hz() / div`, so 100 MHz at most at
//! 200 MHz. clk_sys belongs to RP1 (clock authority) and is not voted on through SCMI. The
//! reset is RESETS CTRL1 bit 10 + n, DONE1 the same bit (rp1-regdb resets.toml).
//!
//! The controller's own chip-select outputs are not routed: SER selects CS0 internally because
//! the DW SSI only shifts with a slave selected, and [`Device`] drives a GPIO as chip select.
use crate::{
    gpio::{Output, Pin, SpiFunction},
    read, write,
};
use embedded_hal::{
    delay::DelayNs,
    spi::{ErrorKind, ErrorType, Mode, Operation, Phase, Polarity, SpiBus, SpiDevice},
};

use crate::pac::dw_ssi::{RegisterBlock, ctrlr0, int, sr};
macro_rules! offsets {
    ($($name:ident = $field:ident),* $(,)?) => {$(const $name: usize = core::mem::offset_of!(RegisterBlock, $field);)*};
}
offsets! {
    CTRLR0 = ctrlr0, SSIENR = ssienr, SER = ser, BAUDR = baudr, TXFTLR = txftlr, RXFTLR = rxftlr,
    RXFLR = rxflr, SR = sr, IMR = imr, RISR = risr, ICR = icr, DMACR = dmacr, DR = dr,
    RX_SAMPLE_DLY = rx_sample_dly,
}
/// No progress for this long fails a transfer (one 32-bit frame at the slowest rate is 10 ms).
const TIMEOUT_US: u32 = 100_000;
const RESETS_CLR1: usize = 0x4001_4000 + 0x3000 + 4;
const RESETS_DONE1: usize = 0x4001_4000 + 0x1c;
const fn reset_bit(n: u8) -> u32 {
    1 << (10 + n)
}
const fn base(n: u8) -> usize {
    0x4005_0000 + n as usize * 0x4000
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    /// No transfer progress within 100 ms, or the controller stayed busy.
    Timeout,
    /// Frequency, frame size or word type out of range.
    Invalid,
    /// The clock tree no longer matches the `Clocks` value, or the block is not a DW SSI.
    Precondition,
    /// RX/TX FIFO overflow or underflow flagged by the controller.
    Overrun,
}
impl embedded_hal::spi::Error for Error {
    fn kind(&self) -> ErrorKind {
        match self {
            Self::Overrun => ErrorKind::Overrun,
            _ => ErrorKind::Other,
        }
    }
}

mod sealed {
    pub trait Pin {}
    impl<const N: u8> Pin for crate::gpio::Pin<N> {}
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
macro_rules! routes {
    ($n:literal, $f:literal: $sck:literal, $mosi:literal, $miso:literal) => {
        impl Sck<$n> for Pin<$sck> {
            const FUNC: u8 = $f;
        }
        impl Mosi<$n> for Pin<$mosi> {
            const FUNC: u8 = $f;
        }
        impl Miso<$n> for Pin<$miso> {
            const FUNC: u8 = $f;
        }
    };
}
// DOC: RP1 bank-0 function table (SCLK, SIO[0], SIO[1]).
routes!(0, 0: 11, 10, 9);
routes!(1, 0: 21, 20, 19);
routes!(2, 8: 3, 2, 1);
routes!(3, 8: 7, 6, 5);
routes!(5, 8: 15, 14, 13);

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

/// [`Token::init`] settings: `Config::new(hz, mode)` plus the `with_*` setters (the struct is
/// `#[non_exhaustive]`, so fields can be added without breaking callers).
///
/// ```
/// use rp1_hal::spi::Config;
/// let c = Config::new(10_000_000, embedded_hal::spi::MODE_3)
///     .with_frame_bits(16)
///     .with_rx_sample_dly(2);
/// assert_eq!((c.frequency_hz, c.frame_bits, c.rx_sample_dly), (10_000_000, 16, 2));
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Config {
    /// Upper bound for SCLK; the rate used is the highest `clk_sys / even divider` not above it.
    pub frequency_hz: u32,
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
    pub const fn new(frequency_hz: u32, mode: Mode) -> Self {
        Self {
            frequency_hz,
            mode,
            frame_bits: 8,
            rx_sample_dly: 0,
        }
    }
    pub const fn with_frequency_hz(mut self, frequency_hz: u32) -> Self {
        self.frequency_hz = frequency_hz;
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
    _private: (),
}
/// A configured controller. Dropping it without [`Spi::free`] leaves the controller enabled
/// and the pins on the SPI function; the token is gone, so it cannot be initialised again.
pub struct Spi<const N: u8, const SCK: u8, const MOSI: u8, const MISO: u8> {
    _sck: Pin<SCK, SpiFunction>,
    _mosi: Pin<MOSI, SpiFunction>,
    _miso: Pin<MISO, SpiFunction>,
    clocks: crate::clocks::Clocks,
    depth: usize,
    dfs32: bool,
    cr0: u32,
    div: u32,
    rsd: u32,
}
/// [`Token::init`] failed: the error, and the token and pins back (unchanged) for a retry.
pub struct InitError<const N: u8, const SCK: u8, const MOSI: u8, const MISO: u8> {
    pub error: Error,
    pub token: Token<N>,
    pub sck: Pin<SCK>,
    pub mosi: Pin<MOSI>,
    pub miso: Pin<MISO>,
}
impl<const N: u8, const SCK: u8, const MOSI: u8, const MISO: u8> core::fmt::Debug
    for InitError<N, SCK, MOSI, MISO>
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.error.fmt(f)
    }
}
impl<const N: u8, const SCK: u8, const MOSI: u8, const MISO: u8> From<InitError<N, SCK, MOSI, MISO>>
    for Error
{
    fn from(e: InitError<N, SCK, MOSI, MISO>) -> Self {
        e.error
    }
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
        Self { _private: () }
    }
    /// Checks the clocks and the configuration, releases only this controller's reset, checks
    /// the block, detects the FIFO depth and frame field, then routes the typed pins. GPIO bank0
    /// and the raw microsecond timer must already run (cold bring-up); `clocks` may come from
    /// proc0 ([`crate::clocks::Clocks::from_proc0`]). On error the token and the pins come back
    /// in [`InitError`]; the pins are not touched before success.
    pub fn init<const SCK: u8, const MOSI: u8, const MISO: u8>(
        self,
        sck: Pin<SCK>,
        mosi: Pin<MOSI>,
        miso: Pin<MISO>,
        clocks: &crate::clocks::Clocks,
        config: Config,
    ) -> Result<Spi<N, SCK, MOSI, MISO>, InitError<N, SCK, MOSI, MISO>>
    where
        Pin<SCK>: Sck<N>,
        Pin<MOSI>: Mosi<N>,
        Pin<MISO>: Miso<N>,
    {
        match self.start(clocks, config) {
            Ok((depth, dfs32, cr0, div)) => Ok(Spi {
                _sck: sck.into_spi(<Pin<SCK> as Sck<N>>::FUNC, true),
                _mosi: mosi.into_spi(<Pin<MOSI> as Mosi<N>>::FUNC, true),
                _miso: miso.into_spi(<Pin<MISO> as Miso<N>>::FUNC, false),
                clocks: *clocks,
                depth,
                dfs32,
                cr0,
                div,
                rsd: config.rx_sample_dly.into(),
            }),
            Err(error) => Err(InitError {
                error,
                token: self,
                sck,
                mosi,
                miso,
            }),
        }
    }
    /// The register part of [`Token::init`]: (depth, dfs32, CTRLR0, BAUDR).
    fn start(
        &self,
        clocks: &crate::clocks::Clocks,
        config: Config,
    ) -> Result<(usize, bool, u32, u32), Error> {
        if !clocks.still_current() {
            return Err(Error::Precondition);
        }
        let div = divider(clocks.sys_hz(), config.frequency_hz)?;
        // The widest frame any instance takes; the instance's own limit is checked after probe.
        control(config.mode, config.frame_bits, true)?;
        write(RESETS_CLR1, reset_bit(N));
        crate::poll(RESETS_DONE1, reset_bit(N), reset_bit(N)).map_err(|_| Error::Timeout)?;
        let mut io = Mmio { base: base(N) };
        let (depth, dfs32) = probe(&mut io).ok_or(Error::Precondition)?;
        let cr0 = control(config.mode, config.frame_bits, dfs32)?;
        setup(&mut io, cr0, div, config.rx_sample_dly.into());
        Ok((depth, dfs32, cr0, div))
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
fn probe(io: &mut impl Io) -> Option<(usize, bool)> {
    io.write(SSIENR, 0);
    let depth = fifo_depth(io, TXFTLR).min(fifo_depth(io, RXFTLR));
    io.write(CTRLR0, u32::MAX);
    let dfs32 = io.read(CTRLR0) & ctrlr0::DFS_MASK == 0;
    io.write(CTRLR0, 0);
    (depth > 1).then_some((depth as usize, dfs32))
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

/// Disabled -> configured with interrupts and DMA off, CS0 selected -> enabled (FIFOs flushed).
fn setup(io: &mut impl Io, cr0: u32, div: u32, rsd: u32) {
    io.write(SSIENR, 0);
    io.write(IMR, 0);
    io.write(DMACR, 0);
    io.write(CTRLR0, cr0);
    io.write(BAUDR, div);
    io.write(RX_SAMPLE_DLY, rsd);
    io.write(SER, 1);
    io.read(ICR);
    io.write(SSIENR, 1);
}

trait Io {
    fn read(&mut self, offset: usize) -> u32;
    fn write(&mut self, offset: usize, value: u32);
    fn now(&mut self) -> u32;
    /// The clock registers still match `clocks` ([`crate::clocks::Clocks::still_current`]).
    fn clocks_current(&mut self, clocks: &crate::clocks::Clocks) -> bool;
}
struct Mmio {
    base: usize,
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
    fn now(&mut self) -> u32 {
        read(crate::TIMER_RAWL)
    }
    fn clocks_current(&mut self, clocks: &crate::clocks::Clocks) -> bool {
        clocks.still_current()
    }
}

/// Full-duplex exchange of `max(read, write)` frames. Frames beyond `write` send 0; received
/// frames beyond `read` are dropped. At most `depth` frames are in flight, so neither FIFO can
/// overflow; a stall of [`TIMEOUT_US`] without progress fails.
#[rp1_rt::isram]
fn exchange(
    io: &mut impl Io,
    mut read: Rx<'_>,
    write: Tx<'_>,
    mask: u32,
    depth: usize,
) -> Result<(), Error> {
    let len = read.len().max(write.len());
    let (mut sent, mut received) = (0, 0);
    let mut start = io.now();
    while received < len {
        let mut progress = false;
        let mut ready = (io.read(RXFLR) as usize).min(sent - received);
        while ready > 0 {
            read.set(received, io.read(DR) & mask);
            received += 1;
            ready -= 1;
            progress = true;
        }
        let mut room = (depth - (sent - received)).min(len - sent);
        while room > 0 {
            io.write(DR, write.get(sent) & mask);
            sent += 1;
            room -= 1;
            progress = true;
        }
        if io.read(RISR) & (int::RXOI | int::RXUI | int::TXOI) != 0 {
            return Err(Error::Overrun);
        }
        if progress {
            start = io.now();
        } else if io.now().wrapping_sub(start) >= TIMEOUT_US {
            return Err(Error::Timeout);
        }
    }
    Ok(())
}

/// One bus call: `Precondition` before any FIFO access if the clocks no longer match
/// `clocks` (SCLK would not be [`Spi::frequency_hz`]); after a failed exchange the controller
/// is disabled (both FIFOs flushed, shifter stopped) and set up again unchanged.
fn run(
    io: &mut impl Io,
    clocks: &crate::clocks::Clocks,
    (cr0, div, rsd): (u32, u32, u32),
    bits: u8,
    depth: usize,
    read: Rx<'_>,
    write: Tx<'_>,
) -> Result<(), Error> {
    if !io.clocks_current(clocks) {
        return Err(Error::Precondition);
    }
    let mask = u32::MAX >> (32 - u32::from(bits));
    let result = exchange(io, read, write, mask, depth);
    if result.is_err() {
        setup(io, cr0, div, rsd);
    }
    result
}

fn wait_idle(io: &mut impl Io) -> Result<(), Error> {
    let start = io.now();
    loop {
        if io.read(SR) & (sr::BUSY | sr::TFE) == sr::TFE {
            return Ok(());
        }
        if io.now().wrapping_sub(start) >= TIMEOUT_US {
            return Err(Error::Timeout);
        }
    }
}

impl<const N: u8, const SCK: u8, const MOSI: u8, const MISO: u8> Spi<N, SCK, MOSI, MISO> {
    fn io(&self) -> Mmio {
        Mmio { base: base(N) }
    }
    fn reconfigure(&mut self, cr0: u32, div: u32) -> Result<(), Error> {
        if !self.clocks.still_current() {
            return Err(Error::Precondition);
        }
        let mut io = self.io();
        wait_idle(&mut io)?;
        setup(&mut io, cr0, div, self.rsd);
        self.cr0 = cr0;
        self.div = div;
        Ok(())
    }
    /// SCLK actually used: clk_sys / BAUDR.
    pub fn frequency_hz(&self) -> u32 {
        self.clocks.sys_hz() / self.div
    }
    /// BAUDR divider in use.
    pub fn divider(&self) -> u32 {
        self.div
    }
    /// Sets SCLK to at most `hz`; returns the rate used.
    pub fn set_frequency(&mut self, hz: u32) -> Result<u32, Error> {
        let div = divider(self.clocks.sys_hz(), hz)?;
        self.reconfigure(self.cr0, div)?;
        Ok(self.frequency_hz())
    }
    pub fn set_mode(&mut self, mode: Mode) -> Result<(), Error> {
        let cr0 = control(mode, self.frame_bits(), self.dfs32)? | (self.cr0 & ctrlr0::SRL);
        self.reconfigure(cr0, self.div)
    }
    /// Frame size, 4..=16 (4..=32 with [`Spi::has_32_bit_frames`]). `SpiBus<W>` needs
    /// `frame_bits <= W::BITS`.
    pub fn set_frame_bits(&mut self, bits: u8) -> Result<(), Error> {
        let mut mode = embedded_hal::spi::MODE_0;
        if self.cr0 & ctrlr0::SCPH != 0 {
            mode.phase = Phase::CaptureOnSecondTransition;
        }
        if self.cr0 & ctrlr0::SCPOL != 0 {
            mode.polarity = Polarity::IdleHigh;
        }
        let cr0 = control(mode, bits, self.dfs32)? | (self.cr0 & ctrlr0::SRL);
        self.reconfigure(cr0, self.div)
    }
    pub fn frame_bits(&self) -> u8 {
        let dfs = if self.dfs32 {
            (self.cr0 & ctrlr0::DFS32_MASK) >> ctrlr0::DFS32_SHIFT
        } else {
            self.cr0 & ctrlr0::DFS_MASK
        };
        dfs as u8 + 1
    }
    /// Internal loopback (CTRLR0.SRL): received data is the transmitted data, not MISO.
    pub fn set_loopback(&mut self, on: bool) -> Result<(), Error> {
        let cr0 = if on {
            self.cr0 | ctrlr0::SRL
        } else {
            self.cr0 & !ctrlr0::SRL
        };
        self.reconfigure(cr0, self.div)
    }
    /// FIFO depth in frames (detected at init).
    pub fn fifo_depth(&self) -> usize {
        self.depth
    }
    pub fn has_32_bit_frames(&self) -> bool {
        self.dfs32
    }
    fn run<W: Word>(&mut self, read: &mut [W], write: &[W]) -> Result<(), Error> {
        let bits = self.frame_bits();
        if bits > W::BITS {
            return Err(Error::Invalid);
        }
        run(
            &mut self.io(),
            &self.clocks,
            (self.cr0, self.div, self.rsd),
            bits,
            self.depth,
            W::rx(read),
            W::tx(write),
        )
    }
    /// Disables the controller and gives back the token and the (still muxed) pins.
    pub fn free(self) -> (Token<N>, Pin<SCK>, Pin<MOSI>, Pin<MISO>) {
        self.io().write(SSIENR, 0);
        (
            Token { _private: () },
            self._sck.release(),
            self._mosi.release(),
            self._miso.release(),
        )
    }
}
impl<const N: u8, const SCK: u8, const MOSI: u8, const MISO: u8> ErrorType
    for Spi<N, SCK, MOSI, MISO>
{
    type Error = Error;
}
impl<W: Word, const N: u8, const SCK: u8, const MOSI: u8, const MISO: u8> SpiBus<W>
    for Spi<N, SCK, MOSI, MISO>
{
    /// Sends zero frames.
    fn read(&mut self, words: &mut [W]) -> Result<(), Error> {
        self.run(words, &[])
    }
    fn write(&mut self, words: &[W]) -> Result<(), Error> {
        self.run(&mut [], words)
    }
    fn transfer(&mut self, read: &mut [W], write: &[W]) -> Result<(), Error> {
        self.run(read, write)
    }
    fn transfer_in_place(&mut self, words: &mut [W]) -> Result<(), Error> {
        let mut tx = [W::from_u32(0); 32];
        for chunk in words.chunks_mut(tx.len()) {
            let tx = &mut tx[..chunk.len()];
            tx.copy_from_slice(chunk);
            self.run(chunk, tx)?;
        }
        Ok(())
    }
    /// Every call returns only after all frames were received, so this waits for BUSY to clear.
    fn flush(&mut self) -> Result<(), Error> {
        wait_idle(&mut self.io())
    }
}

/// An [`SpiBus`] with a GPIO chip select (active low) for one device: [`SpiDevice`].
pub struct Device<B, const CS: u8> {
    bus: B,
    cs: Pin<CS, Output>,
    delay: crate::timer::Timer,
}
impl<B, const CS: u8> Device<B, CS> {
    /// Drives `cs` high (deselected) and owns the bus exclusively.
    pub fn new(bus: B, cs: Pin<CS>) -> Self {
        Self {
            bus,
            cs: cs.into_output(true),
            delay: crate::timer::Timer::new(),
        }
    }
    pub fn bus_mut(&mut self) -> &mut B {
        &mut self.bus
    }
    pub fn free(self) -> (B, Pin<CS, Output>) {
        (self.bus, self.cs)
    }
}
impl<B, const CS: u8> ErrorType for Device<B, CS> {
    type Error = Error;
}
impl<W: Word, B: SpiBus<W, Error = Error>, const CS: u8> SpiDevice<W> for Device<B, CS> {
    /// CS low, the operations in order (a delay flushes first), flush, CS high. CS is
    /// deasserted even when an operation fails; the first error is returned.
    fn transaction(&mut self, operations: &mut [Operation<'_, W>]) -> Result<(), Error> {
        use embedded_hal::digital::OutputPin;
        let (cs, delay) = (&mut self.cs, &mut self.delay);
        transaction(
            &mut self.bus,
            |high| {
                let _ = if high { cs.set_high() } else { cs.set_low() };
            },
            |ns| delay.delay_ns(ns),
            operations,
        )
    }
}

/// [`SpiDevice::transaction`] with the chip select (`cs(high)`) and the delay passed in.
fn transaction<W: Word, B: SpiBus<W, Error = Error>>(
    bus: &mut B,
    mut cs: impl FnMut(bool),
    mut delay: impl FnMut(u32),
    operations: &mut [Operation<'_, W>],
) -> Result<(), Error> {
    cs(false);
    let mut result = Ok(());
    for op in operations.iter_mut() {
        result = match op {
            Operation::Read(r) => bus.read(r),
            Operation::Write(w) => bus.write(w),
            Operation::Transfer(r, w) => bus.transfer(r, w),
            Operation::TransferInPlace(b) => bus.transfer_in_place(b),
            Operation::DelayNs(ns) => bus.flush().map(|()| delay(*ns)),
        };
        if result.is_err() {
            break;
        }
    }
    let flushed = bus.flush();
    cs(true);
    result.and(flushed)
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use std::{collections::VecDeque, vec::Vec};
    /// Loopback DW SSI: every RXFLR read shifts the whole TX FIFO into the RX FIFO. After the
    /// last push SR keeps BUSY for `hold_busy` more polls (the last frame still on the wire
    /// although RX already has it).
    struct Model {
        tx: VecDeque<u32>,
        rx: VecDeque<u32>,
        depth: usize,
        rx_depth: usize,
        tick: u32,
        stalled: bool,
        hold_busy: u32,
        busy_left: u32,
        risr: u32,
        pushed: Vec<u32>,
        txftlr: u32,
        rxftlr: u32,
        cr0: u32,
        dfs32: bool,
        writes: Vec<(usize, u32)>,
        sys: Option<u32>,
        uart: Option<u32>,
    }
    impl Model {
        fn new(depth: usize) -> Self {
            Self {
                tx: VecDeque::new(),
                rx: VecDeque::new(),
                depth,
                rx_depth: depth,
                tick: u32::MAX - 50,
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
                uart: Some(50_000_000),
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
        fn now(&mut self) -> u32 {
            self.tick = self.tick.wrapping_add(1000);
            self.tick
        }
        fn clocks_current(&mut self, clocks: &crate::clocks::Clocks) -> bool {
            clocks.matches(self.sys, self.uart)
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
        exchange(&mut m, Rx::U8(&mut rx), Tx::U8(&tx), 0xff, depth).unwrap();
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
        assert_eq!(m.tick.wrapping_sub(u32::MAX - 50), 102_000);
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
    fn setup_selects_cs0_with_interrupts_off() {
        let mut m = Model::new(4);
        setup(&mut m, 7, 20, 3);
        assert_eq!(
            m.writes,
            [
                (SSIENR, 0),
                (IMR, 0),
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
            Config::new(1_000_000, embedded_hal::spi::MODE_0).rx_sample_dly,
            0
        );
        use embedded_hal::spi::Error as _;
        assert_eq!(Error::Overrun.kind(), ErrorKind::Overrun);
    }

    /// Review 6 (Codex #1): PLL_SYS and CLK_UART unchanged but clk_sys elsewhere (e.g. XOSC):
    /// `Precondition` before DR is touched, and nothing else is written either.
    #[test]
    fn stale_sys_clock_fails_before_any_fifo_access() {
        for (sys, uart) in [
            (Some(50_000_000), Some(50_000_000)),
            (None, Some(50_000_000)),
            (Some(200_000_000), None),
        ] {
            let mut m = Model::new(4);
            m.sys = sys;
            m.uart = uart;
            assert_eq!(
                run(
                    &mut m,
                    &CLOCKS,
                    (7, 20, 0),
                    8,
                    4,
                    Rx::U8(&mut [0; 4]),
                    Tx::U8(&[1, 2, 3, 4])
                ),
                Err(Error::Precondition)
            );
            assert!(
                m.writes.is_empty() && m.pushed.is_empty(),
                "{sys:?} {uart:?}"
            );
        }
        let mut m = Model::new(4);
        let mut rx = [0; 4];
        run(
            &mut m,
            &CLOCKS,
            (7, 20, 0),
            8,
            4,
            Rx::U16(&mut rx),
            Tx::U16(&[1, 2, 3, 0x1ff]),
        )
        .unwrap();
        assert_eq!(rx, [1, 2, 3, 0xff]);
        // An exchange error re-runs setup with the same configuration.
        let mut m = Model::new(4);
        m.risr = int::RXOI;
        assert_eq!(
            run(
                &mut m,
                &CLOCKS,
                (7, 20, 2),
                8,
                4,
                Rx::U8(&mut [0; 2]),
                Tx::U8(&[1, 2])
            ),
            Err(Error::Overrun)
        );
        assert!(
            m.writes
                .ends_with(&[(RX_SAMPLE_DLY, 2), (SER, 1), (SSIENR, 1)])
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

    /// A bus over [`Model`] that records each call in a shared event log; `flush` is logged
    /// only once the model reports idle.
    struct LogBus<'a> {
        m: Model,
        log: &'a core::cell::RefCell<Vec<String>>,
        fail_op: Option<(usize, Error)>,
        fail_flush: Option<Error>,
        ops: usize,
    }
    use std::string::{String, ToString};
    impl ErrorType for LogBus<'_> {
        type Error = Error;
    }
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
    impl SpiBus<u8> for LogBus<'_> {
        fn read(&mut self, words: &mut [u8]) -> Result<(), Error> {
            self.op("read", words, &[])
        }
        fn write(&mut self, words: &[u8]) -> Result<(), Error> {
            self.op("write", &mut [], words)
        }
        fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Error> {
            self.op("transfer", read, write)
        }
        fn transfer_in_place(&mut self, words: &mut [u8]) -> Result<(), Error> {
            let tx = words.to_vec();
            self.op("in_place", words, &tx)
        }
        fn flush(&mut self) -> Result<(), Error> {
            if let Some(e) = self.fail_flush {
                self.log.borrow_mut().push("flush err".to_string());
                return Err(e);
            }
            wait_idle(&mut self.m)?;
            assert_eq!(self.m.busy_left, 0);
            self.log.borrow_mut().push("flush".to_string());
            Ok(())
        }
    }
    fn logged(
        fail_op: Option<(usize, Error)>,
        fail_flush: Option<Error>,
        ops: &mut [Operation<'_, u8>],
    ) -> (Result<(), Error>, Vec<String>) {
        let log = core::cell::RefCell::new(Vec::new());
        let mut m = Model::new(4);
        m.hold_busy = 3;
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
            |ns| log.borrow_mut().push(std::format!("delay {ns}")),
            ops,
        );
        (r, log.into_inner())
    }

    /// Review 6 (Codex #2): CS low -> operations -> flush -> CS high, a flush before each
    /// delay, and on error the flush and CS release still happen with the first error kept.
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
            ],
        );
        assert_eq!(r, Ok(()));
        assert_eq!(
            log,
            [
                "cs low",
                "write",
                "flush",
                "delay 1500",
                "read",
                "in_place",
                "flush",
                "cs high"
            ]
        );
        assert_eq!(buf, [5, 6]);
        // An operation fails: no later operation or delay, flush and CS high still run, and
        // the operation's error wins over the flush's.
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
            ["cs low", "write", "write err", "flush err", "cs high"]
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
            ["cs low", "write", "flush err", "flush err", "cs high"]
        );
        // Only the final flush fails.
        let (r, log) = logged(None, Some(Error::Timeout), &mut [Operation::Write(&[1])]);
        assert_eq!(r, Err(Error::Timeout));
        assert_eq!(log, ["cs low", "write", "flush err", "cs high"]);
        // No operations: still a framed, flushed CS pulse.
        let (r, log) = logged(None, None, &mut []);
        assert_eq!(r, Ok(()));
        assert_eq!(log, ["cs low", "flush", "cs high"]);
    }
}
