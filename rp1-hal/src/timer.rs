use crate::{Error, read, write};
use embedded_hal::delay::DelayNs;

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
    pub fn now(&self) -> u64 {
        loop {
            let hi = read(0x400a_c024);
            let lo = read(0x400a_c028);
            if hi == read(0x400a_c024) {
                return (u64::from(hi) << 32) | u64::from(lo);
            }
        }
    }
    /// Waits at least `us` microseconds (up to about 1 µs more).
    pub fn delay_us(&self, us: u32) {
        wait_at_least(|| self.now(), us);
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
        Timer::delay_us(self, us);
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
    pub fn schedule(&mut self, deadline: u32) -> Result<(), Error> {
        let delta = deadline.wrapping_sub(read(0x400a_c028));
        if delta == 0 || delta > i32::MAX as u32 {
            return Err(Error::Invalid);
        }
        write(0x400a_c010 + N as usize * 4, deadline);
        // A deadline can expire between admission and arming; disarm rather than wait a full wrap.
        if deadline.wrapping_sub(read(0x400a_c028)) > i32::MAX as u32 {
            self.cancel();
            return Err(Error::Timeout);
        }
        Ok(())
    }
    pub fn cancel(&mut self) {
        write(0x400a_c020, 1 << N);
    }
    pub fn pending(&self) -> bool {
        read(0x400a_c034) & (1 << N) != 0
    }
    pub fn clear(&mut self) {
        write(0x400a_c034, 1 << N);
    }
    /// Unmask this alarm's timer interrupt.
    ///
    /// v0.1 alarms are polling-only ([`Alarm::pending`] / [`Alarm::clear`]): `rp1-rt` has no
    /// handler registration and every vector points at a default handler that never returns.
    ///
    /// ```compile_fail,E0133
    /// let mut alarm = unsafe { rp1_hal::timer::Alarm::<0>::new() };
    /// alarm.enable_interrupt(); // unsafe: no handler can be registered in v0.1
    /// ```
    ///
    /// # Safety
    /// Before this IRQ is enabled in the NVIC and unmasked on the CPU, the caller has
    /// installed its own handler for it (its own vector table via VTOR); otherwise the core
    /// parks in the default handler.
    pub unsafe fn enable_interrupt(&mut self) {
        write(0x400a_e038, 1 << N);
    }
    pub fn disable_interrupt(&mut self) {
        write(0x400a_f038, 1 << N);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::cell::Cell;

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
