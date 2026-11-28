use crate::{
    Error, Message, RingRx, RingTx,
    notify::{
        LiveSession, TicketDoorbell, TicketIrq, doorbell_events,
        sealed::{Notify, Token},
    },
};
use rp1_hal::timer::{self, Deadline, Timer, WaitContext};
use rp1_proto::{
    shm::wake::Waiter,
    wake::{Session, SessionError},
};

/// Failure of a deadline-bounded RP1 ring wait.
#[derive(Debug, Eq, PartialEq)]
pub enum WaitError {
    /// The ring or its session failed.
    Link(Error),
    /// The deadline elapsed or the thread's deadline alarm was unavailable.
    Timer(timer::Error),
    /// The condition became ready during the wait without a timely wake.
    NotifyLost,
    /// The endpoint has no ticket for this wait.
    Unsupported,
}

impl<T: Message> RingTx<T, TicketIrq> {
    /// Wait in thread mode for one writable slot, using the core's one-shot timer.
    pub fn wait_writable(
        &mut self,
        context: &mut WaitContext<'_>,
        end: Deadline,
    ) -> Result<(), WaitError> {
        self.notify.check(Token(())).map_err(WaitError::Link)?;
        if self.ring.writable() != 0 {
            return Ok(());
        }
        let ticket = self.notify.credit.as_mut().ok_or(WaitError::Unsupported)?;
        wait(context, end, ticket, || self.ring.free() != 0)
    }

    /// Send one message, waiting only until the given deadline for space.
    pub fn send_until(
        &mut self,
        value: &T,
        context: &mut WaitContext<'_>,
        end: Deadline,
    ) -> Result<(), WaitError> {
        self.wait_writable(context, end)?;
        if Timer::new().now() >= end.instant() {
            return Err(WaitError::Timer(timer::Error::Timeout));
        }
        self.send(value).map_err(WaitError::Link)
    }
}

impl<T: Message, const PIN: u8, const CORE: u8> RingRx<T, TicketDoorbell<PIN, CORE>> {
    /// Wait in thread mode for a message using the core's one-shot timer.
    pub fn wait(&mut self, context: &mut WaitContext<'_>, end: Deadline) -> Result<(), WaitError> {
        let ticket = self.wake.ticket.as_mut().ok_or(WaitError::Unsupported)?;
        wait(context, end, ticket, || self.ring.available() != 0)
    }
}

fn wait(
    context: &mut WaitContext<'_>,
    end: Deadline,
    ticket: &mut Waiter<Session, LiveSession>,
    ready: impl Fn() -> bool,
) -> Result<(), WaitError> {
    let mut checked_events = doorbell_events();
    let mut prepared = false;
    let result = context.wait_until_with_final(end, |final_probe| {
        let scanned = doorbell_events();
        match probe(ticket, &ready, &mut prepared, final_probe, checked_events) {
            Ok(false) => {
                checked_events = scanned;
                None
            }
            Ok(true) => Some(Ok(())),
            Err(error) => Some(Err(error)),
        }
    });
    let cleanup = if prepared {
        ticket.cancel().map_err(reset).map(|_| ())
    } else {
        Ok(())
    };
    result.map_err(WaitError::Timer)?.and(cleanup)
}

fn probe(
    ticket: &mut Waiter<Session, LiveSession>,
    ready: &impl Fn() -> bool,
    prepared: &mut bool,
    final_probe: bool,
    checked_events: u32,
) -> Result<bool, WaitError> {
    ticket.check_live().map_err(reset)?;
    if !*prepared {
        if ready() {
            return Ok(true);
        }
        ticket.arm().map_err(reset)?;
        *prepared = true;
    }
    if !ready() {
        return Ok(false);
    }
    rp1_proto::barrier();
    ticket.acknowledge_and_cancel().map_err(reset)?;
    if final_probe && doorbell_events() == checked_events {
        return Err(WaitError::NotifyLost);
    }
    Ok(true)
}

fn reset(_: SessionError) -> WaitError {
    WaitError::Link(Error::Reset)
}

mod sealed {
    pub trait Ready {
        fn ready(&self) -> Result<bool, crate::Error>;
    }
}

/// A strict channel or RPC server whose service returned idle.
pub trait Waitable: sealed::Ready {}

impl<Tx: Message, Rx: Message, const PIN: u8, const CORE: u8> Waitable
    for crate::Channel<Tx, Rx, TicketIrq, TicketDoorbell<PIN, CORE>>
{
}

impl<Tx: Message, Rx: Message, const PIN: u8, const CORE: u8> sealed::Ready
    for crate::Channel<Tx, Rx, TicketIrq, TicketDoorbell<PIN, CORE>>
{
    fn ready(&self) -> Result<bool, Error> {
        service_ready(&self.tx, &self.rx)
    }
}

impl<Req: Message, Rep: Message, const PIN: u8, const CORE: u8> Waitable
    for crate::Server<Req, Rep, TicketIrq, TicketDoorbell<PIN, CORE>>
{
}

impl<Req: Message, Rep: Message, const PIN: u8, const CORE: u8> sealed::Ready
    for crate::Server<Req, Rep, TicketIrq, TicketDoorbell<PIN, CORE>>
{
    fn ready(&self) -> Result<bool, Error> {
        if self.failed {
            return Err(Error::Reset);
        }
        if self.reply_requested && self.tx.notify.reply_wake_ready(&self.tx.ring)? {
            return Ok(true);
        }
        service_ready(&self.tx, &self.rx)
    }
}

fn service_ready<Tx: Message, Rx: Message, const PIN: u8, const CORE: u8>(
    tx: &RingTx<Tx, TicketIrq>,
    rx: &RingRx<Rx, TicketDoorbell<PIN, CORE>>,
) -> Result<bool, Error> {
    use crate::notify::sealed::Wake;
    tx.notify.check(Token(()))?;
    rx.wake.check(Token(()))?;
    Ok(tx.ring.can_write() && (tx.notify.waiting_for_credit() || rx.available() != 0))
}

/// Wait for one of the strict services on this core. Run a bounded slice of
/// each service first and call only once they all return idle. Their tickets
/// stay armed for the next slice, which observes and retires them.
pub fn wait_any(
    context: &mut WaitContext<'_>,
    end: Deadline,
    endpoints: &mut [&mut dyn Waitable],
) -> Result<usize, WaitError> {
    if endpoints.is_empty() {
        return Err(WaitError::Unsupported);
    }
    let mut scanned_events = doorbell_events();
    let mut entered = false;
    context
        .wait_until_with_final(end, |final_probe| {
            let current_events = doorbell_events();
            let mut ready = None;
            for (i, endpoint) in endpoints.iter().enumerate() {
                match endpoint.ready() {
                    Ok(true) => {
                        ready.get_or_insert(i);
                    }
                    Ok(false) => {}
                    Err(error) => return Some(Err(WaitError::Link(error))),
                }
            }
            let lost = entered && final_probe && doorbell_events() == scanned_events;
            entered = true;
            scanned_events = current_events;
            ready.map(|i| {
                if lost {
                    Err(WaitError::NotifyLost)
                } else {
                    Ok(i)
                }
            })
        })
        .map_err(WaitError::Timer)?
}
