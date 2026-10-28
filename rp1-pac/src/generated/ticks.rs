// Generated from rp1-regdb/data/peripherals/ticks.toml.
// Do not edit manually.
// Regenerate with: cargo run -p rp1-regdb -- generate

use crate::Reg;

/// RP1 TICKS block at 0x40174000: 8 groups, 12-byte group stride, and 3 registers per group. Boot ROM initialization writes cycle value 0x32 and enables each group; that software initialization is distinct from hardware reset values and does not authorize additional writers.
/// Atomic view: Rp1Native alias windows; ownership remains with this logical peripheral.
#[repr(C)]
pub struct RegisterBlock {
    timer_ctrl: TIMER_CTRL,
    timer_cycles: TIMER_CYCLES,
    timer_count: TIMER_COUNT,
    watchdog_ctrl: WATCHDOG_CTRL,
    watchdog_cycles: WATCHDOG_CYCLES,
    watchdog_count: WATCHDOG_COUNT,
    proc_ctrl: PROC_CTRL,
    proc_cycles: PROC_CYCLES,
    proc_count: PROC_COUNT,
    dma0_ctrl: DMA0_CTRL,
    dma0_cycles: DMA0_CYCLES,
    dma0_count: DMA0_COUNT,
    dma1_ctrl: DMA1_CTRL,
    dma1_cycles: DMA1_CYCLES,
    dma1_count: DMA1_COUNT,
    io_bank0_ctrl: IO_BANK0_CTRL,
    io_bank0_cycles: IO_BANK0_CYCLES,
    io_bank0_count: IO_BANK0_COUNT,
    io_bank1_ctrl: IO_BANK1_CTRL,
    io_bank1_cycles: IO_BANK1_CYCLES,
    io_bank1_count: IO_BANK1_COUNT,
    io_bank2_ctrl: IO_BANK2_CTRL,
    io_bank2_cycles: IO_BANK2_CYCLES,
    io_bank2_count: IO_BANK2_COUNT,
}

impl RegisterBlock {
    /// Controls the tick generator
    /// Hardware access: read Plain; write Plain.
    #[inline(always)]
    pub const fn timer_ctrl(&self) -> &TIMER_CTRL {
        &self.timer_ctrl
    }
    /// Total number of clk_tick cycles before next tick
    /// Boot ROM initialization value 0x32 was Confirmed in scope "Boot ROM TICKS group initialization"; this is not the hardware reset value.
    /// Hardware access: read Plain; write Plain.
    #[inline(always)]
    pub const fn timer_cycles(&self) -> &TIMER_CYCLES {
        &self.timer_cycles
    }
    /// Countdown timer before next tick
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn timer_count(&self) -> &TIMER_COUNT {
        &self.timer_count
    }
    /// Controls the tick generator
    /// Hardware access: read Plain; write Plain.
    #[inline(always)]
    pub const fn watchdog_ctrl(&self) -> &WATCHDOG_CTRL {
        &self.watchdog_ctrl
    }
    /// Total number of clk_tick cycles before next tick
    /// Boot ROM initialization value 0x32 was Confirmed in scope "Boot ROM TICKS group initialization"; this is not the hardware reset value.
    /// Hardware access: read Plain; write Plain.
    #[inline(always)]
    pub const fn watchdog_cycles(&self) -> &WATCHDOG_CYCLES {
        &self.watchdog_cycles
    }
    /// Countdown timer before next tick
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn watchdog_count(&self) -> &WATCHDOG_COUNT {
        &self.watchdog_count
    }
    /// Controls the tick generator
    /// Hardware access: read Plain; write Plain.
    #[inline(always)]
    pub const fn proc_ctrl(&self) -> &PROC_CTRL {
        &self.proc_ctrl
    }
    /// Total number of clk_tick cycles before next tick
    /// Boot ROM initialization value 0x32 was Confirmed in scope "Boot ROM TICKS group initialization"; this is not the hardware reset value.
    /// Hardware access: read Plain; write Plain.
    #[inline(always)]
    pub const fn proc_cycles(&self) -> &PROC_CYCLES {
        &self.proc_cycles
    }
    /// Countdown timer before next tick
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn proc_count(&self) -> &PROC_COUNT {
        &self.proc_count
    }
    /// Controls the tick generator
    /// Hardware access: read Plain; write Plain.
    #[inline(always)]
    pub const fn dma0_ctrl(&self) -> &DMA0_CTRL {
        &self.dma0_ctrl
    }
    /// Total number of clk_tick cycles before next tick
    /// Boot ROM initialization value 0x32 was Confirmed in scope "Boot ROM TICKS group initialization"; this is not the hardware reset value.
    /// Hardware access: read Plain; write Plain.
    #[inline(always)]
    pub const fn dma0_cycles(&self) -> &DMA0_CYCLES {
        &self.dma0_cycles
    }
    /// Countdown timer before next tick
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn dma0_count(&self) -> &DMA0_COUNT {
        &self.dma0_count
    }
    /// Controls the tick generator
    /// Hardware access: read Plain; write Plain.
    #[inline(always)]
    pub const fn dma1_ctrl(&self) -> &DMA1_CTRL {
        &self.dma1_ctrl
    }
    /// Total number of clk_tick cycles before next tick
    /// Boot ROM initialization value 0x32 was Confirmed in scope "Boot ROM TICKS group initialization"; this is not the hardware reset value.
    /// Hardware access: read Plain; write Plain.
    #[inline(always)]
    pub const fn dma1_cycles(&self) -> &DMA1_CYCLES {
        &self.dma1_cycles
    }
    /// Countdown timer before next tick
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn dma1_count(&self) -> &DMA1_COUNT {
        &self.dma1_count
    }
    /// Controls the tick generator
    /// Hardware access: read Plain; write Plain.
    #[inline(always)]
    pub const fn io_bank0_ctrl(&self) -> &IO_BANK0_CTRL {
        &self.io_bank0_ctrl
    }
    /// Total number of clk_tick cycles before next tick
    /// Boot ROM initialization value 0x32 was Confirmed in scope "Boot ROM TICKS group initialization"; this is not the hardware reset value.
    /// Hardware access: read Plain; write Plain.
    #[inline(always)]
    pub const fn io_bank0_cycles(&self) -> &IO_BANK0_CYCLES {
        &self.io_bank0_cycles
    }
    /// Countdown timer before next tick
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn io_bank0_count(&self) -> &IO_BANK0_COUNT {
        &self.io_bank0_count
    }
    /// Controls the tick generator
    /// Hardware access: read Plain; write Plain.
    #[inline(always)]
    pub const fn io_bank1_ctrl(&self) -> &IO_BANK1_CTRL {
        &self.io_bank1_ctrl
    }
    /// Total number of clk_tick cycles before next tick
    /// Boot ROM initialization value 0x32 was Confirmed in scope "Boot ROM TICKS group initialization"; this is not the hardware reset value.
    /// Hardware access: read Plain; write Plain.
    #[inline(always)]
    pub const fn io_bank1_cycles(&self) -> &IO_BANK1_CYCLES {
        &self.io_bank1_cycles
    }
    /// Countdown timer before next tick
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn io_bank1_count(&self) -> &IO_BANK1_COUNT {
        &self.io_bank1_count
    }
    /// Controls the tick generator
    /// Hardware access: read Plain; write Plain.
    #[inline(always)]
    pub const fn io_bank2_ctrl(&self) -> &IO_BANK2_CTRL {
        &self.io_bank2_ctrl
    }
    /// Total number of clk_tick cycles before next tick
    /// Boot ROM initialization value 0x32 was Confirmed in scope "Boot ROM TICKS group initialization"; this is not the hardware reset value.
    /// Hardware access: read Plain; write Plain.
    #[inline(always)]
    pub const fn io_bank2_cycles(&self) -> &IO_BANK2_CYCLES {
        &self.io_bank2_cycles
    }
    /// Countdown timer before next tick
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn io_bank2_count(&self) -> &IO_BANK2_COUNT {
        &self.io_bank2_count
    }
}

