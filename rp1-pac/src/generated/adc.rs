// Generated from rp1-regdb/data/peripherals/adc.toml.
// Do not edit manually.
// Regenerate with: cargo run -p rp1-regdb -- generate

use crate::Reg;

/// RP1 ADC_RW register block (12-bit SAR ADC: four external inputs and the temperature sensor). Bus=APB. Atomic alias: normal read/write access.
/// Atomic view: Rp1Native alias windows; ownership remains with this logical peripheral.
#[repr(C)]
pub struct RegisterBlock {
    cs: CS,
    result: RESULT,
    fcs: FCS,
    fifo: FIFO,
    div: DIV,
    intr: INTR,
    inte: INTE,
    intf: INTF,
    ints: INTS,
}

impl RegisterBlock {
    /// Control and status. \[Source: Linux RP1 ADC hwmon driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn cs(&self) -> &CS {
        &self.cs
    }
    /// Result of the most recent conversion. \[Source: Linux RP1 ADC hwmon driver.\]
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn result(&self) -> &RESULT {
        &self.result
    }
    /// FIFO control and status. \[Source: Linux RP1 ADC hwmon driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn fcs(&self) -> &FCS {
        &self.fcs
    }
    /// Conversion result FIFO. \[Source: Linux RP1 ADC hwmon driver.\]
    /// Hardware access: read Fifo; write None.
    #[inline(always)]
    pub const fn fifo(&self) -> &FIFO {
        &self.fifo
    }
    /// Free-running sample period divider. \[Source: Linux RP1 ADC hwmon driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn div(&self) -> &DIV {
        &self.div
    }
    /// Raw interrupts. \[Source: Linux RP1 ADC hwmon driver.\]
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn intr(&self) -> &INTR {
        &self.intr
    }
    /// Interrupt enable (Linux writes 0). \[Source: Linux RP1 ADC hwmon driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn inte(&self) -> &INTE {
        &self.inte
    }
    /// Interrupt force. \[Source: Linux RP1 ADC hwmon driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn intf(&self) -> &INTF {
        &self.intf
    }
    /// Interrupt status after masking and forcing. \[Source: Linux RP1 ADC hwmon driver.\]
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn ints(&self) -> &INTS {
        &self.ints
    }
}

/// Control and status. \[Source: Linux RP1 ADC hwmon driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type CS = Reg<cs::CS_SPEC>;
/// Control and status. \[Source: Linux RP1 ADC hwmon driver.\]
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
    /// Power on the ADC and enable its clock.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed because write behavior is not hardware/official verified.
    pub mod en {
        pub type R = crate::BitReader;
    }
    /// Power on the temperature sensor.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed because write behavior is not hardware/official verified.
    pub mod ts_en {
        pub type R = crate::BitReader;
    }
    /// Start one conversion.
    /// Field write behavior: SelfClearing.
    /// Typed write API is not exposed because write behavior is not hardware/official verified.
    pub mod start_once {
        pub type R = crate::BitReader;
    }
    /// Free-running conversions.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed because write behavior is not hardware/official verified.
    pub mod start_many {
        pub type R = crate::BitReader;
    }
    /// Ready to start a conversion; also set when one completes.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod ready {
        pub type R = crate::BitReader;
    }
    /// The most recent result had a conversion error.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod err {
        pub type R = crate::BitReader;
    }
    /// Some past result had a conversion error.
    /// Field write behavior: WriteOneToClear.
    /// Typed write API is not exposed because write behavior is not hardware/official verified.
    pub mod err_sticky {
        pub type R = crate::BitReader;
    }
    /// Input select: 0..3 external inputs, 4 temperature sensor.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed because write behavior is not hardware/official verified.
    pub mod ainsel {
        pub type R = crate::FieldReader<u32>;
    }
    /// Round-robin input mask for free-running conversions.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed because write behavior is not hardware/official verified.
    pub mod rrobin {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn en(&self) -> en::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn ts_en(&self) -> ts_en::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
        #[inline(always)]
        pub fn start_once(&self) -> start_once::R {
            crate::BitReader::new(((self.bits() >> 2) & 1) != 0)
        }
        #[inline(always)]
        pub fn start_many(&self) -> start_many::R {
            crate::BitReader::new(((self.bits() >> 3) & 1) != 0)
        }
        #[inline(always)]
        pub fn ready(&self) -> ready::R {
            crate::BitReader::new(((self.bits() >> 8) & 1) != 0)
        }
        #[inline(always)]
        pub fn err(&self) -> err::R {
            crate::BitReader::new(((self.bits() >> 9) & 1) != 0)
        }
        #[inline(always)]
        pub fn err_sticky(&self) -> err_sticky::R {
            crate::BitReader::new(((self.bits() >> 10) & 1) != 0)
        }
        #[inline(always)]
        pub fn ainsel(&self) -> ainsel::R {
            crate::FieldReader::new((self.bits() >> 12) & 0x7)
        }
        #[inline(always)]
        pub fn rrobin(&self) -> rrobin::R {
            crate::FieldReader::new((self.bits() >> 16) & 0x1f)
        }
    }
}

