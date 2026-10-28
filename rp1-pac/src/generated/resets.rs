// Generated from rp1-regdb/data/peripherals/resets.toml.
// Do not edit manually.
// Regenerate with: cargo run -p rp1-regdb -- generate

use crate::Reg;

/// Logical reset controller. XOR/SET/CLR are views of this ownership, not independent peripherals.
/// Atomic view: Rp1Native alias windows; ownership remains with this logical peripheral.
#[repr(C)]
pub struct RegisterBlock {
    ctrl0: CTRL0,
    ctrl1: CTRL1,
    ctrl2: CTRL2,
    _reserved0: [u8; 12],
    done0: DONE0,
    done1: DONE1,
    done2: DONE2,
}

impl RegisterBlock {
    /// Reset control bank 0. Unknown bits remain accessible through raw register writes.
    /// Hardware access: read Plain; write Plain.
    #[inline(always)]
    pub const fn ctrl0(&self) -> &CTRL0 {
        &self.ctrl0
    }
    /// Reset control bank 1. Unknown bits remain accessible through raw register writes.
    /// Hardware access: read Plain; write Plain.
    #[inline(always)]
    pub const fn ctrl1(&self) -> &CTRL1 {
        &self.ctrl1
    }
    /// Reset control bank 2. Unknown bits remain accessible through raw register writes.
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn ctrl2(&self) -> &CTRL2 {
        &self.ctrl2
    }
    /// Reset completion bank 0.
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn done0(&self) -> &DONE0 {
        &self.done0
    }
    /// Reset completion bank 1.
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn done1(&self) -> &DONE1 {
        &self.done1
    }
    /// Address follows the bank pattern; unlike DONE0/DONE1, it is not hardware-proven.
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn done2(&self) -> &DONE2 {
        &self.done2
    }
}

