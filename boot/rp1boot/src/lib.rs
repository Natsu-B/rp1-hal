#![no_std]
//! Hardware-independent parts of the rp1boot loader (host-tested): bundle verification,
//! the RP1 ELF image, the arm64 Image header and placement checks. See docs/loader.md.
extern crate alloc;

use alloc::{format, string::String, vec, vec::Vec};
use rp1_proto::bundle::{self, Bundle};

pub type Result<T> = core::result::Result<T, String>;

pub fn hex(b: &[u8]) -> String {
    use core::fmt::Write;
    b.iter().fold(String::new(), |mut s, b| {
        let _ = write!(s, "{b:02x}");
        s
    })
}

/// Check that `bytes` is the bundle named by the `rp1.bundle=` token in `bootargs`:
/// header layout, id (SHA-256 of the header) against the token, then every element's SHA-256.
pub fn verify_bundle<'a>(
    bytes: &'a [u8],
    bootargs: &str,
    sha256: impl Fn(&[u8]) -> [u8; 32],
) -> Result<Bundle<'a>> {
    // cargo-rp1 puts the token last, so a firmware-truncated cmdline loses (part of) it.
    let last = bootargs.split_ascii_whitespace().last().unwrap_or("");
    let truncated = (bundle::TOKEN.starts_with(last) && !last.is_empty())
        || last
            .strip_prefix(bundle::TOKEN)
            .is_some_and(|v| v.len() < 16);
    if truncated {
        return Err(format!(
            "bootargs end in {last:?} after {} B: the firmware truncated the command line; \
             shorten cmdline.txt",
            bootargs.len()
        ));
    }
    let token = bundle::find_token(bootargs).map_err(|e| match e {
        rp1_proto::Error::Mismatch => String::from("bootargs carry more than one rp1.bundle="),
        _ => String::from(
            "bootargs have no rp1.bundle= token: the generated cmdline.txt was not used",
        ),
    })?;
    let b = Bundle::parse(bytes).map_err(|_| {
        String::from(
            "the initrd range is not an rp1 bundle: config.txt must load rp1-bundle.img as initramfs",
        )
    })?;
    let id = sha256(b.header());
    let want = bundle::id_hex(&id);
    if token.as_bytes() != want {
        return Err(format!(
            "bundle id {} does not match cmdline rp1.bundle={token}: cmdline.txt and \
             rp1-bundle.img are from different builds",
            core::str::from_utf8(&want).unwrap_or("?")
        ));
    }
    for e in b.entries() {
        let got = sha256(b.data(&e));
        if got != e.sha256 {
            return Err(format!(
                "bundle element {} sha256 {} != header {} (corrupted bundle)",
                kind_name(e.kind),
                hex(&got),
                hex(&e.sha256)
            ));
        }
    }
    Ok(b)
}

pub fn kind_name(kind: u32) -> &'static str {
    match kind {
        bundle::FIRMWARE => "rp1.elf",
        bundle::KERNEL => "kernel",
        bundle::INITRAMFS => "initramfs",
        _ => "?",
    }
}

fn le16(b: &[u8], at: usize) -> Result<u16> {
    b.get(at..at + 2)
        .map(|s| u16::from_le_bytes([s[0], s[1]]))
        .ok_or_else(|| String::from("truncated ELF"))
}
fn le32(b: &[u8], at: usize) -> Result<u32> {
    b.get(at..at + 4)
        .map(|s| u32::from_le_bytes([s[0], s[1], s[2], s[3]]))
        .ok_or_else(|| String::from("truncated ELF"))
}
fn le64(b: &[u8], at: usize) -> Result<u64> {
    Ok(u64::from(le32(b, at)?) | (u64::from(le32(b, at + 4)?) << 32))
}

/// ELF32 little-endian ARM executable.
pub struct Elf<'a>(&'a [u8]);

