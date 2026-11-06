//! RP1 -> Linux interrupt that leaves hwirq 58 (rp1-mailbox / SCMI) alone: pulse the
//! `MSIX_CFG.TEST` bit of the MSI-X vector no Linux driver uses (`rp1_proto::irq`).
//! Linux receives it on the `rp1:irq` UIO node (`read()` / `poll()` on `/dev/uioN`, then
//! `write(1)` to re-enable).
//!
//! **Unstable** (cargo feature `unstable-host-irq`, declaration `host_irq = true`). TEST is ORed
//! with the vector's own hardware source, TIMER alarm 3 (`TIMER.INTS` bit 3). `host_irq` owns
//! `TIMER.ALARM3` (the declaration rejects any other owner), and nothing in rp1-hal unmasks or
//! forces it. Firmware that ran earlier (or a previous image on this RP1) might have, so
//! [`HostIrq::new`] takes the source over: it masks and un-forces alarm 3 and clears its raw
//! flag, leaving TEST as the only thing that can raise the vector (HW 2026-11-05: forcing
//! `TIMER.INTF` bit 3 raises the vector once and then swallows TEST pulses until released).
//!
//! Delivery is a hint, not a message: Linux's receiver must re-check the data it waits for
//! after every wake and after a bounded timeout. The vector is edge-triggered, so
//! - a pulse while Linux has not enabled the vector (before the UIO's first `write(1)`) is lost;
//! - pulses while the UIO is disabled (between a wake and the next `write(1)`) merge into one
//!   pending wake (lazy disable);
//! - an asserted TIMER alarm 3 interrupt would swallow pulses (above).
use crate::write;
use rp1_proto::irq;

// rp1-regdb `timer.toml`: TIMER at 0x400a_c000, atomic clear alias at +0x3000;
// INTS = (INTR & INTE) | INTF; INTS bit N is alarm N's interrupt (device.toml: TIMER_N = 26 + N).
const TIMER_BASE: usize = 0x400a_c000;
const ATOMIC_CLR: usize = 0x3000;
/// `INTR`: raw alarm flags, write-one-to-clear (regdb: verified_write = "hardware").
const TIMER_INTR: usize = TIMER_BASE + 0x34;
/// `INTE`: alarm interrupt enable, plain RW (regdb: write unverified; cleared through the
/// atomic clear alias, as `timer::Alarm::disable_interrupt` does).
const TIMER_INTE: usize = TIMER_BASE + 0x38;
/// `INTF`: alarm interrupt force. regdb marks write semantics unverified; on HW (2026-11-05,
/// devmem) writing 0x8 forced bit 3 and writing 0 released it, i.e. plain RW. The atomic clear
/// alias clears bit 3 without touching the other alarms' bits under plain-RW semantics.
const TIMER_INTF: usize = TIMER_BASE + 0x3c;
const ALARM_BIT: u32 = 1 << irq::TIMER_ALARM;

/// Sole right to raise the RP1 -> Linux interrupt. The generated resources give it to the core
/// that declares `host_irq = true`; there is exactly one. Exclusive ownership keeps the
/// set/clear pair of one pulse from interleaving with another caller's: if a second caller set
/// TEST while it was still high, no new edge would reach Linux and that wake would be lost.
pub struct HostIrq {
    _private: (),
}

