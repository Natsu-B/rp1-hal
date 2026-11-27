//! The 1 MHz timer, independent one-shot alarms, and thread-mode delays.

use crate::{linux_blocks::Mmio, read, scmi::Registers, write};
use fugit::TimerInstantU32;
use rp1_rt::interrupt::ThreadGuard;

mod wait;
pub use rp1_rt::interrupt::ContextError;
pub use wait::{Delay, WaitConfig, WaitContext, WaitTimer};
#[cfg(test)]
pub(crate) use wait::{WaitAlarm, wait as test_wait};

pub type Instant = fugit::TimerInstantU64<1_000_000>;
pub type Duration = fugit::MicrosDurationU32;
pub type Instant32 = TimerInstantU32<1_000_000>;

const MAX_ARM: u32 = i32::MAX as u32;
const ALARM0: usize = 0x400a_c010;
const ARMED: usize = 0x400a_c020;
const RAWH: usize = 0x400a_c024;
const RAWL: usize = 0x400a_c028;
const INTR: usize = 0x400a_c034;
const INTE_SET: usize = 0x400a_e038;
const INTE_CLEAR: usize = 0x400a_f038;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum Error {
    Timeout,
    InvalidDeadline,
    WrongContext,
    Masked,
    IrqUnavailable,
    /// The dedicated alarm could not be disarmed or acknowledged.
    AlarmUnavailable,
}

impl From<ContextError> for Error {
    fn from(error: ContextError) -> Self {
        match error {
            ContextError::Masked => Self::Masked,
            ContextError::IrqUnavailable => Self::IrqUnavailable,
            _ => Self::WrongContext,
        }
    }
}

/// An absolute end time, checked against the full counter when used.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Deadline(Instant);

impl Deadline {
    /// Builds a timeout from the current timer tick. The deadline is reached between
    /// `timeout - 1 µs` and `timeout` after the counter read. Use [`WaitTimer::delay`]
    /// when the duration is a lower bound.
    pub fn after(timeout: Duration) -> Result<Self, Error> {
        Self::after_ticks(Timer::new().now(), timeout.ticks())
    }

    /// Wraps an instant. The consuming operation checks its horizon.
    pub const fn at(instant: Instant) -> Self {
        Self(instant)
    }

    pub const fn instant(self) -> Instant {
        self.0
    }

    pub(crate) fn after_ticks(now: Instant, timeout: u32) -> Result<Self, Error> {
        if timeout == 0 {
            return Err(Error::Timeout);
        }
        if timeout > MAX_ARM {
            return Err(Error::InvalidDeadline);
        }
        Ok(Self(Instant::from_ticks(
            now.ticks()
                .checked_add(u64::from(timeout))
                .ok_or(Error::InvalidDeadline)?,
        )))
    }

    pub(crate) fn check(self, now: Instant) -> Result<(), Error> {
        match self.0.ticks().checked_sub(now.ticks()) {
            None | Some(0) => Err(Error::Timeout),
            Some(delta) if delta > u64::from(MAX_ARM) => Err(Error::InvalidDeadline),
            Some(_) => Ok(()),
        }
    }
}

impl From<Instant> for Deadline {
    fn from(instant: Instant) -> Self {
        Self::at(instant)
    }
}

pub struct Timer;
impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}
impl Timer {
    pub const fn new() -> Self {
        Self
    }

    /// Reads the full 64-bit timer at 1 MHz.
    pub fn now(&self) -> Instant {
        read_instant(&mut Mmio)
    }

    /// Reads the low word in one MMIO access, for use with [`Alarm::try_arm`].
    pub fn now_32(&self) -> Instant32 {
        Instant32::from_ticks(read(RAWL))
    }

    /// Busy-waits in thread mode for at least `duration` and about 1 µs more,
    /// plus preemption. A handler or any nonzero PRIMASK, FAULTMASK or BASEPRI is rejected.
    pub fn delay_polling(&self, duration: Duration) -> Result<(), Error> {
        ThreadGuard::new()?;
        wait_at_least(|| self.now().ticks(), u64::from(duration.ticks()));
        Ok(())
    }
}

