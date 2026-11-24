//! The detector on a model of RP1's PCIe interrupt registers and proc0's NVIC line 53.
//!
//! PCIE_INTS = (INTR & INTE) | INTF. A PERSTn deassertion (`Edge::Up`) latches INTR bit 1; an
//! assertion (`Edge::Down`) latches it only if H2 holds (`latch_on_assert`, HYP). The NVIC
//! enable and pending bits are separate. IRQ53 preempts init and thread code at each access
//! boundary, never itself.

extern crate std;

use super::*;
use std::{collections::BTreeMap, vec::Vec};

/// PCIE_MONITOR2 bit 17: PERSTn deasserted. With the link up and core alive.
const PERSTN_HIGH: u32 = 1 << 17;
const MONITOR2_UP: u32 = 0x001b_0000 | crate::cold::LINK;
/// Bits 0, 3 and 6 of PCIE_INTR are still latched after the bring-up (HW 2026-11).
const BRING_UP_LATCHES: u32 = 0x49;
/// An address the thread workload reads: anything the detector does not touch.
const THREAD_REGISTER: usize = 0x4001_0000;
const VECTORS: usize = 0x2000_0000;
const HANDLER: usize = 0x2000_1235;
const MAX_MMIO_PER_ENTRY: usize = 12;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Context {
    Init,
    Thread,
    Irq,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Op {
    Enter,
    Read(usize, u32),
    Write(usize, u32),
    Priority(u8),
    Barrier,
    Store(Word, u32),
}

/// How a pending PCIE_INTS reaches the NVIC.
#[derive(Clone, Copy, Debug)]
enum Delivery {
    /// Pends while INTS is nonzero, also again after the handler returns.
    Level,
    /// Pends once when INTS becomes nonzero.
    Pulse,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Edge {
    Down,
    Up,
}

/// When an edge or a spurious pend is injected.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum At {
    /// Right after init's ISER write, before its next access.
    Iser,
    /// Before the n-th thread access.
    Thread(usize),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Event {
    Edge(Edge),
    /// The NVIC pends the line with INTS = 0.
    Pend,
}

struct Fake {
    registers: BTreeMap<usize, u32>,
    /// The detector's DSRAM words, indexed by `Word`.
    words: [u32; 5],
    delivery: Delivery,
    latch_on_assert: bool,
    context: Context,
    enabled: bool,
    pending: bool,
    trace: Vec<(Context, Op)>,
    injections: Vec<(At, Event)>,
    thread_accesses: usize,
    now: u32,
}

impl Fake {
    /// The endpoint as the bring-up leaves it: link up, PERSTn high, the bring-up's own latches.
    fn after_bring_up(delivery: Delivery) -> Self {
        let registers = [
            (PCIE_INTR, BRING_UP_LATCHES),
            (PCIE_INTE, 0x4b),
            (PCIE_MONITOR2, MONITOR2_UP),
            (SHM_STATUS, 1),
            (STATUS_WORD, status::PLAT_PCIE_INITIALIZING),
            (VTOR, VECTORS as u32),
            (VECTORS + 4 * (16 + IRQ), HANDLER as u32),
        ];
        Self {
            registers: registers.into_iter().collect(),
            words: [status::PLAT_PCIE_INITIALIZING, 0, 0, 0, 0],
            delivery,
            latch_on_assert: true,
            context: Context::Init,
            enabled: false,
            pending: false,
            trace: Vec::new(),
            injections: Vec::new(),
            thread_accesses: 0,
            now: 0,
        }
    }
    fn get(&self, address: usize) -> u32 {
        self.registers.get(&address).copied().unwrap_or(0)
    }
    fn ints(&self) -> u32 {
        (self.get(PCIE_INTR) & self.get(PCIE_INTE)) | self.get(PCIE_INTF)
    }
    /// The detector's own state word, decoded like the published one.
    fn state(&self) -> PcieState {
        PcieState::from_word(self.words[Word::State as usize])
    }
    fn lost(&self) -> bool {
        matches!(self.state(), PcieState::Lost { .. })
    }
    fn detail(&self) -> LostDetail {
        LostDetail {
            monitor2: self.words[Word::Monitor2 as usize],
            at_us: self.words[Word::AtUs as usize],
        }
    }
    fn published(&self) -> PcieState {
        PcieState::from_word(self.get(STATUS_WORD))
    }
    /// Sets PCIE_INTR bits; a pulse pends on INTS becoming nonzero.
    fn latch(&mut self, bits: u32) {
        let before = self.ints();
        self.registers.insert(PCIE_INTR, self.get(PCIE_INTR) | bits);
        if before == 0 && self.ints() != 0 {
            self.pending = true;
        }
    }
    fn edge(&mut self, edge: Edge) {
        let monitor2 = self.get(PCIE_MONITOR2);
        match edge {
            Edge::Down => {
                self.registers
                    .insert(PCIE_MONITOR2, monitor2 & !PERSTN_HIGH);
                if self.latch_on_assert {
                    self.latch(PERSTN_EDGE);
                }
            }
            Edge::Up => {
                self.registers.insert(PCIE_MONITOR2, monitor2 | PERSTN_HIGH);
                self.latch(PERSTN_EDGE);
            }
        }
    }
    fn inject(&mut self, at: At) {
        while let Some(i) = self.injections.iter().position(|(when, _)| *when == at) {
            match self.injections.remove(i).1 {
                Event::Edge(edge) => self.edge(edge),
                Event::Pend => self.pending = true,
            }
        }
    }
    fn boundary(&mut self) {
        if self.context == Context::Thread {
            self.inject(At::Thread(self.thread_accesses));
            self.thread_accesses += 1;
        }
        if self.context == Context::Irq {
            return;
        }
        if matches!(self.delivery, Delivery::Level) && self.ints() != 0 {
            self.pending = true;
        }
        while self.enabled && self.pending {
            self.pending = false;
            let interrupted = core::mem::replace(&mut self.context, Context::Irq);
            self.trace.push((Context::Irq, Op::Enter));
            assert!(self.entries() <= 64, "IRQ53 failed to quiesce");
            handle(self);
            self.context = interrupted;
            if matches!(self.delivery, Delivery::Level) && self.ints() != 0 {
                self.pending = true;
            }
        }
    }
    fn record(&mut self, op: Op) {
        self.now += 1;
        self.trace.push((self.context, op));
    }
    /// Arms (step 10) and takes any IRQ53 that is due before init returns.
    fn arm(&mut self) -> Armed {
        let armed = arm(self);
        self.boundary();
        self.context = Context::Thread;
        armed
    }
    /// The application's thread: `n` reads of a register the detector does not use. An event
    /// injected at a boundary is delivered at that boundary.
    fn run_thread(&mut self, n: usize) {
        for _ in 0..n {
            self.read(THREAD_REGISTER);
        }
    }
    fn entries(&self) -> usize {
        self.trace.iter().filter(|(_, op)| *op == Op::Enter).count()
    }
    fn writes(&self, address: usize) -> Vec<(Context, u32)> {
        self.trace
            .iter()
            .filter_map(|&(context, op)| match op {
                Op::Write(a, value) if a == address => Some((context, value)),
                _ => None,
            })
            .collect()
    }
    fn init_ops(&self) -> Vec<Op> {
        self.trace
            .iter()
            .filter_map(|&(context, op)| (context == Context::Init).then_some(op))
            .collect()
    }
    /// The ops of each IRQ53 entry.
    fn entry_ops(&self) -> Vec<Vec<Op>> {
        let mut entries = Vec::new();
        for &(context, op) in &self.trace {
            match (context, op) {
                (_, Op::Enter) => entries.push(Vec::new()),
                (Context::Irq, op) => entries.last_mut().unwrap().push(op),
                _ => {}
            }
        }
        entries
    }
    /// G1, G4, G6, G8 and D8, checked on every run.
    fn check_invariants(&self) {
        // G4, at rest.
        assert!(!self.pending || !self.enabled, "IRQ53 pending at rest");
        match self.state() {
            PcieState::Up => assert!(self.enabled, "Up with IRQ53 disabled"),
            PcieState::Lost { .. } | PcieState::DetectorOff { .. } => {
                assert!(!self.enabled);
                assert_eq!(self.get(PCIE_INTE), 0);
            }
            state => panic!("{state:?} after arming"),
        }
        // G1: INTE gets init's 0x02, or 0 from IRQ53 or a failed arming.
        for (context, value) in self.writes(PCIE_INTE) {
            assert!(
                value == 0 || (context == Context::Init && value == PERSTN_EDGE),
                "{context:?} INTE = {value:#x}"
            );
        }
        // D8 and G1, G6 for each entry.
        for ops in self.entry_ops() {
            let mmio = ops
                .iter()
                .filter(|op| matches!(op, Op::Read(..) | Op::Write(..)))
                .count();
            assert!(mmio <= MAX_MMIO_PER_ENTRY, "{mmio} accesses: {ops:?}");
            for op in ops {
                if let Op::Write(address, _) = op {
                    assert!(
                        [PCIE_INTE, ICER, SHM_STATUS, STATUS_WORD].contains(&address),
                        "IRQ53 wrote {address:#x}"
                    );
                }
            }
        }
        // G8: after ISER53, init writes neither the DSRAM state nor PLAT_PCIE.
        let init = self.init_ops();
        if let Some(iser) = init.iter().position(|op| *op == Op::Write(ISER, IRQ_BIT)) {
            for op in &init[iser..] {
                assert!(
                    !matches!(op, Op::Store(..) | Op::Write(STATUS_WORD, _)),
                    "init after ISER53: {op:?}"
                );
            }
        }
    }
    fn assert_lost_once(&self) {
        assert!(self.lost(), "{:?}", self.state());
        let lost = self.writes(SHM_STATUS);
        assert_eq!(lost, [(Context::Irq, link::SHM_WINDOW_LOST)]);
        assert_eq!(self.published(), self.state());
    }
}

impl Io for Fake {
    fn read(&mut self, address: usize) -> u32 {
        self.boundary();
        let value = match address {
            PCIE_INTS => self.ints(),
            ISER => u32::from(self.enabled) * IRQ_BIT,
            TIMER_RAWL => self.now,
            _ => self.get(address),
        };
        self.record(Op::Read(address, value));
        value
    }
    fn write(&mut self, address: usize, value: u32) {
        self.boundary();
        self.record(Op::Write(address, value));
        match address {
            ISER => {
                assert_eq!(value, IRQ_BIT);
                self.enabled = true;
                self.inject(At::Iser);
            }
            ICER => self.enabled &= value & IRQ_BIT == 0,
            ISPR => self.pending |= value & IRQ_BIT != 0,
            ICPR => self.pending &= value & IRQ_BIT == 0,
            PCIE_INTR => panic!("PCIE_INTR is never written"),
            _ => {
                let before = self.ints();
                self.registers.insert(address, value);
                if before == 0 && self.ints() != 0 {
                    self.pending = true;
                }
            }
        }
    }
    fn set_priority(&mut self, value: u8) {
        self.boundary();
        self.record(Op::Priority(value));
    }
    fn barrier(&mut self) {
        self.boundary();
        self.record(Op::Barrier);
    }
    fn load(&mut self, word: Word) -> u32 {
        self.words[word as usize]
    }
    fn store(&mut self, word: Word, value: u32) {
        self.boundary();
        self.record(Op::Store(word, value));
        self.words[word as usize] = value;
    }
}

const DELIVERIES: [Delivery; 2] = [Delivery::Level, Delivery::Pulse];

/// Init's first detector state: counters zeroed, then the state word and its publication.
fn start(state: u32) -> [Op; 5] {
    [
        Op::Store(Word::Entries, 0),
        Op::Store(Word::Spurious, 0),
        Op::Store(Word::State, state),
        Op::Write(STATUS_WORD, state),
        Op::Barrier,
    ]
}

/// D1: a normal arming. INTR's other latches (bits 0, 3, 6) are not a reset.
#[test]
fn arming_with_only_other_latches_stays_up() {
    for delivery in DELIVERIES {
        let mut f = Fake::after_bring_up(delivery);
        let armed = f.arm();
        assert_eq!(
            f.init_ops(),
            [
                Op::Write(PCIE_INTE, PERSTN_EDGE),
                Op::Read(PCIE_INTF, 0),
                Op::Read(PCIE_INTR, BRING_UP_LATCHES),
            ]
            .into_iter()
            .chain(start(status::PLAT_PCIE_UP))
            .chain([
                Op::Priority(0),
                Op::Write(ICPR, IRQ_BIT),
                Op::Write(ISER, IRQ_BIT),
                Op::Read(PCIE_INTR, BRING_UP_LATCHES),
            ])
            .collect::<Vec<_>>()
        );
        assert_eq!(
            armed,
            Armed {
                intf: 0,
                intr_at_arm: BRING_UP_LATCHES
            }
        );
        f.run_thread(8);
        assert_eq!(f.entries(), 0);
        assert_eq!(f.state(), PcieState::Up);
        assert_eq!(f.published(), PcieState::Up);
        f.check_invariants();
        assert_eq!(f.get(SHM_STATUS), 1);
    }
}

/// D2: a forced source at arming leaves the detector off and IRQ53 disabled.
#[test]
fn forced_source_at_arming_turns_the_detector_off() {
    let mut f = Fake::after_bring_up(Delivery::Level);
    f.registers.insert(PCIE_INTF, 1 << 3);
    let armed = f.arm();
    let off = detector_off(status::PLAT_PCIE_OFF_ARM_FAULT);
    assert_eq!(
        armed,
        Armed {
            intf: 1 << 3,
            intr_at_arm: 0
        }
    );
    let arming = [
        Op::Write(PCIE_INTE, PERSTN_EDGE),
        Op::Read(PCIE_INTF, 1 << 3),
        Op::Write(PCIE_INTE, 0),
    ];
    assert_eq!(f.init_ops(), [&arming[..], &start(off)].concat());
    assert_eq!(
        f.published(),
        PcieState::DetectorOff {
            reason: status::PLAT_PCIE_OFF_ARM_FAULT
        }
    );
    f.run_thread(4);
    assert_eq!(f.entries(), 0);
    f.check_invariants();
}

/// D3: a PERSTn edge latched before arming is a reset after the bring-up: Lost, once.
#[test]
fn edge_latched_before_arming_is_lost_once() {
    for delivery in DELIVERIES {
        let mut f = Fake::after_bring_up(delivery);
        f.registers
            .insert(PCIE_INTR, BRING_UP_LATCHES | PERSTN_EDGE);
        let armed = f.arm();
        assert_eq!(armed.intr_at_arm, BRING_UP_LATCHES | PERSTN_EDGE);
        f.run_thread(4);
        f.assert_lost_once();
        assert_eq!(f.entries(), 1, "{delivery:?}");
        f.check_invariants();
    }
}

/// D3': IRQ53 runs right after ISER53 and before init's next access. What it wrote survives
/// init's return, and init's remaining accesses are the INTR read and the ISPR.
#[test]
fn immediate_detection_survives_the_rest_of_init() {
    for delivery in DELIVERIES {
        let mut f = Fake::after_bring_up(delivery);
        f.injections.push((At::Iser, Event::Edge(Edge::Up)));
        f.arm();
        let iser = f
            .trace
            .iter()
            .position(|op| *op == (Context::Init, Op::Write(ISER, IRQ_BIT)))
            .unwrap();
        assert_eq!(f.trace[iser + 1], (Context::Irq, Op::Enter), "{delivery:?}");
        let after: Vec<_> = f.trace[iser + 1..]
            .iter()
            .filter_map(|&(context, op)| (context == Context::Init).then_some(op))
            .collect();
        assert_eq!(
            after,
            [
                Op::Read(PCIE_INTR, BRING_UP_LATCHES | PERSTN_EDGE),
                Op::Write(ISPR, IRQ_BIT)
            ]
        );
        f.run_thread(4);
        f.assert_lost_once();
        assert!(!f.enabled);
        // The ISPR leaves a pend on the disabled line (G4 allows it).
        assert!(f.pending);
        f.check_invariants();
    }
}

/// D4 and G5: a host reset (assert, then deassert) at every thread access boundary. With H2
/// the assertion is detected, without it the deassertion; once either way.
#[test]
fn reset_at_any_thread_access_is_lost_once() {
    const THREAD: usize = 6;
    for delivery in DELIVERIES {
        for latch_on_assert in [true, false] {
            for down in 0..THREAD {
                let mut f = Fake::after_bring_up(delivery);
                f.latch_on_assert = latch_on_assert;
                f.arm();
                f.injections
                    .push((At::Thread(down), Event::Edge(Edge::Down)));
                f.injections
                    .push((At::Thread(down + 1), Event::Edge(Edge::Up)));
                f.run_thread(down + 1);
                assert_eq!(f.lost(), latch_on_assert, "after the assertion");
                f.run_thread(THREAD);
                f.assert_lost_once();
                assert_eq!(f.entries(), 1);
                f.check_invariants();
                let detail = f.detail();
                // The detector reads PCIE_MONITOR2 at the edge it saw.
                let perstn_high = detail.monitor2 & PERSTN_HIGH != 0;
                assert_eq!(perstn_high, !latch_on_assert);
                assert_eq!(
                    f.published(),
                    PcieState::Lost {
                        monitor2_hi: (detail.monitor2 >> 16) as u16
                    }
                );
            }
        }
    }
}

/// D5: entries with INTS = 0. 15 leave the detector up, the 16th turns it off.
#[test]
fn sixteenth_spurious_entry_turns_the_detector_off() {
    for pends in [15, 16] {
        let mut f = Fake::after_bring_up(Delivery::Level);
        f.arm();
        for n in 0..pends {
            f.injections.push((At::Thread(n), Event::Pend));
        }
        f.run_thread(pends + 2);
        assert_eq!(f.entries(), pends);
        assert_eq!(f.words[Word::Spurious as usize], pends as u32);
        let storm = status::PLAT_PCIE_OFF_SPURIOUS_STORM;
        let state = if pends == 16 {
            PcieState::DetectorOff { reason: storm }
        } else {
            PcieState::Up
        };
        assert_eq!(f.state(), state);
        assert_eq!(f.published(), state);
        assert_eq!(f.get(SHM_STATUS), 1, "a storm is not a reset");
        f.check_invariants();
    }
}

/// D6: an INTS bit other than the PERSTn edge (a source forced after arming, an INTE that
/// let the bring-up's latches through) is treated as a reset.
#[test]
fn any_other_interrupt_source_is_lost() {
    type Corrupt = fn(&mut Fake);
    let corruptions: [Corrupt; 2] = [|f| f.write(PCIE_INTF, 1 << 3), |f| f.write(PCIE_INTE, 0x4b)];
    for corrupt in corruptions {
        let mut f = Fake::after_bring_up(Delivery::Level);
        f.arm();
        corrupt(&mut f);
        f.run_thread(2);
        f.assert_lost_once();
        assert_eq!(f.get(PCIE_INTR), BRING_UP_LATCHES, "INTR kept");
    }
}

/// D7: after Lost, more edges do not enter IRQ53.
#[test]
fn edges_after_lost_do_not_enter() {
    for delivery in DELIVERIES {
        let mut f = Fake::after_bring_up(delivery);
        f.arm();
        let edges = [Edge::Up, Edge::Down, Edge::Up, Edge::Down, Edge::Up];
        for (n, edge) in edges.into_iter().enumerate() {
            f.injections.push((At::Thread(n), Event::Edge(edge)));
        }
        f.run_thread(edges.len() + 2);
        assert_eq!(f.entries(), 1);
        f.assert_lost_once();
        f.check_invariants();
    }
}

/// D8, the detection entry in full: 7 MMIO accesses, PCIe side INTE = 0 only, the window
/// lost before the state, the published word last.
#[test]
fn detection_entry_order() {
    let mut f = Fake::after_bring_up(Delivery::Pulse);
    f.arm();
    f.injections.push((At::Thread(0), Event::Edge(Edge::Up)));
    f.run_thread(1);
    let entries = f.entry_ops();
    let [ops] = &entries[..] else {
        panic!("{entries:?}")
    };
    let monitor2 = MONITOR2_UP;
    let detail = f.detail();
    assert_eq!(detail.monitor2, monitor2);
    assert_eq!(
        ops[..],
        [
            Op::Store(Word::Entries, 1),
            Op::Read(PCIE_INTS, PERSTN_EDGE),
            Op::Write(PCIE_INTE, 0),
            Op::Write(ICER, IRQ_BIT),
            Op::Barrier,
            Op::Read(PCIE_MONITOR2, monitor2),
            Op::Read(TIMER_RAWL, detail.at_us),
            Op::Write(SHM_STATUS, link::SHM_WINDOW_LOST),
            Op::Barrier,
            Op::Store(Word::Monitor2, monitor2),
            Op::Store(Word::AtUs, detail.at_us),
            Op::Store(Word::State, lost(monitor2)),
            Op::Write(STATUS_WORD, lost(monitor2)),
            Op::Barrier,
        ]
    );
    f.check_invariants();
}

/// G7: the handlers do not log (`log!` would also wait on or drop into the UART).
#[test]
fn platform_handlers_do_not_log() {
    for source in [
        include_str!("../perstn.rs"),
        include_str!("../../scmi/irq.rs"),
    ] {
        assert!(!source.contains("log!"));
        assert!(!source.contains("logging::"));
    }
}

/// I5-53 at run time: the report compares vector 53 with the handler.
#[test]
fn diagnose_reads_inte_the_line_and_the_vector() {
    let mut f = Fake::after_bring_up(Delivery::Level);
    f.arm();
    assert_eq!(diagnose(&mut f, HANDLER), (PERSTN_EDGE, true, true));
    assert_eq!(diagnose(&mut f, HANDLER + 2), (PERSTN_EDGE, true, false));
    f.injections
        .push((At::Thread(f.thread_accesses), Event::Edge(Edge::Up)));
    assert_eq!(diagnose(&mut f, HANDLER), (0, false, true));
}

#[test]
fn published_words_decode_strictly() {
    let cases = [
        (status::PLAT_PCIE_UP, PcieState::Up),
        (
            lost(0x001b_0000),
            PcieState::Lost {
                monitor2_hi: 0x001b,
            },
        ),
        (
            detector_off(status::PLAT_PCIE_OFF_SPURIOUS_STORM),
            PcieState::DetectorOff {
                reason: status::PLAT_PCIE_OFF_SPURIOUS_STORM,
            },
        ),
    ];
    for (word, state) in cases {
        assert_eq!(PcieState::from_word(word), state);
    }
    assert_eq!(PcieState::from_word(0), PcieState::NotStarted);
    assert_eq!(PcieState::from_word(1), PcieState::Initializing);
    for word in [5, 0x102, 0x1_0002, 0x1_0004, 0x103] {
        assert_eq!(PcieState::from_word(word), PcieState::Unknown(word));
    }
}
