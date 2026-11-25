#![no_std]
//! Two cores share code; ordinary statics have a private DSRAM copy on each core.
//!
//! Startup contract: each core installs its own vector table, disables all 64 NVIC
//! lines, clears pending IRQs, sets their priority to 4, marks itself running in the link
//! page's [`status`] words (proc0 first starts a new epoch). Before entering main, each
//! core sets AIRCR.PRIGROUP to 0 (all implemented priority bits select preemption
//! priority), sets CCR.STKALIGN for 8-byte exception stack alignment, and clears
//! BASEPRI and FAULTMASK. It sets the configurable faults to priority 1 and enables them
//! before clearing PRIMASK. Peripheral IRQs remain disabled until
//! explicitly enabled. Unhandled external IRQs mask themselves and update
//! [`interrupt::unhandled`].
//!
//! # Fault containment
//!
//! STATIC / 未検証 on hardware. IRQ53 has priority 0, IRQ57 and MemManage, BusFault
//! and UsageFault have priority 1, and user IRQs have priorities 2 through 7.
//!
//! | Context or fault | Outcome |
//! | --- | --- |
//! | proc0 thread or user ISR panic | The planned default panic handler parks the context with IRQ53/57 eligible to run (HYP). |
//! | proc1 panic | The planned default panic handler records `Panicked` and parks proc1 (HYP). |
//! | proc0 thread or user ISR with a configurable fault and a valid MSP frame | Records the stacked PC and CFSR and returns to a wait loop in the original context. BASEPRI masks user IRQs before exception return, leaving IRQ53/57 eligible to run. |
//! | proc1 thread or user ISR with a configurable fault and a valid MSP frame | Records `Fault` for proc1 and parks it. proc0 can continue. |
//! | Stacking or unstacking error, INVPC, or a PSP frame | Terminal without frame access. Recording is best effort. IRQ53 may still run on proc0, while IRQ57 cannot preempt the priority-1 handler. A broken stack can lock up the core. |
//! | Imprecise BusFault taken in a containable context | Returns to the same wait loop. The saved CFSR includes IMPRECISERR. The stacked PC and parked context may differ from the cause. |
//! | Panic in IRQ57 | SCMI would stop while IRQ53 would remain eligible under the planned handler (HYP). |
//! | Fault in IRQ53/57 or while PRIMASK is set, or HardFault | Terminal on the affected core. HardFault attempts to record CFSR and the stacked PC when stack status and EXC_RETURN allow it. IRQ53/57 cannot preempt it. |
//! | Infinite loop holding PRIMASK or BASEPRI | No fault containment or watchdog recovery. Linux can detect a stalled heartbeat if configured. |
use core::{cell::UnsafeCell, marker::PhantomData};
pub use rp1_macros::{cold, entry, interrupt, isram, proc0, proc1, shared};
pub use rp1_proto::SharedSafe;
#[cfg(any(test, all(target_arch = "arm", target_os = "none")))]
mod fault;
pub mod interrupt;
#[cfg(all(target_arch = "arm", target_os = "none"))]
mod startup;
pub mod status;
pub mod sync;
pub use rp1_pac as pac;

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
