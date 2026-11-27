//! RP1 PWM0/1: four independent, trailing-edge PWM channels per owned block.
//!
//! Register evidence, scoped PWM0 HW results and extended acceptance plan: `docs/pwm.md`. No FIFO, DMA,
//! interrupts or channel binding. The functional clock is XOSC / 1 = 50 MHz, held by SCMI
//! for a declared PWM block. All four channels, the reset and the clock belong to one core;
//! [`Channels`] must not be distributed between cores. GPIO routes are checked by [`PinRoute`].
use crate::{
    clocks::{Clocks, XOSC_HZ},
    fugit::{HertzU32, NanosDurationU64},
    gpio::{Alternate, Pin},
    linux_blocks::{Hardware, Mmio},
    scmi::{Generator, Kind, PhysicalState, RP1_CLOCKS, Round, id},
};
use embedded_hal::pwm::{ErrorKind, ErrorType, SetDutyCycle};

const GLOBAL_CTRL: usize = 0;
const FIFO_CTRL: usize = 4;
const CHAN_CTRL: usize = 0x14;
const CHAN_RANGE: usize = 0x18;
const CHAN_PHASE: usize = 0x1c;
const CHAN_DUTY: usize = 0x20;
const INTE: usize = 0x58;
const INTF: usize = 0x5c;
const STRIDE: usize = 0x10;
const SET: usize = 0x2000;
const CLR: usize = 0x3000;
const UPDATE: u32 = 1 << 31;
const TRAILING_EDGE: u32 = (1 << 8) | 1;
/// CHANn_CTRL MODE[2:0] (regdb pwm0.toml); 0 = generate 0.
const MODE_MASK: u32 = 0x7;
const RESETS_CLR1: usize = 0x4001_7004;
const RESETS_DONE1: usize = 0x4001_401c;
const TIMEOUT_US: u32 = 10_000;
const DEFAULT_PERIOD: u32 = u16::MAX as u32;
const NS_PER_TICK: u64 = 1_000_000_000 / XOSC_HZ as u64;
const MAX_PERIOD_NS: u64 = u32::MAX as u64 * NS_PER_TICK;

const fn base(n: u8) -> usize {
    0x4009_8000 + n as usize * 0x4000
}
fn generator(n: u8) -> Generator {
    let Kind::Linux(clk) = RP1_CLOCKS[id::PWM0 as usize + n as usize].kind else {
        unreachable!()
    };
    clk
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    /// The requested frequency, period or duty is outside the supported range.
    Invalid,
    /// Hardware still reports the channel enabled.
    Enabled,
    /// Stale `Clocks`, or clk_pwm is no longer enabled at 50 MHz.
    Precondition,
    /// The block's reset DONE did not assert within 10 ms (`Token::init` only).
    Timeout,
}
impl embedded_hal::pwm::Error for Error {
    fn kind(&self) -> ErrorKind {
        ErrorKind::Other
    }
}

mod sealed {
    pub trait Pin {}
    impl<const P: u8> Pin for crate::gpio::Pin<P> {}
}
/// Bank-0 output route of PWM block N, channel C, generated from rp1-regdb.
/// PWM1 has no bank-0 route; it may be used without a pin for internal reload diagnostics.
pub trait PinRoute<const N: u8, const C: u8>: sealed::Pin {
    const FUNC: u8;
}

/// Sole ownership of a PWM block (not one channel), issued by generated resources.
#[derive(Debug)]
pub struct Token<const N: u8> {
    _private: core::marker::PhantomData<core::cell::Cell<()>>,
}
/// `?` on a failed [`Token::init`].
impl<R> From<crate::InitError<R, Error>> for Error {
    fn from(e: crate::InitError<R, Error>) -> Self {
        e.error
    }
}
impl<const N: u8> Token<N> {
    /// # Safety
    /// Generated resources only: one core owns all four channels, reset and clock of PWM N.
    /// Linux's PWM node and conflicting consumers must be disabled; SCMI must hold clk_pwmN.
    pub const unsafe fn new() -> Self {
        const { assert!(N < 2) };
        Self {
            _private: core::marker::PhantomData,
        }
    }
    /// Starts XOSC / 1, releases only this block's reset, disables DMA/interrupts and all
    /// channels. Periods start at 65535 ticks, duty at zero. GPIO bank0 and the raw timer must
    /// already run (cold bring-up). On failure the token comes back in
    /// [`InitError`](crate::InitError), no pin has been routed, and the block may be partly
    /// initialized.
    pub fn init(self, clocks: &Clocks) -> Result<Channels<N>, crate::InitError<Token<N>, Error>> {
        if !clocks.still_current() {
            return Err(crate::InitError::new(Error::Precondition, self));
        }
        match start(&mut Mmio, N) {
            Ok(()) => Ok(Channels {
                channel0: Channel::new(),
                channel1: Channel::new(),
                channel2: Channel::new(),
                channel3: Channel::new(),
            }),
            Err(error) => Err(crate::InitError::new(error, self)),
        }
    }
}
/// Four channels from one declared block. Restore any moved channels before [`Self::free`]
/// or [`Self::abort`].
/// Channels are exclusive resources. They can move into a core-local interrupt slot,
/// and cannot be shared or sent through the inter-core link.
///
/// ```compile_fail,E0308
/// use rp1_hal::pwm::{Channel, Channels, Running};
/// fn restore(channels: Channels<0>, channel0: Channel<0, 0, Running>) -> Channels<0> {
///     Channels { channel0, ..channels }
/// }
/// ```
#[derive(Debug)]
pub struct Channels<const N: u8> {
    pub channel0: Channel<N, 0>,
    pub channel1: Channel<N, 1>,
    pub channel2: Channel<N, 2>,
    pub channel3: Channel<N, 3>,
}
impl<const N: u8> Channels<N> {
    pub const fn clock(&self) -> fugit::HertzU32 {
        fugit::HertzU32::from_raw(XOSC_HZ)
    }
    /// Disables all channels and returns the token. Clock/reset remain running.
    ///
    /// [`Error::Precondition`]: clk_pwm is no longer enabled at 50 MHz, so the disable could
    /// not latch; nothing was written and the channels come back in
    /// [`FreeError`](crate::FreeError) with the error, to retry
    /// once the clock runs again or to [`Self::abort`].
    ///
    /// ```no_run
    /// # use rp1_hal::pwm::{Channels, Token};
    /// fn release(channels: Channels<0>) -> Token<0> {
    ///     channels.free().unwrap_or_else(|e| e.into_driver().abort())
    /// }
    /// ```
    pub fn free(self) -> Result<Token<N>, crate::FreeError<Self, Error>> {
        match release_block(&mut Mmio, N) {
            Ok(()) => Ok(Token {
                _private: core::marker::PhantomData,
            }),
            Err(error) => Err(crate::FreeError {
                driver: self,
                error,
            }),
        }
    }
    /// [`Self::free`] without the clock check: the disable is written, but with clk_pwm
    /// stopped or at another rate it may not latch, and the outputs may keep their last
    /// level or waveform (HW: not verified). [`Token::init`] starts the clock and disables all four
    /// channels again.
    pub fn abort(self) -> Token<N> {
        stop_block(&mut Mmio, N);
        Token {
            _private: core::marker::PhantomData,
        }
    }
}
/// A channel that may be configured, routed or enabled.
#[derive(Debug)]
pub struct Disabled;

