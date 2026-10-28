// Generated from rp1-regdb/data/peripherals/io-bank1.toml.
// Do not edit manually.
// Regenerate with: cargo run -p rp1-regdb -- generate

use crate::Reg;

#[repr(C)]
struct GPIO_STATUSArrayElement {
    gpio_status: GPIO_STATUS,
    gpio_ctrl: GPIO_CTRL,
}

/// RP1 IO_BANK1 register block with supplemental register semantics. Alias behavior: normal read/write access. Bank geometry and interrupt offsets are recovered from the Linux pinctrl driver; register fields are inherited from the public bank-0 layout. Unknown offsets remain split RAW arrays.
/// Atomic view: Rp1Native alias windows; ownership remains with this logical peripheral.
#[repr(C)]
pub struct RegisterBlock {
    gpio_status: [GPIO_STATUSArrayElement; 6],
    _reserved0: [u8; 208],
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
    /// GPIO status
    /// Hardware access: read Plain; write None.
    /// Array layout: 6 physical entries, 6 public entries, stride 0x8 bytes.
    #[inline(always)]
    pub fn gpio_status(&self, index: usize) -> &GPIO_STATUS {
        assert!(index < 6);
        &self.gpio_status[index].gpio_status
    }
    /// GPIO control including function select and overrides
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    /// Array layout: 6 physical entries, 6 public entries, stride 0x8 bytes.
    #[inline(always)]
    pub fn gpio_ctrl(&self, index: usize) -> &GPIO_CTRL {
        assert!(index < 6);
        &self.gpio_status[index].gpio_ctrl
    }
    /// Raw interrupts
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn intr(&self) -> &INTR {
        &self.intr
    }
    /// Interrupt enable for proc0
    /// Hardware access: read Plain; write Plain.
    #[inline(always)]
    pub const fn proc0_inte(&self) -> &PROC0_INTE {
        &self.proc0_inte
    }
    /// Interrupt force for proc0
    /// Hardware access: read Plain; write Plain.
    #[inline(always)]
    pub const fn proc0_intf(&self) -> &PROC0_INTF {
        &self.proc0_intf
    }
    /// Interrupt status after masking and forcing for proc0
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn proc0_ints(&self) -> &PROC0_INTS {
        &self.proc0_ints
    }
    /// Interrupt enable for proc1
    /// Hardware access: read Plain; write Plain.
    #[inline(always)]
    pub const fn proc1_inte(&self) -> &PROC1_INTE {
        &self.proc1_inte
    }
    /// Interrupt force for proc1
    /// Hardware access: read Plain; write Plain.
    #[inline(always)]
    pub const fn proc1_intf(&self) -> &PROC1_INTF {
        &self.proc1_intf
    }
    /// Interrupt status after masking and forcing for proc1
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn proc1_ints(&self) -> &PROC1_INTS {
        &self.proc1_ints
    }
    /// Interrupt enable for PCIe
    /// Hardware access: read Plain; write Plain.
    #[inline(always)]
    pub const fn pcie_inte(&self) -> &PCIE_INTE {
        &self.pcie_inte
    }
    /// Interrupt force for PCIe
    /// Hardware access: read Plain; write Plain.
    #[inline(always)]
    pub const fn pcie_intf(&self) -> &PCIE_INTF {
        &self.pcie_intf
    }
    /// Interrupt status after masking and forcing for PCIe
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn pcie_ints(&self) -> &PCIE_INTS {
        &self.pcie_ints
    }
}

