//! STATIC: predict boot-time ownership with the lab firmware's declaration and test overlays.
//! The default tests use a synthetic tree. Set `RP1_TEST_DTB` to an unmodified
//! `bcm2712-rpi-cm5l-cm5io.dtb` to also check the stock CM5 Lite board tree.
use rp1_build::{declaration, generate, metadata::TypeInfo, overlay};
use rp1_dt::check::{Disable, Plan};
use std::{fs, path::Path, process::Command};

const BOARD: &str = include_str!("fixtures/cm5l-rp1.dts");
const GPIOMEM: &str = "/pcie/rp1/gpiomem@c0400d0000";
const UART3: &str = "/pcie/rp1/serial3";
/// `dtoverlay=uart1-pi5`: Linux UART1 on GPIO 0/1 (the lab's config.txt).
const UART1_PI5: &str = "&uart1 { status = \"okay\"; pinctrl-0 = <&uart1_pins>; };";
/// `dtoverlay=uart3-pi5`: UART3 on GPIO 8/9 (SPI0 CS and MISO in the lab firmware).
const UART3_PI5: &str = "&uart3 { status = \"okay\"; pinctrl-0 = <&uart3_pins>; };";
/// The LED's parent configures GPIO22 and its child uses GPIO22 as an output.
const GPIO_LED_22: &str = r#"
&rp1_gpio {
    indicator_pin: indicator-pin {
        groups = "gpio22";
        function = "gpio";
    };
};

&{/} {
    ownership-indicators {
        pinctrl-0 = <&indicator_pin>;
        pinctrl-names = "default";
        compatible = "gpio-leds";

        conflict {
            gpios = <&rp1_gpio 22 0>;
        };
    };
};
"#;

fn dtc(dir: &Path, name: &str, source: &str) -> Vec<u8> {
    let dts = dir.join(format!("{name}.dts"));
    let out = dir.join(format!("{name}.dtb"));
    fs::write(&dts, source).unwrap();
    let o = Command::new("dtc")
        .args(["-@", "-q", "-I", "dts", "-O", "dtb", "-o"])
        .arg(&out)
        .arg(&dts)
        .output()
        .expect("dtc required for DTB tests");
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    fs::read(out).unwrap()
}

/// The firmware DTB after the given config.txt overlays, and the lab firmware's `.rp1.dtbo` and
/// note. Then what rp1boot would do.
fn predict(board: &[u8], overlays: &[&str]) -> overlay::BootPrediction {
    let tmp = tempfile::tempdir().unwrap();
    let mut base = rp1_dt::fdt::Fdt::parse(board).unwrap();
    for (i, source) in overlays.iter().enumerate() {
        let dtbo = dtc(
            tmp.path(),
            &format!("config{i}"),
            &format!("/dts-v1/;\n/plugin/;\n{source}\n"),
        );
        rp1_dt::overlay::apply(&mut base, &dtbo).unwrap();
    }
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR")).join("../lab/shm-firmware/Cargo.toml");
    let (name, version, mut d) = declaration(&manifest).unwrap();
    d.types.extend([
        ("Sample".into(), TypeInfo::of::<[u32; 4]>()),
        ("Control".into(), TypeInfo::of::<[u32; 16]>()),
        ("Bulk".into(), TypeInfo::of::<[u32; 262144]>()),
        ("Wide64".into(), TypeInfo::of::<[u32; 16]>()),
        ("Wide256".into(), TypeInfo::of::<[u32; 64]>()),
    ]);
    let a = generate(&name, &version, &d, [0; 32], [0; 20]).unwrap();
    let app = dtc(tmp.path(), "app", &a.dts);
    overlay::predict_boot(&base.to_bytes(), &app, &a.note).unwrap()
}

fn fixture() -> Vec<u8> {
    let tmp = tempfile::tempdir().unwrap();
    dtc(tmp.path(), "board", BOARD)
}

fn disabled(p: overlay::BootPrediction) -> Vec<Disable> {
    let Plan::Disable(nodes) = p.plan else {
        panic!("unexpected refusal: {:?}", p.plan);
    };
    nodes
}

fn assert_gpiomem_only(prediction: overlay::BootPrediction, path: &str) {
    let nodes = disabled(prediction);
    assert_eq!(nodes.len(), 1, "{nodes:?}");
    assert_eq!(nodes[0].path, path);
    assert!(nodes[0].policy_only, "{nodes:?}");
}

