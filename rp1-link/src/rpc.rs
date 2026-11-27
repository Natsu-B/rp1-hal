use crate::{
    Error, Message, Progress, RingRx, RingTx,
    notify::{Notify, Poll, Wake, sealed::Token},
};

/// The firmware endpoint of a named request/reply pair.
///
/// ```compile_fail
/// fn split(server: rp1_link::Server<u32, u32>) {
///     let _ = server.split();
/// }
/// ```
pub struct Server<Req, Rep, N = Poll, W = Poll> {
    tx: RingTx<Rep, N>,
    rx: RingRx<Req, W>,
    failed: bool,
    reply_requested: bool,
}

impl<Req: Message, Rep: Message, N: Notify, W: Wake> Server<Req, Rep, N, W> {
    pub(crate) fn new(tx: RingTx<Rep, N>, rx: RingRx<Req, W>) -> Self {
        Self {
            tx,
            rx,
            failed: false,
            reply_requested: false,
        }
    }

    /// Reply to up to `max` requests in order (`0` counts as one).
    ///
    /// Readiness and notification follow [`RingRx::drain`]. If publishing a reply fails, later
    /// calls return [`Error::Reset`] because request and reply indices can no longer match.
    pub fn serve(
        &mut self,
        max: u32,
        mut handle: impl FnMut(Req) -> Rep,
    ) -> Result<Progress, Error> {
        if self.failed {
            return Err(Error::Reset);
        }
        let progress = self.rx.try_drain_requests(max, |request, notify| {
            let result = self.tx.send_reply(&handle(request), notify);
            self.failed = result.is_err();
            self.reply_requested = notify == Some(true);
            result
        })?;
        // The host can arm after the reply. Its second force runs this check
        // even if there are no requests left to drain.
        if self.reply_requested {
            self.tx.notify.wake_requested(Token(()), &self.tx.ring);
        }
        Ok(progress)
    }
}

impl<Req, Rep, W> Server<Req, Rep, crate::notify::TicketIrq, W> {
    /// Number of failed attempts to pulse the owned host vector.
    pub fn pulse_failures(&self) -> u32 {
        self.tx.notify.pulse_failures()
    }

    /// Retry a delivery whose local pulse failed. Call from the vector owner
    /// while its SHM window is live, before blocking or after another event.
    pub fn retry_notification(&mut self) -> Result<bool, Error> {
        crate::ring::window_ready()?;
        Ok(self.tx.notify.retry())
    }
}
