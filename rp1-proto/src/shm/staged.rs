//! The DMA ring producer: slot images staged in ordinary memory, copied by a [`SlotCopy`]
//! engine, then published with one `head` store.
use super::access::{aligned, st};
use super::{CONTROL_SIZE, HEAD, Overflow, RingTx, slot_size};
use crate::{Message, barrier};
use core::{
    mem::{MaybeUninit, align_of, size_of},
    ptr,
};

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

/// Copies the first `n` slot images of `stage` to the ring slots from index `head` on (one
/// block, or two when they wrap around the end of the ring), then publishes `head + n`.
/// `ring` is the `data` mapping, the index mask and the slot size: the message type is not
/// needed, so all rings share this code. On `Err` nothing was published and the caller
/// must fence the ring.
///
/// # Safety
/// `data` is a ring's writable `data` mapping in which the `n` slots from `head` on are free.
#[cfg_attr(feature = "isram", rp1_macros::isram)]
unsafe fn commit_slots<C: SlotCopy>(
    copy: &mut C,
    stage: &[u8],
    (data, mask, size): (*mut u8, u32, usize),
    head: &mut u32,
    n: u32,
) -> Result<(), C::Error> {
    let first = n.min(mask + 1 - (*head & mask));
    for (k, m) in [(0, first), (first, n - first)] {
        if m != 0 {
            let src = &stage[k as usize * size..(k + m) as usize * size];
            let slot = CONTROL_SIZE + (head.wrapping_add(k) & mask) as usize * size;
            unsafe { copy.copy_to_remote(src, data.add(slot))? };
        }
    }
    barrier();
    *head = head.wrapping_add(n);
    unsafe { st(data, HEAD, *head) };
    Ok(())
}

/// Error of [`RingTx::push_slice_staged`]. No messages were published by the failed call.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Staged<E> {
    /// The stage is not 8-byte aligned or smaller than one slot.
    Invalid,
    /// The copy failed.
    Copy(E),
    /// An earlier copy failed; this writer cannot publish again.
    Fenced,
    /// The shm window is gone (PERSTn) and nothing was accessed. Only endpoints that check the
    /// window status (rp1-link) return it.
    Reset,
}

/// Writes the header and padding shared by copied and generated slot images.
///
/// # Safety
/// `dst` is 8-byte aligned and `slot_size(size)` bytes are writable.
#[inline(always)]
unsafe fn stage_header(dst: *mut u8, index: u32, size: usize) {
    // Volatile stores avoid an out-of-line memset running from shared SRAM on RP1.
    unsafe {
        dst.cast::<u32>().write_volatile(index);
        dst.add(4).cast::<u32>().write_volatile(size as u32);
        for i in 8 + size..slot_size(size) {
            dst.add(i).write_volatile(0);
        }
    }
}

/// Writes the slot image of message `index` at `dst`.
///
/// # Safety
/// `dst` is 8-byte aligned and `slot_size(size_of::<T>())` bytes are writable.
#[inline(always)]
unsafe fn stage_slot<T: Message>(dst: *mut u8, index: u32, value: &T) {
    // Volatile stores avoid an out-of-line memcpy running from shared SRAM on RP1.
    let size = size_of::<T>();
    unsafe {
        stage_header(dst, index, size);
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
    }
}

impl<T: Message> RingTx<T> {
    /// A copy failed; only an external reset/drain can make destination reuse safe.
    pub fn is_fenced(&self) -> bool {
        self.fenced
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
        let base = stage.as_mut_ptr();
        for (k, v) in values[..n as usize].iter().enumerate() {
            // SAFETY: k < capacity, so the slot image lies inside `stage` (8-byte aligned).
            unsafe { stage_slot(base.add(k * slot), self.head.wrapping_add(k as u32), v) };
        }
        self.commit_staged(n, stage, copy)
    }
    /// Generate payloads into staged slot images, then copy and publish as in
    /// [`Self::push_slice_staged`], following the [callback contract](Self::push_with).
    /// Stage payloads guarantee only 8-byte alignment; types aligned above that use one
    /// aligned temporary. Stage bytes are ordinary initialized memory, not a Device mapping.
    /// The library writes sequence, length and zero padding. All callbacks finish before
    /// copying begins. See `docs/dma.md` for the DMA ordering contract.
    ///
    /// Stage validation also applies to overwrite rings, which delegate to [`Self::push_with`].
    /// No callback runs when fenced, full, or stage validation fails.
    /// Copy errors fence all publication paths as for [`Self::push_slice_staged`].
    ///
    /// # Panics
    /// A callback panic leaves a reject ring's head unchanged and starts no copy.
    /// Overwrite rings may have published earlier messages.
    #[cfg_attr(feature = "isram", rp1_macros::isram)]
    pub fn push_with_staged<C: SlotCopy>(
        &mut self,
        count: usize,
        stage: &mut [u8],
        copy: &mut C,
        mut fill: impl FnMut(usize, &mut T),
    ) -> Result<usize, Staged<C::Error>> {
        if self.fenced {
            return Err(Staged::Fenced);
        }
        let (size, slot) = (size_of::<T>(), slot_size(size_of::<T>()));
        if !aligned(stage.as_ptr()) || stage.len() < slot {
            return Err(Staged::Invalid);
        }
        if self.overflow == Overflow::Overwrite {
            return Ok(self.push_with(count, fill));
        }
        let want = count
            .min(Self::staged_capacity(stage.len()))
            .min(u32::MAX as usize) as u32;
        let n = self.reject_free(want).min(want);
        let base = stage.as_mut_ptr();
        if align_of::<T>() <= 8 {
            for k in 0..n as usize {
                // SAFETY: k < capacity, so the slot image lies inside `stage`. The stage and
                // the slot size are multiples of 8, hence the payload at +8 is aligned for T;
                // stage is a `&mut [u8]`, so every byte is initialized, and Message permits
                // every bit pattern. The borrow ends before header and padding stores.
                unsafe {
                    let dst = base.add(k * slot);
                    fill(k, &mut *dst.add(8).cast::<T>());
                    stage_header(dst, self.head.wrapping_add(k as u32), size);
                }
            }
        } else if n != 0 {
            // SAFETY: Message permits zero bits, including for highly aligned types.
            let mut value = unsafe { MaybeUninit::<T>::zeroed().assume_init() };
            for k in 0..n as usize {
                fill(k, &mut value);
                // SAFETY: as above; the slot image lies inside the 8-byte aligned `stage`.
                unsafe { stage_slot(base.add(k * slot), self.head.wrapping_add(k as u32), &value) };
            }
        }
        self.commit_staged(n, stage, copy)
    }
    /// Commit fully initialized slot images. Both staged APIs share failure and ordering rules.
    #[inline(always)]
    fn commit_staged<C: SlotCopy>(
        &mut self,
        n: u32,
        stage: &[u8],
        copy: &mut C,
    ) -> Result<usize, Staged<C::Error>> {
        if n == 0 {
            return Ok(0);
        }
        self.store_dropped();
        let ring = (self.data, self.mask, slot_size(size_of::<T>()));
        // SAFETY: the destination is the free slots from `head` on (reject mode: the reader
        // released them), inside the mapping `attach` was given.
        match unsafe { commit_slots(copy, stage, ring, &mut self.head, n) } {
            Ok(()) => Ok(n as usize),
            Err(e) => {
                self.fenced = true;
                Err(Staged::Copy(e))
            }
        }
    }
}
