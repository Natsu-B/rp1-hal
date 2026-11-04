// Generated from rp1-regdb/data/pinmux/bank0.toml; do not edit.
//! DOC: bank-0 routes of the signals rp1-hal drives, with their function select
//! (RP1 Peripherals Table 4). SPI4 is slave-only and has no driver.
use crate::{gpio::Pin, i2c, pwm, spi, uart};
impl i2c::Scl<0> for Pin<1> {
    const FUNC: u8 = 3;
}
impl i2c::Scl<0> for Pin<9> {
    const FUNC: u8 = 3;
}
impl i2c::Scl<1> for Pin<3> {
    const FUNC: u8 = 3;
}
impl i2c::Scl<1> for Pin<11> {
    const FUNC: u8 = 3;
}
impl i2c::Scl<2> for Pin<5> {
    const FUNC: u8 = 3;
}
impl i2c::Scl<2> for Pin<13> {
    const FUNC: u8 = 3;
}
impl i2c::Scl<3> for Pin<7> {
    const FUNC: u8 = 3;
}
impl i2c::Scl<3> for Pin<15> {
    const FUNC: u8 = 3;
}
impl i2c::Scl<3> for Pin<23> {
    const FUNC: u8 = 3;
}
impl i2c::Sda<0> for Pin<0> {
    const FUNC: u8 = 3;
}
impl i2c::Sda<0> for Pin<8> {
    const FUNC: u8 = 3;
}
impl i2c::Sda<1> for Pin<2> {
    const FUNC: u8 = 3;
}
impl i2c::Sda<1> for Pin<10> {
    const FUNC: u8 = 3;
}
impl i2c::Sda<2> for Pin<4> {
    const FUNC: u8 = 3;
}
impl i2c::Sda<2> for Pin<12> {
    const FUNC: u8 = 3;
}
impl i2c::Sda<3> for Pin<6> {
    const FUNC: u8 = 3;
}
impl i2c::Sda<3> for Pin<14> {
    const FUNC: u8 = 3;
}
impl i2c::Sda<3> for Pin<22> {
    const FUNC: u8 = 3;
}
impl pwm::PinRoute<0, 0> for Pin<12> {
    const FUNC: u8 = 0;
}
impl pwm::PinRoute<0, 1> for Pin<13> {
    const FUNC: u8 = 0;
}
impl pwm::PinRoute<0, 2> for Pin<14> {
    const FUNC: u8 = 0;
}
impl pwm::PinRoute<0, 2> for Pin<18> {
    const FUNC: u8 = 3;
}
impl pwm::PinRoute<0, 3> for Pin<15> {
    const FUNC: u8 = 0;
}
impl pwm::PinRoute<0, 3> for Pin<19> {
    const FUNC: u8 = 3;
}
impl spi::Miso<0> for Pin<9> {
    const FUNC: u8 = 0;
}
impl spi::Miso<1> for Pin<19> {
    const FUNC: u8 = 0;
}
impl spi::Miso<2> for Pin<1> {
    const FUNC: u8 = 8;
}
impl spi::Miso<3> for Pin<5> {
    const FUNC: u8 = 8;
}
impl spi::Miso<5> for Pin<13> {
    const FUNC: u8 = 8;
}
impl spi::Mosi<0> for Pin<10> {
    const FUNC: u8 = 0;
}
impl spi::Mosi<1> for Pin<20> {
    const FUNC: u8 = 0;
}
impl spi::Mosi<2> for Pin<2> {
    const FUNC: u8 = 8;
}
impl spi::Mosi<3> for Pin<6> {
    const FUNC: u8 = 8;
}
impl spi::Mosi<5> for Pin<14> {
    const FUNC: u8 = 8;
}
impl spi::Sck<0> for Pin<11> {
    const FUNC: u8 = 0;
}
impl spi::Sck<1> for Pin<21> {
    const FUNC: u8 = 0;
}
impl spi::Sck<2> for Pin<3> {
    const FUNC: u8 = 8;
}
impl spi::Sck<3> for Pin<7> {
    const FUNC: u8 = 8;
}
impl spi::Sck<5> for Pin<15> {
    const FUNC: u8 = 8;
}
impl uart::Rx<0> for Pin<15> {
    const FUNC: u8 = 4;
}
impl uart::Rx<1> for Pin<1> {
    const FUNC: u8 = 2;
}
impl uart::Rx<2> for Pin<5> {
    const FUNC: u8 = 2;
}
impl uart::Rx<3> for Pin<9> {
    const FUNC: u8 = 2;
}
impl uart::Rx<4> for Pin<13> {
    const FUNC: u8 = 2;
}
impl uart::Tx<0> for Pin<14> {
    const FUNC: u8 = 4;
}
impl uart::Tx<1> for Pin<0> {
    const FUNC: u8 = 2;
}
impl uart::Tx<2> for Pin<4> {
    const FUNC: u8 = 2;
}
impl uart::Tx<3> for Pin<8> {
    const FUNC: u8 = 2;
}
impl uart::Tx<4> for Pin<12> {
    const FUNC: u8 = 2;
}
