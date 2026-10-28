use rp1_hal::{clocks::Clocks, gpio::Pin, i2c::{Speed, Token}};
fn wrong(clocks: &Clocks) {
    let token = unsafe { Token::<1>::new() };
    let _ = token.init(unsafe { Pin::<3>::new() }, unsafe { Pin::<2>::new() }, clocks, Speed::Standard);
}
fn main() {}