/// A channel whose period and pin routing are fixed.
#[derive(Debug)]
pub struct Running;

/// A failed state transition, retaining the original owner.
pub type TransitionError<D> = crate::FreeError<D, Error>;

/// Native compare ticks. The channel checks the value against its programmed range.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DutyTicks(u32);

impl DutyTicks {
    pub const fn new(ticks: u32) -> Self {
        Self(ticks)
    }

    pub const fn ticks(self) -> u32 {
        self.0
    }
}

/// A trailing-edge PWM period request. Rounds to the nearest cycle length,
/// choosing the longer cycle on a tie. Requests outside the supported interval fail.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Config(Period);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Period {
    Frequency(HertzU32),
    Nanoseconds(NanosDurationU64),
    Ticks(u32),
}

impl Config {
    pub const fn from_frequency(rate: HertzU32) -> Self {
        Self(Period::Frequency(rate))
    }

    pub const fn from_period(period: NanosDurationU64) -> Self {
        Self(Period::Nanoseconds(period))
    }

    /// An exact native period, primarily for register-level diagnostics.
    pub const fn from_ticks(ticks: u32) -> Self {
        Self(Period::Ticks(ticks))
    }

    fn resolve(self) -> Result<Actual, Error> {
        let ticks = match self.0 {
            Period::Frequency(rate) if (1..=XOSC_HZ).contains(&rate.raw()) => {
                round_nearest(u64::from(XOSC_HZ), u64::from(rate.raw()))
            }
            Period::Nanoseconds(period)
                if (NS_PER_TICK..=MAX_PERIOD_NS).contains(&period.ticks()) =>
            {
                round_nearest(period.ticks(), NS_PER_TICK)
            }
            Period::Ticks(ticks) if ticks != 0 => u64::from(ticks),
            _ => return Err(Error::Invalid),
        };
        let range = u32::try_from(ticks).map_err(|_| Error::Invalid)?;
        Ok(Actual { range })
    }
}

// All callers bound the numerator and denominator before rounding.
const fn round_nearest(numerator: u64, denominator: u64) -> u64 {
    (numerator + denominator / 2) / denominator
}

/// Effective values calculated from the programmed registers, using the
/// trailing-edge RANGE convention. These are calculated, not measured.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Actual {
    range: u32,
}

impl Actual {
    pub const fn clock(self) -> HertzU32 {
        HertzU32::from_raw(XOSC_HZ)
    }

    /// Denominator of the exact programmed frequency, `clock() / cycle_ticks()`.
    /// The RANGE endpoint convention is HW unverified.
    pub const fn cycle_ticks(self) -> u64 {
        self.range as u64
    }

    pub const fn range_ticks(self) -> u32 {
        self.range
    }

    pub const fn period(self) -> NanosDurationU64 {
        NanosDurationU64::from_ticks(self.cycle_ticks() * NS_PER_TICK)
    }

    /// Whole-Hz display, rounded to nearest with ties up. May be zero for long periods.
    pub const fn frequency_hz(self) -> HertzU32 {
        HertzU32::from_raw(round_nearest(XOSC_HZ as u64, self.cycle_ticks()) as u32)
    }
}

/// One channel with a 32-bit period and compare. Dropping either state leaves hardware
/// as configured; explicitly disable a running channel or output before dropping it.
#[derive(Debug)]
pub struct Channel<const N: u8, const C: u8, State = Disabled> {
    actual: Actual,
    _state: core::marker::PhantomData<State>,
    _core: core::marker::PhantomData<core::cell::Cell<()>>,
}

