use rp1_hal::{gpio::Pin, i2c::I2c, spi::Spi};

fn spi(raw: &mut Spi<0, (Pin<11>, Pin<10>, Pin<9>)>) {
    fn needs_sync_bus<B: embedded_hal::spi::SpiBus<u8>>(_: &mut B) {}
    needs_sync_bus(raw);
}

fn i2c(raw: &mut I2c<1, (Pin<2>, Pin<3>)>) {
    let _ = embedded_hal::i2c::I2c::write(raw, 0x2d, &[1]);
}

fn main() {}
