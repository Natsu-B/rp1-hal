// Generated from rp1-regdb/data/peripherals/pwm0.toml.
// Do not edit manually.
// Regenerate with: cargo run -p rp1-regdb -- generate

use crate::Reg;

/// PWM block with four waveform channels. Waveforms run on `clk_pwm`; the register bus runs on `clk_sys`. Bus=APB. Atomic aliases: XOR at +0x1000, SET at +0x2000, CLR at +0x3000.
/// Atomic view: Rp1Native alias windows; ownership remains with this logical peripheral.
#[repr(C)]
pub struct RegisterBlock {
    global_ctrl: GLOBAL_CTRL,
    fifo_ctrl: FIFO_CTRL,
    common_range: COMMON_RANGE,
    common_duty: COMMON_DUTY,
    duty_fifo: DUTY_FIFO,
    chan0_ctrl: CHAN0_CTRL,
    chan0_range: CHAN0_RANGE,
    chan0_phase: CHAN0_PHASE,
    chan0_duty: CHAN0_DUTY,
    chan1_ctrl: CHAN1_CTRL,
    chan1_range: CHAN1_RANGE,
    chan1_phase: CHAN1_PHASE,
    chan1_duty: CHAN1_DUTY,
    chan2_ctrl: CHAN2_CTRL,
    chan2_range: CHAN2_RANGE,
    chan2_phase: CHAN2_PHASE,
    chan2_duty: CHAN2_DUTY,
    chan3_ctrl: CHAN3_CTRL,
    chan3_range: CHAN3_RANGE,
    chan3_phase: CHAN3_PHASE,
    chan3_duty: CHAN3_DUTY,
    intr: INTR,
    inte: INTE,
    intf: INTF,
    ints: INTS,
}

impl RegisterBlock {
    /// Staged enables for the four channels, plus the trigger for a synchronized update of the waveform engine. Use the SET alias to trigger `set_update` without rewriting the staged enables.
    /// Hardware access: read Plain; write Plain.
    #[inline(always)]
    pub const fn global_ctrl(&self) -> &GLOBAL_CTRL {
        &self.global_ctrl
    }
    /// FIFO level, flush and DMA request control. The registers are in the `clk_sys` domain; FIFO consumption and flush completion happen in `clk_pwm`.
    /// Hardware access: read Plain; write Plain.
    #[inline(always)]
    pub const fn fifo_ctrl(&self) -> &FIFO_CTRL {
        &self.fifo_ctrl
    }
    /// Shared counter limit for channels with `bind=1`. Applied by `global_ctrl.set_update`.
    /// Hardware access: read Plain; write Plain.
    #[inline(always)]
    pub const fn common_range(&self) -> &COMMON_RANGE {
        &self.common_range
    }
    /// Shared duty value for bound channels that take duty from a register.
    /// Hardware access: read Plain; write Plain.
    #[inline(always)]
    pub const fn common_duty(&self) -> &COMMON_DUTY {
        &self.common_duty
    }
    /// Waveform setup for channel 0. Changes take effect after a `global_ctrl.set_update` write.
    /// Hardware access: read Plain; write Plain.
    #[inline(always)]
    pub const fn chan0_ctrl(&self) -> &CHAN0_CTRL {
        &self.chan0_ctrl
    }
    /// Channel 0's own counter limit, used when `bind=0`. A write takes effect at the next rollover.
    /// Hardware access: read Plain; write Plain.
    #[inline(always)]
    pub const fn chan0_range(&self) -> &CHAN0_RANGE {
        &self.chan0_range
    }
    /// Starting counter offset for channel 0, applied by `global_ctrl.set_update`.
    /// Hardware access: read Plain; write Plain.
    #[inline(always)]
    pub const fn chan0_phase(&self) -> &CHAN0_PHASE {
        &self.chan0_phase
    }
    /// Channel 0's own duty value, used when `bind=0` and `usefifo=0`. A new value is latched at the next rollover.
    /// Hardware access: read Plain; write Plain.
    #[inline(always)]
    pub const fn chan0_duty(&self) -> &CHAN0_DUTY {
        &self.chan0_duty
    }
    /// Waveform setup for channel 1. Changes take effect after a `global_ctrl.set_update` write.
    /// Hardware access: read Plain; write Plain.
    #[inline(always)]
    pub const fn chan1_ctrl(&self) -> &CHAN1_CTRL {
        &self.chan1_ctrl
    }
    /// Channel 1's own counter limit, used when `bind=0`. A write takes effect at the next rollover.
    /// Hardware access: read Plain; write Plain.
    #[inline(always)]
    pub const fn chan1_range(&self) -> &CHAN1_RANGE {
        &self.chan1_range
    }
    /// Starting counter offset for channel 1, applied by `global_ctrl.set_update`.
    /// Hardware access: read Plain; write Plain.
    #[inline(always)]
    pub const fn chan1_phase(&self) -> &CHAN1_PHASE {
        &self.chan1_phase
    }
    /// Channel 1's own duty value, used when `bind=0` and `usefifo=0`. A new value is latched at the next rollover.
    /// Hardware access: read Plain; write Plain.
    #[inline(always)]
    pub const fn chan1_duty(&self) -> &CHAN1_DUTY {
        &self.chan1_duty
    }
    /// Waveform setup for channel 2. Changes take effect after a `global_ctrl.set_update` write.
    /// Hardware access: read Plain; write Plain.
    #[inline(always)]
    pub const fn chan2_ctrl(&self) -> &CHAN2_CTRL {
        &self.chan2_ctrl
    }
    /// Channel 2's own counter limit, used when `bind=0`. A write takes effect at the next rollover.
    /// Hardware access: read Plain; write Plain.
    #[inline(always)]
    pub const fn chan2_range(&self) -> &CHAN2_RANGE {
        &self.chan2_range
    }
    /// Starting counter offset for channel 2, applied by `global_ctrl.set_update`.
    /// Hardware access: read Plain; write Plain.
    #[inline(always)]
    pub const fn chan2_phase(&self) -> &CHAN2_PHASE {
        &self.chan2_phase
    }
    /// Channel 2's own duty value, used when `bind=0` and `usefifo=0`. A new value is latched at the next rollover.
    /// Hardware access: read Plain; write Plain.
    #[inline(always)]
    pub const fn chan2_duty(&self) -> &CHAN2_DUTY {
        &self.chan2_duty
    }
    /// Waveform setup for channel 3. Changes take effect after a `global_ctrl.set_update` write.
    /// Hardware access: read Plain; write Plain.
    #[inline(always)]
    pub const fn chan3_ctrl(&self) -> &CHAN3_CTRL {
        &self.chan3_ctrl
    }
    /// Channel 3's own counter limit, used when `bind=0`. A write takes effect at the next rollover.
    /// Hardware access: read Plain; write Plain.
    #[inline(always)]
    pub const fn chan3_range(&self) -> &CHAN3_RANGE {
        &self.chan3_range
    }
    /// Starting counter offset for channel 3, applied by `global_ctrl.set_update`.
    /// Hardware access: read Plain; write Plain.
    #[inline(always)]
    pub const fn chan3_phase(&self) -> &CHAN3_PHASE {
        &self.chan3_phase
    }
    /// Channel 3's own duty value, used when `bind=0` and `usefifo=0`. A new value is latched at the next rollover.
    /// Hardware access: read Plain; write Plain.
    #[inline(always)]
    pub const fn chan3_duty(&self) -> &CHAN3_DUTY {
        &self.chan3_duty
    }
    /// Raw interrupt status, before the enable mask.
    /// Hardware access: read Plain; write WriteOneToClear.
    /// Write-one-to-clear register; general write and modify APIs are not generated.
    #[inline(always)]
    pub const fn intr(&self) -> &INTR {
        &self.intr
    }
    /// Interrupt enable mask, one bit per source: 0 masks the source, 1 passes it to the IRQ output.
    /// Hardware access: read Plain; write Plain.
    #[inline(always)]
    pub const fn inte(&self) -> &INTE {
        &self.inte
    }
    /// Interrupt force bits, one per source: 1 asserts the interrupt from software, 0 removes the force.
    /// Hardware access: read Plain; write Plain.
    #[inline(always)]
    pub const fn intf(&self) -> &INTF {
        &self.intf
    }
    /// Masked interrupt status: (INTR & INTE) | INTF.
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn ints(&self) -> &INTS {
        &self.ints
    }
}

/// Staged enables for the four channels, plus the trigger for a synchronized update of the waveform engine. Use the SET alias to trigger `set_update` without rewriting the staged enables.
/// Hardware access: read Plain; write Plain.
pub type GLOBAL_CTRL = Reg<global_ctrl::GLOBAL_CTRL_SPEC>;
/// Staged enables for the four channels, plus the trigger for a synchronized update of the waveform engine. Use the SET alias to trigger `set_update` without rewriting the staged enables.
/// Hardware access: read Plain; write Plain.
pub mod global_ctrl {
    pub struct GLOBAL_CTRL_SPEC;
    impl crate::RegisterSpec for GLOBAL_CTRL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for GLOBAL_CTRL_SPEC {}
    pub type R = crate::R<GLOBAL_CTRL_SPEC>;
    impl crate::UnsafeWritable for GLOBAL_CTRL_SPEC {}
    impl crate::ResetValue for GLOBAL_CTRL_SPEC {
        const RESET_VALUE: u32 = 0x0;
    }
    impl crate::TypedAtomicSet for GLOBAL_CTRL_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicSet for GLOBAL_CTRL_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::TypedAtomicClear for GLOBAL_CTRL_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicClear for GLOBAL_CTRL_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::TypedAtomicXor for GLOBAL_CTRL_SPEC {
        const OFFSET: usize = 0x1000;
    }
    impl crate::UnsafeAtomicXor for GLOBAL_CTRL_SPEC {
        const OFFSET: usize = 0x1000;
    }
    pub type AtomicSetW = crate::AtomicSetW<GLOBAL_CTRL_SPEC>;
    pub type AtomicClearW = crate::AtomicClearW<GLOBAL_CTRL_SPEC>;
    pub type AtomicXorW = crate::AtomicXorW<GLOBAL_CTRL_SPEC>;
    /// Channel 0 enable: 0 off, 1 on in its configured mode. Takes effect via `set_update`.
    /// Field write behavior: Plain.
    pub mod chan0_en {
        pub type R = crate::BitReader;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 0>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 0>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 0>;
    }
    /// Channel 1 enable: 0 off, 1 on in its configured mode. Takes effect via `set_update`.
    /// Field write behavior: Plain.
    pub mod chan1_en {
        pub type R = crate::BitReader;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 1>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 1>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 1>;
    }
    /// Channel 2 enable: 0 off, 1 on in its configured mode. Takes effect via `set_update`.
    /// Field write behavior: Plain.
    pub mod chan2_en {
        pub type R = crate::BitReader;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 2>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 2>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 2>;
    }
    /// Channel 3 enable: 0 off, 1 on in its configured mode. Takes effect via `set_update`.
    /// Field write behavior: Plain.
    pub mod chan3_en {
        pub type R = crate::BitReader;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 3>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 3>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 3>;
    }
    /// Write 1 to apply the staged enables, channel control and phase, and `common_range`. Hardware clears the bit. Local range and duty writes do not need it; they apply at the channel's rollover.
    /// Field write behavior: SelfClearing.
    pub mod set_update {
        pub type R = crate::BitReader;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 31>;
    }
    impl R {
        #[inline(always)]
        pub fn chan0_en(&self) -> chan0_en::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn chan1_en(&self) -> chan1_en::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
        #[inline(always)]
        pub fn chan2_en(&self) -> chan2_en::R {
            crate::BitReader::new(((self.bits() >> 2) & 1) != 0)
        }
        #[inline(always)]
        pub fn chan3_en(&self) -> chan3_en::R {
            crate::BitReader::new(((self.bits() >> 3) & 1) != 0)
        }
        #[inline(always)]
        pub fn set_update(&self) -> set_update::R {
            crate::BitReader::new(((self.bits() >> 31) & 1) != 0)
        }
    }
    impl AtomicSetW {
        /// Channel 0 enable: 0 off, 1 on in its configured mode. Takes effect via `set_update`.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn chan0_en(&mut self) -> chan0_en::ASW<'_> {
            chan0_en::ASW::new(self)
        }
        /// Channel 1 enable: 0 off, 1 on in its configured mode. Takes effect via `set_update`.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn chan1_en(&mut self) -> chan1_en::ASW<'_> {
            chan1_en::ASW::new(self)
        }
        /// Channel 2 enable: 0 off, 1 on in its configured mode. Takes effect via `set_update`.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn chan2_en(&mut self) -> chan2_en::ASW<'_> {
            chan2_en::ASW::new(self)
        }
        /// Channel 3 enable: 0 off, 1 on in its configured mode. Takes effect via `set_update`.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn chan3_en(&mut self) -> chan3_en::ASW<'_> {
            chan3_en::ASW::new(self)
        }
        /// Write 1 to apply the staged enables, channel control and phase, and `common_range`. Hardware clears the bit. Local range and duty writes do not need it; they apply at the channel's rollover.
        /// Field write behavior: SelfClearing.
        #[inline(always)]
        pub fn set_update(&mut self) -> set_update::ASW<'_> {
            set_update::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// Channel 0 enable: 0 off, 1 on in its configured mode. Takes effect via `set_update`.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn chan0_en(&mut self) -> chan0_en::ACW<'_> {
            chan0_en::ACW::new(self)
        }
        /// Channel 1 enable: 0 off, 1 on in its configured mode. Takes effect via `set_update`.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn chan1_en(&mut self) -> chan1_en::ACW<'_> {
            chan1_en::ACW::new(self)
        }
        /// Channel 2 enable: 0 off, 1 on in its configured mode. Takes effect via `set_update`.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn chan2_en(&mut self) -> chan2_en::ACW<'_> {
            chan2_en::ACW::new(self)
        }
        /// Channel 3 enable: 0 off, 1 on in its configured mode. Takes effect via `set_update`.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn chan3_en(&mut self) -> chan3_en::ACW<'_> {
            chan3_en::ACW::new(self)
        }
    }
    impl AtomicXorW {
        /// Channel 0 enable: 0 off, 1 on in its configured mode. Takes effect via `set_update`.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn chan0_en(&mut self) -> chan0_en::AXW<'_> {
            chan0_en::AXW::new(self)
        }
        /// Channel 1 enable: 0 off, 1 on in its configured mode. Takes effect via `set_update`.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn chan1_en(&mut self) -> chan1_en::AXW<'_> {
            chan1_en::AXW::new(self)
        }
        /// Channel 2 enable: 0 off, 1 on in its configured mode. Takes effect via `set_update`.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn chan2_en(&mut self) -> chan2_en::AXW<'_> {
            chan2_en::AXW::new(self)
        }
        /// Channel 3 enable: 0 off, 1 on in its configured mode. Takes effect via `set_update`.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn chan3_en(&mut self) -> chan3_en::AXW<'_> {
            chan3_en::AXW::new(self)
        }
    }
}

/// FIFO level, flush and DMA request control. The registers are in the `clk_sys` domain; FIFO consumption and flush completion happen in `clk_pwm`.
/// Hardware access: read Plain; write Plain.
pub type FIFO_CTRL = Reg<fifo_ctrl::FIFO_CTRL_SPEC>;
/// FIFO level, flush and DMA request control. The registers are in the `clk_sys` domain; FIFO consumption and flush completion happen in `clk_pwm`.
/// Hardware access: read Plain; write Plain.
pub mod fifo_ctrl {
    pub struct FIFO_CTRL_SPEC;
    impl crate::RegisterSpec for FIFO_CTRL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for FIFO_CTRL_SPEC {}
    pub type R = crate::R<FIFO_CTRL_SPEC>;
    impl crate::Modifiable for FIFO_CTRL_SPEC {}
    impl crate::UnsafeWritable for FIFO_CTRL_SPEC {}
    impl crate::TypedAtomicSet for FIFO_CTRL_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicSet for FIFO_CTRL_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::TypedAtomicClear for FIFO_CTRL_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicClear for FIFO_CTRL_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::TypedAtomicXor for FIFO_CTRL_SPEC {
        const OFFSET: usize = 0x1000;
    }
    impl crate::UnsafeAtomicXor for FIFO_CTRL_SPEC {
        const OFFSET: usize = 0x1000;
    }
    pub type W = crate::W<FIFO_CTRL_SPEC>;
    pub type AtomicSetW = crate::AtomicSetW<FIFO_CTRL_SPEC>;
    pub type AtomicClearW = crate::AtomicClearW<FIFO_CTRL_SPEC>;
    pub type AtomicXorW = crate::AtomicXorW<FIFO_CTRL_SPEC>;
    /// Number of words in the FIFO. How it counts when several channels are packed into one entry is unknown.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod level {
        pub type R = crate::FieldReader<u32>;
    }
    /// Write 1 to request that queued duty samples be discarded. Wait for `flush_done`, since completion crosses clock domains. HW (PWM0, clock running, channels idle): a written 1 stayed set and `flush_done` stayed 0, so software writes 0 afterwards. Behavior while channels consume the FIFO is unverified.
    /// Field write behavior: Plain.
    pub mod flush {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 5>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 5>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 5>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 5>;
    }
    /// 1 when the PWM clock domain has finished the flush.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod flush_done {
        pub type R = crate::BitReader;
    }
    /// A DMA request is raised while `level` is at or below this value. Same units as `level`.
    /// Field write behavior: Plain.
    pub mod threshold {
        pub type R = crate::FieldReader<u32>;
        pub type W<'a> = crate::FieldWriter<'a, super::W, 11, 5>;
        pub type ASW<'a> = crate::AtomicSetFieldWriter<'a, super::AtomicSetW, 11, 5>;
        pub type ACW<'a> = crate::AtomicClearFieldWriter<'a, super::AtomicClearW, 11, 5>;
        pub type AXW<'a> = crate::AtomicXorFieldWriter<'a, super::AtomicXorW, 11, 5>;
    }
    /// Minimum gap between DMA requests in bus cycles, giving in-flight writes time to reach the FIFO.
    /// Field write behavior: Plain.
    pub mod dwell_time {
        pub type R = crate::FieldReader<u32>;
        pub type W<'a> = crate::FieldWriter<'a, super::W, 16, 5>;
        pub type ASW<'a> = crate::AtomicSetFieldWriter<'a, super::AtomicSetW, 16, 5>;
        pub type ACW<'a> = crate::AtomicClearFieldWriter<'a, super::AtomicClearW, 16, 5>;
        pub type AXW<'a> = crate::AtomicXorFieldWriter<'a, super::AtomicXorW, 16, 5>;
    }
    /// 1 enables the DMA request output. 0 masks it, but the `dreq_active` interrupt source still works.
    /// Field write behavior: Plain.
    pub mod dreq_en {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 31>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 31>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 31>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 31>;
    }
    impl R {
        #[inline(always)]
        pub fn level(&self) -> level::R {
            crate::FieldReader::new((self.bits() >> 0) & 0x1f)
        }
        #[inline(always)]
        pub fn flush(&self) -> flush::R {
            crate::BitReader::new(((self.bits() >> 5) & 1) != 0)
        }
        #[inline(always)]
        pub fn flush_done(&self) -> flush_done::R {
            crate::BitReader::new(((self.bits() >> 6) & 1) != 0)
        }
        #[inline(always)]
        pub fn threshold(&self) -> threshold::R {
            crate::FieldReader::new((self.bits() >> 11) & 0x1f)
        }
        #[inline(always)]
        pub fn dwell_time(&self) -> dwell_time::R {
            crate::FieldReader::new((self.bits() >> 16) & 0x1f)
        }
        #[inline(always)]
        pub fn dreq_en(&self) -> dreq_en::R {
            crate::BitReader::new(((self.bits() >> 31) & 1) != 0)
        }
    }
    impl W {
        #[inline(always)]
        pub fn flush(&mut self) -> flush::W<'_> {
            flush::W::new(self)
        }
        #[inline(always)]
        pub fn threshold(&mut self) -> threshold::W<'_> {
            threshold::W::new(self)
        }
        #[inline(always)]
        pub fn dwell_time(&mut self) -> dwell_time::W<'_> {
            dwell_time::W::new(self)
        }
        #[inline(always)]
        pub fn dreq_en(&mut self) -> dreq_en::W<'_> {
            dreq_en::W::new(self)
        }
    }
    impl AtomicSetW {
        /// Write 1 to request that queued duty samples be discarded. Wait for `flush_done`, since completion crosses clock domains. HW (PWM0, clock running, channels idle): a written 1 stayed set and `flush_done` stayed 0, so software writes 0 afterwards. Behavior while channels consume the FIFO is unverified.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn flush(&mut self) -> flush::ASW<'_> {
            flush::ASW::new(self)
        }
        /// A DMA request is raised while `level` is at or below this value. Same units as `level`.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn threshold(&mut self) -> threshold::ASW<'_> {
            threshold::ASW::new(self)
        }
        /// Minimum gap between DMA requests in bus cycles, giving in-flight writes time to reach the FIFO.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn dwell_time(&mut self) -> dwell_time::ASW<'_> {
            dwell_time::ASW::new(self)
        }
        /// 1 enables the DMA request output. 0 masks it, but the `dreq_active` interrupt source still works.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn dreq_en(&mut self) -> dreq_en::ASW<'_> {
            dreq_en::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// Write 1 to request that queued duty samples be discarded. Wait for `flush_done`, since completion crosses clock domains. HW (PWM0, clock running, channels idle): a written 1 stayed set and `flush_done` stayed 0, so software writes 0 afterwards. Behavior while channels consume the FIFO is unverified.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn flush(&mut self) -> flush::ACW<'_> {
            flush::ACW::new(self)
        }
        /// A DMA request is raised while `level` is at or below this value. Same units as `level`.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn threshold(&mut self) -> threshold::ACW<'_> {
            threshold::ACW::new(self)
        }
        /// Minimum gap between DMA requests in bus cycles, giving in-flight writes time to reach the FIFO.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn dwell_time(&mut self) -> dwell_time::ACW<'_> {
            dwell_time::ACW::new(self)
        }
        /// 1 enables the DMA request output. 0 masks it, but the `dreq_active` interrupt source still works.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn dreq_en(&mut self) -> dreq_en::ACW<'_> {
            dreq_en::ACW::new(self)
        }
    }
    impl AtomicXorW {
        /// Write 1 to request that queued duty samples be discarded. Wait for `flush_done`, since completion crosses clock domains. HW (PWM0, clock running, channels idle): a written 1 stayed set and `flush_done` stayed 0, so software writes 0 afterwards. Behavior while channels consume the FIFO is unverified.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn flush(&mut self) -> flush::AXW<'_> {
            flush::AXW::new(self)
        }
        /// A DMA request is raised while `level` is at or below this value. Same units as `level`.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn threshold(&mut self) -> threshold::AXW<'_> {
            threshold::AXW::new(self)
        }
        /// Minimum gap between DMA requests in bus cycles, giving in-flight writes time to reach the FIFO.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn dwell_time(&mut self) -> dwell_time::AXW<'_> {
            dwell_time::AXW::new(self)
        }
        /// 1 enables the DMA request output. 0 masks it, but the `dreq_active` interrupt source still works.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn dreq_en(&mut self) -> dreq_en::AXW<'_> {
            dreq_en::AXW::new(self)
        }
    }
}

