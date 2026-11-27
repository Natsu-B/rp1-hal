//! Fixed slots in exclusively reserved shared SRAM.

use super::{BusAddr, Error};
use core::{cell::Cell, ptr::NonNull, slice};

/// A fixed number of equal-sized local DMA buffers.
///
/// Construct this once from a linker-reserved shared SRAM allocation. The pool's bookkeeping
/// is CPU-local. A slot stays claimed if its lease is forgotten or quarantined.
pub struct Pool<const SLOTS: usize> {
    base: NonNull<u8>,
    slot_len: usize,
    states: [Cell<SlotState>; SLOTS],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SlotState {
    Free,
    Claimed,
    Engine,
}

impl<const SLOTS: usize> Pool<SLOTS> {
    /// Takes exclusive ownership of the reservation for this firmware epoch.
    ///
    /// The backing allocation must be in the RP1 shared SRAM, outside all linker/runtime,
    /// cross-core and host reservations. Prefer the generated `p.dma_pool`. A custom pool
    /// can use a one-time unsafe borrow of a `#[link_section = ".shared"]` static, with
    /// exclusivity across all processors and bus masters.
    /// The backing slice must divide evenly into slots. Align each slot base to 16 bytes
    /// for 128-bit beats. Narrower alignment selects smaller beats.
    pub fn new(memory: &'static mut [u8]) -> Result<Self, Error> {
        let slot_len = memory.len().checked_div(SLOTS).ok_or(Error::Invalid)?;
        if slot_len == 0 || slot_len * SLOTS != memory.len() {
            return Err(Error::Invalid);
        }
        sram(memory.as_ptr() as usize, memory.len())?;
        Ok(Self::from_memory(memory, slot_len))
    }

    /// # Safety
    /// Generated resources only. The pointer must identify the exclusive static reservation
    /// satisfying the requirements of [`Self::new`].
    #[doc(hidden)]
    pub unsafe fn __from_declared<const BYTES: usize>(memory: *mut [[u8; BYTES]; SLOTS]) -> Self {
        // SAFETY: the owning resource constructor grants the only borrow of the reservation.
        Self::from_memory(unsafe { (&mut *memory).as_flattened_mut() }, BYTES)
    }

    fn from_memory(memory: &'static mut [u8], slot_len: usize) -> Self {
        Self {
            base: NonNull::from(memory).cast(),
            slot_len,
            states: [const { Cell::new(SlotState::Free) }; SLOTS],
        }
    }

    /// Borrows the prefix of a slot. The unused bytes stay reserved by the same lease.
    pub fn lease(&self, slot: usize, len: usize) -> Result<Lease<'_>, Error> {
        let state = self.states.get(slot).ok_or(Error::Invalid)?;
        self.check_len(len)?;
        if state.get() != SlotState::Free {
            return Err(Error::Busy);
        }
        Ok(self.claim(slot, len))
    }

    /// Borrows all slots at the same length. The pool is unchanged if any slot is busy.
    pub fn leases(&self, len: usize) -> Result<[Lease<'_>; SLOTS], Error> {
        self.check_len(len)?;
        if self
            .states
            .iter()
            .any(|state| state.get() != SlotState::Free)
        {
            return Err(Error::Busy);
        }
        Ok(core::array::from_fn(|slot| self.claim(slot, len)))
    }

    fn check_len(&self, len: usize) -> Result<(), Error> {
        if len == 0 || len > self.slot_len {
            return Err(Error::Invalid);
        }
        Ok(())
    }

    fn claim(&self, slot: usize, len: usize) -> Lease<'_> {
        let state = &self.states[slot];
        state.set(SlotState::Claimed);
        // Slot index, multiplication and addition are bounded by the validated reservation.
        let ptr = self
            .base
            .map_addr(|base| base.saturating_add(slot * self.slot_len));
        Lease {
            ptr,
            offset: 0,
            len,
            capacity: self.slot_len,
            state,
        }
    }
}

