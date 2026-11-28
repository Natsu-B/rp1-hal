// Generated from rp1-regdb/data/peripherals/eth-cfg.toml.
// Do not edit manually.
// Regenerate with: cargo run -p rp1-regdb -- generate

use crate::Reg;

/// RP1 glue logic around the GEM Ethernet MAC: timestamp unit, RGMII clocking and interrupt routing. Bus=APB. Atomic aliases: normal at base, XOR at +0x1000, SET at +0x2000, CLR at +0x3000.
/// Atomic view: Rp1Native alias windows; ownership remains with this logical peripheral.
#[repr(C)]
pub struct RegisterBlock {
    control: CONTROL,
    status: STATUS,
    tsu_timer_cnt0: TSU_TIMER_CNT0,
    tsu_timer_cnt1: TSU_TIMER_CNT1,
    tsu_timer_cnt2: TSU_TIMER_CNT2,
    clkgen: CLKGEN,
    clk2fc: CLK2FC,
    intr: INTR,
    inte: INTE,
    intf: INTF,
    ints: INTS,
}

impl RegisterBlock {
    /// MAC glue controls for the timestamp unit, bus error reporting and memory power.
    /// Hardware access: read Plain; write Plain.
    #[inline(always)]
    pub const fn control(&self) -> &CONTROL {
        &self.control
    }
    /// RGMII link status and AXI burst-length error flags.
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn status(&self) -> &STATUS {
        &self.status
    }
    /// Bits 31:0 of the 94-bit timestamp counter. How the three words are latched for a consistent read is not known.
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn tsu_timer_cnt0(&self) -> &TSU_TIMER_CNT0 {
        &self.tsu_timer_cnt0
    }
    /// Bits 63:32 of the 94-bit timestamp counter.
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn tsu_timer_cnt1(&self) -> &TSU_TIMER_CNT1 {
        &self.tsu_timer_cnt1
    }
    /// Bits 93:64 of the 94-bit timestamp counter.
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn tsu_timer_cnt2(&self) -> &TSU_TIMER_CNT2 {
        &self.tsu_timer_cnt2
    }
    /// RGMII clock generator controls. Safe to change while running.
    /// Hardware access: read Plain; write Plain.
    #[inline(always)]
    pub const fn clkgen(&self) -> &CLKGEN {
        &self.clkgen
    }
    /// Chooses which RGMII clock is sent to the frequency counter.
    /// Hardware access: read Plain; write Plain.
    #[inline(always)]
    pub const fn clk2fc(&self) -> &CLK2FC {
        &self.clk2fc
    }
    /// Raw interrupt sources, before the enable mask.
    /// Hardware access: read Plain; write Plain.
    /// Generated writes require unsafe raw access.
    #[inline(always)]
    pub const fn intr(&self) -> &INTR {
        &self.intr
    }
    /// Interrupt enable, one bit per source: 1 lets the source reach the IRQ output, 0 masks it.
    /// Hardware access: read Plain; write Plain.
    #[inline(always)]
    pub const fn inte(&self) -> &INTE {
        &self.inte
    }
    /// Interrupt force, one bit per source: 1 asserts the source by software, 0 removes the force.
    /// Hardware access: read Plain; write Plain.
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

/// MAC glue controls for the timestamp unit, bus error reporting and memory power.
/// Hardware access: read Plain; write Plain.
pub type CONTROL = Reg<control::CONTROL_SPEC>;
/// MAC glue controls for the timestamp unit, bus error reporting and memory power.
/// Hardware access: read Plain; write Plain.
pub mod control {
    pub struct CONTROL_SPEC;
    impl crate::RegisterSpec for CONTROL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CONTROL_SPEC {}
    pub type R = crate::R<CONTROL_SPEC>;
    impl crate::Modifiable for CONTROL_SPEC {}
    impl crate::UnsafeWritable for CONTROL_SPEC {}
    impl crate::ResetValue for CONTROL_SPEC {
        const RESET_VALUE: u32 = 0x0;
    }
    impl crate::TypedAtomicSet for CONTROL_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicSet for CONTROL_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::TypedAtomicClear for CONTROL_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicClear for CONTROL_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::TypedAtomicXor for CONTROL_SPEC {
        const OFFSET: usize = 0x1000;
    }
    impl crate::UnsafeAtomicXor for CONTROL_SPEC {
        const OFFSET: usize = 0x1000;
    }
    pub type W = crate::W<CONTROL_SPEC>;
    pub type AtomicSetW = crate::AtomicSetW<CONTROL_SPEC>;
    pub type AtomicClearW = crate::AtomicClearW<CONTROL_SPEC>;
    pub type AtomicXorW = crate::AtomicXorW<CONTROL_SPEC>;
    /// Drives the GEM timestamp unit master/slave select input. Which level means master is not known.
    /// Field write behavior: Plain.
    pub mod tsu_ms {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 0>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 0>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 0>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 0>;
    }
    /// 2-bit value driven to the GEM timestamp increment control input. Code meanings are not known.
    /// Field write behavior: Plain.
    pub mod tsu_inc_ctrl {
        pub type R = crate::FieldReader<u32>;
        pub type W<'a> = crate::FieldWriter<'a, super::W, 1, 2>;
        pub type ASW<'a> = crate::AtomicSetFieldWriter<'a, super::AtomicSetW, 1, 2>;
        pub type ACW<'a> = crate::AtomicClearFieldWriter<'a, super::AtomicClearW, 1, 2>;
        pub type AXW<'a> = crate::AtomicXorFieldWriter<'a, super::AtomicXorW, 1, 2>;
    }
    /// 1 passes GEM bus errors on to the fabric; 0 suppresses them. With this set, accesses to unused MAC address space can raise bus faults.
    /// Field write behavior: Plain.
    pub mod buserr_en {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 3>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 3>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 3>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 3>;
    }
    /// 1 requests memory power-down; 0 keeps memory powered. Content retention and wake-up sequence are not known.
    /// Field write behavior: Plain.
    pub mod mem_pd {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 4>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 4>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 4>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 4>;
    }
    impl R {
        #[inline(always)]
        pub fn tsu_ms(&self) -> tsu_ms::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn tsu_inc_ctrl(&self) -> tsu_inc_ctrl::R {
            crate::FieldReader::new((self.bits() >> 1) & 0x3)
        }
        #[inline(always)]
        pub fn buserr_en(&self) -> buserr_en::R {
            crate::BitReader::new(((self.bits() >> 3) & 1) != 0)
        }
        #[inline(always)]
        pub fn mem_pd(&self) -> mem_pd::R {
            crate::BitReader::new(((self.bits() >> 4) & 1) != 0)
        }
    }
    impl W {
        #[inline(always)]
        pub fn tsu_ms(&mut self) -> tsu_ms::W<'_> {
            tsu_ms::W::new(self)
        }
        #[inline(always)]
        pub fn tsu_inc_ctrl(&mut self) -> tsu_inc_ctrl::W<'_> {
            tsu_inc_ctrl::W::new(self)
        }
        #[inline(always)]
        pub fn buserr_en(&mut self) -> buserr_en::W<'_> {
            buserr_en::W::new(self)
        }
        #[inline(always)]
        pub fn mem_pd(&mut self) -> mem_pd::W<'_> {
            mem_pd::W::new(self)
        }
    }
    impl AtomicSetW {
        /// Drives the GEM timestamp unit master/slave select input. Which level means master is not known.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn tsu_ms(&mut self) -> tsu_ms::ASW<'_> {
            tsu_ms::ASW::new(self)
        }
        /// 2-bit value driven to the GEM timestamp increment control input. Code meanings are not known.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn tsu_inc_ctrl(&mut self) -> tsu_inc_ctrl::ASW<'_> {
            tsu_inc_ctrl::ASW::new(self)
        }
        /// 1 passes GEM bus errors on to the fabric; 0 suppresses them. With this set, accesses to unused MAC address space can raise bus faults.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn buserr_en(&mut self) -> buserr_en::ASW<'_> {
            buserr_en::ASW::new(self)
        }
        /// 1 requests memory power-down; 0 keeps memory powered. Content retention and wake-up sequence are not known.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn mem_pd(&mut self) -> mem_pd::ASW<'_> {
            mem_pd::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// Drives the GEM timestamp unit master/slave select input. Which level means master is not known.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn tsu_ms(&mut self) -> tsu_ms::ACW<'_> {
            tsu_ms::ACW::new(self)
        }
        /// 2-bit value driven to the GEM timestamp increment control input. Code meanings are not known.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn tsu_inc_ctrl(&mut self) -> tsu_inc_ctrl::ACW<'_> {
            tsu_inc_ctrl::ACW::new(self)
        }
        /// 1 passes GEM bus errors on to the fabric; 0 suppresses them. With this set, accesses to unused MAC address space can raise bus faults.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn buserr_en(&mut self) -> buserr_en::ACW<'_> {
            buserr_en::ACW::new(self)
        }
        /// 1 requests memory power-down; 0 keeps memory powered. Content retention and wake-up sequence are not known.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn mem_pd(&mut self) -> mem_pd::ACW<'_> {
            mem_pd::ACW::new(self)
        }
    }
    impl AtomicXorW {
        /// Drives the GEM timestamp unit master/slave select input. Which level means master is not known.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn tsu_ms(&mut self) -> tsu_ms::AXW<'_> {
            tsu_ms::AXW::new(self)
        }
        /// 2-bit value driven to the GEM timestamp increment control input. Code meanings are not known.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn tsu_inc_ctrl(&mut self) -> tsu_inc_ctrl::AXW<'_> {
            tsu_inc_ctrl::AXW::new(self)
        }
        /// 1 passes GEM bus errors on to the fabric; 0 suppresses them. With this set, accesses to unused MAC address space can raise bus faults.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn buserr_en(&mut self) -> buserr_en::AXW<'_> {
            buserr_en::AXW::new(self)
        }
        /// 1 requests memory power-down; 0 keeps memory powered. Content retention and wake-up sequence are not known.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn mem_pd(&mut self) -> mem_pd::AXW<'_> {
            mem_pd::AXW::new(self)
        }
    }
}