/// GPIO status
/// Hardware access: read Plain; write None.
/// Array layout: 6 physical entries, 6 public entries, stride 0x8 bytes.
pub type GPIO_STATUS = Reg<gpio_status::GPIO_STATUS_SPEC>;
/// GPIO status
/// Hardware access: read Plain; write None.
/// Array layout: 6 physical entries, 6 public entries, stride 0x8 bytes.
pub mod gpio_status {
    pub struct GPIO_STATUS_SPEC;
    impl crate::RegisterSpec for GPIO_STATUS_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for GPIO_STATUS_SPEC {}
    pub type R = crate::R<GPIO_STATUS_SPEC>;
    /// Output signal from selected peripheral before override
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod outfromperi {
        pub type R = crate::BitReader;
    }
    /// Output signal to pad after override
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod outtopad {
        pub type R = crate::BitReader;
    }
    /// Output enable from selected peripheral before override
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod oefromperi {
        pub type R = crate::BitReader;
    }
    /// Output enable to pad after override
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod oetopad {
        pub type R = crate::BitReader;
    }
    /// Input signal from pad goes directly to selected peripheral
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod inisdirect {
        pub type R = crate::BitReader;
    }
    /// Input signal from pad before filtering and override
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod infrompad {
        pub type R = crate::BitReader;
    }
    /// Input signal from pad after filtering before override
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod infiltered {
        pub type R = crate::BitReader;
    }
    /// Input signal to peripheral after filtering and override
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod intoperi {
        pub type R = crate::BitReader;
    }
    /// Input pin has seen falling edge
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod event_edge_low {
        pub type R = crate::BitReader;
    }
    /// Input pin has seen rising edge
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod event_edge_high {
        pub type R = crate::BitReader;
    }
    /// Input pin is low
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod event_level_low {
        pub type R = crate::BitReader;
    }
    /// Input pin is high
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod event_level_high {
        pub type R = crate::BitReader;
    }
    /// Input pin has seen a filtered falling edge
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod event_f_edge_low {
        pub type R = crate::BitReader;
    }
    /// Input pin has seen a filtered rising edge
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod event_f_edge_high {
        pub type R = crate::BitReader;
    }
    /// Debounced input pin is low
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod event_db_level_low {
        pub type R = crate::BitReader;
    }
    /// Debounced input pin is high
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod event_db_level_high {
        pub type R = crate::BitReader;
    }
    /// Interrupt to processors after masking
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod irqcombined {
        pub type R = crate::BitReader;
    }
    /// Interrupt to processors after mask and override
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

/// GPIO control including function select and overrides
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
/// Array layout: 6 physical entries, 6 public entries, stride 0x8 bytes.
pub type GPIO_CTRL = Reg<gpio_ctrl::GPIO_CTRL_SPEC>;
/// GPIO control including function select and overrides
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
/// Array layout: 6 physical entries, 6 public entries, stride 0x8 bytes.
pub mod gpio_ctrl {
    pub struct GPIO_CTRL_SPEC;
    impl crate::RegisterSpec for GPIO_CTRL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for GPIO_CTRL_SPEC {}
    pub type R = crate::R<GPIO_CTRL_SPEC>;
    impl crate::UnsafeWritable for GPIO_CTRL_SPEC {}
    impl crate::UnsafeAtomicSet for GPIO_CTRL_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for GPIO_CTRL_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for GPIO_CTRL_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// Function select; 31 is NULL
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod funcsel {
        pub type R = crate::FieldReader<u32>;
    }
    /// Filter/debounce time constant M
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod f_m {
        pub type R = crate::FieldReader<u32>;
    }
    /// Output override
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod outover {
        pub type R = crate::FieldReader<u32>;
    }
    /// Output-enable override
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod oeover {
        pub type R = crate::FieldReader<u32>;
    }
    /// Input override
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod inover {
        pub type R = crate::FieldReader<u32>;
    }
    /// Masks edge-low interrupt into interrupt output
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod irqmask_edge_low {
        pub type R = crate::BitReader;
    }
    /// Masks edge-high interrupt into interrupt output
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod irqmask_edge_high {
        pub type R = crate::BitReader;
    }
    /// Masks level-low interrupt into interrupt output
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod irqmask_level_low {
        pub type R = crate::BitReader;
    }
    /// Masks level-high interrupt into interrupt output
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod irqmask_level_high {
        pub type R = crate::BitReader;
    }
    /// Masks filtered edge-low interrupt into interrupt output
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod irqmask_f_edge_low {
        pub type R = crate::BitReader;
    }
    /// Masks filtered edge-high interrupt into interrupt output
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod irqmask_f_edge_high {
        pub type R = crate::BitReader;
    }
    /// Masks debounced level-low interrupt into interrupt output
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod irqmask_db_level_low {
        pub type R = crate::BitReader;
    }
    /// Masks debounced level-high interrupt into interrupt output
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod irqmask_db_level_high {
        pub type R = crate::BitReader;
    }
    /// Reset interrupt edge detector; self-clearing
    /// Field write behavior: WriteOneToSet.
    /// Typed write API is not exposed by register policy.
    pub mod irqreset {
        pub type R = crate::BitReader;
    }
    /// Interrupt override
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod irqover {
        pub type R = crate::FieldReader<u32>;
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

/// Raw interrupts
/// Hardware access: read Plain; write None.
pub type INTR = Reg<intr::INTR_SPEC>;
/// Raw interrupts
/// Hardware access: read Plain; write None.
pub mod intr {
    pub struct INTR_SPEC;
    impl crate::RegisterSpec for INTR_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for INTR_SPEC {}
    pub type R = crate::R<INTR_SPEC>;
    /// GPIO28 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio28 {
        pub type R = crate::BitReader;
    }
    /// GPIO29 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio29 {
        pub type R = crate::BitReader;
    }
    /// GPIO30 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio30 {
        pub type R = crate::BitReader;
    }
    /// GPIO31 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio31 {
        pub type R = crate::BitReader;
    }
    /// GPIO32 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio32 {
        pub type R = crate::BitReader;
    }
    /// GPIO33 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio33 {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn gpio28(&self) -> gpio28::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio29(&self) -> gpio29::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio30(&self) -> gpio30::R {
            crate::BitReader::new(((self.bits() >> 2) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio31(&self) -> gpio31::R {
            crate::BitReader::new(((self.bits() >> 3) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio32(&self) -> gpio32::R {
            crate::BitReader::new(((self.bits() >> 4) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio33(&self) -> gpio33::R {
            crate::BitReader::new(((self.bits() >> 5) & 1) != 0)
        }
    }
}

/// Interrupt enable for proc0
/// Hardware access: read Plain; write Plain.
pub type PROC0_INTE = Reg<proc0_inte::PROC0_INTE_SPEC>;
/// Interrupt enable for proc0
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
    /// GPIO28 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio28 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 0>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 0>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 0>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 0>;
    }
    /// GPIO29 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio29 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 1>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 1>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 1>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 1>;
    }
    /// GPIO30 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio30 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 2>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 2>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 2>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 2>;
    }
    /// GPIO31 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio31 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 3>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 3>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 3>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 3>;
    }
    /// GPIO32 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio32 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 4>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 4>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 4>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 4>;
    }
    /// GPIO33 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio33 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 5>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 5>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 5>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 5>;
    }
    impl R {
        #[inline(always)]
        pub fn gpio28(&self) -> gpio28::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio29(&self) -> gpio29::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio30(&self) -> gpio30::R {
            crate::BitReader::new(((self.bits() >> 2) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio31(&self) -> gpio31::R {
            crate::BitReader::new(((self.bits() >> 3) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio32(&self) -> gpio32::R {
            crate::BitReader::new(((self.bits() >> 4) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio33(&self) -> gpio33::R {
            crate::BitReader::new(((self.bits() >> 5) & 1) != 0)
        }
    }
    impl W {
        #[inline(always)]
        pub fn gpio28(&mut self) -> gpio28::W<'_> {
            gpio28::W::new(self)
        }
        #[inline(always)]
        pub fn gpio29(&mut self) -> gpio29::W<'_> {
            gpio29::W::new(self)
        }
        #[inline(always)]
        pub fn gpio30(&mut self) -> gpio30::W<'_> {
            gpio30::W::new(self)
        }
        #[inline(always)]
        pub fn gpio31(&mut self) -> gpio31::W<'_> {
            gpio31::W::new(self)
        }
        #[inline(always)]
        pub fn gpio32(&mut self) -> gpio32::W<'_> {
            gpio32::W::new(self)
        }
        #[inline(always)]
        pub fn gpio33(&mut self) -> gpio33::W<'_> {
            gpio33::W::new(self)
        }
    }
    impl AtomicSetW {
        /// GPIO28 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio28(&mut self) -> gpio28::ASW<'_> {
            gpio28::ASW::new(self)
        }
        /// GPIO29 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio29(&mut self) -> gpio29::ASW<'_> {
            gpio29::ASW::new(self)
        }
        /// GPIO30 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio30(&mut self) -> gpio30::ASW<'_> {
            gpio30::ASW::new(self)
        }
        /// GPIO31 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio31(&mut self) -> gpio31::ASW<'_> {
            gpio31::ASW::new(self)
        }
        /// GPIO32 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio32(&mut self) -> gpio32::ASW<'_> {
            gpio32::ASW::new(self)
        }
        /// GPIO33 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio33(&mut self) -> gpio33::ASW<'_> {
            gpio33::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// GPIO28 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio28(&mut self) -> gpio28::ACW<'_> {
            gpio28::ACW::new(self)
        }
        /// GPIO29 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio29(&mut self) -> gpio29::ACW<'_> {
            gpio29::ACW::new(self)
        }
        /// GPIO30 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio30(&mut self) -> gpio30::ACW<'_> {
            gpio30::ACW::new(self)
        }
        /// GPIO31 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio31(&mut self) -> gpio31::ACW<'_> {
            gpio31::ACW::new(self)
        }
        /// GPIO32 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio32(&mut self) -> gpio32::ACW<'_> {
            gpio32::ACW::new(self)
        }
        /// GPIO33 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio33(&mut self) -> gpio33::ACW<'_> {
            gpio33::ACW::new(self)
        }
    }
    impl AtomicXorW {
        /// GPIO28 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio28(&mut self) -> gpio28::AXW<'_> {
            gpio28::AXW::new(self)
        }
        /// GPIO29 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio29(&mut self) -> gpio29::AXW<'_> {
            gpio29::AXW::new(self)
        }
        /// GPIO30 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio30(&mut self) -> gpio30::AXW<'_> {
            gpio30::AXW::new(self)
        }
        /// GPIO31 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio31(&mut self) -> gpio31::AXW<'_> {
            gpio31::AXW::new(self)
        }
        /// GPIO32 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio32(&mut self) -> gpio32::AXW<'_> {
            gpio32::AXW::new(self)
        }
        /// GPIO33 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio33(&mut self) -> gpio33::AXW<'_> {
            gpio33::AXW::new(self)
        }
    }
}

