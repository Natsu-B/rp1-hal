extern crate std;

use super::{
    mem,
    model::{Path, Sched, explore},
};
use crate::{
    Error,
    note::{ShmKind, ShmRegion, Writer},
    shm::{
        CONTROL_SIZE, HEAD, Header, HeaderInit, Overflow, RingRx, RingTx, TAIL,
        access::{ld, st},
        wake::*,
    },
    wake::{BootEpoch, LATCHED_WAKE, Request, Session, SessionError, mailbox},
};

#[test]
fn session_header_binds_note_epoch_generation_and_caps() {
    let mut hdr = mem::<64>();
    let mut link = mem::<{ mailbox::END }>();
    let base = hdr.0.as_mut_ptr();
    let link = link.0.as_mut_ptr();
    let epoch = BootEpoch::new(8).unwrap();
    let hash = [5; 32];
    let session = Session::new(epoch.word(), 2).unwrap();
    let init = unsafe { HeaderInit::begin_v2(base, epoch) }.unwrap();
    assert_eq!(unsafe { ld(base, 12) }, 1);
    assert_eq!(
        unsafe { Header::open_v2(base, &hash, session, LATCHED_WAKE) }.err(),
        Some(Error::NotReady)
    );
    assert_eq!(
        unsafe { init.publish_v2(link, &[], &hash, 7, LATCHED_WAKE) },
        Ok(session)
    );
    let header = unsafe { Header::open_v2(base, &hash, session, LATCHED_WAKE) }.unwrap();
    assert_eq!(header.regions(), 0);
    assert_eq!(header.session(), Ok(session));
    assert_eq!(
        unsafe { Header::open(base, &hash) }.err(),
        Some(Error::NotReady)
    );
    for (digest, expected, caps) in [
        ([6; 32], session, LATCHED_WAKE),
        (
            hash,
            Session::new(epoch.word() + 2, 2).unwrap(),
            LATCHED_WAKE,
        ),
        (hash, Session::new(epoch.word(), 4).unwrap(), LATCHED_WAKE),
        (hash, session, 0),
    ] {
        assert_eq!(
            unsafe { Header::open_v2(base, &digest, expected, caps) }.err(),
            Some(Error::Mismatch)
        );
    }
    let _replacement = unsafe { HeaderInit::begin_v2(base, epoch) }.unwrap();
    assert_eq!(unsafe { ld(base, 12) }, 3);
    assert_eq!(header.session(), Err(SessionError::NotReady));

    let epoch = BootEpoch::new(10).unwrap();
    let replacement = unsafe { HeaderInit::begin_v2(base, epoch) }.unwrap();
    unsafe { replacement.publish_v2(link, &[], &hash, 0, LATCHED_WAKE) }.unwrap();
    assert_eq!(unsafe { ld(base, 12) }, 2);
    assert_eq!(header.session(), Err(SessionError::Restarted));
}