/// Shared counter limit for channels with `bind=1`. Applied by `global_ctrl.set_update`.
/// Hardware access: read Plain; write Plain.
pub type COMMON_RANGE = Reg<common_range::COMMON_RANGE_SPEC>;
/// Shared counter limit for channels with `bind=1`. Applied by `global_ctrl.set_update`.
/// Hardware access: read Plain; write Plain.
pub mod common_range {
    pub struct COMMON_RANGE_SPEC;
    impl crate::RegisterSpec for COMMON_RANGE_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for COMMON_RANGE_SPEC {}
    pub type R = crate::R<COMMON_RANGE_SPEC>;
    impl crate::Modifiable for COMMON_RANGE_SPEC {}
    impl crate::UnsafeWritable for COMMON_RANGE_SPEC {}
    impl crate::ResetValue for COMMON_RANGE_SPEC {
        const RESET_VALUE: u32 = 0x0;
    }
    impl crate::TypedAtomicSet for COMMON_RANGE_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicSet for COMMON_RANGE_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::TypedAtomicClear for COMMON_RANGE_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicClear for COMMON_RANGE_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::TypedAtomicXor for COMMON_RANGE_SPEC {
        const OFFSET: usize = 0x1000;
    }
    impl crate::UnsafeAtomicXor for COMMON_RANGE_SPEC {
        const OFFSET: usize = 0x1000;
    }
    pub type W = crate::W<COMMON_RANGE_SPEC>;
    pub type AtomicSetW = crate::AtomicSetW<COMMON_RANGE_SPEC>;
    pub type AtomicClearW = crate::AtomicClearW<COMMON_RANGE_SPEC>;
    pub type AtomicXorW = crate::AtomicXorW<COMMON_RANGE_SPEC>;
    /// Shared counter limit in PWM clock counts; replaces the local range for bound channels.
    /// Field write behavior: Plain.
    pub mod range {
        pub type R = crate::FieldReader<u32>;
        pub type W<'a> = crate::FieldWriter<'a, super::W, 0, 32>;
        pub type ASW<'a> = crate::AtomicSetFieldWriter<'a, super::AtomicSetW, 0, 32>;
        pub type ACW<'a> = crate::AtomicClearFieldWriter<'a, super::AtomicClearW, 0, 32>;
        pub type AXW<'a> = crate::AtomicXorFieldWriter<'a, super::AtomicXorW, 0, 32>;
    }
    impl R {
        #[inline(always)]
        pub fn range(&self) -> range::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffffffff)
        }
    }
    impl W {
        #[inline(always)]
        pub fn range(&mut self) -> range::W<'_> {
            range::W::new(self)
        }
    }
    impl AtomicSetW {
        /// Shared counter limit in PWM clock counts; replaces the local range for bound channels.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn range(&mut self) -> range::ASW<'_> {
            range::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// Shared counter limit in PWM clock counts; replaces the local range for bound channels.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn range(&mut self) -> range::ACW<'_> {
            range::ACW::new(self)
        }
    }
    impl AtomicXorW {
        /// Shared counter limit in PWM clock counts; replaces the local range for bound channels.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn range(&mut self) -> range::AXW<'_> {
            range::AXW::new(self)
        }
    }
}

/// Shared duty value for bound channels that take duty from a register.
/// Hardware access: read Plain; write Plain.
pub type COMMON_DUTY = Reg<common_duty::COMMON_DUTY_SPEC>;
/// Shared duty value for bound channels that take duty from a register.
/// Hardware access: read Plain; write Plain.
pub mod common_duty {
    pub struct COMMON_DUTY_SPEC;
    impl crate::RegisterSpec for COMMON_DUTY_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for COMMON_DUTY_SPEC {}
    pub type R = crate::R<COMMON_DUTY_SPEC>;
    impl crate::Modifiable for COMMON_DUTY_SPEC {}
    impl crate::UnsafeWritable for COMMON_DUTY_SPEC {}
    impl crate::ResetValue for COMMON_DUTY_SPEC {
        const RESET_VALUE: u32 = 0x0;
    }
    impl crate::TypedAtomicSet for COMMON_DUTY_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicSet for COMMON_DUTY_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::TypedAtomicClear for COMMON_DUTY_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicClear for COMMON_DUTY_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::TypedAtomicXor for COMMON_DUTY_SPEC {
        const OFFSET: usize = 0x1000;
    }
    impl crate::UnsafeAtomicXor for COMMON_DUTY_SPEC {
        const OFFSET: usize = 0x1000;
    }
    pub type W = crate::W<COMMON_DUTY_SPEC>;
    pub type AtomicSetW = crate::AtomicSetW<COMMON_DUTY_SPEC>;
    pub type AtomicClearW = crate::AtomicClearW<COMMON_DUTY_SPEC>;
    pub type AtomicXorW = crate::AtomicXorW<COMMON_DUTY_SPEC>;
    /// Shared value whose meaning follows `mode`: compare value, pulse position, density numerator or serial word. Used when `bind=1` and `usefifo=0`.
    /// Field write behavior: Plain.
    pub mod duty {
        pub type R = crate::FieldReader<u32>;
        pub type W<'a> = crate::FieldWriter<'a, super::W, 0, 32>;
        pub type ASW<'a> = crate::AtomicSetFieldWriter<'a, super::AtomicSetW, 0, 32>;
        pub type ACW<'a> = crate::AtomicClearFieldWriter<'a, super::AtomicClearW, 0, 32>;
        pub type AXW<'a> = crate::AtomicXorFieldWriter<'a, super::AtomicXorW, 0, 32>;
    }
    impl R {
        #[inline(always)]
        pub fn duty(&self) -> duty::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffffffff)
        }
    }
    impl W {
        #[inline(always)]
        pub fn duty(&mut self) -> duty::W<'_> {
            duty::W::new(self)
        }
    }
    impl AtomicSetW {
        /// Shared value whose meaning follows `mode`: compare value, pulse position, density numerator or serial word. Used when `bind=1` and `usefifo=0`.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn duty(&mut self) -> duty::ASW<'_> {
            duty::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// Shared value whose meaning follows `mode`: compare value, pulse position, density numerator or serial word. Used when `bind=1` and `usefifo=0`.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn duty(&mut self) -> duty::ACW<'_> {
            duty::ACW::new(self)
        }
    }
    impl AtomicXorW {
        /// Shared value whose meaning follows `mode`: compare value, pulse position, density numerator or serial word. Used when `bind=1` and `usefifo=0`.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn duty(&mut self) -> duty::AXW<'_> {
            duty::AXW::new(self)
        }
    }
}

/// Write-only FIFO input. Each 32-bit write fills one of up to four lanes of a 128-bit entry, rotating through the channels that use the FIFO. One packed entry holds one word per participating channel.
/// Hardware access: read None; write Fifo.
/// FIFO/data port; only dedicated FIFO operations are generated.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type DUTY_FIFO = Reg<duty_fifo::DUTY_FIFO_SPEC>;
/// Write-only FIFO input. Each 32-bit write fills one of up to four lanes of a 128-bit entry, rotating through the channels that use the FIFO. One packed entry holds one word per participating channel.
/// Hardware access: read None; write Fifo.
/// FIFO/data port; only dedicated FIFO operations are generated.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod duty_fifo {
    pub struct DUTY_FIFO_SPEC;
    impl crate::RegisterSpec for DUTY_FIFO_SPEC {
        type Ux = u32;
    }
    impl crate::ResetValue for DUTY_FIFO_SPEC {
        const RESET_VALUE: u32 = 0x0;
    }
    /// One 32-bit duty sample. The channel it goes to follows the lane rotation.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod value {}
}

/// Waveform setup for channel 0. Changes take effect after a `global_ctrl.set_update` write.
/// Hardware access: read Plain; write Plain.
pub type CHAN0_CTRL = Reg<chan0_ctrl::CHAN0_CTRL_SPEC>;
/// Waveform setup for channel 0. Changes take effect after a `global_ctrl.set_update` write.
/// Hardware access: read Plain; write Plain.
pub mod chan0_ctrl {
    pub struct CHAN0_CTRL_SPEC;
    impl crate::RegisterSpec for CHAN0_CTRL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CHAN0_CTRL_SPEC {}
    pub type R = crate::R<CHAN0_CTRL_SPEC>;
    impl crate::Modifiable for CHAN0_CTRL_SPEC {}
    impl crate::UnsafeWritable for CHAN0_CTRL_SPEC {}
    impl crate::TypedAtomicSet for CHAN0_CTRL_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicSet for CHAN0_CTRL_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::TypedAtomicClear for CHAN0_CTRL_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicClear for CHAN0_CTRL_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::TypedAtomicXor for CHAN0_CTRL_SPEC {
        const OFFSET: usize = 0x1000;
    }
    impl crate::UnsafeAtomicXor for CHAN0_CTRL_SPEC {
        const OFFSET: usize = 0x1000;
    }
    pub type W = crate::W<CHAN0_CTRL_SPEC>;
    pub type AtomicSetW = crate::AtomicSetW<CHAN0_CTRL_SPEC>;
    pub type AtomicClearW = crate::AtomicClearW<CHAN0_CTRL_SPEC>;
    pub type AtomicXorW = crate::AtomicXorW<CHAN0_CTRL_SPEC>;
    /// Waveform mode. 0: output 0 (before inversion). 1: trailing-edge PWM, counter counts up. 2: centre-aligned pulse, counter counts down then up. 3: pulse density, high for DUTY/RANGE of the steps; each step adds DUTY to an accumulator, and when the sum reaches RANGE a 1 is output and RANGE is subtracted. 4: serial output starting at bit 31. 5: one high slot when COUNT equals DUTY. 6: leading-edge PWM, counter counts down. 7: serial output starting at bit 0. Modes 1, 2 and 6 output high while COUNT < DUTY. Serial modes wrap the bit index modulo 32.
    /// Field write behavior: Plain.
    pub mod mode {
        pub type R = crate::FieldReader<u32>;
        pub type W<'a> = crate::FieldWriter<'a, super::W, 0, 3>;
        pub type ASW<'a> = crate::AtomicSetFieldWriter<'a, super::AtomicSetW, 0, 3>;
        pub type ACW<'a> = crate::AtomicClearFieldWriter<'a, super::AtomicClearW, 0, 3>;
        pub type AXW<'a> = crate::AtomicXorFieldWriter<'a, super::AtomicXorW, 0, 3>;
    }
    /// 1 inverts the output after mode processing; 0 leaves it unchanged.
    /// Field write behavior: Plain.
    pub mod invert {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 3>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 3>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 3>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 3>;
    }
    /// 0 uses this channel's own range and duty registers. 1 uses `common_range`, with duty from `common_duty` or the FIFO.
    /// Field write behavior: Plain.
    pub mod bind {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 4>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 4>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 4>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 4>;
    }
    /// 0 takes duty from a register; 1 takes duty from the FIFO. Needs `bind=1`; using the FIFO without bind gives unpredictable results.
    /// Field write behavior: Plain.
    pub mod usefifo {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 5>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 5>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 5>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 5>;
    }
    /// 1 enables second-order sigma-delta noise shaping: the duty value is treated as signed 16-bit and quantized to `sdm_bitwidth`+1 bits, advancing on the channel's duty strobe. 0 bypasses shaping.
    /// Field write behavior: Plain.
    pub mod sdm {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 6>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 6>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 6>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 6>;
    }
    /// 1 adds a one-LSB dither into the sigma-delta loop to break up idle tones; 0 turns it off. Only has an effect when `sdm=1`.
    /// Field write behavior: Plain.
    pub mod dither {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 7>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 7>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 7>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 7>;
    }
    /// 1 lets this channel's rollover contribute to the shared FIFO pop (an OR across bound channels). 0 keeps its rollover from popping.
    /// Field write behavior: Plain.
    pub mod fifo_pop_mask {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 8>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 8>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 8>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 8>;
    }
    /// Sigma-delta output precision minus one: values 0-15 give 1-16 bits.
    /// Field write behavior: Plain.
    pub mod sdm_bitwidth {
        pub type R = crate::FieldReader<u32>;
        pub type W<'a> = crate::FieldWriter<'a, super::W, 12, 4>;
        pub type ASW<'a> = crate::AtomicSetFieldWriter<'a, super::AtomicSetW, 12, 4>;
        pub type ACW<'a> = crate::AtomicClearFieldWriter<'a, super::AtomicClearW, 12, 4>;
        pub type AXW<'a> = crate::AtomicXorFieldWriter<'a, super::AtomicXorW, 12, 4>;
    }
    /// Unsigned offset added to the quantized PWM code, for example to move a signed source into an unsigned range.
    /// Field write behavior: Plain.
    pub mod sdm_bias {
        pub type R = crate::FieldReader<u32>;
        pub type W<'a> = crate::FieldWriter<'a, super::W, 16, 16>;
        pub type ASW<'a> = crate::AtomicSetFieldWriter<'a, super::AtomicSetW, 16, 16>;
        pub type ACW<'a> = crate::AtomicClearFieldWriter<'a, super::AtomicClearW, 16, 16>;
        pub type AXW<'a> = crate::AtomicXorFieldWriter<'a, super::AtomicXorW, 16, 16>;
    }
    impl R {
        #[inline(always)]
        pub fn mode(&self) -> mode::R {
            crate::FieldReader::new((self.bits() >> 0) & 0x7)
        }
        #[inline(always)]
        pub fn invert(&self) -> invert::R {
            crate::BitReader::new(((self.bits() >> 3) & 1) != 0)
        }
        #[inline(always)]
        pub fn bind(&self) -> bind::R {
            crate::BitReader::new(((self.bits() >> 4) & 1) != 0)
        }
        #[inline(always)]
        pub fn usefifo(&self) -> usefifo::R {
            crate::BitReader::new(((self.bits() >> 5) & 1) != 0)
        }
        #[inline(always)]
        pub fn sdm(&self) -> sdm::R {
            crate::BitReader::new(((self.bits() >> 6) & 1) != 0)
        }
        #[inline(always)]
        pub fn dither(&self) -> dither::R {
            crate::BitReader::new(((self.bits() >> 7) & 1) != 0)
        }
        #[inline(always)]
        pub fn fifo_pop_mask(&self) -> fifo_pop_mask::R {
            crate::BitReader::new(((self.bits() >> 8) & 1) != 0)
        }
        #[inline(always)]
        pub fn sdm_bitwidth(&self) -> sdm_bitwidth::R {
            crate::FieldReader::new((self.bits() >> 12) & 0xf)
        }
        #[inline(always)]
        pub fn sdm_bias(&self) -> sdm_bias::R {
            crate::FieldReader::new((self.bits() >> 16) & 0xffff)
        }
    }
    impl W {
        #[inline(always)]
        pub fn mode(&mut self) -> mode::W<'_> {
            mode::W::new(self)
        }
        #[inline(always)]
        pub fn invert(&mut self) -> invert::W<'_> {
            invert::W::new(self)
        }
        #[inline(always)]
        pub fn bind(&mut self) -> bind::W<'_> {
            bind::W::new(self)
        }
        #[inline(always)]
        pub fn usefifo(&mut self) -> usefifo::W<'_> {
            usefifo::W::new(self)
        }
        #[inline(always)]
        pub fn sdm(&mut self) -> sdm::W<'_> {
            sdm::W::new(self)
        }
        #[inline(always)]
        pub fn dither(&mut self) -> dither::W<'_> {
            dither::W::new(self)
        }
        #[inline(always)]
        pub fn fifo_pop_mask(&mut self) -> fifo_pop_mask::W<'_> {
            fifo_pop_mask::W::new(self)
        }
        #[inline(always)]
        pub fn sdm_bitwidth(&mut self) -> sdm_bitwidth::W<'_> {
            sdm_bitwidth::W::new(self)
        }
        #[inline(always)]
        pub fn sdm_bias(&mut self) -> sdm_bias::W<'_> {
            sdm_bias::W::new(self)
        }
    }
    impl AtomicSetW {
        /// Waveform mode. 0: output 0 (before inversion). 1: trailing-edge PWM, counter counts up. 2: centre-aligned pulse, counter counts down then up. 3: pulse density, high for DUTY/RANGE of the steps; each step adds DUTY to an accumulator, and when the sum reaches RANGE a 1 is output and RANGE is subtracted. 4: serial output starting at bit 31. 5: one high slot when COUNT equals DUTY. 6: leading-edge PWM, counter counts down. 7: serial output starting at bit 0. Modes 1, 2 and 6 output high while COUNT < DUTY. Serial modes wrap the bit index modulo 32.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn mode(&mut self) -> mode::ASW<'_> {
            mode::ASW::new(self)
        }
        /// 1 inverts the output after mode processing; 0 leaves it unchanged.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn invert(&mut self) -> invert::ASW<'_> {
            invert::ASW::new(self)
        }
        /// 0 uses this channel's own range and duty registers. 1 uses `common_range`, with duty from `common_duty` or the FIFO.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn bind(&mut self) -> bind::ASW<'_> {
            bind::ASW::new(self)
        }
        /// 0 takes duty from a register; 1 takes duty from the FIFO. Needs `bind=1`; using the FIFO without bind gives unpredictable results.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn usefifo(&mut self) -> usefifo::ASW<'_> {
            usefifo::ASW::new(self)
        }
        /// 1 enables second-order sigma-delta noise shaping: the duty value is treated as signed 16-bit and quantized to `sdm_bitwidth`+1 bits, advancing on the channel's duty strobe. 0 bypasses shaping.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn sdm(&mut self) -> sdm::ASW<'_> {
            sdm::ASW::new(self)
        }
        /// 1 adds a one-LSB dither into the sigma-delta loop to break up idle tones; 0 turns it off. Only has an effect when `sdm=1`.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn dither(&mut self) -> dither::ASW<'_> {
            dither::ASW::new(self)
        }
        /// 1 lets this channel's rollover contribute to the shared FIFO pop (an OR across bound channels). 0 keeps its rollover from popping.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn fifo_pop_mask(&mut self) -> fifo_pop_mask::ASW<'_> {
            fifo_pop_mask::ASW::new(self)
        }
        /// Sigma-delta output precision minus one: values 0-15 give 1-16 bits.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn sdm_bitwidth(&mut self) -> sdm_bitwidth::ASW<'_> {
            sdm_bitwidth::ASW::new(self)
        }
        /// Unsigned offset added to the quantized PWM code, for example to move a signed source into an unsigned range.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn sdm_bias(&mut self) -> sdm_bias::ASW<'_> {
            sdm_bias::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// Waveform mode. 0: output 0 (before inversion). 1: trailing-edge PWM, counter counts up. 2: centre-aligned pulse, counter counts down then up. 3: pulse density, high for DUTY/RANGE of the steps; each step adds DUTY to an accumulator, and when the sum reaches RANGE a 1 is output and RANGE is subtracted. 4: serial output starting at bit 31. 5: one high slot when COUNT equals DUTY. 6: leading-edge PWM, counter counts down. 7: serial output starting at bit 0. Modes 1, 2 and 6 output high while COUNT < DUTY. Serial modes wrap the bit index modulo 32.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn mode(&mut self) -> mode::ACW<'_> {
            mode::ACW::new(self)
        }
        /// 1 inverts the output after mode processing; 0 leaves it unchanged.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn invert(&mut self) -> invert::ACW<'_> {
            invert::ACW::new(self)
        }
        /// 0 uses this channel's own range and duty registers. 1 uses `common_range`, with duty from `common_duty` or the FIFO.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn bind(&mut self) -> bind::ACW<'_> {
            bind::ACW::new(self)
        }
        /// 0 takes duty from a register; 1 takes duty from the FIFO. Needs `bind=1`; using the FIFO without bind gives unpredictable results.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn usefifo(&mut self) -> usefifo::ACW<'_> {
            usefifo::ACW::new(self)
        }
        /// 1 enables second-order sigma-delta noise shaping: the duty value is treated as signed 16-bit and quantized to `sdm_bitwidth`+1 bits, advancing on the channel's duty strobe. 0 bypasses shaping.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn sdm(&mut self) -> sdm::ACW<'_> {
            sdm::ACW::new(self)
        }
        /// 1 adds a one-LSB dither into the sigma-delta loop to break up idle tones; 0 turns it off. Only has an effect when `sdm=1`.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn dither(&mut self) -> dither::ACW<'_> {
            dither::ACW::new(self)
        }
        /// 1 lets this channel's rollover contribute to the shared FIFO pop (an OR across bound channels). 0 keeps its rollover from popping.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn fifo_pop_mask(&mut self) -> fifo_pop_mask::ACW<'_> {
            fifo_pop_mask::ACW::new(self)
        }
        /// Sigma-delta output precision minus one: values 0-15 give 1-16 bits.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn sdm_bitwidth(&mut self) -> sdm_bitwidth::ACW<'_> {
            sdm_bitwidth::ACW::new(self)
        }
        /// Unsigned offset added to the quantized PWM code, for example to move a signed source into an unsigned range.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn sdm_bias(&mut self) -> sdm_bias::ACW<'_> {
            sdm_bias::ACW::new(self)
        }
    }
    impl AtomicXorW {
        /// Waveform mode. 0: output 0 (before inversion). 1: trailing-edge PWM, counter counts up. 2: centre-aligned pulse, counter counts down then up. 3: pulse density, high for DUTY/RANGE of the steps; each step adds DUTY to an accumulator, and when the sum reaches RANGE a 1 is output and RANGE is subtracted. 4: serial output starting at bit 31. 5: one high slot when COUNT equals DUTY. 6: leading-edge PWM, counter counts down. 7: serial output starting at bit 0. Modes 1, 2 and 6 output high while COUNT < DUTY. Serial modes wrap the bit index modulo 32.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn mode(&mut self) -> mode::AXW<'_> {
            mode::AXW::new(self)
        }
        /// 1 inverts the output after mode processing; 0 leaves it unchanged.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn invert(&mut self) -> invert::AXW<'_> {
            invert::AXW::new(self)
        }
        /// 0 uses this channel's own range and duty registers. 1 uses `common_range`, with duty from `common_duty` or the FIFO.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn bind(&mut self) -> bind::AXW<'_> {
            bind::AXW::new(self)
        }
        /// 0 takes duty from a register; 1 takes duty from the FIFO. Needs `bind=1`; using the FIFO without bind gives unpredictable results.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn usefifo(&mut self) -> usefifo::AXW<'_> {
            usefifo::AXW::new(self)
        }
        /// 1 enables second-order sigma-delta noise shaping: the duty value is treated as signed 16-bit and quantized to `sdm_bitwidth`+1 bits, advancing on the channel's duty strobe. 0 bypasses shaping.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn sdm(&mut self) -> sdm::AXW<'_> {
            sdm::AXW::new(self)
        }
        /// 1 adds a one-LSB dither into the sigma-delta loop to break up idle tones; 0 turns it off. Only has an effect when `sdm=1`.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn dither(&mut self) -> dither::AXW<'_> {
            dither::AXW::new(self)
        }
        /// 1 lets this channel's rollover contribute to the shared FIFO pop (an OR across bound channels). 0 keeps its rollover from popping.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn fifo_pop_mask(&mut self) -> fifo_pop_mask::AXW<'_> {
            fifo_pop_mask::AXW::new(self)
        }
        /// Sigma-delta output precision minus one: values 0-15 give 1-16 bits.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn sdm_bitwidth(&mut self) -> sdm_bitwidth::AXW<'_> {
            sdm_bitwidth::AXW::new(self)
        }
        /// Unsigned offset added to the quantized PWM code, for example to move a signed source into an unsigned range.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn sdm_bias(&mut self) -> sdm_bias::AXW<'_> {
            sdm_bias::AXW::new(self)
        }
    }
}

