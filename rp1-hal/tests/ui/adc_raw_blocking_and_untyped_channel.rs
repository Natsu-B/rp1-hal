use rp1_hal::{adc::Adc, timer::WaitTimer};

fn raw(adc: &mut Adc, timer: &mut WaitTimer) {
    let _ = adc.blocking(timer);
    let _ = adc.try_start(4_u8);
}

fn main() {}
