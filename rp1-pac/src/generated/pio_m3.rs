// Generated from rp1-regdb/data/peripherals/pio-m3.toml.
// Do not edit manually.
// Regenerate with: cargo run -p rp1-regdb -- generate

use crate::Reg;

#[repr(C)]
struct SM_CLKDIVArrayElement {
    sm_clkdiv: SM_CLKDIV,
    sm_execctrl: SM_EXECCTRL,
    sm_shiftctrl: SM_SHIFTCTRL,
    sm_addr: SM_ADDR,
    sm_instr: SM_INSTR,
    sm_pinctrl: SM_PINCTRL,
    sm_dmatx: SM_DMATX,
    sm_dmarx: SM_DMARX,
}

/// STATIC: M3 control window (MichaelBell/rp1-hacking PIO.md and stock firmware register accesses). FIFO views share the APB window's owner.
#[repr(C)]
pub struct RegisterBlock {
    ctrl: CTRL,
    fstat: FSTAT,
    fdebug: FDEBUG,
    flevel: FLEVEL,
    flevel2: FLEVEL2,
    txf: [TXF; 4],
    rxf: [RXF; 4],
    irq: IRQ,
    irq_force: IRQ_FORCE,
    input_sync_bypass: INPUT_SYNC_BYPASS,
    dbg_padout: DBG_PADOUT,
    dbg_padoe: DBG_PADOE,
    dbg_cfginfo: DBG_CFGINFO,
    instr_mem: [INSTR_MEM; 32],
    sm_clkdiv: [SM_CLKDIVArrayElement; 4],
    intr: INTR,
    irq0_inte: IRQ0_INTE,
    irq0_intf: IRQ0_INTF,
    irq0_ints: IRQ0_INTS,
    irq1_inte: IRQ1_INTE,
    irq1_intf: IRQ1_INTF,
    irq1_ints: IRQ1_INTS,
}

