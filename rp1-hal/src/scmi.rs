//! SCMI 2.0 server (Base + Clock protocols) for `clock.authority = "rp1"`.
//!
//! Linux sees the RP1 clocks through `arm,scmi` instead of clk-rp1. Transport: rp1-mailbox
//! channel 0 + an `arm,scmi-shmem` area at shared SRAM `0x2000_d7c0` (256 bytes, kept out of the link page by rp1-build). Same message layout as the
//! HW-proven PoC server (PoC HAL `Natsu-B/RP1-hal` `crates/rp1-hal/src/scmi_*.rs`, HW 2026-10-22).
//!
//! The table ([`RP1_CLOCKS`]) has two kinds of clock:
//!
//! - [`Kind::Fixed`]: a clock RP1 itself runs on (clk_sys, clk_slow_sys, clk_uart, the UART APB
//!   clock). The rate is fixed and the clock stays on: `CLOCK_RATE_SET` to another rate is
//!   `DENIED`, `CLOCK_CONFIG_SET` only records Linux's vote (SCMI shared-clock semantics). Nothing
//!   writes these registers; reads report the physical state and fail closed if it drifts.
//! - [`Kind::Linux`]: the generator of a block Linux owns (Ethernet, DMA, ADC, PWM, I2S, audio
//!   out). Linux's enable/disable and rate requests program the generator (CTRL.ENABLE, AUXSRC and
//!   the divider), restricted to the parents RP1 keeps running (XOSC and the PLL_SYS outputs).
//!
//! A Linux-kind clock whose block the image declares (v0.1: `ADC`) is *held* by RP1
//! ([`Server::holding`], from [`crate::linux_blocks::LinuxBlocks::held_clocks`]): the block's
//! driver owns the generator, Linux's node is disabled by the overlay, and the server treats the
//! clock like a fixed one at whatever the generator runs (rate changes `DENIED`, enable/disable
//! only votes), so `clk_disable_unused` cannot stop it.
//!
//! The service is polled from proc0's main loop ([`Service::poll`]); no interrupt is used.
use crate::read;

pub const BASE_PROTOCOL: u8 = 0x10;
pub const CLOCK_PROTOCOL: u8 = 0x14;
const VERSION_2_0: u32 = 0x0002_0000;

/// SCMI status codes (signed, first payload word of every response).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum Status {
    Success = 0,
    NotSupported = -1,
    InvalidParameters = -2,
    Denied = -3,
    NotFound = -4,
    HardwareError = -9,
    ProtocolError = -10,
}

/// A clock source RP1 keeps running for Linux-owned generators (DOC clk-rp1 parent names).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    Xosc,
    /// PLL_SYS primary, 1 GHz / 5 / 1.
    PllSys,
    /// PLL_SYS primary phase output, PLL_SYS / 2.
    PllSysPriPh,
    /// PLL_SYS secondary divider, 1 GHz / 8 (programmed by [`crate::linux_blocks`]).
    PllSysSec,
}

impl Source {
    pub const fn hz(self) -> u32 {
        match self {
            Source::Xosc => 50_000_000,
            Source::PllSys => 200_000_000,
            Source::PllSysPriPh => 100_000_000,
            Source::PllSysSec => 125_000_000,
        }
    }
}

/// CLOCK_RATE_SET rounding (SCMI flags bits 3:2; Linux `CLOCK_SET_ROUND_UP` = bit 2,
/// `CLOCK_SET_ROUND_AUTO` = bit 3). Neither bit: round down, the rate must not exceed the request.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Round {
    Down,
    Up,
    Nearest,
}

impl Round {
    pub const fn from_flags(flags: u32) -> Round {
        if flags & 8 != 0 {
            Round::Nearest
        } else if flags & 4 != 0 {
            Round::Up
        } else {
            Round::Down
        }
    }
}

/// A clock-generator in the CLOCKS block (CTRL, DIV_INT, DIV_FRAC, SEL at +0/+4/+8/+0xc).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Generator {
    /// Absolute address of CLK_x_CTRL.
    pub ctrl: usize,
    /// Has a fractional divider (DIV_FRAC, kept at 0).
    pub frac: bool,
    pub div_max: u32,
    pub max_hz: u32,
    /// (CTRL.AUXSRC value, source) for the parents that may be selected; the first is the default.
    pub parents: &'static [(u8, Source)],
}

pub const CTRL_ENABLE: u32 = 1 << 11;
const AUXSRC_SHIFT: u32 = 5;
const AUXSRC_MASK: u32 = 0x1f << AUXSRC_SHIFT;

/// Register access, separate so the generator logic runs on the host in tests.
pub trait Registers {
    fn read(&mut self, address: usize) -> u32;
    fn write(&mut self, address: usize, value: u32);
}

impl Generator {
    fn parent(&self, aux: u32) -> Option<Source> {
        self.parents
            .iter()
            .find(|p| u32::from(p.0) == aux)
            .map(|p| p.1)
    }

    /// Rate and enable state from the registers; `None` if the selected parent is not one RP1
    /// keeps running or a fractional divider is set (this server only writes integer dividers).
    pub fn state(&self, r: &mut impl Registers) -> Option<PhysicalState> {
        let ctrl = r.read(self.ctrl);
        let parent = self.parent((ctrl & AUXSRC_MASK) >> AUXSRC_SHIFT)?;
        if self.frac && r.read(self.ctrl + 8) != 0 {
            return None;
        }
        // DIV_INT 0 means 65536 (clk-rp1).
        let div = match r.read(self.ctrl + 4) {
            0 => 1 << 16,
            n => n,
        };
        Some(PhysicalState {
            rate_hz: parent.hz() / div,
            enabled: ctrl & CTRL_ENABLE != 0,
        })
    }

    /// Legal integer dividers for `parent`: never above `max_hz`, at most `div_max`. Integer
    /// only: the M3 then needs no 64-bit division (code size), and the rates Linux asks for here
    /// (Ethernet 125 / 25 / 2.5 MHz, DMA, ADC, I2S) are integer divisions of the parents.
    fn dividers(&self, parent: Source) -> (u32, u32) {
        (parent.hz().div_ceil(self.max_hz).max(1), self.div_max)
    }

