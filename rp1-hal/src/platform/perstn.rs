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
//! Every write of the `PLAT_PCIE` word is in this module: init's step 0 and step 10, then
//! IRQ53. Init finishes writing the handler's DSRAM state and the word before ISER. Only IRQ53
//! writes them after ISER (rule E1). The handler leaves PCIE_INTR latched as evidence
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
const PRIORITY: u8 = rp1_rt::interrupt::priority::encode(rp1_rt::interrupt::priority::PCIE_RESET);
/// PCIE_INTR / INTE bit 1: a PERSTn edge.
pub(crate) const PERSTN_EDGE: u32 = 1 << 1;
const MAX_SPURIOUS: u32 = 16;
const STATUS_WORD: usize = rp1_proto::LINK_START as usize + status::PLAT_PCIE;
const SHM_STATUS: usize = rp1_proto::LINK_START as usize + link::SHM_STATUS;

/// proc0's published PCIe state ([`pcie_state`]), the same on both cores. In `Lost`, proc0 has
/// the full PCIE_MONITOR2 and the time in [`super::Pcie::lost_detail`].
pub use status::PcieState;

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

/// The number of [`Word`]s: one past the last.
pub(crate) const WORD_COUNT: usize = Word::Spurious as usize + 1;

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

/// What step 10 read while arming the PERSTn detector, and its result.
///
/// On a normal boot: `inte` 0x2, `intf` 0, no PERSTn edge in `intr_at_arm`, `nvic53` and
/// `vec53_ok` true. The other bits of `intr_at_arm` are latches IRQ53 does not look at: print
/// them, do not judge them. (HW 2026-11: PCIE_INTS = 0x49 with INTE = 0x4b after the
/// bring-up; INTR is expected to hold the same bits 0, 3 and 6, HYP.)
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DetectorReport {
    /// PCIE_INTE read after arming: 0 if arming failed or IRQ53 already saw a reset.
    pub inte: u32,
    /// PCIE_INTF at arming. Nonzero leaves the detector off.
    pub intf: u32,
    /// PCIE_INTR at arming; 0 if `intf` was nonzero (not read).
    pub intr_at_arm: u32,
    /// IRQ53 enabled after arming.
    pub nvic53: bool,
    /// proc0 vector 53 is the detector's handler, not the default one.
    pub vec53_ok: bool,
}

impl DetectorReport {
    /// Whether PCIE_INTR held a PERSTn edge at arming; None if it was not read (`intf`
    /// nonzero).
    pub fn perstn_edge_at_arm(&self) -> Option<bool> {
        (self.intf == 0).then_some(self.intr_at_arm & PERSTN_EDGE != 0)
    }
}

/// The two values vector 53 can hold: the detector's handler, and the default handler the
/// linker script provides in its place when the detector's is not linked in.
#[derive(Clone, Copy)]
pub(crate) struct Handlers {
    pub(crate) detector: usize,
    pub(crate) default: usize,
}

impl Handlers {
    fn installed(self, vector: usize) -> bool {
        vector == self.detector && vector != self.default
    }
}

/// Step 10 of init, then the reads for its report. PCIE_INTR keeps its latches: a PERSTn edge
/// already latched is a reset that came after the endpoint bring-up consumed its own, so it is
/// detected, not cleared.
pub(crate) fn arm(io: &mut impl Io, handlers: Handlers) -> DetectorReport {
    io.write(PCIE_INTE, PERSTN_EDGE);
    let intf = io.read(PCIE_INTF);
    let intr_at_arm = if intf != 0 {
        // A forced source would read as a reset on the first entry.
        io.write(PCIE_INTE, 0);
        start(io, detector_off(status::PLAT_PCIE_OFF_ARM_FAULT));
        0
    } else {
        let intr = io.read(PCIE_INTR);
        start(io, status::PLAT_PCIE_UP);
        io.set_priority(PRIORITY);
        io.write(ICPR, IRQ_BIT);
        io.write(ISER, IRQ_BIT);
        // ICPR also cleared the pend of an edge latched before it.
        if io.read(PCIE_INTR) & PERSTN_EDGE != 0 {
            io.write(ISPR, IRQ_BIT);
        }
        intr
    };
    // For the report only: reads, so whatever IRQ53 wrote since ISER stays (rule E1).
    let inte = io.read(PCIE_INTE);
    let nvic53 = io.read(ISER) & IRQ_BIT != 0;
    let slot = io.read(VTOR) as usize + 4 * (16 + IRQ);
    let vector = io.read(slot) as usize;
    DetectorReport {
        inte,
        intf,
        intr_at_arm,
        nvic53,
        vec53_ok: handlers.installed(vector),
    }
}

/// Step 0 of init: `Initializing`, in the detector's state and `PLAT_PCIE`.
pub(crate) fn initializing(io: &mut impl Io) {
    set_state(io, status::PLAT_PCIE_INITIALIZING);
    io.barrier();
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
static mut WORDS: [u32; WORD_COUNT] = [0; WORD_COUNT];

pub(crate) fn entries() -> u32 {
    Mmio.load(Word::Entries)
}

/// The detector's own state word, decoded. proc0 only; IRQ53 may change it unless masked.
pub(crate) fn state() -> PcieState {
    PcieState::from_word(Mmio.load(Word::State))
}

pub(crate) fn lost_detail() -> Option<LostDetail> {
    #[cfg(all(target_arch = "arm", target_os = "none"))]
    let (state, monitor2, at_us) = rp1_rt::interrupt::__primask_free(|| {
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
    #[cfg(all(target_arch = "arm", target_os = "none"))]
    fn __rp1_default_irq();
}

impl Handlers {
    /// The handlers as linked into this image.
    pub(crate) fn linked() -> Self {
        #[cfg(all(target_arch = "arm", target_os = "none"))]
        let default = __rp1_default_irq as *const () as usize;
        // Host builds have no vector table; nothing runs the real step 10 there.
        #[cfg(not(all(target_arch = "arm", target_os = "none")))]
        let default = 0;
        Self {
            detector: __rp1_p0_PCIE_OUT as *const () as usize,
            default,
        }
    }
}

// Platform handler: bypasses #[interrupt], which reserves this line.
#[unsafe(export_name = "__rp1_p0_PCIE_OUT")]
#[cfg_attr(
    all(target_arch = "arm", target_os = "none"),
    unsafe(link_section = ".text.rp1_cold.__rp1_p0_PCIE_OUT")
)]
unsafe extern "C" fn pcie_out_handler() {
    handle(&mut Mmio);
    if matches!(pcie_state(), PcieState::Lost { .. }) {
        rp1_rt::send_event();
    }
}

#[cfg(test)]
pub(crate) mod fake;
#[cfg(test)]
mod tests;
