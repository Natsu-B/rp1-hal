//! Proc0 declarations: `pwm0 = { channel0 = 12 }` and `gpio = [22]`.
//! Connect GPIO12 to GPIO22 through 1 kΩ and share ground. Sample the 1 kHz,
//! 50% duty waveform and check that pulses cease when the channel is disabled.

use embedded_hal::{delay::DelayNs, digital::InputPin};
use rp1_hal::{
    clocks::Clocks,
    fugit::{ExtU32, RateExtU32},
    gpio::{Input, Pin},
    pwm::{Config, DutyTicks, Token},
    timer::{Timer, WaitTimer},
};

pub fn run(
    pwm: Token<0>,
    output: Pin<12>,
    input: Pin<22>,
    clocks: &Clocks,
    timer: &mut WaitTimer,
) -> Result<(), &'static str> {
    // The default pull-down holds the sense pin low if the jumper is missing.
    let mut input = input.into_input();
    let mut channels = pwm.init(clocks).map_err(|_| "PWM0 init failed")?;
    let actual = channels
        .channel0
        .configure(Config::from_frequency(1.kHz()))
        .map_err(|_| "PWM period setup failed")?;
    let mut output = channels.channel0.output(output);
    output
        .try_set_duty_ticks(DutyTicks::new((actual.cycle_ticks() / 2) as u32))
        .map_err(|_| "PWM duty setup failed")?;
    let output = output.enable().map_err(|_| "PWM enable failed")?;
    timer.delay_ms(3); // Let the range and duty latch at overflow.
    let (rising, high, total) = sample(&mut input);
    let _output = output.disable().map_err(|e| {
        let (_, pin) = e.into_driver().abort();
        let _input = pin.into_input();
        "PWM disable failed"
    })?;
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
