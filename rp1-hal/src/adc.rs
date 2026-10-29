//! RP1 ADC: 12-bit one-shot conversions of the four external inputs and the temperature sensor,
//! by polling (no FIFO, DMA or interrupt).
//!
//! Register contract (CS, RESULT; rp1-pac `adc`) and sequence are those of the Linux RP1 ADC
//! hwmon driver (STATIC): enable with the sticky error cleared, select AINSEL through the CLR/SET
//! aliases, START_ONCE, wait READY, reject ERR, read RESULT. The functional clock is clk_adc =
//! XOSC / 1 = 50 MHz, the rate Linux asks for and the one the SCMI table lists for `adc`
//! ([`crate::scmi::RP1_CLOCKS`]); with `ADC` declared, the SCMI server holds that clock (reports
//! it, never changes or stops it) and the overlay disables Linux's `rp1_adc`.
//!
//! Which RP1 ADC inputs are wired to what is board-specific; the CM5 IO board's wiring is not
//! documented here, so only the temperature sensor has a known meaning.
use crate::{
    linux_blocks::Mmio,
    scmi::{Kind, RP1_CLOCKS, Round, id},
};
use crate::{read, write};

const BASE: usize = 0x400c_8000;
const CS: usize = BASE;
const RESULT: usize = BASE + 0x4;
const INTE: usize = BASE + 0x18;
const SET: usize = 0x2000;
const CLR: usize = 0x3000;

const CS_EN: u32 = 1 << 0;
const CS_TS_EN: u32 = 1 << 1;
const CS_START_ONCE: u32 = 1 << 2;
const CS_START_MANY: u32 = 1 << 3;
const CS_READY: u32 = 1 << 8;
const CS_ERR: u32 = 1 << 9;
const CS_ERR_STICKY: u32 = 1 << 10;
const CS_AINSEL: u32 = 0x7 << 12;
const CS_RROBIN: u32 = 0x1f << 16;
const RESULT_MASK: u32 = 0xfff;

/// RESETS CTRL0 bit 0 (HW: releasing it from Linux made the ADC convert). Its DONE bit stays 0
/// even under the stock firmware (HW), so it is not waited for.
const RESETS_CLR0: usize = 0x4001_4000 + CLR;
const RESET_BIT: u32 = 1;

/// A conversion takes 96 clk_adc cycles (≈ 1.9 µs at 50 MHz, DOC RP2040-class ADC); this bound
/// only catches a stopped clock or block.
const TIMEOUT_US: u32 = 1_000;

/// Full-scale reference of the rpi-cm5 board (`vref-supply = <&rp1_vdd_3v3>`, DOC stock DT).
pub const VREF_MV: u32 = 3300;
/// AINSEL of the temperature sensor.
pub const TEMPERATURE: u8 = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    /// The clock tree is not the one [`crate::clocks::Clocks`] was created from, or clk_adc
    /// could not be started.
    Precondition,
    /// READY did not come up within 1 ms.
    Timeout,
    /// The ADC flagged the conversion (CS.ERR).
    Conversion,
    /// No such input (AINSEL 0..=4).
    Invalid,
}

/// Ownership of the ADC block, issued by the generated resources for `peripherals = ["ADC"]`.
pub struct Token {
    _private: (),
}

impl Token {
    /// # Safety
    /// Generated resources only: sole owner of the ADC, its reset and its clock generator, with
    /// Linux's `rp1_adc` disabled and the SCMI server holding clk_adc.
    pub const unsafe fn new() -> Self {
        Self { _private: () }
    }

    /// Starts clk_adc (XOSC / 1), releases the ADC reset, powers the ADC up with the interrupt
    /// off and the sticky error cleared, and waits for READY. `clocks` proves the cold bring-up
    /// and the clock authority ran first (the Linux-block bring-up then leaves clk_adc alone).
    pub fn init(self, clocks: &crate::clocks::Clocks) -> Result<Adc, Error> {
        if !clocks.still_current() {
            return Err(Error::Precondition);
        }
        let Kind::Linux(clk) = RP1_CLOCKS[id::ADC as usize].kind else {
            return Err(Error::Precondition);
        };
        let (aux, div, hz) = clk
            .choose(crate::clocks::XOSC_HZ, Round::Down)
            .ok_or(Error::Precondition)?;
        let mut r = Mmio;
        let running = clk.state(&mut r);
        if running.is_none_or(|s| s.rate_hz != hz) {
            clk.program(&mut r, aux, div);
        }
        clk.set_enabled(&mut r, true);
        if clk.state(&mut r)
            != Some(crate::scmi::PhysicalState {
                rate_hz: hz,
                enabled: true,
            })
        {
            return Err(Error::Precondition);
        }
        write(RESETS_CLR0, RESET_BIT);
        write(INTE, 0);
        write(CS, CS_EN | CS_ERR_STICKY);
        crate::poll(CS, CS_READY, CS_READY).map_err(|_| Error::Timeout)?;
        Ok(Adc {
            clk_hz: hz,
            temperature_sensor: false,
        })
    }
}

/// A powered ADC doing one conversion at a time.
pub struct Adc {
    clk_hz: u32,
    temperature_sensor: bool,
}

impl Adc {
    /// clk_adc as started by [`Token::init`].
    pub const fn clock_hz(&self) -> u32 {
        self.clk_hz
    }

