//! Seven-bit DesignWare controller with explicit [`I2c::polling`] access (DOC: RP1 §9 and Linux i2c-designware).
//! SCL counts are computed from `ic_clk` = clk_sys as reported by [`crate::clocks::Clocks`]
//! (HW: 50 MHz on XOSC, 200 MHz on PLL_SYS); no shared clock or SCMI vote is changed.
//! External pull-ups are required. Only bank-0 routes are exposed; I2C4..6 are internal.
use crate::{
    clocks::Clocks,
    gpio::{Alternate, Pin},
    linux_blocks::Mmio as ResetMmio,
    read,
    scmi::Registers,
    write,
};
use core::marker::PhantomData;
use embedded_hal::i2c::{ErrorKind, NoAcknowledgeSource, Operation};

use crate::pac::dw_i2c::{RegisterBlock, irq_wrap};
macro_rules! offsets {
    ($($name:ident = $field:ident),* $(,)?) => {$(const $name: usize = core::mem::offset_of!(RegisterBlock, $field);)*};
}
offsets! {
    CON = con, TAR = tar, DATA = data_cmd, SS_HCNT = ss_scl_hcnt, SS_LCNT = ss_scl_lcnt,
    FS_HCNT = fs_scl_hcnt, FS_LCNT = fs_scl_lcnt, INTR_STAT = intr_stat, MASK = intr_mask,
    RAW = raw_intr_stat, RX_TL = rx_tl, TX_TL = tx_tl, CLEAR = clr_intr, ENABLE = enable, STATUS = status,
    HOLD = sda_hold, SPKLEN = fs_spklen, ABORT_SOURCE = tx_abrt_source, ENABLE_STATUS = enable_status,
    PARAM = comp_param_1, TYPE = comp_type, IRQ_WRAP = irq_wrap,
}
const STOP: u32 = 1 << 9;
const RESTART: u32 = 1 << 10;
const READ: u32 = 1 << 8;
const STOP_DET: u32 = 1 << 9;
const MASTER_ACTIVITY: u32 = 1 << 5;
const COMP_TYPE: u32 = 0x4457_0140;
const TIMEOUT_US: u32 = 100_000;
// RESETS CTRL0 bits 7..13 are I2C0..6 (rp1-regdb resets.toml); CTRL1 bit 8 is SDIO0.
const RESETS: usize = 0x4001_4000;
const RESETS_SET0: usize = RESETS + 0x2000; // atomic set alias of CTRL0
const RESETS_CLR0: usize = RESETS + 0x3000; // atomic clear alias of CTRL0
const RESETS_DONE0: usize = RESETS + 0x18;
const fn reset_bit(n: u8) -> u32 {
    1 << (7 + n)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Speed {
    Standard,
    Fast,
    FastPlus,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    AddressNack,
    DataNack,
    ArbitrationLost,
    Timeout,
    Invalid,
    Precondition,
    Abort(u32),
    Fifo,
    /// The controller has not stopped, or this handle is being released after cancellation.
    Busy,
}
/// Controller bus speed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Config {
    speed: Speed,
}
impl Config {
    /// Select the controller bus speed.
    pub const fn new(speed: Speed) -> Self {
        Self { speed }
    }
}

impl From<Speed> for Config {
    fn from(speed: Speed) -> Self {
        Self::new(speed)
    }
}

/// An unshifted seven-bit target address.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Address7(u8);
impl Address7 {
    /// Rejects addresses of 0x80 and above with [`Error::Invalid`].
    pub const fn new(address: u8) -> Result<Self, Error> {
        if address < 128 {
            Ok(Self(address))
        } else {
            Err(Error::Invalid)
        }
    }

    /// Constructs a seven-bit address with compile-time validation.
    ///
    /// ```compile_fail,E0080
    /// const BAD: rp1_hal::i2c::Address7 = rp1_hal::i2c::Address7::new_const::<0x80>();
    /// ```
    pub const fn new_const<const ADDRESS: u8>() -> Self {
        const { assert!(ADDRESS < 128) };
        Self(ADDRESS)
    }
}

impl embedded_hal::i2c::Error for Error {
    fn kind(&self) -> ErrorKind {
        match self {
            Self::AddressNack => ErrorKind::NoAcknowledge(NoAcknowledgeSource::Address),
            Self::DataNack => ErrorKind::NoAcknowledge(NoAcknowledgeSource::Data),
            Self::ArbitrationLost => ErrorKind::ArbitrationLoss,
            Self::Fifo => ErrorKind::Overrun,
            _ => ErrorKind::Other,
        }
    }
}
fn abort_error(source: u32) -> Error {
    if source & (1 << 12) != 0 {
        Error::ArbitrationLost
    } else if source & 1 != 0 {
        Error::AddressNack
    } else if source & (1 << 3) != 0 {
        Error::DataNack
    } else {
        Error::Abort(source)
    }
}

mod sealed {
    pub trait CompletePins<const N: u8> {}
    pub trait Pin {}
    impl<const N: u8> Pin for crate::gpio::Pin<N> {}
}
/// SDA route of controller N; `FUNC` is the GPIO function select (`crate::pinmux`).
pub trait Sda<const N: u8>: sealed::Pin {
    const FUNC: u8;
}
/// SCL route of controller N.
pub trait Scl<const N: u8>: sealed::Pin {
    const FUNC: u8;
}
/// Schmitt, input enable, 12 mA sink, weak pull-up; external pull-ups are required. Only the
/// I2C peripheral may sink the line, so the pad output stays disabled while the mux switches.
const PAD: u32 = 0x7a;

pub struct Token<const N: u8> {
    _private: core::marker::PhantomData<core::cell::Cell<()>>,
}
/// The I²C controller role.
pub struct Controller {
    _private: (),
}

#[derive(Clone, Copy)]
enum State {
    Ready,
    Poisoned,
    Stopping,
}

impl State {
    fn ready(self) -> Result<(), Error> {
        match self {
            Self::Ready => Ok(()),
            Self::Poisoned => Err(Error::Precondition),
            Self::Stopping => Err(Error::Busy),
        }
    }
}

/// Initialized controller. Select [`I2c::polling`] for embedded-hal transactions.
pub struct I2c<const N: u8, P: CompletePins<N>, Role = Controller> {
    pins: P::Active,
    rx_depth: usize,
    state: State,
    clocks: Clocks,
    _role: PhantomData<Role>,
}

/// The initialization error retains the block and its pins during recovery.
pub type InitError<const N: u8, P> =
    crate::InitError<Parts<N, P>, Error, crate::Recovery<Parts<N, P>>>;