/// Channel 0's own counter limit, used when `bind=0`. A write takes effect at the next rollover.
/// Hardware access: read Plain; write Plain.
pub type CHAN0_RANGE = Reg<chan0_range::CHAN0_RANGE_SPEC>;
/// Channel 0's own counter limit, used when `bind=0`. A write takes effect at the next rollover.
/// Hardware access: read Plain; write Plain.
pub mod chan0_range {
    pub struct CHAN0_RANGE_SPEC;
    impl crate::RegisterSpec for CHAN0_RANGE_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CHAN0_RANGE_SPEC {}
    pub type R = crate::R<CHAN0_RANGE_SPEC>;
    impl crate::Modifiable for CHAN0_RANGE_SPEC {}
    impl crate::UnsafeWritable for CHAN0_RANGE_SPEC {}
    impl crate::ResetValue for CHAN0_RANGE_SPEC {
        const RESET_VALUE: u32 = 0x0;
    }
    impl crate::TypedAtomicSet for CHAN0_RANGE_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicSet for CHAN0_RANGE_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::TypedAtomicClear for CHAN0_RANGE_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicClear for CHAN0_RANGE_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::TypedAtomicXor for CHAN0_RANGE_SPEC {
        const OFFSET: usize = 0x1000;
    }
    impl crate::UnsafeAtomicXor for CHAN0_RANGE_SPEC {
        const OFFSET: usize = 0x1000;
    }
    pub type W = crate::W<CHAN0_RANGE_SPEC>;
    pub type AtomicSetW = crate::AtomicSetW<CHAN0_RANGE_SPEC>;
    pub type AtomicClearW = crate::AtomicClearW<CHAN0_RANGE_SPEC>;
    pub type AtomicXorW = crate::AtomicXorW<CHAN0_RANGE_SPEC>;
    /// Counter limit in PWM clock counts.
    /// Field write behavior: Plain.
    pub mod range {
        pub type R = crate::FieldReader<u32>;
        pub type W<'a> = crate::FieldWriter<'a, super::W, 0, 32>;
        pub type ASW<'a> = crate::AtomicSetFieldWriter<'a, super::AtomicSetW, 0, 32>;
        pub type ACW<'a> = crate::AtomicClearFieldWriter<'a, super::AtomicClearW, 0, 32>;
        pub type AXW<'a> = crate::AtomicXorFieldWriter<'a, super::AtomicXorW, 0, 32>;
    }
    impl R {
        #[inline(always)]
        pub fn range(&self) -> range::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffffffff)
        }
    }
    impl W {
        #[inline(always)]
        pub fn range(&mut self) -> range::W<'_> {
            range::W::new(self)
        }
    }
    impl AtomicSetW {
        /// Counter limit in PWM clock counts.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn range(&mut self) -> range::ASW<'_> {
            range::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// Counter limit in PWM clock counts.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn range(&mut self) -> range::ACW<'_> {
            range::ACW::new(self)
        }
    }
    impl AtomicXorW {
        /// Counter limit in PWM clock counts.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn range(&mut self) -> range::AXW<'_> {
            range::AXW::new(self)
        }
    }
}

/// Starting counter offset for channel 0, applied by `global_ctrl.set_update`.
/// Hardware access: read Plain; write Plain.
pub type CHAN0_PHASE = Reg<chan0_phase::CHAN0_PHASE_SPEC>;
/// Starting counter offset for channel 0, applied by `global_ctrl.set_update`.
/// Hardware access: read Plain; write Plain.
pub mod chan0_phase {
    pub struct CHAN0_PHASE_SPEC;
    impl crate::RegisterSpec for CHAN0_PHASE_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CHAN0_PHASE_SPEC {}
    pub type R = crate::R<CHAN0_PHASE_SPEC>;
    impl crate::Modifiable for CHAN0_PHASE_SPEC {}
    impl crate::UnsafeWritable for CHAN0_PHASE_SPEC {}
    impl crate::ResetValue for CHAN0_PHASE_SPEC {
        const RESET_VALUE: u32 = 0x0;
    }
    impl crate::TypedAtomicSet for CHAN0_PHASE_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicSet for CHAN0_PHASE_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::TypedAtomicClear for CHAN0_PHASE_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicClear for CHAN0_PHASE_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::TypedAtomicXor for CHAN0_PHASE_SPEC {
        const OFFSET: usize = 0x1000;
    }
    impl crate::UnsafeAtomicXor for CHAN0_PHASE_SPEC {
        const OFFSET: usize = 0x1000;
    }
    pub type W = crate::W<CHAN0_PHASE_SPEC>;
    pub type AtomicSetW = crate::AtomicSetW<CHAN0_PHASE_SPEC>;
    pub type AtomicClearW = crate::AtomicClearW<CHAN0_PHASE_SPEC>;
    pub type AtomicXorW = crate::AtomicXorW<CHAN0_PHASE_SPEC>;
    /// Counter preload in PWM clock counts. Keep it within the effective range.
    /// Field write behavior: Plain.
    pub mod phase {
        pub type R = crate::FieldReader<u32>;
        pub type W<'a> = crate::FieldWriter<'a, super::W, 0, 32>;
        pub type ASW<'a> = crate::AtomicSetFieldWriter<'a, super::AtomicSetW, 0, 32>;
        pub type ACW<'a> = crate::AtomicClearFieldWriter<'a, super::AtomicClearW, 0, 32>;
        pub type AXW<'a> = crate::AtomicXorFieldWriter<'a, super::AtomicXorW, 0, 32>;
    }
    impl R {
        #[inline(always)]
        pub fn phase(&self) -> phase::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffffffff)
        }
    }
    impl W {
        #[inline(always)]
        pub fn phase(&mut self) -> phase::W<'_> {
            phase::W::new(self)
        }
    }
    impl AtomicSetW {
        /// Counter preload in PWM clock counts. Keep it within the effective range.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn phase(&mut self) -> phase::ASW<'_> {
            phase::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// Counter preload in PWM clock counts. Keep it within the effective range.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn phase(&mut self) -> phase::ACW<'_> {
            phase::ACW::new(self)
        }
    }
    impl AtomicXorW {
        /// Counter preload in PWM clock counts. Keep it within the effective range.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn phase(&mut self) -> phase::AXW<'_> {
            phase::AXW::new(self)
        }
    }
}

/// Channel 0's own duty value, used when `bind=0` and `usefifo=0`. A new value is latched at the next rollover.
/// Hardware access: read Plain; write Plain.
pub type CHAN0_DUTY = Reg<chan0_duty::CHAN0_DUTY_SPEC>;
/// Channel 0's own duty value, used when `bind=0` and `usefifo=0`. A new value is latched at the next rollover.
/// Hardware access: read Plain; write Plain.
pub mod chan0_duty {
    pub struct CHAN0_DUTY_SPEC;
    impl crate::RegisterSpec for CHAN0_DUTY_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CHAN0_DUTY_SPEC {}
    pub type R = crate::R<CHAN0_DUTY_SPEC>;
    impl crate::Modifiable for CHAN0_DUTY_SPEC {}
    impl crate::UnsafeWritable for CHAN0_DUTY_SPEC {}
    impl crate::ResetValue for CHAN0_DUTY_SPEC {
        const RESET_VALUE: u32 = 0x0;
    }
    impl crate::TypedAtomicSet for CHAN0_DUTY_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicSet for CHAN0_DUTY_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::TypedAtomicClear for CHAN0_DUTY_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicClear for CHAN0_DUTY_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::TypedAtomicXor for CHAN0_DUTY_SPEC {
        const OFFSET: usize = 0x1000;
    }
    impl crate::UnsafeAtomicXor for CHAN0_DUTY_SPEC {
        const OFFSET: usize = 0x1000;
    }
    pub type W = crate::W<CHAN0_DUTY_SPEC>;
    pub type AtomicSetW = crate::AtomicSetW<CHAN0_DUTY_SPEC>;
    pub type AtomicClearW = crate::AtomicClearW<CHAN0_DUTY_SPEC>;
    pub type AtomicXorW = crate::AtomicXorW<CHAN0_DUTY_SPEC>;
    /// Meaning depends on `mode`: compare value, density numerator, pulse position or serial data word.
    /// Field write behavior: Plain.
    pub mod duty {
        pub type R = crate::FieldReader<u32>;
        pub type W<'a> = crate::FieldWriter<'a, super::W, 0, 32>;
        pub type ASW<'a> = crate::AtomicSetFieldWriter<'a, super::AtomicSetW, 0, 32>;
        pub type ACW<'a> = crate::AtomicClearFieldWriter<'a, super::AtomicClearW, 0, 32>;
        pub type AXW<'a> = crate::AtomicXorFieldWriter<'a, super::AtomicXorW, 0, 32>;
    }
    impl R {
        #[inline(always)]
        pub fn duty(&self) -> duty::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffffffff)
        }
    }
    impl W {
        #[inline(always)]
        pub fn duty(&mut self) -> duty::W<'_> {
            duty::W::new(self)
        }
    }
    impl AtomicSetW {
        /// Meaning depends on `mode`: compare value, density numerator, pulse position or serial data word.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn duty(&mut self) -> duty::ASW<'_> {
            duty::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// Meaning depends on `mode`: compare value, density numerator, pulse position or serial data word.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn duty(&mut self) -> duty::ACW<'_> {
            duty::ACW::new(self)
        }
    }
    impl AtomicXorW {
        /// Meaning depends on `mode`: compare value, density numerator, pulse position or serial data word.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn duty(&mut self) -> duty::AXW<'_> {
            duty::AXW::new(self)
        }
    }
}

