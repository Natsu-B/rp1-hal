//! STATIC: synthetic merged DTBs compiled by dtc, with a validated note descriptor.
use rp1_dt::{
    check::{self, Gpiomem, OnConflict, Plan, Policy, apply_plan, resolve},
    fdt::Fdt,
};
use rp1_proto::note;
use std::{fs, process::Command};

const RP1: &str = "/pcie@1000120000/rp1";

fn descriptor(entries: &[(u8, u16)]) -> Vec<u8> {
    let mut bytes = vec![0; 32];
    fn record(bytes: &mut Vec<u8>, kind: u16, v: &[u8]) {
        bytes.extend(kind.to_le_bytes());
        bytes.extend(1u16.to_le_bytes());
        bytes.extend((v.len() as u32).to_le_bytes());
        bytes.extend(v);
        while !bytes.len().is_multiple_of(4) {
            bytes.push(0);
        }
    }
    let mut firmware = vec![1, b't', 1, b'1'];
    firmware.extend([0; 52]);
    record(&mut bytes, note::FIRMWARE, &firmware);
    record(&mut bytes, note::ENTRY, &[0; 20]);
    let mut own = (entries.len() as u32).to_le_bytes().to_vec();
    for (kind, id) in entries {
        own.extend([*kind, 1]);
        own.extend(id.to_le_bytes());
    }
    record(&mut bytes, note::OWNERSHIP, &own);
    record(&mut bytes, note::CLOCK, &[1, 0, 0, 0, 0]);
    record(&mut bytes, note::BOOT, &[0; 65]);
    bytes[..8].copy_from_slice(note::MAGIC);
    bytes[8..12].copy_from_slice(&[2, 0, 32, 0]);
    let len = bytes.len() as u32;
    bytes[12..16].copy_from_slice(&len.to_le_bytes());
    bytes[24] = 1;
    let crc = note::crc32(&bytes[32..]);
    bytes[20..24].copy_from_slice(&crc.to_le_bytes());
    note::Note::parse(&bytes).unwrap();
    bytes
}

fn fixture(extra: &str) -> (Fdt, Vec<u8>) {
    let tmp = tempfile::tempdir().unwrap();
    let dts = tmp.path().join("fixture.dts");
    let dtb = tmp.path().join("fixture.dtb");
    fs::write(
        &dts,
        format!("{}\n{extra}", include_str!("fixtures/resolve.dts")),
    )
    .unwrap();
    let result = Command::new("dtc")
        .args(["-@", "-q", "-I", "dts", "-O", "dtb", "-o"])
        .arg(&dtb)
        .arg(dts)
        .output()
        .expect("dtc required for synthetic DTB tests");
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let mut fdt = Fdt::parse(&fs::read(dtb).unwrap()).unwrap();
    let note = descriptor(&[(1, 8), (1, 9), (1, 10), (1, 11), (1, 22), (2, 16)]);
    set(&mut fdt, "/rp1-firmware", "rp1,note", note.clone());
    (fdt, note)
}
fn set(fdt: &mut Fdt, path: &str, prop: &str, value: Vec<u8>) {
    let n = fdt.find(path).unwrap();
    fdt.set_prop(n, prop, value);
}
fn disables(plan: &Plan) -> &[check::Disable] {
    match plan {
        Plan::Disable(nodes) => nodes,
        Plan::Refuse(why) => panic!("unexpected refusal: {why:?}"),
    }
}
fn refusal(plan: Plan, text: &str) {
    let Plan::Refuse(why) = plan else {
        panic!("expected refusal, got {plan:?}");
    };
    assert!(why.iter().any(|why| why.contains(text)), "{why:?}");
}
fn uart3() -> &'static str {
    "&rp1_uart3 { status = \"okay\"; };"
}