/// Reset control bank 0. Unknown bits remain accessible through raw register writes.
/// Hardware access: read Plain; write Plain.
pub type CTRL0 = Reg<ctrl0::CTRL0_SPEC>;
/// Reset control bank 0. Unknown bits remain accessible through raw register writes.
/// Hardware access: read Plain; write Plain.
pub mod ctrl0 {
    pub struct CTRL0_SPEC;
    impl crate::RegisterSpec for CTRL0_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CTRL0_SPEC {}
    pub type R = crate::R<CTRL0_SPEC>;
    impl crate::Modifiable for CTRL0_SPEC {}
    impl crate::TypedAtomicSet for CTRL0_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicSet for CTRL0_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::TypedAtomicClear for CTRL0_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicClear for CTRL0_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::TypedAtomicXor for CTRL0_SPEC {
        const OFFSET: usize = 0x1000;
    }
    impl crate::UnsafeAtomicXor for CTRL0_SPEC {
        const OFFSET: usize = 0x1000;
    }
    pub type W = crate::W<CTRL0_SPEC>;
    pub type AtomicSetW = crate::AtomicSetW<CTRL0_SPEC>;
    pub type AtomicClearW = crate::AtomicClearW<CTRL0_SPEC>;
    pub type AtomicXorW = crate::AtomicXorW<CTRL0_SPEC>;
    /// Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\]
    /// Field write behavior: Plain.
    /// Typed write API is not exposed because write behavior is not hardware/official verified.
    pub mod audio_in {
        pub type R = crate::BitReader;
    }
    /// Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\]
    /// Field write behavior: Plain.
    /// Typed write API is not exposed because write behavior is not hardware/official verified.
    pub mod audio_out {
        pub type R = crate::BitReader;
    }
    /// Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\]
    /// Field write behavior: Plain.
    /// Typed write API is not exposed because write behavior is not hardware/official verified.
    pub mod dma_tick {
        pub type R = crate::BitReader;
    }
    /// Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\]
    /// Field write behavior: Plain.
    /// Typed write API is not exposed because write behavior is not hardware/official verified.
    pub mod eth_ip {
        pub type R = crate::BitReader;
    }
    /// Reset line recovered from supplemental analysis. \[Evidence: Inferred.\] Reset probing changed EXAC reads; exact ownership unresolved.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed because this field is inferred.
    pub mod candidate_bit6 {
        pub type R = crate::BitReader;
    }
    /// Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\]
    /// Field write behavior: Plain.
    /// Typed write API is not exposed because write behavior is not hardware/official verified.
    pub mod i2c0 {
        pub type R = crate::BitReader;
    }
    /// Reset line recovered from supplemental analysis. \[Evidence: HardwareProven.\] RP1-hal I2C1 host proof.
    /// Field write behavior: Plain.
    pub mod i2c1 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 8>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 8>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 8>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 8>;
    }
    /// Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\]
    /// Field write behavior: Plain.
    /// Typed write API is not exposed because write behavior is not hardware/official verified.
    pub mod i2c2 {
        pub type R = crate::BitReader;
    }
    /// Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\]
    /// Field write behavior: Plain.
    /// Typed write API is not exposed because write behavior is not hardware/official verified.
    pub mod i2c3 {
        pub type R = crate::BitReader;
    }
    /// Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\]
    /// Field write behavior: Plain.
    /// Typed write API is not exposed because write behavior is not hardware/official verified.
    pub mod i2c4 {
        pub type R = crate::BitReader;
    }
    /// Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\] Boot-ROM bootstrap I2C.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed because write behavior is not hardware/official verified.
    pub mod i2c5 {
        pub type R = crate::BitReader;
    }
    /// Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\]
    /// Field write behavior: Plain.
    /// Typed write API is not exposed because write behavior is not hardware/official verified.
    pub mod i2c6 {
        pub type R = crate::BitReader;
    }
    /// Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\]
    /// Field write behavior: Plain.
    /// Typed write API is not exposed because write behavior is not hardware/official verified.
    pub mod i2s0 {
        pub type R = crate::BitReader;
    }
    /// Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\]
    /// Field write behavior: Plain.
    /// Typed write API is not exposed because write behavior is not hardware/official verified.
    pub mod i2s1 {
        pub type R = crate::BitReader;
    }
    /// Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\]
    /// Field write behavior: Plain.
    /// Typed write API is not exposed because write behavior is not hardware/official verified.
    pub mod i2s2 {
        pub type R = crate::BitReader;
    }
    /// Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\]
    /// Field write behavior: Plain.
    /// Typed write API is not exposed because write behavior is not hardware/official verified.
    pub mod io_bank0 {
        pub type R = crate::BitReader;
    }
    /// Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\] Contains bootstrap-I2C pins.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed because write behavior is not hardware/official verified.
    pub mod io_bank1 {
        pub type R = crate::BitReader;
    }
    /// Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\]
    /// Field write behavior: Plain.
    /// Typed write API is not exposed because write behavior is not hardware/official verified.
    pub mod io_bank2 {
        pub type R = crate::BitReader;
    }
    /// Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\] CSI/DSI/ISP group.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed because write behavior is not hardware/official verified.
    pub mod mipi0 {
        pub type R = crate::BitReader;
    }
    /// Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\] CSI/DSI/ISP group.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed because write behavior is not hardware/official verified.
    pub mod mipi1 {
        pub type R = crate::BitReader;
    }
    /// Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\]
    /// Field write behavior: Plain.
    /// Typed write API is not exposed because write behavior is not hardware/official verified.
    pub mod pads_bank0 {
        pub type R = crate::BitReader;
    }
    /// Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\] Contains bootstrap-I2C pins.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed because write behavior is not hardware/official verified.
    pub mod pads_bank1 {
        pub type R = crate::BitReader;
    }
    /// Reset line recovered from supplemental analysis. \[Evidence: Inferred.\] Probe labels conflict; topology indicates BANK2.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed because this field is inferred.
    pub mod pads_bank2_candidate {
        pub type R = crate::BitReader;
    }
    /// Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\]
    /// Field write behavior: Plain.
    /// Typed write API is not exposed because write behavior is not hardware/official verified.
    pub mod pads_eth {
        pub type R = crate::BitReader;
    }
    /// Reset line recovered from supplemental analysis. \[Evidence: HardwareProven.\] Endpoint clock ownership transition and DONE poll.
    /// Field write behavior: Plain.
    pub mod pcie_apbs {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 26>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 26>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 26>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 26>;
    }
    /// Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\]
    /// Field write behavior: Plain.
    /// Typed write API is not exposed because write behavior is not hardware/official verified.
    pub mod pio_apbs {
        pub type R = crate::BitReader;
    }
    /// Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\]
    /// Field write behavior: Plain.
    /// Typed write API is not exposed because write behavior is not hardware/official verified.
    pub mod pll_audio {
        pub type R = crate::BitReader;
    }
    /// Reset line recovered from supplemental analysis. \[Evidence: HardwareProven.\] Used by APB PCLK and UART clock bring-up.
    /// Field write behavior: Plain.
    pub mod pll_sys {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 29>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 29>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 29>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 29>;
    }
    /// Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\]
    /// Field write behavior: Plain.
    /// Typed write API is not exposed because write behavior is not hardware/official verified.
    pub mod pll_video {
        pub type R = crate::BitReader;
    }
    /// Processor 1 reset control: 1 asserts proc1 reset; 0 releases proc1 reset.
    /// Field write behavior: Plain.
    pub mod proc1 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 31>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 31>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 31>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 31>;
    }
    impl R {
        #[inline(always)]
        pub fn audio_in(&self) -> audio_in::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
        #[inline(always)]
        pub fn audio_out(&self) -> audio_out::R {
            crate::BitReader::new(((self.bits() >> 2) & 1) != 0)
        }
        #[inline(always)]
        pub fn dma_tick(&self) -> dma_tick::R {
            crate::BitReader::new(((self.bits() >> 4) & 1) != 0)
        }
        #[inline(always)]
        pub fn eth_ip(&self) -> eth_ip::R {
            crate::BitReader::new(((self.bits() >> 5) & 1) != 0)
        }
        #[inline(always)]
        pub fn candidate_bit6(&self) -> candidate_bit6::R {
            crate::BitReader::new(((self.bits() >> 6) & 1) != 0)
        }
        #[inline(always)]
        pub fn i2c0(&self) -> i2c0::R {
            crate::BitReader::new(((self.bits() >> 7) & 1) != 0)
        }
        #[inline(always)]
        pub fn i2c1(&self) -> i2c1::R {
            crate::BitReader::new(((self.bits() >> 8) & 1) != 0)
        }
        #[inline(always)]
        pub fn i2c2(&self) -> i2c2::R {
            crate::BitReader::new(((self.bits() >> 9) & 1) != 0)
        }
        #[inline(always)]
        pub fn i2c3(&self) -> i2c3::R {
            crate::BitReader::new(((self.bits() >> 10) & 1) != 0)
        }
        #[inline(always)]
        pub fn i2c4(&self) -> i2c4::R {
            crate::BitReader::new(((self.bits() >> 11) & 1) != 0)
        }
        #[inline(always)]
        pub fn i2c5(&self) -> i2c5::R {
            crate::BitReader::new(((self.bits() >> 12) & 1) != 0)
        }
        #[inline(always)]
        pub fn i2c6(&self) -> i2c6::R {
            crate::BitReader::new(((self.bits() >> 13) & 1) != 0)
        }
        #[inline(always)]
        pub fn i2s0(&self) -> i2s0::R {
            crate::BitReader::new(((self.bits() >> 14) & 1) != 0)
        }
        #[inline(always)]
        pub fn i2s1(&self) -> i2s1::R {
            crate::BitReader::new(((self.bits() >> 15) & 1) != 0)
        }
        #[inline(always)]
        pub fn i2s2(&self) -> i2s2::R {
            crate::BitReader::new(((self.bits() >> 16) & 1) != 0)
        }
        #[inline(always)]
        pub fn io_bank0(&self) -> io_bank0::R {
            crate::BitReader::new(((self.bits() >> 17) & 1) != 0)
        }
        #[inline(always)]
        pub fn io_bank1(&self) -> io_bank1::R {
            crate::BitReader::new(((self.bits() >> 18) & 1) != 0)
        }
        #[inline(always)]
        pub fn io_bank2(&self) -> io_bank2::R {
            crate::BitReader::new(((self.bits() >> 19) & 1) != 0)
        }
        #[inline(always)]
        pub fn mipi0(&self) -> mipi0::R {
            crate::BitReader::new(((self.bits() >> 20) & 1) != 0)
        }
        #[inline(always)]
        pub fn mipi1(&self) -> mipi1::R {
            crate::BitReader::new(((self.bits() >> 21) & 1) != 0)
        }
        #[inline(always)]
        pub fn pads_bank0(&self) -> pads_bank0::R {
            crate::BitReader::new(((self.bits() >> 22) & 1) != 0)
        }
        #[inline(always)]
        pub fn pads_bank1(&self) -> pads_bank1::R {
            crate::BitReader::new(((self.bits() >> 23) & 1) != 0)
        }
        #[inline(always)]
        pub fn pads_bank2_candidate(&self) -> pads_bank2_candidate::R {
            crate::BitReader::new(((self.bits() >> 24) & 1) != 0)
        }
        #[inline(always)]
        pub fn pads_eth(&self) -> pads_eth::R {
            crate::BitReader::new(((self.bits() >> 25) & 1) != 0)
        }
        #[inline(always)]
        pub fn pcie_apbs(&self) -> pcie_apbs::R {
            crate::BitReader::new(((self.bits() >> 26) & 1) != 0)
        }
        #[inline(always)]
        pub fn pio_apbs(&self) -> pio_apbs::R {
            crate::BitReader::new(((self.bits() >> 27) & 1) != 0)
        }
        #[inline(always)]
        pub fn pll_audio(&self) -> pll_audio::R {
            crate::BitReader::new(((self.bits() >> 28) & 1) != 0)
        }
        #[inline(always)]
        pub fn pll_sys(&self) -> pll_sys::R {
            crate::BitReader::new(((self.bits() >> 29) & 1) != 0)
        }
        #[inline(always)]
        pub fn pll_video(&self) -> pll_video::R {
            crate::BitReader::new(((self.bits() >> 30) & 1) != 0)
        }
        #[inline(always)]
        pub fn proc1(&self) -> proc1::R {
            crate::BitReader::new(((self.bits() >> 31) & 1) != 0)
        }
    }
    impl W {
        #[inline(always)]
        pub fn i2c1(&mut self) -> i2c1::W<'_> {
            i2c1::W::new(self)
        }
        #[inline(always)]
        pub fn pcie_apbs(&mut self) -> pcie_apbs::W<'_> {
            pcie_apbs::W::new(self)
        }
        #[inline(always)]
        pub fn pll_sys(&mut self) -> pll_sys::W<'_> {
            pll_sys::W::new(self)
        }
        #[inline(always)]
        pub fn proc1(&mut self) -> proc1::W<'_> {
            proc1::W::new(self)
        }
    }
    impl AtomicSetW {
        /// Reset line recovered from supplemental analysis. \[Evidence: HardwareProven.\] RP1-hal I2C1 host proof.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn i2c1(&mut self) -> i2c1::ASW<'_> {
            i2c1::ASW::new(self)
        }
        /// Reset line recovered from supplemental analysis. \[Evidence: HardwareProven.\] Endpoint clock ownership transition and DONE poll.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn pcie_apbs(&mut self) -> pcie_apbs::ASW<'_> {
            pcie_apbs::ASW::new(self)
        }
        /// Reset line recovered from supplemental analysis. \[Evidence: HardwareProven.\] Used by APB PCLK and UART clock bring-up.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn pll_sys(&mut self) -> pll_sys::ASW<'_> {
            pll_sys::ASW::new(self)
        }
        /// Processor 1 reset control: 1 asserts proc1 reset; 0 releases proc1 reset.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn proc1(&mut self) -> proc1::ASW<'_> {
            proc1::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// Reset line recovered from supplemental analysis. \[Evidence: HardwareProven.\] RP1-hal I2C1 host proof.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn i2c1(&mut self) -> i2c1::ACW<'_> {
            i2c1::ACW::new(self)
        }
        /// Reset line recovered from supplemental analysis. \[Evidence: HardwareProven.\] Endpoint clock ownership transition and DONE poll.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn pcie_apbs(&mut self) -> pcie_apbs::ACW<'_> {
            pcie_apbs::ACW::new(self)
        }
        /// Reset line recovered from supplemental analysis. \[Evidence: HardwareProven.\] Used by APB PCLK and UART clock bring-up.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn pll_sys(&mut self) -> pll_sys::ACW<'_> {
            pll_sys::ACW::new(self)
        }
        /// Processor 1 reset control: 1 asserts proc1 reset; 0 releases proc1 reset.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn proc1(&mut self) -> proc1::ACW<'_> {
            proc1::ACW::new(self)
        }
    }
    impl AtomicXorW {
        /// Reset line recovered from supplemental analysis. \[Evidence: HardwareProven.\] RP1-hal I2C1 host proof.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn i2c1(&mut self) -> i2c1::AXW<'_> {
            i2c1::AXW::new(self)
        }
        /// Reset line recovered from supplemental analysis. \[Evidence: HardwareProven.\] Endpoint clock ownership transition and DONE poll.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn pcie_apbs(&mut self) -> pcie_apbs::AXW<'_> {
            pcie_apbs::AXW::new(self)
        }
        /// Reset line recovered from supplemental analysis. \[Evidence: HardwareProven.\] Used by APB PCLK and UART clock bring-up.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn pll_sys(&mut self) -> pll_sys::AXW<'_> {
            pll_sys::AXW::new(self)
        }
        /// Processor 1 reset control: 1 asserts proc1 reset; 0 releases proc1 reset.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn proc1(&mut self) -> proc1::AXW<'_> {
            proc1::AXW::new(self)
        }
    }
}

