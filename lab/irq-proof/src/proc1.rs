//! proc1's items: its alarm, the critical section and `log!` under a periodic handler, a line
//! without a handler, SPI0 behind the wrapper, and its doorbell (GPIO27) served by a handler:
//! the `ping` RPC and the `down1` ring.
use crate::proof::{
    self, Entries, Shared, Wrapped, expect, mark, now, put, settle, shared, ticker, wait, with,
};
use crate::rings::Reader;
use crate::rp1;
use core::sync::atomic::{AtomicBool, AtomicU8, AtomicU32, Ordering::Relaxed};
use rp1_hal::{log, spi::Spi, timer::Alarm};
use rp1_link::Progress;
use rp1_rt::{
    Proc0, Proc1, interrupt,
    interrupt::{Line, free},
    pac::dw_ssi::int::TXEI,
    sync::Producer,
};

type Spi0 = Spi<0, 11, 10, 9>;
static ALARM: Shared<Alarm<1>> = shared();
static BELL: Shared<rp1::Doorbell1> = shared();
/// The `down1` ring, once main has opened it.
pub static DOWN1: Shared<Reader<27, 1>> = shared();
pub static PING: Shared<rp1::link::Ping> = shared();
static SPI: Shared<Spi0> = shared();
static WRAP: Entries = Entries::new();
/// Handler entries with the doorbell set.
static BELL_ENTRIES: Entries = Entries::new();
/// Ping requests served in the doorbell handler.
static PINGS: Entries = Entries::new();

/// What the alarm handler does besides ticking.
const TICK_ONLY: u8 = 0;
const CHECK_PAIR: u8 = 1;
const LOG_LINE: u8 = 2;
static TICK_JOB: AtomicU8 = AtomicU8::new(TICK_ONLY);

#[interrupt(proc1)]
fn TIMER_1() {
    ticker::on_alarm(&ALARM, 1);
    match TICK_JOB.load(Relaxed) {
        CHECK_PAIR => {
            if !pair_consistent() {
                proof::bump(&TORN);
            }
        }
        LOG_LINE => log_from_handler(),
        _ => {}
    }
}
#[interrupt(proc1)]
fn SPI0() {
    with(&SPI, |spi| {
        spi.disable_interrupt(TXEI);
        // Read back so that the write has finished before the handler returns.
        spi.interrupt_status();
    });
    WRAP.enter(1);
}
/// Both endpoints share a force bit. Ack before draining either one.
#[interrupt(proc1)]
fn IO_BANK0() {
    let Some(bell) = proof::get(&BELL) else {
        return;
    };
    bell.ack();
    BELL_ENTRIES.enter(1);
    let mut more = false;
    with(&PING, |ping| {
        let progress = ping.serve(8, |value| {
            PINGS.enter(1);
            value
        });
        more = match progress {
            Ok(Progress::More) => true,
            Ok(Progress::Idle) => false,
            Err(_) => panic!("ping serve failed"),
        };
    });
    with(&DOWN1, |reader| more |= reader.drain());
    if more {
        bell.raise();
    }
}

const SPI0_BASE: usize = 0x4005_0000;

pub struct Parts {
    pub alarm1: Alarm<1>,
    pub alarm2: Alarm<2>,
    pub spi: Option<Spi0>,
    pub doorbell: rp1::Doorbell1,
    pub irq: rp1::Proc1Irq,
}

/// Runs the items that need no Linux, then leaves the doorbell handler armed.
pub fn run(parts: Parts) {
    let Parts {
        alarm1,
        alarm2,
        spi,
        doorbell,
        mut irq,
    } = parts;
    let idle: &mut dyn FnMut() = &mut || {};
    put(&ALARM, alarm1);
    proof::timer_test(&ALARM, &mut irq.TIMER_1, 1, idle);
    critical_section_test(&mut irq.TIMER_1, idle);
    log_test(&mut irq.TIMER_1);
    default_handler_test(alarm2, &mut irq.TIMER_2, idle);

    put(&BELL, doorbell);
    proof::doorbell_test(doorbell, &BELL_ENTRIES, &mut irq.IO_BANK0, idle);

    match spi {
        Some(spi) => {
            put(&SPI, spi);
            // The driver leaves the controller enabled with an empty TX FIFO: TXEI is active.
            proof::wrap_test(
                &Wrapped {
                    name: "spi0",
                    status: SPI0_BASE + 0x30,
                    mask: SPI0_BASE + 0x2c,
                    source: TXEI,
                    wrap: SPI0_BASE + 0x108,
                    entries: &WRAP,
                    fail: "p1_wrap_spi0",
                },
                &mut irq.SPI0,
                &mut || with(&SPI, |spi| spi.enable_interrupt(TXEI)),
                idle,
            );
        }
        None => expect(false, "p1_spi0_init"),
    }

    PINGS.reset();
    irq.IO_BANK0.enable();
}

