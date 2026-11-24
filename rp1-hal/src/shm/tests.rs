//! [`publish`] on the PERSTn detector's model ([`crate::platform::perstn::fake`]): IRQ53 can
//! run at every thread access outside the two masked sections, and the edges of a host reset
//! can be injected anywhere, masked or not.

extern crate std;

use super::*;
use crate::{
    cold::PCIE_INTF,
    platform::perstn::{self, Word, fake::*},
};
use rp1_proto::status;
use std::{vec, vec::Vec};

const AREA: ShmArea = ShmArea {
    size: 0x10_0000,
    handoff: rp1_proto::HANDOFF,
    sram_end: 0,
    phys: rp1_proto::SHM_PHYS,
};
const HASH: [u8; 32] = [0x5a; 32];
/// Region 0 for [`AREA`]: fabric 0x80_0100_0000.. -> PCIe 0x10_2200_0000.
const AREA_REGION0: [u32; 8] = [
    0,
    0x8000_0000,
    0x0100_0000,
    0x80,
    0x010f_ffff,
    0x2200_0000,
    0x10,
    0,
];
/// The model's `initialize`: these many words zeroed after the header, then this generation.
const FILL_WORDS: usize = 4;
const FILL: usize = SHM_WINDOW as usize + 64;
const GENERATION: u32 = 2;
/// IRQ53's MMIO accesses before its `SHM_WINDOW_LOST` write (INTS, INTE, ICER, MONITOR2,
/// TIMER): with the `cpsie i` before them, how long a reset after section B's checks leaves
/// `SHM_STATUS = 1` visible. Fixed by the code, not a bound on the hardware.
const DETECTION_MMIO_BEFORE_LOST: usize = 5;
const UP: u32 = status::PLAT_PCIE_UP;

impl Io for Fake {
    fn read(&mut self, address: usize) -> u32 {
        perstn::Io::read(self, address)
    }
    fn write(&mut self, address: usize, value: u32) {
        perstn::Io::write(self, address, value);
    }
    fn barrier(&mut self) {
        perstn::Io::barrier(self);
    }
    fn masked<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> T {
        Fake::masked(self, f)
    }
    fn detector(&mut self) -> PcieState {
        self.boundary();
        let word = self.words[Word::State as usize];
        self.record(Op::Load(Word::State, word));
        PcieState::from_word(word)
    }
    fn initialize(&mut self, _: &ShmArea, _: &[u8; 32]) -> Result<u32, Error> {
        for i in 0..FILL_WORDS {
            perstn::Io::write(self, FILL + 4 * i, 0);
        }
        Ok(GENERATION)
    }
}

/// After init: the detector armed and `Up`, the loader handoff for [`AREA`], shm not enabled.
fn armed(delivery: Delivery) -> Fake {
    let mut f = Fake::after_bring_up(delivery);
    f.registers.extend([
        (SHM_STATUS, 0),
        (DBI_COMMAND, MEMORY_AND_MASTER),
        (word(link::HANDOFF_MAGIC), link::MAGIC),
        (word(link::HANDOFF_PHYS_LOW), AREA.phys as u32),
        (word(link::HANDOFF_PHYS_HIGH), (AREA.phys >> 32) as u32),
        (word(link::HANDOFF_SIZE), AREA.size),
    ]);
    f.arm();
    f
}

fn enable(f: &mut Fake, pinned: &mut bool) -> Result<u32, EnableError> {
    publish(f, pinned, &AREA, &HASH)
}

fn thread_ops(f: &Fake) -> Vec<Op> {
    f.trace
        .iter()
        .filter_map(|&(context, op)| (context == Context::Thread).then_some(op))
        .collect()
}

/// The thread access index of `op` in a clean `enable` right after arming: an event injected
/// at `At::Thread(index)` arrives just before it.
fn index_in_enable(op: Op) -> usize {
    let mut f = armed(Delivery::Pulse);
    enable(&mut f, &mut false).unwrap();
    thread_ops(&f).iter().position(|o| *o == op).unwrap()
}

/// Thread writes to the DBI selector or the DBI: what section A writes after its checks.
fn window_writes(f: &Fake) -> usize {
    thread_ops(f)
        .iter()
        .filter(
            |op| matches!(op, Op::Write(a, _) if *a == SELECT || (DBI..DBI + 0x1000).contains(a)),
        )
        .count()
}

