//! Polling, seven-bit DesignWare master (DOC: RP1 §9 and Linux i2c-designware).
//! Uses the adopted 200 MHz clk_sys without changing shared clocks or SCMI votes.
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
    HOLD = sda_hold, ABORT_SOURCE = tx_abrt_source, ENABLE_STATUS = enable_status,
    PARAM = comp_param_1, TYPE = comp_type,
}
const STOP: u32 = 1 << 9;
const RESTART: u32 = 1 << 10;
const READ: u32 = 1 << 8;
const STOP_DET: u32 = 1 << 9;
const TIMEOUT_US: u32 = 100_000;

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
pub struct I2c<const N: u8, const SDA: u8, const SCL: u8> {
    _sda: Pin<SDA, Function<3>>,
    _scl: Pin<SCL, Function<3>>,
    rx_depth: usize,
    usable: bool,
}

impl<const N: u8> Token<N> {
    /// # Safety
    /// Generated resources only: sole owner of I2C N, its reset and IRQ, with Linux disabled.
    pub const unsafe fn new() -> Self {
        assert!(N < 7);
        Self { _private: () }
    }
    /// Selects the typed SDA/SCL routes and releases only this controller's reset.
    /// GPIO bank0 and the raw microsecond timer must already be running (cold bring-up).
    pub fn init<const SDA: u8, const SCL: u8>(
        self,
        sda: Pin<SDA>,
        scl: Pin<SCL>,
        clocks: &crate::clocks::Clocks,
        speed: Speed,
    ) -> Result<I2c<N, SDA, SCL>, Error>
    where
        Pin<SDA>: Sda<N>,
        Pin<SCL>: Scl<N>,
    {
        if clocks.sys_hz != 200_000_000 || !crate::clocks::running_tuple_admitted() {
            return Err(Error::Precondition);
        }
        write(0x4001_7004, 1 << (7 + N));
        crate::poll(0x4001_401c, 1 << (7 + N), 1 << (7 + N)).map_err(|_| Error::Timeout)?;
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
        configure(&mut io, speed);
        let rx_depth = ((io.read(PARAM) >> 8) & 255) as usize + 1;
        Ok(I2c {
            _sda: sda.into_i2c(),
            _scl: scl.into_i2c(),
            rx_depth,
            usable: true,
        })
    }
}

// DOC: rpi-6.12.y clock_calc, RP1 DT rise=100ns/fall=65ns, at the adopted 200MHz.
// Timing includes the controller's hold/stretch behavior; physical bus rate needs measurement.
fn timing(speed: Speed) -> (u32, u32, u32) {
    match speed {
        Speed::Standard => (1, 788, 1179),
        Speed::Fast => (2, 108, 359),
        Speed::FastPlus => (2, 40, 127),
    }
}
fn configure(io: &mut impl Io, speed: Speed) {
    let (mode, high, low) = timing(speed);
    let (h, l) = if speed == Speed::Standard {
        (SS_HCNT, SS_LCNT)
    } else {
        (FS_HCNT, FS_LCNT)
    };
    io.write(h, high);
    io.write(l, low);
    io.write(HOLD, low / 2);
    io.write(CON, 1 | (mode << 1) | (1 << 5) | (1 << 6));
}

// The small register interface also runs the real FIFO/abort loop against a host model.
trait Io {
    fn read(&mut self, offset: usize) -> u32;
    fn write(&mut self, offset: usize, value: u32);
    fn now(&mut self) -> u32;
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
        if !self.usable || !crate::clocks::running_tuple_admitted() {
            return Err(Error::Precondition);
        }
        let mut io = self.io();
        self.usable = false;
        disable(&mut io)?;
        configure(&mut io, speed);
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
        if address >= 128 {
            return Err(Error::Invalid);
        }
        if !self.usable || !crate::clocks::running_tuple_admitted() {
            return Err(Error::Precondition);
        }
        if ops.iter().all(|op| operation_len(op) == 0) {
            return Ok(());
        }
        let mut io = self.io();
        self.usable = false;
        disable(&mut io)?;
        io.read(CLEAR);
        io.write(TAR, u32::from(address));
        io.write(ENABLE, 1);
        let result = transfer(&mut io, ops, self.rx_depth);
        if result.is_err() {
            // Request a STOP/flush before disabling, including when a slave stretches forever.
            io.write(ENABLE, 3);
        }
        let cleanup = disable(&mut io);
        self.usable = cleanup.is_ok();
        io.read(CLEAR);
        result.and(cleanup)
    }
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
        assert_eq!(timing(Speed::Standard), (1, 788, 1179));
        assert_eq!(timing(Speed::Fast), (2, 108, 359));
        assert_eq!(timing(Speed::FastPlus), (2, 40, 127));
        assert_eq!(
            Error::AddressNack.kind(),
            ErrorKind::NoAcknowledge(NoAcknowledgeSource::Address)
        );
        assert_eq!(Error::ArbitrationLost.kind(), ErrorKind::ArbitrationLoss);
    }
}
