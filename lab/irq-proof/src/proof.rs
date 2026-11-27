//! What both cores' proofs share. Ordinary statics have one copy per core (private DSRAM), so
//! a counter here is the running core's own, and the same handler code serves either core.
use core::sync::atomic::{AtomicU32, Ordering::Relaxed};
use rp1_hal::{fugit::TimerInstantU32, gpio::PinIrq, timer::Alarm};
use rp1_link::notify::Doorbell;
use rp1_rt::interrupt::{Line, Slot};

/// A driver object that main and a handler of the same core share.
pub type Shared<T, const CORE: u8> = Slot<Option<T>, CORE>;
pub const fn shared<T, const CORE: u8>() -> Shared<T, CORE> {
    Slot::new(None)
}
pub fn put<T, const CORE: u8>(slot: &Shared<T, CORE>, value: T) {
    slot.try_with(|slot| *slot = Some(value)).unwrap();
}
/// Runs `f` on the object in `slot`, if it holds one.
pub fn with<T, const CORE: u8>(slot: &Shared<T, CORE>, f: impl FnOnce(&mut T)) {
    slot.try_with(|slot| {
        if let Some(value) = slot {
            f(value)
        }
    });
}

/// A copy of the object in `slot`, if it holds one (for the zero-sized doorbell).
pub fn get<T: Copy, const CORE: u8>(slot: &Shared<T, CORE>) -> Option<T> {
    slot.try_with(|slot| *slot).flatten()
}

static NONCE: AtomicU32 = AtomicU32::new(0);
pub fn set_nonce(nonce: u32) {
    NONCE.store(nonce, Relaxed);
}
pub fn nonce() -> u32 {
    NONCE.load(Relaxed)
}
/// One marker line. Every line carries the boot's nonce, so that a checker can tell a line of
/// another boot in the same capture.
macro_rules! mark {
    ($fmt:literal $(, $arg:expr)* $(,)?) => {
        $crate::rp1::log::write(core::format_args!(
            concat!($fmt, " nonce={:08x}\n") $(, $arg)*, $crate::proof::nonce()
        )).unwrap()
    };
}
pub(crate) use mark;

/// Adds one to a counter with a single writer context: main only, or handlers only (they
/// share one priority, so they do not preempt each other). A plain load and store is enough
/// there, and keeps exclusive accesses out of the image.
pub fn bump(counter: &AtomicU32) -> u32 {
    let old = counter.load(Relaxed);
    counter.store(old.wrapping_add(1), Relaxed);
    old
}

static FAILS: AtomicU32 = AtomicU32::new(0);
/// Failed items on this core.
pub fn fails() -> u32 {
    FAILS.load(Relaxed)
}
/// Records `item` as failed unless `ok`. The numbers are in the item's own marker line.
pub fn expect(ok: bool, item: &str) {
    if !ok {
        bump(&FAILS);
        mark!("IRQ_PROOF_FAIL {}", item);
    }
}

pub fn now() -> u32 {
    rd(0x400a_c028)
}
pub fn rd(address: usize) -> u32 {
    unsafe { (address as *const u32).read_volatile() }
}
pub fn wr(address: usize, value: u32) {
    unsafe { (address as *mut u32).write_volatile(value) }
}
fn ipsr() -> u32 {
    let ipsr: u32;
    unsafe {
        core::arch::asm!("mrs {}, IPSR", out(reg) ipsr, options(nomem, nostack, preserves_flags))
    };
    ipsr
}
/// 0 on proc0, 1 on proc1 (the per-core ID register that `log!` also uses).
fn core_id() -> u32 {
    rd(0xe00f_f01c) & 1
}

/// Polls `done` for up to `timeout_us`, running `idle` in between: proc0 keeps its system
/// services alive there. Returns whether `done` came true.
pub fn wait(mut done: impl FnMut() -> bool, timeout_us: u32, idle: &mut dyn FnMut()) -> bool {
    let start = now();
    loop {
        if done() {
            return true;
        }
        if now().wrapping_sub(start) >= timeout_us {
            return false;
        }
        idle();
    }
}
/// Lets a register write reach the NVIC, or a wrong extra interrupt show up.
pub fn settle(us: u32, idle: &mut dyn FnMut()) {
    wait(|| false, us, idle);
}

