//! This firmware's status words on the link page ([`rp1_proto::status`]), for Linux.
//!
//! The startup code begins a new `EPOCH` and marks each core running before its `main`. A core
//! then ends in one terminal state: [`stop`], or a panic handler's [`record_panic`]. The
//! details (reason, line, file) are written before the state, and a terminal state is never
//! overwritten, so Linux can trust the details of the first terminal state it reads.
//! A `#[panic_handler]` calls [`record_panic`] first, then parks the core. A panic before the
//! startup code marked the core running is not recorded.
use core::sync::atomic::{AtomicU32, Ordering::Relaxed};
use rp1_proto::{
    barrier,
    status::{self as s, CoreState},
};

/// This core's number (0 = proc0). `.bss` has one copy per core.
static CORE: AtomicU32 = AtomicU32::new(0);

// Plain word loads and stores: no exclusive access touches shared SRAM.
#[cfg(all(target_arch = "arm", target_os = "none"))]
fn word(offset: usize) -> &'static AtomicU32 {
    unsafe { &*((rp1_proto::LINK_START as usize + offset) as *const AtomicU32) }
}
#[cfg(not(all(target_arch = "arm", target_os = "none")))]
fn word(offset: usize) -> &'static AtomicU32 {
    static PAGE: [AtomicU32; s::END / 4] = [const { AtomicU32::new(0) }; s::END / 4];
    &PAGE[offset / 4]
}
fn field(core: u32, offset: usize) -> &'static AtomicU32 {
    word(block(core) + offset)
}
/// The status block of `core`; any core other than 0 is proc1 (the startup code stores only 0
/// or 1). Without an index, the panic handler's path through here cannot panic again, which
/// keeps it free of recursion for the static stack bound (`stack-paths`).
fn block(core: u32) -> usize {
    let [proc0, proc1] = s::BLOCKS;
    if core == 0 { proc0 } else { proc1 }
}

/// Startup, before `main` and with interrupts masked. proc0 runs it before it can launch proc1.
#[cfg(any(test, all(target_arch = "arm", target_os = "none")))]
pub(crate) fn start(core: u32) {
    if core == 0 {
        init_epoch();
    }
    CORE.store(core, Relaxed);
    field(core, s::STATE).store(CoreState::Running.word(), Relaxed);
    barrier();
}

#[cfg(any(test, all(target_arch = "arm", target_os = "none")))]
fn init_epoch() {
    let old = word(s::EPOCH).load(Relaxed);
    // An odd value is a start that did not finish: continue above it, never repeat a value.
    let epoch = if word(s::ABI).load(Relaxed) == s::ABI_VALUE {
        old & !1
    } else {
        0
    };
    word(s::EPOCH).store(epoch.wrapping_add(1), Relaxed);
    barrier();
    for offset in (s::ABI + 4..s::END).step_by(4) {
        word(offset).store(0, Relaxed);
    }
    word(s::ABI).store(s::ABI_VALUE, Relaxed);
    barrier();
    word(s::EPOCH).store(epoch.wrapping_add(2), Relaxed);
}

/// Liveness mark for Linux's `health` check: stores the TIMER µs (low 32 bits). Call it at
/// least once per declared `heartbeat.period_ms`, from main or a handler.
pub fn beat() {
    beat_with(CORE.load(Relaxed), timer_us);
}
// The read and the store share one critical section: otherwise a handler's beat between them
// would be overwritten by an older time.
fn beat_with(core: u32, now: impl FnOnce() -> u32) {
    critical_section::with(|_| field(core, s::BEAT).store(now(), Relaxed));
}
fn timer_us() -> u32 {
    #[cfg(all(target_arch = "arm", target_os = "none"))]
    unsafe {
        // TIMER.TIMERAWL
        (0x400a_c028 as *const u32).read_volatile()
    }
    #[cfg(not(all(target_arch = "arm", target_os = "none")))]
    0
}

/// Report a deliberate stop with `code`, then park this core in `wait_for_event`.
pub fn stop(code: u32) -> ! {
    finish(CORE.load(Relaxed), CoreState::Stopped, code, 0, 0, "");
    loop {
        crate::wait_for_event();
    }
}
/// Report a panic. For a panic handler; it does not park the core.
///
/// With the `panic-location` feature it also records the file and line. That costs 2.5..4.7 KiB
/// of image in this workspace's firmware: reading `info` keeps every call site's
/// `fmt::Arguments` and the `Debug` code of every unwrapped error, which LTO otherwise removes.
pub fn record_panic(info: &core::panic::PanicInfo<'_>) {
    #[cfg(feature = "panic-location")]
    let (file, line) = info.location().map_or(("", 0), |l| (l.file(), l.line()));
    #[cfg(not(feature = "panic-location"))]
    let (file, line, _) = ("", 0, info);
    finish(CORE.load(Relaxed), CoreState::Panicked, 0, line, 0, file);
}
/// Report a fault: its exception number, stacked PC and CFSR. For a fault handler.
pub fn record_fault(exception: u32, pc: u32, cfsr: u32) {
    finish(
        CORE.load(Relaxed),
        CoreState::Fault,
        exception,
        pc,
        cfsr,
        "",
    );
}

