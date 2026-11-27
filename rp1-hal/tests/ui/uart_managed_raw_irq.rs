use rp1_hal::{
    gpio::Pin,
    uart::{Irq, Uart},
};

fn raw_irq(mut uart: Uart<1, (Pin<0>, Pin<1>), Irq<0>>) {
    uart.enable_rx_interrupt();
    uart.enable_tx_interrupt();
    let (mut tx, mut rx) = uart.split();
    tx.enable_interrupt();
    rx.enable_interrupt();
}

fn main() {}
