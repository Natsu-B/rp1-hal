//! DOC: Bank-0 signal vocabulary from RP1 Peripherals Table 4 and §3.6.1.1.
use std::{collections::BTreeMap, fmt::Write as _, fs, path::Path, process::Command};

use anyhow::{Context, Result, ensure};
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bank0 {
    pub pins: Vec<Pin>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pin {
    pub gpio: u8,
    pub functions: [String; 9],
}

/// Parse and validate all 28 rows before generating any output.
pub fn load(path: &Path) -> Result<Bank0> {
    let text = fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    let bank: Bank0 = toml::from_str(&text).context("parsing bank-0 pin functions")?;
    bank.validate()?;
    Ok(bank)
}

impl Bank0 {
    pub fn validate(&self) -> Result<()> {
        ensure!(self.pins.len() == 28, "bank 0 must contain GPIO 0..27");
        let names = signal_names();
        for (index, pin) in self.pins.iter().enumerate() {
            ensure!(
                usize::from(pin.gpio) == index,
                "expected GPIO{index} in order"
            );
            for (func, name) in pin.functions.iter().enumerate() {
                ensure!(
                    names.contains_key(name),
                    "GPIO{index}/a{func}: unknown signal {name}"
                );
                ensure!(
                    name == "-" || !pin.functions[..func].contains(name),
                    "GPIO{index}: duplicate signal {name}"
                );
            }
            for (func, prefix) in [(5, "SYS_RIO"), (6, "PROC_RIO"), (7, "PIO")] {
                ensure!(
                    pin.functions[func] == format!("{prefix}[{index}]"),
                    "GPIO{index}/a{func}: expected {prefix}[{index}]"
                );
            }
        }
        Ok(())
    }
}

/// Write checked-in Rust, formatted the same way as the PAC generator.
pub fn generate(bank: &Bank0, output: &Path) -> Result<()> {
    bank.validate()?;
    let names = signal_names();
    let mut out = String::from(
        "// Generated from rp1-regdb/data/pinmux/bank0.toml; do not edit.\n\
         use crate::*;\n\
         /// DOC: GPIO 0..27 × FUNCSEL a0..a8, RP1 Peripherals Table 4.\n\
         pub const BANK0: [[Signal; 9]; 28] = [\n",
    );
    for pin in &bank.pins {
        writeln!(out, "// GPIO{}", pin.gpio)?;
        writeln!(
            out,
            "[{}],",
            pin.functions
                .iter()
                .map(|name| names[name].as_str())
                .collect::<Vec<_>>()
                .join(", ")
        )?;
    }
    out.push_str("];\n");
    write_rust(output, &out)
}

/// Write rp1-hal's pin-route impls: one `impl <trait><n> for Pin<gpio>` per bank-0 signal the
/// HAL drives, with its FUNCSEL.
pub fn generate_hal(bank: &Bank0, output: &Path) -> Result<()> {
    bank.validate()?;
    let mut routes = Vec::new();
    for pin in &bank.pins {
        for (func, name) in pin.functions.iter().enumerate() {
            if let Some((route, n)) = hal_route(name) {
                routes.push((route, n, pin.gpio, func));
            }
        }
    }
    routes.sort();
    let mut out = String::from(
        "// Generated from rp1-regdb/data/pinmux/bank0.toml; do not edit.\n\
         //! DOC: bank-0 routes of the signals rp1-hal drives, with their function select\n\
         //! (RP1 Peripherals Table 4). SPI4 is slave-only and has no driver.\n\
         use crate::{gpio::Pin, i2c, spi, uart};\n",
    );
    for (route, n, gpio, func) in routes {
        writeln!(
            out,
            "impl {route}<{n}> for Pin<{gpio}> {{ const FUNC: u8 = {func}; }}"
        )?;
    }
    write_rust(output, &out)
}

/// The rp1-hal pin trait and instance of a bank-0 signal name, e.g. `UART2_TX` →
/// `("uart::Tx", 2)`; `None` for signals the HAL does not route.
fn hal_route(name: &str) -> Option<(&'static str, u8)> {
    let (block, role) = name.split_once('_')?;
    let family = block.trim_end_matches(|c: char| c.is_ascii_digit());
    let n: u8 = block[family.len()..].parse().ok()?;
    let route = match (family, role) {
        ("UART", "TX") => "uart::Tx",
        ("UART", "RX") => "uart::Rx",
        ("I2C", "SDA") => "i2c::Sda",
        ("I2C", "SCL") => "i2c::Scl",
        ("SPI", _) if n == 4 => return None,
        ("SPI", "SCLK") => "spi::Sck",
        ("SPI", "SIO[0]") => "spi::Mosi",
        ("SPI", "SIO[1]") => "spi::Miso",
        _ => return None,
    };
    Some((route, n))
}

fn write_rust(output: &Path, text: &str) -> Result<()> {
    fs::write(output, text)?;
    let status = Command::new("rustfmt")
        .args(["--edition", "2024"])
        .arg(output)
        .status()
        .context("running rustfmt on generated pin functions")?;
    ensure!(
        status.success(),
        "rustfmt failed for generated pin functions"
    );
    Ok(())
}

fn signal_names() -> BTreeMap<String, String> {
    let mut names = BTreeMap::from([("-".into(), "Signal::None".into())]);
    let mut add = |name: String, expression: String| {
        names.insert(name, expression);
    };
    for (family, max, roles) in [
        (
            "UART",
            4,
            &[("TX", "Tx"), ("RX", "Rx"), ("CTS", "Cts"), ("RTS", "Rts")][..],
        ),
        ("I2C", 3, &[("SDA", "Sda"), ("SCL", "Scl")][..]),
    ] {
        let rust = if family == "UART" { "Uart" } else { "I2c" };
        for n in 0..=max {
            for (name, variant) in roles {
                add(
                    format!("{family}{n}_{name}"),
                    format!("Signal::{rust}({n}, {rust}Signal::{variant})"),
                );
            }
        }
    }
    for (name, variant) in [
        ("IR_RX", "IrRx"),
        ("IR_TX", "IrTx"),
        ("RI", "Ri"),
        ("DTR", "Dtr"),
        ("DCD", "Dcd"),
        ("DSR", "Dsr"),
    ] {
        add(
            format!("UART0_{name}"),
            format!("Signal::Uart(0, UartSignal::{variant})"),
        );
    }
    for n in 0..=5 {
        add(
            format!("SPI{n}_SCLK"),
            format!("Signal::Spi({n}, SpiSignal::Sclk)"),
        );
        for (lane, role) in if n == 4 {
            ["Miso", "Mosi", "Sio2", "Sio3"]
        } else {
            ["Mosi", "Miso", "Sio2", "Sio3"]
        }
        .iter()
        .enumerate()
        {
            add(
                format!("SPI{n}_SIO[{lane}]"),
                format!("Signal::Spi({n}, SpiSignal::{role})"),
            );
        }
        for cs in 0..4 {
            add(
                format!("SPI{n}_CSn[{cs}]"),
                format!("Signal::Spi({n}, SpiSignal::Cs({cs}))"),
            );
        }
    }
    for channel in 0..4 {
        add(
            format!("PWM0[{channel}]"),
            format!("Signal::Pwm(0, PwmSignal::Channel{channel})"),
        );
        add(
            format!("SDIO0_DAT[{channel}]"),
            format!("Signal::Sdio(0, SdioSignal::Dat({channel}))"),
        );
    }
    for (name, role) in [("CLK", "Clk"), ("CMD", "Cmd")] {
        add(
            format!("SDIO0_{name}"),
            format!("Signal::Sdio(0, SdioSignal::{role})"),
        );
    }
    for n in 0..2 {
        add(format!("MIPI{n}_DSI_TE"), format!("Signal::MipiDsiTe({n})"));
        for (name, role) in [("SCLK", "Sclk"), ("WS", "Ws")] {
            add(
                format!("I2S{n}_{name}"),
                format!("Signal::I2s({n}, I2sSignal::{role})"),
            );
        }
        for lane in 0..4 {
            for (name, role) in [("SDI", "Sdi"), ("SDO", "Sdo")] {
                add(
                    format!("I2S{n}_{name}[{lane}]"),
                    format!("Signal::I2s({n}, I2sSignal::{role}({lane}))"),
                );
            }
        }
    }
    for (name, role) in [
        ("PCLK", "Pclk"),
        ("DE", "De"),
        ("VSYNC", "Vsync"),
        ("HSYNC", "Hsync"),
    ] {
        add(
            format!("DPI_{name}"),
            format!("Signal::Dpi(DpiSignal::{role})"),
        );
    }
    for lane in 0..24 {
        add(
            format!("DPI_D[{lane}]"),
            format!("Signal::Dpi(DpiSignal::D({lane}))"),
        );
    }
    for n in 0..3 {
        add(format!("GPCLK[{n}]"), format!("Signal::Gpclk({n})"));
    }
    for (name, role) in [("L", "Left"), ("R", "Right")] {
        add(
            format!("AUDIO_OUT_{name}"),
            format!("Signal::AudioOut(AudioOutSignal::{role})"),
        );
    }
    for (name, role) in [("CLK", "Clk"), ("DAT0", "Dat0"), ("DAT1", "Dat1")] {
        add(
            format!("AUDIO_IN_{name}"),
            format!("Signal::AudioIn(AudioInSignal::{role})"),
        );
    }
    for pin in 0..28 {
        for (name, role) in [
            ("SYS_RIO", "SysRio"),
            ("PROC_RIO", "ProcRio"),
            ("PIO", "Pio"),
        ] {
            add(format!("{name}[{pin}]"), format!("Signal::{role}({pin})"));
        }
    }
    names
}
