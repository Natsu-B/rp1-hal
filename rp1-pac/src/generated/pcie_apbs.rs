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
    /// Chooses the internal target and address context used by following DBI accesses. PAC writes are restricted to sequenced state-machine code.
    /// Hardware access: read Plain; write StateMachineOnly.
    /// State-machine-only register; unrestricted writers are intentionally hidden.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn dbi(&self) -> &DBI {
        &self.dbi
    }
    /// Endpoint integration controls: resets, DBI write protection, link training, clock routing and memory power. PAC writes are restricted to sequenced state-machine code.
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
    /// Raw (unmasked) source state for MSI-X vectors 0-31, one bit per vector in index order.
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn intstatl(&self) -> &INTSTATL {
        &self.intstatl
    }
    /// Raw (unmasked) source state for MSI-X vectors 32-63. No RP1 source is known for vectors 61-63.
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn intstath(&self) -> &INTSTATH {
        &self.intstath
    }
    /// PHY test mode controls. Which mode wins when several are set is not known. PAC writes are restricted to sequenced state-machine code.
    /// Hardware access: read Plain; write StateMachineOnly.
    /// State-machine-only register; unrestricted writers are intentionally hidden.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn phy_test(&self) -> &PHY_TEST {
        &self.phy_test
    }
    /// Per-lane RX equalization and TX termination trim for lanes 0-3. PAC writes are restricted to sequenced state-machine code.
    /// Hardware access: read Plain; write StateMachineOnly.
    /// State-machine-only register; unrestricted writers are intentionally hidden.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn phy_param_ctrl0(&self) -> &PHY_PARAM_CTRL0 {
        &self.phy_param_ctrl0
    }
    /// PCS TX swing and de-emphasis settings. PAC writes are restricted to sequenced state-machine code.
    /// Hardware access: read Plain; write StateMachineOnly.
    /// State-machine-only register; unrestricted writers are intentionally hidden.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn phy_param_ctrl1(&self) -> &PHY_PARAM_CTRL1 {
        &self.phy_param_ctrl1
    }
    /// Command register for indirect PHY control-register access. Set addr, stage write data if writing, issue read or write, then poll busy. Keep the PCS/PIPE reset asserted during writes, as the hardware documentation advises. PAC writes are restricted to sequenced state-machine code.
    /// Hardware access: read Plain; write StateMachineOnly.
    /// State-machine-only register; unrestricted writers are intentionally hidden.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn phy_cr_acc_ctrl(&self) -> &PHY_CR_ACC_CTRL {
        &self.phy_cr_acc_ctrl
    }
    /// Data for indirect PHY accesses: write data in bits 15:0, read data in bits 31:16. Reads are conservatively classified as FIFO access because their side effects are unverified.
    /// Hardware access: read Fifo; write StateMachineOnly.
    /// State-machine-only register; unrestricted writers are intentionally hidden.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn phy_cr_acc_data(&self) -> &PHY_CR_ACC_DATA {
        &self.phy_cr_acc_data
    }
    /// Read port of the LTSSM transition FIFO. Each read pops one entry, so check that the FIFO is non-empty first. Reading an empty FIFO has unknown results. The PAC `read()` of this register pops as well.
    /// Hardware access: read Fifo; write None.
    #[inline(always)]
    pub const fn ltssm_state_new(&self) -> &LTSSM_STATE_NEW {
        &self.ltssm_state_new
    }
    /// LTSSM states to drop from the transition FIFO, one entry per register. PAC writes are restricted to sequenced state-machine code.
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
    /// Table that maps each AXI master to the traffic classes of its outgoing packets. Which master uses which entry is not known. PAC writes are restricted to sequenced state-machine code.
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
    /// 32-bit payload for the configuration message. Load it before setting vdm_config_reg1.req.
    /// Hardware access: read Plain; write StateMachineOnly.
    /// State-machine-only register; unrestricted writers are intentionally hidden.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn vdm_config_reg0(&self) -> &VDM_CONFIG_REG0 {
        &self.vdm_config_reg0
    }
    /// Send control for the configuration message staged in vdm_config_reg0. PAC writes are restricted to sequenced state-machine code.
    /// Hardware access: read Plain; write StateMachineOnly.
    /// State-machine-only register; unrestricted writers are intentionally hidden.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn vdm_config_reg1(&self) -> &VDM_CONFIG_REG1 {
        &self.vdm_config_reg1
    }
    /// 24-bit operand of the panic message, held in bits 31:8. PAC writes are restricted to sequenced state-machine code.
    /// Hardware access: read Plain; write StateMachineOnly.
    /// State-machine-only register; unrestricted writers are intentionally hidden.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn vdm_panic_reg(&self) -> &VDM_PANIC_REG {
        &self.vdm_panic_reg
    }
    /// Four test requesters, each with its own traffic class and panic trigger bit. PAC writes are restricted to sequenced state-machine code.
    /// Hardware access: read Plain; write StateMachineOnly.
    /// State-machine-only register; unrestricted writers are intentionally hidden.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn vdm_test(&self) -> &VDM_TEST {
        &self.vdm_test
    }
    /// TLP header fields used when generating vendor-defined messages. PAC writes are restricted to sequenced state-machine code.
    /// Hardware access: read Plain; write StateMachineOnly.
    /// State-machine-only register; unrestricted writers are intentionally hidden.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn vdm_header(&self) -> &VDM_HEADER {
        &self.vdm_header
    }
    /// Inputs and requests to the endpoint power-management state machine. The PAC only allows sequenced writes.
    /// Hardware access: read Plain; write StateMachineOnly.
    /// State-machine-only register; unrestricted writers are intentionally hidden.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn pm_control(&self) -> &PM_CONTROL {
        &self.pm_control
    }
    /// Status from the link power-management controller, including a block on new requests.
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn pm_status(&self) -> &PM_STATUS {
        &self.pm_status
    }
    /// Low word of the controller's internal debug bus.
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn monitor0(&self) -> &MONITOR0 {
        &self.monitor0
    }
    /// High word of the controller's internal debug bus.
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn monitor1(&self) -> &MONITOR1 {
        &self.monitor1
    }
    /// Synchronized sideband signals and extra debug bits from the controller.
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn monitor2(&self) -> &MONITOR2 {
        &self.monitor2
    }
    /// Raw interrupt sources, before the enable mask.
    /// Hardware access: read Plain; write StateMachineOnly.
    /// State-machine-only register; unrestricted writers are intentionally hidden.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn intr(&self) -> &INTR {
        &self.intr
    }
    /// Interrupt enable, one bit per source: 1 lets the source reach the IRQ output, 0 masks it.
    /// Hardware access: read Plain; write StateMachineOnly.
    /// State-machine-only register; unrestricted writers are intentionally hidden.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn inte(&self) -> &INTE {
        &self.inte
    }
    /// Interrupt force, one bit per source: 1 asserts the source by software, 0 removes the force.
    /// Hardware access: read Plain; write StateMachineOnly.
    /// State-machine-only register; unrestricted writers are intentionally hidden.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn intf(&self) -> &INTF {
        &self.intf
    }
    /// Effective interrupt state seen by the IRQ output: (INTR & INTE) | INTF.
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn ints(&self) -> &INTS {
        &self.ints
    }
}

