// Generated from rp1-regdb/data/peripherals/dma-tick.toml.
// Do not edit manually.
// Regenerate with: cargo run -p rp1-regdb -- generate

use crate::Reg;

/// RP1 DMA_TICK_RW register block. Bus=APB. Atomic alias: normal read/write access.
/// Atomic view: Rp1Native alias windows; ownership remains with this logical peripheral.
#[repr(C)]
pub struct RegisterBlock {
    tick0_en: TICK0_EN,
    tick0_ctrl: TICK0_CTRL,
    tick1_en: TICK1_EN,
    tick1_ctrl: TICK1_CTRL,
}

impl RegisterBlock {
    /// DMA tick 0 enable bits
    /// Hardware access: read Plain; write Plain.
    #[inline(always)]
    pub const fn tick0_en(&self) -> &TICK0_EN {
        &self.tick0_en
    }
    /// DMA tick 0 control
    /// Hardware access: read Plain; write Plain.
    #[inline(always)]
    pub const fn tick0_ctrl(&self) -> &TICK0_CTRL {
        &self.tick0_ctrl
    }
    /// DMA tick 1 enable bits
    /// Hardware access: read Plain; write Plain.
    #[inline(always)]
    pub const fn tick1_en(&self) -> &TICK1_EN {
        &self.tick1_en
    }
    /// DMA tick 1 control
    /// Hardware access: read Plain; write Plain.
    #[inline(always)]
    pub const fn tick1_ctrl(&self) -> &TICK1_CTRL {
        &self.tick1_ctrl
    }
}

/// DMA tick 0 enable bits
/// Hardware access: read Plain; write Plain.
pub type TICK0_EN = Reg<tick0_en::TICK0_EN_SPEC>;
/// DMA tick 0 enable bits
/// Hardware access: read Plain; write Plain.
pub mod tick0_en {
    pub struct TICK0_EN_SPEC;
    impl crate::RegisterSpec for TICK0_EN_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for TICK0_EN_SPEC {}
    pub type R = crate::R<TICK0_EN_SPEC>;
    impl crate::Modifiable for TICK0_EN_SPEC {}
    impl crate::UnsafeWritable for TICK0_EN_SPEC {}
    impl crate::ResetValue for TICK0_EN_SPEC {
        const RESET_VALUE: u32 = 0x0;
    }
    impl crate::TypedAtomicSet for TICK0_EN_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicSet for TICK0_EN_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::TypedAtomicClear for TICK0_EN_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicClear for TICK0_EN_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::TypedAtomicXor for TICK0_EN_SPEC {
        const OFFSET: usize = 0x1000;
    }
    impl crate::UnsafeAtomicXor for TICK0_EN_SPEC {
        const OFFSET: usize = 0x1000;
    }
    pub type W = crate::W<TICK0_EN_SPEC>;
    pub type AtomicSetW = crate::AtomicSetW<TICK0_EN_SPEC>;
    pub type AtomicClearW = crate::AtomicClearW<TICK0_EN_SPEC>;
    pub type AtomicXorW = crate::AtomicXorW<TICK0_EN_SPEC>;
    /// Enable generation of handshakes on tick0 output
    /// Field write behavior: Plain.
    pub mod req {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 0>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 0>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 0>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 0>;
    }
    /// Generate dma_single as well as burst requests
    /// Field write behavior: Plain.
    pub mod single {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 1>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 1>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 1>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 1>;
    }
    impl R {
        #[inline(always)]
        pub fn req(&self) -> req::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn single(&self) -> single::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
    }
    impl W {
        #[inline(always)]
        pub fn req(&mut self) -> req::W<'_> {
            req::W::new(self)
        }
        #[inline(always)]
        pub fn single(&mut self) -> single::W<'_> {
            single::W::new(self)
        }
    }
    impl AtomicSetW {
        /// Enable generation of handshakes on tick0 output
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn req(&mut self) -> req::ASW<'_> {
            req::ASW::new(self)
        }
        /// Generate dma_single as well as burst requests
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn single(&mut self) -> single::ASW<'_> {
            single::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// Enable generation of handshakes on tick0 output
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn req(&mut self) -> req::ACW<'_> {
            req::ACW::new(self)
        }
        /// Generate dma_single as well as burst requests
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn single(&mut self) -> single::ACW<'_> {
            single::ACW::new(self)
        }
    }
    impl AtomicXorW {
        /// Enable generation of handshakes on tick0 output
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn req(&mut self) -> req::AXW<'_> {
            req::AXW::new(self)
        }
        /// Generate dma_single as well as burst requests
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn single(&mut self) -> single::AXW<'_> {
            single::AXW::new(self)
        }
    }
}

