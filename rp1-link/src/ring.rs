//! Ring endpoints with their notification built in.
//!
//! Every call that would touch the shm window first reads the window status from the link
//! page (one local load) and returns [`Error::Reset`] without any window access once PERSTn has
//! taken the window away. This is best effort on proc0, whose PERSTn handler can preempt the
//! call after the check (as with [`crate::flush_window`]); the mapping belongs to the current
//! cold boot.
use crate::{
    generated,
    notify::{Doorbell, Notify, Poll, Wake, sealed::Token},
};
use rp1_proto::{
    Error, Message, link,
    shm::{self, Progress},
};

/// Err(Reset) unless the window is Ready.
#[inline(always)]
pub(crate) fn window_ready() -> Result<(), Error> {
    if generated::read_status() == link::SHM_READY {
        Ok(())
    } else {
        Err(Error::Reset)
    }
}

/// RP1 -> Linux ring: the single writer, which wakes Linux's reader through `N`.
pub struct RingTx<T, N = Poll> {
    pub(crate) ring: shm::RingTx<T>,
    pub(crate) notify: N,
}

impl<T: Message, N: Notify> RingTx<T, N> {
    pub(crate) fn new(ring: shm::RingTx<T>, notify: N) -> Self {
        Self { ring, notify }
    }
    /// Publish one message and wake the reader if it sleeps.
    #[inline(always)]
    pub fn send(&mut self, value: &T) -> Result<(), Error> {
        self.send_reply(value, None)
    }

    #[inline(always)]
    pub(crate) fn send_reply(
        &mut self,
        value: &T,
        requested_wake: Option<bool>,
    ) -> Result<(), Error> {
        window_ready()?;
        let sent = self.ring.push(value);
        match requested_wake {
            None => self.notify.wake(Token(()), &mut self.ring),
            Some(true) if sent.is_ok() => self.notify.wake_requested(Token(()), &self.ring),
            Some(_) => false,
        };
        sent
    }
    /// Publish a batch with one `head` store and at most one wake. Returns how many were
    /// written. It wakes also when the ring was full, so a caller retrying the rest of a batch
    /// keeps a reader that went to sleep meanwhile informed.
    #[inline(always)]
    pub fn send_slice(&mut self, values: &[T]) -> Result<usize, Error> {
        window_ready()?;
        let sent = self.ring.push_slice(values);
        self.notify.wake(Token(()), &mut self.ring);
        Ok(sent)
    }
    /// Publish one message without waking the reader (a caller batching by hand); follow with
    /// [`Self::flush`].
    #[inline(always)]
    pub fn push(&mut self, value: &T) -> Result<(), Error> {
        window_ready()?;
        self.ring.push(value)
    }
    /// [`Self::push`] for a batch: one `head` store. Returns how many were written.
    #[inline(always)]
    pub fn push_slice(&mut self, values: &[T]) -> Result<usize, Error> {
        window_ready()?;
        Ok(self.ring.push_slice(values))
    }
    /// Write up to `count` messages in place, `fill(k, slot)` for the k-th; see the
    /// [callback contract](shm::RingTx::push_with). Does not wake the reader.
    #[inline(always)]
    pub fn push_with(
        &mut self,
        count: usize,
        fill: impl FnMut(usize, &mut T),
    ) -> Result<usize, Error> {
        window_ready()?;
        Ok(self.ring.push_with(count, fill))
    }
    /// Wake the reader if it sleeps and was not woken for this sleep yet. Returns whether it
    /// did (always `false` for a `Poll` ring).
    #[inline(always)]
    pub fn flush(&mut self) -> Result<bool, Error> {
        window_ready()?;
        Ok(self.notify.wake(Token(()), &mut self.ring))
    }
    /// Free slots as last seen (a local load).
    pub fn free(&self) -> u32 {
        self.ring.free()
    }
    /// Slots the next publish can fill: all of them for an overwrite ring, none after a failed
    /// DMA copy. Rereads the local `tail` only when the cached one says the ring is full.
    pub fn writable(&mut self) -> u32 {
        self.ring.writable()
    }
    /// Messages published so far (the writer's `head`).
    pub fn head(&self) -> u32 {
        self.ring.head()
    }
    /// Messages an overwrite ring dropped (0 for reject rings).
    pub fn dropped(&self) -> u32 {
        self.ring.dropped()
    }
}