/// Chooses the internal target and address context used by following DBI accesses. PAC writes are restricted to sequenced state-machine code.
/// Hardware access: read Plain; write StateMachineOnly.
/// State-machine-only register; unrestricted writers are intentionally hidden.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type DBI = Reg<dbi::DBI_SPEC>;
/// Chooses the internal target and address context used by following DBI accesses. PAC writes are restricted to sequenced state-machine code.
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
    /// DBI target: 0 normal/misc window, 1 shadow window, 2 ELBI, 3 iATU/DMA. Bit 0 drives CS2; bit 1 selects between the CDM and ELBI paths.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod asel {
        pub type R = crate::FieldReader<u32>;
        pub const NORMAL_MISC: u32 = 0x0;
        pub const SHADOW: u32 = 0x1;
        pub const ELBI: u32 = 0x2;
        pub const IATU_DMA: u32 = 0x3;
    }
    /// PCIe function number for CDM accesses. Which functions exist is not known.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod func_num {
        pub type R = crate::FieldReader<u32>;
    }
    /// Translation region and direction (inbound or outbound) for iATU accesses. How these are packed into the field is not known.
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

/// Endpoint integration controls: resets, DBI write protection, link training, clock routing and memory power. PAC writes are restricted to sequenced state-machine code.
/// Hardware access: read Plain; write StateMachineOnly.
/// State-machine-only register; unrestricted writers are intentionally hidden.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type CONTROL = Reg<control::CONTROL_SPEC>;
/// Endpoint integration controls: resets, DBI write protection, link training, clock routing and memory power. PAC writes are restricted to sequenced state-machine code.
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
    /// 1 answers incoming PCIe configuration requests with retry status, postponing normal handling. 0 handles them normally.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod app_req_retry_en {
        pub type R = crate::BitReader;
    }
    /// 1 prevents writes to the DBI override bit MISC_CONTROL_1.DBI_RO_WR_EN. 0 lifts the block.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod app_dbi_ro_wr_disable {
        pub type R = crate::BitReader;
    }
    /// 1 allows the endpoint to train and bring up the link. 0 holds training off.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod app_ltssm_enable {
        pub type R = crate::BitReader;
    }
    /// Write 1 to allow link training again after a hot reset request. Self-clearing.
    /// Field write behavior: SelfClearing.
    /// Typed write API is not exposed by register policy.
    pub mod set_hot_rst_app_ltssm_enable {
        pub type R = crate::BitReader;
    }
    /// Clock routed to the frequency counter; 0 disconnects it. Nominal rates by code (MHz): 1 variable, 2 125, 3 125 or 250, 4 100, 5 125, 6 and 7 250. The source names, and what picks 125 or 250 for code 3, are not known.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod clk2fc_sel {
        pub type R = crate::FieldReader<u32>;
    }
    /// 1 disables the CPERSTN source. How this interacts with cperstn_assert is not known.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod cperstn_disable {
        pub type R = crate::BitReader;
    }
    /// 1 asks for CPERSTN to be asserted by software, subject to the reset gating. Precedence against other sources is not known.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod cperstn_assert {
        pub type R = crate::BitReader;
    }
    /// 1 powers down the PCIe memory arrays. Content retention and the wake-up sequence are not known.
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
    /// 1 allows this vector to generate MSI-X messages; 0 disables it. Messages are also subject to the PCIe MSI-X capability settings.
    /// Field write behavior: Plain.
    pub mod enable {
        pub type R = crate::BitReader;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 0>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 0>;
    }
    /// 1 ORs an asserted level into the source; 0 uses only the external source. If the vector is enabled, the rising edge can send a message.
    /// Field write behavior: Plain.
    pub mod test {
        pub type R = crate::BitReader;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 1>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 1>;
    }
    /// With iack_en=1, write 1 to release the auto-mask; hardware clears the bit. If the source is still high, another MSI-X write may follow.
    /// Field write behavior: SelfClearing.
    /// Typed write API is not exposed because write behavior is not hardware/official verified.
    pub mod iack {
        pub type R = crate::BitReader;
    }
    /// 0: plain rising-edge signalling. 1: the vector is masked automatically when a message is sent and stays masked until iack is written. Use 1 for level sources that can stay high while being serviced.
    /// Field write behavior: Plain.
    pub mod iack_en {
        pub type R = crate::BitReader;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 3>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 3>;
    }
    /// Traffic class (0-7) of the MSI-X write TLP.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod tc {
        pub type R = crate::FieldReader<u32>;
    }
    /// PCIe function that owns this vector. Which function numbers exist is not known.
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
        /// 1 allows this vector to generate MSI-X messages; 0 disables it. Messages are also subject to the PCIe MSI-X capability settings.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn enable(&mut self) -> enable::ASW<'_> {
            enable::ASW::new(self)
        }
        /// 1 ORs an asserted level into the source; 0 uses only the external source. If the vector is enabled, the rising edge can send a message.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn test(&mut self) -> test::ASW<'_> {
            test::ASW::new(self)
        }
        /// 0: plain rising-edge signalling. 1: the vector is masked automatically when a message is sent and stays masked until iack is written. Use 1 for level sources that can stay high while being serviced.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn iack_en(&mut self) -> iack_en::ASW<'_> {
            iack_en::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// 1 allows this vector to generate MSI-X messages; 0 disables it. Messages are also subject to the PCIe MSI-X capability settings.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn enable(&mut self) -> enable::ACW<'_> {
            enable::ACW::new(self)
        }
        /// 1 ORs an asserted level into the source; 0 uses only the external source. If the vector is enabled, the rising edge can send a message.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn test(&mut self) -> test::ACW<'_> {
            test::ACW::new(self)
        }
        /// 0: plain rising-edge signalling. 1: the vector is masked automatically when a message is sent and stays masked until iack is written. Use 1 for level sources that can stay high while being serviced.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn iack_en(&mut self) -> iack_en::ACW<'_> {
            iack_en::ACW::new(self)
        }
    }
}

