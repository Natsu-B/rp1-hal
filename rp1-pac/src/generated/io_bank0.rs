// Generated from rp1-regdb/data/peripherals/io-bank0.toml.
// Do not edit manually.
// Regenerate with: cargo run -p rp1-regdb -- generate

use crate::Reg;

#[repr(C)]
struct GPIO_STATUSArrayElement {
    gpio_status: GPIO_STATUS,
    gpio_ctrl: GPIO_CTRL,
}

/// GPIO bank 0 function select, signal overrides and per-pin interrupt routing. Bus=APB. Atomic aliases: XOR at +0x1000, SET at +0x2000, CLR at +0x3000.
/// Atomic view: Rp1Native alias windows; ownership remains with this logical peripheral.
#[repr(C)]
pub struct RegisterBlock {
    gpio_status: [GPIO_STATUSArrayElement; 28],
    _reserved0: [u8; 32],
    intr: INTR,
    proc0_inte: PROC0_INTE,
    proc0_intf: PROC0_INTF,
    proc0_ints: PROC0_INTS,
    proc1_inte: PROC1_INTE,
    proc1_intf: PROC1_INTF,
    proc1_ints: PROC1_INTS,
    pcie_inte: PCIE_INTE,
    pcie_intf: PCIE_INTF,
    pcie_ints: PCIE_INTS,
}

impl RegisterBlock {
    /// Per-pad status: signal values along the input and output paths, and the interrupt event detectors. Inactive flags read 0. Edge events stay set until cleared with GPIO_CTRL.IRQRESET; level events follow the current detector state.
    /// Hardware access: read Plain; write None.
    /// Array layout: 28 physical entries, 28 public entries, stride 0x8 bytes.
    #[inline(always)]
    pub fn gpio_status(&self, index: usize) -> &GPIO_STATUS {
        assert!(index < 28);
        &self.gpio_status[index].gpio_status
    }
    /// Per-pad control: function select, input filtering, interrupt event masks and signal overrides.
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    /// Array layout: 28 physical entries, 28 public entries, stride 0x8 bytes.
    #[inline(always)]
    pub fn gpio_ctrl(&self, index: usize) -> &GPIO_CTRL {
        assert!(index < 28);
        &self.gpio_status[index].gpio_ctrl
    }
    /// Raw interrupt state per pin (GPIO_STATUS.IRQTOPROC), before the per-destination enables. Read-only; clear latched events with the pin's GPIO_CTRL.IRQRESET.
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn intr(&self) -> &INTR {
        &self.intr
    }
    /// Per-pin interrupt enables for RP1 processor 0. 1 lets the pin's interrupt through.
    /// Hardware access: read Plain; write Plain.
    #[inline(always)]
    pub const fn proc0_inte(&self) -> &PROC0_INTE {
        &self.proc0_inte
    }
    /// Per-pin interrupt force for RP1 processor 0. 1 forces the matching status bit.
    /// Hardware access: read Plain; write Plain.
    #[inline(always)]
    pub const fn proc0_intf(&self) -> &PROC0_INTF {
        &self.proc0_intf
    }
    /// Per-pin interrupt status seen by RP1 processor 0: the enabled raw interrupt combined with the force bit.
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn proc0_ints(&self) -> &PROC0_INTS {
        &self.proc0_ints
    }
    /// Per-pin interrupt enables for RP1 processor 1. 1 lets the pin's interrupt through.
    /// Hardware access: read Plain; write Plain.
    #[inline(always)]
    pub const fn proc1_inte(&self) -> &PROC1_INTE {
        &self.proc1_inte
    }
    /// Per-pin interrupt force for RP1 processor 1. 1 forces the matching status bit.
    /// Hardware access: read Plain; write Plain.
    #[inline(always)]
    pub const fn proc1_intf(&self) -> &PROC1_INTF {
        &self.proc1_intf
    }
    /// Per-pin interrupt status seen by RP1 processor 1: the enabled raw interrupt combined with the force bit.
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn proc1_ints(&self) -> &PROC1_INTS {
        &self.proc1_ints
    }
    /// Per-pin interrupt enables for the PCIe host. 1 lets the pin's interrupt through.
    /// Hardware access: read Plain; write Plain.
    #[inline(always)]
    pub const fn pcie_inte(&self) -> &PCIE_INTE {
        &self.pcie_inte
    }
    /// Per-pin interrupt force for the PCIe host. 1 forces the matching status bit.
    /// Hardware access: read Plain; write Plain.
    #[inline(always)]
    pub const fn pcie_intf(&self) -> &PCIE_INTF {
        &self.pcie_intf
    }
    /// Per-pin interrupt status seen by the PCIe host: the enabled raw interrupt combined with the force bit. For level-triggered handling, enable MSI-X acknowledge handling so that re-asserted interrupts are not lost.
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn pcie_ints(&self) -> &PCIE_INTS {
        &self.pcie_ints
    }
}

/// Per-pad status: signal values along the input and output paths, and the interrupt event detectors. Inactive flags read 0. Edge events stay set until cleared with GPIO_CTRL.IRQRESET; level events follow the current detector state.
/// Hardware access: read Plain; write None.
/// Array layout: 28 physical entries, 28 public entries, stride 0x8 bytes.
pub type GPIO_STATUS = Reg<gpio_status::GPIO_STATUS_SPEC>;
/// Per-pad status: signal values along the input and output paths, and the interrupt event detectors. Inactive flags read 0. Edge events stay set until cleared with GPIO_CTRL.IRQRESET; level events follow the current detector state.
/// Hardware access: read Plain; write None.
/// Array layout: 28 physical entries, 28 public entries, stride 0x8 bytes.
pub mod gpio_status {
    pub struct GPIO_STATUS_SPEC;
    impl crate::RegisterSpec for GPIO_STATUS_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for GPIO_STATUS_SPEC {}
    pub type R = crate::R<GPIO_STATUS_SPEC>;
    impl crate::ResetValue for GPIO_STATUS_SPEC {
        const RESET_VALUE: u32 = 0x0;
    }
    /// Output data from the selected function, before the output override.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod outfromperi {
        pub type R = crate::BitReader;
    }
    /// Output data after the output override, as sent to the pad.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod outtopad {
        pub type R = crate::BitReader;
    }
    /// Output enable from the selected function, before the output enable override.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod oefromperi {
        pub type R = crate::BitReader;
    }
    /// Output enable after the override, as sent to the pad. The pad's OD bit can still disable the output.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod oetopad {
        pub type R = crate::BitReader;
    }
    /// 1 if the selected function takes the raw pad input directly, bypassing the filter and the input override (for example a clock input).
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod inisdirect {
        pub type R = crate::BitReader;
    }
    /// Input value as received from the pad input buffer, before any processing.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod infrompad {
        pub type R = crate::BitReader;
    }
    /// Input value after the filter, before the input override.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod infiltered {
        pub type R = crate::BitReader;
    }
    /// Input value after the input override, as passed to the peripheral. Direct input paths bypass it.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod intoperi {
        pub type R = crate::BitReader;
    }
    /// Latched when the raw input falls from 1 to 0.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod event_edge_low {
        pub type R = crate::BitReader;
    }
    /// Latched when the raw input rises from 0 to 1.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod event_edge_high {
        pub type R = crate::BitReader;
    }
    /// Set while the raw input is low.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod event_level_low {
        pub type R = crate::BitReader;
    }
    /// Set while the raw input is high.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod event_level_high {
        pub type R = crate::BitReader;
    }
    /// Latched when the filtered input falls.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod event_f_edge_low {
        pub type R = crate::BitReader;
    }
    /// Latched when the filtered input rises.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod event_f_edge_high {
        pub type R = crate::BitReader;
    }
    /// Set once the input has stayed low for the debounce interval.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod event_db_level_low {
        pub type R = crate::BitReader;
    }
    /// Set once the input has stayed high for the debounce interval.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod event_db_level_high {
        pub type R = crate::BitReader;
    }
    /// OR of the eight event flags enabled by this pin's IRQMASK bits, before the interrupt override.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod irqcombined {
        pub type R = crate::BitReader;
    }
    /// This pin's interrupt after the interrupt override, before the per-destination enables.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod irqtoproc {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn outfromperi(&self) -> outfromperi::R {
            crate::BitReader::new(((self.bits() >> 8) & 1) != 0)
        }
        #[inline(always)]
        pub fn outtopad(&self) -> outtopad::R {
            crate::BitReader::new(((self.bits() >> 9) & 1) != 0)
        }
        #[inline(always)]
        pub fn oefromperi(&self) -> oefromperi::R {
            crate::BitReader::new(((self.bits() >> 12) & 1) != 0)
        }
        #[inline(always)]
        pub fn oetopad(&self) -> oetopad::R {
            crate::BitReader::new(((self.bits() >> 13) & 1) != 0)
        }
        #[inline(always)]
        pub fn inisdirect(&self) -> inisdirect::R {
            crate::BitReader::new(((self.bits() >> 16) & 1) != 0)
        }
        #[inline(always)]
        pub fn infrompad(&self) -> infrompad::R {
            crate::BitReader::new(((self.bits() >> 17) & 1) != 0)
        }
        #[inline(always)]
        pub fn infiltered(&self) -> infiltered::R {
            crate::BitReader::new(((self.bits() >> 18) & 1) != 0)
        }
        #[inline(always)]
        pub fn intoperi(&self) -> intoperi::R {
            crate::BitReader::new(((self.bits() >> 19) & 1) != 0)
        }
        #[inline(always)]
        pub fn event_edge_low(&self) -> event_edge_low::R {
            crate::BitReader::new(((self.bits() >> 20) & 1) != 0)
        }
        #[inline(always)]
        pub fn event_edge_high(&self) -> event_edge_high::R {
            crate::BitReader::new(((self.bits() >> 21) & 1) != 0)
        }
        #[inline(always)]
        pub fn event_level_low(&self) -> event_level_low::R {
            crate::BitReader::new(((self.bits() >> 22) & 1) != 0)
        }
        #[inline(always)]
        pub fn event_level_high(&self) -> event_level_high::R {
            crate::BitReader::new(((self.bits() >> 23) & 1) != 0)
        }
        #[inline(always)]
        pub fn event_f_edge_low(&self) -> event_f_edge_low::R {
            crate::BitReader::new(((self.bits() >> 24) & 1) != 0)
        }
        #[inline(always)]
        pub fn event_f_edge_high(&self) -> event_f_edge_high::R {
            crate::BitReader::new(((self.bits() >> 25) & 1) != 0)
        }
        #[inline(always)]
        pub fn event_db_level_low(&self) -> event_db_level_low::R {
            crate::BitReader::new(((self.bits() >> 26) & 1) != 0)
        }
        #[inline(always)]
        pub fn event_db_level_high(&self) -> event_db_level_high::R {
            crate::BitReader::new(((self.bits() >> 27) & 1) != 0)
        }
        #[inline(always)]
        pub fn irqcombined(&self) -> irqcombined::R {
            crate::BitReader::new(((self.bits() >> 28) & 1) != 0)
        }
        #[inline(always)]
        pub fn irqtoproc(&self) -> irqtoproc::R {
            crate::BitReader::new(((self.bits() >> 29) & 1) != 0)
        }
    }
}

/// Per-pad control: function select, input filtering, interrupt event masks and signal overrides.
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
/// Array layout: 28 physical entries, 28 public entries, stride 0x8 bytes.
pub type GPIO_CTRL = Reg<gpio_ctrl::GPIO_CTRL_SPEC>;
/// Per-pad control: function select, input filtering, interrupt event masks and signal overrides.
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
/// Array layout: 28 physical entries, 28 public entries, stride 0x8 bytes.
pub mod gpio_ctrl {
    pub struct GPIO_CTRL_SPEC;
    impl crate::RegisterSpec for GPIO_CTRL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for GPIO_CTRL_SPEC {}
    pub type R = crate::R<GPIO_CTRL_SPEC>;
    impl crate::UnsafeWritable for GPIO_CTRL_SPEC {}
    impl crate::ResetValue for GPIO_CTRL_SPEC {
        const RESET_VALUE: u32 = 0x1f;
    }
    impl crate::UnsafeAtomicSet for GPIO_CTRL_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for GPIO_CTRL_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for GPIO_CTRL_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// Function select. 31 (the reset value) disconnects the pad from every function. Other values are pin-specific; for bank 0 see the pinmux matrix in this database. Values without a named function are reserved. If several pads are routed to the same peripheral input, their inputs are ORed, so normally route at most one pad to each input.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod funcsel {
        pub type R = crate::FieldReader<u32>;
    }
    /// Input filter/debounce setting, counted in GPIO bank ticks. The exact mapping to a time interval, and the effect of 0, are unknown.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod f_m {
        pub type R = crate::FieldReader<u32>;
    }
    /// Output data override, applied to the selected function's output before the pad. 0 passes it through, 1 inverts it, 2 forces 0, 3 forces 1. Whether the pad actually drives also depends on OEOVER and the pad settings.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod outover {
        pub type R = crate::FieldReader<u32>;
        pub const FUNCSEL: u32 = 0x0;
        pub const FUNCSEL_INVERTED: u32 = 0x1;
        pub const LOW: u32 = 0x2;
        pub const HIGH: u32 = 0x3;
    }
    /// Output enable override. 0 uses the selected function's output enable, 1 inverts it, 2 forces the output off (high impedance), 3 forces it on. The pad's OD bit in PADS can still disable the output.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod oeover {
        pub type R = crate::FieldReader<u32>;
        pub const FUNCSEL: u32 = 0x0;
        pub const FUNCSEL_INVERTED: u32 = 0x1;
        pub const DISABLE: u32 = 0x2;
        pub const ENABLE: u32 = 0x3;
    }
    /// Input override, applied to the filtered input before it reaches the peripheral. 0 passes it through, 1 inverts it, 2 forces 0, 3 forces 1. Functions with a direct input path bypass it.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod inover {
        pub type R = crate::FieldReader<u32>;
        pub const NORMAL: u32 = 0x0;
        pub const INVERTED: u32 = 0x1;
        pub const LOW: u32 = 0x2;
        pub const HIGH: u32 = 0x3;
    }
    /// 1 includes the raw falling-edge event in this pin's combined interrupt; 0 leaves it out.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod irqmask_edge_low {
        pub type R = crate::BitReader;
    }
    /// 1 includes the raw rising-edge event in this pin's combined interrupt; 0 leaves it out.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod irqmask_edge_high {
        pub type R = crate::BitReader;
    }
    /// 1 includes the raw low-level event in this pin's combined interrupt; 0 leaves it out.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod irqmask_level_low {
        pub type R = crate::BitReader;
    }
    /// 1 includes the raw high-level event in this pin's combined interrupt; 0 leaves it out.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod irqmask_level_high {
        pub type R = crate::BitReader;
    }
    /// 1 includes the filtered falling-edge event in this pin's combined interrupt; 0 leaves it out.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod irqmask_f_edge_low {
        pub type R = crate::BitReader;
    }
    /// 1 includes the filtered rising-edge event in this pin's combined interrupt; 0 leaves it out.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod irqmask_f_edge_high {
        pub type R = crate::BitReader;
    }
    /// 1 includes the debounced low-level event in this pin's combined interrupt; 0 leaves it out.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod irqmask_db_level_low {
        pub type R = crate::BitReader;
    }
    /// 1 includes the debounced high-level event in this pin's combined interrupt; 0 leaves it out.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod irqmask_db_level_high {
        pub type R = crate::BitReader;
    }
    /// Write 1 to clear this pad's latched edge events; the bit then returns to 0 by itself. Writing 0 has no effect. Level events that are still active assert again.
    /// Field write behavior: SelfClearing.
    /// Typed write API is not exposed by register policy.
    pub mod irqreset {
        pub type R = crate::BitReader;
    }
    /// Interrupt override, applied to this pin's combined interrupt. 0 passes it through, 1 inverts it, 2 forces 0, 3 forces 1.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod irqover {
        pub type R = crate::FieldReader<u32>;
        pub const NORMAL: u32 = 0x0;
        pub const INVERTED: u32 = 0x1;
        pub const LOW: u32 = 0x2;
        pub const HIGH: u32 = 0x3;
    }
    impl R {
        #[inline(always)]
        pub fn funcsel(&self) -> funcsel::R {
            crate::FieldReader::new((self.bits() >> 0) & 0x1f)
        }
        #[inline(always)]
        pub fn f_m(&self) -> f_m::R {
            crate::FieldReader::new((self.bits() >> 5) & 0x7f)
        }
        #[inline(always)]
        pub fn outover(&self) -> outover::R {
            crate::FieldReader::new((self.bits() >> 12) & 0x3)
        }
        #[inline(always)]
        pub fn oeover(&self) -> oeover::R {
            crate::FieldReader::new((self.bits() >> 14) & 0x3)
        }
        #[inline(always)]
        pub fn inover(&self) -> inover::R {
            crate::FieldReader::new((self.bits() >> 16) & 0x3)
        }
        #[inline(always)]
        pub fn irqmask_edge_low(&self) -> irqmask_edge_low::R {
            crate::BitReader::new(((self.bits() >> 20) & 1) != 0)
        }
        #[inline(always)]
        pub fn irqmask_edge_high(&self) -> irqmask_edge_high::R {
            crate::BitReader::new(((self.bits() >> 21) & 1) != 0)
        }
        #[inline(always)]
        pub fn irqmask_level_low(&self) -> irqmask_level_low::R {
            crate::BitReader::new(((self.bits() >> 22) & 1) != 0)
        }
        #[inline(always)]
        pub fn irqmask_level_high(&self) -> irqmask_level_high::R {
            crate::BitReader::new(((self.bits() >> 23) & 1) != 0)
        }
        #[inline(always)]
        pub fn irqmask_f_edge_low(&self) -> irqmask_f_edge_low::R {
            crate::BitReader::new(((self.bits() >> 24) & 1) != 0)
        }
        #[inline(always)]
        pub fn irqmask_f_edge_high(&self) -> irqmask_f_edge_high::R {
            crate::BitReader::new(((self.bits() >> 25) & 1) != 0)
        }
        #[inline(always)]
        pub fn irqmask_db_level_low(&self) -> irqmask_db_level_low::R {
            crate::BitReader::new(((self.bits() >> 26) & 1) != 0)
        }
        #[inline(always)]
        pub fn irqmask_db_level_high(&self) -> irqmask_db_level_high::R {
            crate::BitReader::new(((self.bits() >> 27) & 1) != 0)
        }
        #[inline(always)]
        pub fn irqreset(&self) -> irqreset::R {
            crate::BitReader::new(((self.bits() >> 28) & 1) != 0)
        }
        #[inline(always)]
        pub fn irqover(&self) -> irqover::R {
            crate::FieldReader::new((self.bits() >> 30) & 0x3)
        }
    }
}

