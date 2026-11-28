use rp1_hal::{gpio::Pin, i2c, spi, timer::WaitTimer, uart};

fn uart(uart: &mut uart::Uart<1, (Pin<0>, Pin<1>), uart::Irq<1, 0>>, timer: &mut WaitTimer) {
    let _: uart::Blocking<'_, 0, 1> = uart.blocking(timer);
}

fn spi(
    spi: &mut spi::Spi<1, (Pin<21>, Pin<20>, Pin<19>), spi::Controller, spi::Irq<1, 0>>,
    timer: &mut WaitTimer,
) {
    let _: spi::Blocking<'_, 0, (Pin<11>, Pin<10>, Pin<9>), 1> = spi.blocking(timer);
}

fn i2c(
    i2c: &mut i2c::I2c<1, (Pin<2>, Pin<3>), i2c::Controller, i2c::Irq<1, 0>>,
    timer: &mut WaitTimer,
) {
    let _: i2c::Blocking<'_, 0, (Pin<8>, Pin<9>), 1> = i2c.blocking(timer);
}

fn main() {}
