//! Core-local IRQ + WFE waits on one dedicated alarm per core.

use super::{
    ALARM0, ARMED, Alarm, Deadline, Duration, Error, INTE_CLEAR, INTE_SET, INTR, MAX_ARM, Timer,
};
use crate::{linux_blocks::Mmio, scmi::Registers, write};
use core::sync::atomic::{AtomicU32, Ordering};
use embedded_hal::delay::DelayNs;
use rp1_rt::{
    interrupt::{AlarmIrq, Binding, ContextError, OwnedLine, ThreadGuard},
    send_event, wait_for_event,
};

/// Override a blocking driver's default timeout. The driver admits it at each call.
#[derive(Clone, Copy, Debug, Default)]
pub struct WaitConfig {
    timeout: Option<Duration>,
}

impl WaitConfig {
    pub const fn new() -> Self {
        Self { timeout: None }
    }

    pub const fn default_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn timeout_or(self, default: Duration) -> Duration {
        self.timeout.unwrap_or(default)
    }
}

/// The dedicated alarm and IRQ for synchronous waits on this core.
/// Declare it with `wait_timer = "ALARMn"` alongside the core's `timer` list.
///
/// Waiting inside `Slot::try_with`, `Slot::with` or `free`, or after fault continuation
/// raises BASEPRI, returns [`Error::Masked`].
pub struct WaitTimer {
    alarm: u8,
    line: OwnedLine,
    generation: u32,
}

impl WaitTimer {
    /// # Safety
    /// The generating declaration must route this line exclusively to the matching
    /// [`Self::__on_interrupt`] dispatcher on CORE.
    #[doc(hidden)]
    pub unsafe fn __bind<const N: u8, const IRQ: u8, const CORE: u8>(
        _alarm: Alarm<N>,
        irq: Binding<AlarmIrq<N>, IRQ, CORE>,
    ) -> Self {
        const { assert!(IRQ == 26 + N && N < 4 && CORE < 2) };
        Self {
            alarm: N,
            line: irq.into_owned_line(),
            generation: 0,
        }
    }

    pub fn thread_context(&mut self) -> Result<WaitContext<'_>, Error> {
        let context = WaitContext {
            timer: self,
            guard: ThreadGuard::new()?,
        };
        context.check_alarm()?;
        Ok(context)
    }

    /// Sleeps with WFE for at least `duration`. The target has one tick of phase
    /// allowance. Return latency also includes timer access, IRQ entry and any
    /// preemption. Long delays are split
    /// into comparisons of at most `i32::MAX` µs with a single phase allowance.
    pub fn delay(&mut self, duration: Duration) -> Result<(), Error> {
        self.thread_context()?.delay(duration)
    }

    /// Sleeps until the deadline. An already reached deadline completes immediately.
    pub fn delay_until(&mut self, end: impl Into<Deadline>) -> Result<(), Error> {
        self.thread_context()?.delay_until(end)
    }

    /// Waits for a source that publishes retained state and sends an event.
    /// See [`WaitContext::wait_until`] for the probe and deadline contract.
    pub fn wait_until<T>(
        &mut self,
        end: impl Into<Deadline>,
        probe: impl FnMut() -> Option<T>,
    ) -> Result<T, Error> {
        self.thread_context()?.wait_until(end, probe)
    }

    /// Waits for a bound IRQ after checking that its NVIC line is enabled.
    /// The handler publishes retained state and sends an event after publishing.
    /// The dedicated deadline alarm bounds the wait if the IRQ never arrives.
    /// See [`WaitContext::wait_until`] for the probe and deadline contract.
    pub fn wait_irq_until<S, const IRQ: u8, const CORE: u8, T>(
        &mut self,
        irq: &Binding<S, IRQ, CORE>,
        end: impl Into<Deadline>,
        probe: impl FnMut() -> Option<T>,
    ) -> Result<T, Error> {
        let mut context = self.thread_context()?;
        context.check_irq(irq)?;
        context.wait_until(end, probe)
    }

    /// # Safety
    /// Called only by the generated ISR owning this alarm.
    #[doc(hidden)]
    #[inline(always)]
    pub unsafe fn __on_interrupt<const N: u8>() {
        if acknowledge(&mut Mmio, N) {
            EVENTS[usize::from(N)].publish();
            send_event();
        }
    }
}

