// Generated from rp1-regdb/data/peripherals/dma-tick.toml.
// Do not edit manually.
// Regenerate with: cargo run -p rp1-regdb -- generate

use crate::Reg;

/// Two paced DMA request generators driven by the TICKS DMA tick channels. Bus=APB. Atomic aliases: XOR at +0x1000, SET at +0x2000, CLR at +0x3000.
/// Atomic view: Rp1Native alias windows; ownership remains with this logical peripheral.
#[repr(C)]
pub struct RegisterBlock {
    tick0_en: TICK0_EN,
    tick0_ctrl: TICK0_CTRL,
    tick1_en: TICK1_EN,
    tick1_ctrl: TICK1_CTRL,
}

impl RegisterBlock {
    /// DMA handshake enables for tick channel 0. Hardware clears these enables on an abort, or on a DMA finish when `finish_clear` is set.
    /// Hardware access: read Plain; write Plain.
    #[inline(always)]
    pub const fn tick0_en(&self) -> &TICK0_EN {
        &self.tick0_en
    }
    /// Pacing and termination control for tick channel 0, paced by `TICKS.DMA0`.
    /// Hardware access: read Plain; write Plain.
    #[inline(always)]
    pub const fn tick0_ctrl(&self) -> &TICK0_CTRL {
        &self.tick0_ctrl
    }
    /// DMA handshake enables for tick channel 1. Hardware clears these enables on an abort, or on a DMA finish when `finish_clear` is set.
    /// Hardware access: read Plain; write Plain.
    #[inline(always)]
    pub const fn tick1_en(&self) -> &TICK1_EN {
        &self.tick1_en
    }
    /// Pacing and termination control for tick channel 1, paced by `TICKS.DMA1`.
    /// Hardware access: read Plain; write Plain.
    #[inline(always)]
    pub const fn tick1_ctrl(&self) -> &TICK1_CTRL {
        &self.tick1_ctrl
    }
}

/// DMA handshake enables for tick channel 0. Hardware clears these enables on an abort, or on a DMA finish when `finish_clear` is set.
/// Hardware access: read Plain; write Plain.
pub type TICK0_EN = Reg<tick0_en::TICK0_EN_SPEC>;
/// DMA handshake enables for tick channel 0. Hardware clears these enables on an abort, or on a DMA finish when `finish_clear` is set.
/// Hardware access: read Plain; write Plain.
pub mod tick0_en {
    pub struct TICK0_EN_SPEC;
    impl crate::RegisterSpec for TICK0_EN_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for TICK0_EN_SPEC {}
    pub type R = crate::R<TICK0_EN_SPEC>;
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
    pub type AtomicSetW = crate::AtomicSetW<TICK0_EN_SPEC>;
    pub type AtomicClearW = crate::AtomicClearW<TICK0_EN_SPEC>;
    pub type AtomicXorW = crate::AtomicXorW<TICK0_EN_SPEC>;
    /// Paced request enable: 0 no requests, 1 this tick channel may raise a DMA handshake.
    /// Field write behavior: HardwareMayClear.
    pub mod req {
        pub type R = crate::BitReader;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 0>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 0>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 0>;
    }
    /// Only used with `req=1`. 0 raises burst requests only; 1 also asserts `dma_single`.
    /// Field write behavior: HardwareMayClear.
    pub mod single {
        pub type R = crate::BitReader;
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
    impl AtomicSetW {
        /// Paced request enable: 0 no requests, 1 this tick channel may raise a DMA handshake.
        /// Field write behavior: HardwareMayClear.
        #[inline(always)]
        pub fn req(&mut self) -> req::ASW<'_> {
            req::ASW::new(self)
        }
        /// Only used with `req=1`. 0 raises burst requests only; 1 also asserts `dma_single`.
        /// Field write behavior: HardwareMayClear.
        #[inline(always)]
        pub fn single(&mut self) -> single::ASW<'_> {
            single::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// Paced request enable: 0 no requests, 1 this tick channel may raise a DMA handshake.
        /// Field write behavior: HardwareMayClear.
        #[inline(always)]
        pub fn req(&mut self) -> req::ACW<'_> {
            req::ACW::new(self)
        }
        /// Only used with `req=1`. 0 raises burst requests only; 1 also asserts `dma_single`.
        /// Field write behavior: HardwareMayClear.
        #[inline(always)]
        pub fn single(&mut self) -> single::ACW<'_> {
            single::ACW::new(self)
        }
    }
    impl AtomicXorW {
        /// Paced request enable: 0 no requests, 1 this tick channel may raise a DMA handshake.
        /// Field write behavior: HardwareMayClear.
        #[inline(always)]
        pub fn req(&mut self) -> req::AXW<'_> {
            req::AXW::new(self)
        }
        /// Only used with `req=1`. 0 raises burst requests only; 1 also asserts `dma_single`.
        /// Field write behavior: HardwareMayClear.
        #[inline(always)]
        pub fn single(&mut self) -> single::AXW<'_> {
            single::AXW::new(self)
        }
    }
}

