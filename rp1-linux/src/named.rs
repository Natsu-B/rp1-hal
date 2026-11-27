use crate::{
    Error, Firmware, Mapping, Message, Note, Result, RingRx, RingTx, ShmKind, ShmRegion,
    check_type, find_region, notify::listener::poll_until, proto, strict::StrictSession,
};
use rp1_proto::{
    note::{RPC, rpc_name},
    shm::wake::RPC_WAIT_HEAD,
};
use std::{
    os::fd::{AsFd, AsRawFd},
    rc::Rc,
    time::{Duration, Instant},
};

impl Firmware {
    /// Open both rings of the named channel.
    pub fn channel<Tx: Message, Rx: Message>(&self, name: &str) -> Result<Channel<Tx, Rx>> {
        self.open_pair(name, false, Self::map)
            .map(|(channel, _, _)| channel)
    }

    /// Open the named RPC for one caller at a time.
    pub fn client<Req: Message, Rep: Message>(&self, name: &str) -> Result<Client<Req, Rep>> {
        let (Channel { tx, rx }, depth, request_wake) = self.open_pair(name, true, Self::map)?;
        Ok(Client::new(tx, rx, depth, request_wake))
    }

    fn open_pair<Tx: Message, Rx: Message>(
        &self,
        name: &str,
        rpc: bool,
        mut map: impl FnMut(&ShmRegion<'_>) -> Result<Rc<Mapping>>,
    ) -> Result<(Channel<Tx, Rx>, u32, bool)> {
        let session = self.session()?;
        let (down, up) = pair::<Tx, Rx>(Note::parse(&self.note)?, name, rpc)?;
        let up_map = map(&up)?;
        let down_map = map(&down)?;

        let direct_rpc = rpc
            && down.notify_irq
            && up.notify_irq
            && session
                .strict
                .as_ref()
                .is_some_and(|strict| strict.rpc_reply_wake);
        if direct_rpc {
            crate::strict::retire_rpc_wait(&session, up.sram_offset as usize)?;
        }
        let mut rx = Self::attach_rx(up, up_map, session.clone())?;
        let tx = self.attach_tx(down, down_map, session, direct_rpc)?;
        if !rpc && up.notify_irq && !up.overwrite {
            rx.space_doorbell.clone_from(&tx.doorbell);
            // A previous reader may have exited between publishing tail and the IRQ.
            rx.notify_space();
        }
        Ok((Channel { tx, rx }, down.depth, direct_rpc))
    }
}

fn pair<'a, Tx: Message, Rx: Message>(
    note: Note<'a>,
    name: &str,
    rpc: bool,
) -> Result<(ShmRegion<'a>, ShmRegion<'a>)> {
    if name.contains('-') {
        return Err(Error::Unknown);
    }
    let mut is_rpc = false;
    for record in note
        .records()
        .filter_map(std::result::Result::ok)
        .filter(|r| r.kind == RPC)
    {
        is_rpc |= rpc_name(record.value).map_err(|_| proto::Error::Mismatch)? == name;
    }
    if rpc != is_rpc {
        return Err(Error::Unknown);
    }
    let down = find_region(note, &format!("{name}-dn"));
    let up = find_region(note, &format!("{name}-up"));
    let (down, up) = match (down, up) {
        (Some(down), Some(up)) => (down, up),
        (None, None) if !rpc => return Err(Error::Unknown),
        _ => return Err(proto::Error::Mismatch.into()),
    };
    if down.kind != ShmKind::RingToRp1
        || up.kind != ShmKind::RingToLinux
        || down.core != up.core
        || (rpc && (down.depth != up.depth || down.overwrite || up.overwrite))
    {
        return Err(proto::Error::Mismatch.into());
    }
    check_type::<Tx>(&down)?;
    check_type::<Rx>(&up)?;
    Ok((down, up))
}

/// The Linux endpoint of a named pair of message rings.
pub struct Channel<Tx, Rx> {
    tx: RingTx<Tx>,
    rx: RingRx<Rx>,
}

impl<Tx: Message, Rx: Message> Channel<Tx, Rx> {
    /// Publish one message.
    pub fn send(&mut self, value: &Tx) -> Result<()> {
        self.tx.send(value)
    }

    /// Publish a batch, returning the number written.
    pub fn send_slice(&mut self, values: &[Tx]) -> Result<usize> {
        self.tx.send_slice(values)
    }