/// Reset control bank 1. Unknown bits remain accessible through raw register writes.
/// Hardware access: read Plain; write Plain.
pub type CTRL1 = Reg<ctrl1::CTRL1_SPEC>;
/// Reset control bank 1. Unknown bits remain accessible through raw register writes.
/// Hardware access: read Plain; write Plain.
pub mod ctrl1 {
    pub struct CTRL1_SPEC;
    impl crate::RegisterSpec for CTRL1_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CTRL1_SPEC {}
    pub type R = crate::R<CTRL1_SPEC>;
    impl crate::Modifiable for CTRL1_SPEC {}
    impl crate::TypedAtomicSet for CTRL1_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicSet for CTRL1_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::TypedAtomicClear for CTRL1_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicClear for CTRL1_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::TypedAtomicXor for CTRL1_SPEC {
        const OFFSET: usize = 0x1000;
    }
    impl crate::UnsafeAtomicXor for CTRL1_SPEC {
        const OFFSET: usize = 0x1000;
    }
    pub type W = crate::W<CTRL1_SPEC>;
    pub type AtomicSetW = crate::AtomicSetW<CTRL1_SPEC>;
    pub type AtomicClearW = crate::AtomicClearW<CTRL1_SPEC>;
    pub type AtomicXorW = crate::AtomicXorW<CTRL1_SPEC>;
    /// Reset line recovered from supplemental analysis. \[Evidence: HardwareProven.\] RP1 GPIO12 PWM proof.
    /// Field write behavior: Plain.
    pub mod pwm0 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 4>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 4>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 4>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 4>;
    }
    /// Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\]
    /// Field write behavior: Plain.
    /// Typed write API is not exposed because write behavior is not hardware/official verified.
    pub mod pwm1 {
        pub type R = crate::BitReader;
    }
    /// Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\]
    /// Field write behavior: Plain.
    /// Typed write API is not exposed because write behavior is not hardware/official verified.
    pub mod rosc0 {
        pub type R = crate::BitReader;
    }
    /// Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\]
    /// Field write behavior: Plain.
    /// Typed write API is not exposed because write behavior is not hardware/official verified.
    pub mod rosc1 {
        pub type R = crate::BitReader;
    }
    /// Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\]
    /// Field write behavior: Plain.
    /// Typed write API is not exposed because write behavior is not hardware/official verified.
    pub mod sdio0_apbs {
        pub type R = crate::BitReader;
    }
    /// Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\]
    /// Field write behavior: Plain.
    /// Typed write API is not exposed because write behavior is not hardware/official verified.
    pub mod sdio1_apbs {
        pub type R = crate::BitReader;
    }
    /// Reset line recovered from supplemental analysis. \[Evidence: HardwareProven.\] RP1 SPI0 host proof.
    /// Field write behavior: Plain.
    pub mod spi0 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 10>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 10>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 10>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 10>;
    }
    /// Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\]
    /// Field write behavior: Plain.
    /// Typed write API is not exposed because write behavior is not hardware/official verified.
    pub mod spi1 {
        pub type R = crate::BitReader;
    }
    /// Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\]
    /// Field write behavior: Plain.
    /// Typed write API is not exposed because write behavior is not hardware/official verified.
    pub mod spi2 {
        pub type R = crate::BitReader;
    }
    /// Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\]
    /// Field write behavior: Plain.
    /// Typed write API is not exposed because write behavior is not hardware/official verified.
    pub mod spi3 {
        pub type R = crate::BitReader;
    }
    /// Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\]
    /// Field write behavior: Plain.
    /// Typed write API is not exposed because write behavior is not hardware/official verified.
    pub mod spi4 {
        pub type R = crate::BitReader;
    }
    /// Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\]
    /// Field write behavior: Plain.
    /// Typed write API is not exposed because write behavior is not hardware/official verified.
    pub mod spi5 {
        pub type R = crate::BitReader;
    }
    /// Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\]
    /// Field write behavior: Plain.
    /// Typed write API is not exposed because write behavior is not hardware/official verified.
    pub mod spi6 {
        pub type R = crate::BitReader;
    }
    /// Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\]
    /// Field write behavior: Plain.
    /// Typed write API is not exposed because write behavior is not hardware/official verified.
    pub mod spi7 {
        pub type R = crate::BitReader;
    }
    /// Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\] Used by boot ROM.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed because write behavior is not hardware/official verified.
    pub mod spi8 {
        pub type R = crate::BitReader;
    }
    /// Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\]
    /// Field write behavior: Plain.
    /// Typed write API is not exposed because write behavior is not hardware/official verified.
    pub mod sys_rio0 {
        pub type R = crate::BitReader;
    }
    /// Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\]
    /// Field write behavior: Plain.
    /// Typed write API is not exposed because write behavior is not hardware/official verified.
    pub mod sys_rio1 {
        pub type R = crate::BitReader;
    }
    /// Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\]
    /// Field write behavior: Plain.
    /// Typed write API is not exposed because write behavior is not hardware/official verified.
    pub mod sys_rio2 {
        pub type R = crate::BitReader;
    }
    /// Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\]
    /// Field write behavior: Plain.
    /// Typed write API is not exposed because write behavior is not hardware/official verified.
    pub mod syscfg {
        pub type R = crate::BitReader;
    }
    /// Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\]
    /// Field write behavior: Plain.
    /// Typed write API is not exposed because write behavior is not hardware/official verified.
    pub mod sysinfo {
        pub type R = crate::BitReader;
    }
    /// Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\] Timer register behavior is separately hardware-proven.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed because write behavior is not hardware/official verified.
    pub mod timer {
        pub type R = crate::BitReader;
    }
    /// Reset line recovered from supplemental analysis. \[Evidence: HardwareProven.\] UART0 polled TX/RX and RX IRQ proof.
    /// Field write behavior: Plain.
    pub mod uart0 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 26>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 26>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 26>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 26>;
    }
    /// Reset line recovered from supplemental analysis. \[Evidence: HardwareProven.\] Reset sentinel/DONE and PL011 loopback IRQ proof.
    /// Field write behavior: Plain.
    pub mod uart1 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 27>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 27>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 27>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 27>;
    }
    /// Reset line recovered from supplemental analysis. \[Evidence: HardwareProven.\] Reset sentinel/DONE and PL011 loopback IRQ proof.
    /// Field write behavior: Plain.
    pub mod uart2 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 28>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 28>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 28>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 28>;
    }
    /// Reset line recovered from supplemental analysis. \[Evidence: HardwareProven.\] Reset sentinel/DONE and PL011 loopback IRQ proof.
    /// Field write behavior: Plain.
    pub mod uart3 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 29>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 29>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 29>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 29>;
    }
    /// Reset line recovered from supplemental analysis. \[Evidence: HardwareProven.\] Reset sentinel/DONE and PL011 loopback IRQ proof.
    /// Field write behavior: Plain.
    pub mod uart4 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 30>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 30>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 30>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 30>;
    }
    /// Reset line recovered from supplemental analysis. \[Evidence: HardwareProven.\] Reset sentinel/DONE and PL011 loopback IRQ proof.
    /// Field write behavior: Plain.
    pub mod uart5 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 31>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 31>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 31>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 31>;
    }
    impl R {
        #[inline(always)]
        pub fn pwm0(&self) -> pwm0::R {
            crate::BitReader::new(((self.bits() >> 4) & 1) != 0)
        }
        #[inline(always)]
        pub fn pwm1(&self) -> pwm1::R {
            crate::BitReader::new(((self.bits() >> 5) & 1) != 0)
        }
        #[inline(always)]
        pub fn rosc0(&self) -> rosc0::R {
            crate::BitReader::new(((self.bits() >> 6) & 1) != 0)
        }
        #[inline(always)]
        pub fn rosc1(&self) -> rosc1::R {
            crate::BitReader::new(((self.bits() >> 7) & 1) != 0)
        }
        #[inline(always)]
        pub fn sdio0_apbs(&self) -> sdio0_apbs::R {
            crate::BitReader::new(((self.bits() >> 8) & 1) != 0)
        }
        #[inline(always)]
        pub fn sdio1_apbs(&self) -> sdio1_apbs::R {
            crate::BitReader::new(((self.bits() >> 9) & 1) != 0)
        }
        #[inline(always)]
        pub fn spi0(&self) -> spi0::R {
            crate::BitReader::new(((self.bits() >> 10) & 1) != 0)
        }
        #[inline(always)]
        pub fn spi1(&self) -> spi1::R {
            crate::BitReader::new(((self.bits() >> 11) & 1) != 0)
        }
        #[inline(always)]
        pub fn spi2(&self) -> spi2::R {
            crate::BitReader::new(((self.bits() >> 12) & 1) != 0)
        }
        #[inline(always)]
        pub fn spi3(&self) -> spi3::R {
            crate::BitReader::new(((self.bits() >> 13) & 1) != 0)
        }
        #[inline(always)]
        pub fn spi4(&self) -> spi4::R {
            crate::BitReader::new(((self.bits() >> 14) & 1) != 0)
        }
        #[inline(always)]
        pub fn spi5(&self) -> spi5::R {
            crate::BitReader::new(((self.bits() >> 15) & 1) != 0)
        }
        #[inline(always)]
        pub fn spi6(&self) -> spi6::R {
            crate::BitReader::new(((self.bits() >> 16) & 1) != 0)
        }
        #[inline(always)]
        pub fn spi7(&self) -> spi7::R {
            crate::BitReader::new(((self.bits() >> 17) & 1) != 0)
        }
        #[inline(always)]
        pub fn spi8(&self) -> spi8::R {
            crate::BitReader::new(((self.bits() >> 18) & 1) != 0)
        }
        #[inline(always)]
        pub fn sys_rio0(&self) -> sys_rio0::R {
            crate::BitReader::new(((self.bits() >> 19) & 1) != 0)
        }
        #[inline(always)]
        pub fn sys_rio1(&self) -> sys_rio1::R {
            crate::BitReader::new(((self.bits() >> 20) & 1) != 0)
        }
        #[inline(always)]
        pub fn sys_rio2(&self) -> sys_rio2::R {
            crate::BitReader::new(((self.bits() >> 21) & 1) != 0)
        }
        #[inline(always)]
        pub fn syscfg(&self) -> syscfg::R {
            crate::BitReader::new(((self.bits() >> 22) & 1) != 0)
        }
        #[inline(always)]
        pub fn sysinfo(&self) -> sysinfo::R {
            crate::BitReader::new(((self.bits() >> 23) & 1) != 0)
        }
        #[inline(always)]
        pub fn timer(&self) -> timer::R {
            crate::BitReader::new(((self.bits() >> 25) & 1) != 0)
        }
        #[inline(always)]
        pub fn uart0(&self) -> uart0::R {
            crate::BitReader::new(((self.bits() >> 26) & 1) != 0)
        }
        #[inline(always)]
        pub fn uart1(&self) -> uart1::R {
            crate::BitReader::new(((self.bits() >> 27) & 1) != 0)
        }
        #[inline(always)]
        pub fn uart2(&self) -> uart2::R {
            crate::BitReader::new(((self.bits() >> 28) & 1) != 0)
        }
        #[inline(always)]
        pub fn uart3(&self) -> uart3::R {
            crate::BitReader::new(((self.bits() >> 29) & 1) != 0)
        }
        #[inline(always)]
        pub fn uart4(&self) -> uart4::R {
            crate::BitReader::new(((self.bits() >> 30) & 1) != 0)
        }
        #[inline(always)]
        pub fn uart5(&self) -> uart5::R {
            crate::BitReader::new(((self.bits() >> 31) & 1) != 0)
        }
    }
    impl W {
        #[inline(always)]
        pub fn pwm0(&mut self) -> pwm0::W<'_> {
            pwm0::W::new(self)
        }
        #[inline(always)]
        pub fn spi0(&mut self) -> spi0::W<'_> {
            spi0::W::new(self)
        }
        #[inline(always)]
        pub fn uart0(&mut self) -> uart0::W<'_> {
            uart0::W::new(self)
        }
        #[inline(always)]
        pub fn uart1(&mut self) -> uart1::W<'_> {
            uart1::W::new(self)
        }
        #[inline(always)]
        pub fn uart2(&mut self) -> uart2::W<'_> {
            uart2::W::new(self)
        }
        #[inline(always)]
        pub fn uart3(&mut self) -> uart3::W<'_> {
            uart3::W::new(self)
        }
        #[inline(always)]
        pub fn uart4(&mut self) -> uart4::W<'_> {
            uart4::W::new(self)
        }
        #[inline(always)]
        pub fn uart5(&mut self) -> uart5::W<'_> {
            uart5::W::new(self)
        }
    }
    impl AtomicSetW {
        /// Reset line recovered from supplemental analysis. \[Evidence: HardwareProven.\] RP1 GPIO12 PWM proof.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn pwm0(&mut self) -> pwm0::ASW<'_> {
            pwm0::ASW::new(self)
        }
        /// Reset line recovered from supplemental analysis. \[Evidence: HardwareProven.\] RP1 SPI0 host proof.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn spi0(&mut self) -> spi0::ASW<'_> {
            spi0::ASW::new(self)
        }
        /// Reset line recovered from supplemental analysis. \[Evidence: HardwareProven.\] UART0 polled TX/RX and RX IRQ proof.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn uart0(&mut self) -> uart0::ASW<'_> {
            uart0::ASW::new(self)
        }
        /// Reset line recovered from supplemental analysis. \[Evidence: HardwareProven.\] Reset sentinel/DONE and PL011 loopback IRQ proof.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn uart1(&mut self) -> uart1::ASW<'_> {
            uart1::ASW::new(self)
        }
        /// Reset line recovered from supplemental analysis. \[Evidence: HardwareProven.\] Reset sentinel/DONE and PL011 loopback IRQ proof.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn uart2(&mut self) -> uart2::ASW<'_> {
            uart2::ASW::new(self)
        }
        /// Reset line recovered from supplemental analysis. \[Evidence: HardwareProven.\] Reset sentinel/DONE and PL011 loopback IRQ proof.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn uart3(&mut self) -> uart3::ASW<'_> {
            uart3::ASW::new(self)
        }
        /// Reset line recovered from supplemental analysis. \[Evidence: HardwareProven.\] Reset sentinel/DONE and PL011 loopback IRQ proof.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn uart4(&mut self) -> uart4::ASW<'_> {
            uart4::ASW::new(self)
        }
        /// Reset line recovered from supplemental analysis. \[Evidence: HardwareProven.\] Reset sentinel/DONE and PL011 loopback IRQ proof.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn uart5(&mut self) -> uart5::ASW<'_> {
            uart5::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// Reset line recovered from supplemental analysis. \[Evidence: HardwareProven.\] RP1 GPIO12 PWM proof.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn pwm0(&mut self) -> pwm0::ACW<'_> {
            pwm0::ACW::new(self)
        }
        /// Reset line recovered from supplemental analysis. \[Evidence: HardwareProven.\] RP1 SPI0 host proof.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn spi0(&mut self) -> spi0::ACW<'_> {
            spi0::ACW::new(self)
        }
        /// Reset line recovered from supplemental analysis. \[Evidence: HardwareProven.\] UART0 polled TX/RX and RX IRQ proof.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn uart0(&mut self) -> uart0::ACW<'_> {
            uart0::ACW::new(self)
        }
        /// Reset line recovered from supplemental analysis. \[Evidence: HardwareProven.\] Reset sentinel/DONE and PL011 loopback IRQ proof.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn uart1(&mut self) -> uart1::ACW<'_> {
            uart1::ACW::new(self)
        }
        /// Reset line recovered from supplemental analysis. \[Evidence: HardwareProven.\] Reset sentinel/DONE and PL011 loopback IRQ proof.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn uart2(&mut self) -> uart2::ACW<'_> {
            uart2::ACW::new(self)
        }
        /// Reset line recovered from supplemental analysis. \[Evidence: HardwareProven.\] Reset sentinel/DONE and PL011 loopback IRQ proof.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn uart3(&mut self) -> uart3::ACW<'_> {
            uart3::ACW::new(self)
        }
        /// Reset line recovered from supplemental analysis. \[Evidence: HardwareProven.\] Reset sentinel/DONE and PL011 loopback IRQ proof.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn uart4(&mut self) -> uart4::ACW<'_> {
            uart4::ACW::new(self)
        }
        /// Reset line recovered from supplemental analysis. \[Evidence: HardwareProven.\] Reset sentinel/DONE and PL011 loopback IRQ proof.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn uart5(&mut self) -> uart5::ACW<'_> {
            uart5::ACW::new(self)
        }
    }
    impl AtomicXorW {
        /// Reset line recovered from supplemental analysis. \[Evidence: HardwareProven.\] RP1 GPIO12 PWM proof.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn pwm0(&mut self) -> pwm0::AXW<'_> {
            pwm0::AXW::new(self)
        }
        /// Reset line recovered from supplemental analysis. \[Evidence: HardwareProven.\] RP1 SPI0 host proof.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn spi0(&mut self) -> spi0::AXW<'_> {
            spi0::AXW::new(self)
        }
        /// Reset line recovered from supplemental analysis. \[Evidence: HardwareProven.\] UART0 polled TX/RX and RX IRQ proof.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn uart0(&mut self) -> uart0::AXW<'_> {
            uart0::AXW::new(self)
        }
        /// Reset line recovered from supplemental analysis. \[Evidence: HardwareProven.\] Reset sentinel/DONE and PL011 loopback IRQ proof.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn uart1(&mut self) -> uart1::AXW<'_> {
            uart1::AXW::new(self)
        }
        /// Reset line recovered from supplemental analysis. \[Evidence: HardwareProven.\] Reset sentinel/DONE and PL011 loopback IRQ proof.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn uart2(&mut self) -> uart2::AXW<'_> {
            uart2::AXW::new(self)
        }
        /// Reset line recovered from supplemental analysis. \[Evidence: HardwareProven.\] Reset sentinel/DONE and PL011 loopback IRQ proof.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn uart3(&mut self) -> uart3::AXW<'_> {
            uart3::AXW::new(self)
        }
        /// Reset line recovered from supplemental analysis. \[Evidence: HardwareProven.\] Reset sentinel/DONE and PL011 loopback IRQ proof.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn uart4(&mut self) -> uart4::AXW<'_> {
            uart4::AXW::new(self)
        }
        /// Reset line recovered from supplemental analysis. \[Evidence: HardwareProven.\] Reset sentinel/DONE and PL011 loopback IRQ proof.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn uart5(&mut self) -> uart5::AXW<'_> {
            uart5::AXW::new(self)
        }
    }
}

