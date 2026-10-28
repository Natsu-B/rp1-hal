// Generated from rp1-regdb/data/peripherals/pcie-apbs.toml.
// Do not edit manually.
// Regenerate with: cargo run -p rp1-regdb -- generate

use crate::Reg;

/// Only independently understood pre-DBI controller registers are exposed; state-machine-only controls remain hidden.
/// Atomic view: Rp1Native alias windows; ownership remains with this logical peripheral.
#[repr(C)]
pub struct RegisterBlock {
    dbi: DBI,
    control: CONTROL,
    msix_cfg: [MSIX_CFG; 64],
    intstatl: INTSTATL,
    intstath: INTSTATH,
    phy_test: PHY_TEST,
    phy_param_ctrl0: PHY_PARAM_CTRL0,
    phy_param_ctrl1: PHY_PARAM_CTRL1,
    phy_cr_acc_ctrl: PHY_CR_ACC_CTRL,
    phy_cr_acc_data: PHY_CR_ACC_DATA,
    ltssm_state_new: LTSSM_STATE_NEW,
    _reserved0: [u8; 4],
    ltssm_state_filter: [LTSSM_STATE_FILTER; 4],
    _reserved1: [u8; 4],
    axi_lookup: [AXI_LOOKUP; 16],
    vdm_config_reg0: VDM_CONFIG_REG0,
    vdm_config_reg1: VDM_CONFIG_REG1,
    vdm_panic_reg: VDM_PANIC_REG,
    vdm_test: VDM_TEST,
    vdm_header: VDM_HEADER,
    pm_control: PM_CONTROL,
    pm_status: PM_STATUS,
    monitor0: MONITOR0,
    monitor1: MONITOR1,
    monitor2: MONITOR2,
    intr: INTR,
    inte: INTE,
    intf: INTF,
    ints: INTS,
    _reserved2: [u8; 8],
    undocumented_c0: UNDOCUMENTED_C0,
    _reserved3: [u8; 4],
    undocumented_c8: UNDOCUMENTED_C8,
}

impl RegisterBlock {
    /// Address fields for DBI access
    /// Hardware access: read Plain; write StateMachineOnly.
    /// State-machine-only register; unrestricted writers are intentionally hidden.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn dbi(&self) -> &DBI {
        &self.dbi
    }
    /// Miscellaneous control bits
    /// Hardware access: read Plain; write StateMachineOnly.
    /// State-machine-only register; unrestricted writers are intentionally hidden.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn control(&self) -> &CONTROL {
        &self.control
    }
    /// 64 physical slots are retained for layout; only 61 Linux-visible vectors are publicly indexed. IACK is self-clearing and has no writer.
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    /// Array layout: 64 physical entries, 61 public entries, stride 0x4 bytes.
    #[inline(always)]
    pub fn msix_cfg(&self, index: usize) -> &MSIX_CFG {
        assert!(index < 61);
        &self.msix_cfg[index]
    }
    #[inline(always)]
    pub fn msix_cfg_array(&self) -> &[MSIX_CFG] {
        &self.msix_cfg[..61]
    }
    /// Raw interrupt status \[31:0\]
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn intstatl(&self) -> &INTSTATL {
        &self.intstatl
    }
    /// Raw interrupt status \[63:32\]
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn intstath(&self) -> &INTSTATH {
        &self.intstath
    }
    /// Address fields for PCIe PHY_TEST access
    /// Hardware access: read Plain; write StateMachineOnly.
    /// State-machine-only register; unrestricted writers are intentionally hidden.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn phy_test(&self) -> &PHY_TEST {
        &self.phy_test
    }
    /// PHY parameter control0
    /// Hardware access: read Plain; write StateMachineOnly.
    /// State-machine-only register; unrestricted writers are intentionally hidden.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn phy_param_ctrl0(&self) -> &PHY_PARAM_CTRL0 {
        &self.phy_param_ctrl0
    }
    /// PHY parameter control1
    /// Hardware access: read Plain; write StateMachineOnly.
    /// State-machine-only register; unrestricted writers are intentionally hidden.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn phy_param_ctrl1(&self) -> &PHY_PARAM_CTRL1 {
        &self.phy_param_ctrl1
    }
    /// PHY control-register access control
    /// Hardware access: read Plain; write StateMachineOnly.
    /// State-machine-only register; unrestricted writers are intentionally hidden.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn phy_cr_acc_ctrl(&self) -> &PHY_CR_ACC_CTRL {
        &self.phy_cr_acc_ctrl
    }
    /// PHY control-register access read/write data
    /// Hardware access: read Fifo; write StateMachineOnly.
    /// State-machine-only register; unrestricted writers are intentionally hidden.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn phy_cr_acc_data(&self) -> &PHY_CR_ACC_DATA {
        &self.phy_cr_acc_data
    }
    /// New LTSSM state
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn ltssm_state_new(&self) -> &LTSSM_STATE_NEW {
        &self.ltssm_state_new
    }
    /// LTSSM state filter
    /// Hardware access: read Plain; write StateMachineOnly.
    /// State-machine-only register; unrestricted writers are intentionally hidden.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    /// Array layout: 4 physical entries, 4 public entries, stride 0x4 bytes.
    #[inline(always)]
    pub fn ltssm_state_filter(&self, index: usize) -> &LTSSM_STATE_FILTER {
        assert!(index < 4);
        &self.ltssm_state_filter[index]
    }
    #[inline(always)]
    pub fn ltssm_state_filter_array(&self) -> &[LTSSM_STATE_FILTER] {
        &self.ltssm_state_filter[..4]
    }
    /// Per-master AXI lookup / TLP and VDM traffic classes
    /// Hardware access: read Plain; write StateMachineOnly.
    /// State-machine-only register; unrestricted writers are intentionally hidden.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    /// Array layout: 16 physical entries, 16 public entries, stride 0x4 bytes.
    #[inline(always)]
    pub fn axi_lookup(&self, index: usize) -> &AXI_LOOKUP {
        assert!(index < 16);
        &self.axi_lookup[index]
    }
    #[inline(always)]
    pub fn axi_lookup_array(&self) -> &[AXI_LOOKUP] {
        &self.axi_lookup[..16]
    }
    /// VDM config register 0
    /// Hardware access: read Plain; write StateMachineOnly.
    /// State-machine-only register; unrestricted writers are intentionally hidden.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn vdm_config_reg0(&self) -> &VDM_CONFIG_REG0 {
        &self.vdm_config_reg0
    }
    /// VDM config register 1
    /// Hardware access: read Plain; write StateMachineOnly.
    /// State-machine-only register; unrestricted writers are intentionally hidden.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn vdm_config_reg1(&self) -> &VDM_CONFIG_REG1 {
        &self.vdm_config_reg1
    }
    /// VDM panic register
    /// Hardware access: read Plain; write StateMachineOnly.
    /// State-machine-only register; unrestricted writers are intentionally hidden.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn vdm_panic_reg(&self) -> &VDM_PANIC_REG {
        &self.vdm_panic_reg
    }
    /// VDM test requesters
    /// Hardware access: read Plain; write StateMachineOnly.
    /// State-machine-only register; unrestricted writers are intentionally hidden.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn vdm_test(&self) -> &VDM_TEST {
        &self.vdm_test
    }
    /// Vendor-defined message header
    /// Hardware access: read Plain; write StateMachineOnly.
    /// State-machine-only register; unrestricted writers are intentionally hidden.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn vdm_header(&self) -> &VDM_HEADER {
        &self.vdm_header
    }
    /// Power-management control
    /// Hardware access: read Plain; write StateMachineOnly.
    /// State-machine-only register; unrestricted writers are intentionally hidden.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn pm_control(&self) -> &PM_CONTROL {
        &self.pm_control
    }
    /// Power-management status
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn pm_status(&self) -> &PM_STATUS {
        &self.pm_status
    }
    /// Internal signals for debug
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn monitor0(&self) -> &MONITOR0 {
        &self.monitor0
    }
    /// Internal signals for debug
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn monitor1(&self) -> &MONITOR1 {
        &self.monitor1
    }
    /// Internal signals for debug
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn monitor2(&self) -> &MONITOR2 {
        &self.monitor2
    }
    /// Raw interrupts
    /// Hardware access: read Plain; write StateMachineOnly.
    /// State-machine-only register; unrestricted writers are intentionally hidden.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn intr(&self) -> &INTR {
        &self.intr
    }
    /// Interrupt enable
    /// Hardware access: read Plain; write StateMachineOnly.
    /// State-machine-only register; unrestricted writers are intentionally hidden.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn inte(&self) -> &INTE {
        &self.inte
    }
    /// Interrupt force
    /// Hardware access: read Plain; write StateMachineOnly.
    /// State-machine-only register; unrestricted writers are intentionally hidden.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn intf(&self) -> &INTF {
        &self.intf
    }
    /// Interrupt status after masking and forcing
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn ints(&self) -> &INTS {
        &self.ints
    }
}

/// Address fields for DBI access
/// Hardware access: read Plain; write StateMachineOnly.
/// State-machine-only register; unrestricted writers are intentionally hidden.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type DBI = Reg<dbi::DBI_SPEC>;
/// Address fields for DBI access
/// Hardware access: read Plain; write StateMachineOnly.
/// State-machine-only register; unrestricted writers are intentionally hidden.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod dbi {
    pub struct DBI_SPEC;
    impl crate::RegisterSpec for DBI_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for DBI_SPEC {}
    pub type R = crate::R<DBI_SPEC>;
    /// b0: cs2, b1: cdm/elbi select
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod asel {
        pub type R = crate::FieldReader<u32>;
        pub const NORMAL_MISC: u32 = 0x0;
        pub const SHADOW: u32 = 0x1;
        pub const ELBI: u32 = 0x2;
        pub const IATU_DMA: u32 = 0x3;
    }
    /// CDM access function number
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod func_num {
        pub type R = crate::FieldReader<u32>;
    }
    /// iATU access region number and direction
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod atu_region {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn asel(&self) -> asel::R {
            crate::FieldReader::new((self.bits() >> 0) & 0x3)
        }
        #[inline(always)]
        pub fn func_num(&self) -> func_num::R {
            crate::FieldReader::new((self.bits() >> 2) & 0x7)
        }
        #[inline(always)]
        pub fn atu_region(&self) -> atu_region::R {
            crate::FieldReader::new((self.bits() >> 5) & 0x1ff)
        }
    }
}

