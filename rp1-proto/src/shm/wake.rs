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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Ticket<S> {
    scope: S,
    arm: *const u8,
    word: u32,
}

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

impl<S> Words<S> {
    /// # Safety
    /// The mappings stay valid, with exclusive local ownership of arm and ACK until this
    /// handle is released. Mapping reuse requires revoking old writers and draining posted
    /// accesses before a new session is made visible. No v2 wait may write these words.
    pub unsafe fn waiter(self, scope: S) -> Waiter<S> {
        Waiter { words: self, scope }
    }

    /// # Safety
    /// As for [`Self::waiter`], with exclusive local ownership of seq.
    pub unsafe fn publisher(self, scope: S) -> Publisher<S> {
        Publisher { words: self, scope }
    }
}

pub struct Waiter<S> {
    words: Words<S>,
    scope: S,
}

impl<S: Copy + Eq> Waiter<S> {
    /// Arm a fresh ticket and flush it. Reopen advances even if the previous owner
    /// left an odd arm behind. The caller then acquires and rechecks its ready condition.
    pub fn arm(
        &mut self,
        mut live: impl FnMut() -> Result<S, SessionError>,
    ) -> Result<Ticket<S>, SessionError> {
        check(self.scope, live())?;
        let previous = load(self.words.arm);
        // Reserve the even cancel value as well. Reconnect before any word can repeat.
        let word = (previous | 1)
            .checked_add(if previous & 1 == 0 { 0 } else { 2 })
            .filter(|&word| word != u32::MAX)
            .ok_or(SessionError::Exhausted)?;
        check(self.scope, live())?;
        commit(self.words.arm, word);
        Ok(Ticket {
            scope: self.scope,
            arm: self.words.arm,
            word,
        })
    }

    /// Acquire only pending work for this ticket. The caller rechecks all readiness
    /// conditions after re-enabling a shared vector, including when there is no pending seq.
    pub fn pending(
        &self,
        ticket: Ticket<S>,
        mut live: impl FnMut() -> Result<S, SessionError>,
    ) -> Result<bool, SessionError> {
        check(self.scope, live())?;
        let pending = self.is_current(ticket)?
            && load(self.words.seq) == ticket.word
            && load(self.words.ack) != ticket.word;
        check(self.scope, live())?;
        Ok(pending)
    }

    /// ACK the observed ticket. A late ACK can neither consume a newer ticket nor disarm it.
    pub fn acknowledge(
        &mut self,
        ticket: Ticket<S>,
        mut live: impl FnMut() -> Result<S, SessionError>,
    ) -> Result<bool, SessionError> {
        if !self.pending(ticket, &mut live)? {
            return Ok(false);
        }
        commit(self.words.ack, ticket.word);
        Ok(true)
    }

    /// Cancel only the caller's ticket. Use the ticket with the endpoint that issued it.
    pub fn cancel(
        &mut self,
        ticket: Ticket<S>,
        mut live: impl FnMut() -> Result<S, SessionError>,
    ) -> Result<bool, SessionError> {
        check(self.scope, live())?;
        if !self.is_current(ticket)? {
            return Ok(false);
        }
        check(self.scope, live())?;
        commit(self.words.arm, ticket.word + 1);
        Ok(true)
    }

    fn is_current(&self, ticket: Ticket<S>) -> Result<bool, SessionError> {
        check(self.scope, Ok(ticket.scope))?;
        Ok(core::ptr::eq(self.words.arm, ticket.arm) && load(self.words.arm) == ticket.word)
    }
}

pub struct Publisher<S> {
    words: Words<S>,
    scope: S,
}

impl<S: Copy + Eq> Publisher<S> {
    /// Complete the previous data, tail or terminal commit with a readback through its
    /// window. Publish the current odd ticket and flush it through the seq's window.
    /// A true result asks the vector owner to signal. The seq alone proves no delivery.
    pub fn publish(
        &mut self,
        read_commit: impl FnOnce(),
        mut live: impl FnMut() -> Result<S, SessionError>,
    ) -> Result<bool, SessionError> {
        check(self.scope, live())?;
        barrier();
        read_commit();
        barrier();
        let arm = load(self.words.arm);
        if arm & 1 == 0 || load(self.words.seq) == arm {
            return Ok(false);
        }
        check(self.scope, live())?;
        commit(self.words.seq, arm);
        Ok(true)
    }
}

fn check<S: Eq>(scope: S, live: Result<S, SessionError>) -> Result<(), SessionError> {
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
