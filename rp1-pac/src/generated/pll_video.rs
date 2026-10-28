// Generated from rp1-regdb/data/peripherals/pll-video.toml.
// Do not edit manually.
// Regenerate with: cargo run -p rp1-regdb -- generate

use crate::Reg;

/// RP1 PLL_VIDEO register block with supplemental register semantics. Alias behavior: normal read/write access. PLL layout recovered from the Linux RP1 common-clock driver. Unknown offsets remain split RAW arrays.
/// Atomic view: Rp1Native alias windows; ownership remains with this logical peripheral.
#[repr(C)]
pub struct RegisterBlock {
    cs: CS,
    pwr: PWR,
    fbdiv_int: FBDIV_INT,
    fbdiv_frac: FBDIV_FRAC,
    prim: PRIM,
    sec: SEC,
}

impl RegisterBlock {
    /// PLL control/status. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn cs(&self) -> &CS {
        &self.cs
    }
    /// PLL power control. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn pwr(&self) -> &PWR {
        &self.pwr
    }
    /// Integer feedback divider; exact implemented width remains raw. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn fbdiv_int(&self) -> &FBDIV_INT {
        &self.fbdiv_int
    }
    /// Fractional feedback divider; exact implemented width remains raw. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn fbdiv_frac(&self) -> &FBDIV_FRAC {
        &self.fbdiv_frac
    }
    /// Primary PLL divider and phase-output control. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn prim(&self) -> &PRIM {
        &self.prim
    }
    /// Secondary PLL divider and phase-output control. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn sec(&self) -> &SEC {
        &self.sec
    }
}

/// PLL control/status. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type CS = Reg<cs::CS_SPEC>;
/// PLL control/status. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod cs {
    pub struct CS_SPEC;
    impl crate::RegisterSpec for CS_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CS_SPEC {}
    pub type R = crate::R<CS_SPEC>;
    impl crate::UnsafeWritable for CS_SPEC {}
    impl crate::UnsafeAtomicSet for CS_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for CS_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for CS_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// Reference-divider field as masked by Linux.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed because write behavior is not hardware/official verified.
    pub mod refdiv {
        pub type R = crate::BitReader;
    }
    /// PLL lock indication.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod lock {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn refdiv(&self) -> refdiv::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
        #[inline(always)]
        pub fn lock(&self) -> lock::R {
            crate::BitReader::new(((self.bits() >> 31) & 1) != 0)
        }
    }
}

/// PLL power control. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type PWR = Reg<pwr::PWR_SPEC>;
/// PLL power control. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod pwr {
    pub struct PWR_SPEC;
    impl crate::RegisterSpec for PWR_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for PWR_SPEC {}
    pub type R = crate::R<PWR_SPEC>;
    impl crate::UnsafeWritable for PWR_SPEC {}
    impl crate::UnsafeAtomicSet for PWR_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for PWR_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for PWR_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// PLL power-down.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod pd {
        pub type R = crate::BitReader;
    }
    /// DAC power-down.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod dacpd {
        pub type R = crate::BitReader;
    }
    /// Delta-sigma modulator power-down.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod dsmpd {
        pub type R = crate::BitReader;
    }
    /// Post-divider power-down.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod postdivpd {
        pub type R = crate::BitReader;
    }
    /// Four-phase output power-down.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod phase4pd {
        pub type R = crate::BitReader;
    }
    /// VCO power-down.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod vcopd {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn pd(&self) -> pd::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn dacpd(&self) -> dacpd::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
        #[inline(always)]
        pub fn dsmpd(&self) -> dsmpd::R {
            crate::BitReader::new(((self.bits() >> 2) & 1) != 0)
        }
        #[inline(always)]
        pub fn postdivpd(&self) -> postdivpd::R {
            crate::BitReader::new(((self.bits() >> 3) & 1) != 0)
        }
        #[inline(always)]
        pub fn phase4pd(&self) -> phase4pd::R {
            crate::BitReader::new(((self.bits() >> 4) & 1) != 0)
        }
        #[inline(always)]
        pub fn vcopd(&self) -> vcopd::R {
            crate::BitReader::new(((self.bits() >> 5) & 1) != 0)
        }
    }
}

/// Integer feedback divider; exact implemented width remains raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type FBDIV_INT = Reg<fbdiv_int::FBDIV_INT_SPEC>;
/// Integer feedback divider; exact implemented width remains raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod fbdiv_int {
    pub struct FBDIV_INT_SPEC;
    impl crate::RegisterSpec for FBDIV_INT_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for FBDIV_INT_SPEC {}
    pub type R = crate::R<FBDIV_INT_SPEC>;
    impl crate::UnsafeWritable for FBDIV_INT_SPEC {}
    impl crate::UnsafeAtomicSet for FBDIV_INT_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for FBDIV_INT_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for FBDIV_INT_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// Integer feedback divider; exact implemented width remains raw. \[Source: Linux RP1 common-clock driver.\]
    /// Field write behavior: Plain.
    /// Typed write API is not exposed because write behavior is not hardware/official verified.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffffffff)
        }
    }
}