/// Interrupt force for proc0
/// Hardware access: read Plain; write Plain.
pub type PROC0_INTF = Reg<proc0_intf::PROC0_INTF_SPEC>;
/// Interrupt force for proc0
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
    /// GPIO28 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio28 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 0>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 0>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 0>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 0>;
    }
    /// GPIO29 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio29 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 1>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 1>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 1>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 1>;
    }
    /// GPIO30 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio30 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 2>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 2>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 2>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 2>;
    }
    /// GPIO31 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio31 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 3>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 3>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 3>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 3>;
    }
    /// GPIO32 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio32 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 4>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 4>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 4>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 4>;
    }
    /// GPIO33 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio33 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 5>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 5>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 5>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 5>;
    }
    impl R {
        #[inline(always)]
        pub fn gpio28(&self) -> gpio28::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio29(&self) -> gpio29::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio30(&self) -> gpio30::R {
            crate::BitReader::new(((self.bits() >> 2) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio31(&self) -> gpio31::R {
            crate::BitReader::new(((self.bits() >> 3) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio32(&self) -> gpio32::R {
            crate::BitReader::new(((self.bits() >> 4) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio33(&self) -> gpio33::R {
            crate::BitReader::new(((self.bits() >> 5) & 1) != 0)
        }
    }
    impl W {
        #[inline(always)]
        pub fn gpio28(&mut self) -> gpio28::W<'_> {
            gpio28::W::new(self)
        }
        #[inline(always)]
        pub fn gpio29(&mut self) -> gpio29::W<'_> {
            gpio29::W::new(self)
        }
        #[inline(always)]
        pub fn gpio30(&mut self) -> gpio30::W<'_> {
            gpio30::W::new(self)
        }
        #[inline(always)]
        pub fn gpio31(&mut self) -> gpio31::W<'_> {
            gpio31::W::new(self)
        }
        #[inline(always)]
        pub fn gpio32(&mut self) -> gpio32::W<'_> {
            gpio32::W::new(self)
        }
        #[inline(always)]
        pub fn gpio33(&mut self) -> gpio33::W<'_> {
            gpio33::W::new(self)
        }
    }
    impl AtomicSetW {
        /// GPIO28 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio28(&mut self) -> gpio28::ASW<'_> {
            gpio28::ASW::new(self)
        }
        /// GPIO29 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio29(&mut self) -> gpio29::ASW<'_> {
            gpio29::ASW::new(self)
        }
        /// GPIO30 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio30(&mut self) -> gpio30::ASW<'_> {
            gpio30::ASW::new(self)
        }
        /// GPIO31 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio31(&mut self) -> gpio31::ASW<'_> {
            gpio31::ASW::new(self)
        }
        /// GPIO32 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio32(&mut self) -> gpio32::ASW<'_> {
            gpio32::ASW::new(self)
        }
        /// GPIO33 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio33(&mut self) -> gpio33::ASW<'_> {
            gpio33::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// GPIO28 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio28(&mut self) -> gpio28::ACW<'_> {
            gpio28::ACW::new(self)
        }
        /// GPIO29 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio29(&mut self) -> gpio29::ACW<'_> {
            gpio29::ACW::new(self)
        }
        /// GPIO30 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio30(&mut self) -> gpio30::ACW<'_> {
            gpio30::ACW::new(self)
        }
        /// GPIO31 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio31(&mut self) -> gpio31::ACW<'_> {
            gpio31::ACW::new(self)
        }
        /// GPIO32 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio32(&mut self) -> gpio32::ACW<'_> {
            gpio32::ACW::new(self)
        }
        /// GPIO33 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio33(&mut self) -> gpio33::ACW<'_> {
            gpio33::ACW::new(self)
        }
    }
    impl AtomicXorW {
        /// GPIO28 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio28(&mut self) -> gpio28::AXW<'_> {
            gpio28::AXW::new(self)
        }
        /// GPIO29 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio29(&mut self) -> gpio29::AXW<'_> {
            gpio29::AXW::new(self)
        }
        /// GPIO30 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio30(&mut self) -> gpio30::AXW<'_> {
            gpio30::AXW::new(self)
        }
        /// GPIO31 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio31(&mut self) -> gpio31::AXW<'_> {
            gpio31::AXW::new(self)
        }
        /// GPIO32 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio32(&mut self) -> gpio32::AXW<'_> {
            gpio32::AXW::new(self)
        }
        /// GPIO33 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio33(&mut self) -> gpio33::AXW<'_> {
            gpio33::AXW::new(self)
        }
    }
}

