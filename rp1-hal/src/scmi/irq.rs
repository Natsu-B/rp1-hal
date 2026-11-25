//! SCMI on proc0 IRQ57. Init finishes writing the handler's DSRAM state and
//! published state word before ISER. Only IRQ57 writes them after ISER (rule E1).
//!
//! IRQ53 (priority 0) can preempt IRQ57. A serve time is "clean" when IRQ53's entry count is
//! the same before and after it, so `max_serve_clean_us` is SCMI's own time.

use super::{
    CHANNEL, ClockHardware, HOST_EVENTS_SET, PROC_EVENTS, PROC_EVENTS_CLR, RP1_CLOCKS, Registers,
    Rp1Clocks, SHMEM, Served, Server, Transport,
};
use crate::TIMER_RAWL;
use core::{
    mem::MaybeUninit,
    sync::atomic::{AtomicU32, Ordering::Relaxed},
};
use rp1_proto::status::{PLAT_SCMI_QUARANTINED as QUARANTINED, PLAT_SCMI_SERVING as SERVING};

const IRQ: usize = rp1_rt::pac::Interrupt::PROC_MISC as usize;
const IRQ_BIT: u32 = 1 << (IRQ % 32);
const ISER: usize = 0xe000_e100 + (IRQ / 32) * 4;
const ICER: usize = ISER + 0x80;
const ISPR: usize = ISER + 0x100;
const ICPR: usize = ISER + 0x180;
const IPR: usize = 0xe000_e400 + IRQ;
const PROC_GATE: usize = 0xf800_0008;
const PRIORITY: u8 = 1 << 5;
const MAX_EMPTY_STREAK: u8 = 16;
const STATUS_WORD: usize = rp1_proto::LINK_START as usize + rp1_proto::status::PLAT_SCMI;

/// The SCMI server state published by proc0, readable on either core.
pub use rp1_proto::status::ScmiState;

// Both cores can read the word. Plain atomic loads and stores need no shared-SRAM RMW.
fn status_word() -> &'static AtomicU32 {
    unsafe { AtomicU32::from_ptr(STATUS_WORD as *mut u32) }
}

/// Read proc0's published SCMI state.
pub fn scmi_state() -> ScmiState {
    #[cfg(all(target_arch = "arm", target_os = "none"))]
    let word = status_word().load(Relaxed);
    #[cfg(not(all(target_arch = "arm", target_os = "none")))]
    let word = 0;
    ScmiState::from_word(word)
}

/// A snapshot of proc0's IRQ57 diagnostics.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ScmiStats {
    pub requests: u32,
    pub duplicates: u32,
    pub spurious57: u32,
    /// The longest serve time, including any IRQ53 that preempted it.
    pub max_serve_all_us: u32,
    /// Entries that IRQ53 did not preempt.
    pub clean_n: u32,
    /// The longest serve time among the `clean_n` entries.
    pub max_serve_clean_us: u32,
}

#[derive(Default)]
struct Counters {
    stats: ScmiStats,
    streak: u8,
}

pub(crate) trait Io: Registers {
    fn set_priority(&mut self, value: u8);
    fn publish(&mut self, word: u32);
    fn barrier(&mut self);
    /// IRQ53's entry count ([`crate::platform::perstn`]).
    fn entries53(&mut self) -> u32;
}

struct Mailbox<'a, R>(&'a mut R);

impl<R: Io> Transport for Mailbox<'_, R> {
    fn take_event(&mut self) -> bool {
        if self.0.read(PROC_EVENTS) & CHANNEL == 0 {
            return false;
        }
        self.0.write(PROC_EVENTS_CLR, CHANNEL);
        true
    }
    fn read(&mut self, word: usize) -> u32 {
        self.0.read(SHMEM + 4 * word)
    }
    fn write(&mut self, word: usize, value: u32) {
        self.0.write(SHMEM + 4 * word, value);
    }
    fn barrier(&mut self) {
        self.0.barrier();
    }
    fn notify(&mut self) {
        self.0.write(HOST_EVENTS_SET, CHANNEL);
        self.0.barrier();
    }
}

fn pend_if_event(io: &mut impl Io) {
    if io.read(PROC_EVENTS) & CHANNEL != 0 {
        io.write(ISPR, IRQ_BIT);
    }
}

