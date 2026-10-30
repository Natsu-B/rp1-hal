// A bundle that does not match the pin table cannot even be named.
type Uart = rp1_hal::uart::Parts<2, 5, 4>;
type I2c = rp1_hal::i2c::Parts<1, 2, 4>;
type Spi = rp1_hal::spi::Parts<0, 11, 10, 5>;
fn take(_: Uart, _: I2c, _: Spi) {}
fn main() {}