    /// Take the oldest message without waiting.
    pub fn try_recv(&mut self) -> Result<Option<Rx>> {
        self.rx.try_recv()
    }

    /// Take the oldest message, waiting up to `timeout` (`None` waits indefinitely).
    pub fn recv(&mut self, timeout: Option<Duration>) -> Result<Option<Rx>> {
        self.rx.recv(timeout)
    }

    /// Wait for a reply or message until an absolute deadline.
    pub fn wait(&mut self, end: Instant) -> Result<()> {
        self.rx.wait(end)
    }

    /// Receive a batch, waiting up to `timeout` for its first message.
    pub fn recv_into(&mut self, out: &mut [Rx], timeout: Option<Duration>) -> Result<usize> {
        self.rx.recv_into(out, timeout)
    }

    /// Set the receiver's spin interval before sleeping.
    pub fn set_spin(&mut self, spin: Duration) {
        self.rx.set_spin(spin);
    }

    /// Take independent ownership of the two rings.
    pub fn split(self) -> (RingTx<Tx>, RingRx<Rx>) {
        (self.tx, self.rx)
    }
}

/// The Linux caller of a named request/reply pair.
pub struct Client<Req, Rep> {
    tx: RingTx<Req>,
    rx: RingRx<Rep>,
    depth: u32,
    request_wake: bool,
}

impl<Req: Message, Rep: Message> Client<Req, Rep> {
    pub(crate) fn new(tx: RingTx<Req>, rx: RingRx<Rep>, depth: u32, request_wake: bool) -> Self {
        Self {
            tx,
            rx,
            depth,
            request_wake,
        }
    }

    /// Notification statistics for replies received by this client.
    pub fn wait_stats(&self) -> crate::WaitStats {
        self.rx.wait_stats()
    }

    /// Publish a request and wait for its reply. A zero timeout publishes once if there is
    /// room, then returns [`Error::Timeout`] immediately.
    ///
    /// A timeout leaves the request outstanding. Later calls discard its reply when it
    /// arrives. The peer may already have acted on the request.
    pub fn call(&mut self, request: &Req, timeout: Duration) -> Result<Rep> {
        let end = Instant::now()
            .checked_add(timeout)
            .ok_or(Error::InvalidDeadline)?;
        if timeout.is_zero() {
            self.publish(request, None)?;
            return Err(Error::Timeout);
        }
        self.call_until(request, end)
    }

    /// Publish and await a reply under a single absolute deadline. If the reply
    /// is present at the deadline, return it even if the IRQ count is delayed.
    pub fn call_until(&mut self, request: &Req, end: Instant) -> Result<Rep> {
        if self.request_wake {
            let session = self.rx.session.clone();
            let strict = session
                .strict
                .as_ref()
                .expect("direct RPC uses strict wake");
            let _listener = strict.claim_listener()?;
            let index = self.publish(request, Some(end))?;
            return self.wait_requested_reply(strict, index, end);
        }
        let index = self.publish(request, Some(end))?;
        self.wait_reply(index, end)
    }

    fn publish(&mut self, request: &Req, end: Option<Instant>) -> Result<u32> {
        let check_deadline = || {
            if end.is_some_and(|end| Instant::now() >= end) {
                Err(Error::Timeout)
            } else {
                Ok(())
            }
        };
        check_deadline()?;
        self.rx.session.check()?;
        self.rx.session.check_peer(self.rx.peer.core)?;
        let index = self.tx.inner.head();
        while self.rx.tail() != index {
            check_deadline()?;
            if self.rx.try_recv()?.is_none() {
                break;
            }
        }
        // Unreclaimed requests include replies that are still being produced. Reserve their
        // slots too, so the server always has room to publish in request order.
        if index.wrapping_sub(self.rx.tail()) >= self.depth {
            return Err(Error::Busy);
        }
        check_deadline()?;
        if self.request_wake
            && let Some(doorbell) = &self.tx.doorbell
        {
            self.tx.inner.push_request(request, end.is_some())?;
            self.tx.force_request(doorbell)?;
        } else {
            self.tx.inner.push(request)?;
            self.tx.wake()?;
        }
        Ok(index)
    }

