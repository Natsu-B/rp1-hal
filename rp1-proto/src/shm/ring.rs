//! The ring ends: attach, single and batch push/pop for reject and overwrite rings.
use super::access::{aligned, copy_to_mapping, ld, read_value, st};
use super::{BUSY, CONTROL_SIZE, DROPPED, HEAD, HEAD_HI, TAIL, TAIL_HI, WAITING, slot_size};
use crate::{Error, Message, barrier};
use core::{
    marker::PhantomData,
    mem::{MaybeUninit, size_of},
    ptr,
};

// An RPC reader knows the full expected sequence from its tail, including at wrap.
// Match either exact encoding so the ticket costs no additional posted store.
const REPLY_WAKE: u32 = 1 << 31;

/// Behaviour of a full ring (declaration `overflow`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Overflow {
    /// `push` returns `Err(Full)`; nothing is lost.
    Reject,
    /// `push` overwrites the oldest slot; the reader reports `Err(Lost(n))`.
    Overwrite,
}

fn check_ring<T>(
    data: *const u8,
    data_len: usize,
    ctrl: *const u8,
    ctrl_len: usize,
    depth: u32,
) -> Result<(), Error> {
    let ok = aligned(data)
        && aligned(ctrl)
        && depth.is_power_of_two()
        && depth <= 1 << 30
        && size_of::<T>() != 0
        && size_of::<T>() <= 0xffff
        && ctrl_len >= CONTROL_SIZE
        && (depth as usize)
            .checked_mul(slot_size(size_of::<T>()))
            .and_then(|n| n.checked_add(CONTROL_SIZE))
            .is_some_and(|n| n <= data_len);
    if ok { Ok(()) } else { Err(Error::Invalid) }
}

/// Writer end of a ring. Writes `data` (remote), reads `ctrl` (local).
pub struct RingTx<T> {
    pub(super) data: *mut u8,
    pub(super) ctrl: *const u8,
    pub(super) mask: u32,
    pub(super) overflow: Overflow,
    pub(super) head: u32,
    /// Upper half of the 64-bit logical `head` (overwrite only, see [`HEAD_HI`]).
    head_hi: u32,
    /// Last `tail` read from the local `ctrl`; reread only when the ring looks full.
    tail_seen: u32,
    dropped: u32,
    /// `dropped` changed since it was last stored remotely.
    dropped_dirty: bool,
    /// The odd `waiting` value (reader sleep) already notified; 0 = none.
    pub(super) woken: u32,
    /// A failed DMA copy may leave remote writes in flight.
    #[cfg(feature = "unstable-ring-dma")]
    pub(super) fenced: bool,
    _t: PhantomData<fn(T)>,
}
unsafe impl<T> Send for RingTx<T> {}
impl<T: Message> RingTx<T> {
    /// Attach the single writer. Reads the current `head`/`dropped` once (resume after restart).
    ///
    /// # Safety
    /// `data`/`ctrl` are 8-byte aligned mappings of the given lengths that stay valid, use only
    /// volatile access, and no other writer exists for this ring. No writes from a previous
    /// failed copier may remain in flight: dropping a fenced handle does not drain them.
    pub unsafe fn attach(
        data: *mut u8,
        data_len: usize,
        ctrl: *const u8,
        ctrl_len: usize,
        depth: u32,
        overflow: Overflow,
    ) -> Result<Self, Error> {
        check_ring::<T>(data, data_len, ctrl, ctrl_len, depth)?;
        let head = unsafe { ld(data, HEAD) };
        if overflow == Overflow::Reject {
            // A reject-mode slot's length never changes, so it is written once here instead
            // of with every message (one posted store less per push over PCIe). Rewriting
            // the same value is harmless for slots the reader has not consumed yet.
            for i in 0..depth {
                let slot =
                    unsafe { data.add(CONTROL_SIZE + i as usize * slot_size(size_of::<T>())) };
                unsafe { st(slot, 4, size_of::<T>() as u32) };
            }
        }
        Ok(Self {
            data,
            ctrl,
            mask: depth - 1,
            overflow,
            head,
            head_hi: unsafe { ld(data, HEAD_HI) },
            // Looks full until the first push reads the real tail.
            tail_seen: head.wrapping_sub(depth),
            dropped: unsafe { ld(data, DROPPED) },
            dropped_dirty: false,
            // A reader already asleep when this writer attaches is notified on the next call.
            woken: 0,
            #[cfg(feature = "unstable-ring-dma")]
            fenced: false,
            _t: PhantomData,
        })
    }
    /// Only a failed DMA copy fences a writer; without the DMA producer this is constant.
    #[inline(always)]
    fn fenced(&self) -> bool {
        #[cfg(feature = "unstable-ring-dma")]
        return self.fenced;
        #[cfg(not(feature = "unstable-ring-dma"))]
        false
    }
    #[inline]
    fn slot(&self, index: u32) -> *mut u8 {
        unsafe {
            self.data
                .add(CONTROL_SIZE + (index & self.mask) as usize * slot_size(size_of::<T>()))
        }
    }
    /// Free slots as last seen (reads the local `tail`).
    pub fn free(&self) -> u32 {
        if self.fenced() {
            return 0;
        }
        let tail = unsafe { ld(self.ctrl, TAIL) };
        (self.mask + 1).saturating_sub(self.head.wrapping_sub(tail))
    }

