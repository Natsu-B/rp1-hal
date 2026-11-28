use rp1_hal::{pio::Rx, timer::WaitTimer};

fn no_alarm(rx: &mut Rx<0>, alarm_from_elsewhere: &mut WaitTimer) {
    let _ = rx.blocking(alarm_from_elsewhere);
}

fn main() {}
