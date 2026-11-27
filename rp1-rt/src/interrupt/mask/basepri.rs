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

pub(crate) struct SlotMask(u8);

impl SlotMask {
    #[inline(always)]
    pub(crate) fn raise<const CEILING: u8>() -> Self {
        let saved = basepri();
        set_basepri_max(CEILING << (8 - crate::pac::NVIC_PRIO_BITS));
        #[cfg(all(target_arch = "arm", target_os = "none"))]
        crate::cpu::sync();
        #[cfg(all(target_arch = "arm", target_os = "none", feature = "slot-mask-timing"))]
        if saved == 0 {
            time_slot_mask(true);
        }
        Self(saved)
    }
}

impl Drop for SlotMask {
    #[inline(always)]
    fn drop(&mut self) {
        #[cfg(all(target_arch = "arm", target_os = "none", feature = "slot-mask-timing"))]
        if self.0 == 0 {
            time_slot_mask(false);
        }
        set_basepri(self.0);
    }
}

#[cfg(all(target_arch = "arm", target_os = "none", feature = "slot-mask-timing"))]
fn time_slot_mask(entering: bool) {
    unsafe extern "C" {
        fn __rp1_slot_mask_timing(entering: bool);
    }
    unsafe { __rp1_slot_mask_timing(entering) };
}

// Default for other images when a workspace build enables the lab feature.
#[cfg(all(target_arch = "arm", target_os = "none", feature = "slot-mask-timing"))]
#[unsafe(no_mangle)]
extern "C" fn __rp1_no_slot_mask_timing(_: bool) {}

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
    fn slot_uses_its_ceiling_and_restores_mask_even_on_unwind() {
        use crate::interrupt::Slot;
        use std::panic::{AssertUnwindSafe, catch_unwind};

        let uart = Slot::<_, 0, 5>::new(());
        let motor = Slot::<_, 0, 2>::new(());
        for initial in (0..8).map(|priority| priority << 5) {
            set_basepri(initial);
            uart.try_with(|_| {
                let expected = if initial == 0 {
                    5 << 5
                } else {
                    initial.min(5 << 5)
                };
                assert_eq!(basepri(), expected);
                // Priority 2 can preempt a priority-5 borrow when it was previously enabled.
                assert_eq!(basepri() <= 2 << 5, (1..=2).contains(&(initial >> 5)));
                motor.try_with(|_| assert_eq!(basepri(), expected.min(2 << 5)));
                assert_eq!(basepri(), expected);
            });
            assert_eq!(basepri(), initial);
        }
        set_basepri(0);
        assert!(catch_unwind(AssertUnwindSafe(|| uart.try_with(|_| panic!("borrow")))).is_err());
        assert_eq!(basepri(), 0);
        assert_eq!(uart.try_with(|_| ()), Some(()));
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
