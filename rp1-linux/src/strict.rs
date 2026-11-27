use crate::{Error, Firmware, Mapping, Result, Session, SessionHeader, notify::listener::Listener};
use rp1_proto::{
    self as proto, barrier, link,
    note::LinkWake,
    shm::wake::Words,
    wake::{self, BootEpoch, Request, SessionError, mailbox},
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    time::{Duration, Instant},
};

type Live<S> = Box<dyn Fn() -> std::result::Result<S, SessionError>>;
pub(crate) type Ticket<S> = proto::shm::wake::Waiter<S, Live<S>>;

pub(crate) struct Profile {
    config: LinkWake,
    epoch: Cell<Option<BootEpoch>>,
}

impl Profile {
    pub(crate) fn open(config: LinkWake) -> Result<Self> {
        if config.required_caps & !wake::B2_OFFERED_CAPS != 0 {
            return Err(Error::Unsupported);
        }
        Ok(Self {
            config,
            epoch: Cell::new(None),
        })
    }

    pub(crate) fn ready(&self, fw: &Firmware, timeout: Duration) -> Result<Session> {
        let end = Instant::now()
            .checked_add(timeout)
            .ok_or(Error::InvalidDeadline)?;
        let listener = Rc::new(Listener::open()?);
        let epoch = loop {
            let word = fw.link.read(mailbox::BOOT_EPOCH);
            if word == u32::MAX {
                return Err(Error::LinkDown);
            }
            if let Ok(epoch) = BootEpoch::new(word) {
                break epoch;
            }
            startup_pause(end)?;
        };
        if self.epoch.get().is_some_and(|old| old != epoch) {
            return Err(Error::PeerDown(crate::PeerDown::Restarted));
        }
        self.epoch.set(Some(epoch));
        barrier();
        let offered = fw.link.read(mailbox::OFFERED_CAPS);
        if offered != self.config.offered_caps || offered & wake::LATCHED_WAKE == 0 {
            return Err(Error::Unsupported);
        }
        check_epoch(&fw.link, epoch)?;
        check_status_epoch(&fw.link, epoch)?;
        let request = Request {
            epoch: epoch.word(),
            seq: 1,
            required_caps: self.config.required_caps | wake::LATCHED_WAKE,
        };
        // One request identity per boot. A same-boot reopen reads the committed
        // result, including when the initial requester exited before Ready.
        if fw.link.read(link::SHM_REQUEST) != mailbox::REQUEST_OPCODE
            && fw.link.read(link::SHM_STATUS) == 0
        {
            fw.link.write(mailbox::REQUEST_EPOCH, request.epoch);
            fw.link.write(mailbox::REQUEST_SEQ, request.seq);
            fw.link.write(
                mailbox::REQUIRED_CAPS,
                request.required_caps | (offered & wake::RPC_REPLY_WAKE),
            );
            fw.link.write(link::SHM_REQUEST, mailbox::REQUEST_OPCODE);
            let _ = fw.link.read(link::SHM_REQUEST);
            fw.signal_enable_request();
        }
        loop {
            check_epoch(&fw.link, epoch)?;
            crate::Peer::check_core(&fw.link, self.config.owner)?;
            if self.config.owner != 0 {
                crate::Peer::check_core(&fw.link, 0)?;
            }
            match fw.link.read(link::SHM_STATUS) {
                link::SHM_READY => break,
                link::SHM_WINDOW_LOST | u32::MAX => return Err(Error::LinkDown),
                0 => startup_pause(end)?,
                _ => return Err(Error::Unsupported),
            }
        }
        barrier();
        let caps = fw.link.read(mailbox::ACCEPTED_CAPS);
        request
            .check_result(
                epoch.word(),
                fw.link.read(mailbox::RESULT_SEQ),
                caps,
                offered,
            )
            .map_err(|error| match error {
                SessionError::NotReady => Error::from(proto::Error::Mismatch),
                error => Error::from(error),
            })?;
        let scope = wake::Session::new(epoch.word(), fw.link.read(link::SHM_GENERATION))?;
        // Enabling on open can deliver a pending edge from the previous
        // listener. Take it before returning the newly attached session.
        listener.intake()?;
        let session = Session::open(&fw.note, &fw.area, &fw.link, Some((scope, caps, listener)))?;
        session.check()?;
        check_epoch(&fw.link, epoch)?;
        Ok(session)
    }
}

impl StrictSession {
    pub(crate) fn new(
        listener: Rc<Listener>,
        link: &Rc<Mapping>,
        scope: wake::Session,
        caps: u32,
    ) -> Result<StrictSession> {
        let epoch = BootEpoch::new(scope.epoch())?;
        let live_link = link.clone();
        let live: Box<dyn Fn() -> _> =
            Box::new(move || BootEpoch::new(live_link.read(mailbox::BOOT_EPOCH)));
        let mailbox = unsafe { Words::link(link.ptr()).waiter(epoch, live) }?;
        Ok(StrictSession {
            listener,
            busy: Cell::new(false),
            mailbox: RefCell::new(mailbox),
            epoch,
            rpc_reply_wake: caps & wake::RPC_REPLY_WAKE != 0,
        })
    }
}

