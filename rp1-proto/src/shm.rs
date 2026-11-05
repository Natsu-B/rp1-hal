//! Linux shared memory: SPSC rings and blocks over uncached mappings (protocol.md §1–§5).
//!
//! Both peers use these types on raw pointers into mappings that are *not* ordinary memory:
//! on Linux a UIO mapping (arm64 Device attributes: unaligned access and `DC ZVA` fault), on
//! RP1 the outbound PCIe window (every read is a non-posted PCIe round trip). Therefore:
//!
//! - every access is an aligned volatile 32-bit (or, on 64-bit hosts, 64-bit) load/store;
//!   no `memcpy`/`memset` and no Rust reference into a mapping ever exists;
//! - "reader-local" placement: each side only *writes* the peer's memory and only *reads*
//!   its own. A ring is split into `data` (slots, `head`, `dropped`; local to the reader)
//!   and `ctrl` (`tail`, `waiting`; local to the writer). Each endpoint keeps a private
//!   shadow of the index it owns and reads the remote copy only once, in `attach`;
//! - no exclusive (LDREX/STREX) or RMW access; every shared word has a single writer.
use crate::{Error, Message, barrier};
use core::{
    marker::PhantomData,
    mem::{MaybeUninit, align_of, size_of},
    ptr,
};

/// DDR region header (one per reservation, at offset 0).
pub const HEADER_SIZE: usize = 64;
/// `data` and `ctrl` start with this many bytes of control words (two 64-byte lines).
pub const CONTROL_SIZE: usize = 128;
/// `data`: next slot index the writer will fill.
pub const HEAD: usize = 0;
/// `data`: times `head` wrapped past `u32::MAX` (`overflow = overwrite`; written after `head`).
pub const HEAD_HI: usize = 4;
/// `data`: rejected pushes (`overflow = reject`), observational only.
pub const DROPPED: usize = 64;
/// `ctrl`: next slot index the reader will consume.
pub const TAIL: usize = 0;
/// `ctrl`: times `tail` wrapped past `u32::MAX` (`overflow = overwrite`).
pub const TAIL_HI: usize = 4;
/// `ctrl`: the reader's sleep counter (protocol.md §2.3). Odd = asleep and wants a
/// notification, even = awake. Only the reader writes it, adding 1 at every transition, so
/// consecutive sleeps have different values and the writer notifies at most once per sleep.
/// The counter wraps after 2^31 sleeps: a sleep may then reuse the odd value the writer
/// notified last and get no notification; the reader's bounded re-check covers that.
pub const WAITING: usize = 64;
/// Slot word 1 bit: the writer is rewriting this slot (`overflow = overwrite`).
const BUSY: u32 = 1 << 16;
const MAGIC: [u8; 8] = *b"RP1SHM\0\0";
const VERSION: u32 = 1;

pub const fn slot_size(elem: usize) -> usize {
    8 + ((elem + 7) & !7)
}
/// Bytes of the reader-local ring part; `None` if it does not fit `usize` (same verdict on
/// 32-bit RP1 and 64-bit hosts once compared against a `u32` region length).
pub const fn ring_body_size(elem: usize, depth: usize) -> Option<usize> {
    let Some(slot) = (elem as u64).checked_add(7) else {
        return None;
    };
    let Some(slots) = (depth as u64).checked_mul(8 + (slot & !7)) else {
        return None;
    };
    let Some(total) = slots.checked_add(CONTROL_SIZE as u64) else {
        return None;
    };
    // Computed in u64 so a 32-bit target cannot wrap; regions are u32-sized anyway.
    if total > u32::MAX as u64 {
        None
    } else {
        Some(total as usize)
    }
}
pub const fn block_size(elem: usize) -> usize {
    8 + ((elem + 7) & !7)
}

#[inline(always)]
unsafe fn ld(base: *const u8, offset: usize) -> u32 {
    #[cfg(test)]
    tests::model::before_load(unsafe { base.add(offset) });
    unsafe { ptr::read_volatile(base.add(offset).cast()) }
}
#[inline(always)]
unsafe fn st(base: *mut u8, offset: usize, value: u32) {
    unsafe { ptr::write_volatile(base.add(offset).cast(), value) }
}

/// Copy `bytes` bytes (padded up to 4/8) from a mapping into ordinary memory.
///
/// # Safety
/// `src` is 8-byte aligned (4 on 32-bit targets) and readable for `bytes` rounded up to 8.
/// `dst` is writable ordinary memory for `bytes`.
#[inline(always)]
pub unsafe fn copy_from_mapping(dst: *mut u8, src: *const u8, bytes: usize) {
    #[cfg(target_pointer_width = "64")]
    {
        let mut i = 0;
        while i < bytes {
            let w = unsafe { ptr::read_volatile(src.add(i).cast::<u64>()) }.to_le_bytes();
            let n = if bytes - i < 8 { bytes - i } else { 8 };
            unsafe { ptr::copy_nonoverlapping(w.as_ptr(), dst.add(i), n) };
            i += 8;
        }
    }
    #[cfg(not(target_pointer_width = "64"))]
    {
        let mut i = 0;
        while i < bytes {
            let w = unsafe { ld(src, i) }.to_le_bytes();
            let n = if bytes - i < 4 { bytes - i } else { 4 };
            unsafe { ptr::copy_nonoverlapping(w.as_ptr(), dst.add(i), n) };
            i += 4;
        }
    }
}
/// Copy `bytes` bytes into a mapping; the tail of the last word is zero-filled.
///
/// # Safety
/// As [`copy_from_mapping`] with the roles swapped; `dst` is writable up to `bytes` rounded to 8.
#[inline(always)]
pub unsafe fn copy_to_mapping(dst: *mut u8, src: *const u8, bytes: usize) {
    #[cfg(target_pointer_width = "64")]
    {
        let mut i = 0;
        while i < bytes {
            let mut w = [0u8; 8];
            let n = if bytes - i < 8 { bytes - i } else { 8 };
            unsafe { ptr::copy_nonoverlapping(src.add(i), w.as_mut_ptr(), n) };
            unsafe { ptr::write_volatile(dst.add(i).cast::<u64>(), u64::from_le_bytes(w)) };
            i += 8;
        }
    }
    #[cfg(not(target_pointer_width = "64"))]
    {
        let mut i = 0;
        while i < bytes {
            let mut w = [0u8; 4];
            let n = if bytes - i < 4 { bytes - i } else { 4 };
            unsafe { ptr::copy_nonoverlapping(src.add(i), w.as_mut_ptr(), n) };
            unsafe { st(dst, i, u32::from_le_bytes(w)) };
            i += 4;
        }
    }
}
unsafe fn read_value<T: Message>(src: *const u8) -> T {
    let mut v = MaybeUninit::<T>::uninit();
    unsafe {
        copy_from_mapping(v.as_mut_ptr().cast(), src, size_of::<T>());
        // Message: every bit pattern is valid and there is no padding.
        v.assume_init()
    }
}

fn aligned(p: *const u8) -> bool {
    !p.is_null() && p as usize & 7 == 0
}

/// RP1's initializing handle for the DDR region header (protocol.md §1.3).
///
/// Only [`HeaderInit::begin`] creates it, under a writable-mapping contract; Linux's
/// read-only [`Header`] has no write path, so a `PROT_READ` mapping stays sound.
pub struct HeaderInit {
    base: *mut u8,
    generation: u32,
}
unsafe impl Send for HeaderInit {}
impl HeaderInit {
    /// RP1: mark the region as being initialized (odd generation). Returns the handle whose
    /// [`HeaderInit::publish`] writes the next even generation. `previous` is the old header
    /// generation (read once).
    ///
    /// # Safety
    /// `base` is the aligned, writable 64-byte header mapping, it stays valid while the handle
    /// lives, and the caller is its only writer.
    pub unsafe fn begin(base: *mut u8, previous: u32) -> Self {
        let generation = (previous | 1).wrapping_add(1).max(2);
        unsafe { st(base, 12, generation.wrapping_sub(1)) };
        barrier();
        Self { base, generation }
    }
    /// The generation [`HeaderInit::publish`] will write.
    pub fn generation(&self) -> u32 {
        self.generation
    }
    /// RP1: write the header fields (generation stays odd).
    pub fn write(&mut self, note_sha256: &[u8; 32], regions: u32, uptime_us: u32) {
        let b = self.base;
        unsafe {
            st(
                b,
                0,
                u32::from_le_bytes([MAGIC[0], MAGIC[1], MAGIC[2], MAGIC[3]]),
            );
            st(
                b,
                4,
                u32::from_le_bytes([MAGIC[4], MAGIC[5], MAGIC[6], MAGIC[7]]),
            );
            st(b, 8, ((HEADER_SIZE as u32) << 16) | VERSION);
            for (i, w) in note_sha256.as_chunks::<4>().0.iter().enumerate() {
                st(b, 16 + i * 4, u32::from_le_bytes(*w));
            }
            st(b, 48, regions);
            st(b, 52, uptime_us);
            st(b, 56, 0);
            st(b, 60, 0);
        }
    }
    /// RP1: publish (even generation) and give up write access. Call after all rings/blocks
    /// are initialized; the caller drains posted writes (same-window readback) before it
    /// tells the peer.
    pub fn publish(self) -> u32 {
        barrier();
        unsafe { st(self.base, 12, self.generation) };
        barrier();
        self.generation
    }
}

