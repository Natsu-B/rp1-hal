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
    assert_eq!(before, fs::read(&base).unwrap());
}
