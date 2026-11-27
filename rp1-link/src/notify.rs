//! How the two sides tell each other that a ring has data. Each endpoint carries its notifier
//! as a type parameter, chosen by the declaration (`notify = "poll"` or `"irq"`), and legacy
//! notifiers are zero-sized. [`Poll`] compiles to nothing.
//!
//! | Direction | Notifier | Signal |
//! |---|---|---|
//! | RP1 -> Linux | [`HostIrq`] | one pulse of MSI-X vector `rp1_proto::irq::VECTOR` |
//! | Linux -> RP1 | [`Doorbell`] | the `PROCn_INTF` force bit of a bank-0 pin |
//!
//! # RP1 -> Linux: [`HostIrq`]
//!
//! The interrupt leaves hwirq 58 (rp1-mailbox / SCMI) alone: it pulses the `MSIX_CFG.TEST` bit
//! of the MSI-X vector no Linux driver uses. Linux receives it on the `rp1:irq` UIO node
//! (`read()` / `poll()` on `/dev/uioN`, then `write(1)` to re-enable).
//!
//! The core that declares `host_irq = true` owns it. TEST is ORed with the vector's own
//! hardware source, TIMER alarm 3 (`TIMER.INTS` bit 3). `host_irq` owns `TIMER.ALARM3` (the
//! declaration rejects any other owner), and nothing in rp1-hal unmasks or forces it.
//! Firmware that ran earlier (or a previous image on this RP1) might have, so
//! [`HostIrq::new`] takes the source over: it masks and un-forces alarm 3 and clears its raw
//! flag, leaving TEST as the only thing that can raise the vector (HW 2026-11-05: forcing
//! `TIMER.INTF` bit 3 raises the vector once and then swallows TEST pulses until released).
//!
//! Delivery is a hint, not a message: Linux's receiver must re-check the data it waits for
//! after every wake and after a bounded timeout. The vector is edge-triggered, so
//! - a pulse while Linux has not enabled the vector (before the UIO's first `write(1)`) is lost;
//! - pulses while the UIO is disabled (between a wake and the next `write(1)`) merge into one
//!   pending wake (lazy disable);
//! - an asserted TIMER alarm 3 interrupt would swallow pulses (above).
//!
//! # Linux -> RP1: [`Doorbell`]
//!
//! Linux sets the force bit through the SET alias of `PROCn_INTF` and the bit stays set until
//! the reader acknowledges it (a level, protocol.md §3). Linux sets it on every message that
//! finds the reader asleep, so the reader's handler acks first, then drains every ring that
//! shares the doorbell, and raises the doorbell itself when a drain ran out of budget
//! ([`Progress::More`]).
use crate::mmio::{self, read, write};
use rp1_proto::{
    Error, Message,
    irq::VECTOR,
    shm::{
        self, Progress,
        wake::{Publisher, Waiter, Words},
    },
    wake::{BootEpoch, Session, SessionError, mailbox},
};

/// PCIE_APBS MSIX_CFG(VECTOR), with the set and clear aliases.
const MSIX_CFG: usize = 0x4010_8008 + 4 * VECTOR as usize;
const MSIX_CFG_SET: usize = MSIX_CFG + 0x800;
const MSIX_CFG_CLR: usize = MSIX_CFG + 0xc00;
const MSIX_CFG_TEST: u32 = 1 << 1;
const TIMER0_VECTOR: u32 = 26;

// rp1-regdb `timer.toml`: TIMER at 0x400a_c000, atomic clear alias at +0x3000;
// INTS = (INTR & INTE) | INTF; INTS bit N is alarm N's interrupt (device.toml: TIMER_N = 26 + N).
const TIMER_BASE: usize = 0x400a_c000;
const ATOMIC_CLR: usize = 0x3000;
/// `INTR`: raw alarm flags, write-one-to-clear (regdb: verified_write = "hardware").
const TIMER_INTR: usize = TIMER_BASE + 0x34;
/// `INTE`: alarm interrupt enable, plain RW (regdb: write unverified; cleared through the
/// atomic clear alias, as `timer::Alarm::disable_interrupt` does).
const TIMER_INTE: usize = TIMER_BASE + 0x38;
/// `INTF`: alarm interrupt force. regdb marks write semantics unverified; on HW (2026-11-05,
/// devmem) writing 0x8 forced bit 3 and writing 0 released it, i.e. plain RW. The atomic clear
/// alias clears bit 3 without touching the other alarms' bits under plain-RW semantics.
const TIMER_INTF: usize = TIMER_BASE + 0x3c;
const TIMER_INTS: usize = TIMER_BASE + 0x40;
const ALARM_BIT: u32 = 1 << (VECTOR - TIMER0_VECTOR);

