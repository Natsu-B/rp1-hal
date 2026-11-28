use crate::{
    Error, Firmware, Mapping, PeerDown, Result, Session, SessionHeader,
    notify::{
        Doorbell,
        listener::{Listener, poll_until},
    },
};
use rp1_proto::{
    self as proto, barrier, link,
    note::LinkWake,
    shm::wake::Words,
    wake::{self, BootEpoch, Request, SessionError, StartupError, mailbox},
};
use std::{
    cell::{Cell, RefCell},
    os::fd::{AsFd, AsRawFd},
    rc::Rc,
    time::{Duration, Instant},
};

type Live<S> = Box<dyn Fn() -> std::result::Result<S, SessionError>>;
pub(crate) type Ticket<S> = proto::shm::wake::Waiter<S, Live<S>>;

pub(crate) struct Profile {
    config: LinkWake,
    epoch: Cell<Option<BootEpoch>>,
    pin: u8,
}

impl Profile {
    pub(crate) fn open(config: LinkWake) -> Result<Self> {
        if config.required_caps & !wake::IRQ_STARTUP_CAPS != 0 {
            return Err(Error::Unsupported);
        }
        Ok(Self {
            config,
            epoch: Cell::new(None),
            pin: config.startup_doorbell.ok_or(Error::Unsupported)?,
        })
    }

    pub(crate) fn ready(&self, fw: &Firmware, timeout: Duration) -> Result<Session> {
        let end = Instant::now()
            .checked_add(timeout)
            .ok_or(Error::InvalidDeadline)?;
        let listener = Rc::new(Listener::open()?);
        let (scope, caps) = self.startup(&fw.link, &listener, end, |pin| Doorbell::open(0, pin))?;
        let session = Session::open(&fw.note, &fw.area, &fw.link, Some((scope, caps, listener)))?;
        session.check()?;
        session.check_link_generation()?;
        Ok(session)
    }

    fn startup(
        &self,
        page: &Mapping,
        listener: &Listener,
        end: Instant,
        doorbell: impl FnOnce(u8) -> Result<Doorbell>,
    ) -> Result<(wake::Session, u32)> {
        let word = page.read(mailbox::BOOT_EPOCH);
        if word == u32::MAX {
            return Err(Error::LinkDown);
        }
        // A warm reload can clear the mailbox while BAR2 is still mapped.
        let epoch = BootEpoch::new(word).map_err(|_| Error::PeerDown(PeerDown::Restarted))?;
        if self.epoch.get().is_some_and(|old| old != epoch) {
            return Err(Error::PeerDown(PeerDown::Restarted));
        }
        self.check_startup_peer(page, epoch)?;
        self.epoch.set(Some(epoch));
        barrier();
        let offered = page.read(mailbox::OFFERED_CAPS);
        if offered != self.config.offered_caps || offered & REQUIRED != REQUIRED {
            return Err(Error::Unsupported);
        }
        let request = Request {
            epoch: epoch.word(),
            seq: mailbox::FIRST_REQUEST,
            required_caps: self.config.required_caps | REQUIRED,
        };
        let result = || self.startup_result(page, epoch, request);
        if let Some(ready) = result()? {
            return Ok(ready);
        }
        if Instant::now() >= end {
            return Err(Error::Timeout);
        }
        let doorbell = doorbell(self.pin)?;
        let live = || ticket_epoch(page, epoch);
        let mut ticket = unsafe { Words::link(page.ptr()).waiter(epoch, live) }?;
        ticket.arm()?;
        let ready = (|| {
            if let Some(ready) = result()? {
                return Ok(ready);
            }
            if Instant::now() >= end {
                return Err(Error::Timeout);
            }
            Self::publish_request(page, request, offered, &doorbell)?;
            wait_startup(listener, end, result)
        })();
        let cleanup = ticket.acknowledge_and_cancel();
        ready.and_then(|ready| {
            cleanup?;
            Ok(ready)
        })
    }

    fn publish_request(
        page: &Mapping,
        request: Request,
        offered: u32,
        doorbell: &Doorbell,
    ) -> Result<()> {
        let epoch = BootEpoch::new(request.epoch)?;
        check_epoch(page, epoch)?;
        check_status_epoch(page, epoch)?;
        let publish = match page.read(link::SHM_REQUEST) {
            0 => true,
            mailbox::REQUEST_OPCODE if page.read(mailbox::REQUEST_EPOCH) != request.epoch => true,
            // An exited launcher can leave the request committed before it rang.
            mailbox::REQUEST_OPCODE if page.read(mailbox::REQUEST_SEQ) == request.seq => false,
            u32::MAX => return Err(Error::PeerDown(PeerDown::LinkDown)),
            _ => return Err(Error::Unsupported),
        };
        if publish {
            page.write(mailbox::REQUEST_SEQ, request.seq);
            page.write(
                mailbox::REQUIRED_CAPS,
                request.required_caps | (offered & wake::RPC_REPLY_WAKE),
            );
            // The listener lock serializes Linux writers. With a retained opcode,
            // the new epoch makes the replacement visible to proc0.
            barrier();
            page.write(mailbox::REQUEST_EPOCH, request.epoch);
            barrier();
            page.write(link::SHM_REQUEST, mailbox::REQUEST_OPCODE);
        }
        let _ = page.read(link::SHM_REQUEST);
        barrier();
        check_epoch(page, epoch)?;
        check_status_epoch(page, epoch)?;
        match page.read(link::SHM_STATUS) {
            0 => doorbell.ring_flushed(),
            link::SHM_WINDOW_LOST | u32::MAX => return Err(Error::PeerDown(PeerDown::LinkDown)),
            _ => {}
        }
        Ok(())
    }

