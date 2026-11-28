//! Core-local delivery for generator-owned synchronous device IRQs.

use crate::timer::{self, Deadline, Timer, WaitTimer};
use rp1_rt::interrupt::{Line, OwnedLine, Priority, Slot};

const CEILING: u8 = 4;
const PRIORITY: Priority = Priority::new(CEILING).unwrap();

pub(crate) enum Error {
    Busy,
    IrqUnavailable,
}

pub(crate) enum Limit {
    Absolute(Deadline),
    /// An admitted relative deadline. The first probe may prepare the device
    /// after expiry. The arm path checks the deadline before launching work.
    Relative(Deadline),
}

pub(crate) trait Source<I> {
    type Error: From<Error> + From<timer::Error>;
    // UART only checks here. The other serial blocks reuse their transfer preflight.
    const INLINE_CHECK: bool = false;

    fn is_armed(&self) -> bool;
    fn arm(&mut self, io: &mut I, mask: u32) -> Result<bool, Self::Error>;
    fn service(&mut self, io: &mut I) -> u32;
    fn retire(&mut self, io: &mut I);
}

pub(crate) trait DeviceLine {
    fn is_enabled(&self) -> bool;
    fn start(&mut self);
}

impl<const IRQ: u8> DeviceLine for Line<IRQ> {
    fn is_enabled(&self) -> bool {
        self.is_enabled()
    }

    fn start(&mut self) {
        self.set_priority(PRIORITY);
        self.enable();
    }
}

pub(crate) struct SlotLine(OwnedLine);

// The line stays inside the private Slot, which only lends it on its owning core.
unsafe impl Send for SlotLine {}

impl SlotLine {
    pub(crate) fn new<S, const IRQ: u8, const CORE: u8>(
        line: rp1_rt::interrupt::Binding<S, IRQ, CORE>,
    ) -> Self {
        Self(line.into_owned_line())
    }
}

impl DeviceLine for SlotLine {
    fn is_enabled(&self) -> bool {
        self.0.is_enabled()
    }

    fn start(&mut self) {
        self.0.set_priority(PRIORITY);
        self.0.enable();
    }
}

struct Delivery<L> {
    line: Option<L>,
    started: bool,
}

impl<L: DeviceLine> Delivery<L> {
    // Only the first arm may enable the NVIC line. Later removal is an error.
    fn check(&self) -> Result<(), Error> {
        match &self.line {
            Some(line) if !self.started || line.is_enabled() => Ok(()),
            _ => Err(Error::IrqUnavailable),
        }
    }

    // First arm alone programs NVIC delivery, outside the steady wake path.
    #[rp1_rt::cold]
    fn start(&mut self) {
        if let Some(line) = &mut self.line {
            line.start();
            self.started = true;
        }
    }
}

pub(crate) struct Managed<S, L> {
    delivery: Delivery<L>,
    source: S,
}

impl<S, L: DeviceLine> Managed<S, L> {
    pub(crate) const fn new(source: S) -> Self {
        Self {
            delivery: Delivery {
                line: None,
                started: false,
            },
            source,
        }
    }
}

pub(crate) type CoreState<S, L, const CORE: u8> = Slot<Managed<S, L>, CORE, CEILING>;

#[inline(always)]
pub(crate) fn bind<S, L: DeviceLine, const CORE: u8>(slot: &CoreState<S, L, CORE>, line: L) {
    let stored = slot.try_with(|state| {
        state.delivery = Delivery {
            line: Some(line),
            started: false,
        };
    });
    debug_assert!(stored.is_some());
}

#[inline(always)]
fn check<S, L: DeviceLine, const CORE: u8>(slot: &CoreState<S, L, CORE>) -> Result<(), Error> {
    slot.__try_with_compact(|state| state.delivery.check())
        .ok_or(Error::Busy)?
}

// Select the Source implementation for the same register backend as arm and retire.
#[inline(always)]
fn is_armed<I>(source: &impl Source<I>, _: &I) -> bool {
    source.is_armed()
}

#[inline(never)]
pub(crate) fn check_before_transfer<S, L: DeviceLine, const CORE: u8>(
    slot: &CoreState<S, L, CORE>,
) -> Result<(), Error> {
    check(slot)
}

