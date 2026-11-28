//! Core-local IRQ + WFE waits on one dedicated alarm per core.
//! The checked paths admit deadlines with a full timer sample. The accepted
//! paths reuse a caller's full-sample admission.

use super::{
    ALARM0, ARMED, Alarm, Deadline, Duration, Error, INTE_CLEAR, INTE_SET, INTR, MAX_ARM, Timer,
};
use crate::{linux_blocks::Mmio, scmi::Registers, write_relaxed};
use core::sync::atomic::{AtomicU32, Ordering, compiler_fence};
use embedded_hal::delay::DelayNs;
use rp1_rt::{
    interrupt::{AlarmIrq, Binding, ContextError, LinePriority, OwnedLine, ThreadGuard},
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
    clean: bool,
    line: OwnedLine,
    event: &'static AlarmEvent,
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
            clean: false,
            line: irq.into_owned_line(),
            event: &EVENTS[usize::from(N)],
            generation: 0,
        }
    }

    #[rp1_rt::isram]
    pub fn thread_context(&mut self) -> Result<WaitContext<'_>, Error> {
        let context = WaitContext {
            timer: self,
            guard: ThreadGuard::new()?,
        };
        context.check_alarm_line()?;
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
        self.thread_context()?.delay_checked(end.into())
    }

    /// Waits for a source that publishes retained state and sends an event.
    /// See [`WaitContext::wait_until`] for the probe and deadline contract.
    pub fn wait_until<T>(
        &mut self,
        end: impl Into<Deadline>,
        probe: impl FnMut() -> Option<T>,
    ) -> Result<T, Error> {
        self.thread_context()?.wait_checked(end.into(), probe)
    }

    /// Waits for a bound IRQ after checking that its NVIC line is enabled.
    /// The handler publishes retained state and sends an event after publishing.
    /// The dedicated deadline alarm bounds the wait if the IRQ never arrives.
    /// See [`WaitContext::wait_until`] for the probe and deadline contract.
    pub fn wait_irq_until<S, const IRQ: u8, const CORE: u8, P: LinePriority, T>(
        &mut self,
        irq: &Binding<S, IRQ, CORE, P>,
        end: impl Into<Deadline>,
        probe: impl FnMut() -> Option<T>,
    ) -> Result<T, Error> {
        let mut context = self.thread_context()?;
        context.check_irq(irq)?;
        context.wait_checked(end.into(), probe)
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
    pub fn check_irq<S, const IRQ: u8, const CORE: u8, P: LinePriority>(
        &self,
        irq: &Binding<S, IRQ, CORE, P>,
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
        self.wait_checked(end.into(), probe)
    }

    pub fn delay(&mut self, duration: Duration) -> Result<(), Error> {
        self.delay_us(u64::from(duration.ticks()))
    }

    pub fn delay_until(&mut self, end: impl Into<Deadline>) -> Result<(), Error> {
        self.check_alarm()?;
        self.delay_checked(end.into())
    }

    #[rp1_rt::isram]
    fn delay_checked(&mut self, end: Deadline) -> Result<(), Error> {
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
        self.guard.check()?;
        self.check_alarm_line()
    }

    fn check_alarm_line(&self) -> Result<(), ContextError> {
        // The first arm enables this dedicated line. Later calls leave it enabled between waits.
        if self.timer.generation == 0 || self.timer.line.is_enabled() {
            Ok(())
        } else {
            Err(ContextError::IrqUnavailable)
        }
    }

    fn delay_us(&mut self, us: u64) -> Result<(), Error> {
        self.check_alarm()?;
        delay_chunks(us, |chunk| {
            let end = Deadline::after_ticks(Timer::new().now(), chunk)?;
            self.delay_accepted(end)
        })
    }

    // The caller has bounded the target against a full counter sample.
    pub(crate) fn delay_accepted(&mut self, end: Deadline) -> Result<(), Error> {
        match self.wait_accepted(end, |_| None::<()>) {
            Err(Error::Timeout) => Ok(()),
            result => result,
        }
    }

    fn wait_checked<T>(
        &mut self,
        end: Deadline,
        mut probe: impl FnMut() -> Option<T>,
    ) -> Result<T, Error> {
        end.check(Timer::new().now())?;
        self.wait_accepted(end, |_| probe())
    }

    pub(crate) fn wait_accepted<T>(
        &mut self,
        end: Deadline,
        probe: impl FnMut(bool) -> Option<T>,
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
    fn elapsed_after_arm(&self, end: Deadline) -> bool;
    fn deadline_signaled(&self) -> bool;
    fn sleep(&mut self);
}

pub(crate) fn wait<T>(
    alarm: &mut impl WaitAlarm,
    end: Deadline,
    mut probe: impl FnMut(bool) -> Option<T>,
) -> Result<T, Error> {
    if let Some(value) = probe(false) {
        return Ok(value);
    }
    alarm.arm(end)?;
    if let Some(value) = probe(false) {
        return Ok(value);
    }
    if !alarm.elapsed_after_arm(end) {
        loop {
            alarm.sleep();
            // A retained alarm IRQ goes straight to the final probe. A device
            // wake can still succeed without reading the timer.
            if alarm.deadline_signaled() {
                break;
            }
            if let Some(value) = probe(false) {
                return Ok(value);
            }
            if alarm.elapsed(end) {
                break;
            }
        }
    }
    probe(true).ok_or(Error::Timeout)
}

// Ordinary static DSRAM has one copy per core. Each alarm has its own slot. On this
// core only its ISR publishes, and thread mode invalidates the generation before cleanup.
// The word carries the entire event. send_event orders the interrupt acknowledgment.
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
        self.0.store(current | 1, Ordering::Relaxed);
        true
    }

    fn fired(&self, generation: u32) -> bool {
        self.0.load(Ordering::Relaxed) == (generation | 1)
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
    #[rp1_rt::isram]
    fn retire(&mut self) -> bool {
        let retired = self.retire_with(&mut Mmio);
        self.timer.line.unpend();
        retired
    }

    // The dedicated NVIC line can be cleared without discarding other devices' IRQs.
    // The ISR can only acknowledge this same source. Invalidate the event before
    // clearing it so an IRQ in the retirement window cannot publish a completion.
    #[inline(always)]
    fn retire_with(&mut self, io: &mut impl Registers) -> bool {
        self.timer.event.register(0);
        compiler_fence(Ordering::SeqCst);
        let retired = retire_source(io, self.timer.alarm);
        self.timer.clean = retired;
        retired
    }

    #[inline(always)]
    fn finish(&mut self) -> Result<(), Error> {
        if !self.armed {
            return Ok(());
        }
        self.armed = false;
        if self.retire() {
            Ok(())
        } else {
            Err(Error::AlarmUnavailable)
        }
    }
}

