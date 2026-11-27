//! M6 waveform and native setter latency measured against the RP1 timer.
use embedded_hal::digital::InputPin;
use rp1_hal::{
    clocks::Clocks,
    fugit::{ExtU32, RateExtU32},
    gpio::{Input, Pin},
    log,
    pwm::{Config, DutyTicks, Error, Output, Running},
    timer::{Alarm, Instant32, Timer, WaitTimer},
};
use rp1_rt::{
    interrupt,
    interrupt::{Priority, Slot},
};

use crate::{rp1, sense_pins, status_nonce};

const STEP_US: u32 = 1_000;
const UPDATES_PER_DUTY: u32 = 20_000;
const DUTIES: [u32; 5] = [25, 100, 50, 0, 75];
const SWEEP_STEP_US: u32 = STEP_US + 1;
const SWEEP_UPDATES: u32 = 1_100;
const SWEEP_CYCLES: u32 = 1_050;
const CONVENTION_CYCLES: u32 = 200_000;
const UPDATES: u32 = UPDATES_PER_DUTY * DUTIES.len() as u32;
const SAMPLES: usize = 100;
const PRIORITY: u8 = 2;

type Drive = Output<0, 0, 12, Running>;
#[derive(Clone, Copy)]
struct Wave {
    first_rise: u32,
    samples: [(u16, u16); SAMPLES],
}

#[derive(Default)]
struct Level {
    start: u32,
    samples: u32,
    wrong: u32,
    us: u32,
    max_gap_us: u32,
}