    /// Best (AUXSRC, divider, achieved rate) for `rate` in direction `round`, comparing the
    /// achieved output Hz of the two dividers around `parent / rate` for every parent. `None` for
    /// 0 Hz or when no legal setting lies in the requested direction.
    pub fn choose(&self, rate: u32, round: Round) -> Option<(u8, u32, u32)> {
        if rate == 0 {
            return None;
        }
        let mut best: Option<(u8, u32, u32)> = None;
        for &(aux, parent) in self.parents {
            let p = parent.hz();
            let (lo, hi) = self.dividers(parent);
            // p / (p / rate) >= rate and p / ceil(p / rate) <= rate (integer division).
            for div in [p / rate, p.div_ceil(rate)] {
                let div = div.clamp(lo, hi);
                let got = p / div;
                let fits = match round {
                    Round::Down => got <= rate,
                    Round::Up => got >= rate,
                    Round::Nearest => true,
                };
                if fits && best.is_none_or(|b| got.abs_diff(rate) < b.2.abs_diff(rate)) {
                    best = Some((aux, div, got));
                }
            }
        }
        best
    }

    /// Lowest and highest rate the generator can produce from its allowed parents.
    pub fn range(&self) -> (u32, u32) {
        let mut low = u32::MAX;
        let mut high = 0;
        for &(_, parent) in self.parents {
            low = low.min(parent.hz() / self.div_max);
            high = high.max(parent.hz() / self.dividers(parent).0);
        }
        (low, high)
    }

    /// Programs parent and divider. A running clock is stopped around the change (the AUXSRC
    /// mux is not glitch-free, HYP from RP2040) and restarted.
    pub fn program(&self, r: &mut impl Registers, aux: u8, div: u32) {
        let ctrl = r.read(self.ctrl);
        if ctrl & CTRL_ENABLE != 0 {
            r.write(self.ctrl, ctrl & !CTRL_ENABLE);
        }
        r.write(self.ctrl + 4, div);
        if self.frac {
            r.write(self.ctrl + 8, 0);
        }
        let new = (ctrl & !(AUXSRC_MASK | CTRL_ENABLE)) | (u32::from(aux) << AUXSRC_SHIFT);
        r.write(self.ctrl, new);
        if ctrl & CTRL_ENABLE != 0 {
            r.write(self.ctrl, new | CTRL_ENABLE);
        }
    }

    pub fn set_enabled(&self, r: &mut impl Registers, on: bool) {
        let ctrl = r.read(self.ctrl);
        let new = if on {
            ctrl | CTRL_ENABLE
        } else {
            ctrl & !CTRL_ENABLE
        };
        if new != ctrl {
            r.write(self.ctrl, new);
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// RP1 runs on it: fixed rate, always on, Linux only votes.
    Fixed(u32),
    /// Owned by Linux: requests program the generator.
    Linux(Generator),
}

/// One clock as Linux sees it. The SCMI clock ID is the index in the table.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClockSpec {
    /// At most 15 bytes (SCMI 2.0 name field, NUL terminated).
    pub name: &'static str,
    pub kind: Kind,
}

const CLOCKS: usize = 0x4001_8000;
const fn linux(
    name: &'static str,
    ctrl: usize,
    frac: bool,
    div_max: u32,
    max_hz: u32,
    parents: &'static [(u8, Source)],
) -> ClockSpec {
    ClockSpec {
        name,
        kind: Kind::Linux(Generator {
            ctrl: CLOCKS + ctrl,
            frac,
            div_max,
            max_hz,
            parents,
        }),
    }
}

/// SCMI clock IDs (the `<&rp1_scmi_clocks N>` cells the overlay from rp1-build writes).
pub mod id {
    pub const UART_APB: u32 = 0;
    pub const UART: u32 = 1;
    pub const SYS: u32 = 2;
    pub const SLOW_SYS: u32 = 3;
    pub const DMA: u32 = 4;
    pub const ETH: u32 = 5;
    pub const ETH_TSU: u32 = 6;
    pub const ADC: u32 = 7;
    pub const PWM0: u32 = 8;
    pub const PWM1: u32 = 9;
    pub const AUDIO_OUT: u32 = 10;
    pub const I2S: u32 = 11;
}

/// rpi-cm5 board table. CTRL offsets, AUXSRC indices, divider widths and maximum rates are those
/// of Linux clk-rp1 (DOC) and match rp1-regdb `clocks-main`; only parents RP1 runs are listed.
/// Order must match [`id`].
pub const RP1_CLOCKS: &[ClockSpec] = &[
    // PLL_SYS PRI_PH: the APB clock clk-rp1 names for the PL011s.
    ClockSpec {
        name: "uart_apb",
        kind: Kind::Fixed(100_000_000),
    },
    ClockSpec {
        name: "uart",
        kind: Kind::Fixed(50_000_000),
    },
    ClockSpec {
        name: "sys",
        kind: Kind::Fixed(200_000_000),
    },
    ClockSpec {
        name: "slow_sys",
        kind: Kind::Fixed(50_000_000),
    },
    linux(
        "dma",
        0x44,
        false,
        0xff,
        100_000_000,
        &[(0, Source::PllSysPriPh), (2, Source::Xosc)],
    ),
    linux(
        "eth",
        0x64,
        false,
        0xff,
        125_000_000,
        &[(0, Source::PllSysSec), (1, Source::PllSys)],
    ),
    linux(
        "eth_tsu",
        0x134,
        false,
        0xff,
        50_000_000,
        &[(0, Source::Xosc)],
    ),
    linux("adc", 0x144, false, 0xff, 50_000_000, &[(0, Source::Xosc)]),
    linux("pwm0", 0x74, true, 0xffff, 76_800_000, &[(2, Source::Xosc)]),
    linux("pwm1", 0x84, true, 0xffff, 76_800_000, &[(2, Source::Xosc)]),
    linux(
        "audio_out",
        0xa4,
        false,
        0xff,
        153_600_000,
        &[(3, Source::Xosc)],
    ),
    linux("i2s", 0xb4, false, 0xff, 50_000_000, &[(0, Source::Xosc)]),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PhysicalState {
    pub rate_hz: u32,
    pub enabled: bool,
}

/// Clock hardware: the state of the fixed clocks (checked by [`crate::clocks`]) and register
/// access for the Linux-owned generators.
pub trait ClockHardware: Registers {
    fn fixed_state(&mut self, id: u32) -> Result<PhysicalState, Status>;
}

/// What one request did, for logging by the caller (outside the response path).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Event {
    pub protocol: u8,
    pub message: u8,
    pub status: i32,
    /// First two request words (0 if absent).
    pub args: [u32; 2],
    /// Linux's enable votes after the request, one bit per clock ID.
    pub votes: u32,
}

/// Max request payload (CLOCK_RATE_SET) and response payload (status + 7 words).
pub const MAX_ARGS: usize = 4;
pub const MAX_RESPONSE: usize = 8;

pub struct Response {
    pub words: [u32; MAX_RESPONSE],
    pub len: usize,
}

impl Response {
    fn ok(data: &[u32]) -> Self {
        let mut r = Self {
            words: [0; MAX_RESPONSE],
            len: data.len() + 1,
        };
        r.words[1..r.len].copy_from_slice(data);
        r
    }
    fn err(status: Status) -> Self {
        Self {
            words: [status as i32 as u32, 0, 0, 0, 0, 0, 0, 0],
            len: 1,
        }
    }
    /// `prefix`, 16-byte NUL-terminated name, `suffix`.
    fn named(prefix: &[u32], name: &str, suffix: &[u32]) -> Self {
        let mut data = [0u32; MAX_RESPONSE - 1];
        data[..prefix.len()].copy_from_slice(prefix);
        for (i, b) in name.bytes().take(15).enumerate() {
            data[prefix.len() + i / 4] |= u32::from(b) << ((i % 4) * 8);
        }
        let end = prefix.len() + 4;
        data[end..end + suffix.len()].copy_from_slice(suffix);
        Self::ok(&data[..end + suffix.len()])
    }
    pub fn status(&self) -> i32 {
        self.words[0] as i32
    }
}

pub struct Server<'a, H> {
    clocks: &'a [ClockSpec],
    hw: H,
    votes: u32,
    held: u32,
}

