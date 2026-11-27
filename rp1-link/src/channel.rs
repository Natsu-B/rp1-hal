use crate::{
    Error, Message, Progress, RingRx, RingTx,
    notify::{Notify, Poll, Wake},
};

/// A named pair of message rings.
pub struct Channel<Tx, Rx, N = Poll, W = Poll> {
    pub(crate) tx: RingTx<Tx, N>,
    pub(crate) rx: RingRx<Rx, W>,
}

impl<Tx: Message, Rx: Message, N: Notify, W: Wake> Channel<Tx, Rx, N, W> {
    /// Publish one message.
    pub fn send(&mut self, value: &Tx) -> Result<(), Error> {
        self.tx.send(value)
    }

    /// Publish a batch, returning the number written.
    pub fn send_slice(&mut self, values: &[Tx]) -> Result<usize, Error> {
        self.tx.send_slice(values)
    }

    /// Take the oldest message, if any.
    pub fn recv(&mut self) -> Result<Option<Rx>, Error> {
        self.rx.recv()
    }

    /// Take up to `out.len()` messages.
    pub fn recv_into(&mut self, out: &mut [Rx]) -> Result<usize, Error> {
        self.rx.recv_into(out)
    }

    /// Process a batch and re-arm notifications as in [`RingRx::drain`].
    pub fn drain(&mut self, max: u32, f: impl FnMut(Rx)) -> Result<Progress, Error> {
        self.rx.drain(max, f)
    }

    /// Reply to up to `max` requests, leaving a request queued when there is no reply space.
    ///
    /// With two reject-mode IRQ rings, a current Linux named-channel reader rings the
    /// doorbell after releasing reply slots. [`Progress::Idle`] then means
    /// it is safe to sleep for either a request or reply space. For polling or legacy
    /// peers, arrange a retry before sleeping indefinitely. A zero budget counts as one.
    pub fn respond(
        &mut self,
        max: u32,
        mut handle: impl FnMut(Rx) -> Tx,
    ) -> Result<Progress, Error> {
        crate::ring::window_ready()?;
        let budget = self.tx.ring.writable().min(max.max(1));
        if budget == 0 {
            return Ok(Progress::Idle);
        }
        self.rx
            .try_drain(budget, |request| self.tx.send(&handle(request)))
    }

    /// Take independent ownership of the two rings.
    pub fn split(self) -> (RingTx<Tx, N>, RingRx<Rx, W>) {
        (self.tx, self.rx)
    }
}
