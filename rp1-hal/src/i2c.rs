//! Polling, seven-bit DesignWare master (DOC: RP1 §9 and Linux i2c-designware).
//! SCL counts are computed from `ic_clk` = clk_sys as reported by [`crate::clocks::Clocks`]
//! (HW: 50 MHz on XOSC, 200 MHz on PLL_SYS); no shared clock or SCMI vote is changed.
//! External pull-ups are required. Only bank-0 routes are exposed; I2C4..6 are internal.
use crate::{
    clocks::Clocks,
    gpio::{Function, Pin},
    read, write,
};
use embedded_hal::i2c::{ErrorKind, NoAcknowledgeSource, Operation};

use crate::pac::dw_i2c::RegisterBlock;
macro_rules! offsets {
    ($($name:ident = $field:ident),* $(,)?) => {$(const $name: usize = core::mem::offset_of!(RegisterBlock, $field);)*};
}
offsets! {
    CON = con, TAR = tar, DATA = data_cmd, SS_HCNT = ss_scl_hcnt, SS_LCNT = ss_scl_lcnt,
    FS_HCNT = fs_scl_hcnt, FS_LCNT = fs_scl_lcnt, MASK = intr_mask, RAW = raw_intr_stat,
    RX_TL = rx_tl, TX_TL = tx_tl, CLEAR = clr_intr, ENABLE = enable, STATUS = status,
    HOLD = sda_hold, SPKLEN = fs_spklen, ABORT_SOURCE = tx_abrt_source, ENABLE_STATUS = enable_status,
    PARAM = comp_param_1, TYPE = comp_type,
}
const STOP: u32 = 1 << 9;
const RESTART: u32 = 1 << 10;
const READ: u32 = 1 << 8;
const STOP_DET: u32 = 1 << 9;
const TIMEOUT_US: u32 = 100_000;
// RESETS CTRL0 bits 7..13 are I2C0..6 (rp1-regdb resets.toml); CTRL1 bit 8 is SDIO0.
const RESETS: usize = 0x4001_4000;
const RESETS_CLR0: usize = RESETS + 0x3000; // atomic clear alias of CTRL0
const RESETS_DONE0: usize = RESETS + 0x18;
const fn reset_bit(n: u8) -> u32 {
    1 << (7 + n)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Speed {
    Standard,
    Fast,
    FastPlus,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    AddressNack,
    DataNack,
    ArbitrationLost,
    Timeout,
    Invalid,
    Precondition,
    Abort(u32),
    Fifo,
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
    pub trait Pin {}
    impl<const N: u8> Pin for crate::gpio::Pin<N> {}
}
pub trait Sda<const N: u8>: sealed::Pin {}
pub trait Scl<const N: u8>: sealed::Pin {}
macro_rules! routes {
    ($n:literal: $($sda:literal, $scl:literal);+ $(;)?) => {$(
        impl Sda<$n> for Pin<$sda> {}
        impl Scl<$n> for Pin<$scl> {}
    )+};
}
routes!(0: 0, 1; 8, 9);
routes!(1: 2, 3; 10, 11);
routes!(2: 4, 5; 12, 13);
routes!(3: 6, 7; 14, 15; 22, 23);

pub struct Token<const N: u8> {
    _private: (),
}
/// [`Token::init`] failed: the error, and the token and pins back (unchanged) for a retry.
pub struct InitError<const N: u8, const SDA: u8, const SCL: u8> {
    pub error: Error,
    pub token: Token<N>,
    pub sda: Pin<SDA>,
    pub scl: Pin<SCL>,
}
impl<const N: u8, const SDA: u8, const SCL: u8> core::fmt::Debug for InitError<N, SDA, SCL> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.error.fmt(f)
    }
}
impl<const N: u8, const SDA: u8, const SCL: u8> From<InitError<N, SDA, SCL>> for Error {
    fn from(e: InitError<N, SDA, SCL>) -> Self {
        e.error
    }
}
pub struct I2c<const N: u8, const SDA: u8, const SCL: u8> {
    _sda: Pin<SDA, Function<3>>,
    _scl: Pin<SCL, Function<3>>,
    rx_depth: usize,
    usable: bool,
    clocks: Clocks,
}