/// Waveform setup for channel 1. Changes take effect after a `global_ctrl.set_update` write.
/// Hardware access: read Plain; write Plain.
pub type CHAN1_CTRL = Reg<chan1_ctrl::CHAN1_CTRL_SPEC>;
/// Waveform setup for channel 1. Changes take effect after a `global_ctrl.set_update` write.
/// Hardware access: read Plain; write Plain.
pub mod chan1_ctrl {
    pub struct CHAN1_CTRL_SPEC;
    impl crate::RegisterSpec for CHAN1_CTRL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CHAN1_CTRL_SPEC {}
    pub type R = crate::R<CHAN1_CTRL_SPEC>;
    impl crate::Modifiable for CHAN1_CTRL_SPEC {}
    impl crate::UnsafeWritable for CHAN1_CTRL_SPEC {}
    impl crate::TypedAtomicSet for CHAN1_CTRL_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicSet for CHAN1_CTRL_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::TypedAtomicClear for CHAN1_CTRL_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicClear for CHAN1_CTRL_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::TypedAtomicXor for CHAN1_CTRL_SPEC {
        const OFFSET: usize = 0x1000;
    }
    impl crate::UnsafeAtomicXor for CHAN1_CTRL_SPEC {
        const OFFSET: usize = 0x1000;
    }
    pub type W = crate::W<CHAN1_CTRL_SPEC>;
    pub type AtomicSetW = crate::AtomicSetW<CHAN1_CTRL_SPEC>;
    pub type AtomicClearW = crate::AtomicClearW<CHAN1_CTRL_SPEC>;
    pub type AtomicXorW = crate::AtomicXorW<CHAN1_CTRL_SPEC>;
    /// Waveform mode. 0: output 0 (before inversion). 1: trailing-edge PWM, counter counts up. 2: centre-aligned pulse, counter counts down then up. 3: pulse density, high for DUTY/RANGE of the steps; each step adds DUTY to an accumulator, and when the sum reaches RANGE a 1 is output and RANGE is subtracted. 4: serial output starting at bit 31. 5: one high slot when COUNT equals DUTY. 6: leading-edge PWM, counter counts down. 7: serial output starting at bit 0. Modes 1, 2 and 6 output high while COUNT < DUTY. Serial modes wrap the bit index modulo 32.
    /// Field write behavior: Plain.
    pub mod mode {
        pub type R = crate::FieldReader<u32>;
        pub type W<'a> = crate::FieldWriter<'a, super::W, 0, 3>;
        pub type ASW<'a> = crate::AtomicSetFieldWriter<'a, super::AtomicSetW, 0, 3>;
        pub type ACW<'a> = crate::AtomicClearFieldWriter<'a, super::AtomicClearW, 0, 3>;
        pub type AXW<'a> = crate::AtomicXorFieldWriter<'a, super::AtomicXorW, 0, 3>;
    }
    /// 1 inverts the output after mode processing; 0 leaves it unchanged.
    /// Field write behavior: Plain.
    pub mod invert {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 3>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 3>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 3>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 3>;
    }
    /// 0 uses this channel's own range and duty registers. 1 uses `common_range`, with duty from `common_duty` or the FIFO.
    /// Field write behavior: Plain.
    pub mod bind {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 4>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 4>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 4>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 4>;
    }
    /// 0 takes duty from a register; 1 takes duty from the FIFO. Needs `bind=1`; using the FIFO without bind gives unpredictable results.
    /// Field write behavior: Plain.
    pub mod usefifo {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 5>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 5>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 5>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 5>;
    }
    /// 1 enables second-order sigma-delta noise shaping: the duty value is treated as signed 16-bit and quantized to `sdm_bitwidth`+1 bits, advancing on the channel's duty strobe. 0 bypasses shaping.
    /// Field write behavior: Plain.
    pub mod sdm {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 6>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 6>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 6>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 6>;
    }
    /// 1 adds a one-LSB dither into the sigma-delta loop to break up idle tones; 0 turns it off. Only has an effect when `sdm=1`.
    /// Field write behavior: Plain.
    pub mod dither {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 7>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 7>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 7>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 7>;
    }
    /// 1 lets this channel's rollover contribute to the shared FIFO pop (an OR across bound channels). 0 keeps its rollover from popping.
    /// Field write behavior: Plain.
    pub mod fifo_pop_mask {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 8>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 8>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 8>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 8>;
    }
    /// Sigma-delta output precision minus one: values 0-15 give 1-16 bits.
    /// Field write behavior: Plain.
    pub mod sdm_bitwidth {
        pub type R = crate::FieldReader<u32>;
        pub type W<'a> = crate::FieldWriter<'a, super::W, 12, 4>;
        pub type ASW<'a> = crate::AtomicSetFieldWriter<'a, super::AtomicSetW, 12, 4>;
        pub type ACW<'a> = crate::AtomicClearFieldWriter<'a, super::AtomicClearW, 12, 4>;
        pub type AXW<'a> = crate::AtomicXorFieldWriter<'a, super::AtomicXorW, 12, 4>;
    }
    /// Unsigned offset added to the quantized PWM code, for example to move a signed source into an unsigned range.
    /// Field write behavior: Plain.
    pub mod sdm_bias {
        pub type R = crate::FieldReader<u32>;
        pub type W<'a> = crate::FieldWriter<'a, super::W, 16, 16>;
        pub type ASW<'a> = crate::AtomicSetFieldWriter<'a, super::AtomicSetW, 16, 16>;
        pub type ACW<'a> = crate::AtomicClearFieldWriter<'a, super::AtomicClearW, 16, 16>;
        pub type AXW<'a> = crate::AtomicXorFieldWriter<'a, super::AtomicXorW, 16, 16>;
    }
    impl R {
        #[inline(always)]
        pub fn mode(&self) -> mode::R {
            crate::FieldReader::new((self.bits() >> 0) & 0x7)
        }
        #[inline(always)]
        pub fn invert(&self) -> invert::R {
            crate::BitReader::new(((self.bits() >> 3) & 1) != 0)
        }
        #[inline(always)]
        pub fn bind(&self) -> bind::R {
            crate::BitReader::new(((self.bits() >> 4) & 1) != 0)
        }
        #[inline(always)]
        pub fn usefifo(&self) -> usefifo::R {
            crate::BitReader::new(((self.bits() >> 5) & 1) != 0)
        }
        #[inline(always)]
        pub fn sdm(&self) -> sdm::R {
            crate::BitReader::new(((self.bits() >> 6) & 1) != 0)
        }
        #[inline(always)]
        pub fn dither(&self) -> dither::R {
            crate::BitReader::new(((self.bits() >> 7) & 1) != 0)
        }
        #[inline(always)]
        pub fn fifo_pop_mask(&self) -> fifo_pop_mask::R {
            crate::BitReader::new(((self.bits() >> 8) & 1) != 0)
        }
        #[inline(always)]
        pub fn sdm_bitwidth(&self) -> sdm_bitwidth::R {
            crate::FieldReader::new((self.bits() >> 12) & 0xf)
        }
        #[inline(always)]
        pub fn sdm_bias(&self) -> sdm_bias::R {
            crate::FieldReader::new((self.bits() >> 16) & 0xffff)
        }
    }
    impl W {
        #[inline(always)]
        pub fn mode(&mut self) -> mode::W<'_> {
            mode::W::new(self)
        }
        #[inline(always)]
        pub fn invert(&mut self) -> invert::W<'_> {
            invert::W::new(self)
        }
        #[inline(always)]
        pub fn bind(&mut self) -> bind::W<'_> {
            bind::W::new(self)
        }
        #[inline(always)]
        pub fn usefifo(&mut self) -> usefifo::W<'_> {
            usefifo::W::new(self)
        }
        #[inline(always)]
        pub fn sdm(&mut self) -> sdm::W<'_> {
            sdm::W::new(self)
        }
        #[inline(always)]
        pub fn dither(&mut self) -> dither::W<'_> {
            dither::W::new(self)
        }
        #[inline(always)]
        pub fn fifo_pop_mask(&mut self) -> fifo_pop_mask::W<'_> {
            fifo_pop_mask::W::new(self)
        }
        #[inline(always)]
        pub fn sdm_bitwidth(&mut self) -> sdm_bitwidth::W<'_> {
            sdm_bitwidth::W::new(self)
        }
        #[inline(always)]
        pub fn sdm_bias(&mut self) -> sdm_bias::W<'_> {
            sdm_bias::W::new(self)
        }
    }
    impl AtomicSetW {
        /// Waveform mode. 0: output 0 (before inversion). 1: trailing-edge PWM, counter counts up. 2: centre-aligned pulse, counter counts down then up. 3: pulse density, high for DUTY/RANGE of the steps; each step adds DUTY to an accumulator, and when the sum reaches RANGE a 1 is output and RANGE is subtracted. 4: serial output starting at bit 31. 5: one high slot when COUNT equals DUTY. 6: leading-edge PWM, counter counts down. 7: serial output starting at bit 0. Modes 1, 2 and 6 output high while COUNT < DUTY. Serial modes wrap the bit index modulo 32.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn mode(&mut self) -> mode::ASW<'_> {
            mode::ASW::new(self)
        }
        /// 1 inverts the output after mode processing; 0 leaves it unchanged.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn invert(&mut self) -> invert::ASW<'_> {
            invert::ASW::new(self)
        }
        /// 0 uses this channel's own range and duty registers. 1 uses `common_range`, with duty from `common_duty` or the FIFO.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn bind(&mut self) -> bind::ASW<'_> {
            bind::ASW::new(self)
        }
        /// 0 takes duty from a register; 1 takes duty from the FIFO. Needs `bind=1`; using the FIFO without bind gives unpredictable results.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn usefifo(&mut self) -> usefifo::ASW<'_> {
            usefifo::ASW::new(self)
        }
        /// 1 enables second-order sigma-delta noise shaping: the duty value is treated as signed 16-bit and quantized to `sdm_bitwidth`+1 bits, advancing on the channel's duty strobe. 0 bypasses shaping.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn sdm(&mut self) -> sdm::ASW<'_> {
            sdm::ASW::new(self)
        }
        /// 1 adds a one-LSB dither into the sigma-delta loop to break up idle tones; 0 turns it off. Only has an effect when `sdm=1`.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn dither(&mut self) -> dither::ASW<'_> {
            dither::ASW::new(self)
        }
        /// 1 lets this channel's rollover contribute to the shared FIFO pop (an OR across bound channels). 0 keeps its rollover from popping.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn fifo_pop_mask(&mut self) -> fifo_pop_mask::ASW<'_> {
            fifo_pop_mask::ASW::new(self)
        }
        /// Sigma-delta output precision minus one: values 0-15 give 1-16 bits.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn sdm_bitwidth(&mut self) -> sdm_bitwidth::ASW<'_> {
            sdm_bitwidth::ASW::new(self)
        }
        /// Unsigned offset added to the quantized PWM code, for example to move a signed source into an unsigned range.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn sdm_bias(&mut self) -> sdm_bias::ASW<'_> {
            sdm_bias::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// Waveform mode. 0: output 0 (before inversion). 1: trailing-edge PWM, counter counts up. 2: centre-aligned pulse, counter counts down then up. 3: pulse density, high for DUTY/RANGE of the steps; each step adds DUTY to an accumulator, and when the sum reaches RANGE a 1 is output and RANGE is subtracted. 4: serial output starting at bit 31. 5: one high slot when COUNT equals DUTY. 6: leading-edge PWM, counter counts down. 7: serial output starting at bit 0. Modes 1, 2 and 6 output high while COUNT < DUTY. Serial modes wrap the bit index modulo 32.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn mode(&mut self) -> mode::ACW<'_> {
            mode::ACW::new(self)
        }
        /// 1 inverts the output after mode processing; 0 leaves it unchanged.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn invert(&mut self) -> invert::ACW<'_> {
            invert::ACW::new(self)
        }
        /// 0 uses this channel's own range and duty registers. 1 uses `common_range`, with duty from `common_duty` or the FIFO.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn bind(&mut self) -> bind::ACW<'_> {
            bind::ACW::new(self)
        }
        /// 0 takes duty from a register; 1 takes duty from the FIFO. Needs `bind=1`; using the FIFO without bind gives unpredictable results.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn usefifo(&mut self) -> usefifo::ACW<'_> {
            usefifo::ACW::new(self)
        }
        /// 1 enables second-order sigma-delta noise shaping: the duty value is treated as signed 16-bit and quantized to `sdm_bitwidth`+1 bits, advancing on the channel's duty strobe. 0 bypasses shaping.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn sdm(&mut self) -> sdm::ACW<'_> {
            sdm::ACW::new(self)
        }
        /// 1 adds a one-LSB dither into the sigma-delta loop to break up idle tones; 0 turns it off. Only has an effect when `sdm=1`.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn dither(&mut self) -> dither::ACW<'_> {
            dither::ACW::new(self)
        }
        /// 1 lets this channel's rollover contribute to the shared FIFO pop (an OR across bound channels). 0 keeps its rollover from popping.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn fifo_pop_mask(&mut self) -> fifo_pop_mask::ACW<'_> {
            fifo_pop_mask::ACW::new(self)
        }
        /// Sigma-delta output precision minus one: values 0-15 give 1-16 bits.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn sdm_bitwidth(&mut self) -> sdm_bitwidth::ACW<'_> {
            sdm_bitwidth::ACW::new(self)
        }
        /// Unsigned offset added to the quantized PWM code, for example to move a signed source into an unsigned range.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn sdm_bias(&mut self) -> sdm_bias::ACW<'_> {
            sdm_bias::ACW::new(self)
        }
    }
    impl AtomicXorW {
        /// Waveform mode. 0: output 0 (before inversion). 1: trailing-edge PWM, counter counts up. 2: centre-aligned pulse, counter counts down then up. 3: pulse density, high for DUTY/RANGE of the steps; each step adds DUTY to an accumulator, and when the sum reaches RANGE a 1 is output and RANGE is subtracted. 4: serial output starting at bit 31. 5: one high slot when COUNT equals DUTY. 6: leading-edge PWM, counter counts down. 7: serial output starting at bit 0. Modes 1, 2 and 6 output high while COUNT < DUTY. Serial modes wrap the bit index modulo 32.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn mode(&mut self) -> mode::AXW<'_> {
            mode::AXW::new(self)
        }
        /// 1 inverts the output after mode processing; 0 leaves it unchanged.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn invert(&mut self) -> invert::AXW<'_> {
            invert::AXW::new(self)
        }
        /// 0 uses this channel's own range and duty registers. 1 uses `common_range`, with duty from `common_duty` or the FIFO.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn bind(&mut self) -> bind::AXW<'_> {
            bind::AXW::new(self)
        }
        /// 0 takes duty from a register; 1 takes duty from the FIFO. Needs `bind=1`; using the FIFO without bind gives unpredictable results.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn usefifo(&mut self) -> usefifo::AXW<'_> {
            usefifo::AXW::new(self)
        }
        /// 1 enables second-order sigma-delta noise shaping: the duty value is treated as signed 16-bit and quantized to `sdm_bitwidth`+1 bits, advancing on the channel's duty strobe. 0 bypasses shaping.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn sdm(&mut self) -> sdm::AXW<'_> {
            sdm::AXW::new(self)
        }
        /// 1 adds a one-LSB dither into the sigma-delta loop to break up idle tones; 0 turns it off. Only has an effect when `sdm=1`.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn dither(&mut self) -> dither::AXW<'_> {
            dither::AXW::new(self)
        }
        /// 1 lets this channel's rollover contribute to the shared FIFO pop (an OR across bound channels). 0 keeps its rollover from popping.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn fifo_pop_mask(&mut self) -> fifo_pop_mask::AXW<'_> {
            fifo_pop_mask::AXW::new(self)
        }
        /// Sigma-delta output precision minus one: values 0-15 give 1-16 bits.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn sdm_bitwidth(&mut self) -> sdm_bitwidth::AXW<'_> {
            sdm_bitwidth::AXW::new(self)
        }
        /// Unsigned offset added to the quantized PWM code, for example to move a signed source into an unsigned range.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn sdm_bias(&mut self) -> sdm_bias::AXW<'_> {
            sdm_bias::AXW::new(self)
        }
    }
}

/// Channel 1's own counter limit, used when `bind=0`. A write takes effect at the next rollover.
/// Hardware access: read Plain; write Plain.
pub type CHAN1_RANGE = Reg<chan1_range::CHAN1_RANGE_SPEC>;
/// Channel 1's own counter limit, used when `bind=0`. A write takes effect at the next rollover.
/// Hardware access: read Plain; write Plain.
pub mod chan1_range {
    pub struct CHAN1_RANGE_SPEC;
    impl crate::RegisterSpec for CHAN1_RANGE_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CHAN1_RANGE_SPEC {}
    pub type R = crate::R<CHAN1_RANGE_SPEC>;
    impl crate::Modifiable for CHAN1_RANGE_SPEC {}
    impl crate::UnsafeWritable for CHAN1_RANGE_SPEC {}
    impl crate::ResetValue for CHAN1_RANGE_SPEC {
        const RESET_VALUE: u32 = 0x0;
    }
    impl crate::TypedAtomicSet for CHAN1_RANGE_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicSet for CHAN1_RANGE_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::TypedAtomicClear for CHAN1_RANGE_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicClear for CHAN1_RANGE_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::TypedAtomicXor for CHAN1_RANGE_SPEC {
        const OFFSET: usize = 0x1000;
    }
    impl crate::UnsafeAtomicXor for CHAN1_RANGE_SPEC {
        const OFFSET: usize = 0x1000;
    }
    pub type W = crate::W<CHAN1_RANGE_SPEC>;
    pub type AtomicSetW = crate::AtomicSetW<CHAN1_RANGE_SPEC>;
    pub type AtomicClearW = crate::AtomicClearW<CHAN1_RANGE_SPEC>;
    pub type AtomicXorW = crate::AtomicXorW<CHAN1_RANGE_SPEC>;
    /// Counter limit in PWM clock counts.
    /// Field write behavior: Plain.
    pub mod range {
        pub type R = crate::FieldReader<u32>;
        pub type W<'a> = crate::FieldWriter<'a, super::W, 0, 32>;
        pub type ASW<'a> = crate::AtomicSetFieldWriter<'a, super::AtomicSetW, 0, 32>;
        pub type ACW<'a> = crate::AtomicClearFieldWriter<'a, super::AtomicClearW, 0, 32>;
        pub type AXW<'a> = crate::AtomicXorFieldWriter<'a, super::AtomicXorW, 0, 32>;
    }
    impl R {
        #[inline(always)]
        pub fn range(&self) -> range::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffffffff)
        }
    }
    impl W {
        #[inline(always)]
        pub fn range(&mut self) -> range::W<'_> {
            range::W::new(self)
        }
    }
    impl AtomicSetW {
        /// Counter limit in PWM clock counts.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn range(&mut self) -> range::ASW<'_> {
            range::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// Counter limit in PWM clock counts.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn range(&mut self) -> range::ACW<'_> {
            range::ACW::new(self)
        }
    }
    impl AtomicXorW {
        /// Counter limit in PWM clock counts.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn range(&mut self) -> range::AXW<'_> {
            range::AXW::new(self)
        }
    }
}

/// Starting counter offset for channel 1, applied by `global_ctrl.set_update`.
/// Hardware access: read Plain; write Plain.
pub type CHAN1_PHASE = Reg<chan1_phase::CHAN1_PHASE_SPEC>;
/// Starting counter offset for channel 1, applied by `global_ctrl.set_update`.
/// Hardware access: read Plain; write Plain.
pub mod chan1_phase {
    pub struct CHAN1_PHASE_SPEC;
    impl crate::RegisterSpec for CHAN1_PHASE_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CHAN1_PHASE_SPEC {}
    pub type R = crate::R<CHAN1_PHASE_SPEC>;
    impl crate::Modifiable for CHAN1_PHASE_SPEC {}
    impl crate::UnsafeWritable for CHAN1_PHASE_SPEC {}
    impl crate::ResetValue for CHAN1_PHASE_SPEC {
        const RESET_VALUE: u32 = 0x0;
    }
    impl crate::TypedAtomicSet for CHAN1_PHASE_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicSet for CHAN1_PHASE_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::TypedAtomicClear for CHAN1_PHASE_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicClear for CHAN1_PHASE_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::TypedAtomicXor for CHAN1_PHASE_SPEC {
        const OFFSET: usize = 0x1000;
    }
    impl crate::UnsafeAtomicXor for CHAN1_PHASE_SPEC {
        const OFFSET: usize = 0x1000;
    }
    pub type W = crate::W<CHAN1_PHASE_SPEC>;
    pub type AtomicSetW = crate::AtomicSetW<CHAN1_PHASE_SPEC>;
    pub type AtomicClearW = crate::AtomicClearW<CHAN1_PHASE_SPEC>;
    pub type AtomicXorW = crate::AtomicXorW<CHAN1_PHASE_SPEC>;
    /// Counter preload in PWM clock counts. Keep it within the effective range.
    /// Field write behavior: Plain.
    pub mod phase {
        pub type R = crate::FieldReader<u32>;
        pub type W<'a> = crate::FieldWriter<'a, super::W, 0, 32>;
        pub type ASW<'a> = crate::AtomicSetFieldWriter<'a, super::AtomicSetW, 0, 32>;
        pub type ACW<'a> = crate::AtomicClearFieldWriter<'a, super::AtomicClearW, 0, 32>;
        pub type AXW<'a> = crate::AtomicXorFieldWriter<'a, super::AtomicXorW, 0, 32>;
    }
    impl R {
        #[inline(always)]
        pub fn phase(&self) -> phase::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffffffff)
        }
    }
    impl W {
        #[inline(always)]
        pub fn phase(&mut self) -> phase::W<'_> {
            phase::W::new(self)
        }
    }
    impl AtomicSetW {
        /// Counter preload in PWM clock counts. Keep it within the effective range.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn phase(&mut self) -> phase::ASW<'_> {
            phase::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// Counter preload in PWM clock counts. Keep it within the effective range.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn phase(&mut self) -> phase::ACW<'_> {
            phase::ACW::new(self)
        }
    }
    impl AtomicXorW {
        /// Counter preload in PWM clock counts. Keep it within the effective range.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn phase(&mut self) -> phase::AXW<'_> {
            phase::AXW::new(self)
        }
    }
}

/// Channel 1's own duty value, used when `bind=0` and `usefifo=0`. A new value is latched at the next rollover.
/// Hardware access: read Plain; write Plain.
pub type CHAN1_DUTY = Reg<chan1_duty::CHAN1_DUTY_SPEC>;
/// Channel 1's own duty value, used when `bind=0` and `usefifo=0`. A new value is latched at the next rollover.
/// Hardware access: read Plain; write Plain.
pub mod chan1_duty {
    pub struct CHAN1_DUTY_SPEC;
    impl crate::RegisterSpec for CHAN1_DUTY_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CHAN1_DUTY_SPEC {}
    pub type R = crate::R<CHAN1_DUTY_SPEC>;
    impl crate::Modifiable for CHAN1_DUTY_SPEC {}
    impl crate::UnsafeWritable for CHAN1_DUTY_SPEC {}
    impl crate::ResetValue for CHAN1_DUTY_SPEC {
        const RESET_VALUE: u32 = 0x0;
    }
    impl crate::TypedAtomicSet for CHAN1_DUTY_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicSet for CHAN1_DUTY_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::TypedAtomicClear for CHAN1_DUTY_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicClear for CHAN1_DUTY_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::TypedAtomicXor for CHAN1_DUTY_SPEC {
        const OFFSET: usize = 0x1000;
    }
    impl crate::UnsafeAtomicXor for CHAN1_DUTY_SPEC {
        const OFFSET: usize = 0x1000;
    }
    pub type W = crate::W<CHAN1_DUTY_SPEC>;
    pub type AtomicSetW = crate::AtomicSetW<CHAN1_DUTY_SPEC>;
    pub type AtomicClearW = crate::AtomicClearW<CHAN1_DUTY_SPEC>;
    pub type AtomicXorW = crate::AtomicXorW<CHAN1_DUTY_SPEC>;
    /// Meaning depends on `mode`: compare value, density numerator, pulse position or serial data word.
    /// Field write behavior: Plain.
    pub mod duty {
        pub type R = crate::FieldReader<u32>;
        pub type W<'a> = crate::FieldWriter<'a, super::W, 0, 32>;
        pub type ASW<'a> = crate::AtomicSetFieldWriter<'a, super::AtomicSetW, 0, 32>;
        pub type ACW<'a> = crate::AtomicClearFieldWriter<'a, super::AtomicClearW, 0, 32>;
        pub type AXW<'a> = crate::AtomicXorFieldWriter<'a, super::AtomicXorW, 0, 32>;
    }
    impl R {
        #[inline(always)]
        pub fn duty(&self) -> duty::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffffffff)
        }
    }
    impl W {
        #[inline(always)]
        pub fn duty(&mut self) -> duty::W<'_> {
            duty::W::new(self)
        }
    }
    impl AtomicSetW {
        /// Meaning depends on `mode`: compare value, density numerator, pulse position or serial data word.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn duty(&mut self) -> duty::ASW<'_> {
            duty::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// Meaning depends on `mode`: compare value, density numerator, pulse position or serial data word.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn duty(&mut self) -> duty::ACW<'_> {
            duty::ACW::new(self)
        }
    }
    impl AtomicXorW {
        /// Meaning depends on `mode`: compare value, density numerator, pulse position or serial data word.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn duty(&mut self) -> duty::AXW<'_> {
            duty::AXW::new(self)
        }
    }
}

