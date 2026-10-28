use rp1_build::{
    generate,
    metadata::{Declaration, TypeInfo},
};
use rp1_proto::note::{ENTRY, Note};
use std::{fs, path::Path, process::Command};

fn declaration() -> Declaration {
    toml::from_str(
        r#"
schema=1
board="rpi-cm5"
log={sink="uart0",baudrate=115200}
[proc0]
pins=[22]
peripherals=["TIMER.ALARM0"]
[proc1]
pins=[27]
gpio_irq=[27]
peripherals=["TIMER.ALARM1"]
"#,
    )
    .unwrap()
}
#[test]
fn rejects_mistakes_before_building() {
    let mut d = declaration();
    d.validate().unwrap();
    d.proc0.pins.push(14);
    assert!(d.validate().is_err());
    d.proc0.pins.pop();
    // The log owns UART0: no driver may also take it (review #6).
    d.proc0.peripherals.push("UART0".into());
    assert!(d.validate().is_err());
    d.proc0.peripherals.pop();
    d.proc1.peripherals.push("TIMER.ALARM0".into());
    assert!(d.validate().is_err());
    d.proc1.peripherals.pop();
    d.proc1.gpio_irq.push(26);
    assert!(d.validate().is_err());
    d.proc1.gpio_irq.pop();
    d.clock.authority = "linux".into();
    assert!(d.validate().is_err());
    assert!(toml::from_str::<Declaration>("typo=1").is_err());
}
#[test]
fn i2c_ownership_and_board_management_pins() {
    use rp1_build::metadata::{peripheral_id, peripheral_type};
    for n in 0..4 {
        assert_eq!(peripheral_id(&format!("I2C{n}")).unwrap(), (2, 32 + n));
    }
    for p in ["I2C4", "I2C5", "I2C6", "I2C7", "I2C01"] {
        assert!(peripheral_type(p).is_err());
    }
    let mut d = declaration();
    d.proc0.pins.extend([2, 3]);
    assert!(d.validate().is_err());
    d.proc1.peripherals.push("I2C1".into());
    assert!(
        d.validate().is_err(),
        "a different core cannot unlock GPIO2/3"
    );
    d.proc1.peripherals.clear();
    d.proc0.peripherals.push("I2C1".into());
    d.validate().unwrap();
    let a = generate("i2c", "0.1.0", &d, [0; 32], [0; 20]).unwrap();
    assert!(a.dts.contains("target = <&rp1_i2c1>"));
    let note = Note::parse(&a.note).unwrap();
    assert!(
        note.get(rp1_proto::note::OWNERSHIP).unwrap().value[4..]
            .as_chunks::<4>()
            .0
            .contains(&[2, 1, 33, 0])
    );
    d.proc1.peripherals.push("I2C1".into());
    assert!(d.validate().is_err());
    d.proc1.peripherals.clear();
    d.proc0.pins.push(0);
    assert!(d.validate().is_err(), "Linux UART1 retains GPIO0/1");
}
/// Review #7/#9: declarations v0.1 cannot deliver are build errors, not silent no-ops.
#[test]
fn rejects_unsupported_clock_authority_and_notify() {
    let mut d: Declaration = toml::from_str(
        r#"
[clock]
authority="linux"
[proc0]
pins=[22]
"#,
    )
    .unwrap();
    assert!(d.log.is_none() && d.proc0.peripherals.is_empty());
    assert!(
        d.validate().is_err(),
        "GPIO-only firmware must not take authority=linux"
    );
    d.clock.authority = "rp1".into();
    d.validate().unwrap();
    for (notify, ok) in [("poll", true), ("irq", false), ("doorbell", false)] {
        let d: Declaration = toml::from_str(&format!(
            r#"
[[shm]]
name="up"
kind="ring"
core="proc1"
message="u32"
direction="rp1-to-linux"
depth=16
notify="{notify}"
"#
        ))
        .unwrap();
        assert_eq!(d.validate().is_ok(), ok, "notify={notify}");
    }
}
#[test]
fn typed_regions_and_sram_budget() {
    let mut d: Declaration = toml::from_str(
        r#"
[[shm]]
name="commands"
kind="ring"
core="proc1"
message="u32"
direction="linux-to-rp1"
depth=16
"#,
    )
    .unwrap();
    d.validate().unwrap();
    assert!(d.regions().is_err());
    d.types.insert("u32".into(), TypeInfo::of::<u32>());
    let r = d.regions().unwrap();
    assert_eq!(r[0].sram_len, 384);
    assert_eq!(r[0].ddr_offset, 65536);
    let a = generate("test", "0.1.0", &d, [0; 32], [0; 20]).unwrap();
    assert!(Note::parse(&a.note).is_ok());
    let area = rp1_proto::note::ShmArea::parse(
        Note::parse(&a.note)
            .unwrap()
            .get(rp1_proto::note::SHM_AREA)
            .unwrap()
            .value,
    )
    .unwrap();
    assert_eq!((area.phys, area.size), (0x2200_0000, 0x10_0000));
    d.shm[0].depth = Some(4096);
    assert!(d.regions().is_err());
    d.shm[0].placement = Some("linux".into());
    assert!(d.validate().is_err());
}
#[test]
fn note_is_deterministic_and_rejects_corruption() {
    let d = declaration();
    let a = generate("test", "0.1.0", &d, [1; 32], [2; 20]).unwrap();
    assert_eq!(
        a.note,
        generate("test", "0.1.0", &d, [1; 32], [2; 20])
            .unwrap()
            .note
    );
    let n = Note::parse(&a.note).unwrap();
    assert_eq!(
        &n.get(ENTRY).unwrap().value[..4],
        &0x2000_0401u32.to_le_bytes()
    );
    let mut bad = a.note;
    bad[40] ^= 1;
    assert!(Note::parse(&bad).is_err());
}
#[test]
fn large_block_and_malformed_region() {
    let mut d: Declaration = toml::from_str(
        r#"
[[shm]]
name="bulk"
kind="block"
core="proc1"
type="Bulk"
writer="both"
"#,
    )
    .unwrap();
    d.types
        .insert("Bulk".into(), TypeInfo::of::<[u32; 262144]>());
    let mut a = generate("test", "0.1.0", &d, [0; 32], [0; 20]).unwrap();
    let n = Note::parse(&a.note).unwrap();
    let region = n.get(rp1_proto::note::SHM_REGION).unwrap().value;
    let at = region.as_ptr() as usize - a.note.as_ptr() as usize;
    assert_eq!(d.ddr_size().unwrap(), 0x20_0000);
    // A valid CRC cannot hide an allocation that overlaps the area header.
    a.note[at + 8..at + 12].copy_from_slice(&0u32.to_le_bytes());
    let crc = rp1_proto::note::crc32(&a.note[32..]);
    a.note[20..24].copy_from_slice(&crc.to_le_bytes());
    assert!(Note::parse(&a.note).is_err());
}
#[test]
fn overlay_applies_to_a_copy_of_scmi_linux() {
    let base = std::env::var_os("RP1_TEST_DTB")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| Path::new("<lab>").to_owned());
    if !base.is_file() {
        eprintln!("STATIC fixture unavailable: set RP1_TEST_DTB for integration check");
        return;
    }
    let before = fs::read(&base).unwrap();
    let tmp = tempfile::tempdir().unwrap();
    let copy = tmp.path().join("base.dtb");
    fs::write(&copy, &before).unwrap();
    let source = tmp.path().join("app.dts");
    let overlay = tmp.path().join("app.dtbo");
    let merged = tmp.path().join("merged.dtb");
    let a = generate("test", "0.1.0", &declaration(), [0; 32], [0; 20]).unwrap();
    fs::write(&source, a.dts).unwrap();
    rp1_build::overlay::compile(&source, &overlay).unwrap();
    rp1_build::overlay::apply(&copy, &overlay, &merged).unwrap();
    let out = Command::new("dtc")
        .args(["-I", "dtb", "-O", "dts"])
        .arg(&merged)
        .output()
        .unwrap();
    assert!(out.status.success());
    let dts = String::from_utf8(out.stdout).unwrap();
    assert!(dts.contains("rp1:link"));
    assert!(dts.contains("gpio-reserved-ranges"));
    // rp1-mailbox stays bound (SCMI transport); no UIO node takes an interrupt.
    let base_dts = Command::new("dtc")
        .args(["-I", "dtb", "-O", "dts"])
        .arg(&copy)
        .output()
        .unwrap();
    let base_dts = String::from_utf8(base_dts.stdout).unwrap();
    let mbox = |dts: &str| {
        dts.split("mailbox@8000 {")
            .nth(1)
            .unwrap()
            .split("};")
            .next()
            .unwrap()
            .to_string()
    };
    assert_eq!(
        mbox(&dts),
        mbox(&base_dts),
        "overlay must not touch rp1-mailbox"
    );
    assert!(!dts.contains("rp1:events"));
    for uio in dts.split("\"generic-uio\"").skip(1) {
        let body = &uio[..uio.find("};").unwrap()];
        assert!(
            !body.contains("interrupts"),
            "UIO node with interrupts: {body}"
        );
    }
    // Linux never gains pins: the merged reservation is a superset of the base one.
    let base_pins = rp1_build::overlay::reserved_gpios(&copy).unwrap();
    let merged_pins = rp1_build::overlay::reserved_gpios(&merged).unwrap();
    assert!(merged_pins.is_superset(&base_pins));
    assert!([14, 15, 22, 27].iter().all(|p| merged_pins.contains(p)));
    // Root and /reserved-memory are <2 1>: regs must be 3 cells.
    assert!(
        dts.contains("reg = <0x1f 0x40c000 0x4000>;"),
        "link reg cells"
    );
    assert!(
        dts.contains("reg = <0x00 0x21000000 0x10000>;"),
        "scratch reg cells"
    );
    assert!(dts.contains("rp1-shm@22000000"));
    assert!(dts.contains("reg = <0x00 0x22000000 0x100000>;"));
    let status = |node: &str| {
        let body = dts.split(node).nth(1).unwrap();
        body[..body.find("};").unwrap()].contains("status = \"disabled\"")
    };
    assert!(status("openamp-address-space {"), "remoteproc not disabled");
    assert!(!status("scmi {"), "SCMI must stay enabled");
    let out = Command::new("dtc")
        .args(["-I", "dtb", "-O", "dts"])
        .arg(&overlay)
        .output()
        .unwrap();
    assert!(
        String::from_utf8(out.stdout)
            .unwrap()
            .contains("__fixups__")
    );
    // The real lab profile has no ownership/pinctrl/reservation conflict (review #3).
    assert_eq!(
        rp1_build::overlay::final_conflicts(&fs::read(&merged).unwrap(), &a.note).unwrap(),
        Vec::<String>::new()
    );
    assert_eq!(before, fs::read(&base).unwrap());
}