/// DMA tick 0 control
/// Hardware access: read Plain; write Plain.
pub type TICK0_CTRL = Reg<tick0_ctrl::TICK0_CTRL_SPEC>;
/// DMA tick 0 control
/// Hardware access: read Plain; write Plain.
pub mod tick0_ctrl {
    pub struct TICK0_CTRL_SPEC;
    impl crate::RegisterSpec for TICK0_CTRL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for TICK0_CTRL_SPEC {}
    pub type R = crate::R<TICK0_CTRL_SPEC>;
    impl crate::Modifiable for TICK0_CTRL_SPEC {}
    impl crate::UnsafeWritable for TICK0_CTRL_SPEC {}
    impl crate::ResetValue for TICK0_CTRL_SPEC {
        const RESET_VALUE: u32 = 0x0;
    }
    impl crate::TypedAtomicSet for TICK0_CTRL_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicSet for TICK0_CTRL_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::TypedAtomicClear for TICK0_CTRL_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicClear for TICK0_CTRL_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::TypedAtomicXor for TICK0_CTRL_SPEC {
        const OFFSET: usize = 0x1000;
    }
    impl crate::UnsafeAtomicXor for TICK0_CTRL_SPEC {
        const OFFSET: usize = 0x1000;
    }
    pub type W = crate::W<TICK0_CTRL_SPEC>;
    pub type AtomicSetW = crate::AtomicSetW<TICK0_CTRL_SPEC>;
    pub type AtomicClearW = crate::AtomicClearW<TICK0_CTRL_SPEC>;
    pub type AtomicXorW = crate::AtomicXorW<TICK0_CTRL_SPEC>;
    /// Clear enable/single when DMAC asserts dma_finish
    /// Field write behavior: Plain.
    pub mod finish_clear {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 0>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 0>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 0>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 0>;
    }
    /// Write 1 to SET alias to force-disable handshake
    /// Field write behavior: Plain.
    pub mod disable {
        pub type W<'a> = crate::BitWriter<'a, super::W, 1>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 1>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 1>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 1>;
    }
    /// Handshake state machine idle dwell in bus-clock cycles
    /// Field write behavior: Plain.
    pub mod dwell {
        pub type R = crate::FieldReader<u32>;
        pub type W<'a> = crate::FieldWriter<'a, super::W, 4, 5>;
        pub type ASW<'a> = crate::AtomicSetFieldWriter<'a, super::AtomicSetW, 4, 5>;
        pub type ACW<'a> = crate::AtomicClearFieldWriter<'a, super::AtomicClearW, 4, 5>;
        pub type AXW<'a> = crate::AtomicXorFieldWriter<'a, super::AtomicXorW, 4, 5>;
    }
    /// Current state of dreq input
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod dreq {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn finish_clear(&self) -> finish_clear::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn dwell(&self) -> dwell::R {
            crate::FieldReader::new((self.bits() >> 4) & 0x1f)
        }
        #[inline(always)]
        pub fn dreq(&self) -> dreq::R {
            crate::BitReader::new(((self.bits() >> 12) & 1) != 0)
        }
    }
    impl W {
        #[inline(always)]
        pub fn finish_clear(&mut self) -> finish_clear::W<'_> {
            finish_clear::W::new(self)
        }
        #[inline(always)]
        pub fn disable(&mut self) -> disable::W<'_> {
            disable::W::new(self)
        }
        #[inline(always)]
        pub fn dwell(&mut self) -> dwell::W<'_> {
            dwell::W::new(self)
        }
    }
    impl AtomicSetW {
        /// Clear enable/single when DMAC asserts dma_finish
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn finish_clear(&mut self) -> finish_clear::ASW<'_> {
            finish_clear::ASW::new(self)
        }
        /// Write 1 to SET alias to force-disable handshake
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn disable(&mut self) -> disable::ASW<'_> {
            disable::ASW::new(self)
        }
        /// Handshake state machine idle dwell in bus-clock cycles
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn dwell(&mut self) -> dwell::ASW<'_> {
            dwell::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// Clear enable/single when DMAC asserts dma_finish
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn finish_clear(&mut self) -> finish_clear::ACW<'_> {
            finish_clear::ACW::new(self)
        }
        /// Write 1 to SET alias to force-disable handshake
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn disable(&mut self) -> disable::ACW<'_> {
            disable::ACW::new(self)
        }
        /// Handshake state machine idle dwell in bus-clock cycles
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn dwell(&mut self) -> dwell::ACW<'_> {
            dwell::ACW::new(self)
        }
    }
    impl AtomicXorW {
        /// Clear enable/single when DMAC asserts dma_finish
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn finish_clear(&mut self) -> finish_clear::AXW<'_> {
            finish_clear::AXW::new(self)
        }
        /// Write 1 to SET alias to force-disable handshake
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn disable(&mut self) -> disable::AXW<'_> {
            disable::AXW::new(self)
        }
        /// Handshake state machine idle dwell in bus-clock cycles
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn dwell(&mut self) -> dwell::AXW<'_> {
            dwell::AXW::new(self)
        }
    }
}