/// How the server treats one clock in a request.
#[derive(Clone, Copy)]
enum Role {
    Fixed(u32),
    Linux(Generator),
    /// A Linux-kind generator RP1 holds: report only, never written.
    Held,
}

impl<'a, H: ClockHardware> Server<'a, H> {
    pub const fn new(clocks: &'a [ClockSpec], hw: H) -> Self {
        assert!(clocks.len() <= 32);
        Self {
            clocks,
            hw,
            votes: 0,
            held: 0,
        }
    }

    /// Marks the clocks in `held` (one bit per ID) as held by RP1. Only Linux-kind clocks can
    /// be held; fixed ones already are.
    pub const fn holding(mut self, held: u32) -> Self {
        self.held = held;
        self
    }

    pub fn votes(&self) -> u32 {
        self.votes
    }

    /// Handles one message. `header` is the raw SCMI header word; the transport keeps it as is.
    pub fn handle(&mut self, header: u32, args: &[u32]) -> Response {
        self.dispatch(header, args).unwrap_or_else(Response::err)
    }

    fn clock(&mut self, id: u32) -> Result<(ClockSpec, PhysicalState, Role), Status> {
        let spec = *self.clocks.get(id as usize).ok_or(Status::NotFound)?;
        let (state, role) = match spec.kind {
            Kind::Fixed(rate) => {
                let state = self.hw.fixed_state(id)?;
                // RP1 needs it running at its fixed rate; anything else is a fault.
                if state.rate_hz != rate || !state.enabled {
                    return Err(Status::HardwareError);
                }
                (state, Role::Fixed(rate))
            }
            Kind::Linux(g) => {
                let state = g.state(&mut self.hw).ok_or(Status::HardwareError)?;
                if self.held & (1 << id) != 0 {
                    (state, Role::Held)
                } else {
                    (state, Role::Linux(g))
                }
            }
        };
        Ok((spec, state, role))
    }