pub(crate) fn read_instant(io: &mut impl Registers) -> Instant {
    loop {
        let hi = io.read(RAWH);
        let lo = io.read(RAWL);
        if hi == io.read(RAWH) {
            return Instant::from_ticks((u64::from(hi) << 32) | u64::from(lo));
        }
    }
}

/// A counter read just before a tick can make `us` steps almost 1 µs short:
/// wait for `us + 1` steps instead.
fn wait_at_least(mut now: impl FnMut() -> u64, us: u64) {
    if us == 0 {
        return;
    }
    let start = now();
    while now().wrapping_sub(start) <= us {
        core::hint::spin_loop();
    }
}

/// An independently owned low-word hardware comparison.
pub struct Alarm<const N: u8> {
    _private: core::marker::PhantomData<core::cell::Cell<()>>,
}
impl<const N: u8> Alarm<N> {
    /// # Safety
    /// Exactly one instance for the declared owner, with N < 4.
    pub const unsafe fn new() -> Self {
        assert!(N < 4);
        Self {
            _private: core::marker::PhantomData,
        }
    }

    /// Arms an absolute instant. A low-word instant outside the next half-cycle
    /// returns [`Error::Timeout`]. Use [`Instant`] or [`Deadline`] across wraps.
    /// Acknowledge any previous firing before rearming or a missed instant may report `Ok`.
    pub fn try_arm(&mut self, instant: impl AlarmInstant) -> Result<(), Error> {
        try_arm(&mut Mmio, N, instant)
    }

    /// Disarm this comparison. Interrupt enable and any latched firing are unchanged.
    pub fn cancel(&mut self) {
        write(ARMED, 1 << N);
    }

    /// Whether the alarm has fired, with or without its interrupt enabled.
    pub fn try_expired(&self) -> bool {
        read(INTR) & (1 << N) != 0
    }

    /// Clears the fired flag, which also deasserts the alarm's interrupt.
    pub fn clear_interrupt(&mut self) {
        write(INTR, 1 << N);
    }

    /// Unmasks this alarm's interrupt. The SET alias leaves the other alarms' enables
    /// (including the other core's) alone. The NVIC line is `irq.TIMER_N`.
    pub fn enable_interrupt(&mut self) {
        write(INTE_SET, 1 << N);
    }

    pub fn disable_interrupt(&mut self) {
        write(INTE_CLEAR, 1 << N);
    }
}

/// An instant accepted by [`Alarm::try_arm`]. [`Instant32`] uses low-word half-range
/// ordering. [`Instant`] and [`Deadline`] check the full counter and reject a horizon
/// beyond `i32::MAX` µs with [`Error::InvalidDeadline`].
pub trait AlarmInstant: alarm_instant::Sealed {}

mod alarm_instant {
    use super::*;

    pub trait Sealed {
        fn checked_low_word(self, io: &mut impl Registers) -> Result<u32, Error>;
    }

    impl Sealed for Instant32 {
        fn checked_low_word(self, io: &mut impl Registers) -> Result<u32, Error> {
            let instant = self.ticks();
            if !(1..=MAX_ARM).contains(&instant.wrapping_sub(io.read(RAWL))) {
                return Err(Error::Timeout);
            }
            Ok(instant)
        }
    }

    impl Sealed for Instant {
        fn checked_low_word(self, io: &mut impl Registers) -> Result<u32, Error> {
            Deadline::at(self).checked_low_word(io)
        }
    }

    impl Sealed for Deadline {
        fn checked_low_word(self, io: &mut impl Registers) -> Result<u32, Error> {
            self.check(read_instant(io))?;
            Ok(self.instant().ticks() as u32)
        }
    }
}

impl AlarmInstant for Instant32 {}
impl AlarmInstant for Instant {}
impl AlarmInstant for Deadline {}