struct Control {
    a: Drive,
    alarm: Alarm<1>,
    next: Instant32,
    duty: [DutyTicks; 5],
    phase: usize,
    quarter: u32,
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
        let duty = if control.count < UPDATES {
            control.duty[control.phase].ticks()
        } else {
            control.quarter * (1 + 2 * (control.count & 1))
        };
        let start = timer.now_32().ticks();
        let ok = motor_native_duty(&mut control.a, duty);
        let elapsed = timer.now_32().ticks().wrapping_sub(start);
        control.max_call_us = control.max_call_us.max(elapsed);
        control.errors += u32::from(!ok);
        control.count += 1;
        if control.count == UPDATES + SWEEP_UPDATES {
            control.alarm.disable_interrupt();
            return;
        }
        if control.count < UPDATES && control.count.is_multiple_of(UPDATES_PER_DUTY) {
            control.phase += 1;
        }
        let step = if control.count <= UPDATES {
            STEP_US
        } else {
            SWEEP_STEP_US
        };
        control.next += step.micros();
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
    let invalid_cycles = capture_wave(
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
    let _b = disable(b, &mut sense_b).unwrap();

    let mut irq = p.irq.TIMER_1;
    irq.set_priority(Priority::new(PRIORITY).unwrap());
    let timer = Timer::new();
    let mut control = Control {
        a: a.enable().unwrap(),
        alarm: p.TIMER_ALARM1,
        next: Instant32::from_ticks(0),
        duty: DUTIES.map(|duty| DutyTicks::new((duty / 25) * quarter)),
        quarter,
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

    let zero = Wave {
        first_rise: 0,
        samples: [(0, 0); SAMPLES],
    };
    let mut waves = [zero; 3];
    let mut levels = [Level::default(), Level::default()];
    let mut wave_index = 0;
    for (i, duty) in DUTIES.into_iter().enumerate() {
        // Allow the first update in each plateau to reach the pin. No UART
        // activity is generated on this core during the timed update campaign.
        let offset = i as u32 * UPDATES_PER_DUTY * STEP_US + 5_000;
        p.wait_timer.delay_until(first + offset.micros()).unwrap();
        match duty {
            0 => levels[0] = constant_level(&mut sense_a, false),
            100 => levels[1] = constant_level(&mut sense_a, true),
            _ => {
                waves[wave_index] = capture_wave(&mut sense_a, |_| {}, || {});
                wave_index += 1;
            }
        }
    }
    let sweep_start = UPDATES * STEP_US;
    // Start sampling at the transition into the sweep and finish before the last update.
    p.wait_timer
        .delay_until(first + sweep_start.micros())
        .unwrap();
    let sweep = sweep(&mut sense_a);
    p.wait_timer
        .delay_until(first + (sweep_start + SWEEP_UPDATES * SWEEP_STEP_US).micros())
        .unwrap();
    irq.disable();
    let mut control = CONTROL.take().unwrap();
    control.alarm.disable_interrupt();
    control.alarm.cancel();
    control.alarm.clear_interrupt();
    irq.unpend();
    let (stop_ok, convention_ok) = match disable(control.a, &mut sense_a) {
        Some(output) => (
            true,
            convention(output, &mut sense_a, &mut p.wait_timer, nonce),
        ),
        None => (false, false),
    };
    let elapsed = timer.now().ticks() - (first.ticks() - 10_000);
    let first = first.ticks() as u32;

    let mut wave_ok = true;
    for (i, duty) in DUTIES.into_iter().enumerate() {
        let plateau = i as u32 * UPDATES_PER_DUTY * STEP_US;
        let window = plateau + 5_000..plateau + UPDATES_PER_DUTY * STEP_US;
        wave_ok &= match duty {
            0 => report_level(nonce, duty, &levels[0], first, window),
            100 => report_level(nonce, duty, &levels[1], first, window),
            _ => {
                let index = [25, 50, 75]
                    .iter()
                    .position(|&target| target == duty)
                    .unwrap();
                report_cycles(nonce, "duty", duty, &waves[index], first, Some(window))
            }
        };
    }
    wave_ok &= report_cycles(nonce, "invalid", 50, &invalid_cycles, first, None);
    let Sweep {
        first_rise,
        last_rise,
        low25,
        high75,
        other,
        min_period_us,
        max_period_us,
    } = sweep;
    let start_us = first_rise.wrapping_sub(first);
    let end_us = last_rise.wrapping_sub(first);
    log!("MOTOR_API_SWEEP run={nonce:08x} cycles={SWEEP_CYCLES} low25={low25} high75={high75} other={other} min_period_us={min_period_us} max_period_us={max_period_us} start_us={start_us} end_us={end_us}\n").unwrap();
    let sweep_ok = min_period_us >= 990
        && max_period_us <= 1010
        && other == 0
        && low25 >= 400
        && high75 >= 400
        && start_us >= sweep_start
        && end_us < sweep_start + (SWEEP_UPDATES - 1) * SWEEP_STEP_US;
    let max_call_upper_us = control.max_call_us + 1; // One timer tick for quantization.
    let plateau_updates = control.count.min(UPDATES);
    let sweep_updates = control.count - plateau_updates;
    let missed = UPDATES + SWEEP_UPDATES - control.count;
    let passed = wave_ok
        && sweep_ok
        && convention_ok
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
    log!("MOTOR_API_END run={nonce:08x} updates={plateau_updates} sweep_updates={sweep_updates} missed={missed} errors={} arm_failed={} max_late_us={} max_call_us={} max_call_upper_us={max_call_upper_us} invalid={invalid} invalid_native={invalid_native} register_changes={register_changes} elapsed_us={elapsed} stopped={} result={result}\n",
        control.errors, u8::from(control.missed), control.max_late_us,
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

fn disable<const C: u8, const P: u8, const S: u8>(
    output: Output<0, C, P, Running>,
    sense: &mut Pin<S, Input>,
) -> Option<Output<0, C, P>> {
    match output.disable() {
        Ok(output) => Some(output),
        Err(error) => {
            let (_, pin) = error.into_driver().abort();
            let _input = pin.into_input();
            assert!(
                !sense.is_high().unwrap(),
                "MOTOR_API_FAIL abort left sense high"
            );
            None
        }
    }
}

fn cycles<const P: u8>(
    pin: &mut Pin<P, Input>,
    count: u32,
    mut on_rise: impl FnMut(u32),
    mut observe: impl FnMut(),
    mut sample: impl FnMut(u32, u32, u32, u32),
) -> (u32, u32) {
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
    let first = edge(true);
    let mut rise = first;
    for i in 0..count {
        on_rise(i);
        let fall = edge(false);
        let next = edge(true);
        sample(i, rise, next.wrapping_sub(rise), fall.wrapping_sub(rise));
        rise = next;
    }
    (first, rise)
}

fn capture_wave<const P: u8>(
    pin: &mut Pin<P, Input>,
    on_rise: impl FnMut(u32),
    observe: impl FnMut(),
) -> Wave {
    let mut samples = [(0, 0); SAMPLES];
    let (first_rise, _) = cycles(
        pin,
        SAMPLES as u32,
        on_rise,
        observe,
        |i, _, period, high| {
            samples[i as usize] = (period as u16, high as u16);
        },
    );
    Wave {
        first_rise,
        samples,
    }
}

struct Sweep {
    first_rise: u32,
    last_rise: u32,
    low25: u32,
    high75: u32,
    other: u32,
    min_period_us: u32,
    max_period_us: u32,
}

fn sweep(pin: &mut Pin<22, Input>) -> Sweep {
    let mut result = Sweep {
        first_rise: 0,
        last_rise: 0,
        low25: 0,
        high75: 0,
        other: 0,
        min_period_us: u32::MAX,
        max_period_us: 0,
    };
    (result.first_rise, result.last_rise) = cycles(
        pin,
        SWEEP_CYCLES,
        |_| {},
        || {},
        |_, _, period, high| {
            result.min_period_us = result.min_period_us.min(period);
            result.max_period_us = result.max_period_us.max(period);
            match high {
                240..=260 => result.low25 += 1,
                740..=760 => result.high75 += 1,
                _ => result.other += 1,
            }
        },
    );
    result
}

fn convention(
    mut output: Output<0, 0, 12>,
    sense: &mut Pin<22, Input>,
    wait: &mut WaitTimer,
    nonce: u32,
) -> bool {
    const RANGE: u32 = 2_500;
    output.configure(Config::from_ticks(RANGE)).unwrap();
    output
        .try_set_duty_ticks(DutyTicks::new(RANGE / 2))
        .unwrap();
    let output = output.enable().unwrap();
    wait.delay(3.millis()).unwrap();
    let before = convention_clocks();
    let (mut min_period_us, mut max_period_us) = (u32::MAX, 0);
    let (first, last) = cycles(
        sense,
        CONVENTION_CYCLES,
        |_| {},
        || {},
        |_, _, period, _| {
            min_period_us = min_period_us.min(period);
            max_period_us = max_period_us.max(period);
        },
    );
    let after = convention_clocks();
    let stopped = disable(output, sense).is_some();
    let span_us = last.wrapping_sub(first);
    // RP1 datasheet §3.8: TICKS uses clk_ref at the XOSC rate. Verify the
    // live divide-by-50 tick and the PWM0 XOSC / 1 clock on both sides.
    let shared = before == after
        && before[0] & 3 == 3
        && before[1] == 50
        && before[2] & 0xbe0 == 0x840
        && before[3] == 1
        && before[4] == 0;
    log!(
        "MOTOR_API_CLOCK run={nonce:08x} before={:x},{},{:x},{},{} after={:x},{},{:x},{},{}\n",
        before[0],
        before[1],
        before[2],
        before[3],
        before[4],
        after[0],
        after[1],
        after[2],
        after[3],
        after[4]
    )
    .unwrap();
    log!("MOTOR_API_CONVENTION run={nonce:08x} cycles={CONVENTION_CYCLES} span_us={span_us} min_period_us={min_period_us} max_period_us={max_period_us} shared={} stopped={}\n",
        u8::from(shared), u8::from(stopped)).unwrap();
    stopped
        && (!shared
            || (min_period_us >= 40 && max_period_us <= 60 && span_us.abs_diff(10_000_000) <= 100))
}

fn convention_clocks() -> [u32; 5] {
    // Read-only TICKS.TIMER_CTRL/CYCLES and PWM0 CLK_CTRL/DIV_INT/DIV_FRAC.
    [0x40174000, 0x40174004, 0x40018074, 0x40018078, 0x4001807c]
        .map(|address| unsafe { (address as *const u32).read_volatile() })
}

fn constant_level(pin: &mut Pin<22, Input>, expected: bool) -> Level {
    let timer = Timer::new();
    let start = timer.now_32().ticks();
    let mut previous = start;
    let mut level = Level {
        start,
        ..Level::default()
    };
    loop {
        level.wrong += u32::from(pin.is_high().unwrap() != expected);
        level.samples += 1;
        let now = timer.now_32().ticks();
        level.max_gap_us = level.max_gap_us.max(now.wrapping_sub(previous));
        level.us = now.wrapping_sub(start);
        if level.us >= 100_000 {
            return level;
        }
        previous = now;
    }
}

fn report_level(
    nonce: u32,
    duty: u32,
    level: &Level,
    first: u32,
    window: core::ops::Range<u32>,
) -> bool {
    let Level {
        start,
        samples,
        wrong,
        us,
        max_gap_us,
    } = *level;
    let t_us = start.wrapping_sub(first);
    log!("MOTOR_API_LEVEL run={nonce:08x} duty={duty} samples={samples} wrong={wrong} us={us} max_gap_us={max_gap_us} t_us={t_us}\n").unwrap();
    wrong == 0
        && samples >= 50_000
        && max_gap_us <= 10
        && (100_000..=100_000 + max_gap_us).contains(&us)
        && window.contains(&t_us)
        && window.contains(&(t_us + us))
}

fn report_cycles(
    nonce: u32,
    label: &str,
    target: u32,
    wave: &Wave,
    first: u32,
    window: Option<core::ops::Range<u32>>,
) -> bool {
    let mut valid = true;
    let mut t_us = wave.first_rise.wrapping_sub(first) as i32;
    for (i, &(period, high)) in wave.samples.iter().enumerate() {
        let next = t_us + i32::from(period);
        valid &= (990..=1010).contains(&period)
            && high <= period
            && (u32::from(high) * 100).abs_diff(target * u32::from(period)) <= u32::from(period)
            && match &window {
                Some(window) => window.contains(&(t_us as u32)) && window.contains(&(next as u32)),
                None => next < 0,
            };
        log!("MOTOR_API_SAMPLE run={nonce:08x} phase={label} duty={target} i={i} period_us={period} high_us={high} t_us={t_us}\n").unwrap();
        t_us = next;
    }
    valid
}
