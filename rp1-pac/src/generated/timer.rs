// Generated from rp1-regdb/data/peripherals/timer.toml.
// Do not edit manually.
// Regenerate with: cargo run -p rp1-regdb -- generate

use crate::Reg;

/// RP1 timer block. RAW_HIGH/RAW_LOW advance at one tick per microsecond; coherent raw reads use high-low-high and retry on rollover. Write protocols are exposed only where independently verified.
/// Atomic view: Rp1Native alias windows; ownership remains with this logical peripheral.
#[repr(C)]
pub struct RegisterBlock {
    timehw: TIMEHW,
    timelw: TIMELW,
    timehr: TIMEHR,
    timelr: TIMELR,
    alarm: [ALARM; 4],
    armed: ARMED,
    raw_high: RAW_HIGH,
    raw_low: RAW_LOW,
    debug_pause: DEBUG_PAUSE,
    pause: PAUSE,
    intr: INTR,
    inte: INTE,
    intf: INTF,
    ints: INTS,
}

impl RegisterBlock {
    /// Latched high half; read after TIMELR. \[Evidence: HardwareProven\]
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn timehr(&self) -> &TIMEHR {
        &self.timehr
    }
    /// Low half; reading latches TIMEHR. \[Evidence: HardwareProven\]
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn timelr(&self) -> &TIMELR {
        &self.timelr
    }
    /// Writing an alarm value arms that channel; represented as a command rather than ordinary RW state.
    /// Hardware access: read Plain; write Command.
    /// Command write; ordinary read, write, and modify APIs are not generated.
    /// Array layout: 4 physical entries, 4 public entries, stride 0x4 bytes.
    #[inline(always)]
    pub fn alarm(&self, index: usize) -> &ALARM {
        assert!(index < 4);
        &self.alarm[index]
    }
    #[inline(always)]
    pub fn alarm_array(&self) -> &[ALARM] {
        &self.alarm[..4]
    }
    /// Alarm writes set ARMED and firing automatically clears it. Explicit W1C disarm is not hardware-verified and is not exposed.
    /// Hardware access: read Plain; write WriteOneToClear.
    /// Write-one-to-clear register; general write and modify APIs are not generated.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn armed(&self) -> &ARMED {
        &self.armed
    }
    /// Side-effect-free free-running counter high half; use high-low-high coherent reads. One tick per microsecond was hardware-proven. \[Evidence: HardwareProven\]
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn raw_high(&self) -> &RAW_HIGH {
        &self.raw_high
    }
    /// Side-effect-free free-running counter low half; use high-low-high coherent reads. \[Evidence: HardwareProven\]
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn raw_low(&self) -> &RAW_LOW {
        &self.raw_low
    }
    /// Pause while processor debug ports are active. \[Evidence: HardwareReadProven; write semantics remain unverified.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn debug_pause(&self) -> &DEBUG_PAUSE {
        &self.debug_pause
    }
    /// Global timer pause. \[Evidence: HardwareReadProven; write semantics remain unverified.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn pause(&self) -> &PAUSE {
        &self.pause
    }
    /// Raw alarm interrupt status. W1C behavior was independently hardware-verified.
    /// Hardware access: read Plain; write WriteOneToClear.
    /// Write-one-to-clear register; general write and modify APIs are not generated.
    #[inline(always)]
    pub const fn intr(&self) -> &INTR {
        &self.intr
    }
    /// Alarm interrupt enable. \[Evidence: HardwareReadProven; write semantics remain unverified.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn inte(&self) -> &INTE {
        &self.inte
    }
    /// Alarm interrupt force. \[Evidence: HardwareReadProven; write semantics remain unverified.\]
    /// Hardware access: read Plain; write WriteOneToSet.
    /// Write-one-to-set register; general write and modify APIs are not generated.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn intf(&self) -> &INTF {
        &self.intf
    }
    /// Masked and forced alarm interrupt status. \[Evidence: HardwareReadProven; write semantics remain unverified.\]
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn ints(&self) -> &INTS {
        &self.ints
    }
}

/// Upper half of the coordinated timer write protocol; hidden from the general PAC API.
/// Hardware access: read None; write StateMachineOnly.
/// State-machine-only register; unrestricted writers are intentionally hidden.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type TIMEHW = Reg<timehw::TIMEHW_SPEC>;
/// Upper half of the coordinated timer write protocol; hidden from the general PAC API.
/// Hardware access: read None; write StateMachineOnly.
/// State-machine-only register; unrestricted writers are intentionally hidden.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod timehw {
    pub struct TIMEHW_SPEC;
    impl crate::RegisterSpec for TIMEHW_SPEC {
        type Ux = u32;
    }
    /// Writing the high half commits it with the low half staged in TIMELW. \[Evidence: HardwareProven\]
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod value {}
}

