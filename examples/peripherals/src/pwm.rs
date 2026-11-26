//! Proc0 declarations: `pwm0 = { channel0 = 12 }` and `gpio = [22]`.
//! Connect GPIO12 to GPIO22 through 1 kΩ and share ground. Sample the 1 kHz,
//! 50% duty waveform and check that pulses cease when the channel is disabled.

use embedded_hal::{digital::InputPin, pwm::SetDutyCycle};
use rp1_hal::{
    clocks::Clocks,
    fugit::{ExtU32, HertzU32, RateExtU32},
    gpio::{Input, Pin},
    pwm::Token,
    timer::Timer,
};

pub fn run(
    pwm: Token<0>,
    output: Pin<12>,
    input: Pin<22>,
    clocks: &Clocks,
) -> Result<(), &'static str> {
    // The default pull-down holds the sense pin low if the jumper is missing.
    let mut input = input.into_input();
    let channels = pwm.init(clocks).map_err(|_| "PWM0 init failed")?;
    let frequency: HertzU32 = 1.kHz();
    let period = channels.clock().raw() / frequency.raw();
    let mut output = channels.channel0.output(output);
    output
        .set_period_ticks(period)
        .map_err(|_| "PWM period setup failed")?;
    output
        .set_duty_cycle_percent(50)
        .map_err(|_| "PWM duty setup failed")?;
    output.enable().map_err(|_| "PWM enable failed")?;
    Timer::new().delay(3.millis()); // Let the range and duty latch at overflow.
    let (rising, high, total) = sample(&mut input);
    output.disable().map_err(|_| "PWM disable failed")?;
    if !(95..=105).contains(&rising) || !(45..=55).contains(&(100 * high / total)) {
        return Err("PWM frequency or duty out of range");
    }
    let (rising, high, _) = sample(&mut input);
    if rising != 0 || high != 0 {
        return Err("PWM output still active after disable");
    }
    Ok(())
}

fn sample(pin: &mut Pin<22, Input>) -> (u32, u32, u32) {
    let timer = Timer::new();
    let end = timer.now_32() + 100.millis();
    let mut previous = pin.is_high().unwrap();
    let (mut rising, mut high, mut total) = (0, 0, 0);
    loop {
        let level = pin.is_high().unwrap();
        rising += u32::from(level && !previous);
        high += u32::from(level);
        total += 1;
        previous = level;
        if timer.now_32() >= end {
            return (rising, high, total);
        }
    }
}
