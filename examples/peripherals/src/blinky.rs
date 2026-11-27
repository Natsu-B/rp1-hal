//! Proc0 declaration: `gpio = [13, 23]`.
//! Connect GPIO13 to GPIO23 through 1 kΩ and share ground. Read back three low/high
//! cycles using embedded-hal digital pins and a typed delay.

use embedded_hal::delay::DelayNs;
use embedded_hal::digital::{InputPin, OutputPin};
use rp1_hal::{gpio::Pin, timer::WaitTimer};

pub fn run(output: Pin<13>, input: Pin<23>, timer: &mut WaitTimer) -> Result<(), &'static str> {
    let mut output = output.into_output(false);
    // The default pull-down holds the sense pin low if the jumper is missing.
    let mut input = input.into_input();

    for _ in 0..3 {
        for high in [false, true] {
            output.set_state(high.into()).unwrap();
            timer.delay_ms(20);
            if input.is_high().unwrap() != high {
                output.set_low().unwrap();
                return Err("GPIO23 did not follow GPIO13");
            }
        }
    }
    output.set_low().unwrap();
    Ok(())
}
