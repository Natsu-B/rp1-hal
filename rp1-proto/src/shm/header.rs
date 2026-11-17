//! The DDR region header: RP1 initializes and publishes it, Linux opens and checks it.
use super::HEADER_SIZE;
use super::access::{aligned, ld, st};
use crate::{Error, barrier};

const MAGIC: [u8; 8] = *b"RP1SHM\0\0";
const VERSION: u32 = 1;

/// RP1's initializing handle for the DDR region header (protocol.md §1.3).
///
/// Only [`HeaderInit::begin`] creates it, under a writable-mapping contract; Linux's
/// read-only [`Header`] has no write path, so a `PROT_READ` mapping stays sound.
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
    /// The generation [`HeaderInit::publish`] will write.
    pub fn generation(&self) -> u32 {
        self.generation
    }
    /// RP1: write the header fields (generation stays odd).
    pub fn write(&mut self, note_sha256: &[u8; 32], regions: u32, uptime_us: u32) {
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
            st(b, 8, ((HEADER_SIZE as u32) << 16) | VERSION);
            for (i, w) in note_sha256.as_chunks::<4>().0.iter().enumerate() {
                st(b, 16 + i * 4, u32::from_le_bytes(*w));
            }
            st(b, 48, regions);
            st(b, 52, uptime_us);
            st(b, 56, 0);
            st(b, 60, 0);
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
            || unsafe { ld(base, 8) } != ((HEADER_SIZE as u32) << 16) | VERSION
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
