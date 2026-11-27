//! RP1 ADC, with 12-bit one-shot conversions of four external inputs and the temperature sensor.
//!
//! | Instance | Synchronous waits | Immediate |
//! |---|---|---|
//! | ADC with a generated IRQ capability | [`Adc::blocking`] (IRQ52 + alarm + WFE), or explicit polling | [`Adc::try_start`] / [`Adc::try_sample`] |
//! | ADC without a same-core wait timer, or with `custom_adc_handler` | Explicit [`Adc::read_polling`] | [`Adc::try_start`] / [`Adc::try_sample`] |
//!
//! The CS sequence follows the Linux RP1 ADC hwmon driver (STATIC). Each conversion
//! uses the FIFO at threshold one with DMA and round-robin disabled. The functional
//! clock is clk_adc = XOSC / 1 = 50 MHz, the rate Linux asks for and the one the
//! SCMI table lists for ADC ([`crate::scmi::RP1_CLOCKS`]). With `adc` declared, SCMI
//! holds that clock and the overlay disables Linux's `rp1_adc`.
//!
//! External AIN routes depend on the board. The CM5 declaration provides the
//! internal temperature sensor. A board with verified routes can issue external
//! [`Input`] tokens.
use crate::{
    linux_blocks::Mmio,
    scmi::{Kind, RP1_CLOCKS, Registers, Round, id},
    timer::{self, Deadline, Duration, Timer},
};
use rp1_rt::interrupt::ThreadGuard;

mod irq_wait;
pub use irq_wait::{__on_interrupt, Blocking, Irq};

const BASE: usize = 0x400c_8000;
const CS: usize = BASE;
const FCS: usize = BASE + 0x08;
const FIFO: usize = BASE + 0x0c;
const INTE: usize = BASE + 0x18;
const INTF: usize = BASE + 0x1c;
const INTS: usize = BASE + 0x20;
const CLR: usize = 0x3000;

const CS_EN: u32 = 1 << 0;
const CS_TS_EN: u32 = 1 << 1;
const CS_START_ONCE: u32 = 1 << 2;
const CS_START_MANY: u32 = 1 << 3;
const CS_READY: u32 = 1 << 8;
const CS_ERR_STICKY: u32 = 1 << 10;
const CS_POWERED: u32 = CS_EN | CS_TS_EN;
const CS_IDLE_MASK: u32 = CS_READY | CS_START_ONCE | CS_START_MANY;
const FCS_EMPTY: u32 = 1 << 8;
const FCS_UNDER: u32 = 1 << 10;
const FCS_OVER: u32 = 1 << 11;
const FCS_FLAGS: u32 = FCS_UNDER | FCS_OVER;
const FCS_LEVEL: u32 = 0xf << 16;
const FCS_ONE_SAMPLE: u32 = (1 << 24) | (1 << 2) | 1; // THRESH=1, ERR, EN
const FIFO_ERR: u32 = 1 << 15;
const FIFO_DEPTH: usize = 16;
const RESULT_MASK: u32 = 0xfff;

/// RESETS CTRL0 bit 0 (HW: releasing it from Linux made the ADC convert). Its DONE bit stays 0
/// even under the stock firmware (HW), so it is not waited for.
const RESETS_CLR0: usize = 0x4001_4000 + CLR;
const RESET_BIT: u32 = 1;

/// Budget for the whole one-shot call, including peripheral access and IRQ service.
const TIMEOUT: Duration = Duration::from_ticks(1_000);

/// Full-scale reference of the rpi-cm5 board (`vref-supply = <&rp1_vdd_3v3>`, DOC stock DT).
pub const VREF_MV: u32 = 3300;
/// Internal temperature input. The sensor is powered during ADC setup.
pub const TEMPERATURE: TemperatureSensor = Input { _private: () };

/// Board-issued ADC route for external AIN 0–3, or the internal temperature input (4).
pub struct Input<const N: u8> {
    _private: (),
}
/// Legacy and native channel type for the internal temperature sensor.
pub type TemperatureSensor = Input<4>;

impl<const N: u8> Input<N> {
    /// # Safety
    /// Board or lab setup only. N must name an external AIN route owned by the caller.
    /// The board must keep the pin within the ADC electrical limits.
    #[doc(hidden)]
    pub const unsafe fn __new() -> Self {
        const { assert!(N < 4) };
        Self { _private: () }
    }
}

mod sealed {
    pub trait Channel {
        const SELECT: u8;
    }
}