/// Contents of every watched file (directories recursively), as Cargo's
/// `rerun-if-changed` would see them.
fn snapshot(paths: &[std::path::PathBuf]) -> Vec<(std::path::PathBuf, Vec<u8>)> {
    fn walk(p: &Path, out: &mut Vec<(std::path::PathBuf, Vec<u8>)>) {
        if p.is_dir() {
            let mut entries: Vec<_> = fs::read_dir(p)
                .unwrap()
                .map(|e| e.unwrap().path())
                .collect();
            entries.sort();
            for e in entries {
                walk(&e, out);
            }
        } else if let Ok(b) = fs::read(p) {
            out.push((p.to_path_buf(), b));
        }
    }
    let mut out = Vec::new();
    for p in paths {
        walk(p, &mut out);
    }
    out
}

#[test]
fn branch_commits_change_watched_git_paths() {
    // REVIEW2 #6: on a branch a commit rewrites the ref, not HEAD.
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();
    let git = |args: &[&str]| {
        let st = Command::new("git")
            .args([
                "-c",
                "user.name=t",
                "-c",
                "user.email=t@t",
                "-c",
                "commit.gpgsign=false",
            ])
            .args(args)
            .current_dir(dir)
            .output()
            .unwrap();
        assert!(
            st.status.success(),
            "{args:?}: {}",
            String::from_utf8_lossy(&st.stderr)
        );
    };
    git(&["init", "-q", "-b", "feature/x"]);
    let mut n = 0;
    let mut commit = || {
        n += 1;
        fs::write(dir.join("main.rs"), format!("// {n}\n")).unwrap();
        git(&["add", "main.rs"]);
        git(&["commit", "-q", "-m", "c"]);
    };
    commit();
    for pack in [false, true] {
        if pack {
            git(&["pack-refs", "--all"]);
        }
        let paths = rp1_build::git_watch_paths(dir);
        let before = snapshot(&paths);
        commit();
        assert_ne!(before, snapshot(&paths), "pack={pack} paths={paths:?}");
    }
}