    /// Powers the temperature sensor (CS.TS_EN) on or off. [`Adc::read`] of
    /// [`TEMPERATURE`] turns it on by itself, as Linux does, and leaves it on.
    pub fn set_temperature_sensor(&mut self, on: bool) {
        write(if on { CS + SET } else { CS + CLR }, CS_TS_EN);
        self.temperature_sensor = on;
    }

    /// One conversion of input `ain` (0..=3 external, [`TEMPERATURE`]): the 12-bit result.
    pub fn read(&mut self, ain: u8) -> Result<u16, Error> {
        if ain > TEMPERATURE {
            return Err(Error::Invalid);
        }
        if ain == TEMPERATURE && !self.temperature_sensor {
            self.set_temperature_sensor(true);
        }
        write(CS + CLR, CS_AINSEL | CS_START_MANY | CS_RROBIN);
        write(CS + SET, u32::from(ain) << 12);
        wait_ready()?;
        write(CS + SET, CS_START_ONCE);
        let cs = wait_ready()?;
        if cs & CS_ERR != 0 {
            return Err(Error::Conversion);
        }
        Ok((read(RESULT) & RESULT_MASK) as u16)
    }

    /// One conversion of the temperature sensor, in millidegrees Celsius
    /// ([`millivolts_to_millicelsius`] of the result scaled by [`VREF_MV`]).
    pub fn read_temperature(&mut self) -> Result<i32, Error> {
        let raw = self.read(TEMPERATURE)?;
        Ok(millivolts_to_millicelsius(raw_to_millivolts(raw, VREF_MV)))
    }

    /// Powers the ADC down and gives the block back (the clock and reset stay as they are).
    pub fn free(self) -> Token {
        write(CS, 0);
        Token { _private: () }
    }
}

fn wait_ready() -> Result<u32, Error> {
    let start = read(crate::TIMER_RAWL);
    loop {
        let cs = read(CS);
        if cs & CS_READY != 0 {
            return Ok(cs);
        }
        if read(crate::TIMER_RAWL).wrapping_sub(start) > TIMEOUT_US {
            let cs = read(CS);
            return if cs & CS_READY != 0 {
                Ok(cs)
            } else {
                Err(Error::Timeout)
            };
        }
    }
}

/// `raw` (0..=0xfff full scale) as millivolts of `vref_mv`, rounded down as Linux does.
pub const fn raw_to_millivolts(raw: u16, vref_mv: u32) -> u32 {
    vref_mv * raw as u32 / RESULT_MASK
}

/// Temperature-sensor voltage to millidegrees Celsius: T = 27 − (V − 0.706) / 0.001721
/// (Linux rp1-adc, DOC; the same sensor law as the RP2040). Rounded to nearest.
pub const fn millivolts_to_millicelsius(mv: u32) -> i32 {
    let num = (mv as i64 - 706) * 1_000_000;
    let half = if num < 0 { -1721 / 2 } else { 1721 / 2 };
    27_000 - ((num + half) / 1721) as i32
}

/// AINSEL as a type, for the embedded-hal 0.2 `OneShot` API (embedded-hal 1.0 has no ADC trait).
pub struct Input<const N: u8>;
/// The temperature sensor input.
pub type TemperatureSensor = Input<TEMPERATURE>;
impl<const N: u8> embedded_hal_0_2::adc::Channel<Adc> for Input<N> {
    type ID = u8;
    fn channel() -> u8 {
        const { assert!(N <= TEMPERATURE) };
        N
    }
}
impl<P: embedded_hal_0_2::adc::Channel<Adc, ID = u8>> embedded_hal_0_2::adc::OneShot<Adc, u16, P>
    for Adc
{
    type Error = Error;
    /// Blocking: the conversion is complete when this returns.
    fn read(&mut self, _input: &mut P) -> nb::Result<u16, Error> {
        Adc::read(self, P::channel()).map_err(nb::Error::Other)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offsets_match_the_pac() {
        let block = unsafe { &*(BASE as *const crate::pac::generated::adc::RegisterBlock) };
        assert_eq!(block.cs() as *const _ as usize, CS);
        assert_eq!(block.result() as *const _ as usize, RESULT);
        assert_eq!(block.inte() as *const _ as usize, INTE);
        assert_eq!(crate::pac::ADC::PTR as usize, BASE);
    }

    #[test]
    fn clk_adc_is_xosc_div_1() {
        let Kind::Linux(clk) = RP1_CLOCKS[id::ADC as usize].kind else {
            panic!()
        };
        assert_eq!(
            clk.choose(crate::clocks::XOSC_HZ, Round::Down),
            Some((0, 1, 50_000_000))
        );
    }

    #[test]
    fn conversions_follow_linux() {
        assert_eq!(raw_to_millivolts(0, VREF_MV), 0);
        assert_eq!(raw_to_millivolts(0xfff, VREF_MV), 3300);
        assert_eq!(raw_to_millivolts(876, VREF_MV), 705);
        // 0.706 V is 27 °C; 1.721 mV per degree, falling with temperature.
        assert_eq!(millivolts_to_millicelsius(706), 27_000);
        assert_eq!(millivolts_to_millicelsius(706 - 1721 / 100), 36_878);
        assert_eq!(millivolts_to_millicelsius(689), 36_878);
        assert_eq!(millivolts_to_millicelsius(723), 17_122);
        assert_eq!(millivolts_to_millicelsius(0), 437_227);
    }

    #[test]
    fn input_ids() {
        use embedded_hal_0_2::adc::Channel;
        assert_eq!(<Input<0> as Channel<Adc>>::channel(), 0);
        assert_eq!(<TemperatureSensor as Channel<Adc>>::channel(), 4);
    }
}