pub(crate) fn enable(io: &mut impl Io) {
    io.publish(SERVING);
    io.barrier();

    let gate = io.read(PROC_GATE);
    io.write(PROC_GATE, gate | 1);
    // Drain the posted gate write before arming the NVIC line.
    let _ = io.read(PROC_GATE);
    io.set_priority(PRIORITY);
    io.write(ICPR, IRQ_BIT);
    io.write(ISER, IRQ_BIT);
    // The event bit survives a closed gate or an IRQ pending bit cleared by ICPR.
    pend_if_event(io);
}

fn handle(server: &mut Server<'_, impl ClockHardware>, c: &mut Counters, io: &mut impl Io) {
    let entries53 = io.entries53();
    let start = io.read(TIMER_RAWL);
    let served = super::serve(server, &mut Mailbox(io));
    match served {
        Served::Request => {
            c.stats.requests = c.stats.requests.saturating_add(1);
            c.streak = 0;
        }
        Served::Duplicate => {
            c.stats.duplicates = c.stats.duplicates.saturating_add(1);
            c.streak = 0;
        }
        Served::NoEvent => {
            c.stats.spurious57 = c.stats.spurious57.saturating_add(1);
            c.streak += 1;
        }
    }
    let elapsed = io.read(TIMER_RAWL).wrapping_sub(start);
    if io.entries53() == entries53 {
        c.stats.clean_n = c.stats.clean_n.saturating_add(1);
        c.stats.max_serve_clean_us = c.stats.max_serve_clean_us.max(elapsed);
    }
    c.stats.max_serve_all_us = c.stats.max_serve_all_us.max(elapsed);
    if c.streak >= MAX_EMPTY_STREAK {
        io.write(ICER, IRQ_BIT);
        io.publish(QUARANTINED);
        io.barrier();
        return;
    }
    let requests = c.stats.requests.min(u16::MAX.into());
    io.publish(SERVING | (requests << 16));
    // A request posted after FREE can arrive before exception return without another IRQ entry.
    pend_if_event(io);
}

struct State {
    server: Server<'static, Rp1Clocks>,
    counters: Counters,
}

// Proc0-local DSRAM. Init writes it before ISER, then IRQ57 owns it. Proc0 thread readers
// mask interrupts while copying the stats.
static mut STATE: MaybeUninit<State> = MaybeUninit::uninit();

pub(crate) struct Mmio;

impl Registers for Mmio {
    fn read(&mut self, address: usize) -> u32 {
        super::read(address)
    }
    fn write(&mut self, address: usize, value: u32) {
        crate::write_relaxed(address, value);
    }
}

impl Io for Mmio {
    fn set_priority(&mut self, value: u8) {
        unsafe { (IPR as *mut u8).write_volatile(value) }
    }
    fn publish(&mut self, word: u32) {
        status_word().store(word, Relaxed);
    }
    fn barrier(&mut self) {
        rp1_proto::barrier();
    }
    fn entries53(&mut self) -> u32 {
        crate::platform::perstn::entries()
    }
}

fn prepare_server<H: ClockHardware>(io: &mut impl Io, clocks: H, held: u32) -> Server<'static, H> {
    super::reset_channel(&mut Mailbox(io));
    Server::new(RP1_CLOCKS, clocks).holding(held)
}

/// # Safety
/// Call once on proc0 with clock authority and IRQ57 disabled. The server must
/// have exclusive use of mailbox channel 0, its shared memory, and the
/// Linux-owned clock generators on RP1.
pub(crate) unsafe fn install(held: u32) {
    let state = State {
        server: prepare_server(&mut Mmio, Rp1Clocks { _private: () }, held),
        counters: Counters::default(),
    };
    unsafe { (&raw mut STATE).write(MaybeUninit::new(state)) };
}

pub(crate) fn stats() -> ScmiStats {
    #[cfg(all(target_arch = "arm", target_os = "none"))]
    let stats = rp1_rt::interrupt::__primask_free(|| {
        // Proc0's Pcie handle guarantees install ran. IRQ57 is masked here.
        unsafe { (*(&raw const STATE)).assume_init_ref().counters.stats }
    });
    #[cfg(not(all(target_arch = "arm", target_os = "none")))]
    let stats = ScmiStats::default();
    stats
}

#[rp1_rt::interrupt(proc0, __platform, sram)]
fn PROC_MISC() {
    let state = unsafe { (*(&raw mut STATE)).assume_init_mut() };
    handle(&mut state.server, &mut state.counters, &mut Mmio);
}

#[cfg(test)]
mod tests;