impl<'a> Elf<'a> {
    pub fn parse(b: &'a [u8]) -> Result<Self> {
        if b.get(..7) != Some(b"\x7fELF\x01\x01\x01") || le16(b, 16)? != 2 || le16(b, 18)? != 40 {
            return Err("rp1.elf is not an ARM ELF32 little-endian executable".into());
        }
        Ok(Self(b))
    }
    pub fn entry(&self) -> u32 {
        le32(self.0, 24).unwrap_or(0)
    }
    /// Contents of the section called `name`.
    pub fn section(&self, name: &str) -> Result<&'a [u8]> {
        let b = self.0;
        let (shoff, shentsize, shnum, shstrndx) = (
            le32(b, 32)? as usize,
            usize::from(le16(b, 46)?),
            usize::from(le16(b, 48)?),
            usize::from(le16(b, 50)?),
        );
        if shentsize < 40 || shstrndx >= shnum {
            return Err("bad ELF section table".into());
        }
        let sh = |i: usize| shoff + i * shentsize;
        let strtab = le32(b, sh(shstrndx) + 16)? as usize;
        for i in 0..shnum {
            let n = strtab + le32(b, sh(i))? as usize;
            let s = b.get(n..).ok_or("bad ELF section name")?;
            if s.starts_with(name.as_bytes()) && s.get(name.len()) == Some(&0) {
                let (off, size) = (le32(b, sh(i) + 16)? as usize, le32(b, sh(i) + 20)? as usize);
                return b
                    .get(off..off + size)
                    .ok_or_else(|| format!("ELF section {name} out of range"));
            }
        }
        Err(format!("rp1.elf has no {name} section"))
    }
    /// The PT_LOAD segments laid out by physical address in `[base, base + max)`, zero-filled
    /// to `p_memsz`; overlapping segments are rejected. Returns the image from `base`.
    pub fn image(&self, base: u32, max: usize) -> Result<Vec<u8>> {
        let b = self.0;
        let (phoff, phentsize, phnum) = (
            le32(b, 28)? as usize,
            usize::from(le16(b, 42)?),
            usize::from(le16(b, 44)?),
        );
        if phentsize < 32 {
            return Err("bad ELF program header size".into());
        }
        let mut img = vec![0u8; max];
        let mut used: Vec<(u32, u32)> = Vec::new();
        let mut end = 0usize;
        for i in 0..phnum {
            let ph = phoff + i * phentsize;
            if le32(b, ph)? != 1 {
                continue;
            }
            let (off, paddr, filesz, memsz) = (
                le32(b, ph + 4)? as usize,
                le32(b, ph + 12)?,
                le32(b, ph + 16)? as usize,
                le32(b, ph + 20)?,
            );
            let start = paddr
                .checked_sub(base)
                .ok_or_else(|| format!("PT_LOAD at {paddr:#x} below {base:#x}"))?
                as usize;
            let stop = start + memsz as usize;
            if filesz > memsz as usize || stop > max {
                return Err(format!(
                    "PT_LOAD {paddr:#x}+{memsz:#x} does not fit the {max:#x}-byte RP1 image"
                ));
            }
            if used
                .iter()
                .any(|(a, l)| memsz != 0 && *l != 0 && paddr < a + l && *a < paddr + memsz)
            {
                return Err(format!("PT_LOAD at {paddr:#x} overlaps another segment"));
            }
            used.push((paddr, memsz));
            let data = b.get(off..off + filesz).ok_or("PT_LOAD outside the file")?;
            img[start..start + filesz].copy_from_slice(data);
            end = end.max(stop);
        }
        if end == 0 {
            return Err("rp1.elf has no PT_LOAD".into());
        }
        img.truncate(end.next_multiple_of(4));
        Ok(img)
    }
}

/// Fixed 2 MiB-aligned kernel base and the end of the loader's normal-RAM mapping.
pub const KERNEL_BASE: u64 = 0x0600_0000;
pub const RAM_MAP_END: u64 = 0x4000_0000;

