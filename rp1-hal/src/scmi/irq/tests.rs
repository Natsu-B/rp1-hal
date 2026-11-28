extern crate std;

use super::*;
use crate::scmi::{
    self, BASE_PROTOCOL, CLOCK_PROTOCOL, FLAGS, FREE, HEADER, LENGTH, PAYLOAD, STATUS, Status, id,
};
use std::{cell::RefCell, collections::BTreeMap, rc::Rc, vec::Vec};

const FREE_WORD: usize = SHMEM + 4 * STATUS;
const UART: usize = 0x4001_8054;
const BASE_VERSION: Request = Request::new((BASE_PROTOCOL as u32) << 10, &[]);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Context {
    Init,
    Thread,
    Irq,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Op {
    Read(usize, u32),
    Write(usize, u32),
    Barrier,
    /// IRQ57's read of IRQ53's entry count, and the index of the access boundary it passed.
    Entries53 {
        value: u32,
        boundary: usize,
    },
}

#[derive(Default)]
struct Bus {
    words: BTreeMap<usize, u32>,
    trace: Vec<(Context, Op)>,
    now: u32,
}

impl Bus {
    fn get(&self, address: usize) -> u32 {
        self.words.get(&address).copied().unwrap_or(0)
    }
    fn record(&mut self, context: Context, op: Op) {
        self.now = self.now.wrapping_add(1);
        self.trace.push((context, op));
    }
}

struct ClockBus(Rc<RefCell<Bus>>);

impl Registers for ClockBus {
    fn read(&mut self, address: usize) -> u32 {
        let mut bus = self.0.borrow_mut();
        let value = bus.get(address);
        bus.record(Context::Irq, Op::Read(address, value));
        value
    }
    fn write(&mut self, address: usize, value: u32) {
        let mut bus = self.0.borrow_mut();
        bus.record(Context::Irq, Op::Write(address, value));
        bus.words.insert(address, value);
    }
}

impl ClockHardware for ClockBus {
    fn fixed_state(&mut self, id: u32) -> Result<super::super::PhysicalState, Status> {
        assert_eq!(id, id::UART);
        let rate_hz = crate::clocks::uart_hz_with(&mut |addr| self.read(addr))
            .ok_or(Status::HardwareError)?;
        Ok(scmi::PhysicalState {
            rate_hz,
            enabled: true,
        })
    }
}

#[derive(Clone, Copy)]
struct Request {
    header: u32,
    args: &'static [u32],
}

impl Request {
    const fn new(header: u32, args: &'static [u32]) -> Self {
        Self { header, args }
    }
}

#[derive(Clone, Copy)]
enum Delivery {
    Pulse,
    AtReturn,
    Continuous,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Trigger {
    Enabled,
    BadLengthAfterFree,
    ThreadAccess(usize),
}

struct FakeRp1 {
    bus: Rc<RefCell<Bus>>,
    server: Option<Server<'static, ClockBus>>,
    counters: Counters,
    delivery: Delivery,
    context: Context,
    enabled: bool,
    pending: bool,
    force_level: bool,
    entries: usize,
    inject: Option<(Trigger, Request)>,
    thread_accesses: usize,
    /// IRQ53's entry count, and the IRQ57 access before which one IRQ53 entry preempts.
    entries53: u32,
    irq53_before: Option<usize>,
    irq_accesses: usize,
}

impl FakeRp1 {
    fn ready(delivery: Delivery, held: u32) -> Self {
        let bus = Rc::new(RefCell::new(Bus::default()));
        let mut f = Self {
            server: None,
            bus,
            counters: Counters::default(),
            delivery,
            context: Context::Init,
            enabled: false,
            pending: false,
            force_level: false,
            entries: 0,
            inject: None,
            thread_accesses: 0,
            entries53: 0,
            irq53_before: None,
            irq_accesses: 0,
        };
        let clocks = ClockBus(f.bus.clone());
        f.server = Some(prepare_server(&mut f, clocks, held));
        f
    }
    fn get(&self, address: usize) -> u32 {
        self.bus.borrow().get(address)
    }
    fn set(&mut self, address: usize, value: u32) {
        self.bus.borrow_mut().words.insert(address, value);
    }
    fn kick(&mut self) {
        let events = self.get(PROC_EVENTS);
        self.set(PROC_EVENTS, events | 1);
        if self.get(PROC_GATE) & 1 != 0 && events & 1 == 0 {
            self.pending = true;
        }
    }
    fn request(&mut self, r: Request) {
        assert_eq!(self.get(FREE_WORD), FREE);
        for (word, value) in [
            (STATUS, 0),
            (HEADER, r.header),
            (LENGTH, 4 + 4 * r.args.len() as u32),
            (FLAGS, 1),
        ] {
            self.set(SHMEM + 4 * word, value);
        }
        for (i, value) in r.args.iter().enumerate() {
            self.set(SHMEM + 4 * (PAYLOAD + i), *value);
        }
        self.kick();
    }
    fn inject_at(&mut self, trigger: Trigger) {
        if let Some((at, request)) = self.inject
            && at == trigger
        {
            self.inject = None;
            self.request(request);
            if trigger == Trigger::BadLengthAfterFree {
                self.set(SHMEM + 4 * LENGTH, 5);
                self.set(SHMEM + 4 * FLAGS, 0);
            }
        }
    }
    fn level(&self) -> bool {
        self.force_level || self.get(PROC_GATE) & self.get(PROC_EVENTS) & 1 != 0
    }
    fn boundary(&mut self) {
        if self.context == Context::Thread {
            self.inject_at(Trigger::ThreadAccess(self.thread_accesses));
            self.thread_accesses += 1;
        }
        if matches!(self.delivery, Delivery::Continuous) && self.level() {
            self.pending = true;
        }
        // IRQ57 preempts init/thread at an access boundary. It cannot preempt itself, but
        // IRQ53 (priority 0) preempts it; its only trace here is its entry count.
        if self.context == Context::Irq {
            if self.irq53_before == Some(self.irq_accesses) {
                self.entries53 += 1;
            }
            self.irq_accesses += 1;
            return;
        }
        while self.enabled && self.pending {
            self.entries += 1;
            assert!(self.entries <= 256, "IRQ57 failed to quiesce");
            self.pending = false;
            let interrupted = core::mem::replace(&mut self.context, Context::Irq);
            let mut server = self.server.take().unwrap();
            let mut counters = core::mem::take(&mut self.counters);
            handle(&mut server, &mut counters, self);
            self.counters = counters;
            self.server = Some(server);
            self.context = interrupted;
            if !matches!(self.delivery, Delivery::Pulse) && self.level() {
                self.pending = true;
            }
        }
    }
    fn trace(&self) -> Vec<(Context, Op)> {
        self.bus.borrow().trace.clone()
    }
    fn writes(&self, address: usize) -> Vec<u32> {
        self.trace()
            .into_iter()
            .filter_map(|(_, op)| match op {
                Op::Write(a, v) if a == address => Some(v),
                _ => None,
            })
            .collect()
    }
    fn enable_scmi(&mut self) {
        enable(self);
        self.boundary();
        self.context = Context::Thread;
    }
}

impl Registers for FakeRp1 {
    fn read(&mut self, address: usize) -> u32 {
        self.boundary();
        let value = match address {
            TIMER_RAWL => self.bus.borrow().now,
            _ => self.get(address),
        };
        self.bus
            .borrow_mut()
            .record(self.context, Op::Read(address, value));
        value
    }
    fn write(&mut self, address: usize, value: u32) {
        self.boundary();
        self.bus
            .borrow_mut()
            .record(self.context, Op::Write(address, value));
        match address {
            ISER => {
                assert_eq!(value, IRQ_BIT);
                self.enabled = true;
                self.inject_at(Trigger::Enabled);
                self.pending |= self.force_level;
            }
            ICER => self.enabled &= value & IRQ_BIT == 0,
            ISPR => self.pending |= value & IRQ_BIT != 0,
            ICPR => self.pending &= value & IRQ_BIT == 0,
            PROC_EVENTS_CLR => self.set(PROC_EVENTS, self.get(PROC_EVENTS) & !value),
            _ => self.set(address, value),
        }
        if address == FREE_WORD && value == FREE {
            self.inject_at(Trigger::BadLengthAfterFree);
        }
    }
}

impl Io for FakeRp1 {
    fn set_priority(&mut self, value: u8) {
        self.write(IPR, u32::from(value));
    }
    fn publish(&mut self, word: u32) {
        self.write(STATUS_WORD, word);
    }
    fn barrier(&mut self) {
        self.boundary();
        self.bus.borrow_mut().record(self.context, Op::Barrier);
    }
    fn entries53(&mut self) -> u32 {
        self.boundary();
        let op = Op::Entries53 {
            value: self.entries53,
            boundary: self.irq_accesses - 1,
        };
        self.bus.borrow_mut().record(self.context, op);
        self.entries53
    }
}

// S1–S4, S7: exercise gate opening, W1C, the occupied slot, FREE and unrelated channels.
#[test]
fn pending_channel_is_served_once_and_other_channels_survive() {
    let mut f = FakeRp1::ready(Delivery::Pulse, 0);
    f.set(PROC_GATE, 0x8000);
    f.set(PROC_EVENTS, !1);
    f.pending = true; // A stale pending bit can exist while the line is disabled.
    f.request(BASE_VERSION);
    f.kick();
    f.enable_scmi();
    let init: Vec<_> = f
        .trace()
        .into_iter()
        .filter_map(|(context, op)| (context == Context::Init).then_some(op))
        .collect();
    let step9 = init
        .iter()
        .position(|op| *op == Op::Write(STATUS_WORD, SERVING))
        .unwrap();
    assert_eq!(
        init[step9..],
        [
            Op::Write(STATUS_WORD, SERVING),
            Op::Barrier,
            Op::Read(PROC_GATE, 0x8000),
            Op::Write(PROC_GATE, 0x8001),
            Op::Read(PROC_GATE, 0x8001),
            Op::Write(IPR, 6 << 5),
            Op::Write(ICPR, IRQ_BIT),
            Op::Write(ISER, IRQ_BIT),
            Op::Read(PROC_EVENTS, u32::MAX),
            Op::Write(ISPR, IRQ_BIT),
        ]
    );
    assert_eq!(f.get(PROC_EVENTS), !1);
    assert_eq!(f.writes(PROC_EVENTS_CLR), [1]);
    assert_eq!(f.counters.stats.requests, 1);

    let ops: Vec<_> = f
        .trace()
        .into_iter()
        .filter_map(|(context, op)| (context == Context::Irq).then_some(op))
        .collect();
    let event = ops
        .iter()
        .position(|op| *op == Op::Read(PROC_EVENTS, u32::MAX))
        .unwrap();
    assert_eq!(
        &ops[event..event + 4],
        &[
            Op::Read(PROC_EVENTS, u32::MAX),
            Op::Write(PROC_EVENTS_CLR, 1),
            Op::Barrier,
            Op::Read(FREE_WORD, 0)
        ]
    );
    assert_eq!(f.get(SHMEM + 4 * (PAYLOAD + 1)), 0x0002_0000);
    f.kick();
    f.boundary();
    assert_eq!(f.writes(HOST_EVENTS_SET), [1]);
    assert_eq!(f.counters.stats.duplicates, 1);
    assert_eq!(f.counters.stats.requests, 1);
}

// S5–S6: a next request arrives after FREE, before exception return.
#[test]
fn late_request_is_pended_and_malformed_request_gets_protocol_error() {
    let mut f = FakeRp1::ready(Delivery::Pulse, 0);
    f.enable_scmi();
    f.request(BASE_VERSION);
    f.inject = Some((Trigger::BadLengthAfterFree, BASE_VERSION));
    f.boundary();
    assert_eq!(f.writes(ISPR), [IRQ_BIT]);
    assert_eq!(f.writes(PROC_EVENTS_CLR), [1, 1]);
    assert_eq!(f.writes(HOST_EVENTS_SET), [1]);
    assert_eq!(
        f.get(SHMEM + 4 * PAYLOAD) as i32,
        Status::ProtocolError as i32
    );
    assert_eq!(f.counters.stats.requests, 2);
    assert!(!f.pending);
}

// v3 S9/S11: model both level sampling rules, a pulse, and TIMER_RAWL wraparound.
#[test]
fn delivery_variants_lose_no_requests_and_bound_spurious_entries() {
    for delivery in [Delivery::Pulse, Delivery::AtReturn, Delivery::Continuous] {
        let mut f = FakeRp1::ready(delivery, 0);
        f.enable_scmi();
        // The entries53 read takes one tick: the serve starts at u32::MAX and ends after 0.
        f.bus.borrow_mut().now = u32::MAX - 1;
        f.request(BASE_VERSION);
        f.boundary();
        let mut times = f.trace().into_iter().filter_map(|(_, op)| match op {
            Op::Read(TIMER_RAWL, time) => Some(time),
            _ => None,
        });
        let (start, end) = (times.next().unwrap(), times.next().unwrap());
        if matches!(delivery, Delivery::Pulse) {
            assert_eq!(times.next(), None);
        }
        assert!(end < start);
        let elapsed = end.wrapping_sub(start);
        assert_eq!(f.counters.stats.max_serve_all_us, elapsed);
        for i in 1..100 {
            f.request(BASE_VERSION);
            f.boundary();
            if (i + 1) % 5 == 0 {
                f.kick();
                f.boundary();
            }
        }
        let stats = f.counters.stats;
        assert_eq!((stats.requests, stats.duplicates), (100, 20));
        assert!(stats.spurious57 <= stats.requests + stats.duplicates + 1);
        assert_eq!(stats.max_serve_all_us, elapsed);
        assert!(!f.pending);
    }
}

// S11': inject each request at every access of the real held generator RMW and Fixed check.
#[test]
fn irq_clock_writes_cannot_clobber_thread_clock_updates() {
    let scmi_clock = u32::from(CLOCK_PROTOCOL) << 10;
    let scmi_requests = [
        Request::new(scmi_clock | 7, &[id::PWM0, 0]),
        Request::new(scmi_clock | 6, &[id::UART]),
        Request::new(scmi_clock | 5, &[0, id::ETH, 125_000_000, 0]),
    ];
    let scmi::Kind::Linux(pwm) = RP1_CLOCKS[id::PWM0 as usize].kind else {
        unreachable!()
    };
    let run = |inject| {
        let mut f = FakeRp1::ready(Delivery::Pulse, 1 << id::PWM0);
        f.enable_scmi();
        f.set(pwm.ctrl, (2 << 5) | scmi::CTRL_ENABLE);
        f.set(pwm.ctrl + 4, 1);
        f.set(UART, 0x1000_0840);
        f.set(UART + 4, 1);
        f.set(UART + 0x0c, 1);
        f.inject = inject;
        pwm.program(&mut f, 2, 5);
        assert_eq!(
            crate::clocks::uart_hz_with(&mut |address| f.read(address)),
            Some(50_000_000)
        );
        f.read(TIMER_RAWL);
        f
    };
    let writes = |f: &FakeRp1, from| {
        f.trace()
            .into_iter()
            .filter_map(move |(context, op)| match op {
                Op::Write(address, value)
                    if context == from
                        && (scmi::CLOCKS..scmi::CLOCKS + 0x180).contains(&address) =>
                {
                    Some((address, value))
                }
                _ => None,
            })
            .collect::<Vec<_>>()
    };
    let baseline = run(None);
    let thread = writes(&baseline, Context::Thread);
    assert_eq!(baseline.get(pwm.ctrl + 4), 5);
    for request in scmi_requests {
        for boundary in 0..baseline.thread_accesses {
            let f = run(Some((Trigger::ThreadAccess(boundary), request)));
            assert_eq!(f.counters.stats.requests, 1);
            assert_eq!(f.get(SHMEM + 4 * PAYLOAD), 0);
            assert_eq!(writes(&f, Context::Thread), thread);
            assert_eq!(f.get(pwm.ctrl + 4), 5);
            let irq = writes(&f, Context::Irq);
            assert_eq!(irq.is_empty(), request.args[0] != 0);
            assert!(
                irq.iter()
                    .all(|(address, _)| !thread.iter().any(|(a, _)| a == address))
            );
        }
    }
}

#[test]
fn request_and_duplicate_each_break_the_empty_streak() {
    for duplicate in [false, true] {
        let mut f = FakeRp1::ready(Delivery::Pulse, 0);
        f.enable_scmi();
        for _ in 0..15 {
            f.pending = true;
            f.boundary();
        }
        if duplicate {
            f.kick();
        } else {
            f.request(BASE_VERSION);
        }
        f.boundary();
        f.pending = true;
        f.boundary();
        let stats = f.counters.stats;
        assert_eq!(
            (stats.requests, stats.duplicates),
            (u32::from(!duplicate), u32::from(duplicate))
        );
        assert_eq!(stats.spurious57, 16);
        assert_eq!(f.counters.streak, 1);
        assert!(f.enabled);
    }
}

// S12: a request or the whole storm runs before the first post-ISER init access.
#[test]
fn immediate_irq_outcomes_survive_enable_return() {
    for storm in [false, true] {
        let mut f = FakeRp1::ready(Delivery::AtReturn, 0);
        f.inject = (!storm).then_some((Trigger::Enabled, BASE_VERSION));
        f.force_level = storm;
        f.enable_scmi();
        let trace = f.trace();
        let enabled = trace
            .iter()
            .position(|op| *op == (Context::Init, Op::Write(ISER, IRQ_BIT)))
            .unwrap();
        assert_eq!(trace[enabled + 1].0, Context::Irq);
        let mut init = trace[enabled + 1..]
            .iter()
            .filter_map(|&(context, op)| (context == Context::Init).then_some(op));
        assert!(matches!(init.next(), Some(Op::Read(PROC_EVENTS, _))));
        assert!(matches!(init.next(), None | Some(Op::Write(ISPR, IRQ_BIT))));
        assert_eq!(init.next(), None);
        let (state, requests, spurious, disables) = if storm {
            (ScmiState::Quarantined, 0, 16, &[IRQ_BIT][..])
        } else {
            (ScmiState::Serving { requests: 1 }, 1, 0, &[][..])
        };
        assert_eq!(ScmiState::from_word(f.get(STATUS_WORD)), state);
        assert_eq!(f.counters.stats.requests, requests);
        assert_eq!(f.counters.stats.spurious57, spurious);
        assert_eq!(f.writes(ICER), disables);
        assert_eq!(f.counters.streak, spurious as u8);
    }
}

#[test]
fn published_count_saturates_and_unknown_words_are_preserved() {
    let mut f = FakeRp1::ready(Delivery::Pulse, 0);
    f.enable_scmi();
    f.counters.stats.requests = u16::MAX.into();
    f.request(BASE_VERSION);
    f.boundary();
    assert_eq!(f.counters.stats.requests, u32::from(u16::MAX) + 1);
    assert_eq!(
        ScmiState::from_word(f.get(STATUS_WORD)),
        ScmiState::Serving { requests: u16::MAX }
    );
    assert_eq!(ScmiState::from_word(0), ScmiState::NotStarted);
    assert_eq!(ScmiState::from_word(3), ScmiState::Unknown(3));
}

// S8': IRQ53 preempts one IRQ57 entry before each of its accesses, the entries53 and TIMER_RAWL
// reads included. Only a preemption between the two entries53 reads makes the serve unclean.
#[test]
fn irq53_inside_a_serve_keeps_it_out_of_the_clean_maximum() {
    let serve = |irq53_before| {
        let mut f = FakeRp1::ready(Delivery::Pulse, 0);
        f.enable_scmi();
        f.irq53_before = irq53_before;
        f.request(BASE_VERSION);
        f.boundary();
        f
    };
    let baseline = serve(None);
    assert_eq!(baseline.counters.stats.clean_n, 1);
    // The boundaries of the two entries53 reads. An IRQ53 at a read's boundary runs before it.
    let brackets: Vec<_> = baseline
        .trace()
        .into_iter()
        .filter_map(|(_, op)| match op {
            Op::Entries53 { boundary, .. } => Some(boundary),
            _ => None,
        })
        .collect();
    let [before, after] = brackets[..] else {
        panic!("one bracket: {brackets:?}")
    };
    for boundary in 0..baseline.irq_accesses {
        let f = serve(Some(boundary));
        let stats = f.counters.stats;
        assert_eq!(f.entries53, 1, "boundary {boundary}");
        assert_eq!(stats.requests, 1);
        assert_eq!(f.get(SHMEM + 4 * PAYLOAD), 0, "answered");
        let dirty = before < boundary && boundary <= after;
        assert_eq!(stats.clean_n, u32::from(!dirty), "boundary {boundary}");
        if dirty {
            assert_eq!(stats.max_serve_clean_us, 0);
        }
        assert!(stats.max_serve_clean_us <= stats.max_serve_all_us);
    }
}
