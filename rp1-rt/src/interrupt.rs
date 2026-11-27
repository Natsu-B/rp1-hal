//! Core-local interrupt handlers, NVIC lines, and critical sections.
//!
//! `#[interrupt(proc0)]` and `#[interrupt(proc1)]` bind a PAC interrupt name to a core.
//! Handlers run from ISRAM by default; `sram` keeps the handler in shared SRAM, including
//! when using automatic ISRAM placement. `PCIE_OUT`, `PROC_MISC`, and `CLOCKS_DEFAULT`
//! are reserved for system services. A handler needs a declared line on that core.
//! Each core can bind a declared line once.
//!
//! ```
//! use core::cell::Cell;
//! use rp1_rt::interrupt::{Mutex, free};
//! static TICKS: Mutex<Cell<u32>> = Mutex::new(Cell::new(0));
//!
//! # mod rp1 { pub mod __irq_proc1 { pub trait Handler {} #[allow(non_camel_case_types)] pub enum TIMER_1 {} } }
//! # fn main() {}
//! #[rp1_rt::interrupt(proc1)]
//! fn TIMER_1() {
//!     free(|cs| TICKS.borrow(cs).set(TICKS.borrow(cs).get() + 1));
//!     // Clear the device interrupt source before returning.
//! }
//! ```
//!
//! Ordinary statics, including `TICKS`, have one DSRAM copy per core.
//! The default `critical-section-single-core` feature uses BASEPRI to mask user priorities
//! 2–7 on this core. It restores the previous mask when leaving a section, so sections nest.
//! System interrupts at priorities 0 and 1, NMI and fault handlers must never access data
//! protected by a user `Mutex`. Both cores can enter simultaneously. Use [`crate::sync`] for
//! cross-core data because `Mutex` cannot be placed in `#[shared]`.
//!
//! Enable `critical-section-primask` to mask all configurable exceptions during `free`.
//! This delays platform IRQs and escalates faults in the section to HardFault, so the platform
//! latency and fault continuation guarantees do not apply. Panic containment is still
//! possible with a panic handler that clears PRIMASK before parking.
//! Disable default features when supplying your own implementation. On bare-metal ARM,
//! both built-in implementations and custom implementations use a `u8` restore state.
//!
//! Keep user critical sections below 100 microseconds. Handlers receive no core token and
//! cannot access `#[proc0]`/`#[proc1]` locals. Share state with
//! main through a [`Slot`], ordinary statics with a `Mutex`, or DSRAM atomics. `log!` in a handler does not
//! wait for the log lock: it drops the message when the log is busy, and otherwise sends it
//! synchronously, polling the UART for each byte.

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

mod mask;
#[doc(hidden)]
pub use mask::__primask_free;
#[cfg(test)]
pub(crate) use mask::test_model;
#[cfg(all(target_arch = "arm", target_os = "none"))]
#[doc(hidden)]
pub use mask::{__basepri_acquire, __basepri_release};

/// Critical section for user contexts on this core; see the module docs.
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
/// the default handler masks it and records [`unhandled`]. The line stays on its owning core.
pub struct Line<const IRQ: u8> {
    _core: core::marker::PhantomData<*mut ()>,
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
        Self {
            _core: core::marker::PhantomData,
        }
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

/// A verified NVIC line assigned to a declaration's source and core.
/// The source's device operations remain on the source token.
pub struct Binding<S, const IRQ: u8, const CORE: u8> {
    line: Line<IRQ>,
    _source: core::marker::PhantomData<S>,
}

impl<S, const IRQ: u8, const CORE: u8> Binding<S, IRQ, CORE> {
    /// # Safety
    /// Generated resources only, once on CORE for the verified route from S to IRQ.
    #[doc(hidden)]
    pub const unsafe fn new() -> Self {
        Self {
            line: unsafe { Line::new() },
            _source: core::marker::PhantomData,
        }
    }
}
impl<S, const IRQ: u8, const CORE: u8> core::ops::Deref for Binding<S, IRQ, CORE> {
    type Target = Line<IRQ>;

    fn deref(&self) -> &Self::Target {
        &self.line
    }
}
impl<S, const IRQ: u8, const CORE: u8> core::ops::DerefMut for Binding<S, IRQ, CORE> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.line
    }
}

/// A timer comparison feeding the corresponding TIMER IRQ.
pub enum AlarmIrq<const N: u8> {}
/// An instance's verified interrupt output.
pub enum BlockIrq<const ID: u16> {}
/// The bank-0 line shared by the core's GPIO edges/levels and doorbell force source.
pub enum Bank0Irq {}

mod slot;
pub use slot::Slot;

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

/// Run a thread-mode wait with events on transitions to pending, even for disabled lines.
/// Used for a temporary startup alarm. System and user IRQ delivery are unchanged.
#[doc(hidden)]
pub fn __with_events_on_pending<T>(wait: impl FnOnce() -> T) -> T {
    #[cfg(all(target_arch = "arm", target_os = "none"))]
    let _restore = {
        const SCR: *mut u32 = 0xe000_ed10 as *mut u32;
        const SEVONPEND: u32 = 1 << 4;
        struct Restore(u32);
        impl Drop for Restore {
            fn drop(&mut self) {
                unsafe { SCR.write_volatile(self.0) };
            }
        }
        unsafe {
            let previous = SCR.read_volatile();
            SCR.write_volatile(previous | SEVONPEND);
            core::arch::asm!("dsb sy", "isb sy", options(nostack, preserves_flags));
            Restore(previous)
        }
    };
    wait()
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
