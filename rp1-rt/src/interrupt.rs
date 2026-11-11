//! Core-local interrupt handlers, NVIC lines, and critical sections.
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
//! Ordinary statics, including `TICKS`, have one DSRAM copy per core.
//! The default `critical-section-single-core` feature uses PRIMASK and restores the prior
//! state, so nested sections work. This is **not a cross-core lock**: both cores can enter
//! simultaneously. `Mutex` is not `SharedSafe` and cannot be placed in `#[shared]`.
//! Cross-core communication needs the shared-memory protocols in [`crate::sync`].
//! NMI and fault handlers must not access data protected by this critical section.
//! Disable the default feature when supplying another critical-section implementation.
//!
//! Keep critical sections below 100 microseconds: PRIMASK also delays system IRQs. Handlers
//! receive no core token and cannot access `#[proc0]`/`#[proc1]` locals. Share state with
//! main through ordinary statics and a `Mutex` or DSRAM atomics. `log!` in a handler never
//! waits: it drops the message when the log is busy.

#[cfg(all(target_arch = "arm", target_os = "none"))]
use core::cell::Cell;

#[cfg(all(target_arch = "arm", target_os = "none"))]
static UNHANDLED: Mutex<Cell<(u8, u32)>> = Mutex::new(Cell::new((0, 0)));

/// Last unhandled IRQ number and total unhandled IRQ count on this core.
///
/// Returns `None` before the first unhandled IRQ (and on a host). The count saturates.
pub fn unhandled() -> Option<(u8, u32)> {
    #[cfg(all(target_arch = "arm", target_os = "none"))]
    {
        let (irq, count) = free(|cs| UNHANDLED.borrow(cs).get());
        (count != 0).then_some((irq, count))
    }
    #[cfg(not(all(target_arch = "arm", target_os = "none")))]
    None
}

/// Mask an unhandled external interrupt and return so this core can continue.
#[cfg(all(target_arch = "arm", target_os = "none"))]
#[doc(hidden)]
#[crate::cold]
#[unsafe(no_mangle)]
unsafe extern "C" fn __rp1_default_irq() {
    let ipsr: u32;
    unsafe {
        core::arch::asm!("mrs {}, IPSR", out(reg) ipsr, options(nomem, nostack, preserves_flags))
    };
    // Only external vector slots dispatch here, so IPSR is in 16..80.
    let irq = ipsr - 16;
    Nvic(Hardware).disable(irq as u8);
    // ICER must take effect before exception return to prevent immediate re-entry.
    unsafe { core::arch::asm!("dsb sy", options(nostack, preserves_flags)) };

    free(|cs| {
        let record = UNHANDLED.borrow(cs);
        let (_, count) = record.get();
        record.set((irq as u8, count.saturating_add(1)));
    });
}

/// PRIMASK critical section for this core only; see the module docs.
pub use critical_section::{CriticalSection, Mutex, with as free};

/// User interrupt priority, from 2 (highest user priority) through 7 (lowest).
/// Priorities 0 and 1 are reserved for system services. The startup default is 4.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Priority(u8);

impl Priority {
    /// Construct a user priority. Reserved or out-of-range values return `None`.
    pub const fn new(priority: u8) -> Option<Self> {
        if matches!(priority, 2..=7) {
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
/// the default handler masks it and records [`unhandled`]. `Line` is `Send` so it can
/// sit in a core-local [`Mutex`]; it is neither `SharedSafe` nor `Message`.
pub struct Line<const IRQ: u8> {
    _private: (),
}

impl<const IRQ: u8> Line<IRQ> {
    /// Construct the line token for generated resources.
    ///
    /// # Safety
    /// On the RP1 bare-metal target, construct exactly once on the core that owns this IRQ
    /// in the declaration.
    /// Keep it on that core. System IRQs may only be constructed by system services.
    pub const unsafe fn new() -> Self {
        const { assert!(IRQ < 64, "NVIC IRQ must be in 0..64") };
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
    #[cfg(all(target_arch = "arm", target_os = "none"))]
    {
        let ipsr: u32;
        unsafe {
            core::arch::asm!("mrs {}, IPSR", out(reg) ipsr, options(nomem, nostack, preserves_flags))
        };
        ipsr != 0
    }
    #[cfg(not(all(target_arch = "arm", target_os = "none")))]
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
        unsafe { (address as *const u32).read_volatile() }
    }
    fn write(&self, address: usize, value: u32) {
        unsafe { (address as *mut u32).write_volatile(value) }
    }
    fn write_byte(&self, address: usize, value: u8) {
        unsafe { (address as *mut u8).write_volatile(value) }
    }
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
pub(crate) fn reset_all_lines() {
    Nvic(Hardware).reset();
}

struct Nvic<R>(R);
impl<R: Registers> Nvic<R> {
    #[cfg(any(test, all(target_arch = "arm", target_os = "none")))]
    fn reset(&self) {
        for offset in [0, 4] {
            self.0.write(ICER + offset, u32::MAX);
            self.0.write(ICPR + offset, u32::MAX);
        }
        for irq in 0..64 {
            self.set_priority(irq, Priority::default());
        }
    }

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

    // SAFETY: cpsid i masks all configurable exceptions on this core; release restores
    // the prior PRIMASK state, preserving nesting. NMI and faults must not access protected
    // data. Statics are per-core DSRAM and Mutex is not SharedSafe, so the other core
    // cannot share the protected value through safe code.
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
        assert_eq!(Priority::default().value(), 4);
        assert!(!in_handler());
        assert_eq!(unhandled(), None);
        fn send<T: Send>() {}
        send::<Line<0>>();
    }

    #[test]
    fn reset_disables_and_clears_all_lines_before_setting_priorities() {
        let nvic = Nvic(Fake::default());
        nvic.reset();
        let writes = nvic.0.writes.take();
        let mut expected = std::vec![
            (ICER, u32::MAX, 4),
            (ICPR, u32::MAX, 4),
            (ICER + 4, u32::MAX, 4),
            (ICPR + 4, u32::MAX, 4),
        ];
        expected.extend((0..64).map(|irq| (IPR + irq, 4 << 5, 1)));
        assert_eq!(writes, expected);
    }
}