/// Waveform setup for channel 2. Changes take effect after a `global_ctrl.set_update` write.
/// Hardware access: read Plain; write Plain.
pub type CHAN2_CTRL = Reg<chan2_ctrl::CHAN2_CTRL_SPEC>;
/// Waveform setup for channel 2. Changes take effect after a `global_ctrl.set_update` write.
/// Hardware access: read Plain; write Plain.
pub mod chan2_ctrl {
    pub struct CHAN2_CTRL_SPEC;
    impl crate::RegisterSpec for CHAN2_CTRL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CHAN2_CTRL_SPEC {}
    pub type R = crate::R<CHAN2_CTRL_SPEC>;
    impl crate::Modifiable for CHAN2_CTRL_SPEC {}
    impl crate::UnsafeWritable for CHAN2_CTRL_SPEC {}
    impl crate::TypedAtomicSet for CHAN2_CTRL_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicSet for CHAN2_CTRL_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::TypedAtomicClear for CHAN2_CTRL_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicClear for CHAN2_CTRL_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::TypedAtomicXor for CHAN2_CTRL_SPEC {
        const OFFSET: usize = 0x1000;
    }
    impl crate::UnsafeAtomicXor for CHAN2_CTRL_SPEC {
        const OFFSET: usize = 0x1000;
    }
    pub type W = crate::W<CHAN2_CTRL_SPEC>;
    pub type AtomicSetW = crate::AtomicSetW<CHAN2_CTRL_SPEC>;
    pub type AtomicClearW = crate::AtomicClearW<CHAN2_CTRL_SPEC>;
    pub type AtomicXorW = crate::AtomicXorW<CHAN2_CTRL_SPEC>;
    /// Waveform mode. 0: output 0 (before inversion). 1: trailing-edge PWM, counter counts up. 2: centre-aligned pulse, counter counts down then up. 3: pulse density, high for DUTY/RANGE of the steps; each step adds DUTY to an accumulator, and when the sum reaches RANGE a 1 is output and RANGE is subtracted. 4: serial output starting at bit 31. 5: one high slot when COUNT equals DUTY. 6: leading-edge PWM, counter counts down. 7: serial output starting at bit 0. Modes 1, 2 and 6 output high while COUNT < DUTY. Serial modes wrap the bit index modulo 32.
    /// Field write behavior: Plain.
    pub mod mode {
        pub type R = crate::FieldReader<u32>;
        pub type W<'a> = crate::FieldWriter<'a, super::W, 0, 3>;
        pub type ASW<'a> = crate::AtomicSetFieldWriter<'a, super::AtomicSetW, 0, 3>;
        pub type ACW<'a> = crate::AtomicClearFieldWriter<'a, super::AtomicClearW, 0, 3>;
        pub type AXW<'a> = crate::AtomicXorFieldWriter<'a, super::AtomicXorW, 0, 3>;
    }
    /// 1 inverts the output after mode processing; 0 leaves it unchanged.
    /// Field write behavior: Plain.
    pub mod invert {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 3>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 3>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 3>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 3>;
    }
    /// 0 uses this channel's own range and duty registers. 1 uses `common_range`, with duty from `common_duty` or the FIFO.
    /// Field write behavior: Plain.
    pub mod bind {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 4>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 4>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 4>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 4>;
    }
    /// 0 takes duty from a register; 1 takes duty from the FIFO. Needs `bind=1`; using the FIFO without bind gives unpredictable results.
    /// Field write behavior: Plain.
    pub mod usefifo {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 5>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 5>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 5>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 5>;
    }
    /// 1 enables second-order sigma-delta noise shaping: the duty value is treated as signed 16-bit and quantized to `sdm_bitwidth`+1 bits, advancing on the channel's duty strobe. 0 bypasses shaping.
    /// Field write behavior: Plain.
    pub mod sdm {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 6>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 6>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 6>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 6>;
    }
    /// 1 adds a one-LSB dither into the sigma-delta loop to break up idle tones; 0 turns it off. Only has an effect when `sdm=1`.
    /// Field write behavior: Plain.
    pub mod dither {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 7>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 7>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 7>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 7>;
    }
    /// 1 lets this channel's rollover contribute to the shared FIFO pop (an OR across bound channels). 0 keeps its rollover from popping.
    /// Field write behavior: Plain.
    pub mod fifo_pop_mask {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 8>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 8>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 8>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 8>;
    }
    /// Sigma-delta output precision minus one: values 0-15 give 1-16 bits.
    /// Field write behavior: Plain.
    pub mod sdm_bitwidth {
        pub type R = crate::FieldReader<u32>;
        pub type W<'a> = crate::FieldWriter<'a, super::W, 12, 4>;
        pub type ASW<'a> = crate::AtomicSetFieldWriter<'a, super::AtomicSetW, 12, 4>;
        pub type ACW<'a> = crate::AtomicClearFieldWriter<'a, super::AtomicClearW, 12, 4>;
        pub type AXW<'a> = crate::AtomicXorFieldWriter<'a, super::AtomicXorW, 12, 4>;
    }
    /// Unsigned offset added to the quantized PWM code, for example to move a signed source into an unsigned range.
    /// Field write behavior: Plain.
    pub mod sdm_bias {
        pub type R = crate::FieldReader<u32>;
        pub type W<'a> = crate::FieldWriter<'a, super::W, 16, 16>;
        pub type ASW<'a> = crate::AtomicSetFieldWriter<'a, super::AtomicSetW, 16, 16>;
        pub type ACW<'a> = crate::AtomicClearFieldWriter<'a, super::AtomicClearW, 16, 16>;
        pub type AXW<'a> = crate::AtomicXorFieldWriter<'a, super::AtomicXorW, 16, 16>;
    }
    impl R {
        #[inline(always)]
        pub fn mode(&self) -> mode::R {
            crate::FieldReader::new((self.bits() >> 0) & 0x7)
        }
        #[inline(always)]
        pub fn invert(&self) -> invert::R {
            crate::BitReader::new(((self.bits() >> 3) & 1) != 0)
        }
        #[inline(always)]
        pub fn bind(&self) -> bind::R {
            crate::BitReader::new(((self.bits() >> 4) & 1) != 0)
        }
        #[inline(always)]
        pub fn usefifo(&self) -> usefifo::R {
            crate::BitReader::new(((self.bits() >> 5) & 1) != 0)
        }
        #[inline(always)]
        pub fn sdm(&self) -> sdm::R {
            crate::BitReader::new(((self.bits() >> 6) & 1) != 0)
        }
        #[inline(always)]
        pub fn dither(&self) -> dither::R {
            crate::BitReader::new(((self.bits() >> 7) & 1) != 0)
        }
        #[inline(always)]
        pub fn fifo_pop_mask(&self) -> fifo_pop_mask::R {
            crate::BitReader::new(((self.bits() >> 8) & 1) != 0)
        }
        #[inline(always)]
        pub fn sdm_bitwidth(&self) -> sdm_bitwidth::R {
            crate::FieldReader::new((self.bits() >> 12) & 0xf)
        }
        #[inline(always)]
        pub fn sdm_bias(&self) -> sdm_bias::R {
            crate::FieldReader::new((self.bits() >> 16) & 0xffff)
        }
    }
    impl W {
        #[inline(always)]
        pub fn mode(&mut self) -> mode::W<'_> {
            mode::W::new(self)
        }
        #[inline(always)]
        pub fn invert(&mut self) -> invert::W<'_> {
            invert::W::new(self)
        }
        #[inline(always)]
        pub fn bind(&mut self) -> bind::W<'_> {
            bind::W::new(self)
        }
        #[inline(always)]
        pub fn usefifo(&mut self) -> usefifo::W<'_> {
            usefifo::W::new(self)
        }
        #[inline(always)]
        pub fn sdm(&mut self) -> sdm::W<'_> {
            sdm::W::new(self)
        }
        #[inline(always)]
        pub fn dither(&mut self) -> dither::W<'_> {
            dither::W::new(self)
        }
        #[inline(always)]
        pub fn fifo_pop_mask(&mut self) -> fifo_pop_mask::W<'_> {
            fifo_pop_mask::W::new(self)
        }
        #[inline(always)]
        pub fn sdm_bitwidth(&mut self) -> sdm_bitwidth::W<'_> {
            sdm_bitwidth::W::new(self)
        }
        #[inline(always)]
        pub fn sdm_bias(&mut self) -> sdm_bias::W<'_> {
            sdm_bias::W::new(self)
        }
    }
    impl AtomicSetW {
        /// Waveform mode. 0: output 0 (before inversion). 1: trailing-edge PWM, counter counts up. 2: centre-aligned pulse, counter counts down then up. 3: pulse density, high for DUTY/RANGE of the steps; each step adds DUTY to an accumulator, and when the sum reaches RANGE a 1 is output and RANGE is subtracted. 4: serial output starting at bit 31. 5: one high slot when COUNT equals DUTY. 6: leading-edge PWM, counter counts down. 7: serial output starting at bit 0. Modes 1, 2 and 6 output high while COUNT < DUTY. Serial modes wrap the bit index modulo 32.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn mode(&mut self) -> mode::ASW<'_> {
            mode::ASW::new(self)
        }
        /// 1 inverts the output after mode processing; 0 leaves it unchanged.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn invert(&mut self) -> invert::ASW<'_> {
            invert::ASW::new(self)
        }
        /// 0 uses this channel's own range and duty registers. 1 uses `common_range`, with duty from `common_duty` or the FIFO.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn bind(&mut self) -> bind::ASW<'_> {
            bind::ASW::new(self)
        }
        /// 0 takes duty from a register; 1 takes duty from the FIFO. Needs `bind=1`; using the FIFO without bind gives unpredictable results.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn usefifo(&mut self) -> usefifo::ASW<'_> {
            usefifo::ASW::new(self)
        }
        /// 1 enables second-order sigma-delta noise shaping: the duty value is treated as signed 16-bit and quantized to `sdm_bitwidth`+1 bits, advancing on the channel's duty strobe. 0 bypasses shaping.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn sdm(&mut self) -> sdm::ASW<'_> {
            sdm::ASW::new(self)
        }
        /// 1 adds a one-LSB dither into the sigma-delta loop to break up idle tones; 0 turns it off. Only has an effect when `sdm=1`.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn dither(&mut self) -> dither::ASW<'_> {
            dither::ASW::new(self)
        }
        /// 1 lets this channel's rollover contribute to the shared FIFO pop (an OR across bound channels). 0 keeps its rollover from popping.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn fifo_pop_mask(&mut self) -> fifo_pop_mask::ASW<'_> {
            fifo_pop_mask::ASW::new(self)
        }
        /// Sigma-delta output precision minus one: values 0-15 give 1-16 bits.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn sdm_bitwidth(&mut self) -> sdm_bitwidth::ASW<'_> {
            sdm_bitwidth::ASW::new(self)
        }
        /// Unsigned offset added to the quantized PWM code, for example to move a signed source into an unsigned range.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn sdm_bias(&mut self) -> sdm_bias::ASW<'_> {
            sdm_bias::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// Waveform mode. 0: output 0 (before inversion). 1: trailing-edge PWM, counter counts up. 2: centre-aligned pulse, counter counts down then up. 3: pulse density, high for DUTY/RANGE of the steps; each step adds DUTY to an accumulator, and when the sum reaches RANGE a 1 is output and RANGE is subtracted. 4: serial output starting at bit 31. 5: one high slot when COUNT equals DUTY. 6: leading-edge PWM, counter counts down. 7: serial output starting at bit 0. Modes 1, 2 and 6 output high while COUNT < DUTY. Serial modes wrap the bit index modulo 32.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn mode(&mut self) -> mode::ACW<'_> {
            mode::ACW::new(self)
        }
        /// 1 inverts the output after mode processing; 0 leaves it unchanged.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn invert(&mut self) -> invert::ACW<'_> {
            invert::ACW::new(self)
        }
        /// 0 uses this channel's own range and duty registers. 1 uses `common_range`, with duty from `common_duty` or the FIFO.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn bind(&mut self) -> bind::ACW<'_> {
            bind::ACW::new(self)
        }
        /// 0 takes duty from a register; 1 takes duty from the FIFO. Needs `bind=1`; using the FIFO without bind gives unpredictable results.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn usefifo(&mut self) -> usefifo::ACW<'_> {
            usefifo::ACW::new(self)
        }
        /// 1 enables second-order sigma-delta noise shaping: the duty value is treated as signed 16-bit and quantized to `sdm_bitwidth`+1 bits, advancing on the channel's duty strobe. 0 bypasses shaping.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn sdm(&mut self) -> sdm::ACW<'_> {
            sdm::ACW::new(self)
        }
        /// 1 adds a one-LSB dither into the sigma-delta loop to break up idle tones; 0 turns it off. Only has an effect when `sdm=1`.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn dither(&mut self) -> dither::ACW<'_> {
            dither::ACW::new(self)
        }
        /// 1 lets this channel's rollover contribute to the shared FIFO pop (an OR across bound channels). 0 keeps its rollover from popping.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn fifo_pop_mask(&mut self) -> fifo_pop_mask::ACW<'_> {
            fifo_pop_mask::ACW::new(self)
        }
        /// Sigma-delta output precision minus one: values 0-15 give 1-16 bits.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn sdm_bitwidth(&mut self) -> sdm_bitwidth::ACW<'_> {
            sdm_bitwidth::ACW::new(self)
        }
        /// Unsigned offset added to the quantized PWM code, for example to move a signed source into an unsigned range.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn sdm_bias(&mut self) -> sdm_bias::ACW<'_> {
            sdm_bias::ACW::new(self)
        }
    }
    impl AtomicXorW {
        /// Waveform mode. 0: output 0 (before inversion). 1: trailing-edge PWM, counter counts up. 2: centre-aligned pulse, counter counts down then up. 3: pulse density, high for DUTY/RANGE of the steps; each step adds DUTY to an accumulator, and when the sum reaches RANGE a 1 is output and RANGE is subtracted. 4: serial output starting at bit 31. 5: one high slot when COUNT equals DUTY. 6: leading-edge PWM, counter counts down. 7: serial output starting at bit 0. Modes 1, 2 and 6 output high while COUNT < DUTY. Serial modes wrap the bit index modulo 32.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn mode(&mut self) -> mode::AXW<'_> {
            mode::AXW::new(self)
        }
        /// 1 inverts the output after mode processing; 0 leaves it unchanged.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn invert(&mut self) -> invert::AXW<'_> {
            invert::AXW::new(self)
        }
        /// 0 uses this channel's own range and duty registers. 1 uses `common_range`, with duty from `common_duty` or the FIFO.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn bind(&mut self) -> bind::AXW<'_> {
            bind::AXW::new(self)
        }
        /// 0 takes duty from a register; 1 takes duty from the FIFO. Needs `bind=1`; using the FIFO without bind gives unpredictable results.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn usefifo(&mut self) -> usefifo::AXW<'_> {
            usefifo::AXW::new(self)
        }
        /// 1 enables second-order sigma-delta noise shaping: the duty value is treated as signed 16-bit and quantized to `sdm_bitwidth`+1 bits, advancing on the channel's duty strobe. 0 bypasses shaping.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn sdm(&mut self) -> sdm::AXW<'_> {
            sdm::AXW::new(self)
        }
        /// 1 adds a one-LSB dither into the sigma-delta loop to break up idle tones; 0 turns it off. Only has an effect when `sdm=1`.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn dither(&mut self) -> dither::AXW<'_> {
            dither::AXW::new(self)
        }
        /// 1 lets this channel's rollover contribute to the shared FIFO pop (an OR across bound channels). 0 keeps its rollover from popping.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn fifo_pop_mask(&mut self) -> fifo_pop_mask::AXW<'_> {
            fifo_pop_mask::AXW::new(self)
        }
        /// Sigma-delta output precision minus one: values 0-15 give 1-16 bits.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn sdm_bitwidth(&mut self) -> sdm_bitwidth::AXW<'_> {
            sdm_bitwidth::AXW::new(self)
        }
        /// Unsigned offset added to the quantized PWM code, for example to move a signed source into an unsigned range.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn sdm_bias(&mut self) -> sdm_bias::AXW<'_> {
            sdm_bias::AXW::new(self)
        }
    }
}

/// Channel 2's own counter limit, used when `bind=0`. A write takes effect at the next rollover.
/// Hardware access: read Plain; write Plain.
pub type CHAN2_RANGE = Reg<chan2_range::CHAN2_RANGE_SPEC>;
/// Channel 2's own counter limit, used when `bind=0`. A write takes effect at the next rollover.
/// Hardware access: read Plain; write Plain.
pub mod chan2_range {
    pub struct CHAN2_RANGE_SPEC;
    impl crate::RegisterSpec for CHAN2_RANGE_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CHAN2_RANGE_SPEC {}
    pub type R = crate::R<CHAN2_RANGE_SPEC>;
    impl crate::Modifiable for CHAN2_RANGE_SPEC {}
    impl crate::UnsafeWritable for CHAN2_RANGE_SPEC {}
    impl crate::ResetValue for CHAN2_RANGE_SPEC {
        const RESET_VALUE: u32 = 0x0;
    }
    impl crate::TypedAtomicSet for CHAN2_RANGE_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicSet for CHAN2_RANGE_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::TypedAtomicClear for CHAN2_RANGE_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicClear for CHAN2_RANGE_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::TypedAtomicXor for CHAN2_RANGE_SPEC {
        const OFFSET: usize = 0x1000;
    }
    impl crate::UnsafeAtomicXor for CHAN2_RANGE_SPEC {
        const OFFSET: usize = 0x1000;
    }
    pub type W = crate::W<CHAN2_RANGE_SPEC>;
    pub type AtomicSetW = crate::AtomicSetW<CHAN2_RANGE_SPEC>;
    pub type AtomicClearW = crate::AtomicClearW<CHAN2_RANGE_SPEC>;
    pub type AtomicXorW = crate::AtomicXorW<CHAN2_RANGE_SPEC>;
    /// Counter limit in PWM clock counts.
    /// Field write behavior: Plain.
    pub mod range {
        pub type R = crate::FieldReader<u32>;
        pub type W<'a> = crate::FieldWriter<'a, super::W, 0, 32>;
        pub type ASW<'a> = crate::AtomicSetFieldWriter<'a, super::AtomicSetW, 0, 32>;
        pub type ACW<'a> = crate::AtomicClearFieldWriter<'a, super::AtomicClearW, 0, 32>;
        pub type AXW<'a> = crate::AtomicXorFieldWriter<'a, super::AtomicXorW, 0, 32>;
    }
    impl R {
        #[inline(always)]
        pub fn range(&self) -> range::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffffffff)
        }
    }
    impl W {
        #[inline(always)]
        pub fn range(&mut self) -> range::W<'_> {
            range::W::new(self)
        }
    }
    impl AtomicSetW {
        /// Counter limit in PWM clock counts.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn range(&mut self) -> range::ASW<'_> {
            range::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// Counter limit in PWM clock counts.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn range(&mut self) -> range::ACW<'_> {
            range::ACW::new(self)
        }
    }
    impl AtomicXorW {
        /// Counter limit in PWM clock counts.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn range(&mut self) -> range::AXW<'_> {
            range::AXW::new(self)
        }
    }
}

/// Starting counter offset for channel 2, applied by `global_ctrl.set_update`.
/// Hardware access: read Plain; write Plain.
pub type CHAN2_PHASE = Reg<chan2_phase::CHAN2_PHASE_SPEC>;
/// Starting counter offset for channel 2, applied by `global_ctrl.set_update`.
/// Hardware access: read Plain; write Plain.
pub mod chan2_phase {
    pub struct CHAN2_PHASE_SPEC;
    impl crate::RegisterSpec for CHAN2_PHASE_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CHAN2_PHASE_SPEC {}
    pub type R = crate::R<CHAN2_PHASE_SPEC>;
    impl crate::Modifiable for CHAN2_PHASE_SPEC {}
    impl crate::UnsafeWritable for CHAN2_PHASE_SPEC {}
    impl crate::ResetValue for CHAN2_PHASE_SPEC {
        const RESET_VALUE: u32 = 0x0;
    }
    impl crate::TypedAtomicSet for CHAN2_PHASE_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicSet for CHAN2_PHASE_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::TypedAtomicClear for CHAN2_PHASE_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicClear for CHAN2_PHASE_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::TypedAtomicXor for CHAN2_PHASE_SPEC {
        const OFFSET: usize = 0x1000;
    }
    impl crate::UnsafeAtomicXor for CHAN2_PHASE_SPEC {
        const OFFSET: usize = 0x1000;
    }
    pub type W = crate::W<CHAN2_PHASE_SPEC>;
    pub type AtomicSetW = crate::AtomicSetW<CHAN2_PHASE_SPEC>;
    pub type AtomicClearW = crate::AtomicClearW<CHAN2_PHASE_SPEC>;
    pub type AtomicXorW = crate::AtomicXorW<CHAN2_PHASE_SPEC>;
    /// Counter preload in PWM clock counts. Keep it within the effective range.
    /// Field write behavior: Plain.
    pub mod phase {
        pub type R = crate::FieldReader<u32>;
        pub type W<'a> = crate::FieldWriter<'a, super::W, 0, 32>;
        pub type ASW<'a> = crate::AtomicSetFieldWriter<'a, super::AtomicSetW, 0, 32>;
        pub type ACW<'a> = crate::AtomicClearFieldWriter<'a, super::AtomicClearW, 0, 32>;
        pub type AXW<'a> = crate::AtomicXorFieldWriter<'a, super::AtomicXorW, 0, 32>;
    }
    impl R {
        #[inline(always)]
        pub fn phase(&self) -> phase::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffffffff)
        }
    }
    impl W {
        #[inline(always)]
        pub fn phase(&mut self) -> phase::W<'_> {
            phase::W::new(self)
        }
    }
    impl AtomicSetW {
        /// Counter preload in PWM clock counts. Keep it within the effective range.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn phase(&mut self) -> phase::ASW<'_> {
            phase::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// Counter preload in PWM clock counts. Keep it within the effective range.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn phase(&mut self) -> phase::ACW<'_> {
            phase::ACW::new(self)
        }
    }
    impl AtomicXorW {
        /// Counter preload in PWM clock counts. Keep it within the effective range.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn phase(&mut self) -> phase::AXW<'_> {
            phase::AXW::new(self)
        }
    }
}

