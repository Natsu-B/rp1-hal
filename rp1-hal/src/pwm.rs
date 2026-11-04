//! RP1 PWM0/1: four independent, trailing-edge PWM channels per owned block.
//!
//! STATIC register evidence and HW 未検証 acceptance plan: `docs/pwm.md`. No FIFO, DMA,
//! interrupts or channel binding. The functional clock is XOSC / 1 = 50 MHz, held by SCMI
//! for a declared PWM block. All four channels, the reset and the clock belong to one core;
//! [`Channels`] must not be distributed between cores. GPIO routes are checked by [`PinRoute`].
use crate::{
    clocks::{Clocks, XOSC_HZ},
    gpio::{Alternate, Pin},
    linux_blocks::{Hardware, Mmio},
    scmi::{Generator, Kind, PhysicalState, RP1_CLOCKS, Registers, Round, id},
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
const RESETS_CLR1: usize = 0x4001_7004;
const RESETS_DONE1: usize = 0x4001_401c;
const TIMEOUT_US: u32 = 10_000;
const DEFAULT_PERIOD: u32 = u16::MAX as u32;

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
pub enum Error {
    /// Zero period, or duty greater than period.
    Invalid,
    /// The channel must be disabled before changing its period.
    Enabled,
    /// Stale `Clocks`, or clk_pwm is no longer enabled at 50 MHz.
    Precondition,
    /// Reset DONE or SET_UPDATE did not complete within 10 ms.
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
    _private: (),
}
/// Initialization failed; ownership is retained for a retry. No pin has been routed.
#[derive(Debug)]
pub struct InitError<const N: u8> {
    pub error: Error,
    pub token: Token<N>,
}
impl<const N: u8> Token<N> {
    /// # Safety
    /// Generated resources only: one core owns all four channels, reset and clock of PWM N.
    /// Linux's PWM node and conflicting consumers must be disabled; SCMI must hold clk_pwmN.
    pub const unsafe fn new() -> Self {
        const { assert!(N < 2) };
        Self { _private: () }
    }
    /// Starts XOSC / 1, releases only this block's reset, disables DMA/interrupts and all
    /// channels. Periods start at 65535 ticks, duty at zero. GPIO bank0 and the raw timer must
    /// already run (cold bring-up). On failure the block may be partly initialized.
    pub fn init(self, clocks: &Clocks) -> Result<Channels<N>, InitError<N>> {
        if !clocks.still_current() {
            return Err(InitError {
                error: Error::Precondition,
                token: self,
            });
        }
        match start(&mut Mmio, N) {
            Ok(()) => Ok(Channels {
                channel0: Channel::new(),
                channel1: Channel::new(),
                channel2: Channel::new(),
                channel3: Channel::new(),
            }),
            Err(error) => Err(InitError { error, token: self }),
        }
    }
}
/// Four channels from one declared block. Restore any moved channels before [`Self::free`].
/// Channels are deliberately neither Send nor Sync: a block cannot be split between cores.
#[derive(Debug)]
pub struct Channels<const N: u8> {
    pub channel0: Channel<N, 0>,
    pub channel1: Channel<N, 1>,
    pub channel2: Channel<N, 2>,
    pub channel3: Channel<N, 3>,
}
impl<const N: u8> Channels<N> {
    pub const fn clock_hz(&self) -> u32 {
        XOSC_HZ
    }
    /// Disables all channels and returns the token. Clock/reset remain running. On failure
    /// ownership is consumed; no second token is issued for an unknown hardware state.
    pub fn free(self) -> Result<Token<N>, Error> {
        let io = &mut Mmio;
        ready(io, N)?;
        io.write(base(N) + GLOBAL_CTRL + CLR, 0xf);
        update(io, N)?;
        Ok(Token { _private: () })
    }
}
/// One channel, initially disabled, with a 32-bit period and compare. Dropping this value
/// leaves hardware as configured; use [`Self::disable`] before abandoning a running output.
#[derive(Debug)]
pub struct Channel<const N: u8, const C: u8> {
    period: u32,
    _core: core::marker::PhantomData<*mut ()>,
}
impl<const N: u8, const C: u8> Channel<N, C> {
    fn new() -> Self {
        Self {
            period: DEFAULT_PERIOD,
            _core: core::marker::PhantomData,
        }
    }
    pub const fn period_ticks(&self) -> u32 {
        self.period
    }
    /// Period in 20 ns ticks. Only while disabled; resets duty to zero. A period of P gives
    /// frequency 50 MHz / P (STATIC, Linux RP1 PWM range convention; HW 未検証).
    pub fn set_period_ticks(&mut self, period: u32) -> Result<(), Error> {
        set_period(&mut Mmio, N, C, period)?;
        self.period = period;
        Ok(())
    }
    /// Native 32-bit compare, 0..=period: zero is low, period is continuously high in normal
    /// polarity. Latched with SET_UPDATE like the Linux driver; the new compare may appear only
    /// after the current period ends, not necessarily before return.
    pub fn set_duty_ticks(&mut self, duty: u32) -> Result<(), Error> {
        set_duty(&mut Mmio, N, C, self.period, duty)
    }
    pub fn enable(&mut self) -> Result<(), Error> {
        set_enabled(&mut Mmio, N, C, true)
    }
    pub fn disable(&mut self) -> Result<(), Error> {
        set_enabled(&mut Mmio, N, C, false)
    }
    /// Routes and retains an owned pin. If already enabled, the waveform appears immediately.
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
/// A channel and its pin, implementing embedded-hal 1.0 PWM. [`core::ops::DerefMut`] exposes
/// the channel's period, native duty and enable/disable operations.
pub struct Output<const N: u8, const C: u8, const P: u8> {
    channel: Channel<N, C>,
    pin: Pin<P, Alternate>,
}
impl<const N: u8, const C: u8, const P: u8> Output<N, C, P> {
    /// Disables the channel, then returns channel and unconfigured pin. Pad/mux stay as they
    /// are (the PWM channel is disabled). On failure ownership is consumed, as in Channels::free.
    pub fn free(mut self) -> Result<(Channel<N, C>, Pin<P>), Error> {
        self.channel.disable()?;
        Ok((self.channel, self.pin.release()))
    }
}
impl<const N: u8, const C: u8, const P: u8> core::ops::Deref for Output<N, C, P> {
    type Target = Channel<N, C>;
    fn deref(&self) -> &Self::Target {
        &self.channel
    }
}
impl<const N: u8, const C: u8, const P: u8> core::ops::DerefMut for Output<N, C, P> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.channel
    }
}
impl<const N: u8, const C: u8, const P: u8> ErrorType for Output<N, C, P> {
    type Error = Error;
}
impl<const N: u8, const C: u8, const P: u8> SetDutyCycle for Output<N, C, P> {
    /// Normalized duty: 65535 means 100%, independent of the 32-bit period. Rounded down to
    /// native ticks, so small periods quantize the requested fraction. Does not enable output.
    fn max_duty_cycle(&self) -> u16 {
        u16::MAX
    }
    fn set_duty_cycle(&mut self, duty: u16) -> Result<(), Error> {
        self.channel
            .set_duty_ticks(scale_duty(self.channel.period, duty))
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
fn ready(io: &mut impl Hardware, n: u8) -> Result<(), Error> {
    clock_current(io, n)?;
    wait(io, base(n) + GLOBAL_CTRL, UPDATE, 0)
}
fn update(io: &mut impl Hardware, n: u8) -> Result<(), Error> {
    io.write(base(n) + GLOBAL_CTRL + SET, UPDATE);
    wait(io, base(n) + GLOBAL_CTRL, UPDATE, 0)
}
fn set_enabled(io: &mut impl Hardware, n: u8, c: u8, enabled: bool) -> Result<(), Error> {
    ready(io, n)?;
    io.write(
        base(n) + GLOBAL_CTRL + if enabled { SET } else { CLR },
        1 << c,
    );
    update(io, n)
}
fn set_period(io: &mut impl Hardware, n: u8, c: u8, period: u32) -> Result<(), Error> {
    if period == 0 {
        return Err(Error::Invalid);
    }
    ready(io, n)?;
    if io.read(base(n) + GLOBAL_CTRL) & (1 << c) != 0 {
        return Err(Error::Enabled);
    }
    io.write(base(n) + CHAN_DUTY + c as usize * STRIDE, 0);
    io.write(base(n) + CHAN_RANGE + c as usize * STRIDE, period);
    update(io, n)
}
fn set_duty(io: &mut impl Hardware, n: u8, c: u8, period: u32, duty: u32) -> Result<(), Error> {
    if duty > period {
        return Err(Error::Invalid);
    }
    ready(io, n)?;
    io.write(base(n) + CHAN_DUTY + c as usize * STRIDE, duty);
    update(io, n)
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
    ready(io, n)?;
    io.write(base(n) + GLOBAL_CTRL + CLR, 0xf);
    update(io, n)?;
    io.write(base(n) + FIFO_CTRL, 0); // DREQ off; no FIFO channel is selected below.
    io.write(base(n) + INTE, 0);
    io.write(base(n) + INTF, 0);
    for c in 0..4 {
        let offset = c * STRIDE;
        io.write(base(n) + CHAN_CTRL + offset, TRAILING_EDGE);
        io.write(base(n) + CHAN_PHASE + offset, 0);
        io.write(base(n) + CHAN_DUTY + offset, 0);
        io.write(base(n) + CHAN_RANGE + offset, DEFAULT_PERIOD);
    }
    update(io, n)
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
        update_reads: usize,
    }
    impl Fake {
        fn new(n: u8) -> Self {
            Self {
                regs: [(RESETS_DONE1, 1 << (4 + n))].into(),
                writes: Vec::new(),
                now: u32::MAX - 5000,
                stuck_update: false,
                stuck_clock: false,
                update_reads: 0,
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
            let mut value = self.regs.get(&address).copied().unwrap_or(0);
            if [base(0), base(1)].contains(&address) && value & UPDATE != 0 {
                self.update_reads += 1;
                if !self.stuck_update && self.update_reads.is_multiple_of(2) {
                    value &= !UPDATE;
                    self.regs.insert(address, value);
                }
            }
            value
        }
        fn write(&mut self, address: usize, value: u32) {
            self.writes.push((address, value));
            if self.stuck_clock && [generator(0).ctrl, generator(1).ctrl].contains(&address) {
                return;
            }
            for n in 0..2 {
                if address == base(n) + SET {
                    *self.regs.entry(base(n)).or_default() |= value;
                    return;
                }
                if address == base(n) + CLR {
                    *self.regs.entry(base(n)).or_default() &= !value;
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
            ($n:expr, $module:ident) => {{
                let b =
                    unsafe { &*(base($n) as *const crate::pac::generated::$module::RegisterBlock) };
                let offsets = [
                    b.global_ctrl() as *const _ as usize,
                    b.fifo_ctrl() as *const _ as usize,
                    b.inte() as *const _ as usize,
                    b.intf() as *const _ as usize,
                ];
                assert_eq!(
                    offsets.map(|a| a - base($n)),
                    [GLOBAL_CTRL, FIFO_CTRL, INTE, INTF]
                );
                let channels = [
                    [
                        b.chan0_ctrl() as *const _ as usize,
                        b.chan0_range() as *const _ as usize,
                        b.chan0_phase() as *const _ as usize,
                        b.chan0_duty() as *const _ as usize,
                    ],
                    [
                        b.chan1_ctrl() as *const _ as usize,
                        b.chan1_range() as *const _ as usize,
                        b.chan1_phase() as *const _ as usize,
                        b.chan1_duty() as *const _ as usize,
                    ],
                    [
                        b.chan2_ctrl() as *const _ as usize,
                        b.chan2_range() as *const _ as usize,
                        b.chan2_phase() as *const _ as usize,
                        b.chan2_duty() as *const _ as usize,
                    ],
                    [
                        b.chan3_ctrl() as *const _ as usize,
                        b.chan3_range() as *const _ as usize,
                        b.chan3_phase() as *const _ as usize,
                        b.chan3_duty() as *const _ as usize,
                    ],
                ];
                for (c, registers) in channels.into_iter().enumerate() {
                    assert_eq!(
                        registers.map(|a| a - base($n) - c * STRIDE),
                        [CHAN_CTRL, CHAN_RANGE, CHAN_PHASE, CHAN_DUTY]
                    );
                }
            }};
        }
        check!(0, pwm0);
        check!(1, pwm1);
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
                set_period(&mut f, n, c, u32::MAX).unwrap();
                set_duty(&mut f, n, c, u32::MAX, u32::MAX).unwrap();
                set_enabled(&mut f, n, c, true).unwrap();
                assert_eq!(f.regs[&base(n)], (1 << (c + 1)) - 1);
                let writes = f.writes.len();
                assert_eq!(set_period(&mut f, n, c, 1), Err(Error::Enabled));
                assert_eq!(f.writes.len(), writes);
            }
            set_enabled(&mut f, n, 1, false).unwrap();
            assert_eq!(f.regs[&base(n)], 0xd);
            assert!(!f.writes.iter().any(|&(a, _)| a == base(n)));
        }
    }
    #[test]
    fn invalid_settings_and_stopped_clock_write_nothing() {
        let mut f = Fake::running(0);
        assert_eq!(set_period(&mut f, 0, 0, 0), Err(Error::Invalid));
        assert_eq!(set_duty(&mut f, 0, 0, 100, 101), Err(Error::Invalid));
        assert!(f.writes.is_empty());
        f.regs.insert(generator(0).ctrl, 2 << 5);
        assert_eq!(set_period(&mut f, 0, 0, 100), Err(Error::Precondition));
        assert_eq!(set_duty(&mut f, 0, 0, 100, 50), Err(Error::Precondition));
        assert_eq!(set_enabled(&mut f, 0, 0, true), Err(Error::Precondition));
        assert!(f.writes.is_empty());
    }
    #[test]
    fn reset_update_and_clock_failures_are_bounded() {
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
        let mut f = Fake::running(0);
        f.stuck_update = true;
        assert_eq!(set_enabled(&mut f, 0, 0, true), Err(Error::Timeout));
        f.writes.clear();
        assert_eq!(set_duty(&mut f, 0, 0, 100, 50), Err(Error::Timeout));
        assert!(f.writes.is_empty());
        assert!(f.update_reads < 40);
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
        assert_eq!(<Pin<12> as PinRoute<0, 0>>::FUNC, 0);
        assert_eq!(<Pin<13> as PinRoute<0, 1>>::FUNC, 0);
        assert_eq!(<Pin<14> as PinRoute<0, 2>>::FUNC, 0);
        assert_eq!(<Pin<15> as PinRoute<0, 3>>::FUNC, 0);
        assert_eq!(<Pin<18> as PinRoute<0, 2>>::FUNC, 3);
        assert_eq!(<Pin<19> as PinRoute<0, 3>>::FUNC, 3);
    }
}
