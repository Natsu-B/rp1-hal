//! Firmware-side transport between RP1 and Linux.

#![no_std]

#[cfg(feature = "unstable-host-irq")]
pub mod notify;
#[cfg(feature = "unstable-ring-dma")]
pub mod ring_dma;
