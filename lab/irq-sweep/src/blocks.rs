//! How each swept block raises and lowers its interrupt output without using a pin: the
//! per-line contracts of the sweep. Offsets are relative to the block's base address.
use crate::reg::{CLR, Reset, SET, clear, poll, rd, set, wr};
use crate::sweep::{Check, Found, Pin, Refusal, Rule, SETTLE_US, STEP_US, Source, Why};

/// The RP1 wrapper around a DesignWare block: bit 0 lets its interrupt out.
const WRAPPER: usize = 0x108;

/// A DesignWare I2C master. `TX_EMPTY` is true as soon as the controller is enabled with an
/// empty FIFO; no target address or command is written, so nothing starts on the bus.
pub struct I2c {
    pub base: usize,
    pub reset: Reset,
    pub pins: &'static [Pin],
}
mod i2c {
    pub const CON: usize = 0x00;
    pub const INTR_STAT: usize = 0x2c;
    pub const INTR_MASK: usize = 0x30;
    pub const RAW_INTR_STAT: usize = 0x34;
    pub const TX_TL: usize = 0x3c;
    pub const CLR_INTR: usize = 0x40;
    pub const ENABLE: usize = 0x6c;
    pub const ENABLE_STATUS: usize = 0x9c;
    pub const COMP_TYPE: usize = 0xfc;
    pub const TX_EMPTY: u32 = 1 << 4;
    /// Standard-mode master with restart, target part disabled: what the driver programs.
    pub const CON_MASTER: u32 = 0x63;
    pub const DESIGNWARE: u32 = 0x4457_0140;
}
impl I2c {
    fn disable(&self) -> bool {
        wr(self.base + i2c::ENABLE, 0);
        poll(STEP_US, || rd(self.base + i2c::ENABLE_STATUS) & 1 == 0)
    }
}
impl Source for I2c {
    fn base(&self) -> usize {
        self.base
    }
    fn pins(&self) -> &'static [Pin] {
        self.pins
    }
    fn stages(&self) -> &'static [&'static str] {
        &["wrap0", "wrap1"]
    }
    // The reset is cycled first, so every refusal has touched the block; one of the reset
    // itself leaves it alone.
    fn prepare(&self) -> Result<Found, Refusal> {
        if !self.reset.cycle() {
            return Err(Refusal::Stuck(Why::new("reset")));
        }
        let found = [rd(self.base + WRAPPER), rd(self.base + i2c::COMP_TYPE)];
        if found[1] != i2c::DESIGNWARE {
            return Err(Refusal::Touched(Why::with("not_alive", found[1])));
        }
        wr(self.base + i2c::INTR_MASK, 0);
        clear(self.base + WRAPPER, 1);
        // The controller only takes its configuration while it is disabled.
        if !self.disable() {
            return Err(Refusal::Touched(Why::new("busy")));
        }
        rd(self.base + i2c::CLR_INTR);
        wr(self.base + i2c::CON, i2c::CON_MASTER);
        wr(self.base + i2c::TX_TL, 0);
        Ok(found)
    }
    fn baseline(&self) -> &'static [Check] {
        const BASELINE: &[Check] = &[
            Check::new(i2c::ENABLE_STATUS, 1, 0),
            Check::new(i2c::INTR_MASK, !0, 0),
            Check::new(i2c::INTR_STAT, 0x1fff, 0),
            Check::new(WRAPPER, 1, 0),
        ];
        BASELINE
    }
    fn raise(&self, stage: usize) -> bool {
        wr(self.base + i2c::ENABLE, 1);
        let raw = poll(SETTLE_US, || {
            rd(self.base + i2c::RAW_INTR_STAT) & i2c::TX_EMPTY != 0
        });
        wr(self.base + i2c::INTR_MASK, i2c::TX_EMPTY);
        if stage == 1 {
            set(self.base + WRAPPER, 1);
        }
        raw && rd(self.base + i2c::INTR_STAT) & i2c::TX_EMPTY != 0
            && rd(self.base + WRAPPER) & 1 == stage as u32
    }
    fn status(&self) -> u32 {
        rd(self.base + i2c::INTR_STAT) & 0xffff | (rd(self.base + WRAPPER) & 1) << 16
    }
    fn lower(&self) {
        clear(self.base + WRAPPER, 1);
        wr(self.base + i2c::INTR_MASK, 0);
    }
    fn lowered(&self) -> bool {
        rd(self.base + WRAPPER) & 1 == 0 && rd(self.base + i2c::INTR_STAT) & i2c::TX_EMPTY == 0
    }
    fn rearm(&self) -> bool {
        self.disable()
    }
    fn cleanup(&self) -> bool {
        // A controller that does not report "disabled" is still put back into reset, as long
        // as its mask, wrapper and output read 0: their reset values are not known to be 0.
        self.disable();
        rd(self.base + i2c::INTR_MASK) == 0 && self.lowered() && self.reset.hold()
    }
}

