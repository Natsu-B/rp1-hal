// Generated from rp1-regdb/data/peripherals/sys-rio1.toml.
// Do not edit manually.
// Regenerate with: cargo run -p rp1-regdb -- generate

use crate::Reg;

/// RP1 SYS_RIO1 register block with supplemental register semantics. Alias behavior: normal read/write access. Bank geometry is recovered from the Linux pinctrl driver; register fields are inherited from the public bank-0 layout. Unknown offsets remain split RAW arrays.
/// Atomic view: Rp1Native alias windows; ownership remains with this logical peripheral.
#[repr(C)]
pub struct RegisterBlock {
    out: OUT,
    oe: OE,
    nosync_in: NOSYNC_IN,
    sync_in: SYNC_IN,
}

impl RegisterBlock {
    /// RIO_OUT controls GPIO output drive
    /// Hardware access: read Plain; write Plain.
    #[inline(always)]
    pub const fn out(&self) -> &OUT {
        &self.out
    }
    /// RIO_OE controls GPIO output-enable drive
    /// Hardware access: read Plain; write Plain.
    #[inline(always)]
    pub const fn oe(&self) -> &OE {
        &self.oe
    }
    /// RIO_NOSYNC_IN samples GPIO inputs directly
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn nosync_in(&self) -> &NOSYNC_IN {
        &self.nosync_in
    }
    /// RIO_SYNC_IN samples GPIO inputs synchronised with clk_sys
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn sync_in(&self) -> &SYNC_IN {
        &self.sync_in
    }
}

