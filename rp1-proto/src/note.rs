//! ELF note descriptor v2, checked without allocation.
use crate::Error;

pub const MAGIC: &[u8; 8] = b"RP1NOTE\0";
pub const ELF_TYPE: u32 = 0x5250_3102;
pub const HEADER_LEN: usize = 32;
pub const FIRMWARE: u16 = 1;
pub const ENTRY: u16 = 2;
pub const OWNERSHIP: u16 = 0x10;
pub const CLOCK: u16 = 0x11;
pub const LOG: u16 = 0x12;
pub const BOOT: u16 = 0x13;
pub const SHM_AREA: u16 = 0x20;
pub const SHM_REGION: u16 = 0x21;

pub const fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = !0u32;
    let mut i = 0;
    while i < bytes.len() {
        crc ^= bytes[i] as u32;
        let mut j = 0;
        while j < 8 {
            crc = (crc >> 1) ^ (0xedb8_8320u32 & 0u32.wrapping_sub(crc & 1));
            j += 1;
        }
        i += 1;
    }
    !crc
}

pub fn u16_at(b: &[u8], at: usize) -> Result<u16, Error> {
    Ok(u16::from_le_bytes(
        b.get(at..at.checked_add(2).ok_or(Error::Invalid)?)
            .ok_or(Error::Invalid)?
            .try_into()
            .unwrap(),
    ))
}
pub fn u32_at(b: &[u8], at: usize) -> Result<u32, Error> {
    Ok(u32::from_le_bytes(
        b.get(at..at.checked_add(4).ok_or(Error::Invalid)?)
            .ok_or(Error::Invalid)?
            .try_into()
            .unwrap(),
    ))
}

#[derive(Clone, Copy, Debug)]
pub struct Record<'a> {
    pub kind: u16,
    pub flags: u16,
    pub value: &'a [u8],
}

#[derive(Clone, Copy)]
pub struct Note<'a> {
    bytes: &'a [u8],
}

impl<'a> Note<'a> {
    pub fn parse(bytes: &'a [u8]) -> Result<Self, Error> {
        if bytes.len() < HEADER_LEN
            || &bytes[..8] != MAGIC
            || u16_at(bytes, 8)? != 2
            || u16_at(bytes, 10)? != 32
            || u32_at(bytes, 12)? as usize != bytes.len()
            || u32_at(bytes, 16)? != 0
            || u32_at(bytes, 24)? != 1
            || u32_at(bytes, 28)? != 0
            || u32_at(bytes, 20)? != crc32(&bytes[32..])
        {
            return Err(Error::Invalid);
        }
        let note = Self { bytes };
        let mut seen = 0u32;
        for record in note.records() {
            let r = record?;
            validate_record(&r)?;
            let index = match r.kind {
                FIRMWARE => Some(0),
                ENTRY => Some(1),
                OWNERSHIP => Some(2),
                CLOCK => Some(3),
                LOG => Some(4),
                BOOT => Some(5),
                SHM_AREA => Some(6),
                SHM_REGION => None,
                _ if r.flags & 1 != 0 => return Err(Error::Mismatch),
                _ => None,
            };
            if let Some(i) = index {
                if seen & (1 << i) != 0 {
                    return Err(Error::Invalid);
                }
                seen |= 1 << i;
            }
        }
        // FIRMWARE, ENTRY, OWNERSHIP, CLOCK and BOOT are mandatory.
        if seen & 0x2f != 0x2f {
            return Err(Error::Invalid);
        }
        if let Some(area) = note.get(SHM_AREA) {
            let area = ShmArea::parse(area.value)?;
            let mut count = 0;
            for record in note
                .records()
                .filter_map(Result::ok)
                .filter(|r| r.kind == SHM_REGION)
            {
                count += 1;
                if count > 64 {
                    return Err(Error::Invalid);
                }
                let r = ShmRegion::parse(record.value)?;
                if r.ddr_offset
                    .checked_add(r.ddr_len)
                    .is_none_or(|end| end > area.size)
                    || u32::from(r.sram_offset) + u32::from(r.sram_len) > u32::from(area.sram_end)
                {
                    return Err(Error::Invalid);
                }
                for other in note
                    .records()
                    .filter_map(Result::ok)
                    .filter(|o| o.kind == SHM_REGION)
                {
                    if other.value.as_ptr() == record.value.as_ptr() {
                        break;
                    }
                    let o = ShmRegion::parse(other.value)?;
                    let overlap =
                        |a: u32, n: u32, b: u32, m: u32| n != 0 && m != 0 && a < b + m && b < a + n;
                    if r.name == o.name
                        || overlap(r.ddr_offset, r.ddr_len, o.ddr_offset, o.ddr_len)
                        || overlap(
                            r.sram_offset.into(),
                            r.sram_len.into(),
                            o.sram_offset.into(),
                            o.sram_len.into(),
                        )
                    {
                        return Err(Error::Invalid);
                    }
                }
            }
        } else if note.get(SHM_REGION).is_some() {
            return Err(Error::Invalid);
        }
        Ok(note)
    }
    pub fn records(&self) -> Records<'a> {
        Records {
            bytes: &self.bytes[32..],
        }
    }
    pub fn get(&self, kind: u16) -> Option<Record<'a>> {
        self.records()
            .filter_map(Result::ok)
            .find(|r| r.kind == kind)
    }
    pub fn as_bytes(&self) -> &'a [u8] {
        self.bytes
    }
}

