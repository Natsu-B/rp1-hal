use rp1_hal::{gpio::Pin, spi::{Controller, Irq, Spi}};

fn managed(mut spi: Spi<0, (Pin<11>, Pin<10>, Pin<9>), Controller, Irq<0, 0>>) {
    spi.enable_interrupt(1);
    spi.disable_interrupt(1);
    let _ = spi.interrupt_status();
    spi.clear_interrupt();
}

fn main() {}