/// Stages the lower half of the coordinated timer write protocol; the following TIMEHW write commits both halves. Hidden from the general PAC API.
/// Hardware access: read None; write StateMachineOnly.
/// State-machine-only register; unrestricted writers are intentionally hidden.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type TIMELW = Reg<timelw::TIMELW_SPEC>;
/// Stages the lower half of the coordinated timer write protocol; the following TIMEHW write commits both halves. Hidden from the general PAC API.
/// Hardware access: read None; write StateMachineOnly.
/// State-machine-only register; unrestricted writers are intentionally hidden.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod timelw {
    pub struct TIMELW_SPEC;
    impl crate::RegisterSpec for TIMELW_SPEC {
        type Ux = u32;
    }
    /// Stages the writable low half without immediately changing time. \[Evidence: HardwareProven\]
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod value {}
}

/// Latched high half; read after TIMELR. \[Evidence: HardwareProven\]
/// Hardware access: read Plain; write None.
pub type TIMEHR = Reg<timehr::TIMEHR_SPEC>;
/// Latched high half; read after TIMELR. \[Evidence: HardwareProven\]
/// Hardware access: read Plain; write None.
pub mod timehr {
    pub struct TIMEHR_SPEC;
    impl crate::RegisterSpec for TIMEHR_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for TIMEHR_SPEC {}
    pub type R = crate::R<TIMEHR_SPEC>;
    /// Latched high half; read after TIMELR. \[Evidence: HardwareProven\]
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
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

/// Low half; reading latches TIMEHR. \[Evidence: HardwareProven\]
/// Hardware access: read Plain; write None.
pub type TIMELR = Reg<timelr::TIMELR_SPEC>;
/// Low half; reading latches TIMEHR. \[Evidence: HardwareProven\]
/// Hardware access: read Plain; write None.
pub mod timelr {
    pub struct TIMELR_SPEC;
    impl crate::RegisterSpec for TIMELR_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for TIMELR_SPEC {}
    pub type R = crate::R<TIMELR_SPEC>;
    /// Low half; reading latches TIMEHR. \[Evidence: HardwareProven\]
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
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

/// Writing an alarm value arms that channel; represented as a command rather than ordinary RW state.
/// Hardware access: read Plain; write Command.
/// Command write; ordinary read, write, and modify APIs are not generated.
/// Array layout: 4 physical entries, 4 public entries, stride 0x4 bytes.
pub type ALARM = Reg<alarm::ALARM_SPEC>;
/// Writing an alarm value arms that channel; represented as a command rather than ordinary RW state.
/// Hardware access: read Plain; write Command.
/// Command write; ordinary read, write, and modify APIs are not generated.
/// Array layout: 4 physical entries, 4 public entries, stride 0x4 bytes.
pub mod alarm {
    pub struct ALARM_SPEC;
    impl crate::RegisterSpec for ALARM_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for ALARM_SPEC {}
    pub type R = crate::R<ALARM_SPEC>;
    impl crate::CommandWritable for ALARM_SPEC {}
    pub type CommandW = crate::CommandW<ALARM_SPEC>;
    /// Low-32-bit compare and arm register for alarm 0. \[Evidence: HardwareProven\]
    /// Field write behavior: Command.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
        pub type CMDW<'a> = crate::CommandFieldWriter<'a, super::CommandW, 0, 32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffffffff)
        }
    }
    impl CommandW {
        #[inline(always)]
        pub fn value(&mut self) -> value::CMDW<'_> {
            value::CMDW::new(self)
        }
    }
}

/// Alarm writes set ARMED and firing automatically clears it. Explicit W1C disarm is not hardware-verified and is not exposed.
/// Hardware access: read Plain; write WriteOneToClear.
/// Write-one-to-clear register; general write and modify APIs are not generated.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type ARMED = Reg<armed::ARMED_SPEC>;
/// Alarm writes set ARMED and firing automatically clears it. Explicit W1C disarm is not hardware-verified and is not exposed.
/// Hardware access: read Plain; write WriteOneToClear.
/// Write-one-to-clear register; general write and modify APIs are not generated.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod armed {
    pub struct ARMED_SPEC;
    impl crate::RegisterSpec for ARMED_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for ARMED_SPEC {}
    pub type R = crate::R<ARMED_SPEC>;
    impl crate::UnsafeWritable for ARMED_SPEC {}
    /// Field write behavior: WriteOneToClear.
    /// Typed write API is not exposed because write behavior is not hardware/official verified.
    pub mod alarm {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn alarm(&self) -> alarm::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xf)
        }
    }
}

