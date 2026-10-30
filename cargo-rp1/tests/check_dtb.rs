//! T5: `cargo rp1 check --dtb` predicts rp1boot's boot-time disables (fixture board DTB).
use rp1_build::{generate, metadata::Declaration};
use std::{ffi::OsString, fs, path::Path, process::Command};

const BOARD: &str = include_str!("../../rp1-build/tests/fixtures/board.dts");
const GPIOMEM: &str = r#"/ { gpiomem@c0400d0000 { compatible = "raspberrypi,gpiomem"; reg = <0xc0 0x400d0000 0x30000>; }; };"#;

fn dtc(dir: &Path, name: &str, source: &str) -> Vec<u8> {
    let dts = dir.join(format!("{name}.dts"));
    let out = dir.join(format!("{name}.dtb"));
    fs::write(&dts, source).unwrap();
    let o = Command::new("dtc")
        .args(["-@", "-q", "-I", "dts", "-O", "dtb", "-o"])
        .arg(&out)
        .arg(&dts)
        .output()
        .expect("dtc required for DTB fixture tests");
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    fs::read(out).unwrap()
}

/// Minimal ELF32 with the two sections `check_dtb` reads.
fn elf(note_section: &[u8], dtbo: &[u8]) -> Vec<u8> {
    let shstrtab = b"\0.note.rp1\0.rp1.dtbo\0.shstrtab\0";
    let mut out = vec![0u8; 52];
    let mut sections = Vec::new(); // (name offset, type, offset, size)
    for (name, kind, data) in [
        (1u32, 7u32, note_section),
        (11, 1, dtbo),
        (21, 3, shstrtab.as_slice()),
    ] {
        while !out.len().is_multiple_of(4) {
            out.push(0);
        }
        sections.push((name, kind, out.len() as u32, data.len() as u32));
        out.extend_from_slice(data);
    }
    while !out.len().is_multiple_of(4) {
        out.push(0);
    }
    let shoff = out.len() as u32;
    out.extend_from_slice(&[0; 40]);
    for (name, kind, offset, size) in sections {
        for v in [name, kind, 0, 0, offset, size, 0, 0, 4, 0] {
            out.extend_from_slice(&v.to_le_bytes());
        }
    }
    let mut header = b"\x7fELF\x01\x01\x01\0\0\0\0\0\0\0\0\0".to_vec();
    header.extend(2u16.to_le_bytes()); // ET_EXEC
    header.extend(40u16.to_le_bytes()); // EM_ARM
    header.extend(1u32.to_le_bytes()); // version
    header.extend(0u32.to_le_bytes()); // entry
    header.extend(0u32.to_le_bytes()); // phoff
    header.extend(shoff.to_le_bytes());
    header.extend(0x0500_0000u32.to_le_bytes()); // EABI5
    for v in [52u16, 32, 0, 40, 4, 3] {
        header.extend(v.to_le_bytes()); // ehsize, phentsize, phnum, shentsize, shnum, shstrndx
    }
    out[..52].copy_from_slice(&header);
    out
}

fn firmware(dir: &Path, d: &Declaration) -> Vec<u8> {
    let a = generate("test", "0.1.0", d, [0; 32], [0; 20]).unwrap();
    let dtbo = dtc(dir, "app", &a.dts);
    elf(&rp1_build::wire::elf_note(&a.note), &dtbo)
}

fn board(dir: &Path, extra_rp1: &str, extra_root: &str) -> Vec<u8> {
    dtc(
        dir,
        "board",
        &format!(
            "{}\n{extra_root}\n",
            BOARD
                .replace("@EXTRA_RESERVED@", "")
                .replace("@EXTRA_RP1@", extra_rp1)
        ),
    )
}

fn declaration() -> Declaration {
    toml::from_str("schema=2\nlog={sink='uart0',baudrate=115200}\n[proc0]\ngpio=[22]").unwrap()
}

