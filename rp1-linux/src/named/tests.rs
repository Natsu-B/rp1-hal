use super::*;
use crate::{
    CoreState, LINK_PAGE, open_exclusive, shm,
    tests::{anon, anon_with_file, firmware, irq_pair, named_note, new_session},
};
use std::{fs, thread};

const RINGS: &[(&str, u8, bool)] = &[("ping-dn", 2, false), ("ping-up", 2, false)];

#[test]
fn pair_names_and_kinds_are_checked_before_uio() {
    let bytes = named_note([None; 2], RINGS, &["ping"], 1);
    let note = Note::parse(&bytes).unwrap();
    assert!(pair::<u32, u32>(note, "ping", true).is_ok());
    for (name, rpc) in [
        ("ping", false),
        ("ping-dn", true),
        ("other", true),
        ("other", false),
    ] {
        assert!(matches!(
            pair::<u32, u32>(note, name, rpc),
            Err(Error::Unknown)
        ));
    }
    assert!(matches!(
        pair::<u32, u64>(note, "ping", true),
        Err(Error::Proto(proto::Error::Mismatch))
    ));
}

#[test]
fn channel_owns_both_maps_and_releases_the_first_on_failure() {
    let dir = std::env::temp_dir().join(format!("rp1-linux-pair-{}", std::process::id()));
    fs::create_dir(&dir).unwrap();
    for name in ["ping-up", "ping-dn"] {
        fs::write(dir.join(name), []).unwrap();
    }
    let fw = firmware(named_note([None; 2], RINGS, &[], 1), Some(new_session()));
    let mut opened = Vec::new();
    let mut map = |r: &ShmRegion<'_>| {
        opened.push(r.name.to_owned());
        Ok(anon_with_file(
            r.ddr_len as usize,
            open_exclusive(&dir.join(r.name))?,
        ))
    };
    let held = open_exclusive(&dir.join("ping-dn")).unwrap();
    assert!(matches!(
        fw.open_pair::<u32, u32>("ping", false, &mut map),
        Err(Error::Busy)
    ));
    drop(open_exclusive(&dir.join("ping-up")).unwrap());
    drop(held);
    let (channel, _, _) = fw.open_pair::<u32, u32>("ping", false, &mut map).unwrap();
    assert!(matches!(
        fw.open_pair::<u32, u32>("ping", false, &mut map),
        Err(Error::Busy)
    ));
    assert_eq!(
        opened,
        ["ping-up", "ping-dn", "ping-up", "ping-dn", "ping-up"]
    );
    drop(fw);
    let (mut tx, mut rx) = channel.split();
    tx.send(&1).unwrap();
    assert_eq!(rx.try_recv().unwrap(), None);
    drop((tx, rx));
    drop(open_exclusive(&dir.join("ping-up")).unwrap());
    drop(open_exclusive(&dir.join("ping-dn")).unwrap());
    fs::remove_dir_all(dir).unwrap();
}

struct Rig {
    fw: Firmware,
    down: Rc<Mapping>,
    up: Rc<Mapping>,
}

impl Rig {
    fn new(depth: u32, index: u32) -> Self {
        let fw = firmware(
            named_note([None; 2], RINGS, &["ping"], depth),
            Some(new_session()),
        );
        let down = anon(65536);
        let up = anon(65536);
        fw.session().unwrap().link.write(0x100 + shm::HEAD, index);
        fw.session().unwrap().link.write(0x200 + shm::TAIL, index);
        down.write(shm::TAIL, index);
        up.write(shm::HEAD, index);
        Self { fw, down, up }
    }

    fn client(&self) -> Client<u32, u32> {
        let (Channel { tx, rx }, depth, request_wake) = self
            .fw
            .open_pair("ping", true, |r| {
                Ok(if r.kind == ShmKind::RingToLinux {
                    self.up.clone()
                } else {
                    self.down.clone()
                })
            })
            .unwrap();
        Client::new(tx, rx, depth, request_wake)
    }

    fn server(&self) -> (shm::RingRx<u32>, shm::RingTx<u32>) {
        let (down, up) =
            pair::<u32, u32>(Note::parse(&self.fw.note).unwrap(), "ping", true).unwrap();
        let link = &self.fw.session().unwrap().link;
        let ov = shm::Overflow::Reject;
        let rx = unsafe {
            shm::RingRx::attach(
                link.ptr().add(down.sram_offset as usize),
                down.sram_len as usize,
                self.down.ptr(),
                self.down.len,
                down.depth,
                ov,
            )
        }
        .unwrap();
        let tx = unsafe {
            shm::RingTx::attach(
                self.up.ptr(),
                self.up.len,
                link.ptr().add(up.sram_offset as usize),
                up.sram_len as usize,
                up.depth,
                ov,
            )
        }
        .unwrap();
        (rx, tx)
    }
}

