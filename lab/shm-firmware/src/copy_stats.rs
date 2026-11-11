//! Lab-only copier invocation accounting and optional timing.
use rp1_hal::shm::SlotCopy;

#[derive(Default)]
pub struct CopyStats {
    pub us: u32,
    pub last_end: u32,
    pub blocks: u32,
    pub width8: u32,
    pub width16: u32,
}
pub struct TimedCopy<'a, C> {
    pub inner: &'a mut C,
    pub stats: &'a mut CopyStats,
    pub timed: bool,
}
impl<C: SlotCopy> TimedCopy<'_, C> {
    /// `--timed` only: the copy with its duration and transfer width recorded.
    #[rp1_rt::cold]
    unsafe fn timed_copy(&mut self, src: &[u8], dst: *mut u8) -> Result<(), C::Error> {
        let t = crate::now();
        let r = unsafe { self.inner.copy_to_remote(src, dst) };
        let end = crate::now();
        self.stats.us = self.stats.us.wrapping_add(end.wrapping_sub(t));
        self.stats.last_end = end;
        let w = rp1_hal::dma::width_log2(src.as_ptr() as u64, dst as u64, src.len() as u32);
        self.stats.width8 += u32::from(w == 3);
        self.stats.width16 += u32::from(w == 4);
        r
    }
}
// SAFETY: delegates unchanged to the same blocking copier, preserving its contract.
unsafe impl<C: SlotCopy> SlotCopy for TimedCopy<'_, C> {
    type Error = C::Error;
    #[inline(always)]
    unsafe fn copy_to_remote(&mut self, src: &[u8], dst: *mut u8) -> Result<(), Self::Error> {
        self.stats.blocks += 1;
        if self.timed {
            unsafe { self.timed_copy(src, dst) }
        } else {
            unsafe { self.inner.copy_to_remote(src, dst) }
        }
    }
}
