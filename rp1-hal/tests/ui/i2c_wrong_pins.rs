use rp1_hal::{gpio::Pin, i2c::Pins};

fn wrong(sda: Pin<2>, scl: Pin<3>, wrong_scl: Pin<1>) {
    let _ = Pins::<1>::builder().sda(scl);
    let _ = Pins::<1>::builder().sda(sda).scl(wrong_scl);
}
fn main() {}