/// Reset control bank 2. Unknown bits remain accessible through raw register writes.
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type CTRL2 = Reg<ctrl2::CTRL2_SPEC>;
/// Reset control bank 2. Unknown bits remain accessible through raw register writes.
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod ctrl2 {
    pub struct CTRL2_SPEC;
    impl crate::RegisterSpec for CTRL2_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CTRL2_SPEC {}
    pub type R = crate::R<CTRL2_SPEC>;
    /// Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\]
    /// Field write behavior: Plain.
    /// Typed write API is not exposed because write behavior is not hardware/official verified.
    pub mod usbhost0 {
        pub type R = crate::BitReader;
    }
    /// Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\]
    /// Field write behavior: Plain.
    /// Typed write API is not exposed because write behavior is not hardware/official verified.
    pub mod usbhost1 {
        pub type R = crate::BitReader;
    }
    /// Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\]
    /// Field write behavior: Plain.
    /// Typed write API is not exposed because write behavior is not hardware/official verified.
    pub mod vbusctrl {
        pub type R = crate::BitReader;
    }
    /// Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\]
    /// Field write behavior: Plain.
    /// Typed write API is not exposed because write behavior is not hardware/official verified.
    pub mod video_out_dpi {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn usbhost0(&self) -> usbhost0::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn usbhost1(&self) -> usbhost1::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
        #[inline(always)]
        pub fn vbusctrl(&self) -> vbusctrl::R {
            crate::BitReader::new(((self.bits() >> 2) & 1) != 0)
        }
        #[inline(always)]
        pub fn video_out_dpi(&self) -> video_out_dpi::R {
            crate::BitReader::new(((self.bits() >> 3) & 1) != 0)
        }
    }
}

