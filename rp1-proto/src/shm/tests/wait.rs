//! The sleep/notify handshake, including the exhaustive interleaving check.
use super::{Mem, mem};
use crate::Error;
use crate::shm::access::{ld, st};
use crate::shm::*;
extern crate std;
use std::vec;

fn wait_pair(
    depth: u32,
) -> (
    std::boxed::Box<Mem<4224>>,
    std::boxed::Box<Mem<CONTROL_SIZE>>,
) {
    assert!(CONTROL_SIZE + depth as usize * 24 <= 4224);
    (mem(), mem())
}

#[test]
fn wait_handshake() {
    let (mut data, mut ctrl) = wait_pair(2);
    let (d, c) = (data.0.as_mut_ptr(), ctrl.0.as_mut_ptr());
    let mut tx = unsafe { RingTx::<u32>::attach(d, 160, c, 128, 2, Overflow::Reject) }.unwrap();
    let mut rx = unsafe { RingRx::<u32>::attach(d, 160, c, 128, 2, Overflow::Reject) }.unwrap();
    // Awake reader: no notification.
    assert!(!tx.wake_needed());
    // Arm, then a push: one notification for this sleep, not two.
    assert!(rx.prepare_wait());
    tx.push(&1).unwrap();
    assert!(tx.wake_needed());
    assert!(!tx.wake_needed());
    tx.push(&2).unwrap();
    assert!(!tx.wake_needed());
    rx.finish_wait();
    assert!(!tx.wake_needed());
    // Data present: arming does not sleep and leaves the reader awake.
    assert!(!rx.prepare_wait());
    assert!(!tx.wake_needed());
    assert_eq!(rx.pop().unwrap(), Some(1));
    assert_eq!(rx.pop().unwrap(), Some(2));
    // The next sleep gets a new value and so a new notification.
    assert!(rx.prepare_wait());
    assert!(tx.wake_needed());
    rx.finish_wait();
}

/// The race the handshake closes, in its sequentially consistent orders: a push between
/// the reader's emptiness check and its arm is seen by the re-check; a push after the arm
/// is notified. Either way the reader never sleeps on a non-empty ring unnotified.
#[test]
fn wait_with_never_sleeps_unnotified() {
    let (mut data, mut ctrl) = wait_pair(4);
    let (d, c) = (data.0.as_mut_ptr(), ctrl.0.as_mut_ptr());
    let mut tx = unsafe { RingTx::<u32>::attach(d, 224, c, 128, 4, Overflow::Reject) }.unwrap();
    let mut rx = unsafe { RingRx::<u32>::attach(d, 224, c, 128, 4, Overflow::Reject) }.unwrap();
    // Data already there.
    tx.push(&7).unwrap();
    assert_eq!(rx.wait_with(|| unreachable!()), Wait::Ready);
    assert_eq!(rx.pop().unwrap(), Some(7));
    // Push lands after the arm, before the sleep: the writer notifies.
    let mut notified = false;
    let r = rx.wait_with(|| {
        tx.push(&8).unwrap();
        notified = tx.wake_needed();
        notified
    });
    assert!(notified);
    assert_eq!(r, Wait::Woken);
    assert_eq!(rx.pop().unwrap(), Some(8));
    // Nothing happens: the bounded sleep times out and the reader is awake again.
    assert_eq!(rx.wait_with(|| false), Wait::Timeout);
    assert!(!tx.wake_needed());
}

/// Restarts on either side: a writer attaching while the reader sleeps notifies it; a
/// reader that died asleep comes back awake, and its next sleep is notified again.
#[test]
fn wait_counter_survives_restarts() {
    let (mut data, mut ctrl) = wait_pair(2);
    let (d, c) = (data.0.as_mut_ptr(), ctrl.0.as_mut_ptr());
    let mut rx = unsafe { RingRx::<u32>::attach(d, 160, c, 128, 2, Overflow::Reject) }.unwrap();
    let mut tx = unsafe { RingTx::<u32>::attach(d, 160, c, 128, 2, Overflow::Reject) }.unwrap();
    assert!(rx.prepare_wait());
    assert!(tx.wake_needed());
    // Reader dies asleep; a new one attaches.
    let mut rx = unsafe { RingRx::<u32>::attach(d, 160, c, 128, 2, Overflow::Reject) }.unwrap();
    assert!(!tx.wake_needed());
    assert!(rx.prepare_wait());
    assert!(tx.wake_needed());
    // Writer restarts while the reader sleeps.
    let mut tx = unsafe { RingTx::<u32>::attach(d, 160, c, 128, 2, Overflow::Reject) }.unwrap();
    assert!(tx.wake_needed());
    assert!(!tx.wake_needed());
    rx.finish_wait();
}