// The pair a critical section must keep consistent: B is always !A. Two atomics so that the
// unmasked control is still sound Rust; Relaxed loads and stores are plain LDR/STR, so the pair
// can tear exactly as two ordinary words would (no LDREX/STREX read-modify-write).
static PAIR_A: AtomicU32 = AtomicU32::new(0);
static PAIR_B: AtomicU32 = AtomicU32::new(!0);
static TORN: AtomicU32 = AtomicU32::new(0);

fn pair_consistent() -> bool {
    PAIR_B.load(Relaxed) == !PAIR_A.load(Relaxed)
}
/// Updates the pair in two steps, with a gap that an interrupt can fall into.
fn pair_update() {
    let a = PAIR_A.load(Relaxed).wrapping_add(1);
    PAIR_A.store(a, Relaxed);
    // Not `nomem`: the compiler must then assume the block reads memory, so the store to A
    // stays in front of the gap whatever the optimiser does.
    unsafe { core::arch::asm!(".rept 16", "nop", ".endr", options(nostack)) };
    PAIR_B.store(!a, Relaxed);
}

/// `IRQ_CS`: main updates the pair inside critical sections while a 10 kHz handler checks it.
/// The control repeats the update without masking and must be caught: otherwise `torn=0`
/// would say nothing.
fn critical_section_test(line: &mut Line<27>, idle: &mut dyn FnMut()) {
    const ITERATIONS: u32 = 100_000;
    const HANDLER_RUNS: u32 = 1000;
    TORN.store(0, Relaxed);
    TICK_JOB.store(CHECK_PAIR, Relaxed);
    line.enable();
    ticker::start(&ALARM, 100, u32::MAX);
    let start = now();
    let mut iters = 0u32;
    while (iters < ITERATIONS || ticker::TICKS.count() < HANDLER_RUNS)
        && now().wrapping_sub(start) < 5_000_000
    {
        free(|_| pair_update());
        iters += 1;
    }
    let (isr_n, torn) = (ticker::TICKS.count(), TORN.load(Relaxed));

    TORN.store(0, Relaxed);
    let from = ticker::TICKS.count();
    wait(
        || {
            // Deliberately without a critical section: a torn pair is what this control is
            // meant to produce.
            pair_update();
            ticker::TICKS.count().wrapping_sub(from) >= HANDLER_RUNS
        },
        1_000_000,
        idle,
    );
    ticker::stop(&ALARM);
    line.disable();
    TICK_JOB.store(TICK_ONLY, Relaxed);
    let control_torn = TORN.load(Relaxed);
    let masked = crate::cs_stats::max_us();
    mark!(
        "IRQ_CS iters={} isr_n={} torn={} control_torn={} max_masked_us={}",
        iters,
        isr_n,
        torn,
        control_torn,
        masked
    );
    let ok = iters >= ITERATIONS
        && isr_n >= HANDLER_RUNS
        && torn == 0
        && control_torn != 0
        && masked <= proof::MAX_MASKED_US;
    expect(ok, "p1_cs");
}

static HANDLER_LINES: AtomicU32 = AtomicU32::new(0);
static DROP_MAX_US: AtomicU32 = AtomicU32::new(0);

fn log_from_handler() {
    let (dropped, start) = (rp1::log::dropped(), now());
    let k = proof::bump(&HANDLER_LINES);
    let _ = log!("IRQ_LOG_ISR k={}\n", k);
    // How long a refused message kept the handler: it must not have waited for the lock.
    if rp1::log::dropped() != dropped {
        let us = now().wrapping_sub(start);
        if us > DROP_MAX_US.load(Relaxed) {
            DROP_MAX_US.store(us, Relaxed);
        }
    }
}

