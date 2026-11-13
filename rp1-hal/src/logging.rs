use crate::{Error, poll, write};
use core::{
    fmt::{self, Write},
    sync::atomic::{AtomicBool, AtomicU32, Ordering, compiler_fence},
};

// Ordinary statics are per core (DSRAM, where RMW atomics work), so both are this core's own.
/// Set while this core's code holds or is taking the lock. A handler that interrupted it must
/// not wait: the holder cannot continue until the handler returns.
static IN_LOG: AtomicBool = AtomicBool::new(false);
static DROPPED: AtomicU32 = AtomicU32::new(0);

/// Messages this core's interrupt handlers dropped because the log was busy.
pub fn dropped() -> u32 {
    DROPPED.load(Ordering::Relaxed)
}

/// Single-writer-per-word Peterson lock: all accesses compile to loads/stores, never RMW.
#[repr(C)]
pub struct Logger {
    want: [AtomicU32; 2],
    turn: AtomicU32,
    ready: AtomicU32,
}
unsafe impl rp1_proto::SharedSafe for Logger {}
impl Logger {
    /// # Safety
    /// Only the generated `rp1::log` module may construct a `Logger`: it exists exactly when the
    /// declaration gives UART0 and GPIO14/15 to the log (`log = { sink = "uart0" }`), and that
    /// declaration cannot also hand UART0 or those pins to a driver. One instance per image, in
    /// `.shared`; a second `Logger` would bypass both the ownership and the Peterson lock
    /// (review #6).
    pub const unsafe fn new() -> Self {
        Self {
            want: [const { AtomicU32::new(0) }; 2],
            turn: AtomicU32::new(0),
            ready: AtomicU32::new(0),
        }
    }
    /// Called by proc0 before launching proc1. Owns UART0 and pins 14/15 for the image lifetime.
    pub fn init(&'static self, _core: &mut rp1_rt::Proc0, baud: u32) -> Result<(), Error> {
        if self.ready.load(Ordering::Acquire) != 0 {
            return Err(Error::Precondition);
        }
        // Divisors come from the clk_uart rate read back from the registers.
        let uart_hz = crate::clocks::uart_hz().ok_or(Error::Precondition)?;
        write(0x4001_7004, 1 << 19);
        for pin in [14, 15] {
            write(0x400d_0004 + pin * 8, 4);
            write(0x400f_0004 + pin * 4, 0x46);
        }
        crate::uart::start(0, uart_hz, baud)?;
        self.ready.store(1, Ordering::Release);
        Ok(())
    }
    /// Writes one message to UART0 without masking interrupts. Main code waits for the other
    /// core's message to finish. A handler never waits: if this core or the other one is
    /// already writing, the message is dropped, counted in [`dropped`], and `Ok` is returned.
    /// Interrupts stay enabled, so the other core may wait while a holder is preempted, and a
    /// handler that gets the lock transmits inside the handler.
    pub fn write(&self, args: fmt::Arguments<'_>) -> fmt::Result {
        if self.ready.load(Ordering::Acquire) == 0 {
            return Err(fmt::Error);
        }
        #[cfg(target_arch = "arm")]
        let core = (crate::read(0xe00f_f01c) & 1) as usize;
        #[cfg(not(target_arch = "arm"))]
        let core = 0usize;
        if !self.lock_or_count_drop(core, rp1_rt::interrupt::in_handler()) {
            return Ok(());
        }
        struct Sink;
        impl Write for Sink {
            fn write_str(&mut self, s: &str) -> fmt::Result {
                for b in s.bytes() {
                    poll(0x4003_0018, 1 << 5, 0).map_err(|_| fmt::Error)?;
                    write(0x4003_0000, u32::from(b));
                }
                Ok(())
            }
        }
        let result = Sink.write_fmt(args);
        self.unlock(core);
        result
    }
    fn lock_or_count_drop(&self, core: usize, in_handler: bool) -> bool {
        let taken = self.lock(core, in_handler);
        if !taken {
            DROPPED.fetch_add(1, Ordering::Relaxed);
        }
        taken
    }
    /// Takes the lock for `core`; a handler tries once. `false`: not taken.
    fn lock(&self, core: usize, in_handler: bool) -> bool {
        if in_handler && IN_LOG.load(Ordering::Acquire) {
            return false;
        }
        IN_LOG.store(true, Ordering::Release);
        // A handler between this store and the `want` store must already see IN_LOG, or its
        // unlock would clear our `want`.
        compiler_fence(Ordering::SeqCst);
        let other = 1 - core;
        self.want[core].store(1, Ordering::Release);
        self.turn.store(other as u32, Ordering::Release);
        rp1_proto::barrier();
        while self.want[other].load(Ordering::Acquire) != 0
            && self.turn.load(Ordering::Acquire) == other as u32
        {
            if in_handler {
                self.unlock(core);
                return false;
            }
            core::hint::spin_loop();
        }
        true
    }
    fn unlock(&self, core: usize) {
        self.want[core].store(0, Ordering::Release);
        IN_LOG.store(false, Ordering::Release);
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;

    // On a host IN_LOG is one static shared by all tests, so they take turns.
    static IN_LOG_USERS: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    fn handlers_never_wait_and_main_code_waits_for_the_other_core() {
        let _turn = IN_LOG_USERS.lock().unwrap();
        let log = unsafe { Logger::new() };
        let wants = |core: usize| log.want[core].load(Ordering::Acquire) != 0;
        assert!(log.lock(0, true));
        log.unlock(0);

        // A handler that interrupted this core's message leaves the lock alone.
        assert!(log.lock(0, false));
        assert!(!log.lock(0, true));
        assert!(wants(0) && IN_LOG.load(Ordering::Acquire));
        log.unlock(0);

        // The other core is writing: a handler backs out, main code waits until it is done.
        log.want[1].store(1, Ordering::Release);
        log.turn.store(0, Ordering::Release);
        assert!(!log.lock(0, true));
        assert!(!wants(0) && !IN_LOG.load(Ordering::Acquire));
        std::thread::scope(|s| {
            let main = s.spawn(|| log.lock(0, false));
            while !wants(0) {}
            assert!(!main.is_finished());
            log.want[1].store(0, Ordering::Release);
            assert!(main.join().unwrap());
        });
    }

    #[test]
    fn only_a_refused_handler_message_is_counted_as_dropped() {
        let _turn = IN_LOG_USERS.lock().unwrap();
        let log = unsafe { Logger::new() };
        let before = dropped();
        assert!(log.lock_or_count_drop(0, false));
        assert_eq!(dropped(), before);
        assert!(!log.lock_or_count_drop(0, true));
        assert!(!log.lock_or_count_drop(0, true));
        assert_eq!(dropped(), before + 2);
        log.unlock(0);
        assert!(log.lock_or_count_drop(0, true));
        assert_eq!(dropped(), before + 2);
        log.unlock(0);
    }

    // A model of `lock`..`unlock`, one load or store per step, so that a test can try every
    // interleaving. `the_model_matches_lock_and_unlock` ties its results to the real code; the
    // order of the stores cannot be observed on a host and must be kept equal by hand.

    /// The words the lock uses. On the device each core has its own IN_LOG.
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
    struct Words {
        in_log: [bool; 2],
        want: [bool; 2],
        turn: usize,
    }
    const IDLE: Words = Words {
        in_log: [false; 2],
        want: [false; 2],
        turn: 0,
    };

    /// The next load or store of one caller.
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
    enum Next {
        CheckInLog,
        SetInLog,
        SetWant,
        SetTurn,
        ReadWant,
        ReadTurn,
        /// The start of `unlock`. `taken`: the caller holds the lock until this store; otherwise
        /// it is a handler backing out.
        ClearWant {
            taken: bool,
        },
        ClearInLog {
            taken: bool,
        },
        Done {
            taken: bool,
        },
    }

    #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
    struct Caller {
        core: usize,
        /// 0: main code. Otherwise a handler, which preempts the lower levels of its core.
        level: u8,
        next: Next,
    }
    impl Caller {
        fn new(core: usize, level: u8) -> Self {
            let next = Next::CheckInLog;
            Self { core, level, next }
        }
        fn holds_lock(&self) -> bool {
            self.next == Next::ClearWant { taken: true }
        }
        fn is_done(&self) -> bool {
            matches!(self.next, Next::Done { .. })
        }
        fn has_started(&self) -> bool {
            self.next != Next::CheckInLog
        }
        /// Performs the next load or store. `false`: main code that keeps spinning, or done.
        fn step(&mut self, words: &mut Words) -> bool {
            let (core, other, in_handler) = (self.core, 1 - self.core, self.level > 0);
            self.next = match self.next {
                Next::CheckInLog if in_handler && words.in_log[core] => Next::Done { taken: false },
                Next::CheckInLog => Next::SetInLog,
                Next::SetInLog => {
                    words.in_log[core] = true;
                    Next::SetWant
                }
                Next::SetWant => {
                    words.want[core] = true;
                    Next::SetTurn
                }
                Next::SetTurn => {
                    words.turn = other;
                    Next::ReadWant
                }
                Next::ReadWant if words.want[other] => Next::ReadTurn,
                Next::ReadWant => Next::ClearWant { taken: true },
                Next::ReadTurn if words.turn != other => Next::ClearWant { taken: true },
                Next::ReadTurn if in_handler => Next::ClearWant { taken: false },
                // Another turn of the spin loop reads the same `turn` again unless the other
                // core's `want` has cleared, so only then is it a step.
                Next::ReadTurn if !words.want[other] => Next::ReadWant,
                Next::ReadTurn | Next::Done { .. } => return false,
                Next::ClearWant { taken } => {
                    words.want[core] = false;
                    Next::ClearInLog { taken }
                }
                Next::ClearInLog { taken } => {
                    words.in_log[core] = false;
                    Next::Done { taken }
                }
            };
            true
        }
    }

    /// Runs the real `lock`/`unlock` alone from every starting state and compares the words and
    /// the result with the model's.
    #[test]
    fn the_model_matches_lock_and_unlock() {
        let _turn = IN_LOG_USERS.lock().unwrap();
        let log = unsafe { Logger::new() };
        // The host's single IN_LOG stands for the IN_LOG of the core under test.
        let real_words = |core: usize| {
            let mut words = IDLE;
            words.in_log[core] = IN_LOG.load(Ordering::Relaxed);
            words.want = [0, 1].map(|c| log.want[c].load(Ordering::Relaxed) != 0);
            words.turn = log.turn.load(Ordering::Relaxed) as usize;
            words
        };
        for case in 0..32 {
            let bit = |n: u32| case >> n & 1 == 1;
            let (core, in_handler, in_log, other_wants) =
                (usize::from(bit(0)), bit(1), bit(2), bit(3));
            let mut words = IDLE;
            words.in_log[core] = in_log;
            words.want[1 - core] = other_wants;
            words.turn = usize::from(bit(4));
            IN_LOG.store(in_log, Ordering::Relaxed);
            log.want[core].store(0, Ordering::Relaxed);
            log.want[1 - core].store(u32::from(other_wants), Ordering::Relaxed);
            log.turn.store(words.turn as u32, Ordering::Relaxed);

            let mut model = Caller::new(core, u8::from(in_handler));
            while !model.holds_lock() && model.step(&mut words) {}
            if !model.holds_lock() && !model.is_done() {
                // The real `lock` would spin forever here; the model must have a reason to.
                assert!(!in_handler && other_wants, "case {case}");
                continue;
            }
            assert_eq!(
                log.lock(core, in_handler),
                model.holds_lock(),
                "case {case}"
            );
            assert_eq!(real_words(core), words, "case {case}");
            if model.holds_lock() {
                log.unlock(core);
                while model.step(&mut words) {}
                assert_eq!(real_words(core), words, "case {case}");
            }
        }
        IN_LOG.store(false, Ordering::Relaxed);
    }

    /// Every schedule of four callers that each log once: on core 0 main code, a handler and a
    /// handler that preempts it, and main code on core 1.
    #[test]
    fn every_interleaving_keeps_exclusion_and_handlers_never_wait() {
        let callers = [
            Caller::new(0, 0),
            Caller::new(0, 1),
            Caller::new(0, 2),
            Caller::new(1, 0),
        ];
        let mut todo = std::vec![(IDLE, callers)];
        let mut seen = std::collections::HashSet::new();
        let mut handler_results = std::collections::HashSet::new();
        while let Some(state @ (words, callers)) = todo.pop() {
            if !seen.insert(state) {
                continue;
            }
            let holders = callers.iter().filter(|c| c.holds_lock()).count();
            assert!(holders <= 1, "two callers hold the lock: {state:?}");

            let mut stuck = true;
            for (i, caller) in callers.iter().enumerate() {
                let preempted = callers.iter().any(|c| {
                    c.core == caller.core
                        && c.level > caller.level
                        && c.has_started()
                        && !c.is_done()
                });
                if preempted {
                    continue;
                }
                let (mut words, mut callers) = state;
                if callers[i].step(&mut words) {
                    todo.push((words, callers));
                    stuck = false;
                } else {
                    let spins = !caller.is_done();
                    assert!(!spins || caller.level == 0, "a handler waits: {state:?}");
                }
            }
            // No schedule loops (a caller only goes back to ReadWant after another one advanced),
            // so main code always gets the lock if no reachable state is stuck short of the end.
            if stuck {
                let released = Words {
                    turn: IDLE.turn,
                    ..words
                };
                assert_eq!(released, IDLE, "{state:?}");
                for caller in callers {
                    match caller.next {
                        Next::Done { taken } if caller.level > 0 => {
                            handler_results.insert((caller.level, taken));
                        }
                        next => assert_eq!(next, Next::Done { taken: true }, "{state:?}"),
                    }
                }
            }
        }
        // Both handlers were seen to log and to drop: the schedules are not all of one kind.
        assert_eq!(handler_results.len(), 4);
    }
}
