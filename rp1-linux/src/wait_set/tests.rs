use super::*;
use crate::{CoreState, Mapping, Peer, tests::anon};
use rp1_proto::{
    shm,
    wake::{self, LATCHED_WAKE, mailbox},
};
use std::{
    io::{Read, Write},
    os::unix::net::UnixStream,
    time::Duration,
};

struct Rig {
    session: Rc<Session>,
    peer: UnixStream,
    rx: [RingRx<u32>; 2],
    tx: [shm::RingTx<u32>; 2],
}

const OFFSETS: [usize; 2] = [0x100, 0x200];

impl Rig {
    fn new() -> Self {
        let (session, mut peer) = crate::tests::strict_session(LATCHED_WAKE);
        let mut int = [0; 4];
        peer.read_exact(&mut int).unwrap();
        assert_eq!(u32::from_ne_bytes(int), 1);
        let (a, tx_a) = Self::endpoint(&session, OFFSETS[0]);
        let (b, tx_b) = Self::endpoint(&session, OFFSETS[1]);
        Self {
            session,
            peer,
            rx: [a, b],
            tx: [tx_a, tx_b],
        }
    }

    fn endpoint(session: &Rc<Session>, offset: usize) -> (RingRx<u32>, shm::RingTx<u32>) {
        let data = anon(4096);
        let ctrl = unsafe { session.link.ptr().add(offset) };
        let len = shm::ring_body_size(4, 4).unwrap();
        let overflow = shm::Overflow::Reject;
        let inner =
            unsafe { shm::RingRx::attach(data.ptr(), len, ctrl, 128, 4, overflow) }.unwrap();
        let tx = unsafe { shm::RingTx::attach(data.ptr(), len, ctrl, 128, 4, overflow) }.unwrap();
        let ticket =
            crate::strict::ring_waiter(session, unsafe { shm::wake::Words::rx(data.ptr(), ctrl) })
                .unwrap();
        let rx = RingRx {
            inner,
            ticket: Some(ticket),
            irq: None,
            space_doorbell: None,
            credit: None,
            ctrl_offset: offset,
            peer: Peer {
                link: session.link.clone(),
                core: 1,
            },
            spin: Duration::ZERO,
            stats: Default::default(),
            _map: data,
            session: session.clone(),
        };
        (rx, tx)
    }
}

fn control(link: &Mapping, index: usize, offset: usize) -> u32 {
    link.read(OFFSETS[index] + offset)
}

#[test]
fn expires_after_one_linux_block_without_periodic_checks() {
    let mut rig = Rig::new();
    let now = Instant::now();
    let end = now + Duration::from_millis(30);
    assert!(matches!(rig.rx[0].wait(end), Err(Error::Timeout)));
    assert!(Instant::now() >= end);
    assert_eq!(
        rig.rx[0].wait_stats(),
        crate::WaitStats {
            sleeps: 1,
            deadline_checks: 1,
            ..Default::default()
        }
    );
    assert_eq!(control(&rig.session.link, 0, shm::wake::RX_WAIT), 2);
    assert_eq!(rig.session.link.read(mailbox::WAIT), 2);
}