/// Side-effect-free free-running counter high half; use high-low-high coherent reads. One tick per microsecond was hardware-proven. \[Evidence: HardwareProven\]
/// Hardware access: read Plain; write None.
pub type RAW_HIGH = Reg<raw_high::RAW_HIGH_SPEC>;
/// Side-effect-free free-running counter high half; use high-low-high coherent reads. One tick per microsecond was hardware-proven. \[Evidence: HardwareProven\]
/// Hardware access: read Plain; write None.
pub mod raw_high {
    pub struct RAW_HIGH_SPEC;
    impl crate::RegisterSpec for RAW_HIGH_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for RAW_HIGH_SPEC {}
    pub type R = crate::R<RAW_HIGH_SPEC>;
    /// Side-effect-free free-running counter high half; use high-low-high coherent reads. One tick per microsecond was hardware-proven. \[Evidence: HardwareProven\]
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
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

/// Side-effect-free free-running counter low half; use high-low-high coherent reads. \[Evidence: HardwareProven\]
/// Hardware access: read Plain; write None.
pub type RAW_LOW = Reg<raw_low::RAW_LOW_SPEC>;
/// Side-effect-free free-running counter low half; use high-low-high coherent reads. \[Evidence: HardwareProven\]
/// Hardware access: read Plain; write None.
pub mod raw_low {
    pub struct RAW_LOW_SPEC;
    impl crate::RegisterSpec for RAW_LOW_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for RAW_LOW_SPEC {}
    pub type R = crate::R<RAW_LOW_SPEC>;
    /// Side-effect-free free-running counter low half; use high-low-high coherent reads. \[Evidence: HardwareProven\]
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
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

/// Pause while processor debug ports are active. \[Evidence: HardwareReadProven; write semantics remain unverified.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type DEBUG_PAUSE = Reg<debug_pause::DEBUG_PAUSE_SPEC>;
/// Pause while processor debug ports are active. \[Evidence: HardwareReadProven; write semantics remain unverified.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod debug_pause {
    pub struct DEBUG_PAUSE_SPEC;
    impl crate::RegisterSpec for DEBUG_PAUSE_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for DEBUG_PAUSE_SPEC {}
    pub type R = crate::R<DEBUG_PAUSE_SPEC>;
    /// Pause for processor 0 debug.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod proc0 {
        pub type R = crate::BitReader;
    }
    /// Pause for processor 1 debug.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod proc1 {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn proc0(&self) -> proc0::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
        #[inline(always)]
        pub fn proc1(&self) -> proc1::R {
            crate::BitReader::new(((self.bits() >> 2) & 1) != 0)
        }
    }
}

/// Global timer pause. \[Evidence: HardwareReadProven; write semantics remain unverified.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type PAUSE = Reg<pause::PAUSE_SPEC>;
/// Global timer pause. \[Evidence: HardwareReadProven; write semantics remain unverified.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod pause {
    pub struct PAUSE_SPEC;
    impl crate::RegisterSpec for PAUSE_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for PAUSE_SPEC {}
    pub type R = crate::R<PAUSE_SPEC>;
    /// Pause the timer when set.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod enable {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn enable(&self) -> enable::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
    }
}

/// Raw alarm interrupt status. W1C behavior was independently hardware-verified.
/// Hardware access: read Plain; write WriteOneToClear.
/// Write-one-to-clear register; general write and modify APIs are not generated.
pub type INTR = Reg<intr::INTR_SPEC>;
/// Raw alarm interrupt status. W1C behavior was independently hardware-verified.
/// Hardware access: read Plain; write WriteOneToClear.
/// Write-one-to-clear register; general write and modify APIs are not generated.
pub mod intr {
    pub struct INTR_SPEC;
    impl crate::RegisterSpec for INTR_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for INTR_SPEC {}
    pub type R = crate::R<INTR_SPEC>;
    impl crate::WriteOneToClear for INTR_SPEC {}
    pub type ClearW = crate::ClearW<INTR_SPEC>;
    /// Field write behavior: WriteOneToClear.
    pub mod alarm {
        pub type R = crate::FieldReader<u32>;
        pub type CW<'a> = crate::ClearFieldWriter<'a, super::ClearW, 0, 4>;
    }
    impl R {
        #[inline(always)]
        pub fn alarm(&self) -> alarm::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xf)
        }
    }
    impl ClearW {
        #[inline(always)]
        pub fn alarm(&mut self) -> alarm::CW<'_> {
            alarm::CW::new(self)
        }
    }
}

