//! Schema 2 ownership model and its validation. Error codes:
//! E0 malformed declaration (unknown key, wrong value type), E1 pin claimed twice,
//! E2 pin cannot carry that signal, E3 instance declared twice, E4 not available on the board
//! (kept for Linux, unsupported), E5 missing signal or `gpio_irq` outside `gpio`.
use crate::{
    Result,
    board::{self, Kind},
    metadata::{Declaration, peripheral_type},
};
use rp1_pinmux::{I2cSignal, Signal, SpiSignal, UartSignal, candidate_pins, funcsel};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct UartPins {
    pub tx: Option<u8>,
    pub rx: Option<u8>,
    pub cts: Option<u8>,
    pub rts: Option<u8>,
}
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct I2cPins {
    pub sda: Option<u8>,
    pub scl: Option<u8>,
}
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct SpiPins {
    pub sck: Option<u8>,
    pub mosi: Option<u8>,
    pub miso: Option<u8>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum Instance {
    Uart(UartPins),
    I2c(I2cPins),
    Spi(SpiPins),
    Adc {},
}

fn signal_name(signal: Signal) -> String {
    match signal {
        Signal::Uart(n, role) => format!(
            "UART{n}_{}",
            match role {
                UartSignal::Tx => "TX",
                UartSignal::Rx => "RX",
                UartSignal::Cts => "CTS",
                UartSignal::Rts => "RTS",
                UartSignal::IrRx => "IRRX",
                UartSignal::IrTx => "IRTX",
                UartSignal::Ri => "RI",
                UartSignal::Dtr => "DTR",
                UartSignal::Dcd => "DCD",
                UartSignal::Dsr => "DSR",
            }
        ),
        Signal::I2c(n, role) => format!(
            "I2C{n}_{}",
            match role {
                I2cSignal::Sda => "SDA",
                I2cSignal::Scl => "SCL",
            }
        ),
        Signal::Spi(n, role) => format!(
            "SPI{n}_{}",
            match role {
                SpiSignal::Sclk => "SCK",
                SpiSignal::Mosi => "MOSI",
                SpiSignal::Miso => "MISO",
                SpiSignal::Sio2 => "SIO2",
                SpiSignal::Sio3 => "SIO3",
                SpiSignal::Cs(_) => "CS",
            }
        ),
        _ => unreachable!(),
    }
}

fn parse_pin_table<T: serde::de::DeserializeOwned>(value: toml::Value, path: &str) -> Result<T> {
    if !value.is_table() {
        return Err(format!("E0: {path} must be a pin table").into());
    }
    value
        .try_into()
        .map_err(|e| format!("E0: {path}: {e}").into())
}

impl Instance {
    pub(crate) fn parse(name: &str, value: toml::Value, owner: &str) -> Result<Self> {
        if let Some(why) = board::undeclarable(name) {
            return Err(why.into());
        }
        let Some(info) = board::find_instance(name) else {
            let known: Vec<_> = board::INSTANCES.iter().map(|i| i.name).collect();
            return Err(format!(
                "E0: unknown key '{name}' in {owner} (instances: {})",
                known.join(", ")
            )
            .into());
        };
        let path = format!("{owner}.{name}");
        Ok(match info.kind {
            Kind::Uart => Self::Uart(parse_pin_table(value, &path)?),
            Kind::I2c => Self::I2c(parse_pin_table(value, &path)?),
            Kind::Spi => {
                if let Some(key) = value
                    .as_table()
                    .and_then(|t| t.keys().find(|k| k.starts_with("cs")))
                {
                    return Err(format!("E4: {path}.{key}: hardware CS is unsupported").into());
                }
                Self::Spi(parse_pin_table(value, &path)?)
            }
            Kind::Adc => {
                if value.as_table().is_none_or(|t| !t.is_empty()) {
                    return Err(format!("E0: {path} must be an empty table").into());
                }
                Self::Adc {}
            }
        })
    }

    fn signals(&self, n: u8) -> Vec<(&'static str, Option<u8>, Signal)> {
        match self {
            Self::Uart(p) => vec![
                ("tx", p.tx, Signal::Uart(n, UartSignal::Tx)),
                ("rx", p.rx, Signal::Uart(n, UartSignal::Rx)),
                ("cts", p.cts, Signal::Uart(n, UartSignal::Cts)),
                ("rts", p.rts, Signal::Uart(n, UartSignal::Rts)),
            ],
            Self::I2c(p) => vec![
                ("sda", p.sda, Signal::I2c(n, I2cSignal::Sda)),
                ("scl", p.scl, Signal::I2c(n, I2cSignal::Scl)),
            ],
            Self::Spi(p) => vec![
                ("sck", p.sck, Signal::Spi(n, SpiSignal::Sclk)),
                ("mosi", p.mosi, Signal::Spi(n, SpiSignal::Mosi)),
                ("miso", p.miso, Signal::Spi(n, SpiSignal::Miso)),
            ],
            Self::Adc {} => vec![],
        }
    }
    fn required(&self) -> &'static [&'static str] {
        match self {
            Self::Uart(_) => &["tx", "rx"],
            Self::I2c(_) => &["sda", "scl"],
            Self::Spi(_) => &["sck", "mosi", "miso"],
            Self::Adc {} => &[],
        }
    }
    fn number(name: &str) -> u8 {
        board::find_instance(name).map_or(0, |i| i.number)
    }

    /// Declared signal names and GPIOs for an instance key.
    pub fn pins(&self, name: &str) -> Vec<(&'static str, u8)> {
        self.signals(Self::number(name))
            .into_iter()
            .filter_map(|(role, pin, _)| pin.map(|pin| (role, pin)))
            .collect()
    }
    fn validate(&self, name: &str, owner: &str, unrouted: bool) -> Result<()> {
        // Check the key/variant too: callers may construct or mutate the public model in Rust.
        let parsed = Self::parse(name, toml::Value::try_from(self)?, owner)?;
        if std::mem::discriminant(self) != std::mem::discriminant(&parsed) {
            return Err(format!("E0: {owner}.{name}: wrong instance pin table").into());
        }
        let signals = self.signals(Self::number(name));
        if !unrouted
            && signals
                .iter()
                .any(|(role, pin, _)| self.required().contains(role) && pin.is_none())
        {
            return Err(format!("E5: {owner}.{name} needs {}", self.required().join(", ")).into());
        }
        for (role, pin, signal) in signals {
            let Some(pin) = pin else {
                continue;
            };
            validate_pin(pin)?;
            if funcsel(pin, signal).is_none() {
                let choices = candidate_pins(signal)
                    .map(|p| format!("GPIO{p}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                let actual = rp1_pinmux::BANK0
                    .get(usize::from(pin))
                    .and_then(|row| {
                        row.iter().enumerate().find(|(_, s)| {
                            matches!((signal, s),
                        (Signal::Uart(n, _), Signal::Uart(m, _)) |
                        (Signal::I2c(n, _), Signal::I2c(m, _)) |
                        (Signal::Spi(n, _), Signal::Spi(m, _)) if n == *m)
                        })
                    })
                    .map(|(alt, s)| format!("{} (alt{alt})", signal_name(*s)))
                    .unwrap_or_else(|| "no signal for this block".into());
                return Err(format!("E2: {owner}.{name}.{role} = {pin}: GPIO{pin} is {actual}, not {}; {} is on {choices}", signal_name(signal), signal_name(signal)).into());
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct Core {
    pub gpio: Vec<u8>,
    pub gpio_irq: Vec<u8>,
    pub timer: Vec<String>,
    #[serde(flatten)]
    pub instances: BTreeMap<String, Instance>,
    // Ordering hints preserve schema 1 note bytes without making the old lists authoritative.
    #[serde(skip)]
    pin_order: Vec<u8>,
    #[serde(skip)]
    peripheral_order: Vec<String>,
    #[serde(skip)]
    unrouted: BTreeSet<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct LegacyCore {
    pub pins: Vec<u8>,
    pub peripherals: Vec<String>,
    pub gpio_irq: Vec<u8>,
}
impl Core {
    pub(crate) fn parse(
        mut values: BTreeMap<String, toml::Value>,
        schema: u32,
        owner: &str,
    ) -> Result<Self> {
        if schema == 1 {
            let old: LegacyCore = toml::Value::try_from(values)?.try_into()?;
            let mut core = Self {
                gpio: old.pins.clone(),
                gpio_irq: old.gpio_irq,
                pin_order: old.pins.clone(),
                peripheral_order: old.peripherals.clone(),
                ..Self::default()
            };
            let mut seen = BTreeSet::new();
            for pin in &old.pins {
                if !seen.insert(pin) {
                    return Err(format!("E1: GPIO {pin} is claimed twice in {owner}.pins").into());
                }
            }
            for block in old.peripherals {
                peripheral_type(&block)?;
                if let Some(alarm) = block.strip_prefix("TIMER.") {
                    core.timer.push(alarm.into());
                    continue;
                }
                let name = block.to_ascii_lowercase();
                let mut instance =
                    Instance::parse(&name, toml::Value::Table(toml::Table::new()), owner)?;
                let signals = instance.signals(Instance::number(&name));
                let candidates: Vec<_> = signals
                    .iter()
                    .map(|(_, _, signal)| {
                        candidate_pins(*signal)
                            .filter(|p| old.pins.contains(p))
                            .collect::<Vec<_>>()
                    })
                    .collect();
                let complete = signals.iter().zip(&candidates).all(|((role, _, _), pins)| {
                    !instance.required().contains(role) || !pins.is_empty()
                });
                if complete && candidates.iter().any(|p| p.len() > 1) {
                    return Err(format!("schema 1: cannot infer {block} pins from {:?}; write {name} = {{ {} }} (schema 2)", old.pins,
                        instance.required().iter().map(|r| format!("{r} = ..")).collect::<Vec<_>>().join(", ")).into());
                }
                if complete {
                    let pin = |i: usize| candidates[i].first().copied();
                    instance = match instance {
                        Instance::Uart(_) => Instance::Uart(UartPins {
                            tx: pin(0),
                            rx: pin(1),
                            cts: pin(2),
                            rts: pin(3),
                        }),
                        Instance::I2c(_) => Instance::I2c(I2cPins {
                            sda: pin(0),
                            scl: pin(1),
                        }),
                        Instance::Spi(_) => Instance::Spi(SpiPins {
                            sck: pin(0),
                            mosi: pin(1),
                            miso: pin(2),
                        }),
                        Instance::Adc {} => Instance::Adc {},
                    };
                    for (_, pin) in instance.pins(&name) {
                        core.gpio.retain(|p| *p != pin);
                    }
                } else {
                    // Schema 1 allowed block tokens without a pin route; unused pins stay GPIO.
                    core.unrouted.insert(name.clone());
                }
                if core.instances.insert(name, instance).is_some() {
                    return Err(format!("E3: {block} is declared twice by {owner}").into());
                }
            }
            return Ok(core);
        }
        let mut core = Self::default();
        fn field<T: serde::de::DeserializeOwned>(v: toml::Value, path: String) -> Result<T> {
            v.try_into().map_err(|e| format!("E0: {path}: {e}").into())
        }
        if let Some(v) = values.remove("gpio") {
            core.gpio = field(v, format!("{owner}.gpio"))?;
        }
        if let Some(v) = values.remove("gpio_irq") {
            core.gpio_irq = field(v, format!("{owner}.gpio_irq"))?;
        }
        if let Some(v) = values.remove("timer") {
            core.timer = field(v, format!("{owner}.timer"))?;
        }
        for (name, value) in values {
            core.instances
                .insert(name.clone(), Instance::parse(&name, value, owner)?);
        }
        Ok(core)
    }
    /// All owned pins; legacy declaration order is retained for note compatibility.
    pub fn pins(&self) -> Vec<u8> {
        let mut pins = self.gpio.clone();
        for (name, instance) in &self.instances {
            pins.extend(instance.pins(name).into_iter().map(|(_, p)| p));
        }
        pins.sort_by_key(|p| {
            (
                self.pin_order
                    .iter()
                    .position(|n| n == p)
                    .unwrap_or(usize::MAX),
                *p,
            )
        });
        pins
    }
    pub fn peripherals(&self) -> Vec<String> {
        let mut blocks: Vec<_> = self
            .timer
            .iter()
            .map(|s| format!("TIMER.{s}"))
            .chain(self.instances.keys().map(|s| s.to_ascii_uppercase()))
            .collect();
        blocks.sort_by_key(|p| {
            self.peripheral_order
                .iter()
                .position(|n| n == p)
                .unwrap_or(usize::MAX)
        });
        blocks
    }
}

fn validate_pin(pin: u8) -> Result<()> {
    if let Some(line) = board::linux_line(pin) {
        return Err(
            format!("E4: GPIO {pin} is board line {line}, owned by Linux on rpi-cm5").into(),
        );
    }
    if pin >= 28 {
        return Err(format!("E2: GPIO {pin} is out of range (bank 0 is GPIO 0..27)").into());
    }
    Ok(())
}

impl Declaration {
    pub(crate) fn validate_ownership(&self) -> Result<()> {
        let mut blocks = BTreeMap::new();
        let mut pins = BTreeMap::new();
        if let Some(log) = &self.log {
            if log.sink != "uart0" || log.baudrate == 0 {
                return Err("E0: invalid UART0 log declaration".into());
            }
            blocks.insert("uart0".to_string(), "the log sink".to_string());
            pins.insert(14, "log.uart0.tx".to_string());
            pins.insert(15, "log.uart0.rx".to_string());
        }
        // Attribute the board default to the board and an explicit linux.keep to the user.
        let (keep, keeper, pin_prefix) = match &self.linux.keep {
            Some(keep) => (keep.clone(), "Linux by linux.keep", "linux.keep."),
            None => (
                board::linux_keep_default(),
                "Linux by board rpi-cm5 (default linux.keep)",
                BOARD_KEEP,
            ),
        };
        for (name, instance) in &keep {
            instance.validate(name, "linux.keep", false)?;
            if let Some(previous) = blocks.insert(name.clone(), keeper.into()) {
                return Err(format!(
                    "E3: {} is declared by both {previous} and linux.keep",
                    name.to_ascii_uppercase()
                )
                .into());
            }
            for (role, pin) in instance.pins(name) {
                claim_pin(&mut pins, pin, format!("{pin_prefix}{name}.{role}"))?;
            }
        }
        for (owner, core) in [("proc0", &self.proc0), ("proc1", &self.proc1)] {
            for name in core.instances.keys() {
                if let Some(previous) = blocks.insert(name.clone(), owner.into()) {
                    return Err(format!(
                        "E3: {} is declared by both {previous} and {owner}",
                        name.to_ascii_uppercase()
                    )
                    .into());
                }
            }
            for alarm in &core.timer {
                peripheral_type(&format!("TIMER.{alarm}"))?;
                if let Some(previous) = blocks.insert(format!("timer.{alarm}"), owner.into()) {
                    return Err(format!(
                        "E3: TIMER.{alarm} is declared by both {previous} and {owner}"
                    )
                    .into());
                }
            }
        }
        for (owner, core) in [("proc0", &self.proc0), ("proc1", &self.proc1)] {
            for pin in &core.gpio {
                validate_pin(*pin)?;
                claim_pin(&mut pins, *pin, format!("{owner}.gpio"))?;
            }
            for pin in &core.gpio_irq {
                if !core.gpio.contains(pin) {
                    return Err(format!("E5: gpio_irq {pin} is not in {owner}.gpio").into());
                }
            }
            for (name, instance) in &core.instances {
                instance.validate(
                    name,
                    owner,
                    core.unrouted.contains(name) && instance.pins(name).is_empty(),
                )?;
                for (role, pin) in instance.pins(name) {
                    claim_pin(&mut pins, pin, format!("{owner}.{name}.{role}"))?;
                }
            }
        }
        Ok(())
    }
}
/// Claim prefix of pins the board keeps for Linux by default.
const BOARD_KEEP: &str = "board:";
fn claim_pin(pins: &mut BTreeMap<u8, String>, pin: u8, owner: String) -> Result<()> {
    if let Some(previous) = pins.insert(pin, owner.clone()) {
        if let Some(signal) = previous.strip_prefix(BOARD_KEEP) {
            return Err(format!(
                "E4: GPIO {pin} is kept for Linux ({signal}) by board rpi-cm5; claimed by {owner}"
            )
            .into());
        }
        return Err(format!("E1: GPIO {pin} is claimed by both {previous} and {owner}").into());
    }
    Ok(())
}
