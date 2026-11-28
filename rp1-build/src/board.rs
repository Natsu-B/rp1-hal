//! STATIC: rpi-cm5 ownership policy, independent of the bank-0 mux table.
use crate::{
    Result,
    metadata::{Instance, UartPins},
};
use rp1_proto::{BlockId, note::ownership_kind};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, ops::RangeInclusive};

pub const LINUX_PINS: RangeInclusive<u8> = 28..=53;
/// RP1 blocks Linux needs (shared with the loader's protected-node list).
pub use rp1_proto::board::LINUX_BLOCKS;

/// Block family of a declarable instance; selects its pin table and HAL token.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    Uart,
    I2c,
    Spi,
    Adc,
    Pwm,
    Dma,
    Pio,
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
/// held SCMI clocks follow the reset masks via `LinuxBlocks::held_clocks` (ADC, PWM and DMA).
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
    instance("spi2", Kind::Spi, 2, "rp1_spi2", 1, 12),
    instance("spi3", Kind::Spi, 3, "rp1_spi3", 1, 13),
    instance("spi5", Kind::Spi, 5, "rp1_spi5", 1, 15),
    instance("adc", Kind::Adc, 0, "rp1_adc", 0, 0),
    instance("pwm0", Kind::Pwm, 0, "rp1_pwm0", 1, 4),
    instance("pwm1", Kind::Pwm, 1, "rp1_pwm1", 1, 5),
    // RESETS CTRL0 bit 3 (inferred from the bit order, rp1_hal::linux_blocks::ALLOWED).
    instance("dma", Kind::Dma, 0, "rp1_dma", 0, 3),
    // PIO_APBS stays outside BOARD_LINUX_RESETS. Only the PIO owner controls reset.
    instance("pio", Kind::Pio, 0, "rp1_pio", 0, 27),
];

/// The declaration resource that grants an IRQ line.
#[derive(Clone, Copy)]
pub enum IrqOwner {
    GpioBank0,
    Alarm(u8),
    Instance(BlockId),
}

impl IrqOwner {
    pub(crate) fn source_type(self) -> String {
        match self {
            Self::GpioBank0 => "Bank0Irq".into(),
            Self::Alarm(n) => format!("AlarmIrq<{n}>"),
            Self::Instance(id) => format!("BlockIrq<{}>", id as u16),
        }
    }

    /// OWNERSHIP kind and id used to check a handler's claim.
    pub fn ownership(self) -> (u8, u16) {
        match self {
            Self::GpioBank0 => (ownership_kind::GPIO_IRQ_BANK, 0),
            Self::Alarm(alarm) => (ownership_kind::TIMER_ALARM, u16::from(alarm)),
            Self::Instance(block) => (ownership_kind::BLOCK, block as u16),
        }
    }
}

/// An IRQ route and the resource required to own it.
pub struct VerifiedIrq {
    pub name: &'static str,
    pub irq: u8,
    pub owner: IrqOwner,
}

/// HW: IRQ lines seen pending in the NVIC from their block. PWM0 and TIMER_3
/// were seen on the earlier proof-of-concept firmware. IO_BANK1/2 are routed
/// but their pins are Linux's.
pub const IRQ_VERIFIED: &[VerifiedIrq] = &[
    VerifiedIrq {
        name: "IO_BANK0",
        irq: 0,
        owner: IrqOwner::GpioBank0,
    },
    VerifiedIrq {
        name: "PWM0",
        irq: 5,
        owner: IrqOwner::Instance(BlockId::Pwm0),
    },
    VerifiedIrq {
        name: "I2C0",
        irq: 7,
        owner: IrqOwner::Instance(BlockId::I2c0),
    },
    VerifiedIrq {
        name: "I2C1",
        irq: 8,
        owner: IrqOwner::Instance(BlockId::I2c1),
    },
    VerifiedIrq {
        name: "I2C2",
        irq: 9,
        owner: IrqOwner::Instance(BlockId::I2c2),
    },
    VerifiedIrq {
        name: "I2C3",
        irq: 10,
        owner: IrqOwner::Instance(BlockId::I2c3),
    },
    VerifiedIrq {
        name: "SPI0",
        irq: 19,
        owner: IrqOwner::Instance(BlockId::Spi0),
    },
    VerifiedIrq {
        name: "SPI1",
        irq: 20,
        owner: IrqOwner::Instance(BlockId::Spi1),
    },
    VerifiedIrq {
        name: "SPI3",
        irq: 22,
        owner: IrqOwner::Instance(BlockId::Spi3),
    },
    VerifiedIrq {
        name: "UART0",
        irq: 25,
        owner: IrqOwner::Instance(BlockId::Uart0),
    },
    VerifiedIrq {
        name: "TIMER_0",
        irq: 26,
        owner: IrqOwner::Alarm(0),
    },
    VerifiedIrq {
        name: "TIMER_1",
        irq: 27,
        owner: IrqOwner::Alarm(1),
    },
    VerifiedIrq {
        name: "TIMER_2",
        irq: 28,
        owner: IrqOwner::Alarm(2),
    },
    VerifiedIrq {
        name: "TIMER_3",
        irq: 29,
        owner: IrqOwner::Alarm(3),
    },
    VerifiedIrq {
        name: "PWM1",
        irq: 41,
        owner: IrqOwner::Instance(BlockId::Pwm1),
    },
    VerifiedIrq {
        name: "UART1",
        irq: 42,
        owner: IrqOwner::Instance(BlockId::Uart1),
    },
    VerifiedIrq {
        name: "UART2",
        irq: 43,
        owner: IrqOwner::Instance(BlockId::Uart2),
    },
    VerifiedIrq {
        name: "UART3",
        irq: 44,
        owner: IrqOwner::Instance(BlockId::Uart3),
    },
    VerifiedIrq {
        name: "UART4",
        irq: 45,
        owner: IrqOwner::Instance(BlockId::Uart4),
    },
    // Gate C drives real FIFO sources on both lines, with proc1 as the owner.
    VerifiedIrq {
        name: "PIO_0",
        irq: rp1_proto::board::PIO_IRQ0 as u8,
        owner: IrqOwner::Instance(BlockId::Pio),
    },
    VerifiedIrq {
        name: "PIO_1",
        irq: rp1_proto::board::PIO_IRQ1 as u8,
        owner: IrqOwner::Instance(BlockId::Pio),
    },
    VerifiedIrq {
        name: "ADC_FIFO",
        irq: 52,
        owner: IrqOwner::Instance(BlockId::Adc),
    },
];

