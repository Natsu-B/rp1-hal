//! Review #3: `cargo rp1 check --dtb` must reject ownership conflicts in the final DTB.
use rp1_build::{generate, metadata::Declaration, overlay};
use std::{fs, path::Path, process::Command};

const BASE: &str = r#"/dts-v1/;
/ {
    #address-cells = <2>; #size-cells = <1>;
    reserved-memory { #address-cells = <2>; #size-cells = <1>; ranges;
        scratch@21000000 { reg = <0 0x21000000 0x10000>; no-map; };
        @EXTRA_RESERVED@
    };
    openamp-address-space { status = "okay"; };
    rp1_firmware: rp1_firmware { mboxes = <&rp1_mbox 0>; };
    pcie2: pcie { };
    soc-gpio { soc22: grp22 { pins = "gpio22"; }; };
    soc-user { pinctrl-0 = <&soc22>; };
    rp1 { #address-cells = <1>; #size-cells = <1>;
        rp1_clocks: clocks { status = "okay"; };
        rp1_mbox: mailbox@8000 { #mbox-cells = <1>; };
        sram: sram@400000 { #address-cells = <1>; #size-cells = <1>;
            rp1_fw_shmem: shmem@ff00 { reg = <0xff00 0x100>; };
        };
        rp1_uart1: serial@34000 { status = "disabled"; };
        rp1_uart2: serial@38000 { status = "disabled"; };
        rp1_uart3: serial@3c000 { status = "disabled"; };
        rp1_uart4: serial@40000 { status = "disabled"; };
        rp1_uart5: serial@44000 { status = "disabled"; };
        rp1_adc: adc { }; rp1_eth: ethernet { }; rp1_dma: dma { }; rp1_pio: pio { };
        rp1_usb0: usb0 { }; rp1_usb1: usb1 { };
        rp1_gpio: gpio@d0000 { gpio-reserved-ranges = <2 52>; #gpio-cells = <2>;
            micclk1_hog { gpio-hog; gpios = <46 0>; output-high; };
            uart0_14_15: rp1_uart0_14_15 {
                tx { function = "uart0"; pins = "gpio14"; };
                rx { function = "uart0"; pins = "gpio15"; };
            };
            gpio22: rp1_gpio22 { function = "gpio"; pins = "gpio22"; };
            gpio4: rp1_gpio4 { function = "gpio"; pins = "gpio4"; };
        };
        rp1_uart0: serial@30000 { pinctrl-0 = <&uart0_14_15>; status = "okay"; };
        user4 { pinctrl-0 = <&gpio4>; };
        @EXTRA_RP1@
    };
};
"#;

fn declaration() -> Declaration {
    toml::from_str(
        r#"
schema=1
board="rpi-cm5"
log={sink="uart0",baudrate=115200}
[proc0]
pins=[22]
[proc1]
pins=[27]
"#,
    )
    .unwrap()
}

fn dtc(dir: &Path, name: &str, source: &str) -> std::path::PathBuf {
    let dts = dir.join(format!("{name}.dts"));
    let out = dir.join(format!("{name}.dtb"));
    fs::write(&dts, source).unwrap();
    let o = Command::new("dtc")
        .args(["-@", "-q", "-I", "dts", "-O", "dtb", "-o"])
        .arg(&out)
        .arg(&dts)
        .output()
        .unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    out
}

/// Conflicts after applying the generated overlay (and then `later` overlays) to the fixture.
fn conflicts(extra_reserved: &str, extra_rp1: &str, later: &[&str]) -> Vec<String> {
    let tmp = tempfile::tempdir().unwrap();
    let base = BASE
        .replace("@EXTRA_RESERVED@", extra_reserved)
        .replace("@EXTRA_RP1@", extra_rp1);
    let base = dtc(tmp.path(), "base", &base);
    let a = generate("test", "0.1.0", &declaration(), [0; 32], [0; 20]).unwrap();
    let app = dtc(tmp.path(), "app", &a.dts);
    let mut current = tmp.path().join("merged0.dtb");
    overlay::apply(&base, &app, &current).unwrap();
    for (i, src) in later.iter().enumerate() {
        let o = dtc(
            tmp.path(),
            &format!("later{i}"),
            &format!("/dts-v1/;\n/plugin/;\n{src}\n"),
        );
        let next = tmp.path().join(format!("merged{}.dtb", i + 1));
        overlay::apply(&current, &o, &next).unwrap();
        current = next;
    }
    overlay::final_conflicts(&fs::read(&current).unwrap(), &a.note).unwrap()
}

#[test]
fn clean_lab_profile_passes() {
    assert_eq!(conflicts("", "", &[]), Vec::<String>::new());
    // A disabled consumer of an owned pin is not a conflict.
    assert_eq!(
        conflicts(
            "",
            r#"led { pinctrl-0 = <&gpio22>; status = "disabled"; };"#,
            &[]
        ),
        Vec::<String>::new()
    );
}

#[test]
fn gpio_hog_on_reserved_line_is_rejected() {
    // The generated overlay disables the stock hog; a later overlay re-enabling it is caught.
    let c = conflicts(
        "",
        "",
        &[r#"&rp1_gpio { micclk1_hog { status = "okay"; }; };"#],
    );
    assert!(c.iter().any(|c| c.contains("GPIO46")), "{c:?}");
}

#[test]
fn enabled_pinctrl_on_owned_pin_is_rejected() {
    // The review's counterexample: GPIO22 is RP1's, an enabled Linux node selects its group,
    // and gpio-reserved-ranges is unchanged.
    let c = conflicts("", r#"led { pinctrl-0 = <&gpio22>; };"#, &[]);
    assert!(c.iter().any(|c| c.contains("GPIO22")), "{c:?}");
    // pinctrl-1 (another state) counts too; the SoC bank's "gpio22" does not.
    let c = conflicts("", r#"led { pinctrl-1 = <&gpio22>; };"#, &[]);
    assert_eq!(c.len(), 1, "{c:?}");
}

#[test]
fn owned_block_or_clocks_re_enabled_by_a_later_overlay_are_rejected() {
    let c = conflicts("", "", &[r#"&rp1_uart0 { status = "okay"; };"#]);
    assert!(c.iter().any(|c| c.contains("rp1_uart0")), "{c:?}");
    let c = conflicts("", "", &[r#"&rp1_clocks { status = "okay"; };"#]);
    assert!(c.iter().any(|c| c.contains("rp1_clocks")), "{c:?}");
    let c = conflicts(
        "",
        "",
        &[
            r#"/ { fragment@0 { target-path = "/rp1-firmware"; __overlay__ { rp1,note = [00]; }; }; };"#,
        ],
    );
    assert!(c.iter().any(|c| c.contains("rp1,note")), "{c:?}");
}

#[test]
fn i2c_overlay_and_later_linux_conflict() {
    let tmp = tempfile::tempdir().unwrap();
    let mut d = declaration();
    d.proc0.pins.extend([2, 3]);
    d.proc0.peripherals.push("I2C1".into());
    let base = dtc(
        tmp.path(),
        "base",
        &BASE.replace("@EXTRA_RESERVED@", "").replace(
            "@EXTRA_RP1@",
            r#"rp1_i2c1: i2c@74000 { status = "okay"; };"#,
        ),
    );
    let a = generate("i2c", "0.1.0", &d, [0; 32], [0; 20]).unwrap();
    let app = dtc(tmp.path(), "app", &a.dts);
    let merged = tmp.path().join("merged.dtb");
    overlay::apply(&base, &app, &merged).unwrap();
    let mut fdt = rp1_build::fdt::Fdt::parse(&fs::read(&merged).unwrap()).unwrap();
    assert!(rp1_dt_check(&fdt, &a.note).is_empty());
    assert!(
        rp1_build::overlay::reserved_gpios(&merged)
            .unwrap()
            .is_superset(&[2, 3].into())
    );
    let i2c = fdt.label("rp1_i2c1").unwrap();
    fdt.set_prop(i2c, "status", b"okay\0".to_vec());
    let bad = rp1_dt_check(&fdt, &a.note);
    assert!(bad.iter().any(|c| c.contains("rp1_i2c1")), "{bad:?}");
}

fn rp1_dt_check(fdt: &rp1_build::fdt::Fdt, note: &[u8]) -> Vec<String> {
    overlay::final_conflicts(&fdt.to_bytes(), note).unwrap()
}

#[test]
fn reservation_overlap_is_rejected() {
    let c = conflicts(
        "vring@220f0000 { reg = <0 0x220f0000 0x4000>; no-map; };",
        "",
        &[],
    );
    assert!(c.iter().any(|c| c.contains("overlaps")), "{c:?}");
    // A disabled reservation is ignored by Linux.
    assert!(
        conflicts(
            r#"vring@220f0000 { reg = <0 0x220f0000 0x4000>; status = "disabled"; };"#,
            "",
            &[]
        )
        .is_empty()
    );
    // Shrinking the reservation under the note is rejected.
    let c = conflicts(
        "",
        "",
        &[
            r#"/ { fragment@0 { target-path = "/reserved-memory/rp1-shm@22000000"; __overlay__ { reg = <0 0x22000000 0x10000>; }; }; };"#,
        ],
    );
    assert!(c.iter().any(|c| c.contains("rp1-shm")), "{c:?}");
}

#[test]
fn present_openamp_and_hogs_are_disabled() {
    let tmp = tempfile::tempdir().unwrap();
    let base = BASE
        .replace("@EXTRA_RESERVED@", "")
        .replace("@EXTRA_RP1@", "");
    let base = dtc(tmp.path(), "base", &base);
    let a = generate("test", "0.1.0", &declaration(), [0; 32], [0; 20]).unwrap();
    let app = dtc(tmp.path(), "app", &a.dts);
    let merged = tmp.path().join("merged.dtb");
    overlay::apply(&base, &app, &merged).unwrap();
    let fdt = rp1_build::fdt::Fdt::parse(&fs::read(&merged).unwrap()).unwrap();
    assert!(!fdt.enabled(fdt.find("/openamp-address-space").unwrap()));
    let hog = fdt
        .child(fdt.label("rp1_gpio").unwrap(), "micclk1_hog")
        .unwrap();
    assert!(!fdt.enabled(hog));
    assert!(
        fdt.prop(hog, "gpio-hog").is_some(),
        "existing hog is kept, only disabled"
    );
}