    /// Whether the reader has consumed all published messages.
    #[inline(always)]
    pub fn all_consumed(&self) -> bool {
        unsafe { ld(self.ctrl, TAIL) == self.head }
    }
    /// Slots this writer can publish now (all slots for overwrite, zero when fenced).
    /// Reject mode refreshes the cached tail only when necessary.
    pub fn writable(&mut self) -> u32 {
        if self.fenced() {
            0
        } else if self.overflow == Overflow::Overwrite {
            self.mask + 1
        } else {
            self.reject_free(self.mask + 1)
        }
    }
    /// Free slots for a reject-mode push, rereading `tail` only when the cached one says
    /// fewer than `want` are free.
    #[inline(always)]
    pub(super) fn reject_free(&mut self, want: u32) -> u32 {
        let depth = self.mask + 1;
        let free = depth.saturating_sub(self.head.wrapping_sub(self.tail_seen));
        if free >= want {
            return free;
        }
        self.tail_seen = unsafe { ld(self.ctrl, TAIL) };
        depth.saturating_sub(self.head.wrapping_sub(self.tail_seen))
    }
    #[inline(always)]
    pub(super) fn store_dropped(&mut self) {
        if self.dropped_dirty {
            self.dropped_dirty = false;
            unsafe { st(self.data, DROPPED, self.dropped) };
        }
    }
    /// Fill the slot for index `head + k` (payload, seq; the length was written by
    /// `attach`); not yet published.
    #[inline(always)]
    fn write_slot(&self, k: u32, value: &T, seq_xor: u32) {
        let index = self.head.wrapping_add(k);
        let slot = self.slot(index);
        unsafe {
            copy_to_mapping(slot.add(8), ptr::from_ref(value).cast(), size_of::<T>());
            st(slot, 0, index ^ seq_xor);
        }
    }
    /// Publish one message. Order: payload, seq, length → barrier → head (protocol.md §2.3).
    #[cfg_attr(feature = "isram", rp1_macros::isram)]
    pub fn push(&mut self, value: &T) -> Result<(), Error> {
        self.push_inner(value, 0)
    }

    /// Publish a reject-ring RPC request after negotiating `RPC_REPLY_WAKE`.
    #[inline(always)]
    pub fn push_request(&mut self, value: &T, notify_reply: bool) -> Result<(), Error> {
        if self.overflow != Overflow::Reject {
            return Err(Error::Invalid);
        }
        self.push_inner(value, if notify_reply { REPLY_WAKE } else { 0 })
    }