impl RegisterBlock {
    /// SM enable and restart commands.
    /// Hardware access: read Plain; write Command.
    /// Command write; ordinary write and modify APIs are not generated.
    /// Generated writes require unsafe raw access.
    #[inline(always)]
    pub const fn ctrl(&self) -> &CTRL {
        &self.ctrl
    }
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn fstat(&self) -> &FSTAT {
        &self.fstat
    }
    /// Hardware access: read Plain; write WriteOneToClear.
    /// Write-one-to-clear register; general write and modify APIs are not generated.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn fdebug(&self) -> &FDEBUG {
        &self.fdebug
    }
    /// Level 15 also represents 16 when joined.
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn flevel(&self) -> &FLEVEL {
        &self.flevel
    }
    /// Candidate upper level bits in JOIN mode.
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn flevel2(&self) -> &FLEVEL2 {
        &self.flevel2
    }
    /// Hardware access: read None; write Fifo.
    /// FIFO/data port.
    /// Generated writes require unsafe raw access.
    /// Array layout: 4 physical entries, 4 public entries, stride 0x4 bytes.
    #[inline(always)]
    pub fn txf(&self, index: usize) -> &TXF {
        assert!(index < 4);
        &self.txf[index]
    }
    #[inline(always)]
    pub fn txf_array(&self) -> &[TXF] {
        &self.txf[..4]
    }
    /// Hardware access: read Fifo; write None.
    /// Array layout: 4 physical entries, 4 public entries, stride 0x4 bytes.
    #[inline(always)]
    pub fn rxf(&self, index: usize) -> &RXF {
        assert!(index < 4);
        &self.rxf[index]
    }
    #[inline(always)]
    pub fn rxf_array(&self) -> &[RXF] {
        &self.rxf[..4]
    }
    /// Hardware access: read Plain; write WriteOneToClear.
    /// Write-one-to-clear register; general write and modify APIs are not generated.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn irq(&self) -> &IRQ {
        &self.irq
    }
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn input_sync_bypass(&self) -> &INPUT_SYNC_BYPASS {
        &self.input_sync_bypass
    }
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn dbg_padout(&self) -> &DBG_PADOUT {
        &self.dbg_padout
    }
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn dbg_padoe(&self) -> &DBG_PADOE {
        &self.dbg_padoe
    }
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn dbg_cfginfo(&self) -> &DBG_CFGINFO {
        &self.dbg_cfginfo
    }
    /// 32 instruction slots, each with a 16-bit instruction. Use word writes.
    /// Hardware access: read None; write WriteOnly.
    /// Command write; ordinary read, write, and modify APIs are not generated.
    /// Generated writes require unsafe raw access.
    /// Array layout: 32 physical entries, 32 public entries, stride 0x4 bytes.
    #[inline(always)]
    pub fn instr_mem(&self, index: usize) -> &INSTR_MEM {
        assert!(index < 32);
        &self.instr_mem[index]
    }
    #[inline(always)]
    pub fn instr_mem_array(&self) -> &[INSTR_MEM] {
        &self.instr_mem[..32]
    }
    /// Hardware access: read Plain; write Plain.
    /// Generated writes require unsafe raw access.
    /// Array layout: 4 physical entries, 4 public entries, stride 0x20 bytes.
    #[inline(always)]
    pub fn sm_clkdiv(&self, index: usize) -> &SM_CLKDIV {
        assert!(index < 4);
        &self.sm_clkdiv[index].sm_clkdiv
    }
    /// Hardware access: read Plain; write Plain.
    /// Generated writes require unsafe raw access.
    /// Array layout: 4 physical entries, 4 public entries, stride 0x20 bytes.
    #[inline(always)]
    pub fn sm_execctrl(&self, index: usize) -> &SM_EXECCTRL {
        assert!(index < 4);
        &self.sm_clkdiv[index].sm_execctrl
    }
    /// Hardware access: read Plain; write Plain.
    /// Generated writes require unsafe raw access.
    /// Array layout: 4 physical entries, 4 public entries, stride 0x20 bytes.
    #[inline(always)]
    pub fn sm_shiftctrl(&self, index: usize) -> &SM_SHIFTCTRL {
        assert!(index < 4);
        &self.sm_clkdiv[index].sm_shiftctrl
    }
    /// Hardware access: read Plain; write None.
    /// Array layout: 4 physical entries, 4 public entries, stride 0x20 bytes.
    #[inline(always)]
    pub fn sm_addr(&self, index: usize) -> &SM_ADDR {
        assert!(index < 4);
        &self.sm_clkdiv[index].sm_addr
    }
    /// Hardware access: read Plain; write Command.
    /// Command write; ordinary write and modify APIs are not generated.
    /// Generated writes require unsafe raw access.
    /// Array layout: 4 physical entries, 4 public entries, stride 0x20 bytes.
    #[inline(always)]
    pub fn sm_instr(&self, index: usize) -> &SM_INSTR {
        assert!(index < 4);
        &self.sm_clkdiv[index].sm_instr
    }
    /// Hardware access: read Plain; write Plain.
    /// Generated writes require unsafe raw access.
    /// Array layout: 4 physical entries, 4 public entries, stride 0x20 bytes.
    #[inline(always)]
    pub fn sm_pinctrl(&self, index: usize) -> &SM_PINCTRL {
        assert!(index < 4);
        &self.sm_clkdiv[index].sm_pinctrl
    }
    /// Bit 30 is a read-only raw request status candidate. Bit 31 enables the DMA request.
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    /// Array layout: 4 physical entries, 4 public entries, stride 0x20 bytes.
    #[inline(always)]
    pub fn sm_dmatx(&self, index: usize) -> &SM_DMATX {
        assert!(index < 4);
        &self.sm_clkdiv[index].sm_dmatx
    }
    /// Bit 30 is a read-only raw request status candidate. Bit 31 enables the DMA request.
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    /// Array layout: 4 physical entries, 4 public entries, stride 0x20 bytes.
    #[inline(always)]
    pub fn sm_dmarx(&self, index: usize) -> &SM_DMARX {
        assert!(index < 4);
        &self.sm_clkdiv[index].sm_dmarx
    }
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn intr(&self) -> &INTR {
        &self.intr
    }
    /// Hardware access: read Plain; write Plain.
    /// Generated writes require unsafe raw access.
    #[inline(always)]
    pub const fn irq0_inte(&self) -> &IRQ0_INTE {
        &self.irq0_inte
    }
    /// Hardware access: read Plain; write Plain.
    /// Generated writes require unsafe raw access.
    #[inline(always)]
    pub const fn irq0_intf(&self) -> &IRQ0_INTF {
        &self.irq0_intf
    }
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn irq0_ints(&self) -> &IRQ0_INTS {
        &self.irq0_ints
    }
    /// Hardware access: read Plain; write Plain.
    /// Generated writes require unsafe raw access.
    #[inline(always)]
    pub const fn irq1_inte(&self) -> &IRQ1_INTE {
        &self.irq1_inte
    }
    /// Hardware access: read Plain; write Plain.
    /// Generated writes require unsafe raw access.
    #[inline(always)]
    pub const fn irq1_intf(&self) -> &IRQ1_INTF {
        &self.irq1_intf
    }
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn irq1_ints(&self) -> &IRQ1_INTS {
        &self.irq1_ints
    }
}