/// G2 and G3 on the whole trace: `SHM_STATUS = 1` is written only by thread code inside a
/// masked section that read PCIE_INTR bit 1 clear, while the detector's state is `Up` (so
/// never after `Lost` or `DetectorOff`).
fn check_publication(f: &Fake) {
    let mut state = status::PLAT_PCIE_INITIALIZING;
    let mut masked = false;
    let mut no_edge = false;
    for &(context, op) in &f.trace {
        match op {
            Op::Store(Word::State, value) => state = value,
            Op::Mask => (masked, no_edge) = (true, false),
            Op::Unmask => masked = false,
            Op::Read(PCIE_INTR, intr) if masked => no_edge = intr & PERSTN_EDGE == 0,
            Op::Write(SHM_STATUS, SHM_READY) => assert!(
                context == Context::Thread && masked && no_edge && state == UP,
                "SHM_STATUS = 1 in {context:?}, masked {masked}, no edge {no_edge}, state {state:#x}"
            ),
            _ => {}
        }
    }
}

fn section_a() -> Vec<Op> {
    let mut ops = vec![
        Op::Mask,
        Op::Load(Word::State, UP),
        Op::Read(PCIE_INTR, BRING_UP_LATCHES),
        Op::Read(PCIE_MONITOR2, MONITOR2_UP),
        Op::Read(SELECT, 0),
        Op::Read(DBI_COMMAND, MEMORY_AND_MASTER),
        Op::Read(word(link::HANDOFF_MAGIC), link::MAGIC),
        Op::Read(word(link::HANDOFF_PHYS_LOW), AREA.phys as u32),
        Op::Read(word(link::HANDOFF_PHYS_HIGH), (AREA.phys >> 32) as u32),
        Op::Read(word(link::HANDOFF_SIZE), AREA.size),
        Op::Write(SELECT, SELECT_ATU),
        Op::Write(REGION0 + 4, 0),
    ];
    for i in [0, 2, 3, 4, 5, 6, 7] {
        ops.push(Op::Write(REGION0 + 4 * i, AREA_REGION0[i]));
    }
    ops.extend([Op::Write(REGION0 + 4, AREA_REGION0[1]), Op::Barrier]);
    for (i, value) in AREA_REGION0.into_iter().enumerate() {
        ops.push(Op::Read(REGION0 + 4 * i, value));
    }
    ops.extend([Op::Write(SELECT, 0), Op::Unmask]);
    ops
}

/// M1: on `Up`, section A and B are one masked section each, region 0 is written in the order
/// the PoC used (enable word cleared first, set last) and read back, and the zero fill runs
/// between them, unmasked.
#[test]
fn enable_maps_fills_and_publishes_in_two_masked_sections() {
    let mut f = armed(Delivery::Pulse);
    let mut pinned = false;
    assert_eq!(enable(&mut f, &mut pinned), Ok(GENERATION));
    let fill = (0..FILL_WORDS).map(|i| Op::Write(FILL + 4 * i, 0));
    let section_b = [
        Op::Mask,
        Op::Load(Word::State, UP),
        Op::Read(PCIE_INTR, BRING_UP_LATCHES),
        Op::Write(SHM_GENERATION, GENERATION),
        Op::Barrier,
        Op::Write(SHM_STATUS, SHM_READY),
        Op::Unmask,
    ];
    let expected: Vec<_> = section_a()
        .into_iter()
        .chain(fill)
        .chain(section_b)
        .collect();
    assert_eq!(thread_ops(&f), expected);
    assert!(pinned);
    assert_eq!(f.get(SHM_STATUS), SHM_READY);
    assert_eq!(f.get(SHM_GENERATION), GENERATION);
    f.check_invariants();
    check_publication(&f);
}

