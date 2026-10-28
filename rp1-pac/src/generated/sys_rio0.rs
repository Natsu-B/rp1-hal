// Generated from rp1-regdb/data/peripherals/sys-rio0.toml.
// Do not edit manually.
// Regenerate with: cargo run -p rp1-regdb -- generate

use crate::Reg;

/// RP1 SYS_RIO0_RW register block. Bus=APB. Atomic alias: normal read/write access.
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
    impl crate::ResetValue for OUT_SPEC {
        const RESET_VALUE: u32 = 0x0;
    }
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
    /// GPIO0
    /// Field write behavior: Plain.
    pub mod gpio0 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 0>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 0>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 0>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 0>;
    }
    /// GPIO1
    /// Field write behavior: Plain.
    pub mod gpio1 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 1>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 1>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 1>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 1>;
    }
    /// GPIO2
    /// Field write behavior: Plain.
    pub mod gpio2 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 2>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 2>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 2>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 2>;
    }
    /// GPIO3
    /// Field write behavior: Plain.
    pub mod gpio3 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 3>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 3>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 3>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 3>;
    }
    /// GPIO4
    /// Field write behavior: Plain.
    pub mod gpio4 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 4>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 4>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 4>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 4>;
    }
    /// GPIO5
    /// Field write behavior: Plain.
    pub mod gpio5 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 5>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 5>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 5>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 5>;
    }
    /// GPIO6
    /// Field write behavior: Plain.
    pub mod gpio6 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 6>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 6>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 6>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 6>;
    }
    /// GPIO7
    /// Field write behavior: Plain.
    pub mod gpio7 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 7>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 7>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 7>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 7>;
    }
    /// GPIO8
    /// Field write behavior: Plain.
    pub mod gpio8 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 8>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 8>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 8>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 8>;
    }
    /// GPIO9
    /// Field write behavior: Plain.
    pub mod gpio9 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 9>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 9>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 9>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 9>;
    }
    /// GPIO10
    /// Field write behavior: Plain.
    pub mod gpio10 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 10>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 10>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 10>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 10>;
    }
    /// GPIO11
    /// Field write behavior: Plain.
    pub mod gpio11 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 11>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 11>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 11>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 11>;
    }
    /// GPIO12
    /// Field write behavior: Plain.
    pub mod gpio12 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 12>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 12>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 12>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 12>;
    }
    /// GPIO13
    /// Field write behavior: Plain.
    pub mod gpio13 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 13>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 13>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 13>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 13>;
    }
    /// GPIO14
    /// Field write behavior: Plain.
    pub mod gpio14 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 14>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 14>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 14>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 14>;
    }
    /// GPIO15
    /// Field write behavior: Plain.
    pub mod gpio15 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 15>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 15>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 15>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 15>;
    }
    /// GPIO16
    /// Field write behavior: Plain.
    pub mod gpio16 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 16>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 16>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 16>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 16>;
    }
    /// GPIO17
    /// Field write behavior: Plain.
    pub mod gpio17 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 17>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 17>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 17>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 17>;
    }
    /// GPIO18
    /// Field write behavior: Plain.
    pub mod gpio18 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 18>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 18>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 18>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 18>;
    }
    /// GPIO19
    /// Field write behavior: Plain.
    pub mod gpio19 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 19>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 19>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 19>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 19>;
    }
    /// GPIO20
    /// Field write behavior: Plain.
    pub mod gpio20 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 20>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 20>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 20>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 20>;
    }
    /// GPIO21
    /// Field write behavior: Plain.
    pub mod gpio21 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 21>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 21>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 21>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 21>;
    }
    /// GPIO22
    /// Field write behavior: Plain.
    pub mod gpio22 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 22>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 22>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 22>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 22>;
    }
    /// GPIO23
    /// Field write behavior: Plain.
    pub mod gpio23 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 23>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 23>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 23>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 23>;
    }
    /// GPIO24
    /// Field write behavior: Plain.
    pub mod gpio24 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 24>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 24>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 24>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 24>;
    }
    /// GPIO25
    /// Field write behavior: Plain.
    pub mod gpio25 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 25>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 25>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 25>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 25>;
    }
    /// GPIO26
    /// Field write behavior: Plain.
    pub mod gpio26 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 26>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 26>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 26>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 26>;
    }
    /// GPIO27
    /// Field write behavior: Plain.
    pub mod gpio27 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 27>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 27>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 27>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 27>;
    }
    /// GPIO28
    /// Field write behavior: Plain.
    pub mod gpio28 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 28>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 28>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 28>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 28>;
    }
    /// GPIO29
    /// Field write behavior: Plain.
    pub mod gpio29 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 29>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 29>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 29>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 29>;
    }
    /// GPIO30
    /// Field write behavior: Plain.
    pub mod gpio30 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 30>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 30>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 30>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 30>;
    }
    /// GPIO31
    /// Field write behavior: Plain.
    pub mod gpio31 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 31>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 31>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 31>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 31>;
    }
    impl R {
        #[inline(always)]
        pub fn gpio0(&self) -> gpio0::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio1(&self) -> gpio1::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio2(&self) -> gpio2::R {
            crate::BitReader::new(((self.bits() >> 2) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio3(&self) -> gpio3::R {
            crate::BitReader::new(((self.bits() >> 3) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio4(&self) -> gpio4::R {
            crate::BitReader::new(((self.bits() >> 4) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio5(&self) -> gpio5::R {
            crate::BitReader::new(((self.bits() >> 5) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio6(&self) -> gpio6::R {
            crate::BitReader::new(((self.bits() >> 6) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio7(&self) -> gpio7::R {
            crate::BitReader::new(((self.bits() >> 7) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio8(&self) -> gpio8::R {
            crate::BitReader::new(((self.bits() >> 8) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio9(&self) -> gpio9::R {
            crate::BitReader::new(((self.bits() >> 9) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio10(&self) -> gpio10::R {
            crate::BitReader::new(((self.bits() >> 10) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio11(&self) -> gpio11::R {
            crate::BitReader::new(((self.bits() >> 11) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio12(&self) -> gpio12::R {
            crate::BitReader::new(((self.bits() >> 12) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio13(&self) -> gpio13::R {
            crate::BitReader::new(((self.bits() >> 13) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio14(&self) -> gpio14::R {
            crate::BitReader::new(((self.bits() >> 14) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio15(&self) -> gpio15::R {
            crate::BitReader::new(((self.bits() >> 15) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio16(&self) -> gpio16::R {
            crate::BitReader::new(((self.bits() >> 16) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio17(&self) -> gpio17::R {
            crate::BitReader::new(((self.bits() >> 17) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio18(&self) -> gpio18::R {
            crate::BitReader::new(((self.bits() >> 18) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio19(&self) -> gpio19::R {
            crate::BitReader::new(((self.bits() >> 19) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio20(&self) -> gpio20::R {
            crate::BitReader::new(((self.bits() >> 20) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio21(&self) -> gpio21::R {
            crate::BitReader::new(((self.bits() >> 21) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio22(&self) -> gpio22::R {
            crate::BitReader::new(((self.bits() >> 22) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio23(&self) -> gpio23::R {
            crate::BitReader::new(((self.bits() >> 23) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio24(&self) -> gpio24::R {
            crate::BitReader::new(((self.bits() >> 24) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio25(&self) -> gpio25::R {
            crate::BitReader::new(((self.bits() >> 25) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio26(&self) -> gpio26::R {
            crate::BitReader::new(((self.bits() >> 26) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio27(&self) -> gpio27::R {
            crate::BitReader::new(((self.bits() >> 27) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio28(&self) -> gpio28::R {
            crate::BitReader::new(((self.bits() >> 28) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio29(&self) -> gpio29::R {
            crate::BitReader::new(((self.bits() >> 29) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio30(&self) -> gpio30::R {
            crate::BitReader::new(((self.bits() >> 30) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio31(&self) -> gpio31::R {
            crate::BitReader::new(((self.bits() >> 31) & 1) != 0)
        }
    }
    impl W {
        #[inline(always)]
        pub fn gpio0(&mut self) -> gpio0::W<'_> {
            gpio0::W::new(self)
        }
        #[inline(always)]
        pub fn gpio1(&mut self) -> gpio1::W<'_> {
            gpio1::W::new(self)
        }
        #[inline(always)]
        pub fn gpio2(&mut self) -> gpio2::W<'_> {
            gpio2::W::new(self)
        }
        #[inline(always)]
        pub fn gpio3(&mut self) -> gpio3::W<'_> {
            gpio3::W::new(self)
        }
        #[inline(always)]
        pub fn gpio4(&mut self) -> gpio4::W<'_> {
            gpio4::W::new(self)
        }
        #[inline(always)]
        pub fn gpio5(&mut self) -> gpio5::W<'_> {
            gpio5::W::new(self)
        }
        #[inline(always)]
        pub fn gpio6(&mut self) -> gpio6::W<'_> {
            gpio6::W::new(self)
        }
        #[inline(always)]
        pub fn gpio7(&mut self) -> gpio7::W<'_> {
            gpio7::W::new(self)
        }
        #[inline(always)]
        pub fn gpio8(&mut self) -> gpio8::W<'_> {
            gpio8::W::new(self)
        }
        #[inline(always)]
        pub fn gpio9(&mut self) -> gpio9::W<'_> {
            gpio9::W::new(self)
        }
        #[inline(always)]
        pub fn gpio10(&mut self) -> gpio10::W<'_> {
            gpio10::W::new(self)
        }
        #[inline(always)]
        pub fn gpio11(&mut self) -> gpio11::W<'_> {
            gpio11::W::new(self)
        }
        #[inline(always)]
        pub fn gpio12(&mut self) -> gpio12::W<'_> {
            gpio12::W::new(self)
        }
        #[inline(always)]
        pub fn gpio13(&mut self) -> gpio13::W<'_> {
            gpio13::W::new(self)
        }
        #[inline(always)]
        pub fn gpio14(&mut self) -> gpio14::W<'_> {
            gpio14::W::new(self)
        }
        #[inline(always)]
        pub fn gpio15(&mut self) -> gpio15::W<'_> {
            gpio15::W::new(self)
        }
        #[inline(always)]
        pub fn gpio16(&mut self) -> gpio16::W<'_> {
            gpio16::W::new(self)
        }
        #[inline(always)]
        pub fn gpio17(&mut self) -> gpio17::W<'_> {
            gpio17::W::new(self)
        }
        #[inline(always)]
        pub fn gpio18(&mut self) -> gpio18::W<'_> {
            gpio18::W::new(self)
        }
        #[inline(always)]
        pub fn gpio19(&mut self) -> gpio19::W<'_> {
            gpio19::W::new(self)
        }
        #[inline(always)]
        pub fn gpio20(&mut self) -> gpio20::W<'_> {
            gpio20::W::new(self)
        }
        #[inline(always)]
        pub fn gpio21(&mut self) -> gpio21::W<'_> {
            gpio21::W::new(self)
        }
        #[inline(always)]
        pub fn gpio22(&mut self) -> gpio22::W<'_> {
            gpio22::W::new(self)
        }
        #[inline(always)]
        pub fn gpio23(&mut self) -> gpio23::W<'_> {
            gpio23::W::new(self)
        }
        #[inline(always)]
        pub fn gpio24(&mut self) -> gpio24::W<'_> {
            gpio24::W::new(self)
        }
        #[inline(always)]
        pub fn gpio25(&mut self) -> gpio25::W<'_> {
            gpio25::W::new(self)
        }
        #[inline(always)]
        pub fn gpio26(&mut self) -> gpio26::W<'_> {
            gpio26::W::new(self)
        }
        #[inline(always)]
        pub fn gpio27(&mut self) -> gpio27::W<'_> {
            gpio27::W::new(self)
        }
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
    }
    impl AtomicSetW {
        /// GPIO0
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio0(&mut self) -> gpio0::ASW<'_> {
            gpio0::ASW::new(self)
        }
        /// GPIO1
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio1(&mut self) -> gpio1::ASW<'_> {
            gpio1::ASW::new(self)
        }
        /// GPIO2
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio2(&mut self) -> gpio2::ASW<'_> {
            gpio2::ASW::new(self)
        }
        /// GPIO3
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio3(&mut self) -> gpio3::ASW<'_> {
            gpio3::ASW::new(self)
        }
        /// GPIO4
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio4(&mut self) -> gpio4::ASW<'_> {
            gpio4::ASW::new(self)
        }
        /// GPIO5
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio5(&mut self) -> gpio5::ASW<'_> {
            gpio5::ASW::new(self)
        }
        /// GPIO6
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio6(&mut self) -> gpio6::ASW<'_> {
            gpio6::ASW::new(self)
        }
        /// GPIO7
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio7(&mut self) -> gpio7::ASW<'_> {
            gpio7::ASW::new(self)
        }
        /// GPIO8
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio8(&mut self) -> gpio8::ASW<'_> {
            gpio8::ASW::new(self)
        }
        /// GPIO9
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio9(&mut self) -> gpio9::ASW<'_> {
            gpio9::ASW::new(self)
        }
        /// GPIO10
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio10(&mut self) -> gpio10::ASW<'_> {
            gpio10::ASW::new(self)
        }
        /// GPIO11
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio11(&mut self) -> gpio11::ASW<'_> {
            gpio11::ASW::new(self)
        }
        /// GPIO12
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio12(&mut self) -> gpio12::ASW<'_> {
            gpio12::ASW::new(self)
        }
        /// GPIO13
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio13(&mut self) -> gpio13::ASW<'_> {
            gpio13::ASW::new(self)
        }
        /// GPIO14
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio14(&mut self) -> gpio14::ASW<'_> {
            gpio14::ASW::new(self)
        }
        /// GPIO15
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio15(&mut self) -> gpio15::ASW<'_> {
            gpio15::ASW::new(self)
        }
        /// GPIO16
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio16(&mut self) -> gpio16::ASW<'_> {
            gpio16::ASW::new(self)
        }
        /// GPIO17
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio17(&mut self) -> gpio17::ASW<'_> {
            gpio17::ASW::new(self)
        }
        /// GPIO18
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio18(&mut self) -> gpio18::ASW<'_> {
            gpio18::ASW::new(self)
        }
        /// GPIO19
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio19(&mut self) -> gpio19::ASW<'_> {
            gpio19::ASW::new(self)
        }
        /// GPIO20
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio20(&mut self) -> gpio20::ASW<'_> {
            gpio20::ASW::new(self)
        }
        /// GPIO21
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio21(&mut self) -> gpio21::ASW<'_> {
            gpio21::ASW::new(self)
        }
        /// GPIO22
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio22(&mut self) -> gpio22::ASW<'_> {
            gpio22::ASW::new(self)
        }
        /// GPIO23
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio23(&mut self) -> gpio23::ASW<'_> {
            gpio23::ASW::new(self)
        }
        /// GPIO24
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio24(&mut self) -> gpio24::ASW<'_> {
            gpio24::ASW::new(self)
        }
        /// GPIO25
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio25(&mut self) -> gpio25::ASW<'_> {
            gpio25::ASW::new(self)
        }
        /// GPIO26
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio26(&mut self) -> gpio26::ASW<'_> {
            gpio26::ASW::new(self)
        }
        /// GPIO27
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio27(&mut self) -> gpio27::ASW<'_> {
            gpio27::ASW::new(self)
        }
        /// GPIO28
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio28(&mut self) -> gpio28::ASW<'_> {
            gpio28::ASW::new(self)
        }
        /// GPIO29
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio29(&mut self) -> gpio29::ASW<'_> {
            gpio29::ASW::new(self)
        }
        /// GPIO30
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio30(&mut self) -> gpio30::ASW<'_> {
            gpio30::ASW::new(self)
        }
        /// GPIO31
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio31(&mut self) -> gpio31::ASW<'_> {
            gpio31::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// GPIO0
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio0(&mut self) -> gpio0::ACW<'_> {
            gpio0::ACW::new(self)
        }
        /// GPIO1
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio1(&mut self) -> gpio1::ACW<'_> {
            gpio1::ACW::new(self)
        }
        /// GPIO2
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio2(&mut self) -> gpio2::ACW<'_> {
            gpio2::ACW::new(self)
        }
        /// GPIO3
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio3(&mut self) -> gpio3::ACW<'_> {
            gpio3::ACW::new(self)
        }
        /// GPIO4
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio4(&mut self) -> gpio4::ACW<'_> {
            gpio4::ACW::new(self)
        }
        /// GPIO5
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio5(&mut self) -> gpio5::ACW<'_> {
            gpio5::ACW::new(self)
        }
        /// GPIO6
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio6(&mut self) -> gpio6::ACW<'_> {
            gpio6::ACW::new(self)
        }
        /// GPIO7
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio7(&mut self) -> gpio7::ACW<'_> {
            gpio7::ACW::new(self)
        }
        /// GPIO8
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio8(&mut self) -> gpio8::ACW<'_> {
            gpio8::ACW::new(self)
        }
        /// GPIO9
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio9(&mut self) -> gpio9::ACW<'_> {
            gpio9::ACW::new(self)
        }
        /// GPIO10
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio10(&mut self) -> gpio10::ACW<'_> {
            gpio10::ACW::new(self)
        }
        /// GPIO11
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio11(&mut self) -> gpio11::ACW<'_> {
            gpio11::ACW::new(self)
        }
        /// GPIO12
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio12(&mut self) -> gpio12::ACW<'_> {
            gpio12::ACW::new(self)
        }
        /// GPIO13
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio13(&mut self) -> gpio13::ACW<'_> {
            gpio13::ACW::new(self)
        }
        /// GPIO14
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio14(&mut self) -> gpio14::ACW<'_> {
            gpio14::ACW::new(self)
        }
        /// GPIO15
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio15(&mut self) -> gpio15::ACW<'_> {
            gpio15::ACW::new(self)
        }
        /// GPIO16
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio16(&mut self) -> gpio16::ACW<'_> {
            gpio16::ACW::new(self)
        }
        /// GPIO17
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio17(&mut self) -> gpio17::ACW<'_> {
            gpio17::ACW::new(self)
        }
        /// GPIO18
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio18(&mut self) -> gpio18::ACW<'_> {
            gpio18::ACW::new(self)
        }
        /// GPIO19
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio19(&mut self) -> gpio19::ACW<'_> {
            gpio19::ACW::new(self)
        }
        /// GPIO20
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio20(&mut self) -> gpio20::ACW<'_> {
            gpio20::ACW::new(self)
        }
        /// GPIO21
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio21(&mut self) -> gpio21::ACW<'_> {
            gpio21::ACW::new(self)
        }
        /// GPIO22
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio22(&mut self) -> gpio22::ACW<'_> {
            gpio22::ACW::new(self)
        }
        /// GPIO23
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio23(&mut self) -> gpio23::ACW<'_> {
            gpio23::ACW::new(self)
        }
        /// GPIO24
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio24(&mut self) -> gpio24::ACW<'_> {
            gpio24::ACW::new(self)
        }
        /// GPIO25
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio25(&mut self) -> gpio25::ACW<'_> {
            gpio25::ACW::new(self)
        }
        /// GPIO26
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio26(&mut self) -> gpio26::ACW<'_> {
            gpio26::ACW::new(self)
        }
        /// GPIO27
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio27(&mut self) -> gpio27::ACW<'_> {
            gpio27::ACW::new(self)
        }
        /// GPIO28
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio28(&mut self) -> gpio28::ACW<'_> {
            gpio28::ACW::new(self)
        }
        /// GPIO29
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio29(&mut self) -> gpio29::ACW<'_> {
            gpio29::ACW::new(self)
        }
        /// GPIO30
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio30(&mut self) -> gpio30::ACW<'_> {
            gpio30::ACW::new(self)
        }
        /// GPIO31
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio31(&mut self) -> gpio31::ACW<'_> {
            gpio31::ACW::new(self)
        }
    }
    impl AtomicXorW {
        /// GPIO0
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio0(&mut self) -> gpio0::AXW<'_> {
            gpio0::AXW::new(self)
        }
        /// GPIO1
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio1(&mut self) -> gpio1::AXW<'_> {
            gpio1::AXW::new(self)
        }
        /// GPIO2
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio2(&mut self) -> gpio2::AXW<'_> {
            gpio2::AXW::new(self)
        }
        /// GPIO3
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio3(&mut self) -> gpio3::AXW<'_> {
            gpio3::AXW::new(self)
        }
        /// GPIO4
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio4(&mut self) -> gpio4::AXW<'_> {
            gpio4::AXW::new(self)
        }
        /// GPIO5
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio5(&mut self) -> gpio5::AXW<'_> {
            gpio5::AXW::new(self)
        }
        /// GPIO6
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio6(&mut self) -> gpio6::AXW<'_> {
            gpio6::AXW::new(self)
        }
        /// GPIO7
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio7(&mut self) -> gpio7::AXW<'_> {
            gpio7::AXW::new(self)
        }
        /// GPIO8
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio8(&mut self) -> gpio8::AXW<'_> {
            gpio8::AXW::new(self)
        }
        /// GPIO9
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio9(&mut self) -> gpio9::AXW<'_> {
            gpio9::AXW::new(self)
        }
        /// GPIO10
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio10(&mut self) -> gpio10::AXW<'_> {
            gpio10::AXW::new(self)
        }
        /// GPIO11
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio11(&mut self) -> gpio11::AXW<'_> {
            gpio11::AXW::new(self)
        }
        /// GPIO12
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio12(&mut self) -> gpio12::AXW<'_> {
            gpio12::AXW::new(self)
        }
        /// GPIO13
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio13(&mut self) -> gpio13::AXW<'_> {
            gpio13::AXW::new(self)
        }
        /// GPIO14
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio14(&mut self) -> gpio14::AXW<'_> {
            gpio14::AXW::new(self)
        }
        /// GPIO15
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio15(&mut self) -> gpio15::AXW<'_> {
            gpio15::AXW::new(self)
        }
        /// GPIO16
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio16(&mut self) -> gpio16::AXW<'_> {
            gpio16::AXW::new(self)
        }
        /// GPIO17
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio17(&mut self) -> gpio17::AXW<'_> {
            gpio17::AXW::new(self)
        }
        /// GPIO18
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio18(&mut self) -> gpio18::AXW<'_> {
            gpio18::AXW::new(self)
        }
        /// GPIO19
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio19(&mut self) -> gpio19::AXW<'_> {
            gpio19::AXW::new(self)
        }
        /// GPIO20
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio20(&mut self) -> gpio20::AXW<'_> {
            gpio20::AXW::new(self)
        }
        /// GPIO21
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio21(&mut self) -> gpio21::AXW<'_> {
            gpio21::AXW::new(self)
        }
        /// GPIO22
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio22(&mut self) -> gpio22::AXW<'_> {
            gpio22::AXW::new(self)
        }
        /// GPIO23
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio23(&mut self) -> gpio23::AXW<'_> {
            gpio23::AXW::new(self)
        }
        /// GPIO24
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio24(&mut self) -> gpio24::AXW<'_> {
            gpio24::AXW::new(self)
        }
        /// GPIO25
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio25(&mut self) -> gpio25::AXW<'_> {
            gpio25::AXW::new(self)
        }
        /// GPIO26
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio26(&mut self) -> gpio26::AXW<'_> {
            gpio26::AXW::new(self)
        }
        /// GPIO27
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio27(&mut self) -> gpio27::AXW<'_> {
            gpio27::AXW::new(self)
        }
        /// GPIO28
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio28(&mut self) -> gpio28::AXW<'_> {
            gpio28::AXW::new(self)
        }
        /// GPIO29
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio29(&mut self) -> gpio29::AXW<'_> {
            gpio29::AXW::new(self)
        }
        /// GPIO30
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio30(&mut self) -> gpio30::AXW<'_> {
            gpio30::AXW::new(self)
        }
        /// GPIO31
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio31(&mut self) -> gpio31::AXW<'_> {
            gpio31::AXW::new(self)
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
    impl crate::ResetValue for OE_SPEC {
        const RESET_VALUE: u32 = 0x0;
    }
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
    /// GPIO0
    /// Field write behavior: Plain.
    pub mod gpio0 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 0>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 0>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 0>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 0>;
    }
    /// GPIO1
    /// Field write behavior: Plain.
    pub mod gpio1 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 1>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 1>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 1>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 1>;
    }
    /// GPIO2
    /// Field write behavior: Plain.
    pub mod gpio2 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 2>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 2>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 2>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 2>;
    }
    /// GPIO3
    /// Field write behavior: Plain.
    pub mod gpio3 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 3>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 3>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 3>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 3>;
    }
    /// GPIO4
    /// Field write behavior: Plain.
    pub mod gpio4 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 4>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 4>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 4>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 4>;
    }
    /// GPIO5
    /// Field write behavior: Plain.
    pub mod gpio5 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 5>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 5>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 5>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 5>;
    }
    /// GPIO6
    /// Field write behavior: Plain.
    pub mod gpio6 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 6>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 6>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 6>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 6>;
    }
    /// GPIO7
    /// Field write behavior: Plain.
    pub mod gpio7 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 7>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 7>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 7>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 7>;
    }
    /// GPIO8
    /// Field write behavior: Plain.
    pub mod gpio8 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 8>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 8>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 8>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 8>;
    }
    /// GPIO9
    /// Field write behavior: Plain.
    pub mod gpio9 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 9>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 9>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 9>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 9>;
    }
    /// GPIO10
    /// Field write behavior: Plain.
    pub mod gpio10 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 10>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 10>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 10>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 10>;
    }
    /// GPIO11
    /// Field write behavior: Plain.
    pub mod gpio11 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 11>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 11>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 11>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 11>;
    }
    /// GPIO12
    /// Field write behavior: Plain.
    pub mod gpio12 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 12>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 12>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 12>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 12>;
    }
    /// GPIO13
    /// Field write behavior: Plain.
    pub mod gpio13 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 13>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 13>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 13>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 13>;
    }
    /// GPIO14
    /// Field write behavior: Plain.
    pub mod gpio14 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 14>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 14>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 14>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 14>;
    }
    /// GPIO15
    /// Field write behavior: Plain.
    pub mod gpio15 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 15>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 15>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 15>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 15>;
    }
    /// GPIO16
    /// Field write behavior: Plain.
    pub mod gpio16 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 16>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 16>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 16>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 16>;
    }
    /// GPIO17
    /// Field write behavior: Plain.
    pub mod gpio17 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 17>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 17>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 17>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 17>;
    }
    /// GPIO18
    /// Field write behavior: Plain.
    pub mod gpio18 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 18>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 18>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 18>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 18>;
    }
    /// GPIO19
    /// Field write behavior: Plain.
    pub mod gpio19 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 19>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 19>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 19>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 19>;
    }
    /// GPIO20
    /// Field write behavior: Plain.
    pub mod gpio20 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 20>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 20>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 20>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 20>;
    }
    /// GPIO21
    /// Field write behavior: Plain.
    pub mod gpio21 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 21>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 21>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 21>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 21>;
    }
    /// GPIO22
    /// Field write behavior: Plain.
    pub mod gpio22 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 22>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 22>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 22>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 22>;
    }
    /// GPIO23
    /// Field write behavior: Plain.
    pub mod gpio23 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 23>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 23>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 23>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 23>;
    }
    /// GPIO24
    /// Field write behavior: Plain.
    pub mod gpio24 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 24>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 24>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 24>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 24>;
    }
    /// GPIO25
    /// Field write behavior: Plain.
    pub mod gpio25 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 25>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 25>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 25>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 25>;
    }
    /// GPIO26
    /// Field write behavior: Plain.
    pub mod gpio26 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 26>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 26>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 26>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 26>;
    }
    /// GPIO27
    /// Field write behavior: Plain.
    pub mod gpio27 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 27>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 27>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 27>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 27>;
    }
    /// GPIO28
    /// Field write behavior: Plain.
    pub mod gpio28 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 28>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 28>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 28>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 28>;
    }
    /// GPIO29
    /// Field write behavior: Plain.
    pub mod gpio29 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 29>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 29>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 29>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 29>;
    }
    /// GPIO30
    /// Field write behavior: Plain.
    pub mod gpio30 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 30>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 30>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 30>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 30>;
    }
    /// GPIO31
    /// Field write behavior: Plain.
    pub mod gpio31 {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 31>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 31>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 31>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 31>;
    }
    impl R {
        #[inline(always)]
        pub fn gpio0(&self) -> gpio0::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio1(&self) -> gpio1::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio2(&self) -> gpio2::R {
            crate::BitReader::new(((self.bits() >> 2) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio3(&self) -> gpio3::R {
            crate::BitReader::new(((self.bits() >> 3) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio4(&self) -> gpio4::R {
            crate::BitReader::new(((self.bits() >> 4) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio5(&self) -> gpio5::R {
            crate::BitReader::new(((self.bits() >> 5) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio6(&self) -> gpio6::R {
            crate::BitReader::new(((self.bits() >> 6) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio7(&self) -> gpio7::R {
            crate::BitReader::new(((self.bits() >> 7) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio8(&self) -> gpio8::R {
            crate::BitReader::new(((self.bits() >> 8) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio9(&self) -> gpio9::R {
            crate::BitReader::new(((self.bits() >> 9) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio10(&self) -> gpio10::R {
            crate::BitReader::new(((self.bits() >> 10) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio11(&self) -> gpio11::R {
            crate::BitReader::new(((self.bits() >> 11) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio12(&self) -> gpio12::R {
            crate::BitReader::new(((self.bits() >> 12) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio13(&self) -> gpio13::R {
            crate::BitReader::new(((self.bits() >> 13) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio14(&self) -> gpio14::R {
            crate::BitReader::new(((self.bits() >> 14) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio15(&self) -> gpio15::R {
            crate::BitReader::new(((self.bits() >> 15) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio16(&self) -> gpio16::R {
            crate::BitReader::new(((self.bits() >> 16) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio17(&self) -> gpio17::R {
            crate::BitReader::new(((self.bits() >> 17) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio18(&self) -> gpio18::R {
            crate::BitReader::new(((self.bits() >> 18) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio19(&self) -> gpio19::R {
            crate::BitReader::new(((self.bits() >> 19) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio20(&self) -> gpio20::R {
            crate::BitReader::new(((self.bits() >> 20) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio21(&self) -> gpio21::R {
            crate::BitReader::new(((self.bits() >> 21) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio22(&self) -> gpio22::R {
            crate::BitReader::new(((self.bits() >> 22) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio23(&self) -> gpio23::R {
            crate::BitReader::new(((self.bits() >> 23) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio24(&self) -> gpio24::R {
            crate::BitReader::new(((self.bits() >> 24) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio25(&self) -> gpio25::R {
            crate::BitReader::new(((self.bits() >> 25) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio26(&self) -> gpio26::R {
            crate::BitReader::new(((self.bits() >> 26) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio27(&self) -> gpio27::R {
            crate::BitReader::new(((self.bits() >> 27) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio28(&self) -> gpio28::R {
            crate::BitReader::new(((self.bits() >> 28) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio29(&self) -> gpio29::R {
            crate::BitReader::new(((self.bits() >> 29) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio30(&self) -> gpio30::R {
            crate::BitReader::new(((self.bits() >> 30) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio31(&self) -> gpio31::R {
            crate::BitReader::new(((self.bits() >> 31) & 1) != 0)
        }
    }
    impl W {
        #[inline(always)]
        pub fn gpio0(&mut self) -> gpio0::W<'_> {
            gpio0::W::new(self)
        }
        #[inline(always)]
        pub fn gpio1(&mut self) -> gpio1::W<'_> {
            gpio1::W::new(self)
        }
        #[inline(always)]
        pub fn gpio2(&mut self) -> gpio2::W<'_> {
            gpio2::W::new(self)
        }
        #[inline(always)]
        pub fn gpio3(&mut self) -> gpio3::W<'_> {
            gpio3::W::new(self)
        }
        #[inline(always)]
        pub fn gpio4(&mut self) -> gpio4::W<'_> {
            gpio4::W::new(self)
        }
        #[inline(always)]
        pub fn gpio5(&mut self) -> gpio5::W<'_> {
            gpio5::W::new(self)
        }
        #[inline(always)]
        pub fn gpio6(&mut self) -> gpio6::W<'_> {
            gpio6::W::new(self)
        }
        #[inline(always)]
        pub fn gpio7(&mut self) -> gpio7::W<'_> {
            gpio7::W::new(self)
        }
        #[inline(always)]
        pub fn gpio8(&mut self) -> gpio8::W<'_> {
            gpio8::W::new(self)
        }
        #[inline(always)]
        pub fn gpio9(&mut self) -> gpio9::W<'_> {
            gpio9::W::new(self)
        }
        #[inline(always)]
        pub fn gpio10(&mut self) -> gpio10::W<'_> {
            gpio10::W::new(self)
        }
        #[inline(always)]
        pub fn gpio11(&mut self) -> gpio11::W<'_> {
            gpio11::W::new(self)
        }
        #[inline(always)]
        pub fn gpio12(&mut self) -> gpio12::W<'_> {
            gpio12::W::new(self)
        }
        #[inline(always)]
        pub fn gpio13(&mut self) -> gpio13::W<'_> {
            gpio13::W::new(self)
        }
        #[inline(always)]
        pub fn gpio14(&mut self) -> gpio14::W<'_> {
            gpio14::W::new(self)
        }
        #[inline(always)]
        pub fn gpio15(&mut self) -> gpio15::W<'_> {
            gpio15::W::new(self)
        }
        #[inline(always)]
        pub fn gpio16(&mut self) -> gpio16::W<'_> {
            gpio16::W::new(self)
        }
        #[inline(always)]
        pub fn gpio17(&mut self) -> gpio17::W<'_> {
            gpio17::W::new(self)
        }
        #[inline(always)]
        pub fn gpio18(&mut self) -> gpio18::W<'_> {
            gpio18::W::new(self)
        }
        #[inline(always)]
        pub fn gpio19(&mut self) -> gpio19::W<'_> {
            gpio19::W::new(self)
        }
        #[inline(always)]
        pub fn gpio20(&mut self) -> gpio20::W<'_> {
            gpio20::W::new(self)
        }
        #[inline(always)]
        pub fn gpio21(&mut self) -> gpio21::W<'_> {
            gpio21::W::new(self)
        }
        #[inline(always)]
        pub fn gpio22(&mut self) -> gpio22::W<'_> {
            gpio22::W::new(self)
        }
        #[inline(always)]
        pub fn gpio23(&mut self) -> gpio23::W<'_> {
            gpio23::W::new(self)
        }
        #[inline(always)]
        pub fn gpio24(&mut self) -> gpio24::W<'_> {
            gpio24::W::new(self)
        }
        #[inline(always)]
        pub fn gpio25(&mut self) -> gpio25::W<'_> {
            gpio25::W::new(self)
        }
        #[inline(always)]
        pub fn gpio26(&mut self) -> gpio26::W<'_> {
            gpio26::W::new(self)
        }
        #[inline(always)]
        pub fn gpio27(&mut self) -> gpio27::W<'_> {
            gpio27::W::new(self)
        }
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
    }
    impl AtomicSetW {
        /// GPIO0
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio0(&mut self) -> gpio0::ASW<'_> {
            gpio0::ASW::new(self)
        }
        /// GPIO1
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio1(&mut self) -> gpio1::ASW<'_> {
            gpio1::ASW::new(self)
        }
        /// GPIO2
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio2(&mut self) -> gpio2::ASW<'_> {
            gpio2::ASW::new(self)
        }
        /// GPIO3
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio3(&mut self) -> gpio3::ASW<'_> {
            gpio3::ASW::new(self)
        }
        /// GPIO4
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio4(&mut self) -> gpio4::ASW<'_> {
            gpio4::ASW::new(self)
        }
        /// GPIO5
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio5(&mut self) -> gpio5::ASW<'_> {
            gpio5::ASW::new(self)
        }
        /// GPIO6
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio6(&mut self) -> gpio6::ASW<'_> {
            gpio6::ASW::new(self)
        }
        /// GPIO7
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio7(&mut self) -> gpio7::ASW<'_> {
            gpio7::ASW::new(self)
        }
        /// GPIO8
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio8(&mut self) -> gpio8::ASW<'_> {
            gpio8::ASW::new(self)
        }
        /// GPIO9
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio9(&mut self) -> gpio9::ASW<'_> {
            gpio9::ASW::new(self)
        }
        /// GPIO10
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio10(&mut self) -> gpio10::ASW<'_> {
            gpio10::ASW::new(self)
        }
        /// GPIO11
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio11(&mut self) -> gpio11::ASW<'_> {
            gpio11::ASW::new(self)
        }
        /// GPIO12
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio12(&mut self) -> gpio12::ASW<'_> {
            gpio12::ASW::new(self)
        }
        /// GPIO13
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio13(&mut self) -> gpio13::ASW<'_> {
            gpio13::ASW::new(self)
        }
        /// GPIO14
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio14(&mut self) -> gpio14::ASW<'_> {
            gpio14::ASW::new(self)
        }
        /// GPIO15
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio15(&mut self) -> gpio15::ASW<'_> {
            gpio15::ASW::new(self)
        }
        /// GPIO16
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio16(&mut self) -> gpio16::ASW<'_> {
            gpio16::ASW::new(self)
        }
        /// GPIO17
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio17(&mut self) -> gpio17::ASW<'_> {
            gpio17::ASW::new(self)
        }
        /// GPIO18
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio18(&mut self) -> gpio18::ASW<'_> {
            gpio18::ASW::new(self)
        }
        /// GPIO19
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio19(&mut self) -> gpio19::ASW<'_> {
            gpio19::ASW::new(self)
        }
        /// GPIO20
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio20(&mut self) -> gpio20::ASW<'_> {
            gpio20::ASW::new(self)
        }
        /// GPIO21
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio21(&mut self) -> gpio21::ASW<'_> {
            gpio21::ASW::new(self)
        }
        /// GPIO22
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio22(&mut self) -> gpio22::ASW<'_> {
            gpio22::ASW::new(self)
        }
        /// GPIO23
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio23(&mut self) -> gpio23::ASW<'_> {
            gpio23::ASW::new(self)
        }
        /// GPIO24
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio24(&mut self) -> gpio24::ASW<'_> {
            gpio24::ASW::new(self)
        }
        /// GPIO25
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio25(&mut self) -> gpio25::ASW<'_> {
            gpio25::ASW::new(self)
        }
        /// GPIO26
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio26(&mut self) -> gpio26::ASW<'_> {
            gpio26::ASW::new(self)
        }
        /// GPIO27
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio27(&mut self) -> gpio27::ASW<'_> {
            gpio27::ASW::new(self)
        }
        /// GPIO28
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio28(&mut self) -> gpio28::ASW<'_> {
            gpio28::ASW::new(self)
        }
        /// GPIO29
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio29(&mut self) -> gpio29::ASW<'_> {
            gpio29::ASW::new(self)
        }
        /// GPIO30
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio30(&mut self) -> gpio30::ASW<'_> {
            gpio30::ASW::new(self)
        }
        /// GPIO31
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio31(&mut self) -> gpio31::ASW<'_> {
            gpio31::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// GPIO0
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio0(&mut self) -> gpio0::ACW<'_> {
            gpio0::ACW::new(self)
        }
        /// GPIO1
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio1(&mut self) -> gpio1::ACW<'_> {
            gpio1::ACW::new(self)
        }
        /// GPIO2
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio2(&mut self) -> gpio2::ACW<'_> {
            gpio2::ACW::new(self)
        }
        /// GPIO3
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio3(&mut self) -> gpio3::ACW<'_> {
            gpio3::ACW::new(self)
        }
        /// GPIO4
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio4(&mut self) -> gpio4::ACW<'_> {
            gpio4::ACW::new(self)
        }
        /// GPIO5
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio5(&mut self) -> gpio5::ACW<'_> {
            gpio5::ACW::new(self)
        }
        /// GPIO6
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio6(&mut self) -> gpio6::ACW<'_> {
            gpio6::ACW::new(self)
        }
        /// GPIO7
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio7(&mut self) -> gpio7::ACW<'_> {
            gpio7::ACW::new(self)
        }
        /// GPIO8
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio8(&mut self) -> gpio8::ACW<'_> {
            gpio8::ACW::new(self)
        }
        /// GPIO9
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio9(&mut self) -> gpio9::ACW<'_> {
            gpio9::ACW::new(self)
        }
        /// GPIO10
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio10(&mut self) -> gpio10::ACW<'_> {
            gpio10::ACW::new(self)
        }
        /// GPIO11
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio11(&mut self) -> gpio11::ACW<'_> {
            gpio11::ACW::new(self)
        }
        /// GPIO12
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio12(&mut self) -> gpio12::ACW<'_> {
            gpio12::ACW::new(self)
        }
        /// GPIO13
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio13(&mut self) -> gpio13::ACW<'_> {
            gpio13::ACW::new(self)
        }
        /// GPIO14
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio14(&mut self) -> gpio14::ACW<'_> {
            gpio14::ACW::new(self)
        }
        /// GPIO15
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio15(&mut self) -> gpio15::ACW<'_> {
            gpio15::ACW::new(self)
        }
        /// GPIO16
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio16(&mut self) -> gpio16::ACW<'_> {
            gpio16::ACW::new(self)
        }
        /// GPIO17
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio17(&mut self) -> gpio17::ACW<'_> {
            gpio17::ACW::new(self)
        }
        /// GPIO18
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio18(&mut self) -> gpio18::ACW<'_> {
            gpio18::ACW::new(self)
        }
        /// GPIO19
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio19(&mut self) -> gpio19::ACW<'_> {
            gpio19::ACW::new(self)
        }
        /// GPIO20
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio20(&mut self) -> gpio20::ACW<'_> {
            gpio20::ACW::new(self)
        }
        /// GPIO21
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio21(&mut self) -> gpio21::ACW<'_> {
            gpio21::ACW::new(self)
        }
        /// GPIO22
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio22(&mut self) -> gpio22::ACW<'_> {
            gpio22::ACW::new(self)
        }
        /// GPIO23
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio23(&mut self) -> gpio23::ACW<'_> {
            gpio23::ACW::new(self)
        }
        /// GPIO24
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio24(&mut self) -> gpio24::ACW<'_> {
            gpio24::ACW::new(self)
        }
        /// GPIO25
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio25(&mut self) -> gpio25::ACW<'_> {
            gpio25::ACW::new(self)
        }
        /// GPIO26
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio26(&mut self) -> gpio26::ACW<'_> {
            gpio26::ACW::new(self)
        }
        /// GPIO27
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio27(&mut self) -> gpio27::ACW<'_> {
            gpio27::ACW::new(self)
        }
        /// GPIO28
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio28(&mut self) -> gpio28::ACW<'_> {
            gpio28::ACW::new(self)
        }
        /// GPIO29
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio29(&mut self) -> gpio29::ACW<'_> {
            gpio29::ACW::new(self)
        }
        /// GPIO30
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio30(&mut self) -> gpio30::ACW<'_> {
            gpio30::ACW::new(self)
        }
        /// GPIO31
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio31(&mut self) -> gpio31::ACW<'_> {
            gpio31::ACW::new(self)
        }
    }
    impl AtomicXorW {
        /// GPIO0
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio0(&mut self) -> gpio0::AXW<'_> {
            gpio0::AXW::new(self)
        }
        /// GPIO1
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio1(&mut self) -> gpio1::AXW<'_> {
            gpio1::AXW::new(self)
        }
        /// GPIO2
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio2(&mut self) -> gpio2::AXW<'_> {
            gpio2::AXW::new(self)
        }
        /// GPIO3
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio3(&mut self) -> gpio3::AXW<'_> {
            gpio3::AXW::new(self)
        }
        /// GPIO4
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio4(&mut self) -> gpio4::AXW<'_> {
            gpio4::AXW::new(self)
        }
        /// GPIO5
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio5(&mut self) -> gpio5::AXW<'_> {
            gpio5::AXW::new(self)
        }
        /// GPIO6
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio6(&mut self) -> gpio6::AXW<'_> {
            gpio6::AXW::new(self)
        }
        /// GPIO7
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio7(&mut self) -> gpio7::AXW<'_> {
            gpio7::AXW::new(self)
        }
        /// GPIO8
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio8(&mut self) -> gpio8::AXW<'_> {
            gpio8::AXW::new(self)
        }
        /// GPIO9
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio9(&mut self) -> gpio9::AXW<'_> {
            gpio9::AXW::new(self)
        }
        /// GPIO10
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio10(&mut self) -> gpio10::AXW<'_> {
            gpio10::AXW::new(self)
        }
        /// GPIO11
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio11(&mut self) -> gpio11::AXW<'_> {
            gpio11::AXW::new(self)
        }
        /// GPIO12
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio12(&mut self) -> gpio12::AXW<'_> {
            gpio12::AXW::new(self)
        }
        /// GPIO13
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio13(&mut self) -> gpio13::AXW<'_> {
            gpio13::AXW::new(self)
        }
        /// GPIO14
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio14(&mut self) -> gpio14::AXW<'_> {
            gpio14::AXW::new(self)
        }
        /// GPIO15
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio15(&mut self) -> gpio15::AXW<'_> {
            gpio15::AXW::new(self)
        }
        /// GPIO16
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio16(&mut self) -> gpio16::AXW<'_> {
            gpio16::AXW::new(self)
        }
        /// GPIO17
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio17(&mut self) -> gpio17::AXW<'_> {
            gpio17::AXW::new(self)
        }
        /// GPIO18
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio18(&mut self) -> gpio18::AXW<'_> {
            gpio18::AXW::new(self)
        }
        /// GPIO19
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio19(&mut self) -> gpio19::AXW<'_> {
            gpio19::AXW::new(self)
        }
        /// GPIO20
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio20(&mut self) -> gpio20::AXW<'_> {
            gpio20::AXW::new(self)
        }
        /// GPIO21
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio21(&mut self) -> gpio21::AXW<'_> {
            gpio21::AXW::new(self)
        }
        /// GPIO22
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio22(&mut self) -> gpio22::AXW<'_> {
            gpio22::AXW::new(self)
        }
        /// GPIO23
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio23(&mut self) -> gpio23::AXW<'_> {
            gpio23::AXW::new(self)
        }
        /// GPIO24
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio24(&mut self) -> gpio24::AXW<'_> {
            gpio24::AXW::new(self)
        }
        /// GPIO25
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio25(&mut self) -> gpio25::AXW<'_> {
            gpio25::AXW::new(self)
        }
        /// GPIO26
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio26(&mut self) -> gpio26::AXW<'_> {
            gpio26::AXW::new(self)
        }
        /// GPIO27
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio27(&mut self) -> gpio27::AXW<'_> {
            gpio27::AXW::new(self)
        }
        /// GPIO28
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio28(&mut self) -> gpio28::AXW<'_> {
            gpio28::AXW::new(self)
        }
        /// GPIO29
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio29(&mut self) -> gpio29::AXW<'_> {
            gpio29::AXW::new(self)
        }
        /// GPIO30
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio30(&mut self) -> gpio30::AXW<'_> {
            gpio30::AXW::new(self)
        }
        /// GPIO31
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn gpio31(&mut self) -> gpio31::AXW<'_> {
            gpio31::AXW::new(self)
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
    impl crate::ResetValue for NOSYNC_IN_SPEC {
        const RESET_VALUE: u32 = 0x0;
    }
    /// GPIO0
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio0 {
        pub type R = crate::BitReader;
    }
    /// GPIO1
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio1 {
        pub type R = crate::BitReader;
    }
    /// GPIO2
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio2 {
        pub type R = crate::BitReader;
    }
    /// GPIO3
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio3 {
        pub type R = crate::BitReader;
    }
    /// GPIO4
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio4 {
        pub type R = crate::BitReader;
    }
    /// GPIO5
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio5 {
        pub type R = crate::BitReader;
    }
    /// GPIO6
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio6 {
        pub type R = crate::BitReader;
    }
    /// GPIO7
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio7 {
        pub type R = crate::BitReader;
    }
    /// GPIO8
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio8 {
        pub type R = crate::BitReader;
    }
    /// GPIO9
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio9 {
        pub type R = crate::BitReader;
    }
    /// GPIO10
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio10 {
        pub type R = crate::BitReader;
    }
    /// GPIO11
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio11 {
        pub type R = crate::BitReader;
    }
    /// GPIO12
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio12 {
        pub type R = crate::BitReader;
    }
    /// GPIO13
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio13 {
        pub type R = crate::BitReader;
    }
    /// GPIO14
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio14 {
        pub type R = crate::BitReader;
    }
    /// GPIO15
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio15 {
        pub type R = crate::BitReader;
    }
    /// GPIO16
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio16 {
        pub type R = crate::BitReader;
    }
    /// GPIO17
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio17 {
        pub type R = crate::BitReader;
    }
    /// GPIO18
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio18 {
        pub type R = crate::BitReader;
    }
    /// GPIO19
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio19 {
        pub type R = crate::BitReader;
    }
    /// GPIO20
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio20 {
        pub type R = crate::BitReader;
    }
    /// GPIO21
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio21 {
        pub type R = crate::BitReader;
    }
    /// GPIO22
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio22 {
        pub type R = crate::BitReader;
    }
    /// GPIO23
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio23 {
        pub type R = crate::BitReader;
    }
    /// GPIO24
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio24 {
        pub type R = crate::BitReader;
    }
    /// GPIO25
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio25 {
        pub type R = crate::BitReader;
    }
    /// GPIO26
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio26 {
        pub type R = crate::BitReader;
    }
    /// GPIO27
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio27 {
        pub type R = crate::BitReader;
    }
    /// GPIO28
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio28 {
        pub type R = crate::BitReader;
    }
    /// GPIO29
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio29 {
        pub type R = crate::BitReader;
    }
    /// GPIO30
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio30 {
        pub type R = crate::BitReader;
    }
    /// GPIO31
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio31 {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn gpio0(&self) -> gpio0::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio1(&self) -> gpio1::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio2(&self) -> gpio2::R {
            crate::BitReader::new(((self.bits() >> 2) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio3(&self) -> gpio3::R {
            crate::BitReader::new(((self.bits() >> 3) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio4(&self) -> gpio4::R {
            crate::BitReader::new(((self.bits() >> 4) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio5(&self) -> gpio5::R {
            crate::BitReader::new(((self.bits() >> 5) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio6(&self) -> gpio6::R {
            crate::BitReader::new(((self.bits() >> 6) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio7(&self) -> gpio7::R {
            crate::BitReader::new(((self.bits() >> 7) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio8(&self) -> gpio8::R {
            crate::BitReader::new(((self.bits() >> 8) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio9(&self) -> gpio9::R {
            crate::BitReader::new(((self.bits() >> 9) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio10(&self) -> gpio10::R {
            crate::BitReader::new(((self.bits() >> 10) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio11(&self) -> gpio11::R {
            crate::BitReader::new(((self.bits() >> 11) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio12(&self) -> gpio12::R {
            crate::BitReader::new(((self.bits() >> 12) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio13(&self) -> gpio13::R {
            crate::BitReader::new(((self.bits() >> 13) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio14(&self) -> gpio14::R {
            crate::BitReader::new(((self.bits() >> 14) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio15(&self) -> gpio15::R {
            crate::BitReader::new(((self.bits() >> 15) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio16(&self) -> gpio16::R {
            crate::BitReader::new(((self.bits() >> 16) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio17(&self) -> gpio17::R {
            crate::BitReader::new(((self.bits() >> 17) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio18(&self) -> gpio18::R {
            crate::BitReader::new(((self.bits() >> 18) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio19(&self) -> gpio19::R {
            crate::BitReader::new(((self.bits() >> 19) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio20(&self) -> gpio20::R {
            crate::BitReader::new(((self.bits() >> 20) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio21(&self) -> gpio21::R {
            crate::BitReader::new(((self.bits() >> 21) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio22(&self) -> gpio22::R {
            crate::BitReader::new(((self.bits() >> 22) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio23(&self) -> gpio23::R {
            crate::BitReader::new(((self.bits() >> 23) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio24(&self) -> gpio24::R {
            crate::BitReader::new(((self.bits() >> 24) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio25(&self) -> gpio25::R {
            crate::BitReader::new(((self.bits() >> 25) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio26(&self) -> gpio26::R {
            crate::BitReader::new(((self.bits() >> 26) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio27(&self) -> gpio27::R {
            crate::BitReader::new(((self.bits() >> 27) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio28(&self) -> gpio28::R {
            crate::BitReader::new(((self.bits() >> 28) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio29(&self) -> gpio29::R {
            crate::BitReader::new(((self.bits() >> 29) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio30(&self) -> gpio30::R {
            crate::BitReader::new(((self.bits() >> 30) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio31(&self) -> gpio31::R {
            crate::BitReader::new(((self.bits() >> 31) & 1) != 0)
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
    impl crate::ResetValue for SYNC_IN_SPEC {
        const RESET_VALUE: u32 = 0x0;
    }
    /// GPIO0
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio0 {
        pub type R = crate::BitReader;
    }
    /// GPIO1
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio1 {
        pub type R = crate::BitReader;
    }
    /// GPIO2
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio2 {
        pub type R = crate::BitReader;
    }
    /// GPIO3
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio3 {
        pub type R = crate::BitReader;
    }
    /// GPIO4
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio4 {
        pub type R = crate::BitReader;
    }
    /// GPIO5
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio5 {
        pub type R = crate::BitReader;
    }
    /// GPIO6
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio6 {
        pub type R = crate::BitReader;
    }
    /// GPIO7
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio7 {
        pub type R = crate::BitReader;
    }
    /// GPIO8
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio8 {
        pub type R = crate::BitReader;
    }
    /// GPIO9
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio9 {
        pub type R = crate::BitReader;
    }
    /// GPIO10
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio10 {
        pub type R = crate::BitReader;
    }
    /// GPIO11
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio11 {
        pub type R = crate::BitReader;
    }
    /// GPIO12
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio12 {
        pub type R = crate::BitReader;
    }
    /// GPIO13
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio13 {
        pub type R = crate::BitReader;
    }
    /// GPIO14
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio14 {
        pub type R = crate::BitReader;
    }
    /// GPIO15
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio15 {
        pub type R = crate::BitReader;
    }
    /// GPIO16
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio16 {
        pub type R = crate::BitReader;
    }
    /// GPIO17
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio17 {
        pub type R = crate::BitReader;
    }
    /// GPIO18
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio18 {
        pub type R = crate::BitReader;
    }
    /// GPIO19
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio19 {
        pub type R = crate::BitReader;
    }
    /// GPIO20
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio20 {
        pub type R = crate::BitReader;
    }
    /// GPIO21
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio21 {
        pub type R = crate::BitReader;
    }
    /// GPIO22
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio22 {
        pub type R = crate::BitReader;
    }
    /// GPIO23
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio23 {
        pub type R = crate::BitReader;
    }
    /// GPIO24
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio24 {
        pub type R = crate::BitReader;
    }
    /// GPIO25
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio25 {
        pub type R = crate::BitReader;
    }
    /// GPIO26
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio26 {
        pub type R = crate::BitReader;
    }
    /// GPIO27
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio27 {
        pub type R = crate::BitReader;
    }
    /// GPIO28
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio28 {
        pub type R = crate::BitReader;
    }
    /// GPIO29
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio29 {
        pub type R = crate::BitReader;
    }
    /// GPIO30
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio30 {
        pub type R = crate::BitReader;
    }
    /// GPIO31
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod gpio31 {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn gpio0(&self) -> gpio0::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio1(&self) -> gpio1::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio2(&self) -> gpio2::R {
            crate::BitReader::new(((self.bits() >> 2) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio3(&self) -> gpio3::R {
            crate::BitReader::new(((self.bits() >> 3) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio4(&self) -> gpio4::R {
            crate::BitReader::new(((self.bits() >> 4) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio5(&self) -> gpio5::R {
            crate::BitReader::new(((self.bits() >> 5) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio6(&self) -> gpio6::R {
            crate::BitReader::new(((self.bits() >> 6) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio7(&self) -> gpio7::R {
            crate::BitReader::new(((self.bits() >> 7) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio8(&self) -> gpio8::R {
            crate::BitReader::new(((self.bits() >> 8) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio9(&self) -> gpio9::R {
            crate::BitReader::new(((self.bits() >> 9) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio10(&self) -> gpio10::R {
            crate::BitReader::new(((self.bits() >> 10) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio11(&self) -> gpio11::R {
            crate::BitReader::new(((self.bits() >> 11) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio12(&self) -> gpio12::R {
            crate::BitReader::new(((self.bits() >> 12) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio13(&self) -> gpio13::R {
            crate::BitReader::new(((self.bits() >> 13) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio14(&self) -> gpio14::R {
            crate::BitReader::new(((self.bits() >> 14) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio15(&self) -> gpio15::R {
            crate::BitReader::new(((self.bits() >> 15) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio16(&self) -> gpio16::R {
            crate::BitReader::new(((self.bits() >> 16) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio17(&self) -> gpio17::R {
            crate::BitReader::new(((self.bits() >> 17) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio18(&self) -> gpio18::R {
            crate::BitReader::new(((self.bits() >> 18) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio19(&self) -> gpio19::R {
            crate::BitReader::new(((self.bits() >> 19) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio20(&self) -> gpio20::R {
            crate::BitReader::new(((self.bits() >> 20) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio21(&self) -> gpio21::R {
            crate::BitReader::new(((self.bits() >> 21) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio22(&self) -> gpio22::R {
            crate::BitReader::new(((self.bits() >> 22) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio23(&self) -> gpio23::R {
            crate::BitReader::new(((self.bits() >> 23) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio24(&self) -> gpio24::R {
            crate::BitReader::new(((self.bits() >> 24) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio25(&self) -> gpio25::R {
            crate::BitReader::new(((self.bits() >> 25) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio26(&self) -> gpio26::R {
            crate::BitReader::new(((self.bits() >> 26) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio27(&self) -> gpio27::R {
            crate::BitReader::new(((self.bits() >> 27) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio28(&self) -> gpio28::R {
            crate::BitReader::new(((self.bits() >> 28) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio29(&self) -> gpio29::R {
            crate::BitReader::new(((self.bits() >> 29) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio30(&self) -> gpio30::R {
            crate::BitReader::new(((self.bits() >> 30) & 1) != 0)
        }
        #[inline(always)]
        pub fn gpio31(&self) -> gpio31::R {
            crate::BitReader::new(((self.bits() >> 31) & 1) != 0)
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