/// Two rings, one notification: every ring is armed for the sleep, so either writer
/// notifies; data on a later ring cancels the arms of the earlier ones.
#[test]
fn wait_any_arms_every_ring() {
    let (mut d0, mut c0) = wait_pair(2);
    let (mut d1, mut c1) = wait_pair(2);
    let (d0, c0) = (d0.0.as_mut_ptr(), c0.0.as_mut_ptr());
    let (d1, c1) = (d1.0.as_mut_ptr(), c1.0.as_mut_ptr());
    let ow = Overflow::Reject;
    let mut tx0 = unsafe { RingTx::<u32>::attach(d0, 160, c0, 128, 2, ow) }.unwrap();
    let mut rx0 = unsafe { RingRx::<u32>::attach(d0, 160, c0, 128, 2, ow) }.unwrap();
    let mut tx1 = unsafe { RingTx::<u64>::attach(d1, 160, c1, 128, 2, ow) }.unwrap();
    let mut rx1 = unsafe { RingRx::<u64>::attach(d1, 160, c1, 128, 2, ow) }.unwrap();
    // The second ring's writer notifies while both are armed.
    let r = wait_any_with(&mut [&mut rx0, &mut rx1], || {
        tx1.push(&5).unwrap();
        tx1.wake_needed()
    });
    assert_eq!(r, Wait::Woken);
    assert!(!tx0.wake_needed() && !tx1.wake_needed(), "all disarmed");
    assert_eq!(
        wait_any_with(&mut [&mut rx0, &mut rx1], || unreachable!()),
        Wait::Ready
    );
    assert_eq!(rx1.pop().unwrap(), Some(5));
    // A message that arrives on ring 1 between the first check and its arm: the re-check
    // cancels the sleep and ring 0's arm is undone.
    struct Arrives;
    impl Waiter for Arrives {
        fn available(&self) -> u32 {
            0
        }
        fn prepare_wait(&mut self) -> bool {
            false
        }
        fn finish_wait(&mut self) {}
    }
    assert_eq!(
        wait_any_with(&mut [&mut rx0, &mut Arrives], || unreachable!()),
        Wait::Ready
    );
    assert!(
        !tx0.wake_needed(),
        "ring 0 disarmed after the cancelled sleep"
    );
    // Ring 0's writer notifies too.
    let r = wait_any_with(&mut [&mut rx0, &mut rx1], || {
        tx0.push(&7).unwrap();
        tx0.wake_needed()
    });
    assert_eq!(r, Wait::Woken);
    assert_eq!(rx0.pop().unwrap(), Some(7));
}

/// Counter wrap: after the notified sleep, 2^31 - 1 more sleeps bring the counter back to
/// the same odd value. That sleep is not notified (documented on [`WAITING`]); the reader's
/// bounded re-check still finds the data.
#[test]
fn wait_counter_wrap_can_alias() {
    let (mut data, mut ctrl) = wait_pair(2);
    let (d, c) = (data.0.as_mut_ptr(), ctrl.0.as_mut_ptr());
    let mut tx = unsafe { RingTx::<u32>::attach(d, 160, c, 128, 2, Overflow::Reject) }.unwrap();
    let mut rx = unsafe { RingRx::<u32>::attach(d, 160, c, 128, 2, Overflow::Reject) }.unwrap();
    assert!(rx.prepare_wait());
    assert!(tx.wake_needed());
    rx.finish_wait();
    let w = unsafe { ld(c, WAITING) };
    // 2^31 - 1 more sleeps, 2 steps each.
    unsafe { st(c, WAITING, w.wrapping_add(u32::MAX - 1)) };
    let mut rx = unsafe { RingRx::<u32>::attach(d, 160, c, 128, 2, Overflow::Reject) }.unwrap();
    let r = rx.wait_with(|| {
        tx.push(&1).unwrap();
        assert!(!tx.wake_needed(), "aliased sleep is not notified");
        false
    });
    assert_eq!(r, Wait::Timeout);
    assert_eq!(rx.wait_with(|| unreachable!()), Wait::Ready);
    assert_eq!(rx.pop().unwrap(), Some(1));
}

