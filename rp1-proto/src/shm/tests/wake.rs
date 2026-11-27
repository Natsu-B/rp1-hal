extern crate std;

use super::{
    mem,
    model::{Path, Sched, explore},
};
use crate::{
    Error,
    shm::{
        CONTROL_SIZE, HEAD, Header, HeaderInit, TAIL,
        access::{ld, st},
        wake::*,
    },
    wake::{BootEpoch, LATCHED_WAKE, Request, Session, SessionError, mailbox},
};

#[test]
fn session_header_binds_note_epoch_generation_and_caps() {
    let mut hdr = mem::<64>();
    let base = hdr.0.as_mut_ptr();
    let epoch = 8;
    let hash = [5; 32];
    let session = Session::new(epoch, 2).unwrap();
    let mut init = unsafe { HeaderInit::begin_v2(base, 0) }.unwrap();
    init.write_v2(&hash, 3, 7, epoch, LATCHED_WAKE).unwrap();
    assert!(unsafe { Header::open_v2(base, &hash, session, LATCHED_WAKE) }.is_err());
    assert_eq!(init.publish_v2(), session.generation());
    let header = unsafe { Header::open_v2(base, &hash, session, LATCHED_WAKE) }.unwrap();
    assert_eq!(header.regions(), 3);
    assert_eq!(
        unsafe { Header::open(base, &hash) }.err(),
        Some(Error::NotReady)
    );
    for (digest, expected, caps) in [
        ([6; 32], session, LATCHED_WAKE),
        (hash, Session::new(epoch + 2, 2).unwrap(), LATCHED_WAKE),
        (hash, Session::new(epoch, 4).unwrap(), LATCHED_WAKE),
        (hash, session, 0),
    ] {
        assert_eq!(
            unsafe { Header::open_v2(base, &digest, expected, caps) }.err(),
            Some(Error::Mismatch)
        );
    }
    assert_eq!(
        unsafe { HeaderInit::begin_v2(base, 0xffff_fffe) }.err(),
        Some(SessionError::Exhausted)
    );
    header.check().unwrap();
    let _replacement = unsafe { HeaderInit::begin_v2(base, 2) }.unwrap();
    assert_eq!(header.check(), Err(Error::Reset));
}

#[test]
fn mailbox_rejects_stale_boots_and_results_and_capability_mismatches() {
    let request = Request {
        epoch: 6,
        seq: 11,
        required_caps: LATCHED_WAKE,
    };
    let accept = |opcode, epoch, offered| request.accept(opcode, epoch, offered);
    for opcode in [0, 1, 2, 4] {
        assert_eq!(
            accept(opcode, 6, LATCHED_WAKE),
            Err(SessionError::Unsupported)
        );
    }
    assert_eq!(
        accept(mailbox::REQUEST_OPCODE, 6, 0),
        Err(SessionError::Unsupported)
    );
    assert_eq!(
        accept(mailbox::REQUEST_OPCODE, 8, LATCHED_WAKE),
        Err(SessionError::Restarted)
    );
    assert_eq!(
        accept(mailbox::REQUEST_OPCODE, 6, LATCHED_WAKE),
        Ok(LATCHED_WAKE)
    );
    assert_eq!(
        request.check_result(6, 10, LATCHED_WAKE, LATCHED_WAKE),
        Err(SessionError::NotReady)
    );
    assert_eq!(
        request.check_result(6, 11, 0, LATCHED_WAKE),
        Err(SessionError::Unsupported)
    );
    assert_eq!(
        request.check_result(6, 11, LATCHED_WAKE, LATCHED_WAKE),
        Ok(())
    );
}

#[test]
fn stale_handles_never_store_and_reopen_advances_the_arm() {
    let mut data = mem::<CONTROL_SIZE>();
    let mut ctrl = mem::<CONTROL_SIZE>();
    let base = ctrl.0.as_mut_ptr();
    let words = unsafe { Words::rx(data.0.as_mut_ptr(), base) };
    let boot = Session::new(2, 2).unwrap();
    let live = || Ok(boot);
    let original = {
        let mut waiter = unsafe { words.waiter(boot) };
        let mut publisher = unsafe { words.publisher(boot) };
        let original = waiter.arm(live).unwrap();
        assert!(publisher.publish(|| {}, live).unwrap());
        assert!(waiter.acknowledge(original, live).unwrap());
        assert!(!publisher.publish(|| {}, live).unwrap());
        original // The owner exits before releasing the arm.
    };
    let mut waiter = unsafe { words.waiter(boot) };
    let mut publisher = unsafe { words.publisher(boot) };
    let current = waiter.arm(live).unwrap();
    assert!(!waiter.pending(current, live).unwrap());
    assert!(!waiter.cancel(original, live).unwrap());
    assert!(publisher.publish(|| {}, live).unwrap());
    assert!(!waiter.acknowledge(original, live).unwrap());
    assert!(waiter.pending(current, live).unwrap());

    let old = (data.0, ctrl.0);
    for live in [
        Session::new(4, 2), // boot changed while SHM generation is the same
        Session::new(2, 4),
        Session::new(2, 3), // initialization in progress
    ] {
        assert_eq!(
            publisher.publish(|| panic!("stale readback"), || live),
            Err(SessionError::Restarted)
        );
        assert_eq!(waiter.arm(|| live), Err(SessionError::Restarted));
        assert_eq!(
            waiter.acknowledge(current, || live),
            Err(SessionError::Restarted)
        );
        assert_eq!(
            waiter.cancel(current, || live),
            Err(SessionError::Restarted)
        );
    }
    let restarted_during_operation = || {
        let mut samples = [boot, Session::new(4, 2).unwrap()].into_iter();
        move || Ok(samples.next().unwrap())
    };
    assert_eq!(
        waiter.acknowledge(current, restarted_during_operation()),
        Err(SessionError::Restarted)
    );
    assert_eq!(
        waiter.cancel(current, restarted_during_operation()),
        Err(SessionError::Restarted)
    );
    assert_eq!(
        waiter.arm(restarted_during_operation()),
        Err(SessionError::Restarted)
    );
    assert_eq!((data.0, ctrl.0), old);
    assert!(waiter.cancel(current, live).unwrap());
    assert!(!waiter.pending(current, live).unwrap());

    unsafe { st(base, RX_WAIT, (u32::MAX - 3).to_le()) };
    let last = waiter.arm(live).unwrap();
    assert!(waiter.cancel(last, live).unwrap());
    assert_eq!(waiter.arm(live), Err(SessionError::Exhausted));
    assert_eq!(u32::from_le(unsafe { ld(base, RX_WAIT) }), u32::MAX - 1);
}