/// Explicit polling access to the I²C controller.
pub struct Polling<'a, const N: u8, P: CompletePins<N>>(&'a mut I2c<N, P>);

impl<const N: u8> Token<N> {
    /// # Safety
    /// Generated resources only: sole owner of I2C N, its reset and IRQ, with Linux disabled.
    pub const unsafe fn new() -> Self {
        assert!(N < 7);
        Self {
            _private: core::marker::PhantomData,
        }
    }
}

pin_builder!(Pin, []);
pin_builder!(Pin, [], sda: SDA => Sda);
pin_builder!(Pin, [sda: SDA], scl: SCL => Scl);
complete_pins!(SDA: Sda, SCL: Scl; (Pin<SDA, Alternate>, Pin<SCL, Alternate>));

/// A declared I²C N with its SDA and SCL pins.
pub struct Parts<const N: u8, P: CompletePins<N>> {
    token: Token<N>,
    pins: Pins<N, P>,
}
impl<const N: u8, P: CompletePins<N>> Parts<N, P> {
    /// Assemble the owned block and the routed pin bundle.
    pub fn new(token: Token<N>, pins: Pins<N, P>) -> Self {
        Self { token, pins }
    }

    /// Return the block and pins for manual assembly.
    pub fn into_parts(self) -> (Token<N>, Pins<N, P>) {
        (self.token, self.pins)
    }
}

type PinTuple<const SDA: u8, const SCL: u8, M = crate::gpio::Unconfigured> =
    (Pin<SDA, M>, Pin<SCL, M>);

impl<const N: u8, const SDA: u8, const SCL: u8> Parts<N, PinTuple<SDA, SCL>>
where
    Pin<SDA>: Sda<N>,
    Pin<SCL>: Scl<N>,
{
    /// Checks clk_sys and the bus speed before releasing only this controller's reset.
    /// GPIO bank 0 and the raw microsecond timer must already be running.
    /// The pins are muxed only after configuration succeeds.
    ///
    /// Retry with Standard mode if the requested Fast mode timing is invalid:
    ///
    /// ```no_run
    /// # use rp1_hal::{clocks::Clocks, gpio::Pin, i2c::{self, Speed}};
    /// # type Pins = (Pin<2>, Pin<3>);
    /// fn start(parts: i2c::Parts<1, Pins>, clocks: &Clocks)
    ///     -> Result<i2c::I2c<1, Pins>, i2c::InitError<1, Pins>>
    /// {
    ///     match parts.init(clocks, Speed::Fast) {
    ///         Ok(bus) => Ok(bus),
    ///         Err(error) => error.try_into_parts()?.init(clocks, Speed::Standard),
    ///     }
    /// }
    /// ```
    pub fn init(
        self,
        clocks: &Clocks,
        config: impl Into<Config>,
    ) -> Result<I2c<N, PinTuple<SDA, SCL>>, InitError<N, PinTuple<SDA, SCL>>> {
        self.init_on(
            &mut Mmio::new(N),
            clocks,
            config.into().speed,
            |(sda, scl)| {
                (
                    sda.into_alternate(<Pin<SDA> as Sda<N>>::FUNC, PAD, PAD),
                    scl.into_alternate(<Pin<SCL> as Scl<N>>::FUNC, PAD, PAD),
                )
            },
        )
    }

    fn init_on(
        self,
        io: &mut impl Io,
        clocks: &Clocks,
        speed: Speed,
        route: impl FnOnce(PinTuple<SDA, SCL>) -> PinTuple<SDA, SCL, Alternate>,
    ) -> Result<I2c<N, PinTuple<SDA, SCL>>, InitError<N, PinTuple<SDA, SCL>>> {
        if !io.sys_current_strict(clocks) {
            return Err(InitError::new(Error::Precondition, self));
        }
        let counts = match timing(clocks.sys().raw(), speed) {
            Ok(counts) => counts,
            Err(error) => return Err(InitError::new(error, self)),
        };
        if let Err(error) = io.release_reset(reset_bit(N)) {
            return Err(InitError::recovering(error, self));
        }
        if io.read(TYPE) != COMP_TYPE {
            return Err(InitError::recovering(Error::Precondition, self));
        }
        mask_interrupts(io);
        if let Err(error) = disable(io) {
            return Err(InitError::recovering(error, self));
        }
        io.read(CLEAR);
        io.write(RX_TL, 0);
        io.write(TX_TL, 0);
        configure(io, counts);
        Ok(I2c {
            pins: route(self.pins.0),
            rx_depth: ((io.read(PARAM) >> 8) & 255) as usize + 1,
            state: State::Ready,
            clocks: *clocks,
            _role: PhantomData,
        })
    }
}

impl<const N: u8, P: CompletePins<N>> InitError<N, P> {
    /// Request I²C reset and return the parts once reset is confirmed.
    pub fn try_recover(self) -> Result<Parts<N, P>, Self> {
        self.try_recover_with(&mut ResetMmio)
    }

    fn try_recover_with(self, io: &mut impl Registers) -> Result<Parts<N, P>, Self> {
        self.recover(|| {
            // Init has not muxed the pins. Holding reset also works when the controller
            // registers are inaccessible or a target is holding SCL.
            io.write(RESETS_SET0, reset_bit(N));
            io.read(RESETS_DONE0) & reset_bit(N) == 0
        })
    }
}

/// DW spike filter target: the 50 ns of Fm/Fm+ (HW at 50 MHz: 3 cycles = 60 ns).
const SPIKE_NS: u32 = 50;
/// Time the pull-up rise adds to the measured low period (HW, 100 kHz capture: 5 cycles at 20 ns).
const RISE_NS: u32 = 100;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Counts {
    mode: u32,
    spklen: u32,
    hcnt: u32,
    lcnt: u32,
}

