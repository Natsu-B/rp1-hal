use rp1_hal::{gpio::Pin, i2c, spi, timer::WaitTimer, uart};

fn uart(uart: &mut uart::Uart<1, (Pin<0>, Pin<1>), uart::Irq<1, 0>>, timer: &mut WaitTimer) {
    let _: uart::Blocking<'_, 0, 1> = uart.blocking(timer);
}

fn spi<P>(spi: &mut spi::Spi<1, P, spi::Controller, spi::Irq<1, 0>>, timer: &mut WaitTimer)
where
    P: spi::CompletePins<0> + spi::CompletePins<1>,
{
    let _: spi::Blocking<'_, 0, P, 1> = spi.blocking(timer);
}

fn i2c<P>(i2c: &mut i2c::I2c<1, P, i2c::Controller, i2c::Irq<1, 0>>, timer: &mut WaitTimer)
where
    P: i2c::CompletePins<0> + i2c::CompletePins<1>,
{
    let _: i2c::Blocking<'_, 0, P, 1> = i2c.blocking(timer);
}

fn main() {}
