//! IRQ-backed delay with and without a 1 kHz absolute alarm schedule on proc1.
#![no_std]
#![no_main]

#[path = "../../status-nonce.rs"]
mod status_nonce;

use core::sync::atomic::{
    AtomicBool, AtomicU32,
    Ordering::{Acquire, Relaxed, Release},
};
use embedded_hal::{delay::DelayNs, digital::OutputPin};
use rp1_hal::{
    fugit::ExtU32,
    gpio::{Output, Pin},
    log,
    timer::{Alarm, Instant, Instant32, Timer, WaitTimer},
};
use rp1_rt::{Proc0, Proc1, entry, interrupt, interrupt::Slot};

rp1_rt::bind!();

const PERIOD_US: u32 = 1_000;
const CONTROL_TICKS: u32 = 1_500;
const CONTROL_PRIORITY: u8 = 2;
const MAX_CONTROL_LATE_US: u32 = 50;
const SAMPLES: u32 = 100;
const SETTINGS_US: [u32; 5] = [1, 10, 100, 1_000, 10_000];
// One tick for phase, one for the sampled tick delta, and 20 µs for service.
const MAX_OVERSHOOT_US: u32 = 22;
// The shortest requests can expire during admission and arming.
const WAKE_REQUIRED_FROM_US: u32 = 100;

struct Control {
    alarm: Alarm<1>,
    trace: Pin<13, Output>,
    next: Instant32,
    count: u32,
    worst_us: u32,
    arm_failed: bool,
}
static CONTROL: Slot<Option<Control>, 1, CONTROL_PRIORITY> = Slot::empty();

// Proc1-local DSRAM. Only the respective handler writes each count.
static WAIT_WAKES: AtomicU32 = AtomicU32::new(0);
static ISR_SPANS: AtomicU32 = AtomicU32::new(0);
static DELAY_ACTIVE: AtomicBool = AtomicBool::new(false);
static DELAY_END: AtomicU32 = AtomicU32::new(0);

#[interrupt(proc1)]
fn TIMER_0() {
    // This wrapper is the exclusive IRQ for the lab's WaitTimer on ALARM0.
    WAIT_WAKES.store(WAIT_WAKES.load(Relaxed) + 1, Relaxed);
    unsafe { WaitTimer::__on_interrupt::<0>() };
}

#[interrupt(proc1)]
fn TIMER_1() {
    let timer = Timer::new();
    let entered = timer.now_32().ticks();
    CONTROL.with(|control| {
        if !control.alarm.try_expired() {
            return;
        }
        control
            .trace
            .set_state((control.count & 1 == 0).into())
            .unwrap();
        control.alarm.clear_interrupt();
        control.worst_us = control
            .worst_us
            .max(entered.wrapping_sub(control.next.ticks()));
        control.count += 1;
        if control.count < CONTROL_TICKS {
            control.next += PERIOD_US.micros();
            control.arm_failed |= control.alarm.try_arm(control.next).is_err();
        }
    });

    // Count a deadline inside the ISR's measured entry-to-exit interval. A
    // reached deadline on entry is lateness, not a crossing. The wait is far
    // shorter than a low-word half-cycle, including across a timer wrap.
    if DELAY_ACTIVE.load(Acquire) {
        let end = DELAY_END.load(Relaxed);
        let exited = timer.now_32().ticks();
        if (end.wrapping_sub(entered) as i32) > 0 && (exited.wrapping_sub(end) as i32) >= 0 {
            ISR_SPANS.store(ISR_SPANS.load(Relaxed) + 1, Relaxed);
        }
    }
}

#[entry(proc0)]
fn main0(mut core: Proc0, p: rp1::Proc0Resources) -> ! {
    let sys = rp1::platform::init(&mut core, p.platform).unwrap();
    let nonce = status_nonce::publish(&sys.report);
    log!(
        "PARITY_DELAY_BOOT nonce={nonce:08x} PLAT_INIT ok=1 {}\n",
        sys.report
    )
    .unwrap();
    p.proc1.launch(&mut core).unwrap();
    park()
}