    fn dispatch(&mut self, header: u32, a: &[u32]) -> Result<Response, Status> {
        // Bits 9:8 (type) must be 0 (command); 31:28 reserved.
        if header & 0xf000_0300 != 0 {
            return Err(Status::ProtocolError);
        }
        let protocol = ((header >> 10) & 0xff) as u8;
        let message = (header & 0xff) as u8;
        let expected = match (protocol, message) {
            (BASE_PROTOCOL, 0 | 1 | 3 | 4 | 5) | (CLOCK_PROTOCOL, 0 | 1) => 0,
            (BASE_PROTOCOL, 2 | 6 | 7) | (CLOCK_PROTOCOL, 2 | 3 | 6) => 1,
            (CLOCK_PROTOCOL, 4 | 7) => 2,
            (CLOCK_PROTOCOL, 5) => 4,
            _ => return Err(Status::NotSupported),
        };
        if a.len() != expected {
            return Err(Status::InvalidParameters);
        }
        Ok(match (protocol, message) {
            (_, 0) => Response::ok(&[VERSION_2_0]),
            // One protocol besides Base, two agents (RP1 firmware, Linux).
            (BASE_PROTOCOL, 1) => Response::ok(&[0x0201]),
            (BASE_PROTOCOL, 2) => {
                if a[0] > 7 {
                    return Err(Status::NotSupported);
                }
                Response::ok(&[0])
            }
            (BASE_PROTOCOL, 3) => Response::named(&[], "Natsu-B", &[]),
            (BASE_PROTOCOL, 4) => Response::named(&[], "rp1-hal", &[]),
            (BASE_PROTOCOL, 5) => Response::ok(&[1]),
            (BASE_PROTOCOL, 6) => match a[0] {
                0 => Response::ok(&[1, u32::from(CLOCK_PROTOCOL)]),
                1 => Response::ok(&[0]),
                _ => return Err(Status::InvalidParameters),
            },
            (BASE_PROTOCOL, 7) => match a[0] {
                0 => Response::named(&[0], "RP1", &[]),
                1 | 0xffff_ffff => Response::named(&[1], "OSPM", &[]),
                _ => return Err(Status::NotFound),
            },
            (CLOCK_PROTOCOL, 1) => Response::ok(&[self.clocks.len() as u32]),
            // RATE_GET..CONFIG_SET; notifications (9, 10) and 3.x messages are not supported.
            (CLOCK_PROTOCOL, 2) => {
                if a[0] > 7 {
                    return Err(Status::NotSupported);
                }
                Response::ok(&[0])
            }
            // attributes (bit0 enabled), name[16], clock_enable_latency (2.0).
            (CLOCK_PROTOCOL, 3) => {
                let (spec, state, _) = self.clock(a[0])?;
                Response::named(&[u32::from(state.enabled)], spec.name, &[0])
            }
            // Discrete list, none remaining: the fixed (or held) rate, or the lowest and highest
            // rate of a Linux generator (Linux takes them as its rate range and asks for any
            // rate in it).
            (CLOCK_PROTOCOL, 4) => {
                let (_, state, role) = self.clock(a[0])?;
                match (a[1], role) {
                    (0, Role::Fixed(rate)) => Response::ok(&[1, rate, 0]),
                    (0, Role::Held) => Response::ok(&[1, state.rate_hz, 0]),
                    (0, Role::Linux(g)) => {
                        let (low, high) = g.range();
                        Response::ok(&[2, low, 0, high, 0])
                    }
                    (1, Role::Fixed(_) | Role::Held) | (2, Role::Linux(_)) => Response::ok(&[0]),
                    _ => return Err(Status::InvalidParameters),
                }
            }
            (CLOCK_PROTOCOL, 5) => {
                // flags: bit0 async (not offered), bit2 round up, bit3 round auto (else down).
                // No legal rate in the requested direction is an error, never a silent overshoot.
                if a[0] & !0xc != 0 {
                    return Err(Status::InvalidParameters);
                }
                let (_, state, role) = self.clock(a[1])?;
                if a[3] != 0 {
                    return Err(Status::InvalidParameters);
                }
                match role {
                    Role::Fixed(rate) if a[2] == rate => {}
                    Role::Held if a[2] == state.rate_hz => {}
                    Role::Fixed(_) | Role::Held => return Err(Status::Denied),
                    Role::Linux(g) => {
                        let (aux, div, _) = g
                            .choose(a[2], Round::from_flags(a[0]))
                            .ok_or(Status::InvalidParameters)?;
                        g.program(&mut self.hw, aux, div);
                    }
                }
                Response::ok(&[])
            }
            (CLOCK_PROTOCOL, 6) => {
                let (_, state, _) = self.clock(a[0])?;
                Response::ok(&[state.rate_hz, 0])
            }
            (CLOCK_PROTOCOL, 7) => {
                if a[1] & !1 != 0 {
                    return Err(Status::InvalidParameters);
                }
                let (_, _, role) = self.clock(a[0])?;
                let on = a[1] & 1 != 0;
                // Fixed / held: the vote only, the clock stays on for RP1. Linux: the vote is
                // the state.
                if let Role::Linux(g) = role {
                    g.set_enabled(&mut self.hw, on);
                }
                if on {
                    self.votes |= 1 << a[0];
                } else {
                    self.votes &= !(1 << a[0]);
                }
                Response::ok(&[])
            }
            _ => return Err(Status::NotSupported),
        })
    }
}

/// Mailbox + shared memory access. Separate so the channel logic runs on the host in tests.
pub trait Transport {
    /// Pending request event for this channel; acknowledges it.
    fn take_event(&mut self) -> bool;
    fn read(&mut self, word: usize) -> u32;
    fn write(&mut self, word: usize, value: u32);
    fn barrier(&mut self);
    fn notify(&mut self);
}

// arm,scmi-shmem layout (words).
const STATUS: usize = 1;
const FLAGS: usize = 4;
const LENGTH: usize = 5;
const HEADER: usize = 6;
const PAYLOAD: usize = 7;
const WORDS: usize = 64;
const FREE: u32 = 1;
const FLAG_INTR: u32 = 1;

/// Zeroes the area and marks the channel free. Linux waits for FREE before its first request.
pub fn reset_channel(t: &mut impl Transport) {
    for i in 0..WORDS {
        t.write(i, 0);
    }
    t.barrier();
    t.write(STATUS, FREE);
    t.barrier();
}

/// Serves at most one pending request.
pub fn serve<H: ClockHardware>(
    server: &mut Server<'_, H>,
    t: &mut impl Transport,
) -> Option<Event> {
    if !t.take_event() {
        return None;
    }
    t.barrier();
    if t.read(STATUS) & FREE != 0 {
        // Doorbell without a request (spurious or duplicate): nothing to answer.
        return None;
    }
    let length = t.read(LENGTH) as usize;
    let header = t.read(HEADER);
    let flags = t.read(FLAGS);
    let mut args = [0u32; MAX_ARGS];
    let response = if !(4..=4 + 4 * MAX_ARGS).contains(&length) || !length.is_multiple_of(4) {
        Response::err(Status::ProtocolError)
    } else {
        let n = (length - 4) / 4;
        for (i, v) in args[..n].iter_mut().enumerate() {
            *v = t.read(PAYLOAD + i);
        }
        server.handle(header, &args[..n])
    };
    for (i, v) in response.words[..response.len].iter().enumerate() {
        t.write(PAYLOAD + i, *v);
    }
    // Header stays (token preserved); length = header + status + data.
    t.write(LENGTH, (4 + 4 * response.len) as u32);
    t.barrier();
    t.write(STATUS, FREE);
    t.barrier();
    if flags & FLAG_INTR != 0 {
        t.notify();
    }
    Some(Event {
        protocol: ((header >> 10) & 0xff) as u8,
        message: header as u8,
        status: response.status(),
        args: [args[0], args[1]],
        votes: server.votes(),
    })
}