/// M2: refused before region 0 is written. Each case: a setup after arming, and the error.
#[test]
fn enable_is_refused_before_the_window_is_written() {
    type Setup = fn(&mut Fake, &mut bool);
    let lost = PcieState::Lost {
        monitor2_hi: (MONITOR2_UP >> 16) as u16,
    };
    let storm = PcieState::DetectorOff {
        reason: status::PLAT_PCIE_OFF_SPURIOUS_STORM,
    };
    let cases: [(&str, Setup, EnableError); 8] = [
        (
            "lost",
            |f, _| {
                f.injections.push((At::Thread(0), Event::Edge(Edge::Up)));
                f.run_thread(1);
            },
            EnableError::Detector(lost),
        ),
        (
            "spurious storm",
            |f, _| {
                // The 16th empty entry turns the detector off.
                for n in 0..16 {
                    f.injections.push((At::Thread(n), Event::Pend));
                }
                f.run_thread(16);
            },
            EnableError::Detector(storm),
        ),
        ("pinned", |_, pinned| *pinned = true, EnableError::Pinned),
        (
            "link down",
            |f, _| {
                f.registers.insert(PCIE_MONITOR2, MONITOR2_UP & !LINK);
            },
            EnableError::NotReady,
        ),
        (
            "selector in use",
            |f, _| {
                f.registers.insert(SELECT, SELECT_ATU);
            },
            EnableError::NotReady,
        ),
        (
            "endpoint disabled",
            |f, _| {
                f.registers.insert(DBI_COMMAND, 0);
            },
            EnableError::NotReady,
        ),
        (
            "no handoff",
            |f, _| {
                f.registers.insert(word(link::HANDOFF_MAGIC), 0);
            },
            EnableError::NotReady,
        ),
        (
            "handoff for another size",
            |f, _| {
                f.registers.insert(word(link::HANDOFF_SIZE), 2 * AREA.size);
            },
            EnableError::NotReady,
        ),
    ];
    for (name, setup, error) in cases {
        let mut f = armed(Delivery::Pulse);
        let mut pinned = false;
        setup(&mut f, &mut pinned);
        let was_pinned = pinned;
        assert_eq!(enable(&mut f, &mut pinned), Err(error), "{name}");
        assert_eq!(window_writes(&f), 0, "{name}");
        assert_eq!(pinned, was_pinned, "{name}");
        assert_ne!(f.get(SHM_STATUS), SHM_READY, "{name}");
        f.check_invariants();
        check_publication(&f);
    }
}

/// M2: a PERSTn edge latched inside section A, before its PCIE_INTR read. IRQ53 cannot run
/// there; it marks the window lost right after the section.
#[test]
fn edge_latched_in_section_a_is_refused() {
    let intr_read = index_in_enable(Op::Read(PCIE_INTR, BRING_UP_LATCHES));
    let mut f = armed(Delivery::Pulse);
    f.injections
        .push((At::Thread(intr_read), Event::Edge(Edge::Up)));
    let mut pinned = false;
    assert_eq!(enable(&mut f, &mut pinned), Err(EnableError::NotReady));
    assert_eq!(window_writes(&f), 0);
    assert!(!pinned);
    f.assert_lost_once();
    f.check_invariants();
    check_publication(&f);
}

/// D2, shm part: a forced source at arming left the detector off, so enable refuses.
#[test]
fn enable_is_refused_when_arming_left_the_detector_off() {
    let mut f = Fake::after_bring_up(Delivery::Level);
    f.registers.insert(PCIE_INTF, 1 << 3);
    f.arm();
    let off = PcieState::DetectorOff {
        reason: status::PLAT_PCIE_OFF_ARM_FAULT,
    };
    let mut pinned = false;
    assert_eq!(enable(&mut f, &mut pinned), Err(EnableError::Detector(off)));
    assert_eq!(
        thread_ops(&f),
        [
            Op::Mask,
            Op::Load(Word::State, f.words[Word::State as usize]),
            Op::Unmask
        ]
    );
    check_publication(&f);
}

/// M3: a host reset (assert, then deassert) at each access of the zero fill. IRQ53 marks the
/// window lost, and section B refuses to publish.
#[test]
fn reset_during_the_zero_fill_is_window_lost() {
    let first_fill = index_in_enable(Op::Write(FILL, 0));
    for delivery in [Delivery::Level, Delivery::Pulse] {
        for latch_on_assert in [true, false] {
            for down in first_fill..first_fill + FILL_WORDS {
                let mut f = armed(delivery);
                f.latch_on_assert = latch_on_assert;
                f.injections
                    .push((At::Thread(down), Event::Edge(Edge::Down)));
                f.injections
                    .push((At::Thread(down + 1), Event::Edge(Edge::Up)));
                let mut pinned = false;
                assert_eq!(enable(&mut f, &mut pinned), Err(EnableError::WindowLost));
                assert!(!pinned, "Lost already refuses a later call");
                f.assert_lost_once();
                assert_eq!(f.get(SHM_GENERATION), 0);
                f.check_invariants();
                check_publication(&f);
            }
        }
    }
}

