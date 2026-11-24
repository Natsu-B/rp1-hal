// Generated from rp1-regdb/data/peripherals/pads-bank0.toml.
// Do not edit manually.
// Regenerate with: cargo run -p rp1-regdb -- generate

use crate::Reg;

/// Pad electrical controls and input threshold selection for GPIO bank 0. Bus=APB. Atomic aliases: XOR at +0x1000, SET at +0x2000, CLR at +0x3000.
/// Atomic view: Rp1Native alias windows; ownership remains with this logical peripheral.
#[repr(C)]
pub struct RegisterBlock {
    voltage_select: VOLTAGE_SELECT,
    gpio: [GPIO; 28],
}

impl RegisterBlock {
    /// Selects the input threshold levels for the whole bank. Choose the 1.8 V setting only when VDDIO is nominally 1.8 V; using it with a higher supply can damage the chip. Keeping the default setting at 1.8 V may give thresholds outside specification.
    /// Hardware access: read Plain; write Plain.
    #[inline(always)]
    pub const fn voltage_select(&self) -> &VOLTAGE_SELECT {
        &self.voltage_select
    }
    /// Electrical controls for one pad; function selection is in IO_BANK. With the output not driving, PUE/PDE select the bias: 0/0 floating, 1/0 weak pull to high, 0/1 weak pull to low, 1/1 keeper that holds the last driven level.
    /// Hardware access: read Plain; write Plain.
    /// Array layout: 28 physical entries, 28 public entries, stride 0x4 bytes.
    #[inline(always)]
    pub fn gpio(&self, index: usize) -> &GPIO {
        assert!(index < 28);
        &self.gpio[index]
    }
    #[inline(always)]
    pub fn gpio_array(&self) -> &[GPIO] {
        &self.gpio[..28]
    }
}

/// Selects the input threshold levels for the whole bank. Choose the 1.8 V setting only when VDDIO is nominally 1.8 V; using it with a higher supply can damage the chip. Keeping the default setting at 1.8 V may give thresholds outside specification.
/// Hardware access: read Plain; write Plain.
pub type VOLTAGE_SELECT = Reg<voltage_select::VOLTAGE_SELECT_SPEC>;
/// Selects the input threshold levels for the whole bank. Choose the 1.8 V setting only when VDDIO is nominally 1.8 V; using it with a higher supply can damage the chip. Keeping the default setting at 1.8 V may give thresholds outside specification.
/// Hardware access: read Plain; write Plain.
pub mod voltage_select {
    pub struct VOLTAGE_SELECT_SPEC;
    impl crate::RegisterSpec for VOLTAGE_SELECT_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for VOLTAGE_SELECT_SPEC {}
    pub type R = crate::R<VOLTAGE_SELECT_SPEC>;
    impl crate::Modifiable for VOLTAGE_SELECT_SPEC {}
    impl crate::UnsafeWritable for VOLTAGE_SELECT_SPEC {}
    impl crate::ResetValue for VOLTAGE_SELECT_SPEC {
        const RESET_VALUE: u32 = 0x0;
    }
    impl crate::TypedAtomicSet for VOLTAGE_SELECT_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicSet for VOLTAGE_SELECT_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::TypedAtomicClear for VOLTAGE_SELECT_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicClear for VOLTAGE_SELECT_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::TypedAtomicXor for VOLTAGE_SELECT_SPEC {
        const OFFSET: usize = 0x1000;
    }
    impl crate::UnsafeAtomicXor for VOLTAGE_SELECT_SPEC {
        const OFFSET: usize = 0x1000;
    }
    pub type W = crate::W<VOLTAGE_SELECT_SPEC>;
    pub type AtomicSetW = crate::AtomicSetW<VOLTAGE_SELECT_SPEC>;
    pub type AtomicClearW = crate::AtomicClearW<VOLTAGE_SELECT_SPEC>;
    pub type AtomicXorW = crate::AtomicXorW<VOLTAGE_SELECT_SPEC>;
    /// 0 = thresholds for 2.5 V to 3.3 V VDDIO; 1 = thresholds for 1.8 V VDDIO. Only the input thresholds change, not the supply voltage.
    /// Field write behavior: Plain.
    pub mod voltage_select {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 0>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 0>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 0>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 0>;
    }
    impl R {
        #[inline(always)]
        pub fn voltage_select(&self) -> voltage_select::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
    }
    impl W {
        #[inline(always)]
        pub fn voltage_select(&mut self) -> voltage_select::W<'_> {
            voltage_select::W::new(self)
        }
    }
    impl AtomicSetW {
        /// 0 = thresholds for 2.5 V to 3.3 V VDDIO; 1 = thresholds for 1.8 V VDDIO. Only the input thresholds change, not the supply voltage.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn voltage_select(&mut self) -> voltage_select::ASW<'_> {
            voltage_select::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// 0 = thresholds for 2.5 V to 3.3 V VDDIO; 1 = thresholds for 1.8 V VDDIO. Only the input thresholds change, not the supply voltage.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn voltage_select(&mut self) -> voltage_select::ACW<'_> {
            voltage_select::ACW::new(self)
        }
    }
    impl AtomicXorW {
        /// 0 = thresholds for 2.5 V to 3.3 V VDDIO; 1 = thresholds for 1.8 V VDDIO. Only the input thresholds change, not the supply voltage.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn voltage_select(&mut self) -> voltage_select::AXW<'_> {
            voltage_select::AXW::new(self)
        }
    }
}

