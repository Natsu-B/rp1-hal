//! Reader-local SPSC rings and DDR blocks in the Linux reservation, reached through PCIe
//! outbound region 0. Each core takes its endpoints from an [`Area`] after proc0 calls
//! [`enable`]. Recovery after a host reset requires reloading RP1.

pub use crate::platform::EnableError;
use crate::platform::Pcie;
pub use rp1_proto::shm::{Block, BlockReader, BlockWriter, Overflow, RingRx, RingTx, Wait};
use rp1_proto::{
    Error, Message, SHM_WINDOW, barrier, link,
    note::{Note, SHM_AREA, SHM_REGION, ShmArea, ShmKind, ShmRegion},
};

fn read(at: usize) -> u32 {
    unsafe { (at as *const u32).read_volatile() }
}
fn write(at: usize, value: u32) {
    unsafe { (at as *mut u32).write_volatile(value) }
}
fn word(offset: usize) -> usize {
    rp1_proto::LINK_START as usize + offset
}

/// Maps outbound region 0 onto the declared Linux reservation, zeroes it, and publishes its
/// header, generation and `SHM_STATUS = 1`. Returns the generation.
///
/// Call once, after Linux has asked for the area (its PCIe probe is done) and before any core
/// attaches an [`Area`]; a second call returns [`EnableError::Pinned`].
pub fn enable(pcie: &mut Pcie, descriptor: &[u8], hash: &[u8; 32]) -> Result<u32, EnableError> {
    let note = Note::parse(descriptor).map_err(EnableError::Note)?;
    let area = area(&note).map_err(EnableError::Note)?;
    // The parsed note supplies the checked reservation and the ranges to initialize.
    unsafe { pcie.__publish_shm(area.phys, area.size, || initialize(&note, &area, hash)) }
}

/// Lab only: maps region 0 as [`enable`]'s section A does and returns the area, without
/// zeroing or publishing it, so a lab firmware can test the whole window before Linux enables
/// shm. A later `enable` maps it again.
#[doc(hidden)]
pub fn lab_map_window(pcie: &mut Pcie, descriptor: &[u8]) -> Result<ShmArea, EnableError> {
    let note = Note::parse(descriptor).map_err(EnableError::Note)?;
    let area = area(&note).map_err(EnableError::Note)?;
    unsafe { pcie.__map_shm_for_probe(area.phys, area.size) }?;
    Ok(area)
}

fn area(note: &Note) -> Result<ShmArea, Error> {
    ShmArea::parse(note.get(SHM_AREA).ok_or(Error::Invalid)?.value)
}

fn initialize(note: &Note, area: &ShmArea, hash: &[u8; 32]) -> Result<u32, EnableError> {
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
    for record in note.records() {
        let r = record.map_err(EnableError::Note)?;
        if r.kind != SHM_REGION {
            continue;
        }
        let r = ShmRegion::parse(r.value).map_err(EnableError::Note)?;
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