/// The thread-mode borrow of one core's alarm.
pub struct WaitContext<'a> {
    timer: &'a mut WaitTimer,
    guard: ThreadGuard,
}

impl<'timer> WaitContext<'timer> {
    /// Validate an additional wake source at the entrance to a blocking operation.
    pub fn check_irq<S, const IRQ: u8, const CORE: u8>(
        &self,
        irq: &Binding<S, IRQ, CORE>,
    ) -> Result<(), Error> {
        self.guard.check_irq(irq).map_err(Error::from)
    }

    /// Wait for a retained completion or the one-shot deadline. The source must publish
    /// its state before calling [`send_event`]. The probe must be bounded and must not
    /// change PRIMASK, FAULTMASK or BASEPRI. It is called before arming, after arming,
    /// and after wakes. A final ready result wins at timeout. A deadline already
    /// reached on entry returns [`Error::Timeout`] without calling the probe.
    pub fn wait_until<T>(
        &mut self,
        end: impl Into<Deadline>,
        probe: impl FnMut() -> Option<T>,
    ) -> Result<T, Error> {
        self.check_alarm()?;
        let end = end.into();
        end.check(Timer::new().now())?;
        self.wait_accepted(end, probe)
    }

    pub fn delay(&mut self, duration: Duration) -> Result<(), Error> {
        self.delay_us(u64::from(duration.ticks()))
    }

    pub fn delay_until(&mut self, end: impl Into<Deadline>) -> Result<(), Error> {
        self.check_alarm()?;
        let end = end.into();
        match end.check(Timer::new().now()) {
            Err(Error::Timeout) => Ok(()),
            result => {
                result?;
                self.delay_accepted(end)
            }
        }
    }

    pub fn delay_adapter(&mut self) -> Result<Delay<'_, 'timer>, Error> {
        self.check_alarm()?;
        Ok(Delay { context: self })
    }

    fn check_alarm(&self) -> Result<(), ContextError> {
        // The first arm enables this dedicated line. Later calls leave it enabled between waits.
        if self.timer.generation == 0 {
            self.guard.check()
        } else {
            self.guard.check_owned_line(&self.timer.line)
        }
    }

    fn delay_us(&mut self, us: u64) -> Result<(), Error> {
        self.check_alarm()?;
        delay_chunks(us, |chunk| {
            let end = Deadline::after_ticks(Timer::new().now(), chunk)?;
            self.delay_accepted(end)
        })
    }

    fn delay_accepted(&mut self, end: Deadline) -> Result<(), Error> {
        match self.wait_accepted(end, || None::<()>) {
            Err(Error::Timeout) => Ok(()),
            result => result,
        }
    }

    fn wait_accepted<T>(
        &mut self,
        end: Deadline,
        probe: impl FnMut() -> Option<T>,
    ) -> Result<T, Error> {
        let mut alarm = ActiveAlarm {
            timer: self.timer,
            armed: false,
        };
        let result = wait(&mut alarm, end, probe);
        alarm.finish()?;
        result
    }
}

fn delay_chunks(us: u64, mut delay: impl FnMut(u32) -> Result<(), Error>) -> Result<(), Error> {
    if us == 0 {
        return Ok(());
    }
    // The phase allowance keeps the lower bound even if the first read almost straddles a tick.
    let mut remaining = us + 1;
    while remaining != 0 {
        let chunk = remaining.min(u64::from(MAX_ARM)) as u32;
        delay(chunk)?;
        remaining -= u64::from(chunk);
    }
    Ok(())
}

