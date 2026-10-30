//! Lab-only SPI0 proof, run by proc1 (which owns SPI0 and gets its Clocks from proc0): internal
//! loopback (CTRLR0.SRL) at several rates and frame sizes through `SpiDevice` (GPIO8 chip
//! select), logged on UART0, at boot and on request (link+0x28 = [`REQUEST`], reply at
//! link+0x2c = the request on success, all-ones on failure).
//!
//! SCLK/MOSI/CS (GPIO11/10/8) drive ESP32 inputs; MISO (GPIO9) is an ESP32 input with the RP1
//! pull-down, so with loopback off the received data must be all zeros.
use crate::rp1;
use embedded_hal::spi::{MODE_0, MODE_3, Operation, SpiBus, SpiDevice};
use rp1_hal::{
    log,
    spi::{Device, Error, Spi},
};
pub type Dev = Device<Spi<0, 11, 10, 9>, 8>;
pub const REQUEST: u32 = 0x5b1;
const LINK_REQUEST: usize = rp1_proto::LINK_START as usize + 0x28;
/// Transfers per measurement (each of 32 frames).
const ROUNDS: u32 = 32;

fn now() -> u32 {
    unsafe { (0x400a_c028 as *const u32).read_volatile() }
}

/// A failed step: (rate, frame bits, what, code). Code = first bad index, or 0x100 + the
/// driver `Error` discriminant. Printed with `Display` only: Debug formatting of the nested
/// results costs too much of the 48 KiB shared SRAM image.
pub struct Failure(u32, u8, &'static str, u32);
impl core::fmt::Display for Failure {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{} hz={} bits={} code=0x{:x}",
            self.2, self.0, self.1, self.3
        )
    }
}

fn pattern(seed: u32, i: usize) -> u16 {
    ((i as u32 ^ seed).wrapping_mul(0x9e37_79b1) >> 13) as u16
}
fn fail(hz: u32, bits: u8, what: &'static str) -> impl FnOnce(Error) -> Failure {
    move |e| Failure(hz, bits, what, 0x100 + e as u32)
}

/// `ROUNDS` CS-framed transfers of 32 frames of `bits` (in u16 words) at `hz`, checked, then
/// `SPI_PASS` with the measured rate (bits on the wire per µs of transfer calls).
fn report(dev: &mut Dev, tag: &str, hz: u32, bits: u8) -> Result<(), Failure> {
    dev.bus_mut()
        .set_frame_bits(bits)
        .map_err(fail(hz, bits, "frame_bits"))?;
    let mask = (u32::MAX >> (32 - u32::from(bits))) as u16;
    let mut us = 0;
    for r in 0..ROUNDS {
        let tx: [u16; 32] = core::array::from_fn(|i| pattern(r ^ hz ^ u32::from(bits), i));
        let mut rx = [0u16; 32];
        let start = now();
        dev.transfer(&mut rx, &tx)
            .map_err(fail(hz, bits, "transfer"))?;
        us += now().wrapping_sub(start);
        if let Some(i) = (0..32).find(|&i| rx[i] != tx[i] & mask) {
            return Err(Failure(hz, bits, "data", i as u32));
        }
    }
    let frames = ROUNDS * 32;
    let kbps = frames * u32::from(bits) * 1000 / us.max(1);
    log!(
        "SPI_PASS {} hz={} div={} bits={} frames={} us={} kbps={} wire_pct={}\n",
        tag,
        hz,
        dev.bus_mut().divider(),
        bits,
        frames,
        us,
        kbps,
        kbps / (hz / 100_000)
    )
    .unwrap();
    Ok(())
}

pub fn prove(dev: &mut Dev, tag: &str) -> Result<(), Failure> {
    // Loopback off: MISO is pulled down, so the data path really is the internal loop below.
    let bus = dev.bus_mut();
    bus.set_loopback(false).map_err(fail(0, 8, "loopback"))?;
    bus.set_mode(MODE_0).map_err(fail(0, 8, "mode"))?;
    bus.set_frame_bits(8).map_err(fail(0, 8, "frame_bits"))?;
    bus.set_frequency(1_000_000)
        .map_err(fail(0, 8, "frequency"))?;
    let mut ext = [0xa5u8; 16];
    bus.transfer(&mut ext, &[0xff; 16])
        .map_err(fail(0, 8, "external"))?;
    if ext != [0; 16] {
        return Err(Failure(0, 8, "loopback off data", 0));
    }
    bus.set_loopback(true).map_err(fail(0, 8, "loopback"))?;
    // u8 words on the bus at 10 MHz.
    bus.set_frequency(10_000_000)
        .map_err(fail(0, 8, "frequency"))?;
    let tx: [u8; 48] = core::array::from_fn(|i| pattern(5, i) as u8);
    let mut rx = [0u8; 48];
    bus.transfer(&mut rx, &tx).map_err(fail(0, 8, "u8"))?;
    if rx != tx {
        return Err(Failure(0, 8, "u8 data", 0));
    }
    let (depth, dfs32) = (bus.fifo_depth(), bus.has_32_bit_frames());
    for (want, mode) in [
        (1_000_000, MODE_0),
        (10_000_000, MODE_3),
        (25_000_000, MODE_0),
        (50_000_000, MODE_0),
    ] {
        let bus = dev.bus_mut();
        bus.set_mode(mode).map_err(fail(want, 0, "mode"))?;
        let hz = bus
            .set_frequency(want)
            .map_err(fail(want, 0, "frequency"))?;
        for bits in [8, 12, 16] {
            report(dev, tag, hz, bits)?;
        }
    }
    // In place, and write / delay / read in one CS-framed transaction (the read sends zeros).
    let mut buf: [u16; 40] = core::array::from_fn(|i| pattern(7, i));
    let copy = buf;
    dev.transfer_in_place(&mut buf)
        .map_err(fail(0, 16, "in_place"))?;
    if buf != copy {
        return Err(Failure(0, 16, "in_place data", 0));
    }
    let mut read = [0xa5a5u16; 8];
    dev.transaction(&mut [
        Operation::Write(&copy[..8]),
        Operation::DelayNs(2_000),
        Operation::Read(&mut read),
    ])
    .map_err(fail(0, 16, "transaction"))?;
    if read != [0; 8] {
        return Err(Failure(0, 16, "transaction data", 0));
    }
    log!(
        "SPI_{} depth={} dfs32={} loopback_off=zeros u8=ok in_place=ok transaction=ok PASS\n",
        tag,
        depth,
        dfs32
    )
    .unwrap();
    Ok(())
}

/// `SPI_RESULT <what> Ok` or `SPI_RESULT <what> Err <failure>`.
pub fn log_result(what: &str, result: &Result<(), Failure>) {
    match result {
        Ok(()) => log!("SPI_RESULT {} Ok\n", what),
        Err(f) => log!("SPI_RESULT {} Err {}\n", what, f),
    }
    .unwrap();
}

pub fn run(dev: Option<&mut Dev>, request: u32) {
    let result = match dev {
        Some(dev) => prove(dev, "P1REQ"),
        None => Err(Failure(0, 0, "init", 0x100 + Error::Precondition as u32)),
    };
    log_result("request=0x5b1", &result);
    unsafe {
        (LINK_REQUEST as *mut u32).write_volatile(0);
        ((LINK_REQUEST + 4) as *mut u32).write_volatile(if result.is_ok() {
            request
        } else {
            0xffff_ffff
        });
    }
    rp1_proto::barrier();
}