/// Counts for the SCL high/low targets of `speed` at `ic_clk_hz` (= clk_sys). `Invalid` if it
/// is not a whole number of MHz (`Clocks` only gives 50 or 200 MHz) or too slow for `speed`.
/// Targets keep the I2C minimums (Sm 4.0/4.7 us, Fm 0.6/1.3 us, Fm+ 0.26/0.5 us).
/// HW (DesignWare, measured at 50 MHz): high = HCNT + SPKLEN + 7, low = LCNT + 1 cycles + rise.
fn timing(ic_clk_hz: u32, speed: Speed) -> Result<Counts, Error> {
    let (mode, high_ns, low_ns) = match speed {
        Speed::Standard => (1, 4_600, 5_400),
        Speed::Fast => (2, 900, 1_600),
        Speed::FastPlus => (2, 400, 600),
    };
    // Whole MHz only (in practice 50 or 200 MHz): exact in 32 bits, no 64-bit division.
    if !ic_clk_hz.is_multiple_of(1_000_000) {
        return Err(Error::Invalid);
    }
    let mhz = ic_clk_hz / 1_000_000;
    let cycles = |ns: u32| (ns * mhz).div_ceil(1_000);
    let spklen = cycles(SPIKE_NS).max(1);
    let hcnt = cycles(high_ns).checked_sub(spklen + 7);
    let lcnt = cycles(low_ns).checked_sub(1 + cycles(RISE_NS));
    match (hcnt, lcnt) {
        // DesignWare minimums: HCNT >= SPKLEN + 5, LCNT >= SPKLEN + 7; 16-bit registers.
        (Some(h), Some(l)) if h >= spklen + 5 && l >= spklen + 7 && h.max(l) <= 0xffff => {
            Ok(Counts {
                mode,
                spklen,
                hcnt: h,
                lcnt: l,
            })
        }
        _ => Err(Error::Invalid),
    }
}
/// Masks every controller interrupt and closes the RP1 wrapper.
fn mask_interrupts(io: &mut impl Io) {
    io.write(MASK, 0);
    let wrap = io.read(IRQ_WRAP);
    io.write(IRQ_WRAP, wrap & !irq_wrap::IRQ_EN);
}
/// [`I2c::enable_interrupt`]: the controller mask, then the RP1 wrapper.
fn enable_interrupt(io: &mut impl Io, mask: u32) {
    let unmasked = io.read(MASK) | mask;
    io.write(MASK, unmasked);
    let wrap = io.read(IRQ_WRAP);
    io.write(IRQ_WRAP, wrap | irq_wrap::IRQ_EN);
}
fn configure(io: &mut impl Io, c: Counts) {
    let (h, l) = if c.mode == 1 {
        (SS_HCNT, SS_LCNT)
    } else {
        (FS_HCNT, FS_LCNT)
    };
    io.write(SPKLEN, c.spklen);
    io.write(h, c.hcnt);
    io.write(l, c.lcnt);
    io.write(HOLD, c.lcnt / 2);
    io.write(CON, 1 | (c.mode << 1) | (1 << 5) | (1 << 6));
}

// The small register interface also runs the real FIFO/abort loop against a host model.
trait Io {
    fn read(&mut self, offset: usize) -> u32;
    fn write(&mut self, offset: usize, value: u32);
    fn now(&mut self) -> u32;
    /// clk_sys (`ic_clk`) still matches `clocks`, per transaction ([`Clocks::sys_current`]).
    fn sys_current(&mut self, clocks: &Clocks) -> bool;
    /// clk_sys still matches `clocks`, at init and speed changes
    /// ([`Clocks::sys_current_strict`]).
    fn sys_current_strict(&mut self, clocks: &Clocks) -> bool;
    /// Releases the controller reset `bit` (RESETS CTRL0) and waits for DONE0.
    fn release_reset(&mut self, bit: u32) -> Result<(), Error>;
}
struct Mmio {
    base: usize,
}
impl Mmio {
    fn new(n: u8) -> Self {
        Self {
            base: 0x4007_0000 + n as usize * 0x4000,
        }
    }
}
impl Io for Mmio {
    #[inline(always)]
    fn read(&mut self, offset: usize) -> u32 {
        read(self.base + offset)
    }
    #[inline(always)]
    fn write(&mut self, offset: usize, value: u32) {
        write(self.base + offset, value);
    }
    #[inline(always)]
    fn now(&mut self) -> u32 {
        read(0x400a_c028)
    }
    #[inline(always)]
    fn sys_current(&mut self, clocks: &Clocks) -> bool {
        clocks.sys_current()
    }
    fn sys_current_strict(&mut self, clocks: &Clocks) -> bool {
        clocks.sys_current_strict()
    }
    fn release_reset(&mut self, bit: u32) -> Result<(), Error> {
        write(RESETS_CLR0, bit);
        crate::poll(RESETS_DONE0, bit, bit).map_err(|_| Error::Timeout)
    }
}
fn disable(io: &mut impl Io) -> Result<(), Error> {
    io.write(ENABLE, 0);
    let start = io.now();
    while !stopped(io) {
        if io.now().wrapping_sub(start) >= TIMEOUT_US {
            return Err(Error::Timeout);
        }
    }
    Ok(())
}
fn stopped(io: &mut impl Io) -> bool {
    io.read(ENABLE_STATUS) & 1 == 0 && io.read(STATUS) & MASTER_ACTIVITY == 0
}

/// Keep the interrupt mask if disable times out.
fn release_polling(io: &mut impl Io) -> Result<(), Error> {
    disable(io)?;
    mask_interrupts(io);
    Ok(())
}

/// Outside a transaction IC_ENABLE.ENABLE is already 0: the transaction error path
/// requested ABORT while ENABLE was still set. ABORT is ignored unless ENABLE is set
/// (DOC: DW_apb_i2c IC_ENABLE). Setting both here would enable it again.
fn cancel(io: &mut impl Io) -> Result<(), Error> {
    io.write(ENABLE, 0);
    mask_interrupts(io);
    if stopped(io) {
        Ok(())
    } else {
        Err(Error::Busy)
    }
}