/// arm64 Image header, validated for this loader: (text_offset, image_size).
pub fn kernel_header(image: &[u8]) -> Result<(u64, u64)> {
    if image.get(56..60) != Some(b"ARM\x64") {
        return Err("bundle kernel is not an uncompressed arm64 Image".into());
    }
    let (text_offset, size) = (le64(image, 8)?, le64(image, 16)?);
    let size = if size == 0 { image.len() as u64 } else { size };
    if size < image.len() as u64 {
        return Err("Image header image_size is smaller than the file".into());
    }
    let load = KERNEL_BASE
        .checked_add(text_offset)
        .ok_or("kernel load overflow")?;
    let range = Range::from_size("kernel", load, size)?;
    if range.end > RAM_MAP_END {
        return Err("kernel is outside the loader RAM mapping".into());
    }
    Ok((text_offset, size))
}

#[derive(Clone, Copy, Debug)]
pub struct Range {
    pub name: &'static str,
    pub start: u64,
    pub end: u64,
}

impl Range {
    pub fn from_size(name: &'static str, start: u64, size: u64) -> Result<Self> {
        let end = start.checked_add(size).ok_or("placement overflow")?;
        if start >= end {
            return Err(format!("{name}: empty placement"));
        }
        Ok(Self { name, start, end })
    }

    /// A non-empty destination must fit both mapped RAM and a DT memory bank.
    pub fn check_ram(&self, memory: &[(u64, u64)]) -> Result<()> {
        if self.start < self.end
            && self.end <= RAM_MAP_END
            && memory.iter().any(|&(a, s)| {
                a <= self.start && a.checked_add(s).is_some_and(|end| self.end <= end)
            })
        {
            Ok(())
        } else {
            Err(format!("{}: placement is outside mapped RAM", self.name))
        }
    }
}

