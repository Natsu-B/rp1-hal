use rp1_hal::{
    gpio::Pin,
    timer::WaitTimer,
    uart::{Irq, Parts, Rx, Tx, Uart},
};
use rp1_rt::interrupt::{Binding, BlockIrq};

fn other_instance(uart: &mut Uart<2, (Pin<4>, Pin<5>)>, wait: &mut WaitTimer) {
    let _ = uart.blocking(wait);
}

fn raw_uart1(uart: &mut Uart<1, (Pin<0>, Pin<1>)>, wait: &mut WaitTimer) {
    let _ = uart.blocking(wait);
}

fn wrong_line(
    token: rp1_hal::uart::Token<1>,
    pins: rp1_hal::uart::Pins<1, (Pin<0>, Pin<1>)>,
    irq: Binding<BlockIrq<2>, 43, 0>,
) {
    let _ = unsafe { Parts::<1, _, Irq<1, 0>>::__new(token, pins, irq) };
}

fn wrong_number(
    token: rp1_hal::uart::Token<1>,
    pins: rp1_hal::uart::Pins<1, (Pin<0>, Pin<1>)>,
    irq: Binding<BlockIrq<1>, 43, 0>,
) {
    let _ = unsafe { Parts::<1, _, Irq<1, 0>>::__new(token, pins, irq) };
}

fn wrong_core(tx: Tx<1, 0, Irq<1, 0>>, rx: Rx<1, 1, Irq<1, 1>>) {
    let _ = Uart::join(tx, rx);
}

fn main() {}
