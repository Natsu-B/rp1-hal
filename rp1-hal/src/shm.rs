//! Reader-local SPSC rings and DDR blocks in the Linux reservation, reached through PCIe
//! outbound region 0.
//!
//! proc0 publishes the reservation once per cold boot with [`enable`]; each core then takes its
//! endpoints from an [`Area`]. A host PERSTn resets the endpoint and with it the window (not
//! recovered from: reloading RP1 is). The PERSTn detector ([`crate::platform::pcie_state`])
//! marks the area lost, so `enable` publishes only while the detector is `Up`, in two sections
//! with interrupts masked:
//!
//! - A: the detector is `Up`, no PERSTn edge is latched, the link is up, the DBI selector is
//!   free, the endpoint is enabled and the loader handoff matches the note. Then region 0 is
//!   written and read back.
//! - between them, with interrupts enabled: the reservation is zeroed and its header written.
//!   IRQ53 can mark the window lost here.
//! - B: the detector is still `Up` and no PERSTn edge is latched; then the generation and
//!   `SHM_STATUS = 1`.
//!
//! A reset that starts after section B's checks is reported by IRQ53 right after the section,
//! which overwrites `SHM_STATUS` with `SHM_WINDOW_LOST`.
#[cfg(feature = "unstable-ring-dma")]
mod ring_dma;

use crate::{
    cold::{LINK, PCIE_INTR, PCIE_MONITOR2},
    platform::{
        Pcie, PcieState,
        perstn::{self, PERSTN_EDGE},
    },
};
pub use rp1_proto::shm::{Block, BlockReader, BlockWriter, Overflow, RingRx, RingTx, Wait};
use rp1_proto::{
    Error, Message, SHM_WINDOW, barrier, link,
    note::{Note, SHM_AREA, SHM_REGION, ShmArea, ShmKind, ShmRegion},
};
#[cfg(feature = "unstable-ring-dma")]
pub use {
    ring_dma::RingDma,
    rp1_proto::shm::{SlotCopy, Staged},
};

const SELECT: usize = 0x4010_8000;
/// The SELECT value that maps the outbound ATU registers into the DBI.
const SELECT_ATU: u32 = 3;
const DBI: usize = 0x4010_9000;
/// DBI command register: memory space (bit 1) and bus master (bit 2) enabled by Linux.
const DBI_COMMAND: usize = DBI + 4;
const MEMORY_AND_MASTER: u32 = 6;
/// Outbound region 0 in the DBI while SELECT = [`SELECT_ATU`]: 8 words, word 1 is the enable.
const REGION0: usize = DBI + 0x100;
const REGION0_ENABLE: usize = 1;
/// PCIE_MONITOR2 bits 16 (core alive), 17 (PERSTn deasserted) and the link.
const ENDPOINT_UP: u32 = (3 << 16) | LINK;
const SHM_GENERATION: usize = rp1_proto::LINK_START as usize + link::SHM_GENERATION;
const SHM_STATUS: usize = rp1_proto::LINK_START as usize + link::SHM_STATUS;
const SHM_READY: u32 = 1;

fn read(at: usize) -> u32 {
    unsafe { (at as *const u32).read_volatile() }
}
fn write(at: usize, value: u32) {
    unsafe { (at as *mut u32).write_volatile(value) }
}
fn word(offset: usize) -> usize {
    rp1_proto::LINK_START as usize + offset
}

/// Why [`enable`] did not publish the reservation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EnableError {
    /// The descriptor's note is invalid. Nothing was published (a bad region record is found
    /// only after region 0 was mapped).
    Note(Error),
    /// The PERSTn detector is not `Up`: the host reset the endpoint, or a reset would not be
    /// seen. Nothing was written.
    Detector(PcieState),
    /// `enable` already published the reservation, or region 0 read back wrong. Nothing was
    /// written.
    Pinned,
    /// A PERSTn edge is latched, the link is down, the DBI selector is in use, the endpoint is
    /// not enabled or the loader handoff does not match the note. Nothing was written.
    NotReady,
    /// Region 0 read back differently from what was written. Later calls return `Pinned`.
    Mismatch,
    /// The host reset the endpoint while the reservation was being initialized: not
    /// published, `SHM_STATUS` is `SHM_WINDOW_LOST`.
    WindowLost,
}

/// Maps outbound region 0 onto the declared Linux reservation, zeroes it, and publishes its
/// header, generation and `SHM_STATUS = 1`. Returns the generation.
///
/// Call once, after Linux has asked for the area (its PCIe probe is done) and before any core
/// attaches an [`Area`]; a second call returns [`EnableError::Pinned`].
pub fn enable(pcie: &mut Pcie, descriptor: &[u8], hash: &[u8; 32]) -> Result<u32, EnableError> {
    let note = Note::parse(descriptor).map_err(EnableError::Note)?;
    let area = area(&note).map_err(EnableError::Note)?;
    publish(&mut Mmio { note }, &mut pcie.shm_pinned, &area, hash)
}