impl<const N: u8> Token<N> {
    /// # Safety
    /// Generated resources only: sole owner of I2C N, its reset and IRQ, with Linux disabled.
    pub const unsafe fn new() -> Self {
        assert!(N < 7);
        Self { _private: () }
    }
    /// Checks clk_sys and the speed, releases only this controller's reset, then selects the
    /// typed SDA/SCL routes. GPIO bank0 and the raw microsecond timer must already be running
    /// (cold bring-up). On error the token and the pins come back in [`InitError`]; the pins
    /// are not touched before success, and a stale clk_sys or an impossible speed is reported
    /// before the reset is released.
    ///
    /// ```no_run
    /// # use rp1_hal::{clocks::Clocks, gpio::Pin, i2c::{Error, I2c, Speed, Token}};
    /// fn bring_up(token: Token<1>, sda: Pin<2>, scl: Pin<3>, clocks: &Clocks) -> Result<I2c<1, 2, 3>, Error> {
    ///     // Fast mode, else standard mode, else the error through `?`.
    ///     match token.init(sda, scl, clocks, Speed::Fast) {
    ///         Ok(i2c) => Ok(i2c),
    ///         Err(e) => Ok(e.token.init(e.sda, e.scl, clocks, Speed::Standard)?),
    ///     }
    /// }
    /// ```
    pub fn init<const SDA: u8, const SCL: u8>(
        self,
        sda: Pin<SDA>,
        scl: Pin<SCL>,
        clocks: &Clocks,
        speed: Speed,
    ) -> Result<I2c<N, SDA, SCL>, InitError<N, SDA, SCL>>
    where
        Pin<SDA>: Sda<N>,
        Pin<SCL>: Scl<N>,
    {
        self.init_on(
            &mut Mmio {
                base: 0x4007_0000 + N as usize * 0x4000,
            },
            (sda, scl),
            clocks,
            speed,
            |(sda, scl)| (sda.into_i2c(), scl.into_i2c()),
        )
    }
    /// [`Token::init`] on `io`; `route` muxes the pins and runs only once the controller is set up.
    fn init_on<const SDA: u8, const SCL: u8>(
        self,
        io: &mut impl Io,
        (sda, scl): (Pin<SDA>, Pin<SCL>),
        clocks: &Clocks,
        speed: Speed,
        route: impl FnOnce((Pin<SDA>, Pin<SCL>)) -> (Pin<SDA, Function<3>>, Pin<SCL, Function<3>>),
    ) -> Result<I2c<N, SDA, SCL>, InitError<N, SDA, SCL>> {
        match start(io, reset_bit(N), clocks, speed) {
            Ok(rx_depth) => {
                let (_sda, _scl) = route((sda, scl));
                Ok(I2c {
                    _sda,
                    _scl,
                    rx_depth,
                    usable: true,
                    clocks: *clocks,
                })
            }
            Err(error) => Err(InitError {
                error,
                token: self,
                sda,
                scl,
            }),
        }
    }
}

/// The register part of [`Token::init`]; returns the RX FIFO depth. Nothing is written, and
/// the reset stays asserted, unless clk_sys matches and the speed is possible at it.
fn start(io: &mut impl Io, reset: u32, clocks: &Clocks, speed: Speed) -> Result<usize, Error> {
    if !io.sys_current_strict(clocks) {
        return Err(Error::Precondition);
    }
    let counts = timing(clocks.sys_hz(), speed)?;
    io.release_reset(reset)?;
    if io.read(TYPE) != 0x4457_0140 {
        return Err(Error::Precondition);
    }
    disable(io)?;
    io.write(MASK, 0);
    io.read(CLEAR);
    io.write(RX_TL, 0);
    io.write(TX_TL, 0);
    configure(io, counts);
    Ok(((io.read(PARAM) >> 8) & 255) as usize + 1)
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
    while io.read(ENABLE_STATUS) & 1 != 0 {
        if io.now().wrapping_sub(start) >= TIMEOUT_US {
            return Err(Error::Timeout);
        }
    }
    Ok(())
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
        while io.read(STATUS) & (1 << 5) != 0 {
            check(io, start)?;
        }
    }
    Ok(())
}

