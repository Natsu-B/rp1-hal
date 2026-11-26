//! Proc0 declaration: `gpio = [13, 23]`.
//! Connect GPIO13 to GPIO23 through 1 kΩ and share ground. Read back three low/high
//! cycles using embedded-hal digital pins and a typed delay.

use embedded_hal::digital::{InputPin, OutputPin};
use rp1_hal::{
    fugit::ExtU32,
    gpio::{PadConfig, Pin},
    timer::Timer,
};

pub fn run(output: Pin<13>, input: Pin<23>) -> Result<(), &'static str> {
    let mut output = output.into_output(false);
    // A high-impedance input can sample the driven pin without fighting it.
    let mut input = input.into_input_with_pad(PadConfig::INPUT_DEFAULT);
    let timer = Timer::new();

    for _ in 0..3 {
        for high in [false, true] {
            output.set_state(high.into()).unwrap();
            timer.delay(20.millis());
            if input.is_high().unwrap() != high {
                output.set_low().unwrap();
                return Err("GPIO23 did not follow GPIO13");
            }
        }
    }
    output.set_low().unwrap();
    Ok(())
}
