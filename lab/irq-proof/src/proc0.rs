//! proc0's items: its alarm, its own IO_BANK0 line (GPIO22, forced, and its doorbell GPIO26)
//! and I2C1 behind the wrapper. proc0 also runs the system services, so every wait here goes
//! through `idle`. Once shm is up the IO_BANK0 handler drains `down0` and the alarm handler can
//! send on `tick0`.
use crate::proof::{self, Entries, Wrapped, expect, mark, settle, ticker, wait};
use crate::rings::{Reader, Ticks};
use crate::rp1;
use core::sync::atomic::{AtomicBool, Ordering::Relaxed};
use rp1_hal::{
    gpio::{Pin, PinIrq},
    i2c::I2c,
    timer::Alarm,
};
use rp1_rt::{
    interrupt,
    interrupt::{Line, Slot},
    pac::dw_i2c::intr::TX_EMPTY,
};

type I2c1 = I2c<1, (Pin<2>, Pin<3>)>;
static ALARM: Slot<Option<Alarm<0>>, 0> = Slot::empty();
static PIN: Slot<Option<PinIrq<22, 0>>, 0> = Slot::empty();
static I2C: Slot<Option<I2c1>, 0> = Slot::empty();
static BELL: Slot<Option<rp1::Doorbell0>, 0> = Slot::empty();
/// The `down0` ring and the `tick0` writer, once main has opened them.
pub static DOWN0: Slot<Option<Reader<26, 0>>, 0> = Slot::empty();
pub static TICK0: Slot<Option<Ticks>, 0> = Slot::empty();
static GPIO: Entries = Entries::new();
static WRAP: Entries = Entries::new();
/// Handler entries with the doorbell set.
static BELL_ENTRIES: Entries = Entries::new();
/// The alarm handler also sends on `tick0`.
static SEND_TICKS: AtomicBool = AtomicBool::new(false);

#[interrupt(proc0)]
fn TIMER_0() {
    ticker::on_alarm(&ALARM, 0);
    if SEND_TICKS.load(Relaxed) {
        TICK0.with(|ticks| ticks.on_tick());
    }
}
/// GPIO22's forced interrupt, or the doorbell: Linux forced GPIO26 in PROC0_INTF for `down0`.
/// The doorbell is acknowledged before the drain: an ack after it could drop a later
/// notification.
#[interrupt(proc0)]
fn IO_BANK0() {
    PIN.with(|pin| {
        if pin.is_forced() {
            pin.unforce();
            // Read back so that the write has finished before the handler returns.
            pin.is_forced();
            GPIO.enter(0);
        }
    });
    if let Some(bell) = BELL.with(|bell| *bell)
        && bell.is_pending()
    {
        bell.ack_and_signal();
        BELL_ENTRIES.enter(0);
        DOWN0.with(|reader| reader.on_doorbell(bell));
    }
}

/// `TICKS`: send `count` messages on `tick0`, one per alarm interrupt every `period_us`.
pub fn start_ticks(count: u32, period_us: u32, generation: u32) {
    ticker::stop(&ALARM);
    TICK0.with(|ticks| ticks.start(generation));
    SEND_TICKS.store(true, Relaxed);
    ticker::start(&ALARM, period_us.max(50), count);
}
/// The latest tick run's lateness, for `TICK_REPORT`.
pub fn tick_late_max_us() -> u32 {
    ticker::late_max_us()
}
#[interrupt(proc0)]
fn I2C1() {
    I2C.with(|i2c| {
        i2c.disable_interrupt(TX_EMPTY);
        i2c.interrupt_status();
    });
    WRAP.enter(0);
}

const I2C1_BASE: usize = 0x4007_4000;
const I2C1_ENABLE: usize = I2C1_BASE + 0x6c;