/// Linux's read-only view of a published DDR region header.
#[derive(Clone, Copy)]
pub struct Header {
    base: *const u8,
    generation: u32,
}
unsafe impl Send for Header {}
impl Header {
    /// Linux: open a published header and check the note hash.
    ///
    /// # Safety
    /// `base` stays a readable aligned 64-byte mapping for the lifetime of the handle.
    pub unsafe fn open(base: *const u8, note_sha256: &[u8; 32]) -> Result<Self, Error> {
        if !aligned(base) {
            return Err(Error::Invalid);
        }
        let generation = unsafe { ld(base, 12) };
        barrier();
        let mut magic = [0u8; 8];
        unsafe {
            magic[..4].copy_from_slice(&ld(base, 0).to_le_bytes());
            magic[4..].copy_from_slice(&ld(base, 4).to_le_bytes());
        }
        if generation & 1 != 0
            || generation == 0
            || magic != MAGIC
            || unsafe { ld(base, 8) } != ((HEADER_SIZE as u32) << 16) | VERSION
        {
            return Err(Error::NotReady);
        }
        for (i, w) in note_sha256.as_chunks::<4>().0.iter().enumerate() {
            if unsafe { ld(base, 16 + i * 4) }.to_le_bytes() != *w {
                return Err(Error::Mismatch);
            }
        }
        let h = Self { base, generation };
        barrier();
        h.check()?;
        Ok(h)
    }
    /// Linux: `Err(Reset)` once RP1 re-initialized the region.
    #[inline]
    pub fn check(&self) -> Result<(), Error> {
        if unsafe { ld(self.base, 12) } == self.generation {
            Ok(())
        } else {
            Err(Error::Reset)
        }
    }
    pub fn generation(&self) -> u32 {
        self.generation
    }
    pub fn regions(&self) -> u32 {
        unsafe { ld(self.base, 48) }
    }
}

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

/// Copies staged slot images into the reader's memory for [`RingTx::push_slice_staged`]
/// (`rp1-hal` implements it for a DMA channel).
///
/// # Safety
/// `copy_to_remote` writes exactly `src` to `dst..dst + src.len()` and nothing else.
/// On every return the engine has stopped accessing `src` (the stage may be reused).
/// On `Ok`, ALL writes must have reached the ordering point shared with the caller's
/// subsequent stores through the ring mapping: those stores must not overtake the copy.
/// This is an ordering contract, not a comparison of any particular payload value.
/// On `Err`, remote writes may still be in flight; the caller must fence this ring and
/// must not reuse its destination until an external reset/drain establishes quiescence.
pub unsafe trait SlotCopy {
    type Error;
    /// # Safety
    /// `dst..dst + src.len()` is writable memory of the ring's `data` mapping.
    unsafe fn copy_to_remote(&mut self, src: &[u8], dst: *mut u8) -> Result<(), Self::Error>;
}

/// Error of [`RingTx::push_slice_staged`]. Nothing was published in either case.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Staged<E> {
    /// The stage is not 8-byte aligned or smaller than one slot.
    Invalid,
    /// The copy failed.
    Copy(E),
    /// An earlier copy failed; this writer cannot publish again.
    Fenced,
}

/// Writes the slot image of message `index` (`seq`, length, payload, zero padding) at `dst`.
///
/// # Safety
/// `dst` is 8-byte aligned and `slot_size(size_of::<T>())` bytes are writable.
#[inline(always)]
unsafe fn stage_slot<T: Message>(dst: *mut u8, index: u32, value: &T) {
    // Volatile word stores: a plain loop becomes a `memcpy` call, which on RP1 runs from shared
    // SRAM and made a 256 B slot about 10x slower to stage (HW 2026-11-05).
    let size = size_of::<T>();
    unsafe {
        dst.cast::<u32>().write_volatile(index);
        dst.add(4).cast::<u32>().write_volatile(size as u32);
        let src = ptr::from_ref(value).cast::<u8>();
        if size.is_multiple_of(4) && align_of::<T>() >= 4 {
            for i in 0..size / 4 {
                dst.add(8 + 4 * i)
                    .cast::<u32>()
                    .write_volatile(src.add(4 * i).cast::<u32>().read());
            }
        } else {
            for i in 0..size {
                dst.add(8 + i).write_volatile(src.add(i).read());
            }
        }
        for i in 8 + size..slot_size(size) {
            dst.add(i).write_volatile(0);
        }
    }
}