/// RGMII link status and AXI burst-length error flags.
/// Hardware access: read Plain; write None.
pub type STATUS = Reg<status::STATUS_SPEC>;
/// RGMII link status and AXI burst-length error flags.
/// Hardware access: read Plain; write None.
pub mod status {
    pub struct STATUS_SPEC;
    impl crate::RegisterSpec for STATUS_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for STATUS_SPEC {}
    pub type R = crate::R<STATUS_SPEC>;
    /// RGMII link up/down indication. Polarity is not known.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod rgmii_link_status {
        pub type R = crate::BitReader;
    }
    /// RGMII speed: 0 10 Mb/s, 1 100 Mb/s, 2 1 Gb/s. Code 3 is not known.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod rgmii_speed {
        pub type R = crate::FieldReader<u32>;
    }
    /// RGMII duplex: half or full. Which value means full duplex is not known.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod rgmii_duplex {
        pub type R = crate::BitReader;
    }
    /// 1 when a read burst longer than 16 AXI beats was seen. Recovery needs a block reset and possibly a full system reset.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod arlen_illegal {
        pub type R = crate::BitReader;
    }
    /// 1 when a write burst longer than 16 AXI beats was seen. Recovery needs a block reset and possibly a full system reset.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod awlen_illegal {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn rgmii_link_status(&self) -> rgmii_link_status::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn rgmii_speed(&self) -> rgmii_speed::R {
            crate::FieldReader::new((self.bits() >> 1) & 0x3)
        }
        #[inline(always)]
        pub fn rgmii_duplex(&self) -> rgmii_duplex::R {
            crate::BitReader::new(((self.bits() >> 3) & 1) != 0)
        }
        #[inline(always)]
        pub fn arlen_illegal(&self) -> arlen_illegal::R {
            crate::BitReader::new(((self.bits() >> 4) & 1) != 0)
        }
        #[inline(always)]
        pub fn awlen_illegal(&self) -> awlen_illegal::R {
            crate::BitReader::new(((self.bits() >> 5) & 1) != 0)
        }
    }
}

/// Bits 31:0 of the 94-bit timestamp counter. How the three words are latched for a consistent read is not known.
/// Hardware access: read Plain; write None.
pub type TSU_TIMER_CNT0 = Reg<tsu_timer_cnt0::TSU_TIMER_CNT0_SPEC>;
/// Bits 31:0 of the 94-bit timestamp counter. How the three words are latched for a consistent read is not known.
/// Hardware access: read Plain; write None.
pub mod tsu_timer_cnt0 {
    pub struct TSU_TIMER_CNT0_SPEC;
    impl crate::RegisterSpec for TSU_TIMER_CNT0_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for TSU_TIMER_CNT0_SPEC {}
    pub type R = crate::R<TSU_TIMER_CNT0_SPEC>;
    impl crate::ResetValue for TSU_TIMER_CNT0_SPEC {
        const RESET_VALUE: u32 = 0x0;
    }
    /// Timestamp counter bits 31:0. Counter units are not known.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod cnt0 {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn cnt0(&self) -> cnt0::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffffffff)
        }
    }
}

/// Bits 63:32 of the 94-bit timestamp counter.
/// Hardware access: read Plain; write None.
pub type TSU_TIMER_CNT1 = Reg<tsu_timer_cnt1::TSU_TIMER_CNT1_SPEC>;
/// Bits 63:32 of the 94-bit timestamp counter.
/// Hardware access: read Plain; write None.
pub mod tsu_timer_cnt1 {
    pub struct TSU_TIMER_CNT1_SPEC;
    impl crate::RegisterSpec for TSU_TIMER_CNT1_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for TSU_TIMER_CNT1_SPEC {}
    pub type R = crate::R<TSU_TIMER_CNT1_SPEC>;
    impl crate::ResetValue for TSU_TIMER_CNT1_SPEC {
        const RESET_VALUE: u32 = 0x0;
    }
    /// Timestamp counter bits 63:32.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod cnt1 {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn cnt1(&self) -> cnt1::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffffffff)
        }
    }
}

/// Bits 93:64 of the 94-bit timestamp counter.
/// Hardware access: read Plain; write None.
pub type TSU_TIMER_CNT2 = Reg<tsu_timer_cnt2::TSU_TIMER_CNT2_SPEC>;
/// Bits 93:64 of the 94-bit timestamp counter.
/// Hardware access: read Plain; write None.
pub mod tsu_timer_cnt2 {
    pub struct TSU_TIMER_CNT2_SPEC;
    impl crate::RegisterSpec for TSU_TIMER_CNT2_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for TSU_TIMER_CNT2_SPEC {}
    pub type R = crate::R<TSU_TIMER_CNT2_SPEC>;
    impl crate::ResetValue for TSU_TIMER_CNT2_SPEC {
        const RESET_VALUE: u32 = 0x0;
    }
    /// Timestamp counter bits 93:64, in bits 29:0 of this register.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod cnt2 {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn cnt2(&self) -> cnt2::R {
            crate::FieldReader::new((self.bits() >> 0) & 0x3fffffff)
        }
    }
}

/// RGMII clock generator controls. Safe to change while running.
/// Hardware access: read Plain; write Plain.
pub type CLKGEN = Reg<clkgen::CLKGEN_SPEC>;
/// RGMII clock generator controls. Safe to change while running.
/// Hardware access: read Plain; write Plain.
pub mod clkgen {
    pub struct CLKGEN_SPEC;
    impl crate::RegisterSpec for CLKGEN_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLKGEN_SPEC {}
    pub type R = crate::R<CLKGEN_SPEC>;
    impl crate::Modifiable for CLKGEN_SPEC {}
    impl crate::UnsafeWritable for CLKGEN_SPEC {}
    impl crate::TypedAtomicSet for CLKGEN_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicSet for CLKGEN_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::TypedAtomicClear for CLKGEN_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicClear for CLKGEN_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::TypedAtomicXor for CLKGEN_SPEC {
        const OFFSET: usize = 0x1000;
    }
    impl crate::UnsafeAtomicXor for CLKGEN_SPEC {
        const OFFSET: usize = 0x1000;
    }
    pub type W = crate::W<CLKGEN_SPEC>;
    pub type AtomicSetW = crate::AtomicSetW<CLKGEN_SPEC>;
    pub type AtomicClearW = crate::AtomicClearW<CLKGEN_SPEC>;
    pub type AtomicXorW = crate::AtomicXorW<CLKGEN_SPEC>;
    /// Speed code used when speed_override_en=1: 0 10 Mb/s, 1 100 Mb/s, 2 1 Gb/s. Code 3 is not known.
    /// Field write behavior: Plain.
    pub mod speed_override {
        pub type R = crate::FieldReader<u32>;
        pub type W<'a> = crate::FieldWriter<'a, super::W, 0, 2>;
        pub type ASW<'a> = crate::AtomicSetFieldWriter<'a, super::AtomicSetW, 0, 2>;
        pub type ACW<'a> = crate::AtomicClearFieldWriter<'a, super::AtomicClearW, 0, 2>;
        pub type AXW<'a> = crate::AtomicXorFieldWriter<'a, super::AtomicXorW, 0, 2>;
    }
    /// 0 uses the speed from the MAC; 1 uses speed_override.
    /// Field write behavior: Plain.
    pub mod speed_override_en {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 3>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 3>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 3>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 3>;
    }
    /// Speed code reported by GEM, regardless of the override setting. The exact encoding is not known.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod speed_from_mac {
        pub type R = crate::FieldReader<u32>;
    }
    /// 1 stops the clock immediately without synchronization. Use enable for a glitch-free stop.
    /// Field write behavior: Plain.
    pub mod kill {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 6>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 6>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 6>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 6>;
    }
    /// 1 starts the clock cleanly; 0 stops it cleanly.
    /// Field write behavior: Plain.
    pub mod enable {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 7>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 7>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 7>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 7>;
    }
    /// 1 corrects the duty cycle to 50% for odd divisors; 0 leaves it uncorrected.
    /// Field write behavior: Plain.
    pub mod dc50 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 8>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 8>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 8>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 8>;
    }
    /// 1 adds a delay to the TX clock; 0 bypasses it. The delay amount is not known.
    /// Field write behavior: Plain.
    pub mod txclkdelen {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 9>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 9>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 9>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 9>;
    }
    impl R {
        #[inline(always)]
        pub fn speed_override(&self) -> speed_override::R {
            crate::FieldReader::new((self.bits() >> 0) & 0x3)
        }
        #[inline(always)]
        pub fn speed_override_en(&self) -> speed_override_en::R {
            crate::BitReader::new(((self.bits() >> 3) & 1) != 0)
        }
        #[inline(always)]
        pub fn speed_from_mac(&self) -> speed_from_mac::R {
            crate::FieldReader::new((self.bits() >> 4) & 0x3)
        }
        #[inline(always)]
        pub fn kill(&self) -> kill::R {
            crate::BitReader::new(((self.bits() >> 6) & 1) != 0)
        }
        #[inline(always)]
        pub fn enable(&self) -> enable::R {
            crate::BitReader::new(((self.bits() >> 7) & 1) != 0)
        }
        #[inline(always)]
        pub fn dc50(&self) -> dc50::R {
            crate::BitReader::new(((self.bits() >> 8) & 1) != 0)
        }
        #[inline(always)]
        pub fn txclkdelen(&self) -> txclkdelen::R {
            crate::BitReader::new(((self.bits() >> 9) & 1) != 0)
        }
    }
    impl W {
        #[inline(always)]
        pub fn speed_override(&mut self) -> speed_override::W<'_> {
            speed_override::W::new(self)
        }
        #[inline(always)]
        pub fn speed_override_en(&mut self) -> speed_override_en::W<'_> {
            speed_override_en::W::new(self)
        }
        #[inline(always)]
        pub fn kill(&mut self) -> kill::W<'_> {
            kill::W::new(self)
        }
        #[inline(always)]
        pub fn enable(&mut self) -> enable::W<'_> {
            enable::W::new(self)
        }
        #[inline(always)]
        pub fn dc50(&mut self) -> dc50::W<'_> {
            dc50::W::new(self)
        }
        #[inline(always)]
        pub fn txclkdelen(&mut self) -> txclkdelen::W<'_> {
            txclkdelen::W::new(self)
        }
    }
    impl AtomicSetW {
        /// Speed code used when speed_override_en=1: 0 10 Mb/s, 1 100 Mb/s, 2 1 Gb/s. Code 3 is not known.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn speed_override(&mut self) -> speed_override::ASW<'_> {
            speed_override::ASW::new(self)
        }
        /// 0 uses the speed from the MAC; 1 uses speed_override.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn speed_override_en(&mut self) -> speed_override_en::ASW<'_> {
            speed_override_en::ASW::new(self)
        }
        /// 1 stops the clock immediately without synchronization. Use enable for a glitch-free stop.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn kill(&mut self) -> kill::ASW<'_> {
            kill::ASW::new(self)
        }
        /// 1 starts the clock cleanly; 0 stops it cleanly.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn enable(&mut self) -> enable::ASW<'_> {
            enable::ASW::new(self)
        }
        /// 1 corrects the duty cycle to 50% for odd divisors; 0 leaves it uncorrected.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn dc50(&mut self) -> dc50::ASW<'_> {
            dc50::ASW::new(self)
        }
        /// 1 adds a delay to the TX clock; 0 bypasses it. The delay amount is not known.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn txclkdelen(&mut self) -> txclkdelen::ASW<'_> {
            txclkdelen::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// Speed code used when speed_override_en=1: 0 10 Mb/s, 1 100 Mb/s, 2 1 Gb/s. Code 3 is not known.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn speed_override(&mut self) -> speed_override::ACW<'_> {
            speed_override::ACW::new(self)
        }
        /// 0 uses the speed from the MAC; 1 uses speed_override.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn speed_override_en(&mut self) -> speed_override_en::ACW<'_> {
            speed_override_en::ACW::new(self)
        }
        /// 1 stops the clock immediately without synchronization. Use enable for a glitch-free stop.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn kill(&mut self) -> kill::ACW<'_> {
            kill::ACW::new(self)
        }
        /// 1 starts the clock cleanly; 0 stops it cleanly.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn enable(&mut self) -> enable::ACW<'_> {
            enable::ACW::new(self)
        }
        /// 1 corrects the duty cycle to 50% for odd divisors; 0 leaves it uncorrected.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn dc50(&mut self) -> dc50::ACW<'_> {
            dc50::ACW::new(self)
        }
        /// 1 adds a delay to the TX clock; 0 bypasses it. The delay amount is not known.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn txclkdelen(&mut self) -> txclkdelen::ACW<'_> {
            txclkdelen::ACW::new(self)
        }
    }
    impl AtomicXorW {
        /// Speed code used when speed_override_en=1: 0 10 Mb/s, 1 100 Mb/s, 2 1 Gb/s. Code 3 is not known.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn speed_override(&mut self) -> speed_override::AXW<'_> {
            speed_override::AXW::new(self)
        }
        /// 0 uses the speed from the MAC; 1 uses speed_override.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn speed_override_en(&mut self) -> speed_override_en::AXW<'_> {
            speed_override_en::AXW::new(self)
        }
        /// 1 stops the clock immediately without synchronization. Use enable for a glitch-free stop.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn kill(&mut self) -> kill::AXW<'_> {
            kill::AXW::new(self)
        }
        /// 1 starts the clock cleanly; 0 stops it cleanly.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn enable(&mut self) -> enable::AXW<'_> {
            enable::AXW::new(self)
        }
        /// 1 corrects the duty cycle to 50% for odd divisors; 0 leaves it uncorrected.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn dc50(&mut self) -> dc50::AXW<'_> {
            dc50::AXW::new(self)
        }
        /// 1 adds a delay to the TX clock; 0 bypasses it. The delay amount is not known.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn txclkdelen(&mut self) -> txclkdelen::AXW<'_> {
            txclkdelen::AXW::new(self)
        }
    }
}

