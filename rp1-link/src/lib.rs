//! Firmware-side transport between RP1 and Linux.

#![no_std]

#[cfg(feature = "unstable-host-irq")]
pub mod notify;
#[cfg(feature = "unstable-ring-dma")]
pub mod ring_dma;

#[doc(hidden)]
pub mod generated;

pub use rp1_hal::platform::EnableError;
pub use rp1_proto::{
    Error, Message,
    shm::{Block, BlockReader, BlockWriter, Overflow, RingRx, RingTx, Wait},
};

/// Failure to attach the owning core's shared-memory endpoints.
#[derive(Debug)]
pub enum OpenError<Init> {
    /// Linux has yet to request publication or the reservation is still being initialized.
    NotReady(Init),
    /// PERSTn took the window away. Reload RP1 to recover.
    Reset,
    /// An invalid status or a failed endpoint attachment. The token is spent.
    Attach(Error),
}

/// Drain posted DDR writes while the published window is Ready.
///
/// Call before exposing a response in SRAM or pulsing an interrupt for DDR data. When the
/// window is pending or lost, this returns without reading the aperture. As with an attached
/// endpoint, PERSTn can race access; the mapping belongs to the current cold boot.
#[inline]
pub fn flush_window() {
    if generated::read_status() == rp1_proto::link::SHM_READY {
        unsafe { generated::flush_mapped_window() };
    }
}