    fn wait_requested_reply(
        &mut self,
        strict: &StrictSession,
        index: u32,
        end: Instant,
    ) -> Result<Rep> {
        self.check_requested_peer(strict)?;
        // A ready-first call leaves the reply ticket idle, so RP1 sends no IRQ.
        if Instant::now() < end
            && let Some(reply) = self.take_requested_reply(index)?
        {
            return Ok(reply);
        }
        let mut ticket = self.rx.ticket.take().ok_or(Error::Unsupported)?;
        let result = self.wait_armed_reply(strict, &mut ticket, index, end);
        let cancel = ticket.cancel();
        self.rx.ticket = Some(ticket);
        result.and_then(|reply| {
            cancel?;
            Ok(reply)
        })
    }

    fn wait_armed_reply(
        &mut self,
        strict: &StrictSession,
        ticket: &mut crate::strict::Ticket<proto::wake::Session>,
        index: u32,
        end: Instant,
    ) -> Result<Rep> {
        // Unmask before asking for a pulse. The second force covers an arm
        // that landed after RP1 had already published the reply.
        strict.listener.intake()?;
        self.check_requested_peer(strict)?;
        if Instant::now() < end {
            if let Some(reply) = self.take_requested_reply(index)? {
                return Ok(reply);
            }
            self.rx
                .session
                .link
                .write(self.rx.ctrl_offset + RPC_WAIT_HEAD, index.wrapping_add(1));
            ticket.arm()?;
            if let Some(doorbell) = &self.tx.doorbell {
                self.tx.force_request(doorbell)?;
            }
        }
        let mut pfd = [libc::pollfd {
            fd: strict.listener.as_fd().as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        }];
        // Match WaitSet's intake, peer checks and deadline statistics.
        loop {
            let dispatch = Instant::now() < end;
            if dispatch {
                self.rx.stats.sleeps += 1;
                poll_until(&mut pfd, Some(end))?;
            }
            let woke = strict.listener.intake();
            self.check_requested_peer(strict)?;
            let woke = woke?;
            let revents = pfd[0].revents;
            if revents & (libc::POLLERR | libc::POLLHUP | libc::POLLNVAL) != 0 {
                return Err(Error::setup(format!("rp1:irq poll flags {revents:#x}")));
            }
            let expired = Instant::now() >= end;
            self.rx.stats.woken += u64::from(dispatch && woke);
            self.rx.stats.deadline_checks += u64::from(expired);
            // Once armed, take the count for the matching reply before
            // returning. A head alone can precede the IRQ it requested.
            let notified = woke && ticket.acknowledge()?;
            if (notified || expired)
                && let Some(reply) = self.take_requested_reply(index)?
            {
                self.rx.stats.late += u64::from(expired && !notified);
                return Ok(reply);
            }
            if expired {
                return Err(Error::Timeout);
            }
        }
    }

    fn check_requested_peer(&self, strict: &StrictSession) -> Result<()> {
        let rx = &self.rx;
        let session = &rx.session;
        strict.check_rpc_link(&session.link)?;
        session.check_header()?;
        rx.session.check_peer(0)?;
        if rx.peer.core != 0 {
            rx.session.check_peer(rx.peer.core)?;
        }
        Ok(())
    }

    fn take_requested_reply(&mut self, index: u32) -> Result<Option<Rep>> {
        loop {
            let ready = self.rx.available() != 0;
            if !ready {
                return Ok(None);
            }
            let tail = self.rx.tail();
            let reply = self.rx.inner.pop()?;
            self.rx.session.check_header()?;
            if tail == index || reply.is_none() {
                return Ok(reply);
            }
        }
    }

    fn wait_reply(&mut self, index: u32, end: Instant) -> Result<Rep> {
        loop {
            match if self.rx.ticket.is_some() {
                self.rx.wait_rpc(end)
            } else {
                self.rx.wait(end)
            } {
                Err(Error::Timeout) if self.rx.session.strict.is_none() => {
                    self.rx.peer.check()?;
                    return Err(Error::Timeout);
                }
                result => result?,
            }
            let tail = self.rx.tail();
            let reply = if self.rx.ticket.is_some() {
                // The synchronous IRQ wait just checked link, terminal and generation.
                let reply = self.rx.inner.pop()?;
                self.rx.session.check_header()?;
                reply
            } else {
                self.rx.try_recv()?
            };
            if let Some(reply) = reply
                && tail == index
            {
                return Ok(reply);
            }
        }
    }
}

#[cfg(test)]
mod tests;