    #[inline(always)]
    fn push_inner(&mut self, value: &T, seq_xor: u32) -> Result<(), Error> {
        if self.fenced() {
            return Err(Error::NotReady);
        }
        let len = size_of::<T>() as u32;
        let slot = self.slot(self.head);
        match self.overflow {
            Overflow::Reject => {
                if self.reject_free(1) == 0 {
                    // Counted locally; stored with the next successful push, so a writer
                    // spinning on `Full` issues no remote stores into the reader's memory.
                    self.dropped = self.dropped.wrapping_add(1);
                    self.dropped_dirty = true;
                    return Err(Error::Full);
                }
                self.store_dropped();
                barrier();
                self.write_slot(0, value, seq_xor);
            }
            Overflow::Overwrite => unsafe {
                st(slot, 4, BUSY | len);
                barrier();
                copy_to_mapping(slot.add(8), ptr::from_ref(value).cast(), size_of::<T>());
                st(slot, 0, self.head);
                barrier();
                st(slot, 4, len);
            },
        }
        barrier();
        self.head = self.head.wrapping_add(1);
        unsafe { st(self.data, HEAD, self.head) };
        if self.overflow == Overflow::Overwrite && self.head == 0 {
            // A stopped reader may fall a full 2^32 behind; `head_hi` tells that apart from
            // an empty ring. Stored after `head`, so a reader that sees the new `head_hi`
            // also sees the wrapped `head` (protocol.md §2.3).
            barrier();
            self.head_hi = self.head_hi.wrapping_add(1);
            unsafe { st(self.data, HEAD_HI, self.head_hi) };
        }
        Ok(())
    }
    /// Publish up to `values.len()` messages with one `head` store (and one barrier) for the
    /// whole batch: the reader sees all of them at once. Returns how many were written
    /// (`0` if the ring is full). Unlike [`RingTx::push`] a short write does not count as
    /// `dropped`; the caller still holds the rest. `overwrite` rings push one by one.
    #[cfg_attr(feature = "isram", rp1_macros::isram)]
    pub fn push_slice(&mut self, values: &[T]) -> usize {
        if self.fenced() {
            return 0;
        }
        if self.overflow == Overflow::Overwrite {
            for v in values {
                let _ = self.push(v);
            }
            return values.len();
        }
        let want = values.len().min(u32::MAX as usize) as u32;
        let n = self.reject_free(want).min(want);
        if n == 0 {
            return 0;
        }
        self.store_dropped();
        barrier();
        for (k, v) in values[..n as usize].iter().enumerate() {
            self.write_slot(k as u32, v, 0);
        }
        barrier();
        self.head = self.head.wrapping_add(n);
        unsafe { st(self.data, HEAD, self.head) };
        n as usize
    }
    /// Generate up to `count` payloads and publish them with the CPU. Direct generation
    /// helps the DMA path by avoiding a staging copy; on the CPU path prefer [`Self::push_slice`].
    ///
    /// # Callback contract
    /// `fill` receives the offset within this call and an initialized payload. Previous
    /// contents are visible but unspecified; the callback must write every field.
    /// A reused stage provides no confidentiality isolation across rings or calls; use
    /// separate or sanitized storage across confidentiality boundaries.
    ///
    /// The callback runs once per accepted message, never for a full or fenced ring.
    /// No short write counts as dropped. Reject rings publish the batch together;
    /// overwrite rings publish one by one, as in [`Self::push_slice`].
    ///
    /// # Panics
    /// A callback panic leaves a reject ring's head and remote dropped counter unchanged.
    /// An overwrite ring may have published earlier messages.
    #[cfg_attr(feature = "isram", rp1_macros::isram)]
    pub fn push_with(&mut self, count: usize, mut fill: impl FnMut(usize, &mut T)) -> usize {
        if self.fenced() {
            return 0;
        }
        let n = match self.overflow {
            Overflow::Reject => {
                let want = count.min(u32::MAX as usize) as u32;
                self.reject_free(want).min(want) as usize
            }
            Overflow::Overwrite => count,
        };
        if n == 0 {
            return 0;
        }
        // One temporary per call avoids clearing every generated message.
        // SAFETY: Message has no padding and every bit pattern is valid.
        let mut value = unsafe { MaybeUninit::<T>::zeroed().assume_init() };
        match self.overflow {
            Overflow::Reject => {
                barrier();
                for k in 0..n {
                    fill(k, &mut value);
                    self.write_slot(k as u32, &value, 0);
                }
                self.store_dropped();
                barrier();
                self.head = self.head.wrapping_add(n as u32);
                unsafe { st(self.data, HEAD, self.head) };
            }
            Overflow::Overwrite => {
                for k in 0..n {
                    fill(k, &mut value);
                    let _ = self.push(&value);
                }
            }
        }
        n
    }
    pub fn head(&self) -> u32 {
        self.head
    }
    pub fn dropped(&self) -> u32 {
        self.dropped
    }
}