/// Raw interrupt state per pin (GPIO_STATUS.IRQTOPROC), before the per-destination enables. Read-only; clear latched events with the pin's GPIO_CTRL.IRQRESET.
/// Hardware access: read Plain; write None.
pub type INTR = Reg<intr::INTR_SPEC>;
/// Raw interrupt state per pin (GPIO_STATUS.IRQTOPROC), before the per-destination enables. Read-only; clear latched events with the pin's GPIO_CTRL.IRQRESET.
/// Hardware access: read Plain; write None.
pub mod intr {
    pub struct INTR_SPEC;
    impl crate::RegisterSpec for INTR_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for INTR_SPEC {}
    pub type R = crate::R<INTR_SPEC>;
    impl crate::ResetValue for INTR_SPEC {
        const RESET_VALUE: u32 = 0x0;
    }
    /// 1 if GPIO0 has an interrupt pending, before any destination enable.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio0 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO1 has an interrupt pending, before any destination enable.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio1 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO2 has an interrupt pending, before any destination enable.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio2 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO3 has an interrupt pending, before any destination enable.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio3 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO4 has an interrupt pending, before any destination enable.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio4 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO5 has an interrupt pending, before any destination enable.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio5 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO6 has an interrupt pending, before any destination enable.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio6 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO7 has an interrupt pending, before any destination enable.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio7 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO8 has an interrupt pending, before any destination enable.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio8 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO9 has an interrupt pending, before any destination enable.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio9 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO10 has an interrupt pending, before any destination enable.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio10 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO11 has an interrupt pending, before any destination enable.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio11 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO12 has an interrupt pending, before any destination enable.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio12 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO13 has an interrupt pending, before any destination enable.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio13 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO14 has an interrupt pending, before any destination enable.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio14 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO15 has an interrupt pending, before any destination enable.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio15 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO16 has an interrupt pending, before any destination enable.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio16 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO17 has an interrupt pending, before any destination enable.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio17 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO18 has an interrupt pending, before any destination enable.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio18 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO19 has an interrupt pending, before any destination enable.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio19 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO20 has an interrupt pending, before any destination enable.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio20 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO21 has an interrupt pending, before any destination enable.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio21 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO22 has an interrupt pending, before any destination enable.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio22 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO23 has an interrupt pending, before any destination enable.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio23 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO24 has an interrupt pending, before any destination enable.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio24 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO25 has an interrupt pending, before any destination enable.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio25 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO26 has an interrupt pending, before any destination enable.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio26 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO27 has an interrupt pending, before any destination enable.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio27 {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn gpio0(&self) -> gpio0::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio1(&self) -> gpio1::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio2(&self) -> gpio2::R {
            crate::BitReader::new(((self.bits() >> 2) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio3(&self) -> gpio3::R {
            crate::BitReader::new(((self.bits() >> 3) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio4(&self) -> gpio4::R {
            crate::BitReader::new(((self.bits() >> 4) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio5(&self) -> gpio5::R {
            crate::BitReader::new(((self.bits() >> 5) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio6(&self) -> gpio6::R {
            crate::BitReader::new(((self.bits() >> 6) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio7(&self) -> gpio7::R {
            crate::BitReader::new(((self.bits() >> 7) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio8(&self) -> gpio8::R {
            crate::BitReader::new(((self.bits() >> 8) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio9(&self) -> gpio9::R {
            crate::BitReader::new(((self.bits() >> 9) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio10(&self) -> gpio10::R {
            crate::BitReader::new(((self.bits() >> 10) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio11(&self) -> gpio11::R {
            crate::BitReader::new(((self.bits() >> 11) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio12(&self) -> gpio12::R {
            crate::BitReader::new(((self.bits() >> 12) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio13(&self) -> gpio13::R {
            crate::BitReader::new(((self.bits() >> 13) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio14(&self) -> gpio14::R {
            crate::BitReader::new(((self.bits() >> 14) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio15(&self) -> gpio15::R {
            crate::BitReader::new(((self.bits() >> 15) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio16(&self) -> gpio16::R {
            crate::BitReader::new(((self.bits() >> 16) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio17(&self) -> gpio17::R {
            crate::BitReader::new(((self.bits() >> 17) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio18(&self) -> gpio18::R {
            crate::BitReader::new(((self.bits() >> 18) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio19(&self) -> gpio19::R {
            crate::BitReader::new(((self.bits() >> 19) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio20(&self) -> gpio20::R {
            crate::BitReader::new(((self.bits() >> 20) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio21(&self) -> gpio21::R {
            crate::BitReader::new(((self.bits() >> 21) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio22(&self) -> gpio22::R {
            crate::BitReader::new(((self.bits() >> 22) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio23(&self) -> gpio23::R {
            crate::BitReader::new(((self.bits() >> 23) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio24(&self) -> gpio24::R {
            crate::BitReader::new(((self.bits() >> 24) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio25(&self) -> gpio25::R {
            crate::BitReader::new(((self.bits() >> 25) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio26(&self) -> gpio26::R {
            crate::BitReader::new(((self.bits() >> 26) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio27(&self) -> gpio27::R {
            crate::BitReader::new(((self.bits() >> 27) & 1) != 0)
        }
    }
}

/// Per-pin interrupt enables for RP1 processor 0. 1 lets the pin's interrupt through.
/// Hardware access: read Plain; write Plain.
pub type PROC0_INTE = Reg<proc0_inte::PROC0_INTE_SPEC>;
/// Per-pin interrupt enables for RP1 processor 0. 1 lets the pin's interrupt through.
/// Hardware access: read Plain; write Plain.
pub mod proc0_inte {
    pub struct PROC0_INTE_SPEC;
    impl crate::RegisterSpec for PROC0_INTE_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for PROC0_INTE_SPEC {}
    pub type R = crate::R<PROC0_INTE_SPEC>;
    impl crate::Modifiable for PROC0_INTE_SPEC {}
    impl crate::UnsafeWritable for PROC0_INTE_SPEC {}
    impl crate::ResetValue for PROC0_INTE_SPEC {
        const RESET_VALUE: u32 = 0x0;
    }
    impl crate::TypedAtomicSet for PROC0_INTE_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicSet for PROC0_INTE_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::TypedAtomicClear for PROC0_INTE_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicClear for PROC0_INTE_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::TypedAtomicXor for PROC0_INTE_SPEC {
        const OFFSET: usize = 0x1000;
    }
    impl crate::UnsafeAtomicXor for PROC0_INTE_SPEC {
        const OFFSET: usize = 0x1000;
    }
    pub type W = crate::W<PROC0_INTE_SPEC>;
    pub type AtomicSetW = crate::AtomicSetW<PROC0_INTE_SPEC>;
    pub type AtomicClearW = crate::AtomicClearW<PROC0_INTE_SPEC>;
    pub type AtomicXorW = crate::AtomicXorW<PROC0_INTE_SPEC>;
    /// 1 forwards GPIO0's interrupt to RP1 processor 0; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio0 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 0>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 0>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 0>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 0>;
    }
    /// 1 forwards GPIO1's interrupt to RP1 processor 0; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio1 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 1>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 1>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 1>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 1>;
    }
    /// 1 forwards GPIO2's interrupt to RP1 processor 0; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio2 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 2>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 2>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 2>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 2>;
    }
    /// 1 forwards GPIO3's interrupt to RP1 processor 0; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio3 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 3>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 3>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 3>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 3>;
    }
    /// 1 forwards GPIO4's interrupt to RP1 processor 0; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio4 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 4>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 4>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 4>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 4>;
    }
    /// 1 forwards GPIO5's interrupt to RP1 processor 0; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio5 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 5>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 5>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 5>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 5>;
    }
    /// 1 forwards GPIO6's interrupt to RP1 processor 0; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio6 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 6>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 6>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 6>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 6>;
    }
    /// 1 forwards GPIO7's interrupt to RP1 processor 0; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio7 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 7>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 7>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 7>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 7>;
    }
    /// 1 forwards GPIO8's interrupt to RP1 processor 0; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio8 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 8>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 8>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 8>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 8>;
    }
    /// 1 forwards GPIO9's interrupt to RP1 processor 0; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio9 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 9>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 9>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 9>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 9>;
    }
    /// 1 forwards GPIO10's interrupt to RP1 processor 0; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio10 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 10>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 10>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 10>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 10>;
    }
    /// 1 forwards GPIO11's interrupt to RP1 processor 0; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio11 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 11>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 11>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 11>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 11>;
    }
    /// 1 forwards GPIO12's interrupt to RP1 processor 0; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio12 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 12>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 12>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 12>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 12>;
    }
    /// 1 forwards GPIO13's interrupt to RP1 processor 0; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio13 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 13>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 13>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 13>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 13>;
    }
    /// 1 forwards GPIO14's interrupt to RP1 processor 0; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio14 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 14>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 14>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 14>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 14>;
    }
    /// 1 forwards GPIO15's interrupt to RP1 processor 0; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio15 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 15>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 15>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 15>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 15>;
    }
    /// 1 forwards GPIO16's interrupt to RP1 processor 0; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio16 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 16>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 16>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 16>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 16>;
    }
    /// 1 forwards GPIO17's interrupt to RP1 processor 0; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio17 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 17>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 17>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 17>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 17>;
    }
    /// 1 forwards GPIO18's interrupt to RP1 processor 0; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio18 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 18>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 18>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 18>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 18>;
    }
    /// 1 forwards GPIO19's interrupt to RP1 processor 0; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio19 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 19>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 19>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 19>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 19>;
    }
    /// 1 forwards GPIO20's interrupt to RP1 processor 0; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio20 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 20>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 20>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 20>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 20>;
    }
    /// 1 forwards GPIO21's interrupt to RP1 processor 0; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio21 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 21>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 21>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 21>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 21>;
    }
    /// 1 forwards GPIO22's interrupt to RP1 processor 0; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio22 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 22>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 22>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 22>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 22>;
    }
    /// 1 forwards GPIO23's interrupt to RP1 processor 0; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio23 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 23>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 23>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 23>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 23>;
    }
    /// 1 forwards GPIO24's interrupt to RP1 processor 0; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio24 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 24>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 24>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 24>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 24>;
    }
    /// 1 forwards GPIO25's interrupt to RP1 processor 0; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio25 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 25>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 25>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 25>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 25>;
    }
    /// 1 forwards GPIO26's interrupt to RP1 processor 0; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio26 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 26>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 26>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 26>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 26>;
    }
    /// 1 forwards GPIO27's interrupt to RP1 processor 0; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio27 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 27>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 27>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 27>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 27>;
    }
    impl R {
        #[inline(always)]
        pub fn gpio0(&self) -> gpio0::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio1(&self) -> gpio1::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio2(&self) -> gpio2::R {
            crate::BitReader::new(((self.bits() >> 2) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio3(&self) -> gpio3::R {
            crate::BitReader::new(((self.bits() >> 3) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio4(&self) -> gpio4::R {
            crate::BitReader::new(((self.bits() >> 4) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio5(&self) -> gpio5::R {
            crate::BitReader::new(((self.bits() >> 5) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio6(&self) -> gpio6::R {
            crate::BitReader::new(((self.bits() >> 6) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio7(&self) -> gpio7::R {
            crate::BitReader::new(((self.bits() >> 7) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio8(&self) -> gpio8::R {
            crate::BitReader::new(((self.bits() >> 8) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio9(&self) -> gpio9::R {
            crate::BitReader::new(((self.bits() >> 9) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio10(&self) -> gpio10::R {
            crate::BitReader::new(((self.bits() >> 10) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio11(&self) -> gpio11::R {
            crate::BitReader::new(((self.bits() >> 11) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio12(&self) -> gpio12::R {
            crate::BitReader::new(((self.bits() >> 12) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio13(&self) -> gpio13::R {
            crate::BitReader::new(((self.bits() >> 13) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio14(&self) -> gpio14::R {
            crate::BitReader::new(((self.bits() >> 14) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio15(&self) -> gpio15::R {
            crate::BitReader::new(((self.bits() >> 15) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio16(&self) -> gpio16::R {
            crate::BitReader::new(((self.bits() >> 16) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio17(&self) -> gpio17::R {
            crate::BitReader::new(((self.bits() >> 17) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio18(&self) -> gpio18::R {
            crate::BitReader::new(((self.bits() >> 18) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio19(&self) -> gpio19::R {
            crate::BitReader::new(((self.bits() >> 19) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio20(&self) -> gpio20::R {
            crate::BitReader::new(((self.bits() >> 20) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio21(&self) -> gpio21::R {
            crate::BitReader::new(((self.bits() >> 21) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio22(&self) -> gpio22::R {
            crate::BitReader::new(((self.bits() >> 22) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio23(&self) -> gpio23::R {
            crate::BitReader::new(((self.bits() >> 23) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio24(&self) -> gpio24::R {
            crate::BitReader::new(((self.bits() >> 24) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio25(&self) -> gpio25::R {
            crate::BitReader::new(((self.bits() >> 25) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio26(&self) -> gpio26::R {
            crate::BitReader::new(((self.bits() >> 26) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio27(&self) -> gpio27::R {
            crate::BitReader::new(((self.bits() >> 27) & 1) != 0)
        }
    }
    impl W {
        #[inline(always)]
        pub fn gpio0(&mut self) -> gpio0::W<'_> {
            gpio0::W::new(self)
        }
        #[inline(always)]
        pub fn gpio1(&mut self) -> gpio1::W<'_> {
            gpio1::W::new(self)
        }
        #[inline(always)]
        pub fn gpio2(&mut self) -> gpio2::W<'_> {
            gpio2::W::new(self)
        }
        #[inline(always)]
        pub fn gpio3(&mut self) -> gpio3::W<'_> {
            gpio3::W::new(self)
        }
        #[inline(always)]
        pub fn gpio4(&mut self) -> gpio4::W<'_> {
            gpio4::W::new(self)
        }
        #[inline(always)]
        pub fn gpio5(&mut self) -> gpio5::W<'_> {
            gpio5::W::new(self)
        }
        #[inline(always)]
        pub fn gpio6(&mut self) -> gpio6::W<'_> {
            gpio6::W::new(self)
        }
        #[inline(always)]
        pub fn gpio7(&mut self) -> gpio7::W<'_> {
            gpio7::W::new(self)
        }
        #[inline(always)]
        pub fn gpio8(&mut self) -> gpio8::W<'_> {
            gpio8::W::new(self)
        }
        #[inline(always)]
        pub fn gpio9(&mut self) -> gpio9::W<'_> {
            gpio9::W::new(self)
        }
        #[inline(always)]
        pub fn gpio10(&mut self) -> gpio10::W<'_> {
            gpio10::W::new(self)
        }
        #[inline(always)]
        pub fn gpio11(&mut self) -> gpio11::W<'_> {
            gpio11::W::new(self)
        }
        #[inline(always)]
        pub fn gpio12(&mut self) -> gpio12::W<'_> {
            gpio12::W::new(self)
        }
        #[inline(always)]
        pub fn gpio13(&mut self) -> gpio13::W<'_> {
            gpio13::W::new(self)
        }
        #[inline(always)]
        pub fn gpio14(&mut self) -> gpio14::W<'_> {
            gpio14::W::new(self)
        }
        #[inline(always)]
        pub fn gpio15(&mut self) -> gpio15::W<'_> {
            gpio15::W::new(self)
        }
        #[inline(always)]
        pub fn gpio16(&mut self) -> gpio16::W<'_> {
            gpio16::W::new(self)
        }
        #[inline(always)]
        pub fn gpio17(&mut self) -> gpio17::W<'_> {
            gpio17::W::new(self)
        }
        #[inline(always)]
        pub fn gpio18(&mut self) -> gpio18::W<'_> {
            gpio18::W::new(self)
        }
        #[inline(always)]
        pub fn gpio19(&mut self) -> gpio19::W<'_> {
            gpio19::W::new(self)
        }
        #[inline(always)]
        pub fn gpio20(&mut self) -> gpio20::W<'_> {
            gpio20::W::new(self)
        }
        #[inline(always)]
        pub fn gpio21(&mut self) -> gpio21::W<'_> {
            gpio21::W::new(self)
        }
        #[inline(always)]
        pub fn gpio22(&mut self) -> gpio22::W<'_> {
            gpio22::W::new(self)
        }
        #[inline(always)]
        pub fn gpio23(&mut self) -> gpio23::W<'_> {
            gpio23::W::new(self)
        }
        #[inline(always)]
        pub fn gpio24(&mut self) -> gpio24::W<'_> {
            gpio24::W::new(self)
        }
        #[inline(always)]
        pub fn gpio25(&mut self) -> gpio25::W<'_> {
            gpio25::W::new(self)
        }
        #[inline(always)]
        pub fn gpio26(&mut self) -> gpio26::W<'_> {
            gpio26::W::new(self)
        }
        #[inline(always)]
        pub fn gpio27(&mut self) -> gpio27::W<'_> {
            gpio27::W::new(self)
        }
    }
    impl AtomicSetW {
        /// 1 forwards GPIO0's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio0(&mut self) -> gpio0::ASW<'_> {
            gpio0::ASW::new(self)
        }
        /// 1 forwards GPIO1's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio1(&mut self) -> gpio1::ASW<'_> {
            gpio1::ASW::new(self)
        }
        /// 1 forwards GPIO2's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio2(&mut self) -> gpio2::ASW<'_> {
            gpio2::ASW::new(self)
        }
        /// 1 forwards GPIO3's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio3(&mut self) -> gpio3::ASW<'_> {
            gpio3::ASW::new(self)
        }
        /// 1 forwards GPIO4's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio4(&mut self) -> gpio4::ASW<'_> {
            gpio4::ASW::new(self)
        }
        /// 1 forwards GPIO5's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio5(&mut self) -> gpio5::ASW<'_> {
            gpio5::ASW::new(self)
        }
        /// 1 forwards GPIO6's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio6(&mut self) -> gpio6::ASW<'_> {
            gpio6::ASW::new(self)
        }
        /// 1 forwards GPIO7's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio7(&mut self) -> gpio7::ASW<'_> {
            gpio7::ASW::new(self)
        }
        /// 1 forwards GPIO8's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio8(&mut self) -> gpio8::ASW<'_> {
            gpio8::ASW::new(self)
        }
        /// 1 forwards GPIO9's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio9(&mut self) -> gpio9::ASW<'_> {
            gpio9::ASW::new(self)
        }
        /// 1 forwards GPIO10's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio10(&mut self) -> gpio10::ASW<'_> {
            gpio10::ASW::new(self)
        }
        /// 1 forwards GPIO11's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio11(&mut self) -> gpio11::ASW<'_> {
            gpio11::ASW::new(self)
        }
        /// 1 forwards GPIO12's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio12(&mut self) -> gpio12::ASW<'_> {
            gpio12::ASW::new(self)
        }
        /// 1 forwards GPIO13's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio13(&mut self) -> gpio13::ASW<'_> {
            gpio13::ASW::new(self)
        }
        /// 1 forwards GPIO14's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio14(&mut self) -> gpio14::ASW<'_> {
            gpio14::ASW::new(self)
        }
        /// 1 forwards GPIO15's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio15(&mut self) -> gpio15::ASW<'_> {
            gpio15::ASW::new(self)
        }
        /// 1 forwards GPIO16's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio16(&mut self) -> gpio16::ASW<'_> {
            gpio16::ASW::new(self)
        }
        /// 1 forwards GPIO17's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio17(&mut self) -> gpio17::ASW<'_> {
            gpio17::ASW::new(self)
        }
        /// 1 forwards GPIO18's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio18(&mut self) -> gpio18::ASW<'_> {
            gpio18::ASW::new(self)
        }
        /// 1 forwards GPIO19's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio19(&mut self) -> gpio19::ASW<'_> {
            gpio19::ASW::new(self)
        }
        /// 1 forwards GPIO20's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio20(&mut self) -> gpio20::ASW<'_> {
            gpio20::ASW::new(self)
        }
        /// 1 forwards GPIO21's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio21(&mut self) -> gpio21::ASW<'_> {
            gpio21::ASW::new(self)
        }
        /// 1 forwards GPIO22's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio22(&mut self) -> gpio22::ASW<'_> {
            gpio22::ASW::new(self)
        }
        /// 1 forwards GPIO23's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio23(&mut self) -> gpio23::ASW<'_> {
            gpio23::ASW::new(self)
        }
        /// 1 forwards GPIO24's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio24(&mut self) -> gpio24::ASW<'_> {
            gpio24::ASW::new(self)
        }
        /// 1 forwards GPIO25's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio25(&mut self) -> gpio25::ASW<'_> {
            gpio25::ASW::new(self)
        }
        /// 1 forwards GPIO26's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio26(&mut self) -> gpio26::ASW<'_> {
            gpio26::ASW::new(self)
        }
        /// 1 forwards GPIO27's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio27(&mut self) -> gpio27::ASW<'_> {
            gpio27::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// 1 forwards GPIO0's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio0(&mut self) -> gpio0::ACW<'_> {
            gpio0::ACW::new(self)
        }
        /// 1 forwards GPIO1's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio1(&mut self) -> gpio1::ACW<'_> {
            gpio1::ACW::new(self)
        }
        /// 1 forwards GPIO2's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio2(&mut self) -> gpio2::ACW<'_> {
            gpio2::ACW::new(self)
        }
        /// 1 forwards GPIO3's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio3(&mut self) -> gpio3::ACW<'_> {
            gpio3::ACW::new(self)
        }
        /// 1 forwards GPIO4's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio4(&mut self) -> gpio4::ACW<'_> {
            gpio4::ACW::new(self)
        }
        /// 1 forwards GPIO5's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio5(&mut self) -> gpio5::ACW<'_> {
            gpio5::ACW::new(self)
        }
        /// 1 forwards GPIO6's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio6(&mut self) -> gpio6::ACW<'_> {
            gpio6::ACW::new(self)
        }
        /// 1 forwards GPIO7's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio7(&mut self) -> gpio7::ACW<'_> {
            gpio7::ACW::new(self)
        }
        /// 1 forwards GPIO8's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio8(&mut self) -> gpio8::ACW<'_> {
            gpio8::ACW::new(self)
        }
        /// 1 forwards GPIO9's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio9(&mut self) -> gpio9::ACW<'_> {
            gpio9::ACW::new(self)
        }
        /// 1 forwards GPIO10's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio10(&mut self) -> gpio10::ACW<'_> {
            gpio10::ACW::new(self)
        }
        /// 1 forwards GPIO11's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio11(&mut self) -> gpio11::ACW<'_> {
            gpio11::ACW::new(self)
        }
        /// 1 forwards GPIO12's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio12(&mut self) -> gpio12::ACW<'_> {
            gpio12::ACW::new(self)
        }
        /// 1 forwards GPIO13's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio13(&mut self) -> gpio13::ACW<'_> {
            gpio13::ACW::new(self)
        }
        /// 1 forwards GPIO14's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio14(&mut self) -> gpio14::ACW<'_> {
            gpio14::ACW::new(self)
        }
        /// 1 forwards GPIO15's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio15(&mut self) -> gpio15::ACW<'_> {
            gpio15::ACW::new(self)
        }
        /// 1 forwards GPIO16's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio16(&mut self) -> gpio16::ACW<'_> {
            gpio16::ACW::new(self)
        }
        /// 1 forwards GPIO17's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio17(&mut self) -> gpio17::ACW<'_> {
            gpio17::ACW::new(self)
        }
        /// 1 forwards GPIO18's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio18(&mut self) -> gpio18::ACW<'_> {
            gpio18::ACW::new(self)
        }
        /// 1 forwards GPIO19's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio19(&mut self) -> gpio19::ACW<'_> {
            gpio19::ACW::new(self)
        }
        /// 1 forwards GPIO20's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio20(&mut self) -> gpio20::ACW<'_> {
            gpio20::ACW::new(self)
        }
        /// 1 forwards GPIO21's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio21(&mut self) -> gpio21::ACW<'_> {
            gpio21::ACW::new(self)
        }
        /// 1 forwards GPIO22's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio22(&mut self) -> gpio22::ACW<'_> {
            gpio22::ACW::new(self)
        }
        /// 1 forwards GPIO23's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio23(&mut self) -> gpio23::ACW<'_> {
            gpio23::ACW::new(self)
        }
        /// 1 forwards GPIO24's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio24(&mut self) -> gpio24::ACW<'_> {
            gpio24::ACW::new(self)
        }
        /// 1 forwards GPIO25's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio25(&mut self) -> gpio25::ACW<'_> {
            gpio25::ACW::new(self)
        }
        /// 1 forwards GPIO26's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio26(&mut self) -> gpio26::ACW<'_> {
            gpio26::ACW::new(self)
        }
        /// 1 forwards GPIO27's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio27(&mut self) -> gpio27::ACW<'_> {
            gpio27::ACW::new(self)
        }
    }
    impl AtomicXorW {
        /// 1 forwards GPIO0's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio0(&mut self) -> gpio0::AXW<'_> {
            gpio0::AXW::new(self)
        }
        /// 1 forwards GPIO1's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio1(&mut self) -> gpio1::AXW<'_> {
            gpio1::AXW::new(self)
        }
        /// 1 forwards GPIO2's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio2(&mut self) -> gpio2::AXW<'_> {
            gpio2::AXW::new(self)
        }
        /// 1 forwards GPIO3's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio3(&mut self) -> gpio3::AXW<'_> {
            gpio3::AXW::new(self)
        }
        /// 1 forwards GPIO4's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio4(&mut self) -> gpio4::AXW<'_> {
            gpio4::AXW::new(self)
        }
        /// 1 forwards GPIO5's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio5(&mut self) -> gpio5::AXW<'_> {
            gpio5::AXW::new(self)
        }
        /// 1 forwards GPIO6's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio6(&mut self) -> gpio6::AXW<'_> {
            gpio6::AXW::new(self)
        }
        /// 1 forwards GPIO7's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio7(&mut self) -> gpio7::AXW<'_> {
            gpio7::AXW::new(self)
        }
        /// 1 forwards GPIO8's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio8(&mut self) -> gpio8::AXW<'_> {
            gpio8::AXW::new(self)
        }
        /// 1 forwards GPIO9's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio9(&mut self) -> gpio9::AXW<'_> {
            gpio9::AXW::new(self)
        }
        /// 1 forwards GPIO10's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio10(&mut self) -> gpio10::AXW<'_> {
            gpio10::AXW::new(self)
        }
        /// 1 forwards GPIO11's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio11(&mut self) -> gpio11::AXW<'_> {
            gpio11::AXW::new(self)
        }
        /// 1 forwards GPIO12's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio12(&mut self) -> gpio12::AXW<'_> {
            gpio12::AXW::new(self)
        }
        /// 1 forwards GPIO13's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio13(&mut self) -> gpio13::AXW<'_> {
            gpio13::AXW::new(self)
        }
        /// 1 forwards GPIO14's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio14(&mut self) -> gpio14::AXW<'_> {
            gpio14::AXW::new(self)
        }
        /// 1 forwards GPIO15's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio15(&mut self) -> gpio15::AXW<'_> {
            gpio15::AXW::new(self)
        }
        /// 1 forwards GPIO16's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio16(&mut self) -> gpio16::AXW<'_> {
            gpio16::AXW::new(self)
        }
        /// 1 forwards GPIO17's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio17(&mut self) -> gpio17::AXW<'_> {
            gpio17::AXW::new(self)
        }
        /// 1 forwards GPIO18's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio18(&mut self) -> gpio18::AXW<'_> {
            gpio18::AXW::new(self)
        }
        /// 1 forwards GPIO19's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio19(&mut self) -> gpio19::AXW<'_> {
            gpio19::AXW::new(self)
        }
        /// 1 forwards GPIO20's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio20(&mut self) -> gpio20::AXW<'_> {
            gpio20::AXW::new(self)
        }
        /// 1 forwards GPIO21's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio21(&mut self) -> gpio21::AXW<'_> {
            gpio21::AXW::new(self)
        }
        /// 1 forwards GPIO22's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio22(&mut self) -> gpio22::AXW<'_> {
            gpio22::AXW::new(self)
        }
        /// 1 forwards GPIO23's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio23(&mut self) -> gpio23::AXW<'_> {
            gpio23::AXW::new(self)
        }
        /// 1 forwards GPIO24's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio24(&mut self) -> gpio24::AXW<'_> {
            gpio24::AXW::new(self)
        }
        /// 1 forwards GPIO25's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio25(&mut self) -> gpio25::AXW<'_> {
            gpio25::AXW::new(self)
        }
        /// 1 forwards GPIO26's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio26(&mut self) -> gpio26::AXW<'_> {
            gpio26::AXW::new(self)
        }
        /// 1 forwards GPIO27's interrupt to RP1 processor 0; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio27(&mut self) -> gpio27::AXW<'_> {
            gpio27::AXW::new(self)
        }
    }
}

/// Per-pin interrupt force for RP1 processor 0. 1 forces the matching status bit.
/// Hardware access: read Plain; write Plain.
pub type PROC0_INTF = Reg<proc0_intf::PROC0_INTF_SPEC>;
/// Per-pin interrupt force for RP1 processor 0. 1 forces the matching status bit.
/// Hardware access: read Plain; write Plain.
pub mod proc0_intf {
    pub struct PROC0_INTF_SPEC;
    impl crate::RegisterSpec for PROC0_INTF_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for PROC0_INTF_SPEC {}
    pub type R = crate::R<PROC0_INTF_SPEC>;
    impl crate::Modifiable for PROC0_INTF_SPEC {}
    impl crate::UnsafeWritable for PROC0_INTF_SPEC {}
    impl crate::ResetValue for PROC0_INTF_SPEC {
        const RESET_VALUE: u32 = 0x0;
    }
    impl crate::TypedAtomicSet for PROC0_INTF_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicSet for PROC0_INTF_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::TypedAtomicClear for PROC0_INTF_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicClear for PROC0_INTF_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::TypedAtomicXor for PROC0_INTF_SPEC {
        const OFFSET: usize = 0x1000;
    }
    impl crate::UnsafeAtomicXor for PROC0_INTF_SPEC {
        const OFFSET: usize = 0x1000;
    }
    pub type W = crate::W<PROC0_INTF_SPEC>;
    pub type AtomicSetW = crate::AtomicSetW<PROC0_INTF_SPEC>;
    pub type AtomicClearW = crate::AtomicClearW<PROC0_INTF_SPEC>;
    pub type AtomicXorW = crate::AtomicXorW<PROC0_INTF_SPEC>;
    /// 1 forces GPIO0's interrupt to RP1 processor 0; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio0 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 0>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 0>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 0>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 0>;
    }
    /// 1 forces GPIO1's interrupt to RP1 processor 0; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio1 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 1>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 1>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 1>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 1>;
    }
    /// 1 forces GPIO2's interrupt to RP1 processor 0; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio2 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 2>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 2>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 2>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 2>;
    }
    /// 1 forces GPIO3's interrupt to RP1 processor 0; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio3 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 3>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 3>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 3>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 3>;
    }
    /// 1 forces GPIO4's interrupt to RP1 processor 0; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio4 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 4>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 4>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 4>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 4>;
    }
    /// 1 forces GPIO5's interrupt to RP1 processor 0; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio5 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 5>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 5>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 5>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 5>;
    }
    /// 1 forces GPIO6's interrupt to RP1 processor 0; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio6 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 6>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 6>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 6>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 6>;
    }
    /// 1 forces GPIO7's interrupt to RP1 processor 0; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio7 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 7>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 7>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 7>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 7>;
    }
    /// 1 forces GPIO8's interrupt to RP1 processor 0; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio8 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 8>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 8>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 8>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 8>;
    }
    /// 1 forces GPIO9's interrupt to RP1 processor 0; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio9 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 9>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 9>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 9>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 9>;
    }
    /// 1 forces GPIO10's interrupt to RP1 processor 0; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio10 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 10>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 10>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 10>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 10>;
    }
    /// 1 forces GPIO11's interrupt to RP1 processor 0; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio11 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 11>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 11>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 11>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 11>;
    }
    /// 1 forces GPIO12's interrupt to RP1 processor 0; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio12 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 12>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 12>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 12>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 12>;
    }
    /// 1 forces GPIO13's interrupt to RP1 processor 0; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio13 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 13>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 13>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 13>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 13>;
    }
    /// 1 forces GPIO14's interrupt to RP1 processor 0; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio14 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 14>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 14>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 14>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 14>;
    }
    /// 1 forces GPIO15's interrupt to RP1 processor 0; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio15 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 15>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 15>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 15>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 15>;
    }
    /// 1 forces GPIO16's interrupt to RP1 processor 0; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio16 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 16>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 16>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 16>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 16>;
    }
    /// 1 forces GPIO17's interrupt to RP1 processor 0; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio17 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 17>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 17>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 17>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 17>;
    }
    /// 1 forces GPIO18's interrupt to RP1 processor 0; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio18 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 18>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 18>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 18>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 18>;
    }
    /// 1 forces GPIO19's interrupt to RP1 processor 0; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio19 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 19>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 19>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 19>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 19>;
    }
    /// 1 forces GPIO20's interrupt to RP1 processor 0; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio20 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 20>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 20>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 20>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 20>;
    }
    /// 1 forces GPIO21's interrupt to RP1 processor 0; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio21 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 21>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 21>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 21>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 21>;
    }
    /// 1 forces GPIO22's interrupt to RP1 processor 0; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio22 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 22>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 22>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 22>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 22>;
    }
    /// 1 forces GPIO23's interrupt to RP1 processor 0; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio23 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 23>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 23>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 23>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 23>;
    }
    /// 1 forces GPIO24's interrupt to RP1 processor 0; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio24 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 24>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 24>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 24>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 24>;
    }
    /// 1 forces GPIO25's interrupt to RP1 processor 0; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio25 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 25>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 25>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 25>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 25>;
    }
    /// 1 forces GPIO26's interrupt to RP1 processor 0; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio26 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 26>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 26>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 26>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 26>;
    }
    /// 1 forces GPIO27's interrupt to RP1 processor 0; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio27 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 27>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 27>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 27>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 27>;
    }
    impl R {
        #[inline(always)]
        pub fn gpio0(&self) -> gpio0::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio1(&self) -> gpio1::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio2(&self) -> gpio2::R {
            crate::BitReader::new(((self.bits() >> 2) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio3(&self) -> gpio3::R {
            crate::BitReader::new(((self.bits() >> 3) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio4(&self) -> gpio4::R {
            crate::BitReader::new(((self.bits() >> 4) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio5(&self) -> gpio5::R {
            crate::BitReader::new(((self.bits() >> 5) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio6(&self) -> gpio6::R {
            crate::BitReader::new(((self.bits() >> 6) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio7(&self) -> gpio7::R {
            crate::BitReader::new(((self.bits() >> 7) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio8(&self) -> gpio8::R {
            crate::BitReader::new(((self.bits() >> 8) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio9(&self) -> gpio9::R {
            crate::BitReader::new(((self.bits() >> 9) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio10(&self) -> gpio10::R {
            crate::BitReader::new(((self.bits() >> 10) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio11(&self) -> gpio11::R {
            crate::BitReader::new(((self.bits() >> 11) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio12(&self) -> gpio12::R {
            crate::BitReader::new(((self.bits() >> 12) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio13(&self) -> gpio13::R {
            crate::BitReader::new(((self.bits() >> 13) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio14(&self) -> gpio14::R {
            crate::BitReader::new(((self.bits() >> 14) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio15(&self) -> gpio15::R {
            crate::BitReader::new(((self.bits() >> 15) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio16(&self) -> gpio16::R {
            crate::BitReader::new(((self.bits() >> 16) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio17(&self) -> gpio17::R {
            crate::BitReader::new(((self.bits() >> 17) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio18(&self) -> gpio18::R {
            crate::BitReader::new(((self.bits() >> 18) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio19(&self) -> gpio19::R {
            crate::BitReader::new(((self.bits() >> 19) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio20(&self) -> gpio20::R {
            crate::BitReader::new(((self.bits() >> 20) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio21(&self) -> gpio21::R {
            crate::BitReader::new(((self.bits() >> 21) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio22(&self) -> gpio22::R {
            crate::BitReader::new(((self.bits() >> 22) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio23(&self) -> gpio23::R {
            crate::BitReader::new(((self.bits() >> 23) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio24(&self) -> gpio24::R {
            crate::BitReader::new(((self.bits() >> 24) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio25(&self) -> gpio25::R {
            crate::BitReader::new(((self.bits() >> 25) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio26(&self) -> gpio26::R {
            crate::BitReader::new(((self.bits() >> 26) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio27(&self) -> gpio27::R {
            crate::BitReader::new(((self.bits() >> 27) & 1) != 0)
        }
    }
    impl W {
        #[inline(always)]
        pub fn gpio0(&mut self) -> gpio0::W<'_> {
            gpio0::W::new(self)
        }
        #[inline(always)]
        pub fn gpio1(&mut self) -> gpio1::W<'_> {
            gpio1::W::new(self)
        }
        #[inline(always)]
        pub fn gpio2(&mut self) -> gpio2::W<'_> {
            gpio2::W::new(self)
        }
        #[inline(always)]
        pub fn gpio3(&mut self) -> gpio3::W<'_> {
            gpio3::W::new(self)
        }
        #[inline(always)]
        pub fn gpio4(&mut self) -> gpio4::W<'_> {
            gpio4::W::new(self)
        }
        #[inline(always)]
        pub fn gpio5(&mut self) -> gpio5::W<'_> {
            gpio5::W::new(self)
        }
        #[inline(always)]
        pub fn gpio6(&mut self) -> gpio6::W<'_> {
            gpio6::W::new(self)
        }
        #[inline(always)]
        pub fn gpio7(&mut self) -> gpio7::W<'_> {
            gpio7::W::new(self)
        }
        #[inline(always)]
        pub fn gpio8(&mut self) -> gpio8::W<'_> {
            gpio8::W::new(self)
        }
        #[inline(always)]
        pub fn gpio9(&mut self) -> gpio9::W<'_> {
            gpio9::W::new(self)
        }
        #[inline(always)]
        pub fn gpio10(&mut self) -> gpio10::W<'_> {
            gpio10::W::new(self)
        }
        #[inline(always)]
        pub fn gpio11(&mut self) -> gpio11::W<'_> {
            gpio11::W::new(self)
        }
        #[inline(always)]
        pub fn gpio12(&mut self) -> gpio12::W<'_> {
            gpio12::W::new(self)
        }
        #[inline(always)]
        pub fn gpio13(&mut self) -> gpio13::W<'_> {
            gpio13::W::new(self)
        }
        #[inline(always)]
        pub fn gpio14(&mut self) -> gpio14::W<'_> {
            gpio14::W::new(self)
        }
        #[inline(always)]
        pub fn gpio15(&mut self) -> gpio15::W<'_> {
            gpio15::W::new(self)
        }
        #[inline(always)]
        pub fn gpio16(&mut self) -> gpio16::W<'_> {
            gpio16::W::new(self)
        }
        #[inline(always)]
        pub fn gpio17(&mut self) -> gpio17::W<'_> {
            gpio17::W::new(self)
        }
        #[inline(always)]
        pub fn gpio18(&mut self) -> gpio18::W<'_> {
            gpio18::W::new(self)
        }
        #[inline(always)]
        pub fn gpio19(&mut self) -> gpio19::W<'_> {
            gpio19::W::new(self)
        }
        #[inline(always)]
        pub fn gpio20(&mut self) -> gpio20::W<'_> {
            gpio20::W::new(self)
        }
        #[inline(always)]
        pub fn gpio21(&mut self) -> gpio21::W<'_> {
            gpio21::W::new(self)
        }
        #[inline(always)]
        pub fn gpio22(&mut self) -> gpio22::W<'_> {
            gpio22::W::new(self)
        }
        #[inline(always)]
        pub fn gpio23(&mut self) -> gpio23::W<'_> {
            gpio23::W::new(self)
        }
        #[inline(always)]
        pub fn gpio24(&mut self) -> gpio24::W<'_> {
            gpio24::W::new(self)
        }
        #[inline(always)]
        pub fn gpio25(&mut self) -> gpio25::W<'_> {
            gpio25::W::new(self)
        }
        #[inline(always)]
        pub fn gpio26(&mut self) -> gpio26::W<'_> {
            gpio26::W::new(self)
        }
        #[inline(always)]
        pub fn gpio27(&mut self) -> gpio27::W<'_> {
            gpio27::W::new(self)
        }
    }
    impl AtomicSetW {
        /// 1 forces GPIO0's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio0(&mut self) -> gpio0::ASW<'_> {
            gpio0::ASW::new(self)
        }
        /// 1 forces GPIO1's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio1(&mut self) -> gpio1::ASW<'_> {
            gpio1::ASW::new(self)
        }
        /// 1 forces GPIO2's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio2(&mut self) -> gpio2::ASW<'_> {
            gpio2::ASW::new(self)
        }
        /// 1 forces GPIO3's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio3(&mut self) -> gpio3::ASW<'_> {
            gpio3::ASW::new(self)
        }
        /// 1 forces GPIO4's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio4(&mut self) -> gpio4::ASW<'_> {
            gpio4::ASW::new(self)
        }
        /// 1 forces GPIO5's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio5(&mut self) -> gpio5::ASW<'_> {
            gpio5::ASW::new(self)
        }
        /// 1 forces GPIO6's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio6(&mut self) -> gpio6::ASW<'_> {
            gpio6::ASW::new(self)
        }
        /// 1 forces GPIO7's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio7(&mut self) -> gpio7::ASW<'_> {
            gpio7::ASW::new(self)
        }
        /// 1 forces GPIO8's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio8(&mut self) -> gpio8::ASW<'_> {
            gpio8::ASW::new(self)
        }
        /// 1 forces GPIO9's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio9(&mut self) -> gpio9::ASW<'_> {
            gpio9::ASW::new(self)
        }
        /// 1 forces GPIO10's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio10(&mut self) -> gpio10::ASW<'_> {
            gpio10::ASW::new(self)
        }
        /// 1 forces GPIO11's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio11(&mut self) -> gpio11::ASW<'_> {
            gpio11::ASW::new(self)
        }
        /// 1 forces GPIO12's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio12(&mut self) -> gpio12::ASW<'_> {
            gpio12::ASW::new(self)
        }
        /// 1 forces GPIO13's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio13(&mut self) -> gpio13::ASW<'_> {
            gpio13::ASW::new(self)
        }
        /// 1 forces GPIO14's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio14(&mut self) -> gpio14::ASW<'_> {
            gpio14::ASW::new(self)
        }
        /// 1 forces GPIO15's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio15(&mut self) -> gpio15::ASW<'_> {
            gpio15::ASW::new(self)
        }
        /// 1 forces GPIO16's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio16(&mut self) -> gpio16::ASW<'_> {
            gpio16::ASW::new(self)
        }
        /// 1 forces GPIO17's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio17(&mut self) -> gpio17::ASW<'_> {
            gpio17::ASW::new(self)
        }
        /// 1 forces GPIO18's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio18(&mut self) -> gpio18::ASW<'_> {
            gpio18::ASW::new(self)
        }
        /// 1 forces GPIO19's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio19(&mut self) -> gpio19::ASW<'_> {
            gpio19::ASW::new(self)
        }
        /// 1 forces GPIO20's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio20(&mut self) -> gpio20::ASW<'_> {
            gpio20::ASW::new(self)
        }
        /// 1 forces GPIO21's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio21(&mut self) -> gpio21::ASW<'_> {
            gpio21::ASW::new(self)
        }
        /// 1 forces GPIO22's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio22(&mut self) -> gpio22::ASW<'_> {
            gpio22::ASW::new(self)
        }
        /// 1 forces GPIO23's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio23(&mut self) -> gpio23::ASW<'_> {
            gpio23::ASW::new(self)
        }
        /// 1 forces GPIO24's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio24(&mut self) -> gpio24::ASW<'_> {
            gpio24::ASW::new(self)
        }
        /// 1 forces GPIO25's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio25(&mut self) -> gpio25::ASW<'_> {
            gpio25::ASW::new(self)
        }
        /// 1 forces GPIO26's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio26(&mut self) -> gpio26::ASW<'_> {
            gpio26::ASW::new(self)
        }
        /// 1 forces GPIO27's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio27(&mut self) -> gpio27::ASW<'_> {
            gpio27::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// 1 forces GPIO0's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio0(&mut self) -> gpio0::ACW<'_> {
            gpio0::ACW::new(self)
        }
        /// 1 forces GPIO1's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio1(&mut self) -> gpio1::ACW<'_> {
            gpio1::ACW::new(self)
        }
        /// 1 forces GPIO2's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio2(&mut self) -> gpio2::ACW<'_> {
            gpio2::ACW::new(self)
        }
        /// 1 forces GPIO3's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio3(&mut self) -> gpio3::ACW<'_> {
            gpio3::ACW::new(self)
        }
        /// 1 forces GPIO4's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio4(&mut self) -> gpio4::ACW<'_> {
            gpio4::ACW::new(self)
        }
        /// 1 forces GPIO5's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio5(&mut self) -> gpio5::ACW<'_> {
            gpio5::ACW::new(self)
        }
        /// 1 forces GPIO6's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio6(&mut self) -> gpio6::ACW<'_> {
            gpio6::ACW::new(self)
        }
        /// 1 forces GPIO7's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio7(&mut self) -> gpio7::ACW<'_> {
            gpio7::ACW::new(self)
        }
        /// 1 forces GPIO8's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio8(&mut self) -> gpio8::ACW<'_> {
            gpio8::ACW::new(self)
        }
        /// 1 forces GPIO9's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio9(&mut self) -> gpio9::ACW<'_> {
            gpio9::ACW::new(self)
        }
        /// 1 forces GPIO10's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio10(&mut self) -> gpio10::ACW<'_> {
            gpio10::ACW::new(self)
        }
        /// 1 forces GPIO11's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio11(&mut self) -> gpio11::ACW<'_> {
            gpio11::ACW::new(self)
        }
        /// 1 forces GPIO12's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio12(&mut self) -> gpio12::ACW<'_> {
            gpio12::ACW::new(self)
        }
        /// 1 forces GPIO13's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio13(&mut self) -> gpio13::ACW<'_> {
            gpio13::ACW::new(self)
        }
        /// 1 forces GPIO14's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio14(&mut self) -> gpio14::ACW<'_> {
            gpio14::ACW::new(self)
        }
        /// 1 forces GPIO15's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio15(&mut self) -> gpio15::ACW<'_> {
            gpio15::ACW::new(self)
        }
        /// 1 forces GPIO16's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio16(&mut self) -> gpio16::ACW<'_> {
            gpio16::ACW::new(self)
        }
        /// 1 forces GPIO17's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio17(&mut self) -> gpio17::ACW<'_> {
            gpio17::ACW::new(self)
        }
        /// 1 forces GPIO18's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio18(&mut self) -> gpio18::ACW<'_> {
            gpio18::ACW::new(self)
        }
        /// 1 forces GPIO19's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio19(&mut self) -> gpio19::ACW<'_> {
            gpio19::ACW::new(self)
        }
        /// 1 forces GPIO20's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio20(&mut self) -> gpio20::ACW<'_> {
            gpio20::ACW::new(self)
        }
        /// 1 forces GPIO21's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio21(&mut self) -> gpio21::ACW<'_> {
            gpio21::ACW::new(self)
        }
        /// 1 forces GPIO22's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio22(&mut self) -> gpio22::ACW<'_> {
            gpio22::ACW::new(self)
        }
        /// 1 forces GPIO23's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio23(&mut self) -> gpio23::ACW<'_> {
            gpio23::ACW::new(self)
        }
        /// 1 forces GPIO24's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio24(&mut self) -> gpio24::ACW<'_> {
            gpio24::ACW::new(self)
        }
        /// 1 forces GPIO25's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio25(&mut self) -> gpio25::ACW<'_> {
            gpio25::ACW::new(self)
        }
        /// 1 forces GPIO26's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio26(&mut self) -> gpio26::ACW<'_> {
            gpio26::ACW::new(self)
        }
        /// 1 forces GPIO27's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio27(&mut self) -> gpio27::ACW<'_> {
            gpio27::ACW::new(self)
        }
    }
    impl AtomicXorW {
        /// 1 forces GPIO0's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio0(&mut self) -> gpio0::AXW<'_> {
            gpio0::AXW::new(self)
        }
        /// 1 forces GPIO1's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio1(&mut self) -> gpio1::AXW<'_> {
            gpio1::AXW::new(self)
        }
        /// 1 forces GPIO2's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio2(&mut self) -> gpio2::AXW<'_> {
            gpio2::AXW::new(self)
        }
        /// 1 forces GPIO3's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio3(&mut self) -> gpio3::AXW<'_> {
            gpio3::AXW::new(self)
        }
        /// 1 forces GPIO4's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio4(&mut self) -> gpio4::AXW<'_> {
            gpio4::AXW::new(self)
        }
        /// 1 forces GPIO5's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio5(&mut self) -> gpio5::AXW<'_> {
            gpio5::AXW::new(self)
        }
        /// 1 forces GPIO6's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio6(&mut self) -> gpio6::AXW<'_> {
            gpio6::AXW::new(self)
        }
        /// 1 forces GPIO7's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio7(&mut self) -> gpio7::AXW<'_> {
            gpio7::AXW::new(self)
        }
        /// 1 forces GPIO8's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio8(&mut self) -> gpio8::AXW<'_> {
            gpio8::AXW::new(self)
        }
        /// 1 forces GPIO9's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio9(&mut self) -> gpio9::AXW<'_> {
            gpio9::AXW::new(self)
        }
        /// 1 forces GPIO10's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio10(&mut self) -> gpio10::AXW<'_> {
            gpio10::AXW::new(self)
        }
        /// 1 forces GPIO11's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio11(&mut self) -> gpio11::AXW<'_> {
            gpio11::AXW::new(self)
        }
        /// 1 forces GPIO12's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio12(&mut self) -> gpio12::AXW<'_> {
            gpio12::AXW::new(self)
        }
        /// 1 forces GPIO13's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio13(&mut self) -> gpio13::AXW<'_> {
            gpio13::AXW::new(self)
        }
        /// 1 forces GPIO14's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio14(&mut self) -> gpio14::AXW<'_> {
            gpio14::AXW::new(self)
        }
        /// 1 forces GPIO15's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio15(&mut self) -> gpio15::AXW<'_> {
            gpio15::AXW::new(self)
        }
        /// 1 forces GPIO16's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio16(&mut self) -> gpio16::AXW<'_> {
            gpio16::AXW::new(self)
        }
        /// 1 forces GPIO17's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio17(&mut self) -> gpio17::AXW<'_> {
            gpio17::AXW::new(self)
        }
        /// 1 forces GPIO18's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio18(&mut self) -> gpio18::AXW<'_> {
            gpio18::AXW::new(self)
        }
        /// 1 forces GPIO19's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio19(&mut self) -> gpio19::AXW<'_> {
            gpio19::AXW::new(self)
        }
        /// 1 forces GPIO20's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio20(&mut self) -> gpio20::AXW<'_> {
            gpio20::AXW::new(self)
        }
        /// 1 forces GPIO21's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio21(&mut self) -> gpio21::AXW<'_> {
            gpio21::AXW::new(self)
        }
        /// 1 forces GPIO22's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio22(&mut self) -> gpio22::AXW<'_> {
            gpio22::AXW::new(self)
        }
        /// 1 forces GPIO23's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio23(&mut self) -> gpio23::AXW<'_> {
            gpio23::AXW::new(self)
        }
        /// 1 forces GPIO24's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio24(&mut self) -> gpio24::AXW<'_> {
            gpio24::AXW::new(self)
        }
        /// 1 forces GPIO25's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio25(&mut self) -> gpio25::AXW<'_> {
            gpio25::AXW::new(self)
        }
        /// 1 forces GPIO26's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio26(&mut self) -> gpio26::AXW<'_> {
            gpio26::AXW::new(self)
        }
        /// 1 forces GPIO27's interrupt to RP1 processor 0; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio27(&mut self) -> gpio27::AXW<'_> {
            gpio27::AXW::new(self)
        }
    }
}

