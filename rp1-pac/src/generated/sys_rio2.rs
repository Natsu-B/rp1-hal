// Generated from rp1-regdb/data/peripherals/sys-rio2.toml.
// Do not edit manually.
// Regenerate with: cargo run -p rp1-regdb -- generate

use crate::Reg;

/// RP1 SYS_RIO2 register block with supplemental register semantics. Alias behavior: normal read/write access. Bank geometry is recovered from the Linux pinctrl driver; register fields are inherited from the public bank-0 layout. Unknown offsets remain split RAW arrays.
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
    /// GPIO34 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio34 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 0>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 0>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 0>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 0>;
    }
    /// GPIO35 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio35 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 1>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 1>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 1>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 1>;
    }
    /// GPIO36 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio36 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 2>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 2>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 2>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 2>;
    }
    /// GPIO37 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio37 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 3>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 3>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 3>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 3>;
    }
    /// GPIO38 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio38 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 4>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 4>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 4>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 4>;
    }
    /// GPIO39 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio39 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 5>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 5>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 5>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 5>;
    }
    /// GPIO40 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio40 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 6>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 6>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 6>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 6>;
    }
    /// GPIO41 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio41 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 7>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 7>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 7>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 7>;
    }
    /// GPIO42 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio42 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 8>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 8>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 8>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 8>;
    }
    /// GPIO43 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio43 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 9>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 9>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 9>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 9>;
    }
    /// GPIO44 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio44 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 10>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 10>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 10>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 10>;
    }
    /// GPIO45 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio45 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 11>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 11>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 11>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 11>;
    }
    /// GPIO46 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio46 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 12>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 12>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 12>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 12>;
    }
    /// GPIO47 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio47 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 13>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 13>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 13>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 13>;
    }
    /// GPIO48 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio48 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 14>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 14>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 14>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 14>;
    }
    /// GPIO49 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio49 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 15>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 15>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 15>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 15>;
    }
    /// GPIO50 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio50 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 16>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 16>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 16>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 16>;
    }
    /// GPIO51 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio51 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 17>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 17>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 17>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 17>;
    }
    /// GPIO52 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio52 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 18>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 18>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 18>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 18>;
    }
    /// GPIO53 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio53 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 19>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 19>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 19>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 19>;
    }
    impl R {
        #[inline(always)]
        pub fn gpio34(&self) -> gpio34::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio35(&self) -> gpio35::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio36(&self) -> gpio36::R {
            crate::BitReader::new(((self.bits() >> 2) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio37(&self) -> gpio37::R {
            crate::BitReader::new(((self.bits() >> 3) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio38(&self) -> gpio38::R {
            crate::BitReader::new(((self.bits() >> 4) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio39(&self) -> gpio39::R {
            crate::BitReader::new(((self.bits() >> 5) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio40(&self) -> gpio40::R {
            crate::BitReader::new(((self.bits() >> 6) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio41(&self) -> gpio41::R {
            crate::BitReader::new(((self.bits() >> 7) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio42(&self) -> gpio42::R {
            crate::BitReader::new(((self.bits() >> 8) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio43(&self) -> gpio43::R {
            crate::BitReader::new(((self.bits() >> 9) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio44(&self) -> gpio44::R {
            crate::BitReader::new(((self.bits() >> 10) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio45(&self) -> gpio45::R {
            crate::BitReader::new(((self.bits() >> 11) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio46(&self) -> gpio46::R {
            crate::BitReader::new(((self.bits() >> 12) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio47(&self) -> gpio47::R {
            crate::BitReader::new(((self.bits() >> 13) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio48(&self) -> gpio48::R {
            crate::BitReader::new(((self.bits() >> 14) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio49(&self) -> gpio49::R {
            crate::BitReader::new(((self.bits() >> 15) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio50(&self) -> gpio50::R {
            crate::BitReader::new(((self.bits() >> 16) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio51(&self) -> gpio51::R {
            crate::BitReader::new(((self.bits() >> 17) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio52(&self) -> gpio52::R {
            crate::BitReader::new(((self.bits() >> 18) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio53(&self) -> gpio53::R {
            crate::BitReader::new(((self.bits() >> 19) & 1) != 0)
        }
    }
    impl W {
        #[inline(always)]
        pub fn gpio34(&mut self) -> gpio34::W<'_> {
            gpio34::W::new(self)
        }
        #[inline(always)]
        pub fn gpio35(&mut self) -> gpio35::W<'_> {
            gpio35::W::new(self)
        }
        #[inline(always)]
        pub fn gpio36(&mut self) -> gpio36::W<'_> {
            gpio36::W::new(self)
        }
        #[inline(always)]
        pub fn gpio37(&mut self) -> gpio37::W<'_> {
            gpio37::W::new(self)
        }
        #[inline(always)]
        pub fn gpio38(&mut self) -> gpio38::W<'_> {
            gpio38::W::new(self)
        }
        #[inline(always)]
        pub fn gpio39(&mut self) -> gpio39::W<'_> {
            gpio39::W::new(self)
        }
        #[inline(always)]
        pub fn gpio40(&mut self) -> gpio40::W<'_> {
            gpio40::W::new(self)
        }
        #[inline(always)]
        pub fn gpio41(&mut self) -> gpio41::W<'_> {
            gpio41::W::new(self)
        }
        #[inline(always)]
        pub fn gpio42(&mut self) -> gpio42::W<'_> {
            gpio42::W::new(self)
        }
        #[inline(always)]
        pub fn gpio43(&mut self) -> gpio43::W<'_> {
            gpio43::W::new(self)
        }
        #[inline(always)]
        pub fn gpio44(&mut self) -> gpio44::W<'_> {
            gpio44::W::new(self)
        }
        #[inline(always)]
        pub fn gpio45(&mut self) -> gpio45::W<'_> {
            gpio45::W::new(self)
        }
        #[inline(always)]
        pub fn gpio46(&mut self) -> gpio46::W<'_> {
            gpio46::W::new(self)
        }
        #[inline(always)]
        pub fn gpio47(&mut self) -> gpio47::W<'_> {
            gpio47::W::new(self)
        }
        #[inline(always)]
        pub fn gpio48(&mut self) -> gpio48::W<'_> {
            gpio48::W::new(self)
        }
        #[inline(always)]
        pub fn gpio49(&mut self) -> gpio49::W<'_> {
            gpio49::W::new(self)
        }
        #[inline(always)]
        pub fn gpio50(&mut self) -> gpio50::W<'_> {
            gpio50::W::new(self)
        }
        #[inline(always)]
        pub fn gpio51(&mut self) -> gpio51::W<'_> {
            gpio51::W::new(self)
        }
        #[inline(always)]
        pub fn gpio52(&mut self) -> gpio52::W<'_> {
            gpio52::W::new(self)
        }
        #[inline(always)]
        pub fn gpio53(&mut self) -> gpio53::W<'_> {
            gpio53::W::new(self)
        }
    }
    impl AtomicSetW {
        /// GPIO34 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio34(&mut self) -> gpio34::ASW<'_> {
            gpio34::ASW::new(self)
        }
        /// GPIO35 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio35(&mut self) -> gpio35::ASW<'_> {
            gpio35::ASW::new(self)
        }
        /// GPIO36 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio36(&mut self) -> gpio36::ASW<'_> {
            gpio36::ASW::new(self)
        }
        /// GPIO37 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio37(&mut self) -> gpio37::ASW<'_> {
            gpio37::ASW::new(self)
        }
        /// GPIO38 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio38(&mut self) -> gpio38::ASW<'_> {
            gpio38::ASW::new(self)
        }
        /// GPIO39 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio39(&mut self) -> gpio39::ASW<'_> {
            gpio39::ASW::new(self)
        }
        /// GPIO40 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio40(&mut self) -> gpio40::ASW<'_> {
            gpio40::ASW::new(self)
        }
        /// GPIO41 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio41(&mut self) -> gpio41::ASW<'_> {
            gpio41::ASW::new(self)
        }
        /// GPIO42 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio42(&mut self) -> gpio42::ASW<'_> {
            gpio42::ASW::new(self)
        }
        /// GPIO43 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio43(&mut self) -> gpio43::ASW<'_> {
            gpio43::ASW::new(self)
        }
        /// GPIO44 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio44(&mut self) -> gpio44::ASW<'_> {
            gpio44::ASW::new(self)
        }
        /// GPIO45 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio45(&mut self) -> gpio45::ASW<'_> {
            gpio45::ASW::new(self)
        }
        /// GPIO46 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio46(&mut self) -> gpio46::ASW<'_> {
            gpio46::ASW::new(self)
        }
        /// GPIO47 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio47(&mut self) -> gpio47::ASW<'_> {
            gpio47::ASW::new(self)
        }
        /// GPIO48 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio48(&mut self) -> gpio48::ASW<'_> {
            gpio48::ASW::new(self)
        }
        /// GPIO49 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio49(&mut self) -> gpio49::ASW<'_> {
            gpio49::ASW::new(self)
        }
        /// GPIO50 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio50(&mut self) -> gpio50::ASW<'_> {
            gpio50::ASW::new(self)
        }
        /// GPIO51 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio51(&mut self) -> gpio51::ASW<'_> {
            gpio51::ASW::new(self)
        }
        /// GPIO52 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio52(&mut self) -> gpio52::ASW<'_> {
            gpio52::ASW::new(self)
        }
        /// GPIO53 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio53(&mut self) -> gpio53::ASW<'_> {
            gpio53::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// GPIO34 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio34(&mut self) -> gpio34::ACW<'_> {
            gpio34::ACW::new(self)
        }
        /// GPIO35 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio35(&mut self) -> gpio35::ACW<'_> {
            gpio35::ACW::new(self)
        }
        /// GPIO36 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio36(&mut self) -> gpio36::ACW<'_> {
            gpio36::ACW::new(self)
        }
        /// GPIO37 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio37(&mut self) -> gpio37::ACW<'_> {
            gpio37::ACW::new(self)
        }
        /// GPIO38 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio38(&mut self) -> gpio38::ACW<'_> {
            gpio38::ACW::new(self)
        }
        /// GPIO39 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio39(&mut self) -> gpio39::ACW<'_> {
            gpio39::ACW::new(self)
        }
        /// GPIO40 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio40(&mut self) -> gpio40::ACW<'_> {
            gpio40::ACW::new(self)
        }
        /// GPIO41 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio41(&mut self) -> gpio41::ACW<'_> {
            gpio41::ACW::new(self)
        }
        /// GPIO42 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio42(&mut self) -> gpio42::ACW<'_> {
            gpio42::ACW::new(self)
        }
        /// GPIO43 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio43(&mut self) -> gpio43::ACW<'_> {
            gpio43::ACW::new(self)
        }
        /// GPIO44 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio44(&mut self) -> gpio44::ACW<'_> {
            gpio44::ACW::new(self)
        }
        /// GPIO45 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio45(&mut self) -> gpio45::ACW<'_> {
            gpio45::ACW::new(self)
        }
        /// GPIO46 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio46(&mut self) -> gpio46::ACW<'_> {
            gpio46::ACW::new(self)
        }
        /// GPIO47 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio47(&mut self) -> gpio47::ACW<'_> {
            gpio47::ACW::new(self)
        }
        /// GPIO48 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio48(&mut self) -> gpio48::ACW<'_> {
            gpio48::ACW::new(self)
        }
        /// GPIO49 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio49(&mut self) -> gpio49::ACW<'_> {
            gpio49::ACW::new(self)
        }
        /// GPIO50 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio50(&mut self) -> gpio50::ACW<'_> {
            gpio50::ACW::new(self)
        }
        /// GPIO51 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio51(&mut self) -> gpio51::ACW<'_> {
            gpio51::ACW::new(self)
        }
        /// GPIO52 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio52(&mut self) -> gpio52::ACW<'_> {
            gpio52::ACW::new(self)
        }
        /// GPIO53 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio53(&mut self) -> gpio53::ACW<'_> {
            gpio53::ACW::new(self)
        }
    }
    impl AtomicXorW {
        /// GPIO34 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio34(&mut self) -> gpio34::AXW<'_> {
            gpio34::AXW::new(self)
        }
        /// GPIO35 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio35(&mut self) -> gpio35::AXW<'_> {
            gpio35::AXW::new(self)
        }
        /// GPIO36 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio36(&mut self) -> gpio36::AXW<'_> {
            gpio36::AXW::new(self)
        }
        /// GPIO37 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio37(&mut self) -> gpio37::AXW<'_> {
            gpio37::AXW::new(self)
        }
        /// GPIO38 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio38(&mut self) -> gpio38::AXW<'_> {
            gpio38::AXW::new(self)
        }
        /// GPIO39 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio39(&mut self) -> gpio39::AXW<'_> {
            gpio39::AXW::new(self)
        }
        /// GPIO40 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio40(&mut self) -> gpio40::AXW<'_> {
            gpio40::AXW::new(self)
        }
        /// GPIO41 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio41(&mut self) -> gpio41::AXW<'_> {
            gpio41::AXW::new(self)
        }
        /// GPIO42 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio42(&mut self) -> gpio42::AXW<'_> {
            gpio42::AXW::new(self)
        }
        /// GPIO43 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio43(&mut self) -> gpio43::AXW<'_> {
            gpio43::AXW::new(self)
        }
        /// GPIO44 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio44(&mut self) -> gpio44::AXW<'_> {
            gpio44::AXW::new(self)
        }
        /// GPIO45 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio45(&mut self) -> gpio45::AXW<'_> {
            gpio45::AXW::new(self)
        }
        /// GPIO46 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio46(&mut self) -> gpio46::AXW<'_> {
            gpio46::AXW::new(self)
        }
        /// GPIO47 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio47(&mut self) -> gpio47::AXW<'_> {
            gpio47::AXW::new(self)
        }
        /// GPIO48 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio48(&mut self) -> gpio48::AXW<'_> {
            gpio48::AXW::new(self)
        }
        /// GPIO49 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio49(&mut self) -> gpio49::AXW<'_> {
            gpio49::AXW::new(self)
        }
        /// GPIO50 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio50(&mut self) -> gpio50::AXW<'_> {
            gpio50::AXW::new(self)
        }
        /// GPIO51 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio51(&mut self) -> gpio51::AXW<'_> {
            gpio51::AXW::new(self)
        }
        /// GPIO52 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio52(&mut self) -> gpio52::AXW<'_> {
            gpio52::AXW::new(self)
        }
        /// GPIO53 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio53(&mut self) -> gpio53::AXW<'_> {
            gpio53::AXW::new(self)
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
    /// GPIO34 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio34 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 0>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 0>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 0>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 0>;
    }
    /// GPIO35 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio35 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 1>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 1>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 1>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 1>;
    }
    /// GPIO36 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio36 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 2>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 2>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 2>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 2>;
    }
    /// GPIO37 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio37 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 3>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 3>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 3>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 3>;
    }
    /// GPIO38 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio38 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 4>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 4>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 4>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 4>;
    }
    /// GPIO39 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio39 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 5>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 5>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 5>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 5>;
    }
    /// GPIO40 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio40 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 6>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 6>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 6>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 6>;
    }
    /// GPIO41 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio41 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 7>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 7>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 7>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 7>;
    }
    /// GPIO42 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio42 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 8>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 8>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 8>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 8>;
    }
    /// GPIO43 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio43 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 9>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 9>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 9>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 9>;
    }
    /// GPIO44 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio44 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 10>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 10>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 10>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 10>;
    }
    /// GPIO45 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio45 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 11>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 11>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 11>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 11>;
    }
    /// GPIO46 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio46 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 12>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 12>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 12>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 12>;
    }
    /// GPIO47 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio47 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 13>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 13>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 13>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 13>;
    }
    /// GPIO48 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio48 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 14>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 14>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 14>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 14>;
    }
    /// GPIO49 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio49 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 15>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 15>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 15>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 15>;
    }
    /// GPIO50 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio50 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 16>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 16>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 16>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 16>;
    }
    /// GPIO51 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio51 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 17>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 17>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 17>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 17>;
    }
    /// GPIO52 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio52 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 18>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 18>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 18>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 18>;
    }
    /// GPIO53 bank-local bit.
    /// Field write behavior: Plain.
    pub mod gpio53 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 19>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 19>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 19>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 19>;
    }
    impl R {
        #[inline(always)]
        pub fn gpio34(&self) -> gpio34::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio35(&self) -> gpio35::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio36(&self) -> gpio36::R {
            crate::BitReader::new(((self.bits() >> 2) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio37(&self) -> gpio37::R {
            crate::BitReader::new(((self.bits() >> 3) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio38(&self) -> gpio38::R {
            crate::BitReader::new(((self.bits() >> 4) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio39(&self) -> gpio39::R {
            crate::BitReader::new(((self.bits() >> 5) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio40(&self) -> gpio40::R {
            crate::BitReader::new(((self.bits() >> 6) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio41(&self) -> gpio41::R {
            crate::BitReader::new(((self.bits() >> 7) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio42(&self) -> gpio42::R {
            crate::BitReader::new(((self.bits() >> 8) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio43(&self) -> gpio43::R {
            crate::BitReader::new(((self.bits() >> 9) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio44(&self) -> gpio44::R {
            crate::BitReader::new(((self.bits() >> 10) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio45(&self) -> gpio45::R {
            crate::BitReader::new(((self.bits() >> 11) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio46(&self) -> gpio46::R {
            crate::BitReader::new(((self.bits() >> 12) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio47(&self) -> gpio47::R {
            crate::BitReader::new(((self.bits() >> 13) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio48(&self) -> gpio48::R {
            crate::BitReader::new(((self.bits() >> 14) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio49(&self) -> gpio49::R {
            crate::BitReader::new(((self.bits() >> 15) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio50(&self) -> gpio50::R {
            crate::BitReader::new(((self.bits() >> 16) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio51(&self) -> gpio51::R {
            crate::BitReader::new(((self.bits() >> 17) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio52(&self) -> gpio52::R {
            crate::BitReader::new(((self.bits() >> 18) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio53(&self) -> gpio53::R {
            crate::BitReader::new(((self.bits() >> 19) & 1) != 0)
        }
    }
    impl W {
        #[inline(always)]
        pub fn gpio34(&mut self) -> gpio34::W<'_> {
            gpio34::W::new(self)
        }
        #[inline(always)]
        pub fn gpio35(&mut self) -> gpio35::W<'_> {
            gpio35::W::new(self)
        }
        #[inline(always)]
        pub fn gpio36(&mut self) -> gpio36::W<'_> {
            gpio36::W::new(self)
        }
        #[inline(always)]
        pub fn gpio37(&mut self) -> gpio37::W<'_> {
            gpio37::W::new(self)
        }
        #[inline(always)]
        pub fn gpio38(&mut self) -> gpio38::W<'_> {
            gpio38::W::new(self)
        }
        #[inline(always)]
        pub fn gpio39(&mut self) -> gpio39::W<'_> {
            gpio39::W::new(self)
        }
        #[inline(always)]
        pub fn gpio40(&mut self) -> gpio40::W<'_> {
            gpio40::W::new(self)
        }
        #[inline(always)]
        pub fn gpio41(&mut self) -> gpio41::W<'_> {
            gpio41::W::new(self)
        }
        #[inline(always)]
        pub fn gpio42(&mut self) -> gpio42::W<'_> {
            gpio42::W::new(self)
        }
        #[inline(always)]
        pub fn gpio43(&mut self) -> gpio43::W<'_> {
            gpio43::W::new(self)
        }
        #[inline(always)]
        pub fn gpio44(&mut self) -> gpio44::W<'_> {
            gpio44::W::new(self)
        }
        #[inline(always)]
        pub fn gpio45(&mut self) -> gpio45::W<'_> {
            gpio45::W::new(self)
        }
        #[inline(always)]
        pub fn gpio46(&mut self) -> gpio46::W<'_> {
            gpio46::W::new(self)
        }
        #[inline(always)]
        pub fn gpio47(&mut self) -> gpio47::W<'_> {
            gpio47::W::new(self)
        }
        #[inline(always)]
        pub fn gpio48(&mut self) -> gpio48::W<'_> {
            gpio48::W::new(self)
        }
        #[inline(always)]
        pub fn gpio49(&mut self) -> gpio49::W<'_> {
            gpio49::W::new(self)
        }
        #[inline(always)]
        pub fn gpio50(&mut self) -> gpio50::W<'_> {
            gpio50::W::new(self)
        }
        #[inline(always)]
        pub fn gpio51(&mut self) -> gpio51::W<'_> {
            gpio51::W::new(self)
        }
        #[inline(always)]
        pub fn gpio52(&mut self) -> gpio52::W<'_> {
            gpio52::W::new(self)
        }
        #[inline(always)]
        pub fn gpio53(&mut self) -> gpio53::W<'_> {
            gpio53::W::new(self)
        }
    }
    impl AtomicSetW {
        /// GPIO34 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio34(&mut self) -> gpio34::ASW<'_> {
            gpio34::ASW::new(self)
        }
        /// GPIO35 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio35(&mut self) -> gpio35::ASW<'_> {
            gpio35::ASW::new(self)
        }
        /// GPIO36 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio36(&mut self) -> gpio36::ASW<'_> {
            gpio36::ASW::new(self)
        }
        /// GPIO37 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio37(&mut self) -> gpio37::ASW<'_> {
            gpio37::ASW::new(self)
        }
        /// GPIO38 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio38(&mut self) -> gpio38::ASW<'_> {
            gpio38::ASW::new(self)
        }
        /// GPIO39 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio39(&mut self) -> gpio39::ASW<'_> {
            gpio39::ASW::new(self)
        }
        /// GPIO40 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio40(&mut self) -> gpio40::ASW<'_> {
            gpio40::ASW::new(self)
        }
        /// GPIO41 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio41(&mut self) -> gpio41::ASW<'_> {
            gpio41::ASW::new(self)
        }
        /// GPIO42 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio42(&mut self) -> gpio42::ASW<'_> {
            gpio42::ASW::new(self)
        }
        /// GPIO43 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio43(&mut self) -> gpio43::ASW<'_> {
            gpio43::ASW::new(self)
        }
        /// GPIO44 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio44(&mut self) -> gpio44::ASW<'_> {
            gpio44::ASW::new(self)
        }
        /// GPIO45 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio45(&mut self) -> gpio45::ASW<'_> {
            gpio45::ASW::new(self)
        }
        /// GPIO46 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio46(&mut self) -> gpio46::ASW<'_> {
            gpio46::ASW::new(self)
        }
        /// GPIO47 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio47(&mut self) -> gpio47::ASW<'_> {
            gpio47::ASW::new(self)
        }
        /// GPIO48 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio48(&mut self) -> gpio48::ASW<'_> {
            gpio48::ASW::new(self)
        }
        /// GPIO49 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio49(&mut self) -> gpio49::ASW<'_> {
            gpio49::ASW::new(self)
        }
        /// GPIO50 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio50(&mut self) -> gpio50::ASW<'_> {
            gpio50::ASW::new(self)
        }
        /// GPIO51 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio51(&mut self) -> gpio51::ASW<'_> {
            gpio51::ASW::new(self)
        }
        /// GPIO52 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio52(&mut self) -> gpio52::ASW<'_> {
            gpio52::ASW::new(self)
        }
        /// GPIO53 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio53(&mut self) -> gpio53::ASW<'_> {
            gpio53::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// GPIO34 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio34(&mut self) -> gpio34::ACW<'_> {
            gpio34::ACW::new(self)
        }
        /// GPIO35 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio35(&mut self) -> gpio35::ACW<'_> {
            gpio35::ACW::new(self)
        }
        /// GPIO36 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio36(&mut self) -> gpio36::ACW<'_> {
            gpio36::ACW::new(self)
        }
        /// GPIO37 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio37(&mut self) -> gpio37::ACW<'_> {
            gpio37::ACW::new(self)
        }
        /// GPIO38 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio38(&mut self) -> gpio38::ACW<'_> {
            gpio38::ACW::new(self)
        }
        /// GPIO39 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio39(&mut self) -> gpio39::ACW<'_> {
            gpio39::ACW::new(self)
        }
        /// GPIO40 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio40(&mut self) -> gpio40::ACW<'_> {
            gpio40::ACW::new(self)
        }
        /// GPIO41 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio41(&mut self) -> gpio41::ACW<'_> {
            gpio41::ACW::new(self)
        }
        /// GPIO42 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio42(&mut self) -> gpio42::ACW<'_> {
            gpio42::ACW::new(self)
        }
        /// GPIO43 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio43(&mut self) -> gpio43::ACW<'_> {
            gpio43::ACW::new(self)
        }
        /// GPIO44 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio44(&mut self) -> gpio44::ACW<'_> {
            gpio44::ACW::new(self)
        }
        /// GPIO45 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio45(&mut self) -> gpio45::ACW<'_> {
            gpio45::ACW::new(self)
        }
        /// GPIO46 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio46(&mut self) -> gpio46::ACW<'_> {
            gpio46::ACW::new(self)
        }
        /// GPIO47 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio47(&mut self) -> gpio47::ACW<'_> {
            gpio47::ACW::new(self)
        }
        /// GPIO48 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio48(&mut self) -> gpio48::ACW<'_> {
            gpio48::ACW::new(self)
        }
        /// GPIO49 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio49(&mut self) -> gpio49::ACW<'_> {
            gpio49::ACW::new(self)
        }
        /// GPIO50 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio50(&mut self) -> gpio50::ACW<'_> {
            gpio50::ACW::new(self)
        }
        /// GPIO51 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio51(&mut self) -> gpio51::ACW<'_> {
            gpio51::ACW::new(self)
        }
        /// GPIO52 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio52(&mut self) -> gpio52::ACW<'_> {
            gpio52::ACW::new(self)
        }
        /// GPIO53 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio53(&mut self) -> gpio53::ACW<'_> {
            gpio53::ACW::new(self)
        }
    }
    impl AtomicXorW {
        /// GPIO34 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio34(&mut self) -> gpio34::AXW<'_> {
            gpio34::AXW::new(self)
        }
        /// GPIO35 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio35(&mut self) -> gpio35::AXW<'_> {
            gpio35::AXW::new(self)
        }
        /// GPIO36 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio36(&mut self) -> gpio36::AXW<'_> {
            gpio36::AXW::new(self)
        }
        /// GPIO37 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio37(&mut self) -> gpio37::AXW<'_> {
            gpio37::AXW::new(self)
        }
        /// GPIO38 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio38(&mut self) -> gpio38::AXW<'_> {
            gpio38::AXW::new(self)
        }
        /// GPIO39 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio39(&mut self) -> gpio39::AXW<'_> {
            gpio39::AXW::new(self)
        }
        /// GPIO40 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio40(&mut self) -> gpio40::AXW<'_> {
            gpio40::AXW::new(self)
        }
        /// GPIO41 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio41(&mut self) -> gpio41::AXW<'_> {
            gpio41::AXW::new(self)
        }
        /// GPIO42 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio42(&mut self) -> gpio42::AXW<'_> {
            gpio42::AXW::new(self)
        }
        /// GPIO43 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio43(&mut self) -> gpio43::AXW<'_> {
            gpio43::AXW::new(self)
        }
        /// GPIO44 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio44(&mut self) -> gpio44::AXW<'_> {
            gpio44::AXW::new(self)
        }
        /// GPIO45 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio45(&mut self) -> gpio45::AXW<'_> {
            gpio45::AXW::new(self)
        }
        /// GPIO46 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio46(&mut self) -> gpio46::AXW<'_> {
            gpio46::AXW::new(self)
        }
        /// GPIO47 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio47(&mut self) -> gpio47::AXW<'_> {
            gpio47::AXW::new(self)
        }
        /// GPIO48 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio48(&mut self) -> gpio48::AXW<'_> {
            gpio48::AXW::new(self)
        }
        /// GPIO49 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio49(&mut self) -> gpio49::AXW<'_> {
            gpio49::AXW::new(self)
        }
        /// GPIO50 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio50(&mut self) -> gpio50::AXW<'_> {
            gpio50::AXW::new(self)
        }
        /// GPIO51 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio51(&mut self) -> gpio51::AXW<'_> {
            gpio51::AXW::new(self)
        }
        /// GPIO52 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio52(&mut self) -> gpio52::AXW<'_> {
            gpio52::AXW::new(self)
        }
        /// GPIO53 bank-local bit.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio53(&mut self) -> gpio53::AXW<'_> {
            gpio53::AXW::new(self)
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
    /// GPIO34 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio34 {
        pub type R = crate::BitReader;
    }
    /// GPIO35 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio35 {
        pub type R = crate::BitReader;
    }
    /// GPIO36 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio36 {
        pub type R = crate::BitReader;
    }
    /// GPIO37 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio37 {
        pub type R = crate::BitReader;
    }
    /// GPIO38 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio38 {
        pub type R = crate::BitReader;
    }
    /// GPIO39 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio39 {
        pub type R = crate::BitReader;
    }
    /// GPIO40 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio40 {
        pub type R = crate::BitReader;
    }
    /// GPIO41 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio41 {
        pub type R = crate::BitReader;
    }
    /// GPIO42 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio42 {
        pub type R = crate::BitReader;
    }
    /// GPIO43 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio43 {
        pub type R = crate::BitReader;
    }
    /// GPIO44 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio44 {
        pub type R = crate::BitReader;
    }
    /// GPIO45 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio45 {
        pub type R = crate::BitReader;
    }
    /// GPIO46 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio46 {
        pub type R = crate::BitReader;
    }
    /// GPIO47 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio47 {
        pub type R = crate::BitReader;
    }
    /// GPIO48 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio48 {
        pub type R = crate::BitReader;
    }
    /// GPIO49 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio49 {
        pub type R = crate::BitReader;
    }
    /// GPIO50 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio50 {
        pub type R = crate::BitReader;
    }
    /// GPIO51 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio51 {
        pub type R = crate::BitReader;
    }
    /// GPIO52 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio52 {
        pub type R = crate::BitReader;
    }
    /// GPIO53 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio53 {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn gpio34(&self) -> gpio34::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio35(&self) -> gpio35::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio36(&self) -> gpio36::R {
            crate::BitReader::new(((self.bits() >> 2) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio37(&self) -> gpio37::R {
            crate::BitReader::new(((self.bits() >> 3) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio38(&self) -> gpio38::R {
            crate::BitReader::new(((self.bits() >> 4) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio39(&self) -> gpio39::R {
            crate::BitReader::new(((self.bits() >> 5) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio40(&self) -> gpio40::R {
            crate::BitReader::new(((self.bits() >> 6) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio41(&self) -> gpio41::R {
            crate::BitReader::new(((self.bits() >> 7) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio42(&self) -> gpio42::R {
            crate::BitReader::new(((self.bits() >> 8) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio43(&self) -> gpio43::R {
            crate::BitReader::new(((self.bits() >> 9) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio44(&self) -> gpio44::R {
            crate::BitReader::new(((self.bits() >> 10) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio45(&self) -> gpio45::R {
            crate::BitReader::new(((self.bits() >> 11) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio46(&self) -> gpio46::R {
            crate::BitReader::new(((self.bits() >> 12) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio47(&self) -> gpio47::R {
            crate::BitReader::new(((self.bits() >> 13) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio48(&self) -> gpio48::R {
            crate::BitReader::new(((self.bits() >> 14) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio49(&self) -> gpio49::R {
            crate::BitReader::new(((self.bits() >> 15) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio50(&self) -> gpio50::R {
            crate::BitReader::new(((self.bits() >> 16) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio51(&self) -> gpio51::R {
            crate::BitReader::new(((self.bits() >> 17) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio52(&self) -> gpio52::R {
            crate::BitReader::new(((self.bits() >> 18) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio53(&self) -> gpio53::R {
            crate::BitReader::new(((self.bits() >> 19) & 1) != 0)
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
    /// GPIO34 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio34 {
        pub type R = crate::BitReader;
    }
    /// GPIO35 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio35 {
        pub type R = crate::BitReader;
    }
    /// GPIO36 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio36 {
        pub type R = crate::BitReader;
    }
    /// GPIO37 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio37 {
        pub type R = crate::BitReader;
    }
    /// GPIO38 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio38 {
        pub type R = crate::BitReader;
    }
    /// GPIO39 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio39 {
        pub type R = crate::BitReader;
    }
    /// GPIO40 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio40 {
        pub type R = crate::BitReader;
    }
    /// GPIO41 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio41 {
        pub type R = crate::BitReader;
    }
    /// GPIO42 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio42 {
        pub type R = crate::BitReader;
    }
    /// GPIO43 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio43 {
        pub type R = crate::BitReader;
    }
    /// GPIO44 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio44 {
        pub type R = crate::BitReader;
    }
    /// GPIO45 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio45 {
        pub type R = crate::BitReader;
    }
    /// GPIO46 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio46 {
        pub type R = crate::BitReader;
    }
    /// GPIO47 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio47 {
        pub type R = crate::BitReader;
    }
    /// GPIO48 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio48 {
        pub type R = crate::BitReader;
    }
    /// GPIO49 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio49 {
        pub type R = crate::BitReader;
    }
    /// GPIO50 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio50 {
        pub type R = crate::BitReader;
    }
    /// GPIO51 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio51 {
        pub type R = crate::BitReader;
    }
    /// GPIO52 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio52 {
        pub type R = crate::BitReader;
    }
    /// GPIO53 bank-local bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio53 {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn gpio34(&self) -> gpio34::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio35(&self) -> gpio35::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio36(&self) -> gpio36::R {
            crate::BitReader::new(((self.bits() >> 2) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio37(&self) -> gpio37::R {
            crate::BitReader::new(((self.bits() >> 3) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio38(&self) -> gpio38::R {
            crate::BitReader::new(((self.bits() >> 4) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio39(&self) -> gpio39::R {
            crate::BitReader::new(((self.bits() >> 5) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio40(&self) -> gpio40::R {
            crate::BitReader::new(((self.bits() >> 6) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio41(&self) -> gpio41::R {
            crate::BitReader::new(((self.bits() >> 7) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio42(&self) -> gpio42::R {
            crate::BitReader::new(((self.bits() >> 8) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio43(&self) -> gpio43::R {
            crate::BitReader::new(((self.bits() >> 9) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio44(&self) -> gpio44::R {
            crate::BitReader::new(((self.bits() >> 10) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio45(&self) -> gpio45::R {
            crate::BitReader::new(((self.bits() >> 11) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio46(&self) -> gpio46::R {
            crate::BitReader::new(((self.bits() >> 12) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio47(&self) -> gpio47::R {
            crate::BitReader::new(((self.bits() >> 13) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio48(&self) -> gpio48::R {
            crate::BitReader::new(((self.bits() >> 14) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio49(&self) -> gpio49::R {
            crate::BitReader::new(((self.bits() >> 15) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio50(&self) -> gpio50::R {
            crate::BitReader::new(((self.bits() >> 16) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio51(&self) -> gpio51::R {
            crate::BitReader::new(((self.bits() >> 17) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio52(&self) -> gpio52::R {
            crate::BitReader::new(((self.bits() >> 18) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio53(&self) -> gpio53::R {
            crate::BitReader::new(((self.bits() >> 19) & 1) != 0)
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
