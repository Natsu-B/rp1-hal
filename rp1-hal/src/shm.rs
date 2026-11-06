//! Reader-local SPSC rings and DDR blocks. Cold boot only (ATU is lost on PERSTn).
pub use rp1_proto::shm::{
    Block, BlockReader, BlockWriter, Overflow, RingRx, RingTx, SlotCopy, Staged, Wait,
};
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
/// Moves RP1 → Linux ring batches into Linux DDR by DMA ([`RingTx::push_slice_staged`]):
/// the slot images are staged in a shared-SRAM buffer and copied by a channel of the
/// declared DMA controller (`dma = {}`; the caller lends the channel per call, so the same
/// channel can serve several rings). `head` is published only after the copy finished and
/// satisfies SlotCopy ordering (docs/dma.md, "DMA ring producer").
///
/// Batches whose slot images are smaller than the threshold use the CPU path
/// ([`RingTx::push_slice`], posted word stores), which is faster for them.
pub struct RingDma<'s> {
    stage: &'s mut [u8],
    threshold: usize,
}
impl<'s> RingDma<'s> {
    /// Provisional CPU/DMA crossover in staged bytes, retained from historical HW measurements
    /// with the removed guard (2026-11-05: 16 B messages, DMA wins at 12 slots = 288 B, loses
    /// at 8 = 192 B; 64 B messages: wins at 4 = 288 B, loses at 2 = 144 B; docs/dma.md).
    /// The current path's crossover has not been remeasured.
    pub const DEFAULT_THRESHOLD: usize = 256;
    /// `stage` must be shared SRAM (`#[unsafe(link_section = ".shared")]`) and 8-byte aligned;
    /// its size bounds one DMA batch ([`RingTx::staged_capacity`]).
    pub fn new(stage: &'s mut [u8]) -> Result<Self, crate::dma::Error> {
        use crate::dma::{BusAddr, Error};
        let start = stage.as_ptr() as usize;
        let sram = BusAddr::SRAM..BusAddr::SRAM + 0x1_0000;
        if stage.is_empty() || !start.is_multiple_of(8) {
            return Err(Error::Invalid);
        }
        let first = BusAddr::from_proc(start)?.0;
        let last = BusAddr::from_proc(start + stage.len() - 1)?.0;
        if !sram.contains(&first) || !sram.contains(&last) {
            return Err(Error::NotDmaVisible);
        }
        Ok(Self {
            stage,
            threshold: Self::DEFAULT_THRESHOLD,
        })
    }
    /// Batches of at least `bytes` staged bytes go by DMA (`0` = always when a slot fits the
    /// stage, `usize::MAX` = never).
    pub fn set_threshold(&mut self, bytes: usize) {
        self.threshold = bytes;
    }
    pub fn threshold(&self) -> usize {
        self.threshold
    }
    fn batch_len<T: Message>(&self, ring: &mut RingTx<T>, requested: usize) -> usize {
        let n = requested.min(RingTx::<T>::staged_capacity(self.stage.len()));
        // CPU-only candidates need no tail read before push_slice checks the free slots.
        if n == 0
            || self.threshold == usize::MAX
            || n.saturating_mul(rp1_proto::shm::slot_size(size_of::<T>())) < self.threshold
        {
            return n;
        }
        n.min(ring.writable() as usize)
    }
    /// Publishes up to `values.len()` messages like [`RingTx::push_slice`] (returns how many):
    /// by `copy` when the batch reaches the DMA threshold (then at most
    /// [`RingTx::staged_capacity`] of the stage), otherwise by the CPU. On error nothing was
    /// published. A channel implements [`SlotCopy`]; a wrapper may also implement that same
    /// blocking ordering contract (for example to time copies without HAL diagnostics).
    pub fn push_slice<T: Message, C: SlotCopy>(
        &mut self,
        ring: &mut RingTx<T>,
        copy: &mut C,
        values: &[T],
    ) -> Result<usize, Staged<C::Error>> {
        if ring.is_fenced() {
            return Err(Staged::Fenced);
        }
        let n = self.batch_len(ring, values.len());
        if n == 0
            || self.threshold == usize::MAX
            || n.saturating_mul(rp1_proto::shm::slot_size(size_of::<T>())) < self.threshold
        {
            return Ok(ring.push_slice(values));
        }
        ring.push_slice_staged(values, self.stage, copy)
    }
    /// Generate accepted payloads directly in the stage when the writable batch reaches
    /// the DMA threshold. Otherwise use [`RingTx::push_with`] (including overwrite rings).
    /// `fill` receives the offset within this call and an initialized payload; fill all
    /// fields. See [`RingTx::push_with_staged`] for alignment, panic and failure semantics.
    /// No callback is invoked when fenced, full, or when stage validation fails.
    pub fn push_with<T: Message, C: SlotCopy>(
        &mut self,
        ring: &mut RingTx<T>,
        copy: &mut C,
        count: usize,
        fill: impl FnMut(usize, &mut T),
    ) -> Result<usize, Staged<C::Error>> {
        if ring.is_fenced() {
            return Err(Staged::Fenced);
        }
        let n = self.batch_len(ring, count);
        if n == 0
            || self.threshold == usize::MAX
            || n.saturating_mul(rp1_proto::shm::slot_size(size_of::<T>())) < self.threshold
        {
            return Ok(ring.push_with(count, fill));
        }
        ring.push_with_staged(count, self.stage, copy, fill)
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

    #[test]
    fn dma_threshold_uses_only_writable_slots() {
        use super::*;
        #[repr(align(8))]
        struct Mem([u8; 1024]);
        let (mut data, ctrl, mut stage) = (Mem([0; 1024]), Mem([0; 1024]), Mem([0; 1024]));
        let mut ring = unsafe {
            RingTx::<[u32; 4]>::attach(
                data.0.as_mut_ptr(),
                1024,
                ctrl.0.as_ptr(),
                1024,
                32,
                Overflow::Reject,
            )
        }
        .unwrap();
        assert_eq!(ring.push_slice(&[[0; 4]; 31]), 31);
        let helper = RingDma {
            stage: &mut stage.0,
            threshold: RingDma::DEFAULT_THRESHOLD,
        };
        let n = helper.batch_len(&mut ring, 32);
        assert_eq!(n, 1);
        assert!(n * rp1_proto::shm::slot_size(16) < helper.threshold());
    }

    #[test]
    fn generated_threshold_uses_writable_count_and_copy_failure_fences() {
        use super::*;
        #[repr(align(8))]
        struct Mem([u8; 1024]);
        struct Fail;
        // SAFETY: performs no accesses and returns failure synchronously.
        unsafe impl SlotCopy for Fail {
            type Error = ();
            unsafe fn copy_to_remote(&mut self, _: &[u8], _: *mut u8) -> Result<(), ()> {
                Err(())
            }
        }
        let (mut data, mut ctrl, mut stage) = (Mem([0; 1024]), Mem([0; 1024]), Mem([0; 1024]));
        let mut ring = unsafe {
            RingTx::<[u32; 4]>::attach(
                data.0.as_mut_ptr(),
                1024,
                ctrl.0.as_ptr(),
                1024,
                32,
                Overflow::Reject,
            )
        }
        .unwrap();
        let mut helper = RingDma {
            stage: &mut stage.0,
            threshold: RingDma::DEFAULT_THRESHOLD,
        };
        assert_eq!(ring.push_slice(&[[0; 4]; 31]), 31);
        // The requested batch exceeds the threshold; the one writable message must use CPU.
        assert_eq!(
            helper.push_with(&mut ring, &mut Fail, 32, |k, v| {
                assert_eq!(k, 0);
                *v = [42; 4];
            }),
            Ok(1)
        );
        assert_eq!(ring.head(), 32);
        // Release all slots so the next call takes DMA and fails, fencing every retry.
        unsafe {
            ctrl.0.as_mut_ptr().cast::<u32>().write_volatile(32);
        }
        assert_eq!(
            helper.push_with(&mut ring, &mut Fail, 32, |_, v| *v = [43; 4]),
            Err(Staged::Copy(()))
        );
        assert_eq!(ring.head(), 32);
        assert_eq!(
            helper.push_with(&mut ring, &mut Fail, 32, |_, _| panic!("fenced")),
            Err(Staged::Fenced)
        );
        assert_eq!(
            helper.push_slice(&mut ring, &mut Fail, &[[0; 4]]),
            Err(Staged::Fenced)
        );
    }

    #[test]
    fn cpu_fallback_progresses_without_staging_capacity() {
        use super::*;
        #[repr(align(8))]
        struct Mem([u8; 2048]);
        // Channel contains only (). The CPU path must not access DMA registers.
        let mut channel = unsafe { core::mem::zeroed::<crate::dma::Channel<1>>() };
        for (stage_len, threshold) in [
            (256, RingDma::DEFAULT_THRESHOLD), // smaller than one 264-byte slot
            (256, 0),                          // even "always DMA" must make progress
            (256, usize::MAX),
            (264, usize::MAX), // one staged slot must not cap the CPU batch
        ] {
            for overflow in [Overflow::Reject, Overflow::Overwrite] {
                let (mut data, ctrl, mut stage) = (Mem([0; 2048]), Mem([0; 2048]), Mem([0; 2048]));
                let mut ring = unsafe {
                    RingTx::<[u32; 64]>::attach(
                        data.0.as_mut_ptr(),
                        data.0.len(),
                        ctrl.0.as_ptr(),
                        ctrl.0.len(),
                        4,
                        overflow,
                    )
                }
                .unwrap();
                let mut helper = RingDma {
                    stage: &mut stage.0[..stage_len],
                    threshold,
                };
                let values = [[42; 64]; 3];
                assert_eq!(helper.push_slice(&mut ring, &mut channel, &values), Ok(3));
                if overflow == Overflow::Overwrite {
                    assert_eq!(
                        helper.push_with(&mut ring, &mut channel, 7, |_, v| *v = [43; 64]),
                        Ok(7)
                    );
                    assert_eq!(ring.head(), 10);
                } else {
                    assert_eq!(
                        helper.push_with(&mut ring, &mut channel, 7, |k, v| {
                            assert_eq!(k, 0);
                            *v = [43; 64];
                        }),
                        Ok(1)
                    );
                    assert_eq!(
                        helper.push_with(&mut ring, &mut channel, 7, |_, _| panic!("full")),
                        Ok(0)
                    );
                    assert_eq!(ring.head(), 4);
                }
            }
        }
    }

    #[test]
    fn overwrite_cpu_batch_ignores_staging_capacity() {
        use super::*;
        #[repr(align(8))]
        struct Mem([u8; 2048]);
        let (mut data, ctrl, mut stage) = (Mem([0; 2048]), Mem([0; 2048]), Mem([0; 2048]));
        let mut ring = unsafe {
            RingTx::<[u32; 64]>::attach(
                data.0.as_mut_ptr(),
                data.0.len(),
                ctrl.0.as_ptr(),
                ctrl.0.len(),
                4,
                Overflow::Overwrite,
            )
        }
        .unwrap();
        let mut helper = RingDma {
            stage: &mut stage.0[..264],
            threshold: RingDma::DEFAULT_THRESHOLD,
        };
        // Overwrite must use the CPU even when one staged slot reaches the DMA threshold.
        let mut channel = unsafe { core::mem::zeroed::<crate::dma::Channel<1>>() };
        assert_eq!(
            helper.push_slice(&mut ring, &mut channel, &[[42; 64]; 3]),
            Ok(3)
        );
        assert_eq!(ring.head(), 3);
        // An overwrite CPU batch may also exceed the ring depth.
        assert_eq!(
            helper.push_slice(&mut ring, &mut channel, &[[43; 64]; 7]),
            Ok(7)
        );
        assert_eq!(ring.head(), 10);
    }

    #[test]
    fn ring_dma_stage_must_be_aligned_shared_sram() {
        use super::RingDma;
        use crate::dma::Error;
        // Host memory is not DMAC-visible; a stage is checked before any use.
        #[repr(align(8))]
        struct S([u8; 64]);
        let mut s = S([0; 64]);
        assert!(matches!(RingDma::new(&mut s.0), Err(Error::NotDmaVisible)));
        assert!(matches!(RingDma::new(&mut s.0[1..]), Err(Error::Invalid)));
        assert!(matches!(RingDma::new(&mut []), Err(Error::Invalid)));
    }
}
