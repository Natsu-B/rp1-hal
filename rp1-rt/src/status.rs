//! This firmware's status words on the link page ([`rp1_proto::status`]), for Linux.
//!
//! The startup code begins a new `EPOCH` and marks each core running before its `main`. A core
//! then ends in one terminal state via [`stop`], [`record_panic`] or [`record_fault`]. The
//! details (reason, line, file) are written before the state, and a terminal state is never
//! overwritten, so Linux can trust the details of the first terminal state it reads.
//! The default panic handler calls [`record_panic`] first, then parks the context. A panic before the
//! startup code marked the core running is not recorded.
use core::sync::atomic::{AtomicU32, Ordering::Relaxed};
use rp1_proto::{
    barrier,
    status::{self as s, CoreState},
};

#[cfg(all(
    feature = "default-panic-handler",
    any(test, all(target_arch = "arm", target_os = "none"))
))]
mod panic_handler;

/// Compile-time check used by generated v3 resources.
#[cfg(feature = "experimental-link-wake")]
#[doc(hidden)]
pub const __REQUIRES_EXPERIMENTAL_LINK_WAKE: () = ();

// Only an opted-in firmware owns the mailbox. The v2 layout can place rings here.
#[cfg(any(test, feature = "experimental-link-wake"))]
const INIT_END: usize = rp1_proto::wake::mailbox::END;
#[cfg(not(any(test, feature = "experimental-link-wake")))]
const INIT_END: usize = s::END;

/// This core's number (0 = proc0). `.bss` has one copy per core.
static CORE: AtomicU32 = AtomicU32::new(0);

// Plain word loads and stores: no exclusive access touches shared SRAM.
#[cfg(all(target_arch = "arm", target_os = "none"))]
fn word(offset: usize) -> &'static AtomicU32 {
    unsafe { &*((rp1_proto::LINK_START as usize + offset) as *const AtomicU32) }
}
#[cfg(not(all(target_arch = "arm", target_os = "none")))]
fn word(offset: usize) -> &'static AtomicU32 {
    static PAGE: [AtomicU32; INIT_END / 4] = [const { AtomicU32::new(0) }; INIT_END / 4];
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
    // Clear the v3 mailbox before the boot owner can publish its offered caps and BOOT_EPOCH.
    for offset in (s::ABI + 4..INIT_END).step_by(4) {
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
    finish(
        CORE.load(Relaxed),
        CoreState::Stopped,
        code,
        0,
        0,
        clear_file,
    );
    loop {
        crate::wait_for_event();
    }
}
/// Publish a deliberate stop before the vector owner delivers the terminal wake and parks.
#[cfg(feature = "experimental-link-wake")]
#[doc(hidden)]
pub fn __record_stop_before_park(code: u32) {
    finish(
        CORE.load(Relaxed),
        CoreState::Stopped,
        code,
        0,
        0,
        clear_file,
    );
}

/// An immutable terminal record read from SRAM after its state is committed.
#[doc(hidden)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Terminal {
    pub epoch: u32,
    pub state: CoreState,
    pub reason: u32,
}

/// Scan before waiting for events, including when the vector owner starts late.
#[doc(hidden)]
pub fn terminal(core: u8) -> Option<Terminal> {
    let epoch = word(s::EPOCH).load(Relaxed);
    if epoch == 0 || epoch & 1 != 0 || core > 1 {
        return None;
    }
    barrier();
    let state = CoreState::from_word(field(u32::from(core), s::STATE).load(Relaxed));
    if !state.is_terminal() {
        return None;
    }
    barrier();
    let reason = field(u32::from(core), s::REASON).load(Relaxed);
    barrier();
    (word(s::EPOCH).load(Relaxed) == epoch).then_some(Terminal {
        epoch,
        state,
        reason,
    })
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
    {
        let _ = info;
        finish(CORE.load(Relaxed), CoreState::Panicked, 0, 0, 0, clear_file);
    }
    #[cfg(feature = "panic-location")]
    record_panic_details(file, line);
}

#[cfg(any(test, feature = "panic-location"))]
fn record_panic_details(file: &str, line: u32) {
    finish(
        CORE.load(Relaxed),
        CoreState::Panicked,
        0,
        line,
        0,
        |core| write_file(core, file),
    );
}

#[cfg(any(test, feature = "panic-location"))]
#[inline]
fn write_file(core: u32, file: &str) {
    for (i, word) in s::file_words(file).into_iter().enumerate() {
        field(core, s::FILE + 4 * i).store(word, Relaxed);
    }
}

/// Report a fault: its exception number, stacked PC and CFSR. For a fault handler.
pub fn record_fault(exception: u32, pc: u32, cfsr: u32) {
    finish(
        CORE.load(Relaxed),
        CoreState::Fault,
        exception,
        pc,
        cfsr,
        clear_file,
    );
}

#[inline(always)]
fn clear_file(core: u32) {
    // Write the fixed fields directly without a temporary file buffer on the fault stack.
    const { assert!(s::FILE_LEN == 20) };
    field(core, s::FILE).store(0, Relaxed);
    field(core, s::FILE + 4).store(0, Relaxed);
    field(core, s::FILE + 8).store(0, Relaxed);
    field(core, s::FILE + 12).store(0, Relaxed);
    field(core, s::FILE + 16).store(0, Relaxed);
}
#[inline(always)]
fn finish(
    core: u32,
    state: CoreState,
    reason: u32,
    line: u32,
    aux: u32,
    write_file: impl FnOnce(u32),
) {
    // A priority-1 fault can preempt a user critical section while STATE is still Running.
    crate::interrupt::__primask_free(|| {
        if field(core, s::STATE).load(Relaxed) != CoreState::Running.word() {
            return;
        }
        #[cfg(test)]
        crate::interrupt::test_model::interrupt_point(); // Model preemption after checking STATE.
        field(core, s::REASON).store(reason, Relaxed);
        field(core, s::LINE).store(line, Relaxed);
        field(core, s::AUX).store(aux, Relaxed);
        write_file(core);
        barrier();
        field(core, s::STATE).store(state.word(), Relaxed);
        barrier();
        #[cfg(any(test, feature = "experimental-link-wake"))]
        terminal_event(core);
    });
}

