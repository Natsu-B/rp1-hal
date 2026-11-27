use crate::{
    Error, Firmware, PeerDown, Result, RingRx, Session, Waitable, notify::listener::poll_until,
};
use rp1_proto::Message;
use std::{
    os::fd::{AsFd, AsRawFd, BorrowedFd},
    rc::Rc,
    time::Instant,
};

/// Result of preparing or dispatching a strict wait.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WaitState {
    /// All tickets are armed. Poll the fd until the original deadline.
    Armed,
    /// The first ready endpoint in input order. Other endpoints may be ready too.
    Ready(usize),
}

/// A v3 wait over one or more IRQ receivers sharing an exclusive UIO listener.
///
/// For external polling, call [`prepare`](Self::prepare), poll `POLLIN` on
/// [`as_fd`](Self::as_fd), and call [`dispatch`](Self::dispatch) after every poll
/// return, including timeout and readiness on unrelated fds. Use level-triggered
/// polling. Service any other ready fds after dispatch. Direct reads, irqcontrol
/// writes and use after fork violate the listener's ownership contract.
pub struct WaitSet<'a, 'endpoints> {
    session: Rc<Session>,
    endpoints: &'a mut [&'endpoints mut dyn Waitable],
    end: Option<Instant>,
}

impl Firmware {
    /// Borrow IRQ receivers from this v3 session. Keep at most one live wait set.
    pub fn wait_set<'a, 'endpoints>(
        &self,
        endpoints: &'a mut [&'endpoints mut dyn Waitable],
    ) -> Result<WaitSet<'a, 'endpoints>> {
        WaitSet::new(self.session()?, endpoints)
    }

    /// Block until an IRQ receiver is ready or the absolute deadline is reached.
    pub fn wait_any(&self, endpoints: &mut [&mut dyn Waitable], end: Instant) -> Result<usize> {
        self.wait_set(endpoints)?.wait(end)
    }
}

impl<T: Message> RingRx<T> {
    /// Wait for data until the absolute monotonic deadline. The default IRQ wait
    /// blocks on the session listener. A declared poll ring uses periodic checks.
    pub fn wait(&mut self, end: Instant) -> Result<()> {
        if self.ticket.is_none() {
            return self
                .wait_until(Some(end))?
                .then_some(())
                .ok_or(Error::Timeout);
        }
        let session = self.session.clone();
        let mut endpoints = [self as &mut dyn Waitable];
        WaitSet::new(session, &mut endpoints)?.wait(end)?;
        Ok(())
    }
}