#[test]
fn uart3_overlay_conflicts_with_spi0_gpio8_gpio9_and_is_marked() {
    let (mut fdt, note) = fixture(&format!(
        "{} / {{ __symbols__ {{ aaa_fragment = \"{RP1}/serial@3c000\"; }}; }};",
        uart3()
    ));
    let original = fdt.to_bytes();
    let plan = resolve(&fdt, &note, Policy::default()).unwrap();
    assert_eq!(fdt.to_bytes(), original, "resolve is pure");
    let nodes = disables(&plan);
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0].label.as_deref(), Some("rp1_uart3"));
    assert!(nodes[0].reasons.iter().any(|s| s.contains("GPIO8")));
    assert!(nodes[0].reasons.iter().any(|s| s.contains("GPIO9")));
    let before = fdt.clone();
    apply_plan(&before, &mut fdt, &note, Policy::default(), &plan).unwrap();
    let uart = fdt.label("rp1_uart3").unwrap();
    assert_eq!(fdt.prop(uart, "status"), Some(b"disabled\0".as_slice()));
    assert_eq!(fdt.prop(uart, "rp1,loader-disabled"), Some([].as_slice()));
    assert_eq!(
        fdt.strings(fdt.find("/rp1-firmware").unwrap(), "rp1,loader-disabled"),
        [format!("{RP1}/serial@3c000")]
    );
    let round_trip = Fdt::parse(&fdt.to_bytes()).unwrap();
    assert!(
        check::check(&before, &round_trip, &note)
            .unwrap()
            .is_empty()
    );
    assert!(disables(&resolve(&round_trip, &note, Policy::default()).unwrap()).is_empty());
}

#[test]
fn gpio_led_singular_plural_and_mixed_controller_specifiers() {
    for prop in ["gpios", "gpio", "reset-gpios", "enable-gpio"] {
        let (fdt, note) = fixture(&format!(
            "/ {{ leds {{ compatible = \"gpio-leds\"; led {{ {prop} = <0 &other_gpio 22 0 0 &rp1_gpio 22 1>; }}; }}; }};"
        ));
        let plan = resolve(&fdt, &note, Policy::default()).unwrap();
        let nodes = disables(&plan);
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].path, "/leds/led");
        assert!(nodes[0].reasons[0].contains("GPIO22"));
    }
}

#[test]
fn gpio_hog_and_instance_are_disabled() {
    let (fdt, note) = fixture(
        "&rp1_gpio { hog { gpio-hog; gpios = <22 0>; output-high; }; }; &rp1_spi0 { status = \"okay\"; };",
    );
    let plan = resolve(&fdt, &note, Policy::default()).unwrap();
    let nodes = disables(&plan);
    assert_eq!(nodes.len(), 2);
    assert!(
        nodes
            .iter()
            .any(|n| n.reasons.iter().any(|r| r.contains("hogs GPIO22")))
    );
    assert!(nodes.iter().any(|n| {
        n.reasons
            .iter()
            .any(|r| r.contains("RP1-owned block rp1_spi0"))
    }));
}

#[test]
fn console_direct_alias_and_ancestors_are_protected() {
    for stdout in [
        "serial3:115200n8".to_owned(),
        format!("{RP1}/serial@3c000:115200n8"),
    ] {
        let (fdt, note) = fixture(&format!(
            "{} / {{ chosen {{ stdout-path = \"{stdout}\"; }}; }};",
            uart3()
        ));
        refusal(
            resolve(&fdt, &note, Policy::default()).unwrap(),
            "console stdout-path",
        );
    }
    let (fdt, note) = fixture(
        "/ { console-bus { pinctrl-0 = <&uart3_pins>; console {}; }; chosen { stdout-path = \"/console-bus/console\"; }; };",
    );
    refusal(
        resolve(&fdt, &note, Policy::default()).unwrap(),
        "console stdout-path",
    );
}

#[test]
fn all_design_protected_nodes_and_their_ancestors_refuse() {
    for path in [
        "/",
        "/pcie@1000120000",
        RP1,
        &format!("{RP1}/gpio@d0000"),
        &format!("{RP1}/ethernet@100000"),
        &format!("{RP1}/usb@200000"),
        &format!("{RP1}/usb@300000"),
        &format!("{RP1}/dma@188000"),
        &format!("{RP1}/pio@178000"),
        "/reserved-memory",
        "/reserved-memory/scratch@21000000",
        "/rp1-firmware",
        "/rp1-link@1f0040c000",
        "/firmware",
        "/firmware/scmi",
        "/firmware/scmi/protocol@14",
        "/scmi-shmem",
    ] {
        let (mut fdt, note) = fixture("");
        set(&mut fdt, path, "status", b"okay\0".to_vec());
        let handle = fdt.phandle_of(fdt.label("uart3_pins").unwrap()).unwrap();
        set(&mut fdt, path, "pinctrl-0", handle.to_be_bytes().to_vec());
        refusal(
            resolve(&fdt, &note, Policy::default()).unwrap(),
            "protected:",
        );
    }
}

