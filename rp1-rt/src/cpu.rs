use core::arch::asm;

#[inline(always)]
pub(crate) fn basepri() -> u8 {
    let value: u32;
    unsafe { asm!("mrs {}, BASEPRI", out(reg) value, options(nostack, preserves_flags)) };
    value as u8
}

#[inline(always)]
pub(crate) fn set_basepri(mask: u8) {
    unsafe { asm!("msr BASEPRI, {}", in(reg) u32::from(mask), options(nostack, preserves_flags)) };
}

#[inline(always)]
pub(crate) fn set_basepri_max(mask: u8) {
    unsafe {
        asm!("msr BASEPRI_MAX, {}", in(reg) u32::from(mask), options(nostack, preserves_flags))
    };
}

#[inline(always)]
pub(crate) fn primask_disable() -> u8 {
    let saved: u32;
    unsafe {
        asm!("mrs {}, PRIMASK", "cpsid i", out(reg) saved, options(nostack, preserves_flags))
    };
    saved as u8 & 1
}

#[inline(always)]
pub(crate) fn set_primask(saved: u8) {
    unsafe { asm!("msr PRIMASK, {}", in(reg) u32::from(saved), options(nostack, preserves_flags)) };
}

#[inline(always)]
pub(crate) fn ipsr() -> u32 {
    let value;
    unsafe { asm!("mrs {}, IPSR", out(reg) value, options(nostack, preserves_flags)) };
    value
}

#[inline(always)]
pub(crate) fn sync() {
    unsafe { asm!("dsb sy", "isb sy", options(nostack, preserves_flags)) };
}

#[cfg(feature = "default-panic-handler")]
#[inline(always)]
pub(crate) fn enable_interrupts() {
    unsafe { asm!("cpsie i", options(nostack, preserves_flags)) };
}

#[cfg(feature = "default-panic-handler")]
#[inline(always)]
pub(crate) fn wfi() {
    unsafe { asm!("wfi", options(nomem, nostack, preserves_flags)) };
}
