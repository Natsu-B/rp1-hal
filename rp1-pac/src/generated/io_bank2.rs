// Generated from rp1-regdb/data/peripherals/io-bank2.toml.
// Do not edit manually.
// Regenerate with: cargo run -p rp1-regdb -- generate

use crate::Reg;

#[repr(C)]
struct GPIO_STATUSArrayElement {
    gpio_status: GPIO_STATUS,
    gpio_ctrl: GPIO_CTRL,
}

/// RP1 IO_BANK2 register block with supplemental register semantics. Alias behavior: normal read/write access. Bank geometry and interrupt offsets are recovered from the Linux pinctrl driver; register fields are inherited from the public bank-0 layout. Unknown offsets remain split RAW arrays.
/// Atomic view: Rp1Native alias windows; ownership remains with this logical peripheral.
#[repr(C)]
pub struct RegisterBlock {
    gpio_status: [GPIO_STATUSArrayElement; 20],
    _reserved0: [u8; 96],
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
    /// Array layout: 20 physical entries, 20 public entries, stride 0x8 bytes.
    #[inline(always)]
    pub fn gpio_status(&self, index: usize) -> &GPIO_STATUS {
        assert!(index < 20);
        &self.gpio_status[index].gpio_status
    }
    /// GPIO control including function select and overrides
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    /// Array layout: 20 physical entries, 20 public entries, stride 0x8 bytes.
    #[inline(always)]
    pub fn gpio_ctrl(&self, index: usize) -> &GPIO_CTRL {
        assert!(index < 20);
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
/// Array layout: 20 physical entries, 20 public entries, stride 0x8 bytes.
pub type GPIO_STATUS = Reg<gpio_status::GPIO_STATUS_SPEC>;
/// GPIO status
/// Hardware access: read Plain; write None.
/// Array layout: 20 physical entries, 20 public entries, stride 0x8 bytes.
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
/// Array layout: 20 physical entries, 20 public entries, stride 0x8 bytes.
pub type GPIO_CTRL = Reg<gpio_ctrl::GPIO_CTRL_SPEC>;
/// GPIO control including function select and overrides
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
/// Array layout: 20 physical entries, 20 public entries, stride 0x8 bytes.
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
    /// GPIO34 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio34 {
        pub type R = crate::BitReader;
    }
    /// GPIO35 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio35 {
        pub type R = crate::BitReader;
    }
    /// GPIO36 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio36 {
        pub type R = crate::BitReader;
    }
    /// GPIO37 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio37 {
        pub type R = crate::BitReader;
    }
    /// GPIO38 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio38 {
        pub type R = crate::BitReader;
    }
    /// GPIO39 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio39 {
        pub type R = crate::BitReader;
    }
    /// GPIO40 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio40 {
        pub type R = crate::BitReader;
    }
    /// GPIO41 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio41 {
        pub type R = crate::BitReader;
    }
    /// GPIO42 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio42 {
        pub type R = crate::BitReader;
    }
    /// GPIO43 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio43 {
        pub type R = crate::BitReader;
    }
    /// GPIO44 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio44 {
        pub type R = crate::BitReader;
    }
    /// GPIO45 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio45 {
        pub type R = crate::BitReader;
    }
    /// GPIO46 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio46 {
        pub type R = crate::BitReader;
    }
    /// GPIO47 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio47 {
        pub type R = crate::BitReader;
    }
    /// GPIO48 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio48 {
        pub type R = crate::BitReader;
    }
    /// GPIO49 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio49 {
        pub type R = crate::BitReader;
    }
    /// GPIO50 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio50 {
        pub type R = crate::BitReader;
    }
    /// GPIO51 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio51 {
        pub type R = crate::BitReader;
    }
    /// GPIO52 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio52 {
        pub type R = crate::BitReader;
    }
    /// GPIO53 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio53 {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn gpio34(&self) -> gpio34::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio35(&self) -> gpio35::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio36(&self) -> gpio36::R {
            crate::BitReader::new(((self.bits() >> 2) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio37(&self) -> gpio37::R {
            crate::BitReader::new(((self.bits() >> 3) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio38(&self) -> gpio38::R {
            crate::BitReader::new(((self.bits() >> 4) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio39(&self) -> gpio39::R {
            crate::BitReader::new(((self.bits() >> 5) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio40(&self) -> gpio40::R {
            crate::BitReader::new(((self.bits() >> 6) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio41(&self) -> gpio41::R {
            crate::BitReader::new(((self.bits() >> 7) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio42(&self) -> gpio42::R {
            crate::BitReader::new(((self.bits() >> 8) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio43(&self) -> gpio43::R {
            crate::BitReader::new(((self.bits() >> 9) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio44(&self) -> gpio44::R {
            crate::BitReader::new(((self.bits() >> 10) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio45(&self) -> gpio45::R {
            crate::BitReader::new(((self.bits() >> 11) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio46(&self) -> gpio46::R {
            crate::BitReader::new(((self.bits() >> 12) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio47(&self) -> gpio47::R {
            crate::BitReader::new(((self.bits() >> 13) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio48(&self) -> gpio48::R {
            crate::BitReader::new(((self.bits() >> 14) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio49(&self) -> gpio49::R {
            crate::BitReader::new(((self.bits() >> 15) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio50(&self) -> gpio50::R {
            crate::BitReader::new(((self.bits() >> 16) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio51(&self) -> gpio51::R {
            crate::BitReader::new(((self.bits() >> 17) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio52(&self) -> gpio52::R {
            crate::BitReader::new(((self.bits() >> 18) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio53(&self) -> gpio53::R {
            crate::BitReader::new(((self.bits() >> 19) & 1) != 0)
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
    /// GPIO34 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio34 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 0>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 0>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 0>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 0>;
    }
    /// GPIO35 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio35 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 1>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 1>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 1>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 1>;
    }
    /// GPIO36 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio36 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 2>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 2>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 2>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 2>;
    }
    /// GPIO37 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio37 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 3>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 3>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 3>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 3>;
    }
    /// GPIO38 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio38 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 4>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 4>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 4>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 4>;
    }
    /// GPIO39 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio39 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 5>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 5>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 5>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 5>;
    }
    /// GPIO40 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio40 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 6>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 6>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 6>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 6>;
    }
    /// GPIO41 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio41 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 7>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 7>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 7>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 7>;
    }
    /// GPIO42 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio42 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 8>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 8>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 8>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 8>;
    }
    /// GPIO43 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio43 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 9>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 9>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 9>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 9>;
    }
    /// GPIO44 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio44 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 10>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 10>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 10>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 10>;
    }
    /// GPIO45 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio45 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 11>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 11>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 11>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 11>;
    }
    /// GPIO46 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio46 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 12>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 12>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 12>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 12>;
    }
    /// GPIO47 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio47 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 13>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 13>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 13>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 13>;
    }
    /// GPIO48 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio48 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 14>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 14>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 14>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 14>;
    }
    /// GPIO49 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio49 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 15>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 15>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 15>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 15>;
    }
    /// GPIO50 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio50 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 16>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 16>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 16>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 16>;
    }
    /// GPIO51 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio51 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 17>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 17>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 17>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 17>;
    }
    /// GPIO52 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio52 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 18>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 18>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 18>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 18>;
    }
    /// GPIO53 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio53 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 19>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 19>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 19>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 19>;
    }
    impl R {
        #[inline(always)]
        pub fn gpio34(&self) -> gpio34::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio35(&self) -> gpio35::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio36(&self) -> gpio36::R {
            crate::BitReader::new(((self.bits() >> 2) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio37(&self) -> gpio37::R {
            crate::BitReader::new(((self.bits() >> 3) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio38(&self) -> gpio38::R {
            crate::BitReader::new(((self.bits() >> 4) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio39(&self) -> gpio39::R {
            crate::BitReader::new(((self.bits() >> 5) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio40(&self) -> gpio40::R {
            crate::BitReader::new(((self.bits() >> 6) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio41(&self) -> gpio41::R {
            crate::BitReader::new(((self.bits() >> 7) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio42(&self) -> gpio42::R {
            crate::BitReader::new(((self.bits() >> 8) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio43(&self) -> gpio43::R {
            crate::BitReader::new(((self.bits() >> 9) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio44(&self) -> gpio44::R {
            crate::BitReader::new(((self.bits() >> 10) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio45(&self) -> gpio45::R {
            crate::BitReader::new(((self.bits() >> 11) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio46(&self) -> gpio46::R {
            crate::BitReader::new(((self.bits() >> 12) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio47(&self) -> gpio47::R {
            crate::BitReader::new(((self.bits() >> 13) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio48(&self) -> gpio48::R {
            crate::BitReader::new(((self.bits() >> 14) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio49(&self) -> gpio49::R {
            crate::BitReader::new(((self.bits() >> 15) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio50(&self) -> gpio50::R {
            crate::BitReader::new(((self.bits() >> 16) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio51(&self) -> gpio51::R {
            crate::BitReader::new(((self.bits() >> 17) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio52(&self) -> gpio52::R {
            crate::BitReader::new(((self.bits() >> 18) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio53(&self) -> gpio53::R {
            crate::BitReader::new(((self.bits() >> 19) & 1) != 0)
        }
    }
    impl W {
        #[inline(always)]
        pub fn gpio34(&mut self) -> gpio34::W<'_> {
            gpio34::W::new(self)
        }
        #[inline(always)]
        pub fn gpio35(&mut self) -> gpio35::W<'_> {
            gpio35::W::new(self)
        }
        #[inline(always)]
        pub fn gpio36(&mut self) -> gpio36::W<'_> {
            gpio36::W::new(self)
        }
        #[inline(always)]
        pub fn gpio37(&mut self) -> gpio37::W<'_> {
            gpio37::W::new(self)
        }
        #[inline(always)]
        pub fn gpio38(&mut self) -> gpio38::W<'_> {
            gpio38::W::new(self)
        }
        #[inline(always)]
        pub fn gpio39(&mut self) -> gpio39::W<'_> {
            gpio39::W::new(self)
        }
        #[inline(always)]
        pub fn gpio40(&mut self) -> gpio40::W<'_> {
            gpio40::W::new(self)
        }
        #[inline(always)]
        pub fn gpio41(&mut self) -> gpio41::W<'_> {
            gpio41::W::new(self)
        }
        #[inline(always)]
        pub fn gpio42(&mut self) -> gpio42::W<'_> {
            gpio42::W::new(self)
        }
        #[inline(always)]
        pub fn gpio43(&mut self) -> gpio43::W<'_> {
            gpio43::W::new(self)
        }
        #[inline(always)]
        pub fn gpio44(&mut self) -> gpio44::W<'_> {
            gpio44::W::new(self)
        }
        #[inline(always)]
        pub fn gpio45(&mut self) -> gpio45::W<'_> {
            gpio45::W::new(self)
        }
        #[inline(always)]
        pub fn gpio46(&mut self) -> gpio46::W<'_> {
            gpio46::W::new(self)
        }
        #[inline(always)]
        pub fn gpio47(&mut self) -> gpio47::W<'_> {
            gpio47::W::new(self)
        }
        #[inline(always)]
        pub fn gpio48(&mut self) -> gpio48::W<'_> {
            gpio48::W::new(self)
        }
        #[inline(always)]
        pub fn gpio49(&mut self) -> gpio49::W<'_> {
            gpio49::W::new(self)
        }
        #[inline(always)]
        pub fn gpio50(&mut self) -> gpio50::W<'_> {
            gpio50::W::new(self)
        }
        #[inline(always)]
        pub fn gpio51(&mut self) -> gpio51::W<'_> {
            gpio51::W::new(self)
        }
        #[inline(always)]
        pub fn gpio52(&mut self) -> gpio52::W<'_> {
            gpio52::W::new(self)
        }
        #[inline(always)]
        pub fn gpio53(&mut self) -> gpio53::W<'_> {
            gpio53::W::new(self)
        }
    }
    impl AtomicSetW {
        /// GPIO34 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio34(&mut self) -> gpio34::ASW<'_> {
            gpio34::ASW::new(self)
        }
        /// GPIO35 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio35(&mut self) -> gpio35::ASW<'_> {
            gpio35::ASW::new(self)
        }
        /// GPIO36 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio36(&mut self) -> gpio36::ASW<'_> {
            gpio36::ASW::new(self)
        }
        /// GPIO37 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio37(&mut self) -> gpio37::ASW<'_> {
            gpio37::ASW::new(self)
        }
        /// GPIO38 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio38(&mut self) -> gpio38::ASW<'_> {
            gpio38::ASW::new(self)
        }
        /// GPIO39 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio39(&mut self) -> gpio39::ASW<'_> {
            gpio39::ASW::new(self)
        }
        /// GPIO40 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio40(&mut self) -> gpio40::ASW<'_> {
            gpio40::ASW::new(self)
        }
        /// GPIO41 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio41(&mut self) -> gpio41::ASW<'_> {
            gpio41::ASW::new(self)
        }
        /// GPIO42 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio42(&mut self) -> gpio42::ASW<'_> {
            gpio42::ASW::new(self)
        }
        /// GPIO43 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio43(&mut self) -> gpio43::ASW<'_> {
            gpio43::ASW::new(self)
        }
        /// GPIO44 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio44(&mut self) -> gpio44::ASW<'_> {
            gpio44::ASW::new(self)
        }
        /// GPIO45 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio45(&mut self) -> gpio45::ASW<'_> {
            gpio45::ASW::new(self)
        }
        /// GPIO46 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio46(&mut self) -> gpio46::ASW<'_> {
            gpio46::ASW::new(self)
        }
        /// GPIO47 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio47(&mut self) -> gpio47::ASW<'_> {
            gpio47::ASW::new(self)
        }
        /// GPIO48 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio48(&mut self) -> gpio48::ASW<'_> {
            gpio48::ASW::new(self)
        }
        /// GPIO49 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio49(&mut self) -> gpio49::ASW<'_> {
            gpio49::ASW::new(self)
        }
        /// GPIO50 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio50(&mut self) -> gpio50::ASW<'_> {
            gpio50::ASW::new(self)
        }
        /// GPIO51 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio51(&mut self) -> gpio51::ASW<'_> {
            gpio51::ASW::new(self)
        }
        /// GPIO52 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio52(&mut self) -> gpio52::ASW<'_> {
            gpio52::ASW::new(self)
        }
        /// GPIO53 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio53(&mut self) -> gpio53::ASW<'_> {
            gpio53::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// GPIO34 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio34(&mut self) -> gpio34::ACW<'_> {
            gpio34::ACW::new(self)
        }
        /// GPIO35 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio35(&mut self) -> gpio35::ACW<'_> {
            gpio35::ACW::new(self)
        }
        /// GPIO36 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio36(&mut self) -> gpio36::ACW<'_> {
            gpio36::ACW::new(self)
        }
        /// GPIO37 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio37(&mut self) -> gpio37::ACW<'_> {
            gpio37::ACW::new(self)
        }
        /// GPIO38 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio38(&mut self) -> gpio38::ACW<'_> {
            gpio38::ACW::new(self)
        }
        /// GPIO39 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio39(&mut self) -> gpio39::ACW<'_> {
            gpio39::ACW::new(self)
        }
        /// GPIO40 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio40(&mut self) -> gpio40::ACW<'_> {
            gpio40::ACW::new(self)
        }
        /// GPIO41 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio41(&mut self) -> gpio41::ACW<'_> {
            gpio41::ACW::new(self)
        }
        /// GPIO42 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio42(&mut self) -> gpio42::ACW<'_> {
            gpio42::ACW::new(self)
        }
        /// GPIO43 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio43(&mut self) -> gpio43::ACW<'_> {
            gpio43::ACW::new(self)
        }
        /// GPIO44 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio44(&mut self) -> gpio44::ACW<'_> {
            gpio44::ACW::new(self)
        }
        /// GPIO45 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio45(&mut self) -> gpio45::ACW<'_> {
            gpio45::ACW::new(self)
        }
        /// GPIO46 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio46(&mut self) -> gpio46::ACW<'_> {
            gpio46::ACW::new(self)
        }
        /// GPIO47 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio47(&mut self) -> gpio47::ACW<'_> {
            gpio47::ACW::new(self)
        }
        /// GPIO48 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio48(&mut self) -> gpio48::ACW<'_> {
            gpio48::ACW::new(self)
        }
        /// GPIO49 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio49(&mut self) -> gpio49::ACW<'_> {
            gpio49::ACW::new(self)
        }
        /// GPIO50 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio50(&mut self) -> gpio50::ACW<'_> {
            gpio50::ACW::new(self)
        }
        /// GPIO51 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio51(&mut self) -> gpio51::ACW<'_> {
            gpio51::ACW::new(self)
        }
        /// GPIO52 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio52(&mut self) -> gpio52::ACW<'_> {
            gpio52::ACW::new(self)
        }
        /// GPIO53 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio53(&mut self) -> gpio53::ACW<'_> {
            gpio53::ACW::new(self)
        }
    }
    impl AtomicXorW {
        /// GPIO34 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio34(&mut self) -> gpio34::AXW<'_> {
            gpio34::AXW::new(self)
        }
        /// GPIO35 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio35(&mut self) -> gpio35::AXW<'_> {
            gpio35::AXW::new(self)
        }
        /// GPIO36 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio36(&mut self) -> gpio36::AXW<'_> {
            gpio36::AXW::new(self)
        }
        /// GPIO37 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio37(&mut self) -> gpio37::AXW<'_> {
            gpio37::AXW::new(self)
        }
        /// GPIO38 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio38(&mut self) -> gpio38::AXW<'_> {
            gpio38::AXW::new(self)
        }
        /// GPIO39 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio39(&mut self) -> gpio39::AXW<'_> {
            gpio39::AXW::new(self)
        }
        /// GPIO40 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio40(&mut self) -> gpio40::AXW<'_> {
            gpio40::AXW::new(self)
        }
        /// GPIO41 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio41(&mut self) -> gpio41::AXW<'_> {
            gpio41::AXW::new(self)
        }
        /// GPIO42 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio42(&mut self) -> gpio42::AXW<'_> {
            gpio42::AXW::new(self)
        }
        /// GPIO43 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio43(&mut self) -> gpio43::AXW<'_> {
            gpio43::AXW::new(self)
        }
        /// GPIO44 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio44(&mut self) -> gpio44::AXW<'_> {
            gpio44::AXW::new(self)
        }
        /// GPIO45 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio45(&mut self) -> gpio45::AXW<'_> {
            gpio45::AXW::new(self)
        }
        /// GPIO46 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio46(&mut self) -> gpio46::AXW<'_> {
            gpio46::AXW::new(self)
        }
        /// GPIO47 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio47(&mut self) -> gpio47::AXW<'_> {
            gpio47::AXW::new(self)
        }
        /// GPIO48 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio48(&mut self) -> gpio48::AXW<'_> {
            gpio48::AXW::new(self)
        }
        /// GPIO49 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio49(&mut self) -> gpio49::AXW<'_> {
            gpio49::AXW::new(self)
        }
        /// GPIO50 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio50(&mut self) -> gpio50::AXW<'_> {
            gpio50::AXW::new(self)
        }
        /// GPIO51 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio51(&mut self) -> gpio51::AXW<'_> {
            gpio51::AXW::new(self)
        }
        /// GPIO52 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio52(&mut self) -> gpio52::AXW<'_> {
            gpio52::AXW::new(self)
        }
        /// GPIO53 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio53(&mut self) -> gpio53::AXW<'_> {
            gpio53::AXW::new(self)
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
    /// GPIO34 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio34 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 0>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 0>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 0>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 0>;
    }
    /// GPIO35 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio35 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 1>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 1>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 1>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 1>;
    }
    /// GPIO36 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio36 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 2>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 2>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 2>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 2>;
    }
    /// GPIO37 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio37 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 3>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 3>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 3>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 3>;
    }
    /// GPIO38 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio38 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 4>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 4>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 4>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 4>;
    }
    /// GPIO39 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio39 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 5>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 5>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 5>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 5>;
    }
    /// GPIO40 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio40 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 6>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 6>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 6>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 6>;
    }
    /// GPIO41 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio41 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 7>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 7>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 7>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 7>;
    }
    /// GPIO42 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio42 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 8>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 8>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 8>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 8>;
    }
    /// GPIO43 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio43 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 9>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 9>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 9>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 9>;
    }
    /// GPIO44 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio44 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 10>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 10>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 10>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 10>;
    }
    /// GPIO45 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio45 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 11>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 11>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 11>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 11>;
    }
    /// GPIO46 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio46 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 12>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 12>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 12>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 12>;
    }
    /// GPIO47 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio47 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 13>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 13>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 13>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 13>;
    }
    /// GPIO48 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio48 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 14>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 14>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 14>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 14>;
    }
    /// GPIO49 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio49 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 15>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 15>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 15>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 15>;
    }
    /// GPIO50 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio50 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 16>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 16>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 16>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 16>;
    }
    /// GPIO51 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio51 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 17>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 17>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 17>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 17>;
    }
    /// GPIO52 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio52 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 18>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 18>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 18>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 18>;
    }
    /// GPIO53 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio53 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 19>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 19>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 19>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 19>;
    }
    impl R {
        #[inline(always)]
        pub fn gpio34(&self) -> gpio34::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio35(&self) -> gpio35::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio36(&self) -> gpio36::R {
            crate::BitReader::new(((self.bits() >> 2) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio37(&self) -> gpio37::R {
            crate::BitReader::new(((self.bits() >> 3) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio38(&self) -> gpio38::R {
            crate::BitReader::new(((self.bits() >> 4) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio39(&self) -> gpio39::R {
            crate::BitReader::new(((self.bits() >> 5) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio40(&self) -> gpio40::R {
            crate::BitReader::new(((self.bits() >> 6) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio41(&self) -> gpio41::R {
            crate::BitReader::new(((self.bits() >> 7) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio42(&self) -> gpio42::R {
            crate::BitReader::new(((self.bits() >> 8) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio43(&self) -> gpio43::R {
            crate::BitReader::new(((self.bits() >> 9) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio44(&self) -> gpio44::R {
            crate::BitReader::new(((self.bits() >> 10) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio45(&self) -> gpio45::R {
            crate::BitReader::new(((self.bits() >> 11) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio46(&self) -> gpio46::R {
            crate::BitReader::new(((self.bits() >> 12) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio47(&self) -> gpio47::R {
            crate::BitReader::new(((self.bits() >> 13) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio48(&self) -> gpio48::R {
            crate::BitReader::new(((self.bits() >> 14) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio49(&self) -> gpio49::R {
            crate::BitReader::new(((self.bits() >> 15) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio50(&self) -> gpio50::R {
            crate::BitReader::new(((self.bits() >> 16) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio51(&self) -> gpio51::R {
            crate::BitReader::new(((self.bits() >> 17) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio52(&self) -> gpio52::R {
            crate::BitReader::new(((self.bits() >> 18) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio53(&self) -> gpio53::R {
            crate::BitReader::new(((self.bits() >> 19) & 1) != 0)
        }
    }
    impl W {
        #[inline(always)]
        pub fn gpio34(&mut self) -> gpio34::W<'_> {
            gpio34::W::new(self)
        }
        #[inline(always)]
        pub fn gpio35(&mut self) -> gpio35::W<'_> {
            gpio35::W::new(self)
        }
        #[inline(always)]
        pub fn gpio36(&mut self) -> gpio36::W<'_> {
            gpio36::W::new(self)
        }
        #[inline(always)]
        pub fn gpio37(&mut self) -> gpio37::W<'_> {
            gpio37::W::new(self)
        }
        #[inline(always)]
        pub fn gpio38(&mut self) -> gpio38::W<'_> {
            gpio38::W::new(self)
        }
        #[inline(always)]
        pub fn gpio39(&mut self) -> gpio39::W<'_> {
            gpio39::W::new(self)
        }
        #[inline(always)]
        pub fn gpio40(&mut self) -> gpio40::W<'_> {
            gpio40::W::new(self)
        }
        #[inline(always)]
        pub fn gpio41(&mut self) -> gpio41::W<'_> {
            gpio41::W::new(self)
        }
        #[inline(always)]
        pub fn gpio42(&mut self) -> gpio42::W<'_> {
            gpio42::W::new(self)
        }
        #[inline(always)]
        pub fn gpio43(&mut self) -> gpio43::W<'_> {
            gpio43::W::new(self)
        }
        #[inline(always)]
        pub fn gpio44(&mut self) -> gpio44::W<'_> {
            gpio44::W::new(self)
        }
        #[inline(always)]
        pub fn gpio45(&mut self) -> gpio45::W<'_> {
            gpio45::W::new(self)
        }
        #[inline(always)]
        pub fn gpio46(&mut self) -> gpio46::W<'_> {
            gpio46::W::new(self)
        }
        #[inline(always)]
        pub fn gpio47(&mut self) -> gpio47::W<'_> {
            gpio47::W::new(self)
        }
        #[inline(always)]
        pub fn gpio48(&mut self) -> gpio48::W<'_> {
            gpio48::W::new(self)
        }
        #[inline(always)]
        pub fn gpio49(&mut self) -> gpio49::W<'_> {
            gpio49::W::new(self)
        }
        #[inline(always)]
        pub fn gpio50(&mut self) -> gpio50::W<'_> {
            gpio50::W::new(self)
        }
        #[inline(always)]
        pub fn gpio51(&mut self) -> gpio51::W<'_> {
            gpio51::W::new(self)
        }
        #[inline(always)]
        pub fn gpio52(&mut self) -> gpio52::W<'_> {
            gpio52::W::new(self)
        }
        #[inline(always)]
        pub fn gpio53(&mut self) -> gpio53::W<'_> {
            gpio53::W::new(self)
        }
    }
    impl AtomicSetW {
        /// GPIO34 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio34(&mut self) -> gpio34::ASW<'_> {
            gpio34::ASW::new(self)
        }
        /// GPIO35 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio35(&mut self) -> gpio35::ASW<'_> {
            gpio35::ASW::new(self)
        }
        /// GPIO36 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio36(&mut self) -> gpio36::ASW<'_> {
            gpio36::ASW::new(self)
        }
        /// GPIO37 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio37(&mut self) -> gpio37::ASW<'_> {
            gpio37::ASW::new(self)
        }
        /// GPIO38 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio38(&mut self) -> gpio38::ASW<'_> {
            gpio38::ASW::new(self)
        }
        /// GPIO39 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio39(&mut self) -> gpio39::ASW<'_> {
            gpio39::ASW::new(self)
        }
        /// GPIO40 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio40(&mut self) -> gpio40::ASW<'_> {
            gpio40::ASW::new(self)
        }
        /// GPIO41 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio41(&mut self) -> gpio41::ASW<'_> {
            gpio41::ASW::new(self)
        }
        /// GPIO42 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio42(&mut self) -> gpio42::ASW<'_> {
            gpio42::ASW::new(self)
        }
        /// GPIO43 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio43(&mut self) -> gpio43::ASW<'_> {
            gpio43::ASW::new(self)
        }
        /// GPIO44 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio44(&mut self) -> gpio44::ASW<'_> {
            gpio44::ASW::new(self)
        }
        /// GPIO45 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio45(&mut self) -> gpio45::ASW<'_> {
            gpio45::ASW::new(self)
        }
        /// GPIO46 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio46(&mut self) -> gpio46::ASW<'_> {
            gpio46::ASW::new(self)
        }
        /// GPIO47 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio47(&mut self) -> gpio47::ASW<'_> {
            gpio47::ASW::new(self)
        }
        /// GPIO48 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio48(&mut self) -> gpio48::ASW<'_> {
            gpio48::ASW::new(self)
        }
        /// GPIO49 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio49(&mut self) -> gpio49::ASW<'_> {
            gpio49::ASW::new(self)
        }
        /// GPIO50 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio50(&mut self) -> gpio50::ASW<'_> {
            gpio50::ASW::new(self)
        }
        /// GPIO51 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio51(&mut self) -> gpio51::ASW<'_> {
            gpio51::ASW::new(self)
        }
        /// GPIO52 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio52(&mut self) -> gpio52::ASW<'_> {
            gpio52::ASW::new(self)
        }
        /// GPIO53 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio53(&mut self) -> gpio53::ASW<'_> {
            gpio53::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// GPIO34 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio34(&mut self) -> gpio34::ACW<'_> {
            gpio34::ACW::new(self)
        }
        /// GPIO35 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio35(&mut self) -> gpio35::ACW<'_> {
            gpio35::ACW::new(self)
        }
        /// GPIO36 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio36(&mut self) -> gpio36::ACW<'_> {
            gpio36::ACW::new(self)
        }
        /// GPIO37 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio37(&mut self) -> gpio37::ACW<'_> {
            gpio37::ACW::new(self)
        }
        /// GPIO38 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio38(&mut self) -> gpio38::ACW<'_> {
            gpio38::ACW::new(self)
        }
        /// GPIO39 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio39(&mut self) -> gpio39::ACW<'_> {
            gpio39::ACW::new(self)
        }
        /// GPIO40 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio40(&mut self) -> gpio40::ACW<'_> {
            gpio40::ACW::new(self)
        }
        /// GPIO41 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio41(&mut self) -> gpio41::ACW<'_> {
            gpio41::ACW::new(self)
        }
        /// GPIO42 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio42(&mut self) -> gpio42::ACW<'_> {
            gpio42::ACW::new(self)
        }
        /// GPIO43 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio43(&mut self) -> gpio43::ACW<'_> {
            gpio43::ACW::new(self)
        }
        /// GPIO44 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio44(&mut self) -> gpio44::ACW<'_> {
            gpio44::ACW::new(self)
        }
        /// GPIO45 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio45(&mut self) -> gpio45::ACW<'_> {
            gpio45::ACW::new(self)
        }
        /// GPIO46 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio46(&mut self) -> gpio46::ACW<'_> {
            gpio46::ACW::new(self)
        }
        /// GPIO47 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio47(&mut self) -> gpio47::ACW<'_> {
            gpio47::ACW::new(self)
        }
        /// GPIO48 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio48(&mut self) -> gpio48::ACW<'_> {
            gpio48::ACW::new(self)
        }
        /// GPIO49 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio49(&mut self) -> gpio49::ACW<'_> {
            gpio49::ACW::new(self)
        }
        /// GPIO50 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio50(&mut self) -> gpio50::ACW<'_> {
            gpio50::ACW::new(self)
        }
        /// GPIO51 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio51(&mut self) -> gpio51::ACW<'_> {
            gpio51::ACW::new(self)
        }
        /// GPIO52 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio52(&mut self) -> gpio52::ACW<'_> {
            gpio52::ACW::new(self)
        }
        /// GPIO53 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio53(&mut self) -> gpio53::ACW<'_> {
            gpio53::ACW::new(self)
        }
    }
    impl AtomicXorW {
        /// GPIO34 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio34(&mut self) -> gpio34::AXW<'_> {
            gpio34::AXW::new(self)
        }
        /// GPIO35 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio35(&mut self) -> gpio35::AXW<'_> {
            gpio35::AXW::new(self)
        }
        /// GPIO36 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio36(&mut self) -> gpio36::AXW<'_> {
            gpio36::AXW::new(self)
        }
        /// GPIO37 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio37(&mut self) -> gpio37::AXW<'_> {
            gpio37::AXW::new(self)
        }
        /// GPIO38 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio38(&mut self) -> gpio38::AXW<'_> {
            gpio38::AXW::new(self)
        }
        /// GPIO39 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio39(&mut self) -> gpio39::AXW<'_> {
            gpio39::AXW::new(self)
        }
        /// GPIO40 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio40(&mut self) -> gpio40::AXW<'_> {
            gpio40::AXW::new(self)
        }
        /// GPIO41 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio41(&mut self) -> gpio41::AXW<'_> {
            gpio41::AXW::new(self)
        }
        /// GPIO42 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio42(&mut self) -> gpio42::AXW<'_> {
            gpio42::AXW::new(self)
        }
        /// GPIO43 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio43(&mut self) -> gpio43::AXW<'_> {
            gpio43::AXW::new(self)
        }
        /// GPIO44 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio44(&mut self) -> gpio44::AXW<'_> {
            gpio44::AXW::new(self)
        }
        /// GPIO45 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio45(&mut self) -> gpio45::AXW<'_> {
            gpio45::AXW::new(self)
        }
        /// GPIO46 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio46(&mut self) -> gpio46::AXW<'_> {
            gpio46::AXW::new(self)
        }
        /// GPIO47 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio47(&mut self) -> gpio47::AXW<'_> {
            gpio47::AXW::new(self)
        }
        /// GPIO48 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio48(&mut self) -> gpio48::AXW<'_> {
            gpio48::AXW::new(self)
        }
        /// GPIO49 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio49(&mut self) -> gpio49::AXW<'_> {
            gpio49::AXW::new(self)
        }
        /// GPIO50 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio50(&mut self) -> gpio50::AXW<'_> {
            gpio50::AXW::new(self)
        }
        /// GPIO51 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio51(&mut self) -> gpio51::AXW<'_> {
            gpio51::AXW::new(self)
        }
        /// GPIO52 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio52(&mut self) -> gpio52::AXW<'_> {
            gpio52::AXW::new(self)
        }
        /// GPIO53 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio53(&mut self) -> gpio53::AXW<'_> {
            gpio53::AXW::new(self)
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
    /// GPIO34 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio34 {
        pub type R = crate::BitReader;
    }
    /// GPIO35 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio35 {
        pub type R = crate::BitReader;
    }
    /// GPIO36 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio36 {
        pub type R = crate::BitReader;
    }
    /// GPIO37 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio37 {
        pub type R = crate::BitReader;
    }
    /// GPIO38 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio38 {
        pub type R = crate::BitReader;
    }
    /// GPIO39 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio39 {
        pub type R = crate::BitReader;
    }
    /// GPIO40 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio40 {
        pub type R = crate::BitReader;
    }
    /// GPIO41 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio41 {
        pub type R = crate::BitReader;
    }
    /// GPIO42 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio42 {
        pub type R = crate::BitReader;
    }
    /// GPIO43 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio43 {
        pub type R = crate::BitReader;
    }
    /// GPIO44 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio44 {
        pub type R = crate::BitReader;
    }
    /// GPIO45 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio45 {
        pub type R = crate::BitReader;
    }
    /// GPIO46 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio46 {
        pub type R = crate::BitReader;
    }
    /// GPIO47 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio47 {
        pub type R = crate::BitReader;
    }
    /// GPIO48 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio48 {
        pub type R = crate::BitReader;
    }
    /// GPIO49 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio49 {
        pub type R = crate::BitReader;
    }
    /// GPIO50 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio50 {
        pub type R = crate::BitReader;
    }
    /// GPIO51 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio51 {
        pub type R = crate::BitReader;
    }
    /// GPIO52 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio52 {
        pub type R = crate::BitReader;
    }
    /// GPIO53 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio53 {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn gpio34(&self) -> gpio34::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio35(&self) -> gpio35::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio36(&self) -> gpio36::R {
            crate::BitReader::new(((self.bits() >> 2) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio37(&self) -> gpio37::R {
            crate::BitReader::new(((self.bits() >> 3) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio38(&self) -> gpio38::R {
            crate::BitReader::new(((self.bits() >> 4) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio39(&self) -> gpio39::R {
            crate::BitReader::new(((self.bits() >> 5) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio40(&self) -> gpio40::R {
            crate::BitReader::new(((self.bits() >> 6) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio41(&self) -> gpio41::R {
            crate::BitReader::new(((self.bits() >> 7) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio42(&self) -> gpio42::R {
            crate::BitReader::new(((self.bits() >> 8) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio43(&self) -> gpio43::R {
            crate::BitReader::new(((self.bits() >> 9) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio44(&self) -> gpio44::R {
            crate::BitReader::new(((self.bits() >> 10) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio45(&self) -> gpio45::R {
            crate::BitReader::new(((self.bits() >> 11) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio46(&self) -> gpio46::R {
            crate::BitReader::new(((self.bits() >> 12) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio47(&self) -> gpio47::R {
            crate::BitReader::new(((self.bits() >> 13) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio48(&self) -> gpio48::R {
            crate::BitReader::new(((self.bits() >> 14) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio49(&self) -> gpio49::R {
            crate::BitReader::new(((self.bits() >> 15) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio50(&self) -> gpio50::R {
            crate::BitReader::new(((self.bits() >> 16) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio51(&self) -> gpio51::R {
            crate::BitReader::new(((self.bits() >> 17) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio52(&self) -> gpio52::R {
            crate::BitReader::new(((self.bits() >> 18) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio53(&self) -> gpio53::R {
            crate::BitReader::new(((self.bits() >> 19) & 1) != 0)
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
    /// GPIO34 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio34 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 0>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 0>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 0>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 0>;
    }
    /// GPIO35 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio35 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 1>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 1>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 1>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 1>;
    }
    /// GPIO36 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio36 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 2>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 2>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 2>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 2>;
    }
    /// GPIO37 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio37 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 3>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 3>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 3>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 3>;
    }
    /// GPIO38 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio38 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 4>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 4>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 4>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 4>;
    }
    /// GPIO39 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio39 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 5>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 5>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 5>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 5>;
    }
    /// GPIO40 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio40 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 6>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 6>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 6>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 6>;
    }
    /// GPIO41 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio41 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 7>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 7>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 7>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 7>;
    }
    /// GPIO42 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio42 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 8>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 8>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 8>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 8>;
    }
    /// GPIO43 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio43 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 9>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 9>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 9>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 9>;
    }
    /// GPIO44 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio44 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 10>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 10>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 10>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 10>;
    }
    /// GPIO45 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio45 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 11>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 11>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 11>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 11>;
    }
    /// GPIO46 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio46 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 12>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 12>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 12>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 12>;
    }
    /// GPIO47 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio47 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 13>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 13>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 13>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 13>;
    }
    /// GPIO48 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio48 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 14>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 14>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 14>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 14>;
    }
    /// GPIO49 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio49 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 15>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 15>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 15>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 15>;
    }
    /// GPIO50 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio50 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 16>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 16>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 16>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 16>;
    }
    /// GPIO51 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio51 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 17>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 17>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 17>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 17>;
    }
    /// GPIO52 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio52 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 18>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 18>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 18>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 18>;
    }
    /// GPIO53 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio53 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 19>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 19>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 19>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 19>;
    }
    impl R {
        #[inline(always)]
        pub fn gpio34(&self) -> gpio34::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio35(&self) -> gpio35::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio36(&self) -> gpio36::R {
            crate::BitReader::new(((self.bits() >> 2) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio37(&self) -> gpio37::R {
            crate::BitReader::new(((self.bits() >> 3) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio38(&self) -> gpio38::R {
            crate::BitReader::new(((self.bits() >> 4) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio39(&self) -> gpio39::R {
            crate::BitReader::new(((self.bits() >> 5) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio40(&self) -> gpio40::R {
            crate::BitReader::new(((self.bits() >> 6) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio41(&self) -> gpio41::R {
            crate::BitReader::new(((self.bits() >> 7) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio42(&self) -> gpio42::R {
            crate::BitReader::new(((self.bits() >> 8) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio43(&self) -> gpio43::R {
            crate::BitReader::new(((self.bits() >> 9) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio44(&self) -> gpio44::R {
            crate::BitReader::new(((self.bits() >> 10) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio45(&self) -> gpio45::R {
            crate::BitReader::new(((self.bits() >> 11) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio46(&self) -> gpio46::R {
            crate::BitReader::new(((self.bits() >> 12) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio47(&self) -> gpio47::R {
            crate::BitReader::new(((self.bits() >> 13) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio48(&self) -> gpio48::R {
            crate::BitReader::new(((self.bits() >> 14) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio49(&self) -> gpio49::R {
            crate::BitReader::new(((self.bits() >> 15) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio50(&self) -> gpio50::R {
            crate::BitReader::new(((self.bits() >> 16) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio51(&self) -> gpio51::R {
            crate::BitReader::new(((self.bits() >> 17) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio52(&self) -> gpio52::R {
            crate::BitReader::new(((self.bits() >> 18) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio53(&self) -> gpio53::R {
            crate::BitReader::new(((self.bits() >> 19) & 1) != 0)
        }
    }
    impl W {
        #[inline(always)]
        pub fn gpio34(&mut self) -> gpio34::W<'_> {
            gpio34::W::new(self)
        }
        #[inline(always)]
        pub fn gpio35(&mut self) -> gpio35::W<'_> {
            gpio35::W::new(self)
        }
        #[inline(always)]
        pub fn gpio36(&mut self) -> gpio36::W<'_> {
            gpio36::W::new(self)
        }
        #[inline(always)]
        pub fn gpio37(&mut self) -> gpio37::W<'_> {
            gpio37::W::new(self)
        }
        #[inline(always)]
        pub fn gpio38(&mut self) -> gpio38::W<'_> {
            gpio38::W::new(self)
        }
        #[inline(always)]
        pub fn gpio39(&mut self) -> gpio39::W<'_> {
            gpio39::W::new(self)
        }
        #[inline(always)]
        pub fn gpio40(&mut self) -> gpio40::W<'_> {
            gpio40::W::new(self)
        }
        #[inline(always)]
        pub fn gpio41(&mut self) -> gpio41::W<'_> {
            gpio41::W::new(self)
        }
        #[inline(always)]
        pub fn gpio42(&mut self) -> gpio42::W<'_> {
            gpio42::W::new(self)
        }
        #[inline(always)]
        pub fn gpio43(&mut self) -> gpio43::W<'_> {
            gpio43::W::new(self)
        }
        #[inline(always)]
        pub fn gpio44(&mut self) -> gpio44::W<'_> {
            gpio44::W::new(self)
        }
        #[inline(always)]
        pub fn gpio45(&mut self) -> gpio45::W<'_> {
            gpio45::W::new(self)
        }
        #[inline(always)]
        pub fn gpio46(&mut self) -> gpio46::W<'_> {
            gpio46::W::new(self)
        }
        #[inline(always)]
        pub fn gpio47(&mut self) -> gpio47::W<'_> {
            gpio47::W::new(self)
        }
        #[inline(always)]
        pub fn gpio48(&mut self) -> gpio48::W<'_> {
            gpio48::W::new(self)
        }
        #[inline(always)]
        pub fn gpio49(&mut self) -> gpio49::W<'_> {
            gpio49::W::new(self)
        }
        #[inline(always)]
        pub fn gpio50(&mut self) -> gpio50::W<'_> {
            gpio50::W::new(self)
        }
        #[inline(always)]
        pub fn gpio51(&mut self) -> gpio51::W<'_> {
            gpio51::W::new(self)
        }
        #[inline(always)]
        pub fn gpio52(&mut self) -> gpio52::W<'_> {
            gpio52::W::new(self)
        }
        #[inline(always)]
        pub fn gpio53(&mut self) -> gpio53::W<'_> {
            gpio53::W::new(self)
        }
    }
    impl AtomicSetW {
        /// GPIO34 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio34(&mut self) -> gpio34::ASW<'_> {
            gpio34::ASW::new(self)
        }
        /// GPIO35 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio35(&mut self) -> gpio35::ASW<'_> {
            gpio35::ASW::new(self)
        }
        /// GPIO36 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio36(&mut self) -> gpio36::ASW<'_> {
            gpio36::ASW::new(self)
        }
        /// GPIO37 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio37(&mut self) -> gpio37::ASW<'_> {
            gpio37::ASW::new(self)
        }
        /// GPIO38 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio38(&mut self) -> gpio38::ASW<'_> {
            gpio38::ASW::new(self)
        }
        /// GPIO39 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio39(&mut self) -> gpio39::ASW<'_> {
            gpio39::ASW::new(self)
        }
        /// GPIO40 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio40(&mut self) -> gpio40::ASW<'_> {
            gpio40::ASW::new(self)
        }
        /// GPIO41 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio41(&mut self) -> gpio41::ASW<'_> {
            gpio41::ASW::new(self)
        }
        /// GPIO42 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio42(&mut self) -> gpio42::ASW<'_> {
            gpio42::ASW::new(self)
        }
        /// GPIO43 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio43(&mut self) -> gpio43::ASW<'_> {
            gpio43::ASW::new(self)
        }
        /// GPIO44 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio44(&mut self) -> gpio44::ASW<'_> {
            gpio44::ASW::new(self)
        }
        /// GPIO45 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio45(&mut self) -> gpio45::ASW<'_> {
            gpio45::ASW::new(self)
        }
        /// GPIO46 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio46(&mut self) -> gpio46::ASW<'_> {
            gpio46::ASW::new(self)
        }
        /// GPIO47 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio47(&mut self) -> gpio47::ASW<'_> {
            gpio47::ASW::new(self)
        }
        /// GPIO48 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio48(&mut self) -> gpio48::ASW<'_> {
            gpio48::ASW::new(self)
        }
        /// GPIO49 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio49(&mut self) -> gpio49::ASW<'_> {
            gpio49::ASW::new(self)
        }
        /// GPIO50 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio50(&mut self) -> gpio50::ASW<'_> {
            gpio50::ASW::new(self)
        }
        /// GPIO51 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio51(&mut self) -> gpio51::ASW<'_> {
            gpio51::ASW::new(self)
        }
        /// GPIO52 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio52(&mut self) -> gpio52::ASW<'_> {
            gpio52::ASW::new(self)
        }
        /// GPIO53 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio53(&mut self) -> gpio53::ASW<'_> {
            gpio53::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// GPIO34 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio34(&mut self) -> gpio34::ACW<'_> {
            gpio34::ACW::new(self)
        }
        /// GPIO35 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio35(&mut self) -> gpio35::ACW<'_> {
            gpio35::ACW::new(self)
        }
        /// GPIO36 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio36(&mut self) -> gpio36::ACW<'_> {
            gpio36::ACW::new(self)
        }
        /// GPIO37 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio37(&mut self) -> gpio37::ACW<'_> {
            gpio37::ACW::new(self)
        }
        /// GPIO38 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio38(&mut self) -> gpio38::ACW<'_> {
            gpio38::ACW::new(self)
        }
        /// GPIO39 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio39(&mut self) -> gpio39::ACW<'_> {
            gpio39::ACW::new(self)
        }
        /// GPIO40 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio40(&mut self) -> gpio40::ACW<'_> {
            gpio40::ACW::new(self)
        }
        /// GPIO41 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio41(&mut self) -> gpio41::ACW<'_> {
            gpio41::ACW::new(self)
        }
        /// GPIO42 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio42(&mut self) -> gpio42::ACW<'_> {
            gpio42::ACW::new(self)
        }
        /// GPIO43 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio43(&mut self) -> gpio43::ACW<'_> {
            gpio43::ACW::new(self)
        }
        /// GPIO44 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio44(&mut self) -> gpio44::ACW<'_> {
            gpio44::ACW::new(self)
        }
        /// GPIO45 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio45(&mut self) -> gpio45::ACW<'_> {
            gpio45::ACW::new(self)
        }
        /// GPIO46 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio46(&mut self) -> gpio46::ACW<'_> {
            gpio46::ACW::new(self)
        }
        /// GPIO47 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio47(&mut self) -> gpio47::ACW<'_> {
            gpio47::ACW::new(self)
        }
        /// GPIO48 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio48(&mut self) -> gpio48::ACW<'_> {
            gpio48::ACW::new(self)
        }
        /// GPIO49 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio49(&mut self) -> gpio49::ACW<'_> {
            gpio49::ACW::new(self)
        }
        /// GPIO50 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio50(&mut self) -> gpio50::ACW<'_> {
            gpio50::ACW::new(self)
        }
        /// GPIO51 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio51(&mut self) -> gpio51::ACW<'_> {
            gpio51::ACW::new(self)
        }
        /// GPIO52 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio52(&mut self) -> gpio52::ACW<'_> {
            gpio52::ACW::new(self)
        }
        /// GPIO53 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio53(&mut self) -> gpio53::ACW<'_> {
            gpio53::ACW::new(self)
        }
    }
    impl AtomicXorW {
        /// GPIO34 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio34(&mut self) -> gpio34::AXW<'_> {
            gpio34::AXW::new(self)
        }
        /// GPIO35 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio35(&mut self) -> gpio35::AXW<'_> {
            gpio35::AXW::new(self)
        }
        /// GPIO36 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio36(&mut self) -> gpio36::AXW<'_> {
            gpio36::AXW::new(self)
        }
        /// GPIO37 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio37(&mut self) -> gpio37::AXW<'_> {
            gpio37::AXW::new(self)
        }
        /// GPIO38 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio38(&mut self) -> gpio38::AXW<'_> {
            gpio38::AXW::new(self)
        }
        /// GPIO39 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio39(&mut self) -> gpio39::AXW<'_> {
            gpio39::AXW::new(self)
        }
        /// GPIO40 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio40(&mut self) -> gpio40::AXW<'_> {
            gpio40::AXW::new(self)
        }
        /// GPIO41 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio41(&mut self) -> gpio41::AXW<'_> {
            gpio41::AXW::new(self)
        }
        /// GPIO42 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio42(&mut self) -> gpio42::AXW<'_> {
            gpio42::AXW::new(self)
        }
        /// GPIO43 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio43(&mut self) -> gpio43::AXW<'_> {
            gpio43::AXW::new(self)
        }
        /// GPIO44 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio44(&mut self) -> gpio44::AXW<'_> {
            gpio44::AXW::new(self)
        }
        /// GPIO45 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio45(&mut self) -> gpio45::AXW<'_> {
            gpio45::AXW::new(self)
        }
        /// GPIO46 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio46(&mut self) -> gpio46::AXW<'_> {
            gpio46::AXW::new(self)
        }
        /// GPIO47 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio47(&mut self) -> gpio47::AXW<'_> {
            gpio47::AXW::new(self)
        }
        /// GPIO48 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio48(&mut self) -> gpio48::AXW<'_> {
            gpio48::AXW::new(self)
        }
        /// GPIO49 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio49(&mut self) -> gpio49::AXW<'_> {
            gpio49::AXW::new(self)
        }
        /// GPIO50 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio50(&mut self) -> gpio50::AXW<'_> {
            gpio50::AXW::new(self)
        }
        /// GPIO51 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio51(&mut self) -> gpio51::AXW<'_> {
            gpio51::AXW::new(self)
        }
        /// GPIO52 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio52(&mut self) -> gpio52::AXW<'_> {
            gpio52::AXW::new(self)
        }
        /// GPIO53 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio53(&mut self) -> gpio53::AXW<'_> {
            gpio53::AXW::new(self)
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
    /// GPIO34 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio34 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 0>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 0>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 0>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 0>;
    }
    /// GPIO35 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio35 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 1>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 1>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 1>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 1>;
    }
    /// GPIO36 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio36 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 2>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 2>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 2>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 2>;
    }
    /// GPIO37 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio37 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 3>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 3>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 3>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 3>;
    }
    /// GPIO38 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio38 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 4>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 4>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 4>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 4>;
    }
    /// GPIO39 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio39 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 5>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 5>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 5>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 5>;
    }
    /// GPIO40 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio40 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 6>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 6>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 6>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 6>;
    }
    /// GPIO41 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio41 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 7>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 7>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 7>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 7>;
    }
    /// GPIO42 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio42 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 8>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 8>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 8>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 8>;
    }
    /// GPIO43 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio43 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 9>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 9>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 9>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 9>;
    }
    /// GPIO44 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio44 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 10>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 10>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 10>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 10>;
    }
    /// GPIO45 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio45 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 11>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 11>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 11>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 11>;
    }
    /// GPIO46 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio46 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 12>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 12>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 12>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 12>;
    }
    /// GPIO47 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio47 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 13>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 13>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 13>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 13>;
    }
    /// GPIO48 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio48 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 14>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 14>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 14>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 14>;
    }
    /// GPIO49 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio49 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 15>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 15>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 15>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 15>;
    }
    /// GPIO50 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio50 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 16>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 16>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 16>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 16>;
    }
    /// GPIO51 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio51 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 17>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 17>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 17>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 17>;
    }
    /// GPIO52 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio52 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 18>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 18>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 18>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 18>;
    }
    /// GPIO53 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio53 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 19>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 19>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 19>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 19>;
    }
    impl R {
        #[inline(always)]
        pub fn gpio34(&self) -> gpio34::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio35(&self) -> gpio35::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio36(&self) -> gpio36::R {
            crate::BitReader::new(((self.bits() >> 2) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio37(&self) -> gpio37::R {
            crate::BitReader::new(((self.bits() >> 3) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio38(&self) -> gpio38::R {
            crate::BitReader::new(((self.bits() >> 4) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio39(&self) -> gpio39::R {
            crate::BitReader::new(((self.bits() >> 5) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio40(&self) -> gpio40::R {
            crate::BitReader::new(((self.bits() >> 6) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio41(&self) -> gpio41::R {
            crate::BitReader::new(((self.bits() >> 7) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio42(&self) -> gpio42::R {
            crate::BitReader::new(((self.bits() >> 8) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio43(&self) -> gpio43::R {
            crate::BitReader::new(((self.bits() >> 9) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio44(&self) -> gpio44::R {
            crate::BitReader::new(((self.bits() >> 10) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio45(&self) -> gpio45::R {
            crate::BitReader::new(((self.bits() >> 11) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio46(&self) -> gpio46::R {
            crate::BitReader::new(((self.bits() >> 12) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio47(&self) -> gpio47::R {
            crate::BitReader::new(((self.bits() >> 13) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio48(&self) -> gpio48::R {
            crate::BitReader::new(((self.bits() >> 14) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio49(&self) -> gpio49::R {
            crate::BitReader::new(((self.bits() >> 15) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio50(&self) -> gpio50::R {
            crate::BitReader::new(((self.bits() >> 16) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio51(&self) -> gpio51::R {
            crate::BitReader::new(((self.bits() >> 17) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio52(&self) -> gpio52::R {
            crate::BitReader::new(((self.bits() >> 18) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio53(&self) -> gpio53::R {
            crate::BitReader::new(((self.bits() >> 19) & 1) != 0)
        }
    }
    impl W {
        #[inline(always)]
        pub fn gpio34(&mut self) -> gpio34::W<'_> {
            gpio34::W::new(self)
        }
        #[inline(always)]
        pub fn gpio35(&mut self) -> gpio35::W<'_> {
            gpio35::W::new(self)
        }
        #[inline(always)]
        pub fn gpio36(&mut self) -> gpio36::W<'_> {
            gpio36::W::new(self)
        }
        #[inline(always)]
        pub fn gpio37(&mut self) -> gpio37::W<'_> {
            gpio37::W::new(self)
        }
        #[inline(always)]
        pub fn gpio38(&mut self) -> gpio38::W<'_> {
            gpio38::W::new(self)
        }
        #[inline(always)]
        pub fn gpio39(&mut self) -> gpio39::W<'_> {
            gpio39::W::new(self)
        }
        #[inline(always)]
        pub fn gpio40(&mut self) -> gpio40::W<'_> {
            gpio40::W::new(self)
        }
        #[inline(always)]
        pub fn gpio41(&mut self) -> gpio41::W<'_> {
            gpio41::W::new(self)
        }
        #[inline(always)]
        pub fn gpio42(&mut self) -> gpio42::W<'_> {
            gpio42::W::new(self)
        }
        #[inline(always)]
        pub fn gpio43(&mut self) -> gpio43::W<'_> {
            gpio43::W::new(self)
        }
        #[inline(always)]
        pub fn gpio44(&mut self) -> gpio44::W<'_> {
            gpio44::W::new(self)
        }
        #[inline(always)]
        pub fn gpio45(&mut self) -> gpio45::W<'_> {
            gpio45::W::new(self)
        }
        #[inline(always)]
        pub fn gpio46(&mut self) -> gpio46::W<'_> {
            gpio46::W::new(self)
        }
        #[inline(always)]
        pub fn gpio47(&mut self) -> gpio47::W<'_> {
            gpio47::W::new(self)
        }
        #[inline(always)]
        pub fn gpio48(&mut self) -> gpio48::W<'_> {
            gpio48::W::new(self)
        }
        #[inline(always)]
        pub fn gpio49(&mut self) -> gpio49::W<'_> {
            gpio49::W::new(self)
        }
        #[inline(always)]
        pub fn gpio50(&mut self) -> gpio50::W<'_> {
            gpio50::W::new(self)
        }
        #[inline(always)]
        pub fn gpio51(&mut self) -> gpio51::W<'_> {
            gpio51::W::new(self)
        }
        #[inline(always)]
        pub fn gpio52(&mut self) -> gpio52::W<'_> {
            gpio52::W::new(self)
        }
        #[inline(always)]
        pub fn gpio53(&mut self) -> gpio53::W<'_> {
            gpio53::W::new(self)
        }
    }
    impl AtomicSetW {
        /// GPIO34 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio34(&mut self) -> gpio34::ASW<'_> {
            gpio34::ASW::new(self)
        }
        /// GPIO35 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio35(&mut self) -> gpio35::ASW<'_> {
            gpio35::ASW::new(self)
        }
        /// GPIO36 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio36(&mut self) -> gpio36::ASW<'_> {
            gpio36::ASW::new(self)
        }
        /// GPIO37 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio37(&mut self) -> gpio37::ASW<'_> {
            gpio37::ASW::new(self)
        }
        /// GPIO38 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio38(&mut self) -> gpio38::ASW<'_> {
            gpio38::ASW::new(self)
        }
        /// GPIO39 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio39(&mut self) -> gpio39::ASW<'_> {
            gpio39::ASW::new(self)
        }
        /// GPIO40 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio40(&mut self) -> gpio40::ASW<'_> {
            gpio40::ASW::new(self)
        }
        /// GPIO41 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio41(&mut self) -> gpio41::ASW<'_> {
            gpio41::ASW::new(self)
        }
        /// GPIO42 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio42(&mut self) -> gpio42::ASW<'_> {
            gpio42::ASW::new(self)
        }
        /// GPIO43 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio43(&mut self) -> gpio43::ASW<'_> {
            gpio43::ASW::new(self)
        }
        /// GPIO44 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio44(&mut self) -> gpio44::ASW<'_> {
            gpio44::ASW::new(self)
        }
        /// GPIO45 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio45(&mut self) -> gpio45::ASW<'_> {
            gpio45::ASW::new(self)
        }
        /// GPIO46 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio46(&mut self) -> gpio46::ASW<'_> {
            gpio46::ASW::new(self)
        }
        /// GPIO47 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio47(&mut self) -> gpio47::ASW<'_> {
            gpio47::ASW::new(self)
        }
        /// GPIO48 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio48(&mut self) -> gpio48::ASW<'_> {
            gpio48::ASW::new(self)
        }
        /// GPIO49 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio49(&mut self) -> gpio49::ASW<'_> {
            gpio49::ASW::new(self)
        }
        /// GPIO50 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio50(&mut self) -> gpio50::ASW<'_> {
            gpio50::ASW::new(self)
        }
        /// GPIO51 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio51(&mut self) -> gpio51::ASW<'_> {
            gpio51::ASW::new(self)
        }
        /// GPIO52 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio52(&mut self) -> gpio52::ASW<'_> {
            gpio52::ASW::new(self)
        }
        /// GPIO53 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio53(&mut self) -> gpio53::ASW<'_> {
            gpio53::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// GPIO34 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio34(&mut self) -> gpio34::ACW<'_> {
            gpio34::ACW::new(self)
        }
        /// GPIO35 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio35(&mut self) -> gpio35::ACW<'_> {
            gpio35::ACW::new(self)
        }
        /// GPIO36 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio36(&mut self) -> gpio36::ACW<'_> {
            gpio36::ACW::new(self)
        }
        /// GPIO37 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio37(&mut self) -> gpio37::ACW<'_> {
            gpio37::ACW::new(self)
        }
        /// GPIO38 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio38(&mut self) -> gpio38::ACW<'_> {
            gpio38::ACW::new(self)
        }
        /// GPIO39 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio39(&mut self) -> gpio39::ACW<'_> {
            gpio39::ACW::new(self)
        }
        /// GPIO40 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio40(&mut self) -> gpio40::ACW<'_> {
            gpio40::ACW::new(self)
        }
        /// GPIO41 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio41(&mut self) -> gpio41::ACW<'_> {
            gpio41::ACW::new(self)
        }
        /// GPIO42 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio42(&mut self) -> gpio42::ACW<'_> {
            gpio42::ACW::new(self)
        }
        /// GPIO43 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio43(&mut self) -> gpio43::ACW<'_> {
            gpio43::ACW::new(self)
        }
        /// GPIO44 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio44(&mut self) -> gpio44::ACW<'_> {
            gpio44::ACW::new(self)
        }
        /// GPIO45 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio45(&mut self) -> gpio45::ACW<'_> {
            gpio45::ACW::new(self)
        }
        /// GPIO46 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio46(&mut self) -> gpio46::ACW<'_> {
            gpio46::ACW::new(self)
        }
        /// GPIO47 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio47(&mut self) -> gpio47::ACW<'_> {
            gpio47::ACW::new(self)
        }
        /// GPIO48 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio48(&mut self) -> gpio48::ACW<'_> {
            gpio48::ACW::new(self)
        }
        /// GPIO49 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio49(&mut self) -> gpio49::ACW<'_> {
            gpio49::ACW::new(self)
        }
        /// GPIO50 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio50(&mut self) -> gpio50::ACW<'_> {
            gpio50::ACW::new(self)
        }
        /// GPIO51 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio51(&mut self) -> gpio51::ACW<'_> {
            gpio51::ACW::new(self)
        }
        /// GPIO52 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio52(&mut self) -> gpio52::ACW<'_> {
            gpio52::ACW::new(self)
        }
        /// GPIO53 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio53(&mut self) -> gpio53::ACW<'_> {
            gpio53::ACW::new(self)
        }
    }
    impl AtomicXorW {
        /// GPIO34 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio34(&mut self) -> gpio34::AXW<'_> {
            gpio34::AXW::new(self)
        }
        /// GPIO35 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio35(&mut self) -> gpio35::AXW<'_> {
            gpio35::AXW::new(self)
        }
        /// GPIO36 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio36(&mut self) -> gpio36::AXW<'_> {
            gpio36::AXW::new(self)
        }
        /// GPIO37 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio37(&mut self) -> gpio37::AXW<'_> {
            gpio37::AXW::new(self)
        }
        /// GPIO38 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio38(&mut self) -> gpio38::AXW<'_> {
            gpio38::AXW::new(self)
        }
        /// GPIO39 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio39(&mut self) -> gpio39::AXW<'_> {
            gpio39::AXW::new(self)
        }
        /// GPIO40 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio40(&mut self) -> gpio40::AXW<'_> {
            gpio40::AXW::new(self)
        }
        /// GPIO41 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio41(&mut self) -> gpio41::AXW<'_> {
            gpio41::AXW::new(self)
        }
        /// GPIO42 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio42(&mut self) -> gpio42::AXW<'_> {
            gpio42::AXW::new(self)
        }
        /// GPIO43 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio43(&mut self) -> gpio43::AXW<'_> {
            gpio43::AXW::new(self)
        }
        /// GPIO44 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio44(&mut self) -> gpio44::AXW<'_> {
            gpio44::AXW::new(self)
        }
        /// GPIO45 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio45(&mut self) -> gpio45::AXW<'_> {
            gpio45::AXW::new(self)
        }
        /// GPIO46 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio46(&mut self) -> gpio46::AXW<'_> {
            gpio46::AXW::new(self)
        }
        /// GPIO47 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio47(&mut self) -> gpio47::AXW<'_> {
            gpio47::AXW::new(self)
        }
        /// GPIO48 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio48(&mut self) -> gpio48::AXW<'_> {
            gpio48::AXW::new(self)
        }
        /// GPIO49 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio49(&mut self) -> gpio49::AXW<'_> {
            gpio49::AXW::new(self)
        }
        /// GPIO50 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio50(&mut self) -> gpio50::AXW<'_> {
            gpio50::AXW::new(self)
        }
        /// GPIO51 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio51(&mut self) -> gpio51::AXW<'_> {
            gpio51::AXW::new(self)
        }
        /// GPIO52 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio52(&mut self) -> gpio52::AXW<'_> {
            gpio52::AXW::new(self)
        }
        /// GPIO53 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio53(&mut self) -> gpio53::AXW<'_> {
            gpio53::AXW::new(self)
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
    /// GPIO34 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio34 {
        pub type R = crate::BitReader;
    }
    /// GPIO35 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio35 {
        pub type R = crate::BitReader;
    }
    /// GPIO36 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio36 {
        pub type R = crate::BitReader;
    }
    /// GPIO37 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio37 {
        pub type R = crate::BitReader;
    }
    /// GPIO38 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio38 {
        pub type R = crate::BitReader;
    }
    /// GPIO39 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio39 {
        pub type R = crate::BitReader;
    }
    /// GPIO40 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio40 {
        pub type R = crate::BitReader;
    }
    /// GPIO41 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio41 {
        pub type R = crate::BitReader;
    }
    /// GPIO42 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio42 {
        pub type R = crate::BitReader;
    }
    /// GPIO43 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio43 {
        pub type R = crate::BitReader;
    }
    /// GPIO44 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio44 {
        pub type R = crate::BitReader;
    }
    /// GPIO45 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio45 {
        pub type R = crate::BitReader;
    }
    /// GPIO46 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio46 {
        pub type R = crate::BitReader;
    }
    /// GPIO47 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio47 {
        pub type R = crate::BitReader;
    }
    /// GPIO48 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio48 {
        pub type R = crate::BitReader;
    }
    /// GPIO49 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio49 {
        pub type R = crate::BitReader;
    }
    /// GPIO50 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio50 {
        pub type R = crate::BitReader;
    }
    /// GPIO51 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio51 {
        pub type R = crate::BitReader;
    }
    /// GPIO52 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio52 {
        pub type R = crate::BitReader;
    }
    /// GPIO53 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio53 {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn gpio34(&self) -> gpio34::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio35(&self) -> gpio35::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio36(&self) -> gpio36::R {
            crate::BitReader::new(((self.bits() >> 2) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio37(&self) -> gpio37::R {
            crate::BitReader::new(((self.bits() >> 3) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio38(&self) -> gpio38::R {
            crate::BitReader::new(((self.bits() >> 4) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio39(&self) -> gpio39::R {
            crate::BitReader::new(((self.bits() >> 5) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio40(&self) -> gpio40::R {
            crate::BitReader::new(((self.bits() >> 6) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio41(&self) -> gpio41::R {
            crate::BitReader::new(((self.bits() >> 7) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio42(&self) -> gpio42::R {
            crate::BitReader::new(((self.bits() >> 8) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio43(&self) -> gpio43::R {
            crate::BitReader::new(((self.bits() >> 9) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio44(&self) -> gpio44::R {
            crate::BitReader::new(((self.bits() >> 10) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio45(&self) -> gpio45::R {
            crate::BitReader::new(((self.bits() >> 11) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio46(&self) -> gpio46::R {
            crate::BitReader::new(((self.bits() >> 12) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio47(&self) -> gpio47::R {
            crate::BitReader::new(((self.bits() >> 13) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio48(&self) -> gpio48::R {
            crate::BitReader::new(((self.bits() >> 14) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio49(&self) -> gpio49::R {
            crate::BitReader::new(((self.bits() >> 15) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio50(&self) -> gpio50::R {
            crate::BitReader::new(((self.bits() >> 16) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio51(&self) -> gpio51::R {
            crate::BitReader::new(((self.bits() >> 17) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio52(&self) -> gpio52::R {
            crate::BitReader::new(((self.bits() >> 18) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio53(&self) -> gpio53::R {
            crate::BitReader::new(((self.bits() >> 19) & 1) != 0)
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
    /// GPIO34 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio34 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 0>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 0>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 0>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 0>;
    }
    /// GPIO35 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio35 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 1>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 1>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 1>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 1>;
    }
    /// GPIO36 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio36 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 2>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 2>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 2>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 2>;
    }
    /// GPIO37 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio37 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 3>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 3>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 3>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 3>;
    }
    /// GPIO38 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio38 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 4>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 4>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 4>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 4>;
    }
    /// GPIO39 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio39 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 5>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 5>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 5>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 5>;
    }
    /// GPIO40 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio40 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 6>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 6>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 6>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 6>;
    }
    /// GPIO41 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio41 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 7>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 7>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 7>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 7>;
    }
    /// GPIO42 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio42 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 8>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 8>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 8>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 8>;
    }
    /// GPIO43 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio43 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 9>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 9>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 9>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 9>;
    }
    /// GPIO44 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio44 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 10>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 10>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 10>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 10>;
    }
    /// GPIO45 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio45 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 11>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 11>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 11>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 11>;
    }
    /// GPIO46 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio46 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 12>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 12>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 12>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 12>;
    }
    /// GPIO47 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio47 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 13>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 13>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 13>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 13>;
    }
    /// GPIO48 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio48 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 14>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 14>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 14>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 14>;
    }
    /// GPIO49 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio49 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 15>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 15>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 15>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 15>;
    }
    /// GPIO50 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio50 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 16>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 16>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 16>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 16>;
    }
    /// GPIO51 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio51 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 17>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 17>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 17>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 17>;
    }
    /// GPIO52 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio52 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 18>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 18>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 18>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 18>;
    }
    /// GPIO53 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio53 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 19>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 19>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 19>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 19>;
    }
    impl R {
        #[inline(always)]
        pub fn gpio34(&self) -> gpio34::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio35(&self) -> gpio35::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio36(&self) -> gpio36::R {
            crate::BitReader::new(((self.bits() >> 2) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio37(&self) -> gpio37::R {
            crate::BitReader::new(((self.bits() >> 3) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio38(&self) -> gpio38::R {
            crate::BitReader::new(((self.bits() >> 4) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio39(&self) -> gpio39::R {
            crate::BitReader::new(((self.bits() >> 5) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio40(&self) -> gpio40::R {
            crate::BitReader::new(((self.bits() >> 6) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio41(&self) -> gpio41::R {
            crate::BitReader::new(((self.bits() >> 7) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio42(&self) -> gpio42::R {
            crate::BitReader::new(((self.bits() >> 8) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio43(&self) -> gpio43::R {
            crate::BitReader::new(((self.bits() >> 9) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio44(&self) -> gpio44::R {
            crate::BitReader::new(((self.bits() >> 10) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio45(&self) -> gpio45::R {
            crate::BitReader::new(((self.bits() >> 11) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio46(&self) -> gpio46::R {
            crate::BitReader::new(((self.bits() >> 12) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio47(&self) -> gpio47::R {
            crate::BitReader::new(((self.bits() >> 13) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio48(&self) -> gpio48::R {
            crate::BitReader::new(((self.bits() >> 14) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio49(&self) -> gpio49::R {
            crate::BitReader::new(((self.bits() >> 15) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio50(&self) -> gpio50::R {
            crate::BitReader::new(((self.bits() >> 16) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio51(&self) -> gpio51::R {
            crate::BitReader::new(((self.bits() >> 17) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio52(&self) -> gpio52::R {
            crate::BitReader::new(((self.bits() >> 18) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio53(&self) -> gpio53::R {
            crate::BitReader::new(((self.bits() >> 19) & 1) != 0)
        }
    }
    impl W {
        #[inline(always)]
        pub fn gpio34(&mut self) -> gpio34::W<'_> {
            gpio34::W::new(self)
        }
        #[inline(always)]
        pub fn gpio35(&mut self) -> gpio35::W<'_> {
            gpio35::W::new(self)
        }
        #[inline(always)]
        pub fn gpio36(&mut self) -> gpio36::W<'_> {
            gpio36::W::new(self)
        }
        #[inline(always)]
        pub fn gpio37(&mut self) -> gpio37::W<'_> {
            gpio37::W::new(self)
        }
        #[inline(always)]
        pub fn gpio38(&mut self) -> gpio38::W<'_> {
            gpio38::W::new(self)
        }
        #[inline(always)]
        pub fn gpio39(&mut self) -> gpio39::W<'_> {
            gpio39::W::new(self)
        }
        #[inline(always)]
        pub fn gpio40(&mut self) -> gpio40::W<'_> {
            gpio40::W::new(self)
        }
        #[inline(always)]
        pub fn gpio41(&mut self) -> gpio41::W<'_> {
            gpio41::W::new(self)
        }
        #[inline(always)]
        pub fn gpio42(&mut self) -> gpio42::W<'_> {
            gpio42::W::new(self)
        }
        #[inline(always)]
        pub fn gpio43(&mut self) -> gpio43::W<'_> {
            gpio43::W::new(self)
        }
        #[inline(always)]
        pub fn gpio44(&mut self) -> gpio44::W<'_> {
            gpio44::W::new(self)
        }
        #[inline(always)]
        pub fn gpio45(&mut self) -> gpio45::W<'_> {
            gpio45::W::new(self)
        }
        #[inline(always)]
        pub fn gpio46(&mut self) -> gpio46::W<'_> {
            gpio46::W::new(self)
        }
        #[inline(always)]
        pub fn gpio47(&mut self) -> gpio47::W<'_> {
            gpio47::W::new(self)
        }
        #[inline(always)]
        pub fn gpio48(&mut self) -> gpio48::W<'_> {
            gpio48::W::new(self)
        }
        #[inline(always)]
        pub fn gpio49(&mut self) -> gpio49::W<'_> {
            gpio49::W::new(self)
        }
        #[inline(always)]
        pub fn gpio50(&mut self) -> gpio50::W<'_> {
            gpio50::W::new(self)
        }
        #[inline(always)]
        pub fn gpio51(&mut self) -> gpio51::W<'_> {
            gpio51::W::new(self)
        }
        #[inline(always)]
        pub fn gpio52(&mut self) -> gpio52::W<'_> {
            gpio52::W::new(self)
        }
        #[inline(always)]
        pub fn gpio53(&mut self) -> gpio53::W<'_> {
            gpio53::W::new(self)
        }
    }
    impl AtomicSetW {
        /// GPIO34 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio34(&mut self) -> gpio34::ASW<'_> {
            gpio34::ASW::new(self)
        }
        /// GPIO35 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio35(&mut self) -> gpio35::ASW<'_> {
            gpio35::ASW::new(self)
        }
        /// GPIO36 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio36(&mut self) -> gpio36::ASW<'_> {
            gpio36::ASW::new(self)
        }
        /// GPIO37 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio37(&mut self) -> gpio37::ASW<'_> {
            gpio37::ASW::new(self)
        }
        /// GPIO38 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio38(&mut self) -> gpio38::ASW<'_> {
            gpio38::ASW::new(self)
        }
        /// GPIO39 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio39(&mut self) -> gpio39::ASW<'_> {
            gpio39::ASW::new(self)
        }
        /// GPIO40 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio40(&mut self) -> gpio40::ASW<'_> {
            gpio40::ASW::new(self)
        }
        /// GPIO41 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio41(&mut self) -> gpio41::ASW<'_> {
            gpio41::ASW::new(self)
        }
        /// GPIO42 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio42(&mut self) -> gpio42::ASW<'_> {
            gpio42::ASW::new(self)
        }
        /// GPIO43 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio43(&mut self) -> gpio43::ASW<'_> {
            gpio43::ASW::new(self)
        }
        /// GPIO44 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio44(&mut self) -> gpio44::ASW<'_> {
            gpio44::ASW::new(self)
        }
        /// GPIO45 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio45(&mut self) -> gpio45::ASW<'_> {
            gpio45::ASW::new(self)
        }
        /// GPIO46 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio46(&mut self) -> gpio46::ASW<'_> {
            gpio46::ASW::new(self)
        }
        /// GPIO47 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio47(&mut self) -> gpio47::ASW<'_> {
            gpio47::ASW::new(self)
        }
        /// GPIO48 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio48(&mut self) -> gpio48::ASW<'_> {
            gpio48::ASW::new(self)
        }
        /// GPIO49 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio49(&mut self) -> gpio49::ASW<'_> {
            gpio49::ASW::new(self)
        }
        /// GPIO50 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio50(&mut self) -> gpio50::ASW<'_> {
            gpio50::ASW::new(self)
        }
        /// GPIO51 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio51(&mut self) -> gpio51::ASW<'_> {
            gpio51::ASW::new(self)
        }
        /// GPIO52 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio52(&mut self) -> gpio52::ASW<'_> {
            gpio52::ASW::new(self)
        }
        /// GPIO53 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio53(&mut self) -> gpio53::ASW<'_> {
            gpio53::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// GPIO34 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio34(&mut self) -> gpio34::ACW<'_> {
            gpio34::ACW::new(self)
        }
        /// GPIO35 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio35(&mut self) -> gpio35::ACW<'_> {
            gpio35::ACW::new(self)
        }
        /// GPIO36 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio36(&mut self) -> gpio36::ACW<'_> {
            gpio36::ACW::new(self)
        }
        /// GPIO37 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio37(&mut self) -> gpio37::ACW<'_> {
            gpio37::ACW::new(self)
        }
        /// GPIO38 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio38(&mut self) -> gpio38::ACW<'_> {
            gpio38::ACW::new(self)
        }
        /// GPIO39 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio39(&mut self) -> gpio39::ACW<'_> {
            gpio39::ACW::new(self)
        }
        /// GPIO40 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio40(&mut self) -> gpio40::ACW<'_> {
            gpio40::ACW::new(self)
        }
        /// GPIO41 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio41(&mut self) -> gpio41::ACW<'_> {
            gpio41::ACW::new(self)
        }
        /// GPIO42 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio42(&mut self) -> gpio42::ACW<'_> {
            gpio42::ACW::new(self)
        }
        /// GPIO43 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio43(&mut self) -> gpio43::ACW<'_> {
            gpio43::ACW::new(self)
        }
        /// GPIO44 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio44(&mut self) -> gpio44::ACW<'_> {
            gpio44::ACW::new(self)
        }
        /// GPIO45 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio45(&mut self) -> gpio45::ACW<'_> {
            gpio45::ACW::new(self)
        }
        /// GPIO46 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio46(&mut self) -> gpio46::ACW<'_> {
            gpio46::ACW::new(self)
        }
        /// GPIO47 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio47(&mut self) -> gpio47::ACW<'_> {
            gpio47::ACW::new(self)
        }
        /// GPIO48 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio48(&mut self) -> gpio48::ACW<'_> {
            gpio48::ACW::new(self)
        }
        /// GPIO49 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio49(&mut self) -> gpio49::ACW<'_> {
            gpio49::ACW::new(self)
        }
        /// GPIO50 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio50(&mut self) -> gpio50::ACW<'_> {
            gpio50::ACW::new(self)
        }
        /// GPIO51 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio51(&mut self) -> gpio51::ACW<'_> {
            gpio51::ACW::new(self)
        }
        /// GPIO52 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio52(&mut self) -> gpio52::ACW<'_> {
            gpio52::ACW::new(self)
        }
        /// GPIO53 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio53(&mut self) -> gpio53::ACW<'_> {
            gpio53::ACW::new(self)
        }
    }
    impl AtomicXorW {
        /// GPIO34 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio34(&mut self) -> gpio34::AXW<'_> {
            gpio34::AXW::new(self)
        }
        /// GPIO35 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio35(&mut self) -> gpio35::AXW<'_> {
            gpio35::AXW::new(self)
        }
        /// GPIO36 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio36(&mut self) -> gpio36::AXW<'_> {
            gpio36::AXW::new(self)
        }
        /// GPIO37 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio37(&mut self) -> gpio37::AXW<'_> {
            gpio37::AXW::new(self)
        }
        /// GPIO38 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio38(&mut self) -> gpio38::AXW<'_> {
            gpio38::AXW::new(self)
        }
        /// GPIO39 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio39(&mut self) -> gpio39::AXW<'_> {
            gpio39::AXW::new(self)
        }
        /// GPIO40 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio40(&mut self) -> gpio40::AXW<'_> {
            gpio40::AXW::new(self)
        }
        /// GPIO41 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio41(&mut self) -> gpio41::AXW<'_> {
            gpio41::AXW::new(self)
        }
        /// GPIO42 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio42(&mut self) -> gpio42::AXW<'_> {
            gpio42::AXW::new(self)
        }
        /// GPIO43 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio43(&mut self) -> gpio43::AXW<'_> {
            gpio43::AXW::new(self)
        }
        /// GPIO44 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio44(&mut self) -> gpio44::AXW<'_> {
            gpio44::AXW::new(self)
        }
        /// GPIO45 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio45(&mut self) -> gpio45::AXW<'_> {
            gpio45::AXW::new(self)
        }
        /// GPIO46 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio46(&mut self) -> gpio46::AXW<'_> {
            gpio46::AXW::new(self)
        }
        /// GPIO47 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio47(&mut self) -> gpio47::AXW<'_> {
            gpio47::AXW::new(self)
        }
        /// GPIO48 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio48(&mut self) -> gpio48::AXW<'_> {
            gpio48::AXW::new(self)
        }
        /// GPIO49 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio49(&mut self) -> gpio49::AXW<'_> {
            gpio49::AXW::new(self)
        }
        /// GPIO50 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio50(&mut self) -> gpio50::AXW<'_> {
            gpio50::AXW::new(self)
        }
        /// GPIO51 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio51(&mut self) -> gpio51::AXW<'_> {
            gpio51::AXW::new(self)
        }
        /// GPIO52 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio52(&mut self) -> gpio52::AXW<'_> {
            gpio52::AXW::new(self)
        }
        /// GPIO53 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio53(&mut self) -> gpio53::AXW<'_> {
            gpio53::AXW::new(self)
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
    /// GPIO34 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio34 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 0>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 0>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 0>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 0>;
    }
    /// GPIO35 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio35 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 1>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 1>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 1>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 1>;
    }
    /// GPIO36 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio36 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 2>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 2>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 2>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 2>;
    }
    /// GPIO37 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio37 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 3>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 3>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 3>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 3>;
    }
    /// GPIO38 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio38 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 4>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 4>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 4>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 4>;
    }
    /// GPIO39 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio39 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 5>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 5>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 5>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 5>;
    }
    /// GPIO40 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio40 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 6>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 6>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 6>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 6>;
    }
    /// GPIO41 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio41 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 7>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 7>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 7>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 7>;
    }
    /// GPIO42 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio42 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 8>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 8>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 8>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 8>;
    }
    /// GPIO43 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio43 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 9>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 9>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 9>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 9>;
    }
    /// GPIO44 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio44 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 10>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 10>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 10>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 10>;
    }
    /// GPIO45 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio45 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 11>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 11>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 11>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 11>;
    }
    /// GPIO46 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio46 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 12>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 12>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 12>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 12>;
    }
    /// GPIO47 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio47 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 13>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 13>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 13>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 13>;
    }
    /// GPIO48 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio48 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 14>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 14>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 14>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 14>;
    }
    /// GPIO49 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio49 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 15>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 15>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 15>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 15>;
    }
    /// GPIO50 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio50 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 16>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 16>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 16>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 16>;
    }
    /// GPIO51 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio51 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 17>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 17>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 17>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 17>;
    }
    /// GPIO52 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio52 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 18>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 18>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 18>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 18>;
    }
    /// GPIO53 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio53 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 19>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 19>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 19>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 19>;
    }
    impl R {
        #[inline(always)]
        pub fn gpio34(&self) -> gpio34::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio35(&self) -> gpio35::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio36(&self) -> gpio36::R {
            crate::BitReader::new(((self.bits() >> 2) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio37(&self) -> gpio37::R {
            crate::BitReader::new(((self.bits() >> 3) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio38(&self) -> gpio38::R {
            crate::BitReader::new(((self.bits() >> 4) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio39(&self) -> gpio39::R {
            crate::BitReader::new(((self.bits() >> 5) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio40(&self) -> gpio40::R {
            crate::BitReader::new(((self.bits() >> 6) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio41(&self) -> gpio41::R {
            crate::BitReader::new(((self.bits() >> 7) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio42(&self) -> gpio42::R {
            crate::BitReader::new(((self.bits() >> 8) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio43(&self) -> gpio43::R {
            crate::BitReader::new(((self.bits() >> 9) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio44(&self) -> gpio44::R {
            crate::BitReader::new(((self.bits() >> 10) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio45(&self) -> gpio45::R {
            crate::BitReader::new(((self.bits() >> 11) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio46(&self) -> gpio46::R {
            crate::BitReader::new(((self.bits() >> 12) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio47(&self) -> gpio47::R {
            crate::BitReader::new(((self.bits() >> 13) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio48(&self) -> gpio48::R {
            crate::BitReader::new(((self.bits() >> 14) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio49(&self) -> gpio49::R {
            crate::BitReader::new(((self.bits() >> 15) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio50(&self) -> gpio50::R {
            crate::BitReader::new(((self.bits() >> 16) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio51(&self) -> gpio51::R {
            crate::BitReader::new(((self.bits() >> 17) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio52(&self) -> gpio52::R {
            crate::BitReader::new(((self.bits() >> 18) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio53(&self) -> gpio53::R {
            crate::BitReader::new(((self.bits() >> 19) & 1) != 0)
        }
    }
    impl W {
        #[inline(always)]
        pub fn gpio34(&mut self) -> gpio34::W<'_> {
            gpio34::W::new(self)
        }
        #[inline(always)]
        pub fn gpio35(&mut self) -> gpio35::W<'_> {
            gpio35::W::new(self)
        }
        #[inline(always)]
        pub fn gpio36(&mut self) -> gpio36::W<'_> {
            gpio36::W::new(self)
        }
        #[inline(always)]
        pub fn gpio37(&mut self) -> gpio37::W<'_> {
            gpio37::W::new(self)
        }
        #[inline(always)]
        pub fn gpio38(&mut self) -> gpio38::W<'_> {
            gpio38::W::new(self)
        }
        #[inline(always)]
        pub fn gpio39(&mut self) -> gpio39::W<'_> {
            gpio39::W::new(self)
        }
        #[inline(always)]
        pub fn gpio40(&mut self) -> gpio40::W<'_> {
            gpio40::W::new(self)
        }
        #[inline(always)]
        pub fn gpio41(&mut self) -> gpio41::W<'_> {
            gpio41::W::new(self)
        }
        #[inline(always)]
        pub fn gpio42(&mut self) -> gpio42::W<'_> {
            gpio42::W::new(self)
        }
        #[inline(always)]
        pub fn gpio43(&mut self) -> gpio43::W<'_> {
            gpio43::W::new(self)
        }
        #[inline(always)]
        pub fn gpio44(&mut self) -> gpio44::W<'_> {
            gpio44::W::new(self)
        }
        #[inline(always)]
        pub fn gpio45(&mut self) -> gpio45::W<'_> {
            gpio45::W::new(self)
        }
        #[inline(always)]
        pub fn gpio46(&mut self) -> gpio46::W<'_> {
            gpio46::W::new(self)
        }
        #[inline(always)]
        pub fn gpio47(&mut self) -> gpio47::W<'_> {
            gpio47::W::new(self)
        }
        #[inline(always)]
        pub fn gpio48(&mut self) -> gpio48::W<'_> {
            gpio48::W::new(self)
        }
        #[inline(always)]
        pub fn gpio49(&mut self) -> gpio49::W<'_> {
            gpio49::W::new(self)
        }
        #[inline(always)]
        pub fn gpio50(&mut self) -> gpio50::W<'_> {
            gpio50::W::new(self)
        }
        #[inline(always)]
        pub fn gpio51(&mut self) -> gpio51::W<'_> {
            gpio51::W::new(self)
        }
        #[inline(always)]
        pub fn gpio52(&mut self) -> gpio52::W<'_> {
            gpio52::W::new(self)
        }
        #[inline(always)]
        pub fn gpio53(&mut self) -> gpio53::W<'_> {
            gpio53::W::new(self)
        }
    }
    impl AtomicSetW {
        /// GPIO34 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio34(&mut self) -> gpio34::ASW<'_> {
            gpio34::ASW::new(self)
        }
        /// GPIO35 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio35(&mut self) -> gpio35::ASW<'_> {
            gpio35::ASW::new(self)
        }
        /// GPIO36 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio36(&mut self) -> gpio36::ASW<'_> {
            gpio36::ASW::new(self)
        }
        /// GPIO37 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio37(&mut self) -> gpio37::ASW<'_> {
            gpio37::ASW::new(self)
        }
        /// GPIO38 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio38(&mut self) -> gpio38::ASW<'_> {
            gpio38::ASW::new(self)
        }
        /// GPIO39 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio39(&mut self) -> gpio39::ASW<'_> {
            gpio39::ASW::new(self)
        }
        /// GPIO40 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio40(&mut self) -> gpio40::ASW<'_> {
            gpio40::ASW::new(self)
        }
        /// GPIO41 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio41(&mut self) -> gpio41::ASW<'_> {
            gpio41::ASW::new(self)
        }
        /// GPIO42 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio42(&mut self) -> gpio42::ASW<'_> {
            gpio42::ASW::new(self)
        }
        /// GPIO43 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio43(&mut self) -> gpio43::ASW<'_> {
            gpio43::ASW::new(self)
        }
        /// GPIO44 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio44(&mut self) -> gpio44::ASW<'_> {
            gpio44::ASW::new(self)
        }
        /// GPIO45 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio45(&mut self) -> gpio45::ASW<'_> {
            gpio45::ASW::new(self)
        }
        /// GPIO46 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio46(&mut self) -> gpio46::ASW<'_> {
            gpio46::ASW::new(self)
        }
        /// GPIO47 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio47(&mut self) -> gpio47::ASW<'_> {
            gpio47::ASW::new(self)
        }
        /// GPIO48 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio48(&mut self) -> gpio48::ASW<'_> {
            gpio48::ASW::new(self)
        }
        /// GPIO49 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio49(&mut self) -> gpio49::ASW<'_> {
            gpio49::ASW::new(self)
        }
        /// GPIO50 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio50(&mut self) -> gpio50::ASW<'_> {
            gpio50::ASW::new(self)
        }
        /// GPIO51 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio51(&mut self) -> gpio51::ASW<'_> {
            gpio51::ASW::new(self)
        }
        /// GPIO52 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio52(&mut self) -> gpio52::ASW<'_> {
            gpio52::ASW::new(self)
        }
        /// GPIO53 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio53(&mut self) -> gpio53::ASW<'_> {
            gpio53::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// GPIO34 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio34(&mut self) -> gpio34::ACW<'_> {
            gpio34::ACW::new(self)
        }
        /// GPIO35 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio35(&mut self) -> gpio35::ACW<'_> {
            gpio35::ACW::new(self)
        }
        /// GPIO36 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio36(&mut self) -> gpio36::ACW<'_> {
            gpio36::ACW::new(self)
        }
        /// GPIO37 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio37(&mut self) -> gpio37::ACW<'_> {
            gpio37::ACW::new(self)
        }
        /// GPIO38 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio38(&mut self) -> gpio38::ACW<'_> {
            gpio38::ACW::new(self)
        }
        /// GPIO39 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio39(&mut self) -> gpio39::ACW<'_> {
            gpio39::ACW::new(self)
        }
        /// GPIO40 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio40(&mut self) -> gpio40::ACW<'_> {
            gpio40::ACW::new(self)
        }
        /// GPIO41 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio41(&mut self) -> gpio41::ACW<'_> {
            gpio41::ACW::new(self)
        }
        /// GPIO42 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio42(&mut self) -> gpio42::ACW<'_> {
            gpio42::ACW::new(self)
        }
        /// GPIO43 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio43(&mut self) -> gpio43::ACW<'_> {
            gpio43::ACW::new(self)
        }
        /// GPIO44 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio44(&mut self) -> gpio44::ACW<'_> {
            gpio44::ACW::new(self)
        }
        /// GPIO45 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio45(&mut self) -> gpio45::ACW<'_> {
            gpio45::ACW::new(self)
        }
        /// GPIO46 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio46(&mut self) -> gpio46::ACW<'_> {
            gpio46::ACW::new(self)
        }
        /// GPIO47 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio47(&mut self) -> gpio47::ACW<'_> {
            gpio47::ACW::new(self)
        }
        /// GPIO48 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio48(&mut self) -> gpio48::ACW<'_> {
            gpio48::ACW::new(self)
        }
        /// GPIO49 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio49(&mut self) -> gpio49::ACW<'_> {
            gpio49::ACW::new(self)
        }
        /// GPIO50 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio50(&mut self) -> gpio50::ACW<'_> {
            gpio50::ACW::new(self)
        }
        /// GPIO51 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio51(&mut self) -> gpio51::ACW<'_> {
            gpio51::ACW::new(self)
        }
        /// GPIO52 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio52(&mut self) -> gpio52::ACW<'_> {
            gpio52::ACW::new(self)
        }
        /// GPIO53 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio53(&mut self) -> gpio53::ACW<'_> {
            gpio53::ACW::new(self)
        }
    }
    impl AtomicXorW {
        /// GPIO34 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio34(&mut self) -> gpio34::AXW<'_> {
            gpio34::AXW::new(self)
        }
        /// GPIO35 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio35(&mut self) -> gpio35::AXW<'_> {
            gpio35::AXW::new(self)
        }
        /// GPIO36 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio36(&mut self) -> gpio36::AXW<'_> {
            gpio36::AXW::new(self)
        }
        /// GPIO37 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio37(&mut self) -> gpio37::AXW<'_> {
            gpio37::AXW::new(self)
        }
        /// GPIO38 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio38(&mut self) -> gpio38::AXW<'_> {
            gpio38::AXW::new(self)
        }
        /// GPIO39 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio39(&mut self) -> gpio39::AXW<'_> {
            gpio39::AXW::new(self)
        }
        /// GPIO40 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio40(&mut self) -> gpio40::AXW<'_> {
            gpio40::AXW::new(self)
        }
        /// GPIO41 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio41(&mut self) -> gpio41::AXW<'_> {
            gpio41::AXW::new(self)
        }
        /// GPIO42 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio42(&mut self) -> gpio42::AXW<'_> {
            gpio42::AXW::new(self)
        }
        /// GPIO43 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio43(&mut self) -> gpio43::AXW<'_> {
            gpio43::AXW::new(self)
        }
        /// GPIO44 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio44(&mut self) -> gpio44::AXW<'_> {
            gpio44::AXW::new(self)
        }
        /// GPIO45 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio45(&mut self) -> gpio45::AXW<'_> {
            gpio45::AXW::new(self)
        }
        /// GPIO46 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio46(&mut self) -> gpio46::AXW<'_> {
            gpio46::AXW::new(self)
        }
        /// GPIO47 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio47(&mut self) -> gpio47::AXW<'_> {
            gpio47::AXW::new(self)
        }
        /// GPIO48 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio48(&mut self) -> gpio48::AXW<'_> {
            gpio48::AXW::new(self)
        }
        /// GPIO49 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio49(&mut self) -> gpio49::AXW<'_> {
            gpio49::AXW::new(self)
        }
        /// GPIO50 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio50(&mut self) -> gpio50::AXW<'_> {
            gpio50::AXW::new(self)
        }
        /// GPIO51 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio51(&mut self) -> gpio51::AXW<'_> {
            gpio51::AXW::new(self)
        }
        /// GPIO52 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio52(&mut self) -> gpio52::AXW<'_> {
            gpio52::AXW::new(self)
        }
        /// GPIO53 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio53(&mut self) -> gpio53::AXW<'_> {
            gpio53::AXW::new(self)
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
    /// GPIO34 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio34 {
        pub type R = crate::BitReader;
    }
    /// GPIO35 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio35 {
        pub type R = crate::BitReader;
    }
    /// GPIO36 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio36 {
        pub type R = crate::BitReader;
    }
    /// GPIO37 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio37 {
        pub type R = crate::BitReader;
    }
    /// GPIO38 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio38 {
        pub type R = crate::BitReader;
    }
    /// GPIO39 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio39 {
        pub type R = crate::BitReader;
    }
    /// GPIO40 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio40 {
        pub type R = crate::BitReader;
    }
    /// GPIO41 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio41 {
        pub type R = crate::BitReader;
    }
    /// GPIO42 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio42 {
        pub type R = crate::BitReader;
    }
    /// GPIO43 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio43 {
        pub type R = crate::BitReader;
    }
    /// GPIO44 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio44 {
        pub type R = crate::BitReader;
    }
    /// GPIO45 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio45 {
        pub type R = crate::BitReader;
    }
    /// GPIO46 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio46 {
        pub type R = crate::BitReader;
    }
    /// GPIO47 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio47 {
        pub type R = crate::BitReader;
    }
    /// GPIO48 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio48 {
        pub type R = crate::BitReader;
    }
    /// GPIO49 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio49 {
        pub type R = crate::BitReader;
    }
    /// GPIO50 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio50 {
        pub type R = crate::BitReader;
    }
    /// GPIO51 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio51 {
        pub type R = crate::BitReader;
    }
    /// GPIO52 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio52 {
        pub type R = crate::BitReader;
    }
    /// GPIO53 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio53 {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn gpio34(&self) -> gpio34::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio35(&self) -> gpio35::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio36(&self) -> gpio36::R {
            crate::BitReader::new(((self.bits() >> 2) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio37(&self) -> gpio37::R {
            crate::BitReader::new(((self.bits() >> 3) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio38(&self) -> gpio38::R {
            crate::BitReader::new(((self.bits() >> 4) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio39(&self) -> gpio39::R {
            crate::BitReader::new(((self.bits() >> 5) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio40(&self) -> gpio40::R {
            crate::BitReader::new(((self.bits() >> 6) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio41(&self) -> gpio41::R {
            crate::BitReader::new(((self.bits() >> 7) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio42(&self) -> gpio42::R {
            crate::BitReader::new(((self.bits() >> 8) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio43(&self) -> gpio43::R {
            crate::BitReader::new(((self.bits() >> 9) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio44(&self) -> gpio44::R {
            crate::BitReader::new(((self.bits() >> 10) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio45(&self) -> gpio45::R {
            crate::BitReader::new(((self.bits() >> 11) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio46(&self) -> gpio46::R {
            crate::BitReader::new(((self.bits() >> 12) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio47(&self) -> gpio47::R {
            crate::BitReader::new(((self.bits() >> 13) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio48(&self) -> gpio48::R {
            crate::BitReader::new(((self.bits() >> 14) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio49(&self) -> gpio49::R {
            crate::BitReader::new(((self.bits() >> 15) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio50(&self) -> gpio50::R {
            crate::BitReader::new(((self.bits() >> 16) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio51(&self) -> gpio51::R {
            crate::BitReader::new(((self.bits() >> 17) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio52(&self) -> gpio52::R {
            crate::BitReader::new(((self.bits() >> 18) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio53(&self) -> gpio53::R {
            crate::BitReader::new(((self.bits() >> 19) & 1) != 0)
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