/// SCMI shmem in shared SRAM (`scmi@d7c0` in scmi_linux.dtb; `rp1_build::metadata::SCMI_OFFSET`).
pub const SHMEM: usize = 0x2000_d7c0;
// Fixed-address scratch must sit above the image (rp1-rt link.x `__rp1_fixed_scratch_floor`)
// and below the ABI reservation.
const _: () = assert!(
    SHMEM >= rp1_proto::LINK_START as usize && SHMEM + 256 <= rp1_proto::RESERVED_START as usize
);
/// rp1-mailbox channel 0 (`mboxes = <&rp1_mbox 0>`). SYSCFG PROC_EVENTS / HOST_EVENTS, with the
/// SET (+0x2000) and CLR (+0x3000) aliases as rp1-mailbox.c uses them (PoC, HW).
const PROC_EVENTS: usize = 0x4000_8008;
const PROC_EVENTS_CLR: usize = PROC_EVENTS + 0x3000;
const HOST_EVENTS_SET: usize = 0x4000_800c + 0x2000;
const CHANNEL: u32 = 1 << 0;

/// Register-level transport on RP1.
pub struct Rp1Transport {
    _private: (),
}

impl Transport for Rp1Transport {
    fn take_event(&mut self) -> bool {
        if read(PROC_EVENTS) & CHANNEL == 0 {
            return false;
        }
        crate::write(PROC_EVENTS_CLR, CHANNEL); // Only our channel's bit.
        true
    }
    fn read(&mut self, word: usize) -> u32 {
        assert!(word < WORDS);
        read(SHMEM + 4 * word)
    }
    fn write(&mut self, word: usize, value: u32) {
        assert!(word < WORDS);
        unsafe { ((SHMEM + 4 * word) as *mut u32).write_volatile(value) }
    }
    fn barrier(&mut self) {
        rp1_proto::barrier();
    }
    fn notify(&mut self) {
        crate::write(HOST_EVENTS_SET, CHANNEL);
    }
}

/// Physical clock state from the registers: the fixed clocks as [`crate::clocks`] checks them
/// (read twice; any difference from the admitted tuple reads as a hardware error), and plain
/// MMIO for the Linux-owned generators.
pub struct Rp1Clocks {
    _private: (),
}

impl Registers for Rp1Clocks {
    fn read(&mut self, address: usize) -> u32 {
        read(address)
    }
    fn write(&mut self, address: usize, value: u32) {
        crate::write(address, value);
    }
}

impl ClockHardware for Rp1Clocks {
    fn fixed_state(&mut self, id: u32) -> Result<PhysicalState, Status> {
        let on = |rate_hz| {
            Ok(PhysicalState {
                rate_hz,
                enabled: true,
            })
        };
        match id {
            // PLL_SYS PRI_PH = PLL_SYS primary (1 GHz / 5 / 1) / 2, a PLL output independent of
            // the clk_sys mux (DOC clk-rp1 `pll_sys_pri_ph`); enabled = PRIM bit 4 (PoC).
            id::UART_APB if crate::clocks::running_tuple_admitted() => Ok(PhysicalState {
                rate_hz: 100_000_000,
                enabled: read(0x4002_0010) & 0x10 != 0,
            }),
            id::UART => crate::clocks::uart_hz().map_or(Err(Status::HardwareError), on),
            id::SYS => crate::clocks::sys_hz().map_or(Err(Status::HardwareError), on),
            // CLK_SLOW_SYS: XOSC (SEL one-hot 1) / DIV_INT 1, always on in hardware (DOC clk-rp1).
            id::SLOW_SYS if read(CLOCKS + 0x28) == 1 && read(CLOCKS + 0x30) == 1 => {
                on(crate::clocks::XOSC_HZ)
            }
            id::UART_APB | id::SLOW_SYS => Err(Status::HardwareError),
            _ => Err(Status::NotFound),
        }
    }
}

/// The SCMI server on proc0. Created by [`crate::clocks::Authority::init`].
pub struct Service {
    server: Server<'static, Rp1Clocks>,
    transport: Rp1Transport,
    linux_blocks: crate::linux_blocks::Report,
}

impl Service {
    /// # Safety
    /// One instance, proc0, holding clock authority; no other user of mailbox channel 0, the
    /// shmem area or the Linux-owned clock generators on RP1.
    pub(crate) unsafe fn new(linux_blocks: crate::linux_blocks::Report, held: u32) -> Self {
        let mut transport = Rp1Transport { _private: () };
        reset_channel(&mut transport);
        Self {
            server: Server::new(RP1_CLOCKS, Rp1Clocks { _private: () }).holding(held),
            transport,
            linux_blocks,
        }
    }

    /// Answers one pending request, if any. Call from proc0's main loop; never blocks.
    ///
    /// The idle check (one SYSCFG read) runs from ISRAM; the request handling is cold and
    /// stays in shared SRAM, so a polling loop does not fetch from shared SRAM while idle.
    #[rp1_rt::isram]
    pub fn poll(&mut self) -> Option<Event> {
        if read(PROC_EVENTS) & CHANNEL == 0 {
            return None;
        }
        self.serve_pending()
    }
    #[rp1_rt::cold]
    fn serve_pending(&mut self) -> Option<Event> {
        serve(&mut self.server, &mut self.transport)
    }

    pub fn votes(&self) -> u32 {
        self.server.votes()
    }

    /// What the bring-up of the Linux-owned blocks did (for logging).
    pub fn linux_blocks(&self) -> crate::linux_blocks::Report {
        self.linux_blocks
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use std::collections::BTreeMap;
    use std::vec::Vec;

    #[derive(Default)]
    struct Hw {
        uart: Option<PhysicalState>,
        regs: BTreeMap<usize, u32>,
        writes: Vec<(usize, u32)>,
    }
    impl Registers for Hw {
        fn read(&mut self, address: usize) -> u32 {
            *self.regs.get(&address).unwrap_or(&0)
        }
        fn write(&mut self, address: usize, value: u32) {
            self.regs.insert(address, value);
            self.writes.push((address, value));
        }
    }
    impl ClockHardware for Hw {
        fn fixed_state(&mut self, id: u32) -> Result<PhysicalState, Status> {
            let on = |rate_hz| PhysicalState {
                rate_hz,
                enabled: true,
            };
            match id {
                0 => Ok(on(100_000_000)),
                1 => Ok(self.uart.unwrap_or(on(50_000_000))),
                2 => Ok(on(200_000_000)),
                3 => Ok(on(50_000_000)),
                _ => Err(Status::NotFound),
            }
        }
    }
    fn server() -> Server<'static, Hw> {
        Server::new(RP1_CLOCKS, Hw::default())
    }
    fn req(s: &mut Server<Hw>, p: u8, m: u8, a: &[u32]) -> Vec<u32> {
        let r = s.handle((0x2a << 18) | (u32::from(p) << 10) | u32::from(m), a);
        r.words[..r.len].to_vec()
    }
    fn generator(id: u32) -> Generator {
        match RP1_CLOCKS[id as usize].kind {
            Kind::Linux(g) => g,
            Kind::Fixed(_) => panic!(),
        }
    }

