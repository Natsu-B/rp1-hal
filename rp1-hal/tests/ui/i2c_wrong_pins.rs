use rp1_hal::{clocks::Clocks, gpio::Pin, i2c::{Speed, Token}};
fn main() {
    let clocks = Clocks { sys_hz: 200_000_000, uart_hz: 50_000_000 };
    let token = unsafe { Token::<1>::new() };
    let _ = token.init(unsafe { Pin::<3>::new() }, unsafe { Pin::<2>::new() }, &clocks, Speed::Standard);
}