/// `IRQ_LOG`: main writes numbered lines while a 1 kHz handler also logs. The handler's lines
/// are dropped while main holds the log, without waiting; main's lines all come out whole.
fn log_test(line: &mut Line<27>) {
    const MAIN_LINES: u32 = 20;
    let dropped = rp1::log::dropped();
    HANDLER_LINES.store(0, Relaxed);
    TICK_JOB.store(LOG_LINE, Relaxed);
    line.enable();
    ticker::start(&ALARM, 1000, u32::MAX);
    for i in 0..MAIN_LINES {
        mark!("IRQ_LOG_MAIN i={}", i);
    }
    ticker::stop(&ALARM);
    line.disable();
    TICK_JOB.store(TICK_ONLY, Relaxed);
    let tries = HANDLER_LINES.load(Relaxed);
    let isr_dropped = rp1::log::dropped().wrapping_sub(dropped);
    let drop_max_us = DROP_MAX_US.load(Relaxed);
    mark!(
        "IRQ_LOG main_lines={} isr_tries={} isr_dropped={} isr_logged={} drop_max_us={}",
        MAIN_LINES,
        tries,
        isr_dropped,
        tries.wrapping_sub(isr_dropped),
        drop_max_us
    );
    expect(
        isr_dropped != 0 && isr_dropped <= tries && drop_max_us <= proof::MAX_MASKED_US,
        "p1_log",
    );
}

/// `IRQ_DEFAULT`: ALARM2's line is enabled with no handler. The default handler must mask the
/// line after one entry and return, and this core must keep running.
fn default_handler_test(mut alarm: Alarm<2>, line: &mut Line<28>, idle: &mut dyn FnMut()) {
    let pre_count = interrupt::unhandled().map_or(0, |(_, count)| count);
    alarm.clear_interrupt();
    line.unpend();
    let pre = line.is_pending();
    let _ = alarm.schedule(now().wrapping_add(200));
    alarm.enable_interrupt();
    line.enable();
    wait(|| interrupt::unhandled().is_some(), 10_000, idle);
    // The alarm still asserts the line: a line left enabled would re-enter here.
    settle(1000, idle);
    let (irq, count) = interrupt::unhandled().unwrap_or((u8::MAX, 0));
    let masked = !line.is_enabled();
    line.disable();
    alarm.disable_interrupt();
    alarm.clear_interrupt();
    line.unpend();
    mark!(
        "IRQ_DEFAULT irq={} count={} masked={} pre_count={} pre_pending={}",
        irq,
        count,
        masked as u32,
        pre_count,
        pre as u32
    );
    expect(
        irq == 28 && count == 1 && masked && pre_count == 0 && !pre,
        "p1_default",
    );
}

static REPORTED: AtomicBool = AtomicBool::new(false);
const REPORT_PINGS: u32 = 1000;
/// Whether the first ping run is complete and ready to report.
#[inline(always)]
pub fn report_due() -> bool {
    PINGS.count() >= REPORT_PINGS && !REPORTED.load(Relaxed)
}

/// proc1's figures for proc0's final lines, in the order they are sent.
pub struct Report {
    pub fails: u32,
    pub wrong_core: u32,
    pub unhandled: u32,
    pub stack_free: u32,
    pub masked_us: u32,
}
impl Report {
    pub const WORDS: usize = 5;
    pub fn from_words(w: [u32; Self::WORDS]) -> Self {
        Self {
            fails: w[0],
            wrong_core: w[1],
            unhandled: w[2],
            stack_free: w[3],
            masked_us: w[4],
        }
    }
}

/// `IRQ_P1_GPIO27`, then proc1's figures to proc0. Main code never answers a doorbell in this
/// image, so every reply Linux got came from the handler.
pub fn report(tx: &mut Producer<'_, u32, 8, Proc1, Proc0>, stack_free: u32) {
    REPORTED.store(true, Relaxed);
    let ipsr = PINGS.ipsr();
    mark!("IRQ_P1_GPIO27 ipsr={} seen={}", ipsr, PINGS.count());
    expect(ipsr == 16, "p1_gpio27");
    let unhandled = interrupt::unhandled().map_or(0, |(_, count)| count);
    for word in [
        proof::fails(),
        proof::wrong_core(),
        unhandled,
        stack_free,
        crate::cs_stats::max_us(),
    ] {
        let _ = tx.push(word);
    }
}