/// DMA tick 1 enable bits
/// Hardware access: read Plain; write Plain.
pub type TICK1_EN = Reg<tick1_en::TICK1_EN_SPEC>;
/// DMA tick 1 enable bits
/// Hardware access: read Plain; write Plain.
pub mod tick1_en {
    pub struct TICK1_EN_SPEC;
    impl crate::RegisterSpec for TICK1_EN_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for TICK1_EN_SPEC {}
    pub type R = crate::R<TICK1_EN_SPEC>;
    impl crate::Modifiable for TICK1_EN_SPEC {}
    impl crate::UnsafeWritable for TICK1_EN_SPEC {}
    impl crate::ResetValue for TICK1_EN_SPEC {
        const RESET_VALUE: u32 = 0x0;
    }
    impl crate::TypedAtomicSet for TICK1_EN_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicSet for TICK1_EN_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::TypedAtomicClear for TICK1_EN_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicClear for TICK1_EN_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::TypedAtomicXor for TICK1_EN_SPEC {
        const OFFSET: usize = 0x1000;
    }
    impl crate::UnsafeAtomicXor for TICK1_EN_SPEC {
        const OFFSET: usize = 0x1000;
    }
    pub type W = crate::W<TICK1_EN_SPEC>;
    pub type AtomicSetW = crate::AtomicSetW<TICK1_EN_SPEC>;
    pub type AtomicClearW = crate::AtomicClearW<TICK1_EN_SPEC>;
    pub type AtomicXorW = crate::AtomicXorW<TICK1_EN_SPEC>;
    /// Enable generation of handshakes on tick1 output
    /// Field write behavior: Plain.
    pub mod req {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 0>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 0>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 0>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 0>;
    }
    /// Generate dma_single as well as burst requests
    /// Field write behavior: Plain.
    pub mod single {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 1>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 1>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 1>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 1>;
    }
    impl R {
        #[inline(always)]
        pub fn req(&self) -> req::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn single(&self) -> single::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
    }
    impl W {
        #[inline(always)]
        pub fn req(&mut self) -> req::W<'_> {
            req::W::new(self)
        }
        #[inline(always)]
        pub fn single(&mut self) -> single::W<'_> {
            single::W::new(self)
        }
    }
    impl AtomicSetW {
        /// Enable generation of handshakes on tick1 output
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn req(&mut self) -> req::ASW<'_> {
            req::ASW::new(self)
        }
        /// Generate dma_single as well as burst requests
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn single(&mut self) -> single::ASW<'_> {
            single::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// Enable generation of handshakes on tick1 output
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn req(&mut self) -> req::ACW<'_> {
            req::ACW::new(self)
        }
        /// Generate dma_single as well as burst requests
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn single(&mut self) -> single::ACW<'_> {
            single::ACW::new(self)
        }
    }
    impl AtomicXorW {
        /// Enable generation of handshakes on tick1 output
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn req(&mut self) -> req::AXW<'_> {
            req::AXW::new(self)
        }
        /// Generate dma_single as well as burst requests
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn single(&mut self) -> single::AXW<'_> {
            single::AXW::new(self)
        }
    }
}

