use crate::{
    Error, Firmware, PeerDown, Result, RingRx, Session, Waitable, notify::listener::poll_until,
    strict::StrictSession,
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
    armed: Option<Armed>,
    cores: u8,
    tickets_armed: bool,
    cleanup_checked: bool,
    rpc: bool,
    mailbox_armed: bool,
    #[cfg(test)]
    before_arm: Option<Box<dyn FnOnce()>>,
}

#[derive(Clone, Copy)]
struct Armed {
    end: Option<Instant>,
    report_lost: bool,
}

impl Firmware {
    /// Borrow one or more IRQ receivers from this v3 session. At most one wait
    /// may be armed at a time. Drop the set before receiving from its endpoints.
    pub fn wait_set<'a, 'endpoints>(
        &self,
        endpoints: &'a mut [&'endpoints mut dyn Waitable],
    ) -> Result<WaitSet<'a, 'endpoints>> {
        WaitSet::new(self.session()?, endpoints)
    }

    /// Block until a receiver is ready or the absolute deadline is reached.
    /// On v3 IRQ, `NotifyLost` leaves the data available to receive.
    pub fn wait_any(&self, endpoints: &mut [&mut dyn Waitable], end: Instant) -> Result<usize> {
        if self.strict.is_some() {
            self.wait_set(endpoints)?.wait(end)
        } else {
            self.wait_any_legacy(endpoints, Some(end))?;
            endpoints
                .iter()
                .position(|ep| ep.available() != 0)
                .ok_or(Error::Timeout)
        }
    }
}

impl<T: Message> RingRx<T> {
    /// Wait for data until the absolute monotonic deadline. The default IRQ wait
    /// blocks on the session listener. A declared poll ring uses periodic checks.
    /// `NotifyLost` may reflect a late count and leaves the data in the ring.
    pub fn wait(&mut self, end: Instant) -> Result<()> {
        if self.ticket.is_none() {
            return self
                .wait_until(Some(end))?
                .then_some(())
                .ok_or(Error::Timeout);
        }
        self.wait_irq(Some(end), true)
    }

    pub(crate) fn wait_irq(&mut self, end: Option<Instant>, report_lost: bool) -> Result<()> {
        self.wait_irq_inner(end, report_lost, false)
    }

    pub(crate) fn wait_rpc(&mut self, end: Instant) -> Result<()> {
        self.wait_irq_inner(Some(end), false, true)
    }

    fn wait_irq_inner(&mut self, end: Option<Instant>, report_lost: bool, rpc: bool) -> Result<()> {
        let session = self.session.clone();
        let mut endpoints = [self as &mut dyn Waitable];
        let mut wait = WaitSet::new(session, &mut endpoints)?;
        wait.rpc = rpc;
        wait.wait_inner(end, report_lost)?;
        Ok(())
    }
}

