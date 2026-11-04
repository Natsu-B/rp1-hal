use rp1_hal::{gpio::Pin, pwm::Channel};
fn bad(ch: Channel<0, 0>, pin: Pin<13>) {
    let _ = ch.output(pin);
}
fn main() {}