/// Channel 2's own duty value, used when `bind=0` and `usefifo=0`. A new value is latched at the next rollover.
/// Hardware access: read Plain; write Plain.
pub type CHAN2_DUTY = Reg<chan2_duty::CHAN2_DUTY_SPEC>;
/// Channel 2's own duty value, used when `bind=0` and `usefifo=0`. A new value is latched at the next rollover.
/// Hardware access: read Plain; write Plain.
pub mod chan2_duty {
    pub struct CHAN2_DUTY_SPEC;
    impl crate::RegisterSpec for CHAN2_DUTY_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CHAN2_DUTY_SPEC {}
    pub type R = crate::R<CHAN2_DUTY_SPEC>;
    impl crate::Modifiable for CHAN2_DUTY_SPEC {}
    impl crate::UnsafeWritable for CHAN2_DUTY_SPEC {}
    impl crate::ResetValue for CHAN2_DUTY_SPEC {
        const RESET_VALUE: u32 = 0x0;
    }
    impl crate::TypedAtomicSet for CHAN2_DUTY_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicSet for CHAN2_DUTY_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::TypedAtomicClear for CHAN2_DUTY_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicClear for CHAN2_DUTY_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::TypedAtomicXor for CHAN2_DUTY_SPEC {
        const OFFSET: usize = 0x1000;
    }
    impl crate::UnsafeAtomicXor for CHAN2_DUTY_SPEC {
        const OFFSET: usize = 0x1000;
    }
    pub type W = crate::W<CHAN2_DUTY_SPEC>;
    pub type AtomicSetW = crate::AtomicSetW<CHAN2_DUTY_SPEC>;
    pub type AtomicClearW = crate::AtomicClearW<CHAN2_DUTY_SPEC>;
    pub type AtomicXorW = crate::AtomicXorW<CHAN2_DUTY_SPEC>;
    /// Meaning depends on `mode`: compare value, density numerator, pulse position or serial data word.
    /// Field write behavior: Plain.
    pub mod duty {
        pub type R = crate::FieldReader<u32>;
        pub type W<'a> = crate::FieldWriter<'a, super::W, 0, 32>;
        pub type ASW<'a> = crate::AtomicSetFieldWriter<'a, super::AtomicSetW, 0, 32>;
        pub type ACW<'a> = crate::AtomicClearFieldWriter<'a, super::AtomicClearW, 0, 32>;
        pub type AXW<'a> = crate::AtomicXorFieldWriter<'a, super::AtomicXorW, 0, 32>;
    }
    impl R {
        #[inline(always)]
        pub fn duty(&self) -> duty::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffffffff)
        }
    }
    impl W {
        #[inline(always)]
        pub fn duty(&mut self) -> duty::W<'_> {
            duty::W::new(self)
        }
    }
    impl AtomicSetW {
        /// Meaning depends on `mode`: compare value, density numerator, pulse position or serial data word.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn duty(&mut self) -> duty::ASW<'_> {
            duty::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// Meaning depends on `mode`: compare value, density numerator, pulse position or serial data word.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn duty(&mut self) -> duty::ACW<'_> {
            duty::ACW::new(self)
        }
    }
    impl AtomicXorW {
        /// Meaning depends on `mode`: compare value, density numerator, pulse position or serial data word.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn duty(&mut self) -> duty::AXW<'_> {
            duty::AXW::new(self)
        }
    }
}

/// Waveform setup for channel 3. Changes take effect after a `global_ctrl.set_update` write.
/// Hardware access: read Plain; write Plain.
pub type CHAN3_CTRL = Reg<chan3_ctrl::CHAN3_CTRL_SPEC>;
/// Waveform setup for channel 3. Changes take effect after a `global_ctrl.set_update` write.
/// Hardware access: read Plain; write Plain.
pub mod chan3_ctrl {
    pub struct CHAN3_CTRL_SPEC;
    impl crate::RegisterSpec for CHAN3_CTRL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CHAN3_CTRL_SPEC {}
    pub type R = crate::R<CHAN3_CTRL_SPEC>;
    impl crate::Modifiable for CHAN3_CTRL_SPEC {}
    impl crate::UnsafeWritable for CHAN3_CTRL_SPEC {}
    impl crate::TypedAtomicSet for CHAN3_CTRL_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicSet for CHAN3_CTRL_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::TypedAtomicClear for CHAN3_CTRL_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicClear for CHAN3_CTRL_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::TypedAtomicXor for CHAN3_CTRL_SPEC {
        const OFFSET: usize = 0x1000;
    }
    impl crate::UnsafeAtomicXor for CHAN3_CTRL_SPEC {
        const OFFSET: usize = 0x1000;
    }
    pub type W = crate::W<CHAN3_CTRL_SPEC>;
    pub type AtomicSetW = crate::AtomicSetW<CHAN3_CTRL_SPEC>;
    pub type AtomicClearW = crate::AtomicClearW<CHAN3_CTRL_SPEC>;
    pub type AtomicXorW = crate::AtomicXorW<CHAN3_CTRL_SPEC>;
    /// Waveform mode. 0: output 0 (before inversion). 1: trailing-edge PWM, counter counts up. 2: centre-aligned pulse, counter counts down then up. 3: pulse density, high for DUTY/RANGE of the steps; each step adds DUTY to an accumulator, and when the sum reaches RANGE a 1 is output and RANGE is subtracted. 4: serial output starting at bit 31. 5: one high slot when COUNT equals DUTY. 6: leading-edge PWM, counter counts down. 7: serial output starting at bit 0. Modes 1, 2 and 6 output high while COUNT < DUTY. Serial modes wrap the bit index modulo 32.
    /// Field write behavior: Plain.
    pub mod mode {
        pub type R = crate::FieldReader<u32>;
        pub type W<'a> = crate::FieldWriter<'a, super::W, 0, 3>;
        pub type ASW<'a> = crate::AtomicSetFieldWriter<'a, super::AtomicSetW, 0, 3>;
        pub type ACW<'a> = crate::AtomicClearFieldWriter<'a, super::AtomicClearW, 0, 3>;
        pub type AXW<'a> = crate::AtomicXorFieldWriter<'a, super::AtomicXorW, 0, 3>;
    }
    /// 1 inverts the output after mode processing; 0 leaves it unchanged.
    /// Field write behavior: Plain.
    pub mod invert {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 3>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 3>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 3>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 3>;
    }
    /// 0 uses this channel's own range and duty registers. 1 uses `common_range`, with duty from `common_duty` or the FIFO.
    /// Field write behavior: Plain.
    pub mod bind {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 4>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 4>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 4>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 4>;
    }
    /// 0 takes duty from a register; 1 takes duty from the FIFO. Needs `bind=1`; using the FIFO without bind gives unpredictable results.
    /// Field write behavior: Plain.
    pub mod usefifo {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 5>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 5>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 5>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 5>;
    }
    /// 1 enables second-order sigma-delta noise shaping: the duty value is treated as signed 16-bit and quantized to `sdm_bitwidth`+1 bits, advancing on the channel's duty strobe. 0 bypasses shaping.
    /// Field write behavior: Plain.
    pub mod sdm {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 6>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 6>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 6>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 6>;
    }
    /// 1 adds a one-LSB dither into the sigma-delta loop to break up idle tones; 0 turns it off. Only has an effect when `sdm=1`.
    /// Field write behavior: Plain.
    pub mod dither {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 7>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 7>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 7>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 7>;
    }
    /// 1 lets this channel's rollover contribute to the shared FIFO pop (an OR across bound channels). 0 keeps its rollover from popping.
    /// Field write behavior: Plain.
    pub mod fifo_pop_mask {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 8>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 8>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 8>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 8>;
    }
    /// Sigma-delta output precision minus one: values 0-15 give 1-16 bits.
    /// Field write behavior: Plain.
    pub mod sdm_bitwidth {
        pub type R = crate::FieldReader<u32>;
        pub type W<'a> = crate::FieldWriter<'a, super::W, 12, 4>;
        pub type ASW<'a> = crate::AtomicSetFieldWriter<'a, super::AtomicSetW, 12, 4>;
        pub type ACW<'a> = crate::AtomicClearFieldWriter<'a, super::AtomicClearW, 12, 4>;
        pub type AXW<'a> = crate::AtomicXorFieldWriter<'a, super::AtomicXorW, 12, 4>;
    }
    /// Unsigned offset added to the quantized PWM code, for example to move a signed source into an unsigned range.
    /// Field write behavior: Plain.
    pub mod sdm_bias {
        pub type R = crate::FieldReader<u32>;
        pub type W<'a> = crate::FieldWriter<'a, super::W, 16, 16>;
        pub type ASW<'a> = crate::AtomicSetFieldWriter<'a, super::AtomicSetW, 16, 16>;
        pub type ACW<'a> = crate::AtomicClearFieldWriter<'a, super::AtomicClearW, 16, 16>;
        pub type AXW<'a> = crate::AtomicXorFieldWriter<'a, super::AtomicXorW, 16, 16>;
    }
    impl R {
        #[inline(always)]
        pub fn mode(&self) -> mode::R {
            crate::FieldReader::new((self.bits() >> 0) & 0x7)
        }
        #[inline(always)]
        pub fn invert(&self) -> invert::R {
            crate::BitReader::new(((self.bits() >> 3) & 1) != 0)
        }
        #[inline(always)]
        pub fn bind(&self) -> bind::R {
            crate::BitReader::new(((self.bits() >> 4) & 1) != 0)
        }
        #[inline(always)]
        pub fn usefifo(&self) -> usefifo::R {
            crate::BitReader::new(((self.bits() >> 5) & 1) != 0)
        }
        #[inline(always)]
        pub fn sdm(&self) -> sdm::R {
            crate::BitReader::new(((self.bits() >> 6) & 1) != 0)
        }
        #[inline(always)]
        pub fn dither(&self) -> dither::R {
            crate::BitReader::new(((self.bits() >> 7) & 1) != 0)
        }
        #[inline(always)]
        pub fn fifo_pop_mask(&self) -> fifo_pop_mask::R {
            crate::BitReader::new(((self.bits() >> 8) & 1) != 0)
        }
        #[inline(always)]
        pub fn sdm_bitwidth(&self) -> sdm_bitwidth::R {
            crate::FieldReader::new((self.bits() >> 12) & 0xf)
        }
        #[inline(always)]
        pub fn sdm_bias(&self) -> sdm_bias::R {
            crate::FieldReader::new((self.bits() >> 16) & 0xffff)
        }
    }
    impl W {
        #[inline(always)]
        pub fn mode(&mut self) -> mode::W<'_> {
            mode::W::new(self)
        }
        #[inline(always)]
        pub fn invert(&mut self) -> invert::W<'_> {
            invert::W::new(self)
        }
        #[inline(always)]
        pub fn bind(&mut self) -> bind::W<'_> {
            bind::W::new(self)
        }
        #[inline(always)]
        pub fn usefifo(&mut self) -> usefifo::W<'_> {
            usefifo::W::new(self)
        }
        #[inline(always)]
        pub fn sdm(&mut self) -> sdm::W<'_> {
            sdm::W::new(self)
        }
        #[inline(always)]
        pub fn dither(&mut self) -> dither::W<'_> {
            dither::W::new(self)
        }
        #[inline(always)]
        pub fn fifo_pop_mask(&mut self) -> fifo_pop_mask::W<'_> {
            fifo_pop_mask::W::new(self)
        }
        #[inline(always)]
        pub fn sdm_bitwidth(&mut self) -> sdm_bitwidth::W<'_> {
            sdm_bitwidth::W::new(self)
        }
        #[inline(always)]
        pub fn sdm_bias(&mut self) -> sdm_bias::W<'_> {
            sdm_bias::W::new(self)
        }
    }
    impl AtomicSetW {
        /// Waveform mode. 0: output 0 (before inversion). 1: trailing-edge PWM, counter counts up. 2: centre-aligned pulse, counter counts down then up. 3: pulse density, high for DUTY/RANGE of the steps; each step adds DUTY to an accumulator, and when the sum reaches RANGE a 1 is output and RANGE is subtracted. 4: serial output starting at bit 31. 5: one high slot when COUNT equals DUTY. 6: leading-edge PWM, counter counts down. 7: serial output starting at bit 0. Modes 1, 2 and 6 output high while COUNT < DUTY. Serial modes wrap the bit index modulo 32.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn mode(&mut self) -> mode::ASW<'_> {
            mode::ASW::new(self)
        }
        /// 1 inverts the output after mode processing; 0 leaves it unchanged.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn invert(&mut self) -> invert::ASW<'_> {
            invert::ASW::new(self)
        }
        /// 0 uses this channel's own range and duty registers. 1 uses `common_range`, with duty from `common_duty` or the FIFO.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn bind(&mut self) -> bind::ASW<'_> {
            bind::ASW::new(self)
        }
        /// 0 takes duty from a register; 1 takes duty from the FIFO. Needs `bind=1`; using the FIFO without bind gives unpredictable results.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn usefifo(&mut self) -> usefifo::ASW<'_> {
            usefifo::ASW::new(self)
        }
        /// 1 enables second-order sigma-delta noise shaping: the duty value is treated as signed 16-bit and quantized to `sdm_bitwidth`+1 bits, advancing on the channel's duty strobe. 0 bypasses shaping.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn sdm(&mut self) -> sdm::ASW<'_> {
            sdm::ASW::new(self)
        }
        /// 1 adds a one-LSB dither into the sigma-delta loop to break up idle tones; 0 turns it off. Only has an effect when `sdm=1`.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn dither(&mut self) -> dither::ASW<'_> {
            dither::ASW::new(self)
        }
        /// 1 lets this channel's rollover contribute to the shared FIFO pop (an OR across bound channels). 0 keeps its rollover from popping.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn fifo_pop_mask(&mut self) -> fifo_pop_mask::ASW<'_> {
            fifo_pop_mask::ASW::new(self)
        }
        /// Sigma-delta output precision minus one: values 0-15 give 1-16 bits.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn sdm_bitwidth(&mut self) -> sdm_bitwidth::ASW<'_> {
            sdm_bitwidth::ASW::new(self)
        }
        /// Unsigned offset added to the quantized PWM code, for example to move a signed source into an unsigned range.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn sdm_bias(&mut self) -> sdm_bias::ASW<'_> {
            sdm_bias::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// Waveform mode. 0: output 0 (before inversion). 1: trailing-edge PWM, counter counts up. 2: centre-aligned pulse, counter counts down then up. 3: pulse density, high for DUTY/RANGE of the steps; each step adds DUTY to an accumulator, and when the sum reaches RANGE a 1 is output and RANGE is subtracted. 4: serial output starting at bit 31. 5: one high slot when COUNT equals DUTY. 6: leading-edge PWM, counter counts down. 7: serial output starting at bit 0. Modes 1, 2 and 6 output high while COUNT < DUTY. Serial modes wrap the bit index modulo 32.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn mode(&mut self) -> mode::ACW<'_> {
            mode::ACW::new(self)
        }
        /// 1 inverts the output after mode processing; 0 leaves it unchanged.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn invert(&mut self) -> invert::ACW<'_> {
            invert::ACW::new(self)
        }
        /// 0 uses this channel's own range and duty registers. 1 uses `common_range`, with duty from `common_duty` or the FIFO.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn bind(&mut self) -> bind::ACW<'_> {
            bind::ACW::new(self)
        }
        /// 0 takes duty from a register; 1 takes duty from the FIFO. Needs `bind=1`; using the FIFO without bind gives unpredictable results.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn usefifo(&mut self) -> usefifo::ACW<'_> {
            usefifo::ACW::new(self)
        }
        /// 1 enables second-order sigma-delta noise shaping: the duty value is treated as signed 16-bit and quantized to `sdm_bitwidth`+1 bits, advancing on the channel's duty strobe. 0 bypasses shaping.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn sdm(&mut self) -> sdm::ACW<'_> {
            sdm::ACW::new(self)
        }
        /// 1 adds a one-LSB dither into the sigma-delta loop to break up idle tones; 0 turns it off. Only has an effect when `sdm=1`.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn dither(&mut self) -> dither::ACW<'_> {
            dither::ACW::new(self)
        }
        /// 1 lets this channel's rollover contribute to the shared FIFO pop (an OR across bound channels). 0 keeps its rollover from popping.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn fifo_pop_mask(&mut self) -> fifo_pop_mask::ACW<'_> {
            fifo_pop_mask::ACW::new(self)
        }
        /// Sigma-delta output precision minus one: values 0-15 give 1-16 bits.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn sdm_bitwidth(&mut self) -> sdm_bitwidth::ACW<'_> {
            sdm_bitwidth::ACW::new(self)
        }
        /// Unsigned offset added to the quantized PWM code, for example to move a signed source into an unsigned range.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn sdm_bias(&mut self) -> sdm_bias::ACW<'_> {
            sdm_bias::ACW::new(self)
        }
    }
    impl AtomicXorW {
        /// Waveform mode. 0: output 0 (before inversion). 1: trailing-edge PWM, counter counts up. 2: centre-aligned pulse, counter counts down then up. 3: pulse density, high for DUTY/RANGE of the steps; each step adds DUTY to an accumulator, and when the sum reaches RANGE a 1 is output and RANGE is subtracted. 4: serial output starting at bit 31. 5: one high slot when COUNT equals DUTY. 6: leading-edge PWM, counter counts down. 7: serial output starting at bit 0. Modes 1, 2 and 6 output high while COUNT < DUTY. Serial modes wrap the bit index modulo 32.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn mode(&mut self) -> mode::AXW<'_> {
            mode::AXW::new(self)
        }
        /// 1 inverts the output after mode processing; 0 leaves it unchanged.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn invert(&mut self) -> invert::AXW<'_> {
            invert::AXW::new(self)
        }
        /// 0 uses this channel's own range and duty registers. 1 uses `common_range`, with duty from `common_duty` or the FIFO.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn bind(&mut self) -> bind::AXW<'_> {
            bind::AXW::new(self)
        }
        /// 0 takes duty from a register; 1 takes duty from the FIFO. Needs `bind=1`; using the FIFO without bind gives unpredictable results.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn usefifo(&mut self) -> usefifo::AXW<'_> {
            usefifo::AXW::new(self)
        }
        /// 1 enables second-order sigma-delta noise shaping: the duty value is treated as signed 16-bit and quantized to `sdm_bitwidth`+1 bits, advancing on the channel's duty strobe. 0 bypasses shaping.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn sdm(&mut self) -> sdm::AXW<'_> {
            sdm::AXW::new(self)
        }
        /// 1 adds a one-LSB dither into the sigma-delta loop to break up idle tones; 0 turns it off. Only has an effect when `sdm=1`.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn dither(&mut self) -> dither::AXW<'_> {
            dither::AXW::new(self)
        }
        /// 1 lets this channel's rollover contribute to the shared FIFO pop (an OR across bound channels). 0 keeps its rollover from popping.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn fifo_pop_mask(&mut self) -> fifo_pop_mask::AXW<'_> {
            fifo_pop_mask::AXW::new(self)
        }
        /// Sigma-delta output precision minus one: values 0-15 give 1-16 bits.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn sdm_bitwidth(&mut self) -> sdm_bitwidth::AXW<'_> {
            sdm_bitwidth::AXW::new(self)
        }
        /// Unsigned offset added to the quantized PWM code, for example to move a signed source into an unsigned range.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn sdm_bias(&mut self) -> sdm_bias::AXW<'_> {
            sdm_bias::AXW::new(self)
        }
    }
}

/// Channel 3's own counter limit, used when `bind=0`. A write takes effect at the next rollover.
/// Hardware access: read Plain; write Plain.
pub type CHAN3_RANGE = Reg<chan3_range::CHAN3_RANGE_SPEC>;
/// Channel 3's own counter limit, used when `bind=0`. A write takes effect at the next rollover.
/// Hardware access: read Plain; write Plain.
pub mod chan3_range {
    pub struct CHAN3_RANGE_SPEC;
    impl crate::RegisterSpec for CHAN3_RANGE_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CHAN3_RANGE_SPEC {}
    pub type R = crate::R<CHAN3_RANGE_SPEC>;
    impl crate::Modifiable for CHAN3_RANGE_SPEC {}
    impl crate::UnsafeWritable for CHAN3_RANGE_SPEC {}
    impl crate::ResetValue for CHAN3_RANGE_SPEC {
        const RESET_VALUE: u32 = 0x0;
    }
    impl crate::TypedAtomicSet for CHAN3_RANGE_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicSet for CHAN3_RANGE_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::TypedAtomicClear for CHAN3_RANGE_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicClear for CHAN3_RANGE_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::TypedAtomicXor for CHAN3_RANGE_SPEC {
        const OFFSET: usize = 0x1000;
    }
    impl crate::UnsafeAtomicXor for CHAN3_RANGE_SPEC {
        const OFFSET: usize = 0x1000;
    }
    pub type W = crate::W<CHAN3_RANGE_SPEC>;
    pub type AtomicSetW = crate::AtomicSetW<CHAN3_RANGE_SPEC>;
    pub type AtomicClearW = crate::AtomicClearW<CHAN3_RANGE_SPEC>;
    pub type AtomicXorW = crate::AtomicXorW<CHAN3_RANGE_SPEC>;
    /// Counter limit in PWM clock counts.
    /// Field write behavior: Plain.
    pub mod range {
        pub type R = crate::FieldReader<u32>;
        pub type W<'a> = crate::FieldWriter<'a, super::W, 0, 32>;
        pub type ASW<'a> = crate::AtomicSetFieldWriter<'a, super::AtomicSetW, 0, 32>;
        pub type ACW<'a> = crate::AtomicClearFieldWriter<'a, super::AtomicClearW, 0, 32>;
        pub type AXW<'a> = crate::AtomicXorFieldWriter<'a, super::AtomicXorW, 0, 32>;
    }
    impl R {
        #[inline(always)]
        pub fn range(&self) -> range::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffffffff)
        }
    }
    impl W {
        #[inline(always)]
        pub fn range(&mut self) -> range::W<'_> {
            range::W::new(self)
        }
    }
    impl AtomicSetW {
        /// Counter limit in PWM clock counts.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn range(&mut self) -> range::ASW<'_> {
            range::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// Counter limit in PWM clock counts.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn range(&mut self) -> range::ACW<'_> {
            range::ACW::new(self)
        }
    }
    impl AtomicXorW {
        /// Counter limit in PWM clock counts.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn range(&mut self) -> range::AXW<'_> {
            range::AXW::new(self)
        }
    }
}