#[entry(proc1)]
fn main1(_core: Proc1, p: rp1::Proc1Resources) -> ! {
    let timer = Timer::new();
    let nonce = unsafe { status_nonce::WORD.read_volatile() };
    let mut trace = p.pins.gpio12.into_output(false);
    let control_trace = p.pins.gpio13.into_output(false);

    // The declared ALARM0 and TIMER_0 binding are unique. The lab wrapper above
    // forwards every entry to the normal HAL dispatcher on this core.
    let mut wait = unsafe { WaitTimer::__bind(p.TIMER_ALARM0, p.irq.TIMER_0) };
    let mut alarm = p.TIMER_ALARM1;
    let mut irq = p.irq.TIMER_1;
    irq.set_priority(interrupt::Priority::new(CONTROL_PRIORITY).unwrap());
    alarm.cancel();
    alarm.clear_interrupt();
    irq.unpend();
    log!("PARITY_DELAY_PHASE nonce={nonce:08x} phase=control\n").unwrap();
    let first = timer.now_32() + PERIOD_US.micros();
    alarm.try_arm(first).unwrap();
    alarm.enable_interrupt();
    CONTROL
        .put(Control {
            alarm,
            trace: control_trace,
            next: first,
            count: 0,
            worst_us: 0,
            arm_failed: false,
        })
        .ok()
        .unwrap();
    irq.enable();

    let controlled = measure(&mut trace, |request_us, start| {
        let end = start + (request_us + 1).micros();
        DELAY_END.store(end.ticks() as u32, Relaxed);
        DELAY_ACTIVE.store(true, Release);
        wait.delay_until(end).unwrap();
        DELAY_ACTIVE.store(false, Relaxed);
    });
    let during = CONTROL.with(|control| control.count).unwrap();

    // Allow the remaining absolute control ticks to run. The dedicated wait
    // alarm still bounds the tail if the control IRQ stopped delivering.
    wait.delay_ms(2_000);
    irq.disable();
    let mut control = CONTROL.take().unwrap();
    control.alarm.disable_interrupt();
    control.alarm.cancel();
    control.alarm.clear_interrupt();
    irq.unpend();
    report(nonce, "control", &controlled);

    // With ALARM1 disabled and proc0 parked, a periodic control IRQ cannot
    // release a lost wait. Count ALARM0 entries in both phases.
    log!("PARITY_DELAY_PHASE nonce={nonce:08x} phase=isolated\n").unwrap();
    let isolated = measure(&mut trace, |request_us, _| wait.delay_us(request_us));
    report(nonce, "isolated", &isolated);

    let rows = controlled.iter().chain(&isolated);
    let under = rows.clone().map(|row| row.under).sum::<u32>();
    let over_limit = rows.clone().map(|row| row.over_limit).sum::<u32>();
    let wake_misses = rows.clone().map(|row| row.wake_misses).sum::<u32>();
    let wait_wakes = rows.clone().map(|row| row.wait_wakes).sum::<u32>();
    let isr_spans = rows.map(|row| row.isr_spans).sum::<u32>();
    let missed = CONTROL_TICKS - control.count;
    let late_us = control.worst_us;
    let arm_failed = u8::from(control.arm_failed);
    let ok = u8::from(
        under == 0
            && over_limit == 0
            && wake_misses == 0
            && missed == 0
            && arm_failed == 0
            && late_us <= MAX_CONTROL_LATE_US
            && (1_000..CONTROL_TICKS).contains(&during),
    );
    log!("PARITY_DELAY_END nonce={nonce:08x} settings=5 samples=1000 under={under} over_limit={over_limit} wait_wakes={wait_wakes} wake_misses={wake_misses} isr_spans={isr_spans} period_us={PERIOD_US} ticks={} during={during} missed={missed} max_late_us={late_us} arm_failed={arm_failed} ok={ok}\n",
        control.count).unwrap();
    park()
}

struct Sample {
    min_us: u32,
    max_us: u32,
    under: u32,
    over_limit: u32,
    wait_wakes: u32,
    wake_misses: u32,
    isr_spans: u32,
}

fn measure(trace: &mut Pin<12, Output>, mut delay: impl FnMut(u32, Instant)) -> [Sample; 5] {
    let timer = Timer::new();
    SETTINGS_US.map(|request_us| {
        let before_spans = ISR_SPANS.load(Relaxed);
        let mut sample = Sample {
            min_us: u32::MAX,
            max_us: 0,
            under: 0,
            over_limit: 0,
            wait_wakes: 0,
            wake_misses: 0,
            isr_spans: 0,
        };
        for _ in 0..SAMPLES {
            let before_wakes = WAIT_WAKES.load(Relaxed);
            trace.set_high().unwrap();
            let start = timer.now();
            delay(request_us, start);
            let elapsed = timer.now_32().ticks().wrapping_sub(start.ticks() as u32);
            trace.set_low().unwrap();
            // The reported tick delta overstates elapsed time by up to one tick.
            sample.under += u32::from(elapsed <= request_us);
            sample.over_limit += u32::from(elapsed > request_us + MAX_OVERSHOOT_US);
            sample.min_us = sample.min_us.min(elapsed);
            sample.max_us = sample.max_us.max(elapsed);
            let wakes = WAIT_WAKES.load(Relaxed) - before_wakes;
            sample.wait_wakes += wakes;
            sample.wake_misses +=
                u32::from(wakes > 1 || (request_us >= WAKE_REQUIRED_FROM_US && wakes == 0));
        }
        sample.isr_spans = ISR_SPANS.load(Relaxed) - before_spans;
        sample
    })
}

fn report(nonce: u32, phase: &str, samples: &[Sample; 5]) {
    for (request_us, row) in SETTINGS_US.into_iter().zip(samples) {
        log!("PARITY_DELAY_SAMPLE nonce={nonce:08x} phase={phase} request_us={request_us} n={SAMPLES} under={} over_limit={} min_us={} max_us={} wait_wakes={} wake_misses={} isr_spans={}\n",
            row.under, row.over_limit, row.min_us, row.max_us, row.wait_wakes, row.wake_misses, row.isr_spans).unwrap();
    }
}

fn park() -> ! {
    loop {
        rp1_rt::wait_for_event();
    }
}
