//! M6 waveform and native setter latency measured against the RP1 timer.
use embedded_hal::digital::InputPin;
use rp1_hal::{
    clocks::Clocks,
    fugit::{ExtU32, RateExtU32},
    gpio::{Input, Pin},
    log,
    pwm::{Config, DutyTicks, Error, Output, Running},
    timer::{Alarm, Instant32, Timer},
};
use rp1_rt::{
    interrupt,
    interrupt::{Priority, Slot},
};

use crate::{rp1, sense_pins, status_nonce};

const STEP_US: u32 = 1_000;
const UPDATES_PER_DUTY: u32 = 20_000;
const DUTIES: [u32; 5] = [0, 25, 50, 75, 100];
const UPDATES: u32 = UPDATES_PER_DUTY * DUTIES.len() as u32;
const SAMPLES: usize = 100;
const PRIORITY: u8 = 2;

type Drive = Output<0, 0, 12, Running>;
type Cycles = [(u16, u16); SAMPLES];

struct Control {
    a: Drive,
    alarm: Alarm<1>,
    next: Instant32,
    duty: [DutyTicks; 5],
    phase: usize,
    count: u32,
    max_call_us: u32,
    max_late_us: u32,
    errors: u32,
    missed: bool,
}

static CONTROL: Slot<Option<Control>, 1, PRIORITY> = Slot::empty();

// The same call runs in the ISR and is disassembled by CI, with a runtime duty and range.
#[unsafe(no_mangle)]
#[inline(never)]
pub extern "C" fn motor_native_duty(output: &mut Drive, ticks: u32) -> bool {
    output.try_set_duty_ticks(DutyTicks::new(ticks)).is_ok()
}

#[interrupt(proc1)]
fn TIMER_1() {
    let timer = Timer::new();
    let entry = timer.now_32().ticks();
    CONTROL.with(|control| {
        if !control.alarm.try_expired() {
            return;
        }
        control.alarm.clear_interrupt();
        control.max_late_us = control
            .max_late_us
            .max(entry.wrapping_sub(control.next.ticks()));
        let duty = control.duty[control.phase].ticks();
        let start = timer.now_32().ticks();
        let ok = motor_native_duty(&mut control.a, duty);
        let elapsed = timer.now_32().ticks().wrapping_sub(start);
        control.max_call_us = control.max_call_us.max(elapsed);
        control.errors += u32::from(!ok);
        control.count += 1;
        if control.count == UPDATES {
            control.alarm.disable_interrupt();
            return;
        }
        if control.count.is_multiple_of(UPDATES_PER_DUTY) {
            control.phase += 1;
        }
        control.next += STEP_US.micros();
        if control.alarm.try_arm(control.next).is_err() {
            control.missed = true;
            control.alarm.disable_interrupt();
        }
    });
}