fn finish(core: u32, state: CoreState, reason: u32, line: u32, aux: u32, file: &str) {
    critical_section::with(|_| {
        if field(core, s::STATE).load(Relaxed) != CoreState::Running.word() {
            return;
        }
        field(core, s::REASON).store(reason, Relaxed);
        field(core, s::LINE).store(line, Relaxed);
        field(core, s::AUX).store(aux, Relaxed);
        for (i, w) in s::file_words(file).into_iter().enumerate() {
            field(core, s::FILE + 4 * i).store(w, Relaxed);
        }
        barrier();
        field(core, s::STATE).store(state.word(), Relaxed);
        barrier();
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate std;
    use std::{sync::Mutex, thread, time::Duration};

    /// The host page is one static: tests that use it run one at a time.
    static PAGE_LOCK: Mutex<()> = Mutex::new(());

    fn read(core: u32, offset: usize) -> u32 {
        field(core, offset).load(Relaxed)
    }
    fn file(core: u32) -> std::string::String {
        let words = core::array::from_fn(|i| read(core, s::FILE + 4 * i));
        let mut bytes = [0; s::FILE_LEN];
        let n = s::file_bytes(&words, &mut bytes);
        std::string::String::from_utf8(bytes[..n].to_vec()).unwrap()
    }

    #[test]
    fn epoch_counts_boots_and_clears_the_blocks() {
        let _page = PAGE_LOCK.lock().unwrap();
        word(s::ABI).store(0, Relaxed);
        word(s::EPOCH).store(7, Relaxed);
        field(1, s::LINE).store(99, Relaxed);
        start(0);
        assert_eq!(word(s::EPOCH).load(Relaxed), 2);
        assert_eq!(word(s::ABI).load(Relaxed), s::ABI_VALUE);
        assert_eq!(read(1, s::LINE), 0);
        assert_eq!(read(0, s::STATE), CoreState::Running.word());
        start(0);
        assert_eq!(word(s::EPOCH).load(Relaxed), 4);
        // A start that stopped half way (odd) is not counted again.
        word(s::EPOCH).store(5, Relaxed);
        start(0);
        assert_eq!(word(s::EPOCH).load(Relaxed), 6);
    }

    #[test]
    fn terminal_state_keeps_its_details() {
        let _page = PAGE_LOCK.lock().unwrap();
        start(0);
        start(1);
        finish(
            1,
            CoreState::Panicked,
            0,
            42,
            0,
            "lab/irq-proof/src/proc1.rs",
        );
        assert_eq!(read(1, s::STATE), CoreState::Panicked.word());
        assert_eq!((read(1, s::REASON), read(1, s::LINE)), (0, 42));
        assert_eq!(file(1), "q-proof/src/proc1.rs");
        // A later stop, panic or fault on the same core changes nothing.
        finish(1, CoreState::Stopped, 5, 0, 0, "");
        finish(1, CoreState::Fault, 3, 0x2000_0401, 0x100, "");
        assert_eq!(read(1, s::STATE), CoreState::Panicked.word());
        assert_eq!(
            (read(1, s::LINE), file(1).as_str()),
            (42, "q-proof/src/proc1.rs")
        );
        // The other core is independent.
        finish(0, CoreState::Fault, 3, 0x2000_0401, 0x100, "");
        let fault = [s::STATE, s::REASON, s::LINE, s::AUX].map(|o| read(0, o));
        assert_eq!(fault, [CoreState::Fault.word(), 3, 0x2000_0401, 0x100]);
        assert_eq!(file(0), "");
    }

    #[test]
    fn any_core_number_has_a_block() {
        // The panic path must not panic on a corrupted core number.
        assert_eq!(block(0), s::BLOCKS[0]);
        for core in [1, 2, u32::MAX] {
            assert_eq!(block(core), s::BLOCKS[1]);
        }
    }

    #[test]
    fn beat_never_goes_back() {
        // A beat from a "handler" (another thread) arrives between this beat's timer read and
        // its store; the critical section holds it back until the store, so the newer time stays.
        let _page = PAGE_LOCK.lock().unwrap();
        start(0);
        thread::scope(|scope| {
            let mut handler = None;
            beat_with(0, || {
                handler = Some(scope.spawn(|| beat_with(0, || 200)));
                thread::sleep(Duration::from_millis(50));
                100
            });
            handler.unwrap().join().unwrap();
        });
        assert_eq!(read(0, s::BEAT), 200);
    }
}