    #[test]
    fn table_matches_ids_and_clk_rp1_offsets() {
        let names: Vec<_> = RP1_CLOCKS.iter().map(|c| c.name).collect();
        assert_eq!(
            names,
            [
                "uart_apb",
                "uart",
                "sys",
                "slow_sys",
                "dma",
                "eth",
                "eth_tsu",
                "adc",
                "pwm0",
                "pwm1",
                "audio_out",
                "i2s"
            ]
        );
        // CLK_x_CTRL addresses (DOC clk-rp1, rp1-regdb clocks-main).
        for (id, ctrl) in [
            (id::DMA, 0x4001_8044),
            (id::ETH, 0x4001_8064),
            (id::ETH_TSU, 0x4001_8134),
            (id::ADC, 0x4001_8144),
            (id::PWM0, 0x4001_8074),
            (id::PWM1, 0x4001_8084),
            (id::AUDIO_OUT, 0x4001_80a4),
            (id::I2S, 0x4001_80b4),
        ] {
            assert_eq!(generator(id).ctrl, ctrl, "{id}");
        }
        assert!(RP1_CLOCKS.iter().all(|c| c.name.len() <= 15));
    }

    #[test]
    fn base_discovery() {
        let mut s = server();
        assert_eq!(req(&mut s, 0x10, 0, &[]), [0, 0x2_0000]);
        assert_eq!(req(&mut s, 0x10, 1, &[]), [0, 0x201]);
        assert_eq!(req(&mut s, 0x10, 3, &[])[1], u32::from_le_bytes(*b"Nats"));
        assert_eq!(req(&mut s, 0x10, 6, &[0]), [0, 1, 0x14]);
        assert_eq!(req(&mut s, 0x10, 6, &[1]), [0, 0]);
        assert_eq!(req(&mut s, 0x10, 7, &[0xffff_ffff])[..2], [0, 1]);
        assert_eq!(req(&mut s, 0x10, 8, &[0])[0] as i32, -1);
        assert_eq!(req(&mut s, 0x10, 2, &[8])[0] as i32, -1);
    }

    #[test]
    fn fixed_clocks_discovery_and_rates() {
        let mut s = server();
        assert_eq!(req(&mut s, 0x14, 1, &[]), [0, 12]);
        let attr = req(&mut s, 0x14, 3, &[1]);
        assert_eq!(attr.len(), 7);
        assert_eq!(attr[..3], [0, 1, u32::from_le_bytes(*b"uart")]);
        assert_eq!(req(&mut s, 0x14, 4, &[0, 0]), [0, 1, 100_000_000, 0]);
        assert_eq!(req(&mut s, 0x14, 6, &[1]), [0, 50_000_000, 0]);
        assert_eq!(req(&mut s, 0x14, 6, &[2]), [0, 200_000_000, 0]);
        assert_eq!(req(&mut s, 0x14, 6, &[3]), [0, 50_000_000, 0]);
        assert_eq!(req(&mut s, 0x14, 6, &[12])[0] as i32, -4);
        assert_eq!(req(&mut s, 0x14, 2, &[9])[0] as i32, -1);
    }

    #[test]
    fn fixed_clocks_deny_rates_and_votes_never_stop_them() {
        let mut s = server();
        assert_eq!(req(&mut s, 0x14, 5, &[0, 1, 48_000_000, 0])[0] as i32, -3);
        assert_eq!(req(&mut s, 0x14, 5, &[0, 2, 100_000_000, 0])[0] as i32, -3);
        assert_eq!(req(&mut s, 0x14, 5, &[0, 1, 50_000_000, 0]), [0]);
        assert_eq!(req(&mut s, 0x14, 5, &[1, 1, 50_000_000, 0])[0] as i32, -2);
        assert_eq!(req(&mut s, 0x14, 7, &[1, 1]), [0]);
        assert_eq!(s.votes(), 2);
        assert_eq!(req(&mut s, 0x14, 7, &[1, 0]), [0]);
        assert_eq!(req(&mut s, 0x14, 7, &[2, 0]), [0]);
        assert_eq!(s.votes(), 0);
        // Still on after Linux dropped its vote, and no register was written.
        assert_eq!(req(&mut s, 0x14, 3, &[1])[1], 1);
        assert!(s.hw.writes.is_empty());
        assert_eq!(req(&mut s, 0x14, 7, &[1, 2])[0] as i32, -2);
    }

    #[test]
    fn drift_and_bad_headers_fail_closed() {
        let mut s = server();
        s.hw.uart = Some(PhysicalState {
            rate_hz: 48_000_000,
            enabled: true,
        });
        assert_eq!(req(&mut s, 0x14, 6, &[1])[0] as i32, -9);
        assert_eq!(req(&mut s, 0x14, 7, &[1, 1])[0] as i32, -9);
        assert_eq!(s.votes(), 0);
        assert_eq!(s.handle(1 << 8, &[]).status(), -10);
        assert_eq!(req(&mut s, 0x14, 6, &[1, 0])[0] as i32, -2);
        assert_eq!(req(&mut s, 0x15, 0, &[])[0] as i32, -1);
    }