/// The handshake never lets the reader sleep on a published message unnotified, and never
/// notifies before the message has landed, in any interleaving of one `push` +
/// `wake_needed` against one `wait_with`. Fails if the reader's re-check or either
/// readback is removed (each gives a schedule where the reader sleeps unnotified).
#[test]
fn wait_handshake_all_interleavings() {
    use super::model::{Path, Sched, explore};
    const LEN: usize = CONTROL_SIZE + 4 * 16;
    // prior = 0: fresh ring; 1: the reader already had one notified sleep and a message.
    for prior in 0..2 {
        let runs = explore(|choose| {
            let (mut data, mut data_shadow) = (mem::<LEN>(), mem::<LEN>());
            let (mut ctrl, mut ctrl_shadow) = (mem::<CONTROL_SIZE>(), mem::<CONTROL_SIZE>());
            let (dr, ds) = (data.0.as_mut_ptr(), data_shadow.0.as_mut_ptr());
            let (cr, cs) = (ctrl.0.as_mut_ptr(), ctrl_shadow.0.as_mut_ptr());
            let head_real = dr as usize;
            let ow = Overflow::Reject;
            let mut tx = unsafe { RingTx::<u32>::attach(ds, LEN, cr, 128, 4, ow) }.unwrap();
            let mut rx = unsafe { RingRx::<u32>::attach(dr, LEN, cs, 128, 4, ow) }.unwrap();
            let sync = || unsafe {
                core::ptr::copy_nonoverlapping(ds, dr, LEN);
                core::ptr::copy_nonoverlapping(cs, cr, CONTROL_SIZE);
            };
            if prior == 1 {
                assert!(rx.prepare_wait());
                sync();
                tx.push(&0).unwrap();
                assert!(tx.wake_needed());
                sync();
                rx.finish_wait();
                assert_eq!(rx.pop().unwrap(), Some(0));
                sync();
            }
            let sched = Sched::new([
                Path {
                    shadow: ds as usize,
                    real: dr as usize,
                    len: LEN,
                },
                Path {
                    shadow: cs as usize,
                    real: cr as usize,
                    len: CONTROL_SIZE,
                },
            ]);
            let mut trace = vec![];
            let notified_now = core::sync::atomic::AtomicBool::new(false);
            let (notified, outcome) = std::thread::scope(|s| {
                let w = s.spawn(|| {
                    sched.side(0, || {
                        tx.push(&1).unwrap();
                        let notify = tx.wake_needed();
                        notified_now.store(notify, core::sync::atomic::Ordering::Relaxed);
                        // The notification must not overtake the message.
                        let landed = unsafe { ld(head_real as *const u8, HEAD) } == tx.head();
                        (notify, landed)
                    })
                });
                let r = s.spawn(|| {
                    sched.side(1, || {
                        rx.wait_with(|| {
                            sched.sleep(1);
                            notified_now.load(core::sync::atomic::Ordering::Relaxed)
                        })
                    })
                });
                sched.drive(choose, &mut trace);
                (w.join().unwrap(), r.join().unwrap())
            });
            let (notified, landed) = notified;
            assert_eq!(outcome == Wait::Woken, notified && outcome != Wait::Ready);
            assert!(
                outcome == Wait::Ready || notified,
                "lost wakeup: reader sleeps on a published message (prior={prior}, \
                 schedule={trace:?}; 0 writer, 1 reader, 2 land data, 3 land ctrl)"
            );
            assert!(
                !notified || landed,
                "notified before head landed (prior={prior}, schedule={trace:?})"
            );
        });
        assert!(runs > 10, "runs={runs}");
    }
}