/// Controls the tick generator
/// Hardware access: read Plain; write Plain.
pub type TIMER_CTRL = Reg<timer_ctrl::TIMER_CTRL_SPEC>;
/// Controls the tick generator
/// Hardware access: read Plain; write Plain.
pub mod timer_ctrl {
    pub struct TIMER_CTRL_SPEC;
    impl crate::RegisterSpec for TIMER_CTRL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for TIMER_CTRL_SPEC {}
    pub type R = crate::R<TIMER_CTRL_SPEC>;
    impl crate::Modifiable for TIMER_CTRL_SPEC {}
    impl crate::UnsafeWritable for TIMER_CTRL_SPEC {}
    impl crate::ResetValue for TIMER_CTRL_SPEC {
        const RESET_VALUE: u32 = 0x0;
    }
    impl crate::TypedAtomicSet for TIMER_CTRL_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicSet for TIMER_CTRL_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::TypedAtomicClear for TIMER_CTRL_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicClear for TIMER_CTRL_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::TypedAtomicXor for TIMER_CTRL_SPEC {
        const OFFSET: usize = 0x1000;
    }
    impl crate::UnsafeAtomicXor for TIMER_CTRL_SPEC {
        const OFFSET: usize = 0x1000;
    }
    pub type W = crate::W<TIMER_CTRL_SPEC>;
    pub type AtomicSetW = crate::AtomicSetW<TIMER_CTRL_SPEC>;
    pub type AtomicClearW = crate::AtomicClearW<TIMER_CTRL_SPEC>;
    pub type AtomicXorW = crate::AtomicXorW<TIMER_CTRL_SPEC>;
    /// Start/stop tick generation
    /// Field write behavior: Plain.
    pub mod enable {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 0>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 0>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 0>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 0>;
    }
    /// Tick generator is running
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod running {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn enable(&self) -> enable::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn running(&self) -> running::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
    }
    impl W {
        #[inline(always)]
        pub fn enable(&mut self) -> enable::W<'_> {
            enable::W::new(self)
        }
    }
    impl AtomicSetW {
        /// Start/stop tick generation
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn enable(&mut self) -> enable::ASW<'_> {
            enable::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// Start/stop tick generation
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn enable(&mut self) -> enable::ACW<'_> {
            enable::ACW::new(self)
        }
    }
    impl AtomicXorW {
        /// Start/stop tick generation
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn enable(&mut self) -> enable::AXW<'_> {
            enable::AXW::new(self)
        }
    }
}

/// Total number of clk_tick cycles before next tick
/// Boot ROM initialization value 0x32 was Confirmed in scope "Boot ROM TICKS group initialization"; this is not the hardware reset value.
/// Hardware access: read Plain; write Plain.
pub type TIMER_CYCLES = Reg<timer_cycles::TIMER_CYCLES_SPEC>;
/// Total number of clk_tick cycles before next tick
/// Boot ROM initialization value 0x32 was Confirmed in scope "Boot ROM TICKS group initialization"; this is not the hardware reset value.
/// Hardware access: read Plain; write Plain.
pub mod timer_cycles {
    pub struct TIMER_CYCLES_SPEC;
    impl crate::RegisterSpec for TIMER_CYCLES_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for TIMER_CYCLES_SPEC {}
    pub type R = crate::R<TIMER_CYCLES_SPEC>;
    impl crate::Modifiable for TIMER_CYCLES_SPEC {}
    impl crate::UnsafeWritable for TIMER_CYCLES_SPEC {}
    impl crate::ResetValue for TIMER_CYCLES_SPEC {
        const RESET_VALUE: u32 = 0x0;
    }
    impl crate::TypedAtomicSet for TIMER_CYCLES_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicSet for TIMER_CYCLES_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::TypedAtomicClear for TIMER_CYCLES_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicClear for TIMER_CYCLES_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::TypedAtomicXor for TIMER_CYCLES_SPEC {
        const OFFSET: usize = 0x1000;
    }
    impl crate::UnsafeAtomicXor for TIMER_CYCLES_SPEC {
        const OFFSET: usize = 0x1000;
    }
    pub type W = crate::W<TIMER_CYCLES_SPEC>;
    pub type AtomicSetW = crate::AtomicSetW<TIMER_CYCLES_SPEC>;
    pub type AtomicClearW = crate::AtomicClearW<TIMER_CYCLES_SPEC>;
    pub type AtomicXorW = crate::AtomicXorW<TIMER_CYCLES_SPEC>;
    /// Total clk_tick cycles before next tick
    /// Field write behavior: Plain.
    pub mod cycles {
        pub type R = crate::FieldReader<u32>;
        pub type W<'a> = crate::FieldWriter<'a, super::W, 0, 9>;
        pub type ASW<'a> = crate::AtomicSetFieldWriter<'a, super::AtomicSetW, 0, 9>;
        pub type ACW<'a> = crate::AtomicClearFieldWriter<'a, super::AtomicClearW, 0, 9>;
        pub type AXW<'a> = crate::AtomicXorFieldWriter<'a, super::AtomicXorW, 0, 9>;
    }
    impl R {
        #[inline(always)]
        pub fn cycles(&self) -> cycles::R {
            crate::FieldReader::new((self.bits() >> 0) & 0x1ff)
        }
    }
    impl W {
        #[inline(always)]
        pub fn cycles(&mut self) -> cycles::W<'_> {
            cycles::W::new(self)
        }
    }
    impl AtomicSetW {
        /// Total clk_tick cycles before next tick
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn cycles(&mut self) -> cycles::ASW<'_> {
            cycles::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// Total clk_tick cycles before next tick
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn cycles(&mut self) -> cycles::ACW<'_> {
            cycles::ACW::new(self)
        }
    }
    impl AtomicXorW {
        /// Total clk_tick cycles before next tick
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn cycles(&mut self) -> cycles::AXW<'_> {
            cycles::AXW::new(self)
        }
    }
}

/// Countdown timer before next tick
/// Hardware access: read Plain; write None.
pub type TIMER_COUNT = Reg<timer_count::TIMER_COUNT_SPEC>;
/// Countdown timer before next tick
/// Hardware access: read Plain; write None.
pub mod timer_count {
    pub struct TIMER_COUNT_SPEC;
    impl crate::RegisterSpec for TIMER_COUNT_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for TIMER_COUNT_SPEC {}
    pub type R = crate::R<TIMER_COUNT_SPEC>;
    /// Remaining clk_tick cycles before next tick
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod count {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn count(&self) -> count::R {
            crate::FieldReader::new((self.bits() >> 0) & 0x1ff)
        }
    }
}

/// Controls the tick generator
/// Hardware access: read Plain; write Plain.
pub type WATCHDOG_CTRL = Reg<watchdog_ctrl::WATCHDOG_CTRL_SPEC>;
/// Controls the tick generator
/// Hardware access: read Plain; write Plain.
pub mod watchdog_ctrl {
    pub struct WATCHDOG_CTRL_SPEC;
    impl crate::RegisterSpec for WATCHDOG_CTRL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for WATCHDOG_CTRL_SPEC {}
    pub type R = crate::R<WATCHDOG_CTRL_SPEC>;
    impl crate::Modifiable for WATCHDOG_CTRL_SPEC {}
    impl crate::UnsafeWritable for WATCHDOG_CTRL_SPEC {}
    impl crate::ResetValue for WATCHDOG_CTRL_SPEC {
        const RESET_VALUE: u32 = 0x0;
    }
    impl crate::TypedAtomicSet for WATCHDOG_CTRL_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicSet for WATCHDOG_CTRL_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::TypedAtomicClear for WATCHDOG_CTRL_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicClear for WATCHDOG_CTRL_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::TypedAtomicXor for WATCHDOG_CTRL_SPEC {
        const OFFSET: usize = 0x1000;
    }
    impl crate::UnsafeAtomicXor for WATCHDOG_CTRL_SPEC {
        const OFFSET: usize = 0x1000;
    }
    pub type W = crate::W<WATCHDOG_CTRL_SPEC>;
    pub type AtomicSetW = crate::AtomicSetW<WATCHDOG_CTRL_SPEC>;
    pub type AtomicClearW = crate::AtomicClearW<WATCHDOG_CTRL_SPEC>;
    pub type AtomicXorW = crate::AtomicXorW<WATCHDOG_CTRL_SPEC>;
    /// Start/stop tick generation
    /// Field write behavior: Plain.
    pub mod enable {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 0>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 0>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 0>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 0>;
    }
    /// Tick generator is running
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod running {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn enable(&self) -> enable::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn running(&self) -> running::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
    }
    impl W {
        #[inline(always)]
        pub fn enable(&mut self) -> enable::W<'_> {
            enable::W::new(self)
        }
    }
    impl AtomicSetW {
        /// Start/stop tick generation
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn enable(&mut self) -> enable::ASW<'_> {
            enable::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// Start/stop tick generation
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn enable(&mut self) -> enable::ACW<'_> {
            enable::ACW::new(self)
        }
    }
    impl AtomicXorW {
        /// Start/stop tick generation
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn enable(&mut self) -> enable::AXW<'_> {
            enable::AXW::new(self)
        }
    }
}