/// A DesignWare SSI master. `TXEI` is the level of the empty transmit FIFO once the block is
/// enabled; no slave is selected and no data is written, so no clock or chip select appears.
pub struct Spi {
    pub base: usize,
    pub reset: Reset,
    pub pins: &'static [Pin],
}
mod ssi {
    pub const SSIENR: usize = 0x08;
    pub const SER: usize = 0x10;
    pub const TXFTLR: usize = 0x18;
    pub const IMR: usize = 0x2c;
    pub const ISR: usize = 0x30;
    pub const RISR: usize = 0x34;
    pub const ICR: usize = 0x48;
    pub const DMACR: usize = 0x4c;
    pub const VERSION_ID: usize = 0x5c;
    pub const TXEI: u32 = 1;
}
impl Source for Spi {
    fn base(&self) -> usize {
        self.base
    }
    fn pins(&self) -> &'static [Pin] {
        self.pins
    }
    fn stages(&self) -> &'static [&'static str] {
        &["wrap0", "wrap1"]
    }
    // As for I2C, every refusal comes after the reset was cycled.
    fn prepare(&self) -> Result<Found, Refusal> {
        if !self.reset.cycle() {
            return Err(Refusal::Stuck(Why::new("reset")));
        }
        let found = [rd(self.base + WRAPPER), rd(self.base + ssi::VERSION_ID)];
        wr(self.base + ssi::SSIENR, 0);
        // The block answers if a writable register keeps a value (the driver's probe).
        wr(self.base + ssi::TXFTLR, 1);
        if rd(self.base + ssi::TXFTLR) != 1 {
            return Err(Refusal::Touched(Why::with("not_alive", found[1])));
        }
        wr(self.base + ssi::IMR, 0);
        clear(self.base + WRAPPER, 1);
        wr(self.base + ssi::SER, 0);
        wr(self.base + ssi::DMACR, 0);
        rd(self.base + ssi::ICR);
        wr(self.base + ssi::TXFTLR, 0);
        Ok(found)
    }
    fn baseline(&self) -> &'static [Check] {
        const BASELINE: &[Check] = &[
            Check::new(ssi::SSIENR, 1, 0),
            Check::new(ssi::IMR, !0, 0),
            Check::new(ssi::ISR, 0x3f, 0),
            Check::new(ssi::SER, !0, 0),
            Check::new(WRAPPER, 1, 0),
        ];
        BASELINE
    }
    fn raise(&self, stage: usize) -> bool {
        wr(self.base + ssi::SSIENR, 1);
        let raw = poll(SETTLE_US, || rd(self.base + ssi::RISR) & ssi::TXEI != 0);
        wr(self.base + ssi::IMR, ssi::TXEI);
        if stage == 1 {
            set(self.base + WRAPPER, 1);
        }
        raw && rd(self.base + ssi::ISR) & ssi::TXEI != 0
            && rd(self.base + WRAPPER) & 1 == stage as u32
    }
    fn status(&self) -> u32 {
        rd(self.base + ssi::ISR) & 0xffff | (rd(self.base + WRAPPER) & 1) << 16
    }
    // The raw TXEI cannot be lowered while the block is enabled; the mask and the wrapper can.
    fn lower(&self) {
        clear(self.base + WRAPPER, 1);
        wr(self.base + ssi::IMR, 0);
    }
    fn lowered(&self) -> bool {
        rd(self.base + WRAPPER) & 1 == 0 && rd(self.base + ssi::ISR) & ssi::TXEI == 0
    }
    fn rearm(&self) -> bool {
        wr(self.base + ssi::SSIENR, 0);
        rd(self.base + ssi::SSIENR) & 1 == 0
    }
    fn cleanup(&self) -> bool {
        wr(self.base + ssi::SSIENR, 0);
        rd(self.base + ssi::IMR) == 0 && self.lowered() && self.reset.hold()
    }
}

