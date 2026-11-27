use super::*;
use crate::{
    adc::tests::FakeAdc,
    timer::{Instant, WaitAlarm, test_wait},
};
use core::cell::RefCell;

#[derive(Clone, Copy)]
enum Arrival {
    DuringArm,
    AfterArm,
    Sleeping,
    FifoBeforeReady,
    MissingWithResult,
    MissingWithoutResult,
}

struct Fixture<'a> {
    source: FifoSource,
    io: WaitIo<'a, FakeAdc>,
    wakes: usize,
    deliveries: u32,
}

struct Alarm<'a, 'b> {
    fixture: &'a RefCell<Fixture<'b>>,
    arrival: Arrival,
}

impl WaitAlarm for Alarm<'_, '_> {
    fn arm(&mut self, _: Deadline) -> Result<(), timer::Error> {
        if matches!(self.arrival, Arrival::AfterArm) {
            let mut f = self.fixture.borrow_mut();
            let Fixture {
                source,
                io,
                deliveries,
                ..
            } = &mut *f;
            io.regs.complete(77);
            *deliveries += source.service(io);
        }
        Ok(())
    }

    fn elapsed(&self, _: Deadline) -> bool {
        self.fixture.borrow().wakes == 2
    }

    fn sleep(&mut self) {
        let mut f = self.fixture.borrow_mut();
        let Fixture {
            source,
            io,
            wakes,
            deliveries,
        } = &mut *f;
        *wakes += 1;
        match (self.arrival, *wakes) {
            (Arrival::Sleeping, 2) => {
                io.regs.complete(77);
                *deliveries += source.service(io);
            }
            (Arrival::FifoBeforeReady, 1) => {
                assert!(!ready(io.regs.cs));
                io.regs.samples.push_back(77);
                *deliveries += source.service(io);
            }
            (Arrival::MissingWithResult, 2) => io.regs.complete(77),
            _ => {}
        }
    }
}

#[test]
fn stale_before_arm_and_completion_during_after_or_without_delivery() {
    for (arrival, wakes, deliveries, result) in [
        (Arrival::DuringArm, 0, 0, Ok(Ok(77))),
        (Arrival::AfterArm, 0, 1, Ok(Ok(77))),
        (Arrival::Sleeping, 2, 1, Ok(Ok(77))),
        (Arrival::FifoBeforeReady, 1, 1, Ok(Ok(77))),
        (Arrival::MissingWithResult, 2, 0, Ok(Ok(77))),
        (
            Arrival::MissingWithoutResult,
            2,
            0,
            Err(timer::Error::Timeout),
        ),
    ] {
        let mut state = State::Idle;
        let mut regs = FakeAdc::default();
        regs.samples.push_back(999); // FIFO already high before arm; old generation.
        if matches!(arrival, Arrival::DuringArm) {
            regs.on_start = Some(77);
        }
        let fixture = RefCell::new(Fixture {
            source: FifoSource { armed: false },
            io: WaitIo {
                regs,
                request: Some((&mut state, 4)),
                prepared: false,
            },
            wakes: 0,
            deliveries: 0,
        });
        let actual = test_wait(
            &mut Alarm {
                fixture: &fixture,
                arrival,
            },
            Deadline::at(Instant::from_ticks(1)),
            |final_probe| {
                let mut f = fixture.borrow_mut();
                let Fixture {
                    source, io, wakes, ..
                } = &mut *f;
                if final_probe {
                    return io.probe();
                }
                if source.armed && io.regs.starts != 0 {
                    return None; // The same unrelated-wake guard as PerCore::wait_for.
                }
                crate::driver_irq::probe_and_arm(io, WaitIo::probe, |io| {
                    if *wakes == 2 {
                        Ok(false)
                    } else {
                        source.arm(io, FIFO_IRQ)
                    }
                })
            },
        );
        assert_eq!(actual, result);
        let Fixture {
            mut source,
            mut io,
            wakes: actual_wakes,
            deliveries: actual_deliveries,
        } = fixture.into_inner();
        source.retire(&mut io);
        assert_eq!(io.regs.starts, 1);
        assert_eq!(io.regs.fifo_reads, if result.is_ok() { 2 } else { 1 });
        assert_eq!(io.regs.inte, 0);
        assert_eq!(actual_wakes, wakes);
        assert_eq!(actual_deliveries, deliveries);
        if result.is_err() {
            let WaitIo { mut regs, .. } = io;
            assert_eq!(state.cancel(&mut regs), Err(Error::Recovering));
            regs.complete(999);
            state.cancel(&mut regs).unwrap();
            state.start(&mut regs, 4).unwrap();
            regs.complete(55);
            assert_eq!(state.sample(&mut regs), Ok(Some(55)));
        }
    }
}

#[test]
fn stale_entry_and_force_cannot_publish_or_storm() {
    let mut source = FifoSource { armed: false };
    let mut state = State::Idle;
    let mut io = WaitIo {
        regs: FakeAdc::default(),
        request: Some((&mut state, 4)),
        prepared: false,
    };
    assert_eq!(source.service(&mut io), 0);
    io.regs.inte = FIFO_IRQ;
    io.regs.samples.push_back(999);
    assert_eq!(source.service(&mut io), 0);
    assert_eq!(io.regs.inte, 0);
    assert_eq!(io.probe(), None);
    io.regs.intf = FIFO_IRQ;
    assert!(source.arm(&mut io, FIFO_IRQ).unwrap());
    assert_eq!(source.service(&mut io), 0);
    assert_eq!(io.regs.read(INTS), 0);
    assert!(source.armed);
    io.regs.complete(77);
    assert_eq!(source.service(&mut io), FIFO_IRQ);
    assert_eq!(source.service(&mut io), 0);
    assert_eq!(io.probe(), Some(Ok(77)));
    assert_eq!(io.regs.fifo_reads, 2);
}
