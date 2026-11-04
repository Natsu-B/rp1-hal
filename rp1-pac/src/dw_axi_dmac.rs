//! Synopsys DesignWare AXI DMAC (DW_axi_dmac 1.02a-lp02) register map of the RP1 `DMA` block.
//!
//! Evidence per register (strength in brackets, as in the repository's design notes):
//! - [DOC] RP1 Peripherals (2023-11, ch. 9): eight channels, one 128-bit AXI master, maximum
//!   block size 2^18 - 1 elements, channel abort allowed, no channel locking, clk_dma 100 MHz,
//!   base 0x4018_8000 (AHB, no atomic aliases). Channels "1 and 2" (Linux numbering 0 and 1)
//!   have 8-beat FIFOs, the others 4-beat.
//! - [DOC] Linux `drivers/dma/dw-axi-dmac/dw-axi-dmac.h` and `dw-axi-dmac-platform.c`
//!   (rpi-6.12.y) for every offset and field below, and for the register layout variant:
//!   the RP1 node has `dma-channels = <8>` (8-channel map, `DMAC_CHEN` write-enable at bit 8)
//!   and `snps,dma-targets = <64>` (> 16, so the driver uses the `CFG2` layout of `CH_CFG`).
//! - [DOC] Linux `rp1.dtsi`: `snps,axi-max-burst-len = <8 8 4 4 4 4 4 4>`,
//!   `snps,block-size = <0x40000 ..>`, `snps,data-width = <4>` (128 bit).
//! - [HW] is added per item only when a lab run confirmed it (see `docs/dma.md`).
//!
//! Every register is 64 bits wide; like Linux, access it as two 32-bit halves (low first).
//! Offsets of the common block are from the DMAC base; channel `n` (0..8) registers are at
//! `CHANNEL_BASE + n * CHANNEL_STRIDE`.

/// RP1 processor (32-bit) view of the register block.
pub const BASE: usize = 0x4018_8000;
pub const CHANNELS: u8 = 8;
/// Elements per block (BLOCK_TS + 1), DOC: datasheet 2^18 - 1; Linux `snps,block-size` 0x40000.
pub const MAX_BLOCK_TS: u32 = (1 << 18) - 1;
pub const CHANNEL_BASE: usize = 0x100;
pub const CHANNEL_STRIDE: usize = 0x100;

/// Common registers.
pub mod common {
    /// R: DMAC ID. [DOC]
    pub const ID: usize = 0x000;
    /// R: component version (`DMAC_COMPVER`). [DOC]
    pub const COMPVER: usize = 0x008;
    /// RW: `DMAC_CFG`: bit 0 DMAC_EN, bit 1 INT_EN. [DOC]
    pub const CFG: usize = 0x010;
    /// RW: `DMAC_CHEN` (8-channel map): bits 0..7 CH_EN, 8..15 CH_EN_WE, 16..23 CH_SUSP,
    /// 24..31 CH_SUSP_WE. A bit is written only together with its write-enable. [DOC]
    pub const CHEN: usize = 0x018;
    /// RW: `DMAC_CHABORTREG` (8-channel map): bits 0..7 abort, 8..15 write-enable. Channel
    /// abort is configured in on RP1 (datasheet). [DOC]
    pub const CHABORT: usize = 0x028;
    /// R: `DMAC_INTSTATUS`: bit n = channel n interrupt, bit 16 common. [DOC]
    pub const INTSTATUS: usize = 0x030;
    /// W: `DMAC_COMMON_INTCLEAR`. [DOC]
    pub const COMMON_INTCLEAR: usize = 0x038;
    /// RW: `DMAC_COMMON_INTSTATUS_ENA`. [DOC]
    pub const COMMON_INTSTATUS_ENA: usize = 0x040;
    /// RW: `DMAC_COMMON_INTSIGNAL_ENA`. [DOC]
    pub const COMMON_INTSIGNAL_ENA: usize = 0x048;
    /// R: `DMAC_COMMON_INTSTATUS`. [DOC]
    pub const COMMON_INTSTATUS: usize = 0x050;
    /// RW: `DMAC_RESET`: write 1, self-clears when the soft reset is done. [DOC]
    pub const RESET: usize = 0x058;

    pub const CFG_DMAC_EN: u32 = 1 << 0;
    pub const CFG_INT_EN: u32 = 1 << 1;
    pub const CHEN_WE_SHIFT: u32 = 8;
}

