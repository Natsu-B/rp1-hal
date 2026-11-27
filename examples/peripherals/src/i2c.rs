//! Proc0 declaration: `i2c1 = { sda = 2, scl = 3 }`.
//! Connect RP1 GPIO2/3 to the 0x2d ESP32 lab peer (GPIO21/22), with 3.3 V pull-ups and
//! common ground. At 100 kHz, check that the peer ACKs a 16-byte write and returns its
//! fixed 64-byte read pattern. Check `bad_frames` on the peer to validate the write data.

use crate::rp1;
use embedded_hal::{delay::DelayNs, i2c::I2c};
use rp1_hal::{
    clocks::Clocks,
    i2c::{Parts, Speed},
    timer::WaitTimer,
};

pub fn run(
    i2c: Parts<1, rp1::I2c1Pins>,
    clocks: &Clocks,
    timer: &mut WaitTimer,
) -> Result<(), &'static str> {
    const PEER: u8 = 0x2d;
    let mut bus = i2c
        .init(clocks, Speed::Standard)
        .map_err(|_| "I2C1 init failed")?;
    let mut bus = bus.polling();
    // D1DR is the lab peer's frame tag. It counts bad frames and always serves 0x31 + 29*i.
    let mut frame: [u8; 16] = core::array::from_fn(|i| i as u8);
    frame[..4].copy_from_slice(b"D1DR");
    let mut reply = [0; 64];

    // The HAL bounds each transfer. A STOP-terminated read resyncs the peer's TX FIFO.
    bus.read(PEER, &mut reply)
        .map_err(|_| "I2C1 resync read failed")?;
    timer.delay_ms(2); // The peer refills its TX FIFO after STOP.
    bus.write(PEER, &frame)
        .map_err(|_| "I2C1 frame write failed")?;
    timer.delay_ms(2);
    bus.read(PEER, &mut reply)
        .map_err(|_| "I2C1 reply read failed")?;
    if reply != core::array::from_fn(|i| (0x31 + i * 29) as u8) {
        return Err("ESP32 reply mismatch");
    }
    Ok(())
}