/// Reader end of a ring. Reads `data` (local), writes `ctrl` (remote).
pub struct RingRx<T> {
    data: *const u8,
    pub(super) ctrl: *mut u8,
    mask: u32,
    overflow: Overflow,
    tail: u32,
    /// Upper half of the 64-bit logical `tail` (overwrite only, see [`TAIL_HI`]).
    tail_hi: u32,
    /// Last `head` read from the local `data`; reread only when the ring looks empty.
    head_seen: u32,
    /// Shadow of the remote `waiting` sleep counter (odd = asleep).
    pub(super) waiting: u32,
    _t: PhantomData<fn() -> T>,
}
unsafe impl<T> Send for RingRx<T> {}
impl<T: Message> RingRx<T> {
    /// Attach the single reader. Reads the current `tail` once (resume after restart).
    ///
    /// # Safety
    /// As [`RingTx::attach`], for the reader.
    pub unsafe fn attach(
        data: *const u8,
        data_len: usize,
        ctrl: *mut u8,
        ctrl_len: usize,
        depth: u32,
        overflow: Overflow,
    ) -> Result<Self, Error> {
        check_ring::<T>(data, data_len, ctrl, ctrl_len, depth)?;
        let tail = unsafe { ld(ctrl, TAIL) };
        // Resume the sleep counter; a previous reader that died asleep is marked awake, so the
        // next sleep gets a value the writer has not notified yet.
        let mut waiting = unsafe { ld(ctrl, WAITING) };
        if waiting & 1 != 0 {
            waiting = waiting.wrapping_add(1);
            unsafe { st(ctrl, WAITING, waiting) };
        }
        Ok(Self {
            data,
            ctrl,
            mask: depth - 1,
            overflow,
            tail,
            tail_hi: unsafe { ld(ctrl, TAIL_HI) },
            head_seen: tail,
            waiting,
            _t: PhantomData,
        })
    }
    #[inline]
    fn slot(&self, index: u32) -> *const u8 {
        unsafe {
            self.data
                .add(CONTROL_SIZE + (index & self.mask) as usize * slot_size(size_of::<T>()))
        }
    }
    /// Messages available as seen now (reads the local `head`). Overwrite rings count with
    /// the 64-bit indices and saturate at `u32::MAX`.
    pub fn available(&self) -> u32 {
        match self.overflow {
            Overflow::Reject => unsafe { ld(self.data, HEAD) }.wrapping_sub(self.tail),
            Overflow::Overwrite => self.available64().min(u32::MAX as u64) as u32,
        }
    }
    /// Overwrite mode: `head - tail` over 64-bit logical indices. `head_hi` is read first: it
    /// is stored after `head`, so a new `head_hi` implies a new `head`, and the only torn
    /// result is a `head` 2^32 too small, which reads as "nothing yet" (like a stale `head`).
    fn available64(&self) -> u64 {
        let hi = unsafe { ld(self.data, HEAD_HI) };
        barrier();
        let lo = unsafe { ld(self.data, HEAD) };
        let head = (u64::from(hi) << 32) | u64::from(lo);
        let tail = (u64::from(self.tail_hi) << 32) | u64::from(self.tail);
        let diff = head.wrapping_sub(tail);
        // A negative distance (torn or stale `head`) is "empty for now".
        if diff > u64::MAX / 2 { 0 } else { diff }
    }
    /// Advance and store `tail` by `n` (one remote store, two on a 2^32 wrap).
    fn advance_tail(&mut self, n: u32) {
        let (tail, wrapped) = self.tail.overflowing_add(n);
        self.tail = tail;
        unsafe { st(self.ctrl, TAIL, self.tail) };
        if wrapped && self.overflow == Overflow::Overwrite {
            self.tail_hi = self.tail_hi.wrapping_add(1);
            unsafe { st(self.ctrl, TAIL_HI, self.tail_hi) };
        }
    }
    fn consume(&mut self) {
        barrier();
        self.advance_tail(1);
    }
    /// Messages readable in reject mode, rereading `head` only when the cached one shows
    /// fewer than `want`. A reread is followed by a barrier so slot reads cannot pass it.
    #[inline(always)]
    fn reject_available(&mut self, want: u32) -> Result<u32, Error> {
        let mut available = self.head_seen.wrapping_sub(self.tail);
        if available < want {
            self.head_seen = unsafe { ld(self.data, HEAD) };
            barrier();
            available = self.head_seen.wrapping_sub(self.tail);
        }
        if available > self.mask + 1 {
            return Err(Error::Corrupt);
        }
        Ok(available)
    }
    /// Read the slot at `tail + k` (reject mode): checks seq and length.
    #[inline(always)]
    fn read_slot(&self, k: u32) -> Result<T, Error> {
        self.read_slot_inner(k, false).map(|(value, _)| value)
    }