fn assert_uart3_conflict(prediction: overlay::BootPrediction, uart_path: &str, gpiomem_path: &str) {
    let nodes = disabled(prediction);
    let [uart3, gpiomem] = nodes.as_slice() else {
        panic!("expected UART3 and gpiomem disables: {nodes:?}");
    };
    assert_eq!(uart3.path, uart_path);
    assert_eq!(gpiomem.path, gpiomem_path);
    assert!(!uart3.policy_only && gpiomem.policy_only, "{nodes:?}");
    for pin in ["GPIO8", "GPIO9"] {
        assert!(uart3.reasons.iter().any(|r| r.contains(pin)), "{uart3:?}");
    }
}

#[test]
fn lab_config_disables_only_gpiomem_by_policy() {
    assert_gpiomem_only(predict(&fixture(), &[UART1_PI5]), GPIOMEM);
}

#[test]
fn uart3_pi5_disables_uart3_and_gpiomem() {
    assert_uart3_conflict(predict(&fixture(), &[UART1_PI5, UART3_PI5]), UART3, GPIOMEM);
}

#[test]
fn gpio_led_on_an_rp1_pin_disables_the_leds_node_its_led_and_gpiomem() {
    let nodes = disabled(predict(&fixture(), &[UART1_PI5, GPIO_LED_22]));
    let paths: Vec<_> = nodes.iter().map(|n| n.path.as_str()).collect();
    assert_eq!(
        paths,
        [
            GPIOMEM,
            "/ownership-indicators",
            "/ownership-indicators/conflict"
        ],
        "{nodes:?}"
    );
    assert!(nodes[0].policy_only, "{nodes:?}");
    assert!(
        nodes[1]
            .reasons
            .iter()
            .any(|r| r.contains("configures RP1-owned GPIO22")),
        "{nodes:?}"
    );
    assert!(
        nodes[2]
            .reasons
            .iter()
            .any(|r| r.contains("gpios uses GPIO22")),
        "{nodes:?}"
    );
}

#[test]
fn consumers_under_a_disabled_parent_are_not_disabled() {
    let child =
        "&rp1_i2c0 { test-child { reset-gpios = <&rp1_gpio 22 0>; pinctrl-0 = <&uart3_pins>; }; };";
    assert_gpiomem_only(predict(&fixture(), &[UART1_PI5, child]), GPIOMEM);
}

#[test]
fn pinctrl_groups_on_another_controller_are_not_rp1_pins() {
    let consumer = "&{/} { soc-user { pinctrl-0 = <&soc_gpio2>; }; };";
    assert_gpiomem_only(predict(&fixture(), &[UART1_PI5, consumer]), GPIOMEM);
}

#[test]
fn unresolvable_console_does_not_stop_a_gpiomem_only_boot() {
    let board = fixture();
    let console = "&{/chosen} { stdout-path = \"serial9:115200n8\"; };";
    assert_gpiomem_only(predict(&board, &[UART1_PI5, console]), GPIOMEM);
    let p = predict(&board, &[UART1_PI5, UART3_PI5, console]);
    let Plan::Refuse(why) = &p.plan else {
        panic!("expected refusal, got {:?}", p.plan);
    };
    assert!(
        why.iter()
            .any(|w| w.contains("cannot tell whether a disable would remove the Linux console")),
        "{why:?}"
    );
}

#[test]
fn real_stock_dtb() {
    let Some(path) = std::env::var_os("RP1_TEST_DTB") else {
        eprintln!("RP1_TEST_DTB unset: skipping stock CM5 Lite DTB checks");
        return;
    };
    let board = fs::read(&path).unwrap_or_else(|err| panic!("reading stock DTB {path:?}: {err}"));
    let gpiomem = "/axi/pcie@1000120000/rp1/gpiomem@d0000";
    let uart3 = "/axi/pcie@1000120000/rp1/serial@3c000";
    let bcm_consumer = "&{/} { soc-user { pinctrl-0 = <&spi10_gpio2>; }; };";

    assert_gpiomem_only(predict(&board, &[UART1_PI5]), gpiomem);
    assert_uart3_conflict(predict(&board, &[UART1_PI5, UART3_PI5]), uart3, gpiomem);
    assert_gpiomem_only(predict(&board, &[UART1_PI5, bcm_consumer]), gpiomem);
}