/// Pacing and termination control for tick channel 0, paced by `TICKS.DMA0`.
/// Hardware access: read Plain; write Plain.
pub type TICK0_CTRL = Reg<tick0_ctrl::TICK0_CTRL_SPEC>;
/// Pacing and termination control for tick channel 0, paced by `TICKS.DMA0`.
/// Hardware access: read Plain; write Plain.
pub mod tick0_ctrl {
    pub struct TICK0_CTRL_SPEC;
    impl crate::RegisterSpec for TICK0_CTRL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for TICK0_CTRL_SPEC {}
    pub type R = crate::R<TICK0_CTRL_SPEC>;
    impl crate::Modifiable for TICK0_CTRL_SPEC {
        const MODIFY_CLEAR_MASK: u32 = 0x2;
    }
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
    /// On a `dma_finish` pulse from the DMAC: 0 keeps the request enables, 1 clears both `req` and `single`.
    /// Field write behavior: Plain.
    pub mod finish_clear {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 0>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 0>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 0>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 0>;
    }
    /// Writing 1 through the SET alias aborts the handshake; writing 0 there does nothing. An abort can race a DMAC transfer already in progress.
    /// Field write behavior: Command.
    pub mod disable {
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 1>;
    }
    /// Extra bus-clock cycles added to the idle gap of the handshake.
    /// Field write behavior: Plain.
    pub mod dwell {
        pub type R = crate::FieldReader<u32>;
        pub type W<'a> = crate::FieldWriter<'a, super::W, 4, 5>;
        pub type ASW<'a> = crate::AtomicSetFieldWriter<'a, super::AtomicSetW, 4, 5>;
        pub type ACW<'a> = crate::AtomicClearFieldWriter<'a, super::AtomicClearW, 4, 5>;
        pub type AXW<'a> = crate::AtomicXorFieldWriter<'a, super::AtomicXorW, 4, 5>;
    }
    /// Debug view of the incoming request pulse, which lasts one cycle.
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
        pub fn dwell(&mut self) -> dwell::W<'_> {
            dwell::W::new(self)
        }
    }
    impl AtomicSetW {
        /// On a `dma_finish` pulse from the DMAC: 0 keeps the request enables, 1 clears both `req` and `single`.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn finish_clear(&mut self) -> finish_clear::ASW<'_> {
            finish_clear::ASW::new(self)
        }
        /// Writing 1 through the SET alias aborts the handshake; writing 0 there does nothing. An abort can race a DMAC transfer already in progress.
        /// Field write behavior: Command.
        #[inline(always)]
        pub fn disable(&mut self) -> disable::ASW<'_> {
            disable::ASW::new(self)
        }
        /// Extra bus-clock cycles added to the idle gap of the handshake.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn dwell(&mut self) -> dwell::ASW<'_> {
            dwell::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// On a `dma_finish` pulse from the DMAC: 0 keeps the request enables, 1 clears both `req` and `single`.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn finish_clear(&mut self) -> finish_clear::ACW<'_> {
            finish_clear::ACW::new(self)
        }
        /// Extra bus-clock cycles added to the idle gap of the handshake.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn dwell(&mut self) -> dwell::ACW<'_> {
            dwell::ACW::new(self)
        }
    }
    impl AtomicXorW {
        /// On a `dma_finish` pulse from the DMAC: 0 keeps the request enables, 1 clears both `req` and `single`.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn finish_clear(&mut self) -> finish_clear::AXW<'_> {
            finish_clear::AXW::new(self)
        }
        /// Extra bus-clock cycles added to the idle gap of the handshake.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn dwell(&mut self) -> dwell::AXW<'_> {
            dwell::AXW::new(self)
        }
    }
}