/// PWM1, by its force register only. No channel is ever enabled, and `GLOBAL_CTRL` is checked
/// for that before every trial; that is what keeps the block's outputs quiet. The fan pin
/// (GPIO45) must also select no function for this line to run. That it is PWM1's only pin is
/// an assumption (HYP): the repository has no pin table for banks 1 and 2.
pub struct Pwm1;
mod pwm {
    pub const BASE: usize = 0x4009_c000;
    pub const GLOBAL_CTRL: usize = 0x00;
    pub const INTR: usize = 0x54;
    pub const INTE: usize = 0x58;
    pub const INTF: usize = 0x5c;
    pub const INTS: usize = 0x60;
    pub const CHANNELS: u32 = 0xf;
    /// The write-to-clear bits of INTR (the others report the FIFO).
    pub const INTR_CLEAR: u32 = 0x1e3;
    pub const FORCED: u32 = 1 << 5;
    pub const RESET: super::Reset = super::Reset { bank: 1, bit: 5 };
}
pub const FAN_PIN: Pin = Pin {
    gpio: 45,
    rule: Rule::Only(crate::sweep::FUNCSEL_NULL),
};
impl Source for Pwm1 {
    fn base(&self) -> usize {
        pwm::BASE
    }
    fn pins(&self) -> &'static [Pin] {
        &[FAN_PIN]
    }
    fn stages(&self) -> &'static [&'static str] {
        &["force"]
    }
    fn prepare(&self) -> Result<Found, Refusal> {
        // Out of reset only if the driver's `init` ran at startup.
        if pwm::RESET.held() {
            return Err(Refusal::Untouched(Why::new("not_initialised")));
        }
        wr(pwm::BASE + pwm::INTR, pwm::INTR_CLEAR);
        Ok([0; 2])
    }
    fn baseline(&self) -> &'static [Check] {
        const BASELINE: &[Check] = &[
            Check::new(pwm::GLOBAL_CTRL, pwm::CHANNELS, 0),
            Check::new(pwm::INTE, !0, 0),
            Check::new(pwm::INTF, !0, 0),
            Check::new(pwm::INTS, 0x1ff, 0),
        ];
        BASELINE
    }
    fn raise(&self, _: usize) -> bool {
        wr(pwm::BASE + pwm::INTF, pwm::FORCED);
        rd(pwm::BASE + pwm::INTS) & pwm::FORCED != 0
    }
    fn status(&self) -> u32 {
        rd(pwm::BASE + pwm::INTS)
    }
    fn lower(&self) {
        wr(pwm::BASE + pwm::INTF, 0);
    }
    fn lowered(&self) -> bool {
        rd(pwm::BASE + pwm::INTF) == 0 && rd(pwm::BASE + pwm::INTS) & pwm::FORCED == 0
    }
    fn rearm(&self) -> bool {
        true
    }
    fn cleanup(&self) -> bool {
        wr(pwm::BASE + pwm::INTE, 0);
        wr(pwm::BASE + pwm::INTR, pwm::INTR_CLEAR);
        rd(pwm::BASE + pwm::GLOBAL_CTRL) & pwm::CHANNELS == 0
            && rd(pwm::BASE + pwm::INTE) == 0
            && self.lowered()
            && pwm::RESET.hold()
    }
}