/// The notifier traits' methods: only endpoints call them, and only generated code makes the
/// notifiers that endpoints carry.
pub(crate) mod sealed {
    use super::{Error, Message, Progress, shm};

    /// Every method takes one. Its field is private to this crate and nothing returns one, so
    /// outside code cannot call the methods, not even through a public `Notify` / `Wake` bound.
    pub struct Token(pub(crate) ());

    pub trait Claim {
        fn claim(_: Token) -> Self;
    }

    pub trait Notify {
        /// Wake the reader of `ring` if it sleeps. Returns whether it notified.
        fn wake<T: Message>(&mut self, _: Token, ring: &mut shm::RingTx<T>) -> bool;
        fn wake_requested<T: Message>(&mut self, _: Token, _: &shm::RingTx<T>) -> bool {
            false
        }
    }

    pub trait Wake {
        /// Make the ring just attached notifiable: the first message wakes this reader.
        fn opened<T: Message>(&mut self, _: Token, ring: &mut shm::RingRx<T>);
        fn try_drain<T: Message>(
            &mut self,
            _: Token,
            ring: &mut shm::RingRx<T>,
            max: u32,
            f: impl FnMut(T, Option<bool>) -> Result<(), Error>,
        ) -> Result<Progress, Error>;
    }
}

/// How an RP1 writer tells Linux that data arrived: [`Poll`] or [`HostIrq`].
pub trait Notify: sealed::Notify {}
/// How Linux wakes an RP1 reader: [`Poll`] or a [`Doorbell`].
pub trait Wake: sealed::Wake {}

/// No notification: the peer polls. Costs nothing, and a `Poll` reader never arms its ring.
#[derive(Clone, Copy, Debug)]
pub struct Poll;

impl Notify for Poll {}
impl sealed::Claim for Poll {
    fn claim(_: sealed::Token) -> Self {
        Poll
    }
}
impl sealed::Notify for Poll {
    #[inline(always)]
    fn wake<T: Message>(&mut self, _: sealed::Token, _: &mut shm::RingTx<T>) -> bool {
        false
    }
}
impl Wake for Poll {}
impl sealed::Wake for Poll {
    #[inline(always)]
    fn opened<T: Message>(&mut self, _: sealed::Token, _: &mut shm::RingRx<T>) {}
    #[inline(always)]
    fn try_drain<T: Message>(
        &mut self,
        _: sealed::Token,
        ring: &mut shm::RingRx<T>,
        max: u32,
        mut f: impl FnMut(T, Option<bool>) -> Result<(), Error>,
    ) -> Result<Progress, Error> {
        for _ in 0..max.max(1) {
            let Some(value) = ring.pop()? else {
                return Ok(Progress::Idle);
            };
            f(value, None)?;
        }
        Ok(Progress::More)
    }
}

/// The right to raise the RP1 -> Linux interrupt. The generated resources give it to the core
/// that declares `host_irq = true` (there is exactly one) and copy it into that core's
/// `notify = "irq"` endpoints, which pulse it from `send*` / `flush`.
///
/// A pulse is two stores (set, then clear TEST) inside one critical section, so a handler on
/// this core cannot interleave its own pulse: TEST set twice in a row would give Linux one edge
/// for two wakes. The section masks user priorities only. The platform's priority 0 / 1 handlers
/// (PERSTn, SCMI) never pulse and cannot reach a `HostIrq`, which lives in user code.
#[derive(Clone, Copy)]
pub struct HostIrq {
    _private: core::marker::PhantomData<core::cell::Cell<()>>,
}

impl HostIrq {
    /// Takes TIMER alarm 3 over as the vector's hardware source, so only TEST pulses raise it:
    /// mask its interrupt (`INTE` bit 3), release a force (`INTF` bit 3), then clear a latched
    /// raw flag (`INTR` bit 3). Masking first means a pending raw flag cannot re-raise the
    /// source between the steps. The alarm itself is not disarmed (`ARMED` W1C is unverified):
    /// with `INTE` bit 3 clear its firing only sets `INTR`, which does not reach `INTS`.
    ///
    /// # Safety
    /// Generated resources only: the single owner of `MSIX_CFG(rp1_proto::irq::VECTOR).TEST`
    /// and of `TIMER.ALARM3`.
    pub unsafe fn new() -> Self {
        write(TIMER_INTE + ATOMIC_CLR, ALARM_BIT);
        write(TIMER_INTF + ATOMIC_CLR, ALARM_BIT);
        write(TIMER_INTR, ALARM_BIT);
        Self {
            _private: core::marker::PhantomData,
        }
    }

