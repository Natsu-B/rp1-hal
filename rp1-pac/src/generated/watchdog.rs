// Generated from rp1-regdb/data/peripherals/watchdog.toml.
// Do not edit manually.
// Regenerate with: cargo run -p rp1-regdb -- generate

use crate::Reg;

/// RP2040-compatible layout where independently verified; no Rp1Lk provenance remains.
/// Atomic view: Rp1Native alias windows; ownership remains with this logical peripheral.
#[repr(C)]
pub struct RegisterBlock {
    ctrl: CTRL,
    load: LOAD,
    reason: REASON,
    boot_magic: BOOT_MAGIC,
    proc0_encoded_entry: PROC0_ENCODED_ENTRY,
    proc1_encoded_entry: PROC1_ENCODED_ENTRY,
    proc0_initial_sp: PROC0_INITIAL_SP,
    proc1_initial_sp: PROC1_INITIAL_SP,
    scratch: [SCRATCH; 3],
    identification_tag: IDENTIFICATION_TAG,
}

impl RegisterBlock {
    /// ENABLE and countdown were hardware-verified. Pause controls and TRIGGER writes remain unverified.
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn ctrl(&self) -> &CTRL {
        &self.ctrl
    }
    /// Bounded countdown load command; not an ordinary RW register.
    /// Hardware access: read None; write Command.
    /// Command write; ordinary read, write, and modify APIs are not generated.
    #[inline(always)]
    pub const fn load(&self) -> &LOAD {
        &self.load
    }
    /// Last reset reason raw value 0x2 was observed. Bit meanings are compatibility-derived; a watchdog-expiry reset was not induced. \[Evidence: HardwareProven\]
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn reason(&self) -> &REASON {
        &self.reason
    }
    /// Boot ROM start-service magic gate; this word is state-machine input, not general scratch memory.
    /// Hardware access: read Plain; write StateMachineOnly.
    /// State-machine-only register; unrestricted writers are intentionally hidden.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn boot_magic(&self) -> &BOOT_MAGIC {
        &self.boot_magic
    }
    /// Boot ROM start-service proc0 encoded entry; encoding XOR constant is 0xb007c0de.
    /// Hardware access: read Plain; write StateMachineOnly.
    /// State-machine-only register; unrestricted writers are intentionally hidden.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn proc0_encoded_entry(&self) -> &PROC0_ENCODED_ENTRY {
        &self.proc0_encoded_entry
    }
    /// Boot ROM start-service proc1 encoded entry; encoding XOR constant is 0x4ff83f2d.
    /// Hardware access: read Plain; write StateMachineOnly.
    /// State-machine-only register; unrestricted writers are intentionally hidden.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn proc1_encoded_entry(&self) -> &PROC1_ENCODED_ENTRY {
        &self.proc1_encoded_entry
    }
    /// Boot ROM start-service proc0 initial stack pointer; this word is not general scratch memory.
    /// Hardware access: read Plain; write StateMachineOnly.
    /// State-machine-only register; unrestricted writers are intentionally hidden.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn proc0_initial_sp(&self) -> &PROC0_INITIAL_SP {
        &self.proc0_initial_sp
    }
    /// Boot ROM start-service proc1 initial stack pointer; this word is not general scratch memory.
    /// Hardware access: read Plain; write StateMachineOnly.
    /// State-machine-only register; unrestricted writers are intentionally hidden.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn proc1_initial_sp(&self) -> &PROC1_INITIAL_SP {
        &self.proc1_initial_sp
    }
    /// Remaining scratch words; unrestricted firmware use and write behavior are not yet verified.
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    /// Array layout: 3 physical entries, 3 public entries, stride 0x4 bytes.
    #[inline(always)]
    pub fn scratch(&self, index: usize) -> &SCRATCH {
        assert!(index < 3);
        &self.scratch[index]
    }
    #[inline(always)]
    pub fn scratch_array(&self) -> &[SCRATCH] {
        &self.scratch[..3]
    }
    /// Observed raw value 0x57444f47. IDENTIFICATION_TAG is a conservative inference; hardware_write=none is fail-closed PAC policy, not physical write proof.
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn identification_tag(&self) -> &IDENTIFICATION_TAG {
        &self.identification_tag
    }
}