impl<'a, 'endpoints> WaitSet<'a, 'endpoints> {
    fn new(
        session: Rc<Session>,
        endpoints: &'a mut [&'endpoints mut dyn Waitable],
    ) -> Result<Self> {
        session.strict.as_ref().ok_or(Error::Unsupported)?;
        if !endpoints
            .iter()
            .all(|ep| ep.strict_receiver(Rc::as_ptr(&session).cast()))
            || endpoints.is_empty()
        {
            return Err(Error::Unsupported);
        }
        let cores = endpoints.iter().fold(1, |mask, ep| mask | (1 << ep.core()));
        Ok(Self {
            session,
            endpoints,
            armed: None,
            cores,
            tickets_armed: false,
            cleanup_checked: false,
            rpc: false,
            mailbox_armed: false,
            #[cfg(test)]
            before_arm: None,
        })
    }

    fn strict(&self) -> &StrictSession {
        self.session
            .strict
            .as_ref()
            .expect("WaitSet requires a v3 session")
    }

    /// The borrowed link fd for level-triggered polling. Only dispatch consumes it.
    pub fn as_fd(&self) -> BorrowedFd<'_> {
        self.strict().listener.as_fd()
    }

    /// Start a fresh wait. A ready return requires no external poll.
    pub fn prepare(&mut self, end: Instant) -> Result<WaitState> {
        self.prepare_wait(Some(end), true)
    }

    fn prepare_wait(&mut self, end: Option<Instant>, report_lost: bool) -> Result<WaitState> {
        if self.strict().busy.replace(true) {
            return Err(Error::Busy);
        }
        self.armed = Some(Armed { end, report_lost });
        let result = self.prepare_inner(end);
        self.finish(result)
    }

    fn prepare_inner(&mut self, end: Option<Instant>) -> Result<WaitState> {
        self.intake(0)?;
        // The local DDR head may already hold the reply.
        if let Some(index) = self.endpoints.iter().position(|ep| ep.available() != 0) {
            self.check()?;
            return Ok(WaitState::Ready(index));
        }
        if end.is_some_and(|end| Instant::now() >= end) {
            return self.scan()?.map(WaitState::Ready).ok_or(Error::Timeout);
        }
        self.session.check_header()?;
        #[cfg(test)]
        if let Some(hook) = self.before_arm.take() {
            hook();
        }
        self.tickets_armed = true;
        if !self.rpc {
            self.arm_mailbox()?;
        }
        for ep in self.endpoints.iter_mut() {
            ep.arm()?;
        }
        if self.rpc {
            // A solicited reply can arrive while we arm its receive ticket and
            // verify liveness. A ready reply needs no terminal mailbox arm.
            if let Some(index) = self.scan()? {
                return Ok(WaitState::Ready(index));
            }
            self.cleanup_checked = false;
            self.arm_mailbox()?;
        }
        match self.scan()? {
            Some(index) => Ok(WaitState::Ready(index)),
            None => Ok(WaitState::Armed),
        }
    }

    fn arm_mailbox(&mut self) -> Result<()> {
        self.strict().mailbox.borrow_mut().arm()?;
        self.mailbox_armed = true;
        Ok(())
    }

    /// Take a queued count, re-enable and scan every condition. Pass the link fd's
    /// poll flags, or zero if only another fd or the deadline fired. After `Armed`,
    /// poll again with the same deadline without rearming.
    pub fn dispatch(&mut self, revents: libc::c_short) -> Result<WaitState> {
        let armed = self.armed.ok_or(Error::Unsupported)?;
        let result = self.dispatch_inner(revents, armed);
        self.finish(result)
    }

    fn dispatch_inner(&mut self, revents: libc::c_short, armed: Armed) -> Result<WaitState> {
        let woke = self.intake(revents)?;
        let expired = armed.end.is_some_and(|end| Instant::now() >= end);
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
                if armed.report_lost {
                    Err(Error::NotifyLost)
                } else {
                    Ok(WaitState::Ready(index))
                }
            }
            Some(index) => Ok(WaitState::Ready(index)),
            None if expired => Err(Error::Timeout),
            None => Ok(WaitState::Armed),
        }
    }

    /// Block with ppoll and retain the same deadline across spurious wakes and signals.
    pub fn wait(&mut self, end: Instant) -> Result<usize> {
        self.wait_inner(Some(end), true)
    }

    pub(crate) fn wait_inner(&mut self, end: Option<Instant>, report_lost: bool) -> Result<usize> {
        let mut state = self.prepare_wait(end, report_lost)?;
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
                    #[cfg(test)]
                    tests::run_before_poll();
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
        let listener = &self.strict().listener;
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
        let strict = self.strict();
        strict.check_link(&self.session.link)?;
        self.session.check_header()?;
        for core in 0..2 {
            if self.cores & (1 << core) != 0 {
                match self.session.check_peer(core) {
                    Err(e @ Error::PeerDown(PeerDown::Terminal { .. })) => {
                        if self.mailbox_armed {
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
        self.cleanup_checked = true;
        let mut ready = None;
        for (index, ep) in self.endpoints.iter_mut().enumerate() {
            if ep.available() != 0 {
                rp1_proto::barrier();
                ready.get_or_insert(index);
                // ACK only a ticket with an observed, sticky readiness condition.
                // Another endpoint may have caused the wake on this shared vector.
                if self.tickets_armed {
                    if self.rpc {
                        ep.acknowledge_and_cancel()?;
                    } else {
                        ep.acknowledge()?;
                    }
                }
            }
        }
        Ok(ready)
    }

    fn finish(&mut self, result: Result<WaitState>) -> Result<WaitState> {
        if matches!(result, Ok(WaitState::Armed)) {
            self.cleanup_checked = false;
            return result;
        }
        let cleanup = self.cancel();
        result.and_then(|value| cleanup.map(|()| value))
    }

    /// Cancel this wait, leaving the listener enabled for the next wait.
    pub fn cancel(&mut self) -> Result<()> {
        if self.armed.take().is_none() {
            return Ok(());
        }
        self.strict().busy.set(false);
        // An immediate ready result held the listener but did not arm anything.
        if !self.tickets_armed {
            self.cleanup_checked = false;
            return Ok(());
        }
        self.tickets_armed = false;
        let mailbox_armed = std::mem::take(&mut self.mailbox_armed);
        if !std::mem::take(&mut self.cleanup_checked) {
            self.session.check()?;
        }
        for ep in self.endpoints.iter_mut() {
            ep.cancel()?;
        }
        if mailbox_armed {
            self.strict().mailbox.borrow_mut().cancel()?;
        }
        Ok(())
    }
}

impl Drop for WaitSet<'_, '_> {
    fn drop(&mut self) {
        let _ = self.cancel();
    }
}

#[cfg(test)]
mod tests;