    /// Wake Linux about data that is not in an endpoint and lives in RP1 SRAM only (the link
    /// page, shm regions in shared SRAM): a barrier orders the SRAM writes before the pulse.
    /// Linux reads SRAM through the same RP1 bus, so nothing else is needed.
    ///
    /// For data in Linux DDR, first call [`crate::flush_window`] to drain posted writes.
    #[inline(always)]
    pub fn pulse_after_sram_write(self) {
        rp1_proto::barrier();
        self.pulse();
    }

    /// Out of line in shared SRAM (`cold` stops automatic ISRAM placement): one call per reader
    /// sleep, and the critical section stays out of ISRAM callers.
    #[rp1_rt::cold]
    fn pulse(self) {
        mmio::section(|| {
            write(MSIX_CFG_SET, MSIX_CFG_TEST);
            write(MSIX_CFG_CLR, MSIX_CFG_TEST);
        });
    }
}

impl Notify for HostIrq {}
impl sealed::Claim for HostIrq {
    fn claim(_: sealed::Token) -> Self {
        Self {
            _private: core::marker::PhantomData,
        }
    }
}
impl sealed::Notify for HostIrq {
    /// [`shm::RingTx::wake_needed`] reads `head` back through the shm window (completing the
    /// posted slot and `head` writes), then reads the reader's sleep counter from the link page:
    /// it pulses only for a sleep not yet notified, so a reader that keeps up costs no
    /// interrupts and a sleeping one gets one (protocol.md §2.3). One PCIe round trip per call.
    #[inline(always)]
    fn wake<T: Message>(&mut self, _: sealed::Token, ring: &mut shm::RingTx<T>) -> bool {
        let asleep = ring.wake_needed();
        if asleep {
            self.pulse();
        }
        asleep
    }
}

/// Single core-local owner of the experimental link-wake vector.
#[doc(hidden)]
pub struct VectorOwner {
    terminal_epochs: [u32; 2],
    _local: core::marker::PhantomData<core::cell::Cell<()>>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VectorError {
    SourceHigh,
    TestStuck,
}

impl VectorOwner {
    /// # Safety
    /// Generated v3 resources only. Exactly one core owns ALARM3 and this TEST bit,
    /// and no legacy HostIrq may be used with them.
    pub unsafe fn __new() -> Self {
        unsafe { HostIrq::new() };
        write(MSIX_CFG_CLR, MSIX_CFG_TEST);
        Self {
            terminal_epochs: [0; 2],
            _local: core::marker::PhantomData,
        }
    }

    /// Check the low baseline after takeover. Both reads complete at the respective banks.
    pub fn ready(&self) -> Result<(), VectorError> {
        Self::check_source()
    }

    fn check_source() -> Result<(), VectorError> {
        if read(TIMER_INTS) & ALARM_BIT != 0 {
            return Err(VectorError::SourceHigh);
        }
        if read(MSIX_CFG) & MSIX_CFG_TEST != 0 {
            return Err(VectorError::TestStuck);
        }
        Ok(())
    }

    /// Pulse after the notification commit's same-window readback. A successful pulse
    /// confirms the local source went high and low. The listener may still miss the edge.
    pub fn pulse(&mut self) -> Result<(), VectorError> {
        Self::pulse_owned()
    }

    // Generated v3 ring publishers execute on this same core. The section
    // serializes all TEST pulses, including calls from a local interrupt.
    #[rp1_rt::isram]
    fn pulse_owned() -> Result<(), VectorError> {
        mmio::section(|| {
            Self::check_source()?;
            write(MSIX_CFG_SET, MSIX_CFG_TEST);
            let high = read(MSIX_CFG) & MSIX_CFG_TEST != 0;
            write(MSIX_CFG_CLR, MSIX_CFG_TEST);
            let low = read(MSIX_CFG) & MSIX_CFG_TEST == 0;
            if high && low {
                Ok(())
            } else {
                Err(VectorError::TestStuck)
            }
        })
    }

