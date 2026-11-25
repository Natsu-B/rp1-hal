#[cfg(all(target_arch = "arm", target_os = "none", not(test)))]
use crate::cpu::{primask_disable, set_primask};
#[cfg(test)]
#[path = "mask/test_model.rs"]
pub(crate) mod test_model;
#[cfg(test)]
use test_model::{primask_disable, set_primask};

#[cfg(all(
    target_arch = "arm",
    target_os = "none",
    feature = "critical-section-primask"
))]
mod primask_section {
    use crate::cpu::{primask_disable, set_primask};

    struct Primask;
    critical_section::set_impl!(Primask);

    // SAFETY: PRIMASK excludes configurable exceptions on this core, and release restores
    // the saved bit. NMI and HardFault do not use the protected values.
    unsafe impl critical_section::Impl for Primask {
        unsafe fn acquire() -> u8 {
            primask_disable()
        }

        unsafe fn release(saved: u8) {
            set_primask(saved);
        }
    }
}

#[cfg(any(test, all(target_arch = "arm", target_os = "none")))]
#[inline]
pub fn __primask_free<T>(f: impl FnOnce() -> T) -> T {
    let saved = primask_disable();
    let value = f();
    set_primask(saved);
    value
}

#[cfg(not(any(test, all(target_arch = "arm", target_os = "none"))))]
#[inline]
pub fn __primask_free<T>(f: impl FnOnce() -> T) -> T {
    critical_section::with(|_| f())
}

#[cfg(any(test, all(target_arch = "arm", target_os = "none")))]
mod basepri;
#[cfg(all(target_arch = "arm", target_os = "none"))]
pub use basepri::{__basepri_acquire, __basepri_release};
#[cfg(all(
    target_arch = "arm",
    target_os = "none",
    feature = "critical-section-single-core",
    not(feature = "critical-section-primask")
))]
mod basepri_section;