#[test]
fn rpc_discards_late_replies_after_reopen_and_index_wrap() {
    let rig = Rig::new(2, u32::MAX);
    let (mut requests, mut replies) = rig.server();
    let mut old = rig.client();
    assert!(matches!(old.call(&1, Duration::ZERO), Err(Error::Timeout)));
    drop(old);
    let mut client = rig.client();
    let (irq, signal) = irq_pair();
    client.rx.irq = Some(irq);
    thread::scope(|scope| {
        let mut signal = &signal;
        scope.spawn(move || {
            use std::io::Write;
            assert_eq!(requests.pop().unwrap(), Some(1));
            let end = Instant::now() + Duration::from_secs(1);
            let second = loop {
                if let Some(value) = requests.pop().unwrap() {
                    break value;
                }
                assert!(Instant::now() < end);
                thread::yield_now();
            };
            replies.push(&1).unwrap();
            signal.write_all(&1u32.to_ne_bytes()).unwrap();
            replies.push(&second).unwrap();
            signal.write_all(&2u32.to_ne_bytes()).unwrap();
        });
        assert_eq!(client.call(&2, Duration::from_secs(1)).unwrap(), 2);
    });
    assert_eq!(client.rx.tail(), 1);
}

#[test]
fn rpc_reports_capacity_timeout_and_peer_down() {
    let rig = Rig::new(1, 0);
    let (mut requests, mut replies) = rig.server();
    let mut client = rig.client();
    assert!(matches!(
        client.call(&7, Duration::ZERO),
        Err(Error::Timeout)
    ));
    assert!(matches!(client.call(&8, Duration::ZERO), Err(Error::Busy)));
    assert_eq!(requests.pop().unwrap(), Some(7));
    replies.push(&7).unwrap();
    assert!(matches!(
        client.call(&8, Duration::from_millis(1)),
        Err(Error::Timeout)
    ));
    assert_eq!(requests.pop().unwrap(), Some(8));
    let link = rig.fw.session().unwrap().link.clone();
    assert_eq!(link.len, LINK_PAGE);
    link.write(
        proto::status::BLOCKS[1] + proto::status::STATE,
        CoreState::Panicked.word(),
    );
    assert!(matches!(
        client.call(&9, Duration::ZERO),
        Err(Error::PeerDown(crate::PeerDown::Terminal {
            core: 1,
            state: CoreState::Panicked,
            ..
        }))
    ));
}

#[test]
fn rpc_takes_the_reply_available_at_the_deadline() {
    let rig = Rig::new(2, 0);
    let (mut requests, mut replies) = rig.server();
    let mut client = rig.client();
    for value in [7, 8] {
        assert!(matches!(
            client.call(&value, Duration::ZERO),
            Err(Error::Timeout)
        ));
        assert_eq!(requests.pop().unwrap(), Some(value));
    }
    for value in [7, 8] {
        replies.push(&value).unwrap();
    }
    assert_eq!(client.wait_reply(1, Instant::now()).unwrap(), 8);
}

mod direct {
    use super::*;
    use crate::{PeerDown, WaitState, Waitable, notify::fake_doorbell, tests::strict_session};
    use rp1_proto::{
        link,
        shm::wake::{RPC_WAIT_HEAD, RX_SEQ, RX_WAIT},
        status,
        wake::IRQ_RPC_CAPS,
    };
    use std::{
        cell::RefCell,
        io::{Read, Write},
        os::unix::net::UnixStream,
    };

    const DOWN: usize = 0x100;
    const UP: usize = 0x200;
    const DOORBELL: usize = 0x2114;
    const BUDGET: Duration = Duration::from_millis(30);

    struct Vector {
        peer: UnixStream,
        deliveries: u32,
        enabled: bool,
        pending: bool,
    }

    impl Vector {
        fn sync_enable(&mut self) {
            let mut word = [0; 4];
            loop {
                match self.peer.read(&mut word) {
                    Ok(4) => assert_eq!(u32::from_ne_bytes(word), 1),
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => return,
                    result => panic!("bad UIO enable {result:?}"),
                }
                self.enabled = true;
                if std::mem::take(&mut self.pending) {
                    self.pulse();
                }
            }
        }

        fn pulse(&mut self) {
            if std::mem::take(&mut self.enabled) {
                self.deliveries += 1;
                self.peer.write_all(&self.deliveries.to_ne_bytes()).unwrap();
            } else {
                self.pending = true;
            }
        }
    }

    struct Direct {
        rig: Rig,
        page: Rc<Mapping>,
        vector: Rc<RefCell<Vector>>,
    }