impl<const N: u8, const SDA: u8, const SCL: u8> I2c<N, SDA, SCL> {
    fn io(&self) -> Mmio {
        Mmio {
            base: 0x4007_0000 + N as usize * 0x4000,
        }
    }
    pub fn set_speed(&mut self, speed: Speed) -> Result<(), Error> {
        let clocks = self.clocks;
        set_speed(&mut self.io(), &clocks, &mut self.usable, speed)
    }
}
impl<const N: u8, const SDA: u8, const SCL: u8> embedded_hal::i2c::ErrorType for I2c<N, SDA, SCL> {
    type Error = Error;
}
impl<const N: u8, const SDA: u8, const SCL: u8> embedded_hal::i2c::I2c for I2c<N, SDA, SCL> {
    /// Empty operations are skipped. Adjacent operations with the same direction continue
    /// without a restart; direction changes restart, and only the final byte gets STOP.
    /// A 100ms interval without progress times out. Failed disable poisons this handle.
    fn transaction(&mut self, address: u8, ops: &mut [Operation<'_>]) -> Result<(), Error> {
        let (clocks, depth) = (self.clocks, self.rx_depth);
        run(
            &mut self.io(),
            &clocks,
            &mut self.usable,
            address,
            ops,
            depth,
        )
    }
}

/// [`I2c::set_speed`]: the strict clk_sys check and the counts first (nothing written on
/// error), then disable and reprogram; a failed disable poisons the handle.
fn set_speed(
    io: &mut impl Io,
    clocks: &Clocks,
    usable: &mut bool,
    speed: Speed,
) -> Result<(), Error> {
    if !*usable || !io.sys_current_strict(clocks) {
        return Err(Error::Precondition);
    }
    let counts = timing(clocks.sys_hz(), speed)?;
    *usable = false;
    disable(io)?;
    configure(io, counts);
    *usable = true;
    Ok(())
}

/// One transaction, one clock check. `Precondition` before any controller access if the
/// handle is poisoned or clk_sys (`ic_clk`) no longer matches `clocks` ([`Clocks::sys_current`]):
/// the SCL counts were computed for it. clk_uart is not read.
fn run(
    io: &mut impl Io,
    clocks: &Clocks,
    usable: &mut bool,
    address: u8,
    ops: &mut [Operation<'_>],
    depth: usize,
) -> Result<(), Error> {
    if address >= 128 {
        return Err(Error::Invalid);
    }
    if !*usable || !io.sys_current(clocks) {
        return Err(Error::Precondition);
    }
    if ops.iter().all(|op| operation_len(op) == 0) {
        return Ok(());
    }
    *usable = false;
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
    *usable = cleanup.is_ok();
    io.read(CLEAR);
    result.and(cleanup)
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
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
                        2 | if self.rx.is_empty() { 0 } else { 8 }
                    }
                }
                DATA => u32::from(self.rx.pop_front().expect("RX underflow")),
                TYPE => 0x4457_0140,
                // IC_RX_BUFFER_DEPTH - 1 in bits 15:8.
                PARAM => (self.depth as u32 - 1) << 8,
                _ => 0,
            }
        }
        fn write(&mut self, offset: usize, value: u32) {
            self.writes += 1;
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
            self.tick = self.tick.wrapping_add(1000);
            self.tick
        }
        fn sys_current(&mut self, clocks: &Clocks) -> bool {
            self.fast += 1;
            self.sys == Some(clocks.sys_hz())
        }
        fn sys_current_strict(&mut self, clocks: &Clocks) -> bool {
            self.strict += 1;
            !self.strict_stale && self.sys == Some(clocks.sys_hz())
        }
        fn release_reset(&mut self, bit: u32) -> Result<(), Error> {
            self.resets.push(bit);
            Ok(())
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
        let mut usable = true;
        assert_eq!(
            run(
                &mut m,
                &CLOCKS,
                &mut usable,
                0x2d,
                &mut [Operation::Write(&[1])],
                2
            ),
            Err(Error::Precondition)
        );
        assert!(usable && m.writes == 0 && m.commands.is_empty());
        assert_eq!((m.fast, m.strict), (1, 0));
        m.sys = Some(200_000_000);
        assert_eq!(
            run(
                &mut m,
                &CLOCKS,
                &mut usable,
                0x2d,
                &mut [Operation::Write(&[1])],
                2
            ),
            Ok(())
        );
        assert!(usable);
        assert_eq!(m.commands, [1 | STOP]);
        // One check for the whole transaction, however many operations it has.
        let checks = m.fast;
        let mut r = [0; 3];
        run(
            &mut m,
            &CLOCKS,
            &mut usable,
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
        let mut poisoned = false;
        assert_eq!(
            run(&mut m, &CLOCKS, &mut poisoned, 0x2d, &mut [], 2),
            Err(Error::Precondition)
        );
        assert_eq!(
            run(&mut m, &CLOCKS, &mut usable, 0x80, &mut [], 2),
            Err(Error::Invalid)
        );
    }
    type Pins = (Pin<2>, Pin<3>);
    fn init_model(
        token: Token<1>,
        m: &mut Model,
        pins: Pins,
        clocks: &Clocks,
        speed: Speed,
        routed: &mut bool,
    ) -> Result<I2c<1, 2, 3>, InitError<1, 2, 3>> {
        token.init_on(m, pins, clocks, speed, |(sda, scl)| {
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
        let mut pins: Pins = unsafe { (Pin::new(), Pin::new()) };
        for (sys, clocks, speed, want) in [
            (Some(50_000_000), CLOCKS, Speed::Fast, Error::Precondition),
            (None, CLOCKS, Speed::Fast, Error::Precondition),
            (Some(1_000_000), slow, Speed::Standard, Error::Invalid),
        ] {
            let mut m = Model::new(16);
            m.sys = sys;
            let mut routed = false;
            let InitError {
                error,
                token: t,
                sda,
                scl,
            } = init_model(token, &mut m, pins, &clocks, speed, &mut routed)
                .err()
                .expect("init must fail");
            assert_eq!(error, want);
            assert!(m.resets.is_empty() && m.writes == 0 && !routed);
            (token, pins) = (t, (sda, scl));
        }
        let mut m = Model::new(16);
        let mut routed = false;
        let i2c =
            init_model(token, &mut m, pins, &CLOCKS, Speed::Fast, &mut routed).expect("retry");
        assert!(routed && i2c.usable);
        assert_eq!(
            (m.resets.as_slice(), i2c.rx_depth, m.strict, m.fast),
            (&[reset_bit(1)][..], 16, 1, 0)
        );
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
        let pins: Pins = unsafe { (Pin::new(), Pin::new()) };
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
        assert_eq!(e.error, Error::Precondition);
        assert!(m.resets.is_empty() && m.writes == 0 && !routed);
        assert_eq!((m.strict, m.fast), (1, 0));

        let mut m = Model::new(16);
        m.strict_stale = true;
        let mut usable = true;
        assert_eq!(
            set_speed(&mut m, &CLOCKS, &mut usable, Speed::Standard),
            Err(Error::Precondition)
        );
        assert!(usable && m.writes == 0);
        assert_eq!((m.strict, m.fast), (1, 0));
        m.strict_stale = false;
        assert_eq!(
            set_speed(&mut m, &CLOCKS, &mut usable, Speed::Standard),
            Ok(())
        );
        assert!(usable && m.writes > 0);
        assert_eq!((m.strict, m.fast), (2, 0));
    }

    /// `?` on an init error inside a function returning the bus error (compile check).
    #[allow(dead_code)]
    fn init_with_question_mark(
        token: Token<1>,
        (sda, scl): Pins,
        clocks: &Clocks,
    ) -> Result<I2c<1, 2, 3>, Error> {
        Ok(token.init(sda, scl, clocks, Speed::Standard)?)
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
        assert_eq!(m.tick.wrapping_sub(u32::MAX - 50), 101_000);
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