// Keep the same ISR body in firmware and the host race fixture.
macro_rules! service_source {
    ($slot:ident, $io:expr, $signal:expr) => {{
        let pending = $slot
            .try_with(|state| state.source.service($io))
            .unwrap_or(0);
        if pending != 0 {
            $signal();
        }
        pending
    }};
}

// Share the wait body with the host model. Expanding it at the call site
// preserves the size of existing serial waiters.
macro_rules! wait_source {
    (state: $slot:ident, io: $io:ident,
     alarm: $context:ident.$wait:ident, deadline: $end:ident,
     sources: $mask:expr, probe: $probe:ident, now: $now:expr, error: $error:ty $(,)?) => {{
        let mut active = false;
        let mut post_alarm_probe = true;
        let result = $context.$wait($end, |final_probe| {
            if final_probe {
                return $probe($io);
            }
            if active {
                // The first probe after alarm arm closes the arm/check race.
                // If it misses, a device IRQ can still leave an event for WFE.
                if core::mem::take(&mut post_alarm_probe) {
                    return $probe($io);
                }
                let fired = $slot
                    .__try_with_compact(|state| !is_armed(&state.source, $io))
                    .unwrap_or(false);
                if !fired {
                    // Unrelated events do not turn an armed wait into polling.
                    return None;
                }
            }
            $crate::driver_irq::probe_and_arm($io, &mut $probe, |io| {
                // Arming can start device work. A preemption before that
                // point can span a timer wrap.
                if $now >= $end.instant() {
                    return Ok(false);
                }
                let armed = $slot
                    .__try_with_compact(|state| {
                        let armed = state.source.arm(io, $mask)?;
                        if armed && !state.delivery.started {
                            state.delivery.start();
                        }
                        Ok::<_, $error>(armed)
                    })
                    .ok_or($crate::driver_irq::Error::Busy)??;
                active |= armed;
                Ok(armed)
            })
        });
        if active {
            // Thread admission prevents overlapping borrows with the ISR.
            $slot
                .__try_with_compact(|state| state.source.retire($io))
                .ok_or($crate::driver_irq::Error::Busy)?;
        }
        result?
    }};
}

#[cfg(test)]
#[path = "pio/irq_wait/tests.rs"]
mod pio_tests;

#[inline(always)]
pub(crate) fn on_interrupt<S: Source<I>, L: DeviceLine, const CORE: u8, I>(
    slot: &CoreState<S, L, CORE>,
    io: &mut I,
) -> u32 {
    service_source!(slot, io, rp1_rt::send_event)
}

#[inline]
pub(crate) fn wait_for<S: Source<I>, L: DeviceLine, const CORE: u8, I, T>(
    slot: &CoreState<S, L, CORE>,
    io: &mut I,
    wait: &mut WaitTimer,
    limit: Limit,
    mask: u32,
    mut probe: impl FnMut(&mut I) -> Option<Result<T, S::Error>>,
) -> Result<T, S::Error> {
    let mut context = wait.thread_context()?;
    if S::INLINE_CHECK {
        check(slot)?;
    } else {
        check_before_transfer(slot)?;
    }
    let end = match limit {
        Limit::Absolute(end) => {
            end.check(Timer::new().now())?;
            end
        }
        // Relative calls already checked the deadline at their entry tick.
        Limit::Relative(end) => end,
    };
    wait_source! {
        state: slot, io: io,
        alarm: context.wait_accepted, deadline: end,
        sources: mask, probe: probe,
        now: Timer::new().now(), error: S::Error,
    }
}
pub(crate) fn probe_and_arm<I, T, E>(
    io: &mut I,
    mut probe: impl FnMut(&mut I) -> Option<Result<T, E>>,
    arm: impl FnOnce(&mut I) -> Result<bool, E>,
) -> Option<Result<T, E>> {
    if let Some(ready) = probe(io) {
        return Some(ready);
    }
    match arm(io) {
        Ok(true) => probe(io),
        Ok(false) => None,
        Err(error) => Some(Err(error)),
    }
}