impl<const N: u8, const C: u8> Channel<N, C, Disabled> {
    fn new() -> Self {
        Self {
            actual: Actual {
                range: DEFAULT_PERIOD,
            },
            _state: core::marker::PhantomData,
            _core: core::marker::PhantomData,
        }
    }

    /// Programs a period and resets duty to zero. On error neither the cached configuration
    /// nor any register is changed. Configure the channel before routing to defer pin drive.
    ///
    /// ```compile_fail,E0599
    /// use rp1_hal::pwm::{Channel, Config, Running};
    /// fn reconfigure(channel: &mut Channel<0, 0, Running>) {
    ///     channel.configure(Config::from_ticks(50_000)).unwrap();
    /// }
    /// ```
    pub fn configure(&mut self, config: Config) -> Result<Actual, Error> {
        let actual = configure(&mut Mmio, N, C, config)?;
        self.actual = actual;
        Ok(actual)
    }

    /// Transitions to running after checking clk_pwm.
    pub fn enable(self) -> Result<Channel<N, C, Running>, TransitionError<Self>> {
        self.transition(&mut Mmio, true)
    }

    /// Routes and retains an owned pin while the channel is disabled.
    ///
    /// ```compile_fail,E0599
    /// use rp1_hal::{gpio::Pin, pwm::{Channel, Running}};
    /// fn route(channel: Channel<0, 0, Running>, pin: Pin<12>) {
    ///     let _ = channel.output(pin);
    /// }
    /// ```
    pub fn output<const P: u8>(self, pin: Pin<P>) -> Output<N, C, P>
    where
        Pin<P>: PinRoute<N, C>,
    {
        Output {
            channel: self,
            pin: pin.into_alternate(<Pin<P> as PinRoute<N, C>>::FUNC, 0x46, 0x63),
        }
    }
}

impl<const N: u8, const C: u8> Channel<N, C, Running> {
    /// Requests disable after checking clk_pwm. The pin transition is asynchronous.
    pub fn disable(self) -> Result<Channel<N, C>, TransitionError<Self>> {
        self.transition(&mut Mmio, false)
    }

    /// Issues the stop without checking clk_pwm. With clk_pwm stopped, the
    /// disable may fail to latch and the output may retain its level (HW unverified).
    pub fn abort(self) -> Channel<N, C> {
        self.abort_with(&mut Mmio)
    }
}

impl<const N: u8, const C: u8, State> Channel<N, C, State> {
    pub const fn actual(&self) -> Actual {
        self.actual
    }

    /// Checks that clk_pwm is enabled with the expected 50 MHz clock configuration.
    /// Use this in a slow supervisor loop outside the control ISR.
    pub fn check_clock(&self) -> Result<(), Error> {
        clock_current(&mut Mmio, N)
    }

    /// Native 32-bit compare, 0..=range. Zero is low and range is continuously high
    /// in normal polarity. DUTY has its own strobe and, per the datasheet (Table 26),
    /// takes effect at the next overflow of a running channel (HW: not verified).
    /// Each call writes block-global SET_UPDATE as in the Linux driver. Calls
    /// across channels can latch on different periods. Synchronized commits
    /// are deferred to the M2 Group API.
    ///
    /// This path relies on the PWM clock held by the declaration and checked during
    /// setup. It performs one range check followed by two ordered MMIO writes.
    /// Success means the writes were issued. A stopped clock can leave the output
    /// at its last level and keep the writes from latching (HW unverified).
    #[inline(always)]
    pub fn try_set_duty_ticks(&mut self, duty: DutyTicks) -> Result<(), Error> {
        set_duty(&mut Mmio, N, C, self.actual.range, duty.ticks())
    }

    fn transition<Next>(
        self,
        io: &mut impl Hardware,
        enabled: bool,
    ) -> Result<Channel<N, C, Next>, TransitionError<Self>> {
        match set_enabled(io, N, C, enabled) {
            Ok(()) => Ok(Channel {
                actual: self.actual,
                _state: core::marker::PhantomData,
                _core: core::marker::PhantomData,
            }),
            Err(error) => Err(TransitionError::new(self, error)),
        }
    }

    fn into_disabled(self) -> Channel<N, C> {
        Channel {
            actual: self.actual,
            _state: core::marker::PhantomData,
            _core: core::marker::PhantomData,
        }
    }

    fn abort_with(self, io: &mut impl Hardware) -> Channel<N, C> {
        stop_channel(io, N, C);
        self.into_disabled()
    }
}

/// A channel and its owned pin, implementing embedded-hal 1.0 PWM. Dereferencing
/// exposes the channel's configuration and duty operations for its current state.
///
/// ```compile_fail,E0599
/// use rp1_hal::pwm::{Config, Output, Running};
/// fn reconfigure(output: &mut Output<0, 0, 12, Running>) {
///     output.configure(Config::from_ticks(50_000)).unwrap();
/// }
/// ```
///
/// ```compile_fail,E0599
/// use rp1_hal::pwm::{Output, Running};
/// fn change_pad(output: &mut Output<0, 0, 12, Running>) {
///     let _ = output.pin_mut();
/// }
/// ```
pub struct Output<const N: u8, const C: u8, const P: u8, State = Disabled> {
    channel: Channel<N, C, State>,
    pin: Pin<P, Alternate>,
}
impl<const N: u8, const C: u8, const P: u8> Output<N, C, P, Disabled> {
    /// Borrows the routed pin to configure its pad before enabling PWM.
    pub fn pin_mut(&mut self) -> &mut Pin<P, Alternate> {
        &mut self.pin
    }