/// Alarm interrupt enable. \[Evidence: HardwareReadProven; write semantics remain unverified.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type INTE = Reg<inte::INTE_SPEC>;
/// Alarm interrupt enable. \[Evidence: HardwareReadProven; write semantics remain unverified.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod inte {
    pub struct INTE_SPEC;
    impl crate::RegisterSpec for INTE_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for INTE_SPEC {}
    pub type R = crate::R<INTE_SPEC>;
    /// Alarm 0 interrupt bit.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod alarm0 {
        pub type R = crate::BitReader;
    }
    /// Alarm 1 interrupt bit.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod alarm1 {
        pub type R = crate::BitReader;
    }
    /// Alarm 2 interrupt bit.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod alarm2 {
        pub type R = crate::BitReader;
    }
    /// Alarm 3 interrupt bit.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod alarm3 {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn alarm0(&self) -> alarm0::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn alarm1(&self) -> alarm1::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
        #[inline(always)]
        pub fn alarm2(&self) -> alarm2::R {
            crate::BitReader::new(((self.bits() >> 2) & 1) != 0)
        }
        #[inline(always)]
        pub fn alarm3(&self) -> alarm3::R {
            crate::BitReader::new(((self.bits() >> 3) & 1) != 0)
        }
    }
}

/// Alarm interrupt force. \[Evidence: HardwareReadProven; write semantics remain unverified.\]
/// Hardware access: read Plain; write WriteOneToSet.
/// Write-one-to-set register; general write and modify APIs are not generated.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type INTF = Reg<intf::INTF_SPEC>;
/// Alarm interrupt force. \[Evidence: HardwareReadProven; write semantics remain unverified.\]
/// Hardware access: read Plain; write WriteOneToSet.
/// Write-one-to-set register; general write and modify APIs are not generated.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod intf {
    pub struct INTF_SPEC;
    impl crate::RegisterSpec for INTF_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for INTF_SPEC {}
    pub type R = crate::R<INTF_SPEC>;
    /// Alarm 0 interrupt bit.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod alarm0 {
        pub type R = crate::BitReader;
    }
    /// Alarm 1 interrupt bit.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod alarm1 {
        pub type R = crate::BitReader;
    }
    /// Alarm 2 interrupt bit.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod alarm2 {
        pub type R = crate::BitReader;
    }
    /// Alarm 3 interrupt bit.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod alarm3 {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn alarm0(&self) -> alarm0::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn alarm1(&self) -> alarm1::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
        #[inline(always)]
        pub fn alarm2(&self) -> alarm2::R {
            crate::BitReader::new(((self.bits() >> 2) & 1) != 0)
        }
        #[inline(always)]
        pub fn alarm3(&self) -> alarm3::R {
            crate::BitReader::new(((self.bits() >> 3) & 1) != 0)
        }
    }
}

/// Masked and forced alarm interrupt status. \[Evidence: HardwareReadProven; write semantics remain unverified.\]
/// Hardware access: read Plain; write None.
pub type INTS = Reg<ints::INTS_SPEC>;
/// Masked and forced alarm interrupt status. \[Evidence: HardwareReadProven; write semantics remain unverified.\]
/// Hardware access: read Plain; write None.
pub mod ints {
    pub struct INTS_SPEC;
    impl crate::RegisterSpec for INTS_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for INTS_SPEC {}
    pub type R = crate::R<INTS_SPEC>;
    /// Alarm 0 interrupt bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod alarm0 {
        pub type R = crate::BitReader;
    }
    /// Alarm 1 interrupt bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod alarm1 {
        pub type R = crate::BitReader;
    }
    /// Alarm 2 interrupt bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod alarm2 {
        pub type R = crate::BitReader;
    }
    /// Alarm 3 interrupt bit.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod alarm3 {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn alarm0(&self) -> alarm0::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn alarm1(&self) -> alarm1::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
        #[inline(always)]
        pub fn alarm2(&self) -> alarm2::R {
            crate::BitReader::new(((self.bits() >> 2) & 1) != 0)
        }
        #[inline(always)]
        pub fn alarm3(&self) -> alarm3::R {
            crate::BitReader::new(((self.bits() >> 3) & 1) != 0)
        }
    }
}

#[cfg(test)]
mod layout_tests {
    use super::RegisterBlock;
    #[test]
    fn timehw_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, timehw), 0x0);
    }
    #[test]
    fn timelw_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, timelw), 0x4);
    }
    #[test]
    fn timehr_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, timehr), 0x8);
    }
    #[test]
    fn timelr_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, timelr), 0xc);
    }
    #[test]
    fn alarm_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, alarm), 0x10);
    }
    #[test]
    fn armed_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, armed), 0x20);
    }
    #[test]
    fn raw_high_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, raw_high), 0x24);
    }
    #[test]
    fn raw_low_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, raw_low), 0x28);
    }
    #[test]
    fn debug_pause_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, debug_pause), 0x2c);
    }
    #[test]
    fn pause_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, pause), 0x30);
    }
    #[test]
    fn intr_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, intr), 0x34);
    }
    #[test]
    fn inte_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, inte), 0x38);
    }
    #[test]
    fn intf_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, intf), 0x3c);
    }
    #[test]
    fn ints_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, ints), 0x40);
    }
    #[test]
    fn register_block_size() {
        assert_eq!(core::mem::size_of::<RegisterBlock>(), 0x44);
    }
}