/// Raw (unmasked) source state for MSI-X vectors 0-31, one bit per vector in index order.
/// Hardware access: read Plain; write None.
pub type INTSTATL = Reg<intstatl::INTSTATL_SPEC>;
/// Raw (unmasked) source state for MSI-X vectors 0-31, one bit per vector in index order.
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
    /// Current input level of MSI-X vector 0, seen before the MSI-X enable and auto-mask. Read-only.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector0 {
        pub type R = crate::BitReader;
    }
    /// Current input level of MSI-X vector 1, seen before the MSI-X enable and auto-mask. Read-only.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector1 {
        pub type R = crate::BitReader;
    }
    /// Current input level of MSI-X vector 2, seen before the MSI-X enable and auto-mask. Read-only.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector2 {
        pub type R = crate::BitReader;
    }
    /// Current input level of MSI-X vector 3, seen before the MSI-X enable and auto-mask. Read-only.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector3 {
        pub type R = crate::BitReader;
    }
    /// Current input level of MSI-X vector 4, seen before the MSI-X enable and auto-mask. Read-only.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector4 {
        pub type R = crate::BitReader;
    }
    /// Current input level of MSI-X vector 5, seen before the MSI-X enable and auto-mask. Read-only.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector5 {
        pub type R = crate::BitReader;
    }
    /// Current input level of MSI-X vector 6, seen before the MSI-X enable and auto-mask. Read-only.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector6 {
        pub type R = crate::BitReader;
    }
    /// Current input level of MSI-X vector 7, seen before the MSI-X enable and auto-mask. Read-only.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector7 {
        pub type R = crate::BitReader;
    }
    /// Current input level of MSI-X vector 8, seen before the MSI-X enable and auto-mask. Read-only.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector8 {
        pub type R = crate::BitReader;
    }
    /// Current input level of MSI-X vector 9, seen before the MSI-X enable and auto-mask. Read-only.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector9 {
        pub type R = crate::BitReader;
    }
    /// Current input level of MSI-X vector 10, seen before the MSI-X enable and auto-mask. Read-only.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector10 {
        pub type R = crate::BitReader;
    }
    /// Current input level of MSI-X vector 11, seen before the MSI-X enable and auto-mask. Read-only.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector11 {
        pub type R = crate::BitReader;
    }
    /// Current input level of MSI-X vector 12, seen before the MSI-X enable and auto-mask. Read-only.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector12 {
        pub type R = crate::BitReader;
    }
    /// Current input level of MSI-X vector 13, seen before the MSI-X enable and auto-mask. Read-only.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector13 {
        pub type R = crate::BitReader;
    }
    /// Current input level of MSI-X vector 14, seen before the MSI-X enable and auto-mask. Read-only.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector14 {
        pub type R = crate::BitReader;
    }
    /// Current input level of MSI-X vector 15, seen before the MSI-X enable and auto-mask. Read-only.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector15 {
        pub type R = crate::BitReader;
    }
    /// Current input level of MSI-X vector 16, seen before the MSI-X enable and auto-mask. Read-only.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector16 {
        pub type R = crate::BitReader;
    }
    /// Current input level of MSI-X vector 17, seen before the MSI-X enable and auto-mask. Read-only.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector17 {
        pub type R = crate::BitReader;
    }
    /// Current input level of MSI-X vector 18, seen before the MSI-X enable and auto-mask. Read-only.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector18 {
        pub type R = crate::BitReader;
    }
    /// Current input level of MSI-X vector 19, seen before the MSI-X enable and auto-mask. Read-only.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector19 {
        pub type R = crate::BitReader;
    }
    /// Current input level of MSI-X vector 20, seen before the MSI-X enable and auto-mask. Read-only.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector20 {
        pub type R = crate::BitReader;
    }
    /// Current input level of MSI-X vector 21, seen before the MSI-X enable and auto-mask. Read-only.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector21 {
        pub type R = crate::BitReader;
    }
    /// Current input level of MSI-X vector 22, seen before the MSI-X enable and auto-mask. Read-only.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector22 {
        pub type R = crate::BitReader;
    }
    /// Current input level of MSI-X vector 23, seen before the MSI-X enable and auto-mask. Read-only.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector23 {
        pub type R = crate::BitReader;
    }
    /// Current input level of MSI-X vector 24, seen before the MSI-X enable and auto-mask. Read-only.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector24 {
        pub type R = crate::BitReader;
    }
    /// Current input level of MSI-X vector 25, seen before the MSI-X enable and auto-mask. Read-only.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector25 {
        pub type R = crate::BitReader;
    }
    /// Current input level of MSI-X vector 26, seen before the MSI-X enable and auto-mask. Read-only.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector26 {
        pub type R = crate::BitReader;
    }
    /// Current input level of MSI-X vector 27, seen before the MSI-X enable and auto-mask. Read-only.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector27 {
        pub type R = crate::BitReader;
    }
    /// Current input level of MSI-X vector 28, seen before the MSI-X enable and auto-mask. Read-only.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector28 {
        pub type R = crate::BitReader;
    }
    /// Current input level of MSI-X vector 29, seen before the MSI-X enable and auto-mask. Read-only.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector29 {
        pub type R = crate::BitReader;
    }
    /// Current input level of MSI-X vector 30, seen before the MSI-X enable and auto-mask. Read-only.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector30 {
        pub type R = crate::BitReader;
    }
    /// Current input level of MSI-X vector 31, seen before the MSI-X enable and auto-mask. Read-only.
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