/// Reset completion bank 0.
/// Hardware access: read Plain; write None.
pub type DONE0 = Reg<done0::DONE0_SPEC>;
/// Reset completion bank 0.
/// Hardware access: read Plain; write None.
pub mod done0 {
    pub struct DONE0_SPEC;
    impl crate::RegisterSpec for DONE0_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for DONE0_SPEC {}
    pub type R = crate::R<DONE0_SPEC>;
    /// Reset completion for AUDIO_IN. Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\]
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod audio_in {
        pub type R = crate::BitReader;
    }
    /// Reset completion for AUDIO_OUT. Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\]
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod audio_out {
        pub type R = crate::BitReader;
    }
    /// Reset completion for DMA_TICK. Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\]
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod dma_tick {
        pub type R = crate::BitReader;
    }
    /// Reset completion for ETH_IP. Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\]
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod eth_ip {
        pub type R = crate::BitReader;
    }
    /// Reset completion for EXAC_CANDIDATE. Reset line recovered from supplemental analysis. \[Evidence: Inferred.\] Reset probing changed EXAC reads; exact ownership unresolved.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod exac_candidate {
        pub type R = crate::BitReader;
    }
    /// Reset completion for I2C0. Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\]
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod i2c0 {
        pub type R = crate::BitReader;
    }
    /// Reset completion for I2C1. Reset line recovered from supplemental analysis. \[Evidence: HardwareProven.\] RP1-hal I2C1 host proof.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod i2c1 {
        pub type R = crate::BitReader;
    }
    /// Reset completion for I2C2. Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\]
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod i2c2 {
        pub type R = crate::BitReader;
    }
    /// Reset completion for I2C3. Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\]
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod i2c3 {
        pub type R = crate::BitReader;
    }
    /// Reset completion for I2C4. Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\]
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod i2c4 {
        pub type R = crate::BitReader;
    }
    /// Reset completion for I2C5. Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\] Boot-ROM bootstrap I2C.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod i2c5 {
        pub type R = crate::BitReader;
    }
    /// Reset completion for I2C6. Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\]
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod i2c6 {
        pub type R = crate::BitReader;
    }
    /// Reset completion for I2S0. Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\]
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod i2s0 {
        pub type R = crate::BitReader;
    }
    /// Reset completion for I2S1. Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\]
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod i2s1 {
        pub type R = crate::BitReader;
    }
    /// Reset completion for I2S2. Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\]
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod i2s2 {
        pub type R = crate::BitReader;
    }
    /// Reset completion for IO_BANK0. Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\]
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod io_bank0 {
        pub type R = crate::BitReader;
    }
    /// Reset completion for IO_BANK1. Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\] Contains bootstrap-I2C pins.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod io_bank1 {
        pub type R = crate::BitReader;
    }
    /// Reset completion for IO_BANK2. Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\]
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod io_bank2 {
        pub type R = crate::BitReader;
    }
    /// Reset completion for MIPI0. Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\] CSI/DSI/ISP group.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod mipi0 {
        pub type R = crate::BitReader;
    }
    /// Reset completion for MIPI1. Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\] CSI/DSI/ISP group.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod mipi1 {
        pub type R = crate::BitReader;
    }
    /// Reset completion for PADS_BANK0. Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\]
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod pads_bank0 {
        pub type R = crate::BitReader;
    }
    /// Reset completion for PADS_BANK1. Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\] Contains bootstrap-I2C pins.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod pads_bank1 {
        pub type R = crate::BitReader;
    }
    /// Reset completion for PADS_BANK2_CANDIDATE. Reset line recovered from supplemental analysis. \[Evidence: Inferred.\] Probe labels conflict; topology indicates BANK2.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod pads_bank2_candidate {
        pub type R = crate::BitReader;
    }
    /// Reset completion for PADS_ETH. Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\]
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod pads_eth {
        pub type R = crate::BitReader;
    }
    /// Reset completion for PCIE_APBS. Reset line recovered from supplemental analysis. \[Evidence: HardwareProven.\] Endpoint clock ownership transition and DONE poll.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod pcie_apbs {
        pub type R = crate::BitReader;
    }
    /// Reset completion for PIO_APBS. Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\]
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod pio_apbs {
        pub type R = crate::BitReader;
    }
    /// Reset completion for PLL_AUDIO. Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\]
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod pll_audio {
        pub type R = crate::BitReader;
    }
    /// Reset completion for PLL_SYS. Reset line recovered from supplemental analysis. \[Evidence: HardwareProven.\] Used by APB PCLK and UART clock bring-up.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod pll_sys {
        pub type R = crate::BitReader;
    }
    /// Reset completion for PLL_VIDEO. Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\]
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod pll_video {
        pub type R = crate::BitReader;
    }
    /// Processor 1 reset completion: 1 means reset release completed; 0 means completion has not been observed.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod proc1 {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn audio_in(&self) -> audio_in::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
        #[inline(always)]
        pub fn audio_out(&self) -> audio_out::R {
            crate::BitReader::new(((self.bits() >> 2) & 1) != 0)
        }
        #[inline(always)]
        pub fn dma_tick(&self) -> dma_tick::R {
            crate::BitReader::new(((self.bits() >> 4) & 1) != 0)
        }
        #[inline(always)]
        pub fn eth_ip(&self) -> eth_ip::R {
            crate::BitReader::new(((self.bits() >> 5) & 1) != 0)
        }
        #[inline(always)]
        pub fn exac_candidate(&self) -> exac_candidate::R {
            crate::BitReader::new(((self.bits() >> 6) & 1) != 0)
        }
        #[inline(always)]
        pub fn i2c0(&self) -> i2c0::R {
            crate::BitReader::new(((self.bits() >> 7) & 1) != 0)
        }
        #[inline(always)]
        pub fn i2c1(&self) -> i2c1::R {
            crate::BitReader::new(((self.bits() >> 8) & 1) != 0)
        }
        #[inline(always)]
        pub fn i2c2(&self) -> i2c2::R {
            crate::BitReader::new(((self.bits() >> 9) & 1) != 0)
        }
        #[inline(always)]
        pub fn i2c3(&self) -> i2c3::R {
            crate::BitReader::new(((self.bits() >> 10) & 1) != 0)
        }
        #[inline(always)]
        pub fn i2c4(&self) -> i2c4::R {
            crate::BitReader::new(((self.bits() >> 11) & 1) != 0)
        }
        #[inline(always)]
        pub fn i2c5(&self) -> i2c5::R {
            crate::BitReader::new(((self.bits() >> 12) & 1) != 0)
        }
        #[inline(always)]
        pub fn i2c6(&self) -> i2c6::R {
            crate::BitReader::new(((self.bits() >> 13) & 1) != 0)
        }
        #[inline(always)]
        pub fn i2s0(&self) -> i2s0::R {
            crate::BitReader::new(((self.bits() >> 14) & 1) != 0)
        }
        #[inline(always)]
        pub fn i2s1(&self) -> i2s1::R {
            crate::BitReader::new(((self.bits() >> 15) & 1) != 0)
        }
        #[inline(always)]
        pub fn i2s2(&self) -> i2s2::R {
            crate::BitReader::new(((self.bits() >> 16) & 1) != 0)
        }
        #[inline(always)]
        pub fn io_bank0(&self) -> io_bank0::R {
            crate::BitReader::new(((self.bits() >> 17) & 1) != 0)
        }
        #[inline(always)]
        pub fn io_bank1(&self) -> io_bank1::R {
            crate::BitReader::new(((self.bits() >> 18) & 1) != 0)
        }
        #[inline(always)]
        pub fn io_bank2(&self) -> io_bank2::R {
            crate::BitReader::new(((self.bits() >> 19) & 1) != 0)
        }
        #[inline(always)]
        pub fn mipi0(&self) -> mipi0::R {
            crate::BitReader::new(((self.bits() >> 20) & 1) != 0)
        }
        #[inline(always)]
        pub fn mipi1(&self) -> mipi1::R {
            crate::BitReader::new(((self.bits() >> 21) & 1) != 0)
        }
        #[inline(always)]
        pub fn pads_bank0(&self) -> pads_bank0::R {
            crate::BitReader::new(((self.bits() >> 22) & 1) != 0)
        }
        #[inline(always)]
        pub fn pads_bank1(&self) -> pads_bank1::R {
            crate::BitReader::new(((self.bits() >> 23) & 1) != 0)
        }
        #[inline(always)]
        pub fn pads_bank2_candidate(&self) -> pads_bank2_candidate::R {
            crate::BitReader::new(((self.bits() >> 24) & 1) != 0)
        }
        #[inline(always)]
        pub fn pads_eth(&self) -> pads_eth::R {
            crate::BitReader::new(((self.bits() >> 25) & 1) != 0)
        }
        #[inline(always)]
        pub fn pcie_apbs(&self) -> pcie_apbs::R {
            crate::BitReader::new(((self.bits() >> 26) & 1) != 0)
        }
        #[inline(always)]
        pub fn pio_apbs(&self) -> pio_apbs::R {
            crate::BitReader::new(((self.bits() >> 27) & 1) != 0)
        }
        #[inline(always)]
        pub fn pll_audio(&self) -> pll_audio::R {
            crate::BitReader::new(((self.bits() >> 28) & 1) != 0)
        }
        #[inline(always)]
        pub fn pll_sys(&self) -> pll_sys::R {
            crate::BitReader::new(((self.bits() >> 29) & 1) != 0)
        }
        #[inline(always)]
        pub fn pll_video(&self) -> pll_video::R {
            crate::BitReader::new(((self.bits() >> 30) & 1) != 0)
        }
        #[inline(always)]
        pub fn proc1(&self) -> proc1::R {
            crate::BitReader::new(((self.bits() >> 31) & 1) != 0)
        }
    }
}