#[test]
fn invalidation_drains_posted_stores_before_reusing_the_mailbox() {
    for drain_before_reuse in [true, false] {
        let mut old_ack = mem::<4>();
        let mut page = mem::<{ mailbox::END }>();
        let shadow = old_ack.0.as_mut_ptr();
        let base = page.0.as_mut_ptr();
        let ack = unsafe { base.add(mailbox::ACK) };
        let old_write = Path::new(0, shadow, ack, 4);

        // The old owner's ACK is issued but has not reached the peer.
        unsafe { st(shadow, 0, 1u32.to_le()) };
        old_write.post();
        assert_eq!(unsafe { ld(ack, 0) }, 0);
        if drain_before_reuse {
            old_write.land(1);
        }

        page.0.fill(0);
        let boot = BootEpoch::new(4).unwrap();
        let words = unsafe { Words::link(base) };
        let mut waiter = unsafe { words.waiter(boot) };
        let mut publisher = unsafe { words.publisher(boot) };
        let ticket = waiter.arm(|| Ok(boot)).unwrap();
        assert!(publisher.publish(|| {}, || Ok(boot)).unwrap());
        if !drain_before_reuse {
            old_write.land(1);
        }
        assert_eq!(
            waiter.pending(ticket, || Ok(boot)).unwrap(),
            drain_before_reuse
        );
    }
}

// Each side's stores go through a different posted window. A writer that observes arm
// raises a held-level signal, otherwise the waiter must see the commit before sleeping.
fn stalled(credit: bool, arm_flush: bool, publish_flush: bool) -> usize {
    let mut stalls = 0;
    let runs = explore(|choose| {
        let mut data = mem::<CONTROL_SIZE>();
        let mut data_shadow = mem::<CONTROL_SIZE>();
        let mut ctrl = mem::<CONTROL_SIZE>();
        let mut ctrl_shadow = mem::<CONTROL_SIZE>();
        let d = data.0.as_mut_ptr() as usize;
        let ds = data_shadow.0.as_mut_ptr() as usize;
        let c = ctrl.0.as_mut_ptr() as usize;
        let cs = ctrl_shadow.0.as_mut_ptr() as usize;
        let paths = [
            Path::new(
                usize::from(credit),
                ds as *const u8,
                d as *const u8,
                CONTROL_SIZE,
            ),
            Path::new(
                usize::from(!credit),
                cs as *const u8,
                c as *const u8,
                CONTROL_SIZE,
            ),
        ];
        let sched = Sched::new(2, paths.into());
        let words = |data, ctrl| {
            if credit {
                unsafe { Words::credit(data, ctrl) }
            } else {
                unsafe { Words::rx(data, ctrl) }
            }
        };
        let (posted, visible, commit, arm) = if credit {
            (cs, c, TAIL, CREDIT_WAIT)
        } else {
            (ds, d, HEAD, RX_WAIT)
        };
        let live = || Session::new(2, 2);
        let ready = std::thread::scope(|scope| {
            let sched = &sched;
            scope.spawn(move || {
                sched.side(0, || {
                    let (data, ctrl) = if credit { (d, cs) } else { (ds, c) };
                    let mut publisher = unsafe {
                        words(data as *mut u8, ctrl as *mut u8).publisher(live().unwrap())
                    };
                    unsafe { st(posted as *mut u8, commit, 1) };
                    if publisher
                        .publish(
                            || {
                                if publish_flush {
                                    let _ = unsafe { ld(posted as *const u8, commit) };
                                }
                            },
                            live,
                        )
                        .unwrap()
                    {
                        sched.raise(0);
                    }
                });
            });
            let waiter = scope.spawn(move || {
                sched.side(1, || {
                    let (data, ctrl, arm_base) = if credit { (ds, c, ds) } else { (d, cs, cs) };
                    let mut waiter =
                        unsafe { words(data as *mut u8, ctrl as *mut u8).waiter(live().unwrap()) };
                    if arm_flush {
                        waiter.arm(live).unwrap();
                    } else {
                        unsafe { st(arm_base as *mut u8, arm, 1) };
                    }
                    (unsafe { ld(visible as *const u8, commit) != 0 }) || sched.wait_doorbell(1)
                })
            });
            sched.drive(choose, &mut std::vec::Vec::new(), 3);
            waiter.join().unwrap()
        });
        stalls += usize::from(!ready);
    });
    assert!(runs > 1);
    stalls
}

#[test]
fn both_posted_readbacks_close_publish_vs_arm() {
    for credit in [false, true] {
        assert_eq!(stalled(credit, true, true), 0);
        assert!(stalled(credit, false, true) > 0);
        assert!(stalled(credit, true, false) > 0);
    }
}