    pub fn enable(self) -> Result<Output<N, C, P, Running>, TransitionError<Self>> {
        self.map_channel(Channel::enable)
    }

    /// Disables the channel and clears its MODE (generate 0), as Linux's `pwm-rp1` free does,
    /// then returns channel and unconfigured pin. The pad stays muxed to the channel, which
    /// then outputs a constant low (DOC/STATIC; HW: not verified). [`Self::enable`] restores the
    /// trailing-edge mode. If the clock check fails, no registers are written and
    /// the error retains the output.
    pub fn try_free(self) -> Result<(Channel<N, C>, Pin<P>), crate::FreeError<Self, Error>> {
        match release_channel(&mut Mmio, N, C) {
            Ok(()) => Ok((self.channel, self.pin.release())),
            Err(error) => Err(crate::FreeError::new(self, error)),
        }
    }
}
impl<const N: u8, const C: u8, const P: u8> Output<N, C, P, Running> {
    /// Requests disable after checking clk_pwm. A failed check writes nothing
    /// and returns the running output in the error.
    pub fn disable(self) -> Result<Output<N, C, P>, TransitionError<Self>> {
        self.map_channel(Channel::disable)
    }
}
impl<const N: u8, const C: u8, const P: u8, State> Output<N, C, P, State> {
    /// Issues the stop without checking clk_pwm, in either state.
    /// The returned pad stays muxed to PWM, which may retain its last level if
    /// clk_pwm has stopped (HW unverified). [`Pin::into_input`] takes it off PWM.
    ///
    /// ```no_run
    /// use rp1_hal::pwm::{Channel, Output, Running};
    /// fn stop(output: Output<0, 0, 12, Running>) -> Channel<0, 0> {
    ///     let (channel, pin) = match output.disable() {
    ///         Ok(output) => output.try_free().unwrap_or_else(|e| e.into_driver().abort()),
    ///         Err(e) => e.into_driver().abort(),
    ///     };
    ///     let _input = pin.into_input();
    ///     channel
    /// }
    /// ```
    pub fn abort(self) -> (Channel<N, C>, Pin<P>) {
        self.abort_with(&mut Mmio)
    }

    fn abort_with(self, io: &mut impl Hardware) -> (Channel<N, C>, Pin<P>) {
        (self.channel.abort_with(io), self.pin.release())
    }

    fn map_channel<Next>(
        self,
        transition: impl FnOnce(
            Channel<N, C, State>,
        )
            -> Result<Channel<N, C, Next>, TransitionError<Channel<N, C, State>>>,
    ) -> Result<Output<N, C, P, Next>, TransitionError<Self>> {
        let Self { channel, pin } = self;
        match transition(channel) {
            Ok(channel) => Ok(Output { channel, pin }),
            Err(e) => {
                let (channel, error) = e.into_parts();
                Err(TransitionError::new(Output { channel, pin }, error))
            }
        }
    }
}
impl<const N: u8, const C: u8, const P: u8, State> core::ops::Deref for Output<N, C, P, State> {
    type Target = Channel<N, C, State>;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.channel
    }
}
impl<const N: u8, const C: u8, const P: u8, State> core::ops::DerefMut for Output<N, C, P, State> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.channel
    }
}
impl<const N: u8, const C: u8, const P: u8, State> ErrorType for Output<N, C, P, State> {
    type Error = Error;
}
impl<const N: u8, const C: u8, const P: u8, State> SetDutyCycle for Output<N, C, P, State> {
    /// Normalized duty: 65535 means 100%, independent of the 32-bit period. Rounded down to
    /// native ticks, so small periods quantize the requested fraction. Does not enable output.
    fn max_duty_cycle(&self) -> u16 {
        u16::MAX
    }
    fn set_duty_cycle(&mut self, duty: u16) -> Result<(), Error> {
        self.check_clock()?;
        let ticks = scale_duty(self.actual().range, duty);
        self.try_set_duty_ticks(DutyTicks::new(ticks))
    }
}
fn scale_duty(period: u32, duty: u16) -> u32 {
    (u64::from(period) * u64::from(duty) / u64::from(u16::MAX)) as u32
}