/// Total number of clk_tick cycles before next tick
/// Boot ROM initialization value 0x32 was Confirmed in scope "Boot ROM TICKS group initialization"; this is not the hardware reset value.
/// Hardware access: read Plain; write Plain.
pub type WATCHDOG_CYCLES = Reg<watchdog_cycles::WATCHDOG_CYCLES_SPEC>;
/// Total number of clk_tick cycles before next tick
/// Boot ROM initialization value 0x32 was Confirmed in scope "Boot ROM TICKS group initialization"; this is not the hardware reset value.
/// Hardware access: read Plain; write Plain.
pub mod watchdog_cycles {
    pub struct WATCHDOG_CYCLES_SPEC;
    impl crate::RegisterSpec for WATCHDOG_CYCLES_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for WATCHDOG_CYCLES_SPEC {}
    pub type R = crate::R<WATCHDOG_CYCLES_SPEC>;
    impl crate::Modifiable for WATCHDOG_CYCLES_SPEC {}
    impl crate::UnsafeWritable for WATCHDOG_CYCLES_SPEC {}
    impl crate::ResetValue for WATCHDOG_CYCLES_SPEC {
        const RESET_VALUE: u32 = 0x0;
    }
    impl crate::TypedAtomicSet for WATCHDOG_CYCLES_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicSet for WATCHDOG_CYCLES_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::TypedAtomicClear for WATCHDOG_CYCLES_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicClear for WATCHDOG_CYCLES_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::TypedAtomicXor for WATCHDOG_CYCLES_SPEC {
        const OFFSET: usize = 0x1000;
    }
    impl crate::UnsafeAtomicXor for WATCHDOG_CYCLES_SPEC {
        const OFFSET: usize = 0x1000;
    }
    pub type W = crate::W<WATCHDOG_CYCLES_SPEC>;
    pub type AtomicSetW = crate::AtomicSetW<WATCHDOG_CYCLES_SPEC>;
    pub type AtomicClearW = crate::AtomicClearW<WATCHDOG_CYCLES_SPEC>;
    pub type AtomicXorW = crate::AtomicXorW<WATCHDOG_CYCLES_SPEC>;
    /// Total clk_tick cycles before next tick
    /// Field write behavior: Plain.
    pub mod cycles {
        pub type R = crate::FieldReader<u32>;
        pub type W<'a> = crate::FieldWriter<'a, super::W, 0, 9>;
        pub type ASW<'a> = crate::AtomicSetFieldWriter<'a, super::AtomicSetW, 0, 9>;
        pub type ACW<'a> = crate::AtomicClearFieldWriter<'a, super::AtomicClearW, 0, 9>;
        pub type AXW<'a> = crate::AtomicXorFieldWriter<'a, super::AtomicXorW, 0, 9>;
    }
    impl R {
        #[inline(always)]
        pub fn cycles(&self) -> cycles::R {
            crate::FieldReader::new((self.bits() >> 0) & 0x1ff)
        }
    }
    impl W {
        #[inline(always)]
        pub fn cycles(&mut self) -> cycles::W<'_> {
            cycles::W::new(self)
        }
    }
    impl AtomicSetW {
        /// Total clk_tick cycles before next tick
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn cycles(&mut self) -> cycles::ASW<'_> {
            cycles::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// Total clk_tick cycles before next tick
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn cycles(&mut self) -> cycles::ACW<'_> {
            cycles::ACW::new(self)
        }
    }
    impl AtomicXorW {
        /// Total clk_tick cycles before next tick
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn cycles(&mut self) -> cycles::AXW<'_> {
            cycles::AXW::new(self)
        }
    }
}

/// Countdown timer before next tick
/// Hardware access: read Plain; write None.
pub type WATCHDOG_COUNT = Reg<watchdog_count::WATCHDOG_COUNT_SPEC>;
/// Countdown timer before next tick
/// Hardware access: read Plain; write None.
pub mod watchdog_count {
    pub struct WATCHDOG_COUNT_SPEC;
    impl crate::RegisterSpec for WATCHDOG_COUNT_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for WATCHDOG_COUNT_SPEC {}
    pub type R = crate::R<WATCHDOG_COUNT_SPEC>;
    /// Remaining clk_tick cycles before next tick
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod count {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn count(&self) -> count::R {
            crate::FieldReader::new((self.bits() >> 0) & 0x1ff)
        }
    }
}

/// Controls the tick generator
/// Hardware access: read Plain; write Plain.
pub type PROC_CTRL = Reg<proc_ctrl::PROC_CTRL_SPEC>;
/// Controls the tick generator
/// Hardware access: read Plain; write Plain.
pub mod proc_ctrl {
    pub struct PROC_CTRL_SPEC;
    impl crate::RegisterSpec for PROC_CTRL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for PROC_CTRL_SPEC {}
    pub type R = crate::R<PROC_CTRL_SPEC>;
    impl crate::Modifiable for PROC_CTRL_SPEC {}
    impl crate::UnsafeWritable for PROC_CTRL_SPEC {}
    impl crate::ResetValue for PROC_CTRL_SPEC {
        const RESET_VALUE: u32 = 0x0;
    }
    impl crate::TypedAtomicSet for PROC_CTRL_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicSet for PROC_CTRL_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::TypedAtomicClear for PROC_CTRL_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicClear for PROC_CTRL_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::TypedAtomicXor for PROC_CTRL_SPEC {
        const OFFSET: usize = 0x1000;
    }
    impl crate::UnsafeAtomicXor for PROC_CTRL_SPEC {
        const OFFSET: usize = 0x1000;
    }
    pub type W = crate::W<PROC_CTRL_SPEC>;
    pub type AtomicSetW = crate::AtomicSetW<PROC_CTRL_SPEC>;
    pub type AtomicClearW = crate::AtomicClearW<PROC_CTRL_SPEC>;
    pub type AtomicXorW = crate::AtomicXorW<PROC_CTRL_SPEC>;
    /// Start/stop tick generation
    /// Field write behavior: Plain.
    pub mod enable {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 0>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 0>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 0>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 0>;
    }
    /// Tick generator is running
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod running {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn enable(&self) -> enable::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn running(&self) -> running::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
    }
    impl W {
        #[inline(always)]
        pub fn enable(&mut self) -> enable::W<'_> {
            enable::W::new(self)
        }
    }
    impl AtomicSetW {
        /// Start/stop tick generation
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn enable(&mut self) -> enable::ASW<'_> {
            enable::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// Start/stop tick generation
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn enable(&mut self) -> enable::ACW<'_> {
            enable::ACW::new(self)
        }
    }
    impl AtomicXorW {
        /// Start/stop tick generation
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn enable(&mut self) -> enable::AXW<'_> {
            enable::AXW::new(self)
        }
    }
}