fn check(io: &mut impl Io, start: u32) -> Result<u32, Error> {
    let raw = io.read(RAW);
    if raw & (1 << 6) != 0 {
        return Err(abort_error(io.read(ABORT_SOURCE)));
    }
    if raw & 0xb != 0 {
        return Err(Error::Fifo);
    }
    if io.now().wrapping_sub(start) >= TIMEOUT_US {
        return Err(Error::Timeout);
    }
    Ok(raw)
}
fn command(read: bool, restart: bool, stop: bool, byte: u8) -> u32 {
    u32::from(byte)
        | if read { READ } else { 0 }
        | if restart { RESTART } else { 0 }
        | if stop { STOP } else { 0 }
}
fn operation_len(op: &Operation<'_>) -> usize {
    match op {
        Operation::Read(bytes) => bytes.len(),
        Operation::Write(bytes) => bytes.len(),
    }
}
#[rp1_rt::isram]
fn transfer(io: &mut impl Io, ops: &mut [Operation<'_>], depth: usize) -> Result<(), Error> {
    let last = ops.iter().rposition(|op| operation_len(op) != 0);
    let mut previous = None;
    for (index, op) in ops
        .iter_mut()
        .enumerate()
        .filter(|(_, op)| operation_len(op) != 0)
    {
        let is_read = matches!(op, Operation::Read(_));
        let restart = previous.is_some_and(|r| r != is_read);
        let len = operation_len(op);
        let (mut sent, mut received) = (0, 0);
        let mut start = io.now();
        while sent < len || (is_read && received < len) {
            check(io, start)?;
            let before = (sent, received);
            let status = io.read(STATUS);
            if let Operation::Read(bytes) = op
                && received < sent
                && status & (1 << 3) != 0
            {
                bytes[received] = io.read(DATA) as u8;
                received += 1;
            }
            if sent < len && status & (1 << 1) != 0 && (!is_read || sent - received < depth) {
                let byte = match op {
                    Operation::Write(bytes) => bytes[sent],
                    _ => 0,
                };
                io.write(
                    DATA,
                    command(
                        is_read,
                        sent == 0 && restart,
                        Some(index) == last && sent + 1 == len,
                        byte,
                    ),
                );
                sent += 1;
            }
            if before != (sent, received) {
                start = io.now();
            }
        }
        previous = Some(is_read);
    }
    if last.is_some() {
        let start = io.now();
        while check(io, start)? & STOP_DET == 0 {}
        while io.read(STATUS) & MASTER_ACTIVITY != 0 {
            check(io, start)?;
        }
    }
    Ok(())
}

impl<const N: u8, P: CompletePins<N>> I2c<N, P> {
    fn io(&self) -> Mmio {
        Mmio::new(N)
    }

    /// Selects the synchronous polling engine.
    pub fn polling(&mut self) -> Polling<'_, N, P> {
        Polling(self)
    }

    /// Poll a transaction using a checked seven-bit address.
    /// Use [`Self::polling`] for the embedded-hal transaction API.
    pub fn transaction_polling(
        &mut self,
        address: Address7,
        ops: &mut [Operation<'_>],
    ) -> Result<(), Error> {
        self.poll_transaction(address.0, ops)
    }

    fn poll_transaction(&mut self, address: u8, ops: &mut [Operation<'_>]) -> Result<(), Error> {
        let (clocks, depth) = (self.clocks, self.rx_depth);
        run(
            &mut self.io(),
            &clocks,
            &mut self.state,
            address,
            ops,
            depth,
        )
    }

    /// Repeat the disable request without waiting. If a target holds SCL the
    /// controller can stay active. Collect the parts with [`Self::try_free`].
    pub fn try_cancel(&mut self) -> Result<(), Error> {
        self.cancel_with(&mut self.io())
    }

    fn cancel_with(&mut self, io: &mut impl Io) -> Result<(), Error> {
        self.state = State::Stopping;
        cancel(io)
    }

    /// Unmasks the [`intr`](crate::pac::dw_i2c::intr) bits in `mask` and opens the RP1 wrapper
    /// that routes the controller interrupt to the cores; the NVIC line comes from the generated
    /// `irq` resources. Polling transactions mask every interrupt while they run and restore
    /// the mask afterwards.
    pub fn enable_interrupt(&mut self, mask: u32) {
        if !matches!(self.state, State::Stopping) {
            enable_interrupt(&mut self.io(), mask);
        }
    }
    pub fn disable_interrupt(&mut self, mask: u32) {
        if !matches!(self.state, State::Stopping) {
            let mut io = self.io();
            let masked = io.read(MASK) & !mask;
            io.write(MASK, masked);
        }
    }
    /// Pending unmasked interrupts (INTR_STAT).
    pub fn interrupt_status(&self) -> u32 {
        if matches!(self.state, State::Stopping) {
            0
        } else {
            self.io().read(INTR_STAT)
        }
    }
    /// Clears the interrupts that software can clear. RX_FULL and TX_EMPTY follow the FIFO
    /// levels instead. Reading CLR_INTR also clears TX_ABRT_SOURCE: read the abort source first.
    pub fn clear_interrupt(&mut self) {
        if !matches!(self.state, State::Stopping) {
            self.io().read(CLEAR);
        }
    }
}
impl<const N: u8, const SDA: u8, const SCL: u8> I2c<N, PinTuple<SDA, SCL>>
where
    Pin<SDA>: Sda<N>,
    Pin<SCL>: Scl<N>,
{
    /// Return the parts only once the controller is disabled and idle.
    /// The controller is normally disabled between transactions. After a disable
    /// timeout (for example, a target holding SCL), this returns [`Error::Busy`]
    /// until the controller stops.
    ///
    /// Masks interrupts and closes the RP1 wrapper. Reset stays released and the pins
    /// stay muxed. Disable and unpend the separately owned NVIC line first, or ensure
    /// its handler accepts the controller being gone.
    pub fn try_free(self) -> Result<Parts<N, PinTuple<SDA, SCL>>, crate::FreeError<Self, Error>> {
        if !stopped(&mut self.io()) {
            return Err(crate::FreeError {
                driver: self,
                error: Error::Busy,
            });
        }
        mask_interrupts(&mut self.io());
        Ok(self.into_parts())
    }

    /// Poll for disable to complete for up to 100 ms. On timeout the driver and its
    /// interrupt mask are retained.
    pub fn free_polling(
        mut self,
    ) -> Result<Parts<N, PinTuple<SDA, SCL>>, crate::FreeError<Self, Error>> {
        match release_polling(&mut self.io()) {
            Ok(()) => Ok(self.into_parts()),
            Err(error) => {
                if !matches!(self.state, State::Stopping) {
                    self.state = State::Poisoned;
                }
                Err(crate::FreeError {
                    driver: self,
                    error,
                })
            }
        }
    }

    fn into_parts(self) -> Parts<N, PinTuple<SDA, SCL>> {
        let (sda, scl) = self.pins;
        Parts::new(
            Token {
                _private: PhantomData,
            },
            Pins::from_required((sda.release(), scl.release())),
        )
    }
}
impl<const N: u8, P: CompletePins<N>> Polling<'_, N, P> {
    /// Sets the bus speed after confirming disable.
    pub fn set_speed(&mut self, speed: Speed) -> Result<(), Error> {
        let clocks = self.0.clocks;
        set_speed(&mut self.0.io(), &clocks, &mut self.0.state, speed)
    }
}
impl<const N: u8, P: CompletePins<N>> embedded_hal::i2c::ErrorType for Polling<'_, N, P> {
    type Error = Error;
}
impl<const N: u8, P: CompletePins<N>> embedded_hal::i2c::I2c for Polling<'_, N, P> {
    /// Empty operations are skipped. Adjacent operations in the same direction continue
    /// without a restart. Direction changes restart, and only the final byte gets STOP.
    /// A 100 ms interval without progress times out. A failed disable poisons this
    /// handle. Further transactions return [`Error::Precondition`]. Use
    /// [`I2c::try_free`] to collect the parts once stopped and initialize again.
    /// A cancelled handle returns [`Error::Busy`].
    fn transaction(&mut self, address: u8, ops: &mut [Operation<'_>]) -> Result<(), Error> {
        self.0.poll_transaction(address, ops)
    }
}

