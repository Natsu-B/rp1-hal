//! Reader-local SPSC rings and DDR blocks. Cold boot only (ATU is lost on PERSTn).
pub use rp1_proto::shm::{Block, BlockReader, BlockWriter, Overflow, RingRx, RingTx};
use rp1_proto::{
    Error, Message, SHM_WINDOW, barrier, link,
    note::{Note, SHM_AREA, SHM_REGION, ShmArea, ShmKind, ShmRegion},
};

const SELECT: usize = 0x4010_8000;
const DBI: usize = 0x4010_9000;
fn read(at: usize) -> u32 {
    unsafe { (at as *const u32).read_volatile() }
}
fn write(at: usize, value: u32) {
    unsafe { (at as *mut u32).write_volatile(value) }
}
fn word(offset: usize) -> usize {
    rp1_proto::LINK_START as usize + offset
}

/// Configure and read back outbound region 0 for the declared Linux reservation.
///
/// # Safety
/// Call on proc0, after Linux's final PCIe probe/PERSTn and before any peer
/// uses the reservation. No concurrent DBI-selector writer, DDR owner, or subsequent PCIe
/// reset is permitted. The loader handoff, DT reservation and note must describe this board.
pub unsafe fn configure_window(descriptor: &[u8]) -> Result<ShmArea, Error> {
    let note = Note::parse(descriptor)?;
    let area = ShmArea::parse(note.get(SHM_AREA).ok_or(Error::Invalid)?.value)?;
    if read(word(link::HANDOFF_MAGIC)) != link::MAGIC
        || read(word(link::HANDOFF_PHYS_LOW)) != area.phys as u32
        || read(word(link::HANDOFF_PHYS_HIGH)) != (area.phys >> 32) as u32
        || read(word(link::HANDOFF_SIZE)) != area.size
        || read(0x4010_81a4) & (1 << 20) == 0
        || read(SELECT) != 0
        || read(DBI + 4) & 6 != 6
    {
        return Err(Error::NotReady);
    }
    // PoC region 0: fabric 0x80_0100_0000 -> PCIe 0x10_0000_0000 + CPU PA.
    let target = 0x10_0000_0000u64 + area.phys;
    let tuple = [
        0,
        0x8000_0000,
        0x0100_0000,
        0x80,
        0x0100_0000 + area.size - 1,
        target as u32,
        (target >> 32) as u32,
        0,
    ];
    write(SELECT, 3);
    write(DBI + 0x104, 0);
    for (i, value) in tuple.iter().enumerate() {
        if i != 1 {
            write(DBI + 0x100 + i * 4, *value);
        }
    }
    write(DBI + 0x104, tuple[1]);
    barrier();
    let valid = tuple
        .iter()
        .enumerate()
        .all(|(i, v)| read(DBI + 0x100 + i * 4) == *v);
    write(SELECT, 0);
    if !valid {
        return Err(Error::Mismatch);
    }
    Ok(area)
}

/// Configure ATU, initialize the reservation and publish header/generation.
///
/// # Safety
/// As [`configure_window`], exactly once before endpoints are attached.
pub unsafe fn enable(descriptor: &[u8], hash: &[u8; 32]) -> Result<u32, Error> {
    let area = unsafe { configure_window(descriptor) }?;
    let note = Note::parse(descriptor)?;
    let base = SHM_WINDOW as *mut u8;
    let previous = if read(SHM_WINDOW as usize) == u32::from_le_bytes(*b"RP1S")
        && read(SHM_WINDOW as usize + 4) == u32::from_le_bytes(*b"HM\0\0")
    {
        read(SHM_WINDOW as usize + 12)
    } else {
        0
    };
    let mut header = unsafe { rp1_proto::shm::HeaderInit::begin(base, previous) };
    // Aligned volatile words only: a mapped Device aperture is never ordinary RAM.
    for offset in (64..area.size as usize).step_by(4) {
        write(SHM_WINDOW as usize + offset, 0);
    }
    let mut count = 0;
    for record in note.records() {
        let r = record?;
        if r.kind != SHM_REGION {
            continue;
        }
        let r = ShmRegion::parse(r.value)?;
        for offset in (0..usize::from(r.sram_len)).step_by(4) {
            write(
                rp1_proto::LINK_START as usize + usize::from(r.sram_offset) + offset,
                0,
            );
        }
        count += 1;
    }
    header.write(hash, count, read(0x400a_c028));
    complete_posted_writes();
    let generation = header.publish();
    complete_posted_writes();
    write(word(link::SHM_GENERATION), generation);
    barrier();
    write(word(link::SHM_STATUS), 1);
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

#[cfg(test)]
mod tests {
    #[test]
    fn expanded_window_translates_last_word() {
        let size = 0x20_0000;
        assert_eq!(0x0100_0000 + size - 1, 0x011f_ffff);
        assert_eq!(0x10_0000_0000u64 + rp1_proto::SHM_PHYS, 0x10_2200_0000);
    }
}