#[test]
fn initialization_clears_retained_ring_tickets_and_scopes_generation_exhaustion() {
    let ring = ShmRegion {
        kind: ShmKind::RingToLinux,
        core: 1,
        notify_irq: true,
        overwrite: false,
        ddr_offset: 0x10000,
        ddr_len: 0x10000,
        sram_offset: 0x100,
        sram_len: CONTROL_SIZE as u16,
        elem_size: 4,
        depth: 1,
        type_id: 0,
        name: "up",
        type_name: "u32",
    };
    let regions = [
        ring,
        ShmRegion {
            kind: ShmKind::RingToRp1,
            overwrite: true,
            ddr_offset: 0x20000,
            sram_offset: 0x200,
            sram_len: 144,
            name: "down",
            ..ring
        },
        ShmRegion {
            kind: ShmKind::Block {
                writer: Writer::Linux,
            },
            ddr_offset: 0x30000,
            depth: 0,
            notify_irq: false,
            sram_offset: 0,
            sram_len: 0,
            name: "block",
            ..ring
        },
    ];
    let mut ddr = mem::<0x40000>();
    let mut sram = mem::<0x300>();
    let base = ddr.0.as_mut_ptr();
    let link = sram.0.as_mut_ptr();
    let up = unsafe { (base.add(0x10000), link.add(0x100)) };
    let down = unsafe { (link.add(0x200), base.add(0x20000)) };
    let block = unsafe { base.add(0x30000) };
    let rx = unsafe { Words::rx(up.0, up.1) };
    let credit = unsafe { Words::credit(up.0, up.1) };
    let down_rx = unsafe { Words::rx(down.0, down.1) };
    let epoch = BootEpoch::new(8).unwrap();
    let hash = [0; 32];

    for previous in [u32::MAX - 1, u32::MAX] {
        unsafe {
            st(base, 12, previous);
            st(base, 56, epoch.word() - 2);
            st(block, 0, 42);
            for (data, ctrl) in [up, down] {
                st(ctrl, RX_WAIT, u32::MAX - 1);
                st(data, RX_SEQ, 1); // Stale seq would suppress the first pulse.
                st(ctrl, RX_ACK, 1); // Stale ACK would hide the first pending ticket.
                st(data, CREDIT_WAIT, u32::MAX - 1);
                st(ctrl, CREDIT_SEQ, 1);
                st(data, CREDIT_ACK, 1);
            }
        }
        let old_epoch = BootEpoch::new(epoch.word() - 2).unwrap();
        assert_eq!(
            unsafe { HeaderInit::begin_v2(base, old_epoch) }.err(),
            Some(SessionError::Exhausted)
        );
        let init = unsafe { HeaderInit::begin_v2(base, epoch) }.unwrap();
        assert_eq!(unsafe { ld(base, 12) }, 1);
        let session = unsafe { init.publish_v2(link, &regions, &hash, 0, LATCHED_WAKE) }.unwrap();
        assert_eq!(session, Session::new(epoch.word(), 2).unwrap());
        let header = unsafe { Header::open_v2(base, &hash, session, LATCHED_WAKE) }.unwrap();
        assert_eq!(header.regions(), regions.len() as u32);
        assert_eq!(unsafe { ld(block, 0) }, 42);
        for (data, ctrl) in [up, down] {
            for (address, offset) in [
                (ctrl, RX_WAIT),
                (data, RX_SEQ),
                (ctrl, RX_ACK),
                (data, CREDIT_WAIT),
                (ctrl, CREDIT_SEQ),
                (data, CREDIT_ACK),
            ] {
                assert_eq!(unsafe { ld(address, offset) }, 0);
            }
        }
        for words in [rx, credit, down_rx] {
            let mut waiter = unsafe { words.waiter(session, || header.session()) }.unwrap();
            let mut publisher = unsafe { words.publisher(session, || header.session()) }.unwrap();
            waiter.arm().unwrap();
            assert!(!waiter.pending().unwrap());
            assert!(publisher.publish(|| {}).unwrap());
            assert!(waiter.pending().unwrap());
            assert!(waiter.acknowledge().unwrap());
            assert!(!waiter.pending().unwrap());
        }

        // A same-boot restart clears the departing owner's state too.
        let init = unsafe { HeaderInit::begin_v2(base, epoch) }.unwrap();
        let replacement =
            unsafe { init.publish_v2(link, &regions, &hash, 0, LATCHED_WAKE) }.unwrap();
        assert_eq!(replacement.generation(), 4);
        assert_eq!(header.session(), Err(SessionError::Restarted));
        assert_eq!(unsafe { ld(up.1, RX_WAIT) }, 0);
    }
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
    for credit in [false, true] {
        let mut data = mem::<CONTROL_SIZE>();
        let mut ctrl = mem::<CONTROL_SIZE>();
        let base = ctrl.0.as_mut_ptr();
        let data_ptr = data.0.as_mut_ptr();
        let (words, arm, offset) = if credit {
            (
                unsafe { Words::credit(data_ptr, base) },
                data_ptr,
                CREDIT_WAIT,
            )
        } else {
            (unsafe { Words::rx(data_ptr, base) }, base, RX_WAIT)
        };
        let session = Session::new(2, 2).unwrap();
        let current = core::cell::Cell::new(Ok(session));
        let live = || current.get();
        {
            let mut waiter = unsafe { words.waiter(session, live) }.unwrap();
            let mut publisher = unsafe { words.publisher(session, live) }.unwrap();
            waiter.arm().unwrap();
            assert!(publisher.publish(|| {}).unwrap());
            assert!(waiter.acknowledge().unwrap());
            assert!(!publisher.publish(|| {}).unwrap());
            // The owner exits before releasing the arm.
        }
        let mut waiter = unsafe { words.waiter(session, live) }.unwrap();
        let mut publisher = unsafe { words.publisher(session, live) }.unwrap();
        assert!(!waiter.pending().unwrap());
        assert!(!waiter.acknowledge().unwrap());
        assert!(!waiter.cancel().unwrap());
        waiter.arm().unwrap();
        assert_eq!(unsafe { ld(arm, offset) }, 3);
        assert!(!waiter.pending().unwrap());
        assert!(publisher.publish(|| {}).unwrap());
        assert!(waiter.pending().unwrap());
        assert!(waiter.cancel().unwrap());
        assert!(!waiter.pending().unwrap());
        waiter.arm().unwrap();

        let old = (data.0, ctrl.0);
        for replacement in [
            Session::new(4, 2), // boot changed while SHM generation is the same
            Session::new(2, 4),
            Session::new(2, 3), // initialization in progress
        ] {
            current.set(replacement);
            assert_eq!(
                publisher.publish(|| panic!("stale readback")),
                Err(SessionError::Restarted)
            );
            assert_eq!(
                publisher.needs_recovery_kick(),
                Err(SessionError::Restarted)
            );
            assert_eq!(waiter.arm(), Err(SessionError::Restarted));
            assert_eq!(waiter.pending(), Err(SessionError::Restarted));
            assert_eq!(waiter.acknowledge(), Err(SessionError::Restarted));
            assert_eq!(
                waiter.acknowledge_and_cancel(),
                Err(SessionError::Restarted)
            );
            assert_eq!(waiter.cancel(), Err(SessionError::Restarted));
        }
        assert_eq!((data.0, ctrl.0), old);
    }
}