    /// Delivery fixture entry, usable before attach. Scan on startup and after each event;
    /// a local pulse failure leaves the record eligible on the next explicit call.
    /// The caller must have a live PCIe link. Only SRAM and the owned vector are accessed.
    pub fn pulse_terminals(&mut self) -> Result<bool, VectorError> {
        let epochs = self.terminal_epochs;
        self.pulse_observed_terminals(|core| {
            let terminal = rp1_rt::status::terminal(core)?;
            if epochs[core as usize] != terminal.epoch {
                publish_terminal_ticket();
            }
            Some(terminal)
        })
    }

    fn pulse_observed_terminals(
        &mut self,
        read_terminal: impl Fn(u8) -> Option<rp1_rt::status::Terminal>,
    ) -> Result<bool, VectorError> {
        let mut epochs = self.terminal_epochs;
        for (core, epoch) in epochs.iter_mut().enumerate() {
            if let Some(terminal) = read_terminal(core as u8) {
                *epoch = terminal.epoch;
            }
        }
        if epochs == self.terminal_epochs {
            return Ok(false);
        }
        self.pulse()?;
        self.terminal_epochs = epochs;
        Ok(true)
    }
}

fn publish_terminal_ticket() {
    let link = rp1_proto::LINK_START as usize;
    let live = || BootEpoch::new(read(link + mailbox::BOOT_EPOCH));
    if let Ok(epoch) = live()
        && let Ok(mut publisher) = unsafe { Words::link(link as *mut u8).publisher(epoch, live) }
    {
        let _ = publisher.publish(|| {
            let _ = read(link + mailbox::BOOT_EPOCH);
        });
    }
}

type LiveSession = fn() -> Result<Session, SessionError>;

/// The v3 RX ticket publisher, called only by the generated vector owner's rings.
#[doc(hidden)]
pub struct TicketIrq {
    publisher: Publisher<Session, LiveSession>,
    data: *mut u8,
    pulse_pending: bool,
    pulse_failures: u32,
}

impl TicketIrq {
    pub(crate) unsafe fn attach(data: *mut u8, ctrl: *mut u8) -> Result<Self, Error> {
        let live: LiveSession = crate::generated::wake_session;
        let scope = live().map_err(|_| Error::Reset)?;
        let publisher =
            unsafe { Words::rx(data, ctrl).publisher(scope, live) }.map_err(|_| Error::Reset)?;
        Ok(Self {
            publisher,
            data,
            pulse_pending: false,
            pulse_failures: 0,
        })
    }

    pub(crate) fn retry(&mut self) -> bool {
        if !self.pulse_pending {
            return false;
        }
        if VectorOwner::pulse_owned().is_err() {
            self.pulse_failures = self.pulse_failures.saturating_add(1);
            return false;
        }
        self.pulse_pending = false;
        true
    }

    pub(crate) fn pulse_failures(&self) -> u32 {
        self.pulse_failures
    }
}

impl Notify for TicketIrq {}
impl sealed::Notify for TicketIrq {
    fn wake<T: Message>(&mut self, _: sealed::Token, _: &mut shm::RingTx<T>) -> bool {
        match self.publisher.publish(|| {
            let _ = unsafe { self.data.cast::<u32>().read_volatile() };
        }) {
            Ok(fresh) => self.pulse_pending |= fresh,
            Err(_) => return false,
        }
        self.retry()
    }

    #[inline(always)]
    fn wake_requested<T: Message>(&mut self, _: sealed::Token, ring: &shm::RingTx<T>) -> bool {
        // Complete the DDR reply before sending the separate MSI-X transaction.
        rp1_proto::barrier();
        let _ = unsafe { self.data.cast::<u32>().read_volatile() };
        rp1_proto::barrier();
        // A caller that already took this reply cannot be waiting for it.
        self.pulse_pending |= !ring.all_consumed();
        self.retry()
    }
}

/// The v3 downlink reader pairs the existing force-level ISR with v3 RX tickets.
#[doc(hidden)]
pub struct TicketDoorbell<const PIN: u8, const CORE: u8> {
    ticket: Option<Waiter<Session, LiveSession>>,
    armed: bool,
}

impl<const PIN: u8, const CORE: u8> TicketDoorbell<PIN, CORE> {
    pub(crate) unsafe fn attach(data: *mut u8, ctrl: *mut u8, direct: bool) -> Result<Self, Error> {
        if direct {
            return Ok(Self {
                ticket: None,
                armed: false,
            });
        }
        let live: LiveSession = crate::generated::wake_session;
        let scope = live().map_err(|_| Error::Reset)?;
        let ticket =
            unsafe { Words::rx(data, ctrl).waiter(scope, live) }.map_err(|_| Error::Reset)?;
        Ok(Self {
            ticket: Some(ticket),
            armed: false,
        })
    }
}

impl<const PIN: u8, const CORE: u8> Wake for TicketDoorbell<PIN, CORE> {}
impl<const PIN: u8, const CORE: u8> sealed::Wake for TicketDoorbell<PIN, CORE> {
    fn opened<T: Message>(&mut self, _: sealed::Token, ring: &mut shm::RingRx<T>) {
        if let Some(ticket) = &mut self.ticket {
            self.armed = ticket.arm().is_ok();
        }
        if (self.armed || self.ticket.is_none()) && ring.available() != 0 {
            unsafe { Doorbell::<PIN, CORE>::new() }.raise();
        }
    }

