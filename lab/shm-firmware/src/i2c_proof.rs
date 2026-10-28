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

pub fn pending() -> u32 {
    unsafe { (REQUEST as *const u32).read_volatile() }
}
pub fn run(bus: Option<&mut Bus>, request: u32) {
    let result = match bus {
        None => Err(Error::Precondition),
        Some(bus) if request == 1 => prove(bus),
        Some(bus) => measure(bus, request),
    };
    log!("I2C_RESULT request={} {:?}\n", request, result).unwrap();
    unsafe {
        (REQUEST as *mut u32).write_volatile(0);
        (REPLY as *mut u32).write_volatile(if result.is_ok() { request } else { 0xffff_ffff });
    }
    rp1_proto::barrier();
}
fn check(bytes: &[u8]) -> Result<(), Error> {
    if bytes
        .iter()
        .enumerate()
        .all(|(i, b)| *b == (0x31u8).wrapping_add((i * 29) as u8))
    {
        Ok(())
    } else {
        Err(Error::Fifo)
    }
}
fn prove(bus: &mut Bus) -> Result<(), Error> {
    let timer = Timer::new();
    let mut frame = core::array::from_fn::<_, 16, _>(|i| i as u8);
    frame[..4].copy_from_slice(&0x5244_3144u32.to_le_bytes());
    for (speed, hz) in [(Speed::Standard, 100_000), (Speed::Fast, 400_000)] {
        bus.set_speed(speed)?;
        let start = timer.now();
        bus.write(PEER, &frame)?;
        let write_us = timer.now() - start;
        timer.delay_us(2000);
        let mut bytes = [0; 64];
        let start = timer.now();
        bus.read(PEER, &mut bytes)?;
        let read_us = timer.now() - start;
        check(&bytes)?;
        timer.delay_us(2000);
        let start = timer.now();
        bus.write_read(PEER, &frame, &mut bytes)?;
        let combined_us = timer.now() - start;
        check(&bytes)?;
        timer.delay_us(2000);
        let (a, b) = bytes.split_at_mut(32);
        bus.transaction(
            PEER,
            &mut [
                Operation::Write(&frame[..8]),
                Operation::Write(&frame[8..]),
                Operation::Read(a),
                Operation::Read(b),
            ],
        )?;
        check(&bytes)?;
        timer.delay_us(2000);
        bus.transaction(
            PEER,
            &mut [Operation::Read(&mut bytes), Operation::Write(&frame)],
        )?;
        check(&bytes)?;
        timer.delay_us(2000);
        let nack = bus.read(0x2e, &mut [0]); // Read-only probe; never write any other address.
        if nack != Err(Error::AddressNack) {
            return Err(Error::Precondition);
        }
        bus.read(PEER, &mut bytes)?;
        check(&bytes)?;
        timer.delay_us(2000);
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
