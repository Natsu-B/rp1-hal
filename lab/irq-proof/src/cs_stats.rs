//! This image's critical section: rp1-rt's PRIMASK section, plus the longest time it kept
//! interrupts masked on this core. Every `critical_section` user in the image goes through
//! it, the libraries included, so the figure covers code the proof did not write.
use core::sync::atomic::{AtomicU32, Ordering::Relaxed};

// Ordinary statics: one copy per core, like the PRIMASK they describe.
static MASKED_AT: AtomicU32 = AtomicU32::new(0);
static MAX_US: AtomicU32 = AtomicU32::new(0);

/// Longest masked interval on this core so far, in microseconds (1 MHz timer).
pub fn max_us() -> u32 {
    MAX_US.load(Relaxed)
}

struct Timed;
critical_section::set_impl!(Timed);

// SAFETY: as rp1-rt's implementation: `cpsid i` masks every configurable exception on this
// core and release restores the previous state, so sections nest. The two statics are only
// written with interrupts masked.
unsafe impl critical_section::Impl for Timed {
    unsafe fn acquire() -> bool {
        let primask: u32;
        unsafe {
            core::arch::asm!("mrs {}, PRIMASK", "cpsid i", out(reg) primask, options(nostack, preserves_flags));
        }
        let was_enabled = primask & 1 == 0;
        // Only the outermost section starts the clock: a nested one does not change the mask.
        if was_enabled {
            MASKED_AT.store(crate::proof::now(), Relaxed);
        }
        was_enabled
    }
    unsafe fn release(was_enabled: bool) {
        if was_enabled {
            let us = crate::proof::now().wrapping_sub(MASKED_AT.load(Relaxed));
            if us > MAX_US.load(Relaxed) {
                MAX_US.store(us, Relaxed);
            }
            unsafe { core::arch::asm!("cpsie i", options(nostack, preserves_flags)) };
        }
    }
}
