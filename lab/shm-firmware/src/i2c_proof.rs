//! Lab-only request/reply at link+0x28/0x2c (the unused part of the first 64 bytes).
use crate::rp1;
use embedded_hal::i2c::{I2c as _, Operation};
use rp1_hal::{
    i2c::{Error, I2c, Speed},
    log,
    timer::Timer,
};
type Bus = I2c<1, 2, 3>;
const PEER: u8 = 0x2d;
const REQUEST: usize = rp1_proto::LINK_START as usize + 0x28;
const REPLY: usize = REQUEST + 4;

#[inline(always)]
pub fn pending() -> u32 {
    unsafe { (REQUEST as *const u32).read_volatile() }
}
pub fn run(bus: Option<&mut Bus>, request: u32) {
    let result = match bus {
        None => Err(Error::Precondition),
        Some(bus) if request == 1 => prove(bus).map_err(|f| {
            log!("I2C_FAIL {:?}\n", f).unwrap();
            Error::Precondition
        }),
        Some(bus) => measure(bus, request),
    };
    log!("I2C_RESULT request={} {:?}\n", request, result).unwrap();
    unsafe {
        (REQUEST as *mut u32).write_volatile(0);
        (REPLY as *mut u32).write_volatile(if result.is_ok() { request } else { 0xffff_ffff });
    }
    rp1_proto::barrier();
}
/// A failed step: its name and the driver error, or the first mismatching reply byte.
#[derive(Debug)]
#[allow(dead_code)] // Fields are read through Debug.
pub enum Failure {
    Bus(&'static str, Error),
    Data(&'static str, usize, u8),
    NoNack(Result<(), Error>),
}
fn check(step: &'static str, bytes: &[u8]) -> Result<(), Failure> {
    let expected = |i: usize| 0x31u8.wrapping_add((i * 29) as u8);
    match bytes.iter().enumerate().find(|(i, b)| **b != expected(*i)) {
        None => Ok(()),
        Some((i, b)) => Err(Failure::Data(step, i, *b)),
    }
}
fn prove(bus: &mut Bus) -> Result<(), Failure> {
    let timer = Timer::new();
    let mut frame = core::array::from_fn::<_, 16, _>(|i| i as u8);
    frame[..4].copy_from_slice(&0x5244_3144u32.to_le_bytes());
    let gap = || timer.delay_us(2000);
    for (speed, hz) in [(Speed::Standard, 100_000), (Speed::Fast, 400_000)] {
        let at = |step| move |e| Failure::Bus(step, e);
        bus.set_speed(speed).map_err(at("set_speed"))?;
        let start = timer.now();
        bus.write(PEER, &frame).map_err(at("write"))?;
        let write_us = timer.now() - start;
        gap();
        let mut bytes = [0; 64];
        let start = timer.now();
        bus.read(PEER, &mut bytes).map_err(at("read"))?;
        let read_us = timer.now() - start;
        check("read", &bytes)?;
        gap();
        bytes.fill(0);
        let start = timer.now();
        bus.write_read(PEER, &frame, &mut bytes)
            .map_err(at("write_read"))?;
        let combined_us = timer.now() - start;
        check("write_read", &bytes)?;
        gap();
        bytes.fill(0);
        let (a, b) = bytes.split_at_mut(32);
        bus.transaction(
            PEER,
            &mut [
                Operation::Write(&frame[..8]),
                Operation::Write(&frame[8..]),
                Operation::Read(a),
                Operation::Read(b),
            ],
        )
        .map_err(at("w8_w8_r32_r32"))?;
        check("w8_w8_r32_r32", &bytes)?;
        gap();
        bytes.fill(0);
        bus.transaction(
            PEER,
            &mut [Operation::Read(&mut bytes), Operation::Write(&frame)],
        )
        .map_err(at("r64_w16"))?;
        check("r64_w16", &bytes)?;
        gap();
        // Read-only probe; never write any other address.
        let nack = bus.read(0x2e, &mut [0]);
        if nack != Err(Error::AddressNack) {
            return Err(Failure::NoNack(nack));
        }
        bytes.fill(0);
        bus.read(PEER, &mut bytes).map_err(at("read_after_nack"))?;
        check("read_after_nack", &bytes)?;
        gap();
        log!("I2C_PASS hz={} write16_us={} read64_us={} write_read16_64_us={} nack={:?} recovery=ok\n",
            hz, write_us, read_us, combined_us, nack).unwrap();
    }
    log!("I2C_LIMIT hz=1000000 payload=unverified esp32_documented_max=400000 use_RMT_probe\n")
        .unwrap();
    Ok(())
}
fn measure(bus: &mut Bus, request: u32) -> Result<(), Error> {
    let speed = match request {
        101 => Speed::Standard,
        102 => Speed::Fast,
        103 => Speed::FastPlus,
        _ => return Err(Error::Invalid),
    };
    bus.set_speed(speed)?;
    if bus.read(0x2e, &mut [0]) == Err(Error::AddressNack) {
        Ok(())
    } else {
        Err(Error::Precondition)
    }
}
