//! The sleep/notify handshake on a ring's `waiting` counter (protocol.md §2.3).
use super::access::{ld, st};
use super::{HEAD, RingRx, RingTx, WAITING};
use crate::{Error, Message, barrier};

/// Result of one [`RingRx::wait_with`] step.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Wait {
    /// Messages were available; the reader did not sleep.
    Ready,
    /// The reader slept and the sleeper reported a notification.
    Woken,
    /// The reader slept until the sleeper's bound without a notification.
    Timeout,
}

/// What a [`RingRx::drain`] call left behind.
#[must_use = "on `More` the ring is not armed: drain again (or raise the doorbell)"]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Progress {
    /// Empty and armed: the next message makes the writer notify.
    Idle,
    /// The budget was used up. Messages may remain and the ring is NOT armed.
    More,
}

impl<T: Message> RingTx<T> {
    /// Whether to notify the reader now; call after `push`/`push_slice` (once per batch is
    /// enough). Returns `true` at most once per reader sleep (an odd `waiting` value not
    /// notified before), so a reader that keeps up costs no notifications at all.
    ///
    /// `head` was a posted write into the reader's memory. A `dsb` only completes it at the
    /// local bridge, so first read `head` back through the same window: the non-posted read
    /// cannot pass the posted writes (slots, then `head`), hence when it returns the reader can
    /// see them. Only then is `waiting` sampled. Paired with [`RingRx::prepare_wait`] (arm,
    /// read back, re-check `head`) this closes the lost-wakeup race: either the reader's
    /// re-check sees the new `head`, or this sees the reader asleep. The readback also
    /// completes the payload, so the caller may notify without another drain. Costs one
    /// round trip per call; pure polling users need not call it.
    ///
    /// "Not notified before" compares with the last notified value only: after 2^31 reader
    /// sleeps without a notification in between, a sleep can reuse that value and is then
    /// not notified (the reader's bounded re-check picks the data up; notifications are hints).
    #[cfg_attr(feature = "isram", rp1_macros::isram)]
    pub fn wake_needed(&mut self) -> bool {
        barrier();
        let _ = unsafe { ld(self.data, HEAD) };
        barrier();
        let waiting = unsafe { ld(self.ctrl, WAITING) };
        if waiting & 1 == 0 || waiting == self.woken {
            return false;
        }
        self.woken = waiting;
        true
    }
    /// Whether the reader sleeps now: [`Self::wake_needed`] without the "already notified"
    /// memory, so it is `true` for every odd `waiting` value, also across the counter wrap.
    /// For a level-type notification (the doorbell force), where notifying again while it is
    /// set costs nothing. Same readback of `head` first, same cost.
    pub fn reader_asleep(&self) -> bool {
        barrier();
        let _ = unsafe { ld(self.data, HEAD) };
        barrier();
        let waiting = unsafe { ld(self.ctrl, WAITING) };
        waiting & 1 != 0
    }
}

impl<T: Message> RingRx<T> {
    /// Arm `waiting` (next odd value), then re-check `head` so a push racing with the arm is
    /// not missed. Returns `true` if the ring is still empty and the caller may sleep; then it
    /// must call [`RingRx::finish_wait`] when it wakes.
    ///
    /// `waiting` is read back through the same window before `head` is re-checked, so the
    /// arm has reached the writer's memory (see [`RingTx::wake_needed`] for the pairing).
    /// The notification itself must not overtake `head`: the writer calls
    /// [`RingTx::wake_needed`] (which drains `head`) before it notifies.
    pub fn prepare_wait(&mut self) -> bool {
        if self.waiting & 1 == 0 {
            self.waiting = self.waiting.wrapping_add(1);
            unsafe { st(self.ctrl, WAITING, self.waiting) };
        }
        barrier();
        let _ = unsafe { ld(self.ctrl, WAITING) };
        barrier();
        let empty = self.available() == 0;
        if !empty {
            self.finish_wait();
        }
        empty
    }
    /// Mark the reader awake (even `waiting`): the writer stops notifying until the next sleep.
    pub fn finish_wait(&mut self) {
        if self.waiting & 1 != 0 {
            self.waiting = self.waiting.wrapping_add(1);
            unsafe { st(self.ctrl, WAITING, self.waiting) };
        }
    }
    /// One wait step for an interrupt-notified ring: return [`Wait::Ready`] if messages are
    /// available, otherwise arm (`prepare_wait`), call `sleep` and disarm. `sleep` blocks for a
    /// bounded time and returns whether a notification arrived. A notification is a hint:
    /// the caller pops after every return and calls again when the ring is still empty, so a
    /// lost or merged notification costs at most one `sleep` bound (protocol.md §3).
    pub fn wait_with(&mut self, sleep: impl FnOnce() -> bool) -> Wait {
        if self.available() != 0 || !self.prepare_wait() {
            return Wait::Ready;
        }
        let woke = sleep();
        self.finish_wait();
        if woke { Wait::Woken } else { Wait::Timeout }
    }
    /// Mark the reader awake, pass up to `max` messages to `f` (`max = 0` counts as 1), then
    /// arm and re-check: [`Progress::Idle`] when the ring is empty and armed. When the budget
    /// runs out first, returns [`Progress::More`] without arming, so writers do not notify.
    /// For a handler of a level-type notification (protocol.md §3). On an error the ring
    /// is not armed.
    #[cfg_attr(feature = "isram", rp1_macros::isram)]
    pub fn drain(&mut self, max: u32, mut f: impl FnMut(T)) -> Result<Progress, Error> {
        self.finish_wait();
        let mut budget = max.max(1);
        loop {
            while budget > 0 {
                let Some(value) = self.pop()? else { break };
                f(value);
                budget -= 1;
            }
            if budget == 0 {
                return Ok(Progress::More);
            }
            if self.prepare_wait() {
                return Ok(Progress::Idle);
            }
        }
    }
}

/// A [`RingRx`] of any message type, for [`wait_any_with`].
pub trait Waiter {
    fn available(&self) -> u32;
    fn prepare_wait(&mut self) -> bool;
    fn finish_wait(&mut self);
}
impl<T: Message> Waiter for RingRx<T> {
    fn available(&self) -> u32 {
        RingRx::available(self)
    }
    fn prepare_wait(&mut self) -> bool {
        RingRx::prepare_wait(self)
    }
    fn finish_wait(&mut self) {
        RingRx::finish_wait(self)
    }
}
/// [`RingRx::wait_with`] for several rings whose writers share one notification: arm every
/// ring (each with its own read back and re-check), sleep once if all are still empty, then
/// disarm them all. [`Wait::Ready`] means some ring has messages; the caller pops every ring
/// after each return.
pub fn wait_any_with(rings: &mut [&mut dyn Waiter], sleep: impl FnOnce() -> bool) -> Wait {
    if rings.iter().any(|r| r.available() != 0) {
        return Wait::Ready;
    }
    for i in 0..rings.len() {
        if !rings[i].prepare_wait() {
            for r in &mut rings[..i] {
                r.finish_wait();
            }
            return Wait::Ready;
        }
    }
    let woke = sleep();
    for r in rings.iter_mut() {
        r.finish_wait();
    }
    if woke { Wait::Woken } else { Wait::Timeout }
}