/// SM enable and restart commands.
/// Hardware access: read Plain; write Command.
/// Command write; ordinary write and modify APIs are not generated.
/// Generated writes require unsafe raw access.
pub type CTRL = Reg<ctrl::CTRL_SPEC>;
/// SM enable and restart commands.
/// Hardware access: read Plain; write Command.
/// Command write; ordinary write and modify APIs are not generated.
/// Generated writes require unsafe raw access.
pub mod ctrl {
    pub struct CTRL_SPEC;
    impl crate::RegisterSpec for CTRL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CTRL_SPEC {}
    pub type R = crate::R<CTRL_SPEC>;
    impl crate::UnsafeWritable for CTRL_SPEC {}
}

/// Hardware access: read Plain; write None.
pub type FSTAT = Reg<fstat::FSTAT_SPEC>;
/// Hardware access: read Plain; write None.
pub mod fstat {
    pub struct FSTAT_SPEC;
    impl crate::RegisterSpec for FSTAT_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for FSTAT_SPEC {}
    pub type R = crate::R<FSTAT_SPEC>;
}

/// Hardware access: read Plain; write WriteOneToClear.
/// Write-one-to-clear register; general write and modify APIs are not generated.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type FDEBUG = Reg<fdebug::FDEBUG_SPEC>;
/// Hardware access: read Plain; write WriteOneToClear.
/// Write-one-to-clear register; general write and modify APIs are not generated.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod fdebug {
    pub struct FDEBUG_SPEC;
    impl crate::RegisterSpec for FDEBUG_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for FDEBUG_SPEC {}
    pub type R = crate::R<FDEBUG_SPEC>;
}

/// Level 15 also represents 16 when joined.
/// Hardware access: read Plain; write None.
pub type FLEVEL = Reg<flevel::FLEVEL_SPEC>;
/// Level 15 also represents 16 when joined.
/// Hardware access: read Plain; write None.
pub mod flevel {
    pub struct FLEVEL_SPEC;
    impl crate::RegisterSpec for FLEVEL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for FLEVEL_SPEC {}
    pub type R = crate::R<FLEVEL_SPEC>;
}

/// Candidate upper level bits in JOIN mode.
/// Hardware access: read Plain; write None.
pub type FLEVEL2 = Reg<flevel2::FLEVEL2_SPEC>;
/// Candidate upper level bits in JOIN mode.
/// Hardware access: read Plain; write None.
pub mod flevel2 {
    pub struct FLEVEL2_SPEC;
    impl crate::RegisterSpec for FLEVEL2_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for FLEVEL2_SPEC {}
    pub type R = crate::R<FLEVEL2_SPEC>;
}

