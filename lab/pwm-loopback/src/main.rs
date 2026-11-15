//! PWM0 loopback: GPIO12 -> GPIO22 and GPIO13 -> GPIO23 (inputs only). See docs/pwm.md.
#![no_std]
#![no_main]
use embedded_hal::{digital::InputPin, pwm::SetDutyCycle};
use rp1_hal::{
    gpio::{Input, Pin},
    log,
    timer::Timer,
};
use rp1_rt::{Proc0, Proc1, entry};
rp1_rt::bind!();

#[entry(proc0)]
fn main0(mut core: Proc0, p: rp1::Proc0Resources) -> ! {
    let programmed = unsafe { rp1_hal::cold::clocks() }.unwrap();
    let (_, mut scmi) = p.clock_authority.init(200_000_000).unwrap();
    rp1::log::init(&mut core).unwrap();
    let report = unsafe { rp1_hal::cold::endpoint_and_syscfg(programmed) }.unwrap();
    log!("PWM_BOOT pcie={:?}\n", report).unwrap();
    p.proc1.launch(&mut core).unwrap();
    let mut endpoint = unsafe { rp1_hal::cold::EndpointService::new() };
    loop {
        endpoint.poll();
        scmi.poll();
    }
}

struct Samples {
    rising: u32,
    high: u32,
    total: u32,
    elapsed_us: u64,
}
fn sample<const P: u8>(pin: &mut Pin<P, Input>) -> Samples {
    let timer = Timer::new();
    let start = timer.now();
    let mut previous = pin.is_high().unwrap();
    let mut s = Samples {
        rising: 0,
        high: 0,
        total: 0,
        elapsed_us: 0,
    };
    loop {
        let high = pin.is_high().unwrap();
        s.rising += u32::from(high && !previous);
        s.high += u32::from(high);
        s.total += 1;
        previous = high;
        s.elapsed_us = timer.now().wrapping_sub(start);
        if s.elapsed_us >= 100_000 {
            return s;
        }
    }
}

#[entry(proc1)]
fn main1(core: Proc1, p: rp1::Proc1Resources) -> ! {
    let clocks = rp1_hal::clocks::Clocks::wait_from_proc0(&core, 10_000).unwrap();
    let mut channels = p.pwm0.init(&clocks).unwrap();
    let mut a = channels.channel0.output(p.pins.gpio12);
    let mut b = channels.channel1.output(p.pins.gpio13);
    let mut in_a = p.pins.gpio22.into_input();
    let mut in_b = p.pins.gpio23.into_input();
    a.set_period_ticks(50_000).unwrap(); // 1 kHz, 20 ns/tick.
    b.set_period_ticks(100_000).unwrap(); // 500 Hz sibling stays active throughout.
    b.set_duty_cycle_percent(50).unwrap();
    b.enable().unwrap();
    a.enable().unwrap();
    let timer = Timer::new();
    for round in 0..60 {
        for percent in [0, 25, 50, 75, 100] {
            a.set_duty_cycle_percent(percent).unwrap();
            timer.delay_us(3_000); // Range/duty latch on overflow; let old period finish.
            let sa = sample(&mut in_a);
            let sb = sample(&mut in_b);
            log!("PWM_SAMPLE round={} duty={} a_rise={} a_high={} a_total={} a_us={} b_rise={} b_high={} b_total={} b_us={}\n",
                round, percent, sa.rising, sa.high, sa.total, sa.elapsed_us,
                sb.rising, sb.high, sb.total, sb.elapsed_us).unwrap();
        }
        // Stop/reconfigure one channel; keep sibling running, then exercise token/pin return.
        a.disable().unwrap();
        a.set_period_ticks(100_000).unwrap();
        a.set_duty_ticks(25_000).unwrap();
        a.enable().unwrap();
        timer.delay_us(5_000);
        let slow = sample(&mut in_a);
        log!(
            "PWM_REPERIOD round={} rise={} high={} total={} us={}\n",
            round,
            slow.rising,
            slow.high,
            slow.total,
            slow.elapsed_us
        )
        .unwrap();
        a.disable().unwrap();
        let off = sample(&mut in_a);
        log!(
            "PWM_DISABLED round={} rise={} high={} total={}\n",
            round,
            off.rising,
            off.high,
            off.total
        )
        .unwrap();
        a.set_period_ticks(50_000).unwrap();
        a.enable().unwrap();
    }
    let (ch0, pin0) = a.free().unwrap();
    let (ch1, pin1) = b.free().unwrap();
    channels.channel0 = ch0;
    channels.channel1 = ch1;
    let token = channels.free().unwrap();
    let channels = token.init(&clocks).unwrap();
    let mut a = channels.channel0.output(pin0);
    let mut b = channels.channel1.output(pin1);
    a.set_period_ticks(50_000).unwrap();
    b.set_period_ticks(100_000).unwrap();
    a.set_duty_cycle_percent(50).unwrap();
    b.set_duty_cycle_percent(50).unwrap();
    a.enable().unwrap();
    b.enable().unwrap();
    log!("PWM_DONE reinitialized=1\n").unwrap();
    loop {
        rp1_rt::wait_for_event();
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo<'_>) -> ! {
    let _ = log!("PWM_FAIL panic\n");
    loop {
        rp1_rt::wait_for_event();
    }
}