pub fn run(
    alarm: Alarm<0>,
    i2c: Option<I2c1>,
    doorbell: rp1::Doorbell0,
    mut irq: rp1::Proc0Irq,
    idle: &mut dyn FnMut(),
) {
    ALARM.put(alarm).ok().unwrap();
    proof::timer_test(&ALARM, &mut irq.TIMER_0, 0, idle);

    proof::intf_only(&mut irq.gpio22, &mut irq.IO_BANK0, idle);
    PIN.put(irq.gpio22).ok().unwrap();
    forced_pin(&mut irq.IO_BANK0, idle);
    BELL.put(doorbell).ok().unwrap();
    proof::doorbell_test(doorbell, &BELL_ENTRIES, &mut irq.IO_BANK0, idle);

    match i2c {
        Some(i2c) => {
            I2C.put(i2c).ok().unwrap();
            // TX_EMPTY is only reported by an enabled controller. Enabling it starts nothing
            // on the bus: no command is ever written.
            proof::wr(I2C1_ENABLE, 1);
            proof::wrap_test(
                &Wrapped {
                    name: "i2c1",
                    status: I2C1_BASE + 0x2c,
                    mask: I2C1_BASE + 0x30,
                    source: TX_EMPTY,
                    wrap: I2C1_BASE + 0x108,
                    entries: &WRAP,
                    fail: "p0_wrap_i2c1",
                },
                &mut irq.I2C1,
                &mut || {
                    I2C.with(|i2c| i2c.enable_interrupt(TX_EMPTY));
                },
                idle,
            );
            proof::wr(I2C1_ENABLE, 0);
        }
        None => expect(false, "p0_i2c1_init"),
    }
    // Left enabled for the shm items: the doorbell drains `down0`, the alarm sends `tick0`.
    irq.IO_BANK0.enable();
    irq.TIMER_0.enable();
}

/// `IRQ_P0_GPIO22`: proc0's IO_BANK0 handler, entered 100 times by `PinIrq::force`. The pin
/// itself is left as it is: a forced interrupt does not depend on the pad.
fn forced_pin(line: &mut Line<0>, idle: &mut dyn FnMut()) {
    GPIO.reset();
    line.unpend();
    let pre = line.is_pending();
    line.enable();
    for n in 1..=100 {
        PIN.with(|pin| pin.force());
        if !wait(|| GPIO.count() >= n, 1000, idle) {
            break;
        }
    }
    settle(1000, idle);
    line.disable();
    PIN.with(|pin| pin.unforce());
    line.unpend();
    let (n, ipsr, post) = (GPIO.count(), GPIO.ipsr(), line.is_pending());
    mark!(
        "IRQ_P0_GPIO22 n={} ipsr={} pre_pending={} post_pending={}",
        n,
        ipsr,
        pre as u32,
        post as u32
    );
    expect(n == 100 && ipsr == 16 && !pre && !post, "p0_gpio22");
}

/// The last lines of the proof, once proc1 has sent its figures.
pub fn finish(p0_stack_free: u32, p1: &crate::proc1::Report) {
    let p0_unhandled = interrupt::unhandled().map_or(0, |(_, count)| count);
    let p0_masked = crate::cs_stats::max_us();
    mark!(
        "IRQ_XCORE p0_seen_27={} p1_seen_26={}",
        proof::wrong_core(),
        p1.wrong_core
    );
    mark!("IRQ_UNHANDLED p0={} p1={}", p0_unhandled, p1.unhandled);
    mark!(
        "IRQ_STACK p0_free={} p1_free={} p0_size={} p1_size={}",
        p0_stack_free,
        p1.stack_free,
        crate::stack::declared(0),
        crate::stack::declared(1)
    );
    mark!(
        "IRQ_MASKED p0_max_us={} p1_max_us={}",
        p0_masked,
        p1.masked_us
    );
    expect(proof::wrong_core() == 0 && p1.wrong_core == 0, "xcore");
    // proc1's one unhandled interrupt is the IRQ_DEFAULT item.
    expect(p0_unhandled == 0 && p1.unhandled == 1, "unhandled");
    expect(
        p0_stack_free >= proof::MIN_STACK_FREE && p1.stack_free >= proof::MIN_STACK_FREE,
        "stack",
    );
    expect(
        p0_masked <= proof::MAX_MASKED_US && p1.masked_us <= proof::MAX_MASKED_US,
        "masked",
    );
    if proof::fails() == 0 && p1.fails == 0 {
        mark!("IRQ_PROOF_PASS");
    } else {
        mark!(
            "IRQ_PROOF_FAIL summary p0_fails={} p1_fails={}",
            proof::fails(),
            p1.fails
        );
    }
}