// Only declared slots and thin entry points are linked. Each block's waiter
// takes a runtime slot and MMIO address, regardless of the instance number.
macro_rules! device_irq {
    ($source:ident, $io:ident, $wait:path, ($($arg:ident: $ty:ty),+) -> $output:ty;
     $($n:literal => $block:ident, $vector:ident;)+) => {
        type IrqState<const CORE: u8> =
            $crate::driver_irq::CoreState<$source, $crate::driver_irq::SlotLine, CORE>;

        #[inline(always)]
        fn irq_state0<const N: u8>() -> &'static IrqState<0> {
            const { assert!($(N == $n)||+) };
            match N {
                $($n => {
                    static STATE: IrqState<0> =
                        rp1_rt::interrupt::Slot::new($crate::driver_irq::Managed::new($source::new($n)));
                    &STATE
                },)+
                _ => unreachable!(),
            }
        }

        #[inline(always)]
        fn irq_state1<const N: u8>() -> &'static IrqState<1> {
            const { assert!($(N == $n)||+) };
            match N {
                $($n => {
                    static STATE: IrqState<1> =
                        rp1_rt::interrupt::Slot::new($crate::driver_irq::Managed::new($source::new($n)));
                    &STATE
                },)+
                _ => unreachable!(),
            }
        }

        $(
            impl<P: CompletePins<$n>, const CORE: u8> Parts<$n, P, Irq<$n, CORE>> {
                /// # Safety
                /// The generator must install the matching handler on CORE
                /// and construct this binding exactly once.
                #[doc(hidden)]
                pub unsafe fn __new(
                    token: Token<$n>,
                    pins: Pins<$n, P>,
                    line: rp1_rt::interrupt::Binding<
                        rp1_rt::interrupt::BlockIrq<{ rp1_proto::BlockId::$block as u16 }>,
                        { $crate::pac::Interrupt::$vector as u8 },
                        CORE,
                    >,
                ) -> Self {
                    const { assert!(CORE < 2) };
                    let line = $crate::driver_irq::SlotLine::new(line);
                    if CORE == 0 {
                        $crate::driver_irq::bind(irq_state0::<$n>(), line);
                    } else {
                        $crate::driver_irq::bind(irq_state1::<$n>(), line);
                    }
                    Self { token, pins, irq: Irq(PhantomData) }
                }
            }
        )+

        // SPI and I2C also check before starting a transfer. UART checks in the waiter.
        #[allow(dead_code)]
        #[inline(always)]
        fn check_irq<const N: u8, const CORE: u8>() -> Result<(), $crate::driver_irq::Error> {
            if CORE == 0 {
                $crate::driver_irq::check_before_transfer(irq_state0::<N>())
            } else {
                $crate::driver_irq::check_before_transfer(irq_state1::<N>())
            }
        }

        #[inline(always)]
        fn service_irq<const N: u8, const CORE: u8, I: $io>(io: &mut I) -> u32 {
            const { assert!(CORE < 2) };
            if CORE == 0 {
                $crate::driver_irq::on_interrupt(irq_state0::<N>(), io)
            } else {
                $crate::driver_irq::on_interrupt(irq_state1::<N>(), io)
            }
        }

        #[inline(always)]
        fn wait_for_irq<const N: u8, const CORE: u8>($($arg: $ty),+) -> $output {
            if CORE == 0 {
                $wait(irq_state0::<N>(), $($arg),+)
            } else {
                $wait(irq_state1::<N>(), $($arg),+)
            }
        }

        #[cfg(test)]
        const IRQ_ROUTES: &[(u8, u16, u8)] = &[
            $(($n, rp1_proto::BlockId::$block as u16, $crate::pac::Interrupt::$vector as u8),)+
        ];
    }
}

pub(crate) use device_irq;

#[cfg(test)]
mod tests {
    use super::*;
    use core::cell::Cell;

    #[test]
    fn data_on_either_side_of_arm_is_consumed_once() {
        for phase in 0..3 {
            let ready = Cell::new(phase == 0);
            let arms = Cell::new(0);
            let read = |_: &mut ()| ready.replace(false).then_some(Ok::<_, timer::Error>(42));
            let arm = |_: &mut ()| {
                arms.set(arms.get() + 1);
                if phase == 1 {
                    ready.set(true);
                }
                Ok(true)
            };
            let first = probe_and_arm(&mut (), read, arm);
            if phase == 2 {
                assert_eq!(first, None);
                ready.set(true);
                assert_eq!(probe_and_arm(&mut (), read, arm), Some(Ok(42)));
            } else {
                assert_eq!(first, Some(Ok(42)));
            }
            assert_eq!(arms.get(), u8::from(phase != 0));
            assert!(!ready.get());
        }
    }