    #[test]
    fn linux_clock_follows_enable_and_rate_requests() {
        let mut s = server();
        let eth = generator(id::ETH);
        // Stock state: AUXSRC 0 (pll_sys_sec), DIV 1, enabled.
        s.hw.regs.insert(eth.ctrl, 0x1000_0800);
        s.hw.regs.insert(eth.ctrl + 4, 1);
        let attr = req(&mut s, 0x14, 3, &[id::ETH]);
        assert_eq!(attr[..3], [0, 1, u32::from_le_bytes(*b"eth\0")]);
        assert_eq!(req(&mut s, 0x14, 6, &[id::ETH]), [0, 125_000_000, 0]);
        // macb at 100 Mb/s: 25 MHz = pll_sys_sec / 5, written with the clock stopped.
        assert_eq!(req(&mut s, 0x14, 5, &[0, id::ETH, 25_000_000, 0]), [0]);
        assert_eq!(
            s.hw.writes,
            [
                (eth.ctrl, 0x1000_0000),
                (eth.ctrl + 4, 5),
                (eth.ctrl, 0x1000_0000),
                (eth.ctrl, 0x1000_0800)
            ]
        );
        assert_eq!(req(&mut s, 0x14, 6, &[id::ETH]), [0, 25_000_000, 0]);
        // 10 Mb/s.
        assert_eq!(req(&mut s, 0x14, 5, &[0, id::ETH, 2_500_000, 0]), [0]);
        assert_eq!(req(&mut s, 0x14, 6, &[id::ETH]), [0, 2_500_000, 0]);
        // Disable writes CTRL.ENABLE.
        assert_eq!(req(&mut s, 0x14, 7, &[id::ETH, 0]), [0]);
        assert_eq!(s.hw.regs[&eth.ctrl] & CTRL_ENABLE, 0);
        assert_eq!(req(&mut s, 0x14, 3, &[id::ETH])[1], 0);
        assert_eq!(req(&mut s, 0x14, 7, &[id::ETH, 1]), [0]);
        assert_eq!(req(&mut s, 0x14, 3, &[id::ETH])[1], 1);
        assert_eq!(s.votes(), 1 << id::ETH);
        // Rate range: 125 MHz / 255 .. 125 MHz.
        assert_eq!(
            req(&mut s, 0x14, 4, &[id::ETH, 0]),
            [0, 2, 125_000_000 / 255, 0, 125_000_000, 0]
        );
        // Above the maximum: the closest legal rate. 0 Hz: invalid.
        assert_eq!(req(&mut s, 0x14, 5, &[0, id::ETH, 400_000_000, 0]), [0]);
        assert_eq!(req(&mut s, 0x14, 6, &[id::ETH]), [0, 125_000_000, 0]);
        assert_eq!(req(&mut s, 0x14, 5, &[0, id::ETH, 0, 0])[0] as i32, -2);
    }

    #[test]
    fn held_clock_reports_but_never_changes() {
        // An image that declares ADC: RP1's driver runs clk_adc at XOSC / 1.
        let mut s = server().holding(1 << id::ADC);
        let adc = generator(id::ADC);
        s.hw.regs.insert(adc.ctrl, CTRL_ENABLE);
        s.hw.regs.insert(adc.ctrl + 4, 1);
        assert_eq!(req(&mut s, 0x14, 3, &[id::ADC])[..2], [0, 1]);
        assert_eq!(req(&mut s, 0x14, 6, &[id::ADC]), [0, 50_000_000, 0]);
        assert_eq!(req(&mut s, 0x14, 4, &[id::ADC, 0]), [0, 1, 50_000_000, 0]);
        assert_eq!(req(&mut s, 0x14, 4, &[id::ADC, 1]), [0, 0]);
        assert_eq!(req(&mut s, 0x14, 4, &[id::ADC, 2])[0] as i32, -2);
        assert_eq!(
            req(&mut s, 0x14, 5, &[0, id::ADC, 25_000_000, 0])[0] as i32,
            -3
        );
        assert_eq!(req(&mut s, 0x14, 5, &[0, id::ADC, 50_000_000, 0]), [0]);
        // clk_disable_unused / a consumer's disable: a vote only, the clock stays on.
        assert_eq!(req(&mut s, 0x14, 7, &[id::ADC, 1]), [0]);
        assert_eq!(s.votes(), 1 << id::ADC);
        assert_eq!(req(&mut s, 0x14, 7, &[id::ADC, 0]), [0]);
        assert_eq!(s.votes(), 0);
        assert_eq!(req(&mut s, 0x14, 3, &[id::ADC])[1], 1);
        assert!(s.hw.writes.is_empty());
        // Other Linux clocks are unaffected.
        assert_eq!(req(&mut s, 0x14, 7, &[id::DMA, 1]), [0]);
        assert_eq!(s.hw.writes.len(), 1);
    }

    #[test]
    fn linux_clock_on_an_unserved_parent_is_a_hardware_error() {
        let mut s = server();
        // PWM0 AUXSRC 0 = pll_audio_pri_ph, which RP1 does not run.
        assert_eq!(req(&mut s, 0x14, 6, &[id::PWM0])[0] as i32, -9);
        assert_eq!(req(&mut s, 0x14, 7, &[id::PWM0, 1])[0] as i32, -9);
        assert!(s.hw.writes.is_empty());
    }