/// A typed ADC channel. Pass [`TEMPERATURE`] or a borrow of a board-issued [`Input`].
pub trait Channel: sealed::Channel {}

impl<const N: u8> sealed::Channel for Input<N> {
    const SELECT: u8 = N;
}
impl<const N: u8> Channel for Input<N> {}
impl<C: Channel> sealed::Channel for &C {
    const SELECT: u8 = C::SELECT;
}
impl<C: Channel> Channel for &C {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    /// The clock tree changed or clk_adc could not be started.
    Precondition,
    /// The operation did not complete by the deadline. See [`Adc::is_recovering`]
    /// for the state after cleanup was attempted.
    Timeout,
    /// The deadline is outside the alarm's supported horizon.
    InvalidDeadline,
    /// A synchronous call was made outside thread mode.
    WrongContext,
    /// Interrupts are masked.
    Masked,
    /// The dedicated ADC interrupt is unavailable.
    IrqUnavailable,
    /// The wait alarm could not be disarmed or acknowledged.
    AlarmUnavailable,
    /// Another conversion owns the ADC.
    Busy,
    /// No conversion is pending for `try_sample`.
    NoConversion,
    /// The ADC flagged the conversion (FIFO.ERR or CS.ERR).
    Conversion,
    /// The ADC FIFO overflowed.
    Overrun,
    /// The one-shot FIFO contained an unexpected number of samples or underflowed.
    FifoState,
    /// Call `try_cancel` again to confirm READY and recover the FIFO.
    Recovering,
}

impl From<timer::Error> for Error {
    fn from(error: timer::Error) -> Self {
        match error {
            timer::Error::Timeout => Self::Timeout,
            timer::Error::InvalidDeadline => Self::InvalidDeadline,
            timer::Error::WrongContext => Self::WrongContext,
            timer::Error::Masked => Self::Masked,
            timer::Error::IrqUnavailable => Self::IrqUnavailable,
            timer::Error::AlarmUnavailable => Self::AlarmUnavailable,
        }
    }
}

/// Ownership of the ADC block and, when generated, its IRQ capability.
pub struct Parts<I = ()> {
    _irq: I,
    _not_sync: core::marker::PhantomData<core::cell::Cell<()>>,
}

/// ADC ownership without a managed IRQ.
pub type Token = Parts;

impl Token {
    /// # Safety
    /// Generated resources only: sole owner of ADC, its reset and its clock generator, with
    /// Linux's `rp1_adc` disabled and the SCMI server holding clk_adc.
    pub const unsafe fn new() -> Self {
        Self {
            _irq: (),
            _not_sync: core::marker::PhantomData,
        }
    }
}

/// A setup error retaining ADC ownership until recovery is confirmed.
pub type InitFailure<I> = crate::InitError<Parts<I>, Error, crate::Recovery<Parts<I>>>;

impl<I> Parts<I> {
    /// Starts clk_adc (XOSC / 1), releases reset, powers the temperature sensor,
    /// and confirms READY and a clean FIFO. The clock and reset stay configured
    /// if setup fails. [`InitFailure::try_recover`] checks whether a retry is safe.
    pub fn init(self, clocks: &crate::clocks::Clocks) -> Result<Adc<I>, InitFailure<I>> {
        let thread = ThreadGuard::new().map_err(timer::Error::from);
        if let Err(error) = thread {
            return Err(crate::InitError::new(error.into(), self));
        }
        let clk_hz = match start_clock(clocks) {
            Ok(hz) => hz,
            Err(error) => return Err(crate::InitError::new(error, self)),
        };

        let mut io = Mmio;
        let timer = Timer::new();
        let start = timer.now_32();
        io.write(RESETS_CLR0, RESET_BIT);
        io.write(INTE, 0);
        io.write(INTF, 0);
        io.write(CS, CS_POWERED | CS_ERR_STICKY);
        while !ready(io.read(CS)) {
            if (timer.now_32() - start).ticks() > TIMEOUT.ticks() {
                return Err(crate::InitError::recovering(Error::Timeout, self));
            }
        }
        if !clean(&mut io) {
            return Err(crate::InitError::recovering(Error::Recovering, self));
        }
        Ok(Adc {
            parts: self,
            clk_hz,
            state: State::Idle,
        })
    }
}

impl<I> InitFailure<I> {
    /// Try to return the parts from failed setup, retaining them if READY or FIFO
    /// cleanup cannot yet be confirmed.
    pub fn try_recover(self) -> Result<Parts<I>, Self> {
        self.recover(|| clean(&mut Mmio))
    }
}