/// Miscellaneous control bits
/// Hardware access: read Plain; write StateMachineOnly.
/// State-machine-only register; unrestricted writers are intentionally hidden.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type CONTROL = Reg<control::CONTROL_SPEC>;
/// Miscellaneous control bits
/// Hardware access: read Plain; write StateMachineOnly.
/// State-machine-only register; unrestricted writers are intentionally hidden.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod control {
    pub struct CONTROL_SPEC;
    impl crate::RegisterSpec for CONTROL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CONTROL_SPEC {}
    pub type R = crate::R<CONTROL_SPEC>;
    /// Defer upstream PCIe configuration requests; complete them with retry status
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod app_req_retry_en {
        pub type R = crate::BitReader;
    }
    /// DBI read-only write disable
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod app_dbi_ro_wr_disable {
        pub type R = crate::BitReader;
    }
    /// Enable link training / establishment
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod app_ltssm_enable {
        pub type R = crate::BitReader;
    }
    /// Enable link training/establishment following hot reset request; self-clearing
    /// Field write behavior: WriteOneToSet.
    /// Typed write API is not exposed by register policy.
    pub mod set_hot_rst_app_ltssm_enable {
        pub type R = crate::BitReader;
    }
    /// Select PCIe clock to send to frequency counter
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod clk2fc_sel {
        pub type R = crate::FieldReader<u32>;
    }
    /// Gate CPERSTN source
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod cperstn_disable {
        pub type R = crate::BitReader;
    }
    /// Assert CPERSTN
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod cperstn_assert {
        pub type R = crate::BitReader;
    }
    /// Power down all PCIe memories
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod mem_pd {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn app_req_retry_en(&self) -> app_req_retry_en::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn app_dbi_ro_wr_disable(&self) -> app_dbi_ro_wr_disable::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
        #[inline(always)]
        pub fn app_ltssm_enable(&self) -> app_ltssm_enable::R {
            crate::BitReader::new(((self.bits() >> 2) & 1) != 0)
        }
        #[inline(always)]
        pub fn set_hot_rst_app_ltssm_enable(&self) -> set_hot_rst_app_ltssm_enable::R {
            crate::BitReader::new(((self.bits() >> 3) & 1) != 0)
        }
        #[inline(always)]
        pub fn clk2fc_sel(&self) -> clk2fc_sel::R {
            crate::FieldReader::new((self.bits() >> 4) & 0x7)
        }
        #[inline(always)]
        pub fn cperstn_disable(&self) -> cperstn_disable::R {
            crate::BitReader::new(((self.bits() >> 7) & 1) != 0)
        }
        #[inline(always)]
        pub fn cperstn_assert(&self) -> cperstn_assert::R {
            crate::BitReader::new(((self.bits() >> 8) & 1) != 0)
        }
        #[inline(always)]
        pub fn mem_pd(&self) -> mem_pd::R {
            crate::BitReader::new(((self.bits() >> 9) & 1) != 0)
        }
    }
}

/// 64 physical slots are retained for layout; only 61 Linux-visible vectors are publicly indexed. IACK is self-clearing and has no writer.
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
/// Array layout: 64 physical entries, 61 public entries, stride 0x4 bytes.
pub type MSIX_CFG = Reg<msix_cfg::MSIX_CFG_SPEC>;
/// 64 physical slots are retained for layout; only 61 Linux-visible vectors are publicly indexed. IACK is self-clearing and has no writer.
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
/// Array layout: 64 physical entries, 61 public entries, stride 0x4 bytes.
pub mod msix_cfg {
    pub struct MSIX_CFG_SPEC;
    impl crate::RegisterSpec for MSIX_CFG_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for MSIX_CFG_SPEC {}
    pub type R = crate::R<MSIX_CFG_SPEC>;
    impl crate::TypedAtomicSet for MSIX_CFG_SPEC {
        const OFFSET: usize = 0x800;
    }
    impl crate::UnsafeAtomicSet for MSIX_CFG_SPEC {
        const OFFSET: usize = 0x800;
    }
    impl crate::TypedAtomicClear for MSIX_CFG_SPEC {
        const OFFSET: usize = 0xc00;
    }
    impl crate::UnsafeAtomicClear for MSIX_CFG_SPEC {
        const OFFSET: usize = 0xc00;
    }
    pub type AtomicSetW = crate::AtomicSetW<MSIX_CFG_SPEC>;
    pub type AtomicClearW = crate::AtomicClearW<MSIX_CFG_SPEC>;
    /// Interrupt enable
    /// Field write behavior: Plain.
    pub mod enable {
        pub type R = crate::BitReader;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 0>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 0>;
    }
    /// ORed with interrupt source for test purposes
    /// Field write behavior: Plain.
    pub mod test {
        pub type R = crate::BitReader;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 1>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 1>;
    }
    /// Interrupt acknowledge; write 1 to clear automatic interrupt mask; self-clearing
    /// Field write behavior: SelfClearing.
    /// Typed write API is not exposed because write behavior is not hardware/official verified.
    pub mod iack {
        pub type R = crate::BitReader;
    }
    /// Enable interrupt acknowledge functionality
    /// Field write behavior: Plain.
    pub mod iack_en {
        pub type R = crate::BitReader;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 3>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 3>;
    }
    /// PCIe traffic class
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod tc {
        pub type R = crate::FieldReader<u32>;
    }
    /// PCIe function
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod func {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn enable(&self) -> enable::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn test(&self) -> test::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
        #[inline(always)]
        pub fn iack(&self) -> iack::R {
            crate::BitReader::new(((self.bits() >> 2) & 1) != 0)
        }
        #[inline(always)]
        pub fn iack_en(&self) -> iack_en::R {
            crate::BitReader::new(((self.bits() >> 3) & 1) != 0)
        }
        #[inline(always)]
        pub fn tc(&self) -> tc::R {
            crate::FieldReader::new((self.bits() >> 12) & 0x7)
        }
        #[inline(always)]
        pub fn func(&self) -> func::R {
            crate::FieldReader::new((self.bits() >> 16) & 0x7)
        }
    }
    impl AtomicSetW {
        /// Interrupt enable
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn enable(&mut self) -> enable::ASW<'_> {
            enable::ASW::new(self)
        }
        /// ORed with interrupt source for test purposes
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn test(&mut self) -> test::ASW<'_> {
            test::ASW::new(self)
        }
        /// Enable interrupt acknowledge functionality
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn iack_en(&mut self) -> iack_en::ASW<'_> {
            iack_en::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// Interrupt enable
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn enable(&mut self) -> enable::ACW<'_> {
            enable::ACW::new(self)
        }
        /// ORed with interrupt source for test purposes
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn test(&mut self) -> test::ACW<'_> {
            test::ACW::new(self)
        }
        /// Enable interrupt acknowledge functionality
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn iack_en(&mut self) -> iack_en::ACW<'_> {
            iack_en::ACW::new(self)
        }
    }
}