#[test]
fn gpiomem_identified_by_compatible_and_reg_not_path() {
    let (mut fdt, note) = fixture(
        "/ { arbitrary-container { #address-cells = <2>; #size-cells = <2>; mapping { compatible = \"vendor,other\", \"raspberrypi,gpiomem\"; reg = <0xc0 0x400d0000 0 0x30000>; }; soc-mapping { compatible = \"raspberrypi,gpiomem\"; reg = <0 0x7e200000 0 0x1000>; }; }; };",
    );
    let plan = resolve(&fdt, &note, Policy::default()).unwrap();
    assert_eq!(disables(&plan).len(), 1);
    assert_eq!(disables(&plan)[0].path, "/arbitrary-container/mapping");
    let before = fdt.clone();
    apply_plan(&before, &mut fdt, &note, Policy::default(), &plan).unwrap();
    assert!(fdt.enabled(fdt.find("/arbitrary-container/soc-mapping").unwrap()));
    assert!(disables(&resolve(&fdt, &note, Policy::default()).unwrap()).is_empty());
}

#[test]
fn gpiomem_absent_kept_or_no_owned_pins() {
    let (fdt, note) = fixture("");
    assert!(disables(&resolve(&fdt, &note, Policy::default()).unwrap()).is_empty());
    let (mut fdt, note) = fixture(
        "/ { renamed { compatible = \"raspberrypi,gpiomem\"; reg = <0xc0 0x400d0000 0x30000>; }; };",
    );
    assert!(
        disables(
            &resolve(
                &fdt,
                &note,
                Policy {
                    gpiomem: Gpiomem::Keep,
                    ..Policy::default()
                }
            )
            .unwrap()
        )
        .is_empty()
    );
    let empty = descriptor(&[]);
    set(&mut fdt, "/rp1-firmware", "rp1,note", empty.clone());
    assert!(disables(&resolve(&fdt, &empty, Policy::default()).unwrap()).is_empty());
}

#[test]
fn strict_mode_refuses_without_applying_partial_plan() {
    let (mut fdt, note) = fixture(uart3());
    let policy = Policy {
        on_conflict: OnConflict::Refuse,
        ..Policy::default()
    };
    let plan = resolve(&fdt, &note, policy).unwrap();
    refusal(plan.clone(), "on_conflict=refuse");
    let before = fdt.clone();
    assert!(apply_plan(&before, &mut fdt, &note, policy, &plan).is_err());
    assert_eq!(fdt.to_bytes(), before.to_bytes());
    let (fdt, note) = fixture("");
    assert!(disables(&resolve(&fdt, &note, policy).unwrap()).is_empty());
}

#[test]
fn no_conflicts_and_non_rp1_groups_or_gpios_are_untouched() {
    let (mut fdt, note) = fixture(
        "/ { user { pinctrl-0 = <&soc_pins>; gpios = <&other_gpio 8 0 0 &rp1_gpio 23 0>; }; bus { status = \"disabled\"; child { pinctrl-0 = <&uart3_pins>; }; }; };",
    );
    let plan = resolve(&fdt, &note, Policy::default()).unwrap();
    assert!(disables(&plan).is_empty());
    let before = fdt.clone();
    apply_plan(&before, &mut fdt, &note, Policy::default(), &plan).unwrap();
    assert_eq!(fdt.to_bytes(), before.to_bytes());
}

#[test]
fn policy_defaults_explicit_values_and_invalid_values() {
    let (mut fdt, _) = fixture("");
    assert_eq!(Policy::from_fdt(&fdt).unwrap(), Policy::default());
    set(
        &mut fdt,
        "/rp1-firmware",
        "rp1,on-conflict",
        b"refuse\0".to_vec(),
    );
    set(&mut fdt, "/rp1-firmware", "rp1,gpiomem", b"keep\0".to_vec());
    assert_eq!(
        Policy::from_fdt(&fdt).unwrap(),
        Policy {
            on_conflict: OnConflict::Refuse,
            gpiomem: Gpiomem::Keep
        }
    );
    for v in [
        b"unknown\0".to_vec(),
        b"disable".to_vec(),
        vec![],
        b"disable\0refuse\0".to_vec(),
        vec![255, 0],
    ] {
        set(&mut fdt, "/rp1-firmware", "rp1,gpiomem", v);
        assert!(Policy::from_fdt(&fdt).is_err());
    }
}

#[test]
fn malformed_specifiers_and_console_return_errors() {
    for extra in [
        "/ { user { gpios = <&rp1_gpio 22>; }; };",
        "/ { user { gpios = [01]; }; };",
        "/ { user { gpios = <0xfffffffe 22 0>; }; };",
        "/ { user { pinctrl-0 = <0xfffffffe>; }; };",
        "/ { user { pinctrl-0 = [01]; }; };",
        "&rp1_gpio { #gpio-cells = <0xffffffff>; };",
        "&rp1_gpio { gpio-reserved-ranges = <0 0xffffffff>; };",
        "&rp1_gpio { gpio-reserved-ranges = [01]; };",
        "/ { chosen { stdout-path = \"missing-alias\"; }; };",
        "/ { chosen { stdout-path = \"/missing\"; }; };",
        "/ { mapping { compatible = \"raspberrypi,gpiomem\"; reg = [01]; }; };",
    ] {
        let (fdt, note) = fixture(extra);
        assert!(resolve(&fdt, &note, Policy::default()).is_err(), "{extra}");
    }
}