#[test]
fn one_merged_count_scans_every_receiver_and_acks_only_current_observed_tickets() {
    let Rig {
        session,
        mut peer,
        rx: [mut a, mut b],
        tx: [mut tx_a, mut tx_b],
    } = Rig::new();
    let a_map = a._map.clone();
    let b_map = b._map.clone();
    let mut endpoints: [&mut dyn Waitable; 2] = [&mut a, &mut b];
    let mut wait = WaitSet::new(session.clone(), &mut endpoints).unwrap();
    assert_eq!(
        wait.prepare(Instant::now() + Duration::from_secs(2))
            .unwrap(),
        WaitState::Armed
    );
    peer.write_all(&77u32.to_ne_bytes()).unwrap();
    // First return was spurious. The same tickets remain armed.
    assert_eq!(wait.dispatch(libc::POLLIN).unwrap(), WaitState::Armed);
    let mut command = [0; 4];
    peer.read_exact(&mut command).unwrap();
    assert_eq!(u32::from_ne_bytes(command), 1);
    assert_eq!(control(&session.link, 0, shm::wake::RX_WAIT), 1);
    assert_eq!(control(&session.link, 1, shm::wake::RX_WAIT), 1);
    assert_eq!(session.link.read(mailbox::WAIT), 1);

    tx_a.push(&1).unwrap();
    a_map.write(shm::wake::RX_SEQ, 1);
    peer.write_all(&78u32.to_ne_bytes()).unwrap();
    // B commits while masked. A's queued count is the only count the fake
    // kernel exposes, even after the re-enable.
    tx_b.push(&2).unwrap();
    b_map.write(shm::wake::RX_SEQ, 1);
    assert_eq!(wait.dispatch(libc::POLLIN).unwrap(), WaitState::Ready(0));
    peer.read_exact(&mut command).unwrap();
    assert_eq!(u32::from_ne_bytes(command), 1);
    assert_eq!(control(&session.link, 0, shm::wake::RX_ACK), 1);
    assert_eq!(control(&session.link, 1, shm::wake::RX_ACK), 1);
    assert_eq!(session.link.read(mailbox::ACK), 0);
    drop(wait);
    assert_eq!(a.try_recv().unwrap(), Some(1));
    assert_eq!(b.try_recv().unwrap(), Some(2));
    let mut endpoints: [&mut dyn Waitable; 1] = [&mut b];
    let mut wait = WaitSet::new(session.clone(), &mut endpoints).unwrap();
    assert_eq!(
        wait.prepare(Instant::now() + Duration::from_secs(2))
            .unwrap(),
        WaitState::Armed
    );
    assert_eq!(control(&session.link, 1, shm::wake::RX_WAIT), 3);
    tx_b.push(&3).unwrap();
    b_map.write(shm::wake::RX_SEQ, 5);
    peer.write_all(&79u32.to_ne_bytes()).unwrap();
    assert_eq!(wait.dispatch(libc::POLLIN).unwrap(), WaitState::Ready(0));
    assert_eq!(control(&session.link, 1, shm::wake::RX_ACK), 1);
}

#[test]
fn deadline_distinguishes_a_queued_hint_from_lost_delivery_and_terminal() {
    for queued in [false, true] {
        let Rig {
            session,
            mut peer,
            mut rx,
            mut tx,
        } = Rig::new();
        let map = rx[0]._map.clone();
        let mut endpoints: [&mut dyn Waitable; 1] = [&mut rx[0]];
        let mut wait = WaitSet::new(session.clone(), &mut endpoints).unwrap();
        let end = Instant::now() + Duration::from_millis(30);
        assert_eq!(wait.prepare(end).unwrap(), WaitState::Armed);
        tx[0].push(&9).unwrap();
        map.write(shm::wake::RX_SEQ, 1);
        if queued {
            peer.write_all(&2u32.to_ne_bytes()).unwrap();
        }
        std::thread::sleep(end.saturating_duration_since(Instant::now()));
        // ppoll may report timeout while the IRQ thread commits a count.
        let result = wait.dispatch(0);
        if queued {
            assert!(matches!(result, Ok(WaitState::Ready(0))));
        } else {
            assert!(matches!(result, Err(Error::NotifyLost)));
        }
        drop(wait);
        assert_eq!(rx[0].try_recv().unwrap(), Some(9));
    }

    let Rig {
        session,
        peer: _peer,
        mut rx,
        mut tx,
        ..
    } = Rig::new();
    let mut endpoints: [&mut dyn Waitable; 1] = [&mut rx[0]];
    let mut wait = WaitSet::new(session.clone(), &mut endpoints).unwrap();
    let end = Instant::now() + Duration::from_millis(30);
    assert_eq!(wait.prepare(end).unwrap(), WaitState::Armed);
    tx[0].push(&11).unwrap();
    let block = rp1_proto::status::BLOCKS[1];
    session.link.write(block + rp1_proto::status::REASON, 5);
    session
        .link
        .write(block + rp1_proto::status::STATE, CoreState::Panicked.word());
    std::thread::sleep(end.saturating_duration_since(Instant::now()));
    assert!(matches!(
        wait.dispatch(0),
        Err(Error::PeerDown(PeerDown::Terminal {
            core: 1,
            state: CoreState::Panicked,
            reason: 5
        }))
    ));
    assert_eq!(session.link.read(mailbox::ACK), 0);
}

