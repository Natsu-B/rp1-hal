#[cfg(not(test))]
use crate::cpu::{basepri, set_basepri, set_basepri_max};
#[cfg(test)]
use model::{basepri, set_basepri, set_basepri_max};

/// # Safety
/// Use only from a core-local critical section implementation that restores the saved mask.
#[doc(hidden)]
#[inline(always)]
pub unsafe fn __basepri_acquire() -> u8 {
    let saved = basepri();
    set_basepri_max(2 << 5);
    saved
}

/// # Safety
/// Restore the value saved by the matching acquire on this core.
#[doc(hidden)]
#[inline(always)]
pub unsafe fn __basepri_release(saved: u8) {
    set_basepri(saved);
}

#[cfg(test)]
mod model {
    extern crate std;
    use core::cell::Cell;

    std::thread_local! {
        static MASK: Cell<u8> = const { Cell::new(0) };
    }

    pub(super) fn basepri() -> u8 {
        MASK.get()
    }

    pub(super) fn set_basepri(mask: u8) {
        MASK.set(mask);
    }

    pub(super) fn set_basepri_max(mask: u8) {
        let old = basepri();
        if mask != 0 && (old == 0 || mask < old) {
            set_basepri(mask);
        }
    }

    #[test]
    fn basepri_nests_and_restores_every_mask() {
        use super::{__basepri_acquire, __basepri_release};

        for initial in (0..8).map(|priority| priority << 5) {
            set_basepri(initial);
            let masked = if initial == 0x20 { 0x20 } else { 0x40 };
            unsafe {
                let outer = __basepri_acquire();
                assert_eq!(outer, initial);
                assert_eq!(basepri(), masked);
                let inner = __basepri_acquire();
                assert_eq!(inner, masked);
                __basepri_release(inner);
                assert_eq!(basepri(), masked);
                __basepri_release(outer);
            }
            assert_eq!(basepri(), initial);
        }
    }
}
