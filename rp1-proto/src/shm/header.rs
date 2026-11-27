//! The DDR region header: RP1 initializes and publishes it, Linux opens and checks it.
use super::HEADER_SIZE;
use super::access::{aligned, ld, st};
use super::wake::Words;
use crate::{
    Error, barrier,
    note::{ShmKind, ShmRegion},
    wake::{self, BootEpoch, Session, SessionError},
};

const MAGIC: [u8; 8] = *b"RP1SHM\0\0";
const VERSION: u32 = 1;
const VERSION_V2: u32 = 2;

/// RP1's initializing handle for the DDR region header (protocol.md §1.3).
///
/// [`HeaderInit::begin`] creates it under a writable-mapping contract. Linux's read-only
/// [`Header`] has no write path, so a `PROT_READ` mapping stays sound.
pub struct HeaderInit {
    base: *mut u8,
    generation: u32,
}
unsafe impl Send for HeaderInit {}
impl HeaderInit {
    /// RP1: mark the region as being initialized (odd generation). Returns the handle whose
    /// [`HeaderInit::publish`] writes the next even generation. `previous` is the old header
    /// generation (read once).
    ///
    /// # Safety
    /// `base` is the aligned, writable 64-byte header mapping, it stays valid while the handle
    /// lives, and the caller is its only writer.
    pub unsafe fn begin(base: *mut u8, previous: u32) -> Self {
        let generation = (previous | 1).wrapping_add(1).max(2);
        unsafe { st(base, 12, generation.wrapping_sub(1)) };
        barrier();
        Self { base, generation }
    }
    /// Begin v2 initialization. Generations advance within a boot and restart at 2 for
    /// a different boot epoch.
    ///
    /// # Safety
    /// As for [`Self::begin`]. Only proc0 may initialize. Old CPU writers and posted
    /// writes/DMA must be drained before reusing the reservation. This reads the odd
    /// generation back through the header window before modifying the reservation.
    pub unsafe fn begin_v2(base: *mut u8, epoch: BootEpoch) -> Result<HeaderInitV2, SessionError> {
        let previous = if unsafe { ld(base, 56) } == epoch.word() {
            unsafe { ld(base, 12) }
        } else {
            0
        };
        let generation = Session::next_generation(previous)?;
        unsafe { st(base, 12, generation - 1) };
        barrier();
        let _ = unsafe { ld(base, 12) };
        barrier();
        Ok(HeaderInitV2 {
            header: Self { base, generation },
            epoch,
        })
    }

    /// The generation [`HeaderInit::publish`] will write.
    pub fn generation(&self) -> u32 {
        self.generation
    }
    /// RP1: write the header fields (generation stays odd).
    pub fn write(&mut self, note_sha256: &[u8; 32], regions: u32, uptime_us: u32) {
        self.write_fields(note_sha256, regions, uptime_us, VERSION, 0, 0);
    }

    fn write_fields(
        &mut self,
        note_sha256: &[u8; 32],
        regions: u32,
        uptime_us: u32,
        version: u32,
        epoch: u32,
        accepted_caps: u32,
    ) {
        let b = self.base;
        unsafe {
            st(
                b,
                0,
                u32::from_le_bytes([MAGIC[0], MAGIC[1], MAGIC[2], MAGIC[3]]),
            );
            st(
                b,
                4,
                u32::from_le_bytes([MAGIC[4], MAGIC[5], MAGIC[6], MAGIC[7]]),
            );
            st(b, 8, ((HEADER_SIZE as u32) << 16) | version);
            for (i, w) in note_sha256.as_chunks::<4>().0.iter().enumerate() {
                st(b, 16 + i * 4, u32::from_le_bytes(*w));
            }
            st(b, 48, regions);
            st(b, 52, uptime_us);
            st(b, 56, epoch);
            st(b, 60, accepted_caps);
        }
    }
    /// RP1: publish (even generation) and give up write access. Call after all rings/blocks
    /// are initialized; the caller drains posted writes (same-window readback) before it
    /// tells the peer.
    pub fn publish(self) -> u32 {
        barrier();
        unsafe { st(self.base, 12, self.generation) };
        barrier();
        self.generation
    }
}

/// An unpublished v2 header. Only [`Self::publish_v2`] can commit its even generation.
pub struct HeaderInitV2 {
    header: HeaderInit,
    epoch: BootEpoch,
}