/// Starting counter offset for channel 3, applied by `global_ctrl.set_update`.
/// Hardware access: read Plain; write Plain.
pub type CHAN3_PHASE = Reg<chan3_phase::CHAN3_PHASE_SPEC>;
/// Starting counter offset for channel 3, applied by `global_ctrl.set_update`.
/// Hardware access: read Plain; write Plain.
pub mod chan3_phase {
    pub struct CHAN3_PHASE_SPEC;
    impl crate::RegisterSpec for CHAN3_PHASE_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CHAN3_PHASE_SPEC {}
    pub type R = crate::R<CHAN3_PHASE_SPEC>;
    impl crate::Modifiable for CHAN3_PHASE_SPEC {}
    impl crate::UnsafeWritable for CHAN3_PHASE_SPEC {}
    impl crate::ResetValue for CHAN3_PHASE_SPEC {
        const RESET_VALUE: u32 = 0x0;
    }
    impl crate::TypedAtomicSet for CHAN3_PHASE_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicSet for CHAN3_PHASE_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::TypedAtomicClear for CHAN3_PHASE_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicClear for CHAN3_PHASE_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::TypedAtomicXor for CHAN3_PHASE_SPEC {
        const OFFSET: usize = 0x1000;
    }
    impl crate::UnsafeAtomicXor for CHAN3_PHASE_SPEC {
        const OFFSET: usize = 0x1000;
    }
    pub type W = crate::W<CHAN3_PHASE_SPEC>;
    pub type AtomicSetW = crate::AtomicSetW<CHAN3_PHASE_SPEC>;
    pub type AtomicClearW = crate::AtomicClearW<CHAN3_PHASE_SPEC>;
    pub type AtomicXorW = crate::AtomicXorW<CHAN3_PHASE_SPEC>;
    /// Counter preload in PWM clock counts. Keep it within the effective range.
    /// Field write behavior: Plain.
    pub mod phase {
        pub type R = crate::FieldReader<u32>;
        pub type W<'a> = crate::FieldWriter<'a, super::W, 0, 32>;
        pub type ASW<'a> = crate::AtomicSetFieldWriter<'a, super::AtomicSetW, 0, 32>;
        pub type ACW<'a> = crate::AtomicClearFieldWriter<'a, super::AtomicClearW, 0, 32>;
        pub type AXW<'a> = crate::AtomicXorFieldWriter<'a, super::AtomicXorW, 0, 32>;
    }
    impl R {
        #[inline(always)]
        pub fn phase(&self) -> phase::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffffffff)
        }
    }
    impl W {
        #[inline(always)]
        pub fn phase(&mut self) -> phase::W<'_> {
            phase::W::new(self)
        }
    }
    impl AtomicSetW {
        /// Counter preload in PWM clock counts. Keep it within the effective range.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn phase(&mut self) -> phase::ASW<'_> {
            phase::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// Counter preload in PWM clock counts. Keep it within the effective range.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn phase(&mut self) -> phase::ACW<'_> {
            phase::ACW::new(self)
        }
    }
    impl AtomicXorW {
        /// Counter preload in PWM clock counts. Keep it within the effective range.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn phase(&mut self) -> phase::AXW<'_> {
            phase::AXW::new(self)
        }
    }
}

/// Channel 3's own duty value, used when `bind=0` and `usefifo=0`. A new value is latched at the next rollover.
/// Hardware access: read Plain; write Plain.
pub type CHAN3_DUTY = Reg<chan3_duty::CHAN3_DUTY_SPEC>;
/// Channel 3's own duty value, used when `bind=0` and `usefifo=0`. A new value is latched at the next rollover.
/// Hardware access: read Plain; write Plain.
pub mod chan3_duty {
    pub struct CHAN3_DUTY_SPEC;
    impl crate::RegisterSpec for CHAN3_DUTY_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CHAN3_DUTY_SPEC {}
    pub type R = crate::R<CHAN3_DUTY_SPEC>;
    impl crate::Modifiable for CHAN3_DUTY_SPEC {}
    impl crate::UnsafeWritable for CHAN3_DUTY_SPEC {}
    impl crate::ResetValue for CHAN3_DUTY_SPEC {
        const RESET_VALUE: u32 = 0x0;
    }
    impl crate::TypedAtomicSet for CHAN3_DUTY_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicSet for CHAN3_DUTY_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::TypedAtomicClear for CHAN3_DUTY_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicClear for CHAN3_DUTY_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::TypedAtomicXor for CHAN3_DUTY_SPEC {
        const OFFSET: usize = 0x1000;
    }
    impl crate::UnsafeAtomicXor for CHAN3_DUTY_SPEC {
        const OFFSET: usize = 0x1000;
    }
    pub type W = crate::W<CHAN3_DUTY_SPEC>;
    pub type AtomicSetW = crate::AtomicSetW<CHAN3_DUTY_SPEC>;
    pub type AtomicClearW = crate::AtomicClearW<CHAN3_DUTY_SPEC>;
    pub type AtomicXorW = crate::AtomicXorW<CHAN3_DUTY_SPEC>;
    /// Meaning depends on `mode`: compare value, density numerator, pulse position or serial data word.
    /// Field write behavior: Plain.
    pub mod duty {
        pub type R = crate::FieldReader<u32>;
        pub type W<'a> = crate::FieldWriter<'a, super::W, 0, 32>;
        pub type ASW<'a> = crate::AtomicSetFieldWriter<'a, super::AtomicSetW, 0, 32>;
        pub type ACW<'a> = crate::AtomicClearFieldWriter<'a, super::AtomicClearW, 0, 32>;
        pub type AXW<'a> = crate::AtomicXorFieldWriter<'a, super::AtomicXorW, 0, 32>;
    }
    impl R {
        #[inline(always)]
        pub fn duty(&self) -> duty::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffffffff)
        }
    }
    impl W {
        #[inline(always)]
        pub fn duty(&mut self) -> duty::W<'_> {
            duty::W::new(self)
        }
    }
    impl AtomicSetW {
        /// Meaning depends on `mode`: compare value, density numerator, pulse position or serial data word.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn duty(&mut self) -> duty::ASW<'_> {
            duty::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// Meaning depends on `mode`: compare value, density numerator, pulse position or serial data word.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn duty(&mut self) -> duty::ACW<'_> {
            duty::ACW::new(self)
        }
    }
    impl AtomicXorW {
        /// Meaning depends on `mode`: compare value, density numerator, pulse position or serial data word.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn duty(&mut self) -> duty::AXW<'_> {
            duty::AXW::new(self)
        }
    }
}

/// Raw interrupt status, before the enable mask.
/// Hardware access: read Plain; write WriteOneToClear.
/// Write-one-to-clear register; general write and modify APIs are not generated.
pub type INTR = Reg<intr::INTR_SPEC>;
/// Raw interrupt status, before the enable mask.
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
    impl crate::UnsafeWritable for INTR_SPEC {}
    impl crate::ResetValue for INTR_SPEC {
        const RESET_VALUE: u32 = 0x0;
    }
    pub type ClearW = crate::ClearW<INTR_SPEC>;
    /// Raw FIFO underflow (read while empty) flag. Clear it by writing 1; writing 0 leaves it pending.
    /// Field write behavior: WriteOneToClear.
    pub mod fifo_underflow {
        pub type R = crate::BitReader;
        pub type CW<'a> = crate::ClearBitWriter<'a, super::ClearW, 0>;
    }
    /// Raw FIFO overflow (write while full) flag. Clear it by writing 1; writing 0 leaves it pending.
    /// Field write behavior: WriteOneToClear.
    pub mod fifo_overflow {
        pub type R = crate::BitReader;
        pub type CW<'a> = crate::ClearBitWriter<'a, super::ClearW, 1>;
    }
    /// Raw FIFO empty state. Follows the source; it is not cleared by writing.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod fifo_empty {
        pub type R = crate::BitReader;
    }
    /// Raw FIFO full state. Follows the source; it is not cleared by writing.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod fifo_full {
        pub type R = crate::BitReader;
    }
    /// Raw DMA request active state. Follows the source; it is not cleared by writing.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod dreq_active {
        pub type R = crate::BitReader;
    }
    /// Raw channel 0 counter reload flag. Clear it by writing 1; writing 0 leaves it pending.
    /// Field write behavior: WriteOneToClear.
    pub mod chan0_reload {
        pub type R = crate::BitReader;
        pub type CW<'a> = crate::ClearBitWriter<'a, super::ClearW, 5>;
    }
    /// Raw channel 1 counter reload flag. Clear it by writing 1; writing 0 leaves it pending.
    /// Field write behavior: WriteOneToClear.
    pub mod chan1_reload {
        pub type R = crate::BitReader;
        pub type CW<'a> = crate::ClearBitWriter<'a, super::ClearW, 6>;
    }
    /// Raw channel 2 counter reload flag. Clear it by writing 1; writing 0 leaves it pending.
    /// Field write behavior: WriteOneToClear.
    pub mod chan2_reload {
        pub type R = crate::BitReader;
        pub type CW<'a> = crate::ClearBitWriter<'a, super::ClearW, 7>;
    }
    /// Raw channel 3 counter reload flag. Clear it by writing 1; writing 0 leaves it pending.
    /// Field write behavior: WriteOneToClear.
    pub mod chan3_reload {
        pub type R = crate::BitReader;
        pub type CW<'a> = crate::ClearBitWriter<'a, super::ClearW, 8>;
    }
    impl R {
        #[inline(always)]
        pub fn fifo_underflow(&self) -> fifo_underflow::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn fifo_overflow(&self) -> fifo_overflow::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
        #[inline(always)]
        pub fn fifo_empty(&self) -> fifo_empty::R {
            crate::BitReader::new(((self.bits() >> 2) & 1) != 0)
        }
        #[inline(always)]
        pub fn fifo_full(&self) -> fifo_full::R {
            crate::BitReader::new(((self.bits() >> 3) & 1) != 0)
        }
        #[inline(always)]
        pub fn dreq_active(&self) -> dreq_active::R {
            crate::BitReader::new(((self.bits() >> 4) & 1) != 0)
        }
        #[inline(always)]
        pub fn chan0_reload(&self) -> chan0_reload::R {
            crate::BitReader::new(((self.bits() >> 5) & 1) != 0)
        }
        #[inline(always)]
        pub fn chan1_reload(&self) -> chan1_reload::R {
            crate::BitReader::new(((self.bits() >> 6) & 1) != 0)
        }
        #[inline(always)]
        pub fn chan2_reload(&self) -> chan2_reload::R {
            crate::BitReader::new(((self.bits() >> 7) & 1) != 0)
        }
        #[inline(always)]
        pub fn chan3_reload(&self) -> chan3_reload::R {
            crate::BitReader::new(((self.bits() >> 8) & 1) != 0)
        }
    }
    impl ClearW {
        #[inline(always)]
        pub fn fifo_underflow(&mut self) -> fifo_underflow::CW<'_> {
            fifo_underflow::CW::new(self)
        }
        #[inline(always)]
        pub fn fifo_overflow(&mut self) -> fifo_overflow::CW<'_> {
            fifo_overflow::CW::new(self)
        }
        #[inline(always)]
        pub fn chan0_reload(&mut self) -> chan0_reload::CW<'_> {
            chan0_reload::CW::new(self)
        }
        #[inline(always)]
        pub fn chan1_reload(&mut self) -> chan1_reload::CW<'_> {
            chan1_reload::CW::new(self)
        }
        #[inline(always)]
        pub fn chan2_reload(&mut self) -> chan2_reload::CW<'_> {
            chan2_reload::CW::new(self)
        }
        #[inline(always)]
        pub fn chan3_reload(&mut self) -> chan3_reload::CW<'_> {
            chan3_reload::CW::new(self)
        }
    }
}

/// Interrupt enable mask, one bit per source: 0 masks the source, 1 passes it to the IRQ output.
/// Hardware access: read Plain; write Plain.
pub type INTE = Reg<inte::INTE_SPEC>;
/// Interrupt enable mask, one bit per source: 0 masks the source, 1 passes it to the IRQ output.
/// Hardware access: read Plain; write Plain.
pub mod inte {
    pub struct INTE_SPEC;
    impl crate::RegisterSpec for INTE_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for INTE_SPEC {}
    pub type R = crate::R<INTE_SPEC>;
    impl crate::Modifiable for INTE_SPEC {}
    impl crate::UnsafeWritable for INTE_SPEC {}
    impl crate::ResetValue for INTE_SPEC {
        const RESET_VALUE: u32 = 0x0;
    }
    impl crate::TypedAtomicSet for INTE_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicSet for INTE_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::TypedAtomicClear for INTE_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicClear for INTE_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::TypedAtomicXor for INTE_SPEC {
        const OFFSET: usize = 0x1000;
    }
    impl crate::UnsafeAtomicXor for INTE_SPEC {
        const OFFSET: usize = 0x1000;
    }
    pub type W = crate::W<INTE_SPEC>;
    pub type AtomicSetW = crate::AtomicSetW<INTE_SPEC>;
    pub type AtomicClearW = crate::AtomicClearW<INTE_SPEC>;
    pub type AtomicXorW = crate::AtomicXorW<INTE_SPEC>;
    /// Enable for the FIFO underflow (read while empty) interrupt: 0 masked, 1 routed to the IRQ output.
    /// Field write behavior: Plain.
    pub mod fifo_underflow {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 0>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 0>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 0>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 0>;
    }
    /// Enable for the FIFO overflow (write while full) interrupt: 0 masked, 1 routed to the IRQ output.
    /// Field write behavior: Plain.
    pub mod fifo_overflow {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 1>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 1>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 1>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 1>;
    }
    /// Enable for the FIFO empty interrupt: 0 masked, 1 routed to the IRQ output.
    /// Field write behavior: Plain.
    pub mod fifo_empty {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 2>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 2>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 2>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 2>;
    }
    /// Enable for the FIFO full interrupt: 0 masked, 1 routed to the IRQ output.
    /// Field write behavior: Plain.
    pub mod fifo_full {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 3>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 3>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 3>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 3>;
    }
    /// Enable for the DMA request active interrupt: 0 masked, 1 routed to the IRQ output.
    /// Field write behavior: Plain.
    pub mod dreq_active {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 4>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 4>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 4>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 4>;
    }
    /// Enable for the channel 0 counter reload interrupt: 0 masked, 1 routed to the IRQ output.
    /// Field write behavior: Plain.
    pub mod chan0_reload {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 5>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 5>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 5>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 5>;
    }
    /// Enable for the channel 1 counter reload interrupt: 0 masked, 1 routed to the IRQ output.
    /// Field write behavior: Plain.
    pub mod chan1_reload {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 6>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 6>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 6>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 6>;
    }
    /// Enable for the channel 2 counter reload interrupt: 0 masked, 1 routed to the IRQ output.
    /// Field write behavior: Plain.
    pub mod chan2_reload {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 7>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 7>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 7>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 7>;
    }
    /// Enable for the channel 3 counter reload interrupt: 0 masked, 1 routed to the IRQ output.
    /// Field write behavior: Plain.
    pub mod chan3_reload {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 8>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 8>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 8>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 8>;
    }
    impl R {
        #[inline(always)]
        pub fn fifo_underflow(&self) -> fifo_underflow::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn fifo_overflow(&self) -> fifo_overflow::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
        #[inline(always)]
        pub fn fifo_empty(&self) -> fifo_empty::R {
            crate::BitReader::new(((self.bits() >> 2) & 1) != 0)
        }
        #[inline(always)]
        pub fn fifo_full(&self) -> fifo_full::R {
            crate::BitReader::new(((self.bits() >> 3) & 1) != 0)
        }
        #[inline(always)]
        pub fn dreq_active(&self) -> dreq_active::R {
            crate::BitReader::new(((self.bits() >> 4) & 1) != 0)
        }
        #[inline(always)]
        pub fn chan0_reload(&self) -> chan0_reload::R {
            crate::BitReader::new(((self.bits() >> 5) & 1) != 0)
        }
        #[inline(always)]
        pub fn chan1_reload(&self) -> chan1_reload::R {
            crate::BitReader::new(((self.bits() >> 6) & 1) != 0)
        }
        #[inline(always)]
        pub fn chan2_reload(&self) -> chan2_reload::R {
            crate::BitReader::new(((self.bits() >> 7) & 1) != 0)
        }
        #[inline(always)]
        pub fn chan3_reload(&self) -> chan3_reload::R {
            crate::BitReader::new(((self.bits() >> 8) & 1) != 0)
        }
    }
    impl W {
        #[inline(always)]
        pub fn fifo_underflow(&mut self) -> fifo_underflow::W<'_> {
            fifo_underflow::W::new(self)
        }
        #[inline(always)]
        pub fn fifo_overflow(&mut self) -> fifo_overflow::W<'_> {
            fifo_overflow::W::new(self)
        }
        #[inline(always)]
        pub fn fifo_empty(&mut self) -> fifo_empty::W<'_> {
            fifo_empty::W::new(self)
        }
        #[inline(always)]
        pub fn fifo_full(&mut self) -> fifo_full::W<'_> {
            fifo_full::W::new(self)
        }
        #[inline(always)]
        pub fn dreq_active(&mut self) -> dreq_active::W<'_> {
            dreq_active::W::new(self)
        }
        #[inline(always)]
        pub fn chan0_reload(&mut self) -> chan0_reload::W<'_> {
            chan0_reload::W::new(self)
        }
        #[inline(always)]
        pub fn chan1_reload(&mut self) -> chan1_reload::W<'_> {
            chan1_reload::W::new(self)
        }
        #[inline(always)]
        pub fn chan2_reload(&mut self) -> chan2_reload::W<'_> {
            chan2_reload::W::new(self)
        }
        #[inline(always)]
        pub fn chan3_reload(&mut self) -> chan3_reload::W<'_> {
            chan3_reload::W::new(self)
        }
    }
    impl AtomicSetW {
        /// Enable for the FIFO underflow (read while empty) interrupt: 0 masked, 1 routed to the IRQ output.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn fifo_underflow(&mut self) -> fifo_underflow::ASW<'_> {
            fifo_underflow::ASW::new(self)
        }
        /// Enable for the FIFO overflow (write while full) interrupt: 0 masked, 1 routed to the IRQ output.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn fifo_overflow(&mut self) -> fifo_overflow::ASW<'_> {
            fifo_overflow::ASW::new(self)
        }
        /// Enable for the FIFO empty interrupt: 0 masked, 1 routed to the IRQ output.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn fifo_empty(&mut self) -> fifo_empty::ASW<'_> {
            fifo_empty::ASW::new(self)
        }
        /// Enable for the FIFO full interrupt: 0 masked, 1 routed to the IRQ output.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn fifo_full(&mut self) -> fifo_full::ASW<'_> {
            fifo_full::ASW::new(self)
        }
        /// Enable for the DMA request active interrupt: 0 masked, 1 routed to the IRQ output.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn dreq_active(&mut self) -> dreq_active::ASW<'_> {
            dreq_active::ASW::new(self)
        }
        /// Enable for the channel 0 counter reload interrupt: 0 masked, 1 routed to the IRQ output.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn chan0_reload(&mut self) -> chan0_reload::ASW<'_> {
            chan0_reload::ASW::new(self)
        }
        /// Enable for the channel 1 counter reload interrupt: 0 masked, 1 routed to the IRQ output.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn chan1_reload(&mut self) -> chan1_reload::ASW<'_> {
            chan1_reload::ASW::new(self)
        }
        /// Enable for the channel 2 counter reload interrupt: 0 masked, 1 routed to the IRQ output.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn chan2_reload(&mut self) -> chan2_reload::ASW<'_> {
            chan2_reload::ASW::new(self)
        }
        /// Enable for the channel 3 counter reload interrupt: 0 masked, 1 routed to the IRQ output.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn chan3_reload(&mut self) -> chan3_reload::ASW<'_> {
            chan3_reload::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// Enable for the FIFO underflow (read while empty) interrupt: 0 masked, 1 routed to the IRQ output.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn fifo_underflow(&mut self) -> fifo_underflow::ACW<'_> {
            fifo_underflow::ACW::new(self)
        }
        /// Enable for the FIFO overflow (write while full) interrupt: 0 masked, 1 routed to the IRQ output.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn fifo_overflow(&mut self) -> fifo_overflow::ACW<'_> {
            fifo_overflow::ACW::new(self)
        }
        /// Enable for the FIFO empty interrupt: 0 masked, 1 routed to the IRQ output.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn fifo_empty(&mut self) -> fifo_empty::ACW<'_> {
            fifo_empty::ACW::new(self)
        }
        /// Enable for the FIFO full interrupt: 0 masked, 1 routed to the IRQ output.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn fifo_full(&mut self) -> fifo_full::ACW<'_> {
            fifo_full::ACW::new(self)
        }
        /// Enable for the DMA request active interrupt: 0 masked, 1 routed to the IRQ output.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn dreq_active(&mut self) -> dreq_active::ACW<'_> {
            dreq_active::ACW::new(self)
        }
        /// Enable for the channel 0 counter reload interrupt: 0 masked, 1 routed to the IRQ output.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn chan0_reload(&mut self) -> chan0_reload::ACW<'_> {
            chan0_reload::ACW::new(self)
        }
        /// Enable for the channel 1 counter reload interrupt: 0 masked, 1 routed to the IRQ output.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn chan1_reload(&mut self) -> chan1_reload::ACW<'_> {
            chan1_reload::ACW::new(self)
        }
        /// Enable for the channel 2 counter reload interrupt: 0 masked, 1 routed to the IRQ output.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn chan2_reload(&mut self) -> chan2_reload::ACW<'_> {
            chan2_reload::ACW::new(self)
        }
        /// Enable for the channel 3 counter reload interrupt: 0 masked, 1 routed to the IRQ output.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn chan3_reload(&mut self) -> chan3_reload::ACW<'_> {
            chan3_reload::ACW::new(self)
        }
    }
    impl AtomicXorW {
        /// Enable for the FIFO underflow (read while empty) interrupt: 0 masked, 1 routed to the IRQ output.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn fifo_underflow(&mut self) -> fifo_underflow::AXW<'_> {
            fifo_underflow::AXW::new(self)
        }
        /// Enable for the FIFO overflow (write while full) interrupt: 0 masked, 1 routed to the IRQ output.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn fifo_overflow(&mut self) -> fifo_overflow::AXW<'_> {
            fifo_overflow::AXW::new(self)
        }
        /// Enable for the FIFO empty interrupt: 0 masked, 1 routed to the IRQ output.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn fifo_empty(&mut self) -> fifo_empty::AXW<'_> {
            fifo_empty::AXW::new(self)
        }
        /// Enable for the FIFO full interrupt: 0 masked, 1 routed to the IRQ output.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn fifo_full(&mut self) -> fifo_full::AXW<'_> {
            fifo_full::AXW::new(self)
        }
        /// Enable for the DMA request active interrupt: 0 masked, 1 routed to the IRQ output.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn dreq_active(&mut self) -> dreq_active::AXW<'_> {
            dreq_active::AXW::new(self)
        }
        /// Enable for the channel 0 counter reload interrupt: 0 masked, 1 routed to the IRQ output.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn chan0_reload(&mut self) -> chan0_reload::AXW<'_> {
            chan0_reload::AXW::new(self)
        }
        /// Enable for the channel 1 counter reload interrupt: 0 masked, 1 routed to the IRQ output.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn chan1_reload(&mut self) -> chan1_reload::AXW<'_> {
            chan1_reload::AXW::new(self)
        }
        /// Enable for the channel 2 counter reload interrupt: 0 masked, 1 routed to the IRQ output.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn chan2_reload(&mut self) -> chan2_reload::AXW<'_> {
            chan2_reload::AXW::new(self)
        }
        /// Enable for the channel 3 counter reload interrupt: 0 masked, 1 routed to the IRQ output.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn chan3_reload(&mut self) -> chan3_reload::AXW<'_> {
            chan3_reload::AXW::new(self)
        }
    }
}