    fn startup_result(
        &self,
        page: &Mapping,
        epoch: BootEpoch,
        request: Request,
    ) -> Result<Option<(wake::Session, u32)>> {
        self.check_startup_peer(page, epoch)?;
        let status = page.read(link::SHM_STATUS);
        if status == 0 {
            return Ok(None);
        }
        barrier();
        let seq = page.read(mailbox::RESULT_SEQ);
        let caps = page.read(mailbox::ACCEPTED_CAPS);
        let generation = page.read(link::SHM_GENERATION);
        self.check_startup_peer(page, epoch)?;
        if status != link::SHM_READY {
            let error = match (StartupError::from_status(status), seq) {
                (Some(StartupError::Timeout), seq) if seq == 0 || seq == request.seq => {
                    Error::Timeout
                }
                (Some(StartupError::Unsupported), seq) if seq == 0 || seq == request.seq => {
                    Error::Unsupported
                }
                (Some(StartupError::StaleRequest), seq) if seq == request.seq => {
                    Error::PeerDown(PeerDown::Restarted)
                }
                (Some(error), seq) if seq == request.seq => Error::Startup(error),
                _ => Error::from(proto::Error::Mismatch),
            };
            return Err(error);
        }
        request
            .check_result(epoch.word(), seq, caps, self.config.offered_caps)
            .map_err(|error| match error {
                SessionError::NotReady => Error::from(proto::Error::Mismatch),
                error => Error::from(error),
            })?;
        if caps != page.read(mailbox::REQUIRED_CAPS) {
            return Err(proto::Error::Mismatch.into());
        }
        let scope = wake::Session::new(epoch.word(), generation)?;
        self.check_startup_peer(page, epoch)?;
        Ok(Some((scope, caps)))
    }

    fn check_startup_peer(&self, page: &Mapping, epoch: BootEpoch) -> Result<()> {
        check_epoch(page, epoch)?;
        check_status_epoch(page, epoch)?;
        let pcie = page.read(proto::status::PLAT_PCIE);
        if matches!(
            page.read(link::SHM_STATUS),
            link::SHM_WINDOW_LOST | u32::MAX
        ) || pcie == u32::MAX
            || matches!(
                proto::status::PcieState::from_word(pcie),
                proto::status::PcieState::Lost { .. }
            )
        {
            return Err(Error::PeerDown(PeerDown::LinkDown));
        }
        crate::Peer::check_core(page, 0)?;
        if self.config.owner != 0 {
            crate::Peer::check_core(page, self.config.owner)?;
        }
        Ok(())
    }
}

const REQUIRED: u32 = wake::LATCHED_WAKE | wake::STARTUP_WAKE;

fn wait_startup<T>(
    listener: &Listener,
    end: Instant,
    mut result: impl FnMut() -> Result<Option<T>>,
) -> Result<T> {
    if let Some(ready) = result()? {
        return Ok(ready);
    }
    let mut fds = [libc::pollfd {
        fd: listener.as_fd().as_raw_fd(),
        events: libc::POLLIN,
        revents: 0,
    }];
    loop {
        poll_until(&mut fds, Some(end))?;
        let woke = listener.intake();
        let ready = result()?;
        let woke = woke?;
        let revents = fds[0].revents;
        if revents & (libc::POLLERR | libc::POLLHUP | libc::POLLNVAL) != 0 {
            return Err(Error::setup(format!("rp1:irq poll flags {revents:#x}")));
        }
        if let Some(ready) = ready {
            return if !woke && Instant::now() >= end {
                Err(Error::NotifyLost)
            } else {
                Ok(ready)
            };
        }
        if Instant::now() >= end {
            return Err(Error::Timeout);
        }
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
        let live: Box<dyn Fn() -> _> = Box::new(move || ticket_epoch(&live_link, epoch));
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
        check_status_epoch(link, self.epoch)?;
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

fn ticket_epoch(
    link: &Mapping,
    expected: BootEpoch,
) -> std::result::Result<BootEpoch, SessionError> {
    // The runtime invalidates status::EPOCH before resetting the mailbox.
    if check_status_epoch(link, expected).is_err()
        || matches!(
            link.read(proto::link::SHM_STATUS),
            proto::link::SHM_WINDOW_LOST | u32::MAX
        )
    {
        return Err(SessionError::Restarted);
    }
    BootEpoch::new(link.read(mailbox::BOOT_EPOCH))
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

pub(crate) fn retire_rpc_wait(session: &Session, ctrl: usize) -> Result<()> {
    let offset = ctrl + proto::shm::wake::RX_WAIT;
    let link = &session.link;
    let arm = link.read(offset);
    if arm == u32::MAX {
        return Err(Error::PeerDown(crate::PeerDown::LinkDown));
    }
    if arm & 1 != 0 {
        // The reply mapping is exclusive. Retire an exited client's arm
        // before the request writer forces any inherited work.
        session.check()?;
        session.check_link_generation()?;
        link.write(offset, arm + 1);
        let _ = link.read(offset);
        barrier();
    }
    Ok(())
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

#[cfg(test)]
mod startup_tests;
