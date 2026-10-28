//! SCMI 2.0 server (Base + Clock protocols) for `clock.authority = "rp1"`.
//!
//! Linux sees the RP1 clocks through `arm,scmi` instead of clk-rp1. Transport is the board's
//! fixed one (scmi_linux.dtb): rp1-mailbox channel 0 + an `arm,scmi-shmem` area at shared SRAM
//! `0x2000_d7c0` (256 bytes, kept out of the link page by rp1-build). Same message layout as the
//! HW-proven PoC server (`RP1-hal-PoC-openamp-20260929/crates/rp1-hal/src/scmi_*.rs`, Oct22 Z2).
//!
//! Policy: every clock exposed here is RP1-owned (UART0 on RP1 shares clk_uart with Linux's
//! UART1). The rate is fixed and the clock stays on: `CLOCK_RATE_SET` to another rate is
//! `DENIED`, `CLOCK_CONFIG_SET` only records Linux's vote (SCMI shared-clock semantics: the
//! platform keeps a clock running while another agent needs it). Nothing here writes a clock
//! register; reads report the physical state and fail closed if it drifts.
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

/// One clock as Linux sees it. The SCMI clock ID is the index in the table.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClockSpec {
    /// At most 15 bytes (SCMI 2.0 name field, NUL terminated).
    pub name: &'static str,
    pub rate_hz: u32,
}