/// Raw (unmasked) source state for MSI-X vectors 32-63. No RP1 source is known for vectors 61-63.
/// Hardware access: read Plain; write None.
pub type INTSTATH = Reg<intstath::INTSTATH_SPEC>;
/// Raw (unmasked) source state for MSI-X vectors 32-63. No RP1 source is known for vectors 61-63.
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
    /// Current input level of MSI-X vector 32, seen before the MSI-X enable and auto-mask. Read-only.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector32 {
        pub type R = crate::BitReader;
    }
    /// Current input level of MSI-X vector 33, seen before the MSI-X enable and auto-mask. Read-only.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector33 {
        pub type R = crate::BitReader;
    }
    /// Current input level of MSI-X vector 34, seen before the MSI-X enable and auto-mask. Read-only.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector34 {
        pub type R = crate::BitReader;
    }
    /// Current input level of MSI-X vector 35, seen before the MSI-X enable and auto-mask. Read-only.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector35 {
        pub type R = crate::BitReader;
    }
    /// Current input level of MSI-X vector 36, seen before the MSI-X enable and auto-mask. Read-only.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector36 {
        pub type R = crate::BitReader;
    }
    /// Current input level of MSI-X vector 37, seen before the MSI-X enable and auto-mask. Read-only.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector37 {
        pub type R = crate::BitReader;
    }
    /// Current input level of MSI-X vector 38, seen before the MSI-X enable and auto-mask. Read-only.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector38 {
        pub type R = crate::BitReader;
    }
    /// Current input level of MSI-X vector 39, seen before the MSI-X enable and auto-mask. Read-only.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector39 {
        pub type R = crate::BitReader;
    }
    /// Current input level of MSI-X vector 40, seen before the MSI-X enable and auto-mask. Read-only.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector40 {
        pub type R = crate::BitReader;
    }
    /// Current input level of MSI-X vector 41, seen before the MSI-X enable and auto-mask. Read-only.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector41 {
        pub type R = crate::BitReader;
    }
    /// Current input level of MSI-X vector 42, seen before the MSI-X enable and auto-mask. Read-only.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector42 {
        pub type R = crate::BitReader;
    }
    /// Current input level of MSI-X vector 43, seen before the MSI-X enable and auto-mask. Read-only.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector43 {
        pub type R = crate::BitReader;
    }
    /// Current input level of MSI-X vector 44, seen before the MSI-X enable and auto-mask. Read-only.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector44 {
        pub type R = crate::BitReader;
    }
    /// Current input level of MSI-X vector 45, seen before the MSI-X enable and auto-mask. Read-only.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector45 {
        pub type R = crate::BitReader;
    }
    /// Current input level of MSI-X vector 46, seen before the MSI-X enable and auto-mask. Read-only.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector46 {
        pub type R = crate::BitReader;
    }
    /// Current input level of MSI-X vector 47, seen before the MSI-X enable and auto-mask. Read-only.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector47 {
        pub type R = crate::BitReader;
    }
    /// Current input level of MSI-X vector 48, seen before the MSI-X enable and auto-mask. Read-only.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector48 {
        pub type R = crate::BitReader;
    }
    /// Current input level of MSI-X vector 49, seen before the MSI-X enable and auto-mask. Read-only.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector49 {
        pub type R = crate::BitReader;
    }
    /// Current input level of MSI-X vector 50, seen before the MSI-X enable and auto-mask. Read-only.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector50 {
        pub type R = crate::BitReader;
    }
    /// Current input level of MSI-X vector 51, seen before the MSI-X enable and auto-mask. Read-only.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector51 {
        pub type R = crate::BitReader;
    }
    /// Current input level of MSI-X vector 52, seen before the MSI-X enable and auto-mask. Read-only.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector52 {
        pub type R = crate::BitReader;
    }
    /// Current input level of MSI-X vector 53, seen before the MSI-X enable and auto-mask. Read-only.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector53 {
        pub type R = crate::BitReader;
    }
    /// Current input level of MSI-X vector 54, seen before the MSI-X enable and auto-mask. Read-only.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector54 {
        pub type R = crate::BitReader;
    }
    /// Current input level of MSI-X vector 55, seen before the MSI-X enable and auto-mask. Read-only.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector55 {
        pub type R = crate::BitReader;
    }
    /// Current input level of MSI-X vector 56, seen before the MSI-X enable and auto-mask. Read-only.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector56 {
        pub type R = crate::BitReader;
    }
    /// Current input level of MSI-X vector 57, seen before the MSI-X enable and auto-mask. Read-only.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector57 {
        pub type R = crate::BitReader;
    }
    /// Current input level of MSI-X vector 58, seen before the MSI-X enable and auto-mask. Read-only.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector58 {
        pub type R = crate::BitReader;
    }
    /// Current input level of MSI-X vector 59, seen before the MSI-X enable and auto-mask. Read-only.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector59 {
        pub type R = crate::BitReader;
    }
    /// Current input level of MSI-X vector 60, seen before the MSI-X enable and auto-mask. Read-only.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector60 {
        pub type R = crate::BitReader;
    }
    /// Slot for vector 61 in the 64-entry layout. No source is known to be connected; its behaviour is not known.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector61 {
        pub type R = crate::BitReader;
    }
    /// Slot for vector 62 in the 64-entry layout. No source is known to be connected; its behaviour is not known.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vector62 {
        pub type R = crate::BitReader;
    }
    /// Slot for vector 63 in the 64-entry layout. No source is known to be connected; its behaviour is not known.
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

/// PHY test mode controls. Which mode wins when several are set is not known. PAC writes are restricted to sequenced state-machine code.
/// Hardware access: read Plain; write StateMachineOnly.
/// State-machine-only register; unrestricted writers are intentionally hidden.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type PHY_TEST = Reg<phy_test::PHY_TEST_SPEC>;
/// PHY test mode controls. Which mode wins when several are set is not known. PAC writes are restricted to sequenced state-machine code.
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
    /// 1 powers down the PHY for IDDQ testing.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod powerdown {
        pub type R = crate::BitReader;
    }
    /// 1 powers down the PHY circuits but keeps the reference clock running.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod bypass {
        pub type R = crate::BitReader;
    }
    /// 1 turns on the PHY burn-in circuits. Exact scope and sequencing are not known.
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