/// Chooses which RGMII clock is sent to the frequency counter.
/// Hardware access: read Plain; write Plain.
pub type CLK2FC = Reg<clk2fc::CLK2FC_SPEC>;
/// Chooses which RGMII clock is sent to the frequency counter.
/// Hardware access: read Plain; write Plain.
pub mod clk2fc {
    pub struct CLK2FC_SPEC;
    impl crate::RegisterSpec for CLK2FC_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK2FC_SPEC {}
    pub type R = crate::R<CLK2FC_SPEC>;
    impl crate::Modifiable for CLK2FC_SPEC {}
    impl crate::UnsafeWritable for CLK2FC_SPEC {}
    impl crate::ResetValue for CLK2FC_SPEC {
        const RESET_VALUE: u32 = 0x0;
    }
    impl crate::TypedAtomicSet for CLK2FC_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicSet for CLK2FC_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::TypedAtomicClear for CLK2FC_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicClear for CLK2FC_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::TypedAtomicXor for CLK2FC_SPEC {
        const OFFSET: usize = 0x1000;
    }
    impl crate::UnsafeAtomicXor for CLK2FC_SPEC {
        const OFFSET: usize = 0x1000;
    }
    pub type W = crate::W<CLK2FC_SPEC>;
    pub type AtomicSetW = crate::AtomicSetW<CLK2FC_SPEC>;
    pub type AtomicClearW = crate::AtomicClearW<CLK2FC_SPEC>;
    pub type AtomicXorW = crate::AtomicXorW<CLK2FC_SPEC>;
    /// 0 none, 1 RGMII TX clock, 2 RGMII RX clock. Code 3 is not known.
    /// Field write behavior: Plain.
    pub mod sel {
        pub type R = crate::FieldReader<u32>;
        pub type W<'a> = crate::FieldWriter<'a, super::W, 0, 2>;
        pub type ASW<'a> = crate::AtomicSetFieldWriter<'a, super::AtomicSetW, 0, 2>;
        pub type ACW<'a> = crate::AtomicClearFieldWriter<'a, super::AtomicClearW, 0, 2>;
        pub type AXW<'a> = crate::AtomicXorFieldWriter<'a, super::AtomicXorW, 0, 2>;
        pub const NONE: u32 = 0x0;
        pub const RGMII_TX_CLK: u32 = 0x1;
        pub const RGMII_RX_CLK: u32 = 0x2;
    }
    impl R {
        #[inline(always)]
        pub fn sel(&self) -> sel::R {
            crate::FieldReader::new((self.bits() >> 0) & 0x3)
        }
    }
    impl W {
        #[inline(always)]
        pub fn sel(&mut self) -> sel::W<'_> {
            sel::W::new(self)
        }
    }
    impl AtomicSetW {
        /// 0 none, 1 RGMII TX clock, 2 RGMII RX clock. Code 3 is not known.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn sel(&mut self) -> sel::ASW<'_> {
            sel::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// 0 none, 1 RGMII TX clock, 2 RGMII RX clock. Code 3 is not known.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn sel(&mut self) -> sel::ACW<'_> {
            sel::ACW::new(self)
        }
    }
    impl AtomicXorW {
        /// 0 none, 1 RGMII TX clock, 2 RGMII RX clock. Code 3 is not known.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn sel(&mut self) -> sel::AXW<'_> {
            sel::AXW::new(self)
        }
    }
}

