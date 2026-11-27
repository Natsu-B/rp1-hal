//! This image's critical section: rp1-rt's BASEPRI section, plus the longest time it kept
//! user interrupts masked on this core. Every `critical_section` user in the image goes through
//! it, the libraries included. Slot raises its own ceiling and uses the timing hook below.
use core::sync::atomic::{AtomicU32, Ordering::Relaxed};
use rp1_rt::interrupt::{__basepri_acquire, __basepri_release};

// Ordinary statics: one copy per core, like the BASEPRI they describe.
static MASKED_AT: AtomicU32 = AtomicU32::new(0);
static MAX_US: AtomicU32 = AtomicU32::new(0);

/// Longest masked interval on this core so far, in microseconds (1 MHz timer).
pub fn max_us() -> u32 {
    MAX_US.load(Relaxed)
}

#[unsafe(no_mangle)]
extern "C" fn __rp1_slot_mask_timing(entering: bool) {
    let now = crate::proof::now();
    if entering {
        MASKED_AT.store(now, Relaxed);
    } else {
        let us = now.wrapping_sub(MASKED_AT.load(Relaxed));
        if us > MAX_US.load(Relaxed) {
            MAX_US.store(us, Relaxed);
        }
    }
}

struct Timed;
critical_section::set_impl!(Timed);

// SAFETY: as in rp1-rt, BASEPRI_MAX masks user interrupts on this core without weakening an
// existing mask, and release restores the saved value. Only user contexts use these statics.
unsafe impl critical_section::Impl for Timed {
    unsafe fn acquire() -> u8 {
        let saved = unsafe { __basepri_acquire() };
        if saved == 0 {
            __rp1_slot_mask_timing(true);
        }
        saved
    }
    unsafe fn release(saved: u8) {
        if saved == 0 {
            __rp1_slot_mask_timing(false);
        }
        unsafe { __basepri_release(saved) };
    }
}