impl HeaderInitV2 {
    /// Reset both sets of wake words in every ring, then commit the even generation.
    /// Other initialization (including draining any payload writes) must finish first.
    ///
    /// # Safety
    /// The proc0 initializer owns both parts of each ring. `regions` is the complete
    /// validated v3 layout for this reservation. The header and link bases must map
    /// the writable DDR reservation and link page for all those regions.
    pub unsafe fn publish_v2(
        mut self,
        link: *mut u8,
        regions: &[ShmRegion<'_>],
        note_sha256: &[u8; 32],
        uptime_us: u32,
        accepted_caps: u32,
    ) -> Result<Session, SessionError> {
        if accepted_caps & !wake::KNOWN_CAPS != 0 {
            return Err(SessionError::Unsupported);
        }
        let base = self.header.base;
        for region in regions {
            let (data, ctrl) = unsafe {
                match region.kind {
                    ShmKind::RingToLinux => (
                        base.add(region.ddr_offset as usize),
                        link.add(region.sram_offset as usize),
                    ),
                    ShmKind::RingToRp1 => (
                        link.add(region.sram_offset as usize),
                        base.add(region.ddr_offset as usize),
                    ),
                    ShmKind::Block { .. } => continue,
                }
            };
            // Credit is reserved even on overwrite rings, which never arm it.
            unsafe {
                Words::rx(data, ctrl).reset();
                Words::reset_credit(data, ctrl);
            }
        }
        let epoch = self.epoch.word();
        self.header.write_fields(
            note_sha256,
            regions.len() as u32,
            uptime_us,
            VERSION_V2,
            epoch,
            accepted_caps,
        );
        let generation = self.header.publish();
        let _ = unsafe { ld(base, 12) };
        barrier();
        Session::new(epoch, generation)
    }
}

/// Linux's read-only view of a published DDR region header.
#[derive(Clone, Copy)]
pub struct Header {
    base: *const u8,
    generation: u32,
}
unsafe impl Send for Header {}
impl Header {
    /// Linux: open a published header and check the note hash.
    ///
    /// # Safety
    /// `base` stays a readable aligned 64-byte mapping for the lifetime of the handle.
    pub unsafe fn open(base: *const u8, note_sha256: &[u8; 32]) -> Result<Self, Error> {
        unsafe { Self::open_version(base, note_sha256, VERSION) }
    }

    /// Open a v2 header and match the committed session and negotiated capability bits.
    ///
    /// # Safety
    /// As for [`Self::open`]. The caller verifies the matching mailbox result and link status.
    pub unsafe fn open_v2(
        base: *const u8,
        note_sha256: &[u8; 32],
        session: Session,
        accepted_caps: u32,
    ) -> Result<HeaderV2, Error> {
        let header = unsafe { Self::open_version(base, note_sha256, VERSION_V2) }?;
        if header.generation != session.generation()
            || unsafe { ld(base, 60) } != accepted_caps
            || accepted_caps & !wake::KNOWN_CAPS != 0
        {
            return Err(Error::Mismatch);
        }
        let header = HeaderV2 { header, session };
        header.session().map_err(|_| Error::Mismatch)?;
        Ok(header)
    }

    unsafe fn open_version(
        base: *const u8,
        note_sha256: &[u8; 32],
        version: u32,
    ) -> Result<Self, Error> {
        if !aligned(base) {
            return Err(Error::Invalid);
        }
        let generation = unsafe { ld(base, 12) };
        barrier();
        let mut magic = [0u8; 8];
        unsafe {
            magic[..4].copy_from_slice(&ld(base, 0).to_le_bytes());
            magic[4..].copy_from_slice(&ld(base, 4).to_le_bytes());
        }
        if generation & 1 != 0
            || generation == 0
            || magic != MAGIC
            || unsafe { ld(base, 8) } != ((HEADER_SIZE as u32) << 16) | version
        {
            return Err(Error::NotReady);
        }
        for (i, w) in note_sha256.as_chunks::<4>().0.iter().enumerate() {
            if unsafe { ld(base, 16 + i * 4) }.to_le_bytes() != *w {
                return Err(Error::Mismatch);
            }
        }
        let h = Self { base, generation };
        barrier();
        h.check()?;
        Ok(h)
    }
    /// Linux: `Err(Reset)` once RP1 re-initialized the region.
    #[inline]
    pub fn check(&self) -> Result<(), Error> {
        if unsafe { ld(self.base, 12) } == self.generation {
            Ok(())
        } else {
            Err(Error::Reset)
        }
    }
    pub fn generation(&self) -> u32 {
        self.generation
    }
    pub fn regions(&self) -> u32 {
        unsafe { ld(self.base, 48) }
    }
}

/// Read-only v2 attachment, bound to both the boot epoch and the even generation.
#[derive(Clone, Copy)]
pub struct HeaderV2 {
    header: Header,
    session: Session,
}
impl HeaderV2 {
    /// Revalidate the attachment, including after a boot that reuses the generation.
    pub fn session(&self) -> Result<Session, SessionError> {
        let base = self.header.base;
        let generation = unsafe { ld(base, 12) };
        barrier();
        let epoch = unsafe { ld(base, 56) };
        barrier();
        let current = Session::new(epoch, generation)?;
        if current != self.session {
            return Err(SessionError::Restarted);
        }
        Ok(current)
    }

    pub fn regions(&self) -> u32 {
        self.header.regions()
    }
}
