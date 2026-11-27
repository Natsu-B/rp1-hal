//! v3 notification words. The waiter owns arm and ACK. The publisher owns seq.

use super::access::{ld, st};
use crate::{
    barrier,
    wake::{BootEpoch, Session, SessionError, mailbox},
};
use core::marker::PhantomData;

/// Offsets into a ring's data and control parts.
pub const RX_WAIT: usize = 64; // ctrl, reused from v2
pub const RX_SEQ: usize = 68; // data
pub const RX_ACK: usize = 68; // ctrl
pub const CREDIT_WAIT: usize = 72; // data
pub const CREDIT_SEQ: usize = 72; // ctrl
pub const CREDIT_ACK: usize = 76; // data
/// Reply head requested by the current RPC receive ticket, in the control part.
pub const RPC_WAIT_HEAD: usize = 76;

#[derive(Clone, Copy)]
pub struct Words<S> {
    arm: *mut u8,
    seq: *mut u8,
    ack: *mut u8,
    scope: PhantomData<S>,
}

impl Words<Session> {
    /// # Safety
    /// Each base is the correctly aligned, mapped ring part (see [`super::CONTROL_SIZE`]).
    pub unsafe fn rx(data: *mut u8, ctrl: *mut u8) -> Self {
        unsafe {
            Self {
                arm: ctrl.add(RX_WAIT),
                seq: data.add(RX_SEQ),
                ack: ctrl.add(RX_ACK),
                scope: PhantomData,
            }
        }
    }

    /// # Safety
    /// As for [`Self::rx`], on a reject ring.
    pub unsafe fn credit(data: *mut u8, ctrl: *mut u8) -> Self {
        unsafe { Self::credit_words(data, ctrl) }
    }

    pub(super) unsafe fn reset_credit(data: *mut u8, ctrl: *mut u8) {
        unsafe { Self::credit_words(data, ctrl) }.reset();
    }

    unsafe fn credit_words(data: *mut u8, ctrl: *mut u8) -> Self {
        unsafe {
            Self {
                arm: data.add(CREDIT_WAIT),
                seq: ctrl.add(CREDIT_SEQ),
                ack: data.add(CREDIT_ACK),
                scope: PhantomData,
            }
        }
    }
}

impl Words<BootEpoch> {
    /// # Safety
    /// `base` is the correctly aligned mapping of the link page with a v3 mailbox.
    pub unsafe fn link(base: *mut u8) -> Self {
        unsafe {
            Self {
                arm: base.add(mailbox::WAIT),
                seq: base.add(mailbox::SEQ),
                ack: base.add(mailbox::ACK),
                scope: PhantomData,
            }
        }
    }
}

impl<S: Copy + Eq> Words<S> {
    /// # Safety
    /// The mappings stay valid, with exclusive local ownership of arm and ACK until this
    /// handle is released. Mapping reuse requires revoking old writers and draining posted
    /// accesses before a new session is made visible. No v2 wait may write these words.
    /// `live` must report the current boot or session, or an error if the link is unavailable.
    pub unsafe fn waiter<F>(self, scope: S, live: F) -> Result<Waiter<S, F>, SessionError>
    where
        F: Fn() -> Result<S, SessionError>,
    {
        check(scope, live())?;
        Ok(Waiter {
            words: self,
            scope,
            live,
            arm: load(self.arm),
            acked: load(self.ack),
            armed: false,
        })
    }

    /// # Safety
    /// As for [`Self::waiter`], with exclusive local ownership of seq.
    pub unsafe fn publisher<F>(self, scope: S, live: F) -> Result<Publisher<S, F>, SessionError>
    where
        F: Fn() -> Result<S, SessionError>,
    {
        check(scope, live())?;
        Ok(Publisher {
            words: self,
            scope,
            live,
            seq: load(self.seq),
        })
    }
}

impl Words<Session> {
    // The initializing proc0 owns both endpoints while the SHM generation is odd.
    pub(super) fn reset(self) {
        commit(self.arm, 0);
        commit(self.seq, 0);
        commit(self.ack, 0);
    }
}

pub struct Waiter<S, F> {
    words: Words<S>,
    scope: S,
    live: F,
    arm: u32,
    acked: u32,
    armed: bool,
}

impl<S: Copy + Eq, F: Fn() -> Result<S, SessionError>> Waiter<S, F> {
    /// Arm a fresh ticket and flush it. Reopen advances even if the previous owner
    /// left an odd arm behind. The caller then acquires and rechecks its ready condition.
    pub fn arm(&mut self) -> Result<(), SessionError> {
        check(self.scope, (self.live)())?;
        let seq = load(self.words.seq);
        self.arm = next_ticket(self.arm, seq, self.acked, LAST_TICKET);
        commit(self.words.arm, self.arm);
        self.armed = true;
        Ok(())
    }