/// Per-lane RX equalization and TX termination trim for lanes 0-3. PAC writes are restricted to sequenced state-machine code.
/// Hardware access: read Plain; write StateMachineOnly.
/// State-machine-only register; unrestricted writers are intentionally hidden.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type PHY_PARAM_CTRL0 = Reg<phy_param_ctrl0::PHY_PARAM_CTRL0_SPEC>;
/// Per-lane RX equalization and TX termination trim for lanes 0-3. PAC writes are restricted to sequenced state-machine code.
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
    /// RX equalizer setting for lane 0. Its response curve and units are not known.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod phy_rx0_eq {
        pub type R = crate::FieldReader<u32>;
    }
    /// TX termination trim for lane 0. Units and code mapping are not known.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod phy_tx0_term_offset {
        pub type R = crate::FieldReader<u32>;
    }
    /// RX equalizer setting for lane 1. Its response curve and units are not known.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod phy_rx1_eq {
        pub type R = crate::FieldReader<u32>;
    }
    /// TX termination trim for lane 1. Units and code mapping are not known.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod phy_tx1_term_offset {
        pub type R = crate::FieldReader<u32>;
    }
    /// RX equalizer setting for lane 2. Its response curve and units are not known.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod phy_rx2_eq {
        pub type R = crate::FieldReader<u32>;
    }
    /// TX termination trim for lane 2. Units and code mapping are not known.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod phy_tx2_term_offset {
        pub type R = crate::FieldReader<u32>;
    }
    /// RX equalizer setting for lane 3. Its response curve and units are not known.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod phy_rx3_eq {
        pub type R = crate::FieldReader<u32>;
    }
    /// TX termination trim for lane 3. Units and code mapping are not known.
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

/// PCS TX swing and de-emphasis settings. PAC writes are restricted to sequenced state-machine code.
/// Hardware access: read Plain; write StateMachineOnly.
/// State-machine-only register; unrestricted writers are intentionally hidden.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type PHY_PARAM_CTRL1 = Reg<phy_param_ctrl1::PHY_PARAM_CTRL1_SPEC>;
/// PCS TX swing and de-emphasis settings. PAC writes are restricted to sequenced state-machine code.
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
    /// De-emphasis setting applied for the Gen2 3.5 dB preset. Code-to-level mapping is not known.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod pcs_tx_deemph_gen2_3p5db {
        pub type R = crate::FieldReader<u32>;
    }
    /// De-emphasis setting applied for the Gen2 6 dB preset. Code-to-level mapping is not known.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod pcs_tx_deemph_gen2_6db {
        pub type R = crate::FieldReader<u32>;
    }
    /// De-emphasis setting named for Gen1. The source table gives it the same Gen2 6 dB description as another field, so its real use and response are not known.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod pcs_tx_deemph_gen1 {
        pub type R = crate::FieldReader<u32>;
    }
    /// TX amplitude setting for full swing. Units and code mapping are not known.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod pcs_tx_swing_full {
        pub type R = crate::FieldReader<u32>;
    }
    /// TX amplitude setting for low (reduced) swing. Units and code mapping are not known.
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

/// Command register for indirect PHY control-register access. Set addr, stage write data if writing, issue read or write, then poll busy. Keep the PCS/PIPE reset asserted during writes, as the hardware documentation advises. PAC writes are restricted to sequenced state-machine code.
/// Hardware access: read Plain; write StateMachineOnly.
/// State-machine-only register; unrestricted writers are intentionally hidden.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type PHY_CR_ACC_CTRL = Reg<phy_cr_acc_ctrl::PHY_CR_ACC_CTRL_SPEC>;
/// Command register for indirect PHY control-register access. Set addr, stage write data if writing, issue read or write, then poll busy. Keep the PCS/PIPE reset asserted during writes, as the hardware documentation advises. PAC writes are restricted to sequenced state-machine code.
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
    /// 16-bit PHY register address for the next indirect access. Whether it counts bytes or words is not known.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod addr {
        pub type R = crate::FieldReader<u32>;
    }
    /// Write 1 to write phy_cr_acc_data.wdata to the PHY register at addr. Self-clearing.
    /// Field write behavior: SelfClearing.
    /// Typed write API is not exposed by register policy.
    pub mod write {
        pub type R = crate::BitReader;
    }
    /// Write 1 to read the addressed PHY register into phy_cr_acc_data.rdata. Self-clearing.
    /// Field write behavior: SelfClearing.
    /// Typed write API is not exposed by register policy.
    pub mod read {
        pub type R = crate::BitReader;
    }
    /// 1 while an indirect PHY access is in progress; 0 when idle.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod busy {
        pub type R = crate::BitReader;
    }
    /// 1 asserts the PCS/PIPE reset; use it while writing PHY control registers. Interaction with other reset sources is not known.
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

/// Data for indirect PHY accesses: write data in bits 15:0, read data in bits 31:16. Reads are conservatively classified as FIFO access because their side effects are unverified.
/// Hardware access: read Fifo; write StateMachineOnly.
/// State-machine-only register; unrestricted writers are intentionally hidden.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type PHY_CR_ACC_DATA = Reg<phy_cr_acc_data::PHY_CR_ACC_DATA_SPEC>;
/// Data for indirect PHY accesses: write data in bits 15:0, read data in bits 31:16. Reads are conservatively classified as FIFO access because their side effects are unverified.
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
    /// 16-bit data for the next indirect write. Set it before triggering phy_cr_acc_ctrl.write.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod wdata {
        pub type R = crate::FieldReader<u32>;
    }
    /// 16-bit data returned by the last indirect read. Check busy is 0 before using it.
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