pub(super) fn sram(address: usize, len: usize) -> Result<BusAddr, Error> {
    if len == 0 {
        return Err(Error::Invalid);
    }
    let end = address.checked_add(len).ok_or(Error::NotDmaVisible)?;
    if address < 0x2000_0000 || end > 0x2001_0000 {
        return Err(Error::NotDmaVisible);
    }
    BusAddr::from_proc(address)
}

/// Exclusive access to a pool slot. DMA borrows the lease and blocks CPU access until
/// the transfer proves quiescence. A forgotten transfer keeps the slot reserved all epoch.
#[derive(Debug)]
pub struct Lease<'a> {
    ptr: NonNull<u8>,
    offset: usize,
    len: usize,
    capacity: usize,
    state: &'a Cell<SlotState>,
}

impl Lease<'_> {
    /// Select the number of bytes at the start of this slot to transfer.
    pub fn set_len(&mut self, len: usize) -> Result<(), Error> {
        self.set_range(0, len)
    }

    /// Select a nonempty range within the slot, keeping the rest of the slot reserved.
    pub fn set_range(&mut self, offset: usize, len: usize) -> Result<(), Error> {
        if self.is_quarantined() {
            return Err(Error::Busy);
        }
        if len == 0
            || offset
                .checked_add(len)
                .is_none_or(|end| end > self.capacity)
        {
            return Err(Error::Invalid);
        }
        self.offset = offset;
        self.len = len;
        Ok(())
    }

    fn data(&self) -> *mut u8 {
        self.ptr.as_ptr().wrapping_add(self.offset)
    }

    /// The selected range, or an empty slice while the engine owns the slot.
    pub fn as_slice(&self) -> &[u8] {
        if self.is_quarantined() {
            return &[];
        }
        // SAFETY: this lease owns the slot and CPU access cannot overlap a live DMA transfer.
        unsafe { slice::from_raw_parts(self.data(), self.len) }
    }

    /// The selected range, or an empty slice while the engine owns the slot.
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        if self.is_quarantined() {
            return &mut [];
        }
        // SAFETY: as for as_slice, with exclusive access to the lease.
        unsafe { slice::from_raw_parts_mut(self.data(), self.len) }
    }

    /// True if a lost transfer or an unconfirmed stop left the slot with the engine.
    pub fn is_quarantined(&self) -> bool {
        self.state.get() == SlotState::Engine
    }

    pub(super) fn hand_to_engine(&mut self) -> EngineClaim<'_> {
        self.state.set(SlotState::Engine);
        EngineClaim(self.state)
    }

    pub(super) fn address(&self) -> Result<BusAddr, Error> {
        if self.is_quarantined() {
            return Err(Error::Busy);
        }
        sram(self.data() as usize, self.len)
    }

    pub(super) fn len(&self) -> u32 {
        self.len as u32
    }
}

// The claim borrows its lease exclusively and has no automatic release. Forgetting it
// leaves the persistent slot state in Engine even after Rust ends the borrow.
pub(super) struct EngineClaim<'a>(&'a Cell<SlotState>);

impl EngineClaim<'_> {
    // Only the originating transfer may return the slot after quiescence, or if
    // admission failed before enable. Controller recovery cannot return claims.
    pub(super) fn return_to_cpu(&mut self) {
        self.0.set(SlotState::Claimed);
    }
}

impl Drop for Lease<'_> {
    fn drop(&mut self) {
        if !self.is_quarantined() {
            self.state.set(SlotState::Free);
        }
    }
}

#[cfg(test)]
pub(super) fn fake_pool<const SLOTS: usize>() -> Pool<SLOTS> {
    // A fake bus range. Tests can exercise ownership and MMIO but must never dereference it.
    Pool {
        base: NonNull::without_provenance(core::num::NonZeroUsize::new(0x2000_2000).unwrap()),
        slot_len: 4096,
        states: [const { Cell::new(SlotState::Free) }; SLOTS],
    }
}
