use crate::{
    Result,
    metadata::{Declaration, Pair, ShmLayout, peripheral_id},
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
    descriptor_with_layout(name, version, d, hash, source, &d.shm_layout()?)
}

pub(crate) fn descriptor_with_layout(
    name: &str,
    version: &str,
    d: &Declaration,
    hash: [u8; 32],
    source: [u8; 20],
    shm: &ShmLayout<'_>,
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
        for pin in &c.pins() {
            claims.extend([ownership_kind::PIN, core]);
            short(&mut claims, u16::from(*pin));
        }
        // DMA reserves the whole controller from Linux. Its channel list is build-time only.
        for p in &c.peripherals() {
            let (kind, id) = peripheral_id(p)?;
            claims.extend([kind, core]);
            short(&mut claims, id);
        }
        if !c.gpio_irq.is_empty() || c.doorbell.is_some() {
            claims.extend([ownership_kind::GPIO_IRQ_BANK, core]);
            short(&mut claims, 0);
        }
    }
    if d.log.is_some() {
        for (kind, id) in [
            (ownership_kind::PIN, 14),
            (ownership_kind::PIN, 15),
            (ownership_kind::BLOCK, 0),
        ] {
            claims.extend([kind, 3]);
            short(&mut claims, id);
        }
    }
    if d.clock.authority == "rp1" {
        claims.extend([ownership_kind::CLOCK, 3]);
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
    let core = |c: &crate::metadata::Core| LinkCore {
        doorbell: c.doorbell,
        host_irq_vector: c.host_irq.then_some(rp1_proto::irq::VECTOR as u8),
        heartbeat_ms: c.heartbeat.map(|h| h.period_ms),
    };
    let link = Link {
        cores: [core(&d.proc0), core(&d.proc1)],
    };
    tlv(&mut out, LINK, false, &link.encode());
    tlv(&mut out, SHM_AREA, true, &shm.area.encode());
    for region in &shm.regions {
        let r = region.wire()?;
        let (kind, direction, writer) = match r.kind {
            ShmKind::RingToLinux => (1, 1, 0),
            ShmKind::RingToRp1 => (1, 2, 0),
            ShmKind::Block { writer } => (
                3,
                0,
                match writer {
                    Writer::Rp1 => 1,
                    Writer::Linux => 2,
                    Writer::Both => 3,
                },
            ),
        };
        v = vec![
            kind,
            direction,
            r.core,
            u8::from(r.notify_irq),
            1,
            writer,
            if r.overwrite { 2 } else { 1 },
            1,
        ];
        word(&mut v, r.ddr_offset);
        word(&mut v, r.ddr_len);
        short(&mut v, r.sram_offset);
        short(&mut v, r.sram_len);
        word(&mut v, r.elem_size);
        word(&mut v, r.depth);
        v.extend(r.type_id.to_le_bytes());
        v.extend(0u64.to_le_bytes());
        string(&mut v, r.name)?;
        string(&mut v, r.type_name)?;
        tlv(&mut out, SHM_REGION, true, &v);
    }
    for endpoint in shm.endpoints.iter().filter(|e| e.pair == Some(Pair::Rpc)) {
        v.clear();
        string(&mut v, endpoint.name)?;
        tlv(&mut out, RPC, false, &v);
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
