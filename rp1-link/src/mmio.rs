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
        Write(usize, u32),
        Enter,
        Leave,
    }

    std::thread_local! {
        static WORDS: RefCell<BTreeMap<usize, u32>> = const { RefCell::new(BTreeMap::new()) };
        static LOG: RefCell<Vec<Access>> = const { RefCell::new(Vec::new()) };
    }

    pub(crate) fn read(address: usize) -> u32 {
        WORDS.with(|w| w.borrow().get(&address).copied().unwrap_or(0))
    }
    pub(crate) fn write(address: usize, value: u32) {
        LOG.with(|l| l.borrow_mut().push(Access::Write(address, value)));
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
    /// The accesses since the last call.
    pub(crate) fn take() -> Vec<Access> {
        LOG.with(|l| l.take())
    }
}