/// Raw interrupt sources, before the enable mask.
/// Hardware access: read Plain; write Plain.
/// Generated writes require unsafe raw access.
pub type INTR = Reg<intr::INTR_SPEC>;
/// Raw interrupt sources, before the enable mask.
/// Hardware access: read Plain; write Plain.
/// Generated writes require unsafe raw access.
pub mod intr {
    pub struct INTR_SPEC;
    impl crate::RegisterSpec for INTR_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for INTR_SPEC {}
    pub type R = crate::R<INTR_SPEC>;
    impl crate::UnsafeWritable for INTR_SPEC {}
    impl crate::ResetValue for INTR_SPEC {
        const RESET_VALUE: u32 = 0x0;
    }
    impl crate::Reg<INTR_SPEC> {
        /// The effect of this alias write on this register's write-one-to-clear flags is unverified.
        ///
        /// # Safety
        /// The caller must prove every selected bit is valid for this register and operation.
        #[inline(always)]
        pub unsafe fn atomic_set_bits_unchecked(&self, mask: u32) {
            unsafe { self.write_at_offset(0x2000, mask) }
        }
    }
    impl crate::Reg<INTR_SPEC> {
        /// The effect of this alias write on this register's write-one-to-clear flags is unverified.
        ///
        /// # Safety
        /// The caller must prove every selected bit is valid for this register and operation.
        #[inline(always)]
        pub unsafe fn atomic_clear_bits_unchecked(&self, mask: u32) {
            unsafe { self.write_at_offset(0x3000, mask) }
        }
    }
    impl crate::Reg<INTR_SPEC> {
        /// The effect of this alias write on this register's write-one-to-clear flags is unverified.
        ///
        /// # Safety
        /// The caller must prove every selected bit is valid for this register and operation.
        #[inline(always)]
        pub unsafe fn atomic_xor_bits_unchecked(&self, mask: u32) {
            unsafe { self.write_at_offset(0x1000, mask) }
        }
    }
    /// 1 while the GEM aggregate interrupt source is active. Read-only level; it cannot be cleared here.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod ethernet {
        pub type R = crate::BitReader;
    }
    /// 1 when the Wake-on-LAN source has fired. Write 1 to clear; writing 0 leaves it pending.
    /// Field write behavior: WriteOneToClear.
    /// Typed write API is not exposed by register policy.
    pub mod wol {
        pub type R = crate::BitReader;
    }
    /// 1 when the PTP Pdelay_Resp transmitted source has fired. Write 1 to clear; writing 0 leaves it pending.
    /// Field write behavior: WriteOneToClear.
    /// Typed write API is not exposed by register policy.
    pub mod ieee1588_pdelay_resp_tx {
        pub type R = crate::BitReader;
    }
    /// 1 when the PTP Pdelay_Req (TX-named) source has fired. Write 1 to clear; writing 0 leaves it pending. The source table describes this TX-named bit as the receive path; the actual direction is not known.
    /// Field write behavior: WriteOneToClear.
    /// Typed write API is not exposed by register policy.
    pub mod ieee1588_pdelay_req_tx {
        pub type R = crate::BitReader;
    }
    /// 1 when the PTP Delay_Req transmitted source has fired. Write 1 to clear; writing 0 leaves it pending.
    /// Field write behavior: WriteOneToClear.
    /// Typed write API is not exposed by register policy.
    pub mod ieee1588_delay_req_tx {
        pub type R = crate::BitReader;
    }
    /// 1 when the PTP Sync transmitted source has fired. Write 1 to clear; writing 0 leaves it pending.
    /// Field write behavior: WriteOneToClear.
    /// Typed write API is not exposed by register policy.
    pub mod ieee1588_sync_frame_tx {
        pub type R = crate::BitReader;
    }
    /// 1 when the start of frame transmitted source has fired. Write 1 to clear; writing 0 leaves it pending. The source drops at the end of the frame.
    /// Field write behavior: WriteOneToClear.
    /// Typed write API is not exposed by register policy.
    pub mod ieee1588_sof_tx {
        pub type R = crate::BitReader;
    }
    /// 1 when the PTP Pdelay_Resp received source has fired. Write 1 to clear; writing 0 leaves it pending.
    /// Field write behavior: WriteOneToClear.
    /// Typed write API is not exposed by register policy.
    pub mod ieee1588_pdelay_resp_rx {
        pub type R = crate::BitReader;
    }
    /// 1 when the PTP Pdelay_Req received source has fired. Write 1 to clear; writing 0 leaves it pending.
    /// Field write behavior: WriteOneToClear.
    /// Typed write API is not exposed by register policy.
    pub mod ieee1588_pdelay_req_rx {
        pub type R = crate::BitReader;
    }
    /// 1 when the PTP Delay_Req received source has fired. Write 1 to clear; writing 0 leaves it pending.
    /// Field write behavior: WriteOneToClear.
    /// Typed write API is not exposed by register policy.
    pub mod ieee1588_delay_req_rx {
        pub type R = crate::BitReader;
    }
    /// 1 when the PTP Sync received source has fired. Write 1 to clear; writing 0 leaves it pending.
    /// Field write behavior: WriteOneToClear.
    /// Typed write API is not exposed by register policy.
    pub mod ieee1588_sync_frame_rx {
        pub type R = crate::BitReader;
    }
    /// 1 when the start of frame received source has fired. Write 1 to clear; writing 0 leaves it pending.
    /// Field write behavior: WriteOneToClear.
    /// Typed write API is not exposed by register policy.
    pub mod ieee1588_sof_rx {
        pub type R = crate::BitReader;
    }
    /// 1 when the timestamp comparator match (upper 70 counter bits) source has fired. Write 1 to clear; writing 0 leaves it pending.
    /// Field write behavior: WriteOneToClear.
    /// Typed write API is not exposed by register policy.
    pub mod ieee1588_tsu_timer_cmp_val {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn ethernet(&self) -> ethernet::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn wol(&self) -> wol::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
        #[inline(always)]
        pub fn ieee1588_pdelay_resp_tx(&self) -> ieee1588_pdelay_resp_tx::R {
            crate::BitReader::new(((self.bits() >> 2) & 1) != 0)
        }
        #[inline(always)]
        pub fn ieee1588_pdelay_req_tx(&self) -> ieee1588_pdelay_req_tx::R {
            crate::BitReader::new(((self.bits() >> 3) & 1) != 0)
        }
        #[inline(always)]
        pub fn ieee1588_delay_req_tx(&self) -> ieee1588_delay_req_tx::R {
            crate::BitReader::new(((self.bits() >> 4) & 1) != 0)
        }
        #[inline(always)]
        pub fn ieee1588_sync_frame_tx(&self) -> ieee1588_sync_frame_tx::R {
            crate::BitReader::new(((self.bits() >> 5) & 1) != 0)
        }
        #[inline(always)]
        pub fn ieee1588_sof_tx(&self) -> ieee1588_sof_tx::R {
            crate::BitReader::new(((self.bits() >> 6) & 1) != 0)
        }
        #[inline(always)]
        pub fn ieee1588_pdelay_resp_rx(&self) -> ieee1588_pdelay_resp_rx::R {
            crate::BitReader::new(((self.bits() >> 7) & 1) != 0)
        }
        #[inline(always)]
        pub fn ieee1588_pdelay_req_rx(&self) -> ieee1588_pdelay_req_rx::R {
            crate::BitReader::new(((self.bits() >> 8) & 1) != 0)
        }
        #[inline(always)]
        pub fn ieee1588_delay_req_rx(&self) -> ieee1588_delay_req_rx::R {
            crate::BitReader::new(((self.bits() >> 9) & 1) != 0)
        }
        #[inline(always)]
        pub fn ieee1588_sync_frame_rx(&self) -> ieee1588_sync_frame_rx::R {
            crate::BitReader::new(((self.bits() >> 10) & 1) != 0)
        }
        #[inline(always)]
        pub fn ieee1588_sof_rx(&self) -> ieee1588_sof_rx::R {
            crate::BitReader::new(((self.bits() >> 11) & 1) != 0)
        }
        #[inline(always)]
        pub fn ieee1588_tsu_timer_cmp_val(&self) -> ieee1588_tsu_timer_cmp_val::R {
            crate::BitReader::new(((self.bits() >> 12) & 1) != 0)
        }
    }
}

