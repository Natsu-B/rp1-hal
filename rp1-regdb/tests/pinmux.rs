use std::{fs, path::PathBuf, process::Command};

use rp1_pinmux::{I2cSignal, Signal, SpiSignal, UartSignal, candidate_pins, funcsel};
use rp1_regdb::pinmux::{self, Bank0};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn bank() -> Bank0 {
    pinmux::load(&root().join("data/pinmux/bank0.toml")).unwrap()
}

#[test]
fn generation_is_current_and_check_rejects_stale_output() {
    let temp = tempfile::tempdir().unwrap();
    let output = temp.path().join("generated.rs");
    pinmux::generate(&bank(), &output).unwrap();
    assert_eq!(
        fs::read(&output).unwrap(),
        fs::read(root().join("../rp1-pinmux/src/generated.rs")).unwrap()
    );
    let check = || {
        Command::new(env!("CARGO_BIN_EXE_rp1-regdb"))
            .args(["generate-pinmux", "--check"])
            .arg(root().join("data/pinmux/bank0.toml"))
            .arg(&output)
            .output()
            .unwrap()
    };
    assert!(check().status.success());
    fs::write(&output, "// stale\n").unwrap();
    assert!(!check().status.success());
}

#[test]
fn rejects_incomplete_unordered_unknown_duplicate_and_mismatched_rows() {
    let mut invalid = bank();
    invalid.pins.pop();
    assert!(invalid.validate().is_err());
    let mut invalid = bank();
    invalid.pins.swap(0, 1);
    assert!(invalid.validate().is_err());
    let mut invalid = bank();
    invalid.pins[0].functions[0] = "SPI0_UNKNOWN".into();
    assert!(invalid.validate().is_err());
    let mut invalid = bank();
    invalid.pins[0].functions[0] = invalid.pins[0].functions[1].clone();
    assert!(invalid.validate().is_err());
    let mut invalid = bank();
    invalid.pins[0].functions[6] = "PROC_RIO[1]".into();
    assert!(invalid.validate().is_err());
    // Array length and unknown TOML fields are rejected by deserialization.
    assert!(toml::from_str::<Bank0>("[[pins]]\ngpio=0\nfunctions=['-']").is_err());
    assert!(toml::from_str::<Bank0>("pins=[]\nextra=1").is_err());
}

fn numbers(text: &str) -> Vec<u8> {
    text.split(|c: char| !c.is_ascii_digit())
        .filter(|s| !s.is_empty())
        .map(|s| s.parse().unwrap())
        .collect()
}

#[test]
fn existing_hal_i2c_and_spi_routes_agree_in_both_directions() {
    let i2c = fs::read_to_string(root().join("../rp1-hal/src/i2c.rs")).unwrap();
    let mut i2c_routes = Vec::new();
    for line in i2c.lines().filter(|line| line.starts_with("routes!(")) {
        let nums = numbers(line);
        let n = nums[0];
        let (pairs, remainder) = nums[1..].as_chunks::<2>();
        assert!(remainder.is_empty());
        for pair in pairs {
            for (&pin, role) in pair.iter().zip([I2cSignal::Sda, I2cSignal::Scl]) {
                let signal = Signal::I2c(n, role);
                assert_eq!(funcsel(pin, signal), Some(3));
                i2c_routes.push((pin, signal));
            }
        }
    }
    assert_eq!(i2c_routes.len(), 18);
    for n in 0..4 {
        for role in [I2cSignal::Sda, I2cSignal::Scl] {
            let signal = Signal::I2c(n, role);
            for pin in candidate_pins(signal) {
                assert!(i2c_routes.contains(&(pin, signal)));
            }
        }
    }
    let spi = fs::read_to_string(root().join("../rp1-hal/src/spi.rs")).unwrap();
    let mut spi_routes = Vec::new();
    for line in spi.lines().filter(|line| line.starts_with("routes!(")) {
        let nums = numbers(line);
        assert_eq!(nums.len(), 5);
        let (n, func) = (nums[0], nums[1]);
        for (&pin, role) in
            nums[2..]
                .iter()
                .zip([SpiSignal::Sclk, SpiSignal::Mosi, SpiSignal::Miso])
        {
            let signal = Signal::Spi(n, role);
            assert_eq!(funcsel(pin, signal), Some(func));
            spi_routes.push((pin, signal));
        }
    }
    assert_eq!(spi_routes.len(), 15);
    for n in [0, 1, 2, 3, 5] {
        for role in [SpiSignal::Sclk, SpiSignal::Mosi, SpiSignal::Miso] {
            let signal = Signal::Spi(n, role);
            for pin in candidate_pins(signal) {
                assert!(spi_routes.contains(&(pin, signal)));
            }
        }
    }
}