#[cfg(any(test, feature = "experimental-link-wake"))]
#[inline(always)]
fn terminal_event(core: u32) {
    // Complete publication in SRAM before waking the surviving owner. Fault handlers
    // do not touch PCIe, and a fault in the owner itself may prevent delivery.
    #[cfg(test)]
    tests::observe_terminal_event(core);
    let _ = core;
    crate::send_event();
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate std;
    use std::{sync::Mutex, thread, time::Duration};

    /// The host page is one static: tests that use it run one at a time.
    pub(super) static PAGE_LOCK: Mutex<()> = Mutex::new(());

    std::thread_local! {
        static TERMINAL_EVENT: core::cell::Cell<Option<Terminal>> = const { core::cell::Cell::new(None) };
    }

    pub(super) fn observe_terminal_event(core: u32) {
        TERMINAL_EVENT.with(|event| event.set(terminal(core as u8)));
    }

    #[test]
    fn event_observes_committed_details_and_only_first_terminal() {
        let _page = PAGE_LOCK.lock().unwrap();
        start(0);
        start(1);
        let epoch = word(s::EPOCH).load(Relaxed);
        assert_eq!(terminal(1), None);
        finish(1, CoreState::Stopped, 42, 0, 0, clear_file);
        let record = Terminal {
            epoch,
            state: CoreState::Stopped,
            reason: 42,
        };
        TERMINAL_EVENT.with(|e| assert_eq!(e.take(), Some(record)));
        finish(1, CoreState::Fault, 6, 0x2000_0401, 1, clear_file);
        TERMINAL_EVENT.with(|e| assert_eq!(e.take(), None));
        assert_eq!(terminal(1), Some(record));
        start(0);
        assert_eq!(terminal(1), None);
    }

    pub(super) fn read(core: u32, offset: usize) -> u32 {
        field(core, offset).load(Relaxed)
    }
    pub(super) fn file(core: u32) -> std::string::String {
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
    fn proc0_clears_the_experimental_mailbox_before_startup() {
        use rp1_proto::wake::mailbox;

        let _page = PAGE_LOCK.lock().unwrap();
        for offset in (mailbox::START..mailbox::END).step_by(4) {
            word(offset).store(u32::MAX, Relaxed);
        }
        word(mailbox::SEQ).store(1, Relaxed);
        word(mailbox::ACK).store(1, Relaxed);
        start(0);
        for offset in (mailbox::START..mailbox::END).step_by(4) {
            assert_eq!(word(offset).load(Relaxed), 0, "offset {offset:#x}");
        }
    }

    #[test]
    fn terminal_state_keeps_its_details() {
        let _page = PAGE_LOCK.lock().unwrap();
        start(0);
        start(1);
        finish(1, CoreState::Panicked, 0, 42, 0, |core| {
            write_file(core, "lab/irq-proof/src/proc1.rs")
        });
        assert_eq!(read(1, s::STATE), CoreState::Panicked.word());
        assert_eq!((read(1, s::REASON), read(1, s::LINE)), (0, 42));
        assert_eq!(file(1), "q-proof/src/proc1.rs");
        // A later stop, panic or fault on the same core changes nothing.
        finish(1, CoreState::Stopped, 5, 0, 0, |core| write_file(core, ""));
        finish(1, CoreState::Fault, 3, 0x2000_0401, 0x100, |core| {
            write_file(core, "")
        });
        assert_eq!(read(1, s::STATE), CoreState::Panicked.word());
        assert_eq!(
            (read(1, s::LINE), file(1).as_str()),
            (42, "q-proof/src/proc1.rs")
        );
        // The other core is independent.
        finish(0, CoreState::Fault, 3, 0x2000_0401, 0x100, |core| {
            write_file(core, "")
        });
        let fault = [s::STATE, s::REASON, s::LINE, s::AUX].map(|o| read(0, o));
        assert_eq!(fault, [CoreState::Fault.word(), 3, 0x2000_0401, 0x100]);
        assert_eq!(file(0), "");
    }

    #[test]
    fn finish_defers_fault_preemption_until_first_details_are_published() {
        use crate::interrupt::test_model::preempt_at_next_point;
        use core::sync::atomic::AtomicBool;

        static FAULT_RAN: AtomicBool = AtomicBool::new(false);

        fn check_first_record() {
            let words = [s::STATE, s::REASON, s::LINE, s::AUX].map(|o| read(0, o));
            assert_eq!(words, [CoreState::Panicked.word(), 5, 42, 7]);
            assert_eq!(file(0), "first.rs");
        }

        let _page = PAGE_LOCK.lock().unwrap();
        start(0);
        critical_section::with(|_| {
            preempt_at_next_point(|| {
                check_first_record();
                record_fault(6, 0x2000_0401, 0x100);
                FAULT_RAN.store(true, Relaxed);
            });
            finish(0, CoreState::Panicked, 5, 42, 7, |core| {
                write_file(core, "first.rs")
            });
            assert!(FAULT_RAN.load(Relaxed));
            check_first_record();
        });
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
