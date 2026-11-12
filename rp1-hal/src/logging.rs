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
        if !self.lock(core, rp1_rt::interrupt::in_handler()) {
            DROPPED.fetch_add(1, Ordering::Relaxed);
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

    // One test: on a host IN_LOG is a single static shared by all tests.
    #[test]
    fn handlers_never_wait_and_main_code_waits_for_the_other_core() {
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
}