// Linux describes peripheral families rather than individual signal roles.
fn linux_function(signal: Signal) -> String {
    match signal {
        Signal::None => "_".into(),
        Signal::Uart(0, UartSignal::IrRx | UartSignal::IrTx) => "ir".into(),
        Signal::Uart(0, UartSignal::Ri) => "ri0".into(),
        Signal::Uart(0, UartSignal::Dtr) => "dtr0".into(),
        Signal::Uart(0, UartSignal::Dcd) => "dcd0".into(),
        Signal::Uart(0, UartSignal::Dsr) => "dsr0".into(),
        Signal::Uart(n, _) => format!("uart{n}"),
        Signal::I2c(n, _) => format!("i2c{n}"),
        Signal::Spi(n, _) => format!("spi{n}"),
        Signal::Pwm(n, _) => format!("pwm{n}"),
        Signal::I2s(n, _) => format!("i2s{n}"),
        Signal::Sdio(n, _) => format!("sd{n}"),
        Signal::Dpi(_) => "dpi".into(),
        Signal::Gpclk(n) => format!("gpclk{n}"),
        Signal::MipiDsiTe(n) => format!("dsi{n}_te_ext"),
        Signal::AudioOut(_) => "aaud".into(),
        Signal::AudioIn(_) => "mic".into(),
        Signal::SysRio(_) => "gpio".into(),
        Signal::ProcRio(_) => "proc_rio".into(),
        Signal::Pio(_) => "pio".into(),
    }
}

/// Rows of `data/pinmux/linux-functions.txt`: GPIO number, then nine function names.
fn rows(text: &str) -> Vec<Vec<String>> {
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| l.split_whitespace().map(String::from).collect())
        .collect()
}

fn linux_rows() -> Vec<Vec<String>> {
    rows(&fs::read_to_string(root().join("data/pinmux/linux-functions.txt")).unwrap())
}

#[test]
fn table_agrees_with_linux_function_names() {
    let linux = linux_rows();
    assert_eq!(linux.len(), 28);
    for (pin, row) in linux.iter().enumerate() {
        assert_eq!(row.len(), 10, "GPIO{pin}");
        assert_eq!(row[0], pin.to_string());
        for (func, &signal) in rp1_pinmux::BANK0[pin].iter().enumerate() {
            assert_eq!(linux_function(signal), row[func + 1], "GPIO{pin}/a{func}");
        }
    }
}

/// Optional: re-derive the checked-in names from a local pinctrl-rp1.c.
#[test]
fn linux_names_match_the_driver_source_when_provided() {
    let Some(path) = std::env::var_os("RP1_LINUX_PINCTRL") else {
        eprintln!("set RP1_LINUX_PINCTRL to pinctrl-rp1.c to run the Linux source comparison");
        return;
    };
    let source = fs::read_to_string(path).unwrap();
    let derived: Vec<_> = source
        .lines()
        .filter_map(|line| line.trim().strip_prefix("PIN("))
        .map(|row| {
            row.trim_end_matches(',')
                .trim_end_matches(')')
                .split(',')
                .map(|f| f.trim().to_string())
                .collect::<Vec<_>>()
        })
        .filter(|row| row[0].parse::<usize>().is_ok_and(|pin| pin < 28))
        .collect();
    assert_eq!(derived, linux_rows());
}