#[test]
fn recheck_rejects_missing_reservations_and_note_mismatch() {
    let (mut fdt, note) = fixture("");
    set(
        &mut fdt,
        &format!("{RP1}/gpio@d0000"),
        "gpio-reserved-ranges",
        Vec::new(),
    );
    refusal(
        resolve(&fdt, &note, Policy::default()).unwrap(),
        "not in gpio-reserved-ranges",
    );
    let (mut fdt, note) = fixture(uart3());
    set(&mut fdt, "/rp1-firmware", "rp1,note", vec![0]);
    refusal(
        resolve(&fdt, &note, Policy::default()).unwrap(),
        "differs from the ELF note",
    );
    assert!(resolve(&fdt, &[0], Policy::default()).is_err());
}

#[test]
fn apply_rejects_stale_plan_and_reservation_shrink_without_mutation() {
    let (mut fdt, note) = fixture(uart3());
    let plan = resolve(&fdt, &note, Policy::default()).unwrap();
    let before = fdt.clone();
    assert!(
        apply_plan(
            &before,
            &mut fdt,
            &note,
            Policy::default(),
            &Plan::Disable(Vec::new())
        )
        .is_err()
    );
    assert_eq!(fdt.to_bytes(), before.to_bytes());
    let mut base = fdt.clone();
    let n = base.label("rp1_gpio").unwrap();
    base.set_prop(
        n,
        "gpio-reserved-ranges",
        [8u32, 4, 22, 2]
            .into_iter()
            .flat_map(u32::to_be_bytes)
            .collect(),
    );
    let err = apply_plan(&base, &mut fdt, &note, Policy::default(), &plan).unwrap_err();
    assert!(err.contains("back to Linux"));
    assert_eq!(fdt.to_bytes(), before.to_bytes());
}

#[test]
fn legacy_pin_groups_and_markers_are_preserved_across_applications() {
    let (mut fdt, note) = fixture("/ { legacy-user { pinctrl-0 = <&legacy_pins>; }; };");
    set(
        &mut fdt,
        "/rp1-firmware",
        "rp1,loader-disabled",
        b"/previous\0".to_vec(),
    );
    let before = fdt.clone();
    let plan = resolve(&fdt, &note, Policy::default()).unwrap();
    apply_plan(&before, &mut fdt, &note, Policy::default(), &plan).unwrap();
    assert_eq!(
        fdt.strings(fdt.find("/rp1-firmware").unwrap(), "rp1,loader-disabled"),
        ["/previous", "/legacy-user"]
    );
    let done = fdt.to_bytes();
    let plan = resolve(&fdt, &note, Policy::default()).unwrap();
    apply_plan(&before, &mut fdt, &note, Policy::default(), &plan).unwrap();
    assert_eq!(fdt.to_bytes(), done);
}

#[test]
fn arbitrary_dtb_bytes_and_valid_tree_property_mutations_never_panic() {
    let (fdt, note) = fixture(uart3());
    let bytes = fdt.to_bytes();
    for len in 0..bytes.len() {
        if let Ok(tree) = Fdt::parse(&bytes[..len]) {
            let _ = resolve(&tree, &note, Policy::default());
        }
    }
    // Includes overflowing structure/string/property offsets on 32-bit targets.
    for at in (0..bytes.len().saturating_sub(4)).step_by(4) {
        let mut damaged = bytes.clone();
        damaged[at..at + 4].copy_from_slice(&u32::MAX.to_be_bytes());
        if let Ok(tree) = Fdt::parse(&damaged) {
            let _ = resolve(&tree, &note, Policy::default());
        }
    }
    for n in 0..fdt.nodes.len() {
        for (name, _) in &fdt.nodes[n].props {
            for value in [vec![], vec![255], u32::MAX.to_be_bytes().to_vec()] {
                let mut tree = fdt.clone();
                tree.set_prop(n, name, value);
                let _ = resolve(&tree, &note, Policy::default());
            }
        }
    }
}

