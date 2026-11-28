use core::{
    cell::UnsafeCell,
    sync::atomic::{AtomicBool, Ordering},
};

/// A value shared between thread mode and handlers on one core.
///
/// A borrow masks this core's interrupts at `CEILING` and lower urgency. The ceiling
/// defaults to the startup priority (4) and can be 2–7. Set each IRQ that uses the slot to
/// a numeric priority at least as large as `CEILING` before enabling it, including after
/// any change to its priority. A more urgent level IRQ that tries to borrow a busy slot
/// may keep firing without being acknowledged.
/// Keep the closure bounded so other handlers at the masked priorities can run.
pub struct Slot<T, const CORE: u8, const CEILING: u8 = 4> {
    value: UnsafeCell<T>,
    busy: AtomicBool,
}

// SAFETY: Only the owning core can borrow on RP1 and preemptions are nested. On a host
// the atomic lock serializes threads. Send allows the value to pass between contexts.
unsafe impl<T: Send, const CORE: u8, const CEILING: u8> Sync for Slot<T, CORE, CEILING> {}

struct Borrow<'a> {
    busy: &'a AtomicBool,
    #[cfg(any(test, all(target_arch = "arm", target_os = "none")))]
    _mask: super::mask::SlotMask,
}

impl Drop for Borrow<'_> {
    fn drop(&mut self) {
        self.busy.store(false, Ordering::Release);
    }
}

impl<T, const CORE: u8, const CEILING: u8> Slot<T, CORE, CEILING> {
    pub const fn new(value: T) -> Self {
        const {
            assert!(CORE < 2, "Slot core must be 0 or 1");
            assert!(
                CEILING >= 2 && CEILING <= 7,
                "Slot ceiling must be in 2..=7"
            );
        }
        Self {
            value: UnsafeCell::new(value),
            busy: AtomicBool::new(false),
        }
    }

    /// Run a closure with exclusive access. The borrow ends before this method returns.
    ///
    /// `None` means a nested borrow (or contention on a host), or a call from the other
    /// core. Neither occurs in ordinary use on RP1. Wrong-core calls also trigger a debug
    /// assertion. Host calls use core 0.
    pub fn try_with<R>(&self, f: impl FnOnce(&mut T) -> R) -> Option<R> {
        let _borrow = try_borrow::<CORE, CEILING>(&self.busy)?;
        Some(f(unsafe { &mut *self.value.get() }))
    }

    /// Share the admission code across several small closures on this slot.
    #[doc(hidden)]
    #[inline(always)]
    pub fn __try_with_compact<R>(&self, f: impl FnOnce(&mut T) -> R) -> Option<R> {
        let _borrow = try_borrow_compact::<CORE, CEILING>(&self.busy)?;
        Some(f(unsafe { &mut *self.value.get() }))
    }
}

#[inline(never)]
fn try_borrow_compact<const CORE: u8, const CEILING: u8>(busy: &AtomicBool) -> Option<Borrow<'_>> {
    try_borrow::<CORE, CEILING>(busy)
}

#[inline(always)]
fn try_borrow<const CORE: u8, const CEILING: u8>(busy: &AtomicBool) -> Option<Borrow<'_>> {
    #[cfg(all(target_arch = "arm", target_os = "none"))]
    let core = (unsafe { crate::pac::core_control::CORE_ID.read_volatile() } & 1) as u8;
    #[cfg(not(all(target_arch = "arm", target_os = "none")))]
    let core = 0;

    if core != CORE {
        debug_assert_eq!(core, CORE, "Slot accessed from the wrong core");
        return None;
    }

    #[cfg(any(test, all(target_arch = "arm", target_os = "none")))]
    let _mask = super::mask::SlotMask::raise::<CEILING>();

    #[cfg(all(target_arch = "arm", target_os = "none"))]
    {
        if busy.load(Ordering::Acquire) {
            return None;
        }
        // A more urgent interrupt between the load and store runs to completion
        // first. Publish busy before touching the value so it cannot borrow it.
        busy.store(true, Ordering::Relaxed);
        core::sync::atomic::compiler_fence(Ordering::SeqCst);
    }
    #[cfg(not(all(target_arch = "arm", target_os = "none")))]
    if busy
        .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
        .is_err()
    {
        return None;
    }

    Some(Borrow {
        busy,
        #[cfg(any(test, all(target_arch = "arm", target_os = "none")))]
        _mask,
    })
}

impl<T, const CORE: u8, const CEILING: u8> Slot<Option<T>, CORE, CEILING> {
    /// Create an empty slot.
    pub const fn empty() -> Self {
        Self::new(None)
    }

    /// Install a value in the empty slot. Returns it if occupied or inaccessible.
    pub fn put(&self, value: T) -> Result<(), T> {
        let mut value = Some(value);
        self.try_with(|slot| {
            if slot.is_none() {
                *slot = value.take();
            }
        });
        value.map_or(Ok(()), Err)
    }

    /// Remove the installed value. Returns `None` if empty or inaccessible.
    pub fn take(&self) -> Option<T> {
        self.try_with(Option::take).flatten()
    }

    /// Borrow the installed value. Returns `None` if empty or inaccessible.
    pub fn with<R>(&self, f: impl FnOnce(&mut T) -> R) -> Option<R> {
        self.try_with(|slot| slot.as_mut().map(f)).flatten()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_slot_keeps_its_value_until_taken() {
        let slot = Slot::<Option<u32>, 0>::empty();
        assert_eq!(slot.with(|n| *n), None);
        assert_eq!(slot.put(1), Ok(()));
        assert_eq!(slot.put(2), Err(2));
        assert_eq!(
            slot.with(|n| {
                assert_eq!(slot.take(), None);
                assert_eq!(slot.put(3), Err(3));
                *n += 1;
            }),
            Some(())
        );
        assert_eq!(slot.take(), Some(2));
        assert_eq!(slot.take(), None);
    }

    #[test]
    #[cfg_attr(
        debug_assertions,
        should_panic(expected = "Slot accessed from the wrong core")
    )]
    fn wrong_core_is_detected() {
        assert_eq!(Slot::<_, 1>::new(0).try_with(|n| *n), None);
    }
}