    impl Direct {
        fn new() -> Self {
            let (session, peer) = strict_session(IRQ_RPC_CAPS);
            let rings = [("ping-dn", 2, true), ("ping-up", 2, true)];
            let fw = firmware(
                named_note([None, Some(27)], &rings, &["ping"], 4),
                Some(session),
            );
            let mut vector = Vector {
                peer,
                deliveries: 0,
                enabled: false,
                pending: false,
            };
            vector.sync_enable();
            assert!(vector.enabled);
            Self {
                rig: Rig {
                    fw,
                    down: anon(65536),
                    up: anon(65536),
                },
                page: anon(0x4000),
                vector: Rc::new(RefCell::new(vector)),
            }
        }

        fn link(&self) -> Rc<Mapping> {
            self.rig.fw.session().unwrap().link.clone()
        }

        fn open(&self) -> Result<Client<u32, u32>> {
            let (Channel { tx, rx }, depth, direct) =
                fake_doorbell::with_page(self.page.clone(), || {
                    self.rig.fw.open_pair("ping", true, |region| {
                        Ok(if region.kind == ShmKind::RingToLinux {
                            self.rig.up.clone()
                        } else {
                            self.rig.down.clone()
                        })
                    })
                })?;
            assert!(direct);
            Ok(Client::new(tx, rx, depth, direct))
        }

        fn hook(
            &self,
            mut step: impl FnMut(
                &Mapping,
                &Mapping,
                &mut Vector,
                &mut shm::RingRx<u32>,
                &mut shm::RingTx<u32>,
            ) + 'static,
        ) {
            let (mut requests, mut replies) = self.rig.server();
            let (link, up, vector) = (self.link(), self.rig.up.clone(), self.vector.clone());
            fake_doorbell::hook(move || {
                let mut vector = vector.borrow_mut();
                vector.sync_enable();
                step(&link, &up, &mut vector, &mut requests, &mut replies);
            });
        }
    }

    #[test]
    fn ready_first_burst_and_masked_replay() {
        for legacy_pulse in [false, true] {
            let direct = Direct::new();
            let mut client = direct.open().unwrap();
            direct.hook(move |link, _, vector, requests, replies| {
                let (value, requested) = requests.pop_request().unwrap().unwrap();
                assert!(requested);
                replies.push(&value).unwrap();
                assert_eq!(link.read(UP + RX_WAIT) & 1, 0);
                if legacy_pulse {
                    vector.pulse();
                }
            });
            for value in [19, 23] {
                assert_eq!(client.call(&value, BUDGET).unwrap(), value);
            }
            fake_doorbell::clear_hook();
            assert_eq!(client.wait_stats(), Default::default());
            assert_eq!(direct.vector.borrow().pending, legacy_pulse);

            let channel = anon(4096);
            let mut region =
                pair::<u32, u32>(Note::parse(&direct.rig.fw.note).unwrap(), "ping", true)
                    .unwrap()
                    .1;
            region.sram_offset += 0x100;
            let session = direct.rig.fw.session().unwrap();
            let mut rx = Firmware::attach_rx::<u32>(region, channel, session.clone()).unwrap();
            let mut ends: [&mut dyn Waitable; 1] = [&mut rx];
            let mut wait = direct.rig.fw.wait_set(&mut ends).unwrap();
            assert_eq!(
                wait.prepare(Instant::now() + BUDGET).unwrap(),
                WaitState::Armed
            );
            direct.vector.borrow_mut().sync_enable();
            if legacy_pulse {
                assert_eq!(wait.dispatch(libc::POLLIN).unwrap(), WaitState::Armed);
            }
            drop(wait);
            assert_eq!(rx.wait_stats().woken, u64::from(legacy_pulse));
            assert_eq!(
                direct.vector.borrow().deliveries,
                if legacy_pulse { 2 } else { 0 }
            );
            assert_eq!(rx.try_recv().unwrap(), None);
        }
    }

    #[test]
    fn direct_client_ready_first_then_intake_before_asking_for_an_irq() {
        let direct = Direct::new();
        let mut client = direct.open().unwrap();
        direct.vector.borrow_mut().pulse();
        let mut deferred = None;
        direct.hook(move |link, up, irq, requests, replies| {
            let ticket = link.read(UP + RX_WAIT);
            if ticket & 1 != 0 {
                assert!(
                    irq.enabled,
                    "take the old count before asking for a reply IRQ"
                );
                let value = deferred.take().unwrap();
                replies.push(&value).unwrap();
                assert_eq!(link.read(UP + RPC_WAIT_HEAD), replies.head());
                up.write(RX_SEQ, ticket);
                irq.pulse();
            } else {
                let (value, requested) = requests.pop_request().unwrap().unwrap();
                assert!(requested);
                assert!(!irq.enabled);
                if value == 7 {
                    replies.push(&value).unwrap();
                } else {
                    deferred = Some(value);
                }
            }
        });
        assert_eq!(client.call(&7, BUDGET).unwrap(), 7);
        assert_eq!(client.wait_stats(), Default::default());
        assert_eq!(client.call(&9, BUDGET).unwrap(), 9);
        fake_doorbell::clear_hook();
        direct.vector.borrow_mut().sync_enable();
        assert_eq!(direct.vector.borrow().deliveries, 2);
        assert_eq!(client.wait_stats().woken, 1);
        assert_eq!(client.wait_stats().deadline_checks, 0);
        assert_eq!(direct.link().read(UP + RX_WAIT) & 1, 0);
    }