/// Per-pin interrupt status seen by RP1 processor 0: the enabled raw interrupt combined with the force bit.
/// Hardware access: read Plain; write None.
pub type PROC0_INTS = Reg<proc0_ints::PROC0_INTS_SPEC>;
/// Per-pin interrupt status seen by RP1 processor 0: the enabled raw interrupt combined with the force bit.
/// Hardware access: read Plain; write None.
pub mod proc0_ints {
    pub struct PROC0_INTS_SPEC;
    impl crate::RegisterSpec for PROC0_INTS_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for PROC0_INTS_SPEC {}
    pub type R = crate::R<PROC0_INTS_SPEC>;
    impl crate::ResetValue for PROC0_INTS_SPEC {
        const RESET_VALUE: u32 = 0x0;
    }
    /// 1 if GPIO0 has an effective interrupt for RP1 processor 0.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio0 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO1 has an effective interrupt for RP1 processor 0.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio1 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO2 has an effective interrupt for RP1 processor 0.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio2 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO3 has an effective interrupt for RP1 processor 0.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio3 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO4 has an effective interrupt for RP1 processor 0.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio4 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO5 has an effective interrupt for RP1 processor 0.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio5 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO6 has an effective interrupt for RP1 processor 0.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio6 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO7 has an effective interrupt for RP1 processor 0.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio7 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO8 has an effective interrupt for RP1 processor 0.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio8 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO9 has an effective interrupt for RP1 processor 0.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio9 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO10 has an effective interrupt for RP1 processor 0.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio10 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO11 has an effective interrupt for RP1 processor 0.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio11 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO12 has an effective interrupt for RP1 processor 0.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio12 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO13 has an effective interrupt for RP1 processor 0.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio13 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO14 has an effective interrupt for RP1 processor 0.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio14 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO15 has an effective interrupt for RP1 processor 0.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio15 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO16 has an effective interrupt for RP1 processor 0.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio16 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO17 has an effective interrupt for RP1 processor 0.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio17 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO18 has an effective interrupt for RP1 processor 0.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio18 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO19 has an effective interrupt for RP1 processor 0.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio19 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO20 has an effective interrupt for RP1 processor 0.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio20 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO21 has an effective interrupt for RP1 processor 0.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio21 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO22 has an effective interrupt for RP1 processor 0.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio22 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO23 has an effective interrupt for RP1 processor 0.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio23 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO24 has an effective interrupt for RP1 processor 0.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio24 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO25 has an effective interrupt for RP1 processor 0.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio25 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO26 has an effective interrupt for RP1 processor 0.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio26 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO27 has an effective interrupt for RP1 processor 0.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio27 {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn gpio0(&self) -> gpio0::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio1(&self) -> gpio1::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio2(&self) -> gpio2::R {
            crate::BitReader::new(((self.bits() >> 2) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio3(&self) -> gpio3::R {
            crate::BitReader::new(((self.bits() >> 3) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio4(&self) -> gpio4::R {
            crate::BitReader::new(((self.bits() >> 4) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio5(&self) -> gpio5::R {
            crate::BitReader::new(((self.bits() >> 5) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio6(&self) -> gpio6::R {
            crate::BitReader::new(((self.bits() >> 6) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio7(&self) -> gpio7::R {
            crate::BitReader::new(((self.bits() >> 7) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio8(&self) -> gpio8::R {
            crate::BitReader::new(((self.bits() >> 8) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio9(&self) -> gpio9::R {
            crate::BitReader::new(((self.bits() >> 9) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio10(&self) -> gpio10::R {
            crate::BitReader::new(((self.bits() >> 10) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio11(&self) -> gpio11::R {
            crate::BitReader::new(((self.bits() >> 11) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio12(&self) -> gpio12::R {
            crate::BitReader::new(((self.bits() >> 12) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio13(&self) -> gpio13::R {
            crate::BitReader::new(((self.bits() >> 13) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio14(&self) -> gpio14::R {
            crate::BitReader::new(((self.bits() >> 14) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio15(&self) -> gpio15::R {
            crate::BitReader::new(((self.bits() >> 15) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio16(&self) -> gpio16::R {
            crate::BitReader::new(((self.bits() >> 16) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio17(&self) -> gpio17::R {
            crate::BitReader::new(((self.bits() >> 17) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio18(&self) -> gpio18::R {
            crate::BitReader::new(((self.bits() >> 18) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio19(&self) -> gpio19::R {
            crate::BitReader::new(((self.bits() >> 19) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio20(&self) -> gpio20::R {
            crate::BitReader::new(((self.bits() >> 20) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio21(&self) -> gpio21::R {
            crate::BitReader::new(((self.bits() >> 21) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio22(&self) -> gpio22::R {
            crate::BitReader::new(((self.bits() >> 22) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio23(&self) -> gpio23::R {
            crate::BitReader::new(((self.bits() >> 23) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio24(&self) -> gpio24::R {
            crate::BitReader::new(((self.bits() >> 24) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio25(&self) -> gpio25::R {
            crate::BitReader::new(((self.bits() >> 25) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio26(&self) -> gpio26::R {
            crate::BitReader::new(((self.bits() >> 26) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio27(&self) -> gpio27::R {
            crate::BitReader::new(((self.bits() >> 27) & 1) != 0)
        }
    }
}

/// Per-pin interrupt enables for RP1 processor 1. 1 lets the pin's interrupt through.
/// Hardware access: read Plain; write Plain.
pub type PROC1_INTE = Reg<proc1_inte::PROC1_INTE_SPEC>;
/// Per-pin interrupt enables for RP1 processor 1. 1 lets the pin's interrupt through.
/// Hardware access: read Plain; write Plain.
pub mod proc1_inte {
    pub struct PROC1_INTE_SPEC;
    impl crate::RegisterSpec for PROC1_INTE_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for PROC1_INTE_SPEC {}
    pub type R = crate::R<PROC1_INTE_SPEC>;
    impl crate::Modifiable for PROC1_INTE_SPEC {}
    impl crate::UnsafeWritable for PROC1_INTE_SPEC {}
    impl crate::ResetValue for PROC1_INTE_SPEC {
        const RESET_VALUE: u32 = 0x0;
    }
    impl crate::TypedAtomicSet for PROC1_INTE_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicSet for PROC1_INTE_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::TypedAtomicClear for PROC1_INTE_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicClear for PROC1_INTE_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::TypedAtomicXor for PROC1_INTE_SPEC {
        const OFFSET: usize = 0x1000;
    }
    impl crate::UnsafeAtomicXor for PROC1_INTE_SPEC {
        const OFFSET: usize = 0x1000;
    }
    pub type W = crate::W<PROC1_INTE_SPEC>;
    pub type AtomicSetW = crate::AtomicSetW<PROC1_INTE_SPEC>;
    pub type AtomicClearW = crate::AtomicClearW<PROC1_INTE_SPEC>;
    pub type AtomicXorW = crate::AtomicXorW<PROC1_INTE_SPEC>;
    /// 1 forwards GPIO0's interrupt to RP1 processor 1; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio0 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 0>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 0>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 0>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 0>;
    }
    /// 1 forwards GPIO1's interrupt to RP1 processor 1; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio1 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 1>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 1>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 1>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 1>;
    }
    /// 1 forwards GPIO2's interrupt to RP1 processor 1; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio2 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 2>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 2>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 2>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 2>;
    }
    /// 1 forwards GPIO3's interrupt to RP1 processor 1; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio3 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 3>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 3>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 3>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 3>;
    }
    /// 1 forwards GPIO4's interrupt to RP1 processor 1; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio4 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 4>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 4>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 4>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 4>;
    }
    /// 1 forwards GPIO5's interrupt to RP1 processor 1; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio5 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 5>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 5>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 5>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 5>;
    }
    /// 1 forwards GPIO6's interrupt to RP1 processor 1; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio6 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 6>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 6>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 6>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 6>;
    }
    /// 1 forwards GPIO7's interrupt to RP1 processor 1; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio7 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 7>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 7>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 7>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 7>;
    }
    /// 1 forwards GPIO8's interrupt to RP1 processor 1; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio8 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 8>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 8>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 8>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 8>;
    }
    /// 1 forwards GPIO9's interrupt to RP1 processor 1; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio9 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 9>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 9>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 9>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 9>;
    }
    /// 1 forwards GPIO10's interrupt to RP1 processor 1; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio10 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 10>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 10>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 10>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 10>;
    }
    /// 1 forwards GPIO11's interrupt to RP1 processor 1; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio11 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 11>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 11>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 11>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 11>;
    }
    /// 1 forwards GPIO12's interrupt to RP1 processor 1; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio12 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 12>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 12>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 12>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 12>;
    }
    /// 1 forwards GPIO13's interrupt to RP1 processor 1; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio13 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 13>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 13>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 13>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 13>;
    }
    /// 1 forwards GPIO14's interrupt to RP1 processor 1; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio14 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 14>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 14>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 14>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 14>;
    }
    /// 1 forwards GPIO15's interrupt to RP1 processor 1; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio15 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 15>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 15>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 15>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 15>;
    }
    /// 1 forwards GPIO16's interrupt to RP1 processor 1; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio16 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 16>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 16>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 16>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 16>;
    }
    /// 1 forwards GPIO17's interrupt to RP1 processor 1; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio17 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 17>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 17>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 17>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 17>;
    }
    /// 1 forwards GPIO18's interrupt to RP1 processor 1; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio18 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 18>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 18>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 18>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 18>;
    }
    /// 1 forwards GPIO19's interrupt to RP1 processor 1; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio19 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 19>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 19>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 19>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 19>;
    }
    /// 1 forwards GPIO20's interrupt to RP1 processor 1; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio20 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 20>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 20>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 20>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 20>;
    }
    /// 1 forwards GPIO21's interrupt to RP1 processor 1; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio21 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 21>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 21>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 21>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 21>;
    }
    /// 1 forwards GPIO22's interrupt to RP1 processor 1; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio22 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 22>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 22>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 22>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 22>;
    }
    /// 1 forwards GPIO23's interrupt to RP1 processor 1; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio23 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 23>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 23>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 23>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 23>;
    }
    /// 1 forwards GPIO24's interrupt to RP1 processor 1; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio24 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 24>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 24>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 24>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 24>;
    }
    /// 1 forwards GPIO25's interrupt to RP1 processor 1; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio25 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 25>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 25>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 25>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 25>;
    }
    /// 1 forwards GPIO26's interrupt to RP1 processor 1; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio26 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 26>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 26>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 26>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 26>;
    }
    /// 1 forwards GPIO27's interrupt to RP1 processor 1; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio27 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 27>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 27>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 27>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 27>;
    }
    impl R {
        #[inline(always)]
        pub fn gpio0(&self) -> gpio0::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio1(&self) -> gpio1::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio2(&self) -> gpio2::R {
            crate::BitReader::new(((self.bits() >> 2) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio3(&self) -> gpio3::R {
            crate::BitReader::new(((self.bits() >> 3) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio4(&self) -> gpio4::R {
            crate::BitReader::new(((self.bits() >> 4) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio5(&self) -> gpio5::R {
            crate::BitReader::new(((self.bits() >> 5) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio6(&self) -> gpio6::R {
            crate::BitReader::new(((self.bits() >> 6) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio7(&self) -> gpio7::R {
            crate::BitReader::new(((self.bits() >> 7) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio8(&self) -> gpio8::R {
            crate::BitReader::new(((self.bits() >> 8) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio9(&self) -> gpio9::R {
            crate::BitReader::new(((self.bits() >> 9) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio10(&self) -> gpio10::R {
            crate::BitReader::new(((self.bits() >> 10) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio11(&self) -> gpio11::R {
            crate::BitReader::new(((self.bits() >> 11) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio12(&self) -> gpio12::R {
            crate::BitReader::new(((self.bits() >> 12) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio13(&self) -> gpio13::R {
            crate::BitReader::new(((self.bits() >> 13) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio14(&self) -> gpio14::R {
            crate::BitReader::new(((self.bits() >> 14) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio15(&self) -> gpio15::R {
            crate::BitReader::new(((self.bits() >> 15) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio16(&self) -> gpio16::R {
            crate::BitReader::new(((self.bits() >> 16) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio17(&self) -> gpio17::R {
            crate::BitReader::new(((self.bits() >> 17) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio18(&self) -> gpio18::R {
            crate::BitReader::new(((self.bits() >> 18) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio19(&self) -> gpio19::R {
            crate::BitReader::new(((self.bits() >> 19) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio20(&self) -> gpio20::R {
            crate::BitReader::new(((self.bits() >> 20) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio21(&self) -> gpio21::R {
            crate::BitReader::new(((self.bits() >> 21) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio22(&self) -> gpio22::R {
            crate::BitReader::new(((self.bits() >> 22) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio23(&self) -> gpio23::R {
            crate::BitReader::new(((self.bits() >> 23) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio24(&self) -> gpio24::R {
            crate::BitReader::new(((self.bits() >> 24) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio25(&self) -> gpio25::R {
            crate::BitReader::new(((self.bits() >> 25) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio26(&self) -> gpio26::R {
            crate::BitReader::new(((self.bits() >> 26) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio27(&self) -> gpio27::R {
            crate::BitReader::new(((self.bits() >> 27) & 1) != 0)
        }
    }
    impl W {
        #[inline(always)]
        pub fn gpio0(&mut self) -> gpio0::W<'_> {
            gpio0::W::new(self)
        }
        #[inline(always)]
        pub fn gpio1(&mut self) -> gpio1::W<'_> {
            gpio1::W::new(self)
        }
        #[inline(always)]
        pub fn gpio2(&mut self) -> gpio2::W<'_> {
            gpio2::W::new(self)
        }
        #[inline(always)]
        pub fn gpio3(&mut self) -> gpio3::W<'_> {
            gpio3::W::new(self)
        }
        #[inline(always)]
        pub fn gpio4(&mut self) -> gpio4::W<'_> {
            gpio4::W::new(self)
        }
        #[inline(always)]
        pub fn gpio5(&mut self) -> gpio5::W<'_> {
            gpio5::W::new(self)
        }
        #[inline(always)]
        pub fn gpio6(&mut self) -> gpio6::W<'_> {
            gpio6::W::new(self)
        }
        #[inline(always)]
        pub fn gpio7(&mut self) -> gpio7::W<'_> {
            gpio7::W::new(self)
        }
        #[inline(always)]
        pub fn gpio8(&mut self) -> gpio8::W<'_> {
            gpio8::W::new(self)
        }
        #[inline(always)]
        pub fn gpio9(&mut self) -> gpio9::W<'_> {
            gpio9::W::new(self)
        }
        #[inline(always)]
        pub fn gpio10(&mut self) -> gpio10::W<'_> {
            gpio10::W::new(self)
        }
        #[inline(always)]
        pub fn gpio11(&mut self) -> gpio11::W<'_> {
            gpio11::W::new(self)
        }
        #[inline(always)]
        pub fn gpio12(&mut self) -> gpio12::W<'_> {
            gpio12::W::new(self)
        }
        #[inline(always)]
        pub fn gpio13(&mut self) -> gpio13::W<'_> {
            gpio13::W::new(self)
        }
        #[inline(always)]
        pub fn gpio14(&mut self) -> gpio14::W<'_> {
            gpio14::W::new(self)
        }
        #[inline(always)]
        pub fn gpio15(&mut self) -> gpio15::W<'_> {
            gpio15::W::new(self)
        }
        #[inline(always)]
        pub fn gpio16(&mut self) -> gpio16::W<'_> {
            gpio16::W::new(self)
        }
        #[inline(always)]
        pub fn gpio17(&mut self) -> gpio17::W<'_> {
            gpio17::W::new(self)
        }
        #[inline(always)]
        pub fn gpio18(&mut self) -> gpio18::W<'_> {
            gpio18::W::new(self)
        }
        #[inline(always)]
        pub fn gpio19(&mut self) -> gpio19::W<'_> {
            gpio19::W::new(self)
        }
        #[inline(always)]
        pub fn gpio20(&mut self) -> gpio20::W<'_> {
            gpio20::W::new(self)
        }
        #[inline(always)]
        pub fn gpio21(&mut self) -> gpio21::W<'_> {
            gpio21::W::new(self)
        }
        #[inline(always)]
        pub fn gpio22(&mut self) -> gpio22::W<'_> {
            gpio22::W::new(self)
        }
        #[inline(always)]
        pub fn gpio23(&mut self) -> gpio23::W<'_> {
            gpio23::W::new(self)
        }
        #[inline(always)]
        pub fn gpio24(&mut self) -> gpio24::W<'_> {
            gpio24::W::new(self)
        }
        #[inline(always)]
        pub fn gpio25(&mut self) -> gpio25::W<'_> {
            gpio25::W::new(self)
        }
        #[inline(always)]
        pub fn gpio26(&mut self) -> gpio26::W<'_> {
            gpio26::W::new(self)
        }
        #[inline(always)]
        pub fn gpio27(&mut self) -> gpio27::W<'_> {
            gpio27::W::new(self)
        }
    }
    impl AtomicSetW {
        /// 1 forwards GPIO0's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio0(&mut self) -> gpio0::ASW<'_> {
            gpio0::ASW::new(self)
        }
        /// 1 forwards GPIO1's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio1(&mut self) -> gpio1::ASW<'_> {
            gpio1::ASW::new(self)
        }
        /// 1 forwards GPIO2's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio2(&mut self) -> gpio2::ASW<'_> {
            gpio2::ASW::new(self)
        }
        /// 1 forwards GPIO3's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio3(&mut self) -> gpio3::ASW<'_> {
            gpio3::ASW::new(self)
        }
        /// 1 forwards GPIO4's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio4(&mut self) -> gpio4::ASW<'_> {
            gpio4::ASW::new(self)
        }
        /// 1 forwards GPIO5's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio5(&mut self) -> gpio5::ASW<'_> {
            gpio5::ASW::new(self)
        }
        /// 1 forwards GPIO6's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio6(&mut self) -> gpio6::ASW<'_> {
            gpio6::ASW::new(self)
        }
        /// 1 forwards GPIO7's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio7(&mut self) -> gpio7::ASW<'_> {
            gpio7::ASW::new(self)
        }
        /// 1 forwards GPIO8's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio8(&mut self) -> gpio8::ASW<'_> {
            gpio8::ASW::new(self)
        }
        /// 1 forwards GPIO9's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio9(&mut self) -> gpio9::ASW<'_> {
            gpio9::ASW::new(self)
        }
        /// 1 forwards GPIO10's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio10(&mut self) -> gpio10::ASW<'_> {
            gpio10::ASW::new(self)
        }
        /// 1 forwards GPIO11's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio11(&mut self) -> gpio11::ASW<'_> {
            gpio11::ASW::new(self)
        }
        /// 1 forwards GPIO12's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio12(&mut self) -> gpio12::ASW<'_> {
            gpio12::ASW::new(self)
        }
        /// 1 forwards GPIO13's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio13(&mut self) -> gpio13::ASW<'_> {
            gpio13::ASW::new(self)
        }
        /// 1 forwards GPIO14's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio14(&mut self) -> gpio14::ASW<'_> {
            gpio14::ASW::new(self)
        }
        /// 1 forwards GPIO15's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio15(&mut self) -> gpio15::ASW<'_> {
            gpio15::ASW::new(self)
        }
        /// 1 forwards GPIO16's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio16(&mut self) -> gpio16::ASW<'_> {
            gpio16::ASW::new(self)
        }
        /// 1 forwards GPIO17's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio17(&mut self) -> gpio17::ASW<'_> {
            gpio17::ASW::new(self)
        }
        /// 1 forwards GPIO18's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio18(&mut self) -> gpio18::ASW<'_> {
            gpio18::ASW::new(self)
        }
        /// 1 forwards GPIO19's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio19(&mut self) -> gpio19::ASW<'_> {
            gpio19::ASW::new(self)
        }
        /// 1 forwards GPIO20's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio20(&mut self) -> gpio20::ASW<'_> {
            gpio20::ASW::new(self)
        }
        /// 1 forwards GPIO21's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio21(&mut self) -> gpio21::ASW<'_> {
            gpio21::ASW::new(self)
        }
        /// 1 forwards GPIO22's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio22(&mut self) -> gpio22::ASW<'_> {
            gpio22::ASW::new(self)
        }
        /// 1 forwards GPIO23's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio23(&mut self) -> gpio23::ASW<'_> {
            gpio23::ASW::new(self)
        }
        /// 1 forwards GPIO24's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio24(&mut self) -> gpio24::ASW<'_> {
            gpio24::ASW::new(self)
        }
        /// 1 forwards GPIO25's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio25(&mut self) -> gpio25::ASW<'_> {
            gpio25::ASW::new(self)
        }
        /// 1 forwards GPIO26's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio26(&mut self) -> gpio26::ASW<'_> {
            gpio26::ASW::new(self)
        }
        /// 1 forwards GPIO27's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio27(&mut self) -> gpio27::ASW<'_> {
            gpio27::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// 1 forwards GPIO0's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio0(&mut self) -> gpio0::ACW<'_> {
            gpio0::ACW::new(self)
        }
        /// 1 forwards GPIO1's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio1(&mut self) -> gpio1::ACW<'_> {
            gpio1::ACW::new(self)
        }
        /// 1 forwards GPIO2's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio2(&mut self) -> gpio2::ACW<'_> {
            gpio2::ACW::new(self)
        }
        /// 1 forwards GPIO3's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio3(&mut self) -> gpio3::ACW<'_> {
            gpio3::ACW::new(self)
        }
        /// 1 forwards GPIO4's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio4(&mut self) -> gpio4::ACW<'_> {
            gpio4::ACW::new(self)
        }
        /// 1 forwards GPIO5's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio5(&mut self) -> gpio5::ACW<'_> {
            gpio5::ACW::new(self)
        }
        /// 1 forwards GPIO6's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio6(&mut self) -> gpio6::ACW<'_> {
            gpio6::ACW::new(self)
        }
        /// 1 forwards GPIO7's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio7(&mut self) -> gpio7::ACW<'_> {
            gpio7::ACW::new(self)
        }
        /// 1 forwards GPIO8's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio8(&mut self) -> gpio8::ACW<'_> {
            gpio8::ACW::new(self)
        }
        /// 1 forwards GPIO9's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio9(&mut self) -> gpio9::ACW<'_> {
            gpio9::ACW::new(self)
        }
        /// 1 forwards GPIO10's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio10(&mut self) -> gpio10::ACW<'_> {
            gpio10::ACW::new(self)
        }
        /// 1 forwards GPIO11's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio11(&mut self) -> gpio11::ACW<'_> {
            gpio11::ACW::new(self)
        }
        /// 1 forwards GPIO12's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio12(&mut self) -> gpio12::ACW<'_> {
            gpio12::ACW::new(self)
        }
        /// 1 forwards GPIO13's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio13(&mut self) -> gpio13::ACW<'_> {
            gpio13::ACW::new(self)
        }
        /// 1 forwards GPIO14's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio14(&mut self) -> gpio14::ACW<'_> {
            gpio14::ACW::new(self)
        }
        /// 1 forwards GPIO15's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio15(&mut self) -> gpio15::ACW<'_> {
            gpio15::ACW::new(self)
        }
        /// 1 forwards GPIO16's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio16(&mut self) -> gpio16::ACW<'_> {
            gpio16::ACW::new(self)
        }
        /// 1 forwards GPIO17's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio17(&mut self) -> gpio17::ACW<'_> {
            gpio17::ACW::new(self)
        }
        /// 1 forwards GPIO18's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio18(&mut self) -> gpio18::ACW<'_> {
            gpio18::ACW::new(self)
        }
        /// 1 forwards GPIO19's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio19(&mut self) -> gpio19::ACW<'_> {
            gpio19::ACW::new(self)
        }
        /// 1 forwards GPIO20's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio20(&mut self) -> gpio20::ACW<'_> {
            gpio20::ACW::new(self)
        }
        /// 1 forwards GPIO21's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio21(&mut self) -> gpio21::ACW<'_> {
            gpio21::ACW::new(self)
        }
        /// 1 forwards GPIO22's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio22(&mut self) -> gpio22::ACW<'_> {
            gpio22::ACW::new(self)
        }
        /// 1 forwards GPIO23's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio23(&mut self) -> gpio23::ACW<'_> {
            gpio23::ACW::new(self)
        }
        /// 1 forwards GPIO24's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio24(&mut self) -> gpio24::ACW<'_> {
            gpio24::ACW::new(self)
        }
        /// 1 forwards GPIO25's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio25(&mut self) -> gpio25::ACW<'_> {
            gpio25::ACW::new(self)
        }
        /// 1 forwards GPIO26's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio26(&mut self) -> gpio26::ACW<'_> {
            gpio26::ACW::new(self)
        }
        /// 1 forwards GPIO27's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio27(&mut self) -> gpio27::ACW<'_> {
            gpio27::ACW::new(self)
        }
    }
    impl AtomicXorW {
        /// 1 forwards GPIO0's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio0(&mut self) -> gpio0::AXW<'_> {
            gpio0::AXW::new(self)
        }
        /// 1 forwards GPIO1's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio1(&mut self) -> gpio1::AXW<'_> {
            gpio1::AXW::new(self)
        }
        /// 1 forwards GPIO2's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio2(&mut self) -> gpio2::AXW<'_> {
            gpio2::AXW::new(self)
        }
        /// 1 forwards GPIO3's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio3(&mut self) -> gpio3::AXW<'_> {
            gpio3::AXW::new(self)
        }
        /// 1 forwards GPIO4's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio4(&mut self) -> gpio4::AXW<'_> {
            gpio4::AXW::new(self)
        }
        /// 1 forwards GPIO5's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio5(&mut self) -> gpio5::AXW<'_> {
            gpio5::AXW::new(self)
        }
        /// 1 forwards GPIO6's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio6(&mut self) -> gpio6::AXW<'_> {
            gpio6::AXW::new(self)
        }
        /// 1 forwards GPIO7's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio7(&mut self) -> gpio7::AXW<'_> {
            gpio7::AXW::new(self)
        }
        /// 1 forwards GPIO8's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio8(&mut self) -> gpio8::AXW<'_> {
            gpio8::AXW::new(self)
        }
        /// 1 forwards GPIO9's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio9(&mut self) -> gpio9::AXW<'_> {
            gpio9::AXW::new(self)
        }
        /// 1 forwards GPIO10's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio10(&mut self) -> gpio10::AXW<'_> {
            gpio10::AXW::new(self)
        }
        /// 1 forwards GPIO11's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio11(&mut self) -> gpio11::AXW<'_> {
            gpio11::AXW::new(self)
        }
        /// 1 forwards GPIO12's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio12(&mut self) -> gpio12::AXW<'_> {
            gpio12::AXW::new(self)
        }
        /// 1 forwards GPIO13's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio13(&mut self) -> gpio13::AXW<'_> {
            gpio13::AXW::new(self)
        }
        /// 1 forwards GPIO14's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio14(&mut self) -> gpio14::AXW<'_> {
            gpio14::AXW::new(self)
        }
        /// 1 forwards GPIO15's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio15(&mut self) -> gpio15::AXW<'_> {
            gpio15::AXW::new(self)
        }
        /// 1 forwards GPIO16's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio16(&mut self) -> gpio16::AXW<'_> {
            gpio16::AXW::new(self)
        }
        /// 1 forwards GPIO17's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio17(&mut self) -> gpio17::AXW<'_> {
            gpio17::AXW::new(self)
        }
        /// 1 forwards GPIO18's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio18(&mut self) -> gpio18::AXW<'_> {
            gpio18::AXW::new(self)
        }
        /// 1 forwards GPIO19's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio19(&mut self) -> gpio19::AXW<'_> {
            gpio19::AXW::new(self)
        }
        /// 1 forwards GPIO20's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio20(&mut self) -> gpio20::AXW<'_> {
            gpio20::AXW::new(self)
        }
        /// 1 forwards GPIO21's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio21(&mut self) -> gpio21::AXW<'_> {
            gpio21::AXW::new(self)
        }
        /// 1 forwards GPIO22's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio22(&mut self) -> gpio22::AXW<'_> {
            gpio22::AXW::new(self)
        }
        /// 1 forwards GPIO23's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio23(&mut self) -> gpio23::AXW<'_> {
            gpio23::AXW::new(self)
        }
        /// 1 forwards GPIO24's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio24(&mut self) -> gpio24::AXW<'_> {
            gpio24::AXW::new(self)
        }
        /// 1 forwards GPIO25's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio25(&mut self) -> gpio25::AXW<'_> {
            gpio25::AXW::new(self)
        }
        /// 1 forwards GPIO26's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio26(&mut self) -> gpio26::AXW<'_> {
            gpio26::AXW::new(self)
        }
        /// 1 forwards GPIO27's interrupt to RP1 processor 1; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio27(&mut self) -> gpio27::AXW<'_> {
            gpio27::AXW::new(self)
        }
    }
}

/// Per-pin interrupt force for RP1 processor 1. 1 forces the matching status bit.
/// Hardware access: read Plain; write Plain.
pub type PROC1_INTF = Reg<proc1_intf::PROC1_INTF_SPEC>;
/// Per-pin interrupt force for RP1 processor 1. 1 forces the matching status bit.
/// Hardware access: read Plain; write Plain.
pub mod proc1_intf {
    pub struct PROC1_INTF_SPEC;
    impl crate::RegisterSpec for PROC1_INTF_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for PROC1_INTF_SPEC {}
    pub type R = crate::R<PROC1_INTF_SPEC>;
    impl crate::Modifiable for PROC1_INTF_SPEC {}
    impl crate::UnsafeWritable for PROC1_INTF_SPEC {}
    impl crate::ResetValue for PROC1_INTF_SPEC {
        const RESET_VALUE: u32 = 0x0;
    }
    impl crate::TypedAtomicSet for PROC1_INTF_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicSet for PROC1_INTF_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::TypedAtomicClear for PROC1_INTF_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicClear for PROC1_INTF_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::TypedAtomicXor for PROC1_INTF_SPEC {
        const OFFSET: usize = 0x1000;
    }
    impl crate::UnsafeAtomicXor for PROC1_INTF_SPEC {
        const OFFSET: usize = 0x1000;
    }
    pub type W = crate::W<PROC1_INTF_SPEC>;
    pub type AtomicSetW = crate::AtomicSetW<PROC1_INTF_SPEC>;
    pub type AtomicClearW = crate::AtomicClearW<PROC1_INTF_SPEC>;
    pub type AtomicXorW = crate::AtomicXorW<PROC1_INTF_SPEC>;
    /// 1 forces GPIO0's interrupt to RP1 processor 1; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio0 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 0>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 0>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 0>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 0>;
    }
    /// 1 forces GPIO1's interrupt to RP1 processor 1; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio1 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 1>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 1>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 1>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 1>;
    }
    /// 1 forces GPIO2's interrupt to RP1 processor 1; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio2 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 2>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 2>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 2>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 2>;
    }
    /// 1 forces GPIO3's interrupt to RP1 processor 1; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio3 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 3>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 3>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 3>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 3>;
    }
    /// 1 forces GPIO4's interrupt to RP1 processor 1; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio4 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 4>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 4>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 4>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 4>;
    }
    /// 1 forces GPIO5's interrupt to RP1 processor 1; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio5 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 5>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 5>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 5>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 5>;
    }
    /// 1 forces GPIO6's interrupt to RP1 processor 1; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio6 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 6>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 6>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 6>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 6>;
    }
    /// 1 forces GPIO7's interrupt to RP1 processor 1; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio7 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 7>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 7>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 7>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 7>;
    }
    /// 1 forces GPIO8's interrupt to RP1 processor 1; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio8 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 8>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 8>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 8>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 8>;
    }
    /// 1 forces GPIO9's interrupt to RP1 processor 1; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio9 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 9>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 9>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 9>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 9>;
    }
    /// 1 forces GPIO10's interrupt to RP1 processor 1; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio10 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 10>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 10>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 10>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 10>;
    }
    /// 1 forces GPIO11's interrupt to RP1 processor 1; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio11 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 11>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 11>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 11>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 11>;
    }
    /// 1 forces GPIO12's interrupt to RP1 processor 1; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio12 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 12>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 12>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 12>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 12>;
    }
    /// 1 forces GPIO13's interrupt to RP1 processor 1; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio13 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 13>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 13>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 13>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 13>;
    }
    /// 1 forces GPIO14's interrupt to RP1 processor 1; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio14 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 14>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 14>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 14>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 14>;
    }
    /// 1 forces GPIO15's interrupt to RP1 processor 1; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio15 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 15>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 15>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 15>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 15>;
    }
    /// 1 forces GPIO16's interrupt to RP1 processor 1; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio16 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 16>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 16>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 16>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 16>;
    }
    /// 1 forces GPIO17's interrupt to RP1 processor 1; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio17 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 17>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 17>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 17>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 17>;
    }
    /// 1 forces GPIO18's interrupt to RP1 processor 1; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio18 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 18>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 18>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 18>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 18>;
    }
    /// 1 forces GPIO19's interrupt to RP1 processor 1; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio19 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 19>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 19>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 19>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 19>;
    }
    /// 1 forces GPIO20's interrupt to RP1 processor 1; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio20 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 20>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 20>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 20>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 20>;
    }
    /// 1 forces GPIO21's interrupt to RP1 processor 1; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio21 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 21>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 21>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 21>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 21>;
    }
    /// 1 forces GPIO22's interrupt to RP1 processor 1; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio22 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 22>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 22>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 22>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 22>;
    }
    /// 1 forces GPIO23's interrupt to RP1 processor 1; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio23 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 23>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 23>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 23>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 23>;
    }
    /// 1 forces GPIO24's interrupt to RP1 processor 1; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio24 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 24>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 24>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 24>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 24>;
    }
    /// 1 forces GPIO25's interrupt to RP1 processor 1; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio25 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 25>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 25>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 25>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 25>;
    }
    /// 1 forces GPIO26's interrupt to RP1 processor 1; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio26 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 26>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 26>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 26>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 26>;
    }
    /// 1 forces GPIO27's interrupt to RP1 processor 1; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio27 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 27>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 27>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 27>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 27>;
    }
    impl R {
        #[inline(always)]
        pub fn gpio0(&self) -> gpio0::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio1(&self) -> gpio1::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio2(&self) -> gpio2::R {
            crate::BitReader::new(((self.bits() >> 2) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio3(&self) -> gpio3::R {
            crate::BitReader::new(((self.bits() >> 3) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio4(&self) -> gpio4::R {
            crate::BitReader::new(((self.bits() >> 4) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio5(&self) -> gpio5::R {
            crate::BitReader::new(((self.bits() >> 5) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio6(&self) -> gpio6::R {
            crate::BitReader::new(((self.bits() >> 6) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio7(&self) -> gpio7::R {
            crate::BitReader::new(((self.bits() >> 7) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio8(&self) -> gpio8::R {
            crate::BitReader::new(((self.bits() >> 8) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio9(&self) -> gpio9::R {
            crate::BitReader::new(((self.bits() >> 9) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio10(&self) -> gpio10::R {
            crate::BitReader::new(((self.bits() >> 10) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio11(&self) -> gpio11::R {
            crate::BitReader::new(((self.bits() >> 11) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio12(&self) -> gpio12::R {
            crate::BitReader::new(((self.bits() >> 12) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio13(&self) -> gpio13::R {
            crate::BitReader::new(((self.bits() >> 13) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio14(&self) -> gpio14::R {
            crate::BitReader::new(((self.bits() >> 14) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio15(&self) -> gpio15::R {
            crate::BitReader::new(((self.bits() >> 15) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio16(&self) -> gpio16::R {
            crate::BitReader::new(((self.bits() >> 16) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio17(&self) -> gpio17::R {
            crate::BitReader::new(((self.bits() >> 17) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio18(&self) -> gpio18::R {
            crate::BitReader::new(((self.bits() >> 18) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio19(&self) -> gpio19::R {
            crate::BitReader::new(((self.bits() >> 19) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio20(&self) -> gpio20::R {
            crate::BitReader::new(((self.bits() >> 20) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio21(&self) -> gpio21::R {
            crate::BitReader::new(((self.bits() >> 21) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio22(&self) -> gpio22::R {
            crate::BitReader::new(((self.bits() >> 22) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio23(&self) -> gpio23::R {
            crate::BitReader::new(((self.bits() >> 23) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio24(&self) -> gpio24::R {
            crate::BitReader::new(((self.bits() >> 24) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio25(&self) -> gpio25::R {
            crate::BitReader::new(((self.bits() >> 25) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio26(&self) -> gpio26::R {
            crate::BitReader::new(((self.bits() >> 26) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio27(&self) -> gpio27::R {
            crate::BitReader::new(((self.bits() >> 27) & 1) != 0)
        }
    }
    impl W {
        #[inline(always)]
        pub fn gpio0(&mut self) -> gpio0::W<'_> {
            gpio0::W::new(self)
        }
        #[inline(always)]
        pub fn gpio1(&mut self) -> gpio1::W<'_> {
            gpio1::W::new(self)
        }
        #[inline(always)]
        pub fn gpio2(&mut self) -> gpio2::W<'_> {
            gpio2::W::new(self)
        }
        #[inline(always)]
        pub fn gpio3(&mut self) -> gpio3::W<'_> {
            gpio3::W::new(self)
        }
        #[inline(always)]
        pub fn gpio4(&mut self) -> gpio4::W<'_> {
            gpio4::W::new(self)
        }
        #[inline(always)]
        pub fn gpio5(&mut self) -> gpio5::W<'_> {
            gpio5::W::new(self)
        }
        #[inline(always)]
        pub fn gpio6(&mut self) -> gpio6::W<'_> {
            gpio6::W::new(self)
        }
        #[inline(always)]
        pub fn gpio7(&mut self) -> gpio7::W<'_> {
            gpio7::W::new(self)
        }
        #[inline(always)]
        pub fn gpio8(&mut self) -> gpio8::W<'_> {
            gpio8::W::new(self)
        }
        #[inline(always)]
        pub fn gpio9(&mut self) -> gpio9::W<'_> {
            gpio9::W::new(self)
        }
        #[inline(always)]
        pub fn gpio10(&mut self) -> gpio10::W<'_> {
            gpio10::W::new(self)
        }
        #[inline(always)]
        pub fn gpio11(&mut self) -> gpio11::W<'_> {
            gpio11::W::new(self)
        }
        #[inline(always)]
        pub fn gpio12(&mut self) -> gpio12::W<'_> {
            gpio12::W::new(self)
        }
        #[inline(always)]
        pub fn gpio13(&mut self) -> gpio13::W<'_> {
            gpio13::W::new(self)
        }
        #[inline(always)]
        pub fn gpio14(&mut self) -> gpio14::W<'_> {
            gpio14::W::new(self)
        }
        #[inline(always)]
        pub fn gpio15(&mut self) -> gpio15::W<'_> {
            gpio15::W::new(self)
        }
        #[inline(always)]
        pub fn gpio16(&mut self) -> gpio16::W<'_> {
            gpio16::W::new(self)
        }
        #[inline(always)]
        pub fn gpio17(&mut self) -> gpio17::W<'_> {
            gpio17::W::new(self)
        }
        #[inline(always)]
        pub fn gpio18(&mut self) -> gpio18::W<'_> {
            gpio18::W::new(self)
        }
        #[inline(always)]
        pub fn gpio19(&mut self) -> gpio19::W<'_> {
            gpio19::W::new(self)
        }
        #[inline(always)]
        pub fn gpio20(&mut self) -> gpio20::W<'_> {
            gpio20::W::new(self)
        }
        #[inline(always)]
        pub fn gpio21(&mut self) -> gpio21::W<'_> {
            gpio21::W::new(self)
        }
        #[inline(always)]
        pub fn gpio22(&mut self) -> gpio22::W<'_> {
            gpio22::W::new(self)
        }
        #[inline(always)]
        pub fn gpio23(&mut self) -> gpio23::W<'_> {
            gpio23::W::new(self)
        }
        #[inline(always)]
        pub fn gpio24(&mut self) -> gpio24::W<'_> {
            gpio24::W::new(self)
        }
        #[inline(always)]
        pub fn gpio25(&mut self) -> gpio25::W<'_> {
            gpio25::W::new(self)
        }
        #[inline(always)]
        pub fn gpio26(&mut self) -> gpio26::W<'_> {
            gpio26::W::new(self)
        }
        #[inline(always)]
        pub fn gpio27(&mut self) -> gpio27::W<'_> {
            gpio27::W::new(self)
        }
    }
    impl AtomicSetW {
        /// 1 forces GPIO0's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio0(&mut self) -> gpio0::ASW<'_> {
            gpio0::ASW::new(self)
        }
        /// 1 forces GPIO1's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio1(&mut self) -> gpio1::ASW<'_> {
            gpio1::ASW::new(self)
        }
        /// 1 forces GPIO2's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio2(&mut self) -> gpio2::ASW<'_> {
            gpio2::ASW::new(self)
        }
        /// 1 forces GPIO3's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio3(&mut self) -> gpio3::ASW<'_> {
            gpio3::ASW::new(self)
        }
        /// 1 forces GPIO4's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio4(&mut self) -> gpio4::ASW<'_> {
            gpio4::ASW::new(self)
        }
        /// 1 forces GPIO5's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio5(&mut self) -> gpio5::ASW<'_> {
            gpio5::ASW::new(self)
        }
        /// 1 forces GPIO6's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio6(&mut self) -> gpio6::ASW<'_> {
            gpio6::ASW::new(self)
        }
        /// 1 forces GPIO7's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio7(&mut self) -> gpio7::ASW<'_> {
            gpio7::ASW::new(self)
        }
        /// 1 forces GPIO8's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio8(&mut self) -> gpio8::ASW<'_> {
            gpio8::ASW::new(self)
        }
        /// 1 forces GPIO9's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio9(&mut self) -> gpio9::ASW<'_> {
            gpio9::ASW::new(self)
        }
        /// 1 forces GPIO10's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio10(&mut self) -> gpio10::ASW<'_> {
            gpio10::ASW::new(self)
        }
        /// 1 forces GPIO11's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio11(&mut self) -> gpio11::ASW<'_> {
            gpio11::ASW::new(self)
        }
        /// 1 forces GPIO12's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio12(&mut self) -> gpio12::ASW<'_> {
            gpio12::ASW::new(self)
        }
        /// 1 forces GPIO13's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio13(&mut self) -> gpio13::ASW<'_> {
            gpio13::ASW::new(self)
        }
        /// 1 forces GPIO14's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio14(&mut self) -> gpio14::ASW<'_> {
            gpio14::ASW::new(self)
        }
        /// 1 forces GPIO15's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio15(&mut self) -> gpio15::ASW<'_> {
            gpio15::ASW::new(self)
        }
        /// 1 forces GPIO16's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio16(&mut self) -> gpio16::ASW<'_> {
            gpio16::ASW::new(self)
        }
        /// 1 forces GPIO17's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio17(&mut self) -> gpio17::ASW<'_> {
            gpio17::ASW::new(self)
        }
        /// 1 forces GPIO18's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio18(&mut self) -> gpio18::ASW<'_> {
            gpio18::ASW::new(self)
        }
        /// 1 forces GPIO19's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio19(&mut self) -> gpio19::ASW<'_> {
            gpio19::ASW::new(self)
        }
        /// 1 forces GPIO20's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio20(&mut self) -> gpio20::ASW<'_> {
            gpio20::ASW::new(self)
        }
        /// 1 forces GPIO21's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio21(&mut self) -> gpio21::ASW<'_> {
            gpio21::ASW::new(self)
        }
        /// 1 forces GPIO22's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio22(&mut self) -> gpio22::ASW<'_> {
            gpio22::ASW::new(self)
        }
        /// 1 forces GPIO23's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio23(&mut self) -> gpio23::ASW<'_> {
            gpio23::ASW::new(self)
        }
        /// 1 forces GPIO24's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio24(&mut self) -> gpio24::ASW<'_> {
            gpio24::ASW::new(self)
        }
        /// 1 forces GPIO25's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio25(&mut self) -> gpio25::ASW<'_> {
            gpio25::ASW::new(self)
        }
        /// 1 forces GPIO26's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio26(&mut self) -> gpio26::ASW<'_> {
            gpio26::ASW::new(self)
        }
        /// 1 forces GPIO27's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio27(&mut self) -> gpio27::ASW<'_> {
            gpio27::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// 1 forces GPIO0's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio0(&mut self) -> gpio0::ACW<'_> {
            gpio0::ACW::new(self)
        }
        /// 1 forces GPIO1's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio1(&mut self) -> gpio1::ACW<'_> {
            gpio1::ACW::new(self)
        }
        /// 1 forces GPIO2's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio2(&mut self) -> gpio2::ACW<'_> {
            gpio2::ACW::new(self)
        }
        /// 1 forces GPIO3's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio3(&mut self) -> gpio3::ACW<'_> {
            gpio3::ACW::new(self)
        }
        /// 1 forces GPIO4's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio4(&mut self) -> gpio4::ACW<'_> {
            gpio4::ACW::new(self)
        }
        /// 1 forces GPIO5's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio5(&mut self) -> gpio5::ACW<'_> {
            gpio5::ACW::new(self)
        }
        /// 1 forces GPIO6's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio6(&mut self) -> gpio6::ACW<'_> {
            gpio6::ACW::new(self)
        }
        /// 1 forces GPIO7's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio7(&mut self) -> gpio7::ACW<'_> {
            gpio7::ACW::new(self)
        }
        /// 1 forces GPIO8's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio8(&mut self) -> gpio8::ACW<'_> {
            gpio8::ACW::new(self)
        }
        /// 1 forces GPIO9's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio9(&mut self) -> gpio9::ACW<'_> {
            gpio9::ACW::new(self)
        }
        /// 1 forces GPIO10's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio10(&mut self) -> gpio10::ACW<'_> {
            gpio10::ACW::new(self)
        }
        /// 1 forces GPIO11's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio11(&mut self) -> gpio11::ACW<'_> {
            gpio11::ACW::new(self)
        }
        /// 1 forces GPIO12's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio12(&mut self) -> gpio12::ACW<'_> {
            gpio12::ACW::new(self)
        }
        /// 1 forces GPIO13's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio13(&mut self) -> gpio13::ACW<'_> {
            gpio13::ACW::new(self)
        }
        /// 1 forces GPIO14's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio14(&mut self) -> gpio14::ACW<'_> {
            gpio14::ACW::new(self)
        }
        /// 1 forces GPIO15's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio15(&mut self) -> gpio15::ACW<'_> {
            gpio15::ACW::new(self)
        }
        /// 1 forces GPIO16's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio16(&mut self) -> gpio16::ACW<'_> {
            gpio16::ACW::new(self)
        }
        /// 1 forces GPIO17's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio17(&mut self) -> gpio17::ACW<'_> {
            gpio17::ACW::new(self)
        }
        /// 1 forces GPIO18's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio18(&mut self) -> gpio18::ACW<'_> {
            gpio18::ACW::new(self)
        }
        /// 1 forces GPIO19's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio19(&mut self) -> gpio19::ACW<'_> {
            gpio19::ACW::new(self)
        }
        /// 1 forces GPIO20's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio20(&mut self) -> gpio20::ACW<'_> {
            gpio20::ACW::new(self)
        }
        /// 1 forces GPIO21's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio21(&mut self) -> gpio21::ACW<'_> {
            gpio21::ACW::new(self)
        }
        /// 1 forces GPIO22's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio22(&mut self) -> gpio22::ACW<'_> {
            gpio22::ACW::new(self)
        }
        /// 1 forces GPIO23's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio23(&mut self) -> gpio23::ACW<'_> {
            gpio23::ACW::new(self)
        }
        /// 1 forces GPIO24's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio24(&mut self) -> gpio24::ACW<'_> {
            gpio24::ACW::new(self)
        }
        /// 1 forces GPIO25's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio25(&mut self) -> gpio25::ACW<'_> {
            gpio25::ACW::new(self)
        }
        /// 1 forces GPIO26's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio26(&mut self) -> gpio26::ACW<'_> {
            gpio26::ACW::new(self)
        }
        /// 1 forces GPIO27's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio27(&mut self) -> gpio27::ACW<'_> {
            gpio27::ACW::new(self)
        }
    }
    impl AtomicXorW {
        /// 1 forces GPIO0's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio0(&mut self) -> gpio0::AXW<'_> {
            gpio0::AXW::new(self)
        }
        /// 1 forces GPIO1's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio1(&mut self) -> gpio1::AXW<'_> {
            gpio1::AXW::new(self)
        }
        /// 1 forces GPIO2's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio2(&mut self) -> gpio2::AXW<'_> {
            gpio2::AXW::new(self)
        }
        /// 1 forces GPIO3's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio3(&mut self) -> gpio3::AXW<'_> {
            gpio3::AXW::new(self)
        }
        /// 1 forces GPIO4's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio4(&mut self) -> gpio4::AXW<'_> {
            gpio4::AXW::new(self)
        }
        /// 1 forces GPIO5's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio5(&mut self) -> gpio5::AXW<'_> {
            gpio5::AXW::new(self)
        }
        /// 1 forces GPIO6's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio6(&mut self) -> gpio6::AXW<'_> {
            gpio6::AXW::new(self)
        }
        /// 1 forces GPIO7's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio7(&mut self) -> gpio7::AXW<'_> {
            gpio7::AXW::new(self)
        }
        /// 1 forces GPIO8's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio8(&mut self) -> gpio8::AXW<'_> {
            gpio8::AXW::new(self)
        }
        /// 1 forces GPIO9's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio9(&mut self) -> gpio9::AXW<'_> {
            gpio9::AXW::new(self)
        }
        /// 1 forces GPIO10's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio10(&mut self) -> gpio10::AXW<'_> {
            gpio10::AXW::new(self)
        }
        /// 1 forces GPIO11's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio11(&mut self) -> gpio11::AXW<'_> {
            gpio11::AXW::new(self)
        }
        /// 1 forces GPIO12's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio12(&mut self) -> gpio12::AXW<'_> {
            gpio12::AXW::new(self)
        }
        /// 1 forces GPIO13's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio13(&mut self) -> gpio13::AXW<'_> {
            gpio13::AXW::new(self)
        }
        /// 1 forces GPIO14's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio14(&mut self) -> gpio14::AXW<'_> {
            gpio14::AXW::new(self)
        }
        /// 1 forces GPIO15's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio15(&mut self) -> gpio15::AXW<'_> {
            gpio15::AXW::new(self)
        }
        /// 1 forces GPIO16's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio16(&mut self) -> gpio16::AXW<'_> {
            gpio16::AXW::new(self)
        }
        /// 1 forces GPIO17's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio17(&mut self) -> gpio17::AXW<'_> {
            gpio17::AXW::new(self)
        }
        /// 1 forces GPIO18's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio18(&mut self) -> gpio18::AXW<'_> {
            gpio18::AXW::new(self)
        }
        /// 1 forces GPIO19's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio19(&mut self) -> gpio19::AXW<'_> {
            gpio19::AXW::new(self)
        }
        /// 1 forces GPIO20's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio20(&mut self) -> gpio20::AXW<'_> {
            gpio20::AXW::new(self)
        }
        /// 1 forces GPIO21's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio21(&mut self) -> gpio21::AXW<'_> {
            gpio21::AXW::new(self)
        }
        /// 1 forces GPIO22's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio22(&mut self) -> gpio22::AXW<'_> {
            gpio22::AXW::new(self)
        }
        /// 1 forces GPIO23's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio23(&mut self) -> gpio23::AXW<'_> {
            gpio23::AXW::new(self)
        }
        /// 1 forces GPIO24's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio24(&mut self) -> gpio24::AXW<'_> {
            gpio24::AXW::new(self)
        }
        /// 1 forces GPIO25's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio25(&mut self) -> gpio25::AXW<'_> {
            gpio25::AXW::new(self)
        }
        /// 1 forces GPIO26's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio26(&mut self) -> gpio26::AXW<'_> {
            gpio26::AXW::new(self)
        }
        /// 1 forces GPIO27's interrupt to RP1 processor 1; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio27(&mut self) -> gpio27::AXW<'_> {
            gpio27::AXW::new(self)
        }
    }
}

