//! Boot bundle (`rp1-bundle.img`, loader.md §1): RP1 firmware ELF + Linux Image [+ initramfs]
//! in one file that the VPU firmware loads as `initramfs`. Checked without allocation.
//!
//! ```text
//! 0   magic "RP1BNDL\0"
//! 8   version u32 (1)
//! 12  header_len u32 (= 32 + 48 * count)
//! 16  count u32 (2..=3)
//! 20  total_len u32 (length of the whole bundle)
//! 24  reserved u64 (0)
//! 32  count x { kind u32, flags u32 (0), offset u32, len u32, sha256 [u8; 32] }
//! ```
//!
//! Elements start on [`ALIGN`] boundaries in ascending order without overlap. The bundle id is
//! the SHA-256 of the header bytes (which carry every element's SHA-256); the generated
//! cmdline.txt carries its first 8 bytes as the token `rp1.bundle=<16 hex digits>`.
use crate::{Error, note::u32_at};

pub const MAGIC: &[u8; 8] = b"RP1BNDL\0";
pub const VERSION: u32 = 1;
pub const HEADER_LEN: usize = 32;
pub const ENTRY_LEN: usize = 48;
pub const ALIGN: u32 = 4096;
/// `rp1.elf`: the RP1 firmware (with `.note.rp1` and `.rp1.dtbo`).
pub const FIRMWARE: u32 = 1;
/// Uncompressed arm64 Linux `Image` (the loader does not decompress).
pub const KERNEL: u32 = 2;
/// Optional initramfs, handed to Linux in place via `/chosen/linux,initrd-*`.
pub const INITRAMFS: u32 = 3;
pub const TOKEN: &str = "rp1.bundle=";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Entry {
    pub kind: u32,
    pub offset: u32,
    pub len: u32,
    pub sha256: [u8; 32],
}

#[derive(Clone, Copy)]
pub struct Bundle<'a> {
    bytes: &'a [u8],
    count: usize,
}

impl<'a> Bundle<'a> {
    /// Check the header and element layout (not the element hashes).
    pub fn parse(bytes: &'a [u8]) -> Result<Self, Error> {
        if bytes.len() < HEADER_LEN || &bytes[..8] != MAGIC || u32_at(bytes, 8)? != VERSION {
            return Err(Error::Invalid);
        }
        let count = u32_at(bytes, 16)? as usize;
        let header_len = u32_at(bytes, 12)? as usize;
        let total = u32_at(bytes, 20)? as usize;
        if !(2..=3).contains(&count)
            || header_len != HEADER_LEN + count * ENTRY_LEN
            || total != bytes.len()
            || bytes[24..32] != [0; 8]
            || header_len > total
        {
            return Err(Error::Invalid);
        }
        let b = Self { bytes, count };
        let mut end = header_len as u64;
        let mut seen = 0u32;
        for e in b.entries() {
            let start = u64::from(e.offset);
            if !(FIRMWARE..=INITRAMFS).contains(&e.kind)
                || seen & (1 << e.kind) != 0
                || u32_at(bytes, b.entry_at(e.kind)? + 4)? != 0
                || !e.offset.is_multiple_of(ALIGN)
                || start < end
                || e.len == 0
                || start + u64::from(e.len) > total as u64
            {
                return Err(Error::Invalid);
            }
            seen |= 1 << e.kind;
            end = start + u64::from(e.len);
        }
        if seen & (1 << FIRMWARE | 1 << KERNEL) != (1 << FIRMWARE | 1 << KERNEL) {
            return Err(Error::Invalid);
        }
        Ok(b)
    }
    fn entry_at(&self, kind: u32) -> Result<usize, Error> {
        (0..self.count)
            .map(|i| HEADER_LEN + i * ENTRY_LEN)
            .find(|at| u32_at(self.bytes, *at) == Ok(kind))
            .ok_or(Error::Invalid)
    }
    pub fn entries(&self) -> impl Iterator<Item = Entry> + 'a {
        let bytes = self.bytes;
        (0..self.count).map(move |i| {
            let at = HEADER_LEN + i * ENTRY_LEN;
            let w = |o: usize| u32::from_le_bytes(bytes[at + o..at + o + 4].try_into().unwrap());
            Entry {
                kind: w(0),
                offset: w(8),
                len: w(12),
                sha256: bytes[at + 16..at + 48].try_into().unwrap(),
            }
        })
    }
    pub fn get(&self, kind: u32) -> Option<Entry> {
        self.entries().find(|e| e.kind == kind)
    }
    /// The bytes the bundle id is computed over.
    pub fn header(&self) -> &'a [u8] {
        &self.bytes[..HEADER_LEN + self.count * ENTRY_LEN]
    }
    pub fn data(&self, e: &Entry) -> &'a [u8] {
        &self.bytes[e.offset as usize..(e.offset + e.len) as usize]
    }
}