/// Raw interrupt status \[31:0\]
/// Hardware access: read Plain; write None.
pub type INTSTATL = Reg<intstatl::INTSTATL_SPEC>;
/// Raw interrupt status \[31:0\]
/// Hardware access: read Plain; write None.
pub mod intstatl {
    pub struct INTSTATL_SPEC;
    impl crate::RegisterSpec for INTSTATL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for INTSTATL_SPEC {}
    pub type R = crate::R<INTSTATL_SPEC>;
    impl crate::ResetValue for INTSTATL_SPEC {
        const RESET_VALUE: u32 = 0x0;
    }
    /// MSIx raw interrupt status vector 0
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector0 {
        pub type R = crate::BitReader;
    }
    /// MSIx raw interrupt status vector 1
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector1 {
        pub type R = crate::BitReader;
    }
    /// MSIx raw interrupt status vector 2
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector2 {
        pub type R = crate::BitReader;
    }
    /// MSIx raw interrupt status vector 3
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector3 {
        pub type R = crate::BitReader;
    }
    /// MSIx raw interrupt status vector 4
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector4 {
        pub type R = crate::BitReader;
    }
    /// MSIx raw interrupt status vector 5
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector5 {
        pub type R = crate::BitReader;
    }
    /// MSIx raw interrupt status vector 6
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector6 {
        pub type R = crate::BitReader;
    }
    /// MSIx raw interrupt status vector 7
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector7 {
        pub type R = crate::BitReader;
    }
    /// MSIx raw interrupt status vector 8
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector8 {
        pub type R = crate::BitReader;
    }
    /// MSIx raw interrupt status vector 9
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector9 {
        pub type R = crate::BitReader;
    }
    /// MSIx raw interrupt status vector 10
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector10 {
        pub type R = crate::BitReader;
    }
    /// MSIx raw interrupt status vector 11
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector11 {
        pub type R = crate::BitReader;
    }
    /// MSIx raw interrupt status vector 12
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector12 {
        pub type R = crate::BitReader;
    }
    /// MSIx raw interrupt status vector 13
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector13 {
        pub type R = crate::BitReader;
    }
    /// MSIx raw interrupt status vector 14
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector14 {
        pub type R = crate::BitReader;
    }
    /// MSIx raw interrupt status vector 15
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector15 {
        pub type R = crate::BitReader;
    }
    /// MSIx raw interrupt status vector 16
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector16 {
        pub type R = crate::BitReader;
    }
    /// MSIx raw interrupt status vector 17
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector17 {
        pub type R = crate::BitReader;
    }
    /// MSIx raw interrupt status vector 18
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector18 {
        pub type R = crate::BitReader;
    }
    /// MSIx raw interrupt status vector 19
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector19 {
        pub type R = crate::BitReader;
    }
    /// MSIx raw interrupt status vector 20
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector20 {
        pub type R = crate::BitReader;
    }
    /// MSIx raw interrupt status vector 21
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector21 {
        pub type R = crate::BitReader;
    }
    /// MSIx raw interrupt status vector 22
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector22 {
        pub type R = crate::BitReader;
    }
    /// MSIx raw interrupt status vector 23
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector23 {
        pub type R = crate::BitReader;
    }
    /// MSIx raw interrupt status vector 24
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector24 {
        pub type R = crate::BitReader;
    }
    /// MSIx raw interrupt status vector 25
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector25 {
        pub type R = crate::BitReader;
    }
    /// MSIx raw interrupt status vector 26
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector26 {
        pub type R = crate::BitReader;
    }
    /// MSIx raw interrupt status vector 27
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector27 {
        pub type R = crate::BitReader;
    }
    /// MSIx raw interrupt status vector 28
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector28 {
        pub type R = crate::BitReader;
    }
    /// MSIx raw interrupt status vector 29
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector29 {
        pub type R = crate::BitReader;
    }
    /// MSIx raw interrupt status vector 30
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector30 {
        pub type R = crate::BitReader;
    }
    /// MSIx raw interrupt status vector 31
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector31 {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn vector0(&self) -> vector0::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn vector1(&self) -> vector1::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
        #[inline(always)]
        pub fn vector2(&self) -> vector2::R {
            crate::BitReader::new(((self.bits() >> 2) & 1) != 0)
        }
        #[inline(always)]
        pub fn vector3(&self) -> vector3::R {
            crate::BitReader::new(((self.bits() >> 3) & 1) != 0)
        }
        #[inline(always)]
        pub fn vector4(&self) -> vector4::R {
            crate::BitReader::new(((self.bits() >> 4) & 1) != 0)
        }
        #[inline(always)]
        pub fn vector5(&self) -> vector5::R {
            crate::BitReader::new(((self.bits() >> 5) & 1) != 0)
        }
        #[inline(always)]
        pub fn vector6(&self) -> vector6::R {
            crate::BitReader::new(((self.bits() >> 6) & 1) != 0)
        }
        #[inline(always)]
        pub fn vector7(&self) -> vector7::R {
            crate::BitReader::new(((self.bits() >> 7) & 1) != 0)
        }
        #[inline(always)]
        pub fn vector8(&self) -> vector8::R {
            crate::BitReader::new(((self.bits() >> 8) & 1) != 0)
        }
        #[inline(always)]
        pub fn vector9(&self) -> vector9::R {
            crate::BitReader::new(((self.bits() >> 9) & 1) != 0)
        }
        #[inline(always)]
        pub fn vector10(&self) -> vector10::R {
            crate::BitReader::new(((self.bits() >> 10) & 1) != 0)
        }
        #[inline(always)]
        pub fn vector11(&self) -> vector11::R {
            crate::BitReader::new(((self.bits() >> 11) & 1) != 0)
        }
        #[inline(always)]
        pub fn vector12(&self) -> vector12::R {
            crate::BitReader::new(((self.bits() >> 12) & 1) != 0)
        }
        #[inline(always)]
        pub fn vector13(&self) -> vector13::R {
            crate::BitReader::new(((self.bits() >> 13) & 1) != 0)
        }
        #[inline(always)]
        pub fn vector14(&self) -> vector14::R {
            crate::BitReader::new(((self.bits() >> 14) & 1) != 0)
        }
        #[inline(always)]
        pub fn vector15(&self) -> vector15::R {
            crate::BitReader::new(((self.bits() >> 15) & 1) != 0)
        }
        #[inline(always)]
        pub fn vector16(&self) -> vector16::R {
            crate::BitReader::new(((self.bits() >> 16) & 1) != 0)
        }
        #[inline(always)]
        pub fn vector17(&self) -> vector17::R {
            crate::BitReader::new(((self.bits() >> 17) & 1) != 0)
        }
        #[inline(always)]
        pub fn vector18(&self) -> vector18::R {
            crate::BitReader::new(((self.bits() >> 18) & 1) != 0)
        }
        #[inline(always)]
        pub fn vector19(&self) -> vector19::R {
            crate::BitReader::new(((self.bits() >> 19) & 1) != 0)
        }
        #[inline(always)]
        pub fn vector20(&self) -> vector20::R {
            crate::BitReader::new(((self.bits() >> 20) & 1) != 0)
        }
        #[inline(always)]
        pub fn vector21(&self) -> vector21::R {
            crate::BitReader::new(((self.bits() >> 21) & 1) != 0)
        }
        #[inline(always)]
        pub fn vector22(&self) -> vector22::R {
            crate::BitReader::new(((self.bits() >> 22) & 1) != 0)
        }
        #[inline(always)]
        pub fn vector23(&self) -> vector23::R {
            crate::BitReader::new(((self.bits() >> 23) & 1) != 0)
        }
        #[inline(always)]
        pub fn vector24(&self) -> vector24::R {
            crate::BitReader::new(((self.bits() >> 24) & 1) != 0)
        }
        #[inline(always)]
        pub fn vector25(&self) -> vector25::R {
            crate::BitReader::new(((self.bits() >> 25) & 1) != 0)
        }
        #[inline(always)]
        pub fn vector26(&self) -> vector26::R {
            crate::BitReader::new(((self.bits() >> 26) & 1) != 0)
        }
        #[inline(always)]
        pub fn vector27(&self) -> vector27::R {
            crate::BitReader::new(((self.bits() >> 27) & 1) != 0)
        }
        #[inline(always)]
        pub fn vector28(&self) -> vector28::R {
            crate::BitReader::new(((self.bits() >> 28) & 1) != 0)
        }
        #[inline(always)]
        pub fn vector29(&self) -> vector29::R {
            crate::BitReader::new(((self.bits() >> 29) & 1) != 0)
        }
        #[inline(always)]
        pub fn vector30(&self) -> vector30::R {
            crate::BitReader::new(((self.bits() >> 30) & 1) != 0)
        }
        #[inline(always)]
        pub fn vector31(&self) -> vector31::R {
            crate::BitReader::new(((self.bits() >> 31) & 1) != 0)
        }
    }
}

/// Raw interrupt status \[63:32\]
/// Hardware access: read Plain; write None.
pub type INTSTATH = Reg<intstath::INTSTATH_SPEC>;
/// Raw interrupt status \[63:32\]
/// Hardware access: read Plain; write None.
pub mod intstath {
    pub struct INTSTATH_SPEC;
    impl crate::RegisterSpec for INTSTATH_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for INTSTATH_SPEC {}
    pub type R = crate::R<INTSTATH_SPEC>;
    impl crate::ResetValue for INTSTATH_SPEC {
        const RESET_VALUE: u32 = 0x0;
    }
    /// MSIx raw interrupt status vector 32
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector32 {
        pub type R = crate::BitReader;
    }
    /// MSIx raw interrupt status vector 33
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector33 {
        pub type R = crate::BitReader;
    }
    /// MSIx raw interrupt status vector 34
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector34 {
        pub type R = crate::BitReader;
    }
    /// MSIx raw interrupt status vector 35
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector35 {
        pub type R = crate::BitReader;
    }
    /// MSIx raw interrupt status vector 36
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector36 {
        pub type R = crate::BitReader;
    }
    /// MSIx raw interrupt status vector 37
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector37 {
        pub type R = crate::BitReader;
    }
    /// MSIx raw interrupt status vector 38
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector38 {
        pub type R = crate::BitReader;
    }
    /// MSIx raw interrupt status vector 39
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector39 {
        pub type R = crate::BitReader;
    }
    /// MSIx raw interrupt status vector 40
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector40 {
        pub type R = crate::BitReader;
    }
    /// MSIx raw interrupt status vector 41
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector41 {
        pub type R = crate::BitReader;
    }
    /// MSIx raw interrupt status vector 42
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector42 {
        pub type R = crate::BitReader;
    }
    /// MSIx raw interrupt status vector 43
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector43 {
        pub type R = crate::BitReader;
    }
    /// MSIx raw interrupt status vector 44
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector44 {
        pub type R = crate::BitReader;
    }
    /// MSIx raw interrupt status vector 45
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector45 {
        pub type R = crate::BitReader;
    }
    /// MSIx raw interrupt status vector 46
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector46 {
        pub type R = crate::BitReader;
    }
    /// MSIx raw interrupt status vector 47
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector47 {
        pub type R = crate::BitReader;
    }
    /// MSIx raw interrupt status vector 48
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector48 {
        pub type R = crate::BitReader;
    }
    /// MSIx raw interrupt status vector 49
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector49 {
        pub type R = crate::BitReader;
    }
    /// MSIx raw interrupt status vector 50
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector50 {
        pub type R = crate::BitReader;
    }
    /// MSIx raw interrupt status vector 51
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector51 {
        pub type R = crate::BitReader;
    }
    /// MSIx raw interrupt status vector 52
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector52 {
        pub type R = crate::BitReader;
    }
    /// MSIx raw interrupt status vector 53
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector53 {
        pub type R = crate::BitReader;
    }
    /// MSIx raw interrupt status vector 54
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector54 {
        pub type R = crate::BitReader;
    }
    /// MSIx raw interrupt status vector 55
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector55 {
        pub type R = crate::BitReader;
    }
    /// MSIx raw interrupt status vector 56
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector56 {
        pub type R = crate::BitReader;
    }
    /// MSIx raw interrupt status vector 57
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector57 {
        pub type R = crate::BitReader;
    }
    /// MSIx raw interrupt status vector 58
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector58 {
        pub type R = crate::BitReader;
    }
    /// MSIx raw interrupt status vector 59
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector59 {
        pub type R = crate::BitReader;
    }
    /// MSIx raw interrupt status vector 60
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector60 {
        pub type R = crate::BitReader;
    }
    /// MSIx raw interrupt status vector 61
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector61 {
        pub type R = crate::BitReader;
    }
    /// MSIx raw interrupt status vector 62
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector62 {
        pub type R = crate::BitReader;
    }
    /// MSIx raw interrupt status vector 63
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector63 {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn vector32(&self) -> vector32::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn vector33(&self) -> vector33::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
        #[inline(always)]
        pub fn vector34(&self) -> vector34::R {
            crate::BitReader::new(((self.bits() >> 2) & 1) != 0)
        }
        #[inline(always)]
        pub fn vector35(&self) -> vector35::R {
            crate::BitReader::new(((self.bits() >> 3) & 1) != 0)
        }
        #[inline(always)]
        pub fn vector36(&self) -> vector36::R {
            crate::BitReader::new(((self.bits() >> 4) & 1) != 0)
        }
        #[inline(always)]
        pub fn vector37(&self) -> vector37::R {
            crate::BitReader::new(((self.bits() >> 5) & 1) != 0)
        }
        #[inline(always)]
        pub fn vector38(&self) -> vector38::R {
            crate::BitReader::new(((self.bits() >> 6) & 1) != 0)
        }
        #[inline(always)]
        pub fn vector39(&self) -> vector39::R {
            crate::BitReader::new(((self.bits() >> 7) & 1) != 0)
        }
        #[inline(always)]
        pub fn vector40(&self) -> vector40::R {
            crate::BitReader::new(((self.bits() >> 8) & 1) != 0)
        }
        #[inline(always)]
        pub fn vector41(&self) -> vector41::R {
            crate::BitReader::new(((self.bits() >> 9) & 1) != 0)
        }
        #[inline(always)]
        pub fn vector42(&self) -> vector42::R {
            crate::BitReader::new(((self.bits() >> 10) & 1) != 0)
        }
        #[inline(always)]
        pub fn vector43(&self) -> vector43::R {
            crate::BitReader::new(((self.bits() >> 11) & 1) != 0)
        }
        #[inline(always)]
        pub fn vector44(&self) -> vector44::R {
            crate::BitReader::new(((self.bits() >> 12) & 1) != 0)
        }
        #[inline(always)]
        pub fn vector45(&self) -> vector45::R {
            crate::BitReader::new(((self.bits() >> 13) & 1) != 0)
        }
        #[inline(always)]
        pub fn vector46(&self) -> vector46::R {
            crate::BitReader::new(((self.bits() >> 14) & 1) != 0)
        }
        #[inline(always)]
        pub fn vector47(&self) -> vector47::R {
            crate::BitReader::new(((self.bits() >> 15) & 1) != 0)
        }
        #[inline(always)]
        pub fn vector48(&self) -> vector48::R {
            crate::BitReader::new(((self.bits() >> 16) & 1) != 0)
        }
        #[inline(always)]
        pub fn vector49(&self) -> vector49::R {
            crate::BitReader::new(((self.bits() >> 17) & 1) != 0)
        }
        #[inline(always)]
        pub fn vector50(&self) -> vector50::R {
            crate::BitReader::new(((self.bits() >> 18) & 1) != 0)
        }
        #[inline(always)]
        pub fn vector51(&self) -> vector51::R {
            crate::BitReader::new(((self.bits() >> 19) & 1) != 0)
        }
        #[inline(always)]
        pub fn vector52(&self) -> vector52::R {
            crate::BitReader::new(((self.bits() >> 20) & 1) != 0)
        }
        #[inline(always)]
        pub fn vector53(&self) -> vector53::R {
            crate::BitReader::new(((self.bits() >> 21) & 1) != 0)
        }
        #[inline(always)]
        pub fn vector54(&self) -> vector54::R {
            crate::BitReader::new(((self.bits() >> 22) & 1) != 0)
        }
        #[inline(always)]
        pub fn vector55(&self) -> vector55::R {
            crate::BitReader::new(((self.bits() >> 23) & 1) != 0)
        }
        #[inline(always)]
        pub fn vector56(&self) -> vector56::R {
            crate::BitReader::new(((self.bits() >> 24) & 1) != 0)
        }
        #[inline(always)]
        pub fn vector57(&self) -> vector57::R {
            crate::BitReader::new(((self.bits() >> 25) & 1) != 0)
        }
        #[inline(always)]
        pub fn vector58(&self) -> vector58::R {
            crate::BitReader::new(((self.bits() >> 26) & 1) != 0)
        }
        #[inline(always)]
        pub fn vector59(&self) -> vector59::R {
            crate::BitReader::new(((self.bits() >> 27) & 1) != 0)
        }
        #[inline(always)]
        pub fn vector60(&self) -> vector60::R {
            crate::BitReader::new(((self.bits() >> 28) & 1) != 0)
        }
        #[inline(always)]
        pub fn vector61(&self) -> vector61::R {
            crate::BitReader::new(((self.bits() >> 29) & 1) != 0)
        }
        #[inline(always)]
        pub fn vector62(&self) -> vector62::R {
            crate::BitReader::new(((self.bits() >> 30) & 1) != 0)
        }
        #[inline(always)]
        pub fn vector63(&self) -> vector63::R {
            crate::BitReader::new(((self.bits() >> 31) & 1) != 0)
        }
    }
}