/// DMA handshake enables for tick channel 1. Hardware clears these enables on an abort, or on a DMA finish when `finish_clear` is set.
/// Hardware access: read Plain; write Plain.
pub type TICK1_EN = Reg<tick1_en::TICK1_EN_SPEC>;
/// DMA handshake enables for tick channel 1. Hardware clears these enables on an abort, or on a DMA finish when `finish_clear` is set.
/// Hardware access: read Plain; write Plain.
pub mod tick1_en {
    pub struct TICK1_EN_SPEC;
    impl crate::RegisterSpec for TICK1_EN_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for TICK1_EN_SPEC {}
    pub type R = crate::R<TICK1_EN_SPEC>;
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
    pub type AtomicSetW = crate::AtomicSetW<TICK1_EN_SPEC>;
    pub type AtomicClearW = crate::AtomicClearW<TICK1_EN_SPEC>;
    pub type AtomicXorW = crate::AtomicXorW<TICK1_EN_SPEC>;
    /// Paced request enable: 0 no requests, 1 this tick channel may raise a DMA handshake.
    /// Field write behavior: HardwareMayClear.
    pub mod req {
        pub type R = crate::BitReader;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 0>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 0>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 0>;
    }
    /// Only used with `req=1`. 0 raises burst requests only; 1 also asserts `dma_single`.
    /// Field write behavior: HardwareMayClear.
    pub mod single {
        pub type R = crate::BitReader;
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
    impl AtomicSetW {
        /// Paced request enable: 0 no requests, 1 this tick channel may raise a DMA handshake.
        /// Field write behavior: HardwareMayClear.
        #[inline(always)]
        pub fn req(&mut self) -> req::ASW<'_> {
            req::ASW::new(self)
        }
        /// Only used with `req=1`. 0 raises burst requests only; 1 also asserts `dma_single`.
        /// Field write behavior: HardwareMayClear.
        #[inline(always)]
        pub fn single(&mut self) -> single::ASW<'_> {
            single::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// Paced request enable: 0 no requests, 1 this tick channel may raise a DMA handshake.
        /// Field write behavior: HardwareMayClear.
        #[inline(always)]
        pub fn req(&mut self) -> req::ACW<'_> {
            req::ACW::new(self)
        }
        /// Only used with `req=1`. 0 raises burst requests only; 1 also asserts `dma_single`.
        /// Field write behavior: HardwareMayClear.
        #[inline(always)]
        pub fn single(&mut self) -> single::ACW<'_> {
            single::ACW::new(self)
        }
    }
    impl AtomicXorW {
        /// Paced request enable: 0 no requests, 1 this tick channel may raise a DMA handshake.
        /// Field write behavior: HardwareMayClear.
        #[inline(always)]
        pub fn req(&mut self) -> req::AXW<'_> {
            req::AXW::new(self)
        }
        /// Only used with `req=1`. 0 raises burst requests only; 1 also asserts `dma_single`.
        /// Field write behavior: HardwareMayClear.
        #[inline(always)]
        pub fn single(&mut self) -> single::AXW<'_> {
            single::AXW::new(self)
        }
    }
}

/// Pacing and termination control for tick channel 1, paced by `TICKS.DMA1`.
/// Hardware access: read Plain; write Plain.
pub type TICK1_CTRL = Reg<tick1_ctrl::TICK1_CTRL_SPEC>;
/// Pacing and termination control for tick channel 1, paced by `TICKS.DMA1`.
/// Hardware access: read Plain; write Plain.
pub mod tick1_ctrl {
    pub struct TICK1_CTRL_SPEC;
    impl crate::RegisterSpec for TICK1_CTRL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for TICK1_CTRL_SPEC {}
    pub type R = crate::R<TICK1_CTRL_SPEC>;
    impl crate::Modifiable for TICK1_CTRL_SPEC {
        const MODIFY_CLEAR_MASK: u32 = 0x2;
    }
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
    /// On a `dma_finish` pulse from the DMAC: 0 keeps the request enables, 1 clears both `req` and `single`.
    /// Field write behavior: Plain.
    pub mod finish_clear {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 0>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 0>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 0>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 0>;
    }
    /// Writing 1 through the SET alias aborts the handshake; writing 0 there does nothing. An abort can race a DMAC transfer already in progress.
    /// Field write behavior: Command.
    pub mod disable {
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 1>;
    }
    /// Extra bus-clock cycles added to the idle gap of the handshake.
    /// Field write behavior: Plain.
    pub mod dwell {
        pub type R = crate::FieldReader<u32>;
        pub type W<'a> = crate::FieldWriter<'a, super::W, 4, 5>;
        pub type ASW<'a> = crate::AtomicSetFieldWriter<'a, super::AtomicSetW, 4, 5>;
        pub type ACW<'a> = crate::AtomicClearFieldWriter<'a, super::AtomicClearW, 4, 5>;
        pub type AXW<'a> = crate::AtomicXorFieldWriter<'a, super::AtomicXorW, 4, 5>;
    }
    /// Debug view of the incoming request pulse, which lasts one cycle.
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
        pub fn dwell(&mut self) -> dwell::W<'_> {
            dwell::W::new(self)
        }
    }
    impl AtomicSetW {
        /// On a `dma_finish` pulse from the DMAC: 0 keeps the request enables, 1 clears both `req` and `single`.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn finish_clear(&mut self) -> finish_clear::ASW<'_> {
            finish_clear::ASW::new(self)
        }
        /// Writing 1 through the SET alias aborts the handshake; writing 0 there does nothing. An abort can race a DMAC transfer already in progress.
        /// Field write behavior: Command.
        #[inline(always)]
        pub fn disable(&mut self) -> disable::ASW<'_> {
            disable::ASW::new(self)
        }
        /// Extra bus-clock cycles added to the idle gap of the handshake.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn dwell(&mut self) -> dwell::ASW<'_> {
            dwell::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// On a `dma_finish` pulse from the DMAC: 0 keeps the request enables, 1 clears both `req` and `single`.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn finish_clear(&mut self) -> finish_clear::ACW<'_> {
            finish_clear::ACW::new(self)
        }
        /// Extra bus-clock cycles added to the idle gap of the handshake.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn dwell(&mut self) -> dwell::ACW<'_> {
            dwell::ACW::new(self)
        }
    }
    impl AtomicXorW {
        /// On a `dma_finish` pulse from the DMAC: 0 keeps the request enables, 1 clears both `req` and `single`.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn finish_clear(&mut self) -> finish_clear::AXW<'_> {
            finish_clear::AXW::new(self)
        }
        /// Extra bus-clock cycles added to the idle gap of the handshake.
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