/// Interrupt force bits, one per source: 1 asserts the interrupt from software, 0 removes the force.
/// Hardware access: read Plain; write Plain.
pub type INTF = Reg<intf::INTF_SPEC>;
/// Interrupt force bits, one per source: 1 asserts the interrupt from software, 0 removes the force.
/// Hardware access: read Plain; write Plain.
pub mod intf {
    pub struct INTF_SPEC;
    impl crate::RegisterSpec for INTF_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for INTF_SPEC {}
    pub type R = crate::R<INTF_SPEC>;
    impl crate::Modifiable for INTF_SPEC {}
    impl crate::UnsafeWritable for INTF_SPEC {}
    impl crate::ResetValue for INTF_SPEC {
        const RESET_VALUE: u32 = 0x0;
    }
    impl crate::TypedAtomicSet for INTF_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicSet for INTF_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::TypedAtomicClear for INTF_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicClear for INTF_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::TypedAtomicXor for INTF_SPEC {
        const OFFSET: usize = 0x1000;
    }
    impl crate::UnsafeAtomicXor for INTF_SPEC {
        const OFFSET: usize = 0x1000;
    }
    pub type W = crate::W<INTF_SPEC>;
    pub type AtomicSetW = crate::AtomicSetW<INTF_SPEC>;
    pub type AtomicClearW = crate::AtomicClearW<INTF_SPEC>;
    pub type AtomicXorW = crate::AtomicXorW<INTF_SPEC>;
    /// Force the FIFO underflow (read while empty) interrupt: 1 asserts it, 0 releases the force.
    /// Field write behavior: Plain.
    pub mod fifo_underflow {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 0>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 0>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 0>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 0>;
    }
    /// Force the FIFO overflow (write while full) interrupt: 1 asserts it, 0 releases the force.
    /// Field write behavior: Plain.
    pub mod fifo_overflow {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 1>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 1>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 1>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 1>;
    }
    /// Force the FIFO empty interrupt: 1 asserts it, 0 releases the force.
    /// Field write behavior: Plain.
    pub mod fifo_empty {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 2>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 2>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 2>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 2>;
    }
    /// Force the FIFO full interrupt: 1 asserts it, 0 releases the force.
    /// Field write behavior: Plain.
    pub mod fifo_full {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 3>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 3>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 3>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 3>;
    }
    /// Force the DMA request active interrupt: 1 asserts it, 0 releases the force.
    /// Field write behavior: Plain.
    pub mod dreq_active {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 4>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 4>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 4>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 4>;
    }
    /// Force the channel 0 counter reload interrupt: 1 asserts it, 0 releases the force.
    /// Field write behavior: Plain.
    pub mod chan0_reload {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 5>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 5>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 5>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 5>;
    }
    /// Force the channel 1 counter reload interrupt: 1 asserts it, 0 releases the force.
    /// Field write behavior: Plain.
    pub mod chan1_reload {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 6>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 6>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 6>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 6>;
    }
    /// Force the channel 2 counter reload interrupt: 1 asserts it, 0 releases the force.
    /// Field write behavior: Plain.
    pub mod chan2_reload {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 7>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 7>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 7>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 7>;
    }
    /// Force the channel 3 counter reload interrupt: 1 asserts it, 0 releases the force.
    /// Field write behavior: Plain.
    pub mod chan3_reload {
        pub type R = crate::BitReader;
        pub type W<'a> = crate::BitWriter<'a, super::W, 8>;
        pub type ASW<'a> = crate::AtomicSetBitWriter<'a, super::AtomicSetW, 8>;
        pub type ACW<'a> = crate::AtomicClearBitWriter<'a, super::AtomicClearW, 8>;
        pub type AXW<'a> = crate::AtomicXorBitWriter<'a, super::AtomicXorW, 8>;
    }
    impl R {
        #[inline(always)]
        pub fn fifo_underflow(&self) -> fifo_underflow::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn fifo_overflow(&self) -> fifo_overflow::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
        #[inline(always)]
        pub fn fifo_empty(&self) -> fifo_empty::R {
            crate::BitReader::new(((self.bits() >> 2) & 1) != 0)
        }
        #[inline(always)]
        pub fn fifo_full(&self) -> fifo_full::R {
            crate::BitReader::new(((self.bits() >> 3) & 1) != 0)
        }
        #[inline(always)]
        pub fn dreq_active(&self) -> dreq_active::R {
            crate::BitReader::new(((self.bits() >> 4) & 1) != 0)
        }
        #[inline(always)]
        pub fn chan0_reload(&self) -> chan0_reload::R {
            crate::BitReader::new(((self.bits() >> 5) & 1) != 0)
        }
        #[inline(always)]
        pub fn chan1_reload(&self) -> chan1_reload::R {
            crate::BitReader::new(((self.bits() >> 6) & 1) != 0)
        }
        #[inline(always)]
        pub fn chan2_reload(&self) -> chan2_reload::R {
            crate::BitReader::new(((self.bits() >> 7) & 1) != 0)
        }
        #[inline(always)]
        pub fn chan3_reload(&self) -> chan3_reload::R {
            crate::BitReader::new(((self.bits() >> 8) & 1) != 0)
        }
    }
    impl W {
        #[inline(always)]
        pub fn fifo_underflow(&mut self) -> fifo_underflow::W<'_> {
            fifo_underflow::W::new(self)
        }
        #[inline(always)]
        pub fn fifo_overflow(&mut self) -> fifo_overflow::W<'_> {
            fifo_overflow::W::new(self)
        }
        #[inline(always)]
        pub fn fifo_empty(&mut self) -> fifo_empty::W<'_> {
            fifo_empty::W::new(self)
        }
        #[inline(always)]
        pub fn fifo_full(&mut self) -> fifo_full::W<'_> {
            fifo_full::W::new(self)
        }
        #[inline(always)]
        pub fn dreq_active(&mut self) -> dreq_active::W<'_> {
            dreq_active::W::new(self)
        }
        #[inline(always)]
        pub fn chan0_reload(&mut self) -> chan0_reload::W<'_> {
            chan0_reload::W::new(self)
        }
        #[inline(always)]
        pub fn chan1_reload(&mut self) -> chan1_reload::W<'_> {
            chan1_reload::W::new(self)
        }
        #[inline(always)]
        pub fn chan2_reload(&mut self) -> chan2_reload::W<'_> {
            chan2_reload::W::new(self)
        }
        #[inline(always)]
        pub fn chan3_reload(&mut self) -> chan3_reload::W<'_> {
            chan3_reload::W::new(self)
        }
    }
    impl AtomicSetW {
        /// Force the FIFO underflow (read while empty) interrupt: 1 asserts it, 0 releases the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn fifo_underflow(&mut self) -> fifo_underflow::ASW<'_> {
            fifo_underflow::ASW::new(self)
        }
        /// Force the FIFO overflow (write while full) interrupt: 1 asserts it, 0 releases the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn fifo_overflow(&mut self) -> fifo_overflow::ASW<'_> {
            fifo_overflow::ASW::new(self)
        }
        /// Force the FIFO empty interrupt: 1 asserts it, 0 releases the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn fifo_empty(&mut self) -> fifo_empty::ASW<'_> {
            fifo_empty::ASW::new(self)
        }
        /// Force the FIFO full interrupt: 1 asserts it, 0 releases the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn fifo_full(&mut self) -> fifo_full::ASW<'_> {
            fifo_full::ASW::new(self)
        }
        /// Force the DMA request active interrupt: 1 asserts it, 0 releases the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn dreq_active(&mut self) -> dreq_active::ASW<'_> {
            dreq_active::ASW::new(self)
        }
        /// Force the channel 0 counter reload interrupt: 1 asserts it, 0 releases the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn chan0_reload(&mut self) -> chan0_reload::ASW<'_> {
            chan0_reload::ASW::new(self)
        }
        /// Force the channel 1 counter reload interrupt: 1 asserts it, 0 releases the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn chan1_reload(&mut self) -> chan1_reload::ASW<'_> {
            chan1_reload::ASW::new(self)
        }
        /// Force the channel 2 counter reload interrupt: 1 asserts it, 0 releases the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn chan2_reload(&mut self) -> chan2_reload::ASW<'_> {
            chan2_reload::ASW::new(self)
        }
        /// Force the channel 3 counter reload interrupt: 1 asserts it, 0 releases the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn chan3_reload(&mut self) -> chan3_reload::ASW<'_> {
            chan3_reload::ASW::new(self)
        }
    }
    impl AtomicClearW {
        /// Force the FIFO underflow (read while empty) interrupt: 1 asserts it, 0 releases the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn fifo_underflow(&mut self) -> fifo_underflow::ACW<'_> {
            fifo_underflow::ACW::new(self)
        }
        /// Force the FIFO overflow (write while full) interrupt: 1 asserts it, 0 releases the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn fifo_overflow(&mut self) -> fifo_overflow::ACW<'_> {
            fifo_overflow::ACW::new(self)
        }
        /// Force the FIFO empty interrupt: 1 asserts it, 0 releases the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn fifo_empty(&mut self) -> fifo_empty::ACW<'_> {
            fifo_empty::ACW::new(self)
        }
        /// Force the FIFO full interrupt: 1 asserts it, 0 releases the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn fifo_full(&mut self) -> fifo_full::ACW<'_> {
            fifo_full::ACW::new(self)
        }
        /// Force the DMA request active interrupt: 1 asserts it, 0 releases the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn dreq_active(&mut self) -> dreq_active::ACW<'_> {
            dreq_active::ACW::new(self)
        }
        /// Force the channel 0 counter reload interrupt: 1 asserts it, 0 releases the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn chan0_reload(&mut self) -> chan0_reload::ACW<'_> {
            chan0_reload::ACW::new(self)
        }
        /// Force the channel 1 counter reload interrupt: 1 asserts it, 0 releases the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn chan1_reload(&mut self) -> chan1_reload::ACW<'_> {
            chan1_reload::ACW::new(self)
        }
        /// Force the channel 2 counter reload interrupt: 1 asserts it, 0 releases the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn chan2_reload(&mut self) -> chan2_reload::ACW<'_> {
            chan2_reload::ACW::new(self)
        }
        /// Force the channel 3 counter reload interrupt: 1 asserts it, 0 releases the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn chan3_reload(&mut self) -> chan3_reload::ACW<'_> {
            chan3_reload::ACW::new(self)
        }
    }
    impl AtomicXorW {
        /// Force the FIFO underflow (read while empty) interrupt: 1 asserts it, 0 releases the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn fifo_underflow(&mut self) -> fifo_underflow::AXW<'_> {
            fifo_underflow::AXW::new(self)
        }
        /// Force the FIFO overflow (write while full) interrupt: 1 asserts it, 0 releases the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn fifo_overflow(&mut self) -> fifo_overflow::AXW<'_> {
            fifo_overflow::AXW::new(self)
        }
        /// Force the FIFO empty interrupt: 1 asserts it, 0 releases the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn fifo_empty(&mut self) -> fifo_empty::AXW<'_> {
            fifo_empty::AXW::new(self)
        }
        /// Force the FIFO full interrupt: 1 asserts it, 0 releases the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn fifo_full(&mut self) -> fifo_full::AXW<'_> {
            fifo_full::AXW::new(self)
        }
        /// Force the DMA request active interrupt: 1 asserts it, 0 releases the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn dreq_active(&mut self) -> dreq_active::AXW<'_> {
            dreq_active::AXW::new(self)
        }
        /// Force the channel 0 counter reload interrupt: 1 asserts it, 0 releases the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn chan0_reload(&mut self) -> chan0_reload::AXW<'_> {
            chan0_reload::AXW::new(self)
        }
        /// Force the channel 1 counter reload interrupt: 1 asserts it, 0 releases the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn chan1_reload(&mut self) -> chan1_reload::AXW<'_> {
            chan1_reload::AXW::new(self)
        }
        /// Force the channel 2 counter reload interrupt: 1 asserts it, 0 releases the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn chan2_reload(&mut self) -> chan2_reload::AXW<'_> {
            chan2_reload::AXW::new(self)
        }
        /// Force the channel 3 counter reload interrupt: 1 asserts it, 0 releases the force.
        /// Field write behavior: Plain.
        #[inline(always)]
        pub fn chan3_reload(&mut self) -> chan3_reload::AXW<'_> {
            chan3_reload::AXW::new(self)
        }
    }
}

/// Masked interrupt status: (INTR & INTE) | INTF.
/// Hardware access: read Plain; write None.
pub type INTS = Reg<ints::INTS_SPEC>;
/// Masked interrupt status: (INTR & INTE) | INTF.
/// Hardware access: read Plain; write None.
pub mod ints {
    pub struct INTS_SPEC;
    impl crate::RegisterSpec for INTS_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for INTS_SPEC {}
    pub type R = crate::R<INTS_SPEC>;
    impl crate::ResetValue for INTS_SPEC {
        const RESET_VALUE: u32 = 0x0;
    }
    /// 1 when the FIFO underflow (read while empty) interrupt is active after masking and forcing.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod fifo_underflow {
        pub type R = crate::BitReader;
    }
    /// 1 when the FIFO overflow (write while full) interrupt is active after masking and forcing.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod fifo_overflow {
        pub type R = crate::BitReader;
    }
    /// 1 when the FIFO empty interrupt is active after masking and forcing.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod fifo_empty {
        pub type R = crate::BitReader;
    }
    /// 1 when the FIFO full interrupt is active after masking and forcing.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod fifo_full {
        pub type R = crate::BitReader;
    }
    /// 1 when the DMA request active interrupt is active after masking and forcing.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod dreq_active {
        pub type R = crate::BitReader;
    }
    /// 1 when the channel 0 counter reload interrupt is active after masking and forcing.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod chan0_reload {
        pub type R = crate::BitReader;
    }
    /// 1 when the channel 1 counter reload interrupt is active after masking and forcing.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod chan1_reload {
        pub type R = crate::BitReader;
    }
    /// 1 when the channel 2 counter reload interrupt is active after masking and forcing.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod chan2_reload {
        pub type R = crate::BitReader;
    }
    /// 1 when the channel 3 counter reload interrupt is active after masking and forcing.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod chan3_reload {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn fifo_underflow(&self) -> fifo_underflow::R {
            crate::BitReader::new(((self.bits() >> 0) & 1) != 0)
        }
        #[inline(always)]
        pub fn fifo_overflow(&self) -> fifo_overflow::R {
            crate::BitReader::new(((self.bits() >> 1) & 1) != 0)
        }
        #[inline(always)]
        pub fn fifo_empty(&self) -> fifo_empty::R {
            crate::BitReader::new(((self.bits() >> 2) & 1) != 0)
        }
        #[inline(always)]
        pub fn fifo_full(&self) -> fifo_full::R {
            crate::BitReader::new(((self.bits() >> 3) & 1) != 0)
        }
        #[inline(always)]
        pub fn dreq_active(&self) -> dreq_active::R {
            crate::BitReader::new(((self.bits() >> 4) & 1) != 0)
        }
        #[inline(always)]
        pub fn chan0_reload(&self) -> chan0_reload::R {
            crate::BitReader::new(((self.bits() >> 5) & 1) != 0)
        }
        #[inline(always)]
        pub fn chan1_reload(&self) -> chan1_reload::R {
            crate::BitReader::new(((self.bits() >> 6) & 1) != 0)
        }
        #[inline(always)]
        pub fn chan2_reload(&self) -> chan2_reload::R {
            crate::BitReader::new(((self.bits() >> 7) & 1) != 0)
        }
        #[inline(always)]
        pub fn chan3_reload(&self) -> chan3_reload::R {
            crate::BitReader::new(((self.bits() >> 8) & 1) != 0)
        }
    }
}

#[cfg(test)]
mod layout_tests {
    use super::RegisterBlock;
    #[test]
    fn global_ctrl_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, global_ctrl), 0x0);
    }
    #[test]
    fn fifo_ctrl_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, fifo_ctrl), 0x4);
    }
    #[test]
    fn common_range_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, common_range), 0x8);
    }
    #[test]
    fn common_duty_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, common_duty), 0xc);
    }
    #[test]
    fn duty_fifo_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, duty_fifo), 0x10);
    }
    #[test]
    fn chan0_ctrl_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, chan0_ctrl), 0x14);
    }
    #[test]
    fn chan0_range_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, chan0_range), 0x18);
    }
    #[test]
    fn chan0_phase_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, chan0_phase), 0x1c);
    }
    #[test]
    fn chan0_duty_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, chan0_duty), 0x20);
    }
    #[test]
    fn chan1_ctrl_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, chan1_ctrl), 0x24);
    }
    #[test]
    fn chan1_range_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, chan1_range), 0x28);
    }
    #[test]
    fn chan1_phase_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, chan1_phase), 0x2c);
    }
    #[test]
    fn chan1_duty_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, chan1_duty), 0x30);
    }
    #[test]
    fn chan2_ctrl_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, chan2_ctrl), 0x34);
    }
    #[test]
    fn chan2_range_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, chan2_range), 0x38);
    }
    #[test]
    fn chan2_phase_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, chan2_phase), 0x3c);
    }
    #[test]
    fn chan2_duty_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, chan2_duty), 0x40);
    }
    #[test]
    fn chan3_ctrl_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, chan3_ctrl), 0x44);
    }
    #[test]
    fn chan3_range_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, chan3_range), 0x48);
    }
    #[test]
    fn chan3_phase_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, chan3_phase), 0x4c);
    }
    #[test]
    fn chan3_duty_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, chan3_duty), 0x50);
    }
    #[test]
    fn intr_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, intr), 0x54);
    }
    #[test]
    fn inte_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, inte), 0x58);
    }
    #[test]
    fn intf_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, intf), 0x5c);
    }
    #[test]
    fn ints_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, ints), 0x60);
    }
    #[test]
    fn register_block_size() {
        assert_eq!(core::mem::size_of::<RegisterBlock>(), 0x64);
    }
}
