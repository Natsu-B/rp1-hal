//! The detector on the model of [`super::fake`].

extern crate std;

use super::{fake::*, *};
use std::vec::Vec;
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

/// Step 10's reads for its report, after the arming.
fn report_reads(inte: u32, nvic53: bool) -> [Op; 4] {
    [
        Op::Read(PCIE_INTE, inte),
        Op::Read(ISER, u32::from(nvic53) * IRQ_BIT),
        Op::Read(VTOR, VECTORS as u32),
        Op::Read(VECTOR53, HANDLERS.detector as u32),
    ]
}

/// D1: a normal arming. INTR's other latches ([`BRING_UP_LATCHES`]) are not a reset.
#[test]
fn arming_with_only_other_latches_stays_up() {
    for delivery in DELIVERIES {
        let mut f = Fake::after_bring_up(delivery);
        let report = f.arm();
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
            .chain(report_reads(PERSTN_EDGE, true))
            .collect::<Vec<_>>()
        );
        assert_eq!(
            report,
            DetectorReport {
                inte: PERSTN_EDGE,
                intf: 0,
                intr_at_arm: BRING_UP_LATCHES,
                nvic53: true,
                vec53_ok: true,
            }
        );
        assert_eq!(report.perstn_edge_at_arm(), Some(false));
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
    let report = f.arm();
    let off = detector_off(status::PLAT_PCIE_OFF_ARM_FAULT);
    assert_eq!(
        report,
        DetectorReport {
            inte: 0,
            intf: 1 << 3,
            intr_at_arm: 0,
            nvic53: false,
            vec53_ok: true,
        }
    );
    assert_eq!(report.perstn_edge_at_arm(), None);
    let arming = [
        Op::Write(PCIE_INTE, PERSTN_EDGE),
        Op::Read(PCIE_INTF, 1 << 3),
        Op::Write(PCIE_INTE, 0),
    ];
    assert_eq!(
        f.init_ops(),
        [&arming[..], &start(off), &report_reads(0, false)].concat()
    );
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
        let report = f.arm();
        assert_eq!(report.intr_at_arm, BRING_UP_LATCHES | PERSTN_EDGE);
        assert_eq!(report.perstn_edge_at_arm(), Some(true));
        f.run_thread(4);
        f.assert_lost_once();
        assert_eq!(f.entries(), 1, "{delivery:?}");
        f.check_invariants();
    }
}

/// D3': IRQ53 runs right after ISER53 and before init's next access. What it wrote survives
/// init's return, and init's remaining accesses are the INTR read, the ISPR and the report's
/// reads, which see the line disabled.
#[test]
fn immediate_detection_survives_the_rest_of_init() {
    for delivery in DELIVERIES {
        let mut f = Fake::after_bring_up(delivery);
        f.injections.push((At::Iser, Event::Edge(Edge::Up)));
        let report = f.arm();
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
                Op::Write(ISPR, IRQ_BIT),
            ]
            .into_iter()
            .chain(report_reads(0, false))
            .collect::<Vec<_>>()
        );
        assert_eq!((report.inte, report.nvic53), (0, false));
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
/// let the bring-up's latches through) is treated as a reset. The corruption is not a write
/// of the platform's, so it is set without a trace.
#[test]
fn any_other_interrupt_source_is_lost() {
    let corruptions = [(PCIE_INTF, 1 << 3), (PCIE_INTE, BRING_UP_INTE)];
    for delivery in DELIVERIES {
        for (address, value) in corruptions {
            let mut f = Fake::after_bring_up(delivery);
            f.arm();
            f.set(address, value);
            f.run_thread(2);
            f.assert_lost_once();
            assert_eq!(f.get(PCIE_INTR), BRING_UP_LATCHES, "INTR kept");
            f.check_invariants();
        }
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

/// I5-53 at run time: `vec53_ok` only for the detector's handler, not for the default one the
/// linker script would put in its place, nor for anything else.
#[test]
fn vec53_ok_needs_the_detectors_handler() {
    for (vector, ok) in [
        (HANDLERS.detector, true),
        (HANDLERS.default, false),
        (HANDLERS.detector + 2, false),
    ] {
        let mut f = Fake::after_bring_up(Delivery::Level);
        f.registers.insert(VECTOR53, vector as u32);
        assert_eq!(f.arm().vec53_ok, ok, "{vector:#x}");
    }
}

/// The words the detector publishes decode to their states (the decoding itself is
/// rp1-proto's, shared with Linux).
#[test]
fn published_words_decode_to_their_states() {
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
}

/// Step 0: `Initializing` in the detector's state, then in `PLAT_PCIE`.
#[test]
fn step_0_publishes_initializing() {
    let mut f = Fake::after_bring_up(Delivery::Level);
    f.words = [0; WORD_COUNT];
    initializing(&mut f);
    let state = status::PLAT_PCIE_INITIALIZING;
    assert_eq!(
        f.init_ops(),
        [
            Op::Store(Word::State, state),
            Op::Write(STATUS_WORD, state),
            Op::Barrier
        ]
    );
    assert_eq!(f.state(), PcieState::Initializing);
    assert_eq!(f.published(), PcieState::Initializing);
}
