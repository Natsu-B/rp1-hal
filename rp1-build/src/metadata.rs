use crate::{
    Result,
    board::{BoardInstance, Kind},
};
use rp1_proto::note::{ShmArea, ShmKind, ShmRegion, Writer, ownership_kind};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet},
};

#[derive(Deserialize)]
#[serde(default, deny_unknown_fields)]
struct InputDeclaration {
    schema: Option<u32>,
    link_wake: Option<LinkWakeProfile>,
    board: String,
    clock: Clock,
    log: Option<Log>,
    stack: Stack,
    link_page: String,
    proc0: BTreeMap<String, toml::Value>,
    proc1: BTreeMap<String, toml::Value>,
    shm: Vec<Shm>,
    linux: Linux,
}
impl Default for InputDeclaration {
    fn default() -> Self {
        Self {
            schema: None,
            link_wake: None,
            board: "rpi-cm5".into(),
            clock: Clock::default(),
            log: None,
            stack: Stack::default(),
            link_page: "16KiB".into(),
            proc0: BTreeMap::new(),
            proc1: BTreeMap::new(),
            shm: Vec::new(),
            linux: Linux::default(),
        }
    }
}
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LinkWakeProfile {
    Experimental,
}

/// Normalized schema 2 declaration.
#[derive(Clone, Debug, Serialize)]
pub struct Declaration {
    pub schema: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link_wake: Option<LinkWakeProfile>,
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
            schema: 2,
            link_wake: None,
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
impl<'de> Deserialize<'de> for Declaration {
    fn deserialize<D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> std::result::Result<Self, D::Error> {
        let input = InputDeclaration::deserialize(deserializer)?;
        Self::from_input(input).map_err(serde::de::Error::custom)
    }
}
impl Declaration {
    pub(crate) fn owned_instances(&self) -> impl Iterator<Item = &'static BoardInstance> + '_ {
        crate::board::INSTANCES.iter().filter(|i| {
            self.proc0.instances.contains_key(i.name)
                || self.proc1.instances.contains_key(i.name)
                || (self.log.is_some() && i.name == "uart0")
        })
    }

    fn from_input(input: InputDeclaration) -> Result<Self> {
        if input.schema != Some(2) {
            return Err("E0: manifest requires schema = 2".into());
        }
        Ok(Self {
            schema: 2,
            link_wake: input.link_wake,
            board: input.board,
            clock: input.clock,
            log: input.log,
            stack: input.stack,
            link_page: input.link_page,
            proc0: Core::parse(input.proc0, "proc0")?,
            proc1: Core::parse(input.proc1, "proc1")?,
            shm: input.shm,
            linux: input.linux,
            types: BTreeMap::new(),
        })
    }
    pub(crate) fn wake_config(&self) -> Option<rp1_proto::note::LinkWake> {
        self.link_wake.map(|_| rp1_proto::note::LinkWake {
            required_caps: rp1_proto::wake::LATCHED_WAKE,
            offered_caps: rp1_proto::wake::LATCHED_WAKE,
            owner: u8::from(self.proc1.host_irq),
            startup_doorbell: None,
        })
    }

