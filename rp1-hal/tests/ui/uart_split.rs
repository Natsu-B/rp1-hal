use rp1_hal::{gpio::Pin, uart::{Rx, Tx, Uart}};

fn mismatched(tx: Tx<2, 4>, rx: Rx<3, 9>) {
    let _ = Uart::join(tx, rx);
}

fn free_half(rx: Rx<2, 5>) {
    let _ = rx.try_free();
}

fn reuse(uart: Uart<2, (Pin<4>, Pin<5>)>) {
    let _ = uart.split();
    let _ = uart.try_free();
}

fn main() {}
