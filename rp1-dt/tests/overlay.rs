//! `rp1_dt::overlay::apply` must produce the same tree as libfdt's `fdtoverlay`.
use rp1_dt::fdt::Fdt;
use std::{fs, path::Path, process::Command};

const BASE: &str = r#"/dts-v1/;
/memreserve/ 0x3fc00000 0x400000;
/ {
    #address-cells = <2>; #size-cells = <1>;
    model = "fixture";
    aliases { serial9 = "/soc/serial@9"; };
    chosen { bootargs = "console=ttyAMA10"; linux,initrd-start = <0x2e000000>; };
    reserved-memory { #address-cells = <2>; #size-cells = <1>; ranges;
        scratch@21000000 { reg = <0 0x21000000 0x10000>; no-map; };
    };
    soc { #address-cells = <1>; #size-cells = <1>;
        gpio: gpio@d0000 { gpio-controller; #gpio-cells = <2>; gpio-reserved-ranges = <2 4>;
            g4: grp4 { pins = "gpio4"; };
        };
        uart0: serial@30000 { pinctrl-0 = <&g4>; status = "okay"; clocks = <&clk 3>; };
        serial@9 { status = "okay"; };
        clk: clocks { #clock-cells = <1>; status = "okay"; };
        scmi: scmi { status = "disabled"; scmi_clk: protocol@14 { #clock-cells = <1>; }; };
    };
};
"#;

const OVERLAY: &str = r#"/dts-v1/;
/plugin/;
/ {
    fragment@0 { target = <&gpio>; __overlay__ { gpio-reserved-ranges = <2 8>; }; };
    fragment@1 { target = <&uart0>; __overlay__ { status = "disabled"; clocks = <&scmi_clk 1>; }; };
    fragment@2 { target-path = "/"; __overlay__ {
        newdev: rp1-link@1f0040c000 { compatible = "generic-uio"; reg = <0x1f 0x40c000 0x4000>; };
        user { dev = <&newdev>; clk = <&clk 7 &newdev>; };
    }; };
    fragment@3 { target-path = "/reserved-memory"; __overlay__ {
        rp1-shm@22000000 { reg = <0 0x22000000 0x200000>; no-map; };
    }; };
    fragment@4 { target-path = "serial9"; __overlay__ { status = "disabled"; }; };
    fragment@5 { target = <&scmi>; __overlay__ { status = "okay"; sub { x = <1>; }; }; };
};
"#;

fn dtc(dir: &Path, name: &str, src: &str) -> Vec<u8> {
    let dts = dir.join(format!("{name}.dts"));
    let out = dir.join(format!("{name}.dtb"));
    fs::write(&dts, src).unwrap();
    let o = Command::new("dtc")
        .args(["-@", "-q", "-I", "dts", "-O", "dtb", "-o"])
        .arg(&out)
        .arg(&dts)
        .output()
        .expect("dtc");
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    fs::read(out).unwrap()
}

fn decompile(dir: &Path, name: &str, dtb: &[u8]) -> String {
    let p = dir.join(name);
    fs::write(&p, dtb).unwrap();
    let o = Command::new("dtc")
        .args(["-q", "-s", "-I", "dtb", "-O", "dts"])
        .arg(&p)
        .output()
        .unwrap();
    assert!(o.status.success());
    String::from_utf8(o.stdout).unwrap()
}

fn fdtoverlay(dir: &Path, base: &[u8], ov: &[u8]) -> Vec<u8> {
    let (b, o, m) = (dir.join("b.dtb"), dir.join("o.dtbo"), dir.join("m.dtb"));
    fs::write(&b, base).unwrap();
    fs::write(&o, ov).unwrap();
    let r = Command::new("fdtoverlay")
        .arg("-i")
        .arg(&b)
        .arg("-o")
        .arg(&m)
        .arg(&o)
        .output()
        .expect("fdtoverlay");
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    fs::read(m).unwrap()
}

fn same_as_libfdt(base: &[u8], ov: &[u8]) {
    let tmp = tempfile::tempdir().unwrap();
    let want = decompile(tmp.path(), "want.dtb", &fdtoverlay(tmp.path(), base, ov));
    let mut fdt = Fdt::parse(base).unwrap();
    rp1_dt::overlay::apply(&mut fdt, ov).unwrap();
    let got = decompile(tmp.path(), "got.dtb", &fdt.to_bytes());
    assert_eq!(got, want);
}

#[test]
fn fixture_matches_fdtoverlay() {
    let tmp = tempfile::tempdir().unwrap();
    same_as_libfdt(
        &dtc(tmp.path(), "base", BASE),
        &dtc(tmp.path(), "ov", OVERLAY),
    );
}

#[test]
fn existing_phandles_keep_old_and_new_consumers() {
    let tmp = tempfile::tempdir().unwrap();
    let base = dtc(
        tmp.path(),
        "base",
        r#"/dts-v1/; / {
        old: device { phandle = <1>; linux,phandle = <1>; x = <1>; };
        user { link = <&old>; };
    };"#,
    );
    let ov = dtc(
        tmp.path(),
        "ov",
        r#"/dts-v1/; /plugin/; / {
        fragment@0 { target-path = "/"; __overlay__ {
            local: device { phandle = <1>; linux,phandle = <1>; x = <2>; };
            use { link = <&local>; };
        }; };
    };"#,
    );
    for omit in [None, Some("phandle"), Some("linux,phandle")] {
        let mut b = Fdt::parse(&base).unwrap();
        let mut o = Fdt::parse(&ov).unwrap();
        if let Some(prop) = omit {
            let device = b.find("/device").unwrap();
            b.remove_prop(device, prop);
            let device = o.find("/fragment@0/__overlay__/device").unwrap();
            o.remove_prop(device, prop);
        }
        same_as_libfdt(&b.to_bytes(), &o.to_bytes());
    }
}

#[test]
fn later_fragments_can_target_added_nodes() {
    let tmp = tempfile::tempdir().unwrap();
    let base = dtc(tmp.path(), "base", "/dts-v1/; / {};");
    for target in ["target = <&local>", "target-path = \"/added\""] {
        let ov = dtc(
            tmp.path(),
            "ov",
            &format!(
                r#"/dts-v1/; /plugin/; / {{
            fragment@0 {{ target-path = "/"; __overlay__ {{ local: added {{ x = <1>; }}; }}; }};
            fragment@1 {{ {target}; __overlay__ {{ x = <2>; }}; }};
        }};"#
            ),
        );
        same_as_libfdt(&base, &ov);
    }
}

