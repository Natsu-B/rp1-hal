//! Proc0 declarations: `spi0 = { sck = 11, mosi = 10, miso = 9 }` and `gpio = [8]`.
//! GPIO8 is CS. Keep external devices clear of GPIO8–11 unless they can safely share the
//! bus. Internal loopback needs no jumper and exercises an embedded-hal SPI device transfer.

use embedded_hal::spi::{MODE_0, SpiDevice};
use rp1_hal::{
    clocks::Clocks,
    fugit::RateExtU32,
    gpio::Pin,
    spi::{Config, Device, Parts},
};

pub fn run(spi: Parts<0, 11, 10, 9>, cs: Pin<8>, clocks: &Clocks) -> Result<(), &'static str> {
    let mut bus = spi
        .init(clocks, Config::new(1.MHz(), MODE_0))
        .map_err(|_| "SPI0 init failed")?;
    bus.set_loopback(true)
        .map_err(|_| "SPI0 loopback setup failed")?;
    let mut device = Device::new(bus, cs);
    let tx = [0x35u8, 0xa6, 0xf0, 0x19];
    let mut rx = [0; 4];
    device
        .transfer(&mut rx, &tx)
        .map_err(|_| "SPI0 transfer failed")?;
    if rx != tx {
        return Err("SPI0 loopback data mismatch");
    }
    Ok(())
}