/// ENABLE and countdown were hardware-verified. Pause controls and TRIGGER writes remain unverified.
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type CTRL = Reg<ctrl::CTRL_SPEC>;
/// ENABLE and countdown were hardware-verified. Pause controls and TRIGGER writes remain unverified.
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod ctrl {
    pub struct CTRL_SPEC;
    impl crate::RegisterSpec for CTRL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CTRL_SPEC {}
    pub type R = crate::R<CTRL_SPEC>;
    impl crate::TypedAtomicSet for CTRL_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicSet for CTRL_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::TypedAtomicClear for CTRL_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicClear for CTRL_SPEC {
        const OFFSET: usize = 0x3000;
    }
    pub type AtomicSetW = crate::AtomicSetW<CTRL_SPEC>;
    pub type AtomicClearW = crate::AtomicClearW<CTRL_SPEC>;
    /// Current watchdog countdown value.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod time {
        pub type R = crate::FieldReader<u32>;
    }
    /// Compatibility-derived: pause while JTAG is active; RP1 write behavior untested.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed because this field is inferred.
    pub mod pause_jtag {
        pub type R = crate::BitReader;
    }
    /// Compatibility-derived: pause while processor 0 is debugged; RP1 write behavior untested.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed because this field is inferred.
    pub mod pause_dbg0 {
        pub type R = crate::BitReader;
    }
    /// Compatibility-derived: pause while processor 1 is debugged; RP1 write behavior untested.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed because this field is inferred.
    pub mod pause_dbg1 {
        pub type R = crate::BitReader;
    }
    /// Enable watchdog countdown; bounded enable/disable hardware proof passed.
    /// Field write behavior: Plain.
    pub mod enable {
        pub type R = crate::BitReader;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 30>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 30>;
    }
    /// Force watchdog action; observed in the verified RP1 boot handoff.
    /// Field write behavior: Command.
    /// Typed write API is not exposed because write behavior is not hardware/official verified.
    pub mod trigger {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn time(&self) -> time::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffffff)
        }
        #[inline(always)]
        pub fn pause_jtag(&self) -> pause_jtag::R {
            crate::BitReader::new(((self.bits() >> 24) & 1) != 0)
        }
        #[inline(always)]
        pub fn pause_dbg0(&self) -> pause_dbg0::R {
            crate::BitReader::new(((self.bits() >> 25) & 1) != 0)
        }
        #[inline(always)]
        pub fn pause_dbg1(&self) -> pause_dbg1::R {
            crate::BitReader::new(((self.bits() >> 26) & 1) != 0)
        }
        #[inline(always)]
        pub fn enable(&self) -> enable::R {
            crate::BitReader::new(((self.bits() >> 30) & 1) != 0)
        }
        #[inline(always)]
        pub fn trigger(&self) -> trigger::R {
            crate::BitReader::new(((self.bits() >> 31) & 1) != 0)
        }
    }
    impl AtomicSetW {
        /// Enable watchdog countdown; bounded enable/disable hardware proof passed.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn enable(&mut self) -> enable::ASW<'_> {
            enable::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// Enable watchdog countdown; bounded enable/disable hardware proof passed.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn enable(&mut self) -> enable::ACW<'_> {
            enable::ACW::new(self)
        }
    }
}

/// Bounded countdown load command; not an ordinary RW register.
/// Hardware access: read None; write Command.
/// Command write; ordinary read, write, and modify APIs are not generated.
pub type LOAD = Reg<load::LOAD_SPEC>;
/// Bounded countdown load command; not an ordinary RW register.
/// Hardware access: read None; write Command.
/// Command write; ordinary read, write, and modify APIs are not generated.
pub mod load {
    pub struct LOAD_SPEC;
    impl crate::RegisterSpec for LOAD_SPEC {
        type Ux = u32;
    }
    impl crate::CommandWritable for LOAD_SPEC {}
    pub type CommandW = crate::CommandW<LOAD_SPEC>;
    /// 24-bit reload value.
    /// Field write behavior: Command.
    pub mod value {
        pub type CMDW<'a> = crate::CommandFieldWriter<'a, super::CommandW, 0, 24>;
    }
    impl CommandW {
        #[inline(always)]
        pub fn value(&mut self) -> value::CMDW<'_> {
            value::CMDW::new(self)
        }
    }
}