    /// Acquire pending work for the current ticket. The caller rechecks all readiness
    /// conditions after re-enabling a shared vector, including when there is no pending seq.
    pub fn pending(&self) -> Result<bool, SessionError> {
        check(self.scope, (self.live)())?;
        Ok(self.armed && self.acked != self.arm && load(self.words.seq) == self.arm)
    }

    /// ACK the current pending ticket so a takeover publisher can suppress a recovery kick.
    pub fn acknowledge(&mut self) -> Result<bool, SessionError> {
        if !self.pending()? {
            return Ok(false);
        }
        commit(self.words.ack, self.arm);
        self.acked = self.arm;
        Ok(true)
    }

    /// ACK an observed ready endpoint and immediately retire its ticket. Arm and
    /// ACK share the control window, so reading back the final arm store completes
    /// both stores. Only call after observing the endpoint's ready condition.
    pub fn acknowledge_and_cancel(&mut self) -> Result<(), SessionError> {
        let pending = self.pending()?;
        if self.armed {
            if pending {
                barrier();
                unsafe { st(self.words.ack, 0, self.arm.to_le()) };
                self.acked = self.arm;
            }
            self.cancel_inner();
        }
        Ok(())
    }

    /// Cancel the current ticket. A newly reopened or already canceled waiter is idle.
    pub fn cancel(&mut self) -> Result<bool, SessionError> {
        check(self.scope, (self.live)())?;
        if !self.armed {
            return Ok(false);
        }
        self.cancel_inner();
        Ok(true)
    }

    fn cancel_inner(&mut self) {
        self.arm += 1;
        commit(self.words.arm, self.arm);
        self.armed = false;
    }
}

// All-ones is reserved so that even the last ticket has a nonzero cancel word.
pub(super) const LAST_TICKET: u32 = u32::MAX - 2;

pub(super) fn next_ticket(previous: u32, seq: u32, acked: u32, last: u32) -> u32 {
    let advance = |word| if word >= last { 1 } else { (word + 1) | 1 };
    let mut ticket = advance(previous);
    // Either retained value could match again after wrap. Skip both before committing arm.
    while ticket == seq || ticket == acked {
        ticket = advance(ticket);
    }
    ticket
}

pub struct Publisher<S, F> {
    words: Words<S>,
    scope: S,
    live: F,
    seq: u32,
}

impl<S: Copy + Eq, F: Fn() -> Result<S, SessionError>> Publisher<S, F> {
    /// Complete the previous data, tail or terminal commit with a readback through its
    /// window. Publish the current odd ticket and flush it through the seq's window.
    /// A true result asks the vector owner to signal. The seq alone proves no delivery.
    ///
    /// The interval from sampling arm to committing seq must span fewer than
    /// 2^31 - 3 fresh arms on these words, so arm cannot be reused during that interval.
    pub fn publish(&mut self, read_commit: impl FnOnce()) -> Result<bool, SessionError> {
        self.publish_inner(read_commit, None::<fn() -> bool>)
    }

    /// Qualify the sampled ticket before committing its seq.
    pub fn publish_if(
        &mut self,
        read_commit: impl FnOnce(),
        ready: impl FnOnce() -> bool,
    ) -> Result<bool, SessionError> {
        self.publish_inner(read_commit, Some(ready))
    }

    fn publish_inner(
        &mut self,
        read_commit: impl FnOnce(),
        ready: Option<impl FnOnce() -> bool>,
    ) -> Result<bool, SessionError> {
        check(self.scope, (self.live)())?;
        barrier();
        read_commit();
        barrier();
        let arm = load(self.words.arm);
        if arm & 1 == 0
            || self.seq == arm
            || ready.is_some_and(|ready| !ready() || load(self.words.arm) != arm)
        {
            return Ok(false);
        }
        commit(self.words.seq, arm);
        self.seq = arm;
        Ok(true)
    }

    /// Use after acquiring exclusive same-session ownership and draining the old publisher.
    /// If work is still ready and the committed ticket is still armed and unacknowledged,
    /// issue one recovery kick. Check this once per takeover, before normal publication.
    pub fn needs_recovery_kick(&self) -> Result<bool, SessionError> {
        check(self.scope, (self.live)())?;
        let arm = load(self.words.arm);
        Ok(arm & 1 != 0 && self.seq == arm && load(self.words.ack) != arm)
    }
}

fn check<S: Eq>(scope: S, live: Result<S, SessionError>) -> Result<(), SessionError> {
    // Validate before touching ticket/data words. The ownership contract excludes
    // invalidation during an in-flight operation from reusing the mapping until it drains.
    // An odd generation (NotReady) also invalidates an already-attached handle.
    match live {
        Ok(current) if current == scope => Ok(()),
        _ => Err(SessionError::Restarted),
    }
}

fn load(word: *const u8) -> u32 {
    let value = u32::from_le(unsafe { ld(word, 0) });
    barrier();
    value
}

fn commit(word: *mut u8, value: u32) {
    barrier();
    unsafe { st(word, 0, value.to_le()) };
    let _ = load(word);
}
