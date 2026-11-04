//! RP1 -> Linux interrupt that leaves hwirq 58 (rp1-mailbox / SCMI) alone: pulse the
//! `MSIX_CFG.TEST` bit of the MSI-X vector no Linux driver uses (`rp1_proto::irq`).
//! Linux receives it on the `rp1:irq` UIO node (`read()` / `poll()` on `/dev/uioN`, then
//! `write(1)` to re-enable).
//!
//! **Unstable** (cargo feature `unstable-host-irq`, declaration `host_irq = true`). TEST is ORed
//! with the vector's own hardware source, TIMER alarm 3 (`TIMER.INTS` bit 3). `host_irq` owns
//! `TIMER.ALARM3` (the declaration rejects any other owner), and nothing in rp1-hal unmasks or
//! forces it, so the source stays low (HW 2026-11-05: forcing `TIMER.INTF` bit 3 raises the
//! vector once and then swallows TEST pulses until released).
//!
//! Delivery is a hint, not a message: Linux's receiver must re-check the data it waits for
//! after every wake and after a bounded timeout. The vector is edge-triggered, so
//! - a pulse while Linux has not enabled the vector (before the UIO's first `write(1)`) is lost;
//! - pulses while the UIO is disabled (between a wake and the next `write(1)`) merge into one
//!   pending wake (lazy disable);
//! - an asserted TIMER alarm 3 interrupt would swallow pulses (above).
use crate::write;
use rp1_proto::irq;

/// Sole right to raise the RP1 -> Linux interrupt. The generated resources give it to the core
/// that declares `host_irq = true`; there is exactly one. Exclusive ownership keeps the
/// set/clear pair of one pulse from interleaving with another caller's: if a second caller set
/// TEST while it was still high, no new edge would reach Linux and that wake would be lost.
pub struct HostIrq {
    _private: (),
}

impl HostIrq {
    /// # Safety
    /// Generated resources only: the single owner of `MSIX_CFG(rp1_proto::irq::VECTOR).TEST`.
    pub const unsafe fn new() -> Self {
        Self { _private: () }
    }

    /// Wake Linux about data that lives in RP1 SRAM only (the link page, shm regions in shared
    /// SRAM). A barrier orders the SRAM writes before the pulse; Linux reads SRAM through the
    /// same RP1 bus, so nothing else is needed.
    ///
    /// Not for data in Linux DDR (`shm` rings and blocks): those writes are posted on PCIe and
    /// may still be in flight after a barrier. Use [`HostIrq::notify_ddr`].
    #[inline(always)]
    pub fn notify_sram(&mut self) {
        rp1_proto::barrier();
        self.pulse();
    }

    /// Wake Linux about data written to Linux DDR through the shm window. Completes the posted
    /// writes first (`shm::complete_posted_writes`: a readback through the same window), so
    /// Linux cannot be woken before its data arrives. The `Area` proves `shm::enable` ran, so
    /// the window is mapped.
    #[inline(always)]
    pub fn notify_ddr(&mut self, _area: &crate::shm::Area<'_>) {
        crate::shm::complete_posted_writes();
        self.pulse();
    }

    /// Set and clear go through the atomic aliases, so they do not race Linux's own ENABLE /
    /// IACK writes to the same register. `write` ends each store with a barrier.
    #[inline(always)]
    fn pulse(&mut self) {
        write(irq::MSIX_CFG_SET as usize, irq::MSIX_CFG_TEST);
        write(irq::MSIX_CFG_CLR as usize, irq::MSIX_CFG_TEST);
    }
}
