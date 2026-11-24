//! The PERSTn detector on proc0 IRQ53 (priority 0, the only one).
//!
//! A host PERSTn after init returns resets the endpoint, and with it the outbound window
//! (lifecycle rule L1: not recovered from; reloading RP1 is). PCIE_INTR bit 1 latches the
//! PERSTn edge. Init arms the detector last: PCIE_INTE = bit 1 only, so any PCIE_INTS bit on an
//! entry is a reset, or a forced or corrupted source; both are treated as the window lost
//! (fail closed). The first such entry writes `SHM_STATUS = SHM_WINDOW_LOST` and `Lost`, and
//! disables IRQ53. An entry that finds INTS = 0 (the pend at arming, a level source pending
//! again) is spurious; the 16th turns the detector off.
//!
//! Init finishes writing the handler's DSRAM state and the `PLAT_PCIE` word before ISER. Only
//! IRQ53 writes them after ISER (rule E1). The handler leaves PCIE_INTR latched as evidence
//! and writes nothing on the PCIe side but INTE = 0: no DBI, selector or RESETS.

use crate::{
    TIMER_RAWL,
    cold::{PCIE_INTE, PCIE_INTF, PCIE_INTR, PCIE_INTS, PCIE_MONITOR2},
};
use rp1_proto::{link, status};

const IRQ: usize = rp1_rt::pac::Interrupt::PCIE_OUT as usize;
const IRQ_BIT: u32 = 1 << (IRQ % 32);
const ISER: usize = 0xe000_e100 + (IRQ / 32) * 4;
const ICER: usize = ISER + 0x80;
const ISPR: usize = ISER + 0x100;
const ICPR: usize = ISER + 0x180;
const IPR: usize = 0xe000_e400 + IRQ;
const VTOR: usize = 0xe000_ed08;
const PRIORITY: u8 = 0;
/// PCIE_INTR / INTE bit 1: a PERSTn edge.
const PERSTN_EDGE: u32 = 1 << 1;
const MAX_SPURIOUS: u32 = 16;
const STATUS_WORD: usize = rp1_proto::LINK_START as usize + status::PLAT_PCIE;
const SHM_STATUS: usize = rp1_proto::LINK_START as usize + link::SHM_STATUS;

/// proc0's published PCIe state ([`pcie_state`]), the same on both cores.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PcieState {
    NotStarted,
    Initializing,
    /// The link is up and the detector is armed.
    Up,
    /// The host reset the endpoint after init. `monitor2_hi` is PCIE_MONITOR2[31:16] at the
    /// detection; proc0 has the full value and the time in [`super::Pcie::lost_detail`].
    Lost {
        monitor2_hi: u16,
    },
    /// A later host reset would not be seen. `reason` is one of
    /// `rp1_proto::status::PLAT_PCIE_OFF_*`.
    DetectorOff {
        reason: u8,
    },
    Unknown(u32),
}

impl PcieState {
    pub(crate) fn from_word(word: u32) -> Self {
        let (state, reason, high) = (word & 0xff, (word >> 8) & 0xff, word >> 16);
        match (state, reason, high) {
            (status::PLAT_PCIE_NOT_STARTED, 0, 0) => Self::NotStarted,
            (status::PLAT_PCIE_INITIALIZING, 0, 0) => Self::Initializing,
            (status::PLAT_PCIE_UP, 0, 0) => Self::Up,
            (status::PLAT_PCIE_LOST, 0, _) => Self::Lost {
                monitor2_hi: high as u16,
            },
            (status::PLAT_PCIE_DETECTOR_OFF, _, 0) => Self::DetectorOff {
                reason: reason as u8,
            },
            _ => Self::Unknown(word),
        }
    }
}

/// Read proc0's published PCIe state.
pub fn pcie_state() -> PcieState {
    #[cfg(all(target_arch = "arm", target_os = "none"))]
    let word = unsafe { (STATUS_WORD as *const u32).read_volatile() };
    #[cfg(not(all(target_arch = "arm", target_os = "none")))]
    let word = 0;
    PcieState::from_word(word)
}

/// What IRQ53 read when it saw the host reset.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LostDetail {
    pub monitor2: u32,
    /// TIMER µs, low 32 bits.
    pub at_us: u32,
}

/// The detector's words in proc0 DSRAM. `State` holds the `PLAT_PCIE` value the detector
/// last published; `Monitor2` and `AtUs` mean something only while it is `Lost`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Word {
    State,
    Monitor2,
    AtUs,
    /// Every IRQ53 entry. IRQ57 reads it to tell whether IRQ53 ran inside a serve.
    Entries,
    Spurious,
}

const fn lost(monitor2: u32) -> u32 {
    status::PLAT_PCIE_LOST | (monitor2 & 0xffff_0000)
}

const fn detector_off(reason: u8) -> u32 {
    status::PLAT_PCIE_DETECTOR_OFF | (reason as u32) << 8
}

pub(crate) trait Io {
    fn read(&mut self, address: usize) -> u32;
    /// A plain store: MMIO, the NVIC or a link page word.
    fn write(&mut self, address: usize, value: u32);
    fn set_priority(&mut self, value: u8);
    fn barrier(&mut self);
    fn load(&mut self, word: Word) -> u32;
    fn store(&mut self, word: Word, value: u32);
}

/// The values step 10 read, for the init report.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Armed {
    pub(crate) intf: u32,
    /// 0 if PCIE_INTF was set: not read, the detector stayed off.
    pub(crate) intr_at_arm: u32,
}

