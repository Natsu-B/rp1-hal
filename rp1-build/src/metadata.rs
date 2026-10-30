use crate::Result;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Declaration {
    pub schema: u32,
    pub board: String,
    pub clock: Clock,
    pub log: Option<Log>,
    pub stack: Stack,
    pub link_page: String,
    pub proc0: Core,
    pub proc1: Core,
    pub shm: Vec<Shm>,
    pub linux: Linux,
    #[serde(skip)]
    pub types: BTreeMap<String, TypeInfo>,
}
impl Default for Declaration {
    fn default() -> Self {
        Self {
            schema: 1,
            board: "rpi-cm5".into(),
            clock: Clock::default(),
            log: None,
            stack: Stack::default(),
            link_page: "16KiB".into(),
            proc0: Core::default(),
            proc1: Core::default(),
            shm: Vec::new(),
            linux: Linux::default(),
            types: BTreeMap::new(),
        }
    }
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Clock {
    pub authority: String,
    pub clk_sys_hz: Option<u32>,
}
impl Default for Clock {
    fn default() -> Self {
        Self {
            authority: "rp1".into(),
            clk_sys_hz: None,
        }
    }
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Log {
    pub sink: String,
    pub baudrate: u32,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Stack {
    pub proc0: String,
    pub proc1: String,
}
impl Default for Stack {
    fn default() -> Self {
        Self {
            proc0: "2KiB".into(),
            proc1: "2KiB".into(),
        }
    }
}
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Core {
    pub pins: Vec<u8>,
    pub peripherals: Vec<String>,
    pub gpio_irq: Vec<u8>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Linux {
    pub uio_group: String,
}
impl Default for Linux {
    fn default() -> Self {
        Self {
            uio_group: "rp1".into(),
        }
    }
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Shm {
    pub name: String,
    pub kind: String,
    pub core: String,
    pub message: Option<String>,
    #[serde(rename = "type")]
    pub block_type: Option<String>,
    pub direction: Option<String>,
    pub depth: Option<u32>,
    pub notify: Option<String>,
    pub overflow: Option<String>,
    pub placement: Option<String>,
    pub writer: Option<String>,
    pub linux_map: Option<String>,
}
#[derive(Clone, Copy, Debug)]
pub struct TypeInfo {
    pub size: usize,
    pub id: u64,
}
impl TypeInfo {
    pub const fn of<T: rp1_proto::Message>() -> Self {
        Self {
            size: std::mem::size_of::<T>(),
            id: T::TYPE_ID,
        }
    }
}
pub struct Region<'a> {
    pub spec: &'a Shm,
    pub ty: TypeInfo,
    pub ddr_offset: u32,
    pub ddr_len: u32,
    pub sram_offset: u16,
    pub sram_len: u16,
}
pub const SCMI_OFFSET: usize = 0x17c0; // Existing scmi_linux.dtb shmem at SRAM + 0xd7c0.
pub const SRAM_ALLOC_START: usize = 64;
pub const fn align(n: usize, a: usize) -> usize {
    (n + a - 1) & !(a - 1)
}

/// Only canonical names (`UART1`, `I2C1`, `TIMER.ALARM0`) are accepted: `TIMER.ALARM00` or
/// `UART+1` would otherwise name the same block under a second string and pass the duplicate
/// check (review 3 #5).
pub fn peripheral_type(p: &str) -> Result<String> {
    if let Some(n) = p
        .strip_prefix("UART")
        .and_then(|s| s.parse::<u8>().ok())
        .filter(|n| *n < 6 && p == format!("UART{n}"))
    {
        Ok(format!("::rp1_hal::uart::Token<{n}>"))
    } else if let Some(n) = p
        .strip_prefix("I2C")
        .and_then(|s| s.parse::<u8>().ok())
        .filter(|n| *n < 4 && p == format!("I2C{n}"))
    {
        Ok(format!("::rp1_hal::i2c::Token<{n}>"))
    } else if let Some(n) = p
        .strip_prefix("SPI")
        .and_then(|s| s.parse::<u8>().ok())
        .filter(|n| p == format!("SPI{n}"))
    {
        // SPI4/7 are slave-only, SPI6/8 are not on bank 0, SPI2's MISO is GPIO1 (Linux UART1
        // on rpi-cm5): only SPI0/1/3/5 can be used here.
        match n {
            0 | 1 | 3 | 5 => Ok(format!("::rp1_hal::spi::Token<{n}>")),
            _ => Err(format!("{p} is not a usable SPI master on rpi-cm5 (SPI0/1/3/5)").into()),
        }
    } else if p == "ADC" {
        Ok("::rp1_hal::adc::Token".into())
    } else if let Some(n) = p
        .strip_prefix("TIMER.ALARM")
        .and_then(|s| s.parse::<u8>().ok())
        .filter(|n| *n < 4 && p == format!("TIMER.ALARM{n}"))
    {
        Ok(format!("::rp1_hal::timer::Alarm<{n}>"))
    } else {
        Err(format!("unknown or v0.1 unsupported peripheral '{p}'").into())
    }
}
pub fn peripheral_id(p: &str) -> Result<(u8, u16)> {
    peripheral_type(p)?;
    Ok(if let Some(n) = p.strip_prefix("UART") {
        (2, n.parse()?)
    } else if let Some(n) = p.strip_prefix("I2C") {
        (2, 32 + n.parse::<u16>()?)
    } else if let Some(n) = p.strip_prefix("SPI") {
        (2, rp1_proto::BlockId::Spi0 as u16 + n.parse::<u16>()?)
    } else if p == "ADC" {
        (2, rp1_proto::BlockId::Adc as u16)
    } else {
        (3, p.strip_prefix("TIMER.ALARM").unwrap().parse()?)
    })
}
/// rpi-cm5: RESETS bits (bank 0, 1, 2) of the RP1 blocks Linux owns unless the image declares
/// them (rp1_hal::linux_blocks::ALLOWED): ADC, AUDIO_OUT, DMA, DMA_TICK, ETH_IP, I2C0..6,
/// I2S0..2, PADS_ETH | PWM0/1, SPI0..8, SYS_RIO1/2, UART0..5 | USBHOST0/1, VBUSCTRL.
pub const BOARD_LINUX_RESETS: [u32; 3] = [
    1 | (1 << 2) | (1 << 3) | (1 << 4) | (1 << 5) | (0x7f << 7) | (0x7 << 14) | (1 << 25),
    (0x3 << 4) | (0x1ff << 10) | (0x3 << 20) | (0x3f << 26),
    0x7,
];
impl Declaration {
    /// [`BOARD_LINUX_RESETS`] without the UARTs, I2Cs, SPIs and ADC the image declares (and UART0
    /// with `log`): rp1-hal releases these for Linux. A declared ADC also makes rp1-hal hold
    /// clk_adc (`rp1_hal::linux_blocks::LinuxBlocks::held_clocks`).
    pub fn linux_resets(&self) -> [u32; 3] {
        let mut r = BOARD_LINUX_RESETS;
        for p in self.proc0.peripherals.iter().chain(&self.proc1.peripherals) {
            if let Some(n) = p.strip_prefix("UART").and_then(|n| n.parse::<u32>().ok()) {
                r[1] &= !(1 << (26 + n));
            } else if let Some(n) = p.strip_prefix("I2C").and_then(|n| n.parse::<u32>().ok()) {
                r[0] &= !(1 << (7 + n));
            } else if let Some(n) = p.strip_prefix("SPI").and_then(|n| n.parse::<u32>().ok()) {
                r[1] &= !(1 << (10 + n));
            } else if p == "ADC" {
                r[0] &= !1;
            }
        }
        if self.log.is_some() {
            r[1] &= !(1 << 26);
        }
        r
    }
    pub fn stack_size(&self, core: usize) -> Result<usize> {
        let s = if core == 0 {
            &self.stack.proc0
        } else {
            &self.stack.proc1
        };
        let n = if let Some(s) = s.strip_suffix("KiB") {
            s.parse::<usize>()?
                .checked_mul(1024)
                .ok_or("stack overflow")?
        } else {
            s.parse()?
        };
        if n == 0 || n > 8192 || n % 8 != 0 {
            return Err("stack must be 8-byte aligned and within 8 KiB".into());
        }
        Ok(n)
    }
    pub fn validate(&self) -> Result<()> {
        if self.schema != 1 || self.board != "rpi-cm5" {
            return Err("unsupported schema or board".into());
        }
        if self.link_page != "16KiB" {
            return Err("v0.1 requires link_page = 16KiB".into());
        }
        if !matches!(self.clock.authority.as_str(), "rp1" | "linux") {
            return Err("invalid clock authority".into());
        }
        // The rpi-cm5 board DTB routes RP1 clocks through SCMI (clk-rp1 disabled); the overlay
        // cannot hand them back to Linux, so nothing would answer its SCMI requests (review #7).
        if self.clock.authority == "linux" {
            return Err(
                "clock.authority = linux is unsupported on the v0.1 rpi-cm5 board; use rp1".into(),
            );
        }
        if self.clock.authority == "linux" && self.clock.clk_sys_hz.is_some() {
            return Err("clk_sys_hz requires clock.authority = rp1".into());
        }
        if self.clock.clk_sys_hz.is_some_and(|h| h != 200_000_000) {
            return Err("v0.1 supports only 200 MHz clk_sys".into());
        }
        self.stack_size(0)?;
        self.stack_size(1)?;
        let mut pins = BTreeSet::new();
        let mut peripherals = BTreeSet::new();
        if let Some(l) = &self.log {
            if l.sink != "uart0" || l.baudrate == 0 {
                return Err("invalid UART0 log declaration".into());
            }
            pins.extend([14, 15]);
            peripherals.insert(peripheral_id("UART0")?);
        }
        for c in [&self.proc0, &self.proc1] {
            for p in &c.pins {
                if *p >= 28 || !pins.insert(*p) {
                    return Err(format!("GPIO {p} unsupported or claimed twice").into());
                }
                // Linux UART1 retains 0/1; 2/3 may be granted to the same core's I2C1.
                if [0, 1].contains(p)
                    || ([2, 3].contains(p) && !c.peripherals.iter().any(|p| p == "I2C1"))
                {
                    return Err(format!("GPIO {p} is reserved by rpi-cm5 board policy").into());
                }
            }
            for p in &c.gpio_irq {
                if !c.pins.contains(p) {
                    return Err(format!("gpio_irq {p} is not in pins").into());
                }
            }
            for p in &c.peripherals {
                // Duplicates by hardware identity, not by spelling.
                if !peripherals.insert(peripheral_id(p)?) {
                    return Err(format!("{p} is claimed twice").into());
                }
            }
        }
        if self.clock.authority == "linux" && peripherals.iter().any(|&(k, n)| k == 2 && n < 32) {
            return Err("UARTs share clk_uart with Linux; set clock.authority = rp1".into());
        }
        let mut names = BTreeSet::new();
        if self.shm.len() > 64 {
            return Err("v0.1 supports at most 64 shm regions".into());
        }
        for r in &self.shm {
            let b = r.name.as_bytes();
            if b.is_empty()
                || b.len() > 24
                || !b[0].is_ascii_lowercase()
                || !b
                    .iter()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == b'_')
                || !names.insert(&r.name)
            {
                return Err("invalid or duplicate shm name".into());
            }
            // The overlay always creates these UIO nodes; a region of the same name would add a
            // second `rp1:<name>` node and Linux-side init would fail on the duplicate.
            if rp1_proto::uio::BUILTIN.contains(&r.name.as_str()) {
                return Err(
                    format!("shm name {} is reserved for a built-in UIO node", r.name).into(),
                );
            }
            if !matches!(r.core.as_str(), "proc0" | "proc1") {
                return Err("invalid shm core".into());
            }
            if r.linux_map.as_deref().is_some_and(|m| m != "uncached") {
                return Err("cached mmap is not supported in v0.1".into());
            }
            match r.kind.as_str() {
                "ring" => {
                    if r.message.is_none()
                        || r.block_type.is_some()
                        || r.writer.is_some()
                        || !matches!(
                            r.direction.as_deref(),
                            Some("rp1-to-linux" | "linux-to-rp1")
                        )
                    {
                        return Err("ring requires message and direction; no type or writer".into());
                    }
                    if !r
                        .depth
                        .is_some_and(|d| d.is_power_of_two() && d <= (1 << 30))
                    {
                        return Err("ring depth must be a power of two <= 2^30".into());
                    }
                    if r.overflow
                        .as_deref()
                        .is_some_and(|s| !matches!(s, "reject" | "overwrite"))
                    {
                        return Err("invalid overflow".into());
                    }
                    if r.placement.as_deref().is_some_and(|s| s != "reader-local") {
                        return Err("placement=linux is unimplemented".into());
                    }
                    if r.notify.as_deref().is_some_and(|s| s != "poll") {
                        return Err("v0.1 supports only notify=poll (SCMI owns hwirq58)".into());
                    }
                }
                "block" => {
                    if r.block_type.is_none()
                        || r.message.is_some()
                        || r.direction.is_some()
                        || r.depth.is_some()
                        || r.notify.is_some()
                        || r.overflow.is_some()
                        || r.placement.is_some()
                        || !matches!(r.writer.as_deref(), Some("rp1" | "linux" | "both"))
                    {
                        return Err(
                            "block requires type and writer; ring-only fields are forbidden".into(),
                        );
                    }
                }
                _ => return Err("v0.1 supports ring and block; use two rings for a channel".into()),
            }
        }
        if self.linux.uio_group.is_empty()
            || !self
                .linux
                .uio_group
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
        {
            return Err("invalid UIO group".into());
        }
        Ok(())
    }
    pub fn regions(&self) -> Result<Vec<Region<'_>>> {
        let mut ddr = 65536usize; // Region header is in the first independently mapped page.
        let mut sram = SRAM_ALLOC_START;
        let mut result = Vec::new();
        for r in &self.shm {
            let name = r.message.as_ref().or(r.block_type.as_ref()).unwrap();
            let ty=*self.types.get(name).ok_or_else(||format!("type {name} missing: use configure_with_types and register TypeInfo::of::<T>() in build.rs"))?;
            if ty.size == 0
                || ty.size > rp1_proto::SHM_MAX as usize - 8
                || (r.kind == "ring" && ty.size > 65535)
            {
                return Err("invalid message size (ring <= 65535, block < 16 MiB)".into());
            }
            let slot = 8usize
                .checked_add(align(ty.size, 8))
                .ok_or("slot overflow")?;
            let (ddr_len, sram_len) = if r.kind == "block" {
                (slot, 0)
            } else {
                let body = (r.depth.unwrap() as usize)
                    .checked_mul(slot)
                    .and_then(|n| n.checked_add(128))
                    .ok_or("ring size overflow")?;
                if r.direction.as_deref() == Some("rp1-to-linux") {
                    (body, 128)
                } else {
                    (128, body)
                }
            };
            sram = align(sram, 64);
            let end = sram.checked_add(sram_len).ok_or("SRAM size overflow")?;
            if sram < SCMI_OFFSET + 256 && end > SCMI_OFFSET {
                sram = align(SCMI_OFFSET + 256, 64);
            }
            if sram
                .checked_add(sram_len)
                .is_none_or(|end| end > rp1_proto::LINK_LEN)
            {
                return Err(
                    "link page SRAM budget exceeded (SCMI and ABI reservation excluded)".into(),
                );
            }
            let ddr_len = align(ddr_len, 65536);
            result.push(Region {
                spec: r,
                ty,
                ddr_offset: u32::try_from(ddr)?,
                ddr_len: u32::try_from(ddr_len)?,
                sram_offset: u16::try_from(sram)?,
                sram_len: u16::try_from(sram_len)?,
            });
            ddr = ddr.checked_add(ddr_len).ok_or("DDR overflow")?;
            if ddr > 0x0100_0000 {
                return Err("DDR exceeds rpi-cm5 reserved window (16 MiB)".into());
            }
            sram += sram_len;
        }
        Ok(result)
    }
    pub fn ddr_size(&self) -> Result<u32> {
        let regions = self.regions()?;
        let end = regions.last().map_or(65536, |r| r.ddr_offset + r.ddr_len);
        Ok(align(end as usize, 0x10_0000) as u32)
    }
}