/// Read port of the LTSSM transition FIFO. Each read pops one entry, so check that the FIFO is non-empty first. Reading an empty FIFO has unknown results. The PAC `read()` of this register pops as well.
/// Hardware access: read Fifo; write None.
pub type LTSSM_STATE_NEW = Reg<ltssm_state_new::LTSSM_STATE_NEW_SPEC>;
/// Read port of the LTSSM transition FIFO. Each read pops one entry, so check that the FIFO is non-empty first. Reading an empty FIFO has unknown results. The PAC `read()` of this register pops as well.
/// Hardware access: read Fifo; write None.
pub mod ltssm_state_new {
    pub struct LTSSM_STATE_NEW_SPEC;
    impl crate::RegisterSpec for LTSSM_STATE_NEW_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for LTSSM_STATE_NEW_SPEC {}
    pub type R = crate::R<LTSSM_STATE_NEW_SPEC>;
    /// LTSSM state entered by the logged transition. The numeric state encoding is not known.
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

/// LTSSM states to drop from the transition FIFO, one entry per register. PAC writes are restricted to sequenced state-machine code.
/// Hardware access: read Plain; write StateMachineOnly.
/// State-machine-only register; unrestricted writers are intentionally hidden.
/// Write API is not exposed because the verified operation semantics are insufficient.
/// Array layout: 4 physical entries, 4 public entries, stride 0x4 bytes.
pub type LTSSM_STATE_FILTER = Reg<ltssm_state_filter::LTSSM_STATE_FILTER_SPEC>;
/// LTSSM states to drop from the transition FIFO, one entry per register. PAC writes are restricted to sequenced state-machine code.
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
    /// 6-bit LTSSM state code; transitions into this state are not logged in the FIFO. The state code map, and whether the reset value 0x3f means "no filter", are not known.
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

/// Table that maps each AXI master to the traffic classes of its outgoing packets. Which master uses which entry is not known. PAC writes are restricted to sequenced state-machine code.
/// Hardware access: read Plain; write StateMachineOnly.
/// State-machine-only register; unrestricted writers are intentionally hidden.
/// Write API is not exposed because the verified operation semantics are insufficient.
/// Array layout: 16 physical entries, 16 public entries, stride 0x4 bytes.
pub type AXI_LOOKUP = Reg<axi_lookup::AXI_LOOKUP_SPEC>;
/// Table that maps each AXI master to the traffic classes of its outgoing packets. Which master uses which entry is not known. PAC writes are restricted to sequenced state-machine code.
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
    /// Traffic class (0-7) for ordinary TLPs issued by this entry's master.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod tlp_tc {
        pub type R = crate::FieldReader<u32>;
    }
    /// Traffic class (0-7) for vendor-defined messages issued by this entry's master.
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

/// 32-bit payload for the configuration message. Load it before setting vdm_config_reg1.req.
/// Hardware access: read Plain; write StateMachineOnly.
/// State-machine-only register; unrestricted writers are intentionally hidden.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type VDM_CONFIG_REG0 = Reg<vdm_config_reg0::VDM_CONFIG_REG0_SPEC>;
/// 32-bit payload for the configuration message. Load it before setting vdm_config_reg1.req.
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
    /// Configuration message payload, all 32 bits.
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

/// Send control for the configuration message staged in vdm_config_reg0. PAC writes are restricted to sequenced state-machine code.
/// Hardware access: read Plain; write StateMachineOnly.
/// State-machine-only register; unrestricted writers are intentionally hidden.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type VDM_CONFIG_REG1 = Reg<vdm_config_reg1::VDM_CONFIG_REG1_SPEC>;
/// Send control for the configuration message staged in vdm_config_reg0. PAC writes are restricted to sequenced state-machine code.
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
    /// Write 1 to send the message using the staged payload. Self-clearing.
    /// Field write behavior: SelfClearing.
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

/// 24-bit operand of the panic message, held in bits 31:8. PAC writes are restricted to sequenced state-machine code.
/// Hardware access: read Plain; write StateMachineOnly.
/// State-machine-only register; unrestricted writers are intentionally hidden.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type VDM_PANIC_REG = Reg<vdm_panic_reg::VDM_PANIC_REG_SPEC>;
/// 24-bit operand of the panic message, held in bits 31:8. PAC writes are restricted to sequenced state-machine code.
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
    /// Upper three bytes of the panic message payload. Where the remaining byte comes from is not known.
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

/// Four test requesters, each with its own traffic class and panic trigger bit. PAC writes are restricted to sequenced state-machine code.
/// Hardware access: read Plain; write StateMachineOnly.
/// State-machine-only register; unrestricted writers are intentionally hidden.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type VDM_TEST = Reg<vdm_test::VDM_TEST_SPEC>;
/// Four test requesters, each with its own traffic class and panic trigger bit. PAC writes are restricted to sequenced state-machine code.
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
    /// Traffic class (0-7) of the test VDM from requester 0.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod req0_tc {
        pub type R = crate::FieldReader<u32>;
    }
    /// Any change of this bit (0 to 1 or 1 to 0) sends a test VDM from requester 0.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod req0_panic {
        pub type R = crate::BitReader;
    }
    /// Traffic class (0-7) of the test VDM from requester 1.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod req1_tc {
        pub type R = crate::FieldReader<u32>;
    }
    /// Any change of this bit (0 to 1 or 1 to 0) sends a test VDM from requester 1.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod req1_panic {
        pub type R = crate::BitReader;
    }
    /// Traffic class (0-7) of the test VDM from requester 2.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod req2_tc {
        pub type R = crate::FieldReader<u32>;
    }
    /// Any change of this bit (0 to 1 or 1 to 0) sends a test VDM from requester 2.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod req2_panic {
        pub type R = crate::BitReader;
    }
    /// Traffic class (0-7) of the test VDM from requester 3.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod req3_tc {
        pub type R = crate::FieldReader<u32>;
    }
    /// Any change of this bit (0 to 1 or 1 to 0) sends a test VDM from requester 3.
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

/// TLP header fields used when generating vendor-defined messages. PAC writes are restricted to sequenced state-machine code.
/// Hardware access: read Plain; write StateMachineOnly.
/// State-machine-only register; unrestricted writers are intentionally hidden.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type VDM_HEADER = Reg<vdm_header::VDM_HEADER_SPEC>;
/// TLP header fields used when generating vendor-defined messages. PAC writes are restricted to sequenced state-machine code.
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
    /// 5-bit TLP Type field. Fmt is fixed at 0b001; the reset value 0x10 is a message (Msg). Other supported types are not known.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod type_ {
        pub type R = crate::FieldReader<u32>;
    }
    /// 3-bit traffic class of the message TLP.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod tc {
        pub type R = crate::FieldReader<u32>;
    }
    /// EP (poisoned) bit of the message; 1 marks the data as poisoned.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod ep {
        pub type R = crate::BitReader;
    }
    /// TD bit; 1 means a TLP digest is present.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod td {
        pub type R = crate::BitReader;
    }
    /// 2-bit TLP Attr field. Which combinations are supported is not known.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod attr {
        pub type R = crate::FieldReader<u32>;
    }
    /// Function number placed in the message. Which values are valid is not known.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod func_num {
        pub type R = crate::FieldReader<u32>;
    }
    /// 8-bit tag of the message.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod tag {
        pub type R = crate::FieldReader<u32>;
    }
    /// 8-bit message code; resets to 0x7f. Other supported codes are not known.
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

