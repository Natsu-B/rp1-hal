//! Lab-only wrappers for the live proc0 platform vectors.
use super::{log, now, rp1};
use core::{
    arch::asm,
    sync::atomic::{AtomicBool, AtomicU32, Ordering::Relaxed},
};

const PLATFORM: [u32; 2] = [53, 57];
const ISER: *const u32 = 0xe000_e100 as *const u32;
const IPR: *const u8 = 0xe000_e400 as *const u8;
// An SCMI burst can span a whole flush and contain many separate entries.
const CAPACITY: usize = 64;
pub(super) const TRIALS: usize = 1_000;
const TRIGGER: *mut u32 = (rp1_proto::LINK_START + 0x28) as *mut u32;
const START: u32 = u32::from_le_bytes(*b"SPIT");

struct Trace {
    total: AtomicU32,
    worst: AtomicU32,
    count: AtomicU32,
    timestamps: [[AtomicU32; 2]; CAPACITY],
}

impl Trace {
    const fn new() -> Self {
        Self {
            total: AtomicU32::new(0),
            worst: AtomicU32::new(0),
            count: AtomicU32::new(0),
            timestamps: [const { [AtomicU32::new(0), AtomicU32::new(0)] }; CAPACITY],
        }
    }
}

static ACTIVE: AtomicBool = AtomicBool::new(false);
static TRACES: [Trace; 2] = [Trace::new(), Trace::new()];
static SCMI_LAST_EXIT: AtomicU32 = AtomicU32::new(0);

unsafe extern "C" {
    fn __rp1_p0_PCIE_OUT();
    fn __rp1_p0_PROC_MISC();
}

#[rp1_rt::isram]
extern "C" fn pcie() {
    stamp(0, || unsafe { __rp1_p0_PCIE_OUT() });
}

#[rp1_rt::isram]
extern "C" fn scmi() {
    stamp(1, || unsafe { __rp1_p0_PROC_MISC() });
}

#[rp1_rt::isram]
extern "C" fn scmi_with_exit() {
    SCMI_LAST_EXIT.store(stamp(1, || unsafe { __rp1_p0_PROC_MISC() }), Relaxed);
}

#[inline(always)]
fn stamp(line: usize, handler: impl FnOnce()) -> u32 {
    let entered = now();
    handler();
    let exited = now();
    let trace = &TRACES[line];
    let elapsed = exited.wrapping_sub(entered);
    trace.total.store(trace.total.load(Relaxed) + 1, Relaxed);
    trace
        .worst
        .store(trace.worst.load(Relaxed).max(elapsed), Relaxed);
    // Each line has its own writer, so IRQ53 can nest inside IRQ57.
    if ACTIVE.load(Relaxed) {
        let slot = trace.count.load(Relaxed) as usize;
        if let Some([start, end]) = trace.timestamps.get(slot) {
            start.store(entered, Relaxed);
            end.store(exited, Relaxed);
        }
        trace.count.store(slot as u32 + 1, Relaxed);
    }
    exited
}

pub(super) fn install() {
    install_vectors(scmi);
}

#[allow(dead_code)]
pub(super) fn install_with_scmi_exit() {
    install_vectors(scmi_with_exit);
}

fn install_vectors(scmi: extern "C" fn()) {
    // Platform init has checked and installed the original vectors. Only the
    // lab modifies the writable table, with proc0 interrupts briefly masked.
    rp1_rt::interrupt::__primask_free(|| unsafe {
        let vectors = (0xe000_ed08 as *const u32).read_volatile() as *mut usize;
        let replacements = [pcie as *const () as usize, scmi as *const () as usize];
        let originals = [
            __rp1_p0_PCIE_OUT as *const () as usize,
            __rp1_p0_PROC_MISC as *const () as usize,
        ];
        for (i, irq) in PLATFORM.into_iter().enumerate() {
            let vector = vectors.add(16 + irq as usize);
            assert_eq!(vector.read_volatile(), originals[i]);
            vector.write_volatile(replacements[i]);
        }
        TRIGGER.write_volatile(0);
        asm!("dsb", "isb", options(nostack, preserves_flags));
    });
}

pub(super) fn inventory(devices: &[u32], alarms: &[u32]) {
    let enabled =
        unsafe { ISER.read_volatile() as u64 | (ISER.add(1).read_volatile() as u64) << 32 };
    let irq_mask = |irq| 1u64 << irq;
    let allowed = PLATFORM
        .into_iter()
        .chain(devices.iter().copied())
        .chain(alarms.iter().copied())
        .map(irq_mask)
        .fold(0, |a, b| a | b);
    assert_eq!(enabled & !allowed, 0, "unstamped enabled IRQs");
    use rp1_rt::interrupt::priority;
    for (irq, priority) in [(53u32, priority::PCIE_RESET), (57, priority::SCMI)]
        .into_iter()
        .chain(
            devices
                .iter()
                .chain(alarms)
                .map(|&irq| (irq, priority::DEVICE)),
        )
    {
        let raw = unsafe { IPR.add(irq as usize).read_volatile() };
        assert_eq!(raw, priority << 5);
        log!(
            "ATTR_IRQ irq={irq} priority={priority} enabled={}\n",
            (enabled & irq_mask(irq) != 0) as u8
        )
        .unwrap();
    }
}