fn retire_source(io: &mut impl Registers, alarm: u8) -> bool {
    let bit = 1 << alarm;
    io.write_relaxed(INTE_CLEAR, bit);
    io.write_relaxed(ARMED, bit);
    io.write_relaxed(INTR, bit);
    // The first read orders the writes. Verify both states independently.
    (io.read(ARMED) | io.read(INTR)) & bit == 0
}

fn clean_before_arm(clean: bool, retire: impl FnOnce() -> bool) -> Result<(), Error> {
    if clean || retire() {
        Ok(())
    } else {
        Err(Error::AlarmUnavailable)
    }
}

fn reached_low(now: u32, end: Deadline) -> bool {
    now.wrapping_sub(end.instant().ticks() as u32) as i32 >= 0
}

impl WaitAlarm for ActiveAlarm<'_> {
    #[rp1_rt::isram]
    fn arm(&mut self, end: Deadline) -> Result<(), Error> {
        clean_before_arm(self.timer.clean, || self.retire())?;
        let first = self.timer.generation == 0;
        self.timer.generation = self.timer.generation.wrapping_add(2).max(2);
        self.timer.event.register(self.timer.generation);
        self.armed = true;
        self.timer.clean = false;
        // Publish the generation before the comparator can fire.
        compiler_fence(Ordering::SeqCst);
        let alarm = usize::from(self.timer.alarm);
        write_relaxed(ALARM0 + alarm * 4, end.instant().ticks() as u32);
        write_relaxed(INTE_SET, 1 << alarm);
        rp1_proto::barrier();
        if first {
            self.timer.line.enable();
        }
        Ok(())
    }

    fn elapsed(&self, end: Deadline) -> bool {
        // The post-arm full sample proved the programmed tick was still
        // ahead. A long preemption that spans a wrap leaves the ISR's event.
        self.timer.event.fired(self.timer.generation)
            || reached_low(Timer::new().now_32().ticks(), end)
    }

    fn deadline_signaled(&self) -> bool {
        self.timer.event.fired(self.timer.generation)
    }

    fn elapsed_after_arm(&self, end: Deadline) -> bool {
        // A delayed compare write can miss the tick. Do not sleep for a wrap.
        Timer::new().now().ticks() >= end.instant().ticks()
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

        fn elapsed_after_arm(&self, end: Deadline) -> bool {
            self.elapsed(end)
        }

        fn deadline_signaled(&self) -> bool {
            false
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
            (11, false, 1, 1, 4, Err(Error::Timeout)),
            (u64::MAX, false, 1, 1, 4, Err(Error::Timeout)),
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
            let result = wait(&mut alarm, end, |_| {
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
        assert_eq!(wait(&mut alarm, end, |_| None::<()>), Err(Error::Timeout));
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
        let ready = wait(&mut alarm, end, |_| {
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
    fn successful_wake_skips_the_clock_and_timeout_probes_again() {
        struct Wake<'a> {
            ready: &'a Cell<bool>,
            device_first: bool,
            checks: u8,
        }

        impl WaitAlarm for Wake<'_> {
            fn arm(&mut self, _: Deadline) -> Result<(), Error> {
                Ok(())
            }
            fn elapsed_after_arm(&self, _: Deadline) -> bool {
                false
            }
            fn deadline_signaled(&self) -> bool {
                false
            }
            fn elapsed(&self, _: Deadline) -> bool {
                assert!(!self.device_first, "a ready wake sampled the clock");
                self.ready.set(true);
                true
            }
            fn sleep(&mut self) {
                self.checks += 1;
                self.ready.set(self.device_first);
            }
        }

        for device_first in [false, true] {
            let ready = Cell::new(false);
            let mut wake = Wake {
                ready: &ready,
                device_first,
                checks: 0,
            };
            let end = Deadline::at(Instant::from_ticks(1));
            let result = wait(&mut wake, end, |final_probe| {
                ready.replace(false).then_some(final_probe)
            });
            assert_eq!(result, Ok(!device_first));
            assert_eq!(wake.checks, 1);
        }
    }

    #[test]
    fn missed_compare_is_caught_by_the_full_check_before_wfe() {
        struct MissedCompare;
        impl WaitAlarm for MissedCompare {
            fn arm(&mut self, _: Deadline) -> Result<(), Error> {
                Ok(())
            }
            fn elapsed_after_arm(&self, end: Deadline) -> bool {
                let now = Instant::from_ticks(1 << 32);
                now >= end.instant()
            }
            fn elapsed(&self, end: Deadline) -> bool {
                reached_low(0, end)
            }
            fn deadline_signaled(&self) -> bool {
                false
            }
            fn sleep(&mut self) {
                panic!("slept on a compare tick that was passed while arming");
            }
        }

        let end = Deadline::at(Instant::from_ticks(1));
        assert_eq!(
            wait(&mut MissedCompare, end, |_| None::<()>),
            Err(Error::Timeout)
        );
    }

    #[test]
    fn an_alarm_irq_goes_directly_to_the_final_probe() {
        #[derive(Default)]
        struct DeadlineIrq(bool);
        impl WaitAlarm for DeadlineIrq {
            fn arm(&mut self, _: Deadline) -> Result<(), Error> {
                Ok(())
            }
            fn elapsed(&self, _: Deadline) -> bool {
                assert!(!self.0);
                false
            }
            fn elapsed_after_arm(&self, _: Deadline) -> bool {
                false
            }
            fn deadline_signaled(&self) -> bool {
                self.0
            }
            fn sleep(&mut self) {
                self.0 = true;
            }
        }
        let mut calls = 0;
        let end = Deadline::at(Instant::from_ticks(1));
        assert_eq!(
            wait(&mut DeadlineIrq::default(), end, |final_probe| {
                calls += 1;
                final_probe.then_some(42)
            }),
            Ok(42)
        );
        assert_eq!(calls, 3);
    }

    #[test]
    fn cleanup_is_required_until_retirement_is_verified() {
        let mut attempts = [false, true].into_iter();
        let mut retire = || attempts.next().unwrap();
        assert_eq!(
            clean_before_arm(false, &mut retire),
            Err(Error::AlarmUnavailable)
        );
        assert_eq!(clean_before_arm(false, &mut retire), Ok(()));
        assert_eq!(clean_before_arm(true, retire), Ok(()));
    }

    #[test]
    fn low_word_deadline_handles_wrap_equality_and_max_distance() {
        let end = Deadline::at(Instant::from_ticks((1 << 32) + 1));
        assert!(!reached_low(u32::MAX, end));
        assert!(!reached_low(0, end));
        assert!(reached_low(1, end));
        assert!(reached_low(2, end));
        assert!(!reached_low(1u32.wrapping_sub(MAX_ARM), end));
        assert!(reached_low(1u32.wrapping_add(MAX_ARM), end));
    }

    #[test]
    fn retirement_orders_posted_writes_before_verifying_both_flags() {
        struct Posted(u8);
        impl Registers for Posted {
            fn read(&mut self, address: usize) -> u32 {
                assert_eq!(address, [ARMED, INTR][usize::from(self.0 - 3)]);
                self.0 += 1;
                0
            }
            fn write(&mut self, _: usize, _: u32) {
                panic!("retirement should use posted writes");
            }
            fn write_relaxed(&mut self, address: usize, value: u32) {
                assert_eq!(address, [INTE_CLEAR, ARMED, INTR][usize::from(self.0)]);
                assert_eq!(value, 1 << 1);
                self.0 += 1;
            }
        }

        let mut io = Posted(0);
        assert!(retire_source(&mut io, 1));
        assert_eq!(io.0, 5);
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
        stuck_pending: u32,
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
                INTR => self.pending &= !(bits & !self.stuck_pending),
                INTE_CLEAR => self.enabled &= !bits,
                _ => panic!("unexpected write"),
            }
        }
    }

    #[test]
    fn active_retirement_invalidates_before_mmio_and_caches_only_verified_cleanup() {
        struct Retiring<'a> {
            sources: FakeSources,
            event: &'a AlarmEvent,
        }

        impl Registers for Retiring<'_> {
            fn read(&mut self, address: usize) -> u32 {
                self.sources.read(address)
            }

            fn write(&mut self, address: usize, bits: u32) {
                // An IRQ can enter before the first write takes effect.
                assert!(!self.event.publish(), "retiring generation accepted an IRQ");
                self.sources.write(address, bits);
            }
        }

        static EVENT: AlarmEvent = AlarmEvent(AtomicU32::new(0));
        EVENT.register(2);
        // SAFETY: The host test owns this token and injects register accesses.
        let line = unsafe { Binding::<AlarmIrq<0>, 26, 0>::new() }.into_owned_line();
        let mut timer = WaitTimer {
            alarm: 0,
            clean: false,
            line,
            event: &EVENT,
            generation: 2,
        };
        let mut active = ActiveAlarm {
            timer: &mut timer,
            armed: false,
        };
        let mut io = Retiring {
            sources: FakeSources {
                armed: 1,
                pending: 1,
                enabled: 1,
                stuck: 0,
                stuck_pending: 1,
            },
            event: &EVENT,
        };

        assert!(!active.retire_with(&mut io));
        assert!(!active.timer.clean);
        io.sources.stuck_pending = 0;
        assert!(active.retire_with(&mut io));
        assert!(active.timer.clean);
    }

    #[test]
    fn retire_clears_only_its_source_and_rejects_stale_delivery() {
        let mut io = FakeSources {
            armed: 0b11,
            pending: 0b11,
            enabled: 0b11,
            stuck: 0,
            stuck_pending: 0,
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
        io.stuck = 0;
        io.pending = 1;
        io.stuck_pending = 1;
        assert!(!retire_source(&mut io, 0));
    }
}
