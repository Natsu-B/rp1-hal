//! Core-local delivery for generator-owned synchronous device IRQs.

use crate::timer::{self, Deadline, Timer, WaitTimer};
use rp1_rt::interrupt::{Line, Priority, Slot};

const CEILING: u8 = 4;
const PRIORITY: Priority = Priority::new(CEILING).unwrap();

pub(crate) enum Error {
    Busy,
    IrqUnavailable,
}

pub(crate) trait Source<I> {
    type Error: From<Error> + From<timer::Error>;

    fn is_armed(&self) -> bool;
    fn arm(&mut self, io: &mut I, mask: u32) -> Result<bool, Self::Error>;
    fn service(&mut self, io: &mut I) -> u32;
    fn retire(&mut self, io: &mut I);
}

enum Delivery<const IRQ: u8> {
    Unavailable,
    Unstarted(Line<IRQ>),
    Started(Line<IRQ>),
}

pub(crate) struct Managed<S, const IRQ: u8> {
    line: Delivery<IRQ>,
    source: S,
}

impl<S, const IRQ: u8> Managed<S, IRQ> {
    pub(crate) const fn new(source: S) -> Self {
        Self {
            line: Delivery::Unavailable,
            source,
        }
    }

    // Only the first arm may enable the NVIC line. Later removal is an error.
    fn check_line(&self) -> Result<(), Error> {
        match &self.line {
            Delivery::Unstarted(_) => Ok(()),
            Delivery::Started(line) if line.is_enabled() => Ok(()),
            _ => Err(Error::IrqUnavailable),
        }
    }

    fn start_line(&mut self) {
        if matches!(self.line, Delivery::Unstarted(_))
            && let Delivery::Unstarted(mut line) =
                core::mem::replace(&mut self.line, Delivery::Unavailable)
        {
            line.set_priority(PRIORITY);
            line.enable();
            self.line = Delivery::Started(line);
        }
    }
}

// Each device instance has its own static in core-local DSRAM. Slot also
// checks the core in the generated handler and resource constructor.
pub(crate) type CoreState<S, const IRQ: u8, const CORE: u8> = Slot<Managed<S, IRQ>, CORE, CEILING>;

// Separate statics allow the linker to discard a core's unused slot.
pub(crate) trait Instance<const IRQ: u8>: Sized + 'static {
    const PROC0: &'static CoreState<Self, IRQ, 0>;
    const PROC1: &'static CoreState<Self, IRQ, 1>;
}

pub(crate) struct PerCore<S, const IRQ: u8>(core::marker::PhantomData<S>);

impl<S: Instance<IRQ>, const IRQ: u8> PerCore<S, IRQ> {
    #[inline(always)]
    fn with<const CORE: u8, T>(f: impl FnOnce(&mut Managed<S, IRQ>) -> T) -> Option<T> {
        const { assert!(CORE < 2) };
        if CORE == 0 {
            S::PROC0.try_with(f)
        } else {
            S::PROC1.try_with(f)
        }
    }

    #[inline(always)]
    pub(crate) fn bind<const CORE: u8>(line: Line<IRQ>) {
        let stored = Self::with::<CORE, _>(|state| state.line = Delivery::Unstarted(line));
        debug_assert!(stored.is_some());
    }

    #[inline(always)]
    pub(crate) fn check<const CORE: u8>() -> Result<(), Error> {
        Self::with::<CORE, _>(|state| state.check_line()).ok_or(Error::Busy)?
    }

    #[inline(always)]
    pub(crate) fn on_interrupt<const CORE: u8, I>(io: &mut I) -> u32
    where
        S: Source<I>,
    {
        let pending = Self::with::<CORE, _>(|state| state.source.service(io)).unwrap_or(0);
        if pending != 0 {
            rp1_rt::send_event();
        }
        pending
    }

    #[inline]
    pub(crate) fn wait_for<const CORE: u8, I, T>(
        io: &mut I,
        wait: &mut WaitTimer,
        end: Deadline,
        mask: u32,
        mut probe: impl FnMut(&mut I) -> Option<Result<T, S::Error>>,
    ) -> Result<T, S::Error>
    where
        S: Source<I>,
    {
        let mut context = wait.thread_context()?;
        Self::check::<CORE>()?;
        let mut active = false;
        let mut post_alarm_probe = true;
        // Register the device inside the timer's checked probe so an already
        // expired call neither touches the peripheral nor enables its IRQ.
        let result = context.wait_until(end, || {
            if active && !core::mem::take(&mut post_alarm_probe) {
                let fired =
                    Self::with::<CORE, _>(|state| !state.source.is_armed()).unwrap_or(false);
                if !fired && Timer::new().now() < end.instant() {
                    // Unrelated events do not turn an armed wait into polling.
                    return None;
                }
            }
            probe_and_arm(io, &mut probe, |io| {
                if Timer::new().now() >= end.instant() {
                    return Ok(false);
                }
                let armed = Self::with::<CORE, _>(|state| {
                    let armed = state.source.arm(io, mask)?;
                    if armed {
                        state.start_line();
                    }
                    Ok::<_, S::Error>(armed)
                })
                .ok_or(Error::Busy)??;
                active |= armed;
                Ok(armed)
            })
        });
        if active {
            // Thread admission prevents overlapping borrows with the ISR.
            Self::with::<CORE, _>(|state| state.source.retire(io)).ok_or(Error::Busy)?;
        }
        result?
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
            let result = timer::test_wait(&mut Alarm(&f, arrival), end, || {
                probe_and_arm(
                    &mut (),
                    |_| f.ready.replace(false).then_some(Ok::<_, timer::Error>(())),
                    |_| {
                        if f.arms.get() != 0 || f.sleeps.get() == 2 {
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
