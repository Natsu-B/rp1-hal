//! STATIC: rpi-cm5 ownership policy, independent of the bank-0 mux table.
use crate::{
    Result,
    metadata::{Instance, UartPins},
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, ops::RangeInclusive};

pub const LINUX_PINS: RangeInclusive<u8> = 28..=53;
pub const LINUX_BLOCKS: &[&str] = &["ETH", "USBHOST0", "USBHOST1", "DMA", "PCIE", "PIO"];

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