pub fn run(mut p: rp1::Proc1Resources, clocks: &Clocks) {
    let nonce = unsafe { status_nonce::WORD.read_volatile() };
    let (mut sense_a, mut sense_b) = sense_pins(p.pins.gpio22, p.pins.gpio23, nonce);
    let mut channels = p.pwm0.init(clocks).unwrap();
    let config = Config::from_frequency(1.kHz());
    let actual = channels.channel0.configure(config).unwrap();
    channels.channel1.configure(config).unwrap();
    let quarter = actual.range_ticks() / 4;
    let mut a = channels.channel0.output(p.pins.gpio12);
    let mut b = channels.channel1.output(p.pins.gpio13);
    b.try_set_duty_ticks(DutyTicks::new(2 * quarter)).unwrap();
    let mut b = b.enable().unwrap();

    // One rejected conversion and one rejected native duty per observed B cycle.
    // A remains disabled. Read back its configuration and watch it stay low.
    p.wait_timer.delay(3.millis()).unwrap();
    let (mut invalid, mut invalid_native, mut register_changes) = (0, 0, 0);
    let expected = channel_registers(0);
    let invalid_cycles = cycles(
        &mut sense_b,
        |i| {
            let config = if i.is_multiple_of(2) {
                Config::from_frequency((50_000_001 + i).Hz())
            } else {
                Config::from_period(rp1_hal::fugit::NanosDurationU64::from_ticks(
                    u64::MAX - u64::from(i),
                ))
            };
            invalid += u32::from(a.configure(config) == Err(Error::Invalid));
            invalid_native += u32::from(
                b.try_set_duty_ticks(DutyTicks::new(actual.range_ticks() + 1 + i))
                    == Err(Error::Invalid),
            );
            register_changes += u32::from(a.actual() != actual || channel_registers(0) != expected);
        },
        || {
            assert!(
                !sense_a.is_high().unwrap(),
                "MOTOR_API_FAIL invalid output edge"
            )
        },
    );
    let _b = b.disable().unwrap();

    let mut irq = p.irq.TIMER_1;
    irq.set_priority(Priority::new(PRIORITY).unwrap());
    let timer = Timer::new();
    let mut control = Control {
        a: a.enable().unwrap(),
        alarm: p.TIMER_ALARM1,
        next: Instant32::from_ticks(0),
        duty: [0, quarter, 2 * quarter, 3 * quarter, actual.range_ticks()].map(DutyTicks::new),
        phase: 0,
        count: 0,
        max_call_us: 0,
        max_late_us: 0,
        errors: 0,
        missed: false,
    };
    control.alarm.cancel();
    control.alarm.clear_interrupt();
    irq.unpend();
    log!(
        "MOTOR_API_START run={nonce:08x} updates={UPDATES} range={}\n",
        actual.range_ticks()
    )
    .unwrap();

    let first = timer.now() + 10.millis();
    control.next = Instant32::from_ticks(first.ticks() as u32);
    control.alarm.try_arm(first).unwrap();
    control.alarm.enable_interrupt();
    assert!(CONTROL.put(control).is_ok());
    irq.enable();

    let mut waves = [[(0, 0); SAMPLES]; 3];
    let mut levels = [(0, 0, 0); 2];
    for (i, duty) in DUTIES.into_iter().enumerate() {
        // Allow the first update in each plateau to reach the pin. No UART
        // activity is generated on this core during the timed update campaign.
        let offset = i as u32 * UPDATES_PER_DUTY * STEP_US + 5_000;
        p.wait_timer.delay_until(first + offset.micros()).unwrap();
        match duty {
            0 => levels[0] = constant_level(&mut sense_a, false),
            100 => levels[1] = constant_level(&mut sense_a, true),
            _ => waves[i - 1] = cycles(&mut sense_a, |_| {}, || {}),
        }
    }
    p.wait_timer
        .delay_until(first + (UPDATES * STEP_US).micros())
        .unwrap();
    irq.disable();
    let mut control = CONTROL.take().unwrap();
    control.alarm.disable_interrupt();
    control.alarm.cancel();
    control.alarm.clear_interrupt();
    irq.unpend();
    let stop_ok = control.a.disable().is_ok();
    let elapsed = timer.now().ticks() - (first.ticks() - 10_000);

    let mut wave_ok = true;
    for (label, target, raw) in [
        ("duty", 25, &waves[0]),
        ("duty", 50, &waves[1]),
        ("duty", 75, &waves[2]),
        ("invalid", 50, &invalid_cycles),
    ] {
        wave_ok &= report_cycles(nonce, label, target, raw);
    }
    for (duty, (samples, wrong, us)) in [(0, levels[0]), (100, levels[1])] {
        wave_ok &= wrong == 0 && samples > 0 && (100_000..=100_010).contains(&us);
        log!(
            "MOTOR_API_LEVEL run={nonce:08x} duty={duty} samples={samples} wrong={wrong} us={us}\n"
        )
        .unwrap();
    }
    let max_call_upper_us = control.max_call_us + 1; // One timer tick for quantization.
    let missed = UPDATES - control.count;
    let passed = wave_ok
        && invalid == SAMPLES as u32
        && invalid_native == SAMPLES as u32
        && register_changes == 0
        && missed == 0
        && !control.missed
        && control.errors == 0
        && control.max_late_us < STEP_US
        && max_call_upper_us <= 10
        && (100_000_000..=120_000_000).contains(&elapsed)
        && stop_ok;
    let result = if passed { "PASS" } else { "FAIL" };
    log!("MOTOR_API_END run={nonce:08x} updates={} missed={missed} errors={} arm_failed={} max_late_us={} max_call_us={} max_call_upper_us={max_call_upper_us} invalid={invalid} invalid_native={invalid_native} register_changes={register_changes} elapsed_us={elapsed} stopped={} result={result}\n",
        control.count, control.errors, u8::from(control.missed), control.max_late_us,
        control.max_call_us, u8::from(stop_ok)).unwrap();
}

fn channel_registers(ch: usize) -> [u32; 4] {
    // HAL-owned PWM0, observed without writes. Exclude GLOBAL_CTRL's SET_UPDATE strobe.
    let read = |offset| unsafe { ((0x4009_8000usize + offset) as *const u32).read_volatile() };
    [
        read(0) & (1 << ch),
        read(0x14 + ch * 0x10),
        read(0x18 + ch * 0x10),
        read(0x20 + ch * 0x10),
    ]
}

fn cycles<const P: u8>(
    pin: &mut Pin<P, Input>,
    mut on_rise: impl FnMut(u32),
    mut observe: impl FnMut(),
) -> Cycles {
    let timer = Timer::new();
    let mut edge = |level| {
        let start = timer.now_32().ticks();
        loop {
            observe();
            let high = pin.is_high().unwrap();
            let now = timer.now_32().ticks();
            assert!(
                now.wrapping_sub(start) <= 2_000,
                "MOTOR_API_FAIL missing edge"
            );
            if high == level {
                return now;
            }
        }
    };
    edge(false);
    let mut rise = edge(true);
    core::array::from_fn(|i| {
        on_rise(i as u32);
        let fall = edge(false);
        let next = edge(true);
        let sample = (
            next.wrapping_sub(rise) as u16,
            fall.wrapping_sub(rise) as u16,
        );
        rise = next;
        sample
    })
}

fn constant_level(pin: &mut Pin<22, Input>, expected: bool) -> (u32, u32, u32) {
    let timer = Timer::new();
    let start = timer.now_32().ticks();
    let (mut samples, mut wrong) = (0, 0);
    loop {
        wrong += u32::from(pin.is_high().unwrap() != expected);
        samples += 1;
        let us = timer.now_32().ticks().wrapping_sub(start);
        if us >= 100_000 {
            return (samples, wrong, us);
        }
    }
}

fn report_cycles(nonce: u32, label: &str, target: u32, raw: &Cycles) -> bool {
    let mut valid = true;
    for (i, &(period, high)) in raw.iter().enumerate() {
        valid &= (990..=1010).contains(&period)
            && high <= period
            && (u32::from(high) * 100).abs_diff(target * u32::from(period)) <= u32::from(period);
        log!("MOTOR_API_SAMPLE run={nonce:08x} phase={label} duty={target} i={i} period_us={period} high_us={high}\n")
            .unwrap();
    }
    valid
}