/// Linux -> RP1 ring: the single reader, which Linux wakes through `W`.
///
/// An endpoint with a [`Doorbell`](crate::notify::Doorbell) is armed when `try_open` returns
/// it: Linux rings the doorbell for its first message. Messages that were there before are
/// announced by one doorbell raised at open.
///
/// # Handing the endpoint to the doorbell handler
///
/// Both of those doorbells can come before the IO_BANK0 handler can reach the endpoint. A
/// handler that runs then acks the doorbell and has nothing to drain: the first message is
/// only late (the next one rings again), but the raise for messages already there is lost and
/// the ring, not armed, stalls for good. So once the handler can reach the endpoint, raise the
/// doorbell one more time (or enable the IO_BANK0 line only after the handoff):
///
/// ```ignore
/// let rp1::link::Proc1Link { down, .. } = init.try_open()?;
/// DOWN.put(down).ok().unwrap(); // the static Slot that the IO_BANK0 handler drains
/// doorbell.raise(); // for what came before the handler could reach `down`
/// ```
pub struct RingRx<T, W = Poll> {
    ring: shm::RingRx<T>,
    wake: W,
}

impl<T: Message, W: Wake> RingRx<T, W> {
    /// Attach the endpoint to a ring just attached and make it notifiable.
    pub(crate) fn open(mut ring: shm::RingRx<T>, mut wake: W) -> Self {
        wake.opened(Token(()), &mut ring);
        Self { ring, wake }
    }
    /// Take the oldest message, if any.
    #[inline(always)]
    pub fn recv(&mut self) -> Result<Option<T>, Error> {
        window_ready()?;
        self.ring.pop()
    }
    /// Take up to `out.len()` messages with one `tail` store. Returns how many.
    #[inline(always)]
    pub fn recv_into(&mut self, out: &mut [T]) -> Result<usize, Error> {
        window_ready()?;
        self.ring.pop_into(out)
    }
    /// Messages available now (a local load).
    pub fn available(&self) -> u32 {
        self.ring.available()
    }
    /// Pass up to `max` messages to `f` (`max = 0` counts as 1). With a doorbell, the reader is
    /// marked awake first and the ring armed and re-checked at the end: [`Progress::Idle`]
    /// means empty and armed. [`Progress::More`] means the budget ran out and the ring is not
    /// armed, so no doorbell comes for what remains: drain again, or raise the doorbell. On an
    /// error the ring is not armed.
    #[inline(always)]
    pub fn drain(&mut self, max: u32, mut f: impl FnMut(T)) -> Result<Progress, Error> {
        self.try_drain(max, |value| {
            f(value);
            Ok(())
        })
    }

    #[inline(always)]
    pub(crate) fn try_drain(
        &mut self,
        max: u32,
        mut f: impl FnMut(T) -> Result<(), Error>,
    ) -> Result<Progress, Error> {
        self.try_drain_requests(max, |value, _| f(value))
    }

    #[inline(always)]
    pub(crate) fn try_drain_requests(
        &mut self,
        max: u32,
        f: impl FnMut(T, Option<bool>) -> Result<(), Error>,
    ) -> Result<Progress, Error> {
        window_ready()?;
        self.wake.try_drain(Token(()), &mut self.ring, max, f)
    }
}

