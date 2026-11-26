use crate::{Error, linux_blocks::Mmio, read, scmi::Registers, write};
use embedded_hal::delay::DelayNs;
use fugit::{MicrosDurationU32, TimerInstantU32, TimerInstantU64};

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
    pub fn now(&self) -> TimerInstantU64<1_000_000> {
        loop {
            let hi = read(0x400a_c024);
            let lo = read(0x400a_c028);
            if hi == read(0x400a_c024) {
                return TimerInstantU64::from_ticks((u64::from(hi) << 32) | u64::from(lo));
            }
        }
    }
    /// Reads the low word of the timer, for use with [`Alarm::schedule`].
    pub fn now_32(&self) -> TimerInstantU32<1_000_000> {
        TimerInstantU32::from_ticks(read(0x400a_c028))
    }

    /// Waits at least `duration` (up to about 1 µs more).
    pub fn delay(&self, duration: MicrosDurationU32) {
        wait_at_least(|| self.now().ticks(), duration.ticks());
    }
}
/// `start` may be read just before the counter steps, so `us` steps can be almost 1 µs short:
/// wait for `us + 1` steps instead.
fn wait_at_least(mut now: impl FnMut() -> u64, us: u32) {
    if us == 0 {
        return;
    }
    let start = now();
    while now().wrapping_sub(start) <= u64::from(us) {
        core::hint::spin_loop();
    }
}
impl DelayNs for Timer {
    fn delay_ns(&mut self, ns: u32) {
        self.delay_us(ns.div_ceil(1000));
    }
    fn delay_us(&mut self, us: u32) {
        self.delay(MicrosDurationU32::from_ticks(us));
    }
}
pub struct Alarm<const N: u8> {
    _private: (),
}
impl<const N: u8> Alarm<N> {
    /// # Safety
    /// Exactly one instance for the declared owner, with N < 4.
    pub const unsafe fn new() -> Self {
        assert!(N < 4);
        Self { _private: () }
    }
    /// Arms this alarm at an absolute timer instant within the next half-cycle.
    pub fn schedule(&mut self, deadline: TimerInstantU32<1_000_000>) -> Result<(), Error> {
        schedule(&mut Mmio, N, deadline.ticks())
    }
    pub fn cancel(&mut self) {
        write(0x400a_c020, 1 << N);
    }
    /// Whether the alarm has fired, with or without its interrupt enabled.
    pub fn finished(&self) -> bool {
        read(0x400a_c034) & (1 << N) != 0
    }
    /// Clears the fired flag, which also deasserts the alarm's interrupt.
    pub fn clear_interrupt(&mut self) {
        write(0x400a_c034, 1 << N);
    }
    /// Unmasks this alarm's interrupt. The SET alias leaves the other alarms' enables alone;
    /// the NVIC line is `irq.TIMER_N` in the generated resources.
    pub fn enable_interrupt(&mut self) {
        write(0x400a_e038, 1 << N);
    }
    pub fn disable_interrupt(&mut self) {
        write(0x400a_f038, 1 << N);
    }
}

fn schedule(io: &mut impl Registers, n: u8, deadline: u32) -> Result<(), Error> {
    let delta = deadline.wrapping_sub(io.read(0x400a_c028));
    if delta == 0 || delta > i32::MAX as u32 {
        return Err(Error::Invalid);
    }
    io.write(0x400a_c010 + n as usize * 4, deadline);
    // A deadline can expire between admission and arming; disarm rather than wait a full wrap.
    if deadline.wrapping_sub(io.read(0x400a_c028)) > i32::MAX as u32 {
        io.write(0x400a_c020, 1 << n);
        return Err(Error::Timeout);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::cell::Cell;

    #[test]
    fn alarm_admits_future_deadlines_across_wrap() {
        struct Fake {
            now: u32,
            writes: usize,
        }
        impl Registers for Fake {
            fn read(&mut self, _: usize) -> u32 {
                self.now
            }
            fn write(&mut self, _: usize, _: u32) {
                self.now = self.now.wrapping_add(2);
                self.writes += 1;
            }
        }
        for (delta, expected, writes) in [
            (0, Err(Error::Invalid), 0),
            (1, Err(Error::Timeout), 2),
            (2, Ok(()), 1),
            (i32::MAX as u32, Ok(()), 1),
            (1 << 31, Err(Error::Invalid), 0),
        ] {
            let mut io = Fake {
                now: u32::MAX,
                writes: 0,
            };
            assert_eq!(schedule(&mut io, 0, u32::MAX.wrapping_add(delta)), expected);
            assert_eq!(io.writes, writes);
        }
    }

    /// Review 6 #3: whatever the phase of the first read, the real time spent is at least
    /// the request (`delay_ns(500)` used to return after ~0 µs when the counter stepped right
    /// after `start`).
    #[test]
    fn delays_are_never_short() {
        for us in [0, 1, 2, 7] {
            for phase_ns in [0, 1, 500, 999] {
                // Real time in ns; each counter read takes 10 ns.
                let t = Cell::new(phase_ns);
                let now = || {
                    let v = t.get() / 1000;
                    t.set(t.get() + 10);
                    v
                };
                let begin = t.get();
                wait_at_least(now, us);
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
