use rp1_hal::pwm::Channel;
fn needs_send<T: Send>() {}
fn main() {
    needs_send::<Channel<0, 0>>();
}