/// `reader_asleep` has no "already notified" memory: it sees every sleep, also across the
/// counter wrap. `wake_needed` on the same sleeps misses exactly the one that reuses the
/// value it notified last (2^31 sleeps ago; the unnotified ones in between are simulated by
/// moving the counter).
#[test]
fn reader_asleep_sees_every_sleep_across_the_wrap() {
    let (mut data, mut ctrl) = wait_pair(2);
    let (d, c) = (data.0.as_mut_ptr(), ctrl.0.as_mut_ptr());
    unsafe { st(c, WAITING, u32::MAX - 3) };
    let mut tx = unsafe { RingTx::<u32>::attach(d, 160, c, 128, 2, Overflow::Reject) }.unwrap();
    let mut rx = unsafe { RingRx::<u32>::attach(d, 160, c, 128, 2, Overflow::Reject) }.unwrap();
    assert!(!tx.reader_asleep());
    // One notified sleep with the value u32::MAX - 2, then 2^31 - 1 unnotified ones.
    assert!(rx.prepare_wait());
    assert!(tx.wake_needed());
    rx.finish_wait();
    unsafe { st(c, WAITING, u32::MAX - 3) };
    let mut rx = unsafe { RingRx::<u32>::attach(d, 160, c, 128, 2, Overflow::Reject) }.unwrap();
    // Sleeps u32::MAX - 2, u32::MAX, 1, 3.
    let (mut asleep, mut woken) = (0, 0);
    for _ in 0..4 {
        assert!(rx.prepare_wait());
        asleep += usize::from(tx.reader_asleep());
        woken += usize::from(tx.wake_needed());
        rx.finish_wait();
        assert!(!tx.reader_asleep());
    }
    assert_eq!(unsafe { ld(c, WAITING) }, 4);
    assert_eq!(asleep, 4);
    assert_eq!(
        woken, 3,
        "wake_needed misses the sleep that reuses u32::MAX - 2"
    );
}

#[test]
fn drain_stops_at_the_budget_without_arming() {
    let (mut data, mut ctrl) = wait_pair(4);
    let (d, c) = (data.0.as_mut_ptr(), ctrl.0.as_mut_ptr());
    let mut tx = unsafe { RingTx::<u32>::attach(d, 224, c, 128, 4, Overflow::Reject) }.unwrap();
    let mut rx = unsafe { RingRx::<u32>::attach(d, 224, c, 128, 4, Overflow::Reject) }.unwrap();
    let mut got = vec![];
    // Exactly `max` messages: all passed, `More`, not armed.
    assert_eq!(tx.push_slice(&[1, 2]), 2);
    assert_eq!(rx.drain(2, |v| got.push(v)), Ok(Progress::More));
    assert_eq!(got, [1, 2]);
    assert!(!tx.reader_asleep());
    // The next call finds the ring empty and arms it.
    assert_eq!(rx.drain(2, |v| got.push(v)), Ok(Progress::Idle));
    assert!(tx.reader_asleep());
    // It marks the reader awake first; `max = 0` passes one message.
    assert_eq!(tx.push_slice(&[3, 4]), 2);
    assert_eq!(rx.drain(0, |v| got.push(v)), Ok(Progress::More));
    assert_eq!(got, [1, 2, 3]);
    assert!(!tx.reader_asleep());
    // Fewer than `max`: everything, then armed.
    assert_eq!(rx.drain(16, |v| got.push(v)), Ok(Progress::Idle));
    assert_eq!(got, [1, 2, 3, 4]);
    assert!(tx.reader_asleep());
}

#[test]
fn drain_reports_corruption_unarmed() {
    let (mut data, mut ctrl) = wait_pair(4);
    let (d, c) = (data.0.as_mut_ptr(), ctrl.0.as_mut_ptr());
    let mut tx = unsafe { RingTx::<u32>::attach(d, 224, c, 128, 4, Overflow::Reject) }.unwrap();
    let mut rx = unsafe { RingRx::<u32>::attach(d, 224, c, 128, 4, Overflow::Reject) }.unwrap();
    assert_eq!(rx.drain(1, |_| unreachable!()), Ok(Progress::Idle));
    assert_eq!(tx.push_slice(&[1, 2]), 2);
    // The second slot's sequence number is wrong.
    unsafe { st(d, CONTROL_SIZE + 16, 7) };
    let mut got = vec![];
    assert_eq!(rx.drain(4, |v| got.push(v)), Err(Error::Corrupt));
    assert_eq!(got, [1]);
    assert!(!tx.reader_asleep());
}