    #[derive(Clone, Copy)]
    enum Arrival {
        BeforeArm,
        DuringArm,
        AfterArm,
        DuringSleep,
        AtDeadline,
        Missing,
    }

    #[derive(Default)]
    struct Fixture {
        ready: Cell<bool>,
        arms: Cell<u8>,
        sleeps: Cell<u8>,
        event: Cell<bool>,
    }

    struct Alarm<'a>(&'a Fixture, Arrival);

    impl timer::WaitAlarm for Alarm<'_> {
        fn arm(&mut self, _: Deadline) -> Result<(), timer::Error> {
            if matches!(self.1, Arrival::AfterArm) {
                self.0.ready.set(true);
                self.0.event.set(true);
            }
            Ok(())
        }

        fn elapsed(&self, _: Deadline) -> bool {
            self.0.sleeps.get() == 2
        }

        fn elapsed_after_arm(&self, _: Deadline) -> bool {
            false
        }

        fn deadline_signaled(&self) -> bool {
            false
        }

        fn sleep(&mut self) {
            // A first unrelated event may wake the waiter. The second wake is the deadline.
            let sleeps = self.0.sleeps.get() + 1;
            self.0.sleeps.set(sleeps);
            match (self.1, sleeps) {
                (Arrival::DuringSleep, 1) => {
                    self.0.ready.set(true);
                    self.0.event.set(true);
                }
                (Arrival::AtDeadline, 2) => self.0.ready.set(true),
                _ => {}
            }
        }
    }

    #[test]
    fn irq_arrival_before_during_after_arm_or_at_deadline() {
        for (arrival, sleeps) in [
            (Arrival::BeforeArm, 0),
            (Arrival::DuringArm, 0),
            (Arrival::AfterArm, 0),
            (Arrival::DuringSleep, 1),
            (Arrival::AtDeadline, 2),
            (Arrival::Missing, 2),
        ] {
            let f = Fixture::default();
            f.ready.set(matches!(arrival, Arrival::BeforeArm));
            let end = Deadline::at(timer::Instant::from_ticks(1));
            let result = timer::test_wait(&mut Alarm(&f, arrival), end, |final_probe| {
                probe_and_arm(
                    &mut (),
                    |_| f.ready.replace(false).then_some(Ok::<_, timer::Error>(())),
                    |_| {
                        if final_probe || f.arms.get() != 0 || f.sleeps.get() == 2 {
                            return Ok(false);
                        }
                        f.arms.set(1);
                        if matches!(arrival, Arrival::DuringArm) {
                            f.ready.set(true);
                            f.event.set(true);
                        }
                        Ok(true)
                    },
                )
            });
            assert_eq!(f.sleeps.get(), sleeps);
            assert_eq!(
                f.arms.get(),
                u8::from(!matches!(arrival, Arrival::BeforeArm))
            );
            assert_eq!(
                f.event.get(),
                matches!(
                    arrival,
                    Arrival::DuringArm | Arrival::AfterArm | Arrival::DuringSleep
                )
            );
            assert_eq!(
                result,
                if matches!(arrival, Arrival::Missing) {
                    Err(timer::Error::Timeout)
                } else {
                    Ok(Ok(()))
                }
            );
        }
    }

    #[test]
    fn deadline_probe_preserves_completion_without_starting_a_new_wait() {
        // A UART IRQ was never delivered. The deadline alarm gets one final
        // observation; it can see retained data or finish with Timeout.
        assert_eq!(
            probe_and_arm(
                &mut (),
                |_| Some(Ok::<_, timer::Error>(42)),
                |_| panic!("rearmed at deadline")
            ),
            Some(Ok(42))
        );
        assert_eq!(
            probe_and_arm(&mut (), |_| None::<Result<u8, timer::Error>>, |_| Ok(false)),
            None
        );
    }
}
