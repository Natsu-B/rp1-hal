#[cfg(all(target_arch = "arm", target_os = "none"))]
mod primask {
    use core::arch::asm;

    #[inline]
    pub(super) fn acquire() -> u8 {
        let saved: u32;
        unsafe {
            asm!("mrs {}, PRIMASK", "cpsid i", out(reg) saved, options(nostack, preserves_flags));
        }
        saved as u8 & 1
    }

    #[inline]
    pub(super) fn release(saved: u8) {
        unsafe {
            asm!("msr PRIMASK, {}", in(reg) u32::from(saved), options(nostack, preserves_flags));
        }
    }
}

#[cfg(all(
    target_arch = "arm",
    target_os = "none",
    feature = "critical-section-primask"
))]
mod primask_section {
    struct Primask;
    critical_section::set_impl!(Primask);

    // SAFETY: PRIMASK excludes all configurable exceptions on this core, and release
    // restores the saved bit. NMI and HardFault do not use the protected values.
    unsafe impl critical_section::Impl for Primask {
        unsafe fn acquire() -> u8 {
            super::primask::acquire()
        }

        unsafe fn release(saved: u8) {
            super::primask::release(saved);
        }
    }
}

/// Run a platform section with configurable exceptions masked on this core.
#[inline]
pub fn __primask_free<T>(f: impl FnOnce() -> T) -> T {
    #[cfg(any(test, all(target_arch = "arm", target_os = "none")))]
    {
        let saved = primask::acquire();
        let value = f();
        primask::release(saved);
        value
    }
    #[cfg(not(any(test, all(target_arch = "arm", target_os = "none"))))]
    critical_section::with(|_| f())
}

#[cfg(any(
    test,
    all(
        target_arch = "arm",
        target_os = "none",
        feature = "critical-section-single-core",
        not(feature = "critical-section-primask")
    )
))]
mod basepri {
    trait Registers {
        fn read(&self) -> u8;
        fn write_max(&self, mask: u8);
        fn write(&self, mask: u8);
    }

    fn acquire(regs: &impl Registers) -> u8 {
        let saved = regs.read();
        regs.write_max(2 << 5);
        saved
    }

    fn release(regs: &impl Registers, saved: u8) {
        regs.write(saved);
    }

    #[cfg(all(
        target_arch = "arm",
        target_os = "none",
        feature = "critical-section-single-core",
        not(feature = "critical-section-primask")
    ))]
    mod hardware {
        use super::Registers;
        use core::arch::asm;

        struct Basepri;
        critical_section::set_impl!(Basepri);

        impl Registers for Basepri {
            fn read(&self) -> u8 {
                let saved: u32;
                unsafe {
                    asm!("mrs {}, BASEPRI", out(reg) saved, options(nostack, preserves_flags))
                };
                saved as u8
            }

            fn write_max(&self, mask: u8) {
                unsafe {
                    asm!("msr BASEPRI_MAX, {}", in(reg) u32::from(mask), options(nostack, preserves_flags));
                }
            }

            fn write(&self, mask: u8) {
                unsafe {
                    asm!("msr BASEPRI, {}", in(reg) u32::from(mask), options(nostack, preserves_flags));
                }
            }
        }

        // SAFETY: BASEPRI_MAX masks all user IRQs on this core without weakening an
        // existing mask. Release restores it. System and fault handlers do not use
        // protected user values, and those values are core-local.
        unsafe impl critical_section::Impl for Basepri {
            unsafe fn acquire() -> u8 {
                super::acquire(&Self)
            }

            unsafe fn release(saved: u8) {
                super::release(&Self, saved);
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use core::cell::Cell;

        struct Fake(Cell<u8>);
        impl Registers for Fake {
            fn read(&self) -> u8 {
                self.0.get()
            }
            fn write_max(&self, mask: u8) {
                let old = self.read();
                if mask != 0 && (old == 0 || mask < old) {
                    self.write(mask);
                }
            }
            fn write(&self, mask: u8) {
                self.0.set(mask);
            }
        }

        #[test]
        fn basepri_nests_and_restores_every_mask() {
            for initial in (0..8).map(|priority| priority << 5) {
                let regs = Fake(Cell::new(initial));
                let masked = if initial == 0x20 { 0x20 } else { 0x40 };
                let outer = acquire(&regs);
                assert_eq!(outer, initial);
                assert_eq!(regs.read(), masked);
                let inner = acquire(&regs);
                assert_eq!(inner, masked);
                release(&regs, inner);
                assert_eq!(regs.read(), masked);
                release(&regs, outer);
                assert_eq!(regs.read(), initial);
            }
        }
    }
}

#[cfg(test)]
use test_primask as primask;

#[cfg(test)]
pub(crate) mod test_primask {
    extern crate std;
    use core::cell::Cell;

    std::thread_local! {
        static MASK: Cell<u8> = const { Cell::new(0) };
        static NEXT: Cell<Option<fn()>> = const { Cell::new(None) };
        static PENDING: Cell<Option<fn()>> = const { Cell::new(None) };
    }

    pub(super) fn acquire() -> u8 {
        MASK.replace(1)
    }

    pub(super) fn release(saved: u8) {
        MASK.set(saved);
        if saved == 0
            && let Some(handler) = PENDING.take()
        {
            handler();
        }
    }

    pub(crate) fn preempt_at_next_point(handler: fn()) {
        NEXT.set(Some(handler));
    }

    pub(crate) fn interrupt_point() {
        if let Some(handler) = NEXT.take() {
            if MASK.get() == 0 {
                handler();
            } else {
                PENDING.set(Some(handler));
            }
        }
    }
}