/// Interrupt status after masking and forcing for proc0
/// Hardware access: read Plain; write None.
pub type PROC0_INTS = Reg<proc0_ints::PROC0_INTS_SPEC>;
/// Interrupt status after masking and forcing for proc0
/// Hardware access: read Plain; write None.
pub mod proc0_ints {
    pub struct PROC0_INTS_SPEC;
    impl crate::RegisterSpec for PROC0_INTS_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for PROC0_INTS_SPEC {}
    pub type R = crate::R<PROC0_INTS_SPEC>;
    /// GPIO28 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio28 {
        pub type R = crate::BitReader;
    }
    /// GPIO29 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio29 {
        pub type R = crate::BitReader;
    }
    /// GPIO30 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio30 {
        pub type R = crate::BitReader;
    }
    /// GPIO31 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio31 {
        pub type R = crate::BitReader;
    }
    /// GPIO32 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio32 {
        pub type R = crate::BitReader;
    }
    /// GPIO33 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio33 {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn gpio28(&self) -> gpio28::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio29(&self) -> gpio29::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio30(&self) -> gpio30::R {
            crate::BitReader::new(((self.bits() >> 2) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio31(&self) -> gpio31::R {
            crate::BitReader::new(((self.bits() >> 3) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio32(&self) -> gpio32::R {
            crate::BitReader::new(((self.bits() >> 4) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio33(&self) -> gpio33::R {
            crate::BitReader::new(((self.bits() >> 5) & 1) != 0)
        }
    }
}

/// Interrupt enable for proc1
/// Hardware access: read Plain; write Plain.
pub type PROC1_INTE = Reg<proc1_inte::PROC1_INTE_SPEC>;
/// Interrupt enable for proc1
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
    /// GPIO28 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio28 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 0>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 0>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 0>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 0>;
    }
    /// GPIO29 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio29 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 1>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 1>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 1>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 1>;
    }
    /// GPIO30 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio30 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 2>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 2>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 2>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 2>;
    }
    /// GPIO31 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio31 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 3>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 3>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 3>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 3>;
    }
    /// GPIO32 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio32 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 4>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 4>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 4>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 4>;
    }
    /// GPIO33 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio33 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 5>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 5>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 5>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 5>;
    }
    impl R {
        #[inline(always)]
        pub fn gpio28(&self) -> gpio28::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio29(&self) -> gpio29::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio30(&self) -> gpio30::R {
            crate::BitReader::new(((self.bits() >> 2) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio31(&self) -> gpio31::R {
            crate::BitReader::new(((self.bits() >> 3) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio32(&self) -> gpio32::R {
            crate::BitReader::new(((self.bits() >> 4) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio33(&self) -> gpio33::R {
            crate::BitReader::new(((self.bits() >> 5) & 1) != 0)
        }
    }
    impl W {
        #[inline(always)]
        pub fn gpio28(&mut self) -> gpio28::W<'_> {
            gpio28::W::new(self)
        }
        #[inline(always)]
        pub fn gpio29(&mut self) -> gpio29::W<'_> {
            gpio29::W::new(self)
        }
        #[inline(always)]
        pub fn gpio30(&mut self) -> gpio30::W<'_> {
            gpio30::W::new(self)
        }
        #[inline(always)]
        pub fn gpio31(&mut self) -> gpio31::W<'_> {
            gpio31::W::new(self)
        }
        #[inline(always)]
        pub fn gpio32(&mut self) -> gpio32::W<'_> {
            gpio32::W::new(self)
        }
        #[inline(always)]
        pub fn gpio33(&mut self) -> gpio33::W<'_> {
            gpio33::W::new(self)
        }
    }
    impl AtomicSetW {
        /// GPIO28 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio28(&mut self) -> gpio28::ASW<'_> {
            gpio28::ASW::new(self)
        }
        /// GPIO29 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio29(&mut self) -> gpio29::ASW<'_> {
            gpio29::ASW::new(self)
        }
        /// GPIO30 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio30(&mut self) -> gpio30::ASW<'_> {
            gpio30::ASW::new(self)
        }
        /// GPIO31 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio31(&mut self) -> gpio31::ASW<'_> {
            gpio31::ASW::new(self)
        }
        /// GPIO32 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio32(&mut self) -> gpio32::ASW<'_> {
            gpio32::ASW::new(self)
        }
        /// GPIO33 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio33(&mut self) -> gpio33::ASW<'_> {
            gpio33::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// GPIO28 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio28(&mut self) -> gpio28::ACW<'_> {
            gpio28::ACW::new(self)
        }
        /// GPIO29 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio29(&mut self) -> gpio29::ACW<'_> {
            gpio29::ACW::new(self)
        }
        /// GPIO30 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio30(&mut self) -> gpio30::ACW<'_> {
            gpio30::ACW::new(self)
        }
        /// GPIO31 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio31(&mut self) -> gpio31::ACW<'_> {
            gpio31::ACW::new(self)
        }
        /// GPIO32 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio32(&mut self) -> gpio32::ACW<'_> {
            gpio32::ACW::new(self)
        }
        /// GPIO33 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio33(&mut self) -> gpio33::ACW<'_> {
            gpio33::ACW::new(self)
        }
    }
    impl AtomicXorW {
        /// GPIO28 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio28(&mut self) -> gpio28::AXW<'_> {
            gpio28::AXW::new(self)
        }
        /// GPIO29 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio29(&mut self) -> gpio29::AXW<'_> {
            gpio29::AXW::new(self)
        }
        /// GPIO30 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio30(&mut self) -> gpio30::AXW<'_> {
            gpio30::AXW::new(self)
        }
        /// GPIO31 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio31(&mut self) -> gpio31::AXW<'_> {
            gpio31::AXW::new(self)
        }
        /// GPIO32 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio32(&mut self) -> gpio32::AXW<'_> {
            gpio32::AXW::new(self)
        }
        /// GPIO33 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio33(&mut self) -> gpio33::AXW<'_> {
            gpio33::AXW::new(self)
        }
    }
}

