use rp1_hal::{gpio::Pin, i2c::{Controller, I2c, Irq}};

fn managed(bus: &mut I2c<1, (Pin<2>, Pin<3>), Controller, Irq<1, 0>>) {
    bus.enable_interrupt(1);
    bus.disable_interrupt(1);
    bus.interrupt_status();
    bus.clear_interrupt();
}

fn main() {}
