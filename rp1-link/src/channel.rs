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

    /// Take independent ownership of the two rings.
    pub fn split(self) -> (RingTx<Tx, N>, RingRx<Rx, W>) {
        (self.tx, self.rx)
    }
}