/// Interrupt force for proc1
/// Hardware access: read Plain; write Plain.
pub type PROC1_INTF = Reg<proc1_intf::PROC1_INTF_SPEC>;
/// Interrupt force for proc1
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
    /// GPIO28 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio28 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 0>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 0>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 0>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 0>;
    }
    /// GPIO29 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio29 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 1>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 1>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 1>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 1>;
    }
    /// GPIO30 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio30 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 2>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 2>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 2>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 2>;
    }
    /// GPIO31 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio31 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 3>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 3>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 3>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 3>;
    }
    /// GPIO32 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio32 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 4>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 4>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 4>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 4>;
    }
    /// GPIO33 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio33 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 5>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 5>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 5>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 5>;
    }
    impl R {
        #[inline(always)]
        pub fn gpio28(&self) -> gpio28::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio29(&self) -> gpio29::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio30(&self) -> gpio30::R {
            crate::BitReader::new(((self.bits() >> 2) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio31(&self) -> gpio31::R {
            crate::BitReader::new(((self.bits() >> 3) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio32(&self) -> gpio32::R {
            crate::BitReader::new(((self.bits() >> 4) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio33(&self) -> gpio33::R {
            crate::BitReader::new(((self.bits() >> 5) & 1) != 0)
        }
    }
    impl W {
        #[inline(always)]
        pub fn gpio28(&mut self) -> gpio28::W<'_> {
            gpio28::W::new(self)
        }
        #[inline(always)]
        pub fn gpio29(&mut self) -> gpio29::W<'_> {
            gpio29::W::new(self)
        }
        #[inline(always)]
        pub fn gpio30(&mut self) -> gpio30::W<'_> {
            gpio30::W::new(self)
        }
        #[inline(always)]
        pub fn gpio31(&mut self) -> gpio31::W<'_> {
            gpio31::W::new(self)
        }
        #[inline(always)]
        pub fn gpio32(&mut self) -> gpio32::W<'_> {
            gpio32::W::new(self)
        }
        #[inline(always)]
        pub fn gpio33(&mut self) -> gpio33::W<'_> {
            gpio33::W::new(self)
        }
    }
    impl AtomicSetW {
        /// GPIO28 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio28(&mut self) -> gpio28::ASW<'_> {
            gpio28::ASW::new(self)
        }
        /// GPIO29 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio29(&mut self) -> gpio29::ASW<'_> {
            gpio29::ASW::new(self)
        }
        /// GPIO30 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio30(&mut self) -> gpio30::ASW<'_> {
            gpio30::ASW::new(self)
        }
        /// GPIO31 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio31(&mut self) -> gpio31::ASW<'_> {
            gpio31::ASW::new(self)
        }
        /// GPIO32 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio32(&mut self) -> gpio32::ASW<'_> {
            gpio32::ASW::new(self)
        }
        /// GPIO33 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio33(&mut self) -> gpio33::ASW<'_> {
            gpio33::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// GPIO28 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio28(&mut self) -> gpio28::ACW<'_> {
            gpio28::ACW::new(self)
        }
        /// GPIO29 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio29(&mut self) -> gpio29::ACW<'_> {
            gpio29::ACW::new(self)
        }
        /// GPIO30 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio30(&mut self) -> gpio30::ACW<'_> {
            gpio30::ACW::new(self)
        }
        /// GPIO31 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio31(&mut self) -> gpio31::ACW<'_> {
            gpio31::ACW::new(self)
        }
        /// GPIO32 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio32(&mut self) -> gpio32::ACW<'_> {
            gpio32::ACW::new(self)
        }
        /// GPIO33 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio33(&mut self) -> gpio33::ACW<'_> {
            gpio33::ACW::new(self)
        }
    }
    impl AtomicXorW {
        /// GPIO28 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio28(&mut self) -> gpio28::AXW<'_> {
            gpio28::AXW::new(self)
        }
        /// GPIO29 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio29(&mut self) -> gpio29::AXW<'_> {
            gpio29::AXW::new(self)
        }
        /// GPIO30 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio30(&mut self) -> gpio30::AXW<'_> {
            gpio30::AXW::new(self)
        }
        /// GPIO31 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio31(&mut self) -> gpio31::AXW<'_> {
            gpio31::AXW::new(self)
        }
        /// GPIO32 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio32(&mut self) -> gpio32::AXW<'_> {
            gpio32::AXW::new(self)
        }
        /// GPIO33 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio33(&mut self) -> gpio33::AXW<'_> {
            gpio33::AXW::new(self)
        }
    }
}

