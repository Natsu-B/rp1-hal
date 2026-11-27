//! Seven-bit DesignWare controller (DOC: RP1 §9 and Linux i2c-designware).
//!
//! | Instance | Synchronous adapters |
//! |---|---|
//! | I2C1 with a generated IRQ | [`I2c::blocking`] (IRQ + WFE), explicit [`I2c::polling`] |
//! | I2C0, unmanaged I2C1, I2C2–3 | Explicit [`I2c::polling`] |
//! | I2C4–6 | No bank-0 routes |
//!
//! SCL counts are computed from `ic_clk` = clk_sys as reported by [`crate::clocks::Clocks`]
//! (HW: 50 MHz on XOSC, 200 MHz on PLL_SYS); no shared clock or SCMI vote is changed.
//! External pull-ups are required. Only bank-0 routes are exposed; I2C4..6 are internal.
use crate::{
    clocks::Clocks,
    gpio::{Alternate, Pin},
    linux_blocks::Mmio as ResetMmio,
    read,
    scmi::Registers,
    timer::{self, Deadline, Duration, Instant, Timer, WaitConfig},
    write,
};
use core::marker::PhantomData;
use embedded_hal::i2c::{ErrorKind, NoAcknowledgeSource, Operation};

use crate::pac::dw_i2c::{
    RegisterBlock,
    intr::{RX_FULL, RX_OVER, RX_UNDER, STOP_DET, TX_ABRT, TX_EMPTY, TX_OVER},
    irq_wrap,
};

mod irq_wait;
pub use crate::Try;
pub use irq_wait::{__on_interrupt, Blocking, Irq};
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
const FIFO_ERRORS: u32 = RX_UNDER | RX_OVER | TX_OVER;
const ERRORS: u32 = TX_ABRT | FIFO_ERRORS;
const TX_NOT_FULL: u32 = 1 << 1;
const RX_NOT_EMPTY: u32 = 1 << 3;
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
    /// The deadline exceeds the supported comparison horizon.
    InvalidDeadline,
    /// Synchronous I/O requires unmasked thread mode.
    WrongContext,
    /// The managed device or wait-timer IRQ cannot be delivered.
    IrqUnavailable,
    /// The dedicated wait alarm could not be retired.
    AlarmUnavailable,
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
impl From<timer::Error> for Error {
    fn from(error: timer::Error) -> Self {
        match error {
            timer::Error::Timeout => Self::Timeout,
            timer::Error::InvalidDeadline => Self::InvalidDeadline,
            timer::Error::Masked | timer::Error::WrongContext => Self::WrongContext,
            timer::Error::IrqUnavailable => Self::IrqUnavailable,
            timer::Error::AlarmUnavailable => Self::AlarmUnavailable,
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

/// Marker for a controller whose IRQ is available to the application.
pub struct NoIrq;

/// Initialized controller. Use [`I2c::blocking`] on managed I2C1, or select [`I2c::polling`].
pub struct I2c<const N: u8, P: CompletePins<N>, Role = Controller, I = NoIrq> {
    pins: P::Active,
    rx_depth: usize,
    state: State,
    clocks: Clocks,
    _role: PhantomData<Role>,
    irq: I,
}

/// The initialization error retains the block and its pins during recovery.
pub type InitError<const N: u8, P, I = NoIrq> =
    crate::InitError<Parts<N, P, I>, Error, crate::Recovery<Parts<N, P, I>>>;

/// Explicit polling access to the I²C controller.
pub struct Polling<'a, const N: u8, P: CompletePins<N>, I = NoIrq> {
    bus: &'a mut I2c<N, P, Controller, I>,
    config: WaitConfig,
}

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

type PinTuple<const SDA: u8, const SCL: u8, M = crate::gpio::Unconfigured> =
    (Pin<SDA, M>, Pin<SCL, M>);

impl<const N: u8, const SDA: u8, const SCL: u8, I> Parts<N, PinTuple<SDA, SCL>, I>
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
    ) -> Result<I2c<N, PinTuple<SDA, SCL>, Controller, I>, InitError<N, PinTuple<SDA, SCL>, I>>
    {
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
    ) -> Result<I2c<N, PinTuple<SDA, SCL>, Controller, I>, InitError<N, PinTuple<SDA, SCL>, I>>
    {
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
        // Refill an entire FIFO per wake at the cost of possible SCL stretching.
        io.write(TX_TL, 0);
        configure(io, counts);
        Ok(I2c {
            pins: route(self.pins.0),
            rx_depth: ((io.read(PARAM) >> 8) & 255) as usize + 1,
            state: State::Ready,
            clocks: *clocks,
            _role: PhantomData,
            irq: self.irq,
        })
    }
}

