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
    let hal = temp.path().join("pinmux.rs");
    pinmux::generate(&bank(), &output).unwrap();
    pinmux::generate_hal(&bank(), &hal).unwrap();
    assert_eq!(
        fs::read(&output).unwrap(),
        fs::read(root().join("../rp1-pinmux/src/generated.rs")).unwrap()
    );
    assert_eq!(
        fs::read(&hal).unwrap(),
        fs::read(root().join("../rp1-hal/src/pinmux.rs")).unwrap()
    );
    let check = || {
        Command::new(env!("CARGO_BIN_EXE_rp1-regdb"))
            .args(["generate-pinmux", "--check"])
            .arg(root().join("data/pinmux/bank0.toml"))
            .args([&output, &hal])
            .output()
            .unwrap()
    };
    assert!(check().status.success());
    for stale in [&output, &hal] {
        let current = fs::read(stale).unwrap();
        fs::write(stale, "// stale\n").unwrap();
        assert!(!check().status.success());
        fs::write(stale, current).unwrap();
    }
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

/// rp1-hal's generated routes are exactly the UART0..4 TX/RX, I2C0..3 and SPI master
/// SCLK/MOSI/MISO cells of the table, and include the hardware-tested ones.
#[test]
fn hal_routes_agree_with_the_table_in_both_directions() {
    let hal = fs::read_to_string(root().join("../rp1-hal/src/pinmux.rs")).unwrap();
    let mut routes = Vec::new();
    for line in hal.lines().filter(|line| line.starts_with("impl ")) {
        let (route, rest) = line["impl ".len()..].split_once('<').unwrap();
        let nums = numbers(rest);
        let (n, pin) = (nums[0], nums[1]);
        let signal = match route {
            "uart::Tx" => Signal::Uart(n, UartSignal::Tx),
            "uart::Rx" => Signal::Uart(n, UartSignal::Rx),
            "i2c::Sda" => Signal::I2c(n, I2cSignal::Sda),
            "i2c::Scl" => Signal::I2c(n, I2cSignal::Scl),
            "spi::Sck" => Signal::Spi(n, SpiSignal::Sclk),
            "spi::Mosi" => Signal::Spi(n, SpiSignal::Mosi),
            "spi::Miso" => Signal::Spi(n, SpiSignal::Miso),
            _ => panic!("{line}"),
        };
        routes.push((pin, signal));
    }
    // FUNC is on the line after the impl.
    let funcs: Vec<u8> = hal
        .lines()
        .filter_map(|line| line.trim().strip_prefix("const FUNC: u8 = "))
        .map(|f| f.trim_end_matches(';').parse().unwrap())
        .collect();
    assert_eq!(funcs.len(), routes.len());
    for (&(pin, signal), &func) in routes.iter().zip(&funcs) {
        assert_eq!(funcsel(pin, signal), Some(func), "GPIO{pin} {signal:?}");
    }
    let mut signals = Vec::new();
    for n in 0..5 {
        signals.extend([UartSignal::Tx, UartSignal::Rx].map(|r| Signal::Uart(n, r)));
    }
    for n in 0..4 {
        signals.extend([I2cSignal::Sda, I2cSignal::Scl].map(|r| Signal::I2c(n, r)));
    }
    for n in [0, 1, 2, 3, 5] {
        signals
            .extend([SpiSignal::Sclk, SpiSignal::Mosi, SpiSignal::Miso].map(|r| Signal::Spi(n, r)));
    }
    let expected: usize = signals.iter().map(|&s| candidate_pins(s).count()).sum();
    assert_eq!(routes.len(), expected);
    assert_eq!(expected, 10 + 18 + 15);
    for &signal in &signals {
        for pin in candidate_pins(signal) {
            assert!(routes.contains(&(pin, signal)), "GPIO{pin} {signal:?}");
        }
    }
    // HW: the UART0 log, I2C1 and SPI0 routes of the lab firmware.
    for (pin, signal, func) in [
        (14, Signal::Uart(0, UartSignal::Tx), 4),
        (15, Signal::Uart(0, UartSignal::Rx), 4),
        (2, Signal::I2c(1, I2cSignal::Sda), 3),
        (3, Signal::I2c(1, I2cSignal::Scl), 3),
        (11, Signal::Spi(0, SpiSignal::Sclk), 0),
        (10, Signal::Spi(0, SpiSignal::Mosi), 0),
        (9, Signal::Spi(0, SpiSignal::Miso), 0),
    ] {
        let i = routes.iter().position(|r| *r == (pin, signal)).unwrap();
        assert_eq!(funcs[i], func);
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