impl<'a, 'endpoints> WaitSet<'a, 'endpoints> {
    fn new(
        session: Rc<Session>,
        endpoints: &'a mut [&'endpoints mut dyn Waitable],
    ) -> Result<Self> {
        let strict = session.strict.as_ref().ok_or(Error::Unsupported)?;
        if !endpoints
            .iter()
            .all(|ep| ep.strict_receiver(Rc::as_ptr(&session).cast()))
            || endpoints.is_empty()
        {
            return Err(Error::Unsupported);
        }
        if strict.busy.replace(true) {
            return Err(Error::Busy);
        }
        Ok(Self {
            session,
            endpoints,
            end: None,
        })
    }

    /// The borrowed link fd for level-triggered polling. Only dispatch consumes it.
    pub fn as_fd(&self) -> BorrowedFd<'_> {
        self.session.strict.as_ref().unwrap().listener.as_fd()
    }

    /// Start a fresh wait. A ready return requires no external poll.
    pub fn prepare(&mut self, end: Instant) -> Result<WaitState> {
        if self.end.is_some() {
            return Err(Error::Busy);
        }
        let result = self.prepare_inner(end);
        self.finish(result)
    }

    fn prepare_inner(&mut self, end: Instant) -> Result<WaitState> {
        self.intake(0)?;
        if let Some(index) = self.scan()? {
            return Ok(WaitState::Ready(index));
        }
        if Instant::now() >= end {
            return Err(Error::Timeout);
        }
        self.end = Some(end);
        self.session
            .strict
            .as_ref()
            .unwrap()
            .mailbox
            .borrow_mut()
            .arm()?;
        for ep in self.endpoints.iter_mut() {
            ep.arm()?;
        }
        match self.scan()? {
            Some(index) => Ok(WaitState::Ready(index)),
            None => Ok(WaitState::Armed),
        }
    }

    /// Take a queued count, re-enable and scan every condition. Pass the link fd's
    /// poll flags, or zero if only another fd or the deadline fired. After `Armed`,
    /// poll again with the same deadline without rearming.
    pub fn dispatch(&mut self, revents: libc::c_short) -> Result<WaitState> {
        let end = self.end.ok_or(Error::Unsupported)?;
        let result = self.dispatch_inner(revents, end);
        self.finish(result)
    }

    fn dispatch_inner(&mut self, revents: libc::c_short, end: Instant) -> Result<WaitState> {
        let woke = self.intake(revents)?;
        let expired = Instant::now() >= end;
        if woke || expired {
            for ep in self.endpoints.iter_mut() {
                let stats = ep.stats();
                stats.woken += u64::from(woke);
                stats.deadline_checks += u64::from(expired);
            }
        }
        match self.scan()? {
            Some(index) if expired && !woke => {
                self.endpoints[index].stats().late += 1;
                Err(Error::NotifyLost)
            }
            Some(index) => Ok(WaitState::Ready(index)),
            None if expired => Err(Error::Timeout),
            None => Ok(WaitState::Armed),
        }
    }

    /// Block with ppoll and retain the same deadline across spurious wakes and signals.
    pub fn wait(&mut self, end: Instant) -> Result<usize> {
        let mut state = self.prepare(end)?;
        loop {
            match state {
                WaitState::Ready(index) => return Ok(index),
                WaitState::Armed => {
                    let mut pfd = [libc::pollfd {
                        fd: self.as_fd().as_raw_fd(),
                        events: libc::POLLIN,
                        revents: 0,
                    }];
                    for ep in self.endpoints.iter_mut() {
                        ep.stats().sleeps += 1;
                    }
                    if let Err(error) = poll_until(&mut pfd, end) {
                        let _ = self.cancel();
                        return Err(error.into());
                    }
                    state = self.dispatch(pfd[0].revents)?;
                }
            }
        }
    }

    fn intake(&self, revents: libc::c_short) -> Result<bool> {
        let listener = &self.session.strict.as_ref().unwrap().listener;
        let result = listener.intake();
        if revents & (libc::POLLERR | libc::POLLHUP | libc::POLLNVAL) != 0 || result.is_err() {
            // A kernel error flag alone does not identify the RP1 session state.
            self.check()?;
            result?;
            return Err(Error::setup(format!("rp1:irq poll flags {revents:#x}")));
        }
        Ok(result?)
    }

    fn check(&self) -> Result<()> {
        let strict = self.session.strict.as_ref().unwrap();
        strict.check_link(&self.session.link)?;
        self.session.check_header()?;
        let mut cores = 1; // proc0 owns the shared window.
        for ep in self.endpoints.iter() {
            cores |= 1 << ep.core();
        }
        for core in 0..2 {
            if cores & (1 << core) != 0 {
                match self.session.check_peer(core) {
                    Err(e @ Error::PeerDown(PeerDown::Terminal { .. })) => {
                        if self.end.is_some() {
                            strict.mailbox.borrow_mut().acknowledge()?;
                        }
                        return Err(e);
                    }
                    result => result?,
                }
            }
        }
        Ok(())
    }

    fn scan(&mut self) -> Result<Option<usize>> {
        self.check()?;
        let mut ready = None;
        for (index, ep) in self.endpoints.iter_mut().enumerate() {
            if ep.available() != 0 {
                rp1_proto::barrier();
                ready.get_or_insert(index);
                // ACK only a ticket with an observed, sticky readiness condition.
                // Another endpoint may have caused the wake on this shared vector.
                if self.end.is_some() {
                    ep.acknowledge()?;
                }
            }
        }
        Ok(ready)
    }

    fn finish(&mut self, result: Result<WaitState>) -> Result<WaitState> {
        if matches!(result, Ok(WaitState::Armed)) {
            return result;
        }
        let cleanup = self.cancel();
        result.and_then(|value| cleanup.map(|()| value))
    }

    /// Cancel this wait, leaving the listener enabled for the next wait.
    pub fn cancel(&mut self) -> Result<()> {
        if self.end.take().is_none() {
            return Ok(());
        }
        let strict = self.session.strict.as_ref().unwrap();
        strict.check_link(&self.session.link)?;
        self.session.check_header()?;
        for ep in self.endpoints.iter_mut() {
            ep.cancel()?;
        }
        strict.mailbox.borrow_mut().cancel()?;
        Ok(())
    }
}

impl Drop for WaitSet<'_, '_> {
    fn drop(&mut self) {
        let _ = self.cancel();
        self.session.strict.as_ref().unwrap().busy.set(false);
    }
}

#[cfg(test)]
mod tests;