#[test]
fn drop_disarms_only_owned_tickets_and_a_restart_blocks_stale_cleanup() {
    let Rig {
        session,
        rx: [mut a, mut b],
        peer: _peer,
        ..
    } = Rig::new();
    let mut ea: [&mut dyn Waitable; 1] = [&mut a];
    let mut eb: [&mut dyn Waitable; 1] = [&mut b];
    let end = Instant::now() + Duration::from_secs(2);
    let mut wait = WaitSet::new(session.clone(), &mut ea).unwrap();
    assert_eq!(wait.prepare(end).unwrap(), WaitState::Armed);
    assert!(matches!(
        WaitSet::new(session.clone(), &mut eb).unwrap().prepare(end),
        Err(Error::Busy)
    ));
    let b_stale_arm = 17;
    session
        .link
        .write(OFFSETS[1] + shm::wake::RX_WAIT, b_stale_arm);
    drop(wait);
    assert_eq!(control(&session.link, 0, shm::wake::RX_WAIT), 2);
    assert_eq!(control(&session.link, 1, shm::wake::RX_WAIT), b_stale_arm);

    let mut wait = WaitSet::new(session.clone(), &mut ea).unwrap();
    assert_eq!(wait.prepare(end).unwrap(), WaitState::Armed);
    session._map.write(12, 4);
    session.link.write(OFFSETS[0] + shm::wake::RX_WAIT, 19);
    session.link.write(mailbox::WAIT, 21);
    assert!(matches!(
        wait.dispatch(0),
        Err(Error::PeerDown(PeerDown::Restarted))
    ));
    drop(wait);
    assert_eq!(control(&session.link, 0, shm::wake::RX_WAIT), 19);
    assert_eq!(control(&session.link, 0, shm::wake::RX_ACK), 0);
    assert_eq!(session.link.read(mailbox::WAIT), 21);
}

#[test]
fn preexisting_work_and_an_explicit_poll_endpoint() {
    let mut rig = Rig::new();
    rig.tx[0].push(&41).unwrap();
    assert!(rig.rx[0].wait(Instant::now()).is_ok());
    assert_eq!(rig.rx[0].try_recv().unwrap(), Some(41));
    assert_eq!(rig.rx[0].wait_stats(), Default::default());
    assert_eq!(control(&rig.session.link, 0, shm::wake::RX_WAIT), 0);

    rig.rx[1].ticket = None;
    let mut endpoints: [&mut dyn Waitable; 2] = {
        let [a, b] = &mut rig.rx;
        [a, b]
    };
    assert!(matches!(
        WaitSet::new(rig.session.clone(), &mut endpoints),
        Err(Error::Unsupported)
    ));
    rig.tx[1].push(&42).unwrap();
    assert!(rig.rx[1].wait(Instant::now()).is_ok());
    assert_eq!(rig.rx[1].try_recv().unwrap(), Some(42));
}

