use crate::{
    Result,
    metadata::{Declaration, peripheral_id},
};
use rp1_proto::note::*;

fn word(out: &mut Vec<u8>, n: u32) {
    out.extend(n.to_le_bytes());
}
fn short(out: &mut Vec<u8>, n: u16) {
    out.extend(n.to_le_bytes());
}
fn string(out: &mut Vec<u8>, s: &str) -> Result<()> {
    out.push(u8::try_from(s.len())?);
    out.extend(s.bytes());
    Ok(())
}
fn tlv(out: &mut Vec<u8>, kind: u16, critical: bool, value: &[u8]) {
    short(out, kind);
    short(out, u16::from(critical));
    word(out, value.len() as u32);
    out.extend(value);
    while !out.len().is_multiple_of(4) {
        out.push(0);
    }
}
pub fn descriptor(
    name: &str,
    version: &str,
    d: &Declaration,
    hash: [u8; 32],
    source: [u8; 20],
) -> Result<Vec<u8>> {
    let mut out = vec![0; 32];
    let mut v = Vec::new();
    string(&mut v, name)?;
    string(&mut v, version)?;
    v.extend(source);
    v.extend(hash);
    tlv(&mut out, FIRMWARE, true, &v);
    v.clear();
    for n in [
        0x2000_0401,
        0x1000_4000,
        0x2000_0000,
        0x2000_0000,
        rp1_proto::IMAGE_END,
    ] {
        word(&mut v, n);
    }
    tlv(&mut out, ENTRY, true, &v);
    let mut claims = Vec::new();
    for (core, c) in [(1, &d.proc0), (2, &d.proc1)] {
        for pin in &c.pins {
            claims.extend([1, core]);
            short(&mut claims, u16::from(*pin));
        }
        for p in &c.peripherals {
            let (kind, id) = peripheral_id(p)?;
            claims.extend([kind, core]);
            short(&mut claims, id);
        }
    }
    if d.log.is_some() {
        for (kind, id) in [(1, 14), (1, 15), (2, 0)] {
            claims.extend([kind, 3]);
            short(&mut claims, id);
        }
    }
    if d.clock.authority == "rp1" {
        claims.extend([5, 3]);
        short(&mut claims, 0);
    }
    v.clear();
    word(&mut v, claims.len() as u32 / 4);
    v.extend(claims);
    tlv(&mut out, OWNERSHIP, true, &v);
    v.clear();
    v.push(if d.clock.authority == "rp1" { 1 } else { 2 });
    word(
        &mut v,
        if d.clock.authority == "rp1" {
            200_000_000
        } else {
            0
        },
    );
    tlv(&mut out, CLOCK, true, &v);
    if let Some(l) = &d.log {
        v.clear();
        v.push(1);
        word(&mut v, l.baudrate);
        tlv(&mut out, LOG, false, &v);
    }
    // Loader/bundle construction is out of scope; zero hashes explicitly mean unbound.
    v = vec![0; 64];
    string(&mut v, "linux_2712.img")?;
    tlv(&mut out, BOOT, true, &v);
    let regions = d.regions()?;
    let ddr_end = regions.last().map_or(65536, |r| r.ddr_offset + r.ddr_len);
    let sram_end = regions
        .last()
        .map_or(64, |r| u32::from(r.sram_offset) + u32::from(r.sram_len));
    v.clear();
    word(&mut v, ddr_end);
    word(&mut v, 65536);
    word(&mut v, rp1_proto::HANDOFF);
    short(&mut v, sram_end as u16);
    tlv(&mut out, SHM_AREA, true, &v);
    for r in regions {
        let spec = r.spec;
        v = vec![
            if spec.kind == "ring" { 1 } else { 3 },
            if spec.direction.as_deref() == Some("rp1-to-linux") {
                1
            } else if spec.kind == "ring" {
                2
            } else {
                0
            },
            if spec.core == "proc0" { 1 } else { 2 },
            if spec.notify.as_deref() == Some("irq") {
                1
            } else {
                0
            },
            1,
            match spec.writer.as_deref() {
                Some("rp1") => 1,
                Some("linux") => 2,
                Some("both") => 3,
                _ => 0,
            },
            if spec.overflow.as_deref() == Some("overwrite") {
                2
            } else {
                1
            },
            1,
        ];
        word(&mut v, r.ddr_offset);
        word(&mut v, r.ddr_len);
        short(&mut v, r.sram_offset);
        short(&mut v, r.sram_len);
        word(&mut v, r.ty.size as u32);
        word(&mut v, spec.depth.unwrap_or(0));
        v.extend(r.ty.id.to_le_bytes());
        v.extend(0u64.to_le_bytes());
        string(&mut v, &spec.name)?;
        string(
            &mut v,
            spec.message.as_ref().or(spec.block_type.as_ref()).unwrap(),
        )?;
        tlv(&mut out, SHM_REGION, true, &v);
    }
    out[..8].copy_from_slice(MAGIC);
    out[8..10].copy_from_slice(&2u16.to_le_bytes());
    out[10..12].copy_from_slice(&32u16.to_le_bytes());
    let len = out.len() as u32;
    out[12..16].copy_from_slice(&len.to_le_bytes());
    let crc = crc32(&out[32..]);
    out[20..24].copy_from_slice(&crc.to_le_bytes());
    out[24..28].copy_from_slice(&1u32.to_le_bytes());
    Ok(out)
}
pub fn elf_note(desc: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    word(&mut out, 4);
    word(&mut out, desc.len() as u32);
    word(&mut out, ELF_TYPE);
    out.extend(b"RP1\0");
    out.extend(desc);
    while out.len() % 4 != 0 {
        out.push(0);
    }
    out
}