/// Per-pin interrupt status seen by RP1 processor 1: the enabled raw interrupt combined with the force bit.
/// Hardware access: read Plain; write None.
pub type PROC1_INTS = Reg<proc1_ints::PROC1_INTS_SPEC>;
/// Per-pin interrupt status seen by RP1 processor 1: the enabled raw interrupt combined with the force bit.
/// Hardware access: read Plain; write None.
pub mod proc1_ints {
    pub struct PROC1_INTS_SPEC;
    impl crate::RegisterSpec for PROC1_INTS_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for PROC1_INTS_SPEC {}
    pub type R = crate::R<PROC1_INTS_SPEC>;
    impl crate::ResetValue for PROC1_INTS_SPEC {
        const RESET_VALUE: u32 = 0x0;
    }
    /// 1 if GPIO0 has an effective interrupt for RP1 processor 1.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio0 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO1 has an effective interrupt for RP1 processor 1.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio1 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO2 has an effective interrupt for RP1 processor 1.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio2 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO3 has an effective interrupt for RP1 processor 1.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio3 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO4 has an effective interrupt for RP1 processor 1.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio4 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO5 has an effective interrupt for RP1 processor 1.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio5 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO6 has an effective interrupt for RP1 processor 1.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio6 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO7 has an effective interrupt for RP1 processor 1.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio7 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO8 has an effective interrupt for RP1 processor 1.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio8 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO9 has an effective interrupt for RP1 processor 1.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio9 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO10 has an effective interrupt for RP1 processor 1.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio10 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO11 has an effective interrupt for RP1 processor 1.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio11 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO12 has an effective interrupt for RP1 processor 1.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio12 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO13 has an effective interrupt for RP1 processor 1.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio13 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO14 has an effective interrupt for RP1 processor 1.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio14 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO15 has an effective interrupt for RP1 processor 1.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio15 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO16 has an effective interrupt for RP1 processor 1.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio16 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO17 has an effective interrupt for RP1 processor 1.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio17 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO18 has an effective interrupt for RP1 processor 1.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio18 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO19 has an effective interrupt for RP1 processor 1.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio19 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO20 has an effective interrupt for RP1 processor 1.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio20 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO21 has an effective interrupt for RP1 processor 1.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio21 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO22 has an effective interrupt for RP1 processor 1.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio22 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO23 has an effective interrupt for RP1 processor 1.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio23 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO24 has an effective interrupt for RP1 processor 1.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio24 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO25 has an effective interrupt for RP1 processor 1.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio25 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO26 has an effective interrupt for RP1 processor 1.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio26 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO27 has an effective interrupt for RP1 processor 1.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio27 {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn gpio0(&self) -> gpio0::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio1(&self) -> gpio1::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio2(&self) -> gpio2::R {
            crate::BitReader::new(((self.bits() >> 2) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio3(&self) -> gpio3::R {
            crate::BitReader::new(((self.bits() >> 3) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio4(&self) -> gpio4::R {
            crate::BitReader::new(((self.bits() >> 4) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio5(&self) -> gpio5::R {
            crate::BitReader::new(((self.bits() >> 5) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio6(&self) -> gpio6::R {
            crate::BitReader::new(((self.bits() >> 6) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio7(&self) -> gpio7::R {
            crate::BitReader::new(((self.bits() >> 7) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio8(&self) -> gpio8::R {
            crate::BitReader::new(((self.bits() >> 8) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio9(&self) -> gpio9::R {
            crate::BitReader::new(((self.bits() >> 9) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio10(&self) -> gpio10::R {
            crate::BitReader::new(((self.bits() >> 10) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio11(&self) -> gpio11::R {
            crate::BitReader::new(((self.bits() >> 11) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio12(&self) -> gpio12::R {
            crate::BitReader::new(((self.bits() >> 12) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio13(&self) -> gpio13::R {
            crate::BitReader::new(((self.bits() >> 13) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio14(&self) -> gpio14::R {
            crate::BitReader::new(((self.bits() >> 14) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio15(&self) -> gpio15::R {
            crate::BitReader::new(((self.bits() >> 15) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio16(&self) -> gpio16::R {
            crate::BitReader::new(((self.bits() >> 16) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio17(&self) -> gpio17::R {
            crate::BitReader::new(((self.bits() >> 17) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio18(&self) -> gpio18::R {
            crate::BitReader::new(((self.bits() >> 18) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio19(&self) -> gpio19::R {
            crate::BitReader::new(((self.bits() >> 19) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio20(&self) -> gpio20::R {
            crate::BitReader::new(((self.bits() >> 20) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio21(&self) -> gpio21::R {
            crate::BitReader::new(((self.bits() >> 21) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio22(&self) -> gpio22::R {
            crate::BitReader::new(((self.bits() >> 22) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio23(&self) -> gpio23::R {
            crate::BitReader::new(((self.bits() >> 23) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio24(&self) -> gpio24::R {
            crate::BitReader::new(((self.bits() >> 24) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio25(&self) -> gpio25::R {
            crate::BitReader::new(((self.bits() >> 25) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio26(&self) -> gpio26::R {
            crate::BitReader::new(((self.bits() >> 26) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio27(&self) -> gpio27::R {
            crate::BitReader::new(((self.bits() >> 27) & 1) != 0)
        }
    }
}

/// Per-pin interrupt enables for the PCIe host. 1 lets the pin's interrupt through.
/// Hardware access: read Plain; write Plain.
pub type PCIE_INTE = Reg<pcie_inte::PCIE_INTE_SPEC>;
/// Per-pin interrupt enables for the PCIe host. 1 lets the pin's interrupt through.
/// Hardware access: read Plain; write Plain.
pub mod pcie_inte {
    pub struct PCIE_INTE_SPEC;
    impl crate::RegisterSpec for PCIE_INTE_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for PCIE_INTE_SPEC {}
    pub type R = crate::R<PCIE_INTE_SPEC>;
    impl crate::Modifiable for PCIE_INTE_SPEC {}
    impl crate::UnsafeWritable for PCIE_INTE_SPEC {}
    impl crate::ResetValue for PCIE_INTE_SPEC {
        const RESET_VALUE: u32 = 0x0;
    }
    impl crate::TypedAtomicSet for PCIE_INTE_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicSet for PCIE_INTE_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::TypedAtomicClear for PCIE_INTE_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicClear for PCIE_INTE_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::TypedAtomicXor for PCIE_INTE_SPEC {
        const OFFSET: usize = 0x1000;
    }
    impl crate::UnsafeAtomicXor for PCIE_INTE_SPEC {
        const OFFSET: usize = 0x1000;
    }
    pub type W = crate::W<PCIE_INTE_SPEC>;
    pub type AtomicSetW = crate::AtomicSetW<PCIE_INTE_SPEC>;
    pub type AtomicClearW = crate::AtomicClearW<PCIE_INTE_SPEC>;
    pub type AtomicXorW = crate::AtomicXorW<PCIE_INTE_SPEC>;
    /// 1 forwards GPIO0's interrupt to the PCIe host; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio0 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 0>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 0>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 0>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 0>;
    }
    /// 1 forwards GPIO1's interrupt to the PCIe host; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio1 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 1>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 1>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 1>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 1>;
    }
    /// 1 forwards GPIO2's interrupt to the PCIe host; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio2 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 2>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 2>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 2>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 2>;
    }
    /// 1 forwards GPIO3's interrupt to the PCIe host; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio3 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 3>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 3>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 3>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 3>;
    }
    /// 1 forwards GPIO4's interrupt to the PCIe host; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio4 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 4>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 4>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 4>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 4>;
    }
    /// 1 forwards GPIO5's interrupt to the PCIe host; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio5 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 5>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 5>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 5>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 5>;
    }
    /// 1 forwards GPIO6's interrupt to the PCIe host; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio6 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 6>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 6>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 6>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 6>;
    }
    /// 1 forwards GPIO7's interrupt to the PCIe host; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio7 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 7>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 7>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 7>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 7>;
    }
    /// 1 forwards GPIO8's interrupt to the PCIe host; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio8 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 8>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 8>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 8>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 8>;
    }
    /// 1 forwards GPIO9's interrupt to the PCIe host; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio9 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 9>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 9>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 9>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 9>;
    }
    /// 1 forwards GPIO10's interrupt to the PCIe host; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio10 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 10>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 10>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 10>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 10>;
    }
    /// 1 forwards GPIO11's interrupt to the PCIe host; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio11 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 11>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 11>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 11>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 11>;
    }
    /// 1 forwards GPIO12's interrupt to the PCIe host; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio12 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 12>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 12>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 12>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 12>;
    }
    /// 1 forwards GPIO13's interrupt to the PCIe host; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio13 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 13>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 13>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 13>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 13>;
    }
    /// 1 forwards GPIO14's interrupt to the PCIe host; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio14 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 14>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 14>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 14>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 14>;
    }
    /// 1 forwards GPIO15's interrupt to the PCIe host; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio15 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 15>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 15>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 15>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 15>;
    }
    /// 1 forwards GPIO16's interrupt to the PCIe host; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio16 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 16>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 16>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 16>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 16>;
    }
    /// 1 forwards GPIO17's interrupt to the PCIe host; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio17 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 17>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 17>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 17>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 17>;
    }
    /// 1 forwards GPIO18's interrupt to the PCIe host; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio18 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 18>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 18>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 18>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 18>;
    }
    /// 1 forwards GPIO19's interrupt to the PCIe host; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio19 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 19>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 19>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 19>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 19>;
    }
    /// 1 forwards GPIO20's interrupt to the PCIe host; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio20 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 20>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 20>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 20>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 20>;
    }
    /// 1 forwards GPIO21's interrupt to the PCIe host; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio21 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 21>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 21>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 21>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 21>;
    }
    /// 1 forwards GPIO22's interrupt to the PCIe host; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio22 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 22>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 22>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 22>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 22>;
    }
    /// 1 forwards GPIO23's interrupt to the PCIe host; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio23 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 23>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 23>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 23>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 23>;
    }
    /// 1 forwards GPIO24's interrupt to the PCIe host; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio24 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 24>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 24>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 24>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 24>;
    }
    /// 1 forwards GPIO25's interrupt to the PCIe host; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio25 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 25>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 25>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 25>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 25>;
    }
    /// 1 forwards GPIO26's interrupt to the PCIe host; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio26 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 26>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 26>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 26>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 26>;
    }
    /// 1 forwards GPIO27's interrupt to the PCIe host; 0 blocks it.
    /// Field write behavior: Plain.
    pub mod gpio27 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 27>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 27>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 27>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 27>;
    }
    impl R {
        #[inline(always)]
        pub fn gpio0(&self) -> gpio0::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio1(&self) -> gpio1::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio2(&self) -> gpio2::R {
            crate::BitReader::new(((self.bits() >> 2) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio3(&self) -> gpio3::R {
            crate::BitReader::new(((self.bits() >> 3) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio4(&self) -> gpio4::R {
            crate::BitReader::new(((self.bits() >> 4) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio5(&self) -> gpio5::R {
            crate::BitReader::new(((self.bits() >> 5) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio6(&self) -> gpio6::R {
            crate::BitReader::new(((self.bits() >> 6) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio7(&self) -> gpio7::R {
            crate::BitReader::new(((self.bits() >> 7) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio8(&self) -> gpio8::R {
            crate::BitReader::new(((self.bits() >> 8) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio9(&self) -> gpio9::R {
            crate::BitReader::new(((self.bits() >> 9) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio10(&self) -> gpio10::R {
            crate::BitReader::new(((self.bits() >> 10) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio11(&self) -> gpio11::R {
            crate::BitReader::new(((self.bits() >> 11) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio12(&self) -> gpio12::R {
            crate::BitReader::new(((self.bits() >> 12) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio13(&self) -> gpio13::R {
            crate::BitReader::new(((self.bits() >> 13) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio14(&self) -> gpio14::R {
            crate::BitReader::new(((self.bits() >> 14) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio15(&self) -> gpio15::R {
            crate::BitReader::new(((self.bits() >> 15) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio16(&self) -> gpio16::R {
            crate::BitReader::new(((self.bits() >> 16) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio17(&self) -> gpio17::R {
            crate::BitReader::new(((self.bits() >> 17) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio18(&self) -> gpio18::R {
            crate::BitReader::new(((self.bits() >> 18) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio19(&self) -> gpio19::R {
            crate::BitReader::new(((self.bits() >> 19) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio20(&self) -> gpio20::R {
            crate::BitReader::new(((self.bits() >> 20) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio21(&self) -> gpio21::R {
            crate::BitReader::new(((self.bits() >> 21) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio22(&self) -> gpio22::R {
            crate::BitReader::new(((self.bits() >> 22) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio23(&self) -> gpio23::R {
            crate::BitReader::new(((self.bits() >> 23) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio24(&self) -> gpio24::R {
            crate::BitReader::new(((self.bits() >> 24) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio25(&self) -> gpio25::R {
            crate::BitReader::new(((self.bits() >> 25) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio26(&self) -> gpio26::R {
            crate::BitReader::new(((self.bits() >> 26) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio27(&self) -> gpio27::R {
            crate::BitReader::new(((self.bits() >> 27) & 1) != 0)
        }
    }
    impl W {
        #[inline(always)]
        pub fn gpio0(&mut self) -> gpio0::W<'_> {
            gpio0::W::new(self)
        }
        #[inline(always)]
        pub fn gpio1(&mut self) -> gpio1::W<'_> {
            gpio1::W::new(self)
        }
        #[inline(always)]
        pub fn gpio2(&mut self) -> gpio2::W<'_> {
            gpio2::W::new(self)
        }
        #[inline(always)]
        pub fn gpio3(&mut self) -> gpio3::W<'_> {
            gpio3::W::new(self)
        }
        #[inline(always)]
        pub fn gpio4(&mut self) -> gpio4::W<'_> {
            gpio4::W::new(self)
        }
        #[inline(always)]
        pub fn gpio5(&mut self) -> gpio5::W<'_> {
            gpio5::W::new(self)
        }
        #[inline(always)]
        pub fn gpio6(&mut self) -> gpio6::W<'_> {
            gpio6::W::new(self)
        }
        #[inline(always)]
        pub fn gpio7(&mut self) -> gpio7::W<'_> {
            gpio7::W::new(self)
        }
        #[inline(always)]
        pub fn gpio8(&mut self) -> gpio8::W<'_> {
            gpio8::W::new(self)
        }
        #[inline(always)]
        pub fn gpio9(&mut self) -> gpio9::W<'_> {
            gpio9::W::new(self)
        }
        #[inline(always)]
        pub fn gpio10(&mut self) -> gpio10::W<'_> {
            gpio10::W::new(self)
        }
        #[inline(always)]
        pub fn gpio11(&mut self) -> gpio11::W<'_> {
            gpio11::W::new(self)
        }
        #[inline(always)]
        pub fn gpio12(&mut self) -> gpio12::W<'_> {
            gpio12::W::new(self)
        }
        #[inline(always)]
        pub fn gpio13(&mut self) -> gpio13::W<'_> {
            gpio13::W::new(self)
        }
        #[inline(always)]
        pub fn gpio14(&mut self) -> gpio14::W<'_> {
            gpio14::W::new(self)
        }
        #[inline(always)]
        pub fn gpio15(&mut self) -> gpio15::W<'_> {
            gpio15::W::new(self)
        }
        #[inline(always)]
        pub fn gpio16(&mut self) -> gpio16::W<'_> {
            gpio16::W::new(self)
        }
        #[inline(always)]
        pub fn gpio17(&mut self) -> gpio17::W<'_> {
            gpio17::W::new(self)
        }
        #[inline(always)]
        pub fn gpio18(&mut self) -> gpio18::W<'_> {
            gpio18::W::new(self)
        }
        #[inline(always)]
        pub fn gpio19(&mut self) -> gpio19::W<'_> {
            gpio19::W::new(self)
        }
        #[inline(always)]
        pub fn gpio20(&mut self) -> gpio20::W<'_> {
            gpio20::W::new(self)
        }
        #[inline(always)]
        pub fn gpio21(&mut self) -> gpio21::W<'_> {
            gpio21::W::new(self)
        }
        #[inline(always)]
        pub fn gpio22(&mut self) -> gpio22::W<'_> {
            gpio22::W::new(self)
        }
        #[inline(always)]
        pub fn gpio23(&mut self) -> gpio23::W<'_> {
            gpio23::W::new(self)
        }
        #[inline(always)]
        pub fn gpio24(&mut self) -> gpio24::W<'_> {
            gpio24::W::new(self)
        }
        #[inline(always)]
        pub fn gpio25(&mut self) -> gpio25::W<'_> {
            gpio25::W::new(self)
        }
        #[inline(always)]
        pub fn gpio26(&mut self) -> gpio26::W<'_> {
            gpio26::W::new(self)
        }
        #[inline(always)]
        pub fn gpio27(&mut self) -> gpio27::W<'_> {
            gpio27::W::new(self)
        }
    }
    impl AtomicSetW {
        /// 1 forwards GPIO0's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio0(&mut self) -> gpio0::ASW<'_> {
            gpio0::ASW::new(self)
        }
        /// 1 forwards GPIO1's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio1(&mut self) -> gpio1::ASW<'_> {
            gpio1::ASW::new(self)
        }
        /// 1 forwards GPIO2's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio2(&mut self) -> gpio2::ASW<'_> {
            gpio2::ASW::new(self)
        }
        /// 1 forwards GPIO3's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio3(&mut self) -> gpio3::ASW<'_> {
            gpio3::ASW::new(self)
        }
        /// 1 forwards GPIO4's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio4(&mut self) -> gpio4::ASW<'_> {
            gpio4::ASW::new(self)
        }
        /// 1 forwards GPIO5's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio5(&mut self) -> gpio5::ASW<'_> {
            gpio5::ASW::new(self)
        }
        /// 1 forwards GPIO6's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio6(&mut self) -> gpio6::ASW<'_> {
            gpio6::ASW::new(self)
        }
        /// 1 forwards GPIO7's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio7(&mut self) -> gpio7::ASW<'_> {
            gpio7::ASW::new(self)
        }
        /// 1 forwards GPIO8's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio8(&mut self) -> gpio8::ASW<'_> {
            gpio8::ASW::new(self)
        }
        /// 1 forwards GPIO9's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio9(&mut self) -> gpio9::ASW<'_> {
            gpio9::ASW::new(self)
        }
        /// 1 forwards GPIO10's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio10(&mut self) -> gpio10::ASW<'_> {
            gpio10::ASW::new(self)
        }
        /// 1 forwards GPIO11's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio11(&mut self) -> gpio11::ASW<'_> {
            gpio11::ASW::new(self)
        }
        /// 1 forwards GPIO12's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio12(&mut self) -> gpio12::ASW<'_> {
            gpio12::ASW::new(self)
        }
        /// 1 forwards GPIO13's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio13(&mut self) -> gpio13::ASW<'_> {
            gpio13::ASW::new(self)
        }
        /// 1 forwards GPIO14's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio14(&mut self) -> gpio14::ASW<'_> {
            gpio14::ASW::new(self)
        }
        /// 1 forwards GPIO15's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio15(&mut self) -> gpio15::ASW<'_> {
            gpio15::ASW::new(self)
        }
        /// 1 forwards GPIO16's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio16(&mut self) -> gpio16::ASW<'_> {
            gpio16::ASW::new(self)
        }
        /// 1 forwards GPIO17's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio17(&mut self) -> gpio17::ASW<'_> {
            gpio17::ASW::new(self)
        }
        /// 1 forwards GPIO18's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio18(&mut self) -> gpio18::ASW<'_> {
            gpio18::ASW::new(self)
        }
        /// 1 forwards GPIO19's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio19(&mut self) -> gpio19::ASW<'_> {
            gpio19::ASW::new(self)
        }
        /// 1 forwards GPIO20's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio20(&mut self) -> gpio20::ASW<'_> {
            gpio20::ASW::new(self)
        }
        /// 1 forwards GPIO21's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio21(&mut self) -> gpio21::ASW<'_> {
            gpio21::ASW::new(self)
        }
        /// 1 forwards GPIO22's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio22(&mut self) -> gpio22::ASW<'_> {
            gpio22::ASW::new(self)
        }
        /// 1 forwards GPIO23's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio23(&mut self) -> gpio23::ASW<'_> {
            gpio23::ASW::new(self)
        }
        /// 1 forwards GPIO24's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio24(&mut self) -> gpio24::ASW<'_> {
            gpio24::ASW::new(self)
        }
        /// 1 forwards GPIO25's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio25(&mut self) -> gpio25::ASW<'_> {
            gpio25::ASW::new(self)
        }
        /// 1 forwards GPIO26's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio26(&mut self) -> gpio26::ASW<'_> {
            gpio26::ASW::new(self)
        }
        /// 1 forwards GPIO27's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio27(&mut self) -> gpio27::ASW<'_> {
            gpio27::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// 1 forwards GPIO0's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio0(&mut self) -> gpio0::ACW<'_> {
            gpio0::ACW::new(self)
        }
        /// 1 forwards GPIO1's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio1(&mut self) -> gpio1::ACW<'_> {
            gpio1::ACW::new(self)
        }
        /// 1 forwards GPIO2's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio2(&mut self) -> gpio2::ACW<'_> {
            gpio2::ACW::new(self)
        }
        /// 1 forwards GPIO3's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio3(&mut self) -> gpio3::ACW<'_> {
            gpio3::ACW::new(self)
        }
        /// 1 forwards GPIO4's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio4(&mut self) -> gpio4::ACW<'_> {
            gpio4::ACW::new(self)
        }
        /// 1 forwards GPIO5's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio5(&mut self) -> gpio5::ACW<'_> {
            gpio5::ACW::new(self)
        }
        /// 1 forwards GPIO6's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio6(&mut self) -> gpio6::ACW<'_> {
            gpio6::ACW::new(self)
        }
        /// 1 forwards GPIO7's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio7(&mut self) -> gpio7::ACW<'_> {
            gpio7::ACW::new(self)
        }
        /// 1 forwards GPIO8's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio8(&mut self) -> gpio8::ACW<'_> {
            gpio8::ACW::new(self)
        }
        /// 1 forwards GPIO9's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio9(&mut self) -> gpio9::ACW<'_> {
            gpio9::ACW::new(self)
        }
        /// 1 forwards GPIO10's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio10(&mut self) -> gpio10::ACW<'_> {
            gpio10::ACW::new(self)
        }
        /// 1 forwards GPIO11's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio11(&mut self) -> gpio11::ACW<'_> {
            gpio11::ACW::new(self)
        }
        /// 1 forwards GPIO12's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio12(&mut self) -> gpio12::ACW<'_> {
            gpio12::ACW::new(self)
        }
        /// 1 forwards GPIO13's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio13(&mut self) -> gpio13::ACW<'_> {
            gpio13::ACW::new(self)
        }
        /// 1 forwards GPIO14's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio14(&mut self) -> gpio14::ACW<'_> {
            gpio14::ACW::new(self)
        }
        /// 1 forwards GPIO15's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio15(&mut self) -> gpio15::ACW<'_> {
            gpio15::ACW::new(self)
        }
        /// 1 forwards GPIO16's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio16(&mut self) -> gpio16::ACW<'_> {
            gpio16::ACW::new(self)
        }
        /// 1 forwards GPIO17's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio17(&mut self) -> gpio17::ACW<'_> {
            gpio17::ACW::new(self)
        }
        /// 1 forwards GPIO18's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio18(&mut self) -> gpio18::ACW<'_> {
            gpio18::ACW::new(self)
        }
        /// 1 forwards GPIO19's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio19(&mut self) -> gpio19::ACW<'_> {
            gpio19::ACW::new(self)
        }
        /// 1 forwards GPIO20's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio20(&mut self) -> gpio20::ACW<'_> {
            gpio20::ACW::new(self)
        }
        /// 1 forwards GPIO21's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio21(&mut self) -> gpio21::ACW<'_> {
            gpio21::ACW::new(self)
        }
        /// 1 forwards GPIO22's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio22(&mut self) -> gpio22::ACW<'_> {
            gpio22::ACW::new(self)
        }
        /// 1 forwards GPIO23's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio23(&mut self) -> gpio23::ACW<'_> {
            gpio23::ACW::new(self)
        }
        /// 1 forwards GPIO24's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio24(&mut self) -> gpio24::ACW<'_> {
            gpio24::ACW::new(self)
        }
        /// 1 forwards GPIO25's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio25(&mut self) -> gpio25::ACW<'_> {
            gpio25::ACW::new(self)
        }
        /// 1 forwards GPIO26's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio26(&mut self) -> gpio26::ACW<'_> {
            gpio26::ACW::new(self)
        }
        /// 1 forwards GPIO27's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio27(&mut self) -> gpio27::ACW<'_> {
            gpio27::ACW::new(self)
        }
    }
    impl AtomicXorW {
        /// 1 forwards GPIO0's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio0(&mut self) -> gpio0::AXW<'_> {
            gpio0::AXW::new(self)
        }
        /// 1 forwards GPIO1's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio1(&mut self) -> gpio1::AXW<'_> {
            gpio1::AXW::new(self)
        }
        /// 1 forwards GPIO2's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio2(&mut self) -> gpio2::AXW<'_> {
            gpio2::AXW::new(self)
        }
        /// 1 forwards GPIO3's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio3(&mut self) -> gpio3::AXW<'_> {
            gpio3::AXW::new(self)
        }
        /// 1 forwards GPIO4's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio4(&mut self) -> gpio4::AXW<'_> {
            gpio4::AXW::new(self)
        }
        /// 1 forwards GPIO5's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio5(&mut self) -> gpio5::AXW<'_> {
            gpio5::AXW::new(self)
        }
        /// 1 forwards GPIO6's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio6(&mut self) -> gpio6::AXW<'_> {
            gpio6::AXW::new(self)
        }
        /// 1 forwards GPIO7's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio7(&mut self) -> gpio7::AXW<'_> {
            gpio7::AXW::new(self)
        }
        /// 1 forwards GPIO8's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio8(&mut self) -> gpio8::AXW<'_> {
            gpio8::AXW::new(self)
        }
        /// 1 forwards GPIO9's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio9(&mut self) -> gpio9::AXW<'_> {
            gpio9::AXW::new(self)
        }
        /// 1 forwards GPIO10's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio10(&mut self) -> gpio10::AXW<'_> {
            gpio10::AXW::new(self)
        }
        /// 1 forwards GPIO11's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio11(&mut self) -> gpio11::AXW<'_> {
            gpio11::AXW::new(self)
        }
        /// 1 forwards GPIO12's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio12(&mut self) -> gpio12::AXW<'_> {
            gpio12::AXW::new(self)
        }
        /// 1 forwards GPIO13's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio13(&mut self) -> gpio13::AXW<'_> {
            gpio13::AXW::new(self)
        }
        /// 1 forwards GPIO14's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio14(&mut self) -> gpio14::AXW<'_> {
            gpio14::AXW::new(self)
        }
        /// 1 forwards GPIO15's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio15(&mut self) -> gpio15::AXW<'_> {
            gpio15::AXW::new(self)
        }
        /// 1 forwards GPIO16's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio16(&mut self) -> gpio16::AXW<'_> {
            gpio16::AXW::new(self)
        }
        /// 1 forwards GPIO17's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio17(&mut self) -> gpio17::AXW<'_> {
            gpio17::AXW::new(self)
        }
        /// 1 forwards GPIO18's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio18(&mut self) -> gpio18::AXW<'_> {
            gpio18::AXW::new(self)
        }
        /// 1 forwards GPIO19's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio19(&mut self) -> gpio19::AXW<'_> {
            gpio19::AXW::new(self)
        }
        /// 1 forwards GPIO20's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio20(&mut self) -> gpio20::AXW<'_> {
            gpio20::AXW::new(self)
        }
        /// 1 forwards GPIO21's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio21(&mut self) -> gpio21::AXW<'_> {
            gpio21::AXW::new(self)
        }
        /// 1 forwards GPIO22's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio22(&mut self) -> gpio22::AXW<'_> {
            gpio22::AXW::new(self)
        }
        /// 1 forwards GPIO23's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio23(&mut self) -> gpio23::AXW<'_> {
            gpio23::AXW::new(self)
        }
        /// 1 forwards GPIO24's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio24(&mut self) -> gpio24::AXW<'_> {
            gpio24::AXW::new(self)
        }
        /// 1 forwards GPIO25's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio25(&mut self) -> gpio25::AXW<'_> {
            gpio25::AXW::new(self)
        }
        /// 1 forwards GPIO26's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio26(&mut self) -> gpio26::AXW<'_> {
            gpio26::AXW::new(self)
        }
        /// 1 forwards GPIO27's interrupt to the PCIe host; 0 blocks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio27(&mut self) -> gpio27::AXW<'_> {
            gpio27::AXW::new(self)
        }
    }
}

/// Per-pin interrupt force for the PCIe host. 1 forces the matching status bit.
/// Hardware access: read Plain; write Plain.
pub type PCIE_INTF = Reg<pcie_intf::PCIE_INTF_SPEC>;
/// Per-pin interrupt force for the PCIe host. 1 forces the matching status bit.
/// Hardware access: read Plain; write Plain.
pub mod pcie_intf {
    pub struct PCIE_INTF_SPEC;
    impl crate::RegisterSpec for PCIE_INTF_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for PCIE_INTF_SPEC {}
    pub type R = crate::R<PCIE_INTF_SPEC>;
    impl crate::Modifiable for PCIE_INTF_SPEC {}
    impl crate::UnsafeWritable for PCIE_INTF_SPEC {}
    impl crate::ResetValue for PCIE_INTF_SPEC {
        const RESET_VALUE: u32 = 0x0;
    }
    impl crate::TypedAtomicSet for PCIE_INTF_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicSet for PCIE_INTF_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::TypedAtomicClear for PCIE_INTF_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicClear for PCIE_INTF_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::TypedAtomicXor for PCIE_INTF_SPEC {
        const OFFSET: usize = 0x1000;
    }
    impl crate::UnsafeAtomicXor for PCIE_INTF_SPEC {
        const OFFSET: usize = 0x1000;
    }
    pub type W = crate::W<PCIE_INTF_SPEC>;
    pub type AtomicSetW = crate::AtomicSetW<PCIE_INTF_SPEC>;
    pub type AtomicClearW = crate::AtomicClearW<PCIE_INTF_SPEC>;
    pub type AtomicXorW = crate::AtomicXorW<PCIE_INTF_SPEC>;
    /// 1 forces GPIO0's interrupt to the PCIe host; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio0 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 0>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 0>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 0>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 0>;
    }
    /// 1 forces GPIO1's interrupt to the PCIe host; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio1 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 1>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 1>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 1>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 1>;
    }
    /// 1 forces GPIO2's interrupt to the PCIe host; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio2 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 2>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 2>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 2>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 2>;
    }
    /// 1 forces GPIO3's interrupt to the PCIe host; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio3 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 3>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 3>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 3>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 3>;
    }
    /// 1 forces GPIO4's interrupt to the PCIe host; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio4 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 4>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 4>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 4>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 4>;
    }
    /// 1 forces GPIO5's interrupt to the PCIe host; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio5 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 5>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 5>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 5>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 5>;
    }
    /// 1 forces GPIO6's interrupt to the PCIe host; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio6 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 6>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 6>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 6>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 6>;
    }
    /// 1 forces GPIO7's interrupt to the PCIe host; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio7 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 7>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 7>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 7>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 7>;
    }
    /// 1 forces GPIO8's interrupt to the PCIe host; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio8 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 8>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 8>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 8>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 8>;
    }
    /// 1 forces GPIO9's interrupt to the PCIe host; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio9 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 9>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 9>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 9>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 9>;
    }
    /// 1 forces GPIO10's interrupt to the PCIe host; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio10 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 10>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 10>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 10>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 10>;
    }
    /// 1 forces GPIO11's interrupt to the PCIe host; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio11 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 11>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 11>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 11>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 11>;
    }
    /// 1 forces GPIO12's interrupt to the PCIe host; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio12 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 12>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 12>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 12>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 12>;
    }
    /// 1 forces GPIO13's interrupt to the PCIe host; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio13 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 13>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 13>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 13>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 13>;
    }
    /// 1 forces GPIO14's interrupt to the PCIe host; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio14 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 14>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 14>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 14>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 14>;
    }
    /// 1 forces GPIO15's interrupt to the PCIe host; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio15 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 15>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 15>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 15>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 15>;
    }
    /// 1 forces GPIO16's interrupt to the PCIe host; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio16 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 16>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 16>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 16>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 16>;
    }
    /// 1 forces GPIO17's interrupt to the PCIe host; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio17 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 17>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 17>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 17>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 17>;
    }
    /// 1 forces GPIO18's interrupt to the PCIe host; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio18 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 18>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 18>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 18>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 18>;
    }
    /// 1 forces GPIO19's interrupt to the PCIe host; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio19 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 19>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 19>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 19>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 19>;
    }
    /// 1 forces GPIO20's interrupt to the PCIe host; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio20 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 20>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 20>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 20>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 20>;
    }
    /// 1 forces GPIO21's interrupt to the PCIe host; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio21 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 21>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 21>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 21>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 21>;
    }
    /// 1 forces GPIO22's interrupt to the PCIe host; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio22 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 22>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 22>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 22>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 22>;
    }
    /// 1 forces GPIO23's interrupt to the PCIe host; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio23 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 23>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 23>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 23>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 23>;
    }
    /// 1 forces GPIO24's interrupt to the PCIe host; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio24 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 24>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 24>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 24>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 24>;
    }
    /// 1 forces GPIO25's interrupt to the PCIe host; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio25 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 25>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 25>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 25>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 25>;
    }
    /// 1 forces GPIO26's interrupt to the PCIe host; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio26 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 26>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 26>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 26>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 26>;
    }
    /// 1 forces GPIO27's interrupt to the PCIe host; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod gpio27 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 27>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 27>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 27>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 27>;
    }
    impl R {
        #[inline(always)]
        pub fn gpio0(&self) -> gpio0::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio1(&self) -> gpio1::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio2(&self) -> gpio2::R {
            crate::BitReader::new(((self.bits() >> 2) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio3(&self) -> gpio3::R {
            crate::BitReader::new(((self.bits() >> 3) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio4(&self) -> gpio4::R {
            crate::BitReader::new(((self.bits() >> 4) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio5(&self) -> gpio5::R {
            crate::BitReader::new(((self.bits() >> 5) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio6(&self) -> gpio6::R {
            crate::BitReader::new(((self.bits() >> 6) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio7(&self) -> gpio7::R {
            crate::BitReader::new(((self.bits() >> 7) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio8(&self) -> gpio8::R {
            crate::BitReader::new(((self.bits() >> 8) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio9(&self) -> gpio9::R {
            crate::BitReader::new(((self.bits() >> 9) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio10(&self) -> gpio10::R {
            crate::BitReader::new(((self.bits() >> 10) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio11(&self) -> gpio11::R {
            crate::BitReader::new(((self.bits() >> 11) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio12(&self) -> gpio12::R {
            crate::BitReader::new(((self.bits() >> 12) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio13(&self) -> gpio13::R {
            crate::BitReader::new(((self.bits() >> 13) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio14(&self) -> gpio14::R {
            crate::BitReader::new(((self.bits() >> 14) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio15(&self) -> gpio15::R {
            crate::BitReader::new(((self.bits() >> 15) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio16(&self) -> gpio16::R {
            crate::BitReader::new(((self.bits() >> 16) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio17(&self) -> gpio17::R {
            crate::BitReader::new(((self.bits() >> 17) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio18(&self) -> gpio18::R {
            crate::BitReader::new(((self.bits() >> 18) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio19(&self) -> gpio19::R {
            crate::BitReader::new(((self.bits() >> 19) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio20(&self) -> gpio20::R {
            crate::BitReader::new(((self.bits() >> 20) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio21(&self) -> gpio21::R {
            crate::BitReader::new(((self.bits() >> 21) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio22(&self) -> gpio22::R {
            crate::BitReader::new(((self.bits() >> 22) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio23(&self) -> gpio23::R {
            crate::BitReader::new(((self.bits() >> 23) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio24(&self) -> gpio24::R {
            crate::BitReader::new(((self.bits() >> 24) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio25(&self) -> gpio25::R {
            crate::BitReader::new(((self.bits() >> 25) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio26(&self) -> gpio26::R {
            crate::BitReader::new(((self.bits() >> 26) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio27(&self) -> gpio27::R {
            crate::BitReader::new(((self.bits() >> 27) & 1) != 0)
        }
    }
    impl W {
        #[inline(always)]
        pub fn gpio0(&mut self) -> gpio0::W<'_> {
            gpio0::W::new(self)
        }
        #[inline(always)]
        pub fn gpio1(&mut self) -> gpio1::W<'_> {
            gpio1::W::new(self)
        }
        #[inline(always)]
        pub fn gpio2(&mut self) -> gpio2::W<'_> {
            gpio2::W::new(self)
        }
        #[inline(always)]
        pub fn gpio3(&mut self) -> gpio3::W<'_> {
            gpio3::W::new(self)
        }
        #[inline(always)]
        pub fn gpio4(&mut self) -> gpio4::W<'_> {
            gpio4::W::new(self)
        }
        #[inline(always)]
        pub fn gpio5(&mut self) -> gpio5::W<'_> {
            gpio5::W::new(self)
        }
        #[inline(always)]
        pub fn gpio6(&mut self) -> gpio6::W<'_> {
            gpio6::W::new(self)
        }
        #[inline(always)]
        pub fn gpio7(&mut self) -> gpio7::W<'_> {
            gpio7::W::new(self)
        }
        #[inline(always)]
        pub fn gpio8(&mut self) -> gpio8::W<'_> {
            gpio8::W::new(self)
        }
        #[inline(always)]
        pub fn gpio9(&mut self) -> gpio9::W<'_> {
            gpio9::W::new(self)
        }
        #[inline(always)]
        pub fn gpio10(&mut self) -> gpio10::W<'_> {
            gpio10::W::new(self)
        }
        #[inline(always)]
        pub fn gpio11(&mut self) -> gpio11::W<'_> {
            gpio11::W::new(self)
        }
        #[inline(always)]
        pub fn gpio12(&mut self) -> gpio12::W<'_> {
            gpio12::W::new(self)
        }
        #[inline(always)]
        pub fn gpio13(&mut self) -> gpio13::W<'_> {
            gpio13::W::new(self)
        }
        #[inline(always)]
        pub fn gpio14(&mut self) -> gpio14::W<'_> {
            gpio14::W::new(self)
        }
        #[inline(always)]
        pub fn gpio15(&mut self) -> gpio15::W<'_> {
            gpio15::W::new(self)
        }
        #[inline(always)]
        pub fn gpio16(&mut self) -> gpio16::W<'_> {
            gpio16::W::new(self)
        }
        #[inline(always)]
        pub fn gpio17(&mut self) -> gpio17::W<'_> {
            gpio17::W::new(self)
        }
        #[inline(always)]
        pub fn gpio18(&mut self) -> gpio18::W<'_> {
            gpio18::W::new(self)
        }
        #[inline(always)]
        pub fn gpio19(&mut self) -> gpio19::W<'_> {
            gpio19::W::new(self)
        }
        #[inline(always)]
        pub fn gpio20(&mut self) -> gpio20::W<'_> {
            gpio20::W::new(self)
        }
        #[inline(always)]
        pub fn gpio21(&mut self) -> gpio21::W<'_> {
            gpio21::W::new(self)
        }
        #[inline(always)]
        pub fn gpio22(&mut self) -> gpio22::W<'_> {
            gpio22::W::new(self)
        }
        #[inline(always)]
        pub fn gpio23(&mut self) -> gpio23::W<'_> {
            gpio23::W::new(self)
        }
        #[inline(always)]
        pub fn gpio24(&mut self) -> gpio24::W<'_> {
            gpio24::W::new(self)
        }
        #[inline(always)]
        pub fn gpio25(&mut self) -> gpio25::W<'_> {
            gpio25::W::new(self)
        }
        #[inline(always)]
        pub fn gpio26(&mut self) -> gpio26::W<'_> {
            gpio26::W::new(self)
        }
        #[inline(always)]
        pub fn gpio27(&mut self) -> gpio27::W<'_> {
            gpio27::W::new(self)
        }
    }
    impl AtomicSetW {
        /// 1 forces GPIO0's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio0(&mut self) -> gpio0::ASW<'_> {
            gpio0::ASW::new(self)
        }
        /// 1 forces GPIO1's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio1(&mut self) -> gpio1::ASW<'_> {
            gpio1::ASW::new(self)
        }
        /// 1 forces GPIO2's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio2(&mut self) -> gpio2::ASW<'_> {
            gpio2::ASW::new(self)
        }
        /// 1 forces GPIO3's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio3(&mut self) -> gpio3::ASW<'_> {
            gpio3::ASW::new(self)
        }
        /// 1 forces GPIO4's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio4(&mut self) -> gpio4::ASW<'_> {
            gpio4::ASW::new(self)
        }
        /// 1 forces GPIO5's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio5(&mut self) -> gpio5::ASW<'_> {
            gpio5::ASW::new(self)
        }
        /// 1 forces GPIO6's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio6(&mut self) -> gpio6::ASW<'_> {
            gpio6::ASW::new(self)
        }
        /// 1 forces GPIO7's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio7(&mut self) -> gpio7::ASW<'_> {
            gpio7::ASW::new(self)
        }
        /// 1 forces GPIO8's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio8(&mut self) -> gpio8::ASW<'_> {
            gpio8::ASW::new(self)
        }
        /// 1 forces GPIO9's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio9(&mut self) -> gpio9::ASW<'_> {
            gpio9::ASW::new(self)
        }
        /// 1 forces GPIO10's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio10(&mut self) -> gpio10::ASW<'_> {
            gpio10::ASW::new(self)
        }
        /// 1 forces GPIO11's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio11(&mut self) -> gpio11::ASW<'_> {
            gpio11::ASW::new(self)
        }
        /// 1 forces GPIO12's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio12(&mut self) -> gpio12::ASW<'_> {
            gpio12::ASW::new(self)
        }
        /// 1 forces GPIO13's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio13(&mut self) -> gpio13::ASW<'_> {
            gpio13::ASW::new(self)
        }
        /// 1 forces GPIO14's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio14(&mut self) -> gpio14::ASW<'_> {
            gpio14::ASW::new(self)
        }
        /// 1 forces GPIO15's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio15(&mut self) -> gpio15::ASW<'_> {
            gpio15::ASW::new(self)
        }
        /// 1 forces GPIO16's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio16(&mut self) -> gpio16::ASW<'_> {
            gpio16::ASW::new(self)
        }
        /// 1 forces GPIO17's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio17(&mut self) -> gpio17::ASW<'_> {
            gpio17::ASW::new(self)
        }
        /// 1 forces GPIO18's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio18(&mut self) -> gpio18::ASW<'_> {
            gpio18::ASW::new(self)
        }
        /// 1 forces GPIO19's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio19(&mut self) -> gpio19::ASW<'_> {
            gpio19::ASW::new(self)
        }
        /// 1 forces GPIO20's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio20(&mut self) -> gpio20::ASW<'_> {
            gpio20::ASW::new(self)
        }
        /// 1 forces GPIO21's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio21(&mut self) -> gpio21::ASW<'_> {
            gpio21::ASW::new(self)
        }
        /// 1 forces GPIO22's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio22(&mut self) -> gpio22::ASW<'_> {
            gpio22::ASW::new(self)
        }
        /// 1 forces GPIO23's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio23(&mut self) -> gpio23::ASW<'_> {
            gpio23::ASW::new(self)
        }
        /// 1 forces GPIO24's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio24(&mut self) -> gpio24::ASW<'_> {
            gpio24::ASW::new(self)
        }
        /// 1 forces GPIO25's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio25(&mut self) -> gpio25::ASW<'_> {
            gpio25::ASW::new(self)
        }
        /// 1 forces GPIO26's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio26(&mut self) -> gpio26::ASW<'_> {
            gpio26::ASW::new(self)
        }
        /// 1 forces GPIO27's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio27(&mut self) -> gpio27::ASW<'_> {
            gpio27::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// 1 forces GPIO0's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio0(&mut self) -> gpio0::ACW<'_> {
            gpio0::ACW::new(self)
        }
        /// 1 forces GPIO1's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio1(&mut self) -> gpio1::ACW<'_> {
            gpio1::ACW::new(self)
        }
        /// 1 forces GPIO2's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio2(&mut self) -> gpio2::ACW<'_> {
            gpio2::ACW::new(self)
        }
        /// 1 forces GPIO3's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio3(&mut self) -> gpio3::ACW<'_> {
            gpio3::ACW::new(self)
        }
        /// 1 forces GPIO4's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio4(&mut self) -> gpio4::ACW<'_> {
            gpio4::ACW::new(self)
        }
        /// 1 forces GPIO5's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio5(&mut self) -> gpio5::ACW<'_> {
            gpio5::ACW::new(self)
        }
        /// 1 forces GPIO6's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio6(&mut self) -> gpio6::ACW<'_> {
            gpio6::ACW::new(self)
        }
        /// 1 forces GPIO7's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio7(&mut self) -> gpio7::ACW<'_> {
            gpio7::ACW::new(self)
        }
        /// 1 forces GPIO8's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio8(&mut self) -> gpio8::ACW<'_> {
            gpio8::ACW::new(self)
        }
        /// 1 forces GPIO9's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio9(&mut self) -> gpio9::ACW<'_> {
            gpio9::ACW::new(self)
        }
        /// 1 forces GPIO10's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio10(&mut self) -> gpio10::ACW<'_> {
            gpio10::ACW::new(self)
        }
        /// 1 forces GPIO11's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio11(&mut self) -> gpio11::ACW<'_> {
            gpio11::ACW::new(self)
        }
        /// 1 forces GPIO12's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio12(&mut self) -> gpio12::ACW<'_> {
            gpio12::ACW::new(self)
        }
        /// 1 forces GPIO13's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio13(&mut self) -> gpio13::ACW<'_> {
            gpio13::ACW::new(self)
        }
        /// 1 forces GPIO14's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio14(&mut self) -> gpio14::ACW<'_> {
            gpio14::ACW::new(self)
        }
        /// 1 forces GPIO15's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio15(&mut self) -> gpio15::ACW<'_> {
            gpio15::ACW::new(self)
        }
        /// 1 forces GPIO16's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio16(&mut self) -> gpio16::ACW<'_> {
            gpio16::ACW::new(self)
        }
        /// 1 forces GPIO17's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio17(&mut self) -> gpio17::ACW<'_> {
            gpio17::ACW::new(self)
        }
        /// 1 forces GPIO18's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio18(&mut self) -> gpio18::ACW<'_> {
            gpio18::ACW::new(self)
        }
        /// 1 forces GPIO19's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio19(&mut self) -> gpio19::ACW<'_> {
            gpio19::ACW::new(self)
        }
        /// 1 forces GPIO20's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio20(&mut self) -> gpio20::ACW<'_> {
            gpio20::ACW::new(self)
        }
        /// 1 forces GPIO21's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio21(&mut self) -> gpio21::ACW<'_> {
            gpio21::ACW::new(self)
        }
        /// 1 forces GPIO22's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio22(&mut self) -> gpio22::ACW<'_> {
            gpio22::ACW::new(self)
        }
        /// 1 forces GPIO23's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio23(&mut self) -> gpio23::ACW<'_> {
            gpio23::ACW::new(self)
        }
        /// 1 forces GPIO24's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio24(&mut self) -> gpio24::ACW<'_> {
            gpio24::ACW::new(self)
        }
        /// 1 forces GPIO25's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio25(&mut self) -> gpio25::ACW<'_> {
            gpio25::ACW::new(self)
        }
        /// 1 forces GPIO26's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio26(&mut self) -> gpio26::ACW<'_> {
            gpio26::ACW::new(self)
        }
        /// 1 forces GPIO27's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio27(&mut self) -> gpio27::ACW<'_> {
            gpio27::ACW::new(self)
        }
    }
    impl AtomicXorW {
        /// 1 forces GPIO0's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio0(&mut self) -> gpio0::AXW<'_> {
            gpio0::AXW::new(self)
        }
        /// 1 forces GPIO1's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio1(&mut self) -> gpio1::AXW<'_> {
            gpio1::AXW::new(self)
        }
        /// 1 forces GPIO2's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio2(&mut self) -> gpio2::AXW<'_> {
            gpio2::AXW::new(self)
        }
        /// 1 forces GPIO3's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio3(&mut self) -> gpio3::AXW<'_> {
            gpio3::AXW::new(self)
        }
        /// 1 forces GPIO4's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio4(&mut self) -> gpio4::AXW<'_> {
            gpio4::AXW::new(self)
        }
        /// 1 forces GPIO5's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio5(&mut self) -> gpio5::AXW<'_> {
            gpio5::AXW::new(self)
        }
        /// 1 forces GPIO6's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio6(&mut self) -> gpio6::AXW<'_> {
            gpio6::AXW::new(self)
        }
        /// 1 forces GPIO7's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio7(&mut self) -> gpio7::AXW<'_> {
            gpio7::AXW::new(self)
        }
        /// 1 forces GPIO8's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio8(&mut self) -> gpio8::AXW<'_> {
            gpio8::AXW::new(self)
        }
        /// 1 forces GPIO9's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio9(&mut self) -> gpio9::AXW<'_> {
            gpio9::AXW::new(self)
        }
        /// 1 forces GPIO10's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio10(&mut self) -> gpio10::AXW<'_> {
            gpio10::AXW::new(self)
        }
        /// 1 forces GPIO11's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio11(&mut self) -> gpio11::AXW<'_> {
            gpio11::AXW::new(self)
        }
        /// 1 forces GPIO12's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio12(&mut self) -> gpio12::AXW<'_> {
            gpio12::AXW::new(self)
        }
        /// 1 forces GPIO13's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio13(&mut self) -> gpio13::AXW<'_> {
            gpio13::AXW::new(self)
        }
        /// 1 forces GPIO14's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio14(&mut self) -> gpio14::AXW<'_> {
            gpio14::AXW::new(self)
        }
        /// 1 forces GPIO15's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio15(&mut self) -> gpio15::AXW<'_> {
            gpio15::AXW::new(self)
        }
        /// 1 forces GPIO16's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio16(&mut self) -> gpio16::AXW<'_> {
            gpio16::AXW::new(self)
        }
        /// 1 forces GPIO17's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio17(&mut self) -> gpio17::AXW<'_> {
            gpio17::AXW::new(self)
        }
        /// 1 forces GPIO18's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio18(&mut self) -> gpio18::AXW<'_> {
            gpio18::AXW::new(self)
        }
        /// 1 forces GPIO19's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio19(&mut self) -> gpio19::AXW<'_> {
            gpio19::AXW::new(self)
        }
        /// 1 forces GPIO20's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio20(&mut self) -> gpio20::AXW<'_> {
            gpio20::AXW::new(self)
        }
        /// 1 forces GPIO21's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio21(&mut self) -> gpio21::AXW<'_> {
            gpio21::AXW::new(self)
        }
        /// 1 forces GPIO22's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio22(&mut self) -> gpio22::AXW<'_> {
            gpio22::AXW::new(self)
        }
        /// 1 forces GPIO23's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio23(&mut self) -> gpio23::AXW<'_> {
            gpio23::AXW::new(self)
        }
        /// 1 forces GPIO24's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio24(&mut self) -> gpio24::AXW<'_> {
            gpio24::AXW::new(self)
        }
        /// 1 forces GPIO25's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio25(&mut self) -> gpio25::AXW<'_> {
            gpio25::AXW::new(self)
        }
        /// 1 forces GPIO26's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio26(&mut self) -> gpio26::AXW<'_> {
            gpio26::AXW::new(self)
        }
        /// 1 forces GPIO27's interrupt to the PCIe host; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio27(&mut self) -> gpio27::AXW<'_> {
            gpio27::AXW::new(self)
        }
    }
}