/// The ADC FIFO interrupt: first forced, then real, from one conversion of the internal
/// temperature sensor into the FIFO (no external input is selected).
pub struct Adc;
mod adc {
    pub const BASE: usize = 0x400c_8000;
    pub const CS: usize = 0x00;
    pub const FCS: usize = 0x08;
    pub const FIFO: usize = 0x0c;
    pub const INTE: usize = 0x18;
    pub const INTF: usize = 0x1c;
    pub const INTS: usize = 0x20;
    pub const CS_EN: u32 = 1;
    pub const CS_START_ONCE: u32 = 1 << 2;
    pub const CS_READY: u32 = 1 << 8;
    /// Powered, temperature sensor on and selected (AINSEL = 4).
    pub const CS_TEMPERATURE: u32 = 0x4003;
    pub const FCS_EMPTY: u32 = 1 << 8;
    pub const FCS_LEVEL: u32 = 0xf << 16;
    /// FIFO on, interrupt threshold of one sample.
    pub const FCS_ONE_SAMPLE: u32 = 0x0100_0001;
    pub const FIFO_DEPTH: usize = 16;
}
impl Adc {
    fn ready() -> bool {
        poll(STEP_US, || rd(adc::BASE + adc::CS) & adc::CS_READY != 0)
    }
    /// Empties the FIFO; returns whether it reports empty.
    fn drain() -> bool {
        for _ in 0..adc::FIFO_DEPTH {
            if rd(adc::BASE + adc::FCS) & adc::FCS_EMPTY != 0 {
                break;
            }
            rd(adc::BASE + adc::FIFO);
        }
        rd(adc::BASE + adc::FCS) & adc::FCS_EMPTY != 0
    }
}
impl Source for Adc {
    fn base(&self) -> usize {
        adc::BASE
    }
    fn pins(&self) -> &'static [Pin] {
        &[]
    }
    fn stages(&self) -> &'static [&'static str] {
        &["force", "real"]
    }
    fn prepare(&self) -> Result<Found, Refusal> {
        // Powered and ready only if the driver's `init` ran at startup.
        let cs = rd(adc::BASE + adc::CS);
        if cs & (adc::CS_EN | adc::CS_READY) != adc::CS_EN | adc::CS_READY {
            return Err(Refusal::Untouched(Why::with("not_initialised", cs)));
        }
        wr(adc::BASE + adc::INTF, 0);
        if !Self::drain() {
            let fcs = rd(adc::BASE + adc::FCS);
            return Err(Refusal::Touched(Why::with("fifo", fcs)));
        }
        Ok([0; 2])
    }
    fn baseline(&self) -> &'static [Check] {
        const BASELINE: &[Check] = &[
            Check::new(adc::INTE, !0, 0),
            Check::new(adc::INTF, !0, 0),
            Check::new(adc::INTS, !0, 0),
            // FIFO off, empty (the EMPTY flag reads 1), and no conversion running.
            Check::new(
                adc::FCS,
                0x1 | adc::FCS_EMPTY | adc::FCS_LEVEL,
                adc::FCS_EMPTY,
            ),
            Check::new(adc::CS, 0xc, 0),
        ];
        BASELINE
    }
    fn raise(&self, stage: usize) -> bool {
        if stage == 0 {
            wr(adc::BASE + adc::INTF, 1);
        } else {
            wr(adc::BASE + adc::CS, adc::CS_TEMPERATURE);
            let powered = Self::ready();
            wr(adc::BASE + adc::FCS, adc::FCS_ONE_SAMPLE);
            wr(
                adc::BASE + adc::CS,
                adc::CS_TEMPERATURE | adc::CS_START_ONCE,
            );
            if !(powered && Self::ready() && rd(adc::BASE + adc::FCS) & adc::FCS_LEVEL != 0) {
                return false;
            }
            wr(adc::BASE + adc::INTE, 1);
        }
        rd(adc::BASE + adc::INTS) & 1 != 0
    }
    fn status(&self) -> u32 {
        rd(adc::BASE + adc::INTS) & 0xffff | (rd(adc::BASE + adc::FCS) & adc::FCS_LEVEL)
    }
    fn lower(&self) {
        wr(adc::BASE + adc::INTF, 0);
        wr(adc::BASE + adc::INTE, 0);
    }
    fn lowered(&self) -> bool {
        rd(adc::BASE + adc::INTS) & 1 == 0
    }
    fn rearm(&self) -> bool {
        let empty = Self::drain();
        wr(adc::BASE + adc::FCS, 0);
        wr(adc::BASE + adc::CS, adc::CS_EN);
        empty
    }
    fn cleanup(&self) -> bool {
        Self::drain();
        wr(adc::BASE + adc::FCS, 0);
        // Powered down like the driver's `free`. The reset stays released: the name of the
        // ADC's reset bit is an estimate.
        wr(adc::BASE + adc::CS, 0);
        rd(adc::BASE + adc::INTE) == 0
            && rd(adc::BASE + adc::INTF) == 0
            && rd(adc::BASE + adc::CS) & 0x3 == 0
            && rd(adc::BASE + adc::FCS) & 1 == 0
            && self.lowered()
    }
}