/// Lowercase hex of the first 8 bytes of the bundle id, as written after [`TOKEN`].
pub fn id_hex(id: &[u8; 32]) -> [u8; 16] {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = [0; 16];
    for (i, b) in id[..8].iter().enumerate() {
        out[2 * i] = HEX[usize::from(b >> 4)];
        out[2 * i + 1] = HEX[usize::from(b & 15)];
    }
    out
}

/// The value of the single `rp1.bundle=` argument in `bootargs` (whitespace-separated tokens;
/// the firmware prepends its own arguments, so only the token is compared).
pub fn find_token(bootargs: &str) -> Result<&str, Error> {
    let mut found = None;
    for arg in bootargs.split_ascii_whitespace() {
        if let Some(v) = arg.strip_prefix(TOKEN) {
            if found.is_some() {
                return Err(Error::Mismatch);
            }
            found = Some(v);
        }
    }
    found.ok_or(Error::Invalid)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bundle(entries: &[(u32, u32, u32)], total: usize) -> [u8; 16384] {
        let mut b = [0u8; 16384];
        b[..8].copy_from_slice(MAGIC);
        b[8..12].copy_from_slice(&VERSION.to_le_bytes());
        let count = entries.len() as u32;
        b[12..16].copy_from_slice(&(32 + 48 * count).to_le_bytes());
        b[16..20].copy_from_slice(&count.to_le_bytes());
        b[20..24].copy_from_slice(&(total as u32).to_le_bytes());
        for (i, (k, o, l)) in entries.iter().enumerate() {
            let at = 32 + 48 * i;
            b[at..at + 4].copy_from_slice(&k.to_le_bytes());
            b[at + 8..at + 12].copy_from_slice(&o.to_le_bytes());
            b[at + 12..at + 16].copy_from_slice(&l.to_le_bytes());
            b[at + 16] = *k as u8;
        }
        b
    }

    #[test]
    fn parses_and_rejects_layouts() {
        let b = bundle(&[(1, 4096, 100), (2, 8192, 4096), (3, 12288, 1)], 12289);
        let p = Bundle::parse(&b[..12289]).unwrap();
        assert_eq!(p.get(KERNEL).unwrap().offset, 8192);
        assert_eq!(p.data(&p.get(INITRAMFS).unwrap()).len(), 1);
        assert_eq!(p.header().len(), 32 + 3 * 48);
        // Missing kernel, duplicate kind, misaligned, overlapping, past the end, wrong length.
        for (e, total) in [
            (&[(1, 4096, 100), (3, 8192, 1)][..], 8193),
            (&[(1, 4096, 100), (1, 8192, 1)][..], 8193),
            (&[(1, 4096, 100), (2, 8200, 1)][..], 8201),
            (&[(1, 4096, 5000), (2, 8192, 1)][..], 8193),
            (&[(1, 4096, 100), (2, 8192, 2)][..], 8193),
        ] {
            let b = bundle(e, total);
            assert!(Bundle::parse(&b[..total]).is_err(), "{e:?}");
        }
        let b = bundle(&[(1, 4096, 100), (2, 8192, 1)], 8193);
        assert!(Bundle::parse(&b[..8194]).is_err(), "total_len must match");
    }

    #[test]
    fn token() {
        let mut id = [0u8; 32];
        id[0] = 0xab;
        id[7] = 0x01;
        assert_eq!(&id_hex(&id), b"ab00000000000001");
        assert_eq!(find_token("a rp1.bundle=xyz b"), Ok("xyz"));
        assert_eq!(find_token("rp1.bundles=1"), Err(Error::Invalid));
        assert_eq!(
            find_token("rp1.bundle=1 rp1.bundle=1"),
            Err(Error::Mismatch)
        );
    }
}