    /// Review 3 #3: RATE_SET honours the rounding flags and compares achieved output Hz.
    #[test]
    fn rate_set_rounding_direction() {
        let mut s = server();
        // ADC: 50 MHz xosc, integer divider. (flags, request, result)
        for (flags, rate, got) in [
            (0, 36_000_000, 25_000_000), // down: never above the request (was 50 MHz)
            (4, 20_000_000, 25_000_000), // up: never below it (was 16.67 MHz)
            (8, 36_000_000, 25_000_000), // auto: nearest output (|11| < |14| MHz)
            (12, 40_000_000, 50_000_000),
            (0, 50_000_000, 50_000_000),
            (4, 50_000_000, 50_000_000),
            (4, 1, 50_000_000 / 0xff),
            (0, 400_000_000, 50_000_000),
        ] {
            assert_eq!(
                req(&mut s, 0x14, 5, &[flags, id::ADC, rate, 0]),
                [0],
                "{flags} {rate}"
            );
            assert_eq!(
                req(&mut s, 0x14, 6, &[id::ADC]),
                [0, got, 0],
                "{flags} {rate}"
            );
        }
        // Nothing legal in the requested direction: an error, and the generator is untouched.
        let before = req(&mut s, 0x14, 6, &[id::ADC]);
        assert_eq!(
            req(&mut s, 0x14, 5, &[4, id::ADC, 60_000_000, 0])[0] as i32,
            -2
        );
        assert_eq!(
            req(&mut s, 0x14, 5, &[0, id::ADC, 100_000, 0])[0] as i32,
            -2
        );
        assert_eq!(req(&mut s, 0x14, 6, &[id::ADC]), before);
        // Bits other than async/round stay invalid.
        assert_eq!(
            req(&mut s, 0x14, 5, &[2, id::ADC, 25_000_000, 0])[0] as i32,
            -2
        );
        // Several parents: every candidate respects the direction.
        let eth = generator(id::ETH);
        for rate in [
            1_000_000u32,
            2_500_000,
            3_000_000,
            33_000_000,
            99_000_000,
            124_000_000,
        ] {
            let down = eth.choose(rate, Round::Down).unwrap().2;
            let up = eth.choose(rate, Round::Up).unwrap().2;
            let near = eth.choose(rate, Round::Nearest).unwrap().2;
            assert!(down <= rate && up >= rate, "{rate}: {down} {up}");
            assert_eq!(
                near.abs_diff(rate),
                down.abs_diff(rate).min(up.abs_diff(rate)),
                "{rate}"
            );
        }
    }

    #[test]
    fn generator_choice() {
        let dma = generator(id::DMA);
        let near = Round::Nearest;
        assert_eq!(dma.choose(100_000_000, near), Some((0, 1, 100_000_000)));
        // 50 MHz: pri_ph / 2 and xosc / 1 are equal; the first parent wins.
        assert_eq!(dma.choose(50_000_000, near), Some((0, 2, 50_000_000)));
        assert_eq!(dma.choose(200_000_000, near).unwrap().2, 100_000_000);
        let eth = generator(id::ETH);
        // 200 MHz parent exceeds max_hz at /1.
        assert_eq!(eth.choose(200_000_000, near).unwrap().2, 125_000_000);
        assert_eq!(eth.choose(100_000_000, near), Some((1, 2, 100_000_000)));
        // PWM: integer divider only, DIV_FRAC written as 0.
        let pwm = generator(id::PWM0);
        let (aux, div, got) = pwm.choose(16_000_000, near).unwrap();
        assert_eq!((aux, div, got), (2, 3, 16_666_666));
        assert_eq!(pwm.range(), (50_000_000 / 0xffff, 50_000_000));
        let mut hw = Hw::default();
        pwm.program(&mut hw, aux, div);
        assert_eq!(
            hw.writes,
            [(pwm.ctrl + 4, 3), (pwm.ctrl + 8, 0), (pwm.ctrl, 2 << 5)]
        );
        assert_eq!(
            pwm.state(&mut hw),
            Some(PhysicalState {
                rate_hz: 16_666_666,
                enabled: false
            })
        );
        // A fractional divider set by someone else is not a state this server made.
        hw.regs.insert(pwm.ctrl + 8, 0x8000_0000);
        assert_eq!(pwm.state(&mut hw), None);
        // DIV_INT 0 reads as 65536 (clk-rp1).
        hw.regs.insert(pwm.ctrl + 4, 0);
        hw.regs.insert(pwm.ctrl + 8, 0);
        assert_eq!(pwm.state(&mut hw).unwrap().rate_hz, 50_000_000 / 65536);
    }

    struct Fake {
        words: [u32; WORDS],
        event: bool,
        log: Vec<(usize, u32)>,
    }
    const BAR: usize = 100;
    const NOTIFY: usize = 101;
    impl Transport for Fake {
        fn take_event(&mut self) -> bool {
            core::mem::take(&mut self.event)
        }
        fn read(&mut self, i: usize) -> u32 {
            self.words[i]
        }
        fn write(&mut self, i: usize, v: u32) {
            self.words[i] = v;
            self.log.push((i, v));
        }
        fn barrier(&mut self) {
            self.log.push((BAR, 0));
        }
        fn notify(&mut self) {
            self.log.push((NOTIFY, 0));
        }
    }
    fn request(header: u32, args: &[u32], flags: u32) -> Fake {
        let mut f = Fake {
            words: [0; WORDS],
            event: true,
            log: Vec::new(),
        };
        f.words[FLAGS] = flags;
        f.words[LENGTH] = 4 + 4 * args.len() as u32;
        f.words[HEADER] = header;
        f.words[PAYLOAD..PAYLOAD + args.len()].copy_from_slice(args);
        f
    }

    #[test]
    fn channel_publishes_free_last_then_notifies() {
        let mut s = server();
        let header = (0x3ff << 18) | (0x14 << 10) | 6;
        let mut f = request(header, &[1], FLAG_INTR);
        let e = serve(&mut s, &mut f).unwrap();
        assert_eq!((e.protocol, e.message, e.status), (0x14, 6, 0));
        assert_eq!(f.words[HEADER], header);
        assert_eq!(f.words[PAYLOAD..PAYLOAD + 3], [0, 50_000_000, 0]);
        assert_eq!(
            f.log[f.log.len() - 5..],
            [
                (LENGTH, 16),
                (BAR, 0),
                (STATUS, FREE),
                (BAR, 0),
                (NOTIFY, 0)
            ]
        );
        // No event, or event on a free channel: nothing happens.
        assert!(serve(&mut s, &mut f).is_none());
        f.event = true;
        assert!(serve(&mut s, &mut f).is_none());
    }

    #[test]
    fn channel_bounds_length_and_polls_without_notify() {
        let mut s = server();
        for len in [0, 3, 5, 24, u32::MAX] {
            let mut f = request(0x14 << 10, &[], 0);
            f.words[LENGTH] = len;
            assert_eq!(serve(&mut s, &mut f).unwrap().status, -10);
            assert!(!f.log.iter().any(|&(i, _)| i == NOTIFY));
        }
        let mut f = request(0x14 << 10, &[], 0);
        reset_channel(&mut f);
        assert_eq!(f.words[STATUS], FREE);
        assert_eq!(f.words[FLAGS], 0);
    }
}