/// Address fields for PCIe PHY_TEST access
/// Hardware access: read Plain; write StateMachineOnly.
/// State-machine-only register; unrestricted writers are intentionally hidden.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type PHY_TEST = Reg<phy_test::PHY_TEST_SPEC>;
/// Address fields for PCIe PHY_TEST access
/// Hardware access: read Plain; write StateMachineOnly.
/// State-machine-only register; unrestricted writers are intentionally hidden.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod phy_test {
    pub struct PHY_TEST_SPEC;
    impl crate::RegisterSpec for PHY_TEST_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for PHY_TEST_SPEC {}
    pub type R = crate::R<PHY_TEST_SPEC>;
    /// PHY IDDQ power-down control
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod powerdown {
        pub type R = crate::BitReader;
    }
    /// All circuits power-down but leave reference clocks active
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod bypass {
        pub type R = crate::BitReader;
    }
    /// All circuits activator
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod burnin {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn powerdown(&self) -> powerdown::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn bypass(&self) -> bypass::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
        #[inline(always)]
        pub fn burnin(&self) -> burnin::R {
            crate::BitReader::new(((self.bits() >> 2) & 1) != 0)
        }
    }
}

/// PHY parameter control0
/// Hardware access: read Plain; write StateMachineOnly.
/// State-machine-only register; unrestricted writers are intentionally hidden.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type PHY_PARAM_CTRL0 = Reg<phy_param_ctrl0::PHY_PARAM_CTRL0_SPEC>;
/// PHY parameter control0
/// Hardware access: read Plain; write StateMachineOnly.
/// State-machine-only register; unrestricted writers are intentionally hidden.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod phy_param_ctrl0 {
    pub struct PHY_PARAM_CTRL0_SPEC;
    impl crate::RegisterSpec for PHY_PARAM_CTRL0_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for PHY_PARAM_CTRL0_SPEC {}
    pub type R = crate::R<PHY_PARAM_CTRL0_SPEC>;
    /// RX0 equalizer
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod phy_rx0_eq {
        pub type R = crate::FieldReader<u32>;
    }
    /// TX0 termination offset
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod phy_tx0_term_offset {
        pub type R = crate::FieldReader<u32>;
    }
    /// RX1 equalizer
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod phy_rx1_eq {
        pub type R = crate::FieldReader<u32>;
    }
    /// TX1 termination offset
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod phy_tx1_term_offset {
        pub type R = crate::FieldReader<u32>;
    }
    /// RX2 equalizer
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod phy_rx2_eq {
        pub type R = crate::FieldReader<u32>;
    }
    /// TX2 termination offset
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod phy_tx2_term_offset {
        pub type R = crate::FieldReader<u32>;
    }
    /// RX3 equalizer
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod phy_rx3_eq {
        pub type R = crate::FieldReader<u32>;
    }
    /// TX3 termination offset
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod phy_tx3_term_offset {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn phy_rx0_eq(&self) -> phy_rx0_eq::R {
            crate::FieldReader::new((self.bits() >> 0) & 0x7)
        }
        #[inline(always)]
        pub fn phy_tx0_term_offset(&self) -> phy_tx0_term_offset::R {
            crate::FieldReader::new((self.bits() >> 3) & 0x1f)
        }
        #[inline(always)]
        pub fn phy_rx1_eq(&self) -> phy_rx1_eq::R {
            crate::FieldReader::new((self.bits() >> 8) & 0x7)
        }
        #[inline(always)]
        pub fn phy_tx1_term_offset(&self) -> phy_tx1_term_offset::R {
            crate::FieldReader::new((self.bits() >> 11) & 0x1f)
        }
        #[inline(always)]
        pub fn phy_rx2_eq(&self) -> phy_rx2_eq::R {
            crate::FieldReader::new((self.bits() >> 16) & 0x7)
        }
        #[inline(always)]
        pub fn phy_tx2_term_offset(&self) -> phy_tx2_term_offset::R {
            crate::FieldReader::new((self.bits() >> 19) & 0x1f)
        }
        #[inline(always)]
        pub fn phy_rx3_eq(&self) -> phy_rx3_eq::R {
            crate::FieldReader::new((self.bits() >> 24) & 0x7)
        }
        #[inline(always)]
        pub fn phy_tx3_term_offset(&self) -> phy_tx3_term_offset::R {
            crate::FieldReader::new((self.bits() >> 27) & 0x1f)
        }
    }
}

/// PHY parameter control1
/// Hardware access: read Plain; write StateMachineOnly.
/// State-machine-only register; unrestricted writers are intentionally hidden.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type PHY_PARAM_CTRL1 = Reg<phy_param_ctrl1::PHY_PARAM_CTRL1_SPEC>;
/// PHY parameter control1
/// Hardware access: read Plain; write StateMachineOnly.
/// State-machine-only register; unrestricted writers are intentionally hidden.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod phy_param_ctrl1 {
    pub struct PHY_PARAM_CTRL1_SPEC;
    impl crate::RegisterSpec for PHY_PARAM_CTRL1_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for PHY_PARAM_CTRL1_SPEC {}
    pub type R = crate::R<PHY_PARAM_CTRL1_SPEC>;
    /// Gen2 TX deemphasis at 3.5 dB
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod pcs_tx_deemph_gen2_3p5db {
        pub type R = crate::FieldReader<u32>;
    }
    /// Gen2 TX deemphasis at 6.0 dB
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod pcs_tx_deemph_gen2_6db {
        pub type R = crate::FieldReader<u32>;
    }
    /// Gen1 TX deemphasis
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod pcs_tx_deemph_gen1 {
        pub type R = crate::FieldReader<u32>;
    }
    /// TX amplitude - full swing mode
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod pcs_tx_swing_full {
        pub type R = crate::FieldReader<u32>;
    }
    /// TX amplitude - low swing mode
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod pcs_tx_swing_low {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn pcs_tx_deemph_gen2_3p5db(&self) -> pcs_tx_deemph_gen2_3p5db::R {
            crate::FieldReader::new((self.bits() >> 0) & 0x3f)
        }
        #[inline(always)]
        pub fn pcs_tx_deemph_gen2_6db(&self) -> pcs_tx_deemph_gen2_6db::R {
            crate::FieldReader::new((self.bits() >> 6) & 0x3f)
        }
        #[inline(always)]
        pub fn pcs_tx_deemph_gen1(&self) -> pcs_tx_deemph_gen1::R {
            crate::FieldReader::new((self.bits() >> 12) & 0x3f)
        }
        #[inline(always)]
        pub fn pcs_tx_swing_full(&self) -> pcs_tx_swing_full::R {
            crate::FieldReader::new((self.bits() >> 18) & 0x7f)
        }
        #[inline(always)]
        pub fn pcs_tx_swing_low(&self) -> pcs_tx_swing_low::R {
            crate::FieldReader::new((self.bits() >> 25) & 0x7f)
        }
    }
}

