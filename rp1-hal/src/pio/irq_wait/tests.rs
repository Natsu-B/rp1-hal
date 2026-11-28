use super::*;
use crate::{
    pio::{FifoSource, WaitError},
    scmi::Registers,
    timer::{Instant, WaitAlarm},
};
use core::cell::Cell;

const IRQ0_INTE: usize = 0xf000_0150;
const INTS_OFFSET: usize = 8;
const RX: u32 = 1 << 1;
const NEIGHBOR: u32 = 1 << 2;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Arrival {
    BeforeArm,
    DuringArm,
    AlarmArm,
    BeforeSleep,
    AfterUnrelatedEvent,
    AtDeadline,
    DeadlineFirst,
    Timeout,
}

struct Model {
    arrival: Arrival,
    now: Cell<u64>,
    inte: Cell<u32>,
    raw: Cell<u32>,
    word: Cell<Option<u32>>,
    probes: Cell<u32>,
    arms: Cell<u32>,
    wakes: Cell<u32>,
    event: Cell<bool>,
}

impl Model {
    fn new(arrival: Arrival) -> Self {
        Self {
            arrival,
            now: Cell::new(1),
            inte: Cell::new(NEIGHBOR),
            // Keep a neighboring source high to check that servicing RX preserves its mask.
            raw: Cell::new(NEIGHBOR),
            word: Cell::new(None),
            probes: Cell::new(0),
            arms: Cell::new(0),
            wakes: Cell::new(0),
            event: Cell::new(false),
        }
    }

    fn receive(&self) {
        self.word.set(Some(42));
        self.raw.set(self.raw.get() | RX);
    }

    fn time(&self) -> Instant {
        Instant::from_ticks(self.now.get())
    }

    fn irq(&self, slot: &TestSlot) {
        let pending = service_source!(slot, &mut Io(self), || {
            assert_eq!(self.inte.get(), NEIGHBOR);
            self.event.set(true);
            self.wakes.set(self.wakes.get() + 1);
        });
        assert_eq!(pending, RX);
    }
}

struct Io<'a>(&'a Model);

impl Io<'_> {
    fn probe(&mut self) -> Option<Result<u32, WaitError>> {
        self.0.probes.set(self.0.probes.get() + 1);
        self.0.word.take().map(|word| {
            self.0.raw.set(self.0.raw.get() & !RX);
            Ok(word)
        })
    }
}

impl Registers for Io<'_> {
    fn read(&mut self, address: usize) -> u32 {
        match address {
            IRQ0_INTE => self.0.inte.get(),
            a if a == IRQ0_INTE + INTS_OFFSET => self.0.raw.get() & self.0.inte.get(),
            _ => panic!("unexpected PIO register {address:#x}"),
        }
    }

    fn write(&mut self, address: usize, value: u32) {
        assert_eq!(address, IRQ0_INTE);
        let prior = self.0.inte.replace(value);
        if value & !prior & RX != 0 {
            self.0.arms.set(self.0.arms.get() + 1);
            if self.0.arrival == Arrival::DuringArm {
                self.0.receive();
            }
        }
    }
}

struct Line(bool);

impl DeviceLine for Line {
    fn is_enabled(&self) -> bool {
        self.0
    }
    fn start(&mut self) {
        assert!(!self.0);
        self.0 = true;
    }
}

type TestSlot = CoreState<FifoSource, Line, 0>;

struct Alarm<'a> {
    model: &'a Model,
    slot: &'a TestSlot,
    end: Deadline,
    armed: bool,
    sleep_count: u8,
}

impl WaitAlarm for Alarm<'_> {
    fn arm(&mut self, _: Deadline) -> Result<(), timer::Error> {
        assert!(!self.armed);
        self.armed = true;
        if self.model.arrival == Arrival::AlarmArm {
            self.model.receive();
            self.model.irq(self.slot);
        }
        Ok(())
    }

    fn elapsed(&self, end: Deadline) -> bool {
        self.model.time() >= end.instant()
    }

    fn elapsed_after_arm(&self, end: Deadline) -> bool {
        self.elapsed(end)
    }

    fn deadline_signaled(&self) -> bool {
        self.elapsed(self.end)
    }

    fn sleep(&mut self) {
        self.sleep_count += 1;
        assert!(self.sleep_count <= 2);
        match self.model.arrival {
            Arrival::AfterUnrelatedEvent if self.sleep_count == 1 => (),
            Arrival::BeforeSleep | Arrival::AfterUnrelatedEvent => {
                self.model.receive();
                self.model.irq(self.slot);
                assert!(self.model.event.replace(false), "WFE lost the device event");
            }
            Arrival::AtDeadline => {
                self.model.receive();
                self.model.irq(self.slot);
                self.model.now.set(self.end.instant().ticks());
            }
            Arrival::DeadlineFirst => {
                self.model.receive();
                self.model.now.set(self.end.instant().ticks());
            }
            Arrival::Timeout => self.model.now.set(self.end.instant().ticks()),
            arrival => panic!("unexpected WFE for {arrival:?}"),
        }
    }
}