/// Electrical controls for one pad; function selection is in IO_BANK. With the output not driving, PUE/PDE select the bias: 0/0 floating, 1/0 weak pull to high, 0/1 weak pull to low, 1/1 keeper that holds the last driven level.
/// Hardware access: read Plain; write Plain.
/// Array layout: 28 physical entries, 28 public entries, stride 0x4 bytes.
pub type GPIO = Reg<gpio::GPIO_SPEC>;
/// Electrical controls for one pad; function selection is in IO_BANK. With the output not driving, PUE/PDE select the bias: 0/0 floating, 1/0 weak pull to high, 0/1 weak pull to low, 1/1 keeper that holds the last driven level.
/// Hardware access: read Plain; write Plain.
/// Array layout: 28 physical entries, 28 public entries, stride 0x4 bytes.
pub mod gpio {
    pub struct GPIO_SPEC;
    impl crate::RegisterSpec for GPIO_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for GPIO_SPEC {}
    pub type R = crate::R<GPIO_SPEC>;
    impl crate::Modifiable for GPIO_SPEC {}
    impl crate::UnsafeWritable for GPIO_SPEC {}
    impl crate::TypedAtomicSet for GPIO_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicSet for GPIO_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::TypedAtomicClear for GPIO_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicClear for GPIO_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::TypedAtomicXor for GPIO_SPEC {
        const OFFSET: usize = 0x1000;
    }
    impl crate::UnsafeAtomicXor for GPIO_SPEC {
        const OFFSET: usize = 0x1000;
    }
    pub type W = crate::W<GPIO_SPEC>;
    pub type AtomicSetW = crate::AtomicSetW<GPIO_SPEC>;
    pub type AtomicClearW = crate::AtomicClearW<GPIO_SPEC>;
    pub type AtomicXorW = crate::AtomicXorW<GPIO_SPEC>;
    /// Output slew rate. 0 slow (limited edge rate), 1 fast.
    /// Field write behavior: Plain.
    pub mod slewfast {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 0>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 0>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 0>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 0>;
    }
    /// Input hysteresis (Schmitt trigger). 0 off, 1 on.
    /// Field write behavior: Plain.
    pub mod schmitt {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 1>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 1>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 1>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 1>;
    }
    /// Weak pull-down (to ground) enable. Setting both PDE and PUE gives a bus keeper.
    /// Field write behavior: Plain.
    pub mod pde {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 2>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 2>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 2>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 2>;
    }
    /// Weak pull-up (to VDDIO) enable. Setting both PUE and PDE gives a bus keeper.
    /// Field write behavior: Plain.
    pub mod pue {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 3>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 3>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 3>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 3>;
    }
    /// Output drive strength: 0 = 2 mA, 1 = 4 mA, 2 = 8 mA, 3 = 12 mA.
    /// Field write behavior: Plain.
    pub mod drive {
        pub type R = crate::FieldReader<u32>;
        pub type W<'a> = crate::FieldWriter<'a, super::W, 4, 2>;
        pub type ASW<'a> = crate::AtomicSetFieldWriter<'a, super::AtomicSetW, 4, 2>;
        pub type ACW<'a> = crate::AtomicClearFieldWriter<'a, super::AtomicClearW, 4, 2>;
        pub type AXW<'a> = crate::AtomicXorFieldWriter<'a, super::AtomicXorW, 4, 2>;
        pub const MA_2: u32 = 0x0;
        pub const MA_4: u32 = 0x1;
        pub const MA_8: u32 = 0x2;
        pub const MA_12: u32 = 0x3;
    }
    /// Input enable. 0 powers the input buffer down; 1 enables it. Unused pads may leave it off.
    /// Field write behavior: Plain.
    pub mod ie {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 6>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 6>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 6>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 6>;
    }
    /// Output disable. 1 blocks the output driver even when the selected function enables output; 0 leaves the output enable to IO_BANK.
    /// Field write behavior: Plain.
    pub mod od {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 7>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 7>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 7>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 7>;
    }
    impl R {
        #[inline(always)]
        pub fn slewfast(&self) -> slewfast::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn schmitt(&self) -> schmitt::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
        #[inline(always)]
        pub fn pde(&self) -> pde::R {
            crate::BitReader::new(((self.bits() >> 2) & 1) != 0)
        }
        #[inline(always)]
        pub fn pue(&self) -> pue::R {
            crate::BitReader::new(((self.bits() >> 3) & 1) != 0)
        }
        #[inline(always)]
        pub fn drive(&self) -> drive::R {
            crate::FieldReader::new((self.bits() >> 4) & 0x3)
        }
        #[inline(always)]
        pub fn ie(&self) -> ie::R {
            crate::BitReader::new(((self.bits() >> 6) & 1) != 0)
        }
        #[inline(always)]
        pub fn od(&self) -> od::R {
            crate::BitReader::new(((self.bits() >> 7) & 1) != 0)
        }
    }
    impl W {
        #[inline(always)]
        pub fn slewfast(&mut self) -> slewfast::W<'_> {
            slewfast::W::new(self)
        }
        #[inline(always)]
        pub fn schmitt(&mut self) -> schmitt::W<'_> {
            schmitt::W::new(self)
        }
        #[inline(always)]
        pub fn pde(&mut self) -> pde::W<'_> {
            pde::W::new(self)
        }
        #[inline(always)]
        pub fn pue(&mut self) -> pue::W<'_> {
            pue::W::new(self)
        }
        #[inline(always)]
        pub fn drive(&mut self) -> drive::W<'_> {
            drive::W::new(self)
        }
        #[inline(always)]
        pub fn ie(&mut self) -> ie::W<'_> {
            ie::W::new(self)
        }
        #[inline(always)]
        pub fn od(&mut self) -> od::W<'_> {
            od::W::new(self)
        }
    }
    impl AtomicSetW {
        /// Output slew rate. 0 slow (limited edge rate), 1 fast.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn slewfast(&mut self) -> slewfast::ASW<'_> {
            slewfast::ASW::new(self)
        }
        /// Input hysteresis (Schmitt trigger). 0 off, 1 on.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn schmitt(&mut self) -> schmitt::ASW<'_> {
            schmitt::ASW::new(self)
        }
        /// Weak pull-down (to ground) enable. Setting both PDE and PUE gives a bus keeper.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn pde(&mut self) -> pde::ASW<'_> {
            pde::ASW::new(self)
        }
        /// Weak pull-up (to VDDIO) enable. Setting both PUE and PDE gives a bus keeper.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn pue(&mut self) -> pue::ASW<'_> {
            pue::ASW::new(self)
        }
        /// Output drive strength: 0 = 2 mA, 1 = 4 mA, 2 = 8 mA, 3 = 12 mA.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn drive(&mut self) -> drive::ASW<'_> {
            drive::ASW::new(self)
        }
        /// Input enable. 0 powers the input buffer down; 1 enables it. Unused pads may leave it off.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn ie(&mut self) -> ie::ASW<'_> {
            ie::ASW::new(self)
        }
        /// Output disable. 1 blocks the output driver even when the selected function enables output; 0 leaves the output enable to IO_BANK.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn od(&mut self) -> od::ASW<'_> {
            od::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// Output slew rate. 0 slow (limited edge rate), 1 fast.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn slewfast(&mut self) -> slewfast::ACW<'_> {
            slewfast::ACW::new(self)
        }
        /// Input hysteresis (Schmitt trigger). 0 off, 1 on.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn schmitt(&mut self) -> schmitt::ACW<'_> {
            schmitt::ACW::new(self)
        }
        /// Weak pull-down (to ground) enable. Setting both PDE and PUE gives a bus keeper.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn pde(&mut self) -> pde::ACW<'_> {
            pde::ACW::new(self)
        }
        /// Weak pull-up (to VDDIO) enable. Setting both PUE and PDE gives a bus keeper.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn pue(&mut self) -> pue::ACW<'_> {
            pue::ACW::new(self)
        }
        /// Output drive strength: 0 = 2 mA, 1 = 4 mA, 2 = 8 mA, 3 = 12 mA.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn drive(&mut self) -> drive::ACW<'_> {
            drive::ACW::new(self)
        }
        /// Input enable. 0 powers the input buffer down; 1 enables it. Unused pads may leave it off.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn ie(&mut self) -> ie::ACW<'_> {
            ie::ACW::new(self)
        }
        /// Output disable. 1 blocks the output driver even when the selected function enables output; 0 leaves the output enable to IO_BANK.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn od(&mut self) -> od::ACW<'_> {
            od::ACW::new(self)
        }
    }
    impl AtomicXorW {
        /// Output slew rate. 0 slow (limited edge rate), 1 fast.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn slewfast(&mut self) -> slewfast::AXW<'_> {
            slewfast::AXW::new(self)
        }
        /// Input hysteresis (Schmitt trigger). 0 off, 1 on.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn schmitt(&mut self) -> schmitt::AXW<'_> {
            schmitt::AXW::new(self)
        }
        /// Weak pull-down (to ground) enable. Setting both PDE and PUE gives a bus keeper.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn pde(&mut self) -> pde::AXW<'_> {
            pde::AXW::new(self)
        }
        /// Weak pull-up (to VDDIO) enable. Setting both PUE and PDE gives a bus keeper.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn pue(&mut self) -> pue::AXW<'_> {
            pue::AXW::new(self)
        }
        /// Output drive strength: 0 = 2 mA, 1 = 4 mA, 2 = 8 mA, 3 = 12 mA.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn drive(&mut self) -> drive::AXW<'_> {
            drive::AXW::new(self)
        }
        /// Input enable. 0 powers the input buffer down; 1 enables it. Unused pads may leave it off.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn ie(&mut self) -> ie::AXW<'_> {
            ie::AXW::new(self)
        }
        /// Output disable. 1 blocks the output driver even when the selected function enables output; 0 leaves the output enable to IO_BANK.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn od(&mut self) -> od::AXW<'_> {
            od::AXW::new(self)
        }
    }
}

#[cfg(test)]
mod layout_tests {
    use super::RegisterBlock;
    #[test]
    fn voltage_select_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, voltage_select), 0x0);
    }
    #[test]
    fn gpio_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, gpio), 0x4);
    }
    #[test]
    fn register_block_size() {
        assert_eq!(core::mem::size_of::<RegisterBlock>(), 0x74);
    }
}
