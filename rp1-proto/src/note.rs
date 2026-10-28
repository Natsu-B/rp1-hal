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
        SHM_AREA => {
            v.len() == 14 && u32_at(v, 4)? == 65536 && u16_at(v, 12)? as usize <= crate::LINK_LEN
        }
        SHM_REGION => {
            let mut b = v.get(44..).ok_or(Error::Invalid)?;
            let name = take_string(&mut b)?;
            take_string(&mut b)?;
            b.is_empty()
                && !name.is_empty()
                && matches!(v[0], 1 | 3)
                && matches!(v[2], 1 | 2)
                && v[3] <= 1
                && v[4] == 1
                && (1..=2).contains(&v[6])
                && v[7] == 1
                && ((v[0] == 1
                    && matches!(v[1], 1 | 2)
                    && v[5] == 0
                    && u32_at(v, 24)?.is_power_of_two())
                    || (v[0] == 3 && v[1] == 0 && (1..=3).contains(&v[5]) && u32_at(v, 24)? == 0))
        }
        _ => true,
    };
    if valid { Ok(()) } else { Err(Error::Invalid) }
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
    #[test]
    fn crc_and_truncated_records() {
        assert_eq!(crc32(b"123456789"), 0xcbf4_3926);
        for len in 1..8 {
            let b = [0; 8];
            assert!(Records { bytes: &b[..len] }.next().unwrap().is_err());
        }
        assert!(Note::parse(&[0; 32]).is_err());
    }
}
