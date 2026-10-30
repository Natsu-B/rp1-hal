//! Review #3: `cargo rp1 check --dtb` must reject ownership conflicts in the final DTB.
use rp1_build::{
    generate,
    metadata::{Declaration, I2cPins, Instance, SpiPins},
    overlay,
};
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
    leds { led_pwr: led-pwr { gpios = <&rp1_gpio 44 1>; }; };
    cam0_reg: cam0_reg { gpio = <&rp1_gpio 34 0>; };
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
        rp1_pwm0: pwm0 { }; rp1_pwm1: pwm1 { }; rp1_audio_out: audio-out { };
        rp1_i2s0: i2s0 { }; rp1_i2s1: i2s1 { }; rp1_i2s2: i2s2 { };
        rp1_i2c0: i2c@70000 { }; rp1_i2c1: i2c@74000 { status = "disabled"; };
        rp1_i2c2: i2c@78000 { }; rp1_i2c3: i2c@7c000 { }; rp1_i2c4: i2c@80000 { };
        rp1_i2c5: i2c@84000 { }; rp1_i2c6: i2c@88000 { };
        rp1_spi0: spi0 { }; rp1_spi1: spi1 { }; rp1_spi2: spi2 { }; rp1_spi3: spi3 { };
        rp1_spi4: spi4 { }; rp1_spi5: spi5 { }; rp1_spi6: spi6 { }; rp1_spi7: spi7 { };
        rp1_spi8: spi8 { };
        rp1_gpio: gpio@d0000 { #gpio-cells = <2>;
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

fn apply_declaration(d: &Declaration, source: &str) -> (rp1_build::fdt::Fdt, Vec<u8>) {
    let tmp = tempfile::tempdir().unwrap();
    let base = dtc(
        tmp.path(),
        "base",
        &source
            .replace("@EXTRA_RESERVED@", "")
            .replace("@EXTRA_RP1@", ""),
    );
    let a = generate("test", "0.1.0", d, [0; 32], [0; 20]).unwrap();
    let app = dtc(tmp.path(), "app", &a.dts);
    let merged = tmp.path().join("merged.dtb");
    overlay::apply(&base, &app, &merged).unwrap();
    (
        rp1_build::fdt::Fdt::parse(&fs::read(merged).unwrap()).unwrap(),
        a.note,
    )
}

#[test]
fn schema2_overlay_reserves_exact_ranges_and_disables_only_owned_instances() {
    let d: Declaration = toml::from_str(
        r#"
schema = 2
log = { sink = "uart0", baudrate = 115200 }
[proc0]
gpio = [22]
i2c1 = { sda = 2, scl = 3 }
[proc1]
gpio = [8, 27]
spi0 = { sck = 11, mosi = 10, miso = 9 }
adc = {}
"#,
    )
    .unwrap();
    let (fdt, note) = apply_declaration(&d, BASE);
    let gpio = fdt.label("rp1_gpio").unwrap();
    assert_eq!(
        fdt.cells(gpio, "gpio-reserved-ranges").unwrap(),
        [2, 2, 8, 4, 14, 2, 22, 1, 27, 1]
    );
    assert_eq!(
        rp1_dt::check::reserved_gpios(&fdt).unwrap(),
        [2, 3, 8, 9, 10, 11, 14, 15, 22, 27].into()
    );
    for label in ["rp1_uart0", "rp1_i2c1", "rp1_spi0", "rp1_adc"] {
        assert!(!fdt.enabled(fdt.label(label).unwrap()), "{label}");
    }
    for label in [
        "rp1_i2c0", "rp1_i2c2", "rp1_i2c3", "rp1_spi1", "rp1_spi3", "rp1_spi5", "rp1_eth",
        "rp1_dma",
    ] {
        assert!(fdt.enabled(fdt.label(label).unwrap()), "{label}");
    }
    assert!(rp1_dt_check(&fdt, &note).is_empty());
}

#[test]
fn linux_uart1_keep_preserves_configured_status_and_scmi_clocks() {
    for enabled in [false, true] {
        let base = BASE.replace(
            "rp1_uart1: serial@34000 { status = \"disabled\"; };",
            &format!(
                "rp1_uart1: serial@34000 {{ status = \"{}\"; }};",
                if enabled { "okay" } else { "disabled" }
            ),
        );
        let d = declaration();
        let (fdt, note) = apply_declaration(&d, &base);
        let uart1 = fdt.label("rp1_uart1").unwrap();
        assert_eq!(fdt.enabled(uart1), enabled);
        assert!(
            rp1_dt::check::reserved_gpios(&fdt)
                .unwrap()
                .is_disjoint(&[0, 1].into())
        );
        let clocks = fdt
            .phandle_of(fdt.label("rp1_scmi_clocks").unwrap())
            .unwrap();
        assert_eq!(fdt.cells(uart1, "clocks").unwrap(), [clocks, 1, clocks, 0]);
        assert_ne!(d.linux_resets()[1] & (1 << 27), 0);
        assert!(rp1_dt_check(&fdt, &note).is_empty());
    }
}

#[test]
fn releasing_uart1_reserves_its_pins_and_disables_its_linux_node() {
    let d: Declaration =
        toml::from_str("schema=2\n[linux]\nkeep={}\n[proc1]\nuart1={tx=0,rx=1}").unwrap();
    let base = BASE.replace(
        "rp1_uart1: serial@34000 { status = \"disabled\"; };",
        "rp1_uart1: serial@34000 { status = \"okay\"; };",
    );
    let (fdt, note) = apply_declaration(&d, &base);
    assert_eq!(rp1_dt::check::reserved_gpios(&fdt).unwrap(), [0, 1].into());
    assert_eq!(
        fdt.cells(fdt.label("rp1_gpio").unwrap(), "gpio-reserved-ranges")
            .unwrap(),
        [0, 2]
    );
    assert!(!fdt.enabled(fdt.label("rp1_uart1").unwrap()));
    assert_eq!(d.linux_resets()[1] & (1 << 27), 0);
    assert!(rp1_dt_check(&fdt, &note).is_empty());
}

#[test]
fn empty_ownership_replaces_base_reservations_but_check_still_rejects_shrinking_them() {
    let base = BASE.replace(
        "#gpio-cells = <2>;",
        "#gpio-cells = <2>; gpio-reserved-ranges = <5 2>;",
    );
    let d = Declaration::default();
    let (after, note) = apply_declaration(&d, &base);
    assert!(rp1_dt::check::reserved_gpios(&after).unwrap().is_empty());
    assert_eq!(
        after
            .cells(after.label("rp1_gpio").unwrap(), "gpio-reserved-ranges")
            .unwrap(),
        []
    );
    let tmp = tempfile::tempdir().unwrap();
    let before = dtc(
        tmp.path(),
        "before",
        &base
            .replace("@EXTRA_RESERVED@", "")
            .replace("@EXTRA_RP1@", ""),
    );
    let before = rp1_build::fdt::Fdt::parse(&fs::read(before).unwrap()).unwrap();
    let conflicts = rp1_dt::check::check(&before, &after, &note).unwrap();
    assert_eq!(conflicts.len(), 1, "{conflicts:?}");
    assert!(conflicts[0].contains("[5, 6] back to Linux"));
}

#[test]
fn policy_properties_survive_overlay_application_without_targeting_gpiomem() {
    for on_conflict in ["disable", "refuse"] {
        for gpiomem in ["disable", "keep"] {
            let d: Declaration = toml::from_str(&format!("schema=2\n[linux]\non_conflict='{on_conflict}'\ngpiomem='{gpiomem}'\n[proc0]\ngpio=[22]")).unwrap();
            // A DTB without gpiomem must accept the overlay too.
            for extra in [
                "",
                "gpiomem@d0000 { compatible = \"raspberrypi,gpiomem\"; reg = <0xd0000 0x30000>; };",
            ] {
                let (fdt, note) = apply_declaration(&d, &BASE.replace("@EXTRA_RP1@", extra));
                let firmware = fdt.find("/rp1-firmware").unwrap();
                assert_eq!(fdt.strings(firmware, "rp1,on-conflict"), [on_conflict]);
                assert_eq!(fdt.strings(firmware, "rp1,gpiomem"), [gpiomem]);
                assert_eq!(fdt.prop(firmware, "rp1,note").unwrap(), note);
                assert_eq!(fdt.cells(firmware, "rp1,abi").unwrap(), [2]);
                if let Some(node) = fdt.find("/rp1/gpiomem@d0000") {
                    assert!(fdt.enabled(node)); // T4/T6 apply the policy.
                }
            }
        }
    }
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
    // GPIO 28..53 are Linux's (stock hog on 46, PWR LED on 44): no conflict.
    assert!(conflicts("", "", &[]).is_empty());
    // A hog or consumer on an RP1-owned pin is caught.
    let c = conflicts(
        "",
        "",
        &[r#"&rp1_gpio { bad_hog { gpio-hog; gpios = <22 0>; output-high; }; };"#],
    );
    assert!(c.iter().any(|c| c.contains("GPIO22")), "{c:?}");
    let c = conflicts("", "", &[r#"&led_pwr { gpios = <&rp1_gpio 27 1>; };"#]);
    assert!(c.iter().any(|c| c.contains("GPIO27")), "{c:?}");
    // GPIO0/1 and undeclared header GPIO5/6 stay Linux's.
    let c = conflicts(
        "",
        r#"user01 { gpios = <&rp1_gpio 0 0 &rp1_gpio 1 0 &rp1_gpio 5 0 &rp1_gpio 6 0>; };"#,
        &[],
    );
    assert!(c.is_empty(), "{c:?}");
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
    d.proc0.instances.insert(
        "i2c1".into(),
        Instance::I2c(I2cPins {
            sda: Some(2),
            scl: Some(3),
        }),
    );
    let base = dtc(
        tmp.path(),
        "base",
        &BASE
            .replace("@EXTRA_RESERVED@", "")
            .replace("@EXTRA_RP1@", "")
            .replace(
                r#"rp1_i2c1: i2c@74000 { status = "disabled"; };"#,
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

#[test]
fn adc_overlay_and_later_linux_conflict() {
    let tmp = tempfile::tempdir().unwrap();
    let mut d = declaration();
    d.proc0.instances.insert("adc".into(), Instance::Adc {});
    let base = dtc(
        tmp.path(),
        "base",
        &BASE
            .replace("@EXTRA_RESERVED@", "")
            .replace("@EXTRA_RP1@", "")
            .replace("rp1_adc: adc { };", r#"rp1_adc: adc { status = "okay"; };"#),
    );
    let a = generate("adc", "0.1.0", &d, [0; 32], [0; 20]).unwrap();
    let app = dtc(tmp.path(), "app", &a.dts);
    let merged = tmp.path().join("merged.dtb");
    overlay::apply(&base, &app, &merged).unwrap();
    let mut fdt = rp1_build::fdt::Fdt::parse(&fs::read(&merged).unwrap()).unwrap();
    assert!(rp1_dt_check(&fdt, &a.note).is_empty());
    let adc = fdt.label("rp1_adc").unwrap();
    assert!(!fdt.enabled(adc));
    fdt.set_prop(adc, "status", b"okay\0".to_vec());
    let bad = rp1_dt_check(&fdt, &a.note);
    assert!(bad.iter().any(|c| c.contains("rp1_adc")), "{bad:?}");
}

#[test]
fn spi_overlay_and_later_linux_conflict() {
    let tmp = tempfile::tempdir().unwrap();
    let mut d = declaration();
    d.proc1.instances.insert(
        "spi0".into(),
        Instance::Spi(SpiPins {
            sck: Some(11),
            mosi: Some(10),
            miso: Some(9),
        }),
    );
    let base = dtc(
        tmp.path(),
        "base",
        &BASE
            .replace("@EXTRA_RESERVED@", "")
            .replace("@EXTRA_RP1@", "")
            .replace(
                "rp1_spi0: spi0 { };",
                r#"rp1_spi0: spi0 { status = "okay"; };"#,
            ),
    );
    let a = generate("spi", "0.1.0", &d, [0; 32], [0; 20]).unwrap();
    let app = dtc(tmp.path(), "app", &a.dts);
    let merged = tmp.path().join("merged.dtb");
    overlay::apply(&base, &app, &merged).unwrap();
    let mut fdt = rp1_build::fdt::Fdt::parse(&fs::read(&merged).unwrap()).unwrap();
    assert!(rp1_dt_check(&fdt, &a.note).is_empty());
    let spi = fdt.label("rp1_spi0").unwrap();
    assert!(!fdt.enabled(spi));
    assert!(fdt.enabled(fdt.label("rp1_spi1").unwrap()));
    fdt.set_prop(spi, "status", b"okay\0".to_vec());
    let bad = rp1_dt_check(&fdt, &a.note);
    assert!(bad.iter().any(|c| c.contains("rp1_spi0")), "{bad:?}");
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
fn present_openamp_is_disabled_and_board_hogs_stay() {
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
    // GPIO46 is a board line Linux keeps: the stock hog is untouched.
    assert!(fdt.enabled(hog));
}
