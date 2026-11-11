//! Core-local interrupt diagnostics. Ordinary statics have a separate DSRAM copy per core.
//!
//! `#[interrupt(proc0)]` and `#[interrupt(proc1)]` bind a PAC interrupt name to a core.
//! Handlers run from ISRAM by default; `sram` keeps the handler in shared SRAM, including
//! when using automatic ISRAM placement. `PCIE_OUT`, `PROC_MISC`, and `CLOCKS_DEFAULT`
//! are reserved for system services. Duplicate definitions on one core fail at link time.
//!
//! ```
//! use core::cell::Cell;
//! use rp1_rt::interrupt::{Mutex, free};
//! static TICKS: Mutex<Cell<u32>> = Mutex::new(Cell::new(0));
//!
//! #[rp1_rt::interrupt(proc1)]
//! fn TIMER_1() {
//!     free(|cs| TICKS.borrow(cs).set(TICKS.borrow(cs).get() + 1));
//!     // Clear the device interrupt source before returning.
//! }
//! ```
//!
//! STATIC safety model: ordinary statics, including `TICKS`, have one DSRAM copy per core.
//! The default `critical-section-single-core` feature uses PRIMASK and restores the prior
//! state, so nested sections work. This is **not a cross-core lock**: both cores can enter
//! simultaneously. `Mutex` is not `SharedSafe` and cannot be placed in `#[shared]`.
//! Cross-core communication needs the shared-memory protocols in [`crate::sync`].
//! Disable the default feature when supplying another critical-section implementation.
//! Host tests use `critical-section/std`, never the PRIMASK provider.
//!
//! Keep critical sections below 100 microseconds: PRIMASK also delays system IRQs. Handlers
//! receive no core token and cannot access `#[proc0]`/`#[proc1]` locals. Share state with
//! main through ordinary statics and a `Mutex` or DSRAM atomics. Until ISR-safe logging is
//! available, keep `log!` calls out of handlers. IRQ resource generation and driver IRQ APIs
//! are separate follow-up work; this module does not yet make timer/device IRQ APIs safe.

#[cfg(target_arch = "arm")]
use core::sync::atomic::{AtomicU32, Ordering};

#[cfg(target_arch = "arm")]
static LAST: AtomicU32 = AtomicU32::new(0);
#[cfg(target_arch = "arm")]
static COUNT: AtomicU32 = AtomicU32::new(0);