/// # Panics
///
/// In a handler, with PRIMASK, FAULTMASK or BASEPRI set (for example inside `free`
/// or a `Slot` borrow), or if the dedicated alarm is unavailable. Use
/// [`WaitTimer::delay`] to handle these errors.
impl DelayNs for WaitTimer {
    fn delay_ns(&mut self, ns: u32) {
        self.delay_us(ns.div_ceil(1000));
    }

    fn delay_us(&mut self, us: u32) {
        self.delay(Duration::from_ticks(us))
            .expect("IRQ delay failed");
    }

    fn delay_ms(&mut self, ms: u32) {
        self.thread_context()
            .and_then(|mut ctx| ctx.delay_us(u64::from(ms) * 1000))
            .expect("IRQ delay failed");
    }
}

/// The IRQ-backed [`DelayNs`] adapter for an existing wait context.
/// Its methods panic under the same conditions as the [`WaitTimer`] implementation.
pub struct Delay<'ctx, 'timer> {
    context: &'ctx mut WaitContext<'timer>,
}

impl DelayNs for Delay<'_, '_> {
    fn delay_ns(&mut self, ns: u32) {
        self.context.timer.delay_ns(ns);
    }

    fn delay_us(&mut self, us: u32) {
        self.context.timer.delay_us(us);
    }

    fn delay_ms(&mut self, ms: u32) {
        self.context.timer.delay_ms(ms);
    }
}

pub(crate) trait WaitAlarm {
    fn arm(&mut self, end: Deadline) -> Result<(), Error>;
    fn elapsed(&self, end: Deadline) -> bool;
    fn sleep(&mut self);
}

pub(crate) fn wait<T>(
    alarm: &mut impl WaitAlarm,
    end: Deadline,
    mut probe: impl FnMut() -> Option<T>,
) -> Result<T, Error> {
    if let Some(value) = probe() {
        return Ok(value);
    }
    alarm.arm(end)?;
    loop {
        let elapsed = alarm.elapsed(end);
        if let Some(value) = probe() {
            return Ok(value);
        }
        if elapsed {
            return Err(Error::Timeout);
        }
        alarm.sleep();
    }
}

// Ordinary static DSRAM has one copy per core. Each alarm has its own slot. On this
// core only its ISR publishes, and thread mode changes the generation with that line disabled.
static EVENTS: [AlarmEvent; 4] = [const { AlarmEvent(AtomicU32::new(0)) }; 4];

struct AlarmEvent(AtomicU32);

impl AlarmEvent {
    fn register(&self, generation: u32) {
        self.0.store(generation, Ordering::Relaxed);
    }

    fn publish(&self) -> bool {
        let current = self.0.load(Ordering::Relaxed);
        if current == 0 || current & 1 != 0 {
            return false;
        }
        self.0.store(current | 1, Ordering::Release);
        true
    }

    fn fired(&self, generation: u32) -> bool {
        self.0.load(Ordering::Acquire) == (generation | 1)
    }
}

fn acknowledge(io: &mut impl Registers, alarm: u8) -> bool {
    let bit = 1 << alarm;
    if io.read(INTR) & bit == 0 {
        return false;
    }
    // The comparator is one-shot. send_event completes the W1C before exception return.
    io.write_relaxed(INTR, bit);
    true
}

struct ActiveAlarm<'a> {
    timer: &'a mut WaitTimer,
    armed: bool,
}

impl ActiveAlarm<'_> {
    fn event(&self) -> &AlarmEvent {
        &EVENTS[usize::from(self.timer.alarm)]
    }

    // The dedicated NVIC line can be cleared without discarding other devices' IRQs.
    // Keep higher-priority control interrupts enabled during the MMIO sequence.
    fn retire(&mut self) -> bool {
        self.timer.line.disable();
        // No ISB: an entry in this window publishes a generation that register(0) discards.
        rp1_proto::barrier();
        self.event().register(0);
        let retired = retire_source(&mut Mmio, self.timer.alarm);
        self.timer.line.unpend();
        retired
    }

    fn finish(&mut self) -> Result<(), Error> {
        if !self.armed {
            return Ok(());
        }
        self.armed = false;
        let retired = self.retire();
        self.timer.line.enable();
        if retired {
            Ok(())
        } else {
            Err(Error::AlarmUnavailable)
        }
    }
}