fn wait(io: &mut impl Hardware, address: usize, mask: u32, value: u32) -> Result<(), Error> {
    let start = io.now_us();
    loop {
        if io.read(address) & mask == value {
            return Ok(());
        }
        if io.now_us().wrapping_sub(start) >= TIMEOUT_US {
            return if io.read(address) & mask == value {
                Ok(())
            } else {
                Err(Error::Timeout)
            };
        }
    }
}
fn clock_current(io: &mut impl Hardware, n: u8) -> Result<(), Error> {
    if generator(n).state(io)
        == Some(PhysicalState {
            rate_hz: XOSC_HZ,
            enabled: true,
        })
    {
        Ok(())
    } else {
        Err(Error::Precondition)
    }
}
/// Request that the channel registers latch (SET_UPDATE, self-clearing). Like Linux's
/// `pwm-rp1` (`rp1_pwm_apply_config`), nothing waits for the bit to clear: if the latch
/// happens at the end of the running period, a wait would have to last up to
/// u32::MAX × 20 ns (about 86 s). A second request while one is pending is what Linux does
/// as well. When the new values take effect is not verified on hardware.
#[inline(always)]
fn update(io: &mut impl Hardware, n: u8) {
    io.write(base(n) + GLOBAL_CTRL + SET, UPDATE);
}
fn set_enabled(io: &mut impl Hardware, n: u8, c: u8, enabled: bool) -> Result<(), Error> {
    clock_current(io, n)?;
    if enabled {
        // Output::try_free leaves MODE = 0; restore the mode Linux's request() writes.
        io.write(base(n) + CHAN_CTRL + c as usize * STRIDE, TRAILING_EDGE);
    }
    io.write(
        base(n) + GLOBAL_CTRL + if enabled { SET } else { CLR },
        1 << c,
    );
    update(io, n);
    Ok(())
}
/// Linux `rp1_pwm_free`: clear MODE (0 = generate 0) and latch. This driver also clears the
/// channel's enable bit.
fn release_channel(io: &mut impl Hardware, n: u8, c: u8) -> Result<(), Error> {
    clock_current(io, n)?;
    stop_channel(io, n, c);
    Ok(())
}
fn stop_channel(io: &mut impl Hardware, n: u8, c: u8) {
    io.write(base(n) + GLOBAL_CTRL + CLR, 1 << c);
    io.write(base(n) + CHAN_CTRL + c as usize * STRIDE + CLR, MODE_MASK);
    update(io, n);
}
/// [`Channels::free`]: nothing is written unless the disable can latch.
fn release_block(io: &mut impl Hardware, n: u8) -> Result<(), Error> {
    clock_current(io, n)?;
    stop_block(io, n);
    Ok(())
}
fn stop_block(io: &mut impl Hardware, n: u8) {
    io.write(base(n) + GLOBAL_CTRL + CLR, 0xf);
    update(io, n);
}
fn configure(io: &mut impl Hardware, n: u8, c: u8, config: Config) -> Result<Actual, Error> {
    let actual = config.resolve()?;
    clock_current(io, n)?;
    if io.read(base(n) + GLOBAL_CTRL) & (1 << c) != 0 {
        return Err(Error::Enabled);
    }
    io.write(base(n) + CHAN_DUTY + c as usize * STRIDE, 0);
    io.write(base(n) + CHAN_RANGE + c as usize * STRIDE, actual.range);
    update(io, n);
    Ok(actual)
}

