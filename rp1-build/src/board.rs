//! STATIC: rpi-cm5 ownership policy, independent of the bank-0 mux table.
use crate::{
    Result,
    metadata::{Instance, UartPins},
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, ops::RangeInclusive};

pub const LINUX_PINS: RangeInclusive<u8> = 28..=53;
pub const LINUX_BLOCKS: &[&str] = &["ETH", "USBHOST0", "USBHOST1", "DMA", "PCIE", "PIO"];

/// STATIC: declarable instance, Linux DT label, reset bank and mask (RP1 RESETS).
/// Held SCMI clocks follow these masks via `LinuxBlocks::held_clocks` (ADC only).
pub(crate) const INSTANCES: &[(&str, &str, usize, u32)] = &[
    ("uart0", "rp1_uart0", 1, 1 << 26),
    ("uart1", "rp1_uart1", 1, 1 << 27),
    ("uart2", "rp1_uart2", 1, 1 << 28),
    ("uart3", "rp1_uart3", 1, 1 << 29),
    ("uart4", "rp1_uart4", 1, 1 << 30),
    ("i2c0", "rp1_i2c0", 0, 1 << 7),
    ("i2c1", "rp1_i2c1", 0, 1 << 8),
    ("i2c2", "rp1_i2c2", 0, 1 << 9),
    ("i2c3", "rp1_i2c3", 0, 1 << 10),
    ("spi0", "rp1_spi0", 1, 1 << 10),
    ("spi1", "rp1_spi1", 1, 1 << 11),
    ("spi3", "rp1_spi3", 1, 1 << 13),
    ("spi5", "rp1_spi5", 1, 1 << 15),
    ("adc", "rp1_adc", 0, 1),
];

pub fn linux_line(pin: u8) -> Option<&'static str> {
    if !LINUX_PINS.contains(&pin) {
        return None;
    }
    Some(match pin {
        32 => "ETH_RST_N",
        37 => "ETH_IRQ_N",
        42 | 43 => "VBUS",
        44 => "PWR_LED",
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