    fn try_drain<T: Message>(
        &mut self,
        _: sealed::Token,
        ring: &mut shm::RingRx<T>,
        max: u32,
        mut handle: impl FnMut(T, Option<bool>) -> Result<(), Error>,
    ) -> Result<Progress, Error> {
        let Some(ticket) = &mut self.ticket else {
            return drain_requests(ring, max, handle);
        };
        let reset = |_| Error::Reset;
        let mut budget = max.max(1);
        loop {
            if ring.available() != 0 {
                while budget != 0 {
                    let Some(value) = ring.pop()? else { break };
                    handle(value, None)?;
                    budget -= 1;
                }
                if self.armed {
                    // The request was observed under this ticket. Complete the reply
                    // first so its pulse can overlap the downlink ACK and re-arm.
                    ticket.acknowledge().map_err(reset)?;
                    ticket.cancel().map_err(reset)?;
                    self.armed = false;
                }
                if budget == 0 {
                    return Ok(Progress::More);
                }
            }
            if self.armed && ticket.pending().map_err(reset)? {
                // A pulse with no observed work can still consume this ticket.
                // Re-arm and rescan so a subsequent publication can notify us.
                ticket.cancel().map_err(reset)?;
                self.armed = false;
            }
            if !self.armed {
                ticket.arm().map_err(reset)?;
                self.armed = true;
                if ring.available() != 0 {
                    continue;
                }
            }
            return Ok(Progress::Idle);
        }
    }
}

#[rp1_rt::isram]
fn drain_requests<T: Message>(
    ring: &mut shm::RingRx<T>,
    max: u32,
    mut handle: impl FnMut(T, Option<bool>) -> Result<(), Error>,
) -> Result<Progress, Error> {
    // The host forces after each head commit. The ISR clears the level
    // and sets an event, including when it races this drain.
    for _ in 0..max.max(1) {
        let Some((value, notify)) = ring.pop_request()? else {
            return Ok(Progress::Idle);
        };
        handle(value, Some(notify))?;
    }
    Ok(Progress::More)
}

/// Linux's doorbell for core CORE: the `PROCn_INTF` force bit of bank-0 pin PIN, declared as
/// `[procN] doorbell = PIN`. The generated resources give it to the core as `doorbell` and copy
/// it into the core's `notify = "irq"` linux-to-rp1 endpoints. The pin's pad stays unused.
#[derive(Clone, Copy)]
pub struct Doorbell<const PIN: u8, const CORE: u8> {
    _private: core::marker::PhantomData<core::cell::Cell<()>>,
}

impl<const PIN: u8, const CORE: u8> Doorbell<PIN, CORE> {
    const BIT: u32 = {
        assert!(
            PIN < 28 && CORE < 2,
            "a doorbell is a bank-0 pin of proc0 or proc1"
        );
        1 << PIN
    };
    /// `PROCn_INTF` of IO_BANK0, with its atomic SET and CLR aliases.
    const INTF: usize = 0x400d_0108 + 0xc * CORE as usize;
    const INTF_SET: usize = Self::INTF + 0x2000;
    const INTF_CLR: usize = Self::INTF + 0x3000;