#[test]
fn wrap_skips_retained_seq_and_ack_then_reuses_tickets_without_reconnecting() {
    let mut data = mem::<CONTROL_SIZE>();
    let mut ctrl = mem::<CONTROL_SIZE>();
    let data = data.0.as_mut_ptr();
    let ctrl = ctrl.0.as_mut_ptr();
    let session = Session::new(2, 2).unwrap();
    let live = || Ok(session);
    let words = unsafe { Words::rx(data, ctrl) };
    unsafe {
        st(ctrl, RX_WAIT, LAST_TICKET - 1);
        st(data, RX_SEQ, 1);
        st(ctrl, RX_ACK, 3);
    }
    let mut waiter = unsafe { words.waiter(session, live) }.unwrap();
    let mut publisher = unsafe { words.publisher(session, live) }.unwrap();
    waiter.arm().unwrap();
    assert_eq!(unsafe { ld(ctrl, RX_WAIT) }, LAST_TICKET);
    assert!(waiter.cancel().unwrap());
    assert_eq!(unsafe { ld(ctrl, RX_WAIT) }, u32::MAX - 1);
    waiter.arm().unwrap();
    assert_eq!(unsafe { ld(ctrl, RX_WAIT) }, 5);
    assert!(!waiter.pending().unwrap());
    assert!(publisher.publish(|| {}).unwrap());
    assert!(waiter.pending().unwrap());
    assert!(waiter.acknowledge().unwrap());
    assert!(waiter.cancel().unwrap());
    waiter.arm().unwrap();
    assert_eq!(unsafe { ld(ctrl, RX_WAIT) }, 7);
}