/// PHY control-register access control
/// Hardware access: read Plain; write StateMachineOnly.
/// State-machine-only register; unrestricted writers are intentionally hidden.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type PHY_CR_ACC_CTRL = Reg<phy_cr_acc_ctrl::PHY_CR_ACC_CTRL_SPEC>;
/// PHY control-register access control
/// Hardware access: read Plain; write StateMachineOnly.
/// State-machine-only register; unrestricted writers are intentionally hidden.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod phy_cr_acc_ctrl {
    pub struct PHY_CR_ACC_CTRL_SPEC;
    impl crate::RegisterSpec for PHY_CR_ACC_CTRL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for PHY_CR_ACC_CTRL_SPEC {}
    pub type R = crate::R<PHY_CR_ACC_CTRL_SPEC>;
    /// PHY control register address
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod addr {
        pub type R = crate::FieldReader<u32>;
    }
    /// Register write command; self-clearing
    /// Field write behavior: WriteOneToSet.
    /// Typed write API is not exposed by register policy.
    pub mod write {
        pub type R = crate::BitReader;
    }
    /// Register read command; self-clearing
    /// Field write behavior: WriteOneToSet.
    /// Typed write API is not exposed by register policy.
    pub mod read {
        pub type R = crate::BitReader;
    }
    /// PHY CR interface busy
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod busy {
        pub type R = crate::BitReader;
    }
    /// Gate into PHY PCS/PIPE reset; advisable during PHY control-register writes
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod phy_pipe_rst {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn addr(&self) -> addr::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffff)
        }
        #[inline(always)]
        pub fn write(&self) -> write::R {
            crate::BitReader::new(((self.bits() >> 16) & 1) != 0)
        }
        #[inline(always)]
        pub fn read(&self) -> read::R {
            crate::BitReader::new(((self.bits() >> 17) & 1) != 0)
        }
        #[inline(always)]
        pub fn busy(&self) -> busy::R {
            crate::BitReader::new(((self.bits() >> 18) & 1) != 0)
        }
        #[inline(always)]
        pub fn phy_pipe_rst(&self) -> phy_pipe_rst::R {
            crate::BitReader::new(((self.bits() >> 19) & 1) != 0)
        }
    }
}

/// PHY control-register access read/write data
/// Hardware access: read Fifo; write StateMachineOnly.
/// State-machine-only register; unrestricted writers are intentionally hidden.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type PHY_CR_ACC_DATA = Reg<phy_cr_acc_data::PHY_CR_ACC_DATA_SPEC>;
/// PHY control-register access read/write data
/// Hardware access: read Fifo; write StateMachineOnly.
/// State-machine-only register; unrestricted writers are intentionally hidden.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod phy_cr_acc_data {
    pub struct PHY_CR_ACC_DATA_SPEC;
    impl crate::RegisterSpec for PHY_CR_ACC_DATA_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for PHY_CR_ACC_DATA_SPEC {}
    pub type R = crate::R<PHY_CR_ACC_DATA_SPEC>;
    /// PHY register write data
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod wdata {
        pub type R = crate::FieldReader<u32>;
    }
    /// PHY register read data
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod rdata {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn wdata(&self) -> wdata::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffff)
        }
        #[inline(always)]
        pub fn rdata(&self) -> rdata::R {
            crate::FieldReader::new((self.bits() >> 16) & 0xffff)
        }
    }
}

/// New LTSSM state
/// Hardware access: read Plain; write None.
pub type LTSSM_STATE_NEW = Reg<ltssm_state_new::LTSSM_STATE_NEW_SPEC>;
/// New LTSSM state
/// Hardware access: read Plain; write None.
pub mod ltssm_state_new {
    pub struct LTSSM_STATE_NEW_SPEC;
    impl crate::RegisterSpec for LTSSM_STATE_NEW_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for LTSSM_STATE_NEW_SPEC {}
    pub type R = crate::R<LTSSM_STATE_NEW_SPEC>;
    /// New LTSSM state; FIFO-style read side effect per RP1 table type RF
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod state {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn state(&self) -> state::R {
            crate::FieldReader::new((self.bits() >> 0) & 0x3f)
        }
    }
}

/// LTSSM state filter
/// Hardware access: read Plain; write StateMachineOnly.
/// State-machine-only register; unrestricted writers are intentionally hidden.
/// Write API is not exposed because the verified operation semantics are insufficient.
/// Array layout: 4 physical entries, 4 public entries, stride 0x4 bytes.
pub type LTSSM_STATE_FILTER = Reg<ltssm_state_filter::LTSSM_STATE_FILTER_SPEC>;
/// LTSSM state filter
/// Hardware access: read Plain; write StateMachineOnly.
/// State-machine-only register; unrestricted writers are intentionally hidden.
/// Write API is not exposed because the verified operation semantics are insufficient.
/// Array layout: 4 physical entries, 4 public entries, stride 0x4 bytes.
pub mod ltssm_state_filter {
    pub struct LTSSM_STATE_FILTER_SPEC;
    impl crate::RegisterSpec for LTSSM_STATE_FILTER_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for LTSSM_STATE_FILTER_SPEC {}
    pub type R = crate::R<LTSSM_STATE_FILTER_SPEC>;
    impl crate::ResetValue for LTSSM_STATE_FILTER_SPEC {
        const RESET_VALUE: u32 = 0x3f;
    }
    /// Filter out state changes to this LTSSM state
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod val {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn val(&self) -> val::R {
            crate::FieldReader::new((self.bits() >> 0) & 0x3f)
        }
    }
}

/// Per-master AXI lookup / TLP and VDM traffic classes
/// Hardware access: read Plain; write StateMachineOnly.
/// State-machine-only register; unrestricted writers are intentionally hidden.
/// Write API is not exposed because the verified operation semantics are insufficient.
/// Array layout: 16 physical entries, 16 public entries, stride 0x4 bytes.
pub type AXI_LOOKUP = Reg<axi_lookup::AXI_LOOKUP_SPEC>;
/// Per-master AXI lookup / TLP and VDM traffic classes
/// Hardware access: read Plain; write StateMachineOnly.
/// State-machine-only register; unrestricted writers are intentionally hidden.
/// Write API is not exposed because the verified operation semantics are insufficient.
/// Array layout: 16 physical entries, 16 public entries, stride 0x4 bytes.
pub mod axi_lookup {
    pub struct AXI_LOOKUP_SPEC;
    impl crate::RegisterSpec for AXI_LOOKUP_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for AXI_LOOKUP_SPEC {}
    pub type R = crate::R<AXI_LOOKUP_SPEC>;
    impl crate::ResetValue for AXI_LOOKUP_SPEC {
        const RESET_VALUE: u32 = 0x0;
    }
    /// TLP traffic class
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod tlp_tc {
        pub type R = crate::FieldReader<u32>;
    }
    /// VDM traffic class
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod vdm_tc {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn tlp_tc(&self) -> tlp_tc::R {
            crate::FieldReader::new((self.bits() >> 0) & 0x7)
        }
        #[inline(always)]
        pub fn vdm_tc(&self) -> vdm_tc::R {
            crate::FieldReader::new((self.bits() >> 4) & 0x7)
        }
    }
}

/// VDM config register 0
/// Hardware access: read Plain; write StateMachineOnly.
/// State-machine-only register; unrestricted writers are intentionally hidden.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type VDM_CONFIG_REG0 = Reg<vdm_config_reg0::VDM_CONFIG_REG0_SPEC>;
/// VDM config register 0
/// Hardware access: read Plain; write StateMachineOnly.
/// State-machine-only register; unrestricted writers are intentionally hidden.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod vdm_config_reg0 {
    pub struct VDM_CONFIG_REG0_SPEC;
    impl crate::RegisterSpec for VDM_CONFIG_REG0_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for VDM_CONFIG_REG0_SPEC {}
    pub type R = crate::R<VDM_CONFIG_REG0_SPEC>;
    impl crate::ResetValue for VDM_CONFIG_REG0_SPEC {
        const RESET_VALUE: u32 = 0x0;
    }
    /// Message payload
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod payload {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn payload(&self) -> payload::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffffffff)
        }
    }
}

/// VDM config register 1
/// Hardware access: read Plain; write StateMachineOnly.
/// State-machine-only register; unrestricted writers are intentionally hidden.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type VDM_CONFIG_REG1 = Reg<vdm_config_reg1::VDM_CONFIG_REG1_SPEC>;
/// VDM config register 1
/// Hardware access: read Plain; write StateMachineOnly.
/// State-machine-only register; unrestricted writers are intentionally hidden.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod vdm_config_reg1 {
    pub struct VDM_CONFIG_REG1_SPEC;
    impl crate::RegisterSpec for VDM_CONFIG_REG1_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for VDM_CONFIG_REG1_SPEC {}
    pub type R = crate::R<VDM_CONFIG_REG1_SPEC>;
    /// Message request; write 1 to generate config message; self-clearing
    /// Field write behavior: WriteOneToSet.
    /// Typed write API is not exposed by register policy.
    pub mod req {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn req(&self) -> req::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
    }
}