fn take_string<'a>(bytes: &mut &'a [u8]) -> Result<&'a str, Error> {
    let (&n, rest) = bytes.split_first().ok_or(Error::Invalid)?;
    let s = rest.get(..n as usize).ok_or(Error::Invalid)?;
    *bytes = &rest[n as usize..];
    core::str::from_utf8(s).map_err(|_| Error::Invalid)
}
fn validate_record(r: &Record<'_>) -> Result<(), Error> {
    let v = r.value;
    if matches!(
        r.kind,
        FIRMWARE | ENTRY | OWNERSHIP | CLOCK | BOOT | SHM_AREA | SHM_REGION
    ) && r.flags != 1
    {
        return Err(Error::Invalid);
    }
    let valid = match r.kind {
        FIRMWARE => {
            let mut b = v;
            take_string(&mut b)?;
            take_string(&mut b)?;
            b.len() == 52
        }
        ENTRY => v.len() == 20,
        OWNERSHIP => {
            let n = u32_at(v, 0)? as usize;
            n.checked_mul(4).and_then(|n| n.checked_add(4)) == Some(v.len())
                && v[4..]
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .all(|c| (1..=5).contains(&c[0]) && (1..=3).contains(&c[1]))
        }
        CLOCK => v.len() == 5 && (1..=2).contains(&v[0]),
        LOG => v.len() == 5 && v[0] == 1 && u32_at(v, 1)? != 0,
        BOOT => {
            let mut b = v.get(64..).ok_or(Error::Invalid)?;
            take_string(&mut b)?;
            b.is_empty()
        }
        SHM_AREA => ShmArea::parse(v).is_ok(),
        SHM_REGION => ShmRegion::parse(v).is_ok(),
        _ => true,
    };
    if valid { Ok(()) } else { Err(Error::Invalid) }
}

