//! Explicit opt-in to the lab's outbound ordering experiment.

use super::{BusAddr, Channel, Error, Lease, Pool, RawTransfer, State, start_block};
use crate::{
    linux_blocks::Mmio,
    timer::{Deadline, Duration},
};
use core::mem::ManuallyDrop;

const TIMEOUT: Duration = Duration::millis(10);

/// A ring copy engine with its own quarantinable SRAM scratch slot.
pub struct ExperimentalRingDma<'a, const N: u8> {
    channel: &'a mut Channel<N>,
    scratch: &'a Pool<1>,
}

impl<const N: u8> Channel<N> {
    /// Select the experimental, polling ring adapter. The scratch slot must be 16-byte aligned and fit a batch plus 15 bytes.
    ///
    /// # Safety
    /// The caller must establish the current outbound mapping and keep it pinned for the
    /// complete DMA lifetime, including across an error. The DMA completion ordering point
    /// must precede later CPU publications to this ring mapping. On error, its destination
    /// requires an external drain before reuse or reattachment. See the HYP in docs/dma.md.
    pub unsafe fn experimental_ring_dma<'a>(
        &'a mut self,
        scratch: &'a Pool<1>,
    ) -> ExperimentalRingDma<'a, N> {
        ExperimentalRingDma {
            channel: self,
            scratch,
        }
    }
}

fn window(address: usize, len: usize) -> Result<BusAddr, Error> {
    let end = address.checked_add(len).ok_or(Error::NotDmaVisible)?;
    if address < 0x8000_0000 || end > 0xc000_0000 {
        return Err(Error::NotDmaVisible);
    }
    BusAddr::from_proc(address)
}

fn stage<'a>(pool: &'a Pool<1>, src: &[u8]) -> Result<(Lease<'a>, BusAddr), Error> {
    // Preserve the original stage alignment, including the second block of a wrap.
    let offset = src.as_ptr() as usize & 15;
    let mut scratch = pool.lease(0, offset + src.len())?;
    let base = scratch.address()?.0;
    if base & 15 != 0 {
        return Err(Error::Invalid);
    }
    scratch.as_mut_slice()[offset..].copy_from_slice(src);
    Ok((scratch, BusAddr(base + offset as u64)))
}

// SAFETY: the constructor requires the outbound lifetime and ordering proofs. DMA reads only
// the private scratch slot, which is quarantined if the local engine's stop is uncertain.
unsafe impl<const N: u8> rp1_proto::shm::SlotCopy for ExperimentalRingDma<'_, N> {
    type Error = Error;

    // Keep admission and staging in shared SRAM. The channel setup and status reads
    // still use ISRAM, alongside the ring publisher.
    #[inline(never)]
    #[cfg_attr(
        all(target_arch = "arm", target_os = "none"),
        unsafe(link_section = ".text.rp1_cold")
    )]
    unsafe fn copy_to_remote(&mut self, src: &[u8], dst: *mut u8) -> Result<(), Error> {
        rp1_rt::interrupt::ThreadGuard::new().map_err(crate::timer::Error::from)?;
        let destination = window(dst as usize, src.len())?;
        let (scratch, source) = stage(self.scratch, src)?;
        let len = src.len() as u32;
        let end = Deadline::after(TIMEOUT)?;
        let mut scratch = ManuallyDrop::new(scratch);
        let mut io = Mmio;
        // Hold the claim even on unwind after enable.
        let started = start_block(
            &mut io,
            N,
            &mut self.channel.state,
            source,
            destination,
            len,
        );
        let result = match started {
            Ok(()) => RawTransfer {
                channel: self.channel,
                result: None,
            }
            .wait_accepted(&mut io, end),
            Err(error) => Err(error),
        };
        if started.is_err() || self.channel.state == State::Idle {
            // SAFETY: the block never started, or the engine retired its use of SRAM.
            unsafe { ManuallyDrop::drop(&mut scratch) };
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outbound_requires_the_whole_range_to_be_in_one_window() {
        assert_eq!(window(0x8000_0000, 4), Ok(BusAddr(BusAddr::ATU)));
        for (start, len) in [(0x2000_0000, 64), (0xbfff_fff0, 32), (usize::MAX, 1)] {
            assert_eq!(window(start, len), Err(Error::NotDmaVisible));
        }
    }
}