#[test]
fn wrap_model_bounds_aba_from_a_stalled_publisher() {
    // Use the production ticket selection on a small ring. While the publisher is
    // stalled seq cannot change; ACK can remain at its old value or advance to seq.
    const LAST: u32 = 13;
    let tickets = (1..=LAST).step_by(2);
    let earliest_reuse = |skip: bool| {
        let mut earliest = u32::MAX;
        for sampled in tickets.clone() {
            for seq in [0].into_iter().chain(tickets.clone()) {
                for acked in [0].into_iter().chain(tickets.clone()) {
                    for ack_at in 0..=7 {
                        let mut arm = sampled;
                        for step in 1..=7 {
                            let ack = if step >= ack_at { seq } else { acked };
                            let (seq, ack) = if skip { (seq, ack) } else { (0, 0) };
                            arm = next_ticket(arm, seq, ack, LAST);
                            if arm == sampled {
                                // A delayed store of the sampled ticket aliases the new arm.
                                earliest = earliest.min(step);
                                break;
                            }
                        }
                    }
                }
            }
        }
        earliest
    };
    assert_eq!(earliest_reuse(true), 5); // Seven odd tickets minus at most two skips.
    assert_eq!(earliest_reuse(false), 7);
    assert_eq!(next_ticket(LAST, 1, 3, LAST), 5);
    assert_eq!(next_ticket(LAST, 0, 0, LAST), 1); // Broken variant aliases a retained seq.
}

#[test]
fn retiring_arm_readback_flushes_its_ack_through_the_same_window() {
    let mut data = mem::<CONTROL_SIZE>();
    let mut real = mem::<CONTROL_SIZE>();
    let mut shadow = mem::<CONTROL_SIZE>();
    let path = Path::new(0, shadow.0.as_mut_ptr(), real.0.as_mut_ptr(), CONTROL_SIZE);
    let sched = Sched::new(1, std::vec![path]);
    std::thread::scope(|scope| {
        scope.spawn(|| {
            sched.side(0, || {
                let data = data.0.as_mut_ptr();
                let ctrl = shadow.0.as_mut_ptr();
                let words = unsafe { Words::rx(data, ctrl) };
                let live = || Session::new(2, 2);
                let mut waiter = unsafe { words.waiter(live().unwrap(), live) }.unwrap();
                waiter.arm().unwrap();
                unsafe { st(data, RX_SEQ, 1) };
                waiter.acknowledge_and_cancel().unwrap();
            })
        });
        sched.drive(|_| 0, &mut std::vec::Vec::new(), 0);
    });
    assert_eq!(unsafe { ld(real.0.as_ptr(), RX_ACK) }, 1);
    assert_eq!(unsafe { ld(real.0.as_ptr(), RX_WAIT) }, 2);
}

#[test]
fn takeover_recovers_only_the_current_unacknowledged_publication() {
    let mut page = mem::<{ mailbox::END }>();
    let base = page.0.as_mut_ptr();
    let boot = BootEpoch::new(4).unwrap();
    let live = || Ok(boot);
    let words = unsafe { Words::link(base) };
    let mut waiter = unsafe { words.waiter(boot, live) }.unwrap();
    waiter.arm().unwrap();
    {
        let mut publisher = unsafe { words.publisher(boot, live) }.unwrap();
        assert!(!publisher.needs_recovery_kick().unwrap());
        assert!(publisher.publish(|| {}).unwrap());
        // The old publisher exits after the seq readback and before the pulse.
    }
    let publisher = unsafe { words.publisher(boot, live) }.unwrap();
    assert!(publisher.needs_recovery_kick().unwrap());
    assert!(waiter.acknowledge().unwrap());
    assert!(!publisher.needs_recovery_kick().unwrap());
    waiter.cancel().unwrap();
    assert!(!publisher.needs_recovery_kick().unwrap());
    waiter.arm().unwrap();
    assert!(!publisher.needs_recovery_kick().unwrap());
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
        let live = || Ok(boot);
        let mut waiter = unsafe { words.waiter(boot, live) }.unwrap();
        let mut publisher = unsafe { words.publisher(boot, live) }.unwrap();
        waiter.arm().unwrap();
        assert!(publisher.publish(|| {}).unwrap());
        if !drain_before_reuse {
            old_write.land(1);
        }
        assert_eq!(publisher.needs_recovery_kick().unwrap(), drain_before_reuse);
    }
}

