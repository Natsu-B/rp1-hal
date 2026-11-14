//! proc1's items: its alarm, the critical section and `log!` under a periodic handler, a line
//! without a handler, SPI0 behind the wrapper, and Linux's GPIO27 doorbell served by a handler.
use crate::proof::{
    self, Entries, Shared, Wrapped, expect, mark, now, put, settle, shared, ticker, wait, with,
};
use crate::rp1;
use core::cell::Cell;
use core::sync::atomic::{AtomicBool, AtomicU8, AtomicU32, Ordering::Relaxed};
use rp1_hal::{gpio::PinIrq, log, spi::Spi, timer::Alarm};
use rp1_proto::link;
use rp1_rt::{
    Proc0, Proc1, interrupt,
    interrupt::{CriticalSection, Line, Mutex, free},
    pac::dw_ssi::int::TXEI,
    sync::Producer,
};

type Spi0 = Spi<0, 11, 10, 9>;
static ALARM: Shared<Alarm<1>> = shared();
static PIN: Shared<PinIrq<27, 1>> = shared();
static SPI: Shared<Spi0> = shared();
static WRAP: Entries = Entries::new();
static DOORBELL: Entries = Entries::new();

/// What the alarm handler does besides ticking.
const TICK_ONLY: u8 = 0;
const CHECK_PAIR: u8 = 1;
const LOG_LINE: u8 = 2;
static TICK_JOB: AtomicU8 = AtomicU8::new(TICK_ONLY);

#[interrupt(proc1)]
fn TIMER_1() {
    ticker::on_alarm(&ALARM, 1);
    match TICK_JOB.load(Relaxed) {
        CHECK_PAIR => free(|cs| {
            if !pair_consistent(cs) {
                proof::bump(&TORN);
            }
        }),
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
/// Linux's doorbell: it wrote a request word and forced GPIO27 in PROC1_INTF. Clear the force,
/// then echo the request as the reply (the protocol of examples/linux-doorbell).
#[interrupt(proc1)]
fn IO_BANK0() {
    with(&PIN, |pin| pin.unforce());
    let page = rp1_proto::LINK_START as usize;
    proof::wr(
        page + link::DOORBELL_REPLY,
        proof::rd(page + link::DOORBELL_REQUEST),
    );
    rp1_proto::barrier();
    DOORBELL.enter(1);
    SERVED.store(true, Relaxed);
}

const SPI0_BASE: usize = 0x4005_0000;

pub struct Parts {
    pub alarm1: Alarm<1>,
    pub alarm2: Alarm<2>,
    pub spi: Option<Spi0>,
    pub irq: rp1::Proc1Irq,
}

/// Runs the items that need no Linux, then leaves the doorbell handler armed.
pub fn run(parts: Parts) {
    let Parts {
        alarm1,
        alarm2,
        spi,
        mut irq,
    } = parts;
    let idle: &mut dyn FnMut() = &mut || {};
    put(&ALARM, alarm1);
    proof::timer_test(&ALARM, &mut irq.TIMER_1, 1, idle);
    critical_section_test(&mut irq.TIMER_1, idle);
    log_test(&mut irq.TIMER_1);
    default_handler_test(alarm2, &mut irq.TIMER_2, idle);

    proof::intf_only(&mut irq.gpio27, &mut irq.IO_BANK0, idle);
    put(&PIN, irq.gpio27);

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

    DOORBELL.reset();
    irq.IO_BANK0.enable();
}

// The pair a critical section must keep consistent: B is always !A.
static PAIR_A: Mutex<Cell<u32>> = Mutex::new(Cell::new(0));
static PAIR_B: Mutex<Cell<u32>> = Mutex::new(Cell::new(!0));
static TORN: AtomicU32 = AtomicU32::new(0);

fn pair_consistent(cs: CriticalSection<'_>) -> bool {
    PAIR_B.borrow(cs).get() == !PAIR_A.borrow(cs).get()
}
/// Updates the pair in two steps, with a gap that an interrupt can fall into.
fn pair_update(cs: CriticalSection<'_>) {
    let a = PAIR_A.borrow(cs).get().wrapping_add(1);
    PAIR_A.borrow(cs).set(a);
    unsafe { core::arch::asm!(".rept 16", "nop", ".endr", options(nomem, nostack)) };
    PAIR_B.borrow(cs).set(!a);
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
        free(pair_update);
        iters += 1;
    }
    let (isr_n, torn) = (ticker::TICKS.count(), TORN.load(Relaxed));

    TORN.store(0, Relaxed);
    let from = ticker::TICKS.count();
    wait(
        || {
            // SAFETY: deliberately not a critical section; the pair is two `Cell<u32>` that
            // only this core touches, and a torn pair is what this control is meant to produce.
            pair_update(unsafe { CriticalSection::new() });
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

static SERVED: AtomicBool = AtomicBool::new(false);
static REPORTED: AtomicBool = AtomicBool::new(false);
const DOORBELLS: u32 = 1000;

/// Whether the handler answered a doorbell since the last call. Main then signals Linux
/// (`rp1:irq`), which the handler leaves to it: `HostIrq` is main's. A reply between the load
/// and the store shares the signal of the one before it; a signal is only a hint to re-check.
#[inline(always)]
pub fn doorbell_served() -> bool {
    let served = SERVED.load(Relaxed);
    if served {
        SERVED.store(false, Relaxed);
    }
    served
}
/// The 1000 doorbells of `linux-doorbell` are in and [`report`] has not run yet.
#[inline(always)]
pub fn report_due() -> bool {
    DOORBELL.count() >= DOORBELLS && !REPORTED.load(Relaxed)
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
    let ipsr = DOORBELL.ipsr();
    mark!(
        "IRQ_P1_GPIO27 ipsr={} seen={}",
        ipsr,
        DOORBELL.count()
    );
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