/// Result of the most recent conversion. \[Source: Linux RP1 ADC hwmon driver.\]
/// Hardware access: read Plain; write None.
pub type RESULT = Reg<result::RESULT_SPEC>;
/// Result of the most recent conversion. \[Source: Linux RP1 ADC hwmon driver.\]
/// Hardware access: read Plain; write None.
pub mod result {
    pub struct RESULT_SPEC;
    impl crate::RegisterSpec for RESULT_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for RESULT_SPEC {}
    pub type R = crate::R<RESULT_SPEC>;
    /// 12-bit conversion result (full scale 0xfff).
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod result {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn result(&self) -> result::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xfff)
        }
    }
}

/// FIFO control and status. \[Source: Linux RP1 ADC hwmon driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type FCS = Reg<fcs::FCS_SPEC>;
/// FIFO control and status. \[Source: Linux RP1 ADC hwmon driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod fcs {
    pub struct FCS_SPEC;
    impl crate::RegisterSpec for FCS_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for FCS_SPEC {}
    pub type R = crate::R<FCS_SPEC>;
    impl crate::UnsafeWritable for FCS_SPEC {}
    impl crate::UnsafeAtomicSet for FCS_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for FCS_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for FCS_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// Write each completed conversion to the FIFO.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed because write behavior is not hardware/official verified.
    pub mod en {
        pub type R = crate::BitReader;
    }
    /// Right-shift FIFO results to 8 bits.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed because write behavior is not hardware/official verified.
    pub mod shift {
        pub type R = crate::BitReader;
    }
    /// Include the error flag in FIFO entries.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed because write behavior is not hardware/official verified.
    pub mod err {
        pub type R = crate::BitReader;
    }
    /// Enable DMA requests.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed because write behavior is not hardware/official verified.
    pub mod dreq_en {
        pub type R = crate::BitReader;
    }
    /// FIFO empty.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod empty {
        pub type R = crate::BitReader;
    }
    /// FIFO full.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod full {
        pub type R = crate::BitReader;
    }
    /// FIFO underflow.
    /// Field write behavior: WriteOneToClear.
    /// Typed write API is not exposed because write behavior is not hardware/official verified.
    pub mod under {
        pub type R = crate::BitReader;
    }
    /// FIFO overflow.
    /// Field write behavior: WriteOneToClear.
    /// Typed write API is not exposed because write behavior is not hardware/official verified.
    pub mod over {
        pub type R = crate::BitReader;
    }
    /// FIFO fill level.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod level {
        pub type R = crate::FieldReader<u32>;
    }
    /// FIFO threshold for DREQ and interrupt.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed because write behavior is not hardware/official verified.
    pub mod thresh {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn en(&self) -> en::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn shift(&self) -> shift::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
        #[inline(always)]
        pub fn err(&self) -> err::R {
            crate::BitReader::new(((self.bits() >> 2) & 1) != 0)
        }
        #[inline(always)]
        pub fn dreq_en(&self) -> dreq_en::R {
            crate::BitReader::new(((self.bits() >> 3) & 1) != 0)
        }
        #[inline(always)]
        pub fn empty(&self) -> empty::R {
            crate::BitReader::new(((self.bits() >> 8) & 1) != 0)
        }
        #[inline(always)]
        pub fn full(&self) -> full::R {
            crate::BitReader::new(((self.bits() >> 9) & 1) != 0)
        }
        #[inline(always)]
        pub fn under(&self) -> under::R {
            crate::BitReader::new(((self.bits() >> 10) & 1) != 0)
        }
        #[inline(always)]
        pub fn over(&self) -> over::R {
            crate::BitReader::new(((self.bits() >> 11) & 1) != 0)
        }
        #[inline(always)]
        pub fn level(&self) -> level::R {
            crate::FieldReader::new((self.bits() >> 16) & 0xf)
        }
        #[inline(always)]
        pub fn thresh(&self) -> thresh::R {
            crate::FieldReader::new((self.bits() >> 24) & 0xf)
        }
    }
}