/// SHM_AREA: the whole Linux DDR reservation (one per note).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ShmArea {
    /// Bytes of the reservation (multiple of 1 MiB, at least 1 MiB).
    pub size: u32,
    /// SRAM handoff/link-page base the loader writes (`crate::HANDOFF`).
    pub handoff: u32,
    /// End of the link-page bytes used by rings (offset from the link page).
    pub sram_end: u16,
    /// Physical (CPU) address of the reservation.
    pub phys: u64,
}
pub const SHM_AREA_LEN: usize = 24;
impl ShmArea {
    pub fn parse(v: &[u8]) -> Result<Self, Error> {
        let a = Self {
            size: u32_at(v, 0)?,
            handoff: u32_at(v, 8)?,
            sram_end: u16_at(v, 12)?,
            phys: u64::from(u32_at(v, 16)?) | (u64::from(u32_at(v, 20)?) << 32),
        };
        let valid = v.len() == SHM_AREA_LEN
            && u32_at(v, 4)? == 65536
            && u16_at(v, 14)? == 0
            && a.size >= 0x10_0000
            && a.size.is_multiple_of(0x10_0000)
            && a.size <= crate::SHM_MAX
            && a.phys.is_multiple_of(0x20_0000)
            && a.phys == crate::SHM_PHYS
            && a.handoff == crate::HANDOFF
            && a.sram_end as usize <= crate::LINK_LEN;
        if valid { Ok(a) } else { Err(Error::Invalid) }
    }
    pub fn encode(&self) -> [u8; SHM_AREA_LEN] {
        let mut v = [0; SHM_AREA_LEN];
        v[0..4].copy_from_slice(&self.size.to_le_bytes());
        v[4..8].copy_from_slice(&65536u32.to_le_bytes());
        v[8..12].copy_from_slice(&self.handoff.to_le_bytes());
        v[12..14].copy_from_slice(&self.sram_end.to_le_bytes());
        v[16..24].copy_from_slice(&self.phys.to_le_bytes());
        v
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShmKind {
    /// Ring from RP1 to Linux: slots and head in DDR, tail in the link page.
    RingToLinux,
    /// Ring from Linux to RP1: slots and head in the link page, tail in DDR.
    RingToRp1,
    Block {
        writer: Writer,
    },
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Writer {
    Rp1,
    Linux,
    Both,
}

/// SHM_REGION: one declared `shm`. Offsets are relative to the DDR reservation and the link page.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ShmRegion<'a> {
    pub kind: ShmKind,
    /// 1 = proc0, 2 = proc1.
    pub core: u8,
    pub notify_irq: bool,
    pub overwrite: bool,
    pub ddr_offset: u32,
    pub ddr_len: u32,
    pub sram_offset: u16,
    pub sram_len: u16,
    pub elem_size: u32,
    pub depth: u32,
    pub type_id: u64,
    pub name: &'a str,
    pub type_name: &'a str,
}
impl<'a> ShmRegion<'a> {
    pub fn parse(v: &'a [u8]) -> Result<Self, Error> {
        let mut b = v.get(44..).ok_or(Error::Invalid)?;
        let name = take_string(&mut b)?;
        let type_name = take_string(&mut b)?;
        let depth = u32_at(v, 24)?;
        let elem_size = u32_at(v, 20)?;
        let kind = match (v[0], v[1], v[5]) {
            (1, 1, 0) => ShmKind::RingToLinux,
            (1, 2, 0) => ShmKind::RingToRp1,
            (3, 0, w @ 1..=3) => ShmKind::Block {
                writer: [Writer::Rp1, Writer::Linux, Writer::Both][w as usize - 1],
            },
            _ => return Err(Error::Invalid),
        };
        let ring = !matches!(kind, ShmKind::Block { .. });
        let r = Self {
            kind,
            core: v[2],
            notify_irq: v[3] == 1,
            overwrite: v[6] == 2,
            ddr_offset: u32_at(v, 8)?,
            ddr_len: u32_at(v, 12)?,
            sram_offset: u16_at(v, 16)?,
            sram_len: u16_at(v, 18)?,
            elem_size,
            depth,
            type_id: u64::from(u32_at(v, 28)?) | (u64::from(u32_at(v, 32)?) << 32),
            name,
            type_name,
        };
        let valid = b.is_empty()
            && !name.is_empty()
            && matches!(v[2], 1 | 2)
            && v[3] <= 1
            && v[4] == 1
            && (1..=2).contains(&v[6])
            && v[7] == 1
            && elem_size != 0
            && elem_size <= crate::SHM_MAX - 8
            && (!ring || (depth.is_power_of_two() && depth <= 1 << 30 && elem_size <= 0xffff))
            && (ring || (depth == 0 && v[6] == 1))
            && r.ddr_offset.is_multiple_of(65536)
            && r.ddr_len.is_multiple_of(65536)
            && r.ddr_len != 0;
        if !valid {
            return Err(Error::Invalid);
        }
        let (ddr, sram) = r.storage()?;
        let start = usize::from(r.sram_offset);
        let end = start.checked_add(sram).ok_or(Error::Invalid)?;
        if ddr > r.ddr_len as usize
            || sram != usize::from(r.sram_len)
            || r.ddr_offset < 65536
            || r.ddr_offset
                .checked_add(r.ddr_len)
                .is_none_or(|e| e > crate::SHM_MAX)
            || (sram != 0
                && (start < 64
                    || !start.is_multiple_of(64)
                    || end > crate::LINK_LEN
                    || (start < 0x18c0 && end > 0x17c0)))
        {
            return Err(Error::Invalid);
        }
        Ok(r)
    }
    /// Bytes this region needs in DDR and in the link page; `Err(Invalid)` if a ring body
    /// does not fit 32 bits (checked arithmetic, identical on RP1 and the host).
    pub fn storage(&self) -> Result<(usize, usize), Error> {
        let body = || {
            crate::shm::ring_body_size(self.elem_size as usize, self.depth as usize)
                .ok_or(Error::Invalid)
        };
        Ok(match self.kind {
            ShmKind::RingToLinux => (body()?, crate::shm::CONTROL_SIZE),
            ShmKind::RingToRp1 => (crate::shm::CONTROL_SIZE, body()?),
            ShmKind::Block { .. } => (crate::shm::block_size(self.elem_size as usize), 0),
        })
    }
}

pub struct Records<'a> {
    bytes: &'a [u8],
}
impl<'a> Iterator for Records<'a> {
    type Item = Result<Record<'a>, Error>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.bytes.is_empty() {
            return None;
        }
        let result = (|| {
            let kind = u16_at(self.bytes, 0)?;
            let flags = u16_at(self.bytes, 2)?;
            if flags & !1 != 0 {
                return Err(Error::Invalid);
            }
            let len = u32_at(self.bytes, 4)? as usize;
            let end = 8usize.checked_add(len).ok_or(Error::Invalid)?;
            let padded = end.checked_add(3).ok_or(Error::Invalid)? & !3;
            let value = self.bytes.get(8..end).ok_or(Error::Invalid)?;
            if self
                .bytes
                .get(end..padded)
                .ok_or(Error::Invalid)?
                .iter()
                .any(|b| *b != 0)
            {
                return Err(Error::Invalid);
            }
            self.bytes = self.bytes.get(padded..).ok_or(Error::Invalid)?;
            Ok(Record { kind, flags, value })
        })();
        if result.is_err() {
            self.bytes = &[];
        }
        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate std;
    #[test]
    fn crc_and_truncated_records() {
        assert_eq!(crc32(b"123456789"), 0xcbf4_3926);
        for len in 1..8 {
            let b = [0; 8];
            assert!(Records { bytes: &b[..len] }.next().unwrap().is_err());
        }
        assert!(Note::parse(&[0; 32]).is_err());
    }