#[test]
fn restart_scope_is_bound_to_both_boot_and_ddr_generation() {
    let mut rig = Rig::new();
    assert_eq!(
        rig.session.generation().unwrap(),
        wake::Session::new(2, 2).unwrap().generation()
    );
    rig.session.link.write(mailbox::BOOT_EPOCH, 4);
    assert!(matches!(
        rig.rx[0].wait(Instant::now()),
        Err(Error::PeerDown(PeerDown::Restarted))
    ));
    assert_eq!(control(&rig.session.link, 0, shm::wake::RX_WAIT), 0);
}

type PollHook = std::cell::RefCell<Option<Box<dyn FnOnce()>>>;

thread_local! {
    static BEFORE_POLL: PollHook = const { std::cell::RefCell::new(None) };
}

pub(super) fn run_before_poll() {
    if let Some(hook) = BEFORE_POLL.with(|hook| hook.take()) {
        hook();
    }
}

#[test]
fn commit_between_first_scan_and_arm_is_seen_before_blocking() {
    let Rig {
        session,
        mut rx,
        tx: [mut tx, _],
        peer: _peer,
    } = Rig::new();
    let mut endpoints: [&mut dyn Waitable; 1] = [&mut rx[0]];
    let mut wait = WaitSet::new(session.clone(), &mut endpoints).unwrap();
    wait.before_arm = Some(Box::new(move || {
        assert_eq!(control(&session.link, 0, shm::wake::RX_WAIT), 0);
        tx.push(&47).unwrap();
    }));
    assert_eq!(
        wait.prepare(Instant::now() + Duration::from_secs(1))
            .unwrap(),
        WaitState::Ready(0)
    );
    drop(wait);
    assert_eq!(rx[0].try_recv().unwrap(), Some(47));
}

#[test]
fn a_ready_receiver_does_not_ack_an_unready_receiver() {
    let Rig {
        session,
        mut peer,
        rx: [mut a, mut b],
        tx: [mut tx, _],
    } = Rig::new();
    let a_data = a._map.clone();
    let b_data = b._map.clone();
    let mut endpoints: [&mut dyn Waitable; 2] = [&mut a, &mut b];
    let mut wait = WaitSet::new(session.clone(), &mut endpoints).unwrap();
    assert_eq!(
        wait.prepare(Instant::now() + Duration::from_secs(1))
            .unwrap(),
        WaitState::Armed
    );
    tx.push(&47).unwrap();
    a_data.write(shm::wake::RX_SEQ, 1);
    b_data.write(shm::wake::RX_SEQ, 1);
    peer.write_all(&2u32.to_ne_bytes()).unwrap();
    assert_eq!(wait.dispatch(libc::POLLIN).unwrap(), WaitState::Ready(0));
    assert_eq!(control(&session.link, 0, shm::wake::RX_ACK), 1);
    assert_eq!(control(&session.link, 1, shm::wake::RX_ACK), 0);
}

#[test]
fn unrelated_pipe_rearms_neither_ticket_then_link_and_deadline_dispatch() {
    let Rig {
        session,
        mut peer,
        mut rx,
        mut tx,
    } = Rig::new();
    let (mut camera, mut frames) = std::io::pipe().unwrap();
    let mut endpoints: [&mut dyn Waitable; 1] = [&mut rx[0]];
    let mut wait = WaitSet::new(session.clone(), &mut endpoints).unwrap();
    let end = Instant::now() + Duration::from_secs(1);
    assert_eq!(wait.prepare(end).unwrap(), WaitState::Armed);
    let mut fds = [wait.as_fd().as_raw_fd(), camera.as_raw_fd()].map(|fd| libc::pollfd {
        fd,
        events: libc::POLLIN,
        revents: 0,
    });
    frames.write_all(&[1]).unwrap();
    poll_until(&mut fds, Some(end)).unwrap();
    assert_eq!((fds[0].revents, fds[1].revents), (0, libc::POLLIN));
    assert_eq!(wait.dispatch(fds[0].revents).unwrap(), WaitState::Armed);
    assert_eq!(control(&session.link, 0, shm::wake::RX_WAIT), 1);
    camera.read_exact(&mut [0; 1]).unwrap();

    tx[0].push(&47).unwrap();
    peer.write_all(&2u32.to_ne_bytes()).unwrap();
    poll_until(&mut fds, Some(end)).unwrap();
    assert_eq!(wait.dispatch(fds[0].revents).unwrap(), WaitState::Ready(0));
    drop(wait);
    assert_eq!(rx[0].try_recv().unwrap(), Some(47));

    let mut endpoints: [&mut dyn Waitable; 1] = [&mut rx[0]];
    let mut wait = WaitSet::new(session, &mut endpoints).unwrap();
    let end = Instant::now() + Duration::from_millis(30);
    assert_eq!(wait.prepare(end).unwrap(), WaitState::Armed);
    poll_until(&mut fds, Some(end)).unwrap();
    assert_eq!((fds[0].revents, fds[1].revents), (0, 0));
    assert!(matches!(wait.dispatch(0), Err(Error::Timeout)));
}