/// Conversion result FIFO. \[Source: Linux RP1 ADC hwmon driver.\]
/// Hardware access: read Fifo; write None.
pub type FIFO = Reg<fifo::FIFO_SPEC>;
/// Conversion result FIFO. \[Source: Linux RP1 ADC hwmon driver.\]
/// Hardware access: read Fifo; write None.
pub mod fifo {
    pub struct FIFO_SPEC;
    impl crate::RegisterSpec for FIFO_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for FIFO_SPEC {}
    pub type R = crate::R<FIFO_SPEC>;
    /// Result.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod val {
        pub type R = crate::FieldReader<u32>;
    }
    /// Conversion error flag of this entry.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod err {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn val(&self) -> val::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xfff)
        }
        #[inline(always)]
        pub fn err(&self) -> err::R {
            crate::BitReader::new(((self.bits() >> 15) & 1) != 0)
        }
    }
}

/// Free-running sample period divider. \[Source: Linux RP1 ADC hwmon driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type DIV = Reg<div::DIV_SPEC>;
/// Free-running sample period divider. \[Source: Linux RP1 ADC hwmon driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod div {
    pub struct DIV_SPEC;
    impl crate::RegisterSpec for DIV_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for DIV_SPEC {}
    pub type R = crate::R<DIV_SPEC>;
    impl crate::UnsafeWritable for DIV_SPEC {}
    impl crate::UnsafeAtomicSet for DIV_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for DIV_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for DIV_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// Fractional part.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed because write behavior is not hardware/official verified.
    pub mod frac {
        pub type R = crate::FieldReader<u32>;
    }
    /// Integer part.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed because write behavior is not hardware/official verified.
    pub mod int {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn frac(&self) -> frac::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xff)
        }
        #[inline(always)]
        pub fn int(&self) -> int::R {
            crate::FieldReader::new((self.bits() >> 8) & 0xffff)
        }
    }
}

/// Raw interrupts. \[Source: Linux RP1 ADC hwmon driver.\]
/// Hardware access: read Plain; write None.
pub type INTR = Reg<intr::INTR_SPEC>;
/// Raw interrupts. \[Source: Linux RP1 ADC hwmon driver.\]
/// Hardware access: read Plain; write None.
pub mod intr {
    pub struct INTR_SPEC;
    impl crate::RegisterSpec for INTR_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for INTR_SPEC {}
    pub type R = crate::R<INTR_SPEC>;
}

/// Interrupt enable (Linux writes 0). \[Source: Linux RP1 ADC hwmon driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type INTE = Reg<inte::INTE_SPEC>;
/// Interrupt enable (Linux writes 0). \[Source: Linux RP1 ADC hwmon driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod inte {
    pub struct INTE_SPEC;
    impl crate::RegisterSpec for INTE_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for INTE_SPEC {}
    pub type R = crate::R<INTE_SPEC>;
    impl crate::UnsafeWritable for INTE_SPEC {}
}

/// Interrupt force. \[Source: Linux RP1 ADC hwmon driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type INTF = Reg<intf::INTF_SPEC>;
/// Interrupt force. \[Source: Linux RP1 ADC hwmon driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod intf {
    pub struct INTF_SPEC;
    impl crate::RegisterSpec for INTF_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for INTF_SPEC {}
    pub type R = crate::R<INTF_SPEC>;
    impl crate::UnsafeWritable for INTF_SPEC {}
}

/// Interrupt status after masking and forcing. \[Source: Linux RP1 ADC hwmon driver.\]
/// Hardware access: read Plain; write None.
pub type INTS = Reg<ints::INTS_SPEC>;
/// Interrupt status after masking and forcing. \[Source: Linux RP1 ADC hwmon driver.\]
/// Hardware access: read Plain; write None.
pub mod ints {
    pub struct INTS_SPEC;
    impl crate::RegisterSpec for INTS_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for INTS_SPEC {}
    pub type R = crate::R<INTS_SPEC>;
}

#[cfg(test)]
mod layout_tests {
    use super::RegisterBlock;
    #[test]
    fn cs_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, cs), 0x0);
    }
    #[test]
    fn result_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, result), 0x4);
    }
    #[test]
    fn fcs_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, fcs), 0x8);
    }
    #[test]
    fn fifo_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, fifo), 0xc);
    }
    #[test]
    fn div_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, div), 0x10);
    }
    #[test]
    fn intr_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, intr), 0x14);
    }
    #[test]
    fn inte_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, inte), 0x18);
    }
    #[test]
    fn intf_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, intf), 0x1c);
    }
    #[test]
    fn ints_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, ints), 0x20);
    }
    #[test]
    fn register_block_size() {
        assert_eq!(core::mem::size_of::<RegisterBlock>(), 0x24);
    }
}