/// proc0's summary interrupt of a GPIO bank Linux owns, by forcing one pin's status bit. Only
/// run while proc0's enable, force and status words all read 0, and nothing but the one force
/// bit is written (through the SET/CLR aliases, so no other bit is rewritten).
pub struct Bank {
    pub base: usize,
}
mod bank {
    pub const PROC0_INTE: usize = 0x104;
    pub const PROC0_INTF: usize = 0x108;
    pub const PROC0_INTS: usize = 0x10c;
}
impl Source for Bank {
    fn base(&self) -> usize {
        self.base
    }
    fn pins(&self) -> &'static [Pin] {
        &[]
    }
    fn stages(&self) -> &'static [&'static str] {
        &["force"]
    }
    // Reads only: a bank in use is left exactly as it was found.
    fn prepare(&self) -> Result<Found, Refusal> {
        let used = [bank::PROC0_INTE, bank::PROC0_INTF, bank::PROC0_INTS]
            .iter()
            .fold(0, |used, offset| used | rd(self.base + offset));
        if used != 0 {
            return Err(Refusal::Untouched(Why::with("proc0_in_use", used)));
        }
        Ok([0; 2])
    }
    fn baseline(&self) -> &'static [Check] {
        const BASELINE: &[Check] = &[
            Check::new(bank::PROC0_INTE, !0, 0),
            Check::new(bank::PROC0_INTF, !0, 0),
            Check::new(bank::PROC0_INTS, !0, 0),
        ];
        BASELINE
    }
    fn raise(&self, _: usize) -> bool {
        wr(self.base + SET + bank::PROC0_INTF, 1);
        rd(self.base + bank::PROC0_INTS) & 1 != 0
    }
    fn status(&self) -> u32 {
        rd(self.base + bank::PROC0_INTS)
    }
    fn lower(&self) {
        wr(self.base + CLR + bank::PROC0_INTF, 1);
    }
    fn lowered(&self) -> bool {
        rd(self.base + bank::PROC0_INTF) & 1 == 0 && rd(self.base + bank::PROC0_INTS) & 1 == 0
    }
    fn rearm(&self) -> bool {
        true
    }
    // The deassert already cleared the one force bit; nothing else of the bank was written.
    fn cleanup(&self) -> bool {
        self.lowered() && rd(self.base + bank::PROC0_INTE) == 0
    }
}
