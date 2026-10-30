//! STATIC: rpi-cm5 ownership policy, independent of the bank-0 mux table.
use crate::{
    Result,
    metadata::{Instance, UartPins},
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, ops::RangeInclusive};

pub const LINUX_PINS: RangeInclusive<u8> = 28..=53;
pub const LINUX_BLOCKS: &[&str] = &["ETH", "USBHOST0", "USBHOST1", "DMA", "PCIE", "PIO"];

/// Block family of a declarable instance; selects its pin table and HAL token.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    Uart,
    I2c,
    Spi,
    Adc,
}

/// One declarable instance: key, family, instance number, Linux DT label and RESETS bit.
pub(crate) struct BoardInstance {
    pub name: &'static str,
    pub kind: Kind,
    pub number: u8,
    pub label: &'static str,
    pub reset_bank: usize,
    pub reset_mask: u32,
}
const fn instance(
    name: &'static str,
    kind: Kind,
    number: u8,
    label: &'static str,
    reset_bank: usize,
    reset_bit: u32,
) -> BoardInstance {
    BoardInstance {
        name,
        kind,
        number,
        label,
        reset_bank,
        reset_mask: 1 << reset_bit,
    }
}

/// STATIC: the single list of declarable instances on rpi-cm5 (RP1 RESETS for the bits).
/// Parsing, peripheral tokens, Linux-node disabling and `linux_resets` all derive from it;
/// held SCMI clocks follow the reset masks via `LinuxBlocks::held_clocks` (ADC only).
pub(crate) const INSTANCES: &[BoardInstance] = &[
    instance("uart0", Kind::Uart, 0, "rp1_uart0", 1, 26),
    instance("uart1", Kind::Uart, 1, "rp1_uart1", 1, 27),
    instance("uart2", Kind::Uart, 2, "rp1_uart2", 1, 28),
    instance("uart3", Kind::Uart, 3, "rp1_uart3", 1, 29),
    instance("uart4", Kind::Uart, 4, "rp1_uart4", 1, 30),
    instance("i2c0", Kind::I2c, 0, "rp1_i2c0", 0, 7),
    instance("i2c1", Kind::I2c, 1, "rp1_i2c1", 0, 8),
    instance("i2c2", Kind::I2c, 2, "rp1_i2c2", 0, 9),
    instance("i2c3", Kind::I2c, 3, "rp1_i2c3", 0, 10),
    instance("spi0", Kind::Spi, 0, "rp1_spi0", 1, 10),
    instance("spi1", Kind::Spi, 1, "rp1_spi1", 1, 11),
    instance("spi3", Kind::Spi, 3, "rp1_spi3", 1, 13),
    instance("spi5", Kind::Spi, 5, "rp1_spi5", 1, 15),
    instance("adc", Kind::Adc, 0, "rp1_adc", 0, 0),
];

pub(crate) fn find_instance(name: &str) -> Option<&'static BoardInstance> {
    INSTANCES.iter().find(|i| i.name == name)
}

/// Why a known RP1 block key cannot be declared on rpi-cm5, if it is one.
pub(crate) fn undeclarable(name: &str) -> Option<String> {
    let block = name.to_ascii_uppercase();
    Some(match name {
        _ if LINUX_BLOCKS.contains(&block.as_str()) => {
            format!("E4: {block} is required by Linux on rpi-cm5")
        }
        "spi4" | "spi7" => format!("E4: {block} is slave-only"),
        "spi2" => format!("E4: {block} is not supported yet"),
        "uart5" | "i2c4" | "i2c5" | "i2c6" | "spi6" | "spi8" => {
            format!("E2: {block} has no bank-0 pins")
        }
        _ => return None,
    })
}

pub fn linux_line(pin: u8) -> Option<&'static str> {
    if !LINUX_PINS.contains(&pin) {
        return None;
    }
    // Names from the gpio-line-names of the stock CM5 board DTB.
    Some(match pin {
        32 => "ETH_RST_N",
        37 => "ETH_IRQ_N",
        42 => "USB_VBUS_EN",
        44 => "RP1_STAT_LED (led-pwr)",
        _ => "board GPIO",
    })
}

pub fn linux_keep_default() -> BTreeMap<String, Instance> {
    BTreeMap::from([(
        "uart1".into(),
        Instance::Uart(UartPins {
            tx: Some(0),
            rx: Some(1),
            cts: None,
            rts: None,
        }),
    )])
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OnConflict {
    #[default]
    Disable,
    Refuse,
}
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Gpiomem {
    #[default]
    Disable,
    Keep,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct Linux {
    pub uio_group: String,
    /// Omitted: board defaults. An explicit table replaces the defaults; `{}` releases UART1.
    #[serde(deserialize_with = "parse_keep")]
    pub keep: BTreeMap<String, Instance>,
    pub gpiomem: Gpiomem,
    pub on_conflict: OnConflict,
}
impl Default for Linux {
    fn default() -> Self {
        Self {
            uio_group: "rp1".into(),
            keep: linux_keep_default(),
            gpiomem: Gpiomem::Disable,
            on_conflict: OnConflict::Disable,
        }
    }
}
fn parse_keep<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<BTreeMap<String, Instance>, D::Error> {
    let values = BTreeMap::<String, toml::Value>::deserialize(d)?;
    values
        .into_iter()
        .map(|(name, value)| {
            let instance: Result<Instance> = Instance::parse(&name, value, "linux.keep");
            instance
                .map(|i| (name, i))
                .map_err(serde::de::Error::custom)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metadata::{BOARD_LINUX_RESETS, peripheral_id, peripheral_type};

    #[test]
    fn every_instance_has_a_label_a_reset_bit_and_a_parse_path() {
        for i in INSTANCES {
            assert_eq!(i.label, format!("rp1_{}", i.name));
            assert!(i.reset_mask.is_power_of_two(), "{}", i.name);
            assert_eq!(
                BOARD_LINUX_RESETS[i.reset_bank] & i.reset_mask,
                i.reset_mask,
                "{}",
                i.name
            );
            let instance = Instance::parse(i.name, toml::Table::new().into(), "proc0").unwrap();
            assert_eq!(
                std::mem::discriminant(&instance),
                std::mem::discriminant(&match i.kind {
                    Kind::Uart => Instance::Uart(UartPins::default()),
                    Kind::I2c => Instance::I2c(Default::default()),
                    Kind::Spi => Instance::Spi(Default::default()),
                    Kind::Adc => Instance::Adc {},
                })
            );
            let block = i.name.to_ascii_uppercase();
            peripheral_type(&block).unwrap();
            peripheral_id(&block).unwrap();
            assert!(undeclarable(i.name).is_none());
        }
    }
}