/// DMA tick 1 control
/// Hardware access: read Plain; write Plain.
pub type TICK1_CTRL = Reg<tick1_ctrl::TICK1_CTRL_SPEC>;
/// DMA tick 1 control
/// Hardware access: read Plain; write Plain.
pub mod tick1_ctrl {
    pub struct TICK1_CTRL_SPEC;
    impl crate::RegisterSpec for TICK1_CTRL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for TICK1_CTRL_SPEC {}
    pub type R = crate::R<TICK1_CTRL_SPEC>;
    impl crate::Modifiable for TICK1_CTRL_SPEC {}
    impl crate::UnsafeWritable for TICK1_CTRL_SPEC {}
    impl crate::ResetValue for TICK1_CTRL_SPEC {
        const RESET_VALUE: u32 = 0x0;
    }
    impl crate::TypedAtomicSet for TICK1_CTRL_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicSet for TICK1_CTRL_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::TypedAtomicClear for TICK1_CTRL_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicClear for TICK1_CTRL_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::TypedAtomicXor for TICK1_CTRL_SPEC {
        const OFFSET: usize = 0x1000;
    }
    impl crate::UnsafeAtomicXor for TICK1_CTRL_SPEC {
        const OFFSET: usize = 0x1000;
    }
    pub type W = crate::W<TICK1_CTRL_SPEC>;
    pub type AtomicSetW = crate::AtomicSetW<TICK1_CTRL_SPEC>;
    pub type AtomicClearW = crate::AtomicClearW<TICK1_CTRL_SPEC>;
    pub type AtomicXorW = crate::AtomicXorW<TICK1_CTRL_SPEC>;
    /// Clear enable/single when DMAC asserts dma_finish
    /// Field write behavior: Plain.
    pub mod finish_clear {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 0>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 0>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 0>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 0>;
    }
    /// Write 1 to SET alias to force-disable handshake
    /// Field write behavior: Plain.
    pub mod disable {
        pub type W<'a> = crate::BitWriter<'a, super::W, 1>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 1>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 1>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 1>;
    }
    /// Handshake state machine idle dwell in bus-clock cycles
    /// Field write behavior: Plain.
    pub mod dwell {
        pub type R = crate::FieldReader<u32>;
        pub type W<'a> = crate::FieldWriter<'a, super::W, 4, 5>;
        pub type ASW<'a> = crate::AtomicSetFieldWriter<'a, super::AtomicSetW, 4, 5>;
        pub type ACW<'a> = crate::AtomicClearFieldWriter<'a, super::AtomicClearW, 4, 5>;
        pub type AXW<'a> = crate::AtomicXorFieldWriter<'a, super::AtomicXorW, 4, 5>;
    }
    /// Current state of dreq input
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod dreq {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn finish_clear(&self) -> finish_clear::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn dwell(&self) -> dwell::R {
            crate::FieldReader::new((self.bits() >> 4) & 0x1f)
        }
        #[inline(always)]
        pub fn dreq(&self) -> dreq::R {
            crate::BitReader::new(((self.bits() >> 12) & 1) != 0)
        }
    }
    impl W {
        #[inline(always)]
        pub fn finish_clear(&mut self) -> finish_clear::W<'_> {
            finish_clear::W::new(self)
        }
        #[inline(always)]
        pub fn disable(&mut self) -> disable::W<'_> {
            disable::W::new(self)
        }
        #[inline(always)]
        pub fn dwell(&mut self) -> dwell::W<'_> {
            dwell::W::new(self)
        }
    }
    impl AtomicSetW {
        /// Clear enable/single when DMAC asserts dma_finish
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn finish_clear(&mut self) -> finish_clear::ASW<'_> {
            finish_clear::ASW::new(self)
        }
        /// Write 1 to SET alias to force-disable handshake
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn disable(&mut self) -> disable::ASW<'_> {
            disable::ASW::new(self)
        }
        /// Handshake state machine idle dwell in bus-clock cycles
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn dwell(&mut self) -> dwell::ASW<'_> {
            dwell::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// Clear enable/single when DMAC asserts dma_finish
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn finish_clear(&mut self) -> finish_clear::ACW<'_> {
            finish_clear::ACW::new(self)
        }
        /// Write 1 to SET alias to force-disable handshake
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn disable(&mut self) -> disable::ACW<'_> {
            disable::ACW::new(self)
        }
        /// Handshake state machine idle dwell in bus-clock cycles
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn dwell(&mut self) -> dwell::ACW<'_> {
            dwell::ACW::new(self)
        }
    }
    impl AtomicXorW {
        /// Clear enable/single when DMAC asserts dma_finish
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn finish_clear(&mut self) -> finish_clear::AXW<'_> {
            finish_clear::AXW::new(self)
        }
        /// Write 1 to SET alias to force-disable handshake
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn disable(&mut self) -> disable::AXW<'_> {
            disable::AXW::new(self)
        }
        /// Handshake state machine idle dwell in bus-clock cycles
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn dwell(&mut self) -> dwell::AXW<'_> {
            dwell::AXW::new(self)
        }
    }
}

#[cfg(test)]
mod layout_tests {
    use super::RegisterBlock;
    #[test]
    fn tick0_en_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, tick0_en), 0x0);
    }
    #[test]
    fn tick0_ctrl_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, tick0_ctrl), 0x4);
    }
    #[test]
    fn tick1_en_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, tick1_en), 0x8);
    }
    #[test]
    fn tick1_ctrl_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, tick1_ctrl), 0xc);
    }
    #[test]
    fn register_block_size() {
        assert_eq!(core::mem::size_of::<RegisterBlock>(), 0x10);
    }
}
