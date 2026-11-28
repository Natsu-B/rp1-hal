use rp1_hal::{pio::Rx, timer::WaitTimer};
use rp1_rt::interrupt::{Binding, BlockIrq};

fn no_alarm(rx: &mut Rx<0>, alarm_from_elsewhere: &mut WaitTimer) {
    let _ = rx.blocking(alarm_from_elsewhere);
}

fn wrong_core(raw: Binding<BlockIrq<66>, 50, 0>) {
    unsafe { rp1_hal::pio::__bind_irq::<0, 50, 1>(raw) };
}

fn main() {}