/// Hardware-observed IRQ routes supported by the declaration generator.
pub fn irq_routes() -> impl Iterator<Item = &'static VerifiedIrq> {
    IRQ_VERIFIED.iter()
}

/// Routes that may be bound by an application on this core.
pub fn irq_routes_for(core: usize) -> impl Iterator<Item = &'static VerifiedIrq> {
    irq_routes().filter(move |line| line.supports_core(core))
}

impl VerifiedIrq {
    pub(crate) fn supports_core(&self, core: usize) -> bool {
        core == 1 || !matches!(self.owner, IrqOwner::Instance(BlockId::Pio))
    }
}

pub(crate) fn find_instance(name: &str) -> Option<&'static BoardInstance> {
    INSTANCES.iter().find(|i| i.name == name)
}

/// Why a known RP1 block key cannot be declared on rpi-cm5, if it is one.
pub(crate) fn undeclarable(name: &str) -> Option<String> {
    let block = name.to_ascii_uppercase();
    Some(match name {
        _ if name != "pio" && LINUX_BLOCKS.iter().any(|(b, _)| *b == block) => {
            format!("E4: {block} is required by Linux on rpi-cm5")
        }
        "spi4" | "spi7" => format!("E4: {block} is slave-only"),
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
        34 => "CAM_GPIO",
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
    /// `None` (omitted): the board default, [`linux_keep_default`]. An explicit table
    /// replaces it; `{}` releases UART1.
    #[serde(
        deserialize_with = "parse_keep",
        skip_serializing_if = "Option::is_none"
    )]
    pub keep: Option<BTreeMap<String, Instance>>,
    pub gpiomem: Gpiomem,
    pub on_conflict: OnConflict,
    /// Optional Linux DMA consumers which the loader may disable, by absolute DT node path.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub dma_disable: Vec<String>,
}
impl Default for Linux {
    fn default() -> Self {
        Self {
            uio_group: "rp1".into(),
            keep: None,
            gpiomem: Gpiomem::Disable,
            on_conflict: OnConflict::Disable,
            dma_disable: Vec::new(),
        }
    }
}
impl Linux {
    /// The instances kept for Linux: the explicit `keep`, or the board default.
    pub fn kept(&self) -> BTreeMap<String, Instance> {
        self.keep.clone().unwrap_or_else(linux_keep_default)
    }
}
fn parse_keep<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<Option<BTreeMap<String, Instance>>, D::Error> {
    let values = BTreeMap::<String, toml::Value>::deserialize(d)?;
    values
        .into_iter()
        .map(|(name, value)| {
            let instance: Result<Instance> = Instance::parse(&name, value, "linux.keep");
            instance
                .map(|i| (name, i))
                .map_err(serde::de::Error::custom)
        })
        .collect::<std::result::Result<_, _>>()
        .map(Some)
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
            let table = if i.kind == Kind::Pio {
                assert_eq!(BOARD_LINUX_RESETS[i.reset_bank] & i.reset_mask, 0);
                toml::from_str("sms = [0]").unwrap()
            } else {
                assert_eq!(
                    BOARD_LINUX_RESETS[i.reset_bank] & i.reset_mask,
                    i.reset_mask,
                    "{}",
                    i.name
                );
                toml::Table::new()
            };
            let instance = Instance::parse(i.name, table.into(), "proc0").unwrap();
            assert_eq!(
                std::mem::discriminant(&instance),
                std::mem::discriminant(&match i.kind {
                    Kind::Uart => Instance::Uart(UartPins::default()),
                    Kind::I2c => Instance::I2c(Default::default()),
                    Kind::Spi => Instance::Spi(Default::default()),
                    Kind::Adc => Instance::Adc {},
                    Kind::Pwm => Instance::Pwm(Default::default()),
                    Kind::Dma => Instance::Dma(Default::default()),
                    Kind::Pio => Instance::Pio(Default::default()),
                })
            );
            let block = i.name.to_ascii_uppercase();
            peripheral_type(&block).unwrap();
            peripheral_id(&block).unwrap();
            assert!(undeclarable(i.name).is_none());
        }
    }
}