/// Interrupt status after masking and forcing for proc1
/// Hardware access: read Plain; write None.
pub type PROC1_INTS = Reg<proc1_ints::PROC1_INTS_SPEC>;
/// Interrupt status after masking and forcing for proc1
/// Hardware access: read Plain; write None.
pub mod proc1_ints {
    pub struct PROC1_INTS_SPEC;
    impl crate::RegisterSpec for PROC1_INTS_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for PROC1_INTS_SPEC {}
    pub type R = crate::R<PROC1_INTS_SPEC>;
    /// GPIO28 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio28 {
        pub type R = crate::BitReader;
    }
    /// GPIO29 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio29 {
        pub type R = crate::BitReader;
    }
    /// GPIO30 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio30 {
        pub type R = crate::BitReader;
    }
    /// GPIO31 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio31 {
        pub type R = crate::BitReader;
    }
    /// GPIO32 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio32 {
        pub type R = crate::BitReader;
    }
    /// GPIO33 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio33 {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn gpio28(&self) -> gpio28::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio29(&self) -> gpio29::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio30(&self) -> gpio30::R {
            crate::BitReader::new(((self.bits() >> 2) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio31(&self) -> gpio31::R {
            crate::BitReader::new(((self.bits() >> 3) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio32(&self) -> gpio32::R {
            crate::BitReader::new(((self.bits() >> 4) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio33(&self) -> gpio33::R {
            crate::BitReader::new(((self.bits() >> 5) & 1) != 0)
        }
    }
}

/// Interrupt enable for PCIe
/// Hardware access: read Plain; write Plain.
pub type PCIE_INTE = Reg<pcie_inte::PCIE_INTE_SPEC>;
/// Interrupt enable for PCIe
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
    /// GPIO28 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio28 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 0>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 0>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 0>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 0>;
    }
    /// GPIO29 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio29 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 1>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 1>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 1>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 1>;
    }
    /// GPIO30 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio30 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 2>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 2>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 2>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 2>;
    }
    /// GPIO31 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio31 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 3>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 3>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 3>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 3>;
    }
    /// GPIO32 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio32 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 4>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 4>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 4>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 4>;
    }
    /// GPIO33 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio33 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 5>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 5>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 5>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 5>;
    }
    impl R {
        #[inline(always)]
        pub fn gpio28(&self) -> gpio28::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio29(&self) -> gpio29::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio30(&self) -> gpio30::R {
            crate::BitReader::new(((self.bits() >> 2) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio31(&self) -> gpio31::R {
            crate::BitReader::new(((self.bits() >> 3) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio32(&self) -> gpio32::R {
            crate::BitReader::new(((self.bits() >> 4) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio33(&self) -> gpio33::R {
            crate::BitReader::new(((self.bits() >> 5) & 1) != 0)
        }
    }
    impl W {
        #[inline(always)]
        pub fn gpio28(&mut self) -> gpio28::W<'_> {
            gpio28::W::new(self)
        }
        #[inline(always)]
        pub fn gpio29(&mut self) -> gpio29::W<'_> {
            gpio29::W::new(self)
        }
        #[inline(always)]
        pub fn gpio30(&mut self) -> gpio30::W<'_> {
            gpio30::W::new(self)
        }
        #[inline(always)]
        pub fn gpio31(&mut self) -> gpio31::W<'_> {
            gpio31::W::new(self)
        }
        #[inline(always)]
        pub fn gpio32(&mut self) -> gpio32::W<'_> {
            gpio32::W::new(self)
        }
        #[inline(always)]
        pub fn gpio33(&mut self) -> gpio33::W<'_> {
            gpio33::W::new(self)
        }
    }
    impl AtomicSetW {
        /// GPIO28 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio28(&mut self) -> gpio28::ASW<'_> {
            gpio28::ASW::new(self)
        }
        /// GPIO29 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio29(&mut self) -> gpio29::ASW<'_> {
            gpio29::ASW::new(self)
        }
        /// GPIO30 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio30(&mut self) -> gpio30::ASW<'_> {
            gpio30::ASW::new(self)
        }
        /// GPIO31 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio31(&mut self) -> gpio31::ASW<'_> {
            gpio31::ASW::new(self)
        }
        /// GPIO32 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio32(&mut self) -> gpio32::ASW<'_> {
            gpio32::ASW::new(self)
        }
        /// GPIO33 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio33(&mut self) -> gpio33::ASW<'_> {
            gpio33::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// GPIO28 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio28(&mut self) -> gpio28::ACW<'_> {
            gpio28::ACW::new(self)
        }
        /// GPIO29 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio29(&mut self) -> gpio29::ACW<'_> {
            gpio29::ACW::new(self)
        }
        /// GPIO30 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio30(&mut self) -> gpio30::ACW<'_> {
            gpio30::ACW::new(self)
        }
        /// GPIO31 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio31(&mut self) -> gpio31::ACW<'_> {
            gpio31::ACW::new(self)
        }
        /// GPIO32 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio32(&mut self) -> gpio32::ACW<'_> {
            gpio32::ACW::new(self)
        }
        /// GPIO33 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio33(&mut self) -> gpio33::ACW<'_> {
            gpio33::ACW::new(self)
        }
    }
    impl AtomicXorW {
        /// GPIO28 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio28(&mut self) -> gpio28::AXW<'_> {
            gpio28::AXW::new(self)
        }
        /// GPIO29 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio29(&mut self) -> gpio29::AXW<'_> {
            gpio29::AXW::new(self)
        }
        /// GPIO30 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio30(&mut self) -> gpio30::AXW<'_> {
            gpio30::AXW::new(self)
        }
        /// GPIO31 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio31(&mut self) -> gpio31::AXW<'_> {
            gpio31::AXW::new(self)
        }
        /// GPIO32 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio32(&mut self) -> gpio32::AXW<'_> {
            gpio32::AXW::new(self)
        }
        /// GPIO33 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio33(&mut self) -> gpio33::AXW<'_> {
            gpio33::AXW::new(self)
        }
    }
}