/// Hardware access: read None; write Fifo.
/// FIFO/data port.
/// Generated writes require unsafe raw access.
/// Array layout: 4 physical entries, 4 public entries, stride 0x4 bytes.
pub type TXF = Reg<txf::TXF_SPEC>;
/// Hardware access: read None; write Fifo.
/// FIFO/data port.
/// Generated writes require unsafe raw access.
/// Array layout: 4 physical entries, 4 public entries, stride 0x4 bytes.
pub mod txf {
    pub struct TXF_SPEC;
    impl crate::RegisterSpec for TXF_SPEC {
        type Ux = u32;
    }
    impl crate::UnsafeWritable for TXF_SPEC {}
}

/// Hardware access: read Fifo; write None.
/// Array layout: 4 physical entries, 4 public entries, stride 0x4 bytes.
pub type RXF = Reg<rxf::RXF_SPEC>;
/// Hardware access: read Fifo; write None.
/// Array layout: 4 physical entries, 4 public entries, stride 0x4 bytes.
pub mod rxf {
    pub struct RXF_SPEC;
    impl crate::RegisterSpec for RXF_SPEC {
        type Ux = u32;
    }
    impl crate::FifoPop for RXF_SPEC {}
}

/// Hardware access: read Plain; write WriteOneToClear.
/// Write-one-to-clear register; general write and modify APIs are not generated.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type IRQ = Reg<irq::IRQ_SPEC>;
/// Hardware access: read Plain; write WriteOneToClear.
/// Write-one-to-clear register; general write and modify APIs are not generated.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod irq {
    pub struct IRQ_SPEC;
    impl crate::RegisterSpec for IRQ_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for IRQ_SPEC {}
    pub type R = crate::R<IRQ_SPEC>;
}

/// Hardware access: read None; write Command.
/// Command write; ordinary read, write, and modify APIs are not generated.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type IRQ_FORCE = Reg<irq_force::IRQ_FORCE_SPEC>;
/// Hardware access: read None; write Command.
/// Command write; ordinary read, write, and modify APIs are not generated.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod irq_force {
    pub struct IRQ_FORCE_SPEC;
    impl crate::RegisterSpec for IRQ_FORCE_SPEC {
        type Ux = u32;
    }
}

/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type INPUT_SYNC_BYPASS = Reg<input_sync_bypass::INPUT_SYNC_BYPASS_SPEC>;
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod input_sync_bypass {
    pub struct INPUT_SYNC_BYPASS_SPEC;
    impl crate::RegisterSpec for INPUT_SYNC_BYPASS_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for INPUT_SYNC_BYPASS_SPEC {}
    pub type R = crate::R<INPUT_SYNC_BYPASS_SPEC>;
}

/// Hardware access: read Plain; write None.
pub type DBG_PADOUT = Reg<dbg_padout::DBG_PADOUT_SPEC>;
/// Hardware access: read Plain; write None.
pub mod dbg_padout {
    pub struct DBG_PADOUT_SPEC;
    impl crate::RegisterSpec for DBG_PADOUT_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for DBG_PADOUT_SPEC {}
    pub type R = crate::R<DBG_PADOUT_SPEC>;
}

/// Hardware access: read Plain; write None.
pub type DBG_PADOE = Reg<dbg_padoe::DBG_PADOE_SPEC>;
/// Hardware access: read Plain; write None.
pub mod dbg_padoe {
    pub struct DBG_PADOE_SPEC;
    impl crate::RegisterSpec for DBG_PADOE_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for DBG_PADOE_SPEC {}
    pub type R = crate::R<DBG_PADOE_SPEC>;
}

/// Hardware access: read Plain; write None.
pub type DBG_CFGINFO = Reg<dbg_cfginfo::DBG_CFGINFO_SPEC>;
/// Hardware access: read Plain; write None.
pub mod dbg_cfginfo {
    pub struct DBG_CFGINFO_SPEC;
    impl crate::RegisterSpec for DBG_CFGINFO_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for DBG_CFGINFO_SPEC {}
    pub type R = crate::R<DBG_CFGINFO_SPEC>;
}