/// Writer end of a ring. Writes `data` (remote), reads `ctrl` (local).
pub struct RingTx<T> {
    data: *mut u8,
    ctrl: *const u8,
    mask: u32,
    overflow: Overflow,
    head: u32,
    /// Upper half of the 64-bit logical `head` (overwrite only, see [`HEAD_HI`]).
    head_hi: u32,
    /// Last `tail` read from the local `ctrl`; reread only when the ring looks full.
    tail_seen: u32,
    dropped: u32,
    /// `dropped` changed since it was last stored remotely.
    dropped_dirty: bool,
    /// The odd `waiting` value (reader sleep) already notified; 0 = none.
    woken: u32,
    /// A failed copy may leave remote writes in flight.
    fenced: bool,
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
            fenced: false,
            _t: PhantomData,
        })
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
        if self.fenced {
            return 0;
        }
        let tail = unsafe { ld(self.ctrl, TAIL) };
        (self.mask + 1).saturating_sub(self.head.wrapping_sub(tail))
    }
    /// Slots this writer can publish now (all slots for overwrite, zero when fenced).
    /// Reject mode refreshes the cached tail only when necessary.
    pub fn writable(&mut self) -> u32 {
        if self.fenced {
            0
        } else if self.overflow == Overflow::Overwrite {
            self.mask + 1
        } else {
            self.reject_free(self.mask + 1)
        }
    }
    /// A copy failed; only an external reset/drain can make destination reuse safe.
    pub fn is_fenced(&self) -> bool {
        self.fenced
    }
    /// Free slots for a reject-mode push, rereading `tail` only when the cached one says
    /// fewer than `want` are free.
    #[inline(always)]
    fn reject_free(&mut self, want: u32) -> u32 {
        let depth = self.mask + 1;
        let free = depth.saturating_sub(self.head.wrapping_sub(self.tail_seen));
        if free >= want {
            return free;
        }
        self.tail_seen = unsafe { ld(self.ctrl, TAIL) };
        depth.saturating_sub(self.head.wrapping_sub(self.tail_seen))
    }
    #[inline(always)]
    fn store_dropped(&mut self) {
        if self.dropped_dirty {
            self.dropped_dirty = false;
            unsafe { st(self.data, DROPPED, self.dropped) };
        }
    }
    /// Fill the slot for index `head + k` (payload, seq; the length was written by
    /// `attach`); not yet published.
    #[inline(always)]
    fn write_slot(&self, k: u32, value: &T) {
        let index = self.head.wrapping_add(k);
        let slot = self.slot(index);
        unsafe {
            copy_to_mapping(slot.add(8), ptr::from_ref(value).cast(), size_of::<T>());
            st(slot, 0, index);
        }
    }
    /// Publish one message. Order: payload, seq, length → barrier → head (protocol.md §2.3).
    #[cfg_attr(feature = "isram", rp1_macros::isram)]
    pub fn push(&mut self, value: &T) -> Result<(), Error> {
        if self.fenced {
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
                self.write_slot(0, value);
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
        if self.fenced {
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
            self.write_slot(k as u32, v);
        }
        barrier();
        self.head = self.head.wrapping_add(n);
        unsafe { st(self.data, HEAD, self.head) };
        n as usize
    }
    /// How many messages of `values` one [`RingTx::push_slice_staged`] call can stage in
    /// `stage_len` bytes (whole slot images, [`slot_size`] each).
    pub const fn staged_capacity(stage_len: usize) -> usize {
        stage_len / slot_size(size_of::<T>())
    }
    /// Like [`RingTx::push_slice`], but the slots are written by `copy` (a DMA channel) from
    /// slot images staged in `stage` (8-byte aligned, at least one slot): for each message the
    /// writer stages `seq`, the length and the zero-padded payload, `copy` moves them to the
    /// reader's slots in one block, or two when the batch wraps around the end of the ring, and
    /// only then is `head` published. Returns how many were written (at most
    /// [`RingTx::staged_capacity`]; `0` if the ring is full). `overwrite` rings use
    /// [`RingTx::push_slice`] (the CPU path).
    ///
    /// Ordering follows the unsafe [`SlotCopy`] contract for every block, then a barrier
    /// and `head`. A failed copy leaves `head` unchanged and permanently fences this handle:
    /// `push` returns `NotReady`, `push_slice` returns zero, staged pushes return `Fenced`.
    /// Dropping/re-attaching does not drain remote writes; reset/drain the transport first.
    #[cfg_attr(feature = "isram", rp1_macros::isram)]
    pub fn push_slice_staged<C: SlotCopy>(
        &mut self,
        values: &[T],
        stage: &mut [u8],
        copy: &mut C,
    ) -> Result<usize, Staged<C::Error>> {
        if self.fenced {
            return Err(Staged::Fenced);
        }
        let slot = slot_size(size_of::<T>());
        if !aligned(stage.as_ptr()) || stage.len() < slot {
            return Err(Staged::Invalid);
        }
        if self.overflow == Overflow::Overwrite {
            return Ok(self.push_slice(values));
        }
        let want = values
            .len()
            .min(Self::staged_capacity(stage.len()))
            .min(u32::MAX as usize) as u32;
        let n = self.reject_free(want).min(want);
        if n == 0 {
            return Ok(0);
        }
        let base = stage.as_mut_ptr();
        for (k, v) in values[..n as usize].iter().enumerate() {
            // SAFETY: k < capacity, so the slot image lies inside `stage` (8-byte aligned).
            unsafe { stage_slot(base.add(k * slot), self.head.wrapping_add(k as u32), v) };
        }
        let first = n.min(self.mask + 1 - (self.head & self.mask));
        let parts = [(0, first), (first, n - first)];
        self.store_dropped();
        for &(k, m) in &parts {
            if m == 0 {
                continue;
            }
            let src = &stage[k as usize * slot..(k + m) as usize * slot];
            // SAFETY: the destination is the free slots `head + k ..` (reject mode: the reader
            // released them), contiguous inside the mapping `attach` was given.
            if let Err(e) =
                unsafe { copy.copy_to_remote(src, self.slot(self.head.wrapping_add(k))) }
            {
                self.fenced = true;
                return Err(Staged::Copy(e));
            }
        }
        barrier();
        self.head = self.head.wrapping_add(n);
        unsafe { st(self.data, HEAD, self.head) };
        Ok(n as usize)
    }
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
    ctrl: *mut u8,
    mask: u32,
    overflow: Overflow,
    tail: u32,
    /// Upper half of the 64-bit logical `tail` (overwrite only, see [`TAIL_HI`]).
    tail_hi: u32,
    /// Last `head` read from the local `data`; reread only when the ring looks empty.
    head_seen: u32,
    /// Shadow of the remote `waiting` sleep counter (odd = asleep).
    waiting: u32,
    _t: PhantomData<fn() -> T>,
}

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
        let index = self.tail.wrapping_add(k);
        let slot = self.slot(index);
        let (seq, word) = unsafe { (ld(slot, 0), ld(slot, 4)) };
        if seq != index {
            return Err(Error::Corrupt);
        }
        if word != size_of::<T>() as u32 {
            return Err(Error::Invalid);
        }
        Ok(unsafe { read_value::<T>(slot.add(8)) })
    }
    /// `Ok(None)` when empty. `Err(Lost(n))` (overwrite only) reports skipped messages; the
    /// reader has already moved to the oldest surviving slot, so just call `pop` again.
    #[cfg_attr(feature = "isram", rp1_macros::isram)]
    pub fn pop(&mut self) -> Result<Option<T>, Error> {
        let len = size_of::<T>() as u32;
        match self.overflow {
            Overflow::Reject => {
                if self.reject_available(1)? == 0 {
                    return Ok(None);
                }
                let value = self.read_slot(0)?;
                self.consume();
                Ok(Some(value))
            }
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
    pub fn tail(&self) -> u32 {
        self.tail
    }
    /// Rejected pushes counted by the writer (observational).
    pub fn dropped(&self) -> u32 {
        unsafe { ld(self.data, DROPPED) }
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

/// A SeqLock-protected block (protocol.md §5): `seq` u32, `flags` u32, then `T`.
///
/// Large `T` (tables, bulk buffers) are accessed word by word through [`BlockWriter`] and
/// [`BlockReader`] so that no copy of `T` is needed in RP1 memory.
pub struct Block<T> {
    base: *mut u8,
    seq: u32,
    _t: PhantomData<T>,
}
unsafe impl<T> Send for Block<T> {}
impl<T: Message> Block<T> {
    /// # Safety
    /// `base` is an 8-byte aligned mapping of `len >= block_size(size_of::<T>())` bytes that
    /// stays valid and is accessed only through volatile operations.
    pub unsafe fn attach(base: *mut u8, len: usize) -> Result<Self, Error> {
        if !aligned(base) || len < block_size(size_of::<T>()) {
            return Err(Error::Invalid);
        }
        Ok(Self {
            base,
            seq: unsafe { ld(base, 0) } & !1,
            _t: PhantomData,
        })
    }
    /// Words of `T` (rounded up).
    pub const WORDS: usize = size_of::<T>().div_ceil(4);
    /// Writer: replace the whole value.
    ///
    /// # Safety
    /// This handle is the declared sole writer (`writer = "rp1"` / `"linux"`).
    pub unsafe fn write(&mut self, value: &T) {
        let mut w = unsafe { self.begin_write() };
        unsafe { copy_to_mapping(w.data(), ptr::from_ref(value).cast(), size_of::<T>()) };
        w.commit();
    }
    /// Writer: open a write section (odd `seq`). Readers retry until [`BlockWriter::commit`].
    ///
    /// # Safety
    /// As [`Block::write`].
    pub unsafe fn begin_write(&mut self) -> BlockWriter<'_, T> {
        // writer=both transfers ownership between peers; the previous writer advanced seq.
        self.seq = unsafe { ld(self.base, 0) }.wrapping_add(1) | 1;
        unsafe { st(self.base, 0, self.seq) };
        barrier();
        BlockWriter { block: self }
    }
    /// Current SeqLock sequence (one aligned load). Cheap change detection for pollers: an
    /// unchanged even value means no write section ran since it was last seen. Over the RP1
    /// outbound window this is one PCIe round trip instead of a whole-value read.
    pub fn sequence(&self) -> u32 {
        unsafe { ld(self.base, 0) }
    }
    /// Reader: copy the whole value; `Err(NotReady)` if a write was in progress or overlapped.
    pub fn read(&self) -> Result<T, Error> {
        self.read_with(|r| unsafe { read_value::<T>(r.block.base.add(8)) })
    }
    /// Reader: run `f` inside one SeqLock read section.
    pub fn read_with<R>(&self, f: impl FnOnce(&BlockReader<'_, T>) -> R) -> Result<R, Error> {
        let seq = unsafe { ld(self.base, 0) };
        if seq & 1 != 0 {
            return Err(Error::NotReady);
        }
        barrier();
        let r = f(&BlockReader { block: self });
        barrier();
        if unsafe { ld(self.base, 0) } != seq {
            return Err(Error::NotReady);
        }
        Ok(r)
    }
    /// Raw word access without SeqLock (`writer = "both"`: caller synchronizes).
    ///
    /// # Safety
    /// The caller provides the synchronization the SeqLock would.
    pub unsafe fn raw(&self) -> BlockReader<'_, T> {
        BlockReader { block: self }
    }
}
pub struct BlockWriter<'a, T> {
    block: &'a mut Block<T>,
}
impl<T: Message> BlockWriter<'_, T> {
    fn data(&mut self) -> *mut u8 {
        unsafe { self.block.base.add(8) }
    }
    /// Store `words` at word index `at` of `T`.
    pub fn write_words(&mut self, at: usize, words: &[u32]) -> Result<(), Error> {
        if at
            .checked_add(words.len())
            .is_none_or(|end| end > Block::<T>::WORDS)
        {
            return Err(Error::Invalid);
        }
        let d = self.data();
        for (i, w) in words.iter().enumerate() {
            unsafe { st(d, (at + i) * 4, *w) };
        }
        Ok(())
    }
    /// Fill words `at..at+count` with `f(index)` without a staging buffer.
    pub fn fill(
        &mut self,
        at: usize,
        count: usize,
        mut f: impl FnMut(usize) -> u32,
    ) -> Result<(), Error> {
        if at
            .checked_add(count)
            .is_none_or(|end| end > Block::<T>::WORDS)
        {
            return Err(Error::Invalid);
        }
        let d = self.data();
        for i in at..at + count {
            unsafe { st(d, i * 4, f(i)) };
        }
        Ok(())
    }
    /// Base of `T` in the mapping, for specialised (e.g. burst) writers.
    ///
    /// # Safety
    /// Stay within `Block::<T>::WORDS` words and use only aligned volatile stores.
    pub unsafe fn as_ptr(&mut self) -> *mut u32 {
        self.data().cast()
    }
    /// End the write section (even `seq`).
    pub fn commit(self) {}
}
impl<T> Drop for BlockWriter<'_, T> {
    fn drop(&mut self) {
        barrier();
        self.block.seq = self.block.seq.wrapping_add(1);
        unsafe { st(self.block.base, 0, self.block.seq) };
        barrier();
    }
}
pub struct BlockReader<'a, T> {
    block: &'a Block<T>,
}
impl<T: Message> BlockReader<'_, T> {
    pub fn read_word(&self, at: usize) -> Result<u32, Error> {
        if at >= Block::<T>::WORDS {
            return Err(Error::Invalid);
        }
        Ok(unsafe { ld(self.block.base, 8 + at * 4) })
    }
    /// Load words `at..at+out.len()` of `T`.
    pub fn read_words(&self, at: usize, out: &mut [u32]) -> Result<(), Error> {
        if at
            .checked_add(out.len())
            .is_none_or(|end| end > Block::<T>::WORDS)
        {
            return Err(Error::Invalid);
        }
        for (i, word) in out.iter_mut().enumerate() {
            *word = unsafe { ld(self.block.base, 8 + (at + i) * 4) };
        }
        Ok(())
    }
    /// Base of `T` in the mapping, for specialised (e.g. burst) readers.
    ///
    /// # Safety
    /// Stay within `Block::<T>::WORDS` words and use only aligned volatile loads.
    pub unsafe fn as_ptr(&self) -> *const u32 {
        unsafe { self.block.base.add(8).cast() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate std;
    use std::vec;

    #[repr(C, align(8))]
    struct Mem<const N: usize>([u8; N]);
    fn mem<const N: usize>() -> std::boxed::Box<Mem<N>> {
        std::boxed::Box::new(Mem([0; N]))
    }

    #[test]
    fn reject_ring_full_wrap_and_resume() {
        let mut data = mem::<{ CONTROL_SIZE + 4 * 16 }>();
        let mut ctrl = mem::<CONTROL_SIZE>();
        let (d, c) = (data.0.as_mut_ptr(), ctrl.0.as_mut_ptr());
        let mut tx =
            unsafe { RingTx::<[u32; 2]>::attach(d, 192, c, 128, 4, Overflow::Reject) }.unwrap();
        let mut rx =
            unsafe { RingRx::<[u32; 2]>::attach(d, 192, c, 128, 4, Overflow::Reject) }.unwrap();
        assert_eq!(rx.pop(), Ok(None));
        for i in 0..4 {
            tx.push(&[i, !i]).unwrap();
        }
        assert_eq!(tx.push(&[9, 9]), Err(Error::Full));
        assert_eq!(tx.push(&[9, 9]), Err(Error::Full));
        // Spinning on Full stores nothing remotely; the count travels with the next push.
        assert_eq!((tx.dropped(), rx.dropped()), (2, 0));
        for round in 0..1000u32 {
            assert_eq!(rx.pop(), Ok(Some([round, !round])));
            tx.push(&[round + 4, !(round + 4)]).unwrap();
            assert_eq!(rx.dropped(), 2);
        }
        // A restarted reader resumes from the shared tail.
        let mut again =
            unsafe { RingRx::<[u32; 2]>::attach(d, 192, c, 128, 4, Overflow::Reject) }.unwrap();
        assert_eq!(again.pop(), Ok(Some([1000, !1000])));
        // Corruption of a slot sequence is detected.
        unsafe { st(d, CONTROL_SIZE + 16, 7) };
        assert_eq!(again.pop(), Err(Error::Corrupt));
    }

    /// Synchronous copy satisfying the ordering contract; can fail after a copied block.
    struct MemCopy {
        blocks: vec::Vec<(usize, usize)>,
        fail_at: Option<usize>,
    }
    unsafe impl SlotCopy for MemCopy {
        type Error = ();
        unsafe fn copy_to_remote(&mut self, src: &[u8], dst: *mut u8) -> Result<(), ()> {
            if self.fail_at == Some(self.blocks.len()) {
                return Err(());
            }
            self.blocks.push((dst as usize, src.len()));
            for (i, b) in src.iter().enumerate() {
                unsafe { dst.add(i).write_volatile(*b) };
            }
            Ok(())
        }
    }
    fn mem_copy() -> MemCopy {
        MemCopy {
            blocks: vec::Vec::new(),
            fail_at: None,
        }
    }
    #[repr(C, align(8))]
    struct Stage([u8; 256]);

    #[test]
    fn staged_batch_wraps_in_two_blocks_and_publishes_after_copy() {
        // [u32; 3]: 12 B payload, 24 B slots (4 B of padding).
        let mut data = mem::<{ CONTROL_SIZE + 8 * 24 }>();
        let mut ctrl = mem::<CONTROL_SIZE>();
        let (d, c) = (data.0.as_mut_ptr(), ctrl.0.as_mut_ptr());
        unsafe {
            st(d, HEAD, u32::MAX - 2);
            st(c, TAIL, u32::MAX - 2);
        }
        type M = [u32; 3];
        let len = CONTROL_SIZE + 8 * 24;
        let mut tx = unsafe { RingTx::<M>::attach(d, len, c, 128, 8, Overflow::Reject) }.unwrap();
        let mut rx = unsafe { RingRx::<M>::attach(d, len, c, 128, 8, Overflow::Reject) }.unwrap();
        // Garbage in the padding of the remote slots is overwritten with zeros.
        for i in 0..8 {
            unsafe { st(d, CONTROL_SIZE + i * 24 + 20, 0xdead_beef) };
        }
        let mut stage = Stage([0xa5; 256]);
        let values: vec::Vec<M> = (0..20u32).map(|i| [i, !i, i * 7]).collect();
        let mut copy = mem_copy();
        // head & 7 = 5: three slots up to the end of the ring, then two from slot 0.
        assert_eq!(RingTx::<M>::staged_capacity(256), 10);
        assert_eq!(
            tx.push_slice_staged(&values[..5], &mut stage.0, &mut copy),
            Ok(5)
        );
        assert_eq!(
            copy.blocks,
            [
                (d as usize + CONTROL_SIZE + 5 * 24, 3 * 24),
                (d as usize + CONTROL_SIZE, 2 * 24)
            ]
        );
        assert_eq!(unsafe { ld(d, HEAD) }, (u32::MAX - 2).wrapping_add(5));
        assert_eq!(unsafe { ld(d, CONTROL_SIZE + 5 * 24 + 20) }, 0);
        assert_eq!(unsafe { ld(d, CONTROL_SIZE + 5 * 24 + 4) }, 12);
        let mut out = [[0u32; 3]; 8];
        assert_eq!(rx.pop_into(&mut out), Ok(5));
        assert_eq!(out[..5], values[..5]);
        // Bounded by the free slots (8) and the stage (10 slots); wraps again (6 + 2).
        copy.blocks.clear();
        assert_eq!(
            tx.push_slice_staged(&values[5..], &mut stage.0, &mut copy),
            Ok(8)
        );
        assert_eq!(copy.blocks.len(), 2);
        assert_eq!(
            tx.push_slice_staged(&values[13..], &mut stage.0, &mut copy),
            Ok(0)
        );
        assert_eq!(rx.pop_into(&mut out), Ok(8));
        assert_eq!(out[..8], values[5..13]);
        // Mixed with CPU pushes on the same ring.
        assert_eq!(tx.push_slice(&values[13..15]), 2);
        assert_eq!(
            tx.push_slice_staged(&values[15..], &mut stage.0, &mut copy),
            Ok(5)
        );
        assert_eq!(rx.pop_into(&mut out), Ok(7));
        assert_eq!(out[..7], values[13..20]);
    }

    #[test]
    fn failed_copy_fences_all_publish_paths() {
        let mut data = mem::<{ CONTROL_SIZE + 4 * 24 }>();
        let mut ctrl = mem::<CONTROL_SIZE>();
        let (d, c) = (data.0.as_mut_ptr(), ctrl.0.as_mut_ptr());
        let mut tx =
            unsafe { RingTx::<[u32; 3]>::attach(d, data.0.len(), c, 128, 4, Overflow::Reject) }
                .unwrap();
        let mut stage = Stage([0; 256]);
        let mut copy = mem_copy();
        copy.fail_at = Some(0);
        assert_eq!(
            tx.push_slice_staged(&[[1, 2, 0]], &mut stage.0, &mut copy),
            Err(Staged::Copy(()))
        );
        assert!(tx.is_fenced());
        assert_eq!(tx.writable(), 0);
        assert_eq!(tx.free(), 0);
        copy.fail_at = None;
        assert_eq!(
            tx.push_slice_staged(&[[9, 8, 0]], &mut stage.0, &mut copy),
            Err(Staged::Fenced)
        );
        assert_eq!(tx.push(&[9, 8, 0]), Err(Error::NotReady));
        assert_eq!(tx.push_slice(&[[9, 8, 0]]), 0);
        assert_eq!(unsafe { ld(d, HEAD) }, 0);
        assert!(copy.blocks.is_empty());
    }

    #[test]
    fn second_wrap_block_failure_fences_even_with_equal_tails_and_padding() {
        let mut data = mem::<{ CONTROL_SIZE + 4 * 24 }>();
        let mut ctrl = mem::<CONTROL_SIZE>();
        let (d, c) = (data.0.as_mut_ptr(), ctrl.0.as_mut_ptr());
        unsafe {
            st(d, HEAD, 3);
            st(c, TAIL, 3);
        }
        let mut tx =
            unsafe { RingTx::<[u32; 3]>::attach(d, data.0.len(), c, 128, 4, Overflow::Reject) }
                .unwrap();
        let mut stage = Stage([0; 256]);
        let mut copy = mem_copy();
        copy.fail_at = Some(1);
        let values = [[1, 2, 0], [3, 4, 0]];
        assert_eq!(
            tx.push_slice_staged(&values, &mut stage.0, &mut copy),
            Err(Staged::Copy(()))
        );
        assert_eq!(copy.blocks.len(), 1);
        assert_eq!(unsafe { ld(d, HEAD) }, 3);
        // A late interior write can still land, but no retry/CPU push reuses its slot.
        unsafe {
            st(d, CONTROL_SIZE + 8, 99);
        }
        assert_eq!(tx.push_slice(&[[8, 9, 0]]), 0);
        assert_eq!(
            tx.push_slice_staged(&[[8, 9, 0]], &mut stage.0, &mut copy),
            Err(Staged::Fenced)
        );
        assert_eq!(unsafe { ld(d, CONTROL_SIZE + 8) }, 99);
    }

    #[test]
    fn staged_rejects_bad_stages_and_overwrite_uses_the_cpu_path() {
        let mut data = mem::<{ CONTROL_SIZE + 4 * 16 }>();
        let mut ctrl = mem::<CONTROL_SIZE>();
        let (d, c) = (data.0.as_mut_ptr(), ctrl.0.as_mut_ptr());
        let len = CONTROL_SIZE + 4 * 16;
        let mut tx = unsafe { RingTx::<u64>::attach(d, len, c, 128, 4, Overflow::Reject) }.unwrap();
        let mut stage = Stage([0; 256]);
        let mut copy = mem_copy();
        assert_eq!(
            tx.push_slice_staged(&[1], &mut stage.0[4..], &mut copy),
            Err(Staged::Invalid)
        );
        assert_eq!(
            tx.push_slice_staged(&[1], &mut stage.0[..15], &mut copy),
            Err(Staged::Invalid)
        );
        assert!(copy.blocks.is_empty());
        let mut ow =
            unsafe { RingTx::<u64>::attach(d, len, c, 128, 4, Overflow::Overwrite) }.unwrap();
        assert_eq!(
            ow.push_slice_staged(&[1, 2], &mut stage.0, &mut copy),
            Ok(2)
        );
        assert!(copy.blocks.is_empty());
    }

    #[test]
    fn batch_push_and_pop_publish_once() {
        let mut data = mem::<{ CONTROL_SIZE + 8 * 16 }>();
        let mut ctrl = mem::<CONTROL_SIZE>();
        let (d, c) = (data.0.as_mut_ptr(), ctrl.0.as_mut_ptr());
        unsafe {
            st(d, HEAD, u32::MAX - 2);
            st(c, TAIL, u32::MAX - 2);
        }
        let mut tx = unsafe { RingTx::<u64>::attach(d, 256, c, 128, 8, Overflow::Reject) }.unwrap();
        let mut rx = unsafe { RingRx::<u64>::attach(d, 256, c, 128, 8, Overflow::Reject) }.unwrap();
        // Reject-mode slot lengths are written once by the writer's attach.
        for i in 0..8 {
            assert_eq!(unsafe { ld(d, CONTROL_SIZE + i * 16 + 4) }, 8);
        }
        let values: vec::Vec<u64> = (0..20).collect();
        // Only the free slots are written; one head store covers the batch.
        assert_eq!(tx.push_slice(&values[..5]), 5);
        assert_eq!(unsafe { ld(d, HEAD) }, (u32::MAX - 2).wrapping_add(5));
        assert_eq!(tx.push_slice(&values[5..]), 3);
        assert_eq!(tx.push_slice(&values[8..]), 0);
        assert_eq!(tx.push(&99), Err(Error::Full));
        let mut out = [0u64; 3];
        assert_eq!(rx.pop_into(&mut out), Ok(3));
        assert_eq!(out, [0, 1, 2]);
        assert_eq!(unsafe { ld(c, TAIL) }, (u32::MAX - 2).wrapping_add(3));
        // The writer sees the released slots only through the stored tail.
        assert_eq!(tx.push_slice(&values[8..]), 3);
        let mut out = [0u64; 16];
        assert_eq!(rx.pop_into(&mut out), Ok(8));
        assert_eq!(out[..8], values[3..11]);
        assert_eq!(rx.pop_into(&mut out), Ok(0));
        assert_eq!(rx.pop(), Ok(None));
        // Single push/pop interleave with batches.
        tx.push(&50).unwrap();
        assert_eq!(tx.push_slice(&[51, 52]), 2);
        assert_eq!(rx.pop(), Ok(Some(50)));
        assert_eq!(rx.pop_into(&mut out[..1]), Ok(1));
        assert_eq!(out[0], 51);
        // A corrupt slot after a good one: the good one is returned, the error follows.
        assert_eq!(tx.push_slice(&[53, 54]), 2);
        let bad = rx.tail().wrapping_add(2);
        unsafe { st(d, CONTROL_SIZE + (bad & 7) as usize * 16, 7) };
        assert_eq!(rx.newest(), Ok(Some((bad, Err(Error::Corrupt)))));
        assert_eq!(rx.pop_into_report(&mut out), (2, Some(Error::Corrupt)));
        assert_eq!(out[..2], [52, 53]);
        // The slot becomes valid before another call: the first report retained the error.
        unsafe { st(d, CONTROL_SIZE + (bad & 7) as usize * 16, bad) };
        assert_eq!(rx.pop_into_report(&mut out), (1, None));
        assert_eq!(out[0], 54);
    }

    #[test]
    fn overwrite_pop_into_reports_lost() {
        let mut data = mem::<{ CONTROL_SIZE + 4 * 16 }>();
        let mut ctrl = mem::<CONTROL_SIZE>();
        let (d, c) = (data.0.as_mut_ptr(), ctrl.0.as_mut_ptr());
        let mut tx =
            unsafe { RingTx::<u64>::attach(d, 192, c, 128, 4, Overflow::Overwrite) }.unwrap();
        let mut rx =
            unsafe { RingRx::<u64>::attach(d, 192, c, 128, 4, Overflow::Overwrite) }.unwrap();
        assert_eq!(tx.push_slice(&[0, 1, 2, 3, 4, 5]), 6);
        let mut out = [0u64; 8];
        assert_eq!(rx.pop_into(&mut out), Err(Error::Lost(2)));
        assert_eq!(rx.pop_into(&mut out), Ok(1));
        assert_eq!(out[0], 2);
    }

    #[test]
    fn reject_ring_index_wraps_u32() {
        let mut data = mem::<{ CONTROL_SIZE + 2 * 16 }>();
        let mut ctrl = mem::<CONTROL_SIZE>();
        let (d, c) = (data.0.as_mut_ptr(), ctrl.0.as_mut_ptr());
        unsafe {
            st(d, HEAD, u32::MAX - 1);
            st(c, TAIL, u32::MAX - 1);
        }
        let mut tx = unsafe { RingTx::<u32>::attach(d, 160, c, 128, 2, Overflow::Reject) }.unwrap();
        let mut rx = unsafe { RingRx::<u32>::attach(d, 160, c, 128, 2, Overflow::Reject) }.unwrap();
        for i in 0..10 {
            tx.push(&i).unwrap();
            assert_eq!(rx.pop(), Ok(Some(i)));
        }
    }

    #[test]
    fn overwrite_ring_reports_lost_and_follows() {
        let mut data = mem::<{ CONTROL_SIZE + 4 * 16 }>();
        let mut ctrl = mem::<CONTROL_SIZE>();
        let (d, c) = (data.0.as_mut_ptr(), ctrl.0.as_mut_ptr());
        let mut tx =
            unsafe { RingTx::<u64>::attach(d, 192, c, 128, 4, Overflow::Overwrite) }.unwrap();
        let mut rx =
            unsafe { RingRx::<u64>::attach(d, 192, c, 128, 4, Overflow::Overwrite) }.unwrap();
        for i in 0..10u64 {
            tx.push(&i).unwrap();
        }
        assert_eq!(rx.pop(), Err(Error::Lost(6)));
        let got: vec::Vec<_> = core::iter::from_fn(|| rx.pop().unwrap()).collect();
        assert_eq!(got, [6, 7, 8, 9]);
        // A slot marked busy is not returned.
        tx.push(&10).unwrap();
        unsafe { st(d, CONTROL_SIZE + (10 & 3) * 16 + 4, BUSY | 8) };
        assert_eq!(rx.pop(), Err(Error::NotReady));
    }

    #[test]
    fn overwrite_stopped_reader_survives_full_index_lap() {
        // REVIEW2 #4: a reader stopped at tail=0 while the writer runs 2^32 pushes.
        let mut data = mem::<{ CONTROL_SIZE + 4 * 16 }>();
        let mut ctrl = mem::<CONTROL_SIZE>();
        let (d, c) = (data.0.as_mut_ptr(), ctrl.0.as_mut_ptr());
        unsafe { st(d, HEAD, u32::MAX - 3) };
        let mut tx =
            unsafe { RingTx::<u64>::attach(d, 192, c, 128, 4, Overflow::Overwrite) }.unwrap();
        let mut rx =
            unsafe { RingRx::<u64>::attach(d, 192, c, 128, 4, Overflow::Overwrite) }.unwrap();
        for i in 0..4u64 {
            tx.push(&i).unwrap();
        }
        assert_eq!(unsafe { (ld(d, HEAD), ld(d, HEAD_HI)) }, (0, 1));
        assert_eq!(rx.available(), u32::MAX);
        assert_eq!(rx.pop(), Err(Error::Lost(u32::MAX - 3)));
        let got: vec::Vec<_> = core::iter::from_fn(|| rx.pop().unwrap()).collect();
        assert_eq!(got, [0, 1, 2, 3]);
        assert_eq!(unsafe { (ld(c, TAIL), ld(c, TAIL_HI)) }, (0, 1));
        // The reader's lap survives a restart through the shared `tail_hi`.
        tx.push(&4).unwrap();
        let mut again =
            unsafe { RingRx::<u64>::attach(d, 192, c, 128, 4, Overflow::Overwrite) }.unwrap();
        assert_eq!(again.pop(), Ok(Some(4)));
        assert_eq!(again.pop(), Ok(None));
    }

    #[test]
    fn overwrite_lost_beyond_u32_is_reported_in_parts() {
        let mut data = mem::<{ CONTROL_SIZE + 4 * 16 }>();
        let mut ctrl = mem::<CONTROL_SIZE>();
        let (d, c) = (data.0.as_mut_ptr(), ctrl.0.as_mut_ptr());
        unsafe { st(d, HEAD, u32::MAX - 3) };
        unsafe { st(d, HEAD_HI, 1) };
        let mut tx =
            unsafe { RingTx::<u64>::attach(d, 192, c, 128, 4, Overflow::Overwrite) }.unwrap();
        let mut rx =
            unsafe { RingRx::<u64>::attach(d, 192, c, 128, 4, Overflow::Overwrite) }.unwrap();
        for i in 0..4u64 {
            tx.push(&i).unwrap();
        }
        // 2^33 pushed, depth 4 kept: 2^33 - 4 lost in total.
        let mut lost = 0u64;
        loop {
            match rx.pop() {
                Err(Error::Lost(n)) => lost += u64::from(n),
                Ok(Some(v)) => {
                    assert_eq!(v, 0);
                    break;
                }
                other => panic!("{other:?}"),
            }
        }
        assert_eq!(lost, (1u64 << 33) - 4);
    }

    #[test]
    fn overwrite_torn_head_reads_as_empty() {
        // `head` wrapped but `head_hi` not yet visible: the reader must not see 2^32 lost.
        let mut data = mem::<{ CONTROL_SIZE + 4 * 16 }>();
        let mut ctrl = mem::<CONTROL_SIZE>();
        let (d, c) = (data.0.as_mut_ptr(), ctrl.0.as_mut_ptr());
        unsafe {
            st(c, TAIL, u32::MAX);
            st(d, HEAD, 0);
        }
        let mut rx =
            unsafe { RingRx::<u64>::attach(d, 192, c, 128, 4, Overflow::Overwrite) }.unwrap();
        assert_eq!(rx.pop(), Ok(None));
        assert_eq!(rx.available(), 0);
    }

    fn wait_pair(
        depth: u32,
    ) -> (
        std::boxed::Box<Mem<4224>>,
        std::boxed::Box<Mem<CONTROL_SIZE>>,
    ) {
        assert!(CONTROL_SIZE + depth as usize * 24 <= 4224);
        (mem(), mem())
    }

    #[test]
    fn wait_handshake() {
        let (mut data, mut ctrl) = wait_pair(2);
        let (d, c) = (data.0.as_mut_ptr(), ctrl.0.as_mut_ptr());
        let mut tx = unsafe { RingTx::<u32>::attach(d, 160, c, 128, 2, Overflow::Reject) }.unwrap();
        let mut rx = unsafe { RingRx::<u32>::attach(d, 160, c, 128, 2, Overflow::Reject) }.unwrap();
        // Awake reader: no notification.
        assert!(!tx.wake_needed());
        // Arm, then a push: one notification for this sleep, not two.
        assert!(rx.prepare_wait());
        tx.push(&1).unwrap();
        assert!(tx.wake_needed());
        assert!(!tx.wake_needed());
        tx.push(&2).unwrap();
        assert!(!tx.wake_needed());
        rx.finish_wait();
        assert!(!tx.wake_needed());
        // Data present: arming does not sleep and leaves the reader awake.
        assert!(!rx.prepare_wait());
        assert!(!tx.wake_needed());
        assert_eq!(rx.pop().unwrap(), Some(1));
        assert_eq!(rx.pop().unwrap(), Some(2));
        // The next sleep gets a new value and so a new notification.
        assert!(rx.prepare_wait());
        assert!(tx.wake_needed());
        rx.finish_wait();
    }

    /// The race the handshake closes, in its sequentially consistent orders: a push between
    /// the reader's emptiness check and its arm is seen by the re-check; a push after the arm
    /// is notified. Either way the reader never sleeps on a non-empty ring unnotified.
    #[test]
    fn wait_with_never_sleeps_unnotified() {
        let (mut data, mut ctrl) = wait_pair(4);
        let (d, c) = (data.0.as_mut_ptr(), ctrl.0.as_mut_ptr());
        let mut tx = unsafe { RingTx::<u32>::attach(d, 224, c, 128, 4, Overflow::Reject) }.unwrap();
        let mut rx = unsafe { RingRx::<u32>::attach(d, 224, c, 128, 4, Overflow::Reject) }.unwrap();
        // Data already there.
        tx.push(&7).unwrap();
        assert_eq!(rx.wait_with(|| unreachable!()), Wait::Ready);
        assert_eq!(rx.pop().unwrap(), Some(7));
        // Push lands after the arm, before the sleep: the writer notifies.
        let mut notified = false;
        let r = rx.wait_with(|| {
            tx.push(&8).unwrap();
            notified = tx.wake_needed();
            notified
        });
        assert!(notified);
        assert_eq!(r, Wait::Woken);
        assert_eq!(rx.pop().unwrap(), Some(8));
        // Nothing happens: the bounded sleep times out and the reader is awake again.
        assert_eq!(rx.wait_with(|| false), Wait::Timeout);
        assert!(!tx.wake_needed());
    }

    /// Restarts on either side: a writer attaching while the reader sleeps notifies it; a
    /// reader that died asleep comes back awake, and its next sleep is notified again.
    #[test]
    fn wait_counter_survives_restarts() {
        let (mut data, mut ctrl) = wait_pair(2);
        let (d, c) = (data.0.as_mut_ptr(), ctrl.0.as_mut_ptr());
        let mut rx = unsafe { RingRx::<u32>::attach(d, 160, c, 128, 2, Overflow::Reject) }.unwrap();
        let mut tx = unsafe { RingTx::<u32>::attach(d, 160, c, 128, 2, Overflow::Reject) }.unwrap();
        assert!(rx.prepare_wait());
        assert!(tx.wake_needed());
        // Reader dies asleep; a new one attaches.
        let mut rx = unsafe { RingRx::<u32>::attach(d, 160, c, 128, 2, Overflow::Reject) }.unwrap();
        assert!(!tx.wake_needed());
        assert!(rx.prepare_wait());
        assert!(tx.wake_needed());
        // Writer restarts while the reader sleeps.
        let mut tx = unsafe { RingTx::<u32>::attach(d, 160, c, 128, 2, Overflow::Reject) }.unwrap();
        assert!(tx.wake_needed());
        assert!(!tx.wake_needed());
        rx.finish_wait();
    }

    /// Two rings, one notification: every ring is armed for the sleep, so either writer
    /// notifies; data on a later ring cancels the arms of the earlier ones.
    #[test]
    fn wait_any_arms_every_ring() {
        let (mut d0, mut c0) = wait_pair(2);
        let (mut d1, mut c1) = wait_pair(2);
        let (d0, c0) = (d0.0.as_mut_ptr(), c0.0.as_mut_ptr());
        let (d1, c1) = (d1.0.as_mut_ptr(), c1.0.as_mut_ptr());
        let ow = Overflow::Reject;
        let mut tx0 = unsafe { RingTx::<u32>::attach(d0, 160, c0, 128, 2, ow) }.unwrap();
        let mut rx0 = unsafe { RingRx::<u32>::attach(d0, 160, c0, 128, 2, ow) }.unwrap();
        let mut tx1 = unsafe { RingTx::<u64>::attach(d1, 160, c1, 128, 2, ow) }.unwrap();
        let mut rx1 = unsafe { RingRx::<u64>::attach(d1, 160, c1, 128, 2, ow) }.unwrap();
        // The second ring's writer notifies while both are armed.
        let r = wait_any_with(&mut [&mut rx0, &mut rx1], || {
            tx1.push(&5).unwrap();
            tx1.wake_needed()
        });
        assert_eq!(r, Wait::Woken);
        assert!(!tx0.wake_needed() && !tx1.wake_needed(), "all disarmed");
        assert_eq!(
            wait_any_with(&mut [&mut rx0, &mut rx1], || unreachable!()),
            Wait::Ready
        );
        assert_eq!(rx1.pop().unwrap(), Some(5));
        // A message that arrives on ring 1 between the first check and its arm: the re-check
        // cancels the sleep and ring 0's arm is undone.
        struct Arrives;
        impl Waiter for Arrives {
            fn available(&self) -> u32 {
                0
            }
            fn prepare_wait(&mut self) -> bool {
                false
            }
            fn finish_wait(&mut self) {}
        }
        assert_eq!(
            wait_any_with(&mut [&mut rx0, &mut Arrives], || unreachable!()),
            Wait::Ready
        );
        assert!(
            !tx0.wake_needed(),
            "ring 0 disarmed after the cancelled sleep"
        );
        // Ring 0's writer notifies too.
        let r = wait_any_with(&mut [&mut rx0, &mut rx1], || {
            tx0.push(&7).unwrap();
            tx0.wake_needed()
        });
        assert_eq!(r, Wait::Woken);
        assert_eq!(rx0.pop().unwrap(), Some(7));
    }

    /// Counter wrap: after the notified sleep, 2^31 - 1 more sleeps bring the counter back to
    /// the same odd value. That sleep is not notified (documented on [`WAITING`]); the reader's
    /// bounded re-check still finds the data.
    #[test]
    fn wait_counter_wrap_can_alias() {
        let (mut data, mut ctrl) = wait_pair(2);
        let (d, c) = (data.0.as_mut_ptr(), ctrl.0.as_mut_ptr());
        let mut tx = unsafe { RingTx::<u32>::attach(d, 160, c, 128, 2, Overflow::Reject) }.unwrap();
        let mut rx = unsafe { RingRx::<u32>::attach(d, 160, c, 128, 2, Overflow::Reject) }.unwrap();
        assert!(rx.prepare_wait());
        assert!(tx.wake_needed());
        rx.finish_wait();
        let w = unsafe { ld(c, WAITING) };
        // 2^31 - 1 more sleeps, 2 steps each.
        unsafe { st(c, WAITING, w.wrapping_add(u32::MAX - 1)) };
        let mut rx = unsafe { RingRx::<u32>::attach(d, 160, c, 128, 2, Overflow::Reject) }.unwrap();
        let r = rx.wait_with(|| {
            tx.push(&1).unwrap();
            assert!(!tx.wake_needed(), "aliased sleep is not notified");
            false
        });
        assert_eq!(r, Wait::Timeout);
        assert_eq!(rx.wait_with(|| unreachable!()), Wait::Ready);
        assert_eq!(rx.pop().unwrap(), Some(1));
    }

    /// Exhaustive interleaving model of the handshake (protocol.md §2.3) on the real code.
    ///
    /// Each side's stores into the peer's memory are posted: the writer writes a private copy
    /// of `data` and the reader a private copy of `ctrl` (`shadow`); a store reaches the peer's
    /// copy (`real`) only when it lands. Landing is in order, so a path lands as a whole: either
    /// spontaneously at any point, or when its own side reads back through the same window (a
    /// non-posted read does not pass earlier posted writes). Every load in the ring code is a
    /// scheduling point (`ld` calls [`before_load`]); the two sides run on threads that hand a
    /// single token back and forth (so there is no data race), and a depth-first search over the
    /// choices "run writer / run reader / land data / land ctrl" visits every schedule.
    pub(super) mod model {
        extern crate std;
        use std::cell::RefCell;
        use std::sync::{Arc, Condvar, Mutex};
        use std::vec::Vec;

        /// A posted-write path: what the writing side sees and what the peer sees.
        pub struct Path {
            pub shadow: usize,
            pub real: usize,
            pub len: usize,
        }
        impl Path {
            fn pending(&self) -> bool {
                // SAFETY: both buffers live for the run; only the token holder touches them.
                unsafe {
                    std::slice::from_raw_parts(self.shadow as *const u8, self.len)
                        != std::slice::from_raw_parts(self.real as *const u8, self.len)
                }
            }
            fn land(&self) {
                unsafe {
                    core::ptr::copy_nonoverlapping(
                        self.shadow as *const u8,
                        self.real as *mut u8,
                        self.len,
                    )
                }
            }
        }
        #[derive(Default)]
        struct State {
            running: Option<usize>,
            parked: [bool; 2],
            /// Asleep: not runnable until the other side is done.
            asleep: [bool; 2],
            done: [bool; 2],
        }
        pub struct Sched {
            state: Mutex<State>,
            cv: Condvar,
            pub paths: [Path; 2],
        }
        std::thread_local!(static CURRENT: RefCell<Option<(Arc<Sched>, usize)>> = const { RefCell::new(None) });

        /// Called by `ld` before every load; a no-op outside a model run.
        pub fn before_load(addr: *const u8) {
            let Some((s, me)) = CURRENT.with(|c| c.borrow().clone()) else {
                return;
            };
            s.park(me);
            for p in &s.paths {
                if (p.shadow..p.shadow + p.len).contains(&(addr as usize)) {
                    p.land();
                }
            }
        }
        impl Sched {
            pub fn new(paths: [Path; 2]) -> Arc<Self> {
                Arc::new(Self {
                    state: Mutex::new(State::default()),
                    cv: Condvar::new(),
                    paths,
                })
            }
            fn park(&self, me: usize) {
                self.park_as(me, false);
            }
            fn park_as(&self, me: usize, asleep: bool) {
                let mut g = self.state.lock().unwrap();
                g.parked[me] = true;
                g.asleep[me] = asleep;
                g.running = None;
                self.cv.notify_all();
                while g.running != Some(me) {
                    g = self.cv.wait(g).unwrap();
                }
                g.parked[me] = false;
            }
            /// Sleep (inside side `me`) until the other side has finished: a notification
            /// would wake it, and none can come later.
            pub fn sleep(&self, me: usize) {
                self.park_as(me, true);
            }
            /// Run `f` as side `me` (0 = writer, 1 = reader) under the scheduler.
            pub fn side<R>(self: &Arc<Self>, me: usize, f: impl FnOnce() -> R) -> R {
                CURRENT.with(|c| *c.borrow_mut() = Some((self.clone(), me)));
                self.park(me);
                let r = f();
                CURRENT.with(|c| *c.borrow_mut() = None);
                let mut g = self.state.lock().unwrap();
                g.done[me] = true;
                g.running = None;
                self.cv.notify_all();
                r
            }
            /// Drive both sides to completion; `choose(n)` picks one of `n` enabled steps.
            pub fn drive(&self, mut choose: impl FnMut(usize) -> usize, trace: &mut Vec<usize>) {
                loop {
                    let mut g = self.state.lock().unwrap();
                    while g.running.is_some() || (0..2).any(|t| !g.done[t] && !g.parked[t]) {
                        g = self.cv.wait(g).unwrap();
                    }
                    // 0..2: run that side; 2..4: land that path.
                    let steps: Vec<usize> = (0..2)
                        .filter(|&t| g.parked[t] && !(g.asleep[t] && !g.done[1 - t]))
                        .chain((0..2).filter(|&p| self.paths[p].pending()).map(|p| p + 2))
                        .collect();
                    if (0..2).all(|t| g.done[t]) {
                        return;
                    }
                    let step = steps[choose(steps.len())];
                    trace.push(step);
                    if step >= 2 {
                        self.paths[step - 2].land();
                    } else {
                        g.running = Some(step);
                        self.cv.notify_all();
                    }
                }
            }
        }

        /// Depth-first search over every sequence of choices `run` makes; returns the count.
        pub fn explore(mut run: impl FnMut(&mut dyn FnMut(usize) -> usize)) -> usize {
            let mut prefix: Vec<usize> = Vec::new();
            let mut runs = 0;
            loop {
                let mut taken: Vec<(usize, usize)> = Vec::new();
                run(&mut |n| {
                    let c = prefix.get(taken.len()).copied().unwrap_or(0);
                    taken.push((c, n));
                    c
                });
                runs += 1;
                while let Some((c, n)) = taken.pop() {
                    if c + 1 < n {
                        taken.push((c + 1, n));
                        break;
                    }
                }
                if taken.is_empty() {
                    return runs;
                }
                prefix = taken.into_iter().map(|(c, _)| c).collect();
            }
        }
    }

    /// The handshake never lets the reader sleep on a published message unnotified, and never
    /// notifies before the message has landed, in any interleaving of one `push` +
    /// `wake_needed` against one `wait_with`. Fails if the reader's re-check or either
    /// readback is removed (each gives a schedule where the reader sleeps unnotified).
    #[test]
    fn wait_handshake_all_interleavings() {
        use model::{Path, Sched, explore};
        const LEN: usize = CONTROL_SIZE + 4 * 16;
        // prior = 0: fresh ring; 1: the reader already had one notified sleep and a message.
        for prior in 0..2 {
            let runs = explore(|choose| {
                let (mut data, mut data_shadow) = (mem::<LEN>(), mem::<LEN>());
                let (mut ctrl, mut ctrl_shadow) = (mem::<CONTROL_SIZE>(), mem::<CONTROL_SIZE>());
                let (dr, ds) = (data.0.as_mut_ptr(), data_shadow.0.as_mut_ptr());
                let (cr, cs) = (ctrl.0.as_mut_ptr(), ctrl_shadow.0.as_mut_ptr());
                let head_real = dr as usize;
                let ow = Overflow::Reject;
                let mut tx = unsafe { RingTx::<u32>::attach(ds, LEN, cr, 128, 4, ow) }.unwrap();
                let mut rx = unsafe { RingRx::<u32>::attach(dr, LEN, cs, 128, 4, ow) }.unwrap();
                let sync = || unsafe {
                    core::ptr::copy_nonoverlapping(ds, dr, LEN);
                    core::ptr::copy_nonoverlapping(cs, cr, CONTROL_SIZE);
                };
                if prior == 1 {
                    assert!(rx.prepare_wait());
                    sync();
                    tx.push(&0).unwrap();
                    assert!(tx.wake_needed());
                    sync();
                    rx.finish_wait();
                    assert_eq!(rx.pop().unwrap(), Some(0));
                    sync();
                }
                let sched = Sched::new([
                    Path {
                        shadow: ds as usize,
                        real: dr as usize,
                        len: LEN,
                    },
                    Path {
                        shadow: cs as usize,
                        real: cr as usize,
                        len: CONTROL_SIZE,
                    },
                ]);
                let mut trace = vec![];
                let notified_now = core::sync::atomic::AtomicBool::new(false);
                let (notified, outcome) = std::thread::scope(|s| {
                    let w = s.spawn(|| {
                        sched.side(0, || {
                            tx.push(&1).unwrap();
                            let notify = tx.wake_needed();
                            notified_now.store(notify, core::sync::atomic::Ordering::Relaxed);
                            // The notification must not overtake the message.
                            let landed = unsafe { ld(head_real as *const u8, HEAD) } == tx.head();
                            (notify, landed)
                        })
                    });
                    let r = s.spawn(|| {
                        sched.side(1, || {
                            rx.wait_with(|| {
                                sched.sleep(1);
                                notified_now.load(core::sync::atomic::Ordering::Relaxed)
                            })
                        })
                    });
                    sched.drive(choose, &mut trace);
                    (w.join().unwrap(), r.join().unwrap())
                });
                let (notified, landed) = notified;
                assert_eq!(outcome == Wait::Woken, notified && outcome != Wait::Ready);
                assert!(
                    outcome == Wait::Ready || notified,
                    "lost wakeup: reader sleeps on a published message (prior={prior}, \
                     schedule={trace:?}; 0 writer, 1 reader, 2 land data, 3 land ctrl)"
                );
                assert!(
                    !notified || landed,
                    "notified before head landed (prior={prior}, schedule={trace:?})"
                );
            });
            assert!(runs > 10, "runs={runs}");
        }
    }

    #[test]
    fn header_and_block() {
        let mut h = mem::<64>();
        let hp = h.0.as_mut_ptr();
        let mut hdr = unsafe { HeaderInit::begin(hp, 0) };
        hdr.write(&[5; 32], 3, 0);
        assert_eq!(
            unsafe { Header::open(hp, &[5; 32]) }.err(),
            Some(Error::NotReady)
        );
        hdr.publish();
        let l = unsafe { Header::open(hp, &[5; 32]) }.unwrap();
        assert_eq!((l.generation(), l.regions()), (2, 3));
        assert_eq!(
            unsafe { Header::open(hp, &[6; 32]) }.err(),
            Some(Error::Mismatch)
        );
        let again = unsafe { HeaderInit::begin(hp, 2) };
        assert_eq!(l.check(), Err(Error::Reset));
        again.publish();
        assert_eq!(
            unsafe { Header::open(hp, &[5; 32]) }.unwrap().generation(),
            4
        );

        let mut b = mem::<{ 8 + 24 }>();
        let bp = b.0.as_mut_ptr();
        let mut w = unsafe { Block::<[u32; 5]>::attach(bp, 32) }.unwrap();
        let r = unsafe { Block::<[u32; 5]>::attach(bp, 32) }.unwrap();
        unsafe { w.write(&[1, 2, 3, 4, 5]) };
        assert_eq!(r.read(), Ok([1, 2, 3, 4, 5]));
        {
            let mut s = unsafe { w.begin_write() };
            s.fill(0, 5, |i| i as u32 * 10).unwrap();
            assert!(s.write_words(4, &[1, 2]).is_err());
            assert_eq!(r.read(), Err(Error::NotReady));
        }
        let mut out = [0; 3];
        r.read_with(|x| x.read_words(2, &mut out)).unwrap().unwrap();
        assert_eq!(out, [20, 30, 40]);
        r.read_with(|x| x.read_words(1, &mut out)).unwrap().unwrap();
        assert_eq!(out, [10, 20, 30]);
        assert_eq!(unsafe { ld(bp, 0) }, 4);
        let seen = r.sequence();
        assert_eq!(seen & 1, 0);
        let mut other = unsafe { Block::<[u32; 5]>::attach(bp, 32) }.unwrap();
        unsafe {
            other.write(&[5; 5]);
            w.write(&[6; 5]);
        }
        assert_eq!(unsafe { ld(bp, 0) }, 8);
        assert_ne!(r.sequence(), seen);
    }

    #[test]
    fn invalid_mapping_and_odd_message_size() {
        let mut data = mem::<160>();
        let mut ctrl = mem::<128>();
        let d = data.0.as_mut_ptr();
        let c = ctrl.0.as_mut_ptr();
        assert!(
            unsafe { RingTx::<u32>::attach(d.add(4), 156, c, 128, 2, Overflow::Reject) }.is_err()
        );
        assert!(unsafe { RingTx::<u32>::attach(d, 160, c, 128, 3, Overflow::Reject) }.is_err());
        assert!(unsafe { RingTx::<u32>::attach(d, 159, c, 128, 2, Overflow::Reject) }.is_err());
        let mut tx =
            unsafe { RingTx::<[u8; 3]>::attach(d, 160, c, 128, 2, Overflow::Reject) }.unwrap();
        let mut rx =
            unsafe { RingRx::<[u8; 3]>::attach(d, 160, c, 128, 2, Overflow::Reject) }.unwrap();
        tx.push(&[11, 22, 33]).unwrap();
        assert_eq!(rx.pop(), Ok(Some([11, 22, 33])));
        let header = unsafe { HeaderInit::begin(d, u32::MAX) };
        assert_eq!(header.generation(), 2);
    }
}
