//! A model of RP1's PCIe interrupt registers and proc0's NVIC line 53, with the detector
//! installed as the IRQ53 handler. The detector's tests and the shm publication tests use it.
//!
//! PCIE_INTS = (INTR & INTE) | INTF. A PERSTn deassertion (`Edge::Up`) latches INTR bit 1; an
//! assertion (`Edge::Down`) latches it only if H2 holds (`latch_on_assert`, HYP). The NVIC
//! enable and pending bits are separate. IRQ53 preempts init and thread code at each access
//! boundary, never itself, and not while thread code has PRIMASK set ([`Fake::masked`]).

extern crate std;

use super::*;
use std::{collections::BTreeMap, vec::Vec};

/// PCIE_MONITOR2 bit 17: PERSTn deasserted. With the link up and core alive.
pub(crate) const PERSTN_HIGH: u32 = 1 << 17;
pub(crate) const MONITOR2_UP: u32 = 0x001b_0000 | crate::cold::LINK;
/// PCIE_INTE as the bring-up leaves it (HW 2026-11).
pub(crate) const BRING_UP_INTE: u32 = 0x4b;
/// The PCIE_INTR latches expected after the bring-up: bits 0, 3 and 6. On HW 2026-11,
/// PCIE_INTS = 0x49 with INTE = [`BRING_UP_INTE`]; that INTR holds the same bits is HYP until
/// the init report's `intr_at_arm` is read on HW.
pub(crate) const BRING_UP_LATCHES: u32 = 0x49;
/// An address the thread workload reads: anything the detector does not touch.
const THREAD_REGISTER: usize = 0x4001_0000;
const VECTORS: usize = 0x2000_0000;
pub(crate) const HANDLER: usize = 0x2000_1235;
const MAX_MMIO_PER_ENTRY: usize = 12;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Context {
    Init,
    Thread,
    Irq,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Op {
    Enter,
    Read(usize, u32),
    Write(usize, u32),
    Priority(u8),
    Barrier,
    Store(Word, u32),
    /// A thread read of one of the detector's DSRAM words. The handler's own loads are not
    /// recorded.
    Load(Word, u32),
    /// `cpsid i` / `cpsie i`.
    Mask,
    Unmask,
}

/// How a pending PCIE_INTS reaches the NVIC.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Delivery {
    /// Pends while INTS is nonzero, also again after the handler returns.
    Level,
    /// Pends once when INTS becomes nonzero.
    Pulse,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Edge {
    Down,
    Up,
}

/// When an edge or a spurious pend is injected.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum At {
    /// Right after init's ISER write, before its next access.
    Iser,
    /// Before the n-th thread access.
    Thread(usize),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Event {
    Edge(Edge),
    /// The NVIC pends the line with INTS = 0.
    Pend,
}

pub(crate) struct Fake {
    pub(crate) registers: BTreeMap<usize, u32>,
    /// The detector's DSRAM words, indexed by `Word`.
    pub(crate) words: [u32; 5],
    pub(crate) delivery: Delivery,
    pub(crate) latch_on_assert: bool,
    pub(crate) context: Context,
    pub(crate) enabled: bool,
    pub(crate) pending: bool,
    /// PRIMASK: an IRQ53 that is due waits for `Op::Unmask`.
    pub(crate) masked: bool,
    /// Registers that keep their value when written (a readback mismatch).
    pub(crate) read_only: Vec<usize>,
    pub(crate) trace: Vec<(Context, Op)>,
    pub(crate) injections: Vec<(At, Event)>,
    pub(crate) thread_accesses: usize,
    pub(crate) now: u32,
}

impl Fake {
    /// The endpoint as the bring-up leaves it: link up, PERSTn high, the bring-up's own latches.
    pub(crate) fn after_bring_up(delivery: Delivery) -> Self {
        let registers = [
            (PCIE_INTR, BRING_UP_LATCHES),
            (PCIE_INTE, BRING_UP_INTE),
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
            masked: false,
            read_only: Vec::new(),
            trace: Vec::new(),
            injections: Vec::new(),
            thread_accesses: 0,
            now: 0,
        }
    }
    pub(crate) fn get(&self, address: usize) -> u32 {
        self.registers.get(&address).copied().unwrap_or(0)
    }
    fn ints(&self) -> u32 {
        (self.get(PCIE_INTR) & self.get(PCIE_INTE)) | self.get(PCIE_INTF)
    }
    /// The detector's own state word, decoded like the published one.
    pub(crate) fn state(&self) -> PcieState {
        PcieState::from_word(self.words[Word::State as usize])
    }
    pub(crate) fn lost(&self) -> bool {
        matches!(self.state(), PcieState::Lost { .. })
    }
    pub(crate) fn detail(&self) -> LostDetail {
        LostDetail {
            monitor2: self.words[Word::Monitor2 as usize],
            at_us: self.words[Word::AtUs as usize],
        }
    }
    pub(crate) fn published(&self) -> PcieState {
        PcieState::from_word(self.get(STATUS_WORD))
    }
    /// A register changed by the hardware or a write: not traced. A pulse pends on INTS
    /// becoming nonzero.
    pub(crate) fn set(&mut self, address: usize, value: u32) {
        let before = self.ints();
        self.registers.insert(address, value);
        if before == 0 && self.ints() != 0 {
            self.pending = true;
        }
    }
    /// Sets PCIE_INTR bits.
    fn latch(&mut self, bits: u32) {
        self.set(PCIE_INTR, self.get(PCIE_INTR) | bits);
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
    pub(crate) fn boundary(&mut self) {
        if self.context == Context::Thread {
            self.inject(At::Thread(self.thread_accesses));
            self.thread_accesses += 1;
        }
        if self.context != Context::Irq {
            self.deliver();
        }
    }
    /// Takes IRQ53 while it is enabled, pending and not masked.
    fn deliver(&mut self) {
        if matches!(self.delivery, Delivery::Level) && self.ints() != 0 {
            self.pending = true;
        }
        while self.enabled && self.pending && !self.masked {
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
    pub(crate) fn record(&mut self, op: Op) {
        self.now += 1;
        self.trace.push((self.context, op));
    }
    /// Runs `f` as thread code between `cpsid i` and `cpsie i`. An IRQ53 that became due
    /// inside is taken right after `cpsie i`.
    pub(crate) fn masked<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> T {
        self.boundary();
        self.record(Op::Mask);
        self.masked = true;
        let value = f(self);
        self.boundary();
        self.record(Op::Unmask);
        self.masked = false;
        self.deliver();
        value
    }
    /// Arms (step 10) and takes any IRQ53 that is due before init returns.
    pub(crate) fn arm(&mut self) -> Armed {
        let armed = arm(self);
        self.boundary();
        self.context = Context::Thread;
        armed
    }
    /// The application's thread: `n` reads of a register the detector does not use. An event
    /// injected at a boundary is delivered at that boundary.
    pub(crate) fn run_thread(&mut self, n: usize) {
        for _ in 0..n {
            self.read(THREAD_REGISTER);
        }
    }
    pub(crate) fn entries(&self) -> usize {
        self.trace.iter().filter(|(_, op)| *op == Op::Enter).count()
    }
    pub(crate) fn writes(&self, address: usize) -> Vec<(Context, u32)> {
        self.trace
            .iter()
            .filter_map(|&(context, op)| match op {
                Op::Write(a, value) if a == address => Some((context, value)),
                _ => None,
            })
            .collect()
    }
    pub(crate) fn init_ops(&self) -> Vec<Op> {
        self.trace
            .iter()
            .filter_map(|&(context, op)| (context == Context::Init).then_some(op))
            .collect()
    }
    /// The ops of each IRQ53 entry.
    pub(crate) fn entry_ops(&self) -> Vec<Vec<Op>> {
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
    pub(crate) fn check_invariants(&self) {
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
        // G6 for init: step 10 writes INTE, the NVIC line and PLAT_PCIE, nothing else.
        for (context, op) in &self.trace {
            if let (Context::Init, Op::Write(address, _)) = (context, op) {
                assert!(
                    [PCIE_INTE, ICPR, ISER, ISPR, STATUS_WORD].contains(address),
                    "init wrote {address:#x}"
                );
            }
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
    pub(crate) fn assert_lost_once(&self) {
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
            _ if self.read_only.contains(&address) => {}
            _ => self.set(address, value),
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