    /// # Safety
    /// Generated resources only: core CORE declares `doorbell = PIN`.
    pub const unsafe fn new() -> Self {
        let _ = Self::BIT;
        Self {
            _private: core::marker::PhantomData,
        }
    }
    /// Whether the force bit is set (the doorbell rang and was not acknowledged yet).
    pub fn is_pending(self) -> bool {
        read(Self::INTF) & Self::BIT != 0
    }
    /// Drop the force. Once, at the top of the IO_BANK0 handler, before draining any ring that
    /// shares this doorbell: an ack between two drains could drop a ring's notification.
    pub fn ack(self) {
        write(Self::INTF_CLR, Self::BIT);
    }
    /// Acknowledge the doorbell and wake threads waiting for the link on either core.
    ///
    /// Call first in a custom `IO_BANK0` handler on this core, then service the GPIO pins.
    #[inline]
    pub fn ack_and_signal(self) {
        self.ack();
        rp1_rt::send_event();
    }

    /// Experimental v3 ISR entry: complete the CLR in this bank before signaling.
    #[doc(hidden)]
    pub fn __ack_and_signal_wake(self) {
        self.ack();
        let _ = read(Self::INTF);
        // send_event completes the bank read before signaling.
        rp1_rt::send_event();
    }

    /// Force this core's own doorbell bit: the handler runs again (after [`Progress::More`]).
    /// Raise it once after handing the endpoint to its handler ([`RingRx`](crate::RingRx)).
    pub fn raise(self) {
        write(Self::INTF_SET, Self::BIT);
    }
}

impl<const PIN: u8, const CORE: u8> Wake for Doorbell<PIN, CORE> {}
impl<const PIN: u8, const CORE: u8> sealed::Claim for Doorbell<PIN, CORE> {
    fn claim(_: sealed::Token) -> Self {
        unsafe { Self::new() }
    }
}
impl<const PIN: u8, const CORE: u8> sealed::Wake for Doorbell<PIN, CORE> {
    /// Arm the ring. Messages that Linux sent before the arm are already there and rang no
    /// doorbell, so raise it for them: the handler drains them and arms again.
    fn opened<T: Message>(&mut self, _: sealed::Token, ring: &mut shm::RingRx<T>) {
        if !ring.prepare_wait() {
            self.raise();
        }
    }
    #[inline(always)]
    fn try_drain<T: Message>(
        &mut self,
        _: sealed::Token,
        ring: &mut shm::RingRx<T>,
        max: u32,
        mut f: impl FnMut(T, Option<bool>) -> Result<(), Error>,
    ) -> Result<Progress, Error> {
        ring.try_drain(max, |value| f(value, None))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timer_registers_match_the_pac() {
        use rp1_hal::pac::{TIMER, generated::timer::RegisterBlock};

        // A zeroed register block keeps the test's references in mapped host memory.
        let block = unsafe { core::mem::zeroed::<RegisterBlock>() };
        let base = &raw const block as usize;
        let addresses = [
            block.intr() as *const _ as usize,
            block.inte() as *const _ as usize,
            block.intf() as *const _ as usize,
        ];
        let offsets = addresses.map(|address| address - base);
        assert_eq!(TIMER::PTR as usize, TIMER_BASE);
        assert_eq!(
            offsets.map(|offset| TIMER_BASE + offset),
            [TIMER_INTR, TIMER_INTE, TIMER_INTF]
        );
    }

    /// The alarm taken over is the one whose interrupt is the host vector (rp1-regdb
    /// `device.toml`: TIMER_3 = 29).
    #[test]
    fn takes_over_the_vector_source() {
        assert_eq!(rp1_hal::pac::Interrupt::TIMER_0 as u32, TIMER0_VECTOR);
        assert_eq!(rp1_hal::pac::Interrupt::TIMER_3 as u32, VECTOR);
        assert_eq!(ALARM_BIT, 1 << 3);
    }

    /// Both stores of one pulse are inside one critical section, set before clear.
    #[test]
    fn a_pulse_is_set_then_clear_in_one_section() {
        use crate::mmio::fake::{self, Access};
        fake::take();
        let irq = <HostIrq as sealed::Claim>::claim(sealed::Token(()));
        irq.pulse_after_sram_write();
        assert_eq!(
            fake::take(),
            [
                Access::Enter,
                Access::Write(MSIX_CFG_SET, MSIX_CFG_TEST),
                Access::Write(MSIX_CFG_CLR, MSIX_CFG_TEST),
                Access::Leave,
            ]
        );
        assert_eq!(MSIX_CFG, 0x4010_8008 + 4 * 29);
    }

    #[test]
    fn v3_owner_confirms_the_pulse_and_preserves_terminal_for_retry() {
        use crate::mmio::fake::{self, Access::*};
        use rp1_proto::status::CoreState;
        use rp1_rt::status::Terminal;
        fake::trace_reads();
        fake::alias(MSIX_CFG_CLR, MSIX_CFG, false);
        fake::set(MSIX_CFG, MSIX_CFG_TEST);
        let mut owner = unsafe { VectorOwner::__new() };
        assert_eq!(
            fake::take(),
            [
                Write(TIMER_INTE + ATOMIC_CLR, ALARM_BIT),
                Write(TIMER_INTF + ATOMIC_CLR, ALARM_BIT),
                Write(TIMER_INTR, ALARM_BIT),
                Write(MSIX_CFG_CLR, MSIX_CFG_TEST),
            ]
        );

        // A terminal from proc0 can predate the proc1 owner's first scan. No DDR
        // accesses, even for a terminal with no ring or Ready yet.
        let terminal = |core| {
            (core == 0).then_some(Terminal {
                epoch: 2,
                state: CoreState::Fault,
                reason: 3,
            })
        };
        fake::set(TIMER_INTS, ALARM_BIT);
        assert_eq!(
            owner.pulse_observed_terminals(terminal),
            Err(VectorError::SourceHigh)
        );
        assert_eq!(fake::take(), [Enter, Read(TIMER_INTS), Leave]);
        fake::set(TIMER_INTS, 0);
        // TEST SET without the alias models a source that failed to rise.
        assert_eq!(
            owner.pulse_observed_terminals(terminal),
            Err(VectorError::TestStuck)
        );
        fake::take();
        fake::alias(MSIX_CFG_SET, MSIX_CFG, true);
        assert_eq!(owner.pulse_observed_terminals(terminal), Ok(true));
        assert_eq!(
            fake::take(),
            [
                Enter,
                Read(TIMER_INTS),
                Read(MSIX_CFG),
                Write(MSIX_CFG_SET, MSIX_CFG_TEST),
                Read(MSIX_CFG),
                Write(MSIX_CFG_CLR, MSIX_CFG_TEST),
                Read(MSIX_CFG),
                Leave,
            ]
        );
        assert_eq!(owner.pulse_observed_terminals(terminal), Ok(false));
        assert!(fake::take().is_empty());
    }

    #[test]
    fn v3_reply_retries_a_locally_failed_pulse_with_the_same_ticket() {
        use crate::mmio::fake;
        use rp1_proto::{LINK_START, link};

        let window = LINK_START as usize;
        fake::set(window + link::SHM_STATUS, link::SHM_READY);
        fake::set(window + link::SHM_GENERATION, 2);
        fake::set(window + mailbox::BOOT_EPOCH, 2);
        fake::set(TIMER_INTS, ALARM_BIT);
        fake::alias(MSIX_CFG_SET, MSIX_CFG, true);
        fake::alias(MSIX_CFG_CLR, MSIX_CFG, false);
        let mut data = [0u32; 128];
        let mut ctrl = [0u32; 32];
        ctrl[shm::wake::RX_WAIT / 4] = 1;
        let mut ring = unsafe {
            shm::RingTx::attach(
                data.as_mut_ptr().cast(),
                512,
                ctrl.as_ptr().cast(),
                128,
                4,
                shm::Overflow::Reject,
            )
        }
        .unwrap();
        let mut irq =
            unsafe { TicketIrq::attach(data.as_mut_ptr().cast(), ctrl.as_mut_ptr().cast()) }
                .unwrap();
        ring.push(&9u32).unwrap();
        assert!(!sealed::Notify::wake(
            &mut irq,
            sealed::Token(()),
            &mut ring
        ));
        assert_eq!(data[shm::wake::RX_SEQ / 4], 1);
        assert_eq!(irq.pulse_failures(), 1);
        fake::set(TIMER_INTS, 0);
        assert!(sealed::Notify::wake(&mut irq, sealed::Token(()), &mut ring));
        assert!(!irq.retry());
        assert_eq!(irq.pulse_failures(), 1);
    }

    #[test]
    fn v3_requested_wake_skips_a_consumed_reply() {
        use crate::mmio::fake;
        use rp1_proto::{LINK_START, link};

        let window = LINK_START as usize;
        fake::set(window + link::SHM_STATUS, link::SHM_READY);
        fake::set(window + link::SHM_GENERATION, 2);
        fake::set(window + mailbox::BOOT_EPOCH, 2);
        fake::alias(MSIX_CFG_SET, MSIX_CFG, true);
        fake::alias(MSIX_CFG_CLR, MSIX_CFG, false);
        let mut data = [0u64; 64];
        let mut ctrl = [0u64; 16];
        let (data, ctrl) = (data.as_mut_ptr().cast(), ctrl.as_mut_ptr().cast());
        let mut tx =
            unsafe { shm::RingTx::attach(data, 512, ctrl, 128, 4, shm::Overflow::Reject) }.unwrap();
        let mut rx =
            unsafe { shm::RingRx::attach(data, 512, ctrl, 128, 4, shm::Overflow::Reject) }.unwrap();
        let mut irq = unsafe { TicketIrq::attach(data, ctrl) }.unwrap();
        tx.push(&17u32).unwrap();
        assert!(sealed::Notify::wake_requested(
            &mut irq,
            sealed::Token(()),
            &tx
        ));
        assert_eq!(rx.pop(), Ok(Some(17)));
        tx.push(&23).unwrap();
        assert_eq!(rx.pop(), Ok(Some(23)));
        assert!(!sealed::Notify::wake_requested(
            &mut irq,
            sealed::Token(()),
            &tx
        ));
    }

    #[test]
    fn v3_empty_doorbell_with_a_consumed_ticket_rearms_without_ack() {
        use crate::mmio::fake;
        use rp1_proto::{LINK_START, link};

        let window = LINK_START as usize;
        fake::set(window + link::SHM_STATUS, link::SHM_READY);
        fake::set(window + link::SHM_GENERATION, 2);
        fake::set(window + mailbox::BOOT_EPOCH, 2);
        let mut data = [0u64; 64];
        let mut ctrl = [0u64; 16];
        let mut ring = unsafe {
            shm::RingRx::<u32>::attach(
                data.as_ptr().cast(),
                512,
                ctrl.as_mut_ptr().cast(),
                128,
                4,
                shm::Overflow::Reject,
            )
        }
        .unwrap();
        let mut bell = unsafe {
            TicketDoorbell::<27, 1>::attach(
                data.as_mut_ptr().cast(),
                ctrl.as_mut_ptr().cast(),
                false,
            )
        }
        .unwrap();
        sealed::Wake::opened(&mut bell, sealed::Token(()), &mut ring);
        unsafe {
            data.as_mut_ptr()
                .cast::<u32>()
                .add(shm::wake::RX_SEQ / 4)
                .write_volatile(1);
        }
        assert_eq!(
            sealed::Wake::try_drain(&mut bell, sealed::Token(()), &mut ring, 4, |_, _| panic!(
                "empty receiver"
            ),),
            Ok(Progress::Idle)
        );
        assert_eq!(ctrl[shm::wake::RX_WAIT / 8], 3);
    }

    #[test]
    fn v3_doorbell_reads_its_bank_after_clear() {
        use crate::mmio::fake::{self, Access::*};
        type Bell = Doorbell<27, 1>;
        fake::trace_reads();
        fake::alias(Bell::INTF_CLR, Bell::INTF, false);
        fake::set(Bell::INTF, Bell::BIT);
        let bell = unsafe { Bell::new() };
        bell.__ack_and_signal_wake();
        assert_eq!(
            fake::take(),
            [Write(Bell::INTF_CLR, Bell::BIT), Read(Bell::INTF)]
        );
        assert!(!bell.is_pending());
    }

    #[test]
    fn doorbell_registers_match_the_pac() {
        use crate::mmio::fake::{self, Access};
        use rp1_hal::pac::{IO_BANK0, generated::io_bank0::RegisterBlock};

        let block = unsafe { core::mem::zeroed::<RegisterBlock>() };
        let base = &raw const block as usize;
        let intf = [
            block.proc0_intf() as *const _ as usize - base,
            block.proc1_intf() as *const _ as usize - base,
        ]
        .map(|offset| IO_BANK0::PTR as usize + offset);
        assert_eq!(intf, [Doorbell::<26, 0>::INTF, Doorbell::<27, 1>::INTF]);

        let bell = unsafe { Doorbell::<26, 0>::new() };
        fake::take();
        bell.raise();
        bell.ack();
        assert_eq!(
            fake::take(),
            [
                Access::Write(0x400d_2108, 1 << 26),
                Access::Write(0x400d_3108, 1 << 26),
            ]
        );
        assert!(!bell.is_pending());
        fake::set(0x400d_0108, 1 << 26);
        assert!(bell.is_pending());
        // Another pin's force is not this doorbell.
        assert!(!unsafe { Doorbell::<27, 0>::new() }.is_pending());
    }
}