/// Total number of clk_tick cycles before next tick
/// Boot ROM initialization value 0x32 was Confirmed in scope "Boot ROM TICKS group initialization"; this is not the hardware reset value.
/// Hardware access: read Plain; write Plain.
pub type PROC_CYCLES = Reg<proc_cycles::PROC_CYCLES_SPEC>;
/// Total number of clk_tick cycles before next tick
/// Boot ROM initialization value 0x32 was Confirmed in scope "Boot ROM TICKS group initialization"; this is not the hardware reset value.
/// Hardware access: read Plain; write Plain.
pub mod proc_cycles {
    pub struct PROC_CYCLES_SPEC;
    impl crate::RegisterSpec for PROC_CYCLES_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for PROC_CYCLES_SPEC {}
    pub type R = crate::R<PROC_CYCLES_SPEC>;
    impl crate::Modifiable for PROC_CYCLES_SPEC {}
    impl crate::UnsafeWritable for PROC_CYCLES_SPEC {}
    impl crate::ResetValue for PROC_CYCLES_SPEC {
        const RESET_VALUE: u32 = 0x0;
    }
    impl crate::TypedAtomicSet for PROC_CYCLES_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicSet for PROC_CYCLES_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::TypedAtomicClear for PROC_CYCLES_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicClear for PROC_CYCLES_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::TypedAtomicXor for PROC_CYCLES_SPEC {
        const OFFSET: usize = 0x1000;
    }
    impl crate::UnsafeAtomicXor for PROC_CYCLES_SPEC {
        const OFFSET: usize = 0x1000;
    }
    pub type W = crate::W<PROC_CYCLES_SPEC>;
    pub type AtomicSetW = crate::AtomicSetW<PROC_CYCLES_SPEC>;
    pub type AtomicClearW = crate::AtomicClearW<PROC_CYCLES_SPEC>;
    pub type AtomicXorW = crate::AtomicXorW<PROC_CYCLES_SPEC>;
    /// Total clk_tick cycles before next tick
    /// Field write behavior: Plain.
    pub mod cycles {
        pub type R = crate::FieldReader<u32>;
        pub type W<'a> = crate::FieldWriter<'a, super::W, 0, 9>;
        pub type ASW<'a> = crate::AtomicSetFieldWriter<'a, super::AtomicSetW, 0, 9>;
        pub type ACW<'a> = crate::AtomicClearFieldWriter<'a, super::AtomicClearW, 0, 9>;
        pub type AXW<'a> = crate::AtomicXorFieldWriter<'a, super::AtomicXorW, 0, 9>;
    }
    impl R {
        #[inline(always)]
        pub fn cycles(&self) -> cycles::R {
            crate::FieldReader::new((self.bits() >> 0) & 0x1ff)
        }
    }
    impl W {
        #[inline(always)]
        pub fn cycles(&mut self) -> cycles::W<'_> {
            cycles::W::new(self)
        }
    }
    impl AtomicSetW {
        /// Total clk_tick cycles before next tick
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn cycles(&mut self) -> cycles::ASW<'_> {
            cycles::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// Total clk_tick cycles before next tick
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn cycles(&mut self) -> cycles::ACW<'_> {
            cycles::ACW::new(self)
        }
    }
    impl AtomicXorW {
        /// Total clk_tick cycles before next tick
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn cycles(&mut self) -> cycles::AXW<'_> {
            cycles::AXW::new(self)
        }
    }
}

/// Countdown timer before next tick
/// Hardware access: read Plain; write None.
pub type PROC_COUNT = Reg<proc_count::PROC_COUNT_SPEC>;
/// Countdown timer before next tick
/// Hardware access: read Plain; write None.
pub mod proc_count {
    pub struct PROC_COUNT_SPEC;
    impl crate::RegisterSpec for PROC_COUNT_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for PROC_COUNT_SPEC {}
    pub type R = crate::R<PROC_COUNT_SPEC>;
    /// Remaining clk_tick cycles before next tick
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod count {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn count(&self) -> count::R {
            crate::FieldReader::new((self.bits() >> 0) & 0x1ff)
        }
    }
}

/// Controls the tick generator
/// Hardware access: read Plain; write Plain.
pub type DMA0_CTRL = Reg<dma0_ctrl::DMA0_CTRL_SPEC>;
/// Controls the tick generator
/// Hardware access: read Plain; write Plain.
pub mod dma0_ctrl {
    pub struct DMA0_CTRL_SPEC;
    impl crate::RegisterSpec for DMA0_CTRL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for DMA0_CTRL_SPEC {}
    pub type R = crate::R<DMA0_CTRL_SPEC>;
    impl crate::Modifiable for DMA0_CTRL_SPEC {}
    impl crate::UnsafeWritable for DMA0_CTRL_SPEC {}
    impl crate::ResetValue for DMA0_CTRL_SPEC {
        const RESET_VALUE: u32 = 0x0;
    }
    impl crate::TypedAtomicSet for DMA0_CTRL_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicSet for DMA0_CTRL_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::TypedAtomicClear for DMA0_CTRL_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicClear for DMA0_CTRL_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::TypedAtomicXor for DMA0_CTRL_SPEC {
        const OFFSET: usize = 0x1000;
    }
    impl crate::UnsafeAtomicXor for DMA0_CTRL_SPEC {
        const OFFSET: usize = 0x1000;
    }
    pub type W = crate::W<DMA0_CTRL_SPEC>;
    pub type AtomicSetW = crate::AtomicSetW<DMA0_CTRL_SPEC>;
    pub type AtomicClearW = crate::AtomicClearW<DMA0_CTRL_SPEC>;
    pub type AtomicXorW = crate::AtomicXorW<DMA0_CTRL_SPEC>;
    /// Start/stop tick generation
    /// Field write behavior: Plain.
    pub mod enable {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 0>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 0>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 0>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 0>;
    }
    /// Tick generator is running
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod running {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn enable(&self) -> enable::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn running(&self) -> running::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
    }
    impl W {
        #[inline(always)]
        pub fn enable(&mut self) -> enable::W<'_> {
            enable::W::new(self)
        }
    }
    impl AtomicSetW {
        /// Start/stop tick generation
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn enable(&mut self) -> enable::ASW<'_> {
            enable::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// Start/stop tick generation
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn enable(&mut self) -> enable::ACW<'_> {
            enable::ACW::new(self)
        }
    }
    impl AtomicXorW {
        /// Start/stop tick generation
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn enable(&mut self) -> enable::AXW<'_> {
            enable::AXW::new(self)
        }
    }
}