/// Lab only: maps region 0 as [`enable`]'s section A does and returns the area, without
/// zeroing or publishing it, so a lab firmware can test the whole window before Linux enables
/// shm. A later `enable` maps it again.
#[doc(hidden)]
pub fn lab_map_window(pcie: &mut Pcie, descriptor: &[u8]) -> Result<ShmArea, EnableError> {
    let note = Note::parse(descriptor).map_err(EnableError::Note)?;
    let area = area(&note).map_err(EnableError::Note)?;
    Mmio { note }.masked(|io| map_window(io, &mut pcie.shm_pinned, &area))?;
    Ok(area)
}

fn area(note: &Note) -> Result<ShmArea, Error> {
    ShmArea::parse(note.get(SHM_AREA).ok_or(Error::Invalid)?.value)
}

/// What [`enable`] does to the hardware, so the host tests can run it on a model.
pub(crate) trait Io {
    fn read(&mut self, address: usize) -> u32;
    fn write(&mut self, address: usize, value: u32);
    fn barrier(&mut self);
    /// Runs `f` with this core's interrupts masked by PRIMASK, IRQ53 included.
    fn masked<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> T;
    /// The PERSTn detector's own state on proc0.
    fn detector(&mut self) -> PcieState;
    /// Zeroes the reservation and the note's SRAM regions and writes the header (with an odd
    /// generation). Returns the generation to publish.
    fn initialize(&mut self, area: &ShmArea, hash: &[u8; 32]) -> Result<u32, Error>;
}

/// [`enable`] after the note is parsed: section A, the fill, section B (module documentation).
fn publish(
    io: &mut impl Io,
    pinned: &mut bool,
    area: &ShmArea,
    hash: &[u8; 32],
) -> Result<u32, EnableError> {
    io.masked(|io| map_window(io, pinned, area))?;
    let generation = io.initialize(area, hash).map_err(EnableError::Note)?;
    // Section B. IRQ53 cannot run between these checks and the stores.
    io.masked(|io| {
        if io.detector() != PcieState::Up || io.read(PCIE_INTR) & PERSTN_EDGE != 0 {
            return Err(EnableError::WindowLost);
        }
        io.write(SHM_GENERATION, generation);
        io.barrier();
        io.write(SHM_STATUS, SHM_READY);
        *pinned = true;
        Ok(generation)
    })
}

/// Section A, with interrupts masked: the checks, then region 0 written and read back.
fn map_window(io: &mut impl Io, pinned: &mut bool, area: &ShmArea) -> Result<(), EnableError> {
    let state = io.detector();
    if state != PcieState::Up {
        return Err(EnableError::Detector(state));
    }
    if *pinned {
        return Err(EnableError::Pinned);
    }
    let ready = io.read(PCIE_INTR) & PERSTN_EDGE == 0
        && io.read(PCIE_MONITOR2) & ENDPOINT_UP == ENDPOINT_UP
        && io.read(SELECT) == 0
        && io.read(DBI_COMMAND) & MEMORY_AND_MASTER == MEMORY_AND_MASTER
        && io.read(word(link::HANDOFF_MAGIC)) == link::MAGIC
        && io.read(word(link::HANDOFF_PHYS_LOW)) == area.phys as u32
        && io.read(word(link::HANDOFF_PHYS_HIGH)) == (area.phys >> 32) as u32
        && io.read(word(link::HANDOFF_SIZE)) == area.size;
    if !ready {
        return Err(EnableError::NotReady);
    }
    let region = region0(area);
    io.write(SELECT, SELECT_ATU);
    // Disabled while the other words change, enabled last.
    io.write(REGION0 + 4 * REGION0_ENABLE, 0);
    for (i, value) in region.iter().enumerate() {
        if i != REGION0_ENABLE {
            io.write(REGION0 + i * 4, *value);
        }
    }
    io.write(REGION0 + 4 * REGION0_ENABLE, region[REGION0_ENABLE]);
    io.barrier();
    let valid = region
        .iter()
        .enumerate()
        .all(|(i, v)| io.read(REGION0 + i * 4) == *v);
    io.write(SELECT, 0);
    if !valid {
        // The window's state is unknown: enable nothing on it until RP1 is reloaded.
        *pinned = true;
        return Err(EnableError::Mismatch);
    }
    Ok(())
}

/// The 8 words of region 0 for `area`. PoC mapping: fabric 0x80_0100_0000 -> PCIe
/// 0x10_0000_0000 + CPU PA.
fn region0(area: &ShmArea) -> [u32; 8] {
    let target = 0x10_0000_0000u64 + area.phys;
    [
        0,
        0x8000_0000,
        0x0100_0000,
        0x80,
        0x0100_0000 + area.size - 1,
        target as u32,
        (target >> 32) as u32,
        0,
    ]
}

struct Mmio<'a> {
    note: Note<'a>,
}