/// Per-pin interrupt status seen by the PCIe host: the enabled raw interrupt combined with the force bit. For level-triggered handling, enable MSI-X acknowledge handling so that re-asserted interrupts are not lost.
/// Hardware access: read Plain; write None.
pub type PCIE_INTS = Reg<pcie_ints::PCIE_INTS_SPEC>;
/// Per-pin interrupt status seen by the PCIe host: the enabled raw interrupt combined with the force bit. For level-triggered handling, enable MSI-X acknowledge handling so that re-asserted interrupts are not lost.
/// Hardware access: read Plain; write None.
pub mod pcie_ints {
    pub struct PCIE_INTS_SPEC;
    impl crate::RegisterSpec for PCIE_INTS_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for PCIE_INTS_SPEC {}
    pub type R = crate::R<PCIE_INTS_SPEC>;
    impl crate::ResetValue for PCIE_INTS_SPEC {
        const RESET_VALUE: u32 = 0x0;
    }
    /// 1 if GPIO0 has an effective interrupt for the PCIe host.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio0 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO1 has an effective interrupt for the PCIe host.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio1 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO2 has an effective interrupt for the PCIe host.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio2 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO3 has an effective interrupt for the PCIe host.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio3 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO4 has an effective interrupt for the PCIe host.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio4 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO5 has an effective interrupt for the PCIe host.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio5 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO6 has an effective interrupt for the PCIe host.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio6 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO7 has an effective interrupt for the PCIe host.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio7 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO8 has an effective interrupt for the PCIe host.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio8 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO9 has an effective interrupt for the PCIe host.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio9 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO10 has an effective interrupt for the PCIe host.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio10 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO11 has an effective interrupt for the PCIe host.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio11 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO12 has an effective interrupt for the PCIe host.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio12 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO13 has an effective interrupt for the PCIe host.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio13 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO14 has an effective interrupt for the PCIe host.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio14 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO15 has an effective interrupt for the PCIe host.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio15 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO16 has an effective interrupt for the PCIe host.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio16 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO17 has an effective interrupt for the PCIe host.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio17 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO18 has an effective interrupt for the PCIe host.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio18 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO19 has an effective interrupt for the PCIe host.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio19 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO20 has an effective interrupt for the PCIe host.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio20 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO21 has an effective interrupt for the PCIe host.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio21 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO22 has an effective interrupt for the PCIe host.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio22 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO23 has an effective interrupt for the PCIe host.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio23 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO24 has an effective interrupt for the PCIe host.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio24 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO25 has an effective interrupt for the PCIe host.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio25 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO26 has an effective interrupt for the PCIe host.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio26 {
        pub type R = crate::BitReader;
    }
    /// 1 if GPIO27 has an effective interrupt for the PCIe host.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio27 {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn gpio0(&self) -> gpio0::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio1(&self) -> gpio1::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio2(&self) -> gpio2::R {
            crate::BitReader::new(((self.bits() >> 2) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio3(&self) -> gpio3::R {
            crate::BitReader::new(((self.bits() >> 3) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio4(&self) -> gpio4::R {
            crate::BitReader::new(((self.bits() >> 4) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio5(&self) -> gpio5::R {
            crate::BitReader::new(((self.bits() >> 5) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio6(&self) -> gpio6::R {
            crate::BitReader::new(((self.bits() >> 6) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio7(&self) -> gpio7::R {
            crate::BitReader::new(((self.bits() >> 7) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio8(&self) -> gpio8::R {
            crate::BitReader::new(((self.bits() >> 8) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio9(&self) -> gpio9::R {
            crate::BitReader::new(((self.bits() >> 9) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio10(&self) -> gpio10::R {
            crate::BitReader::new(((self.bits() >> 10) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio11(&self) -> gpio11::R {
            crate::BitReader::new(((self.bits() >> 11) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio12(&self) -> gpio12::R {
            crate::BitReader::new(((self.bits() >> 12) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio13(&self) -> gpio13::R {
            crate::BitReader::new(((self.bits() >> 13) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio14(&self) -> gpio14::R {
            crate::BitReader::new(((self.bits() >> 14) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio15(&self) -> gpio15::R {
            crate::BitReader::new(((self.bits() >> 15) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio16(&self) -> gpio16::R {
            crate::BitReader::new(((self.bits() >> 16) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio17(&self) -> gpio17::R {
            crate::BitReader::new(((self.bits() >> 17) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio18(&self) -> gpio18::R {
            crate::BitReader::new(((self.bits() >> 18) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio19(&self) -> gpio19::R {
            crate::BitReader::new(((self.bits() >> 19) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio20(&self) -> gpio20::R {
            crate::BitReader::new(((self.bits() >> 20) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio21(&self) -> gpio21::R {
            crate::BitReader::new(((self.bits() >> 21) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio22(&self) -> gpio22::R {
            crate::BitReader::new(((self.bits() >> 22) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio23(&self) -> gpio23::R {
            crate::BitReader::new(((self.bits() >> 23) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio24(&self) -> gpio24::R {
            crate::BitReader::new(((self.bits() >> 24) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio25(&self) -> gpio25::R {
            crate::BitReader::new(((self.bits() >> 25) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio26(&self) -> gpio26::R {
            crate::BitReader::new(((self.bits() >> 26) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio27(&self) -> gpio27::R {
            crate::BitReader::new(((self.bits() >> 27) & 1) != 0)
        }
    }
}

#[cfg(test)]
mod layout_tests {
    use super::RegisterBlock;
    #[test]
    fn gpio_status_offset() {
        assert_eq!(
            core::mem::offset_of!(RegisterBlock, gpio_status)
                + core::mem::offset_of!(super::GPIO_STATUSArrayElement, gpio_status),
            0x0
        );
    }
    #[test]
    fn gpio_status_stride() {
        assert_eq!(core::mem::size_of::<super::GPIO_STATUSArrayElement>(), 0x8);
    }
    #[test]
    fn gpio_ctrl_offset() {
        assert_eq!(
            core::mem::offset_of!(RegisterBlock, gpio_status)
                + core::mem::offset_of!(super::GPIO_STATUSArrayElement, gpio_ctrl),
            0x4
        );
    }
    #[test]
    fn gpio_ctrl_stride() {
        assert_eq!(core::mem::size_of::<super::GPIO_STATUSArrayElement>(), 0x8);
    }
    #[test]
    fn intr_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, intr), 0x100);
    }
    #[test]
    fn proc0_inte_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, proc0_inte), 0x104);
    }
    #[test]
    fn proc0_intf_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, proc0_intf), 0x108);
    }
    #[test]
    fn proc0_ints_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, proc0_ints), 0x10c);
    }
    #[test]
    fn proc1_inte_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, proc1_inte), 0x110);
    }
    #[test]
    fn proc1_intf_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, proc1_intf), 0x114);
    }
    #[test]
    fn proc1_ints_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, proc1_ints), 0x118);
    }
    #[test]
    fn pcie_inte_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, pcie_inte), 0x11c);
    }
    #[test]
    fn pcie_intf_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, pcie_intf), 0x120);
    }
    #[test]
    fn pcie_ints_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, pcie_ints), 0x124);
    }
    #[test]
    fn register_block_size() {
        assert_eq!(core::mem::size_of::<RegisterBlock>(), 0x128);
    }
}