impl<const N: u8, P: CompletePins<N>, I> InitError<N, P, I> {
    /// Request I²C reset and return the parts once reset is confirmed.
    pub fn try_recover(self) -> Result<Parts<N, P, I>, Self> {
        self.try_recover_with(&mut ResetMmio)
    }

    fn try_recover_with(self, io: &mut impl Registers) -> Result<Parts<N, P, I>, Self> {
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
    fn instant(&mut self) -> Instant;
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
    fn instant(&mut self) -> Instant {
        Timer::new().now()
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

/// Cancellation also covers a native transfer whose handle was forgotten.
fn cancel(io: &mut impl Io) -> Result<(), Error> {
    request_stop(io, true);
    mask_interrupts(io);
    if stopped(io) {
        Ok(())
    } else {
        Err(Error::Busy)
    }
}

fn fault(io: &mut impl Io, raw: u32) -> Result<(), Error> {
    if raw & TX_ABRT != 0 {
        return Err(abort_error(io.read(ABORT_SOURCE)));
    }
    if raw & FIFO_ERRORS != 0 {
        return Err(Error::Fifo);
    }
    Ok(())
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

#[derive(Debug, PartialEq, Eq)]
enum Step {
    Again,
    Pending(u32, u32),
    Complete,
}

/// Fixed budget per call, including when the operation slice contains many empty buffers.
const SERVICE_BUDGET: usize = 16;

struct Cursor {
    scan: usize,
    last: Option<usize>,
    index: usize,
    sent: usize,
    received: usize,
    previous: Option<bool>,
}

impl Cursor {
    fn new(count: usize) -> Self {
        Self {
            scan: count,
            last: None,
            index: 0,
            sent: 0,
            received: 0,
            previous: None,
        }
    }

    fn find_last(&mut self, ops: &[Operation<'_>]) -> bool {
        for _ in 0..SERVICE_BUDGET {
            if self.scan == 0 {
                return true;
            }
            self.scan -= 1;
            if operation_len(&ops[self.scan]) != 0 {
                self.last = Some(self.scan);
                return true;
            }
        }
        false
    }

    /// Move a bounded group of FIFO entries. The waiter only observes readiness.
    /// The thread or native caller owns all command submission and RX buffer access.
    #[rp1_rt::isram]
    fn step(
        &mut self,
        io: &mut impl Io,
        ops: &mut [Operation<'_>],
        depth: usize,
        send: bool,
    ) -> Result<Step, Error> {
        let Some(last) = self.last else {
            return Ok(Step::Complete);
        };
        for _ in 0..SERVICE_BUDGET {
            let raw = io.read(RAW);
            fault(io, raw)?;
            if self.index > last {
                return Ok(if raw & STOP_DET == 0 {
                    Step::Pending(STOP_DET, 0)
                } else {
                    Step::Complete
                });
            }

            let op = &mut ops[self.index];
            let len = operation_len(op);
            if len == 0 {
                self.index += 1;
                continue;
            }
            let is_read = matches!(op, Operation::Read(_));
            let status = io.read(STATUS);
            let mut advanced = false;
            if let Operation::Read(bytes) = op
                && self.received < self.sent
                && status & RX_NOT_EMPTY != 0
            {
                bytes[self.received] = io.read(DATA) as u8;
                self.received += 1;
                advanced = true;
            }
            if send
                && self.sent < len
                && status & TX_NOT_FULL != 0
                && (!is_read || self.sent - self.received < depth)
            {
                let byte = match op {
                    Operation::Write(bytes) => bytes[self.sent],
                    _ => 0,
                };
                io.write(
                    DATA,
                    command(
                        is_read,
                        self.sent == 0 && self.previous.is_some_and(|r| r != is_read),
                        self.index == last && self.sent + 1 == len,
                        byte,
                    ),
                );
                self.sent += 1;
                advanced = true;
            }
            if self.sent == len && (!is_read || self.received == len) {
                self.previous = Some(is_read);
                self.index += 1;
                self.sent = 0;
                self.received = 0;
            } else if !advanced {
                // With outstanding reads, RX data makes room under the credit limit.
                // A TX_EMPTY condition at threshold zero always implies TX has room.
                let outstanding = self.sent - self.received;
                return Ok(if is_read && outstanding != 0 {
                    let threshold = outstanding.min(depth.div_ceil(2)) - 1;
                    Step::Pending(RX_FULL, threshold as u32)
                } else {
                    Step::Pending(TX_EMPTY, 0)
                });
            }
        }
        if self.index > last {
            let raw = io.read(RAW);
            fault(io, raw)?;
            if raw & STOP_DET != 0 {
                return Ok(Step::Complete);
            }
        }
        Ok(Step::Again)
    }
}

fn scan_ops(io: &mut impl Io, ops: &[Operation<'_>], end: Deadline) -> Result<Cursor, Error> {
    let mut cursor = Cursor::new(ops.len());
    while !cursor.find_last(ops) {
        end.check(io.instant())?;
    }
    Ok(cursor)
}

fn transfer<I: Io>(
    io: &mut I,
    ops: &mut [Operation<'_>],
    mut cursor: Cursor,
    depth: usize,
    end: Deadline,
    mut wait: impl FnMut(&mut I, u32, u32, Deadline) -> Result<(), Error>,
) -> Result<(), Error> {
    loop {
        let expired = io.instant() >= end.instant();
        match cursor.step(io, ops, depth, !expired)? {
            Step::Complete => return Ok(()),
            _ if expired => return Err(Error::Timeout),
            Step::Again => {}
            Step::Pending(mask, threshold) => match wait(io, mask, threshold, end) {
                Err(Error::Timeout) => {
                    // Deadline wake gets one final batch of RX and STOP, with no new commands.
                    return match cursor.step(io, ops, depth, false)? {
                        Step::Complete => Ok(()),
                        _ => Err(Error::Timeout),
                    };
                }
                result => result?,
            },
        }
    }
}

impl<const N: u8, P: CompletePins<N>, I> I2c<N, P, Controller, I> {
    fn io(&self) -> Mmio {
        Mmio::new(N)
    }

    /// Selects the synchronous polling engine.
    pub fn polling(&mut self) -> Polling<'_, N, P, I> {
        Polling {
            bus: self,
            config: WaitConfig::new(),
        }
    }

    /// Poll a transaction using a checked seven-bit address.
    /// Use [`Self::polling`] for the embedded-hal transaction API.
    pub fn transaction_polling(
        &mut self,
        address: Address7,
        ops: &mut [Operation<'_>],
    ) -> Result<(), Error> {
        self.poll_default(address.0, ops, WaitConfig::new())
    }

    /// Explicitly poll until the supplied absolute deadline in unmasked thread mode.
    pub fn transaction_polling_until(
        &mut self,
        address: Address7,
        ops: &mut [Operation<'_>],
        end: Deadline,
    ) -> Result<(), Error> {
        poll_context()?;
        self.poll_transaction_with(&mut self.io(), address.0, ops, end)
    }

    fn poll_default(
        &mut self,
        address: u8,
        ops: &mut [Operation<'_>],
        config: WaitConfig,
    ) -> Result<(), Error> {
        poll_context()?;
        self.poll_transaction_with(&mut self.io(), address, ops, default_deadline(config)?)
    }

    fn poll_transaction_with(
        &mut self,
        io: &mut impl Io,
        address: u8,
        ops: &mut [Operation<'_>],
        end: Deadline,
    ) -> Result<(), Error> {
        admit(io, &self.clocks, self.state, address, end)?;
        let cursor = scan_ops(io, ops, end)?;
        if cursor.last.is_none() {
            return Ok(());
        }
        end.check(io.instant())?;
        // A handler would race the transfer loop for the FIFO and the status it polls.
        let mask = io.read(MASK);
        io.write(MASK, 0);
        let result = match begin(io, address) {
            Ok(()) => {
                self.state = State::Poisoned;
                let result = transfer(io, ops, cursor, self.rx_depth, end, |_, _, _, _| Ok(()));
                request_stop(io, result.is_err());
                let start = io.now();
                let cleanup = loop {
                    if stopped(io) {
                        break Ok(());
                    }
                    if io.now().wrapping_sub(start) >= CLEANUP_US {
                        break Err(Error::Timeout);
                    }
                };
                finish(io, &mut self.state, result, cleanup)
            }
            Err(error) => Err(error),
        };
        io.write(MASK, mask);
        result
    }

    /// Admit a nonblocking transaction with one absolute deadline. The handle
    /// owns the driver and buffers until completion or cancellation. Each
    /// `try_complete` services a fixed number of FIFO entries and never waits.
    pub fn try_start_transaction<'a, 'buf>(
        &'a mut self,
        address: Address7,
        ops: &'a mut [Operation<'buf>],
        end: Deadline,
    ) -> Result<Transfer<'a, 'buf, N, P, I>, Error> {
        admit(&mut self.io(), &self.clocks, self.state, address.0, end)?;
        let cursor = Cursor::new(ops.len());
        Ok(Transfer {
            bus: self,
            cursor,
            ops,
            end,
            phase: Phase::Scan(address),
        })
    }

    /// Request abort/disable without waiting. A target holding SCL can keep
    /// the controller active. Use [`Self::try_recover`] or [`Self::try_free`]
    /// after the controller stops.
    pub fn try_cancel(&mut self) -> Result<(), Error> {
        self.cancel_with(&mut self.io())
    }

    fn cancel_with(&mut self, io: &mut impl Io) -> Result<(), Error> {
        self.state = State::Stopping;
        cancel(io)
    }

    /// Confirm that cancellation has stopped the controller, then clear stale sources.
    /// A subsequent transaction rechecks the clock. This never changes the pin mux.
    pub fn try_recover(&mut self) -> Result<(), Error> {
        self.recover_with(&mut self.io())
    }

    fn recover_with(&mut self, io: &mut impl Io) -> Result<(), Error> {
        if !stopped(io) {
            return Err(Error::Busy);
        }
        mask_interrupts(io);
        io.read(CLEAR);
        self.state = State::Ready;
        Ok(())
    }
}

impl<const N: u8, P: CompletePins<N>> I2c<N, P> {
    /// Unmasks the [`intr`](crate::pac::dw_i2c::intr) bits in `mask` and opens the RP1 wrapper
    /// that routes the controller interrupt to the cores; the NVIC line comes from the generated
    /// `irq` resources. Polling transactions mask every interrupt while they run and restore
    /// the mask afterwards.
    pub fn enable_interrupt(&mut self, mask: u32) {
        if matches!(self.state, State::Ready) {
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
impl<const N: u8, const SDA: u8, const SCL: u8, I> I2c<N, PinTuple<SDA, SCL>, Controller, I>
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
    pub fn try_free(
        self,
    ) -> Result<Parts<N, PinTuple<SDA, SCL>, I>, crate::FreeError<Self, Error>> {
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
    ) -> Result<Parts<N, PinTuple<SDA, SCL>, I>, crate::FreeError<Self, Error>> {
        if let Err(error) = poll_context() {
            return Err(crate::FreeError {
                driver: self,
                error,
            });
        }
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

    fn into_parts(self) -> Parts<N, PinTuple<SDA, SCL>, I> {
        let (sda, scl) = self.pins;
        Parts {
            token: Token {
                _private: PhantomData,
            },
            pins: Pins::from_required((sda.release(), scl.release())),
            irq: self.irq,
        }
    }
}
impl<const N: u8, P: CompletePins<N>, I> Polling<'_, N, P, I> {
    pub fn with_wait_config(mut self, config: WaitConfig) -> Self {
        self.config = config;
        self
    }

    /// Sets the bus speed in unmasked thread mode after confirming disable.
    pub fn set_speed(&mut self, speed: Speed) -> Result<(), Error> {
        self.bus.state.ready()?;
        poll_context()?;
        let clocks = self.bus.clocks;
        set_speed(&mut self.bus.io(), &clocks, &mut self.bus.state, speed)
    }
}

impl<const N: u8, P: CompletePins<N>, I> embedded_hal::i2c::ErrorType for Polling<'_, N, P, I> {
    type Error = Error;
}
impl<const N: u8, P: CompletePins<N>, I> embedded_hal::i2c::I2c for Polling<'_, N, P, I> {
    /// Skip empty buffers, restart on direction changes and issue STOP on the final byte.
    /// After a fault other than NACK, confirm recovery before reusing the controller.
    fn transaction(&mut self, address: u8, ops: &mut [Operation<'_>]) -> Result<(), Error> {
        self.bus.poll_default(address, ops, self.config)
    }
}

fn default_deadline(config: WaitConfig) -> Result<Deadline, Error> {
    Ok(Deadline::after(
        config.timeout_or(Duration::from_ticks(TIMEOUT_US)),
    )?)
}

fn poll_context() -> Result<(), Error> {
    rp1_rt::interrupt::ThreadGuard::new()
        .map(|_| ())
        .map_err(timer::Error::from)
        .map_err(Error::from)
}

enum Phase {
    Scan(Address7),
    Transfer,
    Disable(Result<(), Error>),
    Done(Result<(), Error>),
    Cancelled(bool),
}

/// CPU-driven I²C transaction. Dropping an active transfer requests stop
/// and releases the buffers. The caller must confirm recovery before reusing the bus.
pub struct Transfer<'a, 'buf, const N: u8, P: CompletePins<N>, I = NoIrq> {
    bus: &'a mut I2c<N, P, Controller, I>,
    cursor: Cursor,
    ops: &'a mut [Operation<'buf>],
    end: Deadline,
    phase: Phase,
}

impl<const N: u8, P: CompletePins<N>, I> Transfer<'_, '_, N, P, I> {
    /// Service one batch. A NACK permits reuse once disable and idle are confirmed.
    pub fn try_complete(&mut self) -> Result<Try<()>, Error> {
        self.complete_with(&mut self.bus.io())
    }

    fn complete_with(&mut self, io: &mut impl Io) -> Result<Try<()>, Error> {
        let result = self.step(io);
        if result.is_err() && matches!(self.phase, Phase::Transfer | Phase::Disable(_)) {
            let _ = self.cancel_with(io);
        }
        result
    }

    fn step(&mut self, io: &mut impl Io) -> Result<Try<()>, Error> {
        let expired = io.instant() >= self.end.instant();
        match self.phase {
            Phase::Done(result) => return result.map(Try::Ready),
            Phase::Cancelled(_) => return Err(Error::Busy),
            Phase::Scan(address) => {
                if expired {
                    return Err(Error::Timeout);
                }
                if !self.cursor.find_last(self.ops) {
                    return Ok(Try::WouldBlock);
                }
                if self.cursor.last.is_none() {
                    self.phase = Phase::Done(Ok(()));
                    return Ok(Try::Ready(()));
                }
                self.end.check(io.instant())?;
                // Suppress raw delivery before taking FIFO and latched status ownership.
                if !stopped(io) {
                    return Err(Error::Busy);
                }
                mask_interrupts(io);
                start(io, address.0);
                self.bus.state = State::Poisoned;
                self.phase = Phase::Transfer;
            }
            _ => {}
        }
        let result = match self.phase {
            Phase::Disable(result) => result,
            _ => match self.cursor.step(io, self.ops, self.bus.rx_depth, !expired) {
                Ok(Step::Complete) => Ok(()),
                Ok(_) if expired => return Err(Error::Timeout),
                Ok(_) => return Ok(Try::WouldBlock),
                Err(error @ (Error::AddressNack | Error::DataNack)) => Err(error),
                Err(error) => return Err(error),
            },
        };
        if matches!(self.phase, Phase::Transfer) {
            request_stop(io, result.is_err());
            self.phase = Phase::Disable(result);
        }
        if !stopped(io) {
            return if expired {
                result.and(Err(Error::Timeout))
            } else {
                Ok(Try::WouldBlock)
            };
        }
        let result = finish(io, &mut self.bus.state, result, Ok(()));
        self.phase = Phase::Done(result);
        result.map(Try::Ready)
    }

    /// Retire this transfer and request disable without waiting. If the
    /// controller remains active, retry until it stops or release the handle
    /// and call `I2c::try_recover` after the controller stops.
    pub fn try_cancel(&mut self) -> Result<(), Error> {
        self.cancel_with(&mut self.bus.io())
    }

    fn cancel_with(&mut self, io: &mut impl Io) -> Result<(), Error> {
        match self.phase {
            Phase::Done(_) => Ok(()),
            Phase::Scan(_) | Phase::Cancelled(false) => {
                self.phase = Phase::Cancelled(false);
                Ok(())
            }
            _ => {
                self.phase = Phase::Cancelled(true);
                self.bus.cancel_with(io)
            }
        }
    }
}

impl<const N: u8, P: CompletePins<N>, I> Drop for Transfer<'_, '_, N, P, I> {
    fn drop(&mut self) {
        if matches!(self.phase, Phase::Transfer | Phase::Disable(_)) {
            let _ = self.bus.try_cancel();
        }
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

/// One transaction, one clock check. The SCL counts are for clk_sys, not clk_uart.
fn admit(
    io: &mut impl Io,
    clocks: &Clocks,
    state: State,
    address: u8,
    end: Deadline,
) -> Result<(), Error> {
    if matches!(state, State::Stopping) {
        return Err(Error::Busy);
    }
    Address7::new(address)?;
    state.ready()?;
    end.check(io.instant())?;
    if !io.sys_current(clocks) {
        return Err(Error::Precondition);
    }
    Ok(())
}

fn begin(io: &mut impl Io, address: u8) -> Result<(), Error> {
    if !stopped(io) {
        return Err(Error::Busy);
    }
    start(io, address);
    Ok(())
}

fn start(io: &mut impl Io, address: u8) {
    io.read(CLEAR);
    io.write(TAR, u32::from(address));
    io.write(ENABLE, 1);
}

fn request_stop(io: &mut impl Io, abort: bool) {
    // IC_ENABLE.ABORT is only meaningful while ENABLE is set. Never re-enable
    // a controller while a preceding disable waits for a target to release SCL.
    if abort && io.read(ENABLE) & 1 != 0 {
        io.write(ENABLE, 3);
    }
    io.write(ENABLE, 0);
}

fn finish(
    io: &mut impl Io,
    state: &mut State,
    result: Result<(), Error>,
    cleanup: Result<(), Error>,
) -> Result<(), Error> {
    if cleanup.is_ok() {
        io.read(CLEAR);
        // NACK completes on the wire. After other faults, require the caller to
        // confirm recovery explicitly even when the controller is already disabled.
        if matches!(result, Ok(()) | Err(Error::AddressNack | Error::DataNack)) {
            *state = State::Ready;
        }
    }
    result.and(cleanup)
}

// Give disable and master idle time to settle after a transfer deadline.
const CLEANUP_US: u32 = 1_000;

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
        fn model_instant(&self) -> Instant {
            Instant::from_ticks(
                u64::from(u32::MAX - 50) + u64::from(self.tick.wrapping_sub(u32::MAX - 50)),
            )
        }

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
                CLEAR => core::mem::take(&mut self.raw),
                ENABLE => self.enable_writes.last().copied().unwrap_or(0),
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
        fn instant(&mut self) -> Instant {
            let instant = self.model_instant();
            self.now();
            instant
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
    fn run(
        io: &mut Model,
        bus: &mut I2c<1, Pins1>,
        address: u8,
        ops: &mut [Operation<'_>],
    ) -> Result<(), Error> {
        let end = Deadline::after_ticks(io.model_instant(), TIMEOUT_US).unwrap();
        bus.poll_transaction_with(io, address, ops, end)
    }

    fn transfer(io: &mut Model, ops: &mut [Operation<'_>], depth: usize) -> Result<(), Error> {
        let end = Deadline::after_ticks(io.model_instant(), TIMEOUT_US).unwrap();
        let mut cursor = Cursor::new(ops.len());
        while !cursor.find_last(ops) {}
        super::transfer(io, ops, cursor, depth, end, |_, _, _, _| Ok(()))
    }

    const CLOCKS: crate::clocks::Clocks = crate::clocks::Clocks::for_test(200_000_000, 50_000_000);
    /// Review 6: PLL_SYS / CLK_UART unchanged but clk_sys elsewhere fails before any
    /// controller access, and the handle stays usable.
    #[test]
    fn stale_sys_clock_fails_before_the_controller_is_touched() {
        let mut m = Model::new(2);
        let mut bus = model_bus(&mut m);
        m.writes = 0;
        m.strict = 0;
        m.sys = Some(50_000_000);
        m.strict_stale = true;
        assert_eq!(
            run(&mut m, &mut bus, 0x2d, &mut [Operation::Write(&[1])]),
            Err(Error::Precondition)
        );
        assert!(bus.state.ready().is_ok() && m.writes == 0 && m.commands.is_empty());
        assert_eq!((m.fast, m.strict), (1, 0));
        m.sys = Some(200_000_000);
        assert_eq!(
            run(&mut m, &mut bus, 0x2d, &mut [Operation::Write(&[1])]),
            Ok(())
        );
        assert!(bus.state.ready().is_ok());
        assert_eq!(m.commands, [1 | STOP]);
        // One check for the whole transaction, however many operations it has.
        let checks = m.fast;
        let mut r = [0; 3];
        run(
            &mut m,
            &mut bus,
            0x2d,
            &mut [
                Operation::Write(&[1, 2]),
                Operation::Read(&mut r),
                Operation::Write(&[3]),
            ],
        )
        .unwrap();
        assert_eq!((m.fast, m.strict), (checks + 1, 0));
    }

    #[test]
    fn polling_rejects_invalid_address_before_poisoned_state() {
        let mut model = Model::new(2);
        let mut bus = model_bus(&mut model);
        bus.state = State::Poisoned;
        assert_eq!(
            run(&mut model, &mut bus, 0x80, &mut []),
            Err(Error::Invalid)
        );
        assert_eq!(
            run(&mut model, &mut bus, 0x2d, &mut []),
            Err(Error::Precondition)
        );
        bus.state = State::Stopping;
        assert_eq!(run(&mut model, &mut bus, 0x80, &mut []), Err(Error::Busy));
        assert_eq!(model.fast, 0);
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

    fn model_bus(model: &mut Model) -> I2c<1, Pins1> {
        init_model(
            unsafe { Token::new() },
            model,
            unsafe { (Pin::new(), Pin::new()) },
            &CLOCKS,
            Speed::Standard,
            &mut false,
        )
        .unwrap()
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
        assert_eq!(run(&mut model, &mut bus, 0x2d, &mut []), Err(Error::Busy));
        assert_eq!(
            embedded_hal::i2c::I2c::transaction(&mut bus.polling(), 0x2d, &mut []),
            Err(Error::WrongContext)
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
        run(&mut m, &mut i2c, 0x2d, ops).unwrap();
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
    #[test]
    fn native_nack_returns_to_ready_after_idle() {
        for (source, error) in [(1, Error::AddressNack), (1 << 3, Error::DataNack)] {
            let mut model = Model::new(2);
            let mut bus = model_bus(&mut model);
            let end = Deadline::after_ticks(model.model_instant(), TIMEOUT_US).unwrap();
            let mut ops = [Operation::Write(&[1])];
            let mut transfer = Transfer {
                bus: &mut bus,
                cursor: Cursor::new(ops.len()),
                ops: &mut ops,
                end,
                phase: Phase::Scan(Address7::new_const::<0x2d>()),
            };
            model.stalled = true;
            assert_eq!(transfer.complete_with(&mut model), Ok(Try::WouldBlock));
            model.stuck_enabled = true;
            model.raw = TX_ABRT | STOP_DET;
            model.abort = source;
            assert_eq!(transfer.complete_with(&mut model), Ok(Try::WouldBlock));
            assert_eq!(model.raw, TX_ABRT | STOP_DET);

            model.stuck_enabled = false;
            assert_eq!(transfer.complete_with(&mut model), Err(error));
            assert_eq!(transfer.complete_with(&mut model), Err(error));
            drop(transfer);
            assert_eq!(model.raw, 0);
            model.stalled = false;
            assert_eq!(run(&mut model, &mut bus, 0x2d, &mut ops), Ok(()));
        }
    }

    #[test]
    fn native_cancel_waits_for_idle_before_rearm_and_clears_stale_stop() {
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
        let end = Deadline::after_ticks(model.model_instant(), TIMEOUT_US).unwrap();
        let mut ops = [Operation::Write(&[1])];
        let mut transfer = Transfer {
            bus: &mut bus,
            cursor: Cursor::new(ops.len()),
            ops: &mut ops,
            end,
            phase: Phase::Scan(Address7::new_const::<0x2d>()),
        };
        model.stalled = true;
        assert_eq!(transfer.complete_with(&mut model), Ok(Try::WouldBlock));
        model.stuck_enabled = true;
        model.master_active = true;
        assert_eq!(transfer.cancel_with(&mut model), Err(Error::Busy));
        drop(transfer);
        assert_eq!(bus.state.ready(), Err(Error::Busy));
        model.raw = STOP_DET | TX_ABRT;
        model.abort = 1;
        assert_eq!(bus.recover_with(&mut model), Err(Error::Busy));
        assert_eq!(model.raw, STOP_DET | TX_ABRT);
        model.stuck_enabled = false;
        model.master_active = false;
        assert_eq!(bus.recover_with(&mut model), Ok(()));
        assert_eq!(model.raw, 0);
        model.stalled = false;
        assert_eq!(run(&mut model, &mut bus, 0x2d, &mut ops), Ok(()));
        assert_eq!(model.commands, [1 | STOP]);
    }

    #[test]
    fn final_probe_cannot_queue_a_command_after_the_deadline() {
        let mut model = Model::new(2);
        let mut ops = [Operation::Write(&[1, 2])];
        let end = Deadline::after_ticks(model.model_instant(), TIMEOUT_US).unwrap();
        let mut cursor = Cursor::new(ops.len());
        assert!(cursor.find_last(&ops));
        model.stalled = true;
        assert_eq!(
            super::transfer(&mut model, &mut ops, cursor, 2, end, |io, mask, _, _| {
                assert_eq!(mask, TX_EMPTY);
                io.stalled = false;
                Err(Error::Timeout)
            }),
            Err(Error::Timeout)
        );
        assert!(model.commands.is_empty());
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