    #[inline(always)]
    fn read_slot_inner(&self, k: u32, rpc: bool) -> Result<(T, bool), Error> {
        let index = self.tail.wrapping_add(k);
        let slot = self.slot(index);
        let (seq, word) = unsafe { (ld(slot, 0), ld(slot, 4)) };
        let notify_reply = rpc && seq == index ^ REPLY_WAKE;
        if seq != index && !notify_reply {
            return Err(Error::Corrupt);
        }
        if word != size_of::<T>() as u32 {
            return Err(Error::Invalid);
        }
        Ok((unsafe { read_value::<T>(slot.add(8)) }, notify_reply))
    }

    /// Consume a reject-ring request and its negotiated `RPC_REPLY_WAKE` flag.
    #[inline(always)]
    pub fn pop_request(&mut self) -> Result<Option<(T, bool)>, Error> {
        if self.overflow != Overflow::Reject {
            return Err(Error::Invalid);
        }
        self.pop_reject(true)
    }

    #[inline(always)]
    fn pop_reject(&mut self, rpc: bool) -> Result<Option<(T, bool)>, Error> {
        if self.reject_available(1)? == 0 {
            return Ok(None);
        }
        let request = self.read_slot_inner(0, rpc)?;
        self.consume();
        Ok(Some(request))
    }
    /// `Ok(None)` when empty. `Err(Lost(n))` (overwrite only) reports skipped messages; the
    /// reader has already moved to the oldest surviving slot, so just call `pop` again.
    #[cfg_attr(feature = "isram", rp1_macros::isram)]
    pub fn pop(&mut self) -> Result<Option<T>, Error> {
        let len = size_of::<T>() as u32;
        match self.overflow {
            Overflow::Reject => Ok(self.pop_reject(false)?.map(|(value, _)| value)),
            Overflow::Overwrite => {
                for _ in 0..1024 {
                    let available = self.available64();
                    if available == 0 {
                        return Ok(None);
                    }
                    let depth = u64::from(self.mask) + 1;
                    if available > depth {
                        // More than u32::MAX lost: report it in parts, the next pop continues.
                        let lost = (available - depth).min(u32::MAX as u64) as u32;
                        self.advance_tail(lost);
                        return Err(Error::Lost(lost));
                    }
                    let slot = self.slot(self.tail);
                    barrier();
                    let (seq, word) = unsafe { (ld(slot, 0), ld(slot, 4)) };
                    if word & BUSY != 0 || seq != self.tail {
                        // Being (or already) overwritten by a newer lap; head catches up.
                        core::hint::spin_loop();
                        continue;
                    }
                    barrier();
                    let value = unsafe { read_value::<T>(slot.add(8)) };
                    barrier();
                    let (word2, seq2) = unsafe { (ld(slot, 4), ld(slot, 0)) };
                    if word2 != word || seq2 != seq {
                        continue;
                    }
                    if word != len {
                        return Err(Error::Invalid);
                    }
                    self.consume();
                    return Ok(Some(value));
                }
                Err(Error::NotReady)
            }
        }
    }
    /// Inspect the newest published reject-mode slot without releasing any slots.
    /// The index is returned even on error, so observers can count each slot once.
    #[allow(clippy::type_complexity)]
    pub fn newest(&self) -> Result<Option<(u32, Result<T, Error>)>, Error> {
        if self.overflow != Overflow::Reject {
            return Err(Error::Invalid);
        }
        let head = unsafe { ld(self.data, HEAD) };
        barrier();
        let n = head.wrapping_sub(self.tail);
        if n > self.mask + 1 {
            return Err(Error::Corrupt);
        }
        Ok(if n == 0 {
            None
        } else {
            Some((head.wrapping_sub(1), self.read_slot(n - 1)))
        })
    }
    /// Batch read preserving an encountered error even after a valid prefix. The error
    /// concerns the current `tail` after the prefix is released. Overwrite reads one slot.
    pub fn pop_into_report(&mut self, out: &mut [T]) -> (usize, Option<Error>) {
        if self.overflow == Overflow::Overwrite {
            return match out.first_mut() {
                None => (0, None),
                Some(slot) => match self.pop() {
                    Ok(Some(v)) => {
                        *slot = v;
                        (1, None)
                    }
                    Ok(None) => (0, None),
                    Err(e) => (0, Some(e)),
                },
            };
        }
        let want = out.len().min(u32::MAX as usize) as u32;
        let n = match self.reject_available(want) {
            Ok(n) => n.min(want),
            Err(e) => return (0, Some(e)),
        };
        let mut done = 0;
        let mut error = None;
        while done < n {
            match self.read_slot(done) {
                Ok(v) => out[done as usize] = v,
                Err(e) => {
                    error = Some(e);
                    break;
                }
            }
            done += 1;
        }
        if done > 0 {
            barrier();
            self.advance_tail(done);
        }
        (done as usize, error)
    }
    /// Read a batch; a valid prefix is returned even if the following slot failed its check.
    /// Use [`Self::pop_into_report`] to observe transient errors after a valid prefix.
    #[cfg_attr(feature = "isram", rp1_macros::isram)]
    pub fn pop_into(&mut self, out: &mut [T]) -> Result<usize, Error> {
        let (n, error) = self.pop_into_report(out);
        if n > 0 {
            Ok(n)
        } else {
            error.map_or(Ok(0), Err)
        }
    }
    pub fn tail(&self) -> u32 {
        self.tail
    }
    /// Rejected pushes counted by the writer (observational).
    pub fn dropped(&self) -> u32 {
        unsafe { ld(self.data, DROPPED) }
    }
}
