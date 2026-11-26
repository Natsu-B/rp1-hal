//! Proc0 declaration: `i2c1 = { sda = 2, scl = 3 }`.
//! Connect RP1 GPIO2/3 to the 0x2d ESP32 lab peer (GPIO21/22), with 3.3 V pull-ups and
//! common ground. Write the peer's 16-byte frame and verify its 64-byte reply at 100 kHz.

use crate::rp1;
use embedded_hal::i2c::I2c;
use rp1_hal::{
    clocks::Clocks,
    fugit::ExtU32,
    i2c::{Parts, Speed},
    log,
    timer::Timer,
};

pub fn run(i2c: Parts<1, 2, 3>, clocks: &Clocks) -> Result<(), &'static str> {
    const PEER: u8 = 0x2d;
    let failed = |step| {
        move |error| {
            let _ = log!("i2c {step}: {error:?}\n");
            step
        }
    };
    let mut bus = i2c
        .init(clocks, Speed::Standard)
        .map_err(|e| failed("I2C1 init")(e.error))?;
    let mut frame: [u8; 16] = core::array::from_fn(|i| i as u8);
    frame[..4].copy_from_slice(b"D1DR");
    let mut reply = [0; 64];

    // The HAL bounds each transfer. A STOP-terminated read resyncs the peer's TX FIFO.
    bus.read(PEER, &mut reply)
        .map_err(failed("I2C1 resync read"))?;
    Timer::new().delay(2.millis()); // The peer refills its TX FIFO after STOP.
    bus.write(PEER, &frame)
        .map_err(failed("I2C1 frame write"))?;
    Timer::new().delay(2.millis());
    bus.read(PEER, &mut reply)
        .map_err(failed("I2C1 reply read"))?;
    if reply != core::array::from_fn(|i| (0x31 + i * 29) as u8) {
        return Err("ESP32 reply mismatch");
    }
    Ok(())
}