#[test]
fn deeply_nested_pin_groups_use_an_iterative_walk() {
    let (mut fdt, note) = fixture(uart3());
    let mut parent = fdt.label("uart3_pins").unwrap();
    for _ in 0..2048 {
        parent = fdt.child_or_add(parent, "nested");
    }
    fdt.set_prop(parent, "pins", b"gpio22\0".to_vec());
    let tree = Fdt::parse(&fdt.to_bytes()).unwrap();
    let plan = resolve(&tree, &note, Policy::default()).unwrap();
    assert!(
        disables(&plan)[0]
            .reasons
            .iter()
            .any(|s| s.contains("GPIO22"))
    );
}

#[test]
fn dt_symbols_aliases_and_overlay_parameters_are_not_gpio_consumers() {
    let (fdt, note) = fixture(
        "/ { aliases { gpio = &rp1_gpio; }; __overrides__ { gpio = <&rp1_gpio>, \"gpio:0\"; }; __symbols__ { gpio = \"/pcie@1000120000/rp1/gpio@d0000\"; }; };",
    );
    assert!(disables(&resolve(&fdt, &note, Policy::default()).unwrap()).is_empty());
}

#[test]
fn gpiomem_only_disables_are_policy_only_and_conflicts_are_not() {
    let (fdt, note) = fixture(&format!(
        "{} / {{ gpiomem {{ compatible = \"raspberrypi,gpiomem\"; reg = <0xc0 0x400d0000 0x30000>; }}; }};",
        uart3()
    ));
    let plan = resolve(&fdt, &note, Policy::default()).unwrap();
    let nodes = disables(&plan);
    assert_eq!(nodes.len(), 2, "{nodes:?}");
    let gpiomem = nodes.iter().find(|n| n.path == "/gpiomem").unwrap();
    assert!(gpiomem.policy_only);
    let uart = nodes
        .iter()
        .find(|n| n.path.ends_with("/serial@3c000"))
        .unwrap();
    assert!(!uart.policy_only);
    // A gpiomem node that is also a GPIO consumer is a conflict, not only policy.
    let (fdt, note) = fixture(
        "/ { gpiomem { compatible = \"raspberrypi,gpiomem\"; reg = <0xc0 0x400d0000 0x30000>; gpios = <&rp1_gpio 22 0>; }; };",
    );
    let plan = resolve(&fdt, &note, Policy::default()).unwrap();
    assert_eq!(disables(&plan).len(), 1);
    assert!(!disables(&plan)[0].policy_only);
}

#[test]
fn enforce_reads_policy_applies_or_refuses_without_mutation() {
    // Default policy: disable, mark, and leave a clean tree.
    let (mut fdt, note) = fixture(uart3());
    let before = fdt.clone();
    let (policy, plan) = check::enforce(&before, &mut fdt, &note).unwrap();
    assert_eq!(policy, Policy::default());
    assert_eq!(disables(&plan).len(), 1);
    let uart = fdt.label("rp1_uart3").unwrap();
    assert!(!fdt.enabled(uart));
    assert!(fdt.prop(uart, "rp1,loader-disabled").is_some());
    assert!(check::check(&before, &fdt, &note).unwrap().is_empty());
    // Policy from /rp1-firmware: refuse leaves the tree untouched.
    let (mut fdt, note) = fixture(uart3());
    set(
        &mut fdt,
        "/rp1-firmware",
        "rp1,on-conflict",
        b"refuse\0".to_vec(),
    );
    let before = fdt.clone();
    let (policy, plan) = check::enforce(&before, &mut fdt, &note).unwrap();
    assert_eq!(policy.on_conflict, OnConflict::Refuse);
    refusal(plan, "on_conflict=refuse");
    assert_eq!(fdt.to_bytes(), before.to_bytes());
    // A post-disable check failure (reservation shrink) is an error, also without mutation.
    let (mut fdt, note) = fixture(uart3());
    let mut base = fdt.clone();
    let n = base.label("rp1_gpio").unwrap();
    base.set_prop(
        n,
        "gpio-reserved-ranges",
        [8u32, 4, 22, 2]
            .into_iter()
            .flat_map(u32::to_be_bytes)
            .collect(),
    );
    let original = fdt.to_bytes();
    assert!(check::enforce(&base, &mut fdt, &note).is_err());
    assert_eq!(fdt.to_bytes(), original);
    // Unknown policy values fail closed.
    let (mut fdt, note) = fixture("");
    set(
        &mut fdt,
        "/rp1-firmware",
        "rp1,on-conflict",
        b"ignore\0".to_vec(),
    );
    let before = fdt.clone();
    assert!(check::enforce(&before, &mut fdt, &note).is_err());
}