/// The clock step of [`Parts::init`]; returns clk_adc.
fn start_clock(clocks: &crate::clocks::Clocks) -> Result<u32, Error> {
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
    Ok(hz)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    Idle,
    Converting(u8),
    Recovering,
}

/// A powered ADC doing one conversion at a time.
pub struct Adc<I = ()> {
    parts: Parts<I>,
    clk_hz: u32,
    state: State,
}

impl<I> Adc<I> {
    /// clk_adc as started by [`Parts::init`].
    pub const fn clock(&self) -> fugit::HertzU32 {
        fugit::HertzU32::from_raw(self.clk_hz)
    }

    /// Start one conversion without waiting. A pending conversion keeps its channel.
    pub fn try_start<C: Channel>(&mut self, _channel: C) -> Result<(), Error> {
        self.state.start(&mut Mmio, C::SELECT)
    }

    /// Try to take the pending result. `Ok(None)` means it is still converting.
    pub fn try_sample(&mut self) -> Result<Option<u16>, Error> {
        self.state.sample(&mut Mmio)
    }

    /// Whether cleanup is pending. Use [`Self::try_cancel`] before starting again.
    pub const fn is_recovering(&self) -> bool {
        matches!(self.state, State::Recovering)
    }

    /// Discard the current conversion. One-shot hardware may still finish
    /// after cancel. Retry this method if the ADC is recovering.
    pub fn try_cancel(&mut self) -> Result<(), Error> {
        self.state.cancel(&mut Mmio)
    }

    /// Confirm idle, power down and return the parts. A failed release keeps
    /// the driver, including its IRQ capability.
    pub fn try_free(mut self) -> Result<Parts<I>, crate::FreeError<Self, Error>> {
        if let Err(error) = self.state.idle().and_then(|()| self.try_cancel()) {
            return Err(crate::FreeError::new(self, error));
        }
        crate::write(CS, 0);
        Ok(self.parts)
    }

    /// Explicit thread-only polling with the default 1 ms timeout.
    pub fn read_polling(&mut self, channel: impl Channel) -> Result<u16, Error> {
        self.read_polling_until(channel, Deadline::after(TIMEOUT)?)
    }

    /// Explicit polling under an absolute deadline. Handler and masked calls fail.
    pub fn read_polling_until<C: Channel>(
        &mut self,
        _channel: C,
        end: Deadline,
    ) -> Result<u16, Error> {
        ThreadGuard::new().map_err(timer::Error::from)?;
        let timer = Timer::new();
        let start = timer.now();
        end.check(start)?;
        let budget = (end.instant().ticks() - start.ticks()) as u32;
        let start_lo = start.ticks() as u32;
        // The admission check bounds the horizon to i32::MAX ticks.
        let elapsed = || timer.now_32().ticks().wrapping_sub(start_lo) >= budget;
        let mut io = Mmio;
        self.state.prepare(&mut io)?;
        if elapsed() {
            return Err(Error::Timeout);
        }
        self.state.launch(&mut io, C::SELECT);
        loop {
            if let Some(sample) = self.state.sample(&mut io)? {
                return Ok(sample);
            }
            if elapsed() {
                if let Some(sample) = self.state.sample(&mut io)? {
                    return Ok(sample);
                }
                let _ = self.state.cancel(&mut io);
                return Err(Error::Timeout);
            }
        }
    }

    /// Explicitly poll one temperature conversion, in millidegrees Celsius.
    pub fn read_temperature_polling(&mut self) -> Result<i32, Error> {
        self.read_polling(TEMPERATURE).map(temperature)
    }
}

impl State {
    fn idle(self) -> Result<(), Error> {
        match self {
            Self::Idle => Ok(()),
            Self::Converting(_) => Err(Error::Busy),
            Self::Recovering => Err(Error::Recovering),
        }
    }

    fn prepare(&mut self, io: &mut impl Registers) -> Result<(), Error> {
        self.idle()?;
        let cs = io.read(CS);
        if !ready(cs) {
            *self = Self::Recovering;
            return Err(Error::Recovering);
        }
        let fcs = io.read(FCS);
        if fcs & FCS_OVER != 0 {
            return self.fail(io, Error::Overrun);
        }
        if fcs & (FCS_LEVEL | FCS_UNDER) != 0 || fcs & FCS_EMPTY == 0 {
            // Retire an old FIFO entry before writing START_ONCE for this channel.
            self.cancel(io)?;
        }
        Ok(())
    }

