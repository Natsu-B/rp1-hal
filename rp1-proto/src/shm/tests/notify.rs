//! Linux -> RP1 notification in every schedule up to a preemption bound: two writers, each
//! on its own ring, share one doorbell into one reader (protocol.md §3). A writer does
//! `push; if reader_asleep { raise }`; the reader's handler runs while the doorbell is set
//! and does `ack; drain(A); drain(B)`. A run stalls when it ends with a message on a ring
//! and the reader asleep: no later writer notifies it. The negative variants must find
//! such a schedule.
//!
//! Three sides are too many for an unbounded search, so it is bounded by preemptions (CHESS
//! style, [`Sched::drive`]): every negative here stalls within the bound, among them the
//! stock handler with either readback (W3 in `reader_asleep`, R4 in the arm of `drain`)
//! removed, through the lab controls' hooks.
use super::model::{Path, Sched, explore};
use super::{Mem, mem};
use crate::shm::*;
extern crate std;
use std::boxed::Box;
use std::vec::Vec;

const DEPTH: u32 = 4;
const LEN: usize = CONTROL_SIZE + DEPTH as usize * 16;
const READER: usize = 2;
const PREEMPTIONS: usize = 2;

#[derive(Clone, Copy, Eq, PartialEq)]
enum Handler {
    /// `ack` once, then drain every ring; raise the doorbell again on `More`.
    Stock,
    /// Negative: `ack` before each ring's drain.
    AckPerRing,
    /// Negative: drop `More` instead of raising the doorbell.
    IgnoreMore,
}

#[derive(Clone, Copy)]
struct Case {
    handler: Handler,
    max: u32,
    /// Messages each writer sends.
    sends: [u32; 2],
    /// Writer A stops after its first push (before `reader_asleep`) and a new writer
    /// attaches; `Some(true)` with the opening notify, `Some(false)` without it.
    reopen: Option<bool>,
    /// Bound of the search (see [`Sched::drive`]).
    preemptions: usize,
    /// Negative when false: the writers skip the readback of `head` in `reader_asleep` (W3).
    writer_readback: bool,
    /// Negative when false: the reader skips the readback of its arm in `drain` (R4).
    reader_readback: bool,
}

/// One ring: what the reader sees (`data`) and the writer sees (`ctrl`), plus the posted
/// copies of each side's remote stores.
struct Ring {
    data: Box<Mem<LEN>>,
    data_shadow: Box<Mem<LEN>>,
    ctrl: Box<Mem<CONTROL_SIZE>>,
    ctrl_shadow: Box<Mem<CONTROL_SIZE>>,
}
impl Ring {
    fn new() -> Self {
        Self {
            data: mem(),
            data_shadow: mem(),
            ctrl: mem(),
            ctrl_shadow: mem(),
        }
    }
    /// The writer's view: its copy of `data` and the real `ctrl`.
    fn tx_view(&mut self) -> (usize, usize) {
        let d = self.data_shadow.0.as_mut_ptr() as usize;
        (d, self.ctrl.0.as_mut_ptr() as usize)
    }
    fn rx(&mut self) -> RingRx<u32> {
        let (d, c) = (self.data.0.as_mut_ptr(), self.ctrl_shadow.0.as_mut_ptr());
        unsafe { RingRx::attach(d, LEN, c, CONTROL_SIZE, DEPTH, Overflow::Reject) }.unwrap()
    }
    fn paths(&self, writer: usize) -> [Path; 2] {
        [
            Path::new(
                writer,
                self.data_shadow.0.as_ptr(),
                self.data.0.as_ptr(),
                LEN,
            ),
            Path::new(
                READER,
                self.ctrl_shadow.0.as_ptr(),
                self.ctrl.0.as_ptr(),
                CONTROL_SIZE,
            ),
        ]
    }
}

fn attach_tx((data, ctrl): (usize, usize)) -> RingTx<u32> {
    let (d, c) = (data as *mut u8, ctrl as *const u8);
    unsafe { RingTx::attach(d, LEN, c, CONTROL_SIZE, DEPTH, Overflow::Reject) }.unwrap()
}

fn send(sched: &Sched, case: Case, me: usize, tx: &mut RingTx<u32>, value: u32) {
    tx.push(&value).unwrap();
    let asleep = if case.writer_readback {
        tx.reader_asleep()
    } else {
        tx.__reader_asleep_without_readback()
    };
    if asleep {
        sched.raise(me);
    }
}

fn handle(sched: &Sched, case: Case, rx: &mut [RingRx<u32>; 2], got: &mut [Vec<u32>; 2]) {
    if case.handler != Handler::AckPerRing {
        sched.ack(READER);
    }
    for (rx, got) in rx.iter_mut().zip(got.iter_mut()) {
        if case.handler == Handler::AckPerRing {
            sched.ack(READER);
        }
        let progress = if case.reader_readback {
            rx.drain(case.max, |v| got.push(v))
        } else {
            rx.__drain_without_readback(case.max, |v| got.push(v))
        }
        .unwrap();
        if progress == Progress::More && case.handler != Handler::IgnoreMore {
            sched.raise(READER);
        }
    }
}