/// Reset completion bank 1.
/// Hardware access: read Plain; write None.
pub type DONE1 = Reg<done1::DONE1_SPEC>;
/// Reset completion bank 1.
/// Hardware access: read Plain; write None.
pub mod done1 {
    pub struct DONE1_SPEC;
    impl crate::RegisterSpec for DONE1_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for DONE1_SPEC {}
    pub type R = crate::R<DONE1_SPEC>;
    /// Reset completion for PWM0. Reset line recovered from supplemental analysis. \[Evidence: HardwareProven.\] RP1 GPIO12 PWM proof.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod pwm0 {
        pub type R = crate::BitReader;
    }
    /// Reset completion for PWM1. Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\]
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod pwm1 {
        pub type R = crate::BitReader;
    }
    /// Reset completion for ROSC0. Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\]
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod rosc0 {
        pub type R = crate::BitReader;
    }
    /// Reset completion for ROSC1. Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\]
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod rosc1 {
        pub type R = crate::BitReader;
    }
    /// Reset completion for SDIO0_APBS. Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\]
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod sdio0_apbs {
        pub type R = crate::BitReader;
    }
    /// Reset completion for SDIO1_APBS. Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\]
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod sdio1_apbs {
        pub type R = crate::BitReader;
    }
    /// Reset completion for SPI0. Reset line recovered from supplemental analysis. \[Evidence: HardwareProven.\] RP1 SPI0 host proof.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod spi0 {
        pub type R = crate::BitReader;
    }
    /// Reset completion for SPI1. Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\]
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod spi1 {
        pub type R = crate::BitReader;
    }
    /// Reset completion for SPI2. Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\]
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod spi2 {
        pub type R = crate::BitReader;
    }
    /// Reset completion for SPI3. Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\]
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod spi3 {
        pub type R = crate::BitReader;
    }
    /// Reset completion for SPI4. Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\]
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod spi4 {
        pub type R = crate::BitReader;
    }
    /// Reset completion for SPI5. Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\]
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod spi5 {
        pub type R = crate::BitReader;
    }
    /// Reset completion for SPI6. Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\]
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod spi6 {
        pub type R = crate::BitReader;
    }
    /// Reset completion for SPI7. Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\]
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod spi7 {
        pub type R = crate::BitReader;
    }
    /// Reset completion for SPI8. Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\] Used by boot ROM.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod spi8 {
        pub type R = crate::BitReader;
    }
    /// Reset completion for SYS_RIO0. Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\]
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod sys_rio0 {
        pub type R = crate::BitReader;
    }
    /// Reset completion for SYS_RIO1. Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\]
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod sys_rio1 {
        pub type R = crate::BitReader;
    }
    /// Reset completion for SYS_RIO2. Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\]
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod sys_rio2 {
        pub type R = crate::BitReader;
    }
    /// Reset completion for SYSCFG. Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\]
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod syscfg {
        pub type R = crate::BitReader;
    }
    /// Reset completion for SYSINFO. Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\]
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod sysinfo {
        pub type R = crate::BitReader;
    }
    /// Reset completion for TIMER. Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\] Timer register behavior is separately hardware-proven.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod timer {
        pub type R = crate::BitReader;
    }
    /// Reset completion for UART0. Reset line recovered from supplemental analysis. \[Evidence: HardwareProven.\] UART0 polled TX/RX and RX IRQ proof.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod uart0 {
        pub type R = crate::BitReader;
    }
    /// Reset completion for UART1. Reset line recovered from supplemental analysis. \[Evidence: HardwareProven.\] Reset sentinel/DONE and PL011 loopback IRQ proof.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod uart1 {
        pub type R = crate::BitReader;
    }
    /// Reset completion for UART2. Reset line recovered from supplemental analysis. \[Evidence: HardwareProven.\] Reset sentinel/DONE and PL011 loopback IRQ proof.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod uart2 {
        pub type R = crate::BitReader;
    }
    /// Reset completion for UART3. Reset line recovered from supplemental analysis. \[Evidence: HardwareProven.\] Reset sentinel/DONE and PL011 loopback IRQ proof.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod uart3 {
        pub type R = crate::BitReader;
    }
    /// Reset completion for UART4. Reset line recovered from supplemental analysis. \[Evidence: HardwareProven.\] Reset sentinel/DONE and PL011 loopback IRQ proof.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod uart4 {
        pub type R = crate::BitReader;
    }
    /// Reset completion for UART5. Reset line recovered from supplemental analysis. \[Evidence: HardwareProven.\] Reset sentinel/DONE and PL011 loopback IRQ proof.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod uart5 {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn pwm0(&self) -> pwm0::R {
            crate::BitReader::new(((self.bits() >> 4) & 1) != 0)
        }
        #[inline(always)]
        pub fn pwm1(&self) -> pwm1::R {
            crate::BitReader::new(((self.bits() >> 5) & 1) != 0)
        }
        #[inline(always)]
        pub fn rosc0(&self) -> rosc0::R {
            crate::BitReader::new(((self.bits() >> 6) & 1) != 0)
        }
        #[inline(always)]
        pub fn rosc1(&self) -> rosc1::R {
            crate::BitReader::new(((self.bits() >> 7) & 1) != 0)
        }
        #[inline(always)]
        pub fn sdio0_apbs(&self) -> sdio0_apbs::R {
            crate::BitReader::new(((self.bits() >> 8) & 1) != 0)
        }
        #[inline(always)]
        pub fn sdio1_apbs(&self) -> sdio1_apbs::R {
            crate::BitReader::new(((self.bits() >> 9) & 1) != 0)
        }
        #[inline(always)]
        pub fn spi0(&self) -> spi0::R {
            crate::BitReader::new(((self.bits() >> 10) & 1) != 0)
        }
        #[inline(always)]
        pub fn spi1(&self) -> spi1::R {
            crate::BitReader::new(((self.bits() >> 11) & 1) != 0)
        }
        #[inline(always)]
        pub fn spi2(&self) -> spi2::R {
            crate::BitReader::new(((self.bits() >> 12) & 1) != 0)
        }
        #[inline(always)]
        pub fn spi3(&self) -> spi3::R {
            crate::BitReader::new(((self.bits() >> 13) & 1) != 0)
        }
        #[inline(always)]
        pub fn spi4(&self) -> spi4::R {
            crate::BitReader::new(((self.bits() >> 14) & 1) != 0)
        }
        #[inline(always)]
        pub fn spi5(&self) -> spi5::R {
            crate::BitReader::new(((self.bits() >> 15) & 1) != 0)
        }
        #[inline(always)]
        pub fn spi6(&self) -> spi6::R {
            crate::BitReader::new(((self.bits() >> 16) & 1) != 0)
        }
        #[inline(always)]
        pub fn spi7(&self) -> spi7::R {
            crate::BitReader::new(((self.bits() >> 17) & 1) != 0)
        }
        #[inline(always)]
        pub fn spi8(&self) -> spi8::R {
            crate::BitReader::new(((self.bits() >> 18) & 1) != 0)
        }
        #[inline(always)]
        pub fn sys_rio0(&self) -> sys_rio0::R {
            crate::BitReader::new(((self.bits() >> 19) & 1) != 0)
        }
        #[inline(always)]
        pub fn sys_rio1(&self) -> sys_rio1::R {
            crate::BitReader::new(((self.bits() >> 20) & 1) != 0)
        }
        #[inline(always)]
        pub fn sys_rio2(&self) -> sys_rio2::R {
            crate::BitReader::new(((self.bits() >> 21) & 1) != 0)
        }
        #[inline(always)]
        pub fn syscfg(&self) -> syscfg::R {
            crate::BitReader::new(((self.bits() >> 22) & 1) != 0)
        }
        #[inline(always)]
        pub fn sysinfo(&self) -> sysinfo::R {
            crate::BitReader::new(((self.bits() >> 23) & 1) != 0)
        }
        #[inline(always)]
        pub fn timer(&self) -> timer::R {
            crate::BitReader::new(((self.bits() >> 25) & 1) != 0)
        }
        #[inline(always)]
        pub fn uart0(&self) -> uart0::R {
            crate::BitReader::new(((self.bits() >> 26) & 1) != 0)
        }
        #[inline(always)]
        pub fn uart1(&self) -> uart1::R {
            crate::BitReader::new(((self.bits() >> 27) & 1) != 0)
        }
        #[inline(always)]
        pub fn uart2(&self) -> uart2::R {
            crate::BitReader::new(((self.bits() >> 28) & 1) != 0)
        }
        #[inline(always)]
        pub fn uart3(&self) -> uart3::R {
            crate::BitReader::new(((self.bits() >> 29) & 1) != 0)
        }
        #[inline(always)]
        pub fn uart4(&self) -> uart4::R {
            crate::BitReader::new(((self.bits() >> 30) & 1) != 0)
        }
        #[inline(always)]
        pub fn uart5(&self) -> uart5::R {
            crate::BitReader::new(((self.bits() >> 31) & 1) != 0)
        }
    }
}