    fn launch(&mut self, io: &mut impl Registers, channel: u8) {
        *self = Self::Converting(channel);
        // The full known configuration clears START_MANY and RROBIN. ERR_STICKY is W1C.
        io.write(
            CS,
            CS_POWERED | CS_ERR_STICKY | CS_START_ONCE | (u32::from(channel) << 12),
        );
    }

    fn start(&mut self, io: &mut impl Registers, channel: u8) -> Result<(), Error> {
        self.prepare(io)?;
        self.launch(io, channel);
        Ok(())
    }

    fn sample(&mut self, io: &mut impl Registers) -> Result<Option<u16>, Error> {
        match self {
            Self::Idle => return Err(Error::NoConversion),
            Self::Recovering => return Err(Error::Recovering),
            Self::Converting(_) => {}
        }
        let fcs = io.read(FCS);
        if fcs & FCS_OVER != 0 {
            return self.fail(io, Error::Overrun);
        }
        if fcs & FCS_UNDER != 0 {
            return self.fail(io, Error::FifoState);
        }
        if fcs & FCS_EMPTY != 0 {
            return Ok(None);
        }
        if fcs & FCS_LEVEL != 1 << 16 {
            return self.fail(io, Error::FifoState);
        }
        let value = io.read(FIFO);
        // The FIFO was empty at launch and tags this result with its error bit.
        // It can wake the waiter before READY, which gates the next start.
        if value & FIFO_ERR != 0 {
            return self.fail(io, Error::Conversion);
        }
        *self = Self::Idle;
        Ok(Some((value & RESULT_MASK) as u16))
    }

    fn fail<T>(&mut self, io: &mut impl Registers, error: Error) -> Result<T, Error> {
        let _ = self.cancel(io);
        Err(error)
    }

    fn cancel(&mut self, io: &mut impl Registers) -> Result<(), Error> {
        *self = Self::Recovering;
        if !clean(io) {
            return Err(Error::Recovering);
        }
        *self = Self::Idle;
        Ok(())
    }
}

fn ready(cs: u32) -> bool {
    cs & (CS_EN | CS_IDLE_MASK) == CS_EN | CS_READY
}

fn idle_clean(cs: u32, fcs: u32) -> bool {
    ready(cs) && cs & CS_ERR_STICKY == 0 && fcs & (FCS_LEVEL | FCS_FLAGS | FCS_EMPTY) == FCS_EMPTY
}

#[rp1_rt::cold]
fn clean(io: &mut impl Registers) -> bool {
    io.write_relaxed(INTE, 0);
    io.write_relaxed(INTF, 0);
    // START_ONCE can't be retracted. Leave the FIFO receiving until the in-flight
    // conversion has finished, so a late sample cannot become the next result.
    if !ready(io.read(CS)) {
        return false;
    }
    for _ in 0..FIFO_DEPTH {
        if io.read(FCS) & FCS_EMPTY != 0 {
            break;
        }
        io.read(FIFO);
    }
    io.write_relaxed(FCS, FCS_ONE_SAMPLE | FCS_FLAGS);
    io.write_relaxed(CS, CS_POWERED | CS_ERR_STICKY);
    let fcs = io.read(FCS);
    idle_clean(io.read(CS), fcs)
}

fn temperature(raw: u16) -> i32 {
    millivolts_to_millicelsius(raw_to_millivolts(raw, VREF_MV))
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

#[cfg(feature = "eh02-adc")]
impl<I, const N: u8> embedded_hal_0_2::adc::Channel<Adc<I>> for Input<N> {
    type ID = u8;

    fn channel() -> u8 {
        N
    }
}

#[cfg(feature = "eh02-adc")]
impl<I, const N: u8> embedded_hal_0_2::adc::OneShot<Adc<I>, u16, Input<N>> for Adc<I> {
    type Error = Error;

    /// Starts on the first call and returns `WouldBlock` until the result is ready.
    /// A different channel while a conversion is pending returns [`Error::Busy`].
    fn read(&mut self, input: &mut Input<N>) -> nb::Result<u16, Error> {
        match self.state {
            State::Converting(channel) if channel == N => {
                self.try_sample()?.ok_or(nb::Error::WouldBlock)
            }
            _ => {
                self.try_start(&*input)?;
                Err(nb::Error::WouldBlock)
            }
        }
    }
}

#[cfg(test)]
mod tests;
