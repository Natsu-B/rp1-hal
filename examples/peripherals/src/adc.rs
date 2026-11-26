//! Proc0 declaration: `adc = {}`.
//! No wiring. Sample the internal temperature input with the native 12-bit one-shot API.

use crate::rp1; // log! resolves rp1::log in this scope.
use rp1_hal::{
    adc::{TEMPERATURE, Token},
    clocks::Clocks,
    log,
};

pub fn run(adc: Token, clocks: &Clocks) -> Result<(), &'static str> {
    let mut adc = adc.init(clocks).map_err(|_| "ADC init failed")?;
    let mut raw = [0; 3];
    for sample in &mut raw {
        *sample = adc.read(TEMPERATURE).map_err(|_| "ADC conversion failed")?;
        if !(1..=4094).contains(sample) {
            return Err("ADC saturated");
        }
    }
    let mc = adc
        .read_temperature()
        .map_err(|_| "temperature conversion failed")?;
    let _ = log!("adc raw={raw:?} temperature_mc={mc}\n");
    Ok(())
}
