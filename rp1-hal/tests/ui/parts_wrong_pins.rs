use rp1_hal::gpio::Pin;

// A bundle that does not match the pin table cannot even be named.
type Uart = rp1_hal::uart::Parts<2, (Pin<5>, Pin<4>)>;
type I2c = rp1_hal::i2c::Parts<1, (Pin<2>, Pin<4>)>;
type Spi = rp1_hal::spi::Parts<0, (Pin<11>, Pin<10>, Pin<5>)>;
fn take(_: Uart, _: I2c, _: Spi) {}
fn main() {}
