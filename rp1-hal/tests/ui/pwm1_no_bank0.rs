use rp1_hal::{gpio::Pin, pwm::Channel};
fn bad(ch: Channel<1, 0>, pin: Pin<12>) {
    let _ = ch.output(pin);
}
fn main() {}
