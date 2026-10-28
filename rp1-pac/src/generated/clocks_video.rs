// Generated from rp1-regdb/data/peripherals/clocks-video.toml.
// Do not edit manually.
// Regenerate with: cargo run -p rp1-regdb -- generate

use crate::Reg;

/// RP1 CLOCKS_VIDEO register block with supplemental register semantics. Alias behavior: normal read/write access. Video clock layout recovered from the Linux RP1 common-clock driver. Unknown offsets remain split RAW arrays.
/// Atomic view: Rp1Native alias windows; ownership remains with this logical peripheral.
#[repr(C)]
pub struct RegisterBlock {
    clk_vec_ctrl: CLK_VEC_CTRL,
    clk_vec_div_int: CLK_VEC_DIV_INT,
    _reserved0: [u8; 4],
    clk_vec_sel: CLK_VEC_SEL,
    clk_dpi_ctrl: CLK_DPI_CTRL,
    clk_dpi_div_int: CLK_DPI_DIV_INT,
    _reserved1: [u8; 4],
    clk_dpi_sel: CLK_DPI_SEL,
    clk_mipi0_dpi_ctrl: CLK_MIPI0_DPI_CTRL,
    clk_mipi0_dpi_div_int: CLK_MIPI0_DPI_DIV_INT,
    clk_mipi0_dpi_div_frac: CLK_MIPI0_DPI_DIV_FRAC,
    clk_mipi0_dpi_sel: CLK_MIPI0_DPI_SEL,
    clk_mipi1_dpi_ctrl: CLK_MIPI1_DPI_CTRL,
    clk_mipi1_dpi_div_int: CLK_MIPI1_DPI_DIV_INT,
    clk_mipi1_dpi_div_frac: CLK_MIPI1_DPI_DIV_FRAC,
    clk_mipi1_dpi_sel: CLK_MIPI1_DPI_SEL,
}

impl RegisterBlock {
    /// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn clk_vec_ctrl(&self) -> &CLK_VEC_CTRL {
        &self.clk_vec_ctrl
    }
    /// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn clk_vec_div_int(&self) -> &CLK_VEC_DIV_INT {
        &self.clk_vec_div_int
    }
    /// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn clk_vec_sel(&self) -> &CLK_VEC_SEL {
        &self.clk_vec_sel
    }
    /// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn clk_dpi_ctrl(&self) -> &CLK_DPI_CTRL {
        &self.clk_dpi_ctrl
    }
    /// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn clk_dpi_div_int(&self) -> &CLK_DPI_DIV_INT {
        &self.clk_dpi_div_int
    }
    /// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn clk_dpi_sel(&self) -> &CLK_DPI_SEL {
        &self.clk_dpi_sel
    }
    /// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn clk_mipi0_dpi_ctrl(&self) -> &CLK_MIPI0_DPI_CTRL {
        &self.clk_mipi0_dpi_ctrl
    }
    /// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn clk_mipi0_dpi_div_int(&self) -> &CLK_MIPI0_DPI_DIV_INT {
        &self.clk_mipi0_dpi_div_int
    }
    /// Sixteen-bit fractional divider. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn clk_mipi0_dpi_div_frac(&self) -> &CLK_MIPI0_DPI_DIV_FRAC {
        &self.clk_mipi0_dpi_div_frac
    }
    /// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn clk_mipi0_dpi_sel(&self) -> &CLK_MIPI0_DPI_SEL {
        &self.clk_mipi0_dpi_sel
    }
    /// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn clk_mipi1_dpi_ctrl(&self) -> &CLK_MIPI1_DPI_CTRL {
        &self.clk_mipi1_dpi_ctrl
    }
    /// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn clk_mipi1_dpi_div_int(&self) -> &CLK_MIPI1_DPI_DIV_INT {
        &self.clk_mipi1_dpi_div_int
    }
    /// Sixteen-bit fractional divider. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn clk_mipi1_dpi_div_frac(&self) -> &CLK_MIPI1_DPI_DIV_FRAC {
        &self.clk_mipi1_dpi_div_frac
    }
    /// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn clk_mipi1_dpi_sel(&self) -> &CLK_MIPI1_DPI_SEL {
        &self.clk_mipi1_dpi_sel
    }
}

/// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type CLK_VEC_CTRL = Reg<clk_vec_ctrl::CLK_VEC_CTRL_SPEC>;
/// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod clk_vec_ctrl {
    pub struct CLK_VEC_CTRL_SPEC;
    impl crate::RegisterSpec for CLK_VEC_CTRL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_VEC_CTRL_SPEC {}
    pub type R = crate::R<CLK_VEC_CTRL_SPEC>;
    impl crate::UnsafeWritable for CLK_VEC_CTRL_SPEC {}
    impl crate::UnsafeAtomicSet for CLK_VEC_CTRL_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for CLK_VEC_CTRL_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for CLK_VEC_CTRL_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// Auxiliary parent selector.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod auxsrc {
        pub type R = crate::FieldReader<u32>;
    }
    /// Enable generated clock output.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod enable {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn auxsrc(&self) -> auxsrc::R {
            crate::FieldReader::new((self.bits() >> 5) & 0x1f)
        }
        #[inline(always)]
        pub fn enable(&self) -> enable::R {
            crate::BitReader::new(((self.bits() >> 11) & 1) != 0)
        }
    }
}

/// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type CLK_VEC_DIV_INT = Reg<clk_vec_div_int::CLK_VEC_DIV_INT_SPEC>;
/// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod clk_vec_div_int {
    pub struct CLK_VEC_DIV_INT_SPEC;
    impl crate::RegisterSpec for CLK_VEC_DIV_INT_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_VEC_DIV_INT_SPEC {}
    pub type R = crate::R<CLK_VEC_DIV_INT_SPEC>;
    impl crate::UnsafeWritable for CLK_VEC_DIV_INT_SPEC {}
    impl crate::UnsafeAtomicSet for CLK_VEC_DIV_INT_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for CLK_VEC_DIV_INT_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for CLK_VEC_DIV_INT_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// 8-bit integer divider.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xff)
        }
    }
}

/// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write None.
pub type CLK_VEC_SEL = Reg<clk_vec_sel::CLK_VEC_SEL_SPEC>;
/// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write None.
pub mod clk_vec_sel {
    pub struct CLK_VEC_SEL_SPEC;
    impl crate::RegisterSpec for CLK_VEC_SEL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_VEC_SEL_SPEC {}
    pub type R = crate::R<CLK_VEC_SEL_SPEC>;
    /// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
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

/// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type CLK_DPI_CTRL = Reg<clk_dpi_ctrl::CLK_DPI_CTRL_SPEC>;
/// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod clk_dpi_ctrl {
    pub struct CLK_DPI_CTRL_SPEC;
    impl crate::RegisterSpec for CLK_DPI_CTRL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_DPI_CTRL_SPEC {}
    pub type R = crate::R<CLK_DPI_CTRL_SPEC>;
    impl crate::UnsafeWritable for CLK_DPI_CTRL_SPEC {}
    impl crate::UnsafeAtomicSet for CLK_DPI_CTRL_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for CLK_DPI_CTRL_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for CLK_DPI_CTRL_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// Auxiliary parent selector.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod auxsrc {
        pub type R = crate::FieldReader<u32>;
    }
    /// Enable generated clock output.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod enable {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn auxsrc(&self) -> auxsrc::R {
            crate::FieldReader::new((self.bits() >> 5) & 0x1f)
        }
        #[inline(always)]
        pub fn enable(&self) -> enable::R {
            crate::BitReader::new(((self.bits() >> 11) & 1) != 0)
        }
    }
}

/// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type CLK_DPI_DIV_INT = Reg<clk_dpi_div_int::CLK_DPI_DIV_INT_SPEC>;
/// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod clk_dpi_div_int {
    pub struct CLK_DPI_DIV_INT_SPEC;
    impl crate::RegisterSpec for CLK_DPI_DIV_INT_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_DPI_DIV_INT_SPEC {}
    pub type R = crate::R<CLK_DPI_DIV_INT_SPEC>;
    impl crate::UnsafeWritable for CLK_DPI_DIV_INT_SPEC {}
    impl crate::UnsafeAtomicSet for CLK_DPI_DIV_INT_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for CLK_DPI_DIV_INT_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for CLK_DPI_DIV_INT_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// 8-bit integer divider.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xff)
        }
    }
}

/// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write None.
pub type CLK_DPI_SEL = Reg<clk_dpi_sel::CLK_DPI_SEL_SPEC>;
/// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write None.
pub mod clk_dpi_sel {
    pub struct CLK_DPI_SEL_SPEC;
    impl crate::RegisterSpec for CLK_DPI_SEL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_DPI_SEL_SPEC {}
    pub type R = crate::R<CLK_DPI_SEL_SPEC>;
    /// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
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

/// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type CLK_MIPI0_DPI_CTRL = Reg<clk_mipi0_dpi_ctrl::CLK_MIPI0_DPI_CTRL_SPEC>;
/// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod clk_mipi0_dpi_ctrl {
    pub struct CLK_MIPI0_DPI_CTRL_SPEC;
    impl crate::RegisterSpec for CLK_MIPI0_DPI_CTRL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_MIPI0_DPI_CTRL_SPEC {}
    pub type R = crate::R<CLK_MIPI0_DPI_CTRL_SPEC>;
    impl crate::UnsafeWritable for CLK_MIPI0_DPI_CTRL_SPEC {}
    impl crate::UnsafeAtomicSet for CLK_MIPI0_DPI_CTRL_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for CLK_MIPI0_DPI_CTRL_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for CLK_MIPI0_DPI_CTRL_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// Auxiliary parent selector.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod auxsrc {
        pub type R = crate::FieldReader<u32>;
    }
    /// Enable generated clock output.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod enable {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn auxsrc(&self) -> auxsrc::R {
            crate::FieldReader::new((self.bits() >> 5) & 0x1f)
        }
        #[inline(always)]
        pub fn enable(&self) -> enable::R {
            crate::BitReader::new(((self.bits() >> 11) & 1) != 0)
        }
    }
}

/// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type CLK_MIPI0_DPI_DIV_INT = Reg<clk_mipi0_dpi_div_int::CLK_MIPI0_DPI_DIV_INT_SPEC>;
/// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod clk_mipi0_dpi_div_int {
    pub struct CLK_MIPI0_DPI_DIV_INT_SPEC;
    impl crate::RegisterSpec for CLK_MIPI0_DPI_DIV_INT_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_MIPI0_DPI_DIV_INT_SPEC {}
    pub type R = crate::R<CLK_MIPI0_DPI_DIV_INT_SPEC>;
    impl crate::UnsafeWritable for CLK_MIPI0_DPI_DIV_INT_SPEC {}
    impl crate::UnsafeAtomicSet for CLK_MIPI0_DPI_DIV_INT_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for CLK_MIPI0_DPI_DIV_INT_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for CLK_MIPI0_DPI_DIV_INT_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// 8-bit integer divider.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xff)
        }
    }
}

/// Sixteen-bit fractional divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type CLK_MIPI0_DPI_DIV_FRAC = Reg<clk_mipi0_dpi_div_frac::CLK_MIPI0_DPI_DIV_FRAC_SPEC>;
/// Sixteen-bit fractional divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod clk_mipi0_dpi_div_frac {
    pub struct CLK_MIPI0_DPI_DIV_FRAC_SPEC;
    impl crate::RegisterSpec for CLK_MIPI0_DPI_DIV_FRAC_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_MIPI0_DPI_DIV_FRAC_SPEC {}
    pub type R = crate::R<CLK_MIPI0_DPI_DIV_FRAC_SPEC>;
    impl crate::UnsafeWritable for CLK_MIPI0_DPI_DIV_FRAC_SPEC {}
    impl crate::UnsafeAtomicSet for CLK_MIPI0_DPI_DIV_FRAC_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for CLK_MIPI0_DPI_DIV_FRAC_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for CLK_MIPI0_DPI_DIV_FRAC_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// Fractional divider value.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffff)
        }
    }
}

/// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write None.
pub type CLK_MIPI0_DPI_SEL = Reg<clk_mipi0_dpi_sel::CLK_MIPI0_DPI_SEL_SPEC>;
/// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write None.
pub mod clk_mipi0_dpi_sel {
    pub struct CLK_MIPI0_DPI_SEL_SPEC;
    impl crate::RegisterSpec for CLK_MIPI0_DPI_SEL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_MIPI0_DPI_SEL_SPEC {}
    pub type R = crate::R<CLK_MIPI0_DPI_SEL_SPEC>;
    /// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
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

/// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type CLK_MIPI1_DPI_CTRL = Reg<clk_mipi1_dpi_ctrl::CLK_MIPI1_DPI_CTRL_SPEC>;
/// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod clk_mipi1_dpi_ctrl {
    pub struct CLK_MIPI1_DPI_CTRL_SPEC;
    impl crate::RegisterSpec for CLK_MIPI1_DPI_CTRL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_MIPI1_DPI_CTRL_SPEC {}
    pub type R = crate::R<CLK_MIPI1_DPI_CTRL_SPEC>;
    impl crate::UnsafeWritable for CLK_MIPI1_DPI_CTRL_SPEC {}
    impl crate::UnsafeAtomicSet for CLK_MIPI1_DPI_CTRL_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for CLK_MIPI1_DPI_CTRL_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for CLK_MIPI1_DPI_CTRL_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// Auxiliary parent selector.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod auxsrc {
        pub type R = crate::FieldReader<u32>;
    }
    /// Enable generated clock output.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod enable {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn auxsrc(&self) -> auxsrc::R {
            crate::FieldReader::new((self.bits() >> 5) & 0x1f)
        }
        #[inline(always)]
        pub fn enable(&self) -> enable::R {
            crate::BitReader::new(((self.bits() >> 11) & 1) != 0)
        }
    }
}

/// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type CLK_MIPI1_DPI_DIV_INT = Reg<clk_mipi1_dpi_div_int::CLK_MIPI1_DPI_DIV_INT_SPEC>;
/// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod clk_mipi1_dpi_div_int {
    pub struct CLK_MIPI1_DPI_DIV_INT_SPEC;
    impl crate::RegisterSpec for CLK_MIPI1_DPI_DIV_INT_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_MIPI1_DPI_DIV_INT_SPEC {}
    pub type R = crate::R<CLK_MIPI1_DPI_DIV_INT_SPEC>;
    impl crate::UnsafeWritable for CLK_MIPI1_DPI_DIV_INT_SPEC {}
    impl crate::UnsafeAtomicSet for CLK_MIPI1_DPI_DIV_INT_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for CLK_MIPI1_DPI_DIV_INT_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for CLK_MIPI1_DPI_DIV_INT_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// 8-bit integer divider.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xff)
        }
    }
}

/// Sixteen-bit fractional divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type CLK_MIPI1_DPI_DIV_FRAC = Reg<clk_mipi1_dpi_div_frac::CLK_MIPI1_DPI_DIV_FRAC_SPEC>;
/// Sixteen-bit fractional divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod clk_mipi1_dpi_div_frac {
    pub struct CLK_MIPI1_DPI_DIV_FRAC_SPEC;
    impl crate::RegisterSpec for CLK_MIPI1_DPI_DIV_FRAC_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_MIPI1_DPI_DIV_FRAC_SPEC {}
    pub type R = crate::R<CLK_MIPI1_DPI_DIV_FRAC_SPEC>;
    impl crate::UnsafeWritable for CLK_MIPI1_DPI_DIV_FRAC_SPEC {}
    impl crate::UnsafeAtomicSet for CLK_MIPI1_DPI_DIV_FRAC_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for CLK_MIPI1_DPI_DIV_FRAC_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for CLK_MIPI1_DPI_DIV_FRAC_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// Fractional divider value.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffff)
        }
    }
}

/// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write None.
pub type CLK_MIPI1_DPI_SEL = Reg<clk_mipi1_dpi_sel::CLK_MIPI1_DPI_SEL_SPEC>;
/// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write None.
pub mod clk_mipi1_dpi_sel {
    pub struct CLK_MIPI1_DPI_SEL_SPEC;
    impl crate::RegisterSpec for CLK_MIPI1_DPI_SEL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_MIPI1_DPI_SEL_SPEC {}
    pub type R = crate::R<CLK_MIPI1_DPI_SEL_SPEC>;
    /// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
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
    fn clk_vec_ctrl_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, clk_vec_ctrl), 0x0);
    }
    #[test]
    fn clk_vec_div_int_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, clk_vec_div_int), 0x4);
    }
    #[test]
    fn clk_vec_sel_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, clk_vec_sel), 0xc);
    }
    #[test]
    fn clk_dpi_ctrl_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, clk_dpi_ctrl), 0x10);
    }
    #[test]
    fn clk_dpi_div_int_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, clk_dpi_div_int), 0x14);
    }
    #[test]
    fn clk_dpi_sel_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, clk_dpi_sel), 0x1c);
    }
    #[test]
    fn clk_mipi0_dpi_ctrl_offset() {
        assert_eq!(
            core::mem::offset_of!(RegisterBlock, clk_mipi0_dpi_ctrl),
            0x20
        );
    }
    #[test]
    fn clk_mipi0_dpi_div_int_offset() {
        assert_eq!(
            core::mem::offset_of!(RegisterBlock, clk_mipi0_dpi_div_int),
            0x24
        );
    }
    #[test]
    fn clk_mipi0_dpi_div_frac_offset() {
        assert_eq!(
            core::mem::offset_of!(RegisterBlock, clk_mipi0_dpi_div_frac),
            0x28
        );
    }
    #[test]
    fn clk_mipi0_dpi_sel_offset() {
        assert_eq!(
            core::mem::offset_of!(RegisterBlock, clk_mipi0_dpi_sel),
            0x2c
        );
    }
    #[test]
    fn clk_mipi1_dpi_ctrl_offset() {
        assert_eq!(
            core::mem::offset_of!(RegisterBlock, clk_mipi1_dpi_ctrl),
            0x30
        );
    }
    #[test]
    fn clk_mipi1_dpi_div_int_offset() {
        assert_eq!(
            core::mem::offset_of!(RegisterBlock, clk_mipi1_dpi_div_int),
            0x34
        );
    }
    #[test]
    fn clk_mipi1_dpi_div_frac_offset() {
        assert_eq!(
            core::mem::offset_of!(RegisterBlock, clk_mipi1_dpi_div_frac),
            0x38
        );
    }
    #[test]
    fn clk_mipi1_dpi_sel_offset() {
        assert_eq!(
            core::mem::offset_of!(RegisterBlock, clk_mipi1_dpi_sel),
            0x3c
        );
    }
    #[test]
    fn register_block_size() {
        assert_eq!(core::mem::size_of::<RegisterBlock>(), 0x40);
    }
}
