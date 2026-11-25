use super::basepri::{__basepri_acquire, __basepri_release};

struct Basepri;
critical_section::set_impl!(Basepri);

// SAFETY: BASEPRI_MAX masks all user IRQs on this core without weakening an existing mask.
// Release restores it. System and fault handlers do not use protected user values.
unsafe impl critical_section::Impl for Basepri {
    unsafe fn acquire() -> u8 {
        unsafe { __basepri_acquire() }
    }

    unsafe fn release(saved: u8) {
        unsafe { __basepri_release(saved) };
    }
}