/// Total number of clk_tick cycles before next tick
/// Boot ROM initialization value 0x32 was Confirmed in scope "Boot ROM TICKS group initialization"; this is not the hardware reset value.
/// Hardware access: read Plain; write Plain.
pub type DMA0_CYCLES = Reg<dma0_cycles::DMA0_CYCLES_SPEC>;
/// Total number of clk_tick cycles before next tick
/// Boot ROM initialization value 0x32 was Confirmed in scope "Boot ROM TICKS group initialization"; this is not the hardware reset value.
/// Hardware access: read Plain; write Plain.
pub mod dma0_cycles {
    pub struct DMA0_CYCLES_SPEC;
    impl crate::RegisterSpec for DMA0_CYCLES_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for DMA0_CYCLES_SPEC {}
    pub type R = crate::R<DMA0_CYCLES_SPEC>;
    impl crate::Modifiable for DMA0_CYCLES_SPEC {}
    impl crate::UnsafeWritable for DMA0_CYCLES_SPEC {}
    impl crate::ResetValue for DMA0_CYCLES_SPEC {
        const RESET_VALUE: u32 = 0x0;
    }
    impl crate::TypedAtomicSet for DMA0_CYCLES_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicSet for DMA0_CYCLES_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::TypedAtomicClear for DMA0_CYCLES_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicClear for DMA0_CYCLES_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::TypedAtomicXor for DMA0_CYCLES_SPEC {
        const OFFSET: usize = 0x1000;
    }
    impl crate::UnsafeAtomicXor for DMA0_CYCLES_SPEC {
        const OFFSET: usize = 0x1000;
    }
    pub type W = crate::W<DMA0_CYCLES_SPEC>;
    pub type AtomicSetW = crate::AtomicSetW<DMA0_CYCLES_SPEC>;
    pub type AtomicClearW = crate::AtomicClearW<DMA0_CYCLES_SPEC>;
    pub type AtomicXorW = crate::AtomicXorW<DMA0_CYCLES_SPEC>;
    /// Total clk_tick cycles before next tick
    /// Field write behavior: Plain.
    pub mod cycles {
        pub type R = crate::FieldReader<u32>;
        pub type W<'a> = crate::FieldWriter<'a, super::W, 0, 9>;
        pub type ASW<'a> = crate::AtomicSetFieldWriter<'a, super::AtomicSetW, 0, 9>;
        pub type ACW<'a> = crate::AtomicClearFieldWriter<'a, super::AtomicClearW, 0, 9>;
        pub type AXW<'a> = crate::AtomicXorFieldWriter<'a, super::AtomicXorW, 0, 9>;
    }
    impl R {
        #[inline(always)]
        pub fn cycles(&self) -> cycles::R {
            crate::FieldReader::new((self.bits() >> 0) & 0x1ff)
        }
    }
    impl W {
        #[inline(always)]
        pub fn cycles(&mut self) -> cycles::W<'_> {
            cycles::W::new(self)
        }
    }
    impl AtomicSetW {
        /// Total clk_tick cycles before next tick
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn cycles(&mut self) -> cycles::ASW<'_> {
            cycles::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// Total clk_tick cycles before next tick
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn cycles(&mut self) -> cycles::ACW<'_> {
            cycles::ACW::new(self)
        }
    }
    impl AtomicXorW {
        /// Total clk_tick cycles before next tick
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn cycles(&mut self) -> cycles::AXW<'_> {
            cycles::AXW::new(self)
        }
    }
}

/// Countdown timer before next tick
/// Hardware access: read Plain; write None.
pub type DMA0_COUNT = Reg<dma0_count::DMA0_COUNT_SPEC>;
/// Countdown timer before next tick
/// Hardware access: read Plain; write None.
pub mod dma0_count {
    pub struct DMA0_COUNT_SPEC;
    impl crate::RegisterSpec for DMA0_COUNT_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for DMA0_COUNT_SPEC {}
    pub type R = crate::R<DMA0_COUNT_SPEC>;
    /// Remaining clk_tick cycles before next tick
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod count {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn count(&self) -> count::R {
            crate::FieldReader::new((self.bits() >> 0) & 0x1ff)
        }
    }
}

/// Controls the tick generator
/// Hardware access: read Plain; write Plain.
pub type DMA1_CTRL = Reg<dma1_ctrl::DMA1_CTRL_SPEC>;
/// Controls the tick generator
/// Hardware access: read Plain; write Plain.
pub mod dma1_ctrl {
    pub struct DMA1_CTRL_SPEC;
    impl crate::RegisterSpec for DMA1_CTRL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for DMA1_CTRL_SPEC {}
    pub type R = crate::R<DMA1_CTRL_SPEC>;
    impl crate::Modifiable for DMA1_CTRL_SPEC {}
    impl crate::UnsafeWritable for DMA1_CTRL_SPEC {}
    impl crate::ResetValue for DMA1_CTRL_SPEC {
        const RESET_VALUE: u32 = 0x0;
    }
    impl crate::TypedAtomicSet for DMA1_CTRL_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicSet for DMA1_CTRL_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::TypedAtomicClear for DMA1_CTRL_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicClear for DMA1_CTRL_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::TypedAtomicXor for DMA1_CTRL_SPEC {
        const OFFSET: usize = 0x1000;
    }
    impl crate::UnsafeAtomicXor for DMA1_CTRL_SPEC {
        const OFFSET: usize = 0x1000;
    }
    pub type W = crate::W<DMA1_CTRL_SPEC>;
    pub type AtomicSetW = crate::AtomicSetW<DMA1_CTRL_SPEC>;
    pub type AtomicClearW = crate::AtomicClearW<DMA1_CTRL_SPEC>;
    pub type AtomicXorW = crate::AtomicXorW<DMA1_CTRL_SPEC>;
    /// Start/stop tick generation
    /// Field write behavior: Plain.
    pub mod enable {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 0>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 0>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 0>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 0>;
    }
    /// Tick generator is running
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod running {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn enable(&self) -> enable::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn running(&self) -> running::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
    }
    impl W {
        #[inline(always)]
        pub fn enable(&mut self) -> enable::W<'_> {
            enable::W::new(self)
        }
    }
    impl AtomicSetW {
        /// Start/stop tick generation
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn enable(&mut self) -> enable::ASW<'_> {
            enable::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// Start/stop tick generation
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn enable(&mut self) -> enable::ACW<'_> {
            enable::ACW::new(self)
        }
    }
    impl AtomicXorW {
        /// Start/stop tick generation
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn enable(&mut self) -> enable::AXW<'_> {
            enable::AXW::new(self)
        }
    }
}

/// Total number of clk_tick cycles before next tick
/// Boot ROM initialization value 0x32 was Confirmed in scope "Boot ROM TICKS group initialization"; this is not the hardware reset value.
/// Hardware access: read Plain; write Plain.
pub type DMA1_CYCLES = Reg<dma1_cycles::DMA1_CYCLES_SPEC>;
/// Total number of clk_tick cycles before next tick
/// Boot ROM initialization value 0x32 was Confirmed in scope "Boot ROM TICKS group initialization"; this is not the hardware reset value.
/// Hardware access: read Plain; write Plain.
pub mod dma1_cycles {
    pub struct DMA1_CYCLES_SPEC;
    impl crate::RegisterSpec for DMA1_CYCLES_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for DMA1_CYCLES_SPEC {}
    pub type R = crate::R<DMA1_CYCLES_SPEC>;
    impl crate::Modifiable for DMA1_CYCLES_SPEC {}
    impl crate::UnsafeWritable for DMA1_CYCLES_SPEC {}
    impl crate::ResetValue for DMA1_CYCLES_SPEC {
        const RESET_VALUE: u32 = 0x0;
    }
    impl crate::TypedAtomicSet for DMA1_CYCLES_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicSet for DMA1_CYCLES_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::TypedAtomicClear for DMA1_CYCLES_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicClear for DMA1_CYCLES_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::TypedAtomicXor for DMA1_CYCLES_SPEC {
        const OFFSET: usize = 0x1000;
    }
    impl crate::UnsafeAtomicXor for DMA1_CYCLES_SPEC {
        const OFFSET: usize = 0x1000;
    }
    pub type W = crate::W<DMA1_CYCLES_SPEC>;
    pub type AtomicSetW = crate::AtomicSetW<DMA1_CYCLES_SPEC>;
    pub type AtomicClearW = crate::AtomicClearW<DMA1_CYCLES_SPEC>;
    pub type AtomicXorW = crate::AtomicXorW<DMA1_CYCLES_SPEC>;
    /// Total clk_tick cycles before next tick
    /// Field write behavior: Plain.
    pub mod cycles {
        pub type R = crate::FieldReader<u32>;
        pub type W<'a> = crate::FieldWriter<'a, super::W, 0, 9>;
        pub type ASW<'a> = crate::AtomicSetFieldWriter<'a, super::AtomicSetW, 0, 9>;
        pub type ACW<'a> = crate::AtomicClearFieldWriter<'a, super::AtomicClearW, 0, 9>;
        pub type AXW<'a> = crate::AtomicXorFieldWriter<'a, super::AtomicXorW, 0, 9>;
    }
    impl R {
        #[inline(always)]
        pub fn cycles(&self) -> cycles::R {
            crate::FieldReader::new((self.bits() >> 0) & 0x1ff)
        }
    }
    impl W {
        #[inline(always)]
        pub fn cycles(&mut self) -> cycles::W<'_> {
            cycles::W::new(self)
        }
    }
    impl AtomicSetW {
        /// Total clk_tick cycles before next tick
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn cycles(&mut self) -> cycles::ASW<'_> {
            cycles::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// Total clk_tick cycles before next tick
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn cycles(&mut self) -> cycles::ACW<'_> {
            cycles::ACW::new(self)
        }
    }
    impl AtomicXorW {
        /// Total clk_tick cycles before next tick
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn cycles(&mut self) -> cycles::AXW<'_> {
            cycles::AXW::new(self)
        }
    }
}

