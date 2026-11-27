//! ADC, UART, SPI, I²C and the alarm demo use IRQ waits.
#![no_std]
#![no_main]

mod adc;
mod blinky;
mod i2c;
mod interrupt;
mod pwm;
mod spi;
mod uart;

use rp1_hal::log;
use rp1_rt::{Proc0, Proc1, entry};

rp1_rt::bind!();

#[entry(proc0)]
fn main(mut core: Proc0, mut p: rp1::Proc0Resources) -> ! {
    let sys = rp1::platform::init(&mut core, p.platform).unwrap();
    let clocks = &sys.clocks;
    let gpio = p.pins;
    let wait = &mut p.wait_timer;
    report("blinky", blinky::run(gpio.gpio13, gpio.gpio23, wait));
    report(
        "interrupt",
        interrupt::run(p.TIMER_ALARM0, p.irq.TIMER_0, wait),
    );
    report("spi", spi::run(p.spi0, gpio.gpio8, clocks, wait));
    report("adc", adc::run(p.adc, clocks, wait));
    report(
        "pwm",
        pwm::run(p.pwm0, gpio.gpio12, gpio.gpio22, clocks, wait),
    );
    report("i2c", i2c::run(p.i2c1, clocks, wait));
    report("uart", uart::run(p.uart1, clocks, wait));
    loop {
        rp1_rt::wait_for_event();
    }
}

fn report(name: &str, result: Result<(), &'static str>) {
    let _ = match result {
        Ok(()) => log!("EXAMPLE_END name={name} result=PASS\n"),
        Err(cause) => log!("EXAMPLE_FAIL name={name} cause={cause}\n"),
    };
}

// The runtime needs an entry for each core. This example leaves proc1 parked.
#[entry(proc1)]
fn idle(_core: Proc1, _p: rp1::Proc1Resources) -> ! {
    loop {
        rp1_rt::wait_for_event();
    }
}