// Each side's stores go through a different posted window. A writer that observes arm
// raises a held-level signal, otherwise the waiter must see the commit before sleeping.
fn stalled(credit: bool, arm_flush: bool, publish_flush: bool) -> usize {
    let mut stalls = 0;
    let runs = explore(|choose| {
        const DATA: usize = CONTROL_SIZE + 16;
        let mut data = mem::<DATA>();
        let mut data_shadow = mem::<DATA>();
        let mut ctrl = mem::<CONTROL_SIZE>();
        let mut ctrl_shadow = mem::<CONTROL_SIZE>();
        let d = data.0.as_mut_ptr() as usize;
        let ds = data_shadow.0.as_mut_ptr() as usize;
        let c = ctrl.0.as_mut_ptr() as usize;
        let cs = ctrl_shadow.0.as_mut_ptr() as usize;
        let mut writer = credit.then(|| unsafe {
            RingTx::attach(
                ds as *mut u8,
                DATA,
                c as *const u8,
                CONTROL_SIZE,
                1,
                Overflow::Reject,
            )
            .unwrap()
        });
        if let Some(writer) = &mut writer {
            writer.push(&42u32).unwrap();
            data.0 = data_shadow.0;
        }
        let paths = [
            Path::new(usize::from(credit), ds as *const u8, d as *const u8, DATA),
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
                        words(data as *mut u8, ctrl as *mut u8).publisher(live().unwrap(), live)
                    }
                    .unwrap();
                    let notify = if credit {
                        let mut rx = unsafe {
                            RingRx::<u32>::attach_ticketed(
                                data as *const u8,
                                DATA,
                                ctrl as *mut u8,
                                CONTROL_SIZE,
                                1,
                                Overflow::Reject,
                            )
                            .unwrap()
                        };
                        assert_eq!(rx.pop().unwrap(), Some(42));
                        if publish_flush {
                            rx.publish_credit(&mut publisher).unwrap()
                        } else {
                            publisher.publish(|| {}).unwrap()
                        }
                    } else {
                        unsafe { st(posted as *mut u8, commit, 1) };
                        publisher
                            .publish(|| {
                                if publish_flush {
                                    let _ = unsafe { ld(posted as *const u8, commit) };
                                }
                            })
                            .unwrap()
                    };
                    if notify {
                        sched.raise(0);
                    }
                });
            });
            let waiter = scope.spawn(move || {
                sched.side(1, || {
                    let (data, ctrl, arm_base) = if credit { (ds, c, ds) } else { (d, cs, cs) };
                    let mut waiter = unsafe {
                        words(data as *mut u8, ctrl as *mut u8).waiter(live().unwrap(), live)
                    }
                    .unwrap();
                    if arm_flush {
                        waiter.arm().unwrap();
                    } else {
                        unsafe { st(arm_base as *mut u8, arm, 1) };
                    }
                    let ready = || {
                        writer.as_ref().map_or_else(
                            || unsafe { ld(visible as *const u8, commit) != 0 },
                            |writer| writer.free() == 1,
                        )
                    };
                    ready() || (sched.wait_doorbell(1) && ready())
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

#[test]
fn credit_seq_readback_makes_the_ticket_visible_before_notification() {
    let mut data = mem::<CONTROL_SIZE>();
    let mut ctrl = mem::<CONTROL_SIZE>();
    let mut posted = mem::<CONTROL_SIZE>();
    let data = data.0.as_mut_ptr() as usize;
    let real = ctrl.0.as_mut_ptr() as usize;
    let shadow = posted.0.as_mut_ptr() as usize;
    let path = Path::new(0, shadow as *const u8, real as *const u8, CONTROL_SIZE);
    let sched = Sched::new(1, std::vec![path]);
    let scope = Session::new(2, 4).unwrap();
    let words = unsafe { Words::credit(data as *mut u8, real as *mut u8) };
    let mut waiter = unsafe { words.waiter(scope, || Ok(scope)) }.unwrap();
    waiter.arm().unwrap();

    std::thread::scope(|thread| {
        thread.spawn(|| {
            sched.side(0, || {
                let words = unsafe { Words::credit(data as *mut u8, shadow as *mut u8) };
                let mut publisher = unsafe { words.publisher(scope, || Ok(scope)) }.unwrap();
                assert!(publisher.publish(|| {}).unwrap());
            });
        });
        sched.drive(|_| 0, &mut std::vec::Vec::new(), 0);
    });
    assert!(waiter.acknowledge_and_cancel().is_ok());
    assert_eq!(unsafe { ld(data as *const u8, CREDIT_ACK) }, 1);
}