/// Inputs and requests to the endpoint power-management state machine. The PAC only allows sequenced writes.
/// Hardware access: read Plain; write StateMachineOnly.
/// State-machine-only register; unrestricted writers are intentionally hidden.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type PM_CONTROL = Reg<pm_control::PM_CONTROL_SPEC>;
/// Inputs and requests to the endpoint power-management state machine. The PAC only allows sequenced writes.
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
    /// 1 enables clock power management; 0 keeps it off.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod app_clk_pm_en {
        pub type R = crate::BitReader;
    }
    /// 1 tells the controller transfers are pending, which blocks ASPM L1 entry. A PM-initiated L1 entry can still happen, but the link leaves L1 straight away.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod app_xfer_pending {
        pub type R = crate::BitReader;
    }
    /// 1 requests exit from L1; 0 makes no request.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod app_req_exit_l1 {
        pub type R = crate::BitReader;
    }
    /// 1 tells the controller the application is ready to enter L2/L3; 0 means not ready.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod app_ready_entr_l23 {
        pub type R = crate::BitReader;
    }
    /// Write 1 to request entry to ASPM L1. Cleared once the request is taken.
    /// Field write behavior: SelfClearing.
    /// Typed write API is not exposed by register policy.
    pub mod app_req_entr_l1 {
        pub type R = crate::BitReader;
    }
    /// Write 1 to send a PME message to wake from L1/L2; PME support must be enabled. Self-clearing.
    /// Field write behavior: SelfClearing.
    /// Typed write API is not exposed by register policy.
    pub mod apps_pm_xmt_pme {
        pub type R = crate::BitReader;
    }
    /// Active-low reference clock request: 0 means REFCLK is needed, 1 allows it to be removed.
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

/// Status from the link power-management controller, including a block on new requests.
/// Hardware access: read Plain; write None.
pub type PM_STATUS = Reg<pm_status::PM_STATUS_SPEC>;
/// Status from the link power-management controller, including a block on new requests.
/// Hardware access: read Plain; write None.
pub mod pm_status {
    pub struct PM_STATUS_SPEC;
    impl crate::RegisterSpec for PM_STATUS_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for PM_STATUS_SPEC {}
    pub type R = crate::R<PM_STATUS_SPEC>;
    /// 1 when the link is in L0s.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod pm_linkst_in_l0s {
        pub type R = crate::BitReader;
    }
    /// 1 while entry to L1 is in progress.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod pm_l1_entry_started {
        pub type R = crate::BitReader;
    }
    /// 1 when the link is in L1.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod pm_linkst_in_l1 {
        pub type R = crate::BitReader;
    }
    /// 1 when the link is in L2.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod pm_linkst_in_l2 {
        pub type R = crate::BitReader;
    }
    /// 1 when the link is leaving L2.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod pm_linkst_l2_exit {
        pub type R = crate::BitReader;
    }
    /// D-state of the PCI function. The code mapping is not known.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod pm_dstate {
        pub type R = crate::FieldReader<u32>;
    }
    /// Current state of the PM slave state machine. State values are not known.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod pm_slave_state {
        pub type R = crate::FieldReader<u32>;
    }
    /// Current state of the PM master state machine. State values are not known.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod pm_master_state {
        pub type R = crate::FieldReader<u32>;
    }
    /// Debug state code: 0 L0, 1 L0s, 2 L1, 3 L2/L3 ready with aux power, 4 L2/L3 ready without aux power. Codes 5-7 are not known.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod pm_current_state {
        pub type R = crate::FieldReader<u32>;
    }
    /// 1 while new outgoing request TLPs are held off for a power-management transition.
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

/// Low word of the controller's internal debug bus.
/// Hardware access: read Plain; write None.
pub type MONITOR0 = Reg<monitor0::MONITOR0_SPEC>;
/// Low word of the controller's internal debug bus.
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
    /// CXPL debug bits 0-31. Meaning of individual bits is not known.
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

/// High word of the controller's internal debug bus.
/// Hardware access: read Plain; write None.
pub type MONITOR1 = Reg<monitor1::MONITOR1_SPEC>;
/// High word of the controller's internal debug bus.
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
    /// CXPL debug bits 32-63. Meaning of individual bits is not known.
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

/// Synchronized sideband signals and extra debug bits from the controller.
/// Hardware access: read Plain; write None.
pub type MONITOR2 = Reg<monitor2::MONITOR2_SPEC>;
/// Synchronized sideband signals and extra debug bits from the controller.
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
    /// 16 additional CXPL debug bits. Meaning of individual bits is not known.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod cxpl_debug_info_ei {
        pub type R = crate::FieldReader<u32>;
    }
    /// Synchronized copy of the endpoint core activity signal.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod core_alive {
        pub type R = crate::BitReader;
    }
    /// Synchronized copy of the active-low PCIe reset; 0 means reset is asserted.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod perstn {
        pub type R = crate::BitReader;
    }
    /// Synchronized copy of the controller's link reset request.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod link_reset_req {
        pub type R = crate::BitReader;
    }
    /// 1 when the link is up at the physical layer.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod smlh_link_up {
        pub type R = crate::BitReader;
    }
    /// 1 when the link is up at the data link layer.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod rdlh_link_up {
        pub type R = crate::BitReader;
    }
    /// 1 while a transfer is in flight on the bridge slave path. The exact point where it counts as complete is not known.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod brdg_slv_xfer_pending {
        pub type R = crate::BitReader;
    }
    /// 1 while a DBI transfer through the bridge is outstanding. The exact point where it counts as complete is not known.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod brdg_dbi_xfer_pending {
        pub type R = crate::BitReader;
    }
    /// 1 while RADM has a transfer outstanding. The exact point where it counts as complete is not known.
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

