//! How the two sides tell each other that a ring has data. Each endpoint carries its notifier
//! as a type parameter, chosen by the declaration (`notify = "poll"` or `"irq"`), and every
//! notifier is zero-sized: [`Poll`] compiles to nothing.
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
    shm::{self, Progress},
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
const ALARM_BIT: u32 = 1 << (VECTOR - TIMER0_VECTOR);

/// The notifier traits' methods: only endpoints call them, and only generated code makes the
/// notifiers that endpoints carry.
pub(crate) mod sealed {
    use super::{Error, Message, Progress, shm};

    /// Every method takes one. Its field is private to this crate and nothing returns one, so
    /// outside code cannot call the methods, not even through a public `Notify` / `Wake` bound.
    pub struct Token(pub(crate) ());

    pub trait Notify: Copy {
        /// The endpoint's copy of a notifier whose resource the generated code made.
        fn claim(_: Token) -> Self;
        /// Wake the reader of `ring` if it sleeps. Returns whether it notified.
        fn wake<T: Message>(self, _: Token, ring: &mut shm::RingTx<T>) -> bool;
    }

    pub trait Wake: Copy {
        fn claim(_: Token) -> Self;
        /// Make the ring just attached notifiable: the first message wakes this reader.
        fn opened<T: Message>(self, _: Token, ring: &mut shm::RingRx<T>);
        fn try_drain<T: Message>(
            self,
            _: Token,
            ring: &mut shm::RingRx<T>,
            max: u32,
            f: impl FnMut(T) -> Result<(), Error>,
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
impl sealed::Notify for Poll {
    fn claim(_: sealed::Token) -> Self {
        Poll
    }
    #[inline(always)]
    fn wake<T: Message>(self, _: sealed::Token, _: &mut shm::RingTx<T>) -> bool {
        false
    }
}
impl Wake for Poll {}
impl sealed::Wake for Poll {
    fn claim(_: sealed::Token) -> Self {
        Poll
    }
    #[inline(always)]
    fn opened<T: Message>(self, _: sealed::Token, _: &mut shm::RingRx<T>) {}
    #[inline(always)]
    fn try_drain<T: Message>(
        self,
        _: sealed::Token,
        ring: &mut shm::RingRx<T>,
        max: u32,
        mut f: impl FnMut(T) -> Result<(), Error>,
    ) -> Result<Progress, Error> {
        for _ in 0..max.max(1) {
            let Some(value) = ring.pop()? else {
                return Ok(Progress::Idle);
            };
            f(value)?;
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
impl sealed::Notify for HostIrq {
    fn claim(_: sealed::Token) -> Self {
        Self {
            _private: core::marker::PhantomData,
        }
    }
    /// [`shm::RingTx::wake_needed`] reads `head` back through the shm window (completing the
    /// posted slot and `head` writes), then reads the reader's sleep counter from the link page:
    /// it pulses only for a sleep not yet notified, so a reader that keeps up costs no
    /// interrupts and a sleeping one gets one (protocol.md §2.3). One PCIe round trip per call.
    #[inline(always)]
    fn wake<T: Message>(self, _: sealed::Token, ring: &mut shm::RingTx<T>) -> bool {
        let asleep = ring.wake_needed();
        if asleep {
            self.pulse();
        }
        asleep
    }
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

    /// Force this core's own doorbell bit: the handler runs again (after [`Progress::More`]).
    /// Raise it once after handing the endpoint to its handler ([`RingRx`](crate::RingRx)).
    pub fn raise(self) {
        write(Self::INTF_SET, Self::BIT);
    }
}

impl<const PIN: u8, const CORE: u8> Wake for Doorbell<PIN, CORE> {}
impl<const PIN: u8, const CORE: u8> sealed::Wake for Doorbell<PIN, CORE> {
    fn claim(_: sealed::Token) -> Self {
        unsafe { Self::new() }
    }
    /// Arm the ring. Messages that Linux sent before the arm are already there and rang no
    /// doorbell, so raise it for them: the handler drains them and arms again.
    fn opened<T: Message>(self, _: sealed::Token, ring: &mut shm::RingRx<T>) {
        if !ring.prepare_wait() {
            self.raise();
        }
    }
    #[inline(always)]
    fn try_drain<T: Message>(
        self,
        _: sealed::Token,
        ring: &mut shm::RingRx<T>,
        max: u32,
        f: impl FnMut(T) -> Result<(), Error>,
    ) -> Result<Progress, Error> {
        ring.try_drain(max, f)
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
        let irq = <HostIrq as sealed::Notify>::claim(sealed::Token(()));
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