/// RIO_OUT controls GPIO output drive
/// Hardware access: read Plain; write Plain.
pub type OUT = Reg<out::OUT_SPEC>;
/// RIO_OUT controls GPIO output drive
/// Hardware access: read Plain; write Plain.
pub mod out {
    pub struct OUT_SPEC;
    impl crate::RegisterSpec for OUT_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for OUT_SPEC {}
    pub type R = crate::R<OUT_SPEC>;
    impl crate::Modifiable for OUT_SPEC {}
    impl crate::UnsafeWritable for OUT_SPEC {}
    impl crate::TypedAtomicSet for OUT_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicSet for OUT_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::TypedAtomicClear for OUT_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicClear for OUT_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::TypedAtomicXor for OUT_SPEC {
        const OFFSET: usize = 0x1000;
    }
    impl crate::UnsafeAtomicXor for OUT_SPEC {
        const OFFSET: usize = 0x1000;
    }
    pub type W = crate::W<OUT_SPEC>;
    pub type AtomicSetW = crate::AtomicSetW<OUT_SPEC>;
    pub type AtomicClearW = crate::AtomicClearW<OUT_SPEC>;
    pub type AtomicXorW = crate::AtomicXorW<OUT_SPEC>;
    /// GPIO28 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio28 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 0>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 0>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 0>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 0>;
    }
    /// GPIO29 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio29 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 1>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 1>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 1>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 1>;
    }
    /// GPIO30 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio30 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 2>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 2>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 2>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 2>;
    }
    /// GPIO31 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio31 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 3>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 3>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 3>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 3>;
    }
    /// GPIO32 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio32 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 4>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 4>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 4>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 4>;
    }
    /// GPIO33 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio33 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 5>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 5>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 5>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 5>;
    }
    impl R {
        #[inline(always)]
        pub fn gpio28(&self) -> gpio28::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio29(&self) -> gpio29::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio30(&self) -> gpio30::R {
            crate::BitReader::new(((self.bits() >> 2) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio31(&self) -> gpio31::R {
            crate::BitReader::new(((self.bits() >> 3) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio32(&self) -> gpio32::R {
            crate::BitReader::new(((self.bits() >> 4) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio33(&self) -> gpio33::R {
            crate::BitReader::new(((self.bits() >> 5) & 1) != 0)
        }
    }
    impl W {
        #[inline(always)]
        pub fn gpio28(&mut self) -> gpio28::W<'_> {
            gpio28::W::new(self)
        }
        #[inline(always)]
        pub fn gpio29(&mut self) -> gpio29::W<'_> {
            gpio29::W::new(self)
        }
        #[inline(always)]
        pub fn gpio30(&mut self) -> gpio30::W<'_> {
            gpio30::W::new(self)
        }
        #[inline(always)]
        pub fn gpio31(&mut self) -> gpio31::W<'_> {
            gpio31::W::new(self)
        }
        #[inline(always)]
        pub fn gpio32(&mut self) -> gpio32::W<'_> {
            gpio32::W::new(self)
        }
        #[inline(always)]
        pub fn gpio33(&mut self) -> gpio33::W<'_> {
            gpio33::W::new(self)
        }
    }
    impl AtomicSetW {
        /// GPIO28 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio28(&mut self) -> gpio28::ASW<'_> {
            gpio28::ASW::new(self)
        }
        /// GPIO29 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio29(&mut self) -> gpio29::ASW<'_> {
            gpio29::ASW::new(self)
        }
        /// GPIO30 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio30(&mut self) -> gpio30::ASW<'_> {
            gpio30::ASW::new(self)
        }
        /// GPIO31 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio31(&mut self) -> gpio31::ASW<'_> {
            gpio31::ASW::new(self)
        }
        /// GPIO32 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio32(&mut self) -> gpio32::ASW<'_> {
            gpio32::ASW::new(self)
        }
        /// GPIO33 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio33(&mut self) -> gpio33::ASW<'_> {
            gpio33::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// GPIO28 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio28(&mut self) -> gpio28::ACW<'_> {
            gpio28::ACW::new(self)
        }
        /// GPIO29 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio29(&mut self) -> gpio29::ACW<'_> {
            gpio29::ACW::new(self)
        }
        /// GPIO30 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio30(&mut self) -> gpio30::ACW<'_> {
            gpio30::ACW::new(self)
        }
        /// GPIO31 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio31(&mut self) -> gpio31::ACW<'_> {
            gpio31::ACW::new(self)
        }
        /// GPIO32 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio32(&mut self) -> gpio32::ACW<'_> {
            gpio32::ACW::new(self)
        }
        /// GPIO33 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio33(&mut self) -> gpio33::ACW<'_> {
            gpio33::ACW::new(self)
        }
    }
    impl AtomicXorW {
        /// GPIO28 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio28(&mut self) -> gpio28::AXW<'_> {
            gpio28::AXW::new(self)
        }
        /// GPIO29 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio29(&mut self) -> gpio29::AXW<'_> {
            gpio29::AXW::new(self)
        }
        /// GPIO30 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio30(&mut self) -> gpio30::AXW<'_> {
            gpio30::AXW::new(self)
        }
        /// GPIO31 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio31(&mut self) -> gpio31::AXW<'_> {
            gpio31::AXW::new(self)
        }
        /// GPIO32 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio32(&mut self) -> gpio32::AXW<'_> {
            gpio32::AXW::new(self)
        }
        /// GPIO33 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio33(&mut self) -> gpio33::AXW<'_> {
            gpio33::AXW::new(self)
        }
    }
}

