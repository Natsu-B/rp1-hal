use rp1_hal::{gpio::Pin, pwm::Channel};
fn bad(ch: Channel<0, 0>, pin: Pin<12>) {
    let _output = ch.output(pin);
    let _ = pin.into_output(false);
}
fn main() {}