impl Io for Mmio<'_> {
    fn read(&mut self, address: usize) -> u32 {
        read(address)
    }
    fn write(&mut self, address: usize, value: u32) {
        write(address, value);
    }
    fn barrier(&mut self) {
        barrier();
    }
    fn masked<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> T {
        #[cfg(all(target_arch = "arm", target_os = "none"))]
        return crate::with_irqs_masked(|| f(self));
        #[cfg(not(all(target_arch = "arm", target_os = "none")))]
        f(self)
    }
    fn detector(&mut self) -> PcieState {
        perstn::state()
    }
    fn initialize(&mut self, area: &ShmArea, hash: &[u8; 32]) -> Result<u32, Error> {
        let base = SHM_WINDOW as *mut u8;
        let previous = if read(SHM_WINDOW as usize) == u32::from_le_bytes(*b"RP1S")
            && read(SHM_WINDOW as usize + 4) == u32::from_le_bytes(*b"HM\0\0")
        {
            read(SHM_WINDOW as usize + 12)
        } else {
            0
        };
        // SAFETY: section A mapped the whole reservation at SHM_WINDOW, and only proc0's
        // `enable` writes it before the area is published.
        let mut header = unsafe { rp1_proto::shm::HeaderInit::begin(base, previous) };
        // Aligned volatile words only: a mapped Device aperture is never ordinary RAM.
        for offset in (64..area.size as usize).step_by(4) {
            write(SHM_WINDOW as usize + offset, 0);
        }
        let mut count = 0;
        for record in self.note.records() {
            let r = record?;
            if r.kind != SHM_REGION {
                continue;
            }
            let r = ShmRegion::parse(r.value)?;
            for offset in (0..usize::from(r.sram_len)).step_by(4) {
                write(word(usize::from(r.sram_offset) + offset), 0);
            }
            count += 1;
        }
        header.write(hash, count, read(crate::TIMER_RAWL));
        complete_posted_writes();
        let generation = header.publish();
        complete_posted_writes();
        Ok(generation)
    }
}

/// Drain posted DDR writes before publishing a response in SRAM or ringing a doorbell.
pub fn complete_posted_writes() {
    barrier();
    let _ = read(SHM_WINDOW as usize);
    barrier();
}

/// One core's declared endpoints. A region can be taken once from this owner.
pub struct Area<'a> {
    note: Note<'a>,
    core: u8,
    taken: u64,
}
impl<'a> Area<'a> {
    /// # Safety
    /// `enable` has completed; this is the only Area for the selected core (1=proc0, 2=proc1).
    /// The cold-boot mappings remain valid for every returned endpoint's lifetime.
    pub unsafe fn attach(descriptor: &'a [u8], core: u8) -> Result<Self, Error> {
        if !(1..=2).contains(&core) || read(word(link::SHM_STATUS)) != 1 {
            return Err(Error::NotReady);
        }
        Ok(Self {
            note: Note::parse(descriptor)?,
            core,
            taken: 0,
        })
    }
    fn take<T: Message>(&mut self, name: &str, expected: ShmKind) -> Result<ShmRegion<'a>, Error> {
        let mut index = 0;
        for record in self.note.records() {
            let record = record?;
            if record.kind != SHM_REGION {
                continue;
            }
            if index >= 64 {
                return Err(Error::Invalid);
            }
            let r = ShmRegion::parse(record.value)?;
            if r.name == name {
                let kind_ok = r.kind == expected
                    || matches!(
                        (r.kind, expected),
                        (ShmKind::Block { .. }, ShmKind::Block { .. })
                    );
                if !kind_ok
                    || r.core != self.core
                    || r.type_id != T::TYPE_ID
                    || r.elem_size as usize != core::mem::size_of::<T>()
                    || self.taken & (1 << index) != 0
                {
                    return Err(Error::Mismatch);
                }
                self.taken |= 1 << index;
                return Ok(r);
            }
            index += 1;
        }
        Err(Error::Invalid)
    }
    pub fn ring_tx<T: Message>(&mut self, name: &str) -> Result<RingTx<T>, Error> {
        let r = self.take::<T>(name, ShmKind::RingToLinux)?;
        unsafe {
            RingTx::attach(
                (SHM_WINDOW + r.ddr_offset) as *mut u8,
                r.ddr_len as usize,
                (rp1_proto::LINK_START + u32::from(r.sram_offset)) as *const u8,
                r.sram_len as usize,
                r.depth,
                overflow(r.overwrite),
            )
        }
    }
    pub fn ring_rx<T: Message>(&mut self, name: &str) -> Result<RingRx<T>, Error> {
        let r = self.take::<T>(name, ShmKind::RingToRp1)?;
        unsafe {
            RingRx::attach(
                (rp1_proto::LINK_START + u32::from(r.sram_offset)) as *const u8,
                r.sram_len as usize,
                (SHM_WINDOW + r.ddr_offset) as *mut u8,
                r.ddr_len as usize,
                r.depth,
                overflow(r.overwrite),
            )
        }
    }
    pub fn block<T: Message>(&mut self, name: &str) -> Result<Block<T>, Error> {
        let r = self.take::<T>(
            name,
            ShmKind::Block {
                writer: rp1_proto::note::Writer::Both,
            },
        )?;
        unsafe { Block::attach((SHM_WINDOW + r.ddr_offset) as *mut u8, r.ddr_len as usize) }
    }
}

fn overflow(overwrite: bool) -> Overflow {
    if overwrite {
        Overflow::Overwrite
    } else {
        Overflow::Reject
    }
}

#[cfg(test)]
mod tests;