#[test]
fn earlier_fragments_see_preserved_phandles_of_later_ones() {
    // REVIEW4 #2: fragment@0 merges a reference to a node that fragment@1 redefines.
    // The redefined node keeps the base phandle; the earlier reference must follow it.
    let tmp = tempfile::tempdir().unwrap();
    let base = dtc(
        tmp.path(),
        "base",
        "/dts-v1/; / { old: device { phandle = <1>; }; };",
    );
    let ov = dtc(
        tmp.path(),
        "ov",
        r#"/dts-v1/; /plugin/; / {
        fragment@0 { target-path = "/"; __overlay__ { use { link = <&local>; }; }; };
        fragment@1 { target-path = "/"; __overlay__ { local: device { x = <2>; }; }; };
        fragment@2 { target = <&local>; __overlay__ { y = <3>; }; };
    };"#,
    );
    same_as_libfdt(&base, &ov);
    let mut fdt = Fdt::parse(&base).unwrap();
    rp1_dt::overlay::apply(&mut fdt, &ov).unwrap();
    let link = fdt.number(fdt.find("/use").unwrap(), "link").unwrap();
    let device = fdt.find("/device").unwrap();
    assert_eq!(link, 1);
    assert_eq!(fdt.phandle(1), Some(device));
    assert_eq!(fdt.number(device, "y"), Some(3));
}

#[test]
fn earlier_fragment_targets_a_node_a_later_fragment_redefines() {
    // REVIEW5 #6 (i): fragment@0's `target = <&local>` names a node that only fragment@1
    // defines, redefining the base /device. The preserve pass rewrites the target to the
    // base phandle, so fragment@0 lands on the base node.
    let tmp = tempfile::tempdir().unwrap();
    let base = dtc(
        tmp.path(),
        "base",
        "/dts-v1/; / { old: device { phandle = <1>; x = <1>; }; user { link = <&old>; }; };",
    );
    let ov = dtc(
        tmp.path(),
        "ov",
        r#"/dts-v1/; /plugin/; / {
        fragment@0 { target = <&local>; __overlay__ { y = <3>; }; };
        fragment@1 { target-path = "/"; __overlay__ { local: device { x = <2>; }; }; };
    };"#,
    );
    same_as_libfdt(&base, &ov);
    let mut fdt = Fdt::parse(&base).unwrap();
    rp1_dt::overlay::apply(&mut fdt, &ov).unwrap();
    let device = fdt.find("/device").unwrap();
    assert_eq!(fdt.phandle(1), Some(device));
    assert_eq!(fdt.number(device, "x"), Some(2));
    assert_eq!(fdt.number(device, "y"), Some(3));
    assert_eq!(fdt.number(fdt.find("/user").unwrap(), "link"), Some(1));
}

#[test]
fn nested_child_keeps_its_base_phandle() {
    // REVIEW5 #6 (ii): the overlay redefines a child below the target; the base phandle of
    // /parent/child is kept and both the old and the new consumer point at it.
    let tmp = tempfile::tempdir().unwrap();
    let base = dtc(
        tmp.path(),
        "base",
        r#"/dts-v1/; / {
        parent { old: child { phandle = <5>; x = <1>; }; };
        user { link = <&old>; };
    };"#,
    );
    let ov = dtc(
        tmp.path(),
        "ov",
        r#"/dts-v1/; /plugin/; / {
        fragment@0 { target-path = "/"; __overlay__ {
            parent { local: child { x = <2>; }; };
            use { link = <&local>; };
        }; };
    };"#,
    );
    same_as_libfdt(&base, &ov);
    let mut fdt = Fdt::parse(&base).unwrap();
    rp1_dt::overlay::apply(&mut fdt, &ov).unwrap();
    let child = fdt.find("/parent/child").unwrap();
    assert_eq!(fdt.phandle(5), Some(child));
    assert_eq!(fdt.number(child, "x"), Some(2));
    for user in ["/user", "/use"] {
        assert_eq!(fdt.number(fdt.find(user).unwrap(), "link"), Some(5));
    }
}

