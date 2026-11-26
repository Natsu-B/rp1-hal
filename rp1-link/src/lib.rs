//! Firmware-side transport between RP1 and Linux.
//!
//! The generated `rp1::link` module attaches the declared endpoints: [`RingTx`], [`RingRx`],
//! [`Channel`], [`Server`] and [`Block`]. A ring's notification ([`notify`]) is part of its type, so `send*` /
//! `flush` wake Linux by themselves and a doorbell-notified [`RingRx::drain`] re-arms itself.

#![no_std]

mod channel;
mod mmio;
pub mod notify;
mod ring;
#[cfg(feature = "unstable-ring-dma")]
pub mod ring_dma;
mod rpc;

#[doc(hidden)]
pub mod generated;

pub use channel::Channel;
pub use ring::{RingRx, RingTx};
pub use rp1_hal::platform::EnableError;
pub use rp1_proto::{
    Error, Message,
    shm::{Block, BlockReader, BlockWriter, Overflow, Progress},
};
pub use rpc::Server;

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

/// Drain posted writes through the window that the lab probe (`rp1::link::__map_for_probe`)
/// mapped before Ready.
///
/// # Safety
/// The probe must have mapped the window in this cold boot.
#[doc(hidden)]
#[inline]
pub unsafe fn __flush_probe_window() {
    unsafe { generated::flush_mapped_window() };
}