/// VDM panic register
/// Hardware access: read Plain; write StateMachineOnly.
/// State-machine-only register; unrestricted writers are intentionally hidden.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type VDM_PANIC_REG = Reg<vdm_panic_reg::VDM_PANIC_REG_SPEC>;
/// VDM panic register
/// Hardware access: read Plain; write StateMachineOnly.
/// State-machine-only register; unrestricted writers are intentionally hidden.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod vdm_panic_reg {
    pub struct VDM_PANIC_REG_SPEC;
    impl crate::RegisterSpec for VDM_PANIC_REG_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for VDM_PANIC_REG_SPEC {}
    pub type R = crate::R<VDM_PANIC_REG_SPEC>;
    /// Message payload \[31:8\]
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod payload {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn payload(&self) -> payload::R {
            crate::FieldReader::new((self.bits() >> 8) & 0xffffff)
        }
    }
}

/// VDM test requesters
/// Hardware access: read Plain; write StateMachineOnly.
/// State-machine-only register; unrestricted writers are intentionally hidden.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type VDM_TEST = Reg<vdm_test::VDM_TEST_SPEC>;
/// VDM test requesters
/// Hardware access: read Plain; write StateMachineOnly.
/// State-machine-only register; unrestricted writers are intentionally hidden.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod vdm_test {
    pub struct VDM_TEST_SPEC;
    impl crate::RegisterSpec for VDM_TEST_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for VDM_TEST_SPEC {}
    pub type R = crate::R<VDM_TEST_SPEC>;
    /// Requester 0 traffic class
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod req0_tc {
        pub type R = crate::FieldReader<u32>;
    }
    /// Requester 0 panic bit
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod req0_panic {
        pub type R = crate::BitReader;
    }
    /// Requester 1 traffic class
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod req1_tc {
        pub type R = crate::FieldReader<u32>;
    }
    /// Requester 1 panic bit
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod req1_panic {
        pub type R = crate::BitReader;
    }
    /// Requester 2 traffic class
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod req2_tc {
        pub type R = crate::FieldReader<u32>;
    }
    /// Requester 2 panic bit
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod req2_panic {
        pub type R = crate::BitReader;
    }
    /// Requester 3 traffic class
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod req3_tc {
        pub type R = crate::FieldReader<u32>;
    }
    /// Requester 3 panic bit
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod req3_panic {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn req0_tc(&self) -> req0_tc::R {
            crate::FieldReader::new((self.bits() >> 0) & 0x7)
        }
        #[inline(always)]
        pub fn req0_panic(&self) -> req0_panic::R {
            crate::BitReader::new(((self.bits() >> 3) & 1) != 0)
        }
        #[inline(always)]
        pub fn req1_tc(&self) -> req1_tc::R {
            crate::FieldReader::new((self.bits() >> 4) & 0x7)
        }
        #[inline(always)]
        pub fn req1_panic(&self) -> req1_panic::R {
            crate::BitReader::new(((self.bits() >> 7) & 1) != 0)
        }
        #[inline(always)]
        pub fn req2_tc(&self) -> req2_tc::R {
            crate::FieldReader::new((self.bits() >> 8) & 0x7)
        }
        #[inline(always)]
        pub fn req2_panic(&self) -> req2_panic::R {
            crate::BitReader::new(((self.bits() >> 11) & 1) != 0)
        }
        #[inline(always)]
        pub fn req3_tc(&self) -> req3_tc::R {
            crate::FieldReader::new((self.bits() >> 12) & 0x7)
        }
        #[inline(always)]
        pub fn req3_panic(&self) -> req3_panic::R {
            crate::BitReader::new(((self.bits() >> 15) & 1) != 0)
        }
    }
}

/// Vendor-defined message header
/// Hardware access: read Plain; write StateMachineOnly.
/// State-machine-only register; unrestricted writers are intentionally hidden.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type VDM_HEADER = Reg<vdm_header::VDM_HEADER_SPEC>;
/// Vendor-defined message header
/// Hardware access: read Plain; write StateMachineOnly.
/// State-machine-only register; unrestricted writers are intentionally hidden.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod vdm_header {
    pub struct VDM_HEADER_SPEC;
    impl crate::RegisterSpec for VDM_HEADER_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for VDM_HEADER_SPEC {}
    pub type R = crate::R<VDM_HEADER_SPEC>;
    /// Message type; fmt hardwired to 3'b001, default Msg
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod type_ {
        pub type R = crate::FieldReader<u32>;
    }
    /// Traffic class
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod tc {
        pub type R = crate::FieldReader<u32>;
    }
    /// Error poisoned
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod ep {
        pub type R = crate::BitReader;
    }
    /// TLP digest
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod td {
        pub type R = crate::BitReader;
    }
    /// Attribute
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod attr {
        pub type R = crate::FieldReader<u32>;
    }
    /// Function number
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod func_num {
        pub type R = crate::FieldReader<u32>;
    }
    /// Tag
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod tag {
        pub type R = crate::FieldReader<u32>;
    }
    /// Vendor-defined message code
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod code {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn type_(&self) -> type_::R {
            crate::FieldReader::new((self.bits() >> 0) & 0x1f)
        }
        #[inline(always)]
        pub fn tc(&self) -> tc::R {
            crate::FieldReader::new((self.bits() >> 5) & 0x7)
        }
        #[inline(always)]
        pub fn ep(&self) -> ep::R {
            crate::BitReader::new(((self.bits() >> 8) & 1) != 0)
        }
        #[inline(always)]
        pub fn td(&self) -> td::R {
            crate::BitReader::new(((self.bits() >> 9) & 1) != 0)
        }
        #[inline(always)]
        pub fn attr(&self) -> attr::R {
            crate::FieldReader::new((self.bits() >> 10) & 0x3)
        }
        #[inline(always)]
        pub fn func_num(&self) -> func_num::R {
            crate::FieldReader::new((self.bits() >> 12) & 0x7)
        }
        #[inline(always)]
        pub fn tag(&self) -> tag::R {
            crate::FieldReader::new((self.bits() >> 15) & 0xff)
        }
        #[inline(always)]
        pub fn code(&self) -> code::R {
            crate::FieldReader::new((self.bits() >> 23) & 0xff)
        }
    }
}

/// Power-management control
/// Hardware access: read Plain; write StateMachineOnly.
/// State-machine-only register; unrestricted writers are intentionally hidden.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type PM_CONTROL = Reg<pm_control::PM_CONTROL_SPEC>;
/// Power-management control
/// Hardware access: read Plain; write StateMachineOnly.
/// State-machine-only register; unrestricted writers are intentionally hidden.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod pm_control {
    pub struct PM_CONTROL_SPEC;
    impl crate::RegisterSpec for PM_CONTROL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for PM_CONTROL_SPEC {}
    pub type R = crate::R<PM_CONTROL_SPEC>;
    /// Enable clock PM feature
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod app_clk_pm_en {
        pub type R = crate::BitReader;
    }
    /// Application has transfers pending
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod app_xfer_pending {
        pub type R = crate::BitReader;
    }
    /// Request L1 exit
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod app_req_exit_l1 {
        pub type R = crate::BitReader;
    }
    /// Ready to enter L2/L3 state
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod app_ready_entr_l23 {
        pub type R = crate::BitReader;
    }
    /// Request L1-ASPM entry; self-clearing
    /// Field write behavior: WriteOneToSet.
    /// Typed write API is not exposed by register policy.
    pub mod app_req_entr_l1 {
        pub type R = crate::BitReader;
    }
    /// Request L1/L2 PM exit; self-clearing PME support must be enabled
    /// Field write behavior: WriteOneToSet.
    /// Typed write API is not exposed by register policy.
    pub mod apps_pm_xmt_pme {
        pub type R = crate::BitReader;
    }
    /// Reference clock required; set if refclk can be removed
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod app_clk_req_n {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn app_clk_pm_en(&self) -> app_clk_pm_en::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn app_xfer_pending(&self) -> app_xfer_pending::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
        #[inline(always)]
        pub fn app_req_exit_l1(&self) -> app_req_exit_l1::R {
            crate::BitReader::new(((self.bits() >> 2) & 1) != 0)
        }
        #[inline(always)]
        pub fn app_ready_entr_l23(&self) -> app_ready_entr_l23::R {
            crate::BitReader::new(((self.bits() >> 3) & 1) != 0)
        }
        #[inline(always)]
        pub fn app_req_entr_l1(&self) -> app_req_entr_l1::R {
            crate::BitReader::new(((self.bits() >> 4) & 1) != 0)
        }
        #[inline(always)]
        pub fn apps_pm_xmt_pme(&self) -> apps_pm_xmt_pme::R {
            crate::BitReader::new(((self.bits() >> 5) & 1) != 0)
        }
        #[inline(always)]
        pub fn app_clk_req_n(&self) -> app_clk_req_n::R {
            crate::BitReader::new(((self.bits() >> 6) & 1) != 0)
        }
    }
}

/// Power-management status
/// Hardware access: read Plain; write None.
pub type PM_STATUS = Reg<pm_status::PM_STATUS_SPEC>;
/// Power-management status
/// Hardware access: read Plain; write None.
pub mod pm_status {
    pub struct PM_STATUS_SPEC;
    impl crate::RegisterSpec for PM_STATUS_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for PM_STATUS_SPEC {}
    pub type R = crate::R<PM_STATUS_SPEC>;
    /// PM in L0s state
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod pm_linkst_in_l0s {
        pub type R = crate::BitReader;
    }
    /// L1 entry process is in progress
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod pm_l1_entry_started {
        pub type R = crate::BitReader;
    }
    /// PM in L1 state
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod pm_linkst_in_l1 {
        pub type R = crate::BitReader;
    }
    /// PM in L2 state
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod pm_linkst_in_l2 {
        pub type R = crate::BitReader;
    }
    /// PM exiting L2 state
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod pm_linkst_l2_exit {
        pub type R = crate::BitReader;
    }
    /// Current D-state
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod pm_dstate {
        pub type R = crate::FieldReader<u32>;
    }
    /// Current state of PM slave controller
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod pm_slave_state {
        pub type R = crate::FieldReader<u32>;
    }
    /// Current state of PM master controller
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod pm_master_state {
        pub type R = crate::FieldReader<u32>;
    }
    /// Current link state for debug
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod pm_current_state {
        pub type R = crate::FieldReader<u32>;
    }
    /// Application must stop generating new outgoing requests due to PM state
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod pm_xtlh_block_tlp {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn pm_linkst_in_l0s(&self) -> pm_linkst_in_l0s::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn pm_l1_entry_started(&self) -> pm_l1_entry_started::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
        #[inline(always)]
        pub fn pm_linkst_in_l1(&self) -> pm_linkst_in_l1::R {
            crate::BitReader::new(((self.bits() >> 2) & 1) != 0)
        }
        #[inline(always)]
        pub fn pm_linkst_in_l2(&self) -> pm_linkst_in_l2::R {
            crate::BitReader::new(((self.bits() >> 3) & 1) != 0)
        }
        #[inline(always)]
        pub fn pm_linkst_l2_exit(&self) -> pm_linkst_l2_exit::R {
            crate::BitReader::new(((self.bits() >> 4) & 1) != 0)
        }
        #[inline(always)]
        pub fn pm_dstate(&self) -> pm_dstate::R {
            crate::FieldReader::new((self.bits() >> 5) & 0x7)
        }
        #[inline(always)]
        pub fn pm_slave_state(&self) -> pm_slave_state::R {
            crate::FieldReader::new((self.bits() >> 8) & 0x1f)
        }
        #[inline(always)]
        pub fn pm_master_state(&self) -> pm_master_state::R {
            crate::FieldReader::new((self.bits() >> 13) & 0x1f)
        }
        #[inline(always)]
        pub fn pm_current_state(&self) -> pm_current_state::R {
            crate::FieldReader::new((self.bits() >> 18) & 0x7)
        }
        #[inline(always)]
        pub fn pm_xtlh_block_tlp(&self) -> pm_xtlh_block_tlp::R {
            crate::BitReader::new(((self.bits() >> 21) & 1) != 0)
        }
    }
}