fn fdtoverlay_fails(dir: &Path, base: &[u8], ov: &[u8]) -> bool {
    let (b, o, m) = (dir.join("b.dtb"), dir.join("o.dtbo"), dir.join("m.dtb"));
    fs::write(&b, base).unwrap();
    fs::write(&o, ov).unwrap();
    !Command::new("fdtoverlay")
        .arg("-i")
        .arg(&b)
        .arg("-o")
        .arg(&m)
        .arg(&o)
        .output()
        .expect("fdtoverlay")
        .status
        .success()
}

#[test]
fn reserved_phandle_results_are_rejected_like_libfdt() {
    // REVIEW2 #7: base max 0xfffffffe + overlay phandle 1 would give 0xffffffff.
    let tmp = tempfile::tempdir().unwrap();
    let base = dtc(
        tmp.path(),
        "base",
        "/dts-v1/; / { top: dev { phandle = <0xfffffffe>; }; };",
    );
    let ov = dtc(
        tmp.path(),
        "ov",
        r#"/dts-v1/; /plugin/; / {
        fragment@0 { target-path = "/"; __overlay__ {
            local: added { x = <1>; };
            use { link = <&local>; };
        }; };
    };"#,
    );
    assert!(fdtoverlay_fails(tmp.path(), &base, &ov));
    let mut fdt = Fdt::parse(&base).unwrap();
    let before = fdt.to_bytes();
    assert!(rp1_dt::overlay::apply(&mut fdt, &ov).is_err());
    assert_eq!(
        fdt.to_bytes(),
        before,
        "a failed apply leaves the base untouched"
    );

    // Reserved values in the overlay's own definitions are rejected too.
    let base = dtc(tmp.path(), "base", "/dts-v1/; / {};");
    for bad in [0u32, u32::MAX] {
        let mut o = Fdt::parse(&ov).unwrap();
        let added = o.find("/fragment@0/__overlay__/added").unwrap();
        o.set_prop(added, "phandle", bad.to_be_bytes().to_vec());
        let mut fdt = Fdt::parse(&base).unwrap();
        assert!(
            rp1_dt::overlay::apply(&mut fdt, &o.to_bytes()).is_err(),
            "{bad:#x}"
        );
    }
}

#[test]
fn round_trip_is_lossless() {
    let tmp = tempfile::tempdir().unwrap();
    let base = dtc(tmp.path(), "base", BASE);
    let fdt = Fdt::parse(&base).unwrap();
    assert_eq!(fdt.memreserve, [(0x3fc0_0000, 0x40_0000)]);
    assert_eq!(
        decompile(tmp.path(), "a", &base),
        decompile(tmp.path(), "b", &fdt.to_bytes())
    );
}

#[test]
fn missing_label_fails_without_partial_application() {
    let tmp = tempfile::tempdir().unwrap();
    let base = dtc(tmp.path(), "base", BASE);
    let ov = dtc(
        tmp.path(),
        "bad",
        "/dts-v1/;\n/plugin/;\n/ { fragment@0 { target = <&nosuch>; __overlay__ { x = <1>; }; }; };\n",
    );
    let mut fdt = Fdt::parse(&base).unwrap();
    let err = rp1_dt::overlay::apply(&mut fdt, &ov).unwrap_err();
    assert!(err.contains("nosuch"), "{err}");
    let missing_path = dtc(
        tmp.path(),
        "bad2",
        "/dts-v1/;\n/plugin/;\n/ { fragment@0 { target = <&uart0>; __overlay__ { y = <1>; }; };\n fragment@1 { target-path = \"/nope\"; __overlay__ { x = <1>; }; }; };\n",
    );
    assert!(rp1_dt::overlay::apply(&mut fdt, &missing_path).is_err());
    let uart = fdt.label("uart0").unwrap();
    assert!(fdt.prop(uart, "y").is_none(), "no fragment applied");
}

/// The lab's real base DTB and firmware overlay, when present on this host (not committed:
/// the base carries board-specific data).
#[test]
fn lab_dtb_matches_fdtoverlay() {
    let (Ok(base), Ok(ov)) = (
        std::env::var("RP1_DT_TEST_BASE"),
        std::env::var("RP1_DT_TEST_OVERLAY"),
    ) else {
        eprintln!("skipped: set RP1_DT_TEST_BASE and RP1_DT_TEST_OVERLAY");
        return;
    };
    same_as_libfdt(&fs::read(base).unwrap(), &fs::read(ov).unwrap());
}