/// The strict clk_sys check and counts come first. A failed disable makes the handle unusable.
fn set_speed(
    io: &mut impl Io,
    clocks: &Clocks,
    state: &mut State,
    speed: Speed,
) -> Result<(), Error> {
    state.ready()?;
    if !io.sys_current_strict(clocks) {
        return Err(Error::Precondition);
    }
    let counts = timing(clocks.sys().raw(), speed)?;
    *state = State::Poisoned;
    disable(io)?;
    configure(io, counts);
    *state = State::Ready;
    Ok(())
}

/// One transaction, one clock check. `Precondition` before any controller access if the
/// handle is poisoned or clk_sys (`ic_clk`) no longer matches `clocks` ([`Clocks::sys_current`]):
/// the SCL counts were computed for it. clk_uart is not read.
fn run(
    io: &mut impl Io,
    clocks: &Clocks,
    state: &mut State,
    address: u8,
    ops: &mut [Operation<'_>],
    depth: usize,
) -> Result<(), Error> {
    match *state {
        State::Stopping => return Err(Error::Busy),
        _ if address >= 128 => return Err(Error::Invalid),
        State::Poisoned => return Err(Error::Precondition),
        State::Ready => {}
    }
    if !io.sys_current(clocks) {
        return Err(Error::Precondition);
    }
    if ops.iter().all(|op| operation_len(op) == 0) {
        return Ok(());
    }
    // A handler would race the transfer loop for the FIFO and the status it polls.
    let mask = io.read(MASK);
    io.write(MASK, 0);
    let result = transact(io, state, address, ops, depth);
    io.write(MASK, mask);
    result
}

fn transact(
    io: &mut impl Io,
    state: &mut State,
    address: u8,
    ops: &mut [Operation<'_>],
    depth: usize,
) -> Result<(), Error> {
    *state = State::Poisoned;
    disable(io)?;
    io.read(CLEAR);
    io.write(TAR, u32::from(address));
    io.write(ENABLE, 1);
    let result = transfer(io, ops, depth);
    if result.is_err() {
        // Request a STOP/flush before disabling, including when a slave stretches forever.
        io.write(ENABLE, 3);
    }
    let cleanup = disable(io);
    if cleanup.is_ok() {
        *state = State::Ready;
    }
    io.read(CLEAR);
    result.and(cleanup)
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    /// Model timer step per `now()`.
    const STEP_US: u32 = 1000;
    /// A bounded wait that started reading the timer at `start` gave up at the first look
    /// past [`TIMEOUT_US`], whatever the model's step.
    fn assert_timed_out(tick: u32, start: u32) {
        let waited = tick.wrapping_sub(start) - STEP_US;
        assert!(
            (TIMEOUT_US..TIMEOUT_US + STEP_US).contains(&waited),
            "{waited}"
        );
    }
    use std::{collections::VecDeque, vec::Vec};
    struct Model {
        commands: Vec<u32>,
        rx: VecDeque<u8>,
        depth: usize,
        tick: u32,
        raw: u32,
        abort: u32,
        stalled: bool,
        next: u8,
        max_rx: usize,
        writes: usize,
        /// clk_sys as the clock checks see it.
        sys: Option<u32>,
        /// clk_sys drifted in a way only the strict check sees: the fast check passes.
        strict_stale: bool,
        /// Fast (per transaction) and strict (init, set_speed) clock checks made.
        fast: u32,
        strict: u32,
        resets: Vec<u32>,
        reset_writes: Vec<(usize, u32)>,
        reset_release_stuck: bool,
        reset_assert_stuck: bool,
        component_type: u32,
        mask: u32,
        wrap: u32,
        /// Writes to MASK and IRQ_WRAP, in order.
        interrupt_writes: Vec<(usize, u32)>,
        /// The controller ignores the disable request (a slave holds SCL).
        stuck_enabled: bool,
        master_active: bool,
        enable_writes: Vec<u32>,
    }
    impl Model {
        fn new(depth: usize) -> Self {
            Self {
                commands: Vec::new(),
                rx: VecDeque::new(),
                depth,
                tick: u32::MAX - 50,
                raw: 0,
                abort: 0,
                stalled: false,
                next: 0,
                max_rx: 0,
                writes: 0,
                sys: Some(200_000_000),
                strict_stale: false,
                fast: 0,
                strict: 0,
                resets: Vec::new(),
                reset_writes: Vec::new(),
                reset_release_stuck: false,
                reset_assert_stuck: false,
                component_type: COMP_TYPE,
                mask: 0,
                wrap: 0,
                interrupt_writes: Vec::new(),
                stuck_enabled: false,
                master_active: false,
                enable_writes: Vec::new(),
            }
        }
    }
    impl Io for Model {
        fn read(&mut self, offset: usize) -> u32 {
            match offset {
                RAW => self.raw,
                ABORT_SOURCE => self.abort,
                STATUS => {
                    if self.stalled {
                        0
                    } else {
                        let rx_data = if self.rx.is_empty() { 0 } else { 8 };
                        let active = if self.master_active {
                            MASTER_ACTIVITY
                        } else {
                            0
                        };
                        2 | rx_data | active
                    }
                }
                DATA => u32::from(self.rx.pop_front().expect("RX underflow")),
                TYPE => self.component_type,
                MASK => self.mask,
                IRQ_WRAP => self.wrap,
                ENABLE_STATUS => u32::from(self.stuck_enabled),
                // IC_RX_BUFFER_DEPTH - 1 in bits 15:8.
                PARAM => (self.depth as u32 - 1) << 8,
                _ => 0,
            }
        }
        fn write(&mut self, offset: usize, value: u32) {
            self.writes += 1;
            match offset {
                MASK => {
                    self.mask = value;
                    self.interrupt_writes.push((offset, value));
                }
                IRQ_WRAP => {
                    self.wrap = value;
                    self.interrupt_writes.push((offset, value));
                }
                ENABLE => self.enable_writes.push(value),
                _ => {}
            }
            if offset != DATA {
                return;
            }
            self.commands.push(value);
            if value & READ != 0 {
                self.rx.push_back(self.next);
                self.next = self.next.wrapping_add(1);
                self.max_rx = self.max_rx.max(self.rx.len());
                assert!(self.rx.len() <= self.depth, "RX overflow");
            }
            if value & STOP != 0 {
                self.raw |= STOP_DET;
            }
        }
        fn now(&mut self) -> u32 {
            self.tick = self.tick.wrapping_add(STEP_US);
            self.tick
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
            if self.reset_release_stuck {
                Err(Error::Timeout)
            } else {
                Ok(())
            }
        }
    }
    impl Registers for Model {
        fn read(&mut self, address: usize) -> u32 {
            assert_eq!(address, RESETS_DONE0);
            // Simulate an unrelated controller staying out of reset.
            reset_bit(2)
                | if self.reset_assert_stuck {
                    reset_bit(1)
                } else {
                    0
                }
        }
        fn write(&mut self, address: usize, value: u32) {
            assert_eq!((address, value), (RESETS_SET0, reset_bit(1)));
            self.reset_writes.push((address, value));
            if !self.reset_assert_stuck {
                self.stuck_enabled = false;
                self.master_active = false;
            }
        }
    }
    const CLOCKS: crate::clocks::Clocks = crate::clocks::Clocks::for_test(200_000_000, 50_000_000);
    /// Review 6: PLL_SYS / CLK_UART unchanged but clk_sys elsewhere fails before any
    /// controller access, and the handle stays usable.
    #[test]
    fn stale_sys_clock_fails_before_the_controller_is_touched() {
        let mut m = Model::new(2);
        m.sys = Some(50_000_000);
        // Review 8: transactions use the fast check only; the strict one would fail here.
        m.strict_stale = true;
        let mut state = State::Ready;
        assert_eq!(
            run(
                &mut m,
                &CLOCKS,
                &mut state,
                0x2d,
                &mut [Operation::Write(&[1])],
                2
            ),
            Err(Error::Precondition)
        );
        assert!(state.ready().is_ok() && m.writes == 0 && m.commands.is_empty());
        assert_eq!((m.fast, m.strict), (1, 0));
        m.sys = Some(200_000_000);
        assert_eq!(
            run(
                &mut m,
                &CLOCKS,
                &mut state,
                0x2d,
                &mut [Operation::Write(&[1])],
                2
            ),
            Ok(())
        );
        assert!(state.ready().is_ok());
        assert_eq!(m.commands, [1 | STOP]);
        // One check for the whole transaction, however many operations it has.
        let checks = m.fast;
        let mut r = [0; 3];
        run(
            &mut m,
            &CLOCKS,
            &mut state,
            0x2d,
            &mut [
                Operation::Write(&[1, 2]),
                Operation::Read(&mut r),
                Operation::Write(&[3]),
            ],
            2,
        )
        .unwrap();
        assert_eq!((m.fast, m.strict), (checks + 1, 0));
        let mut poisoned = State::Poisoned;
        assert_eq!(
            run(&mut m, &CLOCKS, &mut poisoned, 0x2d, &mut [], 2),
            Err(Error::Precondition)
        );
        assert_eq!(
            run(&mut m, &CLOCKS, &mut state, 0x80, &mut [], 2),
            Err(Error::Invalid)
        );
    }
    type Pins1 = PinTuple<2, 3>;
    fn init_model(
        token: Token<1>,
        m: &mut Model,
        pins: Pins1,
        clocks: &Clocks,
        speed: Speed,
        routed: &mut bool,
    ) -> Result<I2c<1, Pins1>, InitError<1, Pins1>> {
        Parts::new(token, Pins::from_required(pins)).init_on(m, clocks, speed, |(sda, scl)| {
            *routed = true;
            (sda.assume(), scl.assume())
        })
    }

    /// Review 7: a stale clk_sys or a speed impossible at it fails before the reset is
    /// released, without a register write or pin routing; the returned token and pins retry.
    #[test]
    fn init_failure_leaves_reset_and_pinmux_untouched_and_retries() {
        let slow = Clocks::for_test(1_000_000, 50_000_000);
        let mut token = unsafe { Token::<1>::new() };
        let mut pins: Pins1 = unsafe { (Pin::new(), Pin::new()) };
        for (sys, clocks, speed, want) in [
            (Some(50_000_000), CLOCKS, Speed::Fast, Error::Precondition),
            (None, CLOCKS, Speed::Fast, Error::Precondition),
            (Some(1_000_000), slow, Speed::Standard, Error::Invalid),
        ] {
            let mut m = Model::new(16);
            m.sys = sys;
            let mut routed = false;
            let e = init_model(token, &mut m, pins, &clocks, speed, &mut routed)
                .err()
                .expect("init must fail");
            let error = *e.error();
            let (t, pins_back) = e.try_into_parts().unwrap().into_parts();
            assert_eq!(error, want);
            assert!(m.resets.is_empty() && m.writes == 0 && !routed);
            (token, pins) = (t, pins_back.into_pins());
        }
        let mut m = Model::new(16);
        let mut routed = false;
        let i2c =
            init_model(token, &mut m, pins, &CLOCKS, Speed::Fast, &mut routed).expect("retry");
        assert!(routed && i2c.state.ready().is_ok());
        assert_eq!(
            (m.resets.as_slice(), i2c.rx_depth, m.strict, m.fast),
            (&[reset_bit(1)][..], 16, 1, 0)
        );
    }

    #[test]
    fn address_admission() {
        assert_eq!(Address7::new(0x7f), Ok(Address7(0x7f)));
        assert_eq!(Address7::new(0x80), Err(Error::Invalid));
    }

    #[test]
    fn init_failures_keep_pins_until_reset_is_confirmed() {
        for (release_stuck, component_type, enable_stuck, cause) in [
            (true, COMP_TYPE, false, Error::Timeout),
            (false, 0, false, Error::Precondition),
            (false, COMP_TYPE, true, Error::Timeout),
        ] {
            let mut model = Model::new(2);
            model.reset_release_stuck = release_stuck;
            model.component_type = component_type;
            model.stuck_enabled = enable_stuck;
            let mut routed = false;
            let pins: Pins1 = unsafe { (Pin::new(), Pin::new()) };
            let error = init_model(
                unsafe { Token::new() },
                &mut model,
                pins,
                &CLOCKS,
                Speed::Standard,
                &mut routed,
            )
            .err()
            .expect("init must fail");
            assert_eq!(*error.error(), cause);
            assert!(!routed);
            let error = error.try_into_parts().err().expect("pins are still owned");

            model.reset_assert_stuck = true;
            let error = error
                .try_recover_with(&mut model)
                .err()
                .expect("reset is still pending");
            model.reset_assert_stuck = false;
            let (token, pins) = error.try_recover_with(&mut model).unwrap().into_parts();
            assert_eq!(model.reset_writes, [(RESETS_SET0, reset_bit(1)); 2]);

            model.reset_release_stuck = false;
            model.component_type = COMP_TYPE;
            assert!(
                init_model(
                    token,
                    &mut model,
                    pins.into_pins(),
                    &CLOCKS,
                    Speed::Standard,
                    &mut routed
                )
                .is_ok()
            );
            assert!(routed);
        }
    }

    #[test]
    fn cancel_gates_interrupts_and_polling_until_release() {
        let mut model = Model::new(2);
        let mut bus = init_model(
            unsafe { Token::new() },
            &mut model,
            unsafe { (Pin::new(), Pin::new()) },
            &CLOCKS,
            Speed::Standard,
            &mut false,
        )
        .unwrap();
        // ENABLE_STATUS alone cannot show that the bus has stopped.
        model.master_active = true;
        assert_eq!(bus.cancel_with(&mut model), Err(Error::Busy));
        bus.enable_interrupt(crate::pac::dw_i2c::intr::TX_EMPTY);
        bus.disable_interrupt(crate::pac::dw_i2c::intr::TX_EMPTY);
        bus.clear_interrupt();
        assert_eq!(bus.interrupt_status(), 0);
        assert_eq!(
            bus.transaction_polling(Address7::new_const::<0x2d>(), &mut []),
            Err(Error::Busy)
        );
        assert_eq!(
            embedded_hal::i2c::I2c::transaction(&mut bus.polling(), 0x2d, &mut []),
            Err(Error::Busy)
        );
        assert_eq!(bus.polling().set_speed(Speed::Fast), Err(Error::Busy));

        model.master_active = false;
        assert_eq!(bus.cancel_with(&mut model), Ok(()));
        assert_eq!(bus.polling().set_speed(Speed::Fast), Err(Error::Busy));
    }

    /// Review 8: init and set_speed use the strict check only. clk_sys drifted in a way only
    /// it sees (the fast check passes): init fails before the reset is released, without a
    /// register write or pin routing; set_speed writes nothing and keeps the handle usable.
    #[test]
    fn init_and_set_speed_use_the_strict_check() {
        let mut m = Model::new(16);
        m.strict_stale = true;
        assert!(m.sys_current(&CLOCKS));
        m.fast = 0;
        let mut routed = false;
        let pins: Pins1 = unsafe { (Pin::new(), Pin::new()) };
        let e = init_model(
            unsafe { Token::new() },
            &mut m,
            pins,
            &CLOCKS,
            Speed::Fast,
            &mut routed,
        )
        .err()
        .expect("strict check must fail");
        assert_eq!(*e.error(), Error::Precondition);
        assert!(m.resets.is_empty() && m.writes == 0 && !routed);
        assert_eq!((m.strict, m.fast), (1, 0));

        let mut m = Model::new(16);
        m.strict_stale = true;
        let mut state = State::Ready;
        assert_eq!(
            set_speed(&mut m, &CLOCKS, &mut state, Speed::Standard),
            Err(Error::Precondition)
        );
        assert!(state.ready().is_ok() && m.writes == 0);
        assert_eq!((m.strict, m.fast), (1, 0));
        m.strict_stale = false;
        assert_eq!(
            set_speed(&mut m, &CLOCKS, &mut state, Speed::Standard),
            Ok(())
        );
        assert!(state.ready().is_ok() && m.writes > 0);
        assert_eq!((m.strict, m.fast), (2, 0));
    }

    #[test]
    fn interrupts_are_off_after_init_and_during_transactions() {
        use crate::pac::dw_i2c::intr::{STOP_DET, TX_ABRT};
        // DW reset value of INTR_MASK; wrapper bit 3 stands for bits the driver must keep.
        let mut m = Model::new(2);
        (m.mask, m.wrap) = (0x8ff, 0x9);
        let pins: Pins1 = unsafe { (Pin::new(), Pin::new()) };
        let mut i2c = init_model(
            unsafe { Token::new() },
            &mut m,
            pins,
            &CLOCKS,
            Speed::Fast,
            &mut false,
        )
        .expect("init");
        assert_eq!((m.mask, m.wrap), (0, 0x8));
        m.interrupt_writes.clear();
        enable_interrupt(&mut m, STOP_DET | TX_ABRT);
        assert_eq!(
            m.interrupt_writes,
            [(MASK, STOP_DET | TX_ABRT), (IRQ_WRAP, 0x9)]
        );
        m.interrupt_writes.clear();
        let ops = &mut [Operation::Write(&[1])];
        run(&mut m, &CLOCKS, &mut i2c.state, 0x2d, ops, 2).unwrap();
        assert_eq!(m.interrupt_writes, [(MASK, 0), (MASK, STOP_DET | TX_ABRT)]);
    }

    #[test]
    fn free_masks_interrupts_and_keeps_them_while_the_controller_stays_enabled() {
        let mut m = Model::new(2);
        (m.mask, m.wrap) = (0x250, 0x9);
        assert_eq!(release_polling(&mut m), Ok(()));
        assert_eq!(
            (m.mask, m.wrap, m.enable_writes.as_slice()),
            (0, 0x8, &[0][..])
        );

        let mut m = Model::new(2);
        (m.mask, m.wrap, m.stuck_enabled) = (0x250, 0x9, true);
        let before = m.tick;
        assert_eq!(release_polling(&mut m), Err(Error::Timeout));
        assert_eq!((m.mask, m.wrap), (0x250, 0x9));
        assert_timed_out(m.tick, before);
        // The slave lets go: the same call now succeeds.
        m.stuck_enabled = false;
        assert_eq!(release_polling(&mut m), Ok(()));
        assert_eq!((m.mask, m.wrap), (0, 0x8));
    }
    #[test]
    fn abort_repeats_the_disable_without_waiting() {
        let mut m = Model::new(2);
        (m.mask, m.wrap, m.stuck_enabled) = (0x250, 0x9, true);
        let before = m.tick;
        assert_eq!(cancel(&mut m), Err(Error::Busy));
        // Never ENABLE | ABORT: that would enable the controller again.
        assert_eq!(m.enable_writes, [0]);
        assert_eq!((m.mask, m.wrap, m.tick), (0, 0x8, before));
    }

    #[test]
    fn long_read_small_fifo_and_direction_changes() {
        let mut m = Model::new(2);
        let mut a = [0; 65];
        let mut b = [0; 3];
        transfer(
            &mut m,
            &mut [
                Operation::Write(&[7]),
                Operation::Write(&[]),
                Operation::Write(&[8]),
                Operation::Read(&mut a),
                Operation::Read(&mut b),
                Operation::Write(&[9]),
            ],
            2,
        )
        .unwrap();
        assert_eq!(a, core::array::from_fn(|i| i as u8));
        assert_eq!(b, [65, 66, 67]);
        assert_eq!(m.commands[0..2], [7, 8]);
        assert_eq!(m.commands[2], READ | RESTART);
        assert_eq!(m.commands[67], READ);
        assert_eq!(*m.commands.last().unwrap(), 9 | RESTART | STOP);
        assert_eq!(m.commands.iter().filter(|c| **c & STOP != 0).count(), 1);
    }
    #[test]
    fn final_read_stop_and_empty_ops() {
        let mut m = Model::new(1);
        let mut bytes = [0; 33];
        transfer(
            &mut m,
            &mut [Operation::Read(&mut bytes), Operation::Write(&[])],
            1,
        )
        .unwrap();
        assert_eq!(*m.commands.last().unwrap(), READ | STOP);
        assert_eq!(m.max_rx, 1);
        let mut m = Model::new(1);
        transfer(&mut m, &mut [Operation::Read(&mut [])], 1).unwrap();
        assert!(m.commands.is_empty());
    }
    #[test]
    fn aborts_timeout_and_timer_wrap_are_bounded() {
        for (source, expected) in [
            (1, Error::AddressNack),
            (8, Error::DataNack),
            (4097, Error::ArbitrationLost),
            (1 << 14, Error::Abort(1 << 14)),
        ] {
            let mut m = Model::new(2);
            m.raw = 64;
            m.abort = source;
            assert_eq!(
                transfer(&mut m, &mut [Operation::Write(&[1])], 2),
                Err(expected)
            );
            assert!(m.commands.is_empty());
        }
        let mut m = Model::new(2);
        m.stalled = true;
        assert_eq!(
            transfer(&mut m, &mut [Operation::Write(&[1])], 2),
            Err(Error::Timeout)
        );
        assert_timed_out(m.tick, u32::MAX - 50);
        let mut m = Model::new(2);
        m.raw = 2;
        assert_eq!(
            transfer(&mut m, &mut [Operation::Write(&[1])], 2),
            Err(Error::Fifo)
        );
    }
    /// The 64-bit formula before review 7, kept as the reference.
    fn timing_wide(ic_clk_hz: u32, speed: Speed) -> Result<Counts, Error> {
        let (mode, high_ns, low_ns) = match speed {
            Speed::Standard => (1, 4_600, 5_400),
            Speed::Fast => (2, 900, 1_600),
            Speed::FastPlus => (2, 400, 600),
        };
        let cycles = |ns: u32| (u64::from(ns) * u64::from(ic_clk_hz)).div_ceil(1_000_000_000);
        let spklen = cycles(SPIKE_NS).max(1);
        let hcnt = cycles(high_ns).checked_sub(spklen + 7);
        let lcnt = cycles(low_ns).checked_sub(1 + cycles(RISE_NS));
        match (hcnt, lcnt) {
            (Some(h), Some(l)) if h >= spklen + 5 && l >= spklen + 7 && h.max(l) <= 0xffff => {
                Ok(Counts {
                    mode,
                    spklen: spklen as u32,
                    hcnt: h as u32,
                    lcnt: l as u32,
                })
            }
            _ => Err(Error::Invalid),
        }
    }
    #[test]
    fn timings_and_error_kinds() {
        use embedded_hal::i2c::Error as _;
        assert_eq!(reset_bit(1), 1 << 8);
        // CTRL1's clear alias 0x4001_7004 is used by cold bring-up; CTRL0's is one word below.
        assert_eq!(RESETS_SET0, 0x4001_6000);
        assert_eq!(RESETS_CLR0, 0x4001_7000);
        assert_eq!(RESETS_DONE0, 0x4001_4018);
        assert_eq!(reset_bit(6), 1 << 13);
        let c = |hz, speed| {
            let t = timing(hz, speed).unwrap();
            (t.mode, t.spklen, t.hcnt, t.lcnt)
        };
        // 50 MHz: the counts measured as 100.2 / 403.6 / 1011 kHz (HW, i2c-driver step 29).
        assert_eq!(c(50_000_000, Speed::Standard), (1, 3, 220, 264));
        assert_eq!(c(50_000_000, Speed::Fast), (2, 3, 35, 74));
        assert_eq!(c(50_000_000, Speed::FastPlus), (2, 3, 10, 24));
        // 200 MHz: same times, four times the cycles.
        assert_eq!(c(200_000_000, Speed::Standard), (1, 10, 903, 1059));
        assert_eq!(c(200_000_000, Speed::Fast), (2, 10, 163, 299));
        assert_eq!(c(200_000_000, Speed::FastPlus), (2, 10, 63, 99));
        // Period = (HCNT + SPKLEN + 7) + (LCNT + 1 + rise) cycles; DesignWare minimums hold.
        for hz in [50_000_000, 200_000_000] {
            for (speed, scl) in [
                (Speed::Standard, 100_000),
                (Speed::Fast, 400_000),
                (Speed::FastPlus, 1_000_000),
            ] {
                let t = timing(hz, speed).unwrap();
                let rise = hz / 10_000_000;
                let cycles = t.hcnt + t.spklen + 7 + t.lcnt + 1 + rise;
                assert_eq!(hz / cycles, scl);
            }
        }
        // Too slow a clock for the I2C or DesignWare minimums.
        assert_eq!(timing(5_000_000, Speed::FastPlus), Err(Error::Invalid));
        assert_eq!(timing(1_000_000, Speed::Standard), Err(Error::Invalid));
        // Review 7: whole MHz only (32-bit math). Review 8: the whole Result equals the 64-bit
        // formula's for every whole-MHz u32 clock, so a new `Invalid` (or a new `Ok`) shows up.
        let speeds = [Speed::Standard, Speed::Fast, Speed::FastPlus];
        let mut ok = 0;
        for mhz in 0..=u32::MAX / 1_000_000 {
            for speed in speeds {
                let hz = mhz * 1_000_000;
                assert_eq!(timing(hz, speed), timing_wide(hz, speed), "{hz} {speed:?}");
                ok += usize::from(timing(hz, speed).is_ok());
            }
        }
        // Not vacuous: all but the slowest clocks give counts at each speed.
        assert!(ok > 3 * 4000, "{ok}");
        // Not whole MHz: `Invalid`, where the 64-bit formula gave counts.
        for hz in [50_000_500, 199_999_999, 125_000_001] {
            for speed in speeds {
                assert_eq!(timing(hz, speed), Err(Error::Invalid), "{hz} {speed:?}");
                assert!(timing_wide(hz, speed).is_ok(), "{hz} {speed:?}");
            }
        }
        assert_eq!(
            Error::AddressNack.kind(),
            ErrorKind::NoAcknowledge(NoAcknowledgeSource::Address)
        );
        assert_eq!(Error::ArbitrationLost.kind(), ErrorKind::ArbitrationLoss);
    }
}