/// M4: a reset that starts after section B's checks (lifecycle rule L1 does not cover it).
/// `SHM_STATUS = 1` is published, and IRQ53 overwrites it right after `cpsie i`.
#[test]
fn reset_after_the_section_b_checks_is_reported_right_after_it() {
    let generation_write = index_in_enable(Op::Write(SHM_GENERATION, GENERATION));
    for latch_on_assert in [true, false] {
        let mut f = armed(Delivery::Pulse);
        f.latch_on_assert = latch_on_assert;
        f.injections
            .push((At::Thread(generation_write), Event::Edge(Edge::Down)));
        f.injections
            .push((At::Thread(generation_write + 1), Event::Edge(Edge::Up)));
        assert_eq!(enable(&mut f, &mut false), Ok(GENERATION));
        assert!(f.lost());
        assert_eq!(f.get(SHM_STATUS), link::SHM_WINDOW_LOST);
        let published = f
            .trace
            .iter()
            .position(|op| *op == (Context::Thread, Op::Write(SHM_STATUS, SHM_READY)))
            .unwrap();
        let lost = f
            .trace
            .iter()
            .position(|op| *op == (Context::Irq, Op::Write(SHM_STATUS, link::SHM_WINDOW_LOST)))
            .unwrap();
        let between = &f.trace[published + 1..lost];
        let thread: Vec<_> = between
            .iter()
            .filter(|(context, _)| *context == Context::Thread)
            .map(|(_, op)| *op)
            .collect();
        assert_eq!(thread, [Op::Unmask]);
        let mmio = between
            .iter()
            .filter(|(_, op)| matches!(op, Op::Read(..) | Op::Write(..)))
            .count();
        assert_eq!(mmio, DETECTION_MMIO_BEFORE_LOST);
        f.check_invariants();
        check_publication(&f);
    }
}

/// M5: a second call is refused without touching the window or the published words.
#[test]
fn second_enable_is_refused() {
    let mut f = armed(Delivery::Pulse);
    let mut pinned = false;
    enable(&mut f, &mut pinned).unwrap();
    let first = f.trace.len();
    assert_eq!(enable(&mut f, &mut pinned), Err(EnableError::Pinned));
    let second: Vec<_> = f.trace[first..].iter().map(|(_, op)| *op).collect();
    assert_eq!(second, [Op::Mask, Op::Load(Word::State, UP), Op::Unmask]);
    assert_eq!(f.get(SHM_STATUS), SHM_READY);
    assert_eq!(f.get(SHM_GENERATION), GENERATION);
    check_publication(&f);
}

/// §7.1: a region 0 readback mismatch leaves the selector at 0, publishes nothing and pins
/// the window.
#[test]
fn readback_mismatch_pins_the_window() {
    let mut f = armed(Delivery::Pulse);
    f.read_only.push(REGION0 + 4 * 4);
    let mut pinned = false;
    assert_eq!(enable(&mut f, &mut pinned), Err(EnableError::Mismatch));
    assert!(pinned);
    assert_eq!(f.get(SELECT), 0);
    assert!(!thread_ops(&f).contains(&Op::Write(FILL, 0)));
    assert_eq!(enable(&mut f, &mut pinned), Err(EnableError::Pinned));
    assert_eq!(f.get(SHM_STATUS), 0);
    check_publication(&f);
}

/// G2, G3 and G5 together: a host reset at every access of `enable`, for both deliveries and
/// with and without H2. Whatever `enable` returns, the window is `SHM_WINDOW_LOST` at rest and
/// `SHM_STATUS = 1` was written only while `Up` with no edge latched.
#[test]
fn reset_at_any_access_of_enable_ends_window_lost() {
    let mut clean = armed(Delivery::Pulse);
    enable(&mut clean, &mut false).unwrap();
    let accesses = thread_ops(&clean).len();
    for delivery in [Delivery::Level, Delivery::Pulse] {
        for latch_on_assert in [true, false] {
            for down in 0..accesses {
                let mut f = armed(delivery);
                f.latch_on_assert = latch_on_assert;
                f.injections
                    .push((At::Thread(down), Event::Edge(Edge::Down)));
                f.injections
                    .push((At::Thread(down + 1), Event::Edge(Edge::Up)));
                let result = enable(&mut f, &mut false);
                f.run_thread(2);
                assert!(
                    f.lost(),
                    "{delivery:?} {latch_on_assert} {down}: {result:?}"
                );
                assert_eq!(f.get(SHM_STATUS), link::SHM_WINDOW_LOST);
                f.check_invariants();
                check_publication(&f);
            }
        }
    }
}

#[test]
fn region0_translates_the_last_word_of_the_reservation() {
    let area = ShmArea {
        size: 0x20_0000,
        ..AREA
    };
    let region = region0(&area);
    assert_eq!(region[4], 0x011f_ffff);
    assert_eq!(
        u64::from(region[6]) << 32 | u64::from(region[5]),
        0x10_2200_0000
    );
}
