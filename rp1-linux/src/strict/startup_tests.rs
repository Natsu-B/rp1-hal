use super::*;
use crate::tests::anon;
use rp1_proto::status;
use std::{
    fs::File,
    io::{Read, Write},
    os::{fd::OwnedFd, unix::net::UnixStream},
};

fn listener() -> (Listener, UnixStream) {
    let (client, mut peer) = UnixStream::pair().unwrap();
    client.set_nonblocking(true).unwrap();
    let listener = Listener::enable(File::from(OwnedFd::from(client))).unwrap();
    let mut word = [0; 4];
    peer.read_exact(&mut word).unwrap();
    assert_eq!(u32::from_ne_bytes(word), 1);
    (listener, peer)
}

fn profile() -> Profile {
    Profile::open(LinkWake {
        startup_doorbell: Some(26),
        owner: 1,
        required_caps: REQUIRED,
        offered_caps: wake::IRQ_STARTUP_CAPS,
    })
    .unwrap()
}

fn page() -> Rc<Mapping> {
    let page = anon(crate::LINK_PAGE);
    page.write(status::ABI, status::ABI_VALUE);
    page.write(status::EPOCH, 2);
    page.write(status::PLAT_PCIE, status::PLAT_PCIE_UP);
    page.write(mailbox::BOOT_EPOCH, 2);
    page.write(mailbox::OFFERED_CAPS, wake::IRQ_STARTUP_CAPS);
    page
}

fn ready(page: &Mapping) {
    page.write(mailbox::RESULT_SEQ, mailbox::FIRST_REQUEST);
    page.write(mailbox::ACCEPTED_CAPS, wake::IRQ_STARTUP_CAPS);
    page.write(link::SHM_GENERATION, 2);
    page.write(link::SHM_STATUS, link::SHM_READY);
}

fn startup(profile: &Profile, page: &Mapping, listener: &Listener) -> Result<(wake::Session, u32)> {
    profile.startup(
        page,
        listener,
        Instant::now() + Duration::from_millis(50),
        |pin| {
            assert_eq!(pin, 26);
            Ok(Doorbell::new(anon(crate::LINK_PAGE), 0, pin))
        },
    )
}

#[test]
fn startup_arms_before_force_recovers_inherited_request_and_reopens_ready() {
    for inherited in [false, true] {
        let page = page();
        if inherited {
            page.write(mailbox::REQUEST_EPOCH, 2);
            page.write(mailbox::REQUEST_SEQ, mailbox::FIRST_REQUEST);
            page.write(mailbox::REQUIRED_CAPS, wake::IRQ_STARTUP_CAPS);
            page.write(link::SHM_REQUEST, mailbox::REQUEST_OPCODE);
            page.write(mailbox::WAIT, 1);
        }
        let expect_arm = if inherited { 3 } else { 1 };
        let (listener, _peer) = listener();
        let profile = profile();
        let at_force = page.clone();
        crate::notify::fake_doorbell::hook(move || {
            assert_eq!(at_force.read(mailbox::WAIT), expect_arm);
            assert_eq!(at_force.read(mailbox::REQUEST_EPOCH), 2);
            assert_eq!(at_force.read(mailbox::REQUEST_SEQ), mailbox::FIRST_REQUEST);
            assert_eq!(
                at_force.read(mailbox::REQUIRED_CAPS),
                wake::IRQ_STARTUP_CAPS
            );
            assert_eq!(at_force.read(link::SHM_REQUEST), mailbox::REQUEST_OPCODE);
            at_force.write(mailbox::SEQ, expect_arm);
            ready(&at_force);
        });
        assert_eq!(
            startup(&profile, &page, &listener).unwrap().0.generation(),
            2
        );
        assert_eq!(page.read(mailbox::ACK), expect_arm);
        assert_eq!(page.read(mailbox::WAIT), expect_arm + 1);
        crate::notify::fake_doorbell::clear_hook();
        crate::notify::fake_doorbell::hook(|| panic!("reopen rings after Ready"));
        let (second, _peer) = self::listener();
        assert_eq!(startup(&profile, &page, &second).unwrap().0.generation(), 2);
        assert_eq!(page.read(mailbox::WAIT), expect_arm + 1);
        crate::notify::fake_doorbell::clear_hook();
    }
}