impl HostIrq {
    /// Takes TIMER alarm 3 over as the vector's hardware source, so only TEST pulses raise it:
    /// mask its interrupt (`INTE` bit 3), release a force (`INTF` bit 3), then clear a latched
    /// raw flag (`INTR` bit 3). Masking first means a pending raw flag cannot re-raise the
    /// source between the steps. The alarm itself is not disarmed (`ARMED` W1C is unverified):
    /// with `INTE` bit 3 clear its firing only sets `INTR`, which does not reach `INTS`.
    ///
    /// # Safety
    /// Generated resources only: the single owner of `MSIX_CFG(rp1_proto::irq::VECTOR).TEST`
    /// and of `TIMER.ALARM3`.
    pub unsafe fn new() -> Self {
        write(TIMER_INTE + ATOMIC_CLR, ALARM_BIT);
        write(TIMER_INTF + ATOMIC_CLR, ALARM_BIT);
        write(TIMER_INTR, ALARM_BIT);
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

    /// Wake Linux's reader of an RP1 -> Linux ring declared `notify = "irq"`, if it sleeps; call
    /// after `push` / `push_slice` (once per batch is enough). Returns whether it pulsed.
    ///
    /// [`RingTx::wake_needed`](crate::shm::RingTx::wake_needed) reads `head` back through the
    /// shm window (completing the posted slot and `head` writes, like `notify_ddr`), then reads
    /// the reader's sleep counter from the link page: it pulses only for a sleep not yet
    /// notified, so a reader that keeps up costs no interrupts and a sleeping one gets one
    /// (protocol.md §2.3). The readback is one PCIe round trip per call.
    ///
    /// Nothing enforces the call: a producer that pushes without it is not an error, but its
    /// reader finds the data only at its next bounded re-check (`rp1_linux::RECHECK`, 10 ms,
    /// counted in Linux's `WaitStats::late`). [`HostIrq::push_slice_notify`] does both steps.
    #[inline(always)]
    pub fn notify_ring<T: rp1_proto::Message>(&mut self, ring: &mut crate::shm::RingTx<T>) -> bool {
        if ring.wake_needed() {
            self.pulse();
            true
        } else {
            false
        }
    }

    /// `ring.push_slice(values)` then [`HostIrq::notify_ring`]: publish one batch and wake the
    /// reader if it sleeps. Returns (messages written, whether it pulsed). It notifies even when
    /// nothing was written (ring full), because the reader may have gone to sleep since the
    /// last call; a caller retrying the rest of a batch calls it again.
    #[inline(always)]
    pub fn push_slice_notify<T: rp1_proto::Message>(
        &mut self,
        ring: &mut crate::shm::RingTx<T>,
        values: &[T],
    ) -> (usize, bool) {
        let n = ring.push_slice(values);
        (n, self.notify_ring(ring))
    }

    /// [`crate::shm::RingDma::push_slice`] followed by [`HostIrq::notify_ring`] (also after
    /// an error or a full ring: the reader may have armed meanwhile). The DMA path has already
    /// completed the copy under the SlotCopy ordering contract before publishing `head`;
    /// Channel's AXI-to-PCIe ordering chain remains HYP (docs/dma.md). The notification reads
    /// `head` back; it does not verify the payload or establish that ordering chain.
    #[inline(always)]
    pub fn push_slice_dma_notify<T: rp1_proto::Message, const N: u8>(
        &mut self,
        dma: &mut crate::shm::RingDma<'_>,
        ring: &mut crate::shm::RingTx<T>,
        channel: &mut crate::dma::Channel<N>,
        values: &[T],
    ) -> (Result<usize, crate::shm::Staged<crate::dma::Error>>, bool) {
        let n = dma.push_slice(ring, channel, values);
        (n, self.notify_ring(ring))
    }

    /// [`crate::shm::RingDma::push_with`] then [`HostIrq::notify_ring`], including on full
    /// rings and errors. Uses the same ordering and notification contract as
    /// [`HostIrq::push_slice_dma_notify`].
    #[inline(always)]
    pub fn push_with_dma_notify<T: rp1_proto::Message, const N: u8>(
        &mut self,
        dma: &mut crate::shm::RingDma<'_>,
        ring: &mut crate::shm::RingTx<T>,
        channel: &mut crate::dma::Channel<N>,
        count: usize,
        fill: impl FnMut(usize, &mut T),
    ) -> (Result<usize, crate::shm::Staged<crate::dma::Error>>, bool) {
        let n = dma.push_with(ring, channel, count, fill);
        (n, self.notify_ring(ring))
    }

    #[inline(always)]
    fn pulse(&mut self) {
        write(irq::MSIX_CFG_SET as usize, irq::MSIX_CFG_TEST);
        write(irq::MSIX_CFG_CLR as usize, irq::MSIX_CFG_TEST);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timer_registers_match_the_pac() {
        let block = unsafe { &*(TIMER_BASE as *const crate::pac::generated::timer::RegisterBlock) };
        assert_eq!(crate::pac::TIMER::PTR as usize, TIMER_BASE);
        assert_eq!(block.intr() as *const _ as usize, TIMER_INTR);
        assert_eq!(block.inte() as *const _ as usize, TIMER_INTE);
        assert_eq!(block.intf() as *const _ as usize, TIMER_INTF);
    }

    /// The alarm taken over is the one whose interrupt is the host vector (rp1-regdb
    /// `device.toml`: TIMER_3 = 29).
    #[test]
    fn takes_over_the_vector_source() {
        assert_eq!(crate::pac::Interrupt::TIMER_3 as u32, irq::VECTOR);
        assert_eq!(ALARM_BIT, 1 << 3);
    }
}