/// Handler entries on the wrong core: a proc0 handler that ran on proc1 adds to proc1's copy,
/// and the other way round.
static WRONG_CORE: AtomicU32 = AtomicU32::new(0);
pub fn wrong_core() -> u32 {
    WRONG_CORE.load(Relaxed)
}

/// What one handler records about its entries.
pub struct Entries {
    count: AtomicU32,
    ipsr: AtomicU32,
}
impl Entries {
    pub const fn new() -> Self {
        Self {
            count: AtomicU32::new(0),
            ipsr: AtomicU32::new(0),
        }
    }
    /// Called from the handler written for `core`.
    pub fn enter(&self, core: u32) {
        self.ipsr.store(ipsr(), Relaxed);
        if core_id() != core {
            bump(&WRONG_CORE);
        }
        bump(&self.count);
    }
    pub fn reset(&self) {
        self.count.store(0, Relaxed);
        self.ipsr.store(0, Relaxed);
    }
    pub fn count(&self) -> u32 {
        self.count.load(Relaxed)
    }
    /// IPSR of the last entry: 16 + the IRQ number.
    pub fn ipsr(&self) -> u32 {
        self.ipsr.load(Relaxed)
    }
}

/// A periodic alarm interrupt: the handler re-arms the alarm one period after the previous
/// deadline, so the rate does not drift with the handler's latency.
pub mod ticker {
    use super::*;

    pub static TICKS: Entries = Entries::new();
    static PERIOD_US: AtomicU32 = AtomicU32::new(0);
    static LIMIT: AtomicU32 = AtomicU32::new(0);
    static DEADLINE: AtomicU32 = AtomicU32::new(0);
    static LAST_AT: AtomicU32 = AtomicU32::new(0);
    static LATE_MAX_US: AtomicU32 = AtomicU32::new(0);
    static RESYNCS: AtomicU32 = AtomicU32::new(0);

    /// Starts ticking every `period_us`; the handler stops by itself after `limit` ticks.
    pub fn start<const N: u8, const CORE: u8>(
        alarm: &Shared<Alarm<N>, CORE>,
        period_us: u32,
        limit: u32,
    ) {
        TICKS.reset();
        LATE_MAX_US.store(0, Relaxed);
        RESYNCS.store(0, Relaxed);
        PERIOD_US.store(period_us, Relaxed);
        LIMIT.store(limit, Relaxed);
        with(alarm, |alarm| {
            let deadline = now().wrapping_add(period_us);
            DEADLINE.store(deadline, Relaxed);
            alarm.clear_interrupt();
            let _ = alarm.schedule(TimerInstantU32::from_ticks(deadline));
            alarm.enable_interrupt();
        });
    }
    pub fn stop<const N: u8, const CORE: u8>(alarm: &Shared<Alarm<N>, CORE>) {
        with(alarm, |alarm| {
            alarm.disable_interrupt();
            alarm.cancel();
            alarm.clear_interrupt();
        });
    }
    /// The alarm handler of the core that owns `alarm`.
    pub fn on_alarm<const N: u8, const CORE: u8>(alarm: &Shared<Alarm<N>, CORE>, core: u32) {
        let entered = now();
        let late = entered.wrapping_sub(DEADLINE.load(Relaxed));
        if late > LATE_MAX_US.load(Relaxed) {
            LATE_MAX_US.store(late, Relaxed);
        }
        LAST_AT.store(entered, Relaxed);
        with(alarm, |alarm| {
            alarm.clear_interrupt();
            if TICKS.count() + 1 >= LIMIT.load(Relaxed) {
                alarm.disable_interrupt();
                return;
            }
            let period = PERIOD_US.load(Relaxed);
            let mut next = DEADLINE.load(Relaxed).wrapping_add(period);
            // A handler that ran past the next deadline starts a new series from now.
            if alarm.schedule(TimerInstantU32::from_ticks(next)).is_err() {
                bump(&RESYNCS);
                next = now().wrapping_add(period);
                let _ = alarm.schedule(TimerInstantU32::from_ticks(next));
            }
            DEADLINE.store(next, Relaxed);
        });
        TICKS.enter(core);
    }
    /// When the last handler entry began (timer microseconds).
    pub fn last_at() -> u32 {
        LAST_AT.load(Relaxed)
    }
    /// Longest delay from a deadline to its handler's entry since [`start`].
    pub fn late_max_us() -> u32 {
        LATE_MAX_US.load(Relaxed)
    }
    pub fn resyncs() -> u32 {
        RESYNCS.load(Relaxed)
    }
}

