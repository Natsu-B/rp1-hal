use super::*;
use crate::{
    CoreState, LINK_PAGE, open_exclusive, shm,
    tests::{anon, anon_with_file, firmware, irq_pair, named_note, new_session},
};
use std::{collections::HashSet, fs, thread};

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
    let (channel, _) = fw.open_pair::<u32, u32>("ping", false, &mut map).unwrap();
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
        let (Channel { tx, rx }, depth) = self
            .fw
            .open_pair("ping", true, |r| {
                Ok(if r.kind == ShmKind::RingToLinux {
                    self.up.clone()
                } else {
                    self.down.clone()
                })
            })
            .unwrap();
        Client { tx, rx, depth }
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
        Err(Error::PeerDown {
            core: 1,
            state: CoreState::Panicked,
            ..
        })
    ));
}

#[test]
fn request_and_reply_steps_preserve_capacity_and_order() {
    // (request head, request tail, reply head, reply tail, active call's index)
    type State = (u8, u8, u8, u8, Option<u8>);
    for depth in [1, 2] {
        let mut seen = HashSet::new();
        let mut todo: Vec<State> = vec![(0, 0, 0, 0, None)];
        while let Some(state @ (sent, taken, replied, reclaimed, active)) = todo.pop() {
            if !seen.insert(state) {
                continue;
            }
            assert!(reclaimed <= replied && replied <= taken && taken <= sent);
            assert!(sent - reclaimed <= depth);
            if active.is_none() && sent < 5 && sent - reclaimed < depth {
                todo.push((sent + 1, taken, replied, reclaimed, Some(sent)));
            }
            if taken < sent && taken == replied {
                todo.push((sent, taken + 1, replied, reclaimed, active));
            }
            if replied < taken {
                assert!(replied - reclaimed < depth);
                todo.push((sent, taken, replied + 1, reclaimed, active));
            }
            if active.is_some() {
                // Timeout or close/reopen clears only the caller's local index.
                todo.push((sent, taken, replied, reclaimed, None));
            }
            if reclaimed < replied {
                assert!(active.is_none_or(|index| reclaimed <= index));
                let next = active.filter(|&index| reclaimed != index);
                todo.push((sent, taken, replied, reclaimed + 1, next));
            }
        }
    }
}