/// Address follows the bank pattern; unlike DONE0/DONE1, it is not hardware-proven.
/// Hardware access: read Plain; write None.
pub type DONE2 = Reg<done2::DONE2_SPEC>;
/// Address follows the bank pattern; unlike DONE0/DONE1, it is not hardware-proven.
/// Hardware access: read Plain; write None.
pub mod done2 {
    pub struct DONE2_SPEC;
    impl crate::RegisterSpec for DONE2_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for DONE2_SPEC {}
    pub type R = crate::R<DONE2_SPEC>;
    /// Reset completion for USBHOST0. Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\]
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod usbhost0 {
        pub type R = crate::BitReader;
    }
    /// Reset completion for USBHOST1. Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\]
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod usbhost1 {
        pub type R = crate::BitReader;
    }
    /// Reset completion for VBUSCTRL. Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\]
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod vbusctrl {
        pub type R = crate::BitReader;
    }
    /// Reset completion for VIDEO_OUT_DPI. Reset line recovered from supplemental analysis. \[Evidence: ReverseEngineered.\]
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod video_out_dpi {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn usbhost0(&self) -> usbhost0::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn usbhost1(&self) -> usbhost1::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
        #[inline(always)]
        pub fn vbusctrl(&self) -> vbusctrl::R {
            crate::BitReader::new(((self.bits() >> 2) & 1) != 0)
        }
        #[inline(always)]
        pub fn video_out_dpi(&self) -> video_out_dpi::R {
            crate::BitReader::new(((self.bits() >> 3) & 1) != 0)
        }
    }
}

#[cfg(test)]
mod layout_tests {
    use super::RegisterBlock;
    #[test]
    fn ctrl0_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, ctrl0), 0x0);
    }
    #[test]
    fn ctrl1_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, ctrl1), 0x4);
    }
    #[test]
    fn ctrl2_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, ctrl2), 0x8);
    }
    #[test]
    fn done0_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, done0), 0x18);
    }
    #[test]
    fn done1_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, done1), 0x1c);
    }
    #[test]
    fn done2_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, done2), 0x20);
    }
    #[test]
    fn register_block_size() {
        assert_eq!(core::mem::size_of::<RegisterBlock>(), 0x24);
    }
}