/// 32 instruction slots, each with a 16-bit instruction. Use word writes.
/// Hardware access: read None; write WriteOnly.
/// Command write; ordinary read, write, and modify APIs are not generated.
/// Generated writes require unsafe raw access.
/// Array layout: 32 physical entries, 32 public entries, stride 0x4 bytes.
pub type INSTR_MEM = Reg<instr_mem::INSTR_MEM_SPEC>;
/// 32 instruction slots, each with a 16-bit instruction. Use word writes.
/// Hardware access: read None; write WriteOnly.
/// Command write; ordinary read, write, and modify APIs are not generated.
/// Generated writes require unsafe raw access.
/// Array layout: 32 physical entries, 32 public entries, stride 0x4 bytes.
pub mod instr_mem {
    pub struct INSTR_MEM_SPEC;
    impl crate::RegisterSpec for INSTR_MEM_SPEC {
        type Ux = u32;
    }
    impl crate::UnsafeWritable for INSTR_MEM_SPEC {}
}

/// Hardware access: read Plain; write Plain.
/// Generated writes require unsafe raw access.
/// Array layout: 4 physical entries, 4 public entries, stride 0x20 bytes.
pub type SM_CLKDIV = Reg<sm_clkdiv::SM_CLKDIV_SPEC>;
/// Hardware access: read Plain; write Plain.
/// Generated writes require unsafe raw access.
/// Array layout: 4 physical entries, 4 public entries, stride 0x20 bytes.
pub mod sm_clkdiv {
    pub struct SM_CLKDIV_SPEC;
    impl crate::RegisterSpec for SM_CLKDIV_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for SM_CLKDIV_SPEC {}
    pub type R = crate::R<SM_CLKDIV_SPEC>;
    impl crate::UnsafeWritable for SM_CLKDIV_SPEC {}
}

/// Hardware access: read Plain; write Plain.
/// Generated writes require unsafe raw access.
/// Array layout: 4 physical entries, 4 public entries, stride 0x20 bytes.
pub type SM_EXECCTRL = Reg<sm_execctrl::SM_EXECCTRL_SPEC>;
/// Hardware access: read Plain; write Plain.
/// Generated writes require unsafe raw access.
/// Array layout: 4 physical entries, 4 public entries, stride 0x20 bytes.
pub mod sm_execctrl {
    pub struct SM_EXECCTRL_SPEC;
    impl crate::RegisterSpec for SM_EXECCTRL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for SM_EXECCTRL_SPEC {}
    pub type R = crate::R<SM_EXECCTRL_SPEC>;
    impl crate::UnsafeWritable for SM_EXECCTRL_SPEC {}
}

/// Hardware access: read Plain; write Plain.
/// Generated writes require unsafe raw access.
/// Array layout: 4 physical entries, 4 public entries, stride 0x20 bytes.
pub type SM_SHIFTCTRL = Reg<sm_shiftctrl::SM_SHIFTCTRL_SPEC>;
/// Hardware access: read Plain; write Plain.
/// Generated writes require unsafe raw access.
/// Array layout: 4 physical entries, 4 public entries, stride 0x20 bytes.
pub mod sm_shiftctrl {
    pub struct SM_SHIFTCTRL_SPEC;
    impl crate::RegisterSpec for SM_SHIFTCTRL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for SM_SHIFTCTRL_SPEC {}
    pub type R = crate::R<SM_SHIFTCTRL_SPEC>;
    impl crate::UnsafeWritable for SM_SHIFTCTRL_SPEC {}
}

/// Hardware access: read Plain; write None.
/// Array layout: 4 physical entries, 4 public entries, stride 0x20 bytes.
pub type SM_ADDR = Reg<sm_addr::SM_ADDR_SPEC>;
/// Hardware access: read Plain; write None.
/// Array layout: 4 physical entries, 4 public entries, stride 0x20 bytes.
pub mod sm_addr {
    pub struct SM_ADDR_SPEC;
    impl crate::RegisterSpec for SM_ADDR_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for SM_ADDR_SPEC {}
    pub type R = crate::R<SM_ADDR_SPEC>;
}