/// Last unhandled IRQ number and total unhandled IRQ count on this core.
///
/// Returns `None` before the first unhandled IRQ (and on a host). The count saturates.
pub fn unhandled() -> Option<(u8, u32)> {
    #[cfg(target_arch = "arm")]
    return free(|_| {
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
        Nvic(Hardware).disable(irq as u8);
        unsafe { core::arch::asm!("dsb sy", options(nostack, preserves_flags)) };

        free(|_| {
            LAST.store(irq, Ordering::Relaxed);
            COUNT.store(
                COUNT.load(Ordering::Relaxed).saturating_add(1),
                Ordering::Relaxed,
            );
        });
    }
}

/// A critical section protects ordinary DSRAM statics against this core's IRQ handlers.
/// It is **not a cross-core lock**. Keep sections under 100 microseconds; PRIMASK also
/// masks system IRQs. Never share a `Mutex` between cores via raw pointers/sections.
pub use critical_section::{CriticalSection, Mutex, with as free};

/// User interrupt priority, from 2 (highest user priority) through 7 (lowest).
/// Priorities 0 and 1 are reserved for system services. The startup default is 4.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Priority(u8);

impl Priority {
    /// Construct a user priority. Reserved or out-of-range values return `None`.
    pub const fn new(priority: u8) -> Option<Self> {
        if priority >= 2 && priority <= 7 {
            Some(Self(priority))
        } else {
            None
        }
    }

    /// The logical priority before shifting into the NVIC's three implemented bits.
    pub const fn value(self) -> u8 {
        self.0
    }
}

impl Default for Priority {
    fn default() -> Self {
        Self(4)
    }
}

/// Unique ownership of one core's NVIC line.
///
/// Device interrupt masks are separate. Enabling a line without a handler is safe:
/// the default handler masks it and records [`unhandled`]. This token remains `Send`
/// so it can live in a core-local [`Mutex`]; it is neither `SharedSafe` nor `Message`.
pub struct Line<const IRQ: u8> {
    _private: (),
}

impl<const IRQ: u8> Line<IRQ> {
    /// Construct the line token for generated resources.
    ///
    /// # Safety
    /// Construct exactly once on the core that owns this IRQ in the declaration.
    /// Keep it on that core. System IRQs may only be constructed by system services.
    /// All NVIC operations require the RP1 bare-metal target.
    pub const unsafe fn new() -> Self {
        assert!(IRQ < 64, "NVIC IRQ must be in 0..64");
        Self { _private: () }
    }

    /// Enable delivery on this core. Peripheral interrupt masks are unchanged.
    pub fn enable(&mut self) {
        Nvic(Hardware).enable(IRQ);
    }

    /// Disable delivery on this core without clearing its pending state.
    pub fn disable(&mut self) {
        Nvic(Hardware).disable(IRQ);
    }

    /// Whether this core's NVIC line is enabled.
    pub fn is_enabled(&self) -> bool {
        Nvic(Hardware).is_enabled(IRQ)
    }

    /// Whether this core's NVIC line is pending, even if disabled.
    pub fn is_pending(&self) -> bool {
        Nvic(Hardware).is_pending(IRQ)
    }

    /// Set the pending bit on this core.
    pub fn pend(&mut self) {
        Nvic(Hardware).pend(IRQ);
    }

    /// Clear the pending bit on this core. The peripheral may assert it again.
    pub fn unpend(&mut self) {
        Nvic(Hardware).unpend(IRQ);
    }

    /// Set this line's user priority without changing neighboring lines.
    pub fn set_priority(&mut self, priority: Priority) {
        Nvic(Hardware).set_priority(IRQ, priority);
    }
}

/// Whether this core is executing an exception or interrupt handler.
/// Returns `false` on a host.
pub fn in_handler() -> bool {
    #[cfg(target_arch = "arm")]
    {
        let ipsr: u32;
        unsafe {
            core::arch::asm!("mrs {}, IPSR", out(reg) ipsr, options(nomem, nostack, preserves_flags))
        };
        ipsr != 0
    }
    #[cfg(not(target_arch = "arm"))]
    false
}

const ISER: usize = 0xe000_e100;
const ICER: usize = 0xe000_e180;
const ISPR: usize = 0xe000_e200;
const ICPR: usize = 0xe000_e280;
const IPR: usize = 0xe000_e400;

trait Registers {
    fn read(&self, address: usize) -> u32;
    fn write(&self, address: usize, value: u32);
    fn write_byte(&self, address: usize, value: u8);
}

struct Hardware;
impl Registers for Hardware {
    fn read(&self, address: usize) -> u32 {
        #[cfg(all(target_arch = "arm", target_os = "none"))]
        unsafe {
            (address as *const u32).read_volatile()
        }
        #[cfg(not(all(target_arch = "arm", target_os = "none")))]
        {
            let _ = address;
            panic!("NVIC requires the RP1 bare-metal target")
        }
    }
    fn write(&self, address: usize, value: u32) {
        #[cfg(all(target_arch = "arm", target_os = "none"))]
        unsafe {
            (address as *mut u32).write_volatile(value)
        }
        #[cfg(not(all(target_arch = "arm", target_os = "none")))]
        {
            let _ = (address, value);
            panic!("NVIC requires the RP1 bare-metal target")
        }
    }
    fn write_byte(&self, address: usize, value: u8) {
        #[cfg(all(target_arch = "arm", target_os = "none"))]
        unsafe {
            (address as *mut u8).write_volatile(value)
        }
        #[cfg(not(all(target_arch = "arm", target_os = "none")))]
        {
            let _ = (address, value);
            panic!("NVIC requires the RP1 bare-metal target")
        }
    }
}

struct Nvic<R>(R);
impl<R: Registers> Nvic<R> {
    fn write_bit(&self, base: usize, irq: u8) {
        self.0
            .write(base + usize::from(irq / 32) * 4, 1 << (irq % 32));
    }
    fn read_bit(&self, base: usize, irq: u8) -> bool {
        self.0.read(base + usize::from(irq / 32) * 4) & (1 << (irq % 32)) != 0
    }
    fn enable(&self, irq: u8) {
        self.write_bit(ISER, irq);
    }
    fn disable(&self, irq: u8) {
        self.write_bit(ICER, irq);
    }
    fn is_enabled(&self, irq: u8) -> bool {
        self.read_bit(ISER, irq)
    }
    fn is_pending(&self, irq: u8) -> bool {
        self.read_bit(ISPR, irq)
    }
    fn pend(&self, irq: u8) {
        self.write_bit(ISPR, irq);
    }
    fn unpend(&self, irq: u8) {
        self.write_bit(ICPR, irq);
    }
    fn set_priority(&self, irq: u8, priority: Priority) {
        self.0.write_byte(IPR + usize::from(irq), priority.0 << 5);
    }
}

#[cfg(all(
    feature = "critical-section-single-core",
    target_arch = "arm",
    target_os = "none"
))]
mod single_core {
    struct Primask;
    critical_section::set_impl!(Primask);