fn retire_source(io: &mut impl Registers, alarm: u8) -> bool {
    let bit = 1 << alarm;
    io.write(INTE_CLEAR, bit);
    io.write(ARMED, bit);
    io.write(INTR, bit);
    (io.read(ARMED) | io.read(INTR)) & bit == 0
}

impl WaitAlarm for ActiveAlarm<'_> {
    fn arm(&mut self, end: Deadline) -> Result<(), Error> {
        if Timer::new().now().ticks() >= end.instant().ticks() {
            return Ok(());
        }
        if !self.retire() {
            self.timer.line.enable();
            return Err(Error::AlarmUnavailable);
        }
        self.timer.generation = self.timer.generation.wrapping_add(2).max(2);
        self.event().register(self.timer.generation);
        self.armed = true;
        let alarm = usize::from(self.timer.alarm);
        // A compare that races its tick may miss; the full-width check before WFE
        // catches it and the retirement sequence disarms it.
        write(ALARM0 + alarm * 4, end.instant().ticks() as u32);
        write(INTE_SET, 1 << alarm);
        self.timer.line.enable();
        Ok(())
    }

    fn elapsed(&self, end: Deadline) -> bool {
        (self.armed && self.event().fired(self.timer.generation))
            || Timer::new().now().ticks() >= end.instant().ticks()
    }

    fn sleep(&mut self) {
        wait_for_event();
    }
}