/// Last reset reason raw value 0x2 was observed. Bit meanings are compatibility-derived; a watchdog-expiry reset was not induced. \[Evidence: HardwareProven\]
/// Hardware access: read Plain; write None.
pub type REASON = Reg<reason::REASON_SPEC>;
/// Last reset reason raw value 0x2 was observed. Bit meanings are compatibility-derived; a watchdog-expiry reset was not induced. \[Evidence: HardwareProven\]
/// Hardware access: read Plain; write None.
pub mod reason {
    pub struct REASON_SPEC;
    impl crate::RegisterSpec for REASON_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for REASON_SPEC {}
    pub type R = crate::R<REASON_SPEC>;
    /// Compatibility-derived TIMER bit; no watchdog-expiry reset was induced.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod timer {
        pub type R = crate::BitReader;
    }
    /// Compatibility-derived FORCE bit; raw bit 1 was observed set in hardware.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod force {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn timer(&self) -> timer::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn force(&self) -> force::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
    }
}

/// Boot ROM start-service magic gate; this word is state-machine input, not general scratch memory.
/// Hardware access: read Plain; write StateMachineOnly.
/// State-machine-only register; unrestricted writers are intentionally hidden.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type BOOT_MAGIC = Reg<boot_magic::BOOT_MAGIC_SPEC>;
/// Boot ROM start-service magic gate; this word is state-machine input, not general scratch memory.
/// Hardware access: read Plain; write StateMachineOnly.
/// State-machine-only register; unrestricted writers are intentionally hidden.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod boot_magic {
    pub struct BOOT_MAGIC_SPEC;
    impl crate::RegisterSpec for BOOT_MAGIC_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for BOOT_MAGIC_SPEC {}
    pub type R = crate::R<BOOT_MAGIC_SPEC>;
    /// Observed Boot ROM start-service magic-gate state.
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

/// Boot ROM start-service proc0 encoded entry; encoding XOR constant is 0xb007c0de.
/// Hardware access: read Plain; write StateMachineOnly.
/// State-machine-only register; unrestricted writers are intentionally hidden.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type PROC0_ENCODED_ENTRY = Reg<proc0_encoded_entry::PROC0_ENCODED_ENTRY_SPEC>;
/// Boot ROM start-service proc0 encoded entry; encoding XOR constant is 0xb007c0de.
/// Hardware access: read Plain; write StateMachineOnly.
/// State-machine-only register; unrestricted writers are intentionally hidden.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod proc0_encoded_entry {
    pub struct PROC0_ENCODED_ENTRY_SPEC;
    impl crate::RegisterSpec for PROC0_ENCODED_ENTRY_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for PROC0_ENCODED_ENTRY_SPEC {}
    pub type R = crate::R<PROC0_ENCODED_ENTRY_SPEC>;
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

/// Boot ROM start-service proc1 encoded entry; encoding XOR constant is 0x4ff83f2d.
/// Hardware access: read Plain; write StateMachineOnly.
/// State-machine-only register; unrestricted writers are intentionally hidden.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type PROC1_ENCODED_ENTRY = Reg<proc1_encoded_entry::PROC1_ENCODED_ENTRY_SPEC>;
/// Boot ROM start-service proc1 encoded entry; encoding XOR constant is 0x4ff83f2d.
/// Hardware access: read Plain; write StateMachineOnly.
/// State-machine-only register; unrestricted writers are intentionally hidden.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod proc1_encoded_entry {
    pub struct PROC1_ENCODED_ENTRY_SPEC;
    impl crate::RegisterSpec for PROC1_ENCODED_ENTRY_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for PROC1_ENCODED_ENTRY_SPEC {}
    pub type R = crate::R<PROC1_ENCODED_ENTRY_SPEC>;
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

/// Boot ROM start-service proc0 initial stack pointer; this word is not general scratch memory.
/// Hardware access: read Plain; write StateMachineOnly.
/// State-machine-only register; unrestricted writers are intentionally hidden.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type PROC0_INITIAL_SP = Reg<proc0_initial_sp::PROC0_INITIAL_SP_SPEC>;
/// Boot ROM start-service proc0 initial stack pointer; this word is not general scratch memory.
/// Hardware access: read Plain; write StateMachineOnly.
/// State-machine-only register; unrestricted writers are intentionally hidden.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod proc0_initial_sp {
    pub struct PROC0_INITIAL_SP_SPEC;
    impl crate::RegisterSpec for PROC0_INITIAL_SP_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for PROC0_INITIAL_SP_SPEC {}
    pub type R = crate::R<PROC0_INITIAL_SP_SPEC>;
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