/// rpi-cm5 board table. IDs must match the `clocks = <&scmi_clk N>` users in scmi_linux.dtb
/// (serial@34000: uartclk = 1, apb_pclk = 0).
pub const RP1_CLOCKS: &[ClockSpec] = &[
    ClockSpec {
        name: "uart_apb",
        rate_hz: 100_000_000,
    },
    ClockSpec {
        name: "uart",
        rate_hz: 50_000_000,
    },
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PhysicalState {
    pub rate_hz: u32,
    pub enabled: bool,
}

/// Reads the physical state of clock `id` (index in the table).
pub trait ClockHardware {
    fn state(&mut self, id: u32) -> Result<PhysicalState, Status>;
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
}

impl<'a, H: ClockHardware> Server<'a, H> {
    pub const fn new(clocks: &'a [ClockSpec], hw: H) -> Self {
        assert!(clocks.len() <= 32);
        Self {
            clocks,
            hw,
            votes: 0,
        }
    }

    pub fn votes(&self) -> u32 {
        self.votes
    }

    /// Handles one message. `header` is the raw SCMI header word; the transport keeps it as is.
    pub fn handle(&mut self, header: u32, args: &[u32]) -> Response {
        self.dispatch(header, args).unwrap_or_else(Response::err)
    }

    fn clock(&mut self, id: u32) -> Result<(ClockSpec, PhysicalState), Status> {
        let spec = *self.clocks.get(id as usize).ok_or(Status::NotFound)?;
        let state = self.hw.state(id)?;
        // RP1 needs every exposed clock running at its fixed rate; anything else is a fault.
        if state.rate_hz != spec.rate_hz || !state.enabled {
            return Err(Status::HardwareError);
        }
        Ok((spec, state))
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
                let (spec, state) = self.clock(a[0])?;
                Response::named(&[u32::from(state.enabled)], spec.name, &[0])
            }
            (CLOCK_PROTOCOL, 4) => {
                let (spec, _) = self.clock(a[0])?;
                match a[1] {
                    // One discrete rate, none remaining.
                    0 => Response::ok(&[1, spec.rate_hz, 0]),
                    1 => Response::ok(&[0]),
                    _ => return Err(Status::InvalidParameters),
                }
            }
            (CLOCK_PROTOCOL, 5) => {
                // flags: bit0 async (not offered), bits 3:2 round up/down/auto (harmless here).
                if a[0] & !0xc != 0 {
                    return Err(Status::InvalidParameters);
                }
                let (spec, _) = self.clock(a[1])?;
                if a[2] != spec.rate_hz || a[3] != 0 {
                    return Err(Status::Denied);
                }
                Response::ok(&[])
            }
            (CLOCK_PROTOCOL, 6) => {
                let (_, state) = self.clock(a[0])?;
                Response::ok(&[state.rate_hz, 0])
            }
            (CLOCK_PROTOCOL, 7) => {
                if a[1] & !1 != 0 {
                    return Err(Status::InvalidParameters);
                }
                self.clock(a[0])?;
                // The vote only; the clock stays on for RP1.
                if a[1] & 1 != 0 {
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

/// Physical clock state from the registers adopted by [`crate::clocks`] (read twice; any
/// difference from the admitted tuple reads as a hardware error).
pub struct Rp1Clocks {
    _private: (),
}

impl ClockHardware for Rp1Clocks {
    fn state(&mut self, id: u32) -> Result<PhysicalState, Status> {
        match id {
            // clk_sys/2 from PLL_SYS PRIM (1 GHz / 5 / 1 / 2); enabled = PRIM PRI_PH bit 4 (PoC).
            0 if crate::clocks::running_tuple_admitted() => Ok(PhysicalState {
                rate_hz: 100_000_000,
                enabled: read(0x4002_0010) & 0x10 != 0,
            }),
            1 if crate::clocks::uart_clock_is_50mhz() => Ok(PhysicalState {
                rate_hz: 50_000_000,
                enabled: true,
            }),
            0 | 1 => Err(Status::HardwareError),
            _ => Err(Status::NotFound),
        }
    }
}

/// The SCMI server on proc0. Created by [`crate::clocks::Authority::adopt`].
pub struct Service {
    server: Server<'static, Rp1Clocks>,
    transport: Rp1Transport,
}

impl Service {
    /// # Safety
    /// One instance, proc0, holding clock authority; no other user of mailbox channel 0 or the
    /// shmem area on RP1.
    pub(crate) unsafe fn new() -> Self {
        let mut transport = Rp1Transport { _private: () };
        reset_channel(&mut transport);
        Self {
            server: Server::new(RP1_CLOCKS, Rp1Clocks { _private: () }),
            transport,
        }
    }

    /// Answers one pending request, if any. Call from proc0's main loop; never blocks.
    pub fn poll(&mut self) -> Option<Event> {
        serve(&mut self.server, &mut self.transport)
    }

    pub fn votes(&self) -> u32 {
        self.server.votes()
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use std::vec::Vec;

    struct Hw(PhysicalState);
    impl ClockHardware for Hw {
        fn state(&mut self, id: u32) -> Result<PhysicalState, Status> {
            match id {
                0 => Ok(PhysicalState {
                    rate_hz: 100_000_000,
                    ..self.0
                }),
                1 => Ok(self.0),
                _ => Err(Status::NotFound),
            }
        }
    }
    fn server() -> Server<'static, Hw> {
        Server::new(
            RP1_CLOCKS,
            Hw(PhysicalState {
                rate_hz: 50_000_000,
                enabled: true,
            }),
        )
    }
    fn req(s: &mut Server<Hw>, p: u8, m: u8, a: &[u32]) -> Vec<u32> {
        let r = s.handle((0x2a << 18) | (u32::from(p) << 10) | u32::from(m), a);
        r.words[..r.len].to_vec()
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
    fn clock_discovery_and_rates() {
        let mut s = server();
        assert_eq!(req(&mut s, 0x14, 1, &[]), [0, 2]);
        let attr = req(&mut s, 0x14, 3, &[1]);
        assert_eq!(attr.len(), 7);
        assert_eq!(attr[..3], [0, 1, u32::from_le_bytes(*b"uart")]);
        assert_eq!(req(&mut s, 0x14, 4, &[0, 0]), [0, 1, 100_000_000, 0]);
        assert_eq!(req(&mut s, 0x14, 6, &[1]), [0, 50_000_000, 0]);
        assert_eq!(req(&mut s, 0x14, 6, &[2])[0] as i32, -4);
        assert_eq!(req(&mut s, 0x14, 2, &[9])[0] as i32, -1);
    }

    #[test]
    fn rate_change_is_denied_and_votes_never_stop_the_clock() {
        let mut s = server();
        assert_eq!(req(&mut s, 0x14, 5, &[0, 1, 48_000_000, 0])[0] as i32, -3);
        assert_eq!(req(&mut s, 0x14, 5, &[0, 1, 50_000_000, 0]), [0]);
        assert_eq!(req(&mut s, 0x14, 5, &[1, 1, 50_000_000, 0])[0] as i32, -2);
        assert_eq!(req(&mut s, 0x14, 7, &[1, 1]), [0]);
        assert_eq!(s.votes(), 2);
        assert_eq!(req(&mut s, 0x14, 7, &[1, 0]), [0]);
        assert_eq!(s.votes(), 0);
        // Still on after Linux dropped its vote.
        assert_eq!(req(&mut s, 0x14, 3, &[1])[1], 1);
        assert_eq!(req(&mut s, 0x14, 7, &[1, 2])[0] as i32, -2);
    }

    #[test]
    fn drift_and_bad_headers_fail_closed() {
        let mut s = server();
        s.hw.0.rate_hz = 48_000_000;
        assert_eq!(req(&mut s, 0x14, 6, &[1])[0] as i32, -9);
        assert_eq!(req(&mut s, 0x14, 7, &[1, 1])[0] as i32, -9);
        assert_eq!(s.votes(), 0);
        assert_eq!(s.handle(1 << 8, &[]).status(), -10);
        assert_eq!(req(&mut s, 0x14, 6, &[1, 0])[0] as i32, -2);
        assert_eq!(req(&mut s, 0x15, 0, &[])[0] as i32, -1);
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