#[test]
fn stale_boot_cannot_ack_cancel_or_force_on_reopen() {
    let page = page();
    let (listener, _peer) = listener();
    let profile = profile();
    let at_force = page.clone();
    crate::notify::fake_doorbell::hook(move || {
        // The runtime changes EPOCH before clearing the old mailbox.
        at_force.write(status::EPOCH, 3);
        at_force.write(mailbox::WAIT, 7);
        at_force.write(mailbox::SEQ, 1);
        at_force.write(mailbox::ACK, 5);
    });
    assert!(matches!(
        startup(&profile, &page, &listener),
        Err(Error::PeerDown(PeerDown::Restarted))
    ));
    assert_eq!(page.read(mailbox::ACK), 5);
    assert_eq!(page.read(mailbox::WAIT), 7);
    page.write(status::EPOCH, 4);
    page.write(mailbox::BOOT_EPOCH, 4);
    crate::notify::fake_doorbell::clear_hook();
    crate::notify::fake_doorbell::hook(|| panic!("stale profile rang a new boot"));
    assert!(matches!(
        startup(&profile, &page, &listener),
        Err(Error::PeerDown(PeerDown::Restarted))
    ));
    assert_eq!(page.read(mailbox::REQUEST_EPOCH), 2);
}

#[test]
fn startup_replaces_a_request_left_by_a_previous_boot() {
    let page = page();
    page.write(mailbox::REQUEST_EPOCH, 4);
    page.write(mailbox::REQUEST_SEQ, 42);
    page.write(mailbox::REQUIRED_CAPS, wake::CREDIT_WAKE);
    page.write(link::SHM_REQUEST, mailbox::REQUEST_OPCODE);
    let (listener, _peer) = listener();
    let at_force = page.clone();
    crate::notify::fake_doorbell::hook(move || {
        assert_eq!(at_force.read(mailbox::REQUEST_EPOCH), 2);
        assert_eq!(at_force.read(mailbox::REQUEST_SEQ), mailbox::FIRST_REQUEST);
        assert_eq!(
            at_force.read(mailbox::REQUIRED_CAPS),
            wake::IRQ_STARTUP_CAPS
        );
        assert_eq!(at_force.read(link::SHM_REQUEST), mailbox::REQUEST_OPCODE);
        at_force.write(mailbox::SEQ, 1);
        ready(&at_force);
    });
    assert_eq!(
        startup(&profile(), &page, &listener)
            .unwrap()
            .0
            .generation(),
        2
    );
    assert_eq!(page.read(mailbox::ACK), 1);
}

#[test]
fn unadvertised_window_reports_restart_before_arming() {
    let page = page();
    let (listener, _peer) = listener();
    for word in [0, 3] {
        page.write(mailbox::BOOT_EPOCH, word);
        assert!(matches!(
            startup(&profile(), &page, &listener),
            Err(Error::PeerDown(PeerDown::Restarted))
        ));
        assert_eq!(page.read(mailbox::WAIT), 0);
    }
}

#[test]
fn startup_rejects_capability_mismatch_and_prioritizes_window_loss() {
    let page = page();
    let (listener, _peer) = listener();
    let profile = profile();
    crate::notify::fake_doorbell::hook(|| panic!("unsupported request rang the doorbell"));
    page.write(mailbox::OFFERED_CAPS, wake::IRQ_RPC_CAPS);
    assert!(matches!(
        startup(&profile, &page, &listener),
        Err(Error::Unsupported)
    ));
    assert_eq!(page.read(mailbox::WAIT), 0);
    page.write(mailbox::OFFERED_CAPS, wake::IRQ_STARTUP_CAPS);
    page.write(mailbox::RESULT_SEQ, mailbox::FIRST_REQUEST);
    page.write(link::SHM_STATUS, StartupError::Initialization as u32);
    assert!(matches!(
        startup(&profile, &page, &listener),
        Err(Error::Startup(StartupError::Initialization))
    ));
    page.write(status::PLAT_PCIE, status::PLAT_PCIE_LOST);
    assert!(matches!(
        startup(&profile, &page, &listener),
        Err(Error::PeerDown(PeerDown::LinkDown))
    ));
}

#[test]
fn deadline_intakes_the_final_count_before_classifying_late_completion() {
    for count in [false, true] {
        let (listener, mut peer) = listener();
        if count {
            peer.write_all(&1u32.to_ne_bytes()).unwrap();
        }
        let end = Instant::now();
        let mut checks = 0;
        let result = wait_startup(&listener, end, || {
            checks += 1;
            Ok((checks == 2).then_some(23))
        });
        assert_eq!(checks, 2);
        if count {
            assert_eq!(result.unwrap(), 23);
        } else {
            assert!(matches!(result, Err(Error::NotifyLost)));
        }
    }
}