    /// Warnings emitted by build.rs; also available to manifest inspection tools.
    pub fn warnings(&self) -> Vec<&'static str> {
        let mut warnings = Vec::new();
        if !self.linux.kept().contains_key("uart1") {
            warnings.push("linux.keep omits uart1: GPIO 0/1 are available to RP1 on rpi-cm5");
        }
        warnings
    }
    /// SHA-256 of the declaration, recorded in the note's FIRMWARE record.
    pub fn declaration_hash(&self) -> Result<[u8; 32]> {
        Ok(Sha256::digest(toml::to_string(self)?.as_bytes()).into())
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
/// Default stack of each core for the user's code, in bytes. A declared `stack.proc0` or
/// `stack.proc1` replaces it.
pub const DEFAULT_STACK: usize = 2048;
/// Stack that rp1-build always adds to proc0's user stack, declared or default, for the platform
/// handlers (IRQ57, IRQ53, HardFault), which run on proc0's MSP on top of the user's code:
/// `roundup(S_plat, 256)`, with `S_plat = max(H57, Hfault) + H53 + Hhard + 3 * 36` (36 B per
/// exception frame). The linker script gets it as `__stack_margin0`, and `stack-paths` fails an
/// image whose `S_plat` exceeds that symbol.
pub const PLATFORM_STACK_MARGIN: usize = 768;
/// proc0's default stack: [`DEFAULT_STACK`] plus [`PLATFORM_STACK_MARGIN`].
pub const DEFAULT_STACK_PROC0: usize = DEFAULT_STACK + PLATFORM_STACK_MARGIN;
const _: () =
    assert!(PLATFORM_STACK_MARGIN.is_multiple_of(256) && DEFAULT_STACK_PROC0.is_multiple_of(8));

impl Default for Stack {
    fn default() -> Self {
        Self {
            proc0: DEFAULT_STACK.to_string(),
            proc1: DEFAULT_STACK.to_string(),
        }
    }
}
pub use crate::board::{Gpiomem, Linux, OnConflict};
pub use crate::declaration::{
    Core, DmaChannels, DmaPool, Heartbeat, I2cPins, Instance, PwmPins, SpiPins, UartPins,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Shm {
    pub name: String,
    pub kind: String,
    pub core: String,
    pub message: Option<String>,
    pub reply: Option<String>,
    #[serde(rename = "type")]
    pub block_type: Option<String>,
    pub direction: Option<String>,
    pub depth: Option<Directional<u32>>,
    pub notify: Option<Directional<String>>,
    pub overflow: Option<String>,
    pub placement: Option<String>,
    pub writer: Option<String>,
    pub linux_map: Option<String>,
}
/// A setting shared by both directions, or set separately for each ring.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Directional<T> {
    Shared(T),
    PerDirection(Directions<T>),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Directions<T> {
    pub dn: T,
    pub up: T,
}

impl<T> Directional<T> {
    fn shared(&self) -> Option<&T> {
        match self {
            Self::Shared(value) => Some(value),
            Self::PerDirection(_) => None,
        }
    }

    fn pair(&self) -> (&T, &T) {
        match self {
            Self::Shared(value) => (value, value),
            Self::PerDirection(d) => (&d.dn, &d.up),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Pair {
    Channel,
    Rpc,
}

const SHM_NAME_MAX: usize = 24;
const PAIR_NAME_MAX: usize = SHM_NAME_MAX - "-dn".len();

impl Shm {
    fn pair(&self) -> Option<Pair> {
        match self.kind.as_str() {
            "channel" => Some(Pair::Channel),
            "rpc" => Some(Pair::Rpc),
            _ => None,
        }
    }

    fn expand(&self) -> Result<Vec<Cow<'_, Shm>>> {
        let Some(pair) = self.pair() else {
            if self.reply.is_some()
                || self.depth.as_ref().is_some_and(|d| d.shared().is_none())
                || self.notify.as_ref().is_some_and(|n| n.shared().is_none())
            {
                return Err("reply and per-direction settings require a channel or rpc".into());
            }
            return Ok(vec![Cow::Borrowed(self)]);
        };
        if self.direction.is_some()
            || self.overflow.is_some()
            || self.writer.is_some()
            || self.block_type.is_some()
            || self.placement.is_some()
        {
            return Err(
                "channel and rpc forbid direction, overflow, writer, type and placement".into(),
            );
        }
        let message = self
            .message
            .as_ref()
            .ok_or("channel and rpc require message")?;
        let reply = self.reply.as_ref().ok_or("channel and rpc require reply")?;
        let depth = self.depth.as_ref().ok_or("channel and rpc require depth")?;
        if pair == Pair::Rpc && depth.shared().is_none() {
            return Err("rpc requires a common integer depth".into());
        }
        let (dn, up) = depth.pair();
        let (dn_notify, up_notify) = self.notify.as_ref().map_or((None, None), |n| {
            let (dn, up) = n.pair();
            (Some(dn.clone()), Some(up.clone()))
        });
        let ring = |suffix, direction: &str, message: &String, depth, notify: Option<String>| {
            Cow::Owned(Shm {
                name: format!("{}-{suffix}", self.name),
                kind: "ring".into(),
                direction: Some(direction.into()),
                message: Some(message.clone()),
                reply: None,
                depth: Some(Directional::Shared(depth)),
                notify: notify.map(Directional::Shared),
                ..self.clone()
            })
        };
        Ok(vec![
            ring("dn", "linux-to-rp1", message, *dn, dn_notify),
            ring("up", "rp1-to-linux", reply, *up, up_notify),
        ])
    }

    fn depth(&self) -> Option<u32> {
        self.depth.as_ref().and_then(Directional::shared).copied()
    }

    fn notify(&self) -> Option<&str> {
        self.notify
            .as_ref()
            .and_then(Directional::shared)
            .map(String::as_str)
    }
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
    pub spec: Cow<'a, Shm>,
    pub ty: TypeInfo,
    pub ddr_offset: u32,
    pub ddr_len: u32,
    pub sram_offset: u16,
    pub sram_len: u16,
}
pub(crate) struct ShmLayout<'a> {
    pub area: ShmArea,
    pub regions: Vec<Region<'a>>,
    pub endpoints: Vec<Endpoint<'a>>,
}

pub(crate) struct Endpoint<'a> {
    pub name: &'a str,
    pub first: usize,
    pub pair: Option<Pair>,
}

pub const SCMI_OFFSET: usize = 0x17c0; // Existing scmi_linux.dtb shmem at SRAM + 0xd7c0.
pub const SRAM_ALLOC_START: usize = rp1_proto::status::END;
pub const fn align(n: usize, a: usize) -> usize {
    (n + a - 1) & !(a - 1)
}

/// HAL type of a canonical peripheral name (`UART1`, `I2C1`, `ADC`, `TIMER.ALARM0`), from the
/// board instance table. Non-canonical spellings such as `TIMER.ALARM00` or `UART+1` would
/// name the same block twice and are rejected (review 3 #5).
pub fn peripheral_type(p: &str) -> Result<String> {
    if let Some(n) = p
        .strip_prefix("TIMER.ALARM")
        .and_then(|s| s.parse::<u8>().ok())
        .filter(|n| *n < 4 && p == format!("TIMER.ALARM{n}"))
    {
        return Ok(format!("::rp1_hal::timer::Alarm<{n}>"));
    }
    let name = p.to_ascii_lowercase();
    match crate::board::find_instance(&name).filter(|_| p == name.to_ascii_uppercase()) {
        Some(i) => Ok(match i.kind {
            Kind::Uart => format!("::rp1_hal::uart::Token<{}>", i.number),
            Kind::I2c => format!("::rp1_hal::i2c::Token<{}>", i.number),
            Kind::Spi => format!("::rp1_hal::spi::Token<{}>", i.number),
            Kind::Adc => "::rp1_hal::adc::Token".into(),
            Kind::Pwm => format!("::rp1_hal::pwm::Token<{}>", i.number),
            Kind::Dma => "::rp1_hal::dma::Token".into(),
        }),
        None => Err(crate::board::undeclarable(&name)
            .filter(|_| p == name.to_ascii_uppercase())
            .unwrap_or_else(|| format!("unknown or unsupported peripheral '{p}'"))
            .into()),
    }
}
pub fn peripheral_id(p: &str) -> Result<(u8, u16)> {
    peripheral_type(p)?;
    Ok(if let Some(n) = p.strip_prefix("UART") {
        (ownership_kind::BLOCK, n.parse()?)
    } else if let Some(n) = p.strip_prefix("I2C") {
        (ownership_kind::BLOCK, 32 + n.parse::<u16>()?)
    } else if let Some(n) = p.strip_prefix("SPI") {
        (
            ownership_kind::BLOCK,
            rp1_proto::BlockId::Spi0 as u16 + n.parse::<u16>()?,
        )
    } else if let Some(n) = p.strip_prefix("PWM") {
        (
            ownership_kind::BLOCK,
            rp1_proto::BlockId::Pwm0 as u16 + n.parse::<u16>()?,
        )
    } else if p == "ADC" {
        (ownership_kind::BLOCK, rp1_proto::BlockId::Adc as u16)
    } else if p == "DMA" {
        (ownership_kind::BLOCK, rp1_proto::BlockId::Dma as u16)
    } else {
        (
            ownership_kind::TIMER_ALARM,
            p.strip_prefix("TIMER.ALARM").unwrap().parse()?,
        )
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
    /// [`BOARD_LINUX_RESETS`] without the UARTs, I2Cs, SPIs, PWM and ADC the image declares (and UART0
    /// with `log`): rp1-hal releases these for Linux. Declared ADC/PWM blocks also make rp1-hal hold
    /// their functional clocks (`rp1_hal::linux_blocks::LinuxBlocks::held_clocks`).
    pub fn linux_resets(&self) -> [u32; 3] {
        let mut r = BOARD_LINUX_RESETS;
        for i in self.owned_instances() {
            r[i.reset_bank] &= !i.reset_mask;
        }
        r
    }
    /// The whole stack of `core`: the declared (or default) user stack, plus
    /// [`Self::stack_margin`].
    pub fn stack_size(&self, core: usize) -> Result<usize> {
        let s = if core == 0 {
            &self.stack.proc0
        } else {
            &self.stack.proc1
        };
        let user = if let Some(s) = s.strip_suffix("KiB") {
            s.parse::<usize>()?
                .checked_mul(1024)
                .ok_or("stack overflow")?
        } else {
            s.parse()?
        };
        let n = user
            .checked_add(Self::stack_margin(core))
            .ok_or("stack overflow")?;
        if user == 0 || n > 8192 || user % 8 != 0 {
            return Err(if core == 0 {
                "stack.proc0 must be 8-byte aligned and, with the 768 B platform margin, within 8 KiB"
            } else {
                "stack must be 8-byte aligned and within 8 KiB"
            }
            .into());
        }
        Ok(n)
    }
    /// The platform's part of `core`'s stack: [`PLATFORM_STACK_MARGIN`] on proc0, 0 on proc1.
    pub fn stack_margin(core: usize) -> usize {
        if core == 0 { PLATFORM_STACK_MARGIN } else { 0 }
    }
    pub fn validate(&self) -> Result<()> {
        if self.schema != 2 || self.board != "rpi-cm5" {
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
        self.validate_ownership()?;
        if self.link_wake.is_some() && !(self.proc0.host_irq || self.proc1.host_irq) {
            return Err("experimental link_wake requires one host_irq owner".into());
        }
        let mut names = BTreeSet::new();
        let mut region_count = 0;
        for r in &self.shm {
            let b = r.name.as_bytes();
            let max_len = if r.pair().is_some() {
                PAIR_NAME_MAX
            } else {
                SHM_NAME_MAX
            };
            if b.is_empty()
                || b.len() > max_len
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
            let regions = r.expand()?;
            region_count += regions.len();
            if region_count > 64 {
                return Err("v0.1 supports at most 64 shm regions".into());
            }
            for region in regions {
                self.validate_region(&region)?;
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
    fn validate_region(&self, r: &Shm) -> Result<()> {
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
                    .depth()
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
                match r.notify() {
                    None | Some("poll") => {}
                    // RP1 -> Linux: the producing core raises `rp1:irq`, so it must own the
                    // host interrupt. Linux -> RP1: Linux raises the owning core's doorbell.
                    Some("irq") => {
                        let owner = if r.core == "proc0" {
                            &self.proc0
                        } else {
                            &self.proc1
                        };
                        if r.direction.as_deref() == Some("rp1-to-linux") && !owner.host_irq {
                            return Err(format!(
                                "notify=irq on ring {} requires {}.host_irq = true \
                                     (the producing core raises rp1:irq)",
                                r.name, r.core
                            )
                            .into());
                        }
                        if r.direction.as_deref() == Some("linux-to-rp1") {
                            if owner.doorbell.is_none() {
                                return Err(format!(
                                    "notify=irq on ring {} requires {}.doorbell \
                                         (Linux raises the reading core's doorbell)",
                                    r.name, r.core
                                )
                                .into());
                            }
                            if !owner.gpio_irq.is_empty() && !owner.custom_doorbell_handler {
                                return Err(format!(
                                    "E5: {} shares IO_BANK0 between GPIO and doorbell; set custom_doorbell_handler = true and dispatch both sources",
                                    r.core
                                ).into());
                            }
                        }
                    }
                    Some(_) => return Err("invalid notify (poll or irq)".into()),
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
            _ => return Err("shm kind must be ring, block, channel or rpc".into()),
        }
        Ok(())
    }

    pub fn regions(&self) -> Result<Vec<Region<'_>>> {
        Ok(self.shm_layout()?.regions)
    }

    pub fn ddr_size(&self) -> Result<u32> {
        Ok(self.shm_layout()?.area.size)
    }

    pub(crate) fn shm_layout(&self) -> Result<ShmLayout<'_>> {
        let mut ddr = 65536usize; // Region header is in the first independently mapped page.
        let sram_start = if self.link_wake.is_some() {
            rp1_proto::wake::mailbox::END
        } else {
            SRAM_ALLOC_START
        };
        let mut sram = sram_start;
        let mut regions = Vec::new();
        let mut endpoints = Vec::with_capacity(self.shm.len());
        for logical in &self.shm {
            endpoints.push(Endpoint {
                name: &logical.name,
                first: regions.len(),
                pair: logical.pair(),
            });
            for r in logical.expand()? {
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
                    let body = (r.depth().unwrap() as usize)
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
                regions.push(Region {
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
        }
        let area = ShmArea {
            size: align(ddr, 0x10_0000) as u32,
            handoff: rp1_proto::HANDOFF,
            sram_end: regions
                .last()
                .map_or(sram_start as u16, |r| r.sram_offset + r.sram_len),
            phys: rp1_proto::SHM_PHYS,
        };
        Ok(ShmLayout {
            area,
            regions,
            endpoints,
        })
    }
}

impl Region<'_> {
    pub(crate) fn receives_doorbell(&self, core: u8) -> bool {
        self.core() == core + 1
            && self.spec.direction.as_deref() == Some("linux-to-rp1")
            && self.spec.notify() == Some("irq")
    }

    pub(crate) fn wire(&self) -> Result<ShmRegion<'_>> {
        let spec = &self.spec;
        let kind = match (spec.direction.as_deref(), spec.writer.as_deref()) {
            (Some("rp1-to-linux"), _) => ShmKind::RingToLinux,
            (Some("linux-to-rp1"), _) => ShmKind::RingToRp1,
            (_, Some("rp1")) => ShmKind::Block {
                writer: Writer::Rp1,
            },
            (_, Some("linux")) => ShmKind::Block {
                writer: Writer::Linux,
            },
            (_, Some("both")) => ShmKind::Block {
                writer: Writer::Both,
            },
            _ => return Err("invalid SHM direction or writer".into()),
        };
        Ok(ShmRegion {
            kind,
            core: self.core(),
            notify_irq: spec.notify() == Some("irq"),
            overwrite: spec.overflow.as_deref() == Some("overwrite"),
            ddr_offset: self.ddr_offset,
            ddr_len: self.ddr_len,
            sram_offset: self.sram_offset,
            sram_len: self.sram_len,
            elem_size: self.ty.size as u32,
            depth: spec.depth().unwrap_or(0),
            type_id: self.ty.id,
            name: &spec.name,
            type_name: spec
                .message
                .as_deref()
                .or(spec.block_type.as_deref())
                .unwrap(),
        })
    }

    pub(crate) fn core(&self) -> u8 {
        if self.spec.core == "proc0" { 1 } else { 2 }
    }
}