/// Explore every schedule of `case`; returns (runs, stalled runs).
fn check(case: Case) -> (usize, usize) {
    let mut stalls = 0;
    let runs = explore(|choose| {
        let mut rings = [Ring::new(), Ring::new()];
        let views = rings.each_mut().map(Ring::tx_view);
        let txs = views.map(attach_tx);
        let mut rxs = rings.each_mut().map(Ring::rx);
        let paths: Vec<Path> = rings
            .iter_mut()
            .enumerate()
            .flat_map(|(writer, ring)| ring.paths(writer))
            .collect();
        let sched = Sched::new(3, paths);
        let mut trace = Vec::new();
        let got = std::thread::scope(|s| {
            for (me, (mut tx, view)) in txs.into_iter().zip(views).enumerate() {
                let sched = &sched;
                s.spawn(move || {
                    sched.side(me, || {
                        let mut first = 0;
                        if let (0, Some(notify)) = (me, case.reopen) {
                            // W1-W2 only, then the process is gone and a new one attaches.
                            tx.push(&0).unwrap();
                            tx = attach_tx(view);
                            if notify && tx.reader_asleep() {
                                sched.raise(me);
                            }
                            first = 1;
                        }
                        for k in first..case.sends[me] {
                            send(sched, case, me, &mut tx, k);
                        }
                    })
                });
            }
            let reader = s.spawn(|| {
                sched.side(READER, || {
                    let mut got = [Vec::new(), Vec::new()];
                    // Opening the rings drains them once: that arms them.
                    handle(&sched, case, &mut rxs, &mut got);
                    while sched.wait_doorbell(READER) {
                        handle(&sched, case, &mut rxs, &mut got);
                    }
                    got
                })
            });
            sched.drive(choose, &mut trace, case.preemptions);
            reader.join().unwrap()
        });
        for p in &sched.paths {
            p.land_all();
        }
        for (ring, got) in got.iter().enumerate() {
            assert!(
                got.iter().copied().eq(0..got.len() as u32),
                "ring {ring} out of order: {got:?} (schedule={trace:?})"
            );
        }
        let left: u32 = rxs.iter().map(RingRx::available).sum();
        let sent: u32 = case.sends.iter().sum();
        assert_eq!(got.iter().map(Vec::len).sum::<usize>() as u32 + left, sent);
        stalls += usize::from(left != 0);
    });
    (runs, stalls)
}

const STOCK: Case = Case {
    handler: Handler::Stock,
    max: 16,
    sends: [1, 1],
    reopen: None,
    preemptions: PREEMPTIONS,
    writer_readback: true,
    reader_readback: true,
};

fn timed(case: Case) -> (usize, usize) {
    let start = std::time::Instant::now();
    let r = check(case);
    std::println!("runs={} stalls={} in {:?}", r.0, r.1, start.elapsed());
    r
}

/// (i) One `ack` at the top of the handler, then every ring: no schedule stalls (22,691 runs).
#[test]
fn doorbell_shared_by_two_rings_never_stalls() {
    let (runs, stalls) = timed(STOCK);
    assert_eq!(stalls, 0, "runs={runs}");
    assert!(runs > 1000, "runs={runs}");
}

/// (i) negative: an `ack` before each ring's drain clears a force meant for a ring that was
/// already drained and armed (278 of 23,439 runs stall).
#[test]
fn ack_per_ring_stalls() {
    let (_, stalls) = timed(Case {
        handler: Handler::AckPerRing,
        ..STOCK
    });
    assert!(stalls > 0);
}

/// (iii) `More` leaves the ring unarmed; raising the doorbell again keeps it moving
/// (4,367 runs).
#[test]
fn more_with_raise_never_stalls() {
    let (runs, stalls) = timed(Case {
        max: 1,
        sends: [2, 0],
        preemptions: 1,
        ..STOCK
    });
    assert_eq!(stalls, 0, "runs={runs}");
    assert!(runs > 1000, "runs={runs}");
}

/// (iii) negative: a handler that ignores `More` (max 1, two messages) stalls (1,812 of
/// 2,878 runs).
#[test]
fn ignoring_more_stalls() {
    let (_, stalls) = timed(Case {
        handler: Handler::IgnoreMore,
        max: 1,
        sends: [2, 0],
        preemptions: 1,
        ..STOCK
    });
    assert!(stalls > 0);
}

/// (iv) A writer gone between `push` and `reader_asleep`: the next writer's opening notify
/// wakes the reader (28,043 runs).
#[test]
fn reopened_writer_notifies_on_open() {
    let (runs, stalls) = timed(Case {
        reopen: Some(true),
        ..STOCK
    });
    assert_eq!(stalls, 0, "runs={runs}");
    assert!(runs > 1000, "runs={runs}");
}

/// (iv) negative: without the opening notify the message of the gone writer stays (4,194 of
/// 13,201 runs stall).
#[test]
fn reopen_without_opening_notify_stalls() {
    let (_, stalls) = timed(Case {
        reopen: Some(false),
        ..STOCK
    });
    assert!(stalls > 0);
}

/// One writer with one message and one preemption: small enough for the readback negatives,
/// and the stock protocol never stalls there.
const ONE_MESSAGE: Case = Case {
    sends: [1, 0],
    preemptions: 1,
    ..STOCK
};

/// Negative: writers that look at the reader without first reading `head` back (W3) can see
/// it awake before their message has landed, while it arms and re-checks without seeing it
/// (1,591 of 4,125 runs stall; with both readbacks none of 1,021).
#[test]
fn writer_without_readback_stalls() {
    assert_eq!(timed(ONE_MESSAGE).1, 0);
    let (runs, stalls) = timed(Case {
        writer_readback: false,
        ..ONE_MESSAGE
    });
    assert!(stalls > 0, "runs={runs}");
}

/// Negative: a reader that re-checks without first reading its arm back (R4) can miss a
/// message whose writer still sees it awake (243 of 1,778 runs stall).
#[test]
fn reader_without_readback_stalls() {
    let (runs, stalls) = timed(Case {
        reader_readback: false,
        ..ONE_MESSAGE
    });
    assert!(stalls > 0, "runs={runs}");
}
