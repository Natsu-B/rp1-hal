//! Polling, seven-bit DesignWare master (DOC: RP1 §9 and Linux i2c-designware).
//! SCL counts are computed from `ic_clk` = clk_sys as reported by [`crate::clocks::Clocks`]
//! (HW: 50 MHz on XOSC, 200 MHz on PLL_SYS); no shared clock or SCMI vote is changed.
//! External pull-ups are required. Only bank-0 routes are exposed; I2C4..6 are internal.
use crate::{
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
    clocks: crate::clocks::Clocks,
}

impl<const N: u8> Token<N> {
    /// # Safety
    /// Generated resources only: sole owner of I2C N, its reset and IRQ, with Linux disabled.
    pub const unsafe fn new() -> Self {
        assert!(N < 7);
        Self { _private: () }
    }
    /// Checks the clocks and the speed, releases only this controller's reset, then selects the
    /// typed SDA/SCL routes. GPIO bank0 and the raw microsecond timer must already be running
    /// (cold bring-up). On error the token and the pins come back in [`InitError`]; the pins
    /// are not touched before success.
    pub fn init<const SDA: u8, const SCL: u8>(
        self,
        sda: Pin<SDA>,
        scl: Pin<SCL>,
        clocks: &crate::clocks::Clocks,
        speed: Speed,
    ) -> Result<I2c<N, SDA, SCL>, InitError<N, SDA, SCL>>
    where
        Pin<SDA>: Sda<N>,
        Pin<SCL>: Scl<N>,
    {
        match self.start(clocks, speed) {
            Ok(rx_depth) => Ok(I2c {
                _sda: sda.into_i2c(),
                _scl: scl.into_i2c(),
                rx_depth,
                usable: true,
                clocks: *clocks,
            }),
            Err(error) => Err(InitError {
                error,
                token: self,
                sda,
                scl,
            }),
        }
    }
    /// The register part of [`Token::init`]; returns the RX FIFO depth.
    fn start(&self, clocks: &crate::clocks::Clocks, speed: Speed) -> Result<usize, Error> {
        if !clocks.still_current() {
            return Err(Error::Precondition);
        }
        let counts = timing(clocks.sys_hz(), speed)?;
        write(RESETS_CLR0, reset_bit(N));
        crate::poll(RESETS_DONE0, reset_bit(N), reset_bit(N)).map_err(|_| Error::Timeout)?;
        let mut io = Mmio {
            base: 0x4007_0000 + N as usize * 0x4000,
        };
        if io.read(TYPE) != 0x4457_0140 {
            return Err(Error::Precondition);
        }
        disable(&mut io)?;
        io.write(MASK, 0);
        io.read(CLEAR);
        io.write(RX_TL, 0);
        io.write(TX_TL, 0);
        configure(&mut io, counts);
        Ok(((io.read(PARAM) >> 8) & 255) as usize + 1)
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

/// Counts for the SCL high/low targets of `speed` at `ic_clk_hz` (= clk_sys, a whole number of
/// MHz: 50 or 200; anything else is `Invalid`).
/// Targets keep the I2C minimums (Sm 4.0/4.7 us, Fm 0.6/1.3 us, Fm+ 0.26/0.5 us).
/// HW (DesignWare, measured at 50 MHz): high = HCNT + SPKLEN + 7, low = LCNT + 1 cycles + rise.
fn timing(ic_clk_hz: u32, speed: Speed) -> Result<Counts, Error> {
    let (mode, high_ns, low_ns) = match speed {
        Speed::Standard => (1, 4_600, 5_400),
        Speed::Fast => (2, 900, 1_600),
        Speed::FastPlus => (2, 400, 600),
    };
    // Whole MHz only (clk_sys is 50 or 200 MHz): exact in 32 bits, no 64-bit division.
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
        write(self.base + offset, value);
    }
    #[inline(always)]
    fn now(&mut self) -> u32 {
        read(0x400a_c028)
    }
    fn clocks_current(&mut self, clocks: &crate::clocks::Clocks) -> bool {
        clocks.still_current()
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
        if !self.usable || !self.clocks.still_current() {
            return Err(Error::Precondition);
        }
        let counts = timing(self.clocks.sys_hz(), speed)?;
        let mut io = self.io();
        self.usable = false;
        disable(&mut io)?;
        configure(&mut io, counts);
        self.usable = true;
        Ok(())
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

/// One transaction. `Precondition` before any controller access if the handle is poisoned or
/// clk_sys (`ic_clk`) / clk_uart no longer match `clocks`: the SCL counts were computed for it.
fn run(
    io: &mut impl Io,
    clocks: &crate::clocks::Clocks,
    usable: &mut bool,
    address: u8,
    ops: &mut [Operation<'_>],
    depth: usize,
) -> Result<(), Error> {
    if address >= 128 {
        return Err(Error::Invalid);
    }
    if !*usable || !io.clocks_current(clocks) {
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
        sys: Option<u32>,
        uart: Option<u32>,
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
                uart: Some(50_000_000),
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
        fn clocks_current(&mut self, clocks: &crate::clocks::Clocks) -> bool {
            clocks.matches(self.sys, self.uart)
        }
    }
    const CLOCKS: crate::clocks::Clocks = crate::clocks::Clocks::for_test(200_000_000, 50_000_000);
    /// Review 6: PLL_SYS / CLK_UART unchanged but clk_sys elsewhere fails before any
    /// controller access, and the handle stays usable.
    #[test]
    fn stale_sys_clock_fails_before_the_controller_is_touched() {
        let mut m = Model::new(2);
        m.sys = Some(50_000_000);
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
        // Review 7: whole MHz only (32-bit math); the 64-bit formula gave the same counts.
        assert_eq!(timing(50_000_500, Speed::Fast), Err(Error::Invalid));
        for hz in [
            1_000_000,
            50_000_000,
            125_000_000,
            200_000_000,
            4_000_000_000,
        ] {
            for speed in [Speed::Standard, Speed::Fast, Speed::FastPlus] {
                let wide = |ns: u32| (u64::from(ns) * u64::from(hz)).div_ceil(1_000_000_000);
                if let Ok(t) = timing(hz, speed) {
                    let (_, high, low) = match speed {
                        Speed::Standard => (1, 4_600, 5_400),
                        Speed::Fast => (2, 900, 1_600),
                        Speed::FastPlus => (2, 400, 600),
                    };
                    let spklen = wide(SPIKE_NS).max(1);
                    assert_eq!(u64::from(t.spklen), spklen);
                    assert_eq!(u64::from(t.hcnt), wide(high) - spklen - 7);
                    assert_eq!(u64::from(t.lcnt), wide(low) - 1 - wide(RISE_NS));
                }
            }
        }
        assert_eq!(
            Error::AddressNack.kind(),
            ErrorKind::NoAcknowledge(NoAcknowledgeSource::Address)
        );
        assert_eq!(Error::ArbitrationLost.kind(), ErrorKind::ArbitrationLoss);
    }
}
