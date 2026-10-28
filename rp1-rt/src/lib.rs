#![no_std]
//! Two cores share code; ordinary statics have a private DSRAM copy on each core.
use core::{cell::UnsafeCell, marker::PhantomData};
pub use rp1_macros::{cold, entry, isram, proc0, proc1, shared};
pub use rp1_proto::SharedSafe;
#[cfg(target_arch = "arm")]
mod startup;
pub mod sync;

mod sealed {
    pub trait Sealed {}
}
pub trait Core: sealed::Sealed {}
pub struct Proc0 {
    _local: PhantomData<*mut ()>,
}
pub struct Proc1 {
    _local: PhantomData<*mut ()>,
}
macro_rules! core_token {
    ($t:ty) => {
        impl sealed::Sealed for $t {}
        impl Core for $t {}
        impl $t {
            /// # Safety
            /// Runtime only: construct once on this core, in thread mode.
            pub unsafe fn new() -> Self {
                Self {
                    _local: PhantomData,
                }
            }
        }
    };
}
core_token!(Proc0);
core_token!(Proc1);

pub struct Local<T, C: Core> {
    value: UnsafeCell<T>,
    _core: PhantomData<fn(C)>,
}
unsafe impl<T, C: Core> Sync for Local<T, C> {}
impl<T, C: Core> Local<T, C> {
    pub const fn new(value: T) -> Self {
        Self {
            value: UnsafeCell::new(value),
            _core: PhantomData,
        }
    }
    pub fn get<'a>(&'a self, _core: &'a C) -> &'a T {
        unsafe { &*self.value.get() }
    }
}

pub struct Proc1Launcher {
    _private: (),
}
impl Proc1Launcher {
    /// # Safety
    /// Runtime only, once on proc0; proc1 must still be held in reset.
    pub unsafe fn new() -> Self {
        Self { _private: () }
    }
    pub fn launch(self, _core: &mut Proc0) -> Result<(), LaunchError> {
        #[cfg(target_arch = "arm")]
        unsafe {
            unsafe extern "C" {
                fn __rp1_reset1();
            }
            let ctrl = (0x4001_4000 as *const u32).read_volatile();
            let done = (0x4001_4018 as *const u32).read_volatile();
            if ctrl & (1 << 31) == 0 || done & (1 << 31) != 0 {
                return Err(LaunchError::AlreadyRunning);
            }
            (0x4015_400c as *mut u32).write_volatile(0xb007_c0de);
            (0x4015_401c as *mut u32).write_volatile(0x1000_4000);
            rp1_proto::barrier();
            (0x4015_4014 as *mut u32)
                .write_volatile((__rp1_reset1 as *const () as u32 | 1) ^ 0x4ff8_3f2d);
            rp1_proto::barrier();
            // CLR alias touches only proc1 reset, preserving unrelated reset lines.
            (0x4001_7000 as *mut u32).write_volatile(1 << 31);
            rp1_proto::barrier();
            // Bounded wait for RESET_DONE0.proc1, as in the PoC proc1 start (STATIC; HW-proven there).
            let mut released = false;
            for _ in 0..100_000 {
                if (0x4001_4018 as *const u32).read_volatile() & (1 << 31) != 0 {
                    released = true;
                    break;
                }
                core::hint::spin_loop();
            }
            core::arch::asm!("dsb sy", "sev", options(nostack, preserves_flags));
            if released {
                Ok(())
            } else {
                Err(LaunchError::ResetTimeout)
            }
        }
        #[cfg(not(target_arch = "arm"))]
        Err(LaunchError::UnsupportedHost)
    }
}
#[derive(Debug, PartialEq, Eq)]
pub enum LaunchError {
    AlreadyRunning,
    ResetTimeout,
    UnsupportedHost,
}

#[macro_export]
macro_rules! bind {
    () => {
        include!(concat!(env!("OUT_DIR"), "/rp1.rs"));
    };
}

pub fn wait_for_event() {
    #[cfg(target_arch = "arm")]
    unsafe {
        core::arch::asm!("wfe", options(nomem, nostack, preserves_flags));
    }
    #[cfg(not(target_arch = "arm"))]
    core::hint::spin_loop();
}