/// Raw interrupt sources, before the enable mask.
/// Hardware access: read Plain; write StateMachineOnly.
/// State-machine-only register; unrestricted writers are intentionally hidden.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type INTR = Reg<intr::INTR_SPEC>;
/// Raw interrupt sources, before the enable mask.
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
    /// 1 when the endpoint core activity event source has fired. Write 1 to clear; writing 0 leaves it pending.
    /// Field write behavior: WriteOneToClear.
    /// Typed write API is not exposed by register policy.
    pub mod core_alive {
        pub type R = crate::BitReader;
    }
    /// 1 when the PCIe reset (PERST#) event source has fired. Write 1 to clear; writing 0 leaves it pending.
    /// Field write behavior: WriteOneToClear.
    /// Typed write API is not exposed by register policy.
    pub mod perstn {
        pub type R = crate::BitReader;
    }
    /// 1 when the link reset request event source has fired. Write 1 to clear; writing 0 leaves it pending.
    /// Field write behavior: WriteOneToClear.
    /// Typed write API is not exposed by register policy.
    pub mod link_reset_req {
        pub type R = crate::BitReader;
    }
    /// 1 while the LTSSM transition FIFO holds an entry source is active. Read-only level; it cannot be cleared here.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod ltssm_state_fifo_not_empty {
        pub type R = crate::BitReader;
    }
    /// 1 when the PM turn-off message received by RADM source has fired. Write 1 to clear; writing 0 leaves it pending.
    /// Field write behavior: WriteOneToClear.
    /// Typed write API is not exposed by register policy.
    pub mod radm_pm_turnoff {
        pub type R = crate::BitReader;
    }
    /// 1 when the physical layer link-up event source has fired. Write 1 to clear; writing 0 leaves it pending.
    /// Field write behavior: WriteOneToClear.
    /// Typed write API is not exposed by register policy.
    pub mod smlh_link_up {
        pub type R = crate::BitReader;
    }
    /// 1 when the data link layer link-up event source has fired. Write 1 to clear; writing 0 leaves it pending.
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

/// Interrupt enable, one bit per source: 1 lets the source reach the IRQ output, 0 masks it.
/// Hardware access: read Plain; write StateMachineOnly.
/// State-machine-only register; unrestricted writers are intentionally hidden.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type INTE = Reg<inte::INTE_SPEC>;
/// Interrupt enable, one bit per source: 1 lets the source reach the IRQ output, 0 masks it.
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
    /// Enable for the endpoint core activity event source. 1 passes it to the IRQ output; 0 masks it.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod core_alive {
        pub type R = crate::BitReader;
    }
    /// Enable for the PCIe reset (PERST#) event source. 1 passes it to the IRQ output; 0 masks it.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod perstn {
        pub type R = crate::BitReader;
    }
    /// Enable for the link reset request event source. 1 passes it to the IRQ output; 0 masks it.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod link_reset_req {
        pub type R = crate::BitReader;
    }
    /// Enable for the LTSSM transition FIFO holds an entry source. 1 passes it to the IRQ output; 0 masks it.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod ltssm_state_fifo_not_empty {
        pub type R = crate::BitReader;
    }
    /// Enable for the PM turn-off message received by RADM source. 1 passes it to the IRQ output; 0 masks it.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod radm_pm_turnoff {
        pub type R = crate::BitReader;
    }
    /// Enable for the physical layer link-up event source. 1 passes it to the IRQ output; 0 masks it.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod smlh_link_up {
        pub type R = crate::BitReader;
    }
    /// Enable for the data link layer link-up event source. 1 passes it to the IRQ output; 0 masks it.
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

/// Interrupt force, one bit per source: 1 asserts the source by software, 0 removes the force.
/// Hardware access: read Plain; write StateMachineOnly.
/// State-machine-only register; unrestricted writers are intentionally hidden.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type INTF = Reg<intf::INTF_SPEC>;
/// Interrupt force, one bit per source: 1 asserts the source by software, 0 removes the force.
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
    /// Force for the endpoint core activity event source. 1 raises the IRQ by software; 0 removes the force.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod core_alive {
        pub type R = crate::BitReader;
    }
    /// Force for the PCIe reset (PERST#) event source. 1 raises the IRQ by software; 0 removes the force.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod perstn {
        pub type R = crate::BitReader;
    }
    /// Force for the link reset request event source. 1 raises the IRQ by software; 0 removes the force.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod link_reset_req {
        pub type R = crate::BitReader;
    }
    /// Force for the LTSSM transition FIFO holds an entry source. 1 raises the IRQ by software; 0 removes the force.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod ltssm_state_fifo_not_empty {
        pub type R = crate::BitReader;
    }
    /// Force for the PM turn-off message received by RADM source. 1 raises the IRQ by software; 0 removes the force.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod radm_pm_turnoff {
        pub type R = crate::BitReader;
    }
    /// Force for the physical layer link-up event source. 1 raises the IRQ by software; 0 removes the force.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod smlh_link_up {
        pub type R = crate::BitReader;
    }
    /// Force for the data link layer link-up event source. 1 raises the IRQ by software; 0 removes the force.
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

/// Effective interrupt state seen by the IRQ output: (INTR & INTE) | INTF.
/// Hardware access: read Plain; write None.
pub type INTS = Reg<ints::INTS_SPEC>;
/// Effective interrupt state seen by the IRQ output: (INTR & INTE) | INTF.
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
    /// 1 when the endpoint core activity event source is driving the IRQ output.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod core_alive {
        pub type R = crate::BitReader;
    }
    /// 1 when the PCIe reset (PERST#) event source is driving the IRQ output.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod perstn {
        pub type R = crate::BitReader;
    }
    /// 1 when the link reset request event source is driving the IRQ output.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod link_reset_req {
        pub type R = crate::BitReader;
    }
    /// 1 when the LTSSM transition FIFO holds an entry source is driving the IRQ output.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod ltssm_state_fifo_not_empty {
        pub type R = crate::BitReader;
    }
    /// 1 when the PM turn-off message received by RADM source is driving the IRQ output.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod radm_pm_turnoff {
        pub type R = crate::BitReader;
    }
    /// 1 when the physical layer link-up event source is driving the IRQ output.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod smlh_link_up {
        pub type R = crate::BitReader;
    }
    /// 1 when the data link layer link-up event source is driving the IRQ output.
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