#[test]
fn recv_and_rpc_return_data_at_the_deadline_without_a_count() {
    let Rig {
        session,
        peer: _peer,
        rx: [mut a, b],
        tx: [mut tx_a, mut tx_b],
    } = Rig::new();
    BEFORE_POLL.with(|hook| hook.replace(Some(Box::new(move || tx_a.push(&47).unwrap()))));
    assert_eq!(a.recv(Some(Duration::from_millis(30))).unwrap(), Some(47));
    assert_eq!(a.wait_stats().late, 1);

    let data = anon(4096);
    let ctrl = anon(4096);
    let len = shm::ring_body_size(4, 4).unwrap();
    let inner =
        unsafe { shm::RingTx::attach(data.ptr(), len, ctrl.ptr(), 128, 4, shm::Overflow::Reject) }
            .unwrap();
    let tx = crate::RingTx {
        inner,
        doorbell: None,
        publisher: None,
        credit: None,
        poll: false,
        core: 1,
        stats: Default::default(),
        _map: data,
        session,
    };
    let mut client = crate::Client::new(tx, b, 4, false).unwrap();
    BEFORE_POLL.with(|hook| hook.replace(Some(Box::new(move || tx_b.push(&48).unwrap()))));
    assert_eq!(client.call(&7, Duration::from_millis(30)).unwrap(), 48);
    assert_eq!(client.wait_stats().late, 1);
}

#[test]
fn solicited_reply_retires_its_ticket_before_the_mailbox_is_armed() {
    let Rig {
        session,
        mut rx,
        tx: [mut tx, _],
        peer: _peer,
    } = Rig::new();
    let data = rx[0]._map.clone();
    let mut endpoints: [&mut dyn Waitable; 1] = [&mut rx[0]];
    let mut wait = WaitSet::new(session.clone(), &mut endpoints).unwrap();
    wait.rpc = true;
    wait.before_arm = Some(Box::new(move || {
        tx.push(&23).unwrap();
    }));
    assert_eq!(
        wait.prepare(Instant::now() + Duration::from_secs(1))
            .unwrap(),
        WaitState::Ready(0)
    );
    assert_eq!(session.link.read(mailbox::WAIT), 0);
    assert_eq!(control(&session.link, 0, shm::wake::RX_WAIT), 2);
    assert_eq!(control(&session.link, 0, shm::wake::RX_ACK), 0);
    assert_eq!(data.read(shm::wake::RX_SEQ), 0);
}

