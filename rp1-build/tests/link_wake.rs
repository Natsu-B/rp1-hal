use rp1_build::{
    generate,
    metadata::{Declaration, TypeInfo},
    wire,
};
use rp1_proto::{
    note::{self, Note, ShmArea, ShmRegion},
    wake::{LATCHED_WAKE, mailbox},
};

const DECLARATION: &str = r#"
[proc1]
host_irq = true
doorbell = 27

[[shm]]
name = "up"
kind = "ring"
core = "proc1"
direction = "rp1-to-linux"
message = "u32"
depth = 4
notify = "irq"
"#;

fn artifacts(profile: &str) -> rp1_build::Artifacts {
    let mut d: Declaration =
        toml::from_str(&format!("schema=2\n{profile}\n{DECLARATION}")).unwrap();
    d.types.insert("u32".into(), TypeInfo::of::<u32>());
    generate("wake", "0.1.0", &d, d.declaration_hash().unwrap(), [0; 20]).unwrap()
}

#[test]
fn schemas_keep_v2_and_v3_note_layouts_distinct() {
    for (profile, version, ty, start) in [
        ("", 2u16, note::ELF_TYPE, 0xc0),
        ("link_wake='experimental'", 3, note::ELF_TYPE_V3, 0x100),
    ] {
        let a = artifacts(profile);
        let envelope = wire::elf_note(&a.note);
        assert_eq!(note::u32_at(&envelope, 8).unwrap(), ty);
        let parsed = Note::parse(note::descriptor(&envelope).unwrap()).unwrap();
        assert_eq!(parsed.version(), version);
        assert!(a.dts.contains(&format!("rp1,abi = <{version}>;")));
        let region = ShmRegion::parse(parsed.get(note::SHM_REGION).unwrap().value).unwrap();
        assert_eq!(region.sram_offset, start);
        assert_eq!(region.sram_len, 128);
        assert_eq!(region.ddr_offset, 65536);
        let wake = parsed.get(note::LINK_WAKE);
        if version == 3 {
            let wake = wake.unwrap();
            assert_eq!(wake.flags, 1);
            assert_eq!(
                wake.value,
                [1, 0, 0xc0, 0, 64, 0, 1, 0xff, 1, 0, 0, 0, 1, 0, 0, 0]
            );
            let config = note::LinkWake::parse(wake.value).unwrap();
            assert_eq!(config.required_caps, LATCHED_WAKE);
            assert_eq!(config.offered_caps, LATCHED_WAKE);
        } else {
            assert!(wake.is_none());
        }
        let mut mixed = envelope;
        mixed[8] = if version == 3 { 2 } else { 3 };
        assert!(note::descriptor(&mixed).is_err());
    }
}

fn reseal(bytes: &mut [u8]) {
    let len = (bytes.len() as u32).to_le_bytes();
    let crc = note::crc32(&bytes[32..]).to_le_bytes();
    bytes[12..16].copy_from_slice(&len);
    bytes[20..24].copy_from_slice(&crc);
}

#[test]
fn malformed_v3_record_and_stale_layout_are_rejected() {
    let a = artifacts("link_wake='experimental'");
    let note = Note::parse(&a.note).unwrap();
    let offset = |kind| note.get(kind).unwrap().value.as_ptr() as usize - a.note.as_ptr() as usize;
    let wake = offset(note::LINK_WAKE);
    let region = offset(note::SHM_REGION);
    let route = offset(note::LINK) + 9;
    for (at, byte) in [
        (wake - 8, 0x29), // unknown critical TLV and missing LINK_WAKE
        (wake - 6, 0),    // non-critical
        (wake, 2),        // unknown wake ABI
        (wake + 2, 0x80), // mailbox overlaps status
        (wake + 4, 32),   // mailbox truncated
        (wake + 6, 0),    // wrong vector owner
        (wake + 7, 26),   // unadvertised startup route
        (wake + 8, 3),    // required capability absent
        (wake + 8, 0x80), // unknown required capability
        (wake + 12, 0),   // promised capabilities absent
        (route, 58),      // wrong MSI-X vector
    ] {
        let mut changed = a.note.clone();
        changed[at] = byte;
        assert!(Note::parse(&changed).is_err(), "CRC at offset {at}");
        reseal(&mut changed);
        assert!(Note::parse(&changed).is_err(), "invalid TLV at offset {at}");
    }

    let mut overlaps_mailbox = a.note.clone();
    overlaps_mailbox[region + 16..region + 18]
        .copy_from_slice(&(mailbox::START as u16).to_le_bytes());
    reseal(&mut overlaps_mailbox);
    assert!(Note::parse(&overlaps_mailbox).is_err());

    let mut duplicate = a.note.clone();
    duplicate.extend_from_slice(&a.note[wake - 8..wake + note::LinkWake::LEN]);
    reseal(&mut duplicate);
    assert!(Note::parse(&duplicate).is_err());

    let mut legacy = a.note.clone();
    legacy[8] = 2;
    assert!(
        Note::parse(&legacy).is_err(),
        "critical LINK_WAKE cannot enter a v2 descriptor"
    );

    let mut changed = a.note.clone();
    changed[offset(note::SHM_AREA) + 12..offset(note::SHM_AREA) + 14]
        .copy_from_slice(&0xc0u16.to_le_bytes());
    assert_eq!(
        ShmArea::parse(note.get(note::SHM_AREA).unwrap().value)
            .unwrap()
            .sram_end,
        0x180
    );
    reseal(&mut changed);
    assert!(Note::parse(&changed).is_err());
}

#[test]
fn experimental_route_respects_the_existing_ownership_checks() {
    for (cores, valid) in [
        ("[proc1]\nhost_irq=true\ndoorbell=27", true),
        ("[proc1]\ndoorbell=27", false),
        ("[proc0]\nhost_irq=true\n[proc1]\nhost_irq=true", false),
        ("[proc0]\ntimer=['ALARM3']\n[proc1]\nhost_irq=true", false),
        (
            "[proc0]\ngpio=[27]\n[proc1]\nhost_irq=true\ndoorbell=27",
            false,
        ),
    ] {
        let d: Declaration =
            toml::from_str(&format!("schema=2\nlink_wake='experimental'\n{cores}")).unwrap();
        assert_eq!(d.validate().is_ok(), valid, "{cores}");
    }
}