/// `IRQ_Pc_TIMERn`: 1000 alarm interrupts at 1 kHz enter this core's handler and return.
pub fn timer_test<const N: u8, const IRQ: u8, const CORE: u8>(
    alarm: &Shared<Alarm<N>, CORE>,
    line: &mut Line<IRQ>,
    core: u32,
    idle: &mut dyn FnMut(),
) {
    ticker::stop(alarm);
    line.unpend();
    let pre = line.is_pending();
    line.enable();
    let start = now();
    ticker::start(alarm, 1000, 1000);
    wait(|| ticker::TICKS.count() >= 1000, 3_000_000, idle);
    // The time of the 1000th entry, not of this loop noticing it.
    let us = ticker::last_at().wrapping_sub(start);
    // A tick after the limit would show up here.
    settle(5000, idle);
    line.disable();
    let (n, ipsr, post) = (
        ticker::TICKS.count(),
        ticker::TICKS.ipsr(),
        line.is_pending(),
    );
    mark!(
        "IRQ_P{}_TIMER{} n={} ipsr={} us={} late_max_us={} resync={} pre_pending={} post_pending={}",
        core,
        N,
        n,
        ipsr,
        us,
        ticker::late_max_us(),
        ticker::resyncs(),
        pre as u32,
        post as u32
    );
    let ok = n == 1000
        && ipsr == 16 + u32::from(IRQ)
        && (999_000..=1_010_000).contains(&us)
        && !pre
        && !post;
    expect(ok, if core == 0 { "p0_timer0" } else { "p1_timer1" });
}

/// `IRQ_INTF_ONLY`: does PROCn_INTF alone (INTE = 0) raise the pin in PROCn_INTS and at this
/// core's NVIC? `PinIrq::force` and Linux's doorbell write rely on it. If it does not, INTE is
/// set here with a raw write (the HAL sets it only together with an event) and left set, so
/// that the forced-interrupt items after this one can still run; `need_inte` says so.
/// The NVIC line stays disabled: nothing enters a handler.
pub fn intf_only<const N: u8, const CORE: u8>(
    pin: &mut PinIrq<N, CORE>,
    line: &mut Line<0>,
    idle: &mut dyn FnMut(),
) {
    const INTE_SET: usize = 0x400d_2104;
    pin.unforce();
    settle(100, idle);
    line.unpend();
    let pre = pin.is_pending() || line.is_pending();
    pin.force();
    settle(100, idle);
    let (ints, nvic) = (pin.is_pending(), line.is_pending());
    if !ints {
        wr(INTE_SET + usize::from(CORE) * 0xc, 1 << N);
        settle(100, idle);
        mark!(
            "IRQ_INTF_INTE core={} pin={} ints={} nvic={}",
            CORE,
            N,
            pin.is_pending() as u32,
            line.is_pending() as u32
        );
    }
    pin.unforce();
    settle(100, idle);
    line.unpend();
    let post = pin.is_pending() || line.is_pending();
    mark!(
        "IRQ_INTF_ONLY core={} pin={} pre_pending={} ints={} nvic={} post_pending={} need_inte={}",
        CORE,
        N,
        pre as u32,
        ints as u32,
        nvic as u32,
        post as u32,
        !ints as u32
    );
    // Whether INTF alone is enough is a finding, not a pass condition; a pin that was pending
    // before the force, or stays pending after it, would spoil the items that follow.
    expect(!pre && !post, if CORE == 0 { "p0_intf" } else { "p1_intf" });
}