/// Boot ROM start-service proc1 initial stack pointer; this word is not general scratch memory.
/// Hardware access: read Plain; write StateMachineOnly.
/// State-machine-only register; unrestricted writers are intentionally hidden.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type PROC1_INITIAL_SP = Reg<proc1_initial_sp::PROC1_INITIAL_SP_SPEC>;
/// Boot ROM start-service proc1 initial stack pointer; this word is not general scratch memory.
/// Hardware access: read Plain; write StateMachineOnly.
/// State-machine-only register; unrestricted writers are intentionally hidden.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod proc1_initial_sp {
    pub struct PROC1_INITIAL_SP_SPEC;
    impl crate::RegisterSpec for PROC1_INITIAL_SP_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for PROC1_INITIAL_SP_SPEC {}
    pub type R = crate::R<PROC1_INITIAL_SP_SPEC>;
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

/// Remaining scratch words; unrestricted firmware use and write behavior are not yet verified.
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
/// Array layout: 3 physical entries, 3 public entries, stride 0x4 bytes.
pub type SCRATCH = Reg<scratch::SCRATCH_SPEC>;
/// Remaining scratch words; unrestricted firmware use and write behavior are not yet verified.
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
/// Array layout: 3 physical entries, 3 public entries, stride 0x4 bytes.
pub mod scratch {
    pub struct SCRATCH_SPEC;
    impl crate::RegisterSpec for SCRATCH_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for SCRATCH_SPEC {}
    pub type R = crate::R<SCRATCH_SPEC>;
    /// Read-only PAC exposure until general scratch writes are verified.
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

/// Observed raw value 0x57444f47. IDENTIFICATION_TAG is a conservative inference; hardware_write=none is fail-closed PAC policy, not physical write proof.
/// Hardware access: read Plain; write None.
pub type IDENTIFICATION_TAG = Reg<identification_tag::IDENTIFICATION_TAG_SPEC>;
/// Observed raw value 0x57444f47. IDENTIFICATION_TAG is a conservative inference; hardware_write=none is fail-closed PAC policy, not physical write proof.
/// Hardware access: read Plain; write None.
pub mod identification_tag {
    pub struct IDENTIFICATION_TAG_SPEC;
    impl crate::RegisterSpec for IDENTIFICATION_TAG_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for IDENTIFICATION_TAG_SPEC {}
    pub type R = crate::R<IDENTIFICATION_TAG_SPEC>;
    /// Observed ASCII WDOG value (0x57444F47); this is not the watchdog tick generator. PAC read-only exposure is policy; physical write behavior remains unknown. \[Evidence: HardwareReadProven.\]
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

#[cfg(test)]
mod layout_tests {
    use super::RegisterBlock;
    #[test]
    fn ctrl_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, ctrl), 0x0);
    }
    #[test]
    fn load_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, load), 0x4);
    }
    #[test]
    fn reason_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, reason), 0x8);
    }
    #[test]
    fn boot_magic_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, boot_magic), 0xc);
    }
    #[test]
    fn proc0_encoded_entry_offset() {
        assert_eq!(
            core::mem::offset_of!(RegisterBlock, proc0_encoded_entry),
            0x10
        );
    }
    #[test]
    fn proc1_encoded_entry_offset() {
        assert_eq!(
            core::mem::offset_of!(RegisterBlock, proc1_encoded_entry),
            0x14
        );
    }
    #[test]
    fn proc0_initial_sp_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, proc0_initial_sp), 0x18);
    }
    #[test]
    fn proc1_initial_sp_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, proc1_initial_sp), 0x1c);
    }
    #[test]
    fn scratch_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, scratch), 0x20);
    }
    #[test]
    fn identification_tag_offset() {
        assert_eq!(
            core::mem::offset_of!(RegisterBlock, identification_tag),
            0x2c
        );
    }
    #[test]
    fn register_block_size() {
        assert_eq!(core::mem::size_of::<RegisterBlock>(), 0x30);
    }
}