/// Internal signals for debug
/// Hardware access: read Plain; write None.
pub type MONITOR0 = Reg<monitor0::MONITOR0_SPEC>;
/// Internal signals for debug
/// Hardware access: read Plain; write None.
pub mod monitor0 {
    pub struct MONITOR0_SPEC;
    impl crate::RegisterSpec for MONITOR0_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for MONITOR0_SPEC {}
    pub type R = crate::R<MONITOR0_SPEC>;
    impl crate::ResetValue for MONITOR0_SPEC {
        const RESET_VALUE: u32 = 0x0;
    }
    /// cxpl_debug_info\[31:0\]
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod cxpl_debug_info_l {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn cxpl_debug_info_l(&self) -> cxpl_debug_info_l::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffffffff)
        }
    }
}

/// Internal signals for debug
/// Hardware access: read Plain; write None.
pub type MONITOR1 = Reg<monitor1::MONITOR1_SPEC>;
/// Internal signals for debug
/// Hardware access: read Plain; write None.
pub mod monitor1 {
    pub struct MONITOR1_SPEC;
    impl crate::RegisterSpec for MONITOR1_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for MONITOR1_SPEC {}
    pub type R = crate::R<MONITOR1_SPEC>;
    impl crate::ResetValue for MONITOR1_SPEC {
        const RESET_VALUE: u32 = 0x0;
    }
    /// cxpl_debug_info\[63:32\]
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod cxpl_debug_info_h {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn cxpl_debug_info_h(&self) -> cxpl_debug_info_h::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffffffff)
        }
    }
}

/// Internal signals for debug
/// Hardware access: read Plain; write None.
pub type MONITOR2 = Reg<monitor2::MONITOR2_SPEC>;
/// Internal signals for debug
/// Hardware access: read Plain; write None.
pub mod monitor2 {
    pub struct MONITOR2_SPEC;
    impl crate::RegisterSpec for MONITOR2_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for MONITOR2_SPEC {}
    pub type R = crate::R<MONITOR2_SPEC>;
    impl crate::ResetValue for MONITOR2_SPEC {
        const RESET_VALUE: u32 = 0x0;
    }
    /// cxpl_debug_info_ei\[15:0\]
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod cxpl_debug_info_ei {
        pub type R = crate::FieldReader<u32>;
    }
    /// State of core_alive signal, synchronised
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod core_alive {
        pub type R = crate::BitReader;
    }
    /// State of PCIe PERSTN signal, synchronised
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod perstn {
        pub type R = crate::BitReader;
    }
    /// State of link_reset_req signal, synchronised
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod link_reset_req {
        pub type R = crate::BitReader;
    }
    /// PHY link up
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod smlh_link_up {
        pub type R = crate::BitReader;
    }
    /// Data link layer up
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod rdlh_link_up {
        pub type R = crate::BitReader;
    }
    /// brdg_slv_xfer_pending
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod brdg_slv_xfer_pending {
        pub type R = crate::BitReader;
    }
    /// brdg_dbi_xfer_pending
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod brdg_dbi_xfer_pending {
        pub type R = crate::BitReader;
    }
    /// radm_xfer_pending
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod radm_xfer_pending {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn cxpl_debug_info_ei(&self) -> cxpl_debug_info_ei::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffff)
        }
        #[inline(always)]
        pub fn core_alive(&self) -> core_alive::R {
            crate::BitReader::new(((self.bits() >> 16) & 1) != 0)
        }
        #[inline(always)]
        pub fn perstn(&self) -> perstn::R {
            crate::BitReader::new(((self.bits() >> 17) & 1) != 0)
        }
        #[inline(always)]
        pub fn link_reset_req(&self) -> link_reset_req::R {
            crate::BitReader::new(((self.bits() >> 18) & 1) != 0)
        }
        #[inline(always)]
        pub fn smlh_link_up(&self) -> smlh_link_up::R {
            crate::BitReader::new(((self.bits() >> 19) & 1) != 0)
        }
        #[inline(always)]
        pub fn rdlh_link_up(&self) -> rdlh_link_up::R {
            crate::BitReader::new(((self.bits() >> 20) & 1) != 0)
        }
        #[inline(always)]
        pub fn brdg_slv_xfer_pending(&self) -> brdg_slv_xfer_pending::R {
            crate::BitReader::new(((self.bits() >> 21) & 1) != 0)
        }
        #[inline(always)]
        pub fn brdg_dbi_xfer_pending(&self) -> brdg_dbi_xfer_pending::R {
            crate::BitReader::new(((self.bits() >> 22) & 1) != 0)
        }
        #[inline(always)]
        pub fn radm_xfer_pending(&self) -> radm_xfer_pending::R {
            crate::BitReader::new(((self.bits() >> 23) & 1) != 0)
        }
    }
}

/// Raw interrupts
/// Hardware access: read Plain; write StateMachineOnly.
/// State-machine-only register; unrestricted writers are intentionally hidden.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type INTR = Reg<intr::INTR_SPEC>;
/// Raw interrupts
/// Hardware access: read Plain; write StateMachineOnly.
/// State-machine-only register; unrestricted writers are intentionally hidden.
/// Write API is not exposed because the verified operation semantics are insufficient.
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
    /// Core alive
    /// Field write behavior: WriteOneToClear.
    /// Typed write API is not exposed by register policy.
    pub mod core_alive {
        pub type R = crate::BitReader;
    }
    /// PCIe PERSTN
    /// Field write behavior: WriteOneToClear.
    /// Typed write API is not exposed by register policy.
    pub mod perstn {
        pub type R = crate::BitReader;
    }
    /// Link reset request
    /// Field write behavior: WriteOneToClear.
    /// Typed write API is not exposed by register policy.
    pub mod link_reset_req {
        pub type R = crate::BitReader;
    }
    /// LTSSM state FIFO not empty
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod ltssm_state_fifo_not_empty {
        pub type R = crate::BitReader;
    }
    /// RADM PM turnoff
    /// Field write behavior: WriteOneToClear.
    /// Typed write API is not exposed by register policy.
    pub mod radm_pm_turnoff {
        pub type R = crate::BitReader;
    }
    /// PHY link up
    /// Field write behavior: WriteOneToClear.
    /// Typed write API is not exposed by register policy.
    pub mod smlh_link_up {
        pub type R = crate::BitReader;
    }
    /// Data link layer up
    /// Field write behavior: WriteOneToClear.
    /// Typed write API is not exposed by register policy.
    pub mod rdlh_link_up {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn core_alive(&self) -> core_alive::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn perstn(&self) -> perstn::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
        #[inline(always)]
        pub fn link_reset_req(&self) -> link_reset_req::R {
            crate::BitReader::new(((self.bits() >> 2) & 1) != 0)
        }
        #[inline(always)]
        pub fn ltssm_state_fifo_not_empty(&self) -> ltssm_state_fifo_not_empty::R {
            crate::BitReader::new(((self.bits() >> 3) & 1) != 0)
        }
        #[inline(always)]
        pub fn radm_pm_turnoff(&self) -> radm_pm_turnoff::R {
            crate::BitReader::new(((self.bits() >> 4) & 1) != 0)
        }
        #[inline(always)]
        pub fn smlh_link_up(&self) -> smlh_link_up::R {
            crate::BitReader::new(((self.bits() >> 5) & 1) != 0)
        }
        #[inline(always)]
        pub fn rdlh_link_up(&self) -> rdlh_link_up::R {
            crate::BitReader::new(((self.bits() >> 6) & 1) != 0)
        }
    }
}

/// Interrupt enable
/// Hardware access: read Plain; write StateMachineOnly.
/// State-machine-only register; unrestricted writers are intentionally hidden.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type INTE = Reg<inte::INTE_SPEC>;
/// Interrupt enable
/// Hardware access: read Plain; write StateMachineOnly.
/// State-machine-only register; unrestricted writers are intentionally hidden.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod inte {
    pub struct INTE_SPEC;
    impl crate::RegisterSpec for INTE_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for INTE_SPEC {}
    pub type R = crate::R<INTE_SPEC>;
    impl crate::ResetValue for INTE_SPEC {
        const RESET_VALUE: u32 = 0x0;
    }
    /// Core alive
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod core_alive {
        pub type R = crate::BitReader;
    }
    /// PCIe PERSTN
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod perstn {
        pub type R = crate::BitReader;
    }
    /// Link reset request
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod link_reset_req {
        pub type R = crate::BitReader;
    }
    /// LTSSM state FIFO not empty
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod ltssm_state_fifo_not_empty {
        pub type R = crate::BitReader;
    }
    /// RADM PM turnoff
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod radm_pm_turnoff {
        pub type R = crate::BitReader;
    }
    /// PHY link up
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod smlh_link_up {
        pub type R = crate::BitReader;
    }
    /// Data link layer up
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod rdlh_link_up {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn core_alive(&self) -> core_alive::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn perstn(&self) -> perstn::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
        #[inline(always)]
        pub fn link_reset_req(&self) -> link_reset_req::R {
            crate::BitReader::new(((self.bits() >> 2) & 1) != 0)
        }
        #[inline(always)]
        pub fn ltssm_state_fifo_not_empty(&self) -> ltssm_state_fifo_not_empty::R {
            crate::BitReader::new(((self.bits() >> 3) & 1) != 0)
        }
        #[inline(always)]
        pub fn radm_pm_turnoff(&self) -> radm_pm_turnoff::R {
            crate::BitReader::new(((self.bits() >> 4) & 1) != 0)
        }
        #[inline(always)]
        pub fn smlh_link_up(&self) -> smlh_link_up::R {
            crate::BitReader::new(((self.bits() >> 5) & 1) != 0)
        }
        #[inline(always)]
        pub fn rdlh_link_up(&self) -> rdlh_link_up::R {
            crate::BitReader::new(((self.bits() >> 6) & 1) != 0)
        }
    }
}

