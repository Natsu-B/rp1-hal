use crate::{
    Error, Firmware, Mapping, Message, Note, Result, RingRx, RingTx, ShmKind, ShmRegion,
    check_type, find_region, proto,
};
use rp1_proto::note::{RPC, rpc_name};
use std::{
    rc::Rc,
    time::{Duration, Instant},
};

impl Firmware {
    /// Open both rings of the named channel.
    pub fn channel<Tx: Message, Rx: Message>(&self, name: &str) -> Result<Channel<Tx, Rx>> {
        self.open_pair(name, false, Self::map)
            .map(|(channel, _)| channel)
    }

    /// Open the named RPC for one caller at a time.
    pub fn client<Req: Message, Rep: Message>(&self, name: &str) -> Result<Client<Req, Rep>> {
        let (Channel { tx, rx }, depth) = self.open_pair(name, true, Self::map)?;
        Ok(Client { tx, rx, depth })
    }

    fn open_pair<Tx: Message, Rx: Message>(
        &self,
        name: &str,
        rpc: bool,
        mut map: impl FnMut(&ShmRegion<'_>) -> Result<Rc<Mapping>>,
    ) -> Result<(Channel<Tx, Rx>, u32)> {
        let session = self.session()?;
        let (down, up) = pair::<Tx, Rx>(Note::parse(&self.note)?, name, rpc)?;
        let up_map = map(&up)?;
        let down_map = map(&down)?;
        let mut rx = Self::attach_rx(up, up_map, session.clone())?;
        // Attaching the writer notifies for any request that the previous owner left pending.
        let tx = self.attach_tx(down, down_map, session)?;
        if !rpc && up.notify_irq && !up.overwrite {
            rx.space_doorbell.clone_from(&tx.doorbell);
            // A previous reader may have exited between publishing tail and the IRQ.
            rx.notify_space();
        }
        Ok((Channel { tx, rx }, down.depth))
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
}

impl<Req: Message, Rep: Message> Client<Req, Rep> {
    /// Publish a request and wait for its reply. A zero timeout publishes once if there is
    /// room, then returns [`Error::Timeout`] immediately.
    ///
    /// A timeout leaves the request outstanding. Later calls discard its reply when it
    /// arrives. The peer may already have acted on the request.
    pub fn call(&mut self, request: &Req, timeout: Duration) -> Result<Rep> {
        let end = Instant::now()
            .checked_add(timeout)
            .ok_or(Error::InvalidDeadline)?;
        let index = self.publish(request, (!timeout.is_zero()).then_some(end))?;
        if timeout.is_zero() {
            return Err(Error::Timeout);
        }
        self.wait_reply(index, end)
    }

    /// Publish and await a reply under a single absolute deadline.
    pub fn call_until(&mut self, request: &Req, end: Instant) -> Result<Rep> {
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
        self.rx.session.check_header()?;
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
        self.tx.send(request)?;
        Ok(index)
    }

    fn wait_reply(&mut self, index: u32, end: Instant) -> Result<Rep> {
        loop {
            match self.rx.wait(end) {
                Err(Error::Timeout) if self.rx.session.strict.is_none() => {
                    self.rx.peer.check()?;
                    return Err(Error::Timeout);
                }
                result => result?,
            }
            let tail = self.rx.tail();
            if let Some(reply) = self.rx.try_recv()?
                && tail == index
            {
                return Ok(reply);
            }
        }
    }
}

#[cfg(test)]
mod tests;