/// RIO_OE controls GPIO output-enable drive
/// Hardware access: read Plain; write Plain.
pub type OE = Reg<oe::OE_SPEC>;
/// RIO_OE controls GPIO output-enable drive
/// Hardware access: read Plain; write Plain.
pub mod oe {
    pub struct OE_SPEC;
    impl crate::RegisterSpec for OE_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for OE_SPEC {}
    pub type R = crate::R<OE_SPEC>;
    impl crate::Modifiable for OE_SPEC {}
    impl crate::UnsafeWritable for OE_SPEC {}
    impl crate::TypedAtomicSet for OE_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicSet for OE_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::TypedAtomicClear for OE_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicClear for OE_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::TypedAtomicXor for OE_SPEC {
        const OFFSET: usize = 0x1000;
    }
    impl crate::UnsafeAtomicXor for OE_SPEC {
        const OFFSET: usize = 0x1000;
    }
    pub type W = crate::W<OE_SPEC>;
    pub type AtomicSetW = crate::AtomicSetW<OE_SPEC>;
    pub type AtomicClearW = crate::AtomicClearW<OE_SPEC>;
    pub type AtomicXorW = crate::AtomicXorW<OE_SPEC>;
    /// GPIO28 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio28 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 0>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 0>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 0>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 0>;
    }
    /// GPIO29 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio29 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 1>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 1>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 1>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 1>;
    }
    /// GPIO30 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio30 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 2>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 2>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 2>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 2>;
    }
    /// GPIO31 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio31 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 3>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 3>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 3>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 3>;
    }
    /// GPIO32 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio32 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 4>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 4>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 4>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 4>;
    }
    /// GPIO33 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio33 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 5>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 5>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 5>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 5>;
    }
    impl R {
        #[inline(always)]
        pub fn gpio28(&self) -> gpio28::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio29(&self) -> gpio29::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio30(&self) -> gpio30::R {
            crate::BitReader::new(((self.bits() >> 2) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio31(&self) -> gpio31::R {
            crate::BitReader::new(((self.bits() >> 3) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio32(&self) -> gpio32::R {
            crate::BitReader::new(((self.bits() >> 4) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio33(&self) -> gpio33::R {
            crate::BitReader::new(((self.bits() >> 5) & 1) != 0)
        }
    }
    impl W {
        #[inline(always)]
        pub fn gpio28(&mut self) -> gpio28::W<'_> {
            gpio28::W::new(self)
        }
        #[inline(always)]
        pub fn gpio29(&mut self) -> gpio29::W<'_> {
            gpio29::W::new(self)
        }
        #[inline(always)]
        pub fn gpio30(&mut self) -> gpio30::W<'_> {
            gpio30::W::new(self)
        }
        #[inline(always)]
        pub fn gpio31(&mut self) -> gpio31::W<'_> {
            gpio31::W::new(self)
        }
        #[inline(always)]
        pub fn gpio32(&mut self) -> gpio32::W<'_> {
            gpio32::W::new(self)
        }
        #[inline(always)]
        pub fn gpio33(&mut self) -> gpio33::W<'_> {
            gpio33::W::new(self)
        }
    }
    impl AtomicSetW {
        /// GPIO28 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio28(&mut self) -> gpio28::ASW<'_> {
            gpio28::ASW::new(self)
        }
        /// GPIO29 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio29(&mut self) -> gpio29::ASW<'_> {
            gpio29::ASW::new(self)
        }
        /// GPIO30 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio30(&mut self) -> gpio30::ASW<'_> {
            gpio30::ASW::new(self)
        }
        /// GPIO31 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio31(&mut self) -> gpio31::ASW<'_> {
            gpio31::ASW::new(self)
        }
        /// GPIO32 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio32(&mut self) -> gpio32::ASW<'_> {
            gpio32::ASW::new(self)
        }
        /// GPIO33 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio33(&mut self) -> gpio33::ASW<'_> {
            gpio33::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// GPIO28 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio28(&mut self) -> gpio28::ACW<'_> {
            gpio28::ACW::new(self)
        }
        /// GPIO29 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio29(&mut self) -> gpio29::ACW<'_> {
            gpio29::ACW::new(self)
        }
        /// GPIO30 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio30(&mut self) -> gpio30::ACW<'_> {
            gpio30::ACW::new(self)
        }
        /// GPIO31 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio31(&mut self) -> gpio31::ACW<'_> {
            gpio31::ACW::new(self)
        }
        /// GPIO32 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio32(&mut self) -> gpio32::ACW<'_> {
            gpio32::ACW::new(self)
        }
        /// GPIO33 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio33(&mut self) -> gpio33::ACW<'_> {
            gpio33::ACW::new(self)
        }
    }
    impl AtomicXorW {
        /// GPIO28 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio28(&mut self) -> gpio28::AXW<'_> {
            gpio28::AXW::new(self)
        }
        /// GPIO29 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio29(&mut self) -> gpio29::AXW<'_> {
            gpio29::AXW::new(self)
        }
        /// GPIO30 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio30(&mut self) -> gpio30::AXW<'_> {
            gpio30::AXW::new(self)
        }
        /// GPIO31 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio31(&mut self) -> gpio31::AXW<'_> {
            gpio31::AXW::new(self)
        }
        /// GPIO32 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio32(&mut self) -> gpio32::AXW<'_> {
            gpio32::AXW::new(self)
        }
        /// GPIO33 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio33(&mut self) -> gpio33::AXW<'_> {
            gpio33::AXW::new(self)
        }
    }
}