/// Interrupt force
/// Hardware access: read Plain; write StateMachineOnly.
/// State-machine-only register; unrestricted writers are intentionally hidden.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type INTF = Reg<intf::INTF_SPEC>;
/// Interrupt force
/// Hardware access: read Plain; write StateMachineOnly.
/// State-machine-only register; unrestricted writers are intentionally hidden.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod intf {
    pub struct INTF_SPEC;
    impl crate::RegisterSpec for INTF_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for INTF_SPEC {}
    pub type R = crate::R<INTF_SPEC>;
    impl crate::ResetValue for INTF_SPEC {
        const RESET_VALUE: u32 = 0x0;
    }
    /// Core alive
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod core_alive {
        pub type R = crate::BitReader;
    }
    /// PCIe PERSTN
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod perstn {
        pub type R = crate::BitReader;
    }
    /// Link reset request
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod link_reset_req {
        pub type R = crate::BitReader;
    }
    /// LTSSM state FIFO not empty
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod ltssm_state_fifo_not_empty {
        pub type R = crate::BitReader;
    }
    /// RADM PM turnoff
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod radm_pm_turnoff {
        pub type R = crate::BitReader;
    }
    /// PHY link up
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod smlh_link_up {
        pub type R = crate::BitReader;
    }
    /// Data link layer up
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod rdlh_link_up {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn core_alive(&self) -> core_alive::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn perstn(&self) -> perstn::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
        #[inline(always)]
        pub fn link_reset_req(&self) -> link_reset_req::R {
            crate::BitReader::new(((self.bits() >> 2) & 1) != 0)
        }
        #[inline(always)]
        pub fn ltssm_state_fifo_not_empty(&self) -> ltssm_state_fifo_not_empty::R {
            crate::BitReader::new(((self.bits() >> 3) & 1) != 0)
        }
        #[inline(always)]
        pub fn radm_pm_turnoff(&self) -> radm_pm_turnoff::R {
            crate::BitReader::new(((self.bits() >> 4) & 1) != 0)
        }
        #[inline(always)]
        pub fn smlh_link_up(&self) -> smlh_link_up::R {
            crate::BitReader::new(((self.bits() >> 5) & 1) != 0)
        }
        #[inline(always)]
        pub fn rdlh_link_up(&self) -> rdlh_link_up::R {
            crate::BitReader::new(((self.bits() >> 6) & 1) != 0)
        }
    }
}

/// Interrupt status after masking and forcing
/// Hardware access: read Plain; write None.
pub type INTS = Reg<ints::INTS_SPEC>;
/// Interrupt status after masking and forcing
/// Hardware access: read Plain; write None.
pub mod ints {
    pub struct INTS_SPEC;
    impl crate::RegisterSpec for INTS_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for INTS_SPEC {}
    pub type R = crate::R<INTS_SPEC>;
    impl crate::ResetValue for INTS_SPEC {
        const RESET_VALUE: u32 = 0x0;
    }
    /// Core alive
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod core_alive {
        pub type R = crate::BitReader;
    }
    /// PCIe PERSTN
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod perstn {
        pub type R = crate::BitReader;
    }
    /// Link reset request
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod link_reset_req {
        pub type R = crate::BitReader;
    }
    /// LTSSM state FIFO not empty
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod ltssm_state_fifo_not_empty {
        pub type R = crate::BitReader;
    }
    /// RADM PM turnoff
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod radm_pm_turnoff {
        pub type R = crate::BitReader;
    }
    /// PHY link up
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod smlh_link_up {
        pub type R = crate::BitReader;
    }
    /// Data link layer up
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod rdlh_link_up {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn core_alive(&self) -> core_alive::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn perstn(&self) -> perstn::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
        #[inline(always)]
        pub fn link_reset_req(&self) -> link_reset_req::R {
            crate::BitReader::new(((self.bits() >> 2) & 1) != 0)
        }
        #[inline(always)]
        pub fn ltssm_state_fifo_not_empty(&self) -> ltssm_state_fifo_not_empty::R {
            crate::BitReader::new(((self.bits() >> 3) & 1) != 0)
        }
        #[inline(always)]
        pub fn radm_pm_turnoff(&self) -> radm_pm_turnoff::R {
            crate::BitReader::new(((self.bits() >> 4) & 1) != 0)
        }
        #[inline(always)]
        pub fn smlh_link_up(&self) -> smlh_link_up::R {
            crate::BitReader::new(((self.bits() >> 5) & 1) != 0)
        }
        #[inline(always)]
        pub fn rdlh_link_up(&self) -> rdlh_link_up::R {
            crate::BitReader::new(((self.bits() >> 6) & 1) != 0)
        }
    }
}

/// Undocumented endpoint sequencing companion gate. Retained as hidden state-machine metadata; no register accessor or writer is generated.
/// Hardware access: read Plain; write StateMachineOnly.
/// State-machine-only register; unrestricted writers are intentionally hidden.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type UNDOCUMENTED_C0 = Reg<undocumented_c0::UNDOCUMENTED_C0_SPEC>;
/// Undocumented endpoint sequencing companion gate. Retained as hidden state-machine metadata; no register accessor or writer is generated.
/// Hardware access: read Plain; write StateMachineOnly.
/// State-machine-only register; unrestricted writers are intentionally hidden.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod undocumented_c0 {
    pub struct UNDOCUMENTED_C0_SPEC;
    impl crate::RegisterSpec for UNDOCUMENTED_C0_SPEC {
        type Ux = u32;
    }
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod candidate_bit1 {}
}

/// Internal endpoint-core reset sequencing hold/request word. Retained as hidden state-machine metadata; no register accessor or writer is generated.
/// Hardware access: read Plain; write StateMachineOnly.
/// State-machine-only register; unrestricted writers are intentionally hidden.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type UNDOCUMENTED_C8 = Reg<undocumented_c8::UNDOCUMENTED_C8_SPEC>;
/// Internal endpoint-core reset sequencing hold/request word. Retained as hidden state-machine metadata; no register accessor or writer is generated.
/// Hardware access: read Plain; write StateMachineOnly.
/// State-machine-only register; unrestricted writers are intentionally hidden.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod undocumented_c8 {
    pub struct UNDOCUMENTED_C8_SPEC;
    impl crate::RegisterSpec for UNDOCUMENTED_C8_SPEC {
        type Ux = u32;
    }
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod reset_sequence_hold {}
}

#[cfg(test)]
mod layout_tests {
    use super::RegisterBlock;
    #[test]
    fn dbi_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, dbi), 0x0);
    }
    #[test]
    fn control_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, control), 0x4);
    }
    #[test]
    fn msix_cfg_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, msix_cfg), 0x8);
    }
    #[test]
    fn intstatl_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, intstatl), 0x108);
    }
    #[test]
    fn intstath_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, intstath), 0x10c);
    }
    #[test]
    fn phy_test_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, phy_test), 0x110);
    }
    #[test]
    fn phy_param_ctrl0_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, phy_param_ctrl0), 0x114);
    }
    #[test]
    fn phy_param_ctrl1_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, phy_param_ctrl1), 0x118);
    }
    #[test]
    fn phy_cr_acc_ctrl_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, phy_cr_acc_ctrl), 0x11c);
    }
    #[test]
    fn phy_cr_acc_data_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, phy_cr_acc_data), 0x120);
    }
    #[test]
    fn ltssm_state_new_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, ltssm_state_new), 0x124);
    }
    #[test]
    fn ltssm_state_filter_offset() {
        assert_eq!(
            core::mem::offset_of!(RegisterBlock, ltssm_state_filter),
            0x12c
        );
    }
    #[test]
    fn axi_lookup_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, axi_lookup), 0x140);
    }
    #[test]
    fn vdm_config_reg0_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, vdm_config_reg0), 0x180);
    }
    #[test]
    fn vdm_config_reg1_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, vdm_config_reg1), 0x184);
    }
    #[test]
    fn vdm_panic_reg_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, vdm_panic_reg), 0x188);
    }
    #[test]
    fn vdm_test_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, vdm_test), 0x18c);
    }
    #[test]
    fn vdm_header_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, vdm_header), 0x190);
    }
    #[test]
    fn pm_control_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, pm_control), 0x194);
    }
    #[test]
    fn pm_status_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, pm_status), 0x198);
    }
    #[test]
    fn monitor0_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, monitor0), 0x19c);
    }
    #[test]
    fn monitor1_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, monitor1), 0x1a0);
    }
    #[test]
    fn monitor2_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, monitor2), 0x1a4);
    }
    #[test]
    fn intr_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, intr), 0x1a8);
    }
    #[test]
    fn inte_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, inte), 0x1ac);
    }
    #[test]
    fn intf_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, intf), 0x1b0);
    }
    #[test]
    fn ints_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, ints), 0x1b4);
    }
    #[test]
    fn undocumented_c0_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, undocumented_c0), 0x1c0);
    }
    #[test]
    fn undocumented_c8_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, undocumented_c8), 0x1c8);
    }
    #[test]
    fn register_block_size() {
        assert_eq!(core::mem::size_of::<RegisterBlock>(), 0x1cc);
    }
}
