use rp1_hal::{gpio::Pin, i2c::I2c, spi::Spi, uart::Uart};

fn uart(raw: &mut Uart<1, (Pin<0>, Pin<1>)>) {
    let _ = embedded_io::Write::write_all(raw, b"x");
}

fn spi(raw: &mut Spi<0, (Pin<11>, Pin<10>, Pin<9>)>) {
    let _ = embedded_hal::spi::SpiBus::write(raw, &[1u8]);
}

fn i2c(raw: &mut I2c<1, (Pin<2>, Pin<3>)>) {
    let _ = embedded_hal::i2c::I2c::write(raw, 0x2d, &[1]);
}

fn main() {}