impl<T: Message, const PIN: u8, const CORE: u8> RingRx<T, Doorbell<PIN, CORE>> {
    /// Lab control only: [`Self::drain`] whose arm skips the readback that the protocol needs
    /// (`rp1_proto::shm::RingRx::__drain_without_readback`).
    #[doc(hidden)]
    pub fn __drain_without_readback(
        &mut self,
        max: u32,
        f: impl FnMut(T),
    ) -> Result<Progress, Error> {
        window_ready()?;
        self.ring.__drain_without_readback(max, f)
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use crate::mmio::fake::{self, Access};
    use crate::notify::{Doorbell, HostIrq};
    use rp1_proto::{
        LINK_START,
        shm::{Overflow, WAITING},
    };
    use std::vec::Vec;

    type Word = [u32; 4];
    const DEPTH: u32 = 8;
    const STATUS: usize = LINK_START as usize + link::SHM_STATUS;
    const MSIX_SET: usize = 0x4010_8008 + 4 * rp1_proto::irq::VECTOR as usize + 0x800;

    #[repr(align(8))]
    struct Mem([u8; 512]);
    /// One ring's memory: the data side (slots) and the control side (tail, waiting).
    struct Memory {
        data: Mem,
        ctrl: Mem,
    }
    impl Memory {
        fn new() -> Self {
            Self {
                data: Mem([0; 512]),
                ctrl: Mem([0; 512]),
            }
        }
        fn tx<N: Notify + crate::notify::sealed::Claim>(&mut self) -> RingTx<Word, N> {
            let ring = unsafe {
                shm::RingTx::attach(
                    self.data.0.as_mut_ptr(),
                    512,
                    self.ctrl.0.as_ptr(),
                    512,
                    DEPTH,
                    Overflow::Reject,
                )
            };
            RingTx::new(ring.unwrap(), N::claim(Token(())))
        }
        /// A Linux writer of the ring that `rx` reads.
        fn linux_tx(&mut self) -> shm::RingTx<Word> {
            unsafe {
                shm::RingTx::attach(
                    self.data.0.as_mut_ptr(),
                    512,
                    self.ctrl.0.as_ptr(),
                    512,
                    DEPTH,
                    Overflow::Reject,
                )
            }
            .unwrap()
        }
        fn rx<W: Wake + crate::notify::sealed::Claim>(&mut self) -> RingRx<Word, W> {
            let ring = unsafe {
                shm::RingRx::attach(
                    self.data.0.as_ptr(),
                    512,
                    self.ctrl.0.as_mut_ptr(),
                    512,
                    DEPTH,
                    Overflow::Reject,
                )
            };
            RingRx::open(ring.unwrap(), W::claim(Token(())))
        }
        fn waiting(&self) -> u32 {
            u32::from_le_bytes(self.ctrl.0[WAITING..WAITING + 4].try_into().unwrap())
        }
        fn set_waiting(&mut self, value: u32) {
            self.ctrl.0[WAITING..WAITING + 4].copy_from_slice(&value.to_le_bytes());
        }
    }
    fn ready() {
        fake::set(STATUS, link::SHM_READY);
        fake::take();
    }
    fn writes() -> Vec<(usize, u32)> {
        fake::take()
            .into_iter()
            .filter_map(|a| match a {
                Access::Write(address, value) => Some((address, value)),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn notifiers_take_no_room_in_an_endpoint() {
        use core::mem::size_of;
        assert_eq!(size_of::<RingTx<Word>>(), size_of::<shm::RingTx<Word>>());
        assert_eq!(
            size_of::<RingTx<Word, HostIrq>>(),
            size_of::<shm::RingTx<Word>>()
        );
        assert_eq!(size_of::<RingRx<Word>>(), size_of::<shm::RingRx<Word>>());
        assert_eq!(
            size_of::<RingRx<Word, Doorbell<27, 1>>>(),
            size_of::<shm::RingRx<Word>>()
        );
    }

    #[test]
    fn host_irq_ring_pulses_once_per_reader_sleep() {
        ready();
        let mut memory = Memory::new();
        let mut tx = memory.tx::<HostIrq>();
        // Awake reader: no pulse.
        assert_eq!(tx.send(&[1; 4]), Ok(()));
        assert_eq!(writes(), []);
        memory.set_waiting(1);
        assert_eq!(tx.send_slice(&[[2; 4], [3; 4]]), Ok(2));
        assert_eq!(
            fake::take()
                .iter()
                .filter(|a| **a == Access::Write(MSIX_SET, 2))
                .count(),
            1
        );
        // Same sleep: already notified.
        assert_eq!(tx.flush(), Ok(false));
        assert_eq!(tx.head(), 3);
    }

    #[test]
    fn poll_ring_never_notifies() {
        ready();
        let mut memory = Memory::new();
        memory.set_waiting(1);
        let mut tx = memory.tx::<Poll>();
        assert_eq!(tx.send(&[1; 4]), Ok(()));
        assert_eq!(tx.send_slice(&[[2; 4]]), Ok(1));
        assert_eq!(tx.flush(), Ok(false));
        assert_eq!(fake::take(), []);
    }

    #[test]
    fn push_does_not_wake_until_flush() {
        ready();
        let mut memory = Memory::new();
        memory.set_waiting(1);
        let mut tx = memory.tx::<HostIrq>();
        assert_eq!(tx.push(&[1; 4]), Ok(()));
        assert_eq!(tx.push_slice(&[[2; 4]]), Ok(1));
        assert_eq!(tx.push_with(2, |k, v| *v = [k as u32; 4]), Ok(2));
        assert_eq!(fake::take(), []);
        assert_eq!(tx.flush(), Ok(true));
        assert_eq!(tx.free(), DEPTH - 4);
    }

    /// PERSTn (Lost) or a window not published yet: no endpoint call touches ring memory.
    #[test]
    fn a_window_not_ready_is_reset_and_untouched() {
        for status in [0, link::SHM_WINDOW_LOST] {
            ready();
            let mut memory = Memory::new();
            let mut linux = memory.linux_tx();
            linux.push(&[7; 4]).unwrap();
            let mut rx = memory.rx::<Doorbell<26, 0>>();
            let mut tx = memory.tx::<HostIrq>();
            memory.set_waiting(1);
            let before = (memory.data.0, memory.ctrl.0);
            fake::set(STATUS, status);
            fake::take();
            assert_eq!(tx.send(&[1; 4]), Err(Error::Reset));
            assert_eq!(tx.send_slice(&[[1; 4]]), Err(Error::Reset));
            assert_eq!(tx.push(&[1; 4]), Err(Error::Reset));
            assert_eq!(tx.push_slice(&[[1; 4]]), Err(Error::Reset));
            assert_eq!(tx.push_with(1, |_, _| panic!("filled")), Err(Error::Reset));
            assert_eq!(tx.flush(), Err(Error::Reset));
            assert_eq!(rx.recv(), Err(Error::Reset));
            assert_eq!(rx.recv_into(&mut [[0; 4]; 2]), Err(Error::Reset));
            assert_eq!(rx.drain(4, |_| panic!("drained")), Err(Error::Reset));
            assert_eq!((memory.data.0, memory.ctrl.0), before);
            assert_eq!(fake::take(), []);
        }
    }

    /// The first message after open finds the reader asleep, so Linux rings the doorbell.
    #[test]
    fn an_empty_doorbell_ring_is_armed_at_open() {
        ready();
        let mut memory = Memory::new();
        let rx = memory.rx::<Doorbell<26, 0>>();
        assert_eq!(memory.waiting() & 1, 1);
        assert_eq!(writes(), []);
        let mut linux = memory.linux_tx();
        linux.push(&[1; 4]).unwrap();
        assert!(linux.reader_asleep());
        assert_eq!(rx.available(), 1);
    }

    /// Messages sent before open rang no doorbell: open raises it once for them.
    #[test]
    fn a_doorbell_ring_with_messages_raises_its_doorbell_at_open() {
        ready();
        let mut memory = Memory::new();
        memory.linux_tx().push(&[1; 4]).unwrap();
        let mut rx = memory.rx::<Doorbell<26, 0>>();
        assert_eq!(writes(), [(0x400d_2108, 1 << 26)]);
        assert_eq!(memory.waiting() & 1, 0);
        // The handler's drain takes the message and arms.
        let mut got = Vec::new();
        assert_eq!(rx.drain(16, |v| got.push(v)), Ok(Progress::Idle));
        assert_eq!(got, [[1; 4]]);
        assert_eq!(memory.waiting() & 1, 1);
    }

    #[test]
    fn server_replies_in_order_and_rearms() {
        ready();
        let mut down = Memory::new();
        let mut up = Memory::new();
        let mut server = crate::Server::new(up.tx::<Poll>(), down.rx::<Doorbell<27, 1>>());
        let mut replies = up.rx::<Poll>();
        assert_eq!(server.serve(1, |_| panic!("empty")), Ok(Progress::Idle));
        down.linux_tx().push_slice(&[[1; 4], [2; 4]]);
        assert_eq!(server.serve(0, |v| [v[0] + 10; 4]), Ok(Progress::More));
        assert_eq!(server.serve(2, |v| [v[0] + 10; 4]), Ok(Progress::Idle));
        assert_eq!(replies.recv(), Ok(Some([11; 4])));
        assert_eq!(replies.recv(), Ok(Some([12; 4])));
        assert_eq!(down.waiting() & 1, 1);
    }

    #[test]
    fn a_failed_reply_stops_the_server() {
        ready();
        let mut down = Memory::new();
        let mut up = Memory::new();
        down.linux_tx().push_slice(&[[1; 4], [2; 4]]);
        let mut tx = up.tx::<Poll>();
        tx.send_slice(&[[0; 4]; DEPTH as usize]).unwrap();
        let mut server = crate::Server::new(tx, down.rx::<Doorbell<27, 1>>());
        assert_eq!(
            server.serve(2, |v| {
                assert_eq!(v, [1; 4]);
                v
            }),
            Err(Error::Full)
        );
        assert_eq!(
            server.serve(2, |_| panic!("after failure")),
            Err(Error::Reset)
        );
        assert_eq!(down.waiting() & 1, 0);
    }

    #[test]
    fn a_poll_reader_is_never_armed() {
        ready();
        let mut memory = Memory::new();
        let mut rx = memory.rx::<Poll>();
        let mut linux = memory.linux_tx();
        assert_eq!(rx.drain(1, |_| panic!("empty")), Ok(Progress::Idle));
        linux.push_slice(&[[1; 4], [2; 4], [3; 4]]);
        let mut got = Vec::new();
        assert_eq!(rx.drain(2, |v| got.push(v[0])), Ok(Progress::More));
        assert_eq!(rx.drain(0, |v| got.push(v[0])), Ok(Progress::More));
        assert_eq!(rx.drain(2, |v| got.push(v[0])), Ok(Progress::Idle));
        assert_eq!(got, [1, 2, 3]);
        assert_eq!(rx.recv(), Ok(None));
        assert_eq!(memory.waiting(), 0);
        assert_eq!(fake::take(), []);
    }
}