/// Interrupt force for PCIe
/// Hardware access: read Plain; write Plain.
pub type PCIE_INTF = Reg<pcie_intf::PCIE_INTF_SPEC>;
/// Interrupt force for PCIe
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
    /// GPIO28 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio28 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 0>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 0>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 0>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 0>;
    }
    /// GPIO29 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio29 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 1>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 1>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 1>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 1>;
    }
    /// GPIO30 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio30 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 2>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 2>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 2>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 2>;
    }
    /// GPIO31 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio31 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 3>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 3>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 3>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 3>;
    }
    /// GPIO32 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio32 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 4>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 4>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 4>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 4>;
    }
    /// GPIO33 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio33 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 5>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 5>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 5>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 5>;
    }
    impl R {
        #[inline(always)]
        pub fn gpio28(&self) -> gpio28::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio29(&self) -> gpio29::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio30(&self) -> gpio30::R {
            crate::BitReader::new(((self.bits() >> 2) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio31(&self) -> gpio31::R {
            crate::BitReader::new(((self.bits() >> 3) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio32(&self) -> gpio32::R {
            crate::BitReader::new(((self.bits() >> 4) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio33(&self) -> gpio33::R {
            crate::BitReader::new(((self.bits() >> 5) & 1) != 0)
        }
    }
    impl W {
        #[inline(always)]
        pub fn gpio28(&mut self) -> gpio28::W<'_> {
            gpio28::W::new(self)
        }
        #[inline(always)]
        pub fn gpio29(&mut self) -> gpio29::W<'_> {
            gpio29::W::new(self)
        }
        #[inline(always)]
        pub fn gpio30(&mut self) -> gpio30::W<'_> {
            gpio30::W::new(self)
        }
        #[inline(always)]
        pub fn gpio31(&mut self) -> gpio31::W<'_> {
            gpio31::W::new(self)
        }
        #[inline(always)]
        pub fn gpio32(&mut self) -> gpio32::W<'_> {
            gpio32::W::new(self)
        }
        #[inline(always)]
        pub fn gpio33(&mut self) -> gpio33::W<'_> {
            gpio33::W::new(self)
        }
    }
    impl AtomicSetW {
        /// GPIO28 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio28(&mut self) -> gpio28::ASW<'_> {
            gpio28::ASW::new(self)
        }
        /// GPIO29 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio29(&mut self) -> gpio29::ASW<'_> {
            gpio29::ASW::new(self)
        }
        /// GPIO30 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio30(&mut self) -> gpio30::ASW<'_> {
            gpio30::ASW::new(self)
        }
        /// GPIO31 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio31(&mut self) -> gpio31::ASW<'_> {
            gpio31::ASW::new(self)
        }
        /// GPIO32 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio32(&mut self) -> gpio32::ASW<'_> {
            gpio32::ASW::new(self)
        }
        /// GPIO33 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio33(&mut self) -> gpio33::ASW<'_> {
            gpio33::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// GPIO28 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio28(&mut self) -> gpio28::ACW<'_> {
            gpio28::ACW::new(self)
        }
        /// GPIO29 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio29(&mut self) -> gpio29::ACW<'_> {
            gpio29::ACW::new(self)
        }
        /// GPIO30 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio30(&mut self) -> gpio30::ACW<'_> {
            gpio30::ACW::new(self)
        }
        /// GPIO31 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio31(&mut self) -> gpio31::ACW<'_> {
            gpio31::ACW::new(self)
        }
        /// GPIO32 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio32(&mut self) -> gpio32::ACW<'_> {
            gpio32::ACW::new(self)
        }
        /// GPIO33 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio33(&mut self) -> gpio33::ACW<'_> {
            gpio33::ACW::new(self)
        }
    }
    impl AtomicXorW {
        /// GPIO28 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio28(&mut self) -> gpio28::AXW<'_> {
            gpio28::AXW::new(self)
        }
        /// GPIO29 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio29(&mut self) -> gpio29::AXW<'_> {
            gpio29::AXW::new(self)
        }
        /// GPIO30 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio30(&mut self) -> gpio30::AXW<'_> {
            gpio30::AXW::new(self)
        }
        /// GPIO31 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio31(&mut self) -> gpio31::AXW<'_> {
            gpio31::AXW::new(self)
        }
        /// GPIO32 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio32(&mut self) -> gpio32::AXW<'_> {
            gpio32::AXW::new(self)
        }
        /// GPIO33 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio33(&mut self) -> gpio33::AXW<'_> {
            gpio33::AXW::new(self)
        }
    }
}