/// Hardware access: read Plain; write Command.
/// Command write; ordinary write and modify APIs are not generated.
/// Generated writes require unsafe raw access.
/// Array layout: 4 physical entries, 4 public entries, stride 0x20 bytes.
pub type SM_INSTR = Reg<sm_instr::SM_INSTR_SPEC>;
/// Hardware access: read Plain; write Command.
/// Command write; ordinary write and modify APIs are not generated.
/// Generated writes require unsafe raw access.
/// Array layout: 4 physical entries, 4 public entries, stride 0x20 bytes.
pub mod sm_instr {
    pub struct SM_INSTR_SPEC;
    impl crate::RegisterSpec for SM_INSTR_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for SM_INSTR_SPEC {}
    pub type R = crate::R<SM_INSTR_SPEC>;
    impl crate::UnsafeWritable for SM_INSTR_SPEC {}
}

/// Hardware access: read Plain; write Plain.
/// Generated writes require unsafe raw access.
/// Array layout: 4 physical entries, 4 public entries, stride 0x20 bytes.
pub type SM_PINCTRL = Reg<sm_pinctrl::SM_PINCTRL_SPEC>;
/// Hardware access: read Plain; write Plain.
/// Generated writes require unsafe raw access.
/// Array layout: 4 physical entries, 4 public entries, stride 0x20 bytes.
pub mod sm_pinctrl {
    pub struct SM_PINCTRL_SPEC;
    impl crate::RegisterSpec for SM_PINCTRL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for SM_PINCTRL_SPEC {}
    pub type R = crate::R<SM_PINCTRL_SPEC>;
    impl crate::UnsafeWritable for SM_PINCTRL_SPEC {}
}

/// Bit 30 is a read-only raw request status candidate. Bit 31 enables the DMA request.
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
/// Array layout: 4 physical entries, 4 public entries, stride 0x20 bytes.
pub type SM_DMATX = Reg<sm_dmatx::SM_DMATX_SPEC>;
/// Bit 30 is a read-only raw request status candidate. Bit 31 enables the DMA request.
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
/// Array layout: 4 physical entries, 4 public entries, stride 0x20 bytes.
pub mod sm_dmatx {
    pub struct SM_DMATX_SPEC;
    impl crate::RegisterSpec for SM_DMATX_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for SM_DMATX_SPEC {}
    pub type R = crate::R<SM_DMATX_SPEC>;
}

/// Bit 30 is a read-only raw request status candidate. Bit 31 enables the DMA request.
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
/// Array layout: 4 physical entries, 4 public entries, stride 0x20 bytes.
pub type SM_DMARX = Reg<sm_dmarx::SM_DMARX_SPEC>;
/// Bit 30 is a read-only raw request status candidate. Bit 31 enables the DMA request.
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
/// Array layout: 4 physical entries, 4 public entries, stride 0x20 bytes.
pub mod sm_dmarx {
    pub struct SM_DMARX_SPEC;
    impl crate::RegisterSpec for SM_DMARX_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for SM_DMARX_SPEC {}
    pub type R = crate::R<SM_DMARX_SPEC>;
}

/// Hardware access: read Plain; write None.
pub type INTR = Reg<intr::INTR_SPEC>;
/// Hardware access: read Plain; write None.
pub mod intr {
    pub struct INTR_SPEC;
    impl crate::RegisterSpec for INTR_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for INTR_SPEC {}
    pub type R = crate::R<INTR_SPEC>;
}

/// Hardware access: read Plain; write Plain.
/// Generated writes require unsafe raw access.
pub type IRQ0_INTE = Reg<irq0_inte::IRQ0_INTE_SPEC>;
/// Hardware access: read Plain; write Plain.
/// Generated writes require unsafe raw access.
pub mod irq0_inte {
    pub struct IRQ0_INTE_SPEC;
    impl crate::RegisterSpec for IRQ0_INTE_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for IRQ0_INTE_SPEC {}
    pub type R = crate::R<IRQ0_INTE_SPEC>;
    impl crate::UnsafeWritable for IRQ0_INTE_SPEC {}
}