/// `IRQ_Pc_DBELL`: this core raises its own doorbell once (the `PROCn_INTF` force Linux sets)
/// and the IO_BANK0 handler, which acknowledges it, runs exactly once. `entries` is the
/// handler's count of entries with the doorbell set.
pub fn doorbell_test<const PIN: u8, const CORE: u8>(
    bell: Doorbell<PIN, CORE>,
    entries: &Entries,
    line: &mut Line<0>,
    idle: &mut dyn FnMut(),
) {
    entries.reset();
    line.unpend();
    let pre = bell.is_pending() || line.is_pending();
    line.enable();
    bell.raise();
    wait(|| entries.count() != 0, 1000, idle);
    // A second entry would mean that the handler's ack did not drop the force.
    settle(1000, idle);
    line.disable();
    line.unpend();
    let (n, ipsr) = (entries.count(), entries.ipsr());
    let post = bell.is_pending() || line.is_pending();
    mark!(
        "IRQ_P{}_DBELL pin={} n={} ipsr={} pre_pending={} post_pending={}",
        CORE,
        PIN,
        n,
        ipsr,
        pre as u32,
        post as u32
    );
    expect(
        n == 1 && ipsr == 16 && !pre && !post,
        if CORE == 0 { "p0_dbell" } else { "p1_dbell" },
    );
}

/// A DesignWare controller behind the RP1 interrupt wrapper (`base + 0x108` bit 0), with one
/// interrupt source that is active without any bus traffic.
pub struct Wrapped {
    pub name: &'static str,
    /// The masked interrupt status register (INTR_STAT / ISR).
    pub status: usize,
    /// The interrupt mask register (INTR_MASK / IMR).
    pub mask: usize,
    /// The source: TX FIFO empty.
    pub source: u32,
    pub wrap: usize,
    pub entries: &'static Entries,
    pub fail: &'static str,
}

/// `IRQ_WRAP`: with the wrapper closed the unmasked source does not reach the NVIC; opened
/// through the HAL's `enable_interrupt` (`open`), it enters the handler once. The handler masks
/// the source again.
pub fn wrap_test<const IRQ: u8>(
    w: &Wrapped,
    line: &mut Line<IRQ>,
    open: &mut dyn FnMut(),
    idle: &mut dyn FnMut(),
) {
    w.entries.reset();
    line.unpend();
    let pre = line.is_pending();
    // Raw write: the HAL never unmasks a source without opening the wrapper.
    wr(w.mask, w.source);
    settle(100, idle);
    // The controller itself reports the interrupt, so only the wrapper holds it back.
    let off_stat = rd(w.status) & w.source != 0;
    let off_pending = line.is_pending();
    line.enable();
    open();
    wait(|| w.entries.count() != 0, 10_000, idle);
    // A second entry would mean that masking in the handler did not quiesce the line.
    settle(1000, idle);
    let (on_n, ipsr) = (w.entries.count(), w.entries.ipsr());
    line.disable();
    wr(w.mask, 0);
    wr(w.wrap, rd(w.wrap) & !1);
    settle(100, idle);
    line.unpend();
    let post = line.is_pending();
    mark!(
        "IRQ_WRAP {} pre_pending={} off_stat={} off_pending={} on_n={} ipsr={} post_pending={}",
        w.name,
        pre as u32,
        off_stat as u32,
        off_pending as u32,
        on_n,
        ipsr,
        post as u32
    );
    let ok = !pre && off_stat && !off_pending && on_n == 1 && ipsr == 16 + u32::from(IRQ) && !post;
    expect(ok, w.fail);
}

/// Stack and masking limits for one core's final figures.
pub const MIN_STACK_FREE: u32 = 512;
pub const MAX_MASKED_US: u32 = 100;

/// The nonce where Linux can read it (the lab request word of lab/shm-firmware, which this image
/// does not serve): a checker ties the Linux console to this boot's UART0 lines with it.
pub fn publish_nonce() {
    wr(rp1_proto::LINK_START as usize + 0x28, nonce());
    rp1_proto::barrier();
}
