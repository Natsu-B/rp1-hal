// Generated from rp1-regdb/data/peripherals/pio.toml.
// Do not edit manually.
// Regenerate with: cargo run -p rp1-regdb -- generate

use crate::Reg;

/// DOC: APB FIFO window (Linux rp1.dtsi and rp1-pio driver). DMA uses bus address 0xc040178000. Atomic aliases are address-map metadata; FIFO accesses use the base window.
/// Atomic view: Rp1Native alias windows; ownership remains with this logical peripheral.
#[repr(C)]
pub struct RegisterBlock {
    txf: [TXF; 4],
    rxf: [RXF; 4],
}

impl RegisterBlock {
    /// Hardware access: read None; write Fifo.
    /// FIFO/data port; only dedicated FIFO operations are generated.
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
}

/// Hardware access: read None; write Fifo.
/// FIFO/data port; only dedicated FIFO operations are generated.
/// Array layout: 4 physical entries, 4 public entries, stride 0x4 bytes.
pub type TXF = Reg<txf::TXF_SPEC>;
/// Hardware access: read None; write Fifo.
/// FIFO/data port; only dedicated FIFO operations are generated.
/// Array layout: 4 physical entries, 4 public entries, stride 0x4 bytes.
pub mod txf {
    pub struct TXF_SPEC;
    impl crate::RegisterSpec for TXF_SPEC {
        type Ux = u32;
    }
    impl crate::FifoPush for TXF_SPEC {}
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

#[cfg(test)]
mod layout_tests {
    use super::RegisterBlock;
    #[test]
    fn txf_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, txf), 0x0);
    }
    #[test]
    fn rxf_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, rxf), 0x10);
    }
    #[test]
    fn register_block_size() {
        assert_eq!(core::mem::size_of::<RegisterBlock>(), 0x20);
    }
}