impl Alarm<'_> {
    fn wait<T>(
        &mut self,
        end: Deadline,
        probe: impl FnMut(bool) -> Option<T>,
    ) -> Result<T, timer::Error> {
        timer::test_wait(self, end, probe)
    }
}

fn run(slot: &TestSlot, model: &Model) -> (Result<u32, WaitError>, bool, u8) {
    let end = Deadline::at(Instant::from_ticks(10));
    let mut alarm = Alarm {
        model,
        slot,
        end,
        armed: false,
        sleep_count: 0,
    };
    if model.arrival == Arrival::BeforeArm {
        model.receive();
    }
    let io = &mut Io(model);
    let mut probe = |io: &mut Io<'_>| io.probe();
    let result = (|| {
        wait_source! {
            state: slot, io: io,
            alarm: (Alarm::wait, &mut alarm), deadline: end,
            sources: RX, probe: probe,
            now: model.time(), error: WaitError,
        }
    })();
    (result, alarm.armed, alarm.sleep_count)
}

#[test]
fn fifo_wait_races_use_the_shared_waiter_and_retire_the_source() {
    for (arrival, armed, sleeps, probes, wakes) in [
        (Arrival::BeforeArm, false, 0, 1, 0),
        (Arrival::DuringArm, false, 0, 2, 0),
        (Arrival::AlarmArm, true, 0, 3, 1),
        (Arrival::BeforeSleep, true, 1, 4, 1),
        (Arrival::AfterUnrelatedEvent, true, 2, 4, 1),
        (Arrival::AtDeadline, true, 1, 4, 1),
        (Arrival::DeadlineFirst, true, 1, 4, 0),
        (Arrival::Timeout, true, 1, 4, 0),
    ] {
        let slot = Slot::new(Managed::new(FifoSource::new(IRQ0_INTE)));
        bind(&slot, Line(false));
        let model = Model::new(arrival);
        let expected = if arrival == Arrival::Timeout {
            Err(timer::Error::Timeout.into())
        } else {
            Ok(42)
        };
        assert_eq!(run(&slot, &model), (expected, armed, sleeps), "{arrival:?}");
        assert_eq!(model.probes.get(), probes, "{arrival:?}");
        assert_eq!(model.wakes.get(), wakes);
        assert_eq!(model.arms.get(), u32::from(arrival != Arrival::BeforeArm));
        assert_eq!(model.inte.get(), NEIGHBOR);
        assert_eq!(model.word.get(), None);
    }
}

#[test]
fn a_timeout_rearms_the_same_source_and_a_stuck_level_cannot_redeliver() {
    let timeout = Model::new(Arrival::Timeout);
    let slot = Slot::new(Managed::new(FifoSource::new(IRQ0_INTE)));
    bind(&slot, Line(false));
    assert_eq!(run(&slot, &timeout).0, Err(timer::Error::Timeout.into()));

    // The retired source can assert without a new delivery even with INTE's
    // neighboring source high. Preserve the neighbor's mask throughout.
    timeout.receive();
    assert_eq!(on_interrupt(&slot, &mut Io(&timeout)), 0);
    assert_eq!(timeout.inte.get(), NEIGHBOR);
    assert_eq!(run(&slot, &Model::new(Arrival::BeforeSleep)).0, Ok(42));

    let model = Model::new(Arrival::BeforeSleep);
    let mut source = FifoSource::new(IRQ0_INTE);
    let mut io = Io(&model);
    // Exercise the ISR mask logic with two armed bits in the same snapshot.
    assert!(source.arm(&mut io, RX | NEIGHBOR).unwrap());
    model.receive();
    assert_eq!(source.service(&mut io), RX | NEIGHBOR);
    assert_eq!(model.raw.get(), RX | NEIGHBOR);
    assert_eq!(source.service(&mut io), 0);
    assert_eq!(model.inte.get(), 0);
}