/// Countdown timer before next tick
/// Hardware access: read Plain; write None.
pub type DMA1_COUNT = Reg<dma1_count::DMA1_COUNT_SPEC>;
/// Countdown timer before next tick
/// Hardware access: read Plain; write None.
pub mod dma1_count {
    pub struct DMA1_COUNT_SPEC;
    impl crate::RegisterSpec for DMA1_COUNT_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for DMA1_COUNT_SPEC {}
    pub type R = crate::R<DMA1_COUNT_SPEC>;
    /// Remaining clk_tick cycles before next tick
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod count {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn count(&self) -> count::R {
            crate::FieldReader::new((self.bits() >> 0) & 0x1ff)
        }
    }
}

/// Controls the tick generator
/// Hardware access: read Plain; write Plain.
pub type IO_BANK0_CTRL = Reg<io_bank0_ctrl::IO_BANK0_CTRL_SPEC>;
/// Controls the tick generator
/// Hardware access: read Plain; write Plain.
pub mod io_bank0_ctrl {
    pub struct IO_BANK0_CTRL_SPEC;
    impl crate::RegisterSpec for IO_BANK0_CTRL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for IO_BANK0_CTRL_SPEC {}
    pub type R = crate::R<IO_BANK0_CTRL_SPEC>;
    impl crate::Modifiable for IO_BANK0_CTRL_SPEC {}
    impl crate::UnsafeWritable for IO_BANK0_CTRL_SPEC {}
    impl crate::ResetValue for IO_BANK0_CTRL_SPEC {
        const RESET_VALUE: u32 = 0x0;
    }
    impl crate::TypedAtomicSet for IO_BANK0_CTRL_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicSet for IO_BANK0_CTRL_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::TypedAtomicClear for IO_BANK0_CTRL_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicClear for IO_BANK0_CTRL_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::TypedAtomicXor for IO_BANK0_CTRL_SPEC {
        const OFFSET: usize = 0x1000;
    }
    impl crate::UnsafeAtomicXor for IO_BANK0_CTRL_SPEC {
        const OFFSET: usize = 0x1000;
    }
    pub type W = crate::W<IO_BANK0_CTRL_SPEC>;
    pub type AtomicSetW = crate::AtomicSetW<IO_BANK0_CTRL_SPEC>;
    pub type AtomicClearW = crate::AtomicClearW<IO_BANK0_CTRL_SPEC>;
    pub type AtomicXorW = crate::AtomicXorW<IO_BANK0_CTRL_SPEC>;
    /// Start/stop tick generation
    /// Field write behavior: Plain.
    pub mod enable {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 0>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 0>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 0>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 0>;
    }
    /// Tick generator is running
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod running {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn enable(&self) -> enable::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn running(&self) -> running::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
    }
    impl W {
        #[inline(always)]
        pub fn enable(&mut self) -> enable::W<'_> {
            enable::W::new(self)
        }
    }
    impl AtomicSetW {
        /// Start/stop tick generation
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn enable(&mut self) -> enable::ASW<'_> {
            enable::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// Start/stop tick generation
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn enable(&mut self) -> enable::ACW<'_> {
            enable::ACW::new(self)
        }
    }
    impl AtomicXorW {
        /// Start/stop tick generation
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn enable(&mut self) -> enable::AXW<'_> {
            enable::AXW::new(self)
        }
    }
}

/// Total number of clk_tick cycles before next tick
/// Boot ROM initialization value 0x32 was Confirmed in scope "Boot ROM TICKS group initialization"; this is not the hardware reset value.
/// Hardware access: read Plain; write Plain.
pub type IO_BANK0_CYCLES = Reg<io_bank0_cycles::IO_BANK0_CYCLES_SPEC>;
/// Total number of clk_tick cycles before next tick
/// Boot ROM initialization value 0x32 was Confirmed in scope "Boot ROM TICKS group initialization"; this is not the hardware reset value.
/// Hardware access: read Plain; write Plain.
pub mod io_bank0_cycles {
    pub struct IO_BANK0_CYCLES_SPEC;
    impl crate::RegisterSpec for IO_BANK0_CYCLES_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for IO_BANK0_CYCLES_SPEC {}
    pub type R = crate::R<IO_BANK0_CYCLES_SPEC>;
    impl crate::Modifiable for IO_BANK0_CYCLES_SPEC {}
    impl crate::UnsafeWritable for IO_BANK0_CYCLES_SPEC {}
    impl crate::ResetValue for IO_BANK0_CYCLES_SPEC {
        const RESET_VALUE: u32 = 0x0;
    }
    impl crate::TypedAtomicSet for IO_BANK0_CYCLES_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicSet for IO_BANK0_CYCLES_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::TypedAtomicClear for IO_BANK0_CYCLES_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicClear for IO_BANK0_CYCLES_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::TypedAtomicXor for IO_BANK0_CYCLES_SPEC {
        const OFFSET: usize = 0x1000;
    }
    impl crate::UnsafeAtomicXor for IO_BANK0_CYCLES_SPEC {
        const OFFSET: usize = 0x1000;
    }
    pub type W = crate::W<IO_BANK0_CYCLES_SPEC>;
    pub type AtomicSetW = crate::AtomicSetW<IO_BANK0_CYCLES_SPEC>;
    pub type AtomicClearW = crate::AtomicClearW<IO_BANK0_CYCLES_SPEC>;
    pub type AtomicXorW = crate::AtomicXorW<IO_BANK0_CYCLES_SPEC>;
    /// Total clk_tick cycles before next tick
    /// Field write behavior: Plain.
    pub mod cycles {
        pub type R = crate::FieldReader<u32>;
        pub type W<'a> = crate::FieldWriter<'a, super::W, 0, 28>;
        pub type ASW<'a> = crate::AtomicSetFieldWriter<'a, super::AtomicSetW, 0, 28>;
        pub type ACW<'a> = crate::AtomicClearFieldWriter<'a, super::AtomicClearW, 0, 28>;
        pub type AXW<'a> = crate::AtomicXorFieldWriter<'a, super::AtomicXorW, 0, 28>;
    }
    impl R {
        #[inline(always)]
        pub fn cycles(&self) -> cycles::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xfffffff)
        }
    }
    impl W {
        #[inline(always)]
        pub fn cycles(&mut self) -> cycles::W<'_> {
            cycles::W::new(self)
        }
    }
    impl AtomicSetW {
        /// Total clk_tick cycles before next tick
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn cycles(&mut self) -> cycles::ASW<'_> {
            cycles::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// Total clk_tick cycles before next tick
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn cycles(&mut self) -> cycles::ACW<'_> {
            cycles::ACW::new(self)
        }
    }
    impl AtomicXorW {
        /// Total clk_tick cycles before next tick
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn cycles(&mut self) -> cycles::AXW<'_> {
            cycles::AXW::new(self)
        }
    }
}

/// Countdown timer before next tick
/// Hardware access: read Plain; write None.
pub type IO_BANK0_COUNT = Reg<io_bank0_count::IO_BANK0_COUNT_SPEC>;
/// Countdown timer before next tick
/// Hardware access: read Plain; write None.
pub mod io_bank0_count {
    pub struct IO_BANK0_COUNT_SPEC;
    impl crate::RegisterSpec for IO_BANK0_COUNT_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for IO_BANK0_COUNT_SPEC {}
    pub type R = crate::R<IO_BANK0_COUNT_SPEC>;
    /// Remaining clk_tick cycles before next tick
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod count {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn count(&self) -> count::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xfffffff)
        }
    }
}