#[test]
fn clean_board_and_gpiomem_policy_pass() {
    let tmp = tempfile::tempdir().unwrap();
    let fw = firmware(tmp.path(), &declaration());
    let lines = cargo_rp1::check_dtb(&fw, &board(tmp.path(), "", ""), false).unwrap();
    assert!(lines.is_empty(), "{lines:?}");
    let lines = cargo_rp1::check_dtb(&fw, &board(tmp.path(), "", GPIOMEM), false).unwrap();
    assert_eq!(lines.len(), 1, "{lines:?}");
    assert!(lines[0].contains("gpiomem@c0400d0000") && lines[0].contains("gpiomem policy"));
}

#[test]
fn conflicting_node_fails_unless_boot_disable_is_allowed() {
    let tmp = tempfile::tempdir().unwrap();
    let fw = firmware(tmp.path(), &declaration());
    let dtb = board(tmp.path(), r#"led { pinctrl-0 = <&gpio22>; };"#, "");
    let e = cargo_rp1::check_dtb(&fw, &dtb, false)
        .unwrap_err()
        .to_string();
    assert!(e.contains("rp1boot would disable /rp1/led"), "{e}");
    assert!(e.contains("--allow-boot-disable"), "{e}");
    let lines = cargo_rp1::check_dtb(&fw, &dtb, true).unwrap();
    assert!(
        lines
            .iter()
            .any(|l| l.contains("/rp1/led") && l.contains("GPIO22"))
    );
    assert!(
        lines
            .iter()
            .any(|l| l.starts_with("warning: rp1boot will disable 1"))
    );
}

#[test]
fn refusal_fails_even_with_allow_boot_disable() {
    let tmp = tempfile::tempdir().unwrap();
    let fw = firmware(tmp.path(), &declaration());
    let dtb = board(
        tmp.path(),
        r#"led { pinctrl-0 = <&gpio22>; };"#,
        r#"/ { chosen { stdout-path = "/rp1/led"; }; };"#,
    );
    let e = cargo_rp1::check_dtb(&fw, &dtb, true)
        .unwrap_err()
        .to_string();
    assert!(e.contains("would refuse") && e.contains("console"), "{e}");
    assert!(!e.contains("--allow-boot-disable"), "{e}");
    // The declaration's strict policy refuses as well.
    let strict: Declaration =
        toml::from_str("schema=2\n[linux]\non_conflict='refuse'\n[proc0]\ngpio=[22]").unwrap();
    let fw = firmware(tmp.path(), &strict);
    let dtb = board(tmp.path(), r#"led { pinctrl-0 = <&gpio22>; };"#, "");
    let e = cargo_rp1::check_dtb(&fw, &dtb, true)
        .unwrap_err()
        .to_string();
    assert!(e.contains("on_conflict=refuse"), "{e}");
}

#[test]
fn check_arguments() {
    let args = |a: &[&str]| a.iter().map(OsString::from).collect::<Vec<_>>();
    let c = cargo_rp1::parse_check_args(&args(&["fw.elf"])).unwrap();
    assert_eq!((c.dtb, c.allow_boot_disable), (None, false));
    for order in [
        &["fw.elf", "--dtb", "b.dtb", "--allow-boot-disable"][..],
        &["--allow-boot-disable", "fw.elf", "--dtb", "b.dtb"],
        &["--dtb", "b.dtb", "fw.elf", "--allow-boot-disable"],
    ] {
        let c = cargo_rp1::parse_check_args(&args(order)).unwrap();
        assert_eq!(c.elf, Path::new("fw.elf"));
        assert_eq!(c.dtb.as_deref(), Some(Path::new("b.dtb")));
        assert!(c.allow_boot_disable);
    }
    for bad in [
        &[][..],
        &["fw.elf", "--dtb"],
        &["fw.elf", "--allow-boot-disable"],
        &["fw.elf", "--dtb", "a", "--dtb", "b"],
        &["fw.elf", "other.elf"],
        &["fw.elf", "--frobnicate"],
    ] {
        assert!(cargo_rp1::parse_check_args(&args(bad)).is_err(), "{bad:?}");
    }
}
