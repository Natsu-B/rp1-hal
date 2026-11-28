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
        #[inline(always)]
        fn check(&self, _: Token) -> Result<(), Error> {
            crate::ring::window_ready()
        }

        /// Wake the reader of `ring` if it sleeps. Returns whether it notified.
        fn wake<T: Message>(&mut self, _: Token, ring: &mut shm::RingTx<T>) -> bool;
        /// Reserve up to `max` slots. A strict writer arms credit before rechecking Full.
        #[inline(always)]
        fn reserve<T: Message>(
            &mut self,
            _: Token,
            ring: &mut shm::RingTx<T>,
            max: u32,
        ) -> Result<u32, Error> {
            Ok(ring.writable().min(max))
        }

        /// Pulse for an armed, flagged reply unless the reader has already consumed it.
        fn wake_requested<T: Message>(&mut self, _: Token, _: &shm::RingTx<T>) -> bool {
            false
        }
    }

    pub trait Wake {
        #[inline(always)]
        fn check(&self, _: Token) -> Result<(), Error> {
            crate::ring::window_ready()
        }

        /// Make the ring just attached notifiable: the first message wakes this reader.
        fn opened<T: Message>(&mut self, _: Token, ring: &mut shm::RingRx<T>);
        fn publish_credit<T: Message>(
            &mut self,
            _: Token,
            _: &shm::RingRx<T>,
        ) -> Result<(), Error> {
            Ok(())
        }

        fn cancel_receive(&mut self, _: Token) -> Result<(), Error> {
            Ok(())
        }

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

/// Notify the vector owner after proc0 commits a startup result.
///
/// # Safety
/// Generated proc0 startup only; PIN and OWNER are the validated startup route.
#[doc(hidden)]
pub unsafe fn __kick_startup_owner<const PIN: u8, const OWNER: u8>() {
    if OWNER == 0 {
        deliver_startup();
    } else {
        type Kick<const P: u8> = Doorbell<P, 1>;
        write(Kick::<PIN>::INTF_SET, Kick::<PIN>::BIT);
        let _ = read(Kick::<PIN>::INTF);
        rp1_proto::barrier();
    }
}

/// Scan after taking ownership, even when reset cleared a kick that arrived first.
///
/// # Safety
/// Generated code on the proc1 vector owner, after taking over the vector.
#[doc(hidden)]
pub unsafe fn __startup_owner_enter() {
    deliver_startup();
}

/// # Safety
/// Generated proc1 vector-owner dispatcher only; this force belongs to startup.
#[doc(hidden)]
pub unsafe fn __startup_owner_irq<const PIN: u8>() {
    on_startup_kick::<PIN>(deliver_startup);
}

fn on_startup_kick<const PIN: u8>(deliver: impl FnOnce()) {
    let kick = unsafe { Doorbell::<PIN, 1>::new() };
    if kick.is_pending() {
        kick.__ack_and_signal_wake();
        deliver();
    }
}

#[rp1_rt::cold]
fn deliver_startup() {
    deliver_startup_from(
        rp1_proto::LINK_START as *mut u8,
        rp1_hal::platform::__startup_window_live,
    );
}

fn deliver_startup_from(link: *mut u8, window_live: impl Fn() -> bool) {
    // Entry and IRQ can both observe the commit. Serialize link-ticket publication
    // with local pulses, while leaving PERSTn and SCMI service available.
    mmio::section(|| {
        let word = |offset| unsafe { link.add(offset).cast::<u32>().read_volatile() };
        let status = || word(rp1_proto::link::SHM_STATUS);
        if matches!(status(), 0 | rp1_proto::link::SHM_WINDOW_LOST) || !window_live() {
            return;
        }
        let live = || {
            let epoch = BootEpoch::new(word(mailbox::BOOT_EPOCH))?;
            if epoch.word() != word(rp1_proto::status::EPOCH) {
                return Err(rp1_proto::wake::SessionError::Restarted);
            }
            Ok(epoch)
        };
        if let Ok(epoch) = live()
            && let Ok(mut publisher) = unsafe { Words::link(link).publisher(epoch, live) }
            && publisher
                .publish(|| {
                    let _ = status();
                })
                .unwrap_or(false)
            && window_live()
        {
            let _ = VectorOwner::pulse_owned();
        }
    });
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

pub(crate) type LiveSession = fn() -> Result<Session, SessionError>;

/// The v3 RX ticket publisher, called only by the generated vector owner's rings.
#[doc(hidden)]
pub struct TicketIrq {
    publisher: Publisher<Session, LiveSession>,
    pub(crate) credit: Option<Waiter<Session, LiveSession>>,
    data: *mut u8,
    ctrl: *mut u8,
    pulse_pending: bool,
    pulse_failures: u32,
}

impl TicketIrq {
    pub(crate) unsafe fn attach(data: *mut u8, ctrl: *mut u8, credit: bool) -> Result<Self, Error> {
        let live: LiveSession = crate::generated::wake_session;
        let scope = live().map_err(|_| Error::Reset)?;
        let publisher =
            unsafe { Words::rx(data, ctrl).publisher(scope, live) }.map_err(|_| Error::Reset)?;
        let credit = credit
            .then(|| unsafe { Words::credit(data, ctrl).waiter(scope, live) })
            .transpose()
            .map_err(|_| Error::Reset)?;
        Ok(Self {
            publisher,
            credit,
            data,
            ctrl,
            pulse_pending: false,
            pulse_failures: 0,
        })
    }

    pub(crate) fn waiting_for_credit(&self) -> bool {
        self.credit.as_ref().is_some_and(Waiter::is_armed)
    }

    pub(crate) fn reply_wake_ready<T: Message>(
        &self,
        ring: &shm::RingTx<T>,
    ) -> Result<bool, Error> {
        Ok(self.publisher.needs_commit().map_err(|_| Error::Reset)?
            && Self::requested_head(self.ctrl, ring))
    }

    fn requested_head<T: Message>(ctrl: *const u8, ring: &shm::RingTx<T>) -> bool {
        let head = unsafe {
            ctrl.add(shm::wake::RPC_WAIT_HEAD)
                .cast::<u32>()
                .read_volatile()
        };
        u32::from_le(head) == ring.head() && !ring.all_consumed()
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
    #[inline(always)]
    fn check(&self, _: sealed::Token) -> Result<(), Error> {
        self.publisher.check_live().map_err(|_| Error::Reset)
    }

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
    fn reserve<T: Message>(
        &mut self,
        _: sealed::Token,
        ring: &mut shm::RingTx<T>,
        max: u32,
    ) -> Result<u32, Error> {
        let free = ring.writable().min(max);
        let Some(ticket) = &mut self.credit else {
            return Ok(free);
        };
        if free == 0 || ticket.is_armed() {
            reserve_credit(ticket, free, || ring.writable().min(max))
        } else {
            Ok(free)
        }
    }

    #[inline(always)]
    fn wake_requested<T: Message>(&mut self, _: sealed::Token, ring: &shm::RingTx<T>) -> bool {
        let arm = unsafe {
            self.ctrl
                .add(shm::wake::RX_WAIT)
                .cast::<u32>()
                .read_volatile()
        };
        if u32::from_le(arm) & 1 == 0 {
            return false;
        }
        let fresh = self.publisher.publish_if(
            || {
                let _ = unsafe { self.data.cast::<u32>().read_volatile() };
            },
            || Self::requested_head(self.ctrl, ring),
        );
        if let Ok(fresh) = fresh {
            self.pulse_pending |= fresh;
        }
        self.retry()
    }
}

#[rp1_rt::cold]
fn reserve_credit(
    ticket: &mut Waiter<Session, LiveSession>,
    mut free: u32,
    reread: impl FnOnce() -> u32,
) -> Result<u32, Error> {
    let reset = |_| Error::Reset;
    if free == 0 {
        if !ticket.is_armed() || ticket.pending().map_err(reset)? {
            ticket.arm().map_err(reset)?;
            free = reread();
        }
        if free == 0 {
            return Ok(0);
        }
    }
    if ticket.is_armed() {
        ticket.acknowledge_and_cancel().map_err(reset)?;
    }
    rp1_proto::barrier();
    Ok(free)
}

#[rp1_rt::cold]
fn publish_credit<T: Message>(
    ticket: &mut Publisher<Session, LiveSession>,
    ring: &shm::RingRx<T>,
) -> Result<(), Error> {
    if ring.publish_credit(ticket).map_err(|_| Error::Reset)? {
        let _ = VectorOwner::pulse_owned();
    }
    Ok(())
}

/// The v3 downlink reader pairs the existing force-level ISR with v3 RX tickets.
#[doc(hidden)]
pub struct TicketDoorbell<const PIN: u8, const CORE: u8> {
    pub(crate) ticket: Option<Waiter<Session, LiveSession>>,
    credit: Option<Publisher<Session, LiveSession>>,
    scope: Session,
}

impl<const PIN: u8, const CORE: u8> TicketDoorbell<PIN, CORE> {
    pub(crate) unsafe fn attach(
        data: *mut u8,
        ctrl: *mut u8,
        direct: bool,
        credit: bool,
    ) -> Result<Self, Error> {
        let live: LiveSession = crate::generated::wake_session;
        let scope = live().map_err(|_| Error::Reset)?;
        let reset = |_| Error::Reset;
        let ticket = (!direct)
            .then(|| unsafe { Words::rx(data, ctrl).waiter(scope, live) })
            .transpose()
            .map_err(reset)?;
        let credit = credit
            .then(|| unsafe { Words::credit(data, ctrl).publisher(scope, live) })
            .transpose()
            .map_err(reset)?;
        Ok(Self {
            ticket,
            credit,
            scope,
        })
    }
}

impl<const PIN: u8, const CORE: u8> Wake for TicketDoorbell<PIN, CORE> {}
impl<const PIN: u8, const CORE: u8> sealed::Wake for TicketDoorbell<PIN, CORE> {
    #[inline(always)]
    fn check(&self, _: sealed::Token) -> Result<(), Error> {
        match crate::generated::wake_session() {
            Ok(current) if current == self.scope => Ok(()),
            _ => Err(Error::Reset),
        }
    }

    fn opened<T: Message>(&mut self, _: sealed::Token, _: &mut shm::RingRx<T>) {
        // The first thread slice scans queued work, including a publication
        // that preceded open. The force ISR only wakes the thread.
        if let Some(ticket) = &mut self.ticket {
            let _ = ticket.arm();
        }
    }

    fn publish_credit<T: Message>(
        &mut self,
        _: sealed::Token,
        ring: &shm::RingRx<T>,
    ) -> Result<(), Error> {
        if let Some(credit) = &mut self.credit {
            publish_credit(credit, ring)?;
        }
        Ok(())
    }

    fn cancel_receive(&mut self, _: sealed::Token) -> Result<(), Error> {
        if let Some(ticket) = &mut self.ticket
            && ticket.is_armed()
        {
            ticket.cancel().map_err(|_| Error::Reset)?;
        }
        Ok(())
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
                if ticket.is_armed() {
                    // The request was observed under this ticket. Complete the reply
                    // first so its pulse can overlap the downlink ACK and re-arm.
                    ticket.acknowledge_and_cancel().map_err(reset)?;
                }
                if budget == 0 {
                    return Ok(Progress::More);
                }
            }
            if ticket.is_armed() && ticket.pending().map_err(reset)? {
                // A pulse with no observed work can still consume this ticket.
                // Re-arm and rescan so a subsequent publication can notify us.
                ticket.cancel().map_err(reset)?;
            }
            if !ticket.is_armed() {
                ticket.arm().map_err(reset)?;
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

// Each core has its own DSRAM copy. Only its doorbell ISR updates it.
static DOORBELL_EVENTS: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);

pub(crate) fn doorbell_events() -> u32 {
    DOORBELL_EVENTS.load(core::sync::atomic::Ordering::Relaxed)
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
        DOORBELL_EVENTS.store(
            doorbell_events().wrapping_add(1),
            core::sync::atomic::Ordering::Relaxed,
        );
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
            unsafe { TicketIrq::attach(data.as_mut_ptr().cast(), ctrl.as_mut_ptr().cast(), false) }
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
    fn v3_rpc_services_preexisting_requests_and_only_armed_flagged_replies_pulse() {
        use crate::{
            RingRx, RingTx, Server,
            mmio::fake::{self, Access},
        };
        use rp1_proto::{
            LINK_START, link,
            shm::wake::{RPC_WAIT_HEAD, RX_SEQ, RX_WAIT},
        };

        let window = LINK_START as usize;
        fake::set(window + link::SHM_STATUS, link::SHM_READY);
        fake::set(window + link::SHM_GENERATION, 2);
        fake::set(window + mailbox::BOOT_EPOCH, 2);
        fake::alias(MSIX_CFG_SET, MSIX_CFG, true);
        fake::alias(MSIX_CFG_CLR, MSIX_CFG, false);
        let (mut down, mut down_ctrl) = ([0u64; 64], [0u64; 16]);
        let (mut up, mut up_ctrl) = ([0u64; 64], [0u64; 16]);
        let (down, down_ctrl, up, up_ctrl): (*mut u8, *mut u8, *mut u8, *mut u8) = (
            down.as_mut_ptr().cast(),
            down_ctrl.as_mut_ptr().cast(),
            up.as_mut_ptr().cast(),
            up_ctrl.as_mut_ptr().cast(),
        );
        let ov = shm::Overflow::Reject;
        let mut requests =
            unsafe { shm::RingTx::attach(down, 512, down_ctrl, 128, 4, ov) }.unwrap();
        let mut replies = unsafe { shm::RingRx::attach(up, 512, up_ctrl, 128, 4, ov) }.unwrap();
        requests.push_request(&17u32, true).unwrap();
        let doorbell =
            unsafe { TicketDoorbell::<27, 1>::attach(down, down_ctrl, true, false) }.unwrap();
        let rx = RingRx::open(
            unsafe { shm::RingRx::<u32>::attach(down, 512, down_ctrl, 128, 4, ov) }.unwrap(),
            doorbell,
        );
        assert!(!fake::take().contains(&Access::Write(0x400d_2114, 1 << 27)));
        let tx = RingTx::new(
            unsafe { shm::RingTx::<u32>::attach(up, 512, up_ctrl, 128, 4, ov) }.unwrap(),
            unsafe { TicketIrq::attach(up, up_ctrl, false) }.unwrap(),
        );
        let mut server = Server::new(tx, rx);
        let pulses = || {
            fake::take()
                .into_iter()
                .filter(|access| *access == Access::Write(MSIX_CFG_SET, MSIX_CFG_TEST))
                .count()
        };
        let arm = |head, ticket| unsafe {
            up_ctrl
                .add(RPC_WAIT_HEAD)
                .cast::<u32>()
                .write_volatile(head);
            up_ctrl.add(RX_WAIT).cast::<u32>().write_volatile(ticket);
        };

        assert_eq!(server.serve(0, |n| n), Ok(Progress::More));
        assert_eq!(pulses(), 0, "an unarmed ready-first reply must never pulse");
        arm(2, 1);
        assert_eq!(server.serve(4, |_| unreachable!()), Ok(Progress::Idle));
        assert_eq!(pulses(), 0, "the ticket is for a later reply");
        arm(1, 3);
        assert_eq!(server.serve(4, |_| unreachable!()), Ok(Progress::Idle));
        assert_eq!(
            pulses(),
            1,
            "the second force can arrive after an early reply"
        );
        assert_eq!(unsafe { up.add(RX_SEQ).cast::<u32>().read_volatile() }, 3);
        assert_eq!(server.serve(4, |_| unreachable!()), Ok(Progress::Idle));
        assert_eq!(pulses(), 0, "one pulse per ticket");
        assert_eq!(replies.pop(), Ok(Some(17)));

        requests.push_request(&23u32, false).unwrap();
        arm(2, 5);
        assert_eq!(server.serve(4, |n| n), Ok(Progress::Idle));
        assert_eq!(pulses(), 0, "an unflagged request cannot send an IRQ");
        assert_eq!(replies.pop(), Ok(Some(23)));
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

#[cfg(test)]
mod startup_tests {
    use super::*;
    use crate::mmio::fake::{self, Access};
    use rp1_proto::{link, status};

    #[test]
    fn proc1_recovers_a_kick_that_arrived_before_its_vector_was_enabled() {
        type Kick = Doorbell<26, 1>;
        fake::alias(Kick::INTF_SET, Kick::INTF, true);
        fake::alias(Kick::INTF_CLR, Kick::INTF, false);
        fake::alias(MSIX_CFG_SET, MSIX_CFG, true);
        fake::alias(MSIX_CFG_CLR, MSIX_CFG, false);
        fake::trace_reads();

        let mut page = [0u32; mailbox::END / 4];
        page[status::EPOCH / 4] = 2;
        page[mailbox::BOOT_EPOCH / 4] = 2;
        page[mailbox::WAIT / 4] = 1;
        page[link::SHM_STATUS / 4] = link::SHM_READY;
        let base = page.as_mut_ptr().cast();

        unsafe { __kick_startup_owner::<26, 1>() };
        assert_eq!(
            fake::take(),
            [
                Access::Write(Kick::INTF_SET, 1 << 26),
                Access::Read(Kick::INTF),
            ]
        );
        // A pending-vector reset can drop the first kick. The owner's initial
        // scan still delivers the retained result once.
        deliver_startup_from(base, || true);
        assert_eq!(page[mailbox::SEQ / 4], 1);
        assert_eq!(
            fake::take()
                .iter()
                .filter(|op| **op == Access::Write(MSIX_CFG_SET, MSIX_CFG_TEST))
                .count(),
            1
        );
        on_startup_kick::<26>(|| deliver_startup_from(base, || true));
        let accesses = fake::take();
        assert_eq!(
            &accesses[..3],
            &[
                Access::Read(Kick::INTF),
                Access::Write(Kick::INTF_CLR, 1 << 26),
                Access::Read(Kick::INTF),
            ]
        );
        assert!(!accesses.contains(&Access::Write(MSIX_CFG_SET, MSIX_CFG_TEST)));
        assert_eq!(fake::read(Kick::INTF), 0);

        // An old kick with an armed word cannot publish into an invalidated boot.
        page[status::EPOCH / 4] = 3;
        page[mailbox::WAIT / 4] = 3;
        deliver_startup_from(base, || true);
        assert_eq!(page[mailbox::SEQ / 4], 1);
        assert!(!fake::take().contains(&Access::Write(MSIX_CFG_SET, MSIX_CFG_TEST)));
    }
}

#[cfg(test)]
mod credit_tests {
    use super::*;
    use crate::{Channel, RingRx, RingTx, mmio::fake};
    use rp1_proto::{LINK_START, link, shm::Overflow};

    #[repr(align(8))]
    struct RingMemory {
        data: [u32; 128],
        ctrl: [u32; 32],
    }

    impl RingMemory {
        fn wrapping() -> Self {
            let mut mem = Self {
                data: [0; 128],
                ctrl: [0; 32],
            };
            mem.data[shm::HEAD / 4] = u32::MAX - 1;
            mem.ctrl[shm::TAIL / 4] = u32::MAX - 1;
            mem
        }

        fn ends(&mut self) -> (shm::RingTx<u32>, shm::RingRx<u32>, Words<Session>) {
            let data = self.data.as_mut_ptr().cast();
            let ctrl = self.ctrl.as_mut_ptr().cast();
            let writer =
                unsafe { shm::RingTx::attach(data, 512, ctrl, 128, 16, Overflow::Reject) }.unwrap();
            let reader =
                unsafe { shm::RingRx::attach_ticketed(data, 512, ctrl, 128, 16, Overflow::Reject) }
                    .unwrap();
            (writer, reader, unsafe { Words::credit(data, ctrl) })
        }
    }

    #[test]
    fn full_channel_reserves_replies_and_releases_wrapping_request_batches() {
        use shm::wake::{CREDIT_ACK, CREDIT_SEQ, CREDIT_WAIT, RX_WAIT};
        let link_base = LINK_START as usize;
        fake::set(link_base + link::SHM_STATUS, link::SHM_READY);
        fake::set(link_base + link::SHM_GENERATION, 2);
        fake::set(link_base + mailbox::BOOT_EPOCH, 4);
        fake::alias(MSIX_CFG_SET, MSIX_CFG, true);
        fake::alias(MSIX_CFG_CLR, MSIX_CFG, false);
        let live: LiveSession = crate::generated::wake_session;
        let scope = live().unwrap();

        let mut up = RingMemory::wrapping();
        let (mut reply_writer, mut replies, reply_words) = up.ends();
        assert_eq!(reply_writer.push_slice(&[0; 16]), 16);
        let mut peer_publish = unsafe { reply_words.publisher(scope, live) }.unwrap();

        let mut down = RingMemory::wrapping();
        let (mut requests, request_reader, request_words) = down.ends();
        let mut peer_wait = unsafe { request_words.waiter(scope, live) }.unwrap();
        let down_data = down.data.as_mut_ptr().cast();
        let down_ctrl = down.ctrl.as_mut_ptr().cast();
        let up_data = up.data.as_mut_ptr().cast();
        let up_ctrl = up.ctrl.as_mut_ptr().cast();
        let tx = RingTx::new(
            reply_writer,
            unsafe { TicketIrq::attach(up_data, up_ctrl, true) }.unwrap(),
        );
        let rx = RingRx::open(
            request_reader,
            unsafe { TicketDoorbell::<27, 1>::attach(down_data, down_ctrl, false, true) }.unwrap(),
        );
        let mut channel = Channel { tx, rx };

        assert_eq!(
            requests.push_slice(&core::array::from_fn::<_, 16, _>(|i| i as u32 + 1)),
            16
        );
        peer_wait.arm().unwrap();
        assert_eq!(
            channel.respond(16, |_| unreachable!("reply ring is full")),
            Ok(Progress::Idle)
        );
        assert_eq!(down.ctrl[shm::TAIL / 4], u32::MAX - 1);
        assert_eq!(down.ctrl[RX_WAIT / 4] & 1, 0);
        assert_eq!(up.data[CREDIT_WAIT / 4] & 1, 1);

        assert_eq!(replies.pop().unwrap(), Some(0));
        assert!(replies.publish_credit(&mut peer_publish).unwrap());
        assert_eq!(up.ctrl[CREDIT_SEQ / 4], up.data[CREDIT_WAIT / 4]);
        assert_eq!(
            channel.respond(16, |n| {
                assert_eq!(n, 1);
                n
            }),
            Ok(Progress::More)
        );
        assert_eq!(down.ctrl[shm::TAIL / 4], u32::MAX);
        assert_eq!(up.data[CREDIT_ACK / 4], up.ctrl[CREDIT_SEQ / 4]);
        assert!(peer_wait.acknowledge().unwrap());
        assert!(fake::take().contains(&fake::Access::Write(MSIX_CFG_SET, MSIX_CFG_TEST)));

        let mut batch = [0; 16];
        assert_eq!(replies.pop_into(&mut batch).unwrap(), 16);
        assert_eq!(&batch[..15], &[0; 15]);
        assert_eq!(batch[15], 1);
        assert_eq!(channel.respond(u32::MAX, |n| n), Ok(Progress::More));
        assert_eq!(down.ctrl[shm::TAIL / 4], 7);

        let mut other_down = RingMemory::wrapping();
        let (mut other_requests, mut other_rx, _) = other_down.ends();
        other_requests.push(&123).unwrap();
        assert_eq!(other_rx.pop().unwrap(), Some(123));
        assert_eq!(channel.respond(u32::MAX, |n| n), Ok(Progress::Idle));
        assert_eq!(replies.pop_into(&mut batch).unwrap(), 15);
        assert_eq!(
            &batch[..15],
            &core::array::from_fn::<_, 15, _>(|i| i as u32 + 2)
        );
        requests.push(&99).unwrap();
        let old = (up.data, up.ctrl, down.data, down.ctrl);
        fake::set(link_base + link::SHM_GENERATION, 4);
        assert_eq!(
            channel.respond(8, |_| unreachable!("stale session")),
            Err(Error::Reset)
        );
        assert_eq!(channel.send(&1), Err(Error::Reset));
        assert_eq!((up.data, up.ctrl, down.data, down.ctrl), old);
    }
}