    // SAFETY: ordinary statics are per-core DSRAM. SharedSafe excludes Mutex, so safe
    // code cannot use this implementation to protect the same value on both cores.
    unsafe impl critical_section::Impl for Primask {
        unsafe fn acquire() -> critical_section::RawRestoreState {
            let primask: u32;
            unsafe {
                core::arch::asm!("mrs {}, PRIMASK", "cpsid i", out(reg) primask, options(nostack, preserves_flags));
            }
            primask & 1 == 0
        }
        unsafe fn release(was_enabled: critical_section::RawRestoreState) {
            if was_enabled {
                unsafe { core::arch::asm!("cpsie i", options(nostack, preserves_flags)) };
            }
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use core::cell::RefCell;
    use std::{collections::BTreeMap, vec::Vec};

    #[derive(Default)]
    struct Fake {
        words: RefCell<BTreeMap<usize, u32>>,
        writes: RefCell<Vec<(usize, u32, u8)>>,
    }
    impl Registers for Fake {
        fn read(&self, address: usize) -> u32 {
            *self.words.borrow().get(&address).unwrap_or(&0)
        }
        fn write(&self, address: usize, value: u32) {
            self.writes.borrow_mut().push((address, value, 4));
            let (read_base, set) = match address & !4 {
                ICER => (ISER, false),
                ICPR => (ISPR, false),
                base => (base, true),
            };
            let mut words = self.words.borrow_mut();
            let word = words.entry(read_base + (address & 4)).or_default();
            if set {
                *word |= value;
            } else {
                *word &= !value;
            }
        }
        fn write_byte(&self, address: usize, value: u8) {
            self.writes
                .borrow_mut()
                .push((address, u32::from(value), 1));
        }
    }

    #[test]
    fn bank_boundaries_and_priority_bits() {
        let nvic = Nvic(Fake::default());
        for irq in [0, 31, 32, 53, 57, 63] {
            let offset = usize::from(irq / 32) * 4;
            let bit = 1 << (irq % 32);
            assert!(!nvic.is_enabled(irq));
            assert!(!nvic.is_pending(irq));
            nvic.enable(irq);
            assert!(nvic.is_enabled(irq));
            nvic.pend(irq);
            assert!(nvic.is_pending(irq));
            nvic.disable(irq);
            assert!(!nvic.is_enabled(irq));
            assert!(nvic.is_pending(irq));
            nvic.unpend(irq);
            assert!(!nvic.is_pending(irq));
            let writes = nvic.0.writes.take();
            assert_eq!(
                writes,
                [
                    (ISER + offset, bit, 4),
                    (ISPR + offset, bit, 4),
                    (ICER + offset, bit, 4),
                    (ICPR + offset, bit, 4)
                ]
            );
            for priority in 2..=7 {
                nvic.set_priority(irq, Priority::new(priority).unwrap());
                assert_eq!(
                    nvic.0.writes.take(),
                    [(IPR + usize::from(irq), u32::from(priority << 5), 1)]
                );
            }
        }
    }

    #[test]
    fn user_priorities_and_host_diagnostics() {
        for value in 0..=u8::MAX {
            assert_eq!(
                Priority::new(value).map(Priority::value),
                (2..=7).contains(&value).then_some(value)
            );
        }
        assert_eq!(Priority::new(0), None);
        assert_eq!(Priority::default().value(), 4);
        assert!(!in_handler());
        assert_eq!(unhandled(), None);
        fn send<T: Send>() {}
        send::<Line<0>>();
    }

    #[test]
    fn nested_host_sections_preserve_access() {
        static VALUE: Mutex<core::cell::Cell<u32>> = Mutex::new(core::cell::Cell::new(0));
        free(|outer| {
            VALUE.borrow(outer).set(1);
            free(|inner| VALUE.borrow(inner).set(2));
            assert_eq!(VALUE.borrow(outer).get(), 2);
        });
    }
}
