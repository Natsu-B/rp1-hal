use rp1_hal::{clocks::Clocks, gpio::Pin, uart::Token};
fn wrong(clocks: &Clocks) {
    // UART2's TX/RX are GPIO4/5; GPIO6 is UART2 CTS and GPIO14 is UART0 TX.
    let token = unsafe { Token::<2>::new() };
    let _ = token.init(unsafe { Pin::<6>::new() }, unsafe { Pin::<14>::new() }, clocks, 115_200);
}
fn main() {}