/// Hardware access: read Plain; write Plain.
/// Generated writes require unsafe raw access.
pub type IRQ0_INTF = Reg<irq0_intf::IRQ0_INTF_SPEC>;
/// Hardware access: read Plain; write Plain.
/// Generated writes require unsafe raw access.
pub mod irq0_intf {
    pub struct IRQ0_INTF_SPEC;
    impl crate::RegisterSpec for IRQ0_INTF_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for IRQ0_INTF_SPEC {}
    pub type R = crate::R<IRQ0_INTF_SPEC>;
    impl crate::UnsafeWritable for IRQ0_INTF_SPEC {}
}

/// Hardware access: read Plain; write None.
pub type IRQ0_INTS = Reg<irq0_ints::IRQ0_INTS_SPEC>;
/// Hardware access: read Plain; write None.
pub mod irq0_ints {
    pub struct IRQ0_INTS_SPEC;
    impl crate::RegisterSpec for IRQ0_INTS_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for IRQ0_INTS_SPEC {}
    pub type R = crate::R<IRQ0_INTS_SPEC>;
}

/// Hardware access: read Plain; write Plain.
/// Generated writes require unsafe raw access.
pub type IRQ1_INTE = Reg<irq1_inte::IRQ1_INTE_SPEC>;
/// Hardware access: read Plain; write Plain.
/// Generated writes require unsafe raw access.
pub mod irq1_inte {
    pub struct IRQ1_INTE_SPEC;
    impl crate::RegisterSpec for IRQ1_INTE_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for IRQ1_INTE_SPEC {}
    pub type R = crate::R<IRQ1_INTE_SPEC>;
    impl crate::UnsafeWritable for IRQ1_INTE_SPEC {}
}

/// Hardware access: read Plain; write Plain.
/// Generated writes require unsafe raw access.
pub type IRQ1_INTF = Reg<irq1_intf::IRQ1_INTF_SPEC>;
/// Hardware access: read Plain; write Plain.
/// Generated writes require unsafe raw access.
pub mod irq1_intf {
    pub struct IRQ1_INTF_SPEC;
    impl crate::RegisterSpec for IRQ1_INTF_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for IRQ1_INTF_SPEC {}
    pub type R = crate::R<IRQ1_INTF_SPEC>;
    impl crate::UnsafeWritable for IRQ1_INTF_SPEC {}
}

/// Hardware access: read Plain; write None.
pub type IRQ1_INTS = Reg<irq1_ints::IRQ1_INTS_SPEC>;
/// Hardware access: read Plain; write None.
pub mod irq1_ints {
    pub struct IRQ1_INTS_SPEC;
    impl crate::RegisterSpec for IRQ1_INTS_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for IRQ1_INTS_SPEC {}
    pub type R = crate::R<IRQ1_INTS_SPEC>;
}