/// Channel registers (offsets within one channel's 0x100 window).
pub mod ch {
    /// RW: source address, 64 bit. [DOC]
    pub const SAR: usize = 0x000;
    /// RW: destination address, 64 bit. [DOC]
    pub const DAR: usize = 0x008;
    /// RW: BLOCK_TS = elements of the source width per block, minus one (22 bits). [DOC]
    pub const BLOCK_TS: usize = 0x010;
    /// RW: `CH_CTL` low / high halves. [DOC]
    pub const CTL_L: usize = 0x018;
    pub const CTL_H: usize = 0x01c;
    /// RW: `CH_CFG` low / high halves (CFG2 layout on RP1). [DOC]
    pub const CFG_L: usize = 0x020;
    pub const CFG_H: usize = 0x024;
    /// RW: linked-list pointer (64 bit, bit 0 = LMS master select). [DOC]
    pub const LLP: usize = 0x028;
    /// R: `CH_STATUS`: low half = CMPLTD_BLK_TFR_SIZE (elements done). [DOC]
    pub const STATUS: usize = 0x030;
    /// RW: `CH_INTSTATUS_ENA`. [DOC]
    pub const INTSTATUS_ENA: usize = 0x080;
    /// R: `CH_INTSTATUS` (the Linux header marks it R/W; the driver only reads it). [DOC]
    pub const INTSTATUS: usize = 0x088;
    /// RW: `CH_INTSIGNAL_ENA` (routes status bits to the DMA interrupt line). [DOC]
    pub const INTSIGNAL_ENA: usize = 0x090;
    /// W: `CH_INTCLEAR`. [DOC]
    pub const INTCLEAR: usize = 0x098;
}

/// `CH_CTL` low half. [DOC]
pub mod ctl_l {
    /// AXI master for source / destination fetch (single master on RP1: always 0).
    pub const SMS: u32 = 1 << 0;
    pub const DMS: u32 = 1 << 2;
    /// 0 = increment, 1 = no change.
    pub const SINC_NOINC: u32 = 1 << 4;
    pub const DINC_NOINC: u32 = 1 << 6;
    /// Transfer width: log2(bytes), 0..=4 on RP1 (128-bit master).
    pub const SRC_TR_WIDTH_SHIFT: u32 = 8;
    pub const DST_TR_WIDTH_SHIFT: u32 = 11;
    /// Burst transaction length, encoded 0 = 1, 1 = 4, 2 = 8, .. (`DWAXIDMAC_BURST_TRANS_LEN_*`).
    pub const SRC_MSIZE_SHIFT: u32 = 14;
    pub const DST_MSIZE_SHIFT: u32 = 18;
    pub const MSIZE_4: u32 = 1;
}

/// `CH_CTL` high half. [DOC]
pub mod ctl_h {
    /// Override the AXI read/write burst length with ARLEN / AWLEN.
    pub const ARLEN_EN: u32 = 1 << 6;
    pub const ARLEN_SHIFT: u32 = 7;
    pub const AWLEN_EN: u32 = 1 << 15;
    pub const AWLEN_SHIFT: u32 = 16;
    /// Shadow-register / LLI last and valid (multi-block modes only).
    pub const LLI_LAST: u32 = 1 << 30;
    pub const LLI_VALID: u32 = 1 << 31;
}

/// `CH_CFG`, CFG2 layout (`dma-targets` > 16). [DOC]
pub mod cfg2 {
    /// Low half: multi-block type per side (0 = contiguous, 3 = linked list).
    pub const SRC_MULTBLK_TYPE_SHIFT: u32 = 0;
    pub const DST_MULTBLK_TYPE_SHIFT: u32 = 2;
    /// Low half: hardware handshake interface numbers (6 bits each with 64 targets).
    pub const SRC_PER_SHIFT: u32 = 4;
    pub const DST_PER_SHIFT: u32 = 11;
    /// High half: transfer type and flow control (0 = memory to memory, DMAC flow control).
    pub const TT_FC_SHIFT: u32 = 0;
    /// High half: 1 = software handshake (the default for memory).
    pub const HS_SEL_SRC: u32 = 1 << 3;
    pub const HS_SEL_DST: u32 = 1 << 4;
    pub const PRIORITY_SHIFT: u32 = 20;
}

/// `CH_INTSTATUS` / `_ENA` / `INTSIGNAL_ENA` / `INTCLEAR` bits. [DOC]
pub mod int {
    pub const BLOCK_TFR_DONE: u32 = 1 << 0;
    pub const DMA_TFR_DONE: u32 = 1 << 1;
    pub const SRC_DEC_ERR: u32 = 1 << 5;
    pub const DST_DEC_ERR: u32 = 1 << 6;
    pub const SRC_SLV_ERR: u32 = 1 << 7;
    pub const DST_SLV_ERR: u32 = 1 << 8;
    pub const CH_DISABLED: u32 = 1 << 30;
    pub const CH_ABORTED: u32 = 1 << 31;
    /// `DWAXIDMAC_IRQ_ALL_ERR`: bits 5..=14 and 16..=21.
    pub const ALL_ERR: u32 = (0x3f << 16) | (0x3ff << 5);
    pub const ALL: u32 = u32::MAX;
}