/// RIO_NOSYNC_IN samples GPIO inputs directly
/// Hardware access: read Plain; write None.
pub type NOSYNC_IN = Reg<nosync_in::NOSYNC_IN_SPEC>;
/// RIO_NOSYNC_IN samples GPIO inputs directly
/// Hardware access: read Plain; write None.
pub mod nosync_in {
    pub struct NOSYNC_IN_SPEC;
    impl crate::RegisterSpec for NOSYNC_IN_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for NOSYNC_IN_SPEC {}
    pub type R = crate::R<NOSYNC_IN_SPEC>;
    /// GPIO28 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio28 {
        pub type R = crate::BitReader;
    }
    /// GPIO29 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio29 {
        pub type R = crate::BitReader;
    }
    /// GPIO30 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio30 {
        pub type R = crate::BitReader;
    }
    /// GPIO31 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio31 {
        pub type R = crate::BitReader;
    }
    /// GPIO32 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio32 {
        pub type R = crate::BitReader;
    }
    /// GPIO33 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio33 {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn gpio28(&self) -> gpio28::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio29(&self) -> gpio29::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio30(&self) -> gpio30::R {
            crate::BitReader::new(((self.bits() >> 2) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio31(&self) -> gpio31::R {
            crate::BitReader::new(((self.bits() >> 3) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio32(&self) -> gpio32::R {
            crate::BitReader::new(((self.bits() >> 4) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio33(&self) -> gpio33::R {
            crate::BitReader::new(((self.bits() >> 5) & 1) != 0)
        }
    }
}

/// RIO_SYNC_IN samples GPIO inputs synchronised with clk_sys
/// Hardware access: read Plain; write None.
pub type SYNC_IN = Reg<sync_in::SYNC_IN_SPEC>;
/// RIO_SYNC_IN samples GPIO inputs synchronised with clk_sys
/// Hardware access: read Plain; write None.
pub mod sync_in {
    pub struct SYNC_IN_SPEC;
    impl crate::RegisterSpec for SYNC_IN_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for SYNC_IN_SPEC {}
    pub type R = crate::R<SYNC_IN_SPEC>;
    /// GPIO28 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio28 {
        pub type R = crate::BitReader;
    }
    /// GPIO29 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio29 {
        pub type R = crate::BitReader;
    }
    /// GPIO30 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio30 {
        pub type R = crate::BitReader;
    }
    /// GPIO31 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio31 {
        pub type R = crate::BitReader;
    }
    /// GPIO32 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio32 {
        pub type R = crate::BitReader;
    }
    /// GPIO33 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio33 {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn gpio28(&self) -> gpio28::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio29(&self) -> gpio29::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio30(&self) -> gpio30::R {
            crate::BitReader::new(((self.bits() >> 2) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio31(&self) -> gpio31::R {
            crate::BitReader::new(((self.bits() >> 3) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio32(&self) -> gpio32::R {
            crate::BitReader::new(((self.bits() >> 4) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio33(&self) -> gpio33::R {
            crate::BitReader::new(((self.bits() >> 5) & 1) != 0)
        }
    }
}

#[cfg(test)]
mod layout_tests {
    use super::RegisterBlock;
    #[test]
    fn out_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, out), 0x0);
    }
    #[test]
    fn oe_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, oe), 0x4);
    }
    #[test]
    fn nosync_in_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, nosync_in), 0x8);
    }
    #[test]
    fn sync_in_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, sync_in), 0xc);
    }
    #[test]
    fn register_block_size() {
        assert_eq!(core::mem::size_of::<RegisterBlock>(), 0x10);
    }
}