    fn region(depth: u32, ddr_len: u32) -> std::vec::Vec<u8> {
        let mut v = std::vec![1, 1, 1, 0, 1, 0, 1, 1];
        v.extend(65536u32.to_le_bytes());
        v.extend(ddr_len.to_le_bytes());
        v.extend(64u16.to_le_bytes());
        v.extend(128u16.to_le_bytes());
        v.extend(4u32.to_le_bytes());
        v.extend(depth.to_le_bytes());
        v.extend([0; 16]);
        v.extend([2, b'u', b'p', 3, b'u', b'3', b'2']);
        v
    }

    #[test]
    fn ring_size_overflow_is_rejected_not_wrapped() {
        // REVIEW #14: 128 + 2^30 * 16 wraps to 128 in 32-bit usize arithmetic.
        assert_eq!(crate::shm::ring_body_size(4, 1 << 30), None);
        assert_eq!(
            ShmRegion::parse(&region(1 << 30, 65536)).err(),
            Some(Error::Invalid)
        );
        assert_eq!(crate::shm::ring_body_size(4, 4), Some(128 + 4 * 16));
        let ok = region(16, 65536);
        assert_eq!(
            ShmRegion::parse(&ok).unwrap().storage(),
            Ok((128 + 16 * 16, 128))
        );
    }
}
