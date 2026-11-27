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
    claimed: [Cell<bool>; SLOTS],
}

impl<const SLOTS: usize> Pool<SLOTS> {
    /// Takes exclusive ownership of the reservation for this firmware epoch.
    ///
    /// The backing allocation must be in the RP1 shared SRAM, outside all linker/runtime,
    /// cross-core and host reservations. Safe Rust callers can pass an exclusive static slice
    /// obtained from their board's allocator. A caller constructing that slice from a linker
    /// symbol is responsible for enforcing that exclusivity across all processors and bus masters.
    /// Each slot has the same capacity. The backing allocation must divide evenly into slots.
    pub fn new(memory: &'static mut [u8]) -> Result<Self, Error> {
        let slot_len = memory.len().checked_div(SLOTS).ok_or(Error::Invalid)?;
        if slot_len == 0 || slot_len * SLOTS != memory.len() {
            return Err(Error::Invalid);
        }
        sram(memory.as_ptr() as usize, memory.len())?;
        Ok(Self {
            base: NonNull::from(memory).cast(),
            slot_len,
            claimed: [const { Cell::new(false) }; SLOTS],
        })
    }

    /// Borrows the prefix of a slot. The unused bytes stay reserved by the same lease.
    pub fn lease(&self, slot: usize, len: usize) -> Result<Lease<'_>, Error> {
        let claimed = self.claimed.get(slot).ok_or(Error::Invalid)?;
        if len == 0 || len > self.slot_len {
            return Err(Error::Invalid);
        }
        if claimed.replace(true) {
            return Err(Error::Busy);
        }
        // Slot index, multiplication and addition are bounded by the validated reservation.
        let ptr = self
            .base
            .map_addr(|base| base.saturating_add(slot * self.slot_len));
        Ok(Lease {
            ptr,
            offset: 0,
            len,
            capacity: self.slot_len,
            claimed,
        })
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

/// Exclusive access to a pool slot. DMA takes the lease by value and only hands it back
/// after observing quiescence.
#[derive(Debug)]
pub struct Lease<'a> {
    ptr: NonNull<u8>,
    offset: usize,
    len: usize,
    capacity: usize,
    claimed: &'a Cell<bool>,
}

impl Lease<'_> {
    /// Select the number of bytes at the start of this slot to transfer.
    pub fn set_len(&mut self, len: usize) -> Result<(), Error> {
        self.set_range(0, len)
    }

    /// Select a nonempty range within the slot, keeping the rest of the slot reserved.
    pub fn set_range(&mut self, offset: usize, len: usize) -> Result<(), Error> {
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

    pub fn as_slice(&self) -> &[u8] {
        // SAFETY: this lease owns the slot and CPU access cannot overlap a live DMA transfer.
        unsafe { slice::from_raw_parts(self.data(), self.len) }
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        // SAFETY: as for as_slice, with exclusive access to the lease.
        unsafe { slice::from_raw_parts_mut(self.data(), self.len) }
    }

    pub(super) fn address(&self) -> Result<BusAddr, Error> {
        sram(self.data() as usize, self.len)
    }

    pub(super) fn len(&self) -> u32 {
        self.len as u32
    }
}

impl Drop for Lease<'_> {
    fn drop(&mut self) {
        self.claimed.set(false);
    }
}

#[cfg(test)]
pub(super) fn fake_pool<const SLOTS: usize>() -> Pool<SLOTS> {
    // A fake bus range. Tests can exercise ownership and MMIO but must never dereference it.
    Pool {
        base: NonNull::without_provenance(core::num::NonZeroUsize::new(0x2000_2000).unwrap()),
        slot_len: 4096,
        claimed: [const { Cell::new(false) }; SLOTS],
    }
}
