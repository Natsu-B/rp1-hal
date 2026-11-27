//! Register and link-page words of RP1. Host unit tests replace them with a fake register file
//! that records every write and critical section, so the notifiers' sequences can be checked.

#[cfg(not(test))]
#[inline(always)]
pub(crate) fn read(address: usize) -> u32 {
    unsafe { (address as *const u32).read_volatile() }
}

/// One store, completed before the next access (barrier).
#[cfg(not(test))]
#[inline(always)]
pub(crate) fn write(address: usize, value: u32) {
    unsafe { (address as *mut u32).write_volatile(value) };
    rp1_proto::barrier();
}

/// Run `f` with this core's user interrupts masked.
#[cfg(not(test))]
#[inline(always)]
pub(crate) fn section<R>(f: impl FnOnce() -> R) -> R {
    rp1_rt::interrupt::free(|_| f())
}

#[cfg(test)]
pub(crate) use fake::{read, section, write};

#[cfg(test)]
pub(crate) mod fake {
    extern crate std;
    use core::cell::RefCell;
    use std::{collections::BTreeMap, vec::Vec};

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub(crate) enum Access {
        Read(usize),
        Write(usize, u32),
        Enter,
        Leave,
    }

    std::thread_local! {
        static WORDS: RefCell<BTreeMap<usize, u32>> = const { RefCell::new(BTreeMap::new()) };
        // Only the v3 tests assert readback order. An alias mirrors writes into its bank.
        static ALIASES: RefCell<BTreeMap<usize, (usize, bool)>> = const { RefCell::new(BTreeMap::new()) };
        static TRACE_READS: core::cell::Cell<bool> = const { core::cell::Cell::new(false) };
        static LOG: RefCell<Vec<Access>> = const { RefCell::new(Vec::new()) };
    }

    pub(crate) fn read(address: usize) -> u32 {
        TRACE_READS.with(|trace| {
            if trace.get() {
                LOG.with(|log| log.borrow_mut().push(Access::Read(address)));
            }
        });
        WORDS.with(|w| w.borrow().get(&address).copied().unwrap_or(0))
    }
    pub(crate) fn write(address: usize, value: u32) {
        LOG.with(|l| l.borrow_mut().push(Access::Write(address, value)));
        ALIASES.with(|aliases| {
            if let Some(&(target, set)) = aliases.borrow().get(&address) {
                WORDS.with(|words| {
                    let mut words = words.borrow_mut();
                    let word = words.entry(target).or_default();
                    *word = if set { *word | value } else { *word & !value };
                });
            }
        });
    }
    pub(crate) fn section<R>(f: impl FnOnce() -> R) -> R {
        LOG.with(|l| l.borrow_mut().push(Access::Enter));
        let r = f();
        LOG.with(|l| l.borrow_mut().push(Access::Leave));
        r
    }
    /// Make `read(address)` return `value`.
    pub(crate) fn set(address: usize, value: u32) {
        WORDS.with(|w| w.borrow_mut().insert(address, value));
    }
    pub(crate) fn trace_reads() {
        TRACE_READS.with(|trace| trace.set(true));
    }

    pub(crate) fn alias(address: usize, target: usize, set: bool) {
        ALIASES.with(|aliases| aliases.borrow_mut().insert(address, (target, set)));
    }

    /// The accesses since the last call.
    pub(crate) fn take() -> Vec<Access> {
        LOG.with(|l| l.take())
    }
}
