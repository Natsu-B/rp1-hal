//! Core-local interrupt diagnostics. Ordinary statics have a separate DSRAM copy per core.

#[cfg(target_arch = "arm")]
use core::sync::atomic::{AtomicU32, Ordering};

#[cfg(target_arch = "arm")]
static LAST: AtomicU32 = AtomicU32::new(0);
#[cfg(target_arch = "arm")]
static COUNT: AtomicU32 = AtomicU32::new(0);

// Protect the two-word snapshot against a preempting IRQ. This is not a cross-core lock.
#[cfg(target_arch = "arm")]
fn masked<T>(f: impl FnOnce() -> T) -> T {
    let primask: u32;
    unsafe {
        core::arch::asm!("mrs {}, PRIMASK", "cpsid i", out(reg) primask, options(nostack, preserves_flags));
    }
    let result = f();
    if primask & 1 == 0 {
        unsafe { core::arch::asm!("cpsie i", options(nostack, preserves_flags)) };
    }
    result
}

/// Last unhandled IRQ number and total unhandled IRQ count on this core.
///
/// Returns `None` before the first unhandled IRQ (and on a host). The count saturates.
pub fn unhandled() -> Option<(u8, u32)> {
    #[cfg(target_arch = "arm")]
    return masked(|| {
        let count = COUNT.load(Ordering::Relaxed);
        (count != 0).then(|| (LAST.load(Ordering::Relaxed) as u8, count))
    });
    #[cfg(not(target_arch = "arm"))]
    None
}

/// Mask an unhandled external interrupt and return so this core can continue.
#[cfg(target_arch = "arm")]
#[doc(hidden)]
#[crate::cold]
#[unsafe(no_mangle)]
unsafe extern "C" fn __rp1_default_irq() {
    let ipsr: u32;
    unsafe {
        core::arch::asm!("mrs {}, IPSR", out(reg) ipsr, options(nomem, nostack, preserves_flags))
    };
    if let Some(irq) = ipsr.checked_sub(16).filter(|irq| *irq < 64) {
        unsafe {
            (0xe000_e180 as *mut u32)
                .add(irq as usize / 32)
                .write_volatile(1 << (irq % 32));
            core::arch::asm!("dsb sy", options(nostack, preserves_flags));
        }
        masked(|| {
            LAST.store(irq, Ordering::Relaxed);
            COUNT.store(
                COUNT.load(Ordering::Relaxed).saturating_add(1),
                Ordering::Relaxed,
            );
        });
    }
}
