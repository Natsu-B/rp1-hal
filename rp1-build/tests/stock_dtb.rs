//! STATIC: the boot-time enforcement on the real CM5 Lite board tree (a reduced copy of the stock
//! DTB, `fixtures/cm5l-rp1.dts`) with the lab firmware's declaration, and config.txt overlays
//! written as the equivalents of the stock `uart1-pi5`, `uart3-pi5` and `gpio-led` overlays.
use rp1_build::{declaration, generate, metadata::TypeInfo, overlay};
use rp1_dt::check::Plan;
use std::{fs, path::Path, process::Command};

const BOARD: &str = include_str!("fixtures/cm5l-rp1.dts");
const RP1: &str = "/axi/pcie@1000120000/rp1";
/// `dtoverlay=uart1-pi5`: Linux UART1 on GPIO 0/1 (the lab's config.txt).
const UART1_PI5: &str = "&uart1 { status = \"okay\"; pinctrl-0 = <&uart1_pins>; };";
/// `dtoverlay=uart3-pi5`: UART3 on GPIO 8/9 (SPI0 CS and MISO in the lab firmware).
const UART3_PI5: &str = "&uart3 { status = \"okay\"; pinctrl-0 = <&uart3_pins>; };";
/// `dtoverlay=gpio-led,gpio=22`: a pin group under `gpio` and a gpio-leds node using GPIO22.
const GPIO_LED_22: &str = "&gpio { led_pins: led_pins@22 { brcm,pins = <22>; brcm,function = <1>; brcm,pull = <0>; }; };
&{/} { leds@0 { compatible = \"gpio-leds\"; pinctrl-names = \"default\"; pinctrl-0 = <&led_pins>; status = \"okay\";
    led { label = \"myled1\"; gpios = <&gpio 22 0>; linux,default-trigger = \"none\"; }; }; };";

fn dtc(dir: &Path, name: &str, source: &str, symbols: bool) -> Vec<u8> {
    let dts = dir.join(format!("{name}.dts"));
    let out = dir.join(format!("{name}.dtb"));
    fs::write(&dts, source).unwrap();
    let mut cmd = Command::new("dtc");
    if symbols {
        cmd.arg("-@");
    }
    let o = cmd
        .args(["-q", "-I", "dts", "-O", "dtb", "-o"])
        .arg(&out)
        .arg(&dts)
        .output()
        .expect("dtc required for DTB tests");
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    fs::read(out).unwrap()
}

/// The firmware DTB after the given config.txt overlays, and the lab firmware's `.rp1.dtbo` and
/// note. Then what rp1boot would do.
fn predict(overlays: &[&str]) -> overlay::BootPrediction {
    let tmp = tempfile::tempdir().unwrap();
    let mut base = rp1_dt::fdt::Fdt::parse(&dtc(tmp.path(), "board", BOARD, false)).unwrap();
    for (i, source) in overlays.iter().enumerate() {
        let dtbo = dtc(
            tmp.path(),
            &format!("config{i}"),
            &format!("/dts-v1/;\n/plugin/;\n{source}\n"),
            true,
        );
        rp1_dt::overlay::apply(&mut base, &dtbo).unwrap();
    }
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR")).join("../lab/shm-firmware/Cargo.toml");
    let (name, version, mut d) = declaration(&manifest).unwrap();
    d.types.extend([
        ("Sample".into(), TypeInfo::of::<[u32; 4]>()),
        ("Control".into(), TypeInfo::of::<[u32; 16]>()),
        ("Bulk".into(), TypeInfo::of::<[u32; 262144]>()),
    ]);
    let a = generate(&name, &version, &d, [0; 32], [0; 20]).unwrap();
    let app = dtc(tmp.path(), "app", &a.dts, true);
    overlay::predict_boot(&base.to_bytes(), &app, &a.note).unwrap()
}

fn disabled(p: &overlay::BootPrediction) -> Vec<(String, bool, String)> {
    let Plan::Disable(nodes) = &p.plan else {
        panic!("unexpected refusal: {:?}", p.plan);
    };
    nodes
        .iter()
        .map(|n| (n.path.clone(), n.policy_only, n.reasons.join("; ")))
        .collect()
}

#[test]
fn lab_config_disables_only_gpiomem_by_policy() {
    let nodes = disabled(&predict(&[UART1_PI5]));
    assert_eq!(nodes.len(), 1, "{nodes:?}");
    assert_eq!(nodes[0].0, format!("{RP1}/gpiomem@d0000"));
    assert!(nodes[0].1, "{nodes:?}");
}

#[test]
fn uart3_pi5_disables_serial_3c000_and_gpiomem() {
    let nodes = disabled(&predict(&[UART1_PI5, UART3_PI5]));
    let paths: Vec<_> = nodes.iter().map(|n| n.0.as_str()).collect();
    assert_eq!(
        paths,
        [
            format!("{RP1}/serial@3c000"),
            format!("{RP1}/gpiomem@d0000")
        ],
        "{nodes:?}"
    );
    let uart3 = &nodes[0];
    assert!(!uart3.1);
    assert!(
        uart3.2.contains("GPIO8") && uart3.2.contains("GPIO9"),
        "{uart3:?}"
    );
}

#[test]
fn gpio_led_on_an_rp1_pin_disables_the_leds_node_its_led_and_gpiomem() {
    let nodes = disabled(&predict(&[UART1_PI5, GPIO_LED_22]));
    let paths: Vec<_> = nodes.iter().map(|n| n.0.as_str()).collect();
    assert_eq!(
        paths,
        [&format!("{RP1}/gpiomem@d0000"), "/leds@0", "/leds@0/led"],
        "{nodes:?}"
    );
    assert!(nodes[0].1, "{nodes:?}");
    assert!(
        nodes[1].2.contains("configures RP1-owned GPIO22"),
        "{nodes:?}"
    );
    assert!(nodes[2].2.contains("gpios uses GPIO22"), "{nodes:?}");
}

#[test]
fn consumers_under_a_disabled_parent_are_not_disabled() {
    // rp1_i2c0 (i2c@70000) is disabled in the board DTB; its child's reset GPIO is RP1's.
    let nodes = disabled(&predict(&[
        UART1_PI5,
        "&rp1_i2c0 { status = \"disabled\"; dev@50 { reg = <0x50>; reset-gpios = <&rp1_gpio 22 0>; pinctrl-0 = <&uart3_pins>; }; };",
    ]));
    assert_eq!(nodes.len(), 1, "{nodes:?}");
    assert!(nodes[0].1, "{nodes:?}");
}

#[test]
fn bcm2712_pinctrl_groups_named_gpio_n_are_not_rp1_pins() {
    // spi10_gpio2 is a BCM2712 group on its GPIO 2..4: same names, other controller.
    let nodes = disabled(&predict(&[
        UART1_PI5,
        "&{/} { bcm-user { pinctrl-0 = <&spi10_gpio2>; status = \"okay\"; }; };",
    ]));
    assert_eq!(nodes.len(), 1, "{nodes:?}");
    assert!(nodes[0].1, "{nodes:?}");
}

#[test]
fn unresolvable_console_does_not_stop_a_gpiomem_only_boot() {
    // Review 12 (Claude L3): the stock tree always has gpiomem@d0000, so the default policy
    // always plans one disable. An unknown console stops the boot only next to a conflict.
    let console = "&{/chosen} { stdout-path = \"serial9:115200n8\"; };";
    let nodes = disabled(&predict(&[UART1_PI5, console]));
    assert_eq!(nodes.len(), 1, "{nodes:?}");
    assert_eq!(nodes[0].0, format!("{RP1}/gpiomem@d0000"));
    assert!(nodes[0].1, "{nodes:?}");
    let p = predict(&[UART1_PI5, UART3_PI5, console]);
    let Plan::Refuse(why) = &p.plan else {
        panic!("expected refusal, got {:?}", p.plan);
    };
    assert!(
        why.iter()
            .any(|w| w.contains("cannot tell whether a disable would remove the Linux console")),
        "{why:?}"
    );
}