/// Interrupt enable, one bit per source: 1 lets the source reach the IRQ output, 0 masks it.
/// Hardware access: read Plain; write Plain.
pub type INTE = Reg<inte::INTE_SPEC>;
/// Interrupt enable, one bit per source: 1 lets the source reach the IRQ output, 0 masks it.
/// Hardware access: read Plain; write Plain.
pub mod inte {
    pub struct INTE_SPEC;
    impl crate::RegisterSpec for INTE_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for INTE_SPEC {}
    pub type R = crate::R<INTE_SPEC>;
    impl crate::Modifiable for INTE_SPEC {}
    impl crate::UnsafeWritable for INTE_SPEC {}
    impl crate::ResetValue for INTE_SPEC {
        const RESET_VALUE: u32 = 0x0;
    }
    impl crate::TypedAtomicSet for INTE_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicSet for INTE_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::TypedAtomicClear for INTE_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicClear for INTE_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::TypedAtomicXor for INTE_SPEC {
        const OFFSET: usize = 0x1000;
    }
    impl crate::UnsafeAtomicXor for INTE_SPEC {
        const OFFSET: usize = 0x1000;
    }
    pub type W = crate::W<INTE_SPEC>;
    pub type AtomicSetW = crate::AtomicSetW<INTE_SPEC>;
    pub type AtomicClearW = crate::AtomicClearW<INTE_SPEC>;
    pub type AtomicXorW = crate::AtomicXorW<INTE_SPEC>;
    /// Enable for the GEM aggregate interrupt source. 1 passes it to the IRQ output; 0 masks it.
    /// Field write behavior: Plain.
    pub mod ethernet {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 0>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 0>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 0>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 0>;
    }
    /// Enable for the Wake-on-LAN source. 1 passes it to the IRQ output; 0 masks it.
    /// Field write behavior: Plain.
    pub mod wol {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 1>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 1>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 1>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 1>;
    }
    /// Enable for the PTP Pdelay_Resp transmitted source. 1 passes it to the IRQ output; 0 masks it.
    /// Field write behavior: Plain.
    pub mod ieee1588_pdelay_resp_tx {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 2>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 2>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 2>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 2>;
    }
    /// Enable for the PTP Pdelay_Req (TX-named) source. 1 passes it to the IRQ output; 0 masks it.
    /// Field write behavior: Plain.
    pub mod ieee1588_pdelay_req_tx {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 3>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 3>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 3>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 3>;
    }
    /// Enable for the PTP Delay_Req transmitted source. 1 passes it to the IRQ output; 0 masks it.
    /// Field write behavior: Plain.
    pub mod ieee1588_delay_req_tx {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 4>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 4>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 4>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 4>;
    }
    /// Enable for the PTP Sync transmitted source. 1 passes it to the IRQ output; 0 masks it.
    /// Field write behavior: Plain.
    pub mod ieee1588_sync_frame_tx {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 5>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 5>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 5>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 5>;
    }
    /// Enable for the start of frame transmitted source. 1 passes it to the IRQ output; 0 masks it.
    /// Field write behavior: Plain.
    pub mod ieee1588_sof_tx {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 6>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 6>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 6>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 6>;
    }
    /// Enable for the PTP Pdelay_Resp received source. 1 passes it to the IRQ output; 0 masks it.
    /// Field write behavior: Plain.
    pub mod ieee1588_pdelay_resp_rx {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 7>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 7>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 7>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 7>;
    }
    /// Enable for the PTP Pdelay_Req received source. 1 passes it to the IRQ output; 0 masks it.
    /// Field write behavior: Plain.
    pub mod ieee1588_pdelay_req_rx {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 8>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 8>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 8>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 8>;
    }
    /// Enable for the PTP Delay_Req received source. 1 passes it to the IRQ output; 0 masks it.
    /// Field write behavior: Plain.
    pub mod ieee1588_delay_req_rx {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 9>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 9>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 9>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 9>;
    }
    /// Enable for the PTP Sync received source. 1 passes it to the IRQ output; 0 masks it.
    /// Field write behavior: Plain.
    pub mod ieee1588_sync_frame_rx {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 10>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 10>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 10>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 10>;
    }
    /// Enable for the start of frame received source. 1 passes it to the IRQ output; 0 masks it.
    /// Field write behavior: Plain.
    pub mod ieee1588_sof_rx {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 11>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 11>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 11>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 11>;
    }
    /// Enable for the timestamp comparator match (upper 70 counter bits) source. 1 passes it to the IRQ output; 0 masks it.
    /// Field write behavior: Plain.
    pub mod ieee1588_tsu_timer_cmp_val {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 12>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 12>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 12>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 12>;
    }
    impl R {
        #[inline(always)]
        pub fn ethernet(&self) -> ethernet::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn wol(&self) -> wol::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
        #[inline(always)]
        pub fn ieee1588_pdelay_resp_tx(&self) -> ieee1588_pdelay_resp_tx::R {
            crate::BitReader::new(((self.bits() >> 2) & 1) != 0)
        }
        #[inline(always)]
        pub fn ieee1588_pdelay_req_tx(&self) -> ieee1588_pdelay_req_tx::R {
            crate::BitReader::new(((self.bits() >> 3) & 1) != 0)
        }
        #[inline(always)]
        pub fn ieee1588_delay_req_tx(&self) -> ieee1588_delay_req_tx::R {
            crate::BitReader::new(((self.bits() >> 4) & 1) != 0)
        }
        #[inline(always)]
        pub fn ieee1588_sync_frame_tx(&self) -> ieee1588_sync_frame_tx::R {
            crate::BitReader::new(((self.bits() >> 5) & 1) != 0)
        }
        #[inline(always)]
        pub fn ieee1588_sof_tx(&self) -> ieee1588_sof_tx::R {
            crate::BitReader::new(((self.bits() >> 6) & 1) != 0)
        }
        #[inline(always)]
        pub fn ieee1588_pdelay_resp_rx(&self) -> ieee1588_pdelay_resp_rx::R {
            crate::BitReader::new(((self.bits() >> 7) & 1) != 0)
        }
        #[inline(always)]
        pub fn ieee1588_pdelay_req_rx(&self) -> ieee1588_pdelay_req_rx::R {
            crate::BitReader::new(((self.bits() >> 8) & 1) != 0)
        }
        #[inline(always)]
        pub fn ieee1588_delay_req_rx(&self) -> ieee1588_delay_req_rx::R {
            crate::BitReader::new(((self.bits() >> 9) & 1) != 0)
        }
        #[inline(always)]
        pub fn ieee1588_sync_frame_rx(&self) -> ieee1588_sync_frame_rx::R {
            crate::BitReader::new(((self.bits() >> 10) & 1) != 0)
        }
        #[inline(always)]
        pub fn ieee1588_sof_rx(&self) -> ieee1588_sof_rx::R {
            crate::BitReader::new(((self.bits() >> 11) & 1) != 0)
        }
        #[inline(always)]
        pub fn ieee1588_tsu_timer_cmp_val(&self) -> ieee1588_tsu_timer_cmp_val::R {
            crate::BitReader::new(((self.bits() >> 12) & 1) != 0)
        }
    }
    impl W {
        #[inline(always)]
        pub fn ethernet(&mut self) -> ethernet::W<'_> {
            ethernet::W::new(self)
        }
        #[inline(always)]
        pub fn wol(&mut self) -> wol::W<'_> {
            wol::W::new(self)
        }
        #[inline(always)]
        pub fn ieee1588_pdelay_resp_tx(&mut self) -> ieee1588_pdelay_resp_tx::W<'_> {
            ieee1588_pdelay_resp_tx::W::new(self)
        }
        #[inline(always)]
        pub fn ieee1588_pdelay_req_tx(&mut self) -> ieee1588_pdelay_req_tx::W<'_> {
            ieee1588_pdelay_req_tx::W::new(self)
        }
        #[inline(always)]
        pub fn ieee1588_delay_req_tx(&mut self) -> ieee1588_delay_req_tx::W<'_> {
            ieee1588_delay_req_tx::W::new(self)
        }
        #[inline(always)]
        pub fn ieee1588_sync_frame_tx(&mut self) -> ieee1588_sync_frame_tx::W<'_> {
            ieee1588_sync_frame_tx::W::new(self)
        }
        #[inline(always)]
        pub fn ieee1588_sof_tx(&mut self) -> ieee1588_sof_tx::W<'_> {
            ieee1588_sof_tx::W::new(self)
        }
        #[inline(always)]
        pub fn ieee1588_pdelay_resp_rx(&mut self) -> ieee1588_pdelay_resp_rx::W<'_> {
            ieee1588_pdelay_resp_rx::W::new(self)
        }
        #[inline(always)]
        pub fn ieee1588_pdelay_req_rx(&mut self) -> ieee1588_pdelay_req_rx::W<'_> {
            ieee1588_pdelay_req_rx::W::new(self)
        }
        #[inline(always)]
        pub fn ieee1588_delay_req_rx(&mut self) -> ieee1588_delay_req_rx::W<'_> {
            ieee1588_delay_req_rx::W::new(self)
        }
        #[inline(always)]
        pub fn ieee1588_sync_frame_rx(&mut self) -> ieee1588_sync_frame_rx::W<'_> {
            ieee1588_sync_frame_rx::W::new(self)
        }
        #[inline(always)]
        pub fn ieee1588_sof_rx(&mut self) -> ieee1588_sof_rx::W<'_> {
            ieee1588_sof_rx::W::new(self)
        }
        #[inline(always)]
        pub fn ieee1588_tsu_timer_cmp_val(&mut self) -> ieee1588_tsu_timer_cmp_val::W<'_> {
            ieee1588_tsu_timer_cmp_val::W::new(self)
        }
    }
    impl AtomicSetW {
        /// Enable for the GEM aggregate interrupt source. 1 passes it to the IRQ output; 0 masks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn ethernet(&mut self) -> ethernet::ASW<'_> {
            ethernet::ASW::new(self)
        }
        /// Enable for the Wake-on-LAN source. 1 passes it to the IRQ output; 0 masks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn wol(&mut self) -> wol::ASW<'_> {
            wol::ASW::new(self)
        }
        /// Enable for the PTP Pdelay_Resp transmitted source. 1 passes it to the IRQ output; 0 masks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn ieee1588_pdelay_resp_tx(&mut self) -> ieee1588_pdelay_resp_tx::ASW<'_> {
            ieee1588_pdelay_resp_tx::ASW::new(self)
        }
        /// Enable for the PTP Pdelay_Req (TX-named) source. 1 passes it to the IRQ output; 0 masks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn ieee1588_pdelay_req_tx(&mut self) -> ieee1588_pdelay_req_tx::ASW<'_> {
            ieee1588_pdelay_req_tx::ASW::new(self)
        }
        /// Enable for the PTP Delay_Req transmitted source. 1 passes it to the IRQ output; 0 masks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn ieee1588_delay_req_tx(&mut self) -> ieee1588_delay_req_tx::ASW<'_> {
            ieee1588_delay_req_tx::ASW::new(self)
        }
        /// Enable for the PTP Sync transmitted source. 1 passes it to the IRQ output; 0 masks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn ieee1588_sync_frame_tx(&mut self) -> ieee1588_sync_frame_tx::ASW<'_> {
            ieee1588_sync_frame_tx::ASW::new(self)
        }
        /// Enable for the start of frame transmitted source. 1 passes it to the IRQ output; 0 masks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn ieee1588_sof_tx(&mut self) -> ieee1588_sof_tx::ASW<'_> {
            ieee1588_sof_tx::ASW::new(self)
        }
        /// Enable for the PTP Pdelay_Resp received source. 1 passes it to the IRQ output; 0 masks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn ieee1588_pdelay_resp_rx(&mut self) -> ieee1588_pdelay_resp_rx::ASW<'_> {
            ieee1588_pdelay_resp_rx::ASW::new(self)
        }
        /// Enable for the PTP Pdelay_Req received source. 1 passes it to the IRQ output; 0 masks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn ieee1588_pdelay_req_rx(&mut self) -> ieee1588_pdelay_req_rx::ASW<'_> {
            ieee1588_pdelay_req_rx::ASW::new(self)
        }
        /// Enable for the PTP Delay_Req received source. 1 passes it to the IRQ output; 0 masks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn ieee1588_delay_req_rx(&mut self) -> ieee1588_delay_req_rx::ASW<'_> {
            ieee1588_delay_req_rx::ASW::new(self)
        }
        /// Enable for the PTP Sync received source. 1 passes it to the IRQ output; 0 masks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn ieee1588_sync_frame_rx(&mut self) -> ieee1588_sync_frame_rx::ASW<'_> {
            ieee1588_sync_frame_rx::ASW::new(self)
        }
        /// Enable for the start of frame received source. 1 passes it to the IRQ output; 0 masks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn ieee1588_sof_rx(&mut self) -> ieee1588_sof_rx::ASW<'_> {
            ieee1588_sof_rx::ASW::new(self)
        }
        /// Enable for the timestamp comparator match (upper 70 counter bits) source. 1 passes it to the IRQ output; 0 masks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn ieee1588_tsu_timer_cmp_val(&mut self) -> ieee1588_tsu_timer_cmp_val::ASW<'_> {
            ieee1588_tsu_timer_cmp_val::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// Enable for the GEM aggregate interrupt source. 1 passes it to the IRQ output; 0 masks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn ethernet(&mut self) -> ethernet::ACW<'_> {
            ethernet::ACW::new(self)
        }
        /// Enable for the Wake-on-LAN source. 1 passes it to the IRQ output; 0 masks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn wol(&mut self) -> wol::ACW<'_> {
            wol::ACW::new(self)
        }
        /// Enable for the PTP Pdelay_Resp transmitted source. 1 passes it to the IRQ output; 0 masks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn ieee1588_pdelay_resp_tx(&mut self) -> ieee1588_pdelay_resp_tx::ACW<'_> {
            ieee1588_pdelay_resp_tx::ACW::new(self)
        }
        /// Enable for the PTP Pdelay_Req (TX-named) source. 1 passes it to the IRQ output; 0 masks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn ieee1588_pdelay_req_tx(&mut self) -> ieee1588_pdelay_req_tx::ACW<'_> {
            ieee1588_pdelay_req_tx::ACW::new(self)
        }
        /// Enable for the PTP Delay_Req transmitted source. 1 passes it to the IRQ output; 0 masks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn ieee1588_delay_req_tx(&mut self) -> ieee1588_delay_req_tx::ACW<'_> {
            ieee1588_delay_req_tx::ACW::new(self)
        }
        /// Enable for the PTP Sync transmitted source. 1 passes it to the IRQ output; 0 masks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn ieee1588_sync_frame_tx(&mut self) -> ieee1588_sync_frame_tx::ACW<'_> {
            ieee1588_sync_frame_tx::ACW::new(self)
        }
        /// Enable for the start of frame transmitted source. 1 passes it to the IRQ output; 0 masks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn ieee1588_sof_tx(&mut self) -> ieee1588_sof_tx::ACW<'_> {
            ieee1588_sof_tx::ACW::new(self)
        }
        /// Enable for the PTP Pdelay_Resp received source. 1 passes it to the IRQ output; 0 masks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn ieee1588_pdelay_resp_rx(&mut self) -> ieee1588_pdelay_resp_rx::ACW<'_> {
            ieee1588_pdelay_resp_rx::ACW::new(self)
        }
        /// Enable for the PTP Pdelay_Req received source. 1 passes it to the IRQ output; 0 masks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn ieee1588_pdelay_req_rx(&mut self) -> ieee1588_pdelay_req_rx::ACW<'_> {
            ieee1588_pdelay_req_rx::ACW::new(self)
        }
        /// Enable for the PTP Delay_Req received source. 1 passes it to the IRQ output; 0 masks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn ieee1588_delay_req_rx(&mut self) -> ieee1588_delay_req_rx::ACW<'_> {
            ieee1588_delay_req_rx::ACW::new(self)
        }
        /// Enable for the PTP Sync received source. 1 passes it to the IRQ output; 0 masks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn ieee1588_sync_frame_rx(&mut self) -> ieee1588_sync_frame_rx::ACW<'_> {
            ieee1588_sync_frame_rx::ACW::new(self)
        }
        /// Enable for the start of frame received source. 1 passes it to the IRQ output; 0 masks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn ieee1588_sof_rx(&mut self) -> ieee1588_sof_rx::ACW<'_> {
            ieee1588_sof_rx::ACW::new(self)
        }
        /// Enable for the timestamp comparator match (upper 70 counter bits) source. 1 passes it to the IRQ output; 0 masks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn ieee1588_tsu_timer_cmp_val(&mut self) -> ieee1588_tsu_timer_cmp_val::ACW<'_> {
            ieee1588_tsu_timer_cmp_val::ACW::new(self)
        }
    }
    impl AtomicXorW {
        /// Enable for the GEM aggregate interrupt source. 1 passes it to the IRQ output; 0 masks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn ethernet(&mut self) -> ethernet::AXW<'_> {
            ethernet::AXW::new(self)
        }
        /// Enable for the Wake-on-LAN source. 1 passes it to the IRQ output; 0 masks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn wol(&mut self) -> wol::AXW<'_> {
            wol::AXW::new(self)
        }
        /// Enable for the PTP Pdelay_Resp transmitted source. 1 passes it to the IRQ output; 0 masks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn ieee1588_pdelay_resp_tx(&mut self) -> ieee1588_pdelay_resp_tx::AXW<'_> {
            ieee1588_pdelay_resp_tx::AXW::new(self)
        }
        /// Enable for the PTP Pdelay_Req (TX-named) source. 1 passes it to the IRQ output; 0 masks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn ieee1588_pdelay_req_tx(&mut self) -> ieee1588_pdelay_req_tx::AXW<'_> {
            ieee1588_pdelay_req_tx::AXW::new(self)
        }
        /// Enable for the PTP Delay_Req transmitted source. 1 passes it to the IRQ output; 0 masks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn ieee1588_delay_req_tx(&mut self) -> ieee1588_delay_req_tx::AXW<'_> {
            ieee1588_delay_req_tx::AXW::new(self)
        }
        /// Enable for the PTP Sync transmitted source. 1 passes it to the IRQ output; 0 masks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn ieee1588_sync_frame_tx(&mut self) -> ieee1588_sync_frame_tx::AXW<'_> {
            ieee1588_sync_frame_tx::AXW::new(self)
        }
        /// Enable for the start of frame transmitted source. 1 passes it to the IRQ output; 0 masks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn ieee1588_sof_tx(&mut self) -> ieee1588_sof_tx::AXW<'_> {
            ieee1588_sof_tx::AXW::new(self)
        }
        /// Enable for the PTP Pdelay_Resp received source. 1 passes it to the IRQ output; 0 masks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn ieee1588_pdelay_resp_rx(&mut self) -> ieee1588_pdelay_resp_rx::AXW<'_> {
            ieee1588_pdelay_resp_rx::AXW::new(self)
        }
        /// Enable for the PTP Pdelay_Req received source. 1 passes it to the IRQ output; 0 masks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn ieee1588_pdelay_req_rx(&mut self) -> ieee1588_pdelay_req_rx::AXW<'_> {
            ieee1588_pdelay_req_rx::AXW::new(self)
        }
        /// Enable for the PTP Delay_Req received source. 1 passes it to the IRQ output; 0 masks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn ieee1588_delay_req_rx(&mut self) -> ieee1588_delay_req_rx::AXW<'_> {
            ieee1588_delay_req_rx::AXW::new(self)
        }
        /// Enable for the PTP Sync received source. 1 passes it to the IRQ output; 0 masks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn ieee1588_sync_frame_rx(&mut self) -> ieee1588_sync_frame_rx::AXW<'_> {
            ieee1588_sync_frame_rx::AXW::new(self)
        }
        /// Enable for the start of frame received source. 1 passes it to the IRQ output; 0 masks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn ieee1588_sof_rx(&mut self) -> ieee1588_sof_rx::AXW<'_> {
            ieee1588_sof_rx::AXW::new(self)
        }
        /// Enable for the timestamp comparator match (upper 70 counter bits) source. 1 passes it to the IRQ output; 0 masks it.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn ieee1588_tsu_timer_cmp_val(&mut self) -> ieee1588_tsu_timer_cmp_val::AXW<'_> {
            ieee1588_tsu_timer_cmp_val::AXW::new(self)
        }
    }
}