/// Interrupt status after masking and forcing for PCIe
/// Hardware access: read Plain; write None.
pub type PCIE_INTS = Reg<pcie_ints::PCIE_INTS_SPEC>;
/// Interrupt status after masking and forcing for PCIe
/// Hardware access: read Plain; write None.
pub mod pcie_ints {
    pub struct PCIE_INTS_SPEC;
    impl crate::RegisterSpec for PCIE_INTS_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for PCIE_INTS_SPEC {}
    pub type R = crate::R<PCIE_INTS_SPEC>;
    /// GPIO28 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio28 {
        pub type R = crate::BitReader;
    }
    /// GPIO29 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio29 {
        pub type R = crate::BitReader;
    }
    /// GPIO30 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio30 {
        pub type R = crate::BitReader;
    }
    /// GPIO31 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio31 {
        pub type R = crate::BitReader;
    }
    /// GPIO32 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio32 {
        pub type R = crate::BitReader;
    }
    /// GPIO33 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio33 {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn gpio28(&self) -> gpio28::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio29(&self) -> gpio29::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio30(&self) -> gpio30::R {
            crate::BitReader::new(((self.bits() >> 2) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio31(&self) -> gpio31::R {
            crate::BitReader::new(((self.bits() >> 3) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio32(&self) -> gpio32::R {
            crate::BitReader::new(((self.bits() >> 4) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio33(&self) -> gpio33::R {
            crate::BitReader::new(((self.bits() >> 5) & 1) != 0)
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
