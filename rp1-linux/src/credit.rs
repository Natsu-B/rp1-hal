use crate::{
    Doorbell, Error, Irq, Message, POLL, Result, RingTx, Session, WaitSet, WaitStats, Waitable,
    remaining, sealed, shm,
    strict::{self, Ticket},
};
use rp1_proto::wake;
use std::{rc::Rc, time::Instant};

pub(crate) fn enabled(session: &Session) -> bool {
    session
        .strict
        .as_ref()
        .is_some_and(|strict| strict.credit_wake)
}

pub(crate) struct Publisher {
    ticket: strict::Publisher,
    doorbell: Doorbell,
}

impl Publisher {
    pub(crate) fn new(
        session: &Session,
        words: shm::wake::Words<wake::Session>,
        doorbell: Doorbell,
    ) -> Result<Self> {
        Ok(Self {
            ticket: strict::ring_publisher(session, words)?,
            doorbell,
        })
    }

    pub(crate) fn publish<T: Message>(&mut self, ring: &shm::RingRx<T>) -> Result<()> {
        if ring.publish_credit(&mut self.ticket)? {
            self.doorbell.ring_flushed();
        }
        Ok(())
    }

    pub(crate) fn reopen<T: Message>(&mut self, ring: &shm::RingRx<T>) -> Result<()> {
        if self.ticket.needs_recovery_kick()? {
            self.doorbell.ring_flushed();
            Ok(())
        } else {
            self.publish(ring)
        }
    }
}

impl<T: Message> RingTx<T> {
    fn credit_ticket(&mut self) -> Result<&mut Ticket<wake::Session>> {
        self.credit.as_mut().ok_or(Error::Unsupported)
    }

    /// Wait for a writable slot until the absolute monotonic deadline.
    /// A strict reject ring sleeps on the link listener. An explicit poll ring
    /// checks at intervals. Overwrite rings always have space.
    pub fn wait_writable(&mut self, end: Instant) -> Result<()> {
        if self.credit.is_some() {
            let session = self.session.clone();
            let mut endpoints = [self as &mut dyn Waitable];
            return WaitSet::new(session, &mut endpoints)?.wait(end).map(|_| ());
        }

        loop {
            sealed::Waitable::check(self)?;
            if self.inner.writable() != 0 {
                return Ok(());
            }
            if !self.poll {
                return Err(Error::Unsupported);
            }
            let delay = remaining(Some(end), POLL);
            if delay.is_zero() {
                return Err(Error::Timeout);
            }
            std::thread::sleep(delay);
        }
    }

    /// Publish one message, waiting for space until the original deadline.
    /// If a wake is lost, the message is left with the caller.
    pub fn send_until(&mut self, value: &T, end: Instant) -> Result<()> {
        self.wait_writable(end)?;
        if Instant::now() >= end {
            return Err(Error::Timeout);
        }
        self.send(value)
    }

    /// Statistics for waits for writable space.
    pub fn wait_stats(&self) -> WaitStats {
        self.stats
    }
}

impl<T: Message> Waitable for RingTx<T> {}
impl<T: Message> sealed::Waitable for RingTx<T> {
    fn check(&self) -> Result<()> {
        self.session.check()?;
        self.session.check_peer(self.core)
    }

    fn parts(&mut self) -> Option<(&mut dyn shm::Waiter, Option<&mut Irq>)> {
        None
    }

    fn strict_endpoint(&self, session: *const ()) -> bool {
        self.credit.is_some() && Rc::as_ptr(&self.session).cast() == session
    }

    fn core(&self) -> u8 {
        self.core
    }

    fn available(&self) -> u32 {
        self.inner.free()
    }

    fn arm(&mut self) -> Result<()> {
        Ok(self.credit_ticket()?.arm()?)
    }

    fn acknowledge(&mut self) -> Result<()> {
        self.credit_ticket()?.acknowledge()?;
        Ok(())
    }

    fn acknowledge_and_cancel(&mut self) -> Result<()> {
        Ok(self.credit_ticket()?.acknowledge_and_cancel()?)
    }

    fn cancel(&mut self) -> Result<()> {
        self.credit_ticket()?.cancel()?;
        Ok(())
    }

    fn stats(&mut self) -> &mut WaitStats {
        &mut self.stats
    }
}