/// Step 10 of init. PCIE_INTR keeps its latches: a PERSTn edge already latched is a reset
/// that came after the endpoint bring-up consumed its own, so it is detected, not cleared.
pub(crate) fn arm(io: &mut impl Io) -> Armed {
    io.write(PCIE_INTE, PERSTN_EDGE);
    let intf = io.read(PCIE_INTF);
    if intf != 0 {
        // A forced source would read as a reset on the first entry.
        io.write(PCIE_INTE, 0);
        start(io, detector_off(status::PLAT_PCIE_OFF_ARM_FAULT));
        return Armed {
            intf,
            intr_at_arm: 0,
        };
    }
    let intr = io.read(PCIE_INTR);
    start(io, status::PLAT_PCIE_UP);
    io.set_priority(PRIORITY);
    io.write(ICPR, IRQ_BIT);
    io.write(ISER, IRQ_BIT);
    // ICPR also cleared the pend of an edge latched before it.
    if io.read(PCIE_INTR) & PERSTN_EDGE != 0 {
        io.write(ISPR, IRQ_BIT);
    }
    Armed {
        intf,
        intr_at_arm: intr,
    }
}

/// The detector's first state, with zeroed counters: only init calls this, before ISER.
fn start(io: &mut impl Io, state: u32) {
    io.store(Word::Entries, 0);
    io.store(Word::Spurious, 0);
    set_state(io, state);
    io.barrier();
}

fn set_state(io: &mut impl Io, state: u32) {
    io.store(Word::State, state);
    io.write(STATUS_WORD, state);
}

/// After step 10, for the report only: (PCIE_INTE, IRQ53 enabled, vector 53 is `handler`).
pub(crate) fn diagnose(io: &mut impl Io, handler: usize) -> (u32, bool, bool) {
    let inte = io.read(PCIE_INTE);
    let enabled = io.read(ISER) & IRQ_BIT != 0;
    let vector = io.read(VTOR) as usize + 4 * (16 + IRQ);
    (inte, enabled, io.read(vector) as usize == handler)
}

fn handle(io: &mut impl Io) {
    let entries = io.load(Word::Entries) + 1;
    io.store(Word::Entries, entries);
    if io.read(PCIE_INTS) == 0 {
        let spurious = io.load(Word::Spurious) + 1;
        io.store(Word::Spurious, spurious);
        if spurious >= MAX_SPURIOUS {
            stop(io);
            set_state(io, detector_off(status::PLAT_PCIE_OFF_SPURIOUS_STORM));
        }
        return;
    }
    stop(io);
    let monitor2 = io.read(PCIE_MONITOR2);
    let at_us = io.read(TIMER_RAWL);
    io.write(SHM_STATUS, link::SHM_WINDOW_LOST);
    io.barrier();
    io.store(Word::Monitor2, monitor2);
    io.store(Word::AtUs, at_us);
    set_state(io, lost(monitor2));
    io.barrier();
}

fn stop(io: &mut impl Io) {
    io.write(PCIE_INTE, 0);
    io.write(ICER, IRQ_BIT);
    // ICER must take effect before exception return.
    io.barrier();
}

// proc0-local DSRAM, indexed by `Word`. Init writes it before ISER, IRQ53 after. IRQ57 reads
// only `Entries`; thread readers copy it with interrupts masked.
static mut WORDS: [u32; 5] = [status::PLAT_PCIE_INITIALIZING, 0, 0, 0, 0];

pub(crate) fn entries() -> u32 {
    Mmio.load(Word::Entries)
}

pub(crate) fn lost_detail() -> Option<LostDetail> {
    #[cfg(all(target_arch = "arm", target_os = "none"))]
    let (state, monitor2, at_us) = crate::with_irqs_masked(|| {
        let mut io = Mmio;
        (
            io.load(Word::State),
            io.load(Word::Monitor2),
            io.load(Word::AtUs),
        )
    });
    #[cfg(not(all(target_arch = "arm", target_os = "none")))]
    let (state, monitor2, at_us) = (status::PLAT_PCIE_INITIALIZING, 0, 0);
    matches!(PcieState::from_word(state), PcieState::Lost { .. })
        .then_some(LostDetail { monitor2, at_us })
}

pub(crate) struct Mmio;

impl Io for Mmio {
    fn read(&mut self, address: usize) -> u32 {
        crate::read(address)
    }
    fn write(&mut self, address: usize, value: u32) {
        crate::write_relaxed(address, value);
    }
    fn set_priority(&mut self, value: u8) {
        unsafe { (IPR as *mut u8).write_volatile(value) }
    }
    fn barrier(&mut self) {
        rp1_proto::barrier();
    }
    fn load(&mut self, word: Word) -> u32 {
        unsafe { (&raw const WORDS[word as usize]).read_volatile() }
    }
    fn store(&mut self, word: Word, value: u32) {
        unsafe { (&raw mut WORDS[word as usize]).write_volatile(value) };
    }
}

unsafe extern "C" {
    fn __rp1_p0_PCIE_OUT();
}

/// The address vector 53 must hold.
pub(crate) fn handler() -> usize {
    __rp1_p0_PCIE_OUT as *const () as usize
}

#[rp1_rt::interrupt(proc0, __platform, sram)]
fn PCIE_OUT() {
    handle(&mut Mmio);
}

#[cfg(test)]
mod tests;