    #[derive(Clone, Copy)]
    enum Deadline {
        Reply,
        Empty,
        Terminal,
        Restarted,
    }

    #[test]
    fn direct_client_checks_data_and_peer_at_deadline_without_an_irq() {
        use Deadline::*;
        for case in [Reply, Empty, Terminal, Restarted] {
            let direct = Direct::new();
            let mut client = direct.open().unwrap();
            direct.hook(move |link, _, _, requests, replies| {
                if link.read(UP + RX_WAIT) & 1 == 0 {
                    assert_eq!(requests.pop_request(), Ok(Some((7, true))));
                    return;
                }
                if !matches!(case, Empty) {
                    replies.push(&7).unwrap();
                }
                match case {
                    Terminal => {
                        let block = status::BLOCKS[1];
                        link.write(block + status::REASON, 5);
                        link.write(block + status::STATE, CoreState::Panicked.word());
                    }
                    Restarted => link.write(status::EPOCH, 4),
                    _ => {}
                }
            });
            let result = client.call(&7, BUDGET);
            fake_doorbell::clear_hook();
            match case {
                Reply => {
                    assert_eq!(result.unwrap(), 7);
                    assert_eq!(client.wait_stats().late, 1);
                    assert_eq!(client.wait_stats().deadline_checks, 1);
                }
                Empty => {
                    assert!(matches!(result, Err(Error::Timeout)));
                    assert_eq!(client.wait_stats().deadline_checks, 1);
                }
                Terminal => assert!(matches!(
                    result,
                    Err(Error::PeerDown(PeerDown::Terminal {
                        core: 1,
                        state: CoreState::Panicked,
                        reason: 5
                    }))
                )),
                Restarted => assert!(matches!(result, Err(Error::PeerDown(PeerDown::Restarted)))),
            }
        }
    }

    #[test]
    fn direct_client_discards_stale_flagged_and_zero_timeout_replies() {
        let direct = Direct::new();
        let mut client = direct.open().unwrap();
        assert!(matches!(client.call(&7, BUDGET), Err(Error::Timeout)));
        assert!(matches!(
            client.call(&8, Duration::ZERO),
            Err(Error::Timeout)
        ));
        direct.hook(|_, _, _, requests, replies| {
            for (item, wake) in [(7, true), (8, false), (9, true)] {
                assert_eq!(requests.pop_request(), Ok(Some((item, wake))));
                replies.push(&item).unwrap();
            }
        });
        assert_eq!(client.call(&9, BUDGET).unwrap(), 9);
        fake_doorbell::clear_hook();
        assert_eq!(client.rx.tail(), 3);
    }

    #[test]
    fn direct_client_reopen_forces_pending_request_after_validating_generation() {
        let direct = Direct::new();
        let mut client = direct.open().unwrap();
        client.tx.inner.push_request(&7, true).unwrap();
        drop(client);
        assert_eq!(direct.link().read(DOWN + shm::HEAD), 1);
        assert_eq!(direct.page.read(DOORBELL), 0);
        direct.link().write(link::SHM_GENERATION, 4);
        assert!(matches!(
            direct.open(),
            Err(Error::PeerDown(PeerDown::Restarted))
        ));
        assert_eq!(direct.page.read(DOORBELL), 0);
        direct.link().write(UP + RX_WAIT, 7);
        direct.link().write(UP + RPC_WAIT_HEAD, 1);
        assert!(matches!(
            direct.open(),
            Err(Error::PeerDown(PeerDown::Restarted))
        ));
        assert_eq!(direct.page.read(DOORBELL), 0);
        assert_eq!(direct.link().read(UP + RX_WAIT), 7);
        direct.link().write(link::SHM_GENERATION, 2);
        fake_doorbell::hook({
            let link = direct.link();
            move || assert_eq!(link.read(UP + RX_WAIT), 8)
        });
        let _client = direct.open().unwrap();
        fake_doorbell::clear_hook();
        assert_eq!(direct.page.read(DOORBELL), 1 << 27);
    }
}