impl Drop for ActiveAlarm<'_> {
    fn drop(&mut self) {
        let _ = self.finish();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::timer::Instant;
    use core::cell::Cell;

    struct FakeWake<'a> {
        tick: &'a Cell<u64>,
        ready_at: u64,
        end: u64,
        arms: u32,
        sleeps: u32,
        spurious: bool,
    }

    impl WaitAlarm for FakeWake<'_> {
        fn arm(&mut self, end: Deadline) -> Result<(), Error> {
            self.arms += 1;
            self.end = end.instant().ticks();
            self.tick.set(self.tick.get() + 1);
            Ok(())
        }

        fn elapsed(&self, end: Deadline) -> bool {
            self.tick.get() >= end.instant().ticks()
        }

        fn sleep(&mut self) {
            self.sleeps += 1;
            if self.spurious {
                self.spurious = false;
                return;
            }
            // WFE consumes an event already latched after the post-arm probe, or
            // sleeps until the next publish. Multiple events may share one wake.
            self.tick.set(self.ready_at.min(self.end));
        }
    }

    #[test]
    fn arm_races_and_silence_use_one_deadline_and_preserve_same_tick_completion() {
        let end = Deadline(Instant::from_ticks(10));
        for (ready_at, spurious, arms, sleeps, probes, expected) in [
            (0, false, 0, 0, 1, Ok(0)), // Ready before arming.
            (1, false, 1, 0, 2, Ok(1)), // Ready during arming.
            (2, false, 1, 1, 3, Ok(2)), // Ready between recheck and WFE or while asleep.
            (2, true, 1, 2, 4, Ok(2)),  // A spurious event consumes one WFE.
            (10, false, 1, 1, 3, Ok(10)),
            (11, false, 1, 1, 3, Err(Error::Timeout)),
            (u64::MAX, false, 1, 1, 3, Err(Error::Timeout)),
        ] {
            let tick = Cell::new(0);
            let mut alarm = FakeWake {
                tick: &tick,
                ready_at,
                end: 0,
                arms: 0,
                sleeps: 0,
                spurious,
            };
            let mut reads = 0;
            let result = wait(&mut alarm, end, || {
                reads += 1;
                (tick.get() >= ready_at).then_some(tick.get())
            });
            assert_eq!(result, expected);
            assert_eq!((alarm.arms, alarm.sleeps, reads), (arms, sleeps, probes));
        }
    }

    #[test]
    fn a_compare_reached_during_arm_never_sleeps_for_a_wrap() {
        let tick = Cell::new(u64::from(u32::MAX));
        let mut alarm = FakeWake {
            tick: &tick,
            ready_at: u64::MAX,
            end: 0,
            arms: 0,
            sleeps: 0,
            spurious: false,
        };
        let end = Deadline(Instant::from_ticks(tick.get() + 1));
        assert_eq!(wait(&mut alarm, end, || None::<()>), Err(Error::Timeout));
        assert_eq!(alarm.sleeps, 0);
    }

    #[test]
    fn a_latched_event_between_probe_and_wfe_is_observed() {
        let tick = Cell::new(0);
        let mut alarm = FakeWake {
            tick: &tick,
            ready_at: 2,
            end: 0,
            arms: 0,
            sleeps: 0,
            spurious: false,
        };
        let end = Deadline(Instant::from_ticks(10));
        let ready = wait(&mut alarm, end, || {
            if tick.get() == 1 {
                // The ISR publishes after the probe read, before the thread can sleep.
                tick.set(2);
                return None;
            }
            (tick.get() == 2).then_some(())
        });
        assert_eq!(ready, Ok(()));
        assert_eq!((alarm.arms, alarm.sleeps, tick.get()), (1, 1, 2));
    }

    #[test]
    fn delay_chunks_account_for_phase_and_the_maximum_delay_ns_adapter_inputs() {
        for us in [
            0,
            1,
            u64::from(MAX_ARM),
            u64::from(u32::MAX),
            u64::from(u32::MAX) * 1000,
        ] {
            let mut total = 0;
            delay_chunks(us, |chunk| {
                assert!((1..=MAX_ARM).contains(&chunk));
                total += u64::from(chunk);
                Ok(())
            })
            .unwrap();
            assert_eq!(total, us + u64::from(us != 0));
        }
    }

    struct FakeSources {
        armed: u32,
        pending: u32,
        enabled: u32,
        stuck: u32,
    }

    impl Registers for FakeSources {
        fn read(&mut self, address: usize) -> u32 {
            match address {
                ARMED => self.armed,
                INTR => self.pending,
                _ => panic!("unexpected read"),
            }
        }

        fn write(&mut self, address: usize, bits: u32) {
            match address {
                ARMED => self.armed &= !(bits & !self.stuck),
                INTR => self.pending &= !bits,
                INTE_CLEAR => self.enabled &= !bits,
                _ => panic!("unexpected write"),
            }
        }
    }

    #[test]
    fn retire_clears_only_its_source_and_rejects_stale_delivery() {
        let mut io = FakeSources {
            armed: 0b11,
            pending: 0b11,
            enabled: 0b11,
            stuck: 0,
        };
        let old = 2;
        let new = 4;
        let timer = AlarmEvent(AtomicU32::new(0));
        let neighbor = AlarmEvent(AtomicU32::new(0));
        timer.register(old);
        neighbor.register(old);

        assert!(acknowledge(&mut io, 0) && timer.publish());
        assert!(timer.fired(old));
        assert!(!timer.publish()); // A merged/duplicate delivery does not change the generation.
        assert!(!neighbor.fired(old));
        assert_eq!(io.pending, 0b10);

        timer.register(0);
        assert!(retire_source(&mut io, 0));
        assert!(!timer.publish());
        assert_eq!((io.armed, io.enabled), (0b10, 0b10));
        timer.register(new);
        assert!(!timer.fired(old));
        assert!(!acknowledge(&mut io, 0)); // A delayed NVIC entry has no raw event for this source.
        assert!(!timer.fired(new));
        assert!(acknowledge(&mut io, 1) && neighbor.publish());
        assert!(neighbor.fired(old));
        assert!(!timer.fired(new));

        io.armed = 1;
        io.stuck = 1;
        assert!(!retire_source(&mut io, 0));
    }
}