pub(super) fn wait_for_linux() {
    wait_for_host("ATTR_AWAIT_LINUX", START);
}

pub(super) fn host_requested(value: u32) -> bool {
    unsafe { TRIGGER.read_volatile() == value }
}

#[allow(dead_code)]
pub(super) fn publish_host_marker(value: u32) {
    unsafe { TRIGGER.write_volatile(value) };
    rp1_proto::barrier();
}

pub(super) fn wait_for_host(marker: &str, value: u32) {
    log!("{marker}\n").unwrap();
    // The root console writes this only after Linux reaches multi-user.
    while !host_requested(value) {
        core::hint::spin_loop();
    }
}

#[allow(dead_code)]
pub(super) fn scmi_entries() -> u32 {
    TRACES[1].total.load(Relaxed)
}

#[allow(dead_code)]
pub(super) fn scmi_last_exit() -> u32 {
    SCMI_LAST_EXIT.load(Relaxed)
}

pub(super) fn begin() -> u32 {
    for trace in &TRACES {
        trace.count.store(0, Relaxed);
    }
    ACTIVE.store(true, Relaxed);
    now()
}

pub(super) fn end() -> u32 {
    let returned = now();
    ACTIVE.store(false, Relaxed);
    returned
}

struct Span {
    irq: u32,
    entered: u32,
    exited: u32,
}

#[derive(Default)]
pub(super) struct Summary {
    pub(super) preempted: usize,
    pub(super) failed: usize,
    pub(super) max_plain: u32,
    pub(super) max_preempted: u32,
}

pub(super) struct Flush {
    pub(super) start: u32,
    pub(super) returned: u32,
    pub(super) last_busy: u32,
    pub(super) first_idle: u32,
    pub(super) tail_alarms: u32,
}

pub(super) fn assess(totals: &mut Summary, hz: u32, trial: usize, flush: Flush) -> u8 {
    let Flush {
        start,
        returned,
        last_busy,
        first_idle,
        tail_alarms,
    } = flush;
    let bracket = last_busy != 0
        && first_idle.wrapping_sub(last_busy) <= 6
        && (returned.wrapping_sub(last_busy) as i32) >= 0;
    let upper = returned.wrapping_sub(last_busy).wrapping_add(1);
    let mut spans = [const {
        Span {
            irq: 0,
            entered: 0,
            exited: 0,
        }
    }; CAPACITY * 2];
    let mut len = 0;
    let mut overflow = false;
    let counts = TRACES
        .each_ref()
        .map(|trace| trace.count.load(Relaxed) as usize);
    let duration = returned.wrapping_sub(start);
    for ((irq, trace), count) in PLATFORM.into_iter().zip(&TRACES).zip(counts) {
        overflow |= count > CAPACITY;
        for [entered, exited] in trace.timestamps.iter().take(count) {
            let entered = entered.load(Relaxed);
            let exited = exited.load(Relaxed);
            if entered.wrapping_sub(start) < duration {
                spans[len] = Span {
                    irq,
                    entered,
                    exited,
                };
                len += 1;
            }
        }
    }
    let spans = &mut spans[..len];
    spans.sort_unstable_by_key(|span| span.entered.wrapping_sub(start));

    // Only time overlapping the conservative idle-to-return bracket is
    // creditable. A nested IRQ must not be counted twice.
    let mut credited = 0;
    let mut through = last_busy.wrapping_sub(start);
    for span in spans.iter() {
        let end = span.exited.wrapping_sub(start).min(duration);
        let begin = span.entered.wrapping_sub(start).max(through);
        credited += end.saturating_sub(begin);
        through = through.max(end);
    }
    let preempted = credited != 0;
    if preempted {
        totals.preempted += 1;
        totals.max_preempted = totals.max_preempted.max(upper);
    } else {
        totals.max_plain = totals.max_plain.max(upper);
    }
    let pass = bracket && !overflow && upper <= 3 + credited;
    totals.failed += usize::from(!pass);
    if !spans.is_empty() || !pass {
        log!(
            "ATTR_SAMPLE hz={hz} trial={trial} start={start} busy={last_busy} idle={first_idle} returned={returned} upper={upper} credit={credited} tail_alarms={tail_alarms} irq53_count={} irq57_count={} overflow={} pass={}\n",
            counts[0], counts[1], u8::from(overflow), u8::from(pass),
        )
        .unwrap();
        for span in spans {
            log!(
                "ATTR_ISR hz={hz} trial={trial} irq={} entered={} exited={}\n",
                span.irq,
                span.entered,
                span.exited
            )
            .unwrap();
        }
    }
    upper.min(255) as u8
}

pub(super) fn platform_totals() {
    for (irq, trace) in PLATFORM.into_iter().zip(&TRACES) {
        log!(
            "ATTR_PLATFORM irq={irq} entries={} worst_us={}\n",
            trace.total.load(Relaxed),
            trace.worst.load(Relaxed)
        )
        .unwrap();
    }
}