/// Controls the tick generator
/// Hardware access: read Plain; write Plain.
pub type IO_BANK1_CTRL = Reg<io_bank1_ctrl::IO_BANK1_CTRL_SPEC>;
/// Controls the tick generator
/// Hardware access: read Plain; write Plain.
pub mod io_bank1_ctrl {
    pub struct IO_BANK1_CTRL_SPEC;
    impl crate::RegisterSpec for IO_BANK1_CTRL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for IO_BANK1_CTRL_SPEC {}
    pub type R = crate::R<IO_BANK1_CTRL_SPEC>;
    impl crate::Modifiable for IO_BANK1_CTRL_SPEC {}
    impl crate::UnsafeWritable for IO_BANK1_CTRL_SPEC {}
    impl crate::ResetValue for IO_BANK1_CTRL_SPEC {
        const RESET_VALUE: u32 = 0x0;
    }
    impl crate::TypedAtomicSet for IO_BANK1_CTRL_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicSet for IO_BANK1_CTRL_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::TypedAtomicClear for IO_BANK1_CTRL_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicClear for IO_BANK1_CTRL_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::TypedAtomicXor for IO_BANK1_CTRL_SPEC {
        const OFFSET: usize = 0x1000;
    }
    impl crate::UnsafeAtomicXor for IO_BANK1_CTRL_SPEC {
        const OFFSET: usize = 0x1000;
    }
    pub type W = crate::W<IO_BANK1_CTRL_SPEC>;
    pub type AtomicSetW = crate::AtomicSetW<IO_BANK1_CTRL_SPEC>;
    pub type AtomicClearW = crate::AtomicClearW<IO_BANK1_CTRL_SPEC>;
    pub type AtomicXorW = crate::AtomicXorW<IO_BANK1_CTRL_SPEC>;
    /// Start/stop tick generation
    /// Field write behavior: Plain.
    pub mod enable {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 0>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 0>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 0>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 0>;
    }
    /// Tick generator is running
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod running {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn enable(&self) -> enable::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn running(&self) -> running::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
    }
    impl W {
        #[inline(always)]
        pub fn enable(&mut self) -> enable::W<'_> {
            enable::W::new(self)
        }
    }
    impl AtomicSetW {
        /// Start/stop tick generation
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn enable(&mut self) -> enable::ASW<'_> {
            enable::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// Start/stop tick generation
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn enable(&mut self) -> enable::ACW<'_> {
            enable::ACW::new(self)
        }
    }
    impl AtomicXorW {
        /// Start/stop tick generation
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn enable(&mut self) -> enable::AXW<'_> {
            enable::AXW::new(self)
        }
    }
}

/// Total number of clk_tick cycles before next tick
/// Boot ROM initialization value 0x32 was Confirmed in scope "Boot ROM TICKS group initialization"; this is not the hardware reset value.
/// Hardware access: read Plain; write Plain.
pub type IO_BANK1_CYCLES = Reg<io_bank1_cycles::IO_BANK1_CYCLES_SPEC>;
/// Total number of clk_tick cycles before next tick
/// Boot ROM initialization value 0x32 was Confirmed in scope "Boot ROM TICKS group initialization"; this is not the hardware reset value.
/// Hardware access: read Plain; write Plain.
pub mod io_bank1_cycles {
    pub struct IO_BANK1_CYCLES_SPEC;
    impl crate::RegisterSpec for IO_BANK1_CYCLES_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for IO_BANK1_CYCLES_SPEC {}
    pub type R = crate::R<IO_BANK1_CYCLES_SPEC>;
    impl crate::Modifiable for IO_BANK1_CYCLES_SPEC {}
    impl crate::UnsafeWritable for IO_BANK1_CYCLES_SPEC {}
    impl crate::ResetValue for IO_BANK1_CYCLES_SPEC {
        const RESET_VALUE: u32 = 0x0;
    }
    impl crate::TypedAtomicSet for IO_BANK1_CYCLES_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicSet for IO_BANK1_CYCLES_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::TypedAtomicClear for IO_BANK1_CYCLES_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicClear for IO_BANK1_CYCLES_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::TypedAtomicXor for IO_BANK1_CYCLES_SPEC {
        const OFFSET: usize = 0x1000;
    }
    impl crate::UnsafeAtomicXor for IO_BANK1_CYCLES_SPEC {
        const OFFSET: usize = 0x1000;
    }
    pub type W = crate::W<IO_BANK1_CYCLES_SPEC>;
    pub type AtomicSetW = crate::AtomicSetW<IO_BANK1_CYCLES_SPEC>;
    pub type AtomicClearW = crate::AtomicClearW<IO_BANK1_CYCLES_SPEC>;
    pub type AtomicXorW = crate::AtomicXorW<IO_BANK1_CYCLES_SPEC>;
    /// Total clk_tick cycles before next tick
    /// Field write behavior: Plain.
    pub mod cycles {
        pub type R = crate::FieldReader<u32>;
        pub type W<'a> = crate::FieldWriter<'a, super::W, 0, 28>;
        pub type ASW<'a> = crate::AtomicSetFieldWriter<'a, super::AtomicSetW, 0, 28>;
        pub type ACW<'a> = crate::AtomicClearFieldWriter<'a, super::AtomicClearW, 0, 28>;
        pub type AXW<'a> = crate::AtomicXorFieldWriter<'a, super::AtomicXorW, 0, 28>;
    }
    impl R {
        #[inline(always)]
        pub fn cycles(&self) -> cycles::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xfffffff)
        }
    }
    impl W {
        #[inline(always)]
        pub fn cycles(&mut self) -> cycles::W<'_> {
            cycles::W::new(self)
        }
    }
    impl AtomicSetW {
        /// Total clk_tick cycles before next tick
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn cycles(&mut self) -> cycles::ASW<'_> {
            cycles::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// Total clk_tick cycles before next tick
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn cycles(&mut self) -> cycles::ACW<'_> {
            cycles::ACW::new(self)
        }
    }
    impl AtomicXorW {
        /// Total clk_tick cycles before next tick
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn cycles(&mut self) -> cycles::AXW<'_> {
            cycles::AXW::new(self)
        }
    }
}

/// Countdown timer before next tick
/// Hardware access: read Plain; write None.
pub type IO_BANK1_COUNT = Reg<io_bank1_count::IO_BANK1_COUNT_SPEC>;
/// Countdown timer before next tick
/// Hardware access: read Plain; write None.
pub mod io_bank1_count {
    pub struct IO_BANK1_COUNT_SPEC;
    impl crate::RegisterSpec for IO_BANK1_COUNT_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for IO_BANK1_COUNT_SPEC {}
    pub type R = crate::R<IO_BANK1_COUNT_SPEC>;
    /// Remaining clk_tick cycles before next tick
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod count {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn count(&self) -> count::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xfffffff)
        }
    }
}

