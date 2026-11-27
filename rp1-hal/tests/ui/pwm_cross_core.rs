use rp1_hal::pwm::Channel;
fn needs_message<T: rp1_proto::Message>() {}
fn main() {
    needs_message::<Channel<0, 0>>();
}