#[cfg(test)]
mod layout_tests {
    use super::RegisterBlock;
    #[test]
    fn ctrl_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, ctrl), 0x0);
    }
    #[test]
    fn fstat_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, fstat), 0x4);
    }
    #[test]
    fn fdebug_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, fdebug), 0x8);
    }
    #[test]
    fn flevel_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, flevel), 0xc);
    }
    #[test]
    fn flevel2_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, flevel2), 0x10);
    }
    #[test]
    fn txf_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, txf), 0x14);
    }
    #[test]
    fn rxf_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, rxf), 0x24);
    }
    #[test]
    fn irq_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, irq), 0x34);
    }
    #[test]
    fn irq_force_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, irq_force), 0x38);
    }
    #[test]
    fn input_sync_bypass_offset() {
        assert_eq!(
            core::mem::offset_of!(RegisterBlock, input_sync_bypass),
            0x3c
        );
    }
    #[test]
    fn dbg_padout_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, dbg_padout), 0x40);
    }
    #[test]
    fn dbg_padoe_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, dbg_padoe), 0x44);
    }
    #[test]
    fn dbg_cfginfo_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, dbg_cfginfo), 0x48);
    }
    #[test]
    fn instr_mem_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, instr_mem), 0x4c);
    }
    #[test]
    fn sm_clkdiv_offset() {
        assert_eq!(
            core::mem::offset_of!(RegisterBlock, sm_clkdiv)
                + core::mem::offset_of!(super::SM_CLKDIVArrayElement, sm_clkdiv),
            0xcc
        );
    }
    #[test]
    fn sm_clkdiv_stride() {
        assert_eq!(core::mem::size_of::<super::SM_CLKDIVArrayElement>(), 0x20);
    }
    #[test]
    fn sm_execctrl_offset() {
        assert_eq!(
            core::mem::offset_of!(RegisterBlock, sm_clkdiv)
                + core::mem::offset_of!(super::SM_CLKDIVArrayElement, sm_execctrl),
            0xd0
        );
    }
    #[test]
    fn sm_execctrl_stride() {
        assert_eq!(core::mem::size_of::<super::SM_CLKDIVArrayElement>(), 0x20);
    }
    #[test]
    fn sm_shiftctrl_offset() {
        assert_eq!(
            core::mem::offset_of!(RegisterBlock, sm_clkdiv)
                + core::mem::offset_of!(super::SM_CLKDIVArrayElement, sm_shiftctrl),
            0xd4
        );
    }
    #[test]
    fn sm_shiftctrl_stride() {
        assert_eq!(core::mem::size_of::<super::SM_CLKDIVArrayElement>(), 0x20);
    }
    #[test]
    fn sm_addr_offset() {
        assert_eq!(
            core::mem::offset_of!(RegisterBlock, sm_clkdiv)
                + core::mem::offset_of!(super::SM_CLKDIVArrayElement, sm_addr),
            0xd8
        );
    }
    #[test]
    fn sm_addr_stride() {
        assert_eq!(core::mem::size_of::<super::SM_CLKDIVArrayElement>(), 0x20);
    }
    #[test]
    fn sm_instr_offset() {
        assert_eq!(
            core::mem::offset_of!(RegisterBlock, sm_clkdiv)
                + core::mem::offset_of!(super::SM_CLKDIVArrayElement, sm_instr),
            0xdc
        );
    }
    #[test]
    fn sm_instr_stride() {
        assert_eq!(core::mem::size_of::<super::SM_CLKDIVArrayElement>(), 0x20);
    }
    #[test]
    fn sm_pinctrl_offset() {
        assert_eq!(
            core::mem::offset_of!(RegisterBlock, sm_clkdiv)
                + core::mem::offset_of!(super::SM_CLKDIVArrayElement, sm_pinctrl),
            0xe0
        );
    }
    #[test]
    fn sm_pinctrl_stride() {
        assert_eq!(core::mem::size_of::<super::SM_CLKDIVArrayElement>(), 0x20);
    }
    #[test]
    fn sm_dmatx_offset() {
        assert_eq!(
            core::mem::offset_of!(RegisterBlock, sm_clkdiv)
                + core::mem::offset_of!(super::SM_CLKDIVArrayElement, sm_dmatx),
            0xe4
        );
    }
    #[test]
    fn sm_dmatx_stride() {
        assert_eq!(core::mem::size_of::<super::SM_CLKDIVArrayElement>(), 0x20);
    }
    #[test]
    fn sm_dmarx_offset() {
        assert_eq!(
            core::mem::offset_of!(RegisterBlock, sm_clkdiv)
                + core::mem::offset_of!(super::SM_CLKDIVArrayElement, sm_dmarx),
            0xe8
        );
    }
    #[test]
    fn sm_dmarx_stride() {
        assert_eq!(core::mem::size_of::<super::SM_CLKDIVArrayElement>(), 0x20);
    }
    #[test]
    fn intr_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, intr), 0x14c);
    }
    #[test]
    fn irq0_inte_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, irq0_inte), 0x150);
    }
    #[test]
    fn irq0_intf_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, irq0_intf), 0x154);
    }
    #[test]
    fn irq0_ints_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, irq0_ints), 0x158);
    }
    #[test]
    fn irq1_inte_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, irq1_inte), 0x15c);
    }
    #[test]
    fn irq1_intf_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, irq1_intf), 0x160);
    }
    #[test]
    fn irq1_ints_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, irq1_ints), 0x164);
    }
    #[test]
    fn register_block_size() {
        assert_eq!(core::mem::size_of::<RegisterBlock>(), 0x168);
    }
}