#[inline(always)]
fn set_duty(io: &mut impl Hardware, n: u8, c: u8, range: u32, duty: u32) -> Result<(), Error> {
    if duty > range {
        return Err(Error::Invalid);
    }
    io.write(base(n) + CHAN_DUTY + c as usize * STRIDE, duty);
    update(io, n);
    Ok(())
}
fn start(io: &mut impl Hardware, n: u8) -> Result<(), Error> {
    let clk = generator(n);
    let (aux, div, _) = clk
        .choose(XOSC_HZ, Round::Down)
        .ok_or(Error::Precondition)?;
    if clk.state(io).is_none_or(|s| s.rate_hz != XOSC_HZ) {
        clk.program(io, aux, div);
    }
    clk.set_enabled(io, true);
    clock_current(io, n)?;
    let reset = 1 << (4 + n);
    io.write(RESETS_CLR1, reset);
    wait(io, RESETS_DONE1, reset, reset)?;
    io.write(base(n) + GLOBAL_CTRL + CLR, 0xf);
    update(io, n);
    // DREQ_EN off (FLUSH is not asserted; the FIFO is unused: no channel sets USEFIFO below).
    io.write(base(n) + FIFO_CTRL, 0);
    io.write(base(n) + INTE, 0);
    io.write(base(n) + INTF, 0);
    for c in 0..4 {
        let offset = c * STRIDE;
        io.write(base(n) + CHAN_CTRL + offset, TRAILING_EDGE);
        io.write(base(n) + CHAN_PHASE + offset, 0);
        io.write(base(n) + CHAN_DUTY + offset, 0);
        io.write(base(n) + CHAN_RANGE + offset, DEFAULT_PERIOD);
    }
    update(io, n);
    Ok(())
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use crate::scmi::Registers;
    use std::{collections::BTreeMap, vec::Vec};

    struct Fake {
        regs: BTreeMap<usize, u32>,
        writes: Vec<(usize, u32)>,
        now: u32,
        stuck_update: bool,
        stuck_clock: bool,
    }
    impl Fake {
        fn new(n: u8) -> Self {
            Self {
                regs: [(RESETS_DONE1, 1 << (4 + n))].into(),
                writes: Vec::new(),
                now: u32::MAX - 5000,
                stuck_update: false,
                stuck_clock: false,
            }
        }
        fn running(n: u8) -> Self {
            let mut f = Self::new(n);
            start(&mut f, n).unwrap();
            f.writes.clear();
            f
        }
    }
    impl Registers for Fake {
        fn read(&mut self, address: usize) -> u32 {
            self.regs.get(&address).copied().unwrap_or(0)
        }
        fn write(&mut self, address: usize, value: u32) {
            self.writes.push((address, value));
            if self.stuck_clock && [generator(0).ctrl, generator(1).ctrl].contains(&address) {
                return;
            }
            for n in 0..2 {
                if (base(n) + SET..base(n) + CLR).contains(&address) {
                    let reg = self.regs.entry(address - SET).or_default();
                    *reg |= value;
                    // SET_UPDATE self-clears at once unless the test holds it pending.
                    if address == base(n) + SET && !self.stuck_update {
                        *reg &= !UPDATE;
                    }
                    return;
                }
                if (base(n) + CLR..base(n) + 0x4000).contains(&address) {
                    *self.regs.entry(address - CLR).or_default() &= !value;
                    return;
                }
            }
            self.regs.insert(address, value);
        }
    }
    impl Hardware for Fake {
        fn now_us(&mut self) -> u32 {
            self.now = self.now.wrapping_add(1000);
            self.now
        }
    }
    #[test]
    fn register_offsets_match_both_pac_blocks() {
        macro_rules! check {
            ($module:ident) => {{
                assert_eq!(
                    pac_offsets!($module: global_ctrl, fifo_ctrl, inte, intf),
                    [GLOBAL_CTRL, FIFO_CTRL, INTE, INTF]
                );
                let channels = [
                    pac_offsets!($module: chan0_ctrl, chan0_range, chan0_phase, chan0_duty),
                    pac_offsets!($module: chan1_ctrl, chan1_range, chan1_phase, chan1_duty),
                    pac_offsets!($module: chan2_ctrl, chan2_range, chan2_phase, chan2_duty),
                    pac_offsets!($module: chan3_ctrl, chan3_range, chan3_phase, chan3_duty),
                ];
                for (c, registers) in channels.into_iter().enumerate() {
                    assert_eq!(
                        registers.map(|offset| offset - c * STRIDE),
                        [CHAN_CTRL, CHAN_RANGE, CHAN_PHASE, CHAN_DUTY]
                    );
                }
            }};
        }
        check!(pwm0);
        check!(pwm1);
        assert_eq!(crate::pac::PWM0::PTR as usize, base(0));
        assert_eq!(crate::pac::PWM1::PTR as usize, base(1));
    }
    #[test]
    fn starts_only_owned_clock_reset_and_four_disabled_channels() {
        for n in 0..2 {
            let mut f = Fake::new(n);
            f.regs.insert(base(n), 0xf); // Reinitialization also disables a running block.
            start(&mut f, n).unwrap();
            assert_eq!(
                generator(n).state(&mut f),
                Some(PhysicalState {
                    rate_hz: XOSC_HZ,
                    enabled: true
                })
            );
            assert!(f.writes.contains(&(RESETS_CLR1, 1 << (4 + n))));
            assert_eq!(f.regs[&base(n)], 0);
            assert_eq!(f.regs[&(base(n) + INTE)], 0);
            assert_eq!(f.regs[&(base(n) + INTF)], 0);
            assert_eq!(f.regs[&(base(n) + FIFO_CTRL)], 0);
            for c in 0..4 {
                assert_eq!(f.regs[&(base(n) + CHAN_CTRL + c * STRIDE)], TRAILING_EDGE);
                assert_eq!(f.regs[&(base(n) + CHAN_RANGE + c * STRIDE)], DEFAULT_PERIOD);
                assert_eq!(f.regs[&(base(n) + CHAN_DUTY + c * STRIDE)], 0);
                assert_eq!(f.regs[&(base(n) + CHAN_PHASE + c * STRIDE)], 0);
            }
            assert!(f.writes.iter().all(|&(a, _)| a == RESETS_CLR1
                || (generator(n).ctrl..generator(n).ctrl + 12).contains(&a)
                || (base(n)..base(n) + 0x4000).contains(&a)));
        }
    }
    #[test]
    fn alias_updates_preserve_siblings_and_period_requires_disabled_channel() {
        for n in 0..2 {
            let mut f = Fake::running(n);
            for c in 0..4 {
                configure(&mut f, n, c, Config::from_ticks(u32::MAX)).unwrap();
                set_duty(&mut f, n, c, u32::MAX, u32::MAX).unwrap();
                set_enabled(&mut f, n, c, true).unwrap();
                assert_eq!(f.regs[&base(n)], (1 << (c + 1)) - 1);
                let writes = f.writes.len();
                assert_eq!(
                    configure(&mut f, n, c, Config::from_ticks(1)),
                    Err(Error::Enabled)
                );
                assert_eq!(f.writes.len(), writes);
            }
            set_enabled(&mut f, n, 1, false).unwrap();
            assert_eq!(f.regs[&base(n)], 0xd);
            assert!(!f.writes.iter().any(|&(a, _)| a == base(n)));
        }
    }
    #[test]
    fn typed_configuration_rounding_and_full_range() {
        let ns = |n| Config::from_period(NanosDurationU64::from_ticks(n));
        for (config, range) in [
            (Config::from_frequency(HertzU32::from_raw(XOSC_HZ)), 1),
            (Config::from_frequency(HertzU32::from_raw(1_000)), 50_000),
            (Config::from_frequency(HertzU32::from_raw(20_000_000)), 3),
            (Config::from_frequency(HertzU32::from_raw(20_000_001)), 2),
            (Config::from_frequency(HertzU32::from_raw(19_999_999)), 3),
            (Config::from_frequency(HertzU32::from_raw(1)), 50_000_000),
            (ns(20), 1),
            (ns(29), 1),
            (ns(30), 2),
            (ns(31), 2),
            (ns(MAX_PERIOD_NS - 10), u32::MAX),
            (ns(MAX_PERIOD_NS - 11), u32::MAX - 1),
            (ns(MAX_PERIOD_NS), u32::MAX),
            (Config::from_ticks(u32::MAX), u32::MAX),
        ] {
            let mut f = Fake::running(0);
            let actual = configure(&mut f, 0, 0, config).unwrap();
            assert_eq!(actual.clock().raw(), XOSC_HZ);
            assert_eq!(actual.range_ticks(), range);
            assert_eq!(actual.cycle_ticks(), u64::from(range));
            assert_eq!(actual.period().ticks(), u64::from(range) * 20);
            assert_eq!(
                f.writes,
                [
                    (base(0) + CHAN_DUTY, 0),
                    (base(0) + CHAN_RANGE, range),
                    (base(0) + SET, UPDATE)
                ]
            );
        }
        assert_eq!(
            Config::from_ticks(256)
                .resolve()
                .unwrap()
                .frequency_hz()
                .raw(),
            195_313
        );
        assert_eq!(ns(MAX_PERIOD_NS).resolve().unwrap().frequency_hz().raw(), 0);
    }

    #[test]
    fn invalid_typed_requests_leave_hardware_intact() {
        let mut f = Fake::running(0);
        for rate in [0, XOSC_HZ + 1, u32::MAX] {
            assert_eq!(
                configure(
                    &mut f,
                    0,
                    1,
                    Config::from_frequency(HertzU32::from_raw(rate))
                ),
                Err(Error::Invalid)
            );
        }
        for period in [0, 1, 19, MAX_PERIOD_NS + 1, u64::MAX] {
            let config = Config::from_period(NanosDurationU64::from_ticks(period));
            assert_eq!(configure(&mut f, 0, 1, config), Err(Error::Invalid));
        }
        assert!(f.writes.is_empty());
        assert_eq!(f.regs[&(base(0) + CHAN_RANGE + STRIDE)], DEFAULT_PERIOD);
    }

    #[test]
    fn native_update_uses_only_two_writes_after_range_check() {
        let mut f = Fake::new(1);
        for duty in [0, 3, 7] {
            f.writes.clear();
            set_duty(&mut f, 1, 2, 7, duty).unwrap();
            assert_eq!(
                f.writes,
                [
                    (base(1) + CHAN_DUTY + 2 * STRIDE, duty),
                    (base(1) + SET, UPDATE)
                ]
            );
        }
        f.writes.clear();
        assert_eq!(set_duty(&mut f, 1, 2, 7, 8), Err(Error::Invalid));
        assert!(f.writes.is_empty());
    }

    #[test]
    fn invalid_settings_and_stopped_clock_write_nothing() {
        let mut f = Fake::running(0);
        assert_eq!(
            configure(&mut f, 0, 0, Config::from_ticks(0)),
            Err(Error::Invalid)
        );
        assert_eq!(set_duty(&mut f, 0, 0, 100, 101), Err(Error::Invalid));
        assert!(f.writes.is_empty());
        f.regs.insert(generator(0).ctrl, 2 << 5);
        assert_eq!(
            configure(&mut f, 0, 0, Config::from_ticks(100)),
            Err(Error::Precondition)
        );
        assert_eq!(set_enabled(&mut f, 0, 0, false), Err(Error::Precondition));
        assert_eq!(set_enabled(&mut f, 0, 0, true), Err(Error::Precondition));
        assert!(f.writes.is_empty());
    }

    #[test]
    fn failed_disable_retains_owner_and_abort_issues_stop_regardless_of_clock() {
        for (index, divider) in [None, Some(2)].into_iter().enumerate() {
            let mut f = Fake::running(0);
            let channel: Channel<0, 0, Running> = Channel::new().transition(&mut f, true).unwrap();
            set_enabled(&mut f, 0, 1, true).unwrap();
            match divider {
                Some(divider) => f.regs.insert(generator(0).ctrl + 4, divider),
                None => f.regs.insert(generator(0).ctrl, 2 << 5),
            };
            f.writes.clear();
            let error = channel.transition::<Disabled>(&mut f, false).unwrap_err();
            assert_eq!(*error.error(), Error::Precondition);
            assert!(f.writes.is_empty());

            let running = error.into_driver();
            let channel = if index == 0 {
                running.abort_with(&mut f)
            } else {
                let output = Output {
                    channel: running,
                    // Only the type and pin return are exercised on the host.
                    pin: unsafe { Pin::<12>::new() }.assume(),
                };
                let (channel, _pin): (Channel<0, 0>, Pin<12>) = output.abort_with(&mut f);
                channel
            };
            assert_eq!(channel.actual().range_ticks(), DEFAULT_PERIOD);
            assert_eq!(f.regs[&base(0)] & 0xf, 0b10);
            assert_eq!(
                f.writes,
                [
                    (base(0) + CLR, 1),
                    (base(0) + CHAN_CTRL + CLR, MODE_MASK),
                    (base(0) + SET, UPDATE),
                ]
            );
        }
    }
    #[test]
    fn reset_and_clock_failures_are_bounded() {
        let mut f = Fake::new(1);
        f.regs.insert(RESETS_DONE1, 0);
        assert_eq!(start(&mut f, 1), Err(Error::Timeout));
        assert!(
            !f.writes
                .iter()
                .any(|&(a, _)| (base(1)..base(1) + 0x4000).contains(&a))
        );
        let mut f = Fake::new(0);
        f.stuck_clock = true;
        assert_eq!(start(&mut f, 0), Err(Error::Precondition));
        assert!(!f.writes.iter().any(|&(a, _)| a == RESETS_CLR1));
    }
    /// Review minor 2: SET_UPDATE may stay set until the running period ends (up to ~86 s for
    /// u32::MAX ticks). Like Linux, setters do not wait for it, so a pending update is not
    /// an error, and every setter still writes its value and requests another update.
    #[test]
    fn pending_update_is_not_waited_for() {
        let mut f = Fake::running(0);
        f.stuck_update = true;
        f.regs.insert(base(0), UPDATE);
        configure(&mut f, 0, 0, Config::from_ticks(100)).unwrap();
        set_duty(&mut f, 0, 0, 100, 50).unwrap();
        set_enabled(&mut f, 0, 0, true).unwrap();
        assert_eq!(f.regs[&(base(0) + CHAN_RANGE)], 100);
        assert_eq!(f.regs[&(base(0) + CHAN_DUTY)], 50);
        assert_eq!(f.regs[&base(0)], UPDATE | 1);
        let updates = f.writes.iter().filter(|&&w| w == (base(0) + SET, UPDATE));
        assert_eq!(updates.count(), 3);
    }
    /// Every configure error is returned before a register write, so the
    /// cached period (assigned only on Ok) always matches RANGE.
    #[test]
    fn configure_errors_precede_writes() {
        let mut f = Fake::running(0);
        set_enabled(&mut f, 0, 1, true).unwrap();
        f.writes.clear();
        assert_eq!(
            configure(&mut f, 0, 1, Config::from_ticks(7)),
            Err(Error::Enabled)
        );
        assert_eq!(
            configure(&mut f, 0, 0, Config::from_ticks(0)),
            Err(Error::Invalid)
        );
        f.regs.insert(generator(0).ctrl, 2 << 5);
        assert_eq!(
            configure(&mut f, 0, 0, Config::from_ticks(7)),
            Err(Error::Precondition)
        );
        assert!(f.writes.is_empty());
        assert_eq!(f.regs[&(base(0) + CHAN_RANGE + STRIDE)], DEFAULT_PERIOD);
    }
    /// Review minor 3: freeing an output disables the channel and clears MODE like Linux's
    /// rp1_pwm_free, without touching sibling channels; enabling again restores the mode.
    #[test]
    fn release_clears_mode_and_enable_restores_it() {
        for n in 0..2 {
            let mut f = Fake::running(n);
            for c in 0..4 {
                set_enabled(&mut f, n, c, true).unwrap();
            }
            release_channel(&mut f, n, 2).unwrap();
            assert_eq!(f.regs[&base(n)] & 0xf, 0xb);
            for c in 0..4 {
                let mode = f.regs[&(base(n) + CHAN_CTRL + c * STRIDE)];
                assert_eq!(mode, if c == 2 { 1 << 8 } else { TRAILING_EDGE });
            }
            assert_eq!(f.writes.last(), Some(&(base(n) + SET, UPDATE)));
            set_enabled(&mut f, n, 2, true).unwrap();
            assert_eq!(f.regs[&(base(n) + CHAN_CTRL + 2 * STRIDE)], TRAILING_EDGE);
            assert_eq!(f.regs[&base(n)] & 0xf, 0xf);
            f.regs.insert(generator(n).ctrl, 2 << 5);
            f.writes.clear();
            assert_eq!(release_channel(&mut f, n, 2), Err(Error::Precondition));
            assert!(f.writes.is_empty());
        }
    }
    /// With the clock stopped `free` writes nothing and reports it (the caller keeps the
    /// channels); `abort` writes the disable anyway.
    #[test]
    fn block_release_needs_the_clock_and_abort_does_not() {
        let stopped = [(base(1) + GLOBAL_CTRL + CLR, 0xf), (base(1) + SET, UPDATE)];
        let mut f = Fake::running(1);
        for c in 0..4 {
            set_enabled(&mut f, 1, c, true).unwrap();
        }
        f.writes.clear();
        f.regs.insert(generator(1).ctrl, 2 << 5);
        assert_eq!(release_block(&mut f, 1), Err(Error::Precondition));
        assert!(f.writes.is_empty());
        stop_block(&mut f, 1);
        assert_eq!(f.writes, stopped);
        stop_channel(&mut f, 1, 2);
        assert_eq!(f.regs[&(base(1) + CHAN_CTRL + 2 * STRIDE)], 1 << 8);

        let mut f = Fake::running(1);
        set_enabled(&mut f, 1, 0, true).unwrap();
        f.writes.clear();
        assert_eq!(release_block(&mut f, 1), Ok(()));
        assert_eq!(
            (f.regs[&base(1)] & 0xf, f.writes.as_slice()),
            (0, &stopped[..])
        );
    }
    #[test]
    fn normalized_duty_includes_both_endpoints_without_overflow() {
        for period in [1, 2, 65535, 50_000_000, u32::MAX] {
            assert_eq!(scale_duty(period, 0), 0);
            assert_eq!(scale_duty(period, u16::MAX), period);
            let mut last = 0;
            for duty in 0..=u16::MAX {
                let got = scale_duty(period, duty);
                assert!(got >= last && got <= period);
                last = got;
            }
        }
    }
    #[test]
    fn public_traits_and_bank0_routes() {
        fn pwm<T: SetDutyCycle<Error = Error>>() {}
        pwm::<Output<0, 0, 12>>();
        pwm::<Output<0, 0, 12, Running>>();
        assert_eq!(<Pin<12> as PinRoute<0, 0>>::FUNC, 0);
        assert_eq!(<Pin<13> as PinRoute<0, 1>>::FUNC, 0);
        assert_eq!(<Pin<14> as PinRoute<0, 2>>::FUNC, 0);
        assert_eq!(<Pin<15> as PinRoute<0, 3>>::FUNC, 0);
        assert_eq!(<Pin<18> as PinRoute<0, 2>>::FUNC, 3);
        assert_eq!(<Pin<19> as PinRoute<0, 3>>::FUNC, 3);
    }
}