#[test]
fn writable_credit_wait_distinguishes_delivery_from_deadline_and_reopens() {
    use shm::wake::{CREDIT_ACK, CREDIT_SEQ, CREDIT_WAIT, Words};

    let rig = Rig::new();
    let data = rig.session.link.clone();
    let offset = 0x300;
    let base = unsafe { data.ptr().add(offset) };
    let ctrl = anon(4096);
    let ctrl_ptr = ctrl.ptr();
    let len = shm::ring_body_size(4, 4).unwrap();
    let overflow = shm::Overflow::Reject;
    let words = unsafe { Words::credit(base, ctrl_ptr) };
    let make_tx = || crate::RingTx {
        inner: unsafe { shm::RingTx::attach(base, len, ctrl_ptr, 128, 4, overflow) }.unwrap(),
        doorbell: None,
        publisher: None,
        credit: Some(crate::strict::ring_waiter(&rig.session, words).unwrap()),
        poll: false,
        core: 1,
        stats: Default::default(),
        _map: ctrl.clone(),
        session: rig.session.clone(),
    };
    let mut reader =
        unsafe { shm::RingRx::attach_ticketed(base, len, ctrl_ptr, 128, 4, overflow) }.unwrap();
    let publisher = crate::strict::ring_publisher(&rig.session, words).unwrap();
    let mut tx = make_tx();
    assert_eq!(tx.send_slice(&[4, 5, 6, 7]).unwrap(), 4);
    assert!(matches!(
        tx.send(&8),
        Err(Error::Proto(rp1_proto::Error::Full))
    ));

    assert!(matches!(
        tx.wait_writable(Instant::now() + Duration::from_millis(10)),
        Err(Error::Timeout)
    ));
    let canceled = data.read(offset + CREDIT_WAIT);
    assert_ne!(canceled, 0);
    assert_eq!(canceled & 1, 0);
    assert_eq!(data.read(offset + CREDIT_ACK), 0);
    assert_eq!(tx.wait_stats().sleeps, 1);
    drop(tx);
    let mut tx = make_tx();

    let credit_release =
        |mut reader: shm::RingRx<u32>, mut publisher, value, mut signal: UnixStream, deliver| {
            let data = data.clone();
            let ctrl = ctrl.clone();
            Box::new(move || {
                let arm = data.read(offset + CREDIT_WAIT);
                assert_ne!(arm, canceled - 1);
                assert_eq!(arm & 1, 1);
                assert_eq!(reader.pop().unwrap(), Some(value));
                assert!(reader.publish_credit(&mut publisher).unwrap());
                assert_eq!(ctrl.read(CREDIT_SEQ), arm);
                if deliver {
                    signal.write_all(&1u32.to_ne_bytes()).unwrap();
                }
            })
        };
    BEFORE_POLL.with(|hook| {
        hook.replace(Some(credit_release(
            reader,
            publisher,
            4,
            rig.peer.try_clone().unwrap(),
            true,
        )));
    });
    tx.send_until(&8, Instant::now() + Duration::from_secs(1))
        .unwrap();
    assert!(tx.wait_stats().woken > 0);
    assert_eq!(
        data.read(offset + CREDIT_ACK),
        data.read(offset + CREDIT_WAIT) - 1
    );
    assert_eq!(tx.free(), 0);

    reader =
        unsafe { shm::RingRx::attach_ticketed(base, len, ctrl_ptr, 128, 4, overflow) }.unwrap();
    let publisher = crate::strict::ring_publisher(&rig.session, words).unwrap();
    BEFORE_POLL.with(|hook| {
        hook.replace(Some(credit_release(
            reader,
            publisher,
            5,
            rig.peer.try_clone().unwrap(),
            false,
        )));
    });
    let error = tx.send_until(&9, Instant::now() + Duration::from_millis(10));
    assert!(matches!(error, Err(Error::NotifyLost)), "{error:?}");
    assert_eq!(tx.free(), 1);
    let mut reader =
        unsafe { shm::RingRx::attach_ticketed(base, len, ctrl_ptr, 128, 4, overflow) }.unwrap();
    let mut remaining = [0; 4];
    assert_eq!(reader.pop_into(&mut remaining).unwrap(), 3);
    assert_eq!(&remaining[..3], &[6, 7, 8]);
}