fn try_arm(io: &mut impl Registers, n: u8, instant: impl AlarmInstant) -> Result<(), Error> {
    let instant = instant.checked_low_word(io)?;
    io.write(ALARM0 + usize::from(n) * 4, instant);
    let remaining = instant.wrapping_sub(io.read(RAWL));
    if (remaining == 0 || remaining > MAX_ARM) && io.read(INTR) & (1 << n) == 0 {
        // The instant can pass while the write is in flight. Unless it already fired,
        // disarm rather than wait a full 32-bit wrap (~71 min).
        io.write(ARMED, 1 << n);
        return Err(Error::Timeout);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::cell::Cell;

    #[test]
    fn deadlines_use_full_width_and_the_low_word_half_range() {
        let start = Instant::from_ticks(u32::MAX as u64 - 1);
        let end = Deadline::after_ticks(start, 3).unwrap();
        assert_eq!(end.instant().ticks(), 1u64 << 32 | 1);
        assert_eq!(end.check(Instant::from_ticks(1u64 << 32)), Ok(()));
        assert_eq!(end.check(end.instant()), Err(Error::Timeout));
        assert_eq!(
            end.check(Instant::from_ticks(start.ticks() + (1u64 << 32))),
            Err(Error::Timeout),
        );
        assert!(Deadline::after_ticks(start, MAX_ARM).is_ok());
        assert_eq!(
            Deadline::after_ticks(start, MAX_ARM + 1),
            Err(Error::InvalidDeadline),
        );
        assert_eq!(
            Deadline::after_ticks(Instant::from_ticks(u64::MAX), 1),
            Err(Error::InvalidDeadline),
        );
    }

    #[test]
    fn alarm_admits_future_deadlines_across_wrap() {
        struct Fake {
            now: u32,
            writes: usize,
        }
        impl Registers for Fake {
            fn read(&mut self, address: usize) -> u32 {
                if address == RAWL { self.now } else { 0 }
            }
            fn write(&mut self, _: usize, _: u32) {
                self.now = self.now.wrapping_add(2);
                self.writes += 1;
            }
        }
        for (delta, expected, writes) in [
            (0, Err(Error::Timeout), 0),
            (1, Err(Error::Timeout), 2),
            (2, Err(Error::Timeout), 2),
            (i32::MAX as u32, Ok(()), 1),
            (1 << 31, Err(Error::Timeout), 0),
            (u32::MAX, Err(Error::Timeout), 0),
        ] {
            let mut io = Fake {
                now: u32::MAX,
                writes: 0,
            };
            let instant = Instant32::from_ticks(u32::MAX.wrapping_add(delta));
            assert_eq!(try_arm(&mut io, 0, instant), expected);
            assert_eq!(io.writes, writes);
        }
        for (instant, expected, writes) in [
            (0, Err(Error::Timeout), 0),
            (u64::from(u32::MAX) + 1, Err(Error::Timeout), 2),
            (u64::from(u32::MAX) + 3, Ok(()), 1),
            (
                u64::from(u32::MAX) + 1 + u64::from(MAX_ARM),
                Err(Error::InvalidDeadline),
                0,
            ),
        ] {
            let mut io = Fake {
                now: u32::MAX,
                writes: 0,
            };
            assert_eq!(try_arm(&mut io, 0, Instant::from_ticks(instant)), expected);
            assert_eq!(io.writes, writes);
        }
    }

    /// Review 6 #3: whatever the phase of the first read, the real time spent is at least
    /// the request (`delay_ns(500)` used to return after ~0 µs when the counter stepped right
    /// after `start`).
    #[test]
    fn delays_are_never_short() {
        for us in [0u32, 1, 2, 7] {
            for phase_ns in [0, 1, 500, 999] {
                // Real time in ns; each counter read takes 10 ns.
                let t = Cell::new(phase_ns);
                let now = || {
                    let v = t.get() / 1000;
                    t.set(t.get() + 10);
                    v
                };
                let begin = t.get();
                wait_at_least(now, u64::from(us));
                let spent = t.get() - begin;
                assert!(
                    spent >= u64::from(us) * 1000,
                    "us={us} phase={phase_ns} {spent}"
                );
                assert!(
                    spent <= u64::from(us) * 1000 + 1020,
                    "us={us} phase={phase_ns} {spent}"
                );
            }
        }
    }
}
