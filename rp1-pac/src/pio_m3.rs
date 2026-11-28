//! Threshold-only writes until the PIO-to-DMAC request gate is verified.

use crate::generated::pio_m3::{SM_DMARX, SM_DMATX};

impl SM_DMATX {
    /// Write the TX threshold with the DMA request disabled.
    ///
    /// # Safety
    /// The caller must own the PIO block and ensure no DMA channel uses this FIFO.
    #[inline(always)]
    pub unsafe fn write_disabled_threshold(&self, threshold: u8) {
        assert!(threshold <= 0x1f);
        unsafe { self.write_at_offset(0, u32::from(threshold)) };
    }
}

impl SM_DMARX {
    /// Write the RX threshold with the DMA request disabled.
    ///
    /// # Safety
    /// The caller must own the PIO block and ensure no DMA channel uses this FIFO.
    #[inline(always)]
    pub unsafe fn write_disabled_threshold(&self, threshold: u8) {
        assert!(threshold <= 0x1f);
        unsafe { self.write_at_offset(0, u32::from(threshold)) };
    }
}