/// Interrupt force, one bit per source: 1 asserts the source by software, 0 removes the force.
/// Hardware access: read Plain; write Plain.
pub type INTF = Reg<intf::INTF_SPEC>;
/// Interrupt force, one bit per source: 1 asserts the source by software, 0 removes the force.
/// Hardware access: read Plain; write Plain.
pub mod intf {
    pub struct INTF_SPEC;
    impl crate::RegisterSpec for INTF_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for INTF_SPEC {}
    pub type R = crate::R<INTF_SPEC>;
    impl crate::Modifiable for INTF_SPEC {}
    impl crate::UnsafeWritable for INTF_SPEC {}
    impl crate::ResetValue for INTF_SPEC {
        const RESET_VALUE: u32 = 0x0;
    }
    impl crate::TypedAtomicSet for INTF_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicSet for INTF_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::TypedAtomicClear for INTF_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicClear for INTF_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::TypedAtomicXor for INTF_SPEC {
        const OFFSET: usize = 0x1000;
    }
    impl crate::UnsafeAtomicXor for INTF_SPEC {
        const OFFSET: usize = 0x1000;
    }
    pub type W = crate::W<INTF_SPEC>;
    pub type AtomicSetW = crate::AtomicSetW<INTF_SPEC>;
    pub type AtomicClearW = crate::AtomicClearW<INTF_SPEC>;
    pub type AtomicXorW = crate::AtomicXorW<INTF_SPEC>;
    /// Force for the GEM aggregate interrupt source. 1 raises the IRQ by software; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod ethernet {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 0>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 0>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 0>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 0>;
    }
    /// Force for the Wake-on-LAN source. 1 raises the IRQ by software; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod wol {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 1>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 1>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 1>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 1>;
    }
    /// Force for the PTP Pdelay_Resp transmitted source. 1 raises the IRQ by software; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod ieee1588_pdelay_resp_tx {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 2>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 2>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 2>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 2>;
    }
    /// Force for the PTP Pdelay_Req (TX-named) source. 1 raises the IRQ by software; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod ieee1588_pdelay_req_tx {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 3>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 3>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 3>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 3>;
    }
    /// Force for the PTP Delay_Req transmitted source. 1 raises the IRQ by software; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod ieee1588_delay_req_tx {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 4>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 4>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 4>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 4>;
    }
    /// Force for the PTP Sync transmitted source. 1 raises the IRQ by software; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod ieee1588_sync_frame_tx {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 5>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 5>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 5>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 5>;
    }
    /// Force for the start of frame transmitted source. 1 raises the IRQ by software; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod ieee1588_sof_tx {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 6>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 6>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 6>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 6>;
    }
    /// Force for the PTP Pdelay_Resp received source. 1 raises the IRQ by software; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod ieee1588_pdelay_resp_rx {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 7>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 7>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 7>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 7>;
    }
    /// Force for the PTP Pdelay_Req received source. 1 raises the IRQ by software; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod ieee1588_pdelay_req_rx {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 8>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 8>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 8>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 8>;
    }
    /// Force for the PTP Delay_Req received source. 1 raises the IRQ by software; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod ieee1588_delay_req_rx {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 9>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 9>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 9>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 9>;
    }
    /// Force for the PTP Sync received source. 1 raises the IRQ by software; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod ieee1588_sync_frame_rx {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 10>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 10>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 10>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 10>;
    }
    /// Force for the start of frame received source. 1 raises the IRQ by software; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod ieee1588_sof_rx {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 11>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 11>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 11>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 11>;
    }
    /// Force for the timestamp comparator match (upper 70 counter bits) source. 1 raises the IRQ by software; 0 removes the force.
    /// Field write behavior: Plain.
    pub mod ieee1588_tsu_timer_cmp_val {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 12>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 12>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 12>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 12>;
    }
    impl R {
        #[inline(always)]
        pub fn ethernet(&self) -> ethernet::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn wol(&self) -> wol::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
        #[inline(always)]
        pub fn ieee1588_pdelay_resp_tx(&self) -> ieee1588_pdelay_resp_tx::R {
            crate::BitReader::new(((self.bits() >> 2) & 1) != 0)
        }
        #[inline(always)]
        pub fn ieee1588_pdelay_req_tx(&self) -> ieee1588_pdelay_req_tx::R {
            crate::BitReader::new(((self.bits() >> 3) & 1) != 0)
        }
        #[inline(always)]
        pub fn ieee1588_delay_req_tx(&self) -> ieee1588_delay_req_tx::R {
            crate::BitReader::new(((self.bits() >> 4) & 1) != 0)
        }
        #[inline(always)]
        pub fn ieee1588_sync_frame_tx(&self) -> ieee1588_sync_frame_tx::R {
            crate::BitReader::new(((self.bits() >> 5) & 1) != 0)
        }
        #[inline(always)]
        pub fn ieee1588_sof_tx(&self) -> ieee1588_sof_tx::R {
            crate::BitReader::new(((self.bits() >> 6) & 1) != 0)
        }
        #[inline(always)]
        pub fn ieee1588_pdelay_resp_rx(&self) -> ieee1588_pdelay_resp_rx::R {
            crate::BitReader::new(((self.bits() >> 7) & 1) != 0)
        }
        #[inline(always)]
        pub fn ieee1588_pdelay_req_rx(&self) -> ieee1588_pdelay_req_rx::R {
            crate::BitReader::new(((self.bits() >> 8) & 1) != 0)
        }
        #[inline(always)]
        pub fn ieee1588_delay_req_rx(&self) -> ieee1588_delay_req_rx::R {
            crate::BitReader::new(((self.bits() >> 9) & 1) != 0)
        }
        #[inline(always)]
        pub fn ieee1588_sync_frame_rx(&self) -> ieee1588_sync_frame_rx::R {
            crate::BitReader::new(((self.bits() >> 10) & 1) != 0)
        }
        #[inline(always)]
        pub fn ieee1588_sof_rx(&self) -> ieee1588_sof_rx::R {
            crate::BitReader::new(((self.bits() >> 11) & 1) != 0)
        }
        #[inline(always)]
        pub fn ieee1588_tsu_timer_cmp_val(&self) -> ieee1588_tsu_timer_cmp_val::R {
            crate::BitReader::new(((self.bits() >> 12) & 1) != 0)
        }
    }
    impl W {
        #[inline(always)]
        pub fn ethernet(&mut self) -> ethernet::W<'_> {
            ethernet::W::new(self)
        }
        #[inline(always)]
        pub fn wol(&mut self) -> wol::W<'_> {
            wol::W::new(self)
        }
        #[inline(always)]
        pub fn ieee1588_pdelay_resp_tx(&mut self) -> ieee1588_pdelay_resp_tx::W<'_> {
            ieee1588_pdelay_resp_tx::W::new(self)
        }
        #[inline(always)]
        pub fn ieee1588_pdelay_req_tx(&mut self) -> ieee1588_pdelay_req_tx::W<'_> {
            ieee1588_pdelay_req_tx::W::new(self)
        }
        #[inline(always)]
        pub fn ieee1588_delay_req_tx(&mut self) -> ieee1588_delay_req_tx::W<'_> {
            ieee1588_delay_req_tx::W::new(self)
        }
        #[inline(always)]
        pub fn ieee1588_sync_frame_tx(&mut self) -> ieee1588_sync_frame_tx::W<'_> {
            ieee1588_sync_frame_tx::W::new(self)
        }
        #[inline(always)]
        pub fn ieee1588_sof_tx(&mut self) -> ieee1588_sof_tx::W<'_> {
            ieee1588_sof_tx::W::new(self)
        }
        #[inline(always)]
        pub fn ieee1588_pdelay_resp_rx(&mut self) -> ieee1588_pdelay_resp_rx::W<'_> {
            ieee1588_pdelay_resp_rx::W::new(self)
        }
        #[inline(always)]
        pub fn ieee1588_pdelay_req_rx(&mut self) -> ieee1588_pdelay_req_rx::W<'_> {
            ieee1588_pdelay_req_rx::W::new(self)
        }
        #[inline(always)]
        pub fn ieee1588_delay_req_rx(&mut self) -> ieee1588_delay_req_rx::W<'_> {
            ieee1588_delay_req_rx::W::new(self)
        }
        #[inline(always)]
        pub fn ieee1588_sync_frame_rx(&mut self) -> ieee1588_sync_frame_rx::W<'_> {
            ieee1588_sync_frame_rx::W::new(self)
        }
        #[inline(always)]
        pub fn ieee1588_sof_rx(&mut self) -> ieee1588_sof_rx::W<'_> {
            ieee1588_sof_rx::W::new(self)
        }
        #[inline(always)]
        pub fn ieee1588_tsu_timer_cmp_val(&mut self) -> ieee1588_tsu_timer_cmp_val::W<'_> {
            ieee1588_tsu_timer_cmp_val::W::new(self)
        }
    }
    impl AtomicSetW {
        /// Force for the GEM aggregate interrupt source. 1 raises the IRQ by software; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn ethernet(&mut self) -> ethernet::ASW<'_> {
            ethernet::ASW::new(self)
        }
        /// Force for the Wake-on-LAN source. 1 raises the IRQ by software; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn wol(&mut self) -> wol::ASW<'_> {
            wol::ASW::new(self)
        }
        /// Force for the PTP Pdelay_Resp transmitted source. 1 raises the IRQ by software; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn ieee1588_pdelay_resp_tx(&mut self) -> ieee1588_pdelay_resp_tx::ASW<'_> {
            ieee1588_pdelay_resp_tx::ASW::new(self)
        }
        /// Force for the PTP Pdelay_Req (TX-named) source. 1 raises the IRQ by software; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn ieee1588_pdelay_req_tx(&mut self) -> ieee1588_pdelay_req_tx::ASW<'_> {
            ieee1588_pdelay_req_tx::ASW::new(self)
        }
        /// Force for the PTP Delay_Req transmitted source. 1 raises the IRQ by software; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn ieee1588_delay_req_tx(&mut self) -> ieee1588_delay_req_tx::ASW<'_> {
            ieee1588_delay_req_tx::ASW::new(self)
        }
        /// Force for the PTP Sync transmitted source. 1 raises the IRQ by software; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn ieee1588_sync_frame_tx(&mut self) -> ieee1588_sync_frame_tx::ASW<'_> {
            ieee1588_sync_frame_tx::ASW::new(self)
        }
        /// Force for the start of frame transmitted source. 1 raises the IRQ by software; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn ieee1588_sof_tx(&mut self) -> ieee1588_sof_tx::ASW<'_> {
            ieee1588_sof_tx::ASW::new(self)
        }
        /// Force for the PTP Pdelay_Resp received source. 1 raises the IRQ by software; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn ieee1588_pdelay_resp_rx(&mut self) -> ieee1588_pdelay_resp_rx::ASW<'_> {
            ieee1588_pdelay_resp_rx::ASW::new(self)
        }
        /// Force for the PTP Pdelay_Req received source. 1 raises the IRQ by software; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn ieee1588_pdelay_req_rx(&mut self) -> ieee1588_pdelay_req_rx::ASW<'_> {
            ieee1588_pdelay_req_rx::ASW::new(self)
        }
        /// Force for the PTP Delay_Req received source. 1 raises the IRQ by software; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn ieee1588_delay_req_rx(&mut self) -> ieee1588_delay_req_rx::ASW<'_> {
            ieee1588_delay_req_rx::ASW::new(self)
        }
        /// Force for the PTP Sync received source. 1 raises the IRQ by software; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn ieee1588_sync_frame_rx(&mut self) -> ieee1588_sync_frame_rx::ASW<'_> {
            ieee1588_sync_frame_rx::ASW::new(self)
        }
        /// Force for the start of frame received source. 1 raises the IRQ by software; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn ieee1588_sof_rx(&mut self) -> ieee1588_sof_rx::ASW<'_> {
            ieee1588_sof_rx::ASW::new(self)
        }
        /// Force for the timestamp comparator match (upper 70 counter bits) source. 1 raises the IRQ by software; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn ieee1588_tsu_timer_cmp_val(&mut self) -> ieee1588_tsu_timer_cmp_val::ASW<'_> {
            ieee1588_tsu_timer_cmp_val::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// Force for the GEM aggregate interrupt source. 1 raises the IRQ by software; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn ethernet(&mut self) -> ethernet::ACW<'_> {
            ethernet::ACW::new(self)
        }
        /// Force for the Wake-on-LAN source. 1 raises the IRQ by software; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn wol(&mut self) -> wol::ACW<'_> {
            wol::ACW::new(self)
        }
        /// Force for the PTP Pdelay_Resp transmitted source. 1 raises the IRQ by software; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn ieee1588_pdelay_resp_tx(&mut self) -> ieee1588_pdelay_resp_tx::ACW<'_> {
            ieee1588_pdelay_resp_tx::ACW::new(self)
        }
        /// Force for the PTP Pdelay_Req (TX-named) source. 1 raises the IRQ by software; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn ieee1588_pdelay_req_tx(&mut self) -> ieee1588_pdelay_req_tx::ACW<'_> {
            ieee1588_pdelay_req_tx::ACW::new(self)
        }
        /// Force for the PTP Delay_Req transmitted source. 1 raises the IRQ by software; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn ieee1588_delay_req_tx(&mut self) -> ieee1588_delay_req_tx::ACW<'_> {
            ieee1588_delay_req_tx::ACW::new(self)
        }
        /// Force for the PTP Sync transmitted source. 1 raises the IRQ by software; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn ieee1588_sync_frame_tx(&mut self) -> ieee1588_sync_frame_tx::ACW<'_> {
            ieee1588_sync_frame_tx::ACW::new(self)
        }
        /// Force for the start of frame transmitted source. 1 raises the IRQ by software; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn ieee1588_sof_tx(&mut self) -> ieee1588_sof_tx::ACW<'_> {
            ieee1588_sof_tx::ACW::new(self)
        }
        /// Force for the PTP Pdelay_Resp received source. 1 raises the IRQ by software; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn ieee1588_pdelay_resp_rx(&mut self) -> ieee1588_pdelay_resp_rx::ACW<'_> {
            ieee1588_pdelay_resp_rx::ACW::new(self)
        }
        /// Force for the PTP Pdelay_Req received source. 1 raises the IRQ by software; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn ieee1588_pdelay_req_rx(&mut self) -> ieee1588_pdelay_req_rx::ACW<'_> {
            ieee1588_pdelay_req_rx::ACW::new(self)
        }
        /// Force for the PTP Delay_Req received source. 1 raises the IRQ by software; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn ieee1588_delay_req_rx(&mut self) -> ieee1588_delay_req_rx::ACW<'_> {
            ieee1588_delay_req_rx::ACW::new(self)
        }
        /// Force for the PTP Sync received source. 1 raises the IRQ by software; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn ieee1588_sync_frame_rx(&mut self) -> ieee1588_sync_frame_rx::ACW<'_> {
            ieee1588_sync_frame_rx::ACW::new(self)
        }
        /// Force for the start of frame received source. 1 raises the IRQ by software; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn ieee1588_sof_rx(&mut self) -> ieee1588_sof_rx::ACW<'_> {
            ieee1588_sof_rx::ACW::new(self)
        }
        /// Force for the timestamp comparator match (upper 70 counter bits) source. 1 raises the IRQ by software; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn ieee1588_tsu_timer_cmp_val(&mut self) -> ieee1588_tsu_timer_cmp_val::ACW<'_> {
            ieee1588_tsu_timer_cmp_val::ACW::new(self)
        }
    }
    impl AtomicXorW {
        /// Force for the GEM aggregate interrupt source. 1 raises the IRQ by software; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn ethernet(&mut self) -> ethernet::AXW<'_> {
            ethernet::AXW::new(self)
        }
        /// Force for the Wake-on-LAN source. 1 raises the IRQ by software; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn wol(&mut self) -> wol::AXW<'_> {
            wol::AXW::new(self)
        }
        /// Force for the PTP Pdelay_Resp transmitted source. 1 raises the IRQ by software; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn ieee1588_pdelay_resp_tx(&mut self) -> ieee1588_pdelay_resp_tx::AXW<'_> {
            ieee1588_pdelay_resp_tx::AXW::new(self)
        }
        /// Force for the PTP Pdelay_Req (TX-named) source. 1 raises the IRQ by software; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn ieee1588_pdelay_req_tx(&mut self) -> ieee1588_pdelay_req_tx::AXW<'_> {
            ieee1588_pdelay_req_tx::AXW::new(self)
        }
        /// Force for the PTP Delay_Req transmitted source. 1 raises the IRQ by software; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn ieee1588_delay_req_tx(&mut self) -> ieee1588_delay_req_tx::AXW<'_> {
            ieee1588_delay_req_tx::AXW::new(self)
        }
        /// Force for the PTP Sync transmitted source. 1 raises the IRQ by software; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn ieee1588_sync_frame_tx(&mut self) -> ieee1588_sync_frame_tx::AXW<'_> {
            ieee1588_sync_frame_tx::AXW::new(self)
        }
        /// Force for the start of frame transmitted source. 1 raises the IRQ by software; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn ieee1588_sof_tx(&mut self) -> ieee1588_sof_tx::AXW<'_> {
            ieee1588_sof_tx::AXW::new(self)
        }
        /// Force for the PTP Pdelay_Resp received source. 1 raises the IRQ by software; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn ieee1588_pdelay_resp_rx(&mut self) -> ieee1588_pdelay_resp_rx::AXW<'_> {
            ieee1588_pdelay_resp_rx::AXW::new(self)
        }
        /// Force for the PTP Pdelay_Req received source. 1 raises the IRQ by software; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn ieee1588_pdelay_req_rx(&mut self) -> ieee1588_pdelay_req_rx::AXW<'_> {
            ieee1588_pdelay_req_rx::AXW::new(self)
        }
        /// Force for the PTP Delay_Req received source. 1 raises the IRQ by software; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn ieee1588_delay_req_rx(&mut self) -> ieee1588_delay_req_rx::AXW<'_> {
            ieee1588_delay_req_rx::AXW::new(self)
        }
        /// Force for the PTP Sync received source. 1 raises the IRQ by software; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn ieee1588_sync_frame_rx(&mut self) -> ieee1588_sync_frame_rx::AXW<'_> {
            ieee1588_sync_frame_rx::AXW::new(self)
        }
        /// Force for the start of frame received source. 1 raises the IRQ by software; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn ieee1588_sof_rx(&mut self) -> ieee1588_sof_rx::AXW<'_> {
            ieee1588_sof_rx::AXW::new(self)
        }
        /// Force for the timestamp comparator match (upper 70 counter bits) source. 1 raises the IRQ by software; 0 removes the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn ieee1588_tsu_timer_cmp_val(&mut self) -> ieee1588_tsu_timer_cmp_val::AXW<'_> {
            ieee1588_tsu_timer_cmp_val::AXW::new(self)
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
    /// 1 when the GEM aggregate interrupt source is driving the IRQ output.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod ethernet {
        pub type R = crate::BitReader;
    }
    /// 1 when the Wake-on-LAN source is driving the IRQ output.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod wol {
        pub type R = crate::BitReader;
    }
    /// 1 when the PTP Pdelay_Resp transmitted source is driving the IRQ output.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod ieee1588_pdelay_resp_tx {
        pub type R = crate::BitReader;
    }
    /// 1 when the PTP Pdelay_Req (TX-named) source is driving the IRQ output.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod ieee1588_pdelay_req_tx {
        pub type R = crate::BitReader;
    }
    /// 1 when the PTP Delay_Req transmitted source is driving the IRQ output.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod ieee1588_delay_req_tx {
        pub type R = crate::BitReader;
    }
    /// 1 when the PTP Sync transmitted source is driving the IRQ output.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod ieee1588_sync_frame_tx {
        pub type R = crate::BitReader;
    }
    /// 1 when the start of frame transmitted source is driving the IRQ output.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod ieee1588_sof_tx {
        pub type R = crate::BitReader;
    }
    /// 1 when the PTP Pdelay_Resp received source is driving the IRQ output.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod ieee1588_pdelay_resp_rx {
        pub type R = crate::BitReader;
    }
    /// 1 when the PTP Pdelay_Req received source is driving the IRQ output.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod ieee1588_pdelay_req_rx {
        pub type R = crate::BitReader;
    }
    /// 1 when the PTP Delay_Req received source is driving the IRQ output.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod ieee1588_delay_req_rx {
        pub type R = crate::BitReader;
    }
    /// 1 when the PTP Sync received source is driving the IRQ output.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod ieee1588_sync_frame_rx {
        pub type R = crate::BitReader;
    }
    /// 1 when the start of frame received source is driving the IRQ output.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod ieee1588_sof_rx {
        pub type R = crate::BitReader;
    }
    /// 1 when the timestamp comparator match (upper 70 counter bits) source is driving the IRQ output.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod ieee1588_tsu_timer_cmp_val {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn ethernet(&self) -> ethernet::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn wol(&self) -> wol::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
        #[inline(always)]
        pub fn ieee1588_pdelay_resp_tx(&self) -> ieee1588_pdelay_resp_tx::R {
            crate::BitReader::new(((self.bits() >> 2) & 1) != 0)
        }
        #[inline(always)]
        pub fn ieee1588_pdelay_req_tx(&self) -> ieee1588_pdelay_req_tx::R {
            crate::BitReader::new(((self.bits() >> 3) & 1) != 0)
        }
        #[inline(always)]
        pub fn ieee1588_delay_req_tx(&self) -> ieee1588_delay_req_tx::R {
            crate::BitReader::new(((self.bits() >> 4) & 1) != 0)
        }
        #[inline(always)]
        pub fn ieee1588_sync_frame_tx(&self) -> ieee1588_sync_frame_tx::R {
            crate::BitReader::new(((self.bits() >> 5) & 1) != 0)
        }
        #[inline(always)]
        pub fn ieee1588_sof_tx(&self) -> ieee1588_sof_tx::R {
            crate::BitReader::new(((self.bits() >> 6) & 1) != 0)
        }
        #[inline(always)]
        pub fn ieee1588_pdelay_resp_rx(&self) -> ieee1588_pdelay_resp_rx::R {
            crate::BitReader::new(((self.bits() >> 7) & 1) != 0)
        }
        #[inline(always)]
        pub fn ieee1588_pdelay_req_rx(&self) -> ieee1588_pdelay_req_rx::R {
            crate::BitReader::new(((self.bits() >> 8) & 1) != 0)
        }
        #[inline(always)]
        pub fn ieee1588_delay_req_rx(&self) -> ieee1588_delay_req_rx::R {
            crate::BitReader::new(((self.bits() >> 9) & 1) != 0)
        }
        #[inline(always)]
        pub fn ieee1588_sync_frame_rx(&self) -> ieee1588_sync_frame_rx::R {
            crate::BitReader::new(((self.bits() >> 10) & 1) != 0)
        }
        #[inline(always)]
        pub fn ieee1588_sof_rx(&self) -> ieee1588_sof_rx::R {
            crate::BitReader::new(((self.bits() >> 11) & 1) != 0)
        }
        #[inline(always)]
        pub fn ieee1588_tsu_timer_cmp_val(&self) -> ieee1588_tsu_timer_cmp_val::R {
            crate::BitReader::new(((self.bits() >> 12) & 1) != 0)
        }
    }
}

#[cfg(test)]
mod layout_tests {
    use super::RegisterBlock;
    #[test]
    fn control_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, control), 0x0);
    }
    #[test]
    fn status_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, status), 0x4);
    }
    #[test]
    fn tsu_timer_cnt0_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, tsu_timer_cnt0), 0x8);
    }
    #[test]
    fn tsu_timer_cnt1_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, tsu_timer_cnt1), 0xc);
    }
    #[test]
    fn tsu_timer_cnt2_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, tsu_timer_cnt2), 0x10);
    }
    #[test]
    fn clkgen_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, clkgen), 0x14);
    }
    #[test]
    fn clk2fc_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, clk2fc), 0x18);
    }
    #[test]
    fn intr_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, intr), 0x1c);
    }
    #[test]
    fn inte_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, inte), 0x20);
    }
    #[test]
    fn intf_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, intf), 0x24);
    }
    #[test]
    fn ints_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, ints), 0x28);
    }
    #[test]
    fn register_block_size() {
        assert_eq!(core::mem::size_of::<RegisterBlock>(), 0x2c);
    }
}