/// Fractional feedback divider; exact implemented width remains raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type FBDIV_FRAC = Reg<fbdiv_frac::FBDIV_FRAC_SPEC>;
/// Fractional feedback divider; exact implemented width remains raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod fbdiv_frac {
    pub struct FBDIV_FRAC_SPEC;
    impl crate::RegisterSpec for FBDIV_FRAC_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for FBDIV_FRAC_SPEC {}
    pub type R = crate::R<FBDIV_FRAC_SPEC>;
    impl crate::UnsafeWritable for FBDIV_FRAC_SPEC {}
    impl crate::UnsafeAtomicSet for FBDIV_FRAC_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for FBDIV_FRAC_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for FBDIV_FRAC_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// Fractional feedback divider; exact implemented width remains raw. \[Source: Linux RP1 common-clock driver.\]
    /// Field write behavior: Plain.
    /// Typed write API is not exposed because write behavior is not hardware/official verified.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffffffff)
        }
    }
}

/// Primary PLL divider and phase-output control. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type PRIM = Reg<prim::PRIM_SPEC>;
/// Primary PLL divider and phase-output control. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod prim {
    pub struct PRIM_SPEC;
    impl crate::RegisterSpec for PRIM_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for PRIM_SPEC {}
    pub type R = crate::R<PRIM_SPEC>;
    impl crate::UnsafeWritable for PRIM_SPEC {}
    impl crate::UnsafeAtomicSet for PRIM_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for PRIM_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for PRIM_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// Phase selection: 0/90/180/270 degrees.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod phase {
        pub type R = crate::FieldReader<u32>;
    }
    /// Enable phase output. PLL_SYS bit 4 supplies Linux clock ID 6/APB PCLK and is hardware-proven.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed because write behavior is not hardware/official verified.
    pub mod phase_enable {
        pub type R = crate::BitReader;
    }
    /// Second primary post-divider.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod div2 {
        pub type R = crate::FieldReader<u32>;
    }
    /// First primary post-divider.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod div1 {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn phase(&self) -> phase::R {
            crate::FieldReader::new((self.bits() >> 0) & 0x3)
        }
        #[inline(always)]
        pub fn phase_enable(&self) -> phase_enable::R {
            crate::BitReader::new(((self.bits() >> 4) & 1) != 0)
        }
        #[inline(always)]
        pub fn div2(&self) -> div2::R {
            crate::FieldReader::new((self.bits() >> 12) & 0x7)
        }
        #[inline(always)]
        pub fn div1(&self) -> div1::R {
            crate::FieldReader::new((self.bits() >> 16) & 0x7)
        }
    }
}

/// Secondary PLL divider and phase-output control. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type SEC = Reg<sec::SEC_SPEC>;
/// Secondary PLL divider and phase-output control. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod sec {
    pub struct SEC_SPEC;
    impl crate::RegisterSpec for SEC_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for SEC_SPEC {}
    pub type R = crate::R<SEC_SPEC>;
    impl crate::UnsafeWritable for SEC_SPEC {}
    impl crate::UnsafeAtomicSet for SEC_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for SEC_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for SEC_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// Phase selection: 0/90/180/270 degrees.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod phase {
        pub type R = crate::FieldReader<u32>;
    }
    /// Enable phase output.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod phase_enable {
        pub type R = crate::BitReader;
    }
    /// Secondary divider.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod div {
        pub type R = crate::FieldReader<u32>;
    }
    /// Secondary divider reset.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod reset {
        pub type R = crate::BitReader;
    }
    /// Secondary output implemented indication.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod implemented {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn phase(&self) -> phase::R {
            crate::FieldReader::new((self.bits() >> 0) & 0x3)
        }
        #[inline(always)]
        pub fn phase_enable(&self) -> phase_enable::R {
            crate::BitReader::new(((self.bits() >> 4) & 1) != 0)
        }
        #[inline(always)]
        pub fn div(&self) -> div::R {
            crate::FieldReader::new((self.bits() >> 8) & 0x1f)
        }
        #[inline(always)]
        pub fn reset(&self) -> reset::R {
            crate::BitReader::new(((self.bits() >> 16) & 1) != 0)
        }
        #[inline(always)]
        pub fn implemented(&self) -> implemented::R {
            crate::BitReader::new(((self.bits() >> 31) & 1) != 0)
        }
    }
}

#[cfg(test)]
mod layout_tests {
    use super::RegisterBlock;
    #[test]
    fn cs_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, cs), 0x0);
    }
    #[test]
    fn pwr_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, pwr), 0x4);
    }
    #[test]
    fn fbdiv_int_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, fbdiv_int), 0x8);
    }
    #[test]
    fn fbdiv_frac_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, fbdiv_frac), 0xc);
    }
    #[test]
    fn prim_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, prim), 0x10);
    }
    #[test]
    fn sec_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, sec), 0x14);
    }
    #[test]
    fn register_block_size() {
        assert_eq!(core::mem::size_of::<RegisterBlock>(), 0x18);
    }
}
