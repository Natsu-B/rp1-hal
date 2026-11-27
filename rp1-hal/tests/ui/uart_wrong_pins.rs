use rp1_hal::{gpio::Pin, uart::Pins};

fn wrong(tx: Pin<6>, rx: Pin<14>, valid_tx: Pin<4>) {
    // UART2's TX/RX are GPIO4/5; GPIO6 is UART2 CTS and GPIO14 is UART0 TX.
    let _ = Pins::<2>::builder().tx(tx);
    let _ = Pins::<2>::builder().tx(valid_tx).rx(rx);
}
fn main() {}