/// Controls the tick generator
/// Hardware access: read Plain; write Plain.
pub type IO_BANK2_CTRL = Reg<io_bank2_ctrl::IO_BANK2_CTRL_SPEC>;
/// Controls the tick generator
/// Hardware access: read Plain; write Plain.
pub mod io_bank2_ctrl {
    pub struct IO_BANK2_CTRL_SPEC;
    impl crate::RegisterSpec for IO_BANK2_CTRL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for IO_BANK2_CTRL_SPEC {}
    pub type R = crate::R<IO_BANK2_CTRL_SPEC>;
    impl crate::Modifiable for IO_BANK2_CTRL_SPEC {}
    impl crate::UnsafeWritable for IO_BANK2_CTRL_SPEC {}
    impl crate::ResetValue for IO_BANK2_CTRL_SPEC {
        const RESET_VALUE: u32 = 0x0;
    }
    impl crate::TypedAtomicSet for IO_BANK2_CTRL_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicSet for IO_BANK2_CTRL_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::TypedAtomicClear for IO_BANK2_CTRL_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicClear for IO_BANK2_CTRL_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::TypedAtomicXor for IO_BANK2_CTRL_SPEC {
        const OFFSET: usize = 0x1000;
    }
    impl crate::UnsafeAtomicXor for IO_BANK2_CTRL_SPEC {
        const OFFSET: usize = 0x1000;
    }
    pub type W = crate::W<IO_BANK2_CTRL_SPEC>;
    pub type AtomicSetW = crate::AtomicSetW<IO_BANK2_CTRL_SPEC>;
    pub type AtomicClearW = crate::AtomicClearW<IO_BANK2_CTRL_SPEC>;
    pub type AtomicXorW = crate::AtomicXorW<IO_BANK2_CTRL_SPEC>;
    /// Start/stop tick generation
    /// Field write behavior: Plain.
    pub mod enable {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 0>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 0>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 0>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 0>;
    }
    /// Tick generator is running
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod running {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn enable(&self) -> enable::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn running(&self) -> running::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
    }
    impl W {
        #[inline(always)]
        pub fn enable(&mut self) -> enable::W<'_> {
            enable::W::new(self)
        }
    }
    impl AtomicSetW {
        /// Start/stop tick generation
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn enable(&mut self) -> enable::ASW<'_> {
            enable::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// Start/stop tick generation
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn enable(&mut self) -> enable::ACW<'_> {
            enable::ACW::new(self)
        }
    }
    impl AtomicXorW {
        /// Start/stop tick generation
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn enable(&mut self) -> enable::AXW<'_> {
            enable::AXW::new(self)
        }
    }
}

/// Total number of clk_tick cycles before next tick
/// Boot ROM initialization value 0x32 was Confirmed in scope "Boot ROM TICKS group initialization"; this is not the hardware reset value.
/// Hardware access: read Plain; write Plain.
pub type IO_BANK2_CYCLES = Reg<io_bank2_cycles::IO_BANK2_CYCLES_SPEC>;
/// Total number of clk_tick cycles before next tick
/// Boot ROM initialization value 0x32 was Confirmed in scope "Boot ROM TICKS group initialization"; this is not the hardware reset value.
/// Hardware access: read Plain; write Plain.
pub mod io_bank2_cycles {
    pub struct IO_BANK2_CYCLES_SPEC;
    impl crate::RegisterSpec for IO_BANK2_CYCLES_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for IO_BANK2_CYCLES_SPEC {}
    pub type R = crate::R<IO_BANK2_CYCLES_SPEC>;
    impl crate::Modifiable for IO_BANK2_CYCLES_SPEC {}
    impl crate::UnsafeWritable for IO_BANK2_CYCLES_SPEC {}
    impl crate::ResetValue for IO_BANK2_CYCLES_SPEC {
        const RESET_VALUE: u32 = 0x0;
    }
    impl crate::TypedAtomicSet for IO_BANK2_CYCLES_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicSet for IO_BANK2_CYCLES_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::TypedAtomicClear for IO_BANK2_CYCLES_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicClear for IO_BANK2_CYCLES_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::TypedAtomicXor for IO_BANK2_CYCLES_SPEC {
        const OFFSET: usize = 0x1000;
    }
    impl crate::UnsafeAtomicXor for IO_BANK2_CYCLES_SPEC {
        const OFFSET: usize = 0x1000;
    }
    pub type W = crate::W<IO_BANK2_CYCLES_SPEC>;
    pub type AtomicSetW = crate::AtomicSetW<IO_BANK2_CYCLES_SPEC>;
    pub type AtomicClearW = crate::AtomicClearW<IO_BANK2_CYCLES_SPEC>;
    pub type AtomicXorW = crate::AtomicXorW<IO_BANK2_CYCLES_SPEC>;
    /// Total clk_tick cycles before next tick
    /// Field write behavior: Plain.
    pub mod cycles {
        pub type R = crate::FieldReader<u32>;
        pub type W<'a> = crate::FieldWriter<'a, super::W, 0, 28>;
        pub type ASW<'a> = crate::AtomicSetFieldWriter<'a, super::AtomicSetW, 0, 28>;
        pub type ACW<'a> = crate::AtomicClearFieldWriter<'a, super::AtomicClearW, 0, 28>;
        pub type AXW<'a> = crate::AtomicXorFieldWriter<'a, super::AtomicXorW, 0, 28>;
    }
    impl R {
        #[inline(always)]
        pub fn cycles(&self) -> cycles::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xfffffff)
        }
    }
    impl W {
        #[inline(always)]
        pub fn cycles(&mut self) -> cycles::W<'_> {
            cycles::W::new(self)
        }
    }
    impl AtomicSetW {
        /// Total clk_tick cycles before next tick
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn cycles(&mut self) -> cycles::ASW<'_> {
            cycles::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// Total clk_tick cycles before next tick
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn cycles(&mut self) -> cycles::ACW<'_> {
            cycles::ACW::new(self)
        }
    }
    impl AtomicXorW {
        /// Total clk_tick cycles before next tick
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn cycles(&mut self) -> cycles::AXW<'_> {
            cycles::AXW::new(self)
        }
    }
}

/// Countdown timer before next tick
/// Hardware access: read Plain; write None.
pub type IO_BANK2_COUNT = Reg<io_bank2_count::IO_BANK2_COUNT_SPEC>;
/// Countdown timer before next tick
/// Hardware access: read Plain; write None.
pub mod io_bank2_count {
    pub struct IO_BANK2_COUNT_SPEC;
    impl crate::RegisterSpec for IO_BANK2_COUNT_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for IO_BANK2_COUNT_SPEC {}
    pub type R = crate::R<IO_BANK2_COUNT_SPEC>;
    /// Remaining clk_tick cycles before next tick
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod count {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn count(&self) -> count::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xfffffff)
        }
    }
}

#[cfg(test)]
mod layout_tests {
    use super::RegisterBlock;
    #[test]
    fn timer_ctrl_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, timer_ctrl), 0x0);
    }
    #[test]
    fn timer_cycles_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, timer_cycles), 0x4);
    }
    #[test]
    fn timer_count_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, timer_count), 0x8);
    }
    #[test]
    fn watchdog_ctrl_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, watchdog_ctrl), 0xc);
    }
    #[test]
    fn watchdog_cycles_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, watchdog_cycles), 0x10);
    }
    #[test]
    fn watchdog_count_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, watchdog_count), 0x14);
    }
    #[test]
    fn proc_ctrl_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, proc_ctrl), 0x18);
    }
    #[test]
    fn proc_cycles_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, proc_cycles), 0x1c);
    }
    #[test]
    fn proc_count_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, proc_count), 0x20);
    }
    #[test]
    fn dma0_ctrl_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, dma0_ctrl), 0x24);
    }
    #[test]
    fn dma0_cycles_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, dma0_cycles), 0x28);
    }
    #[test]
    fn dma0_count_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, dma0_count), 0x2c);
    }
    #[test]
    fn dma1_ctrl_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, dma1_ctrl), 0x30);
    }
    #[test]
    fn dma1_cycles_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, dma1_cycles), 0x34);
    }
    #[test]
    fn dma1_count_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, dma1_count), 0x38);
    }
    #[test]
    fn io_bank0_ctrl_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, io_bank0_ctrl), 0x3c);
    }
    #[test]
    fn io_bank0_cycles_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, io_bank0_cycles), 0x40);
    }
    #[test]
    fn io_bank0_count_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, io_bank0_count), 0x44);
    }
    #[test]
    fn io_bank1_ctrl_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, io_bank1_ctrl), 0x48);
    }
    #[test]
    fn io_bank1_cycles_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, io_bank1_cycles), 0x4c);
    }
    #[test]
    fn io_bank1_count_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, io_bank1_count), 0x50);
    }
    #[test]
    fn io_bank2_ctrl_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, io_bank2_ctrl), 0x54);
    }
    #[test]
    fn io_bank2_cycles_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, io_bank2_cycles), 0x58);
    }
    #[test]
    fn io_bank2_count_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, io_bank2_count), 0x5c);
    }
    #[test]
    fn register_block_size() {
        assert_eq!(core::mem::size_of::<RegisterBlock>(), 0x60);
    }
}
