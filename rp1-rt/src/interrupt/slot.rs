use core::{
    cell::UnsafeCell,
    sync::atomic::{AtomicBool, Ordering},
};

/// A value shared between thread mode and handlers on one core. A busy or wrong-core
/// access returns `None`. The closure runs in a user critical section. Keep it bounded
/// and use it from thread mode or user handlers (priorities 2–7).
pub struct Slot<T, const CORE: u8> {
    value: UnsafeCell<T>,
    busy: AtomicBool,
}

// Only the declared core can borrow the value. Nested access sees the busy flag.
#[cfg(all(target_arch = "arm", target_os = "none"))]
unsafe impl<T, const CORE: u8> Sync for Slot<T, CORE> {}
#[cfg(not(all(target_arch = "arm", target_os = "none")))]
unsafe impl<T: Send, const CORE: u8> Sync for Slot<T, CORE> {}

impl<T, const CORE: u8> Slot<T, CORE> {
    pub const fn new(value: T) -> Self {
        const { assert!(CORE < 2) };
        Self {
            value: UnsafeCell::new(value),
            busy: AtomicBool::new(false),
        }
    }

    /// Run a closure with exclusive access. The borrow ends before this method returns.
    pub fn try_with<R>(&self, f: impl FnOnce(&mut T) -> R) -> Option<R> {
        #[cfg(all(target_arch = "arm", target_os = "none"))]
        {
            // The per-processor ID in the Cortex-M3's private peripheral bus.
            let core = unsafe { (0xe00f_f01c as *const u32).read_volatile() } & 1;
            if core != u32::from(CORE) {
                return None;
            }
            // A level source must not preempt its thread-side borrow and starve the owner.
            super::free(|_| self.with_lock(f))
        }
        #[cfg(not(all(target_arch = "arm", target_os = "none")))]
        {
            if CORE != 0 {
                return None;
            }
            self.with_lock(f)
        }
    }

    fn with_lock<R>(&self, f: impl FnOnce(&mut T) -> R) -> Option<R> {
        #[cfg(all(target_arch = "arm", target_os = "none"))]
        let acquired = {
            if self.busy.load(Ordering::Acquire) {
                return None;
            }
            self.busy.store(true, Ordering::Relaxed);
            true
        };
        #[cfg(not(all(target_arch = "arm", target_os = "none")))]
        let acquired = self
            .busy
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_ok();

        if !acquired {
            return None;
        }
        struct Release<'a>(&'a AtomicBool);
        impl Drop for Release<'_> {
            fn drop(&mut self) {
                self.0.store(false, Ordering::Release);
            }
        }
        let _release = Release(&self.busy);
        Some(f(unsafe { &mut *self.value.get() }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_owner_can_borrow_and_nested_access_is_busy() {
        let slot = Slot::<_, 0>::new(0);
        assert_eq!(
            slot.try_with(|n| {
                assert_eq!(slot.try_with(|_| ()), None);
                *n += 1;
            }),
            Some(())
        );
        assert_eq!(slot.try_with(|n| *n), Some(1));
        assert_eq!(Slot::<_, 1>::new(0).try_with(|n| *n), None);
    }
}