pub(crate) struct StrictSession {
    pub(crate) listener: Rc<Listener>,
    pub(crate) busy: Cell<bool>,
    pub(crate) mailbox: RefCell<Ticket<BootEpoch>>,
    epoch: BootEpoch,
    pub(crate) rpc_reply_wake: bool,
}

impl StrictSession {
    pub(crate) fn claim_listener(&self) -> Result<impl Drop + '_> {
        if self.busy.replace(true) {
            return Err(Error::Busy);
        }
        Ok(ListenerGuard(&self.busy))
    }

    pub(crate) fn check_terminal_epoch(&self, link: &Mapping) -> Result<()> {
        check_status_epoch(link, self.epoch)
    }

    pub(crate) fn check_link(&self, link: &Mapping) -> Result<()> {
        check_epoch(link, self.epoch)?;
        Self::check_ready(link)
    }

    pub(crate) fn check_rpc_link(&self, link: &Mapping) -> Result<()> {
        // Ready was negotiated against this status epoch. The status epoch
        // changes before the records are reset and is the source of BOOT_EPOCH.
        Self::check_ready(link)?;
        check_epoch_word(link.read(proto::status::EPOCH), self.epoch)
    }

    fn check_ready(link: &Mapping) -> Result<()> {
        match link.read(proto::link::SHM_STATUS) {
            proto::link::SHM_READY => Ok(()),
            proto::link::SHM_WINDOW_LOST | u32::MAX => {
                Err(Error::PeerDown(crate::PeerDown::LinkDown))
            }
            _ => Err(Error::PeerDown(crate::PeerDown::Restarted)),
        }
    }
}

struct ListenerGuard<'a>(&'a Cell<bool>);

impl Drop for ListenerGuard<'_> {
    fn drop(&mut self) {
        self.0.set(false);
    }
}

fn check_epoch(link: &Mapping, epoch: BootEpoch) -> Result<()> {
    check_epoch_word(link.read(mailbox::BOOT_EPOCH), epoch)
}

fn check_epoch_word(word: u32, epoch: BootEpoch) -> Result<()> {
    match word {
        u32::MAX => Err(Error::PeerDown(crate::PeerDown::LinkDown)),
        word if word != epoch.word() => Err(Error::PeerDown(crate::PeerDown::Restarted)),
        _ => Ok(()),
    }
}

fn check_status_epoch(link: &Mapping, epoch: BootEpoch) -> Result<()> {
    use proto::status::{ABI, ABI_VALUE, EPOCH};
    match (link.read(ABI), link.read(EPOCH)) {
        (u32::MAX, _) | (_, u32::MAX) => Err(Error::PeerDown(crate::PeerDown::LinkDown)),
        (ABI_VALUE, word) if word == epoch.word() => Ok(()),
        _ => Err(Error::PeerDown(crate::PeerDown::Restarted)),
    }
}

pub(crate) fn ring_waiter(
    session: &Session,
    data: &Mapping,
    offset: usize,
) -> Result<Ticket<wake::Session>> {
    let SessionHeader::V3(header) = session.header else {
        return Err(Error::Unsupported);
    };
    let scope = header.session()?;
    let live: Box<dyn Fn() -> _> = Box::new(move || header.session());
    let words = unsafe { Words::rx(data.ptr(), session.link.ptr().add(offset)) };
    Ok(unsafe { words.waiter(scope, live) }?)
}

pub(crate) struct RingPublisher {
    publisher: proto::shm::wake::Publisher<wake::Session, Live<wake::Session>>,
    head: usize,
}

impl RingPublisher {
    pub(crate) fn new(session: &Session, ctrl: &Mapping, head: usize) -> Result<Self> {
        let SessionHeader::V3(header) = session.header else {
            return Err(Error::Unsupported);
        };
        let scope = header.session()?;
        let live: Box<dyn Fn() -> _> = Box::new(move || header.session());
        let words = unsafe { Words::rx(session.link.ptr().add(head), ctrl.ptr()) };
        let publisher = unsafe { words.publisher(scope, live) }?;
        Ok(Self { publisher, head })
    }

    pub(crate) fn publish(&mut self, link: &Mapping) -> Result<bool> {
        let head = self.head;
        Ok(self.publisher.publish(|| Self::read_head(link, head))?)
    }

    fn read_head(link: &Mapping, head: usize) {
        let _ = link.read(head);
    }

    pub(crate) fn recover(&self) -> Result<bool> {
        Ok(self.publisher.needs_recovery_kick()?)
    }
}

fn startup_pause(end: Instant) -> Result<()> {
    // B2 retains the compatibility startup retry until the startup doorbell PR.
    let delay = end.saturating_duration_since(Instant::now());
    if delay.is_zero() {
        return Err(Error::Timeout);
    }
    std::thread::sleep(delay.min(Duration::from_millis(1)));
    Ok(())
}
