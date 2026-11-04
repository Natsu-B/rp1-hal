//! RP1 -> Linux interrupt that leaves hwirq 58 (rp1-mailbox / SCMI) alone: pulse the
//! `MSIX_CFG.TEST` bit of the MSI-X vector no Linux driver uses (`rp1_proto::irq`).
//! Linux receives it on the `rp1:irq` UIO node (`read()` / `poll()` on `/dev/uioN`, then
//! `write(1)` to re-enable). While Linux has not enabled the vector, a pulse sends nothing.
use crate::write;
use rp1_proto::irq;

/// Send one interrupt to Linux. Call it after the data it announces is written and
/// published (`rp1_proto::barrier()`). Set and clear go through the atomic aliases, so they
/// do not race Linux's own ENABLE / IACK writes to the same register.
#[inline(always)]
pub fn notify() {
    write(irq::MSIX_CFG_SET as usize, irq::MSIX_CFG_TEST);
    write(irq::MSIX_CFG_CLR as usize, irq::MSIX_CFG_TEST);
}