/// First pair of overlapping non-empty ranges.
pub fn overlap(ranges: &[Range]) -> Option<(Range, Range)> {
    for (i, a) in ranges.iter().enumerate() {
        for b in &ranges[i + 1..] {
            if a.start < a.end && b.start < b.end && a.start < b.end && b.start < a.end {
                return Some((*a, *b));
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use sha2::{Digest, Sha256};

    fn sha(b: &[u8]) -> [u8; 32] {
        Sha256::digest(b).into()
    }

    /// Bundle with firmware, kernel and initramfs (built like cargo-rp1 does).
    fn sample() -> (Vec<u8>, String) {
        let parts: [(u32, Vec<u8>); 3] = [(1, vec![1; 10]), (2, vec![2; 20]), (3, vec![3; 5])];
        let header_len = 32 + 48 * 3;
        let mut out = vec![0u8; 3 * 4096 + 5];
        out[..8].copy_from_slice(bundle::MAGIC);
        out[8..12].copy_from_slice(&1u32.to_le_bytes());
        out[12..16].copy_from_slice(&(header_len as u32).to_le_bytes());
        out[16..20].copy_from_slice(&3u32.to_le_bytes());
        out[20..24].copy_from_slice(&((3 * 4096 + 5) as u32).to_le_bytes());
        for (i, (k, d)) in parts.iter().enumerate() {
            let at = 32 + 48 * i;
            let off = 4096 * (i + 1);
            out[at..at + 4].copy_from_slice(&k.to_le_bytes());
            out[at + 8..at + 12].copy_from_slice(&(off as u32).to_le_bytes());
            out[at + 12..at + 16].copy_from_slice(&(d.len() as u32).to_le_bytes());
            out[at + 16..at + 48].copy_from_slice(&sha(d));
            out[off..off + d.len()].copy_from_slice(d);
        }
        let id = sha(&out[..header_len]);
        let token = std::string::String::from_utf8(bundle::id_hex(&id).to_vec()).unwrap();
        (out, format!("console=x rp1.bundle={token}"))
    }

    #[test]
    fn bundle_checks() {
        let (b, args) = sample();
        assert!(verify_bundle(&b, &args, sha).is_ok());
        let e = verify_bundle(&b, "console=x", sha).err().unwrap();
        assert!(e.contains("no rp1.bundle="), "{e}");
        for cut in ["console=x rp1.bun", "console=x rp1.bundle=7f6f"] {
            let e = verify_bundle(&b, cut, sha).err().unwrap();
            assert!(e.contains("truncated"), "{e}");
        }
        let e = verify_bundle(&b, "rp1.bundle=0000000000000000", sha)
            .err()
            .unwrap();
        assert!(e.contains("different builds"), "{e}");
        let mut bad = b.clone();
        bad[2 * 4096 + 3] ^= 1;
        let e = verify_bundle(&bad, &args, sha).err().unwrap();
        assert!(e.contains("kernel") && e.contains("corrupted"), "{e}");
        let e = verify_bundle(&b[..100], &args, sha).err().unwrap();
        assert!(e.contains("not an rp1 bundle"), "{e}");
    }

    #[test]
    fn kernel_and_ranges() {
        let mut k = vec![0u8; 64];
        assert!(kernel_header(&k).is_err());
        k[56..60].copy_from_slice(b"ARM\x64");
        k[16..24].copy_from_slice(&4096u64.to_le_bytes());
        assert_eq!(kernel_header(&k).unwrap(), (0, 4096));
        let r = |name, start, end| Range { name, start, end };
        assert!(overlap(&[r("a", 0, 10), r("b", 10, 20)]).is_none());
        assert_eq!(
            overlap(&[r("a", 0, 11), r("b", 10, 20)]).unwrap().1.name,
            "b"
        );
        assert!(overlap(&[r("a", 0, 0), r("b", 0, 20)]).is_none());
    }

    #[test]
    fn kernel_rejects_wrapped_and_unmapped_placement() {
        let mut k = vec![0u8; 64];
        k[56..60].copy_from_slice(b"ARM\x64");
        for (offset, size) in [(0u64, u64::MAX), (0x4000_0000, 0x1000), (u64::MAX, 64)] {
            k[8..16].copy_from_slice(&offset.to_le_bytes());
            k[16..24].copy_from_slice(&size.to_le_bytes());
            assert!(
                kernel_header(&k).is_err(),
                "offset={offset:#x} size={size:#x}"
            );
        }
    }

    #[test]
    fn destinations_must_fit_actual_mapped_ram() {
        let r = Range::from_size("kernel", KERNEL_BASE, 4096).unwrap();
        assert!(r.check_ram(&[(0, RAM_MAP_END)]).is_ok());
        assert!(r.check_ram(&[(0, KERNEL_BASE)]).is_err());
        assert!(r.check_ram(&[(0, u64::MAX), (u64::MAX, 4096)]).is_ok());
        assert!(r.check_ram(&[(KERNEL_BASE, u64::MAX)]).is_err());
        assert!(Range::from_size("bad", u64::MAX, 1).is_err());
        assert!(Range::from_size("empty", 0, 0).is_err());
        assert!(
            Range { end: r.start, ..r }
                .check_ram(&[(0, RAM_MAP_END)])
                .is_err()
        );
        assert!(
            Range::from_size("unmapped", RAM_MAP_END, 1)
                .unwrap()
                .check_ram(&[(0, u64::MAX)])
                .is_err()
        );
    }

    /// The lab firmware ELF, when present (not committed).
    #[test]
    fn real_firmware_image() {
        let Ok(path) = std::env::var("RP1BOOT_TEST_ELF") else {
            return;
        };
        let bytes = std::fs::read(path).unwrap();
        let elf = Elf::parse(&bytes).unwrap();
        let note = rp1_proto::note::descriptor(elf.section(".note.rp1").unwrap()).unwrap();
        rp1_proto::note::Note::parse(note).unwrap();
        rp1_dt::fdt::Fdt::parse(elf.section(".rp1.dtbo").unwrap()).unwrap();
        let img = elf.image(0x2000_0000, 0xc000).unwrap();
        assert_eq!(
            u32::from_le_bytes(img[4..8].try_into().unwrap()),
            elf.entry()
        );
        std::eprintln!("image {} bytes", img.len());
    }
}
