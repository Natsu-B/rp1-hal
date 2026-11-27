//! RP1 DMA controller (Synopsys DW AXI DMAC, eight channels): memory-to-memory single-block
//! transfers with polled completion.
//!
//! Register evidence: [`crate::pac::dw_axi_dmac`]; address map, ownership and HW results:
//! `docs/dma.md`.
//!
//! The controller is one block (one reset, one clock, one interrupt line), so it is declared
//! whole (`dma = {}`) by one core. [`Token::init`] starts clk_dma at 100 MHz (held by the SCMI
//! server for a declared DMA block), releases the reset, soft-resets the DMAC and returns the
//! eight [`Channel`]s. Channels are independent (the channel-enable register has per-channel
//! write enables, so no read-modify-write is shared).
//!
//! # Addresses
//! The DMAC is a 40-bit bus master and does not use the processor's 32-bit map (DOC: RP1
//! Peripherals §2.2, table 1). [`BusAddr::from_proc`] translates the processor addresses it can
//! reach; processor-local ISRAM/DSRAM (stacks, ordinary `static`s) and the ROM are not visible
//! to the DMAC. Linux DDR is reached either through the outbound ATU window for shared DDR
//! (processor `0x8000_0000 + x` = bus `0x80_0000_0000 + x`) or through
//! the PCIe direct-mapped space, where host physical address `p` is bus
//! `0x10_0000_0000 + p` ([`BusAddr::host_phys`], the RP1 `dma-ranges` Linux uses).
//!
//! # Ordering
//! The Cortex-M3 has no data cache, so no cache maintenance is needed. Channel setup writes are
//! posted without `dsb`; immediately before the channel-enable write a compiler fence + `dsb`
//! ([`rp1_proto::barrier`]) completes every earlier CPU store to the source buffer and keeps
//! the compiler from moving buffer stores past the enable. Completion is reported only after
//! `DMA_TFR_DONE` *and* the channel-enable bit cleared, followed by the same barrier.
//! All blocks set `NonPosted_LastWrite_En` and explicitly select non-bufferable `AW_CACHE`.
//! HYP: the final response covers earlier writes and establishes acceptance at the shared
//! PCIe ordering point before later CPU window stores (see `docs/dma.md` for evidence limits).
//! This does not mean a posted PCIe write has already reached host DDR.
//! Cancellation stops the engine's buffer accesses, but does not drain accepted remote writes.
//!
//! # Cancellation
//! A transfer borrows both pool leases while it holds exclusive access to the channel.
//! Cancellation requests graceful disable, then checks for a terminal state with a bounded
//! number of MMIO accesses. A running or ambiguous channel retains its leases. Drop requests
//! the same stop and quarantines the slots if it cannot prove quiescence. A forgotten transfer
//! leaves its pool claims and the channel's in-flight state intact.
//!
//! IRQ40 delivery is pending a separate hardware probe. Waiting for completion is available
//! through `copy_polling` and `wait_polling`. Thread-mode cancellation can use the timer IRQ.
use crate::{
    clocks::Clocks,
    linux_blocks::Mmio,
    pac::dw_axi_dmac::{
        BASE, CHANNEL_BASE, CHANNEL_STRIDE, MAX_BLOCK_TS, cfg2, ch, common, ctl_h, ctl_l, int,
    },
    scmi::{Generator, Kind, PhysicalState, RP1_CLOCKS, Registers, Round, id},
    timer::{self, Deadline, Duration, Timer, WaitTimer, read_instant},
};
mod pool;
#[cfg(feature = "unstable-ring-dma")]
mod ring;
use pool::EngineClaim;
pub use pool::{Lease, Pool};
#[cfg(feature = "unstable-ring-dma")]
pub use ring::ExperimentalRingDma;

const CLEANUP_US: u32 = 999;

/// clk_dma (DOC: RP1 Peripherals ch. 9, and the stock firmware's setting).
pub const CLOCK_HZ: u32 = 100_000_000;
// RESETS CTRL0 bit 3 = DMA (rp1_hal::linux_blocks::ALLOWED).
const RESETS_CLR0: usize = 0x4001_7000;
const RESETS_DONE0: usize = 0x4001_4018;
const RESET_BIT: u32 = 1 << 3;
const INIT_TIMEOUT_US: u32 = 10_000;
/// AXI burst length in beats per channel: Linux `snps,axi-max-burst-len` of rp1.dtsi, which
/// matches the datasheet's FIFO depths (8 beats on channels 0 and 1, 4 on the others). ARLEN /
/// AWLEN take `beats - 1` (AXI encoding, Linux `DWAXIDMAC_ARWLEN_*`); Linux writes the DT value
/// unchanged (one beat more than the FIFO), this driver writes `beats - 1`.
const BURST: [u32; 8] = [8, 8, 4, 4, 4, 4, 4, 4];
/// Largest transfer width: the 128-bit master (`snps,data-width = <4>`).
const MAX_WIDTH_LOG2: u32 = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    /// Invalid length, an overlapping range, or more than one block.
    Invalid,
    /// The complete range is outside the DMA-visible memory allowed by this API.
    NotDmaVisible,
    /// Stale clocks or a stopped DMA clock.
    Precondition,
    /// The transfer or reset deadline elapsed.
    Timeout,
    /// The channel reported a decode, slave or other bus error.
    Bus(u32),
    /// The channel or pool slot is already in use.
    Busy,
    /// Graceful stop or retirement is still unconfirmed.
    Recovering,
    /// An external forced abort leaves bus quiescence unproven.
    Aborted,
    /// Explicit cancellation interrupted a transfer.
    Cancelled,
    /// Invalid deadline or unavailable thread-mode timer context.
    Wait(timer::Error),
}

/// An address on the RP1 40-bit system bus, as the DMAC issues it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BusAddr(pub u64);

impl BusAddr {
    /// Shared SRAM, 64 KiB (DOC: processor 0x2000_0000 = system 0xC0_2000_0000).
    pub const SRAM: u64 = 0xc0_2000_0000;
    /// APB / AHB peripherals (DOC: processor 0x4000_0000..0x4019_0000 = system 0xC0_4000_0000..).
    pub const PERIPHERALS: u64 = 0xc0_4000_0000;
    /// Outbound ATU space (DOC: system 0x80_0000_0000; the processor window 0x8000_0000 is its
    /// first 1 GiB).
    pub const ATU: u64 = 0x80_0000_0000;
    /// PCIe address of host physical 0 through the direct-mapped outbound space (DOC: RP1
    /// `dma-ranges` in the Linux DT).
    pub const HOST_PHYS: u64 = 0x10_0000_0000;

    /// Translates a processor address. Fails for addresses the DMAC cannot reach.
    pub const fn from_proc(address: usize) -> Result<Self, Error> {
        let a = address as u64;
        Ok(Self(match a {
            0x2000_0000..0x2001_0000 => Self::SRAM + (a - 0x2000_0000),
            0x4000_0000..0x4019_0000 => Self::PERIPHERALS + (a - 0x4000_0000),
            0x8000_0000..0xc000_0000 => Self::ATU + (a - 0x8000_0000),
            _ => return Err(Error::NotDmaVisible),
        }))
    }
    /// Host (BCM2712) physical address through the PCIe direct-mapped outbound space.
    pub const fn host_phys(phys: u64) -> Self {
        Self(Self::HOST_PHYS + phys)
    }
    fn range(self, len: u32) -> Result<Self, Error> {
        if self
            .0
            .checked_add(u64::from(len))
            .is_none_or(|end| end > 1 << 40)
        {
            return Err(Error::Invalid);
        }
        Ok(self)
    }
}

fn generator() -> Generator {
    let Kind::Linux(clk) = RP1_CLOCKS[id::DMA as usize].kind else {
        unreachable!()
    };
    clk
}
fn clock_current(io: &mut impl Registers) -> Result<(), Error> {
    if generator().state(io)
        == Some(PhysicalState {
            rate_hz: CLOCK_HZ,
            enabled: true,
        })
    {
        Ok(())
    } else {
        Err(Error::Precondition)
    }
}
impl From<timer::Error> for Error {
    fn from(error: timer::Error) -> Self {
        match error {
            timer::Error::Timeout => Self::Timeout,
            other => Self::Wait(other),
        }
    }
}

// Startup-only register handshake. The timer IRQ need not be configured at this point.
fn wait_init(io: &mut impl Registers, address: usize, mask: u32, value: u32) -> Result<(), Error> {
    let end = Deadline::after_ticks(read_instant(io), INIT_TIMEOUT_US)?;
    loop {
        let remaining = end.check(read_instant(io));
        if io.read(address) & mask == value {
            return Ok(());
        }
        remaining?;
        core::hint::spin_loop();
    }
}

/// Sole ownership of the DMA controller, issued by generated resources.
#[derive(Debug)]
pub struct Token {
    _private: core::marker::PhantomData<core::cell::Cell<()>>,
}
/// `?` on a failed [`Token::init`].
impl<R> From<crate::InitError<R, Error>> for Error {
    fn from(e: crate::InitError<R, Error>) -> Self {
        e.error
    }
}
impl Token {
    /// # Safety
    /// Generated resources only: one core owns the DMAC, its reset and clock. Linux's
    /// `rp1_dma` node and its `dmas` consumers must be disabled; SCMI must hold clk_dma.
    pub const unsafe fn new() -> Self {
        Self {
            _private: core::marker::PhantomData,
        }
    }
    /// Starts clk_dma at 100 MHz, releases the DMA reset, soft-resets the DMAC, disables every
    /// channel and its interrupts and enables the controller (interrupt output off). On error
    /// the token comes back in [`InitError`](crate::InitError) for a retry.
    #[doc(hidden)]
    pub fn init(self, clocks: &Clocks) -> Result<Dma, crate::InitError<Token, Error>> {
        if !clocks.still_current() {
            return Err(crate::InitError::new(Error::Precondition, self));
        }
        match start(&mut Mmio) {
            Ok(()) => Ok(Dma { _token: self }),
            Err(error) => Err(crate::InitError::new(error, self)),
        }
    }
}

fn start(io: &mut impl Registers) -> Result<(), Error> {
    let clk = generator();
    let (aux, div, _) = clk
        .choose(CLOCK_HZ, Round::Down)
        .ok_or(Error::Precondition)?;
    if clk.state(io).is_none_or(|s| s.rate_hz != CLOCK_HZ) {
        clk.program(io, aux, div);
    }
    clk.set_enabled(io, true);
    clock_current(io)?;
    io.write(RESETS_CLR0, RESET_BIT);
    wait_init(io, RESETS_DONE0, RESET_BIT, RESET_BIT)?;
    io.write(BASE + common::RESET, 1);
    wait_init(io, BASE + common::RESET, 1, 0)?;
    for n in 0..8 {
        let c = channel_base(n);
        io.write(c + ch::INTSIGNAL_ENA, 0);
        io.write(c + ch::INTSTATUS_ENA, 0);
        io.write(c + ch::INTCLEAR, int::ALL);
    }
    // Channel enables are cleared by the soft reset; clear them explicitly as Linux does.
    io.write(BASE + common::CHEN, 0xff << common::CHEN_WE_SHIFT);
    io.write(BASE + common::CFG, common::CFG_DMAC_EN);
    Ok(())
}

const fn channel_base(n: u8) -> usize {
    BASE + CHANNEL_BASE + n as usize * CHANNEL_STRIDE
}

/// An initialized DMA controller. The generated channel set holds this owner.
pub struct Dma {
    _token: Token,
}

impl Dma {
    /// Check all hardware channel slots, including reserved channels.
    #[doc(hidden)]
    pub fn __check_idle(&self) -> Result<(), Error> {
        check_controller_idle(&mut Mmio)
    }

    /// Return the controller to the generated declaration.
    ///
    /// # Safety
    /// The caller must own all issued channels and have obtained an idle proof for each
    /// one with no intervening access. Raw users must separately retire any remote memory.
    #[doc(hidden)]
    pub unsafe fn __try_free(self) -> Result<Token, crate::FreeError<Self, Error>> {
        match release_controller(&mut Mmio) {
            Ok(()) => Ok(self._token),
            Err(error) => Err(crate::FreeError {
                driver: self,
                error,
            }),
        }
    }

    /// Issue a channel named in the owning core's declaration.
    ///
    /// # Safety
    /// Generated resources only: issue each selected channel exactly once.
    #[doc(hidden)]
    pub const unsafe fn channel<const N: u8>(&mut self) -> Channel<N> {
        const { assert!(N < 8) };
        Channel {
            state: State::Idle,
            _local: core::marker::PhantomData,
        }
    }
}

fn check_controller_idle(io: &mut impl Registers) -> Result<(), Error> {
    if io.read(BASE + common::CHEN) & 0xff != 0 || io.read(BASE + common::CHEN_H) & 0xff != 0 {
        Err(Error::Busy)
    } else {
        Ok(())
    }
}

fn release_controller(io: &mut impl Registers) -> Result<(), Error> {
    check_controller_idle(io)?;
    for n in 0..8 {
        let c = channel_base(n);
        // An abort cannot be upgraded to a release proof by clearing its status.
        if io.read(c + ch::INTSTATUS) & int::CH_ABORTED != 0 {
            return Err(Error::Aborted);
        }
    }
    for n in 0..8 {
        let c = channel_base(n);
        io.write_relaxed(c + ch::INTSIGNAL_ENA, 0);
        io.write_relaxed(c + ch::INTCLEAR, int::ALL);
        if io.read(c + ch::INTSTATUS) != 0 {
            return Err(Error::Recovering);
        }
    }
    io.write_relaxed(BASE + common::COMMON_INTSIGNAL_ENA, 0);
    let status = io.read(BASE + common::COMMON_INTSTATUS);
    io.write(BASE + common::COMMON_INTCLEAR, status);
    if io.read(BASE + common::COMMON_INTSTATUS) != 0 {
        return Err(Error::Recovering);
    }
    // Preserve status capture enables on a failed release so the returned driver can retry.
    for n in 0..8 {
        io.write_relaxed(channel_base(n) + ch::INTSTATUS_ENA, 0);
    }
    io.write_relaxed(BASE + common::COMMON_INTSTATUS_ENA, 0);
    io.write(BASE + common::CFG, 0);
    Ok(())
}

/// Timer-backed recheck shared by transfers and generated controller sets.
#[doc(hidden)]
pub fn __stop_until(
    wait: &mut WaitTimer,
    end: Deadline,
    mut stop: impl FnMut() -> Result<(), Error>,
) -> Result<(), Error> {
    let entered = Timer::new().now();
    // Cleanup must request the stop even if the context or deadline rules out a wait.
    match stop() {
        Err(Error::Recovering) => {}
        result => return result,
    }
    let mut context = wait.thread_context()?;
    end.check(Timer::new().now())?;
    let limit = Deadline::after_ticks(entered, CLEANUP_US)?;
    let end = Deadline::at(end.instant().min(limit.instant()));
    // IRQ40 is unverified. The timer wakes at the capped deadline even if no DMA event occurs.
    match context.wait_until(end, || match stop() {
        Err(Error::Recovering) => None,
        result => Some(result),
    }) {
        Ok(result) => result,
        Err(timer::Error::Timeout) => stop(),
        Err(error) => Err(error.into()),
    }
}

/// One initialized channel. Linux numbering: channels 0 and 1 have 8-beat FIFOs.
#[derive(Debug)]
pub struct Channel<const N: u8> {
    state: State,
    _local: core::marker::PhantomData<core::cell::Cell<()>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
    Idle,
    Active,
    Stopping,
    Aborted,
}

/// Transfer width log2 (bytes) for a copy: the largest power of two dividing both addresses and
/// the length, at most 16 bytes (Linux `axi_chan_get_xfer_width`).
pub const fn width_log2(src: u64, dst: u64, len: u32) -> u32 {
    let w = (src | dst | len as u64 | (1 << MAX_WIDTH_LOG2)).trailing_zeros();
    if w > MAX_WIDTH_LOG2 {
        MAX_WIDTH_LOG2
    } else {
        w
    }
}
/// Bytes one block can carry at the width chosen for `src`, `dst` and `len`.
pub const fn max_block_bytes(width_log2: u32) -> u32 {
    let max = (MAX_BLOCK_TS as u64) << width_log2;
    if max > u32::MAX as u64 {
        u32::MAX
    } else {
        max as u32
    }
}

impl<const N: u8> Channel<N> {
    /// Starts one local SRAM-to-SRAM block. Admission errors leave both leases as they were.
    pub fn try_start<'t>(
        &'t mut self,
        src: &'t mut Lease<'_>,
        dst: &'t mut Lease<'_>,
    ) -> Result<Transfer<'t>, Error> {
        start_leased(&mut Mmio, N, &mut self.state, src, dst)
    }

    /// Copies equal-length pool leases using bounded polling in thread mode.
    /// On an unconfirmed stop, both leases remain quarantined and yield empty slices.
    pub fn copy_polling(
        &mut self,
        src: &mut Lease<'_>,
        dst: &mut Lease<'_>,
        timeout: Duration,
    ) -> Result<(), Error> {
        copy_leases(N, &mut self.state, src, dst, timeout)
    }

    /// Start a single block with caller-managed bus addresses.
    ///
    /// # Safety
    /// Keep the complete, non-overlapping ranges valid and reserved even if the transfer is
    /// dropped or forgotten. Until quiescence is proven, no Rust reference may cover the
    /// destination, nothing else may write either range, and neither range may be memory
    /// the program relies on through Rust references. If the handle is lost, maintain these
    /// requirements for the firmware epoch. The caller must independently establish remote
    /// drain and window stability before reusing or publishing remote memory. Addresses must
    /// not change this controller, its clocks, firmware state, or any concurrently accessed
    /// memory.
    pub unsafe fn start_raw(
        &mut self,
        src: BusAddr,
        dst: BusAddr,
        len: u32,
    ) -> Result<RawTransfer<'_>, Error> {
        start_block(&mut Mmio, N, &mut self.state, src, dst, len)?;
        Ok(RawTransfer::new(N, &mut self.state))
    }

    /// Acknowledge an externally aborted channel after an independent drain.
    ///
    /// # Safety
    /// The caller must establish that all outstanding operations are drained. CH_ABORTED,
    /// a cleared enable bit and a CPU barrier alone do not establish that condition.
    pub unsafe fn acknowledge_external_abort(&mut self) -> Result<(), Error> {
        let mut io = Mmio;
        let c = channel_base(N);
        if enabled(&mut io, N) || io.read(c + ch::INTSTATUS) & int::CH_ABORTED == 0 {
            return Err(Error::Recovering);
        }
        self.state = State::Aborted;
        io.write(c + ch::INTCLEAR, int::ALL);
        if io.read(c + ch::INTSTATUS) != 0 {
            return Err(Error::Recovering);
        }
        self.state = State::Idle;
        Ok(())
    }

    /// Recheck a stop, including one left behind by a forgotten transfer.
    /// If a forgotten transfer is still active, this requests graceful disable first.
    /// Returns success only after the channel's terminal status has been acknowledged.
    pub fn try_recover(&mut self) -> Result<(), Error> {
        self.recover(&mut Mmio)
    }

    fn recover(&mut self, io: &mut impl Registers) -> Result<(), Error> {
        if self.state == State::Idle {
            return if enabled(io, N) {
                Err(Error::Busy)
            } else {
                Ok(())
            };
        }
        let _ = request_stop(io, N, &mut self.state, Error::Cancelled, &mut 0);
        match self.state {
            State::Idle => Ok(()),
            State::Aborted => Err(Error::Aborted),
            _ => Err(Error::Recovering),
        }
    }
}

#[inline(never)]
fn start_leased<'t>(
    io: &mut impl Registers,
    n: u8,
    state: &'t mut State,
    src: &'t mut Lease<'_>,
    dst: &'t mut Lease<'_>,
) -> Result<Transfer<'t>, Error> {
    let len = src.len();
    if len != dst.len() {
        return Err(Error::Invalid);
    }
    let (source, destination) = (src.address()?, dst.address()?);
    // Mark both slots before enable. Even an unwinding admission keeps the claim if
    // the enable may have made it to the engine. Every returned admission error precedes it.
    let mut src = src.hand_to_engine();
    let mut dst = dst.hand_to_engine();
    if let Err(error) = start_block(io, n, state, source, destination, len) {
        src.return_to_cpu();
        dst.return_to_cpu();
        return Err(error);
    }
    Ok(Transfer {
        raw: RawTransfer::new(n, state),
        src,
        dst,
    })
}

#[inline(never)]
fn copy_leases(
    n: u8,
    state: &mut State,
    src: &mut Lease<'_>,
    dst: &mut Lease<'_>,
    timeout: Duration,
) -> Result<(), Error> {
    rp1_rt::interrupt::ThreadGuard::new().map_err(timer::Error::from)?;
    let end = Deadline::after(timeout)?;
    clock_current(&mut Mmio)?;
    end.check(Timer::new().now())?;
    let mut transfer = start_leased(&mut Mmio, n, state, src, dst)?;
    transfer.raw.wait_accepted(&mut Mmio, end)
}

/// A running or quarantined local transfer. A cancelled destination may contain a
/// prefix or the whole block. Inspect the transfer outcome before using the destination.
#[must_use]
pub struct Transfer<'t> {
    raw: RawTransfer<'t>,
    src: EngineClaim<'t>,
    dst: EngineClaim<'t>,
}

impl Transfer<'_> {
    /// Observe completion once, issuing a graceful stop on a bus error.
    pub fn try_complete(&mut self) -> Option<Result<(), Error>> {
        self.raw.try_complete()
    }

    /// Request graceful disable and make one bounded stop check.
    /// On `Recovering`, this handle still holds both slots for the engine.
    pub fn try_cancel(&mut self) -> Result<(), Error> {
        self.raw.try_cancel()
    }

    /// Request cancellation, then recheck in thread mode using the dedicated timer.
    ///
    /// The recheck deadline is the earlier of `end` and 999 µs after entry. With IRQ40
    /// masked, a stop missed by the initial checks may be reported only at the alarm.
    pub fn cancel_until(&mut self, wait: &mut WaitTimer, end: Deadline) -> Result<(), Error> {
        self.raw.cancel_until(wait, end)
    }

    /// Explicitly poll against one absolute deadline in thread mode.
    pub fn wait_polling(&mut self, end: Deadline) -> Result<(), Error> {
        self.raw.wait_polling(end)
    }

    /// CH_INTSTATUS captured before the terminal ACK, or the unacknowledged abort status.
    /// Returns zero until either is available.
    pub fn terminal_status(&self) -> u32 {
        self.raw.terminal_status
    }

    /// End the borrow after a terminal observation and ACK, returning the transfer outcome.
    /// If quiescence is unconfirmed, the returned handle continues to guard the slots.
    pub fn try_reclaim(self) -> Result<Result<(), Error>, Self> {
        self.reclaim(&mut Mmio)
    }

    fn reclaim(mut self, io: &mut impl Registers) -> Result<Result<(), Error>, Self> {
        match self.raw.poll(io) {
            Some(result) if *self.raw.state == State::Idle => Ok(result),
            _ => Err(self),
        }
    }

    fn release_on_drop(&mut self, io: &mut impl Registers) {
        if *self.raw.state == State::Idle || self.raw.stop(io, Error::Cancelled).is_ok() {
            self.src.return_to_cpu();
            self.dst.return_to_cpu();
        }
        // When the stop is unconfirmed or unwinds, both slots stay with the engine.
    }
}

impl Drop for Transfer<'_> {
    fn drop(&mut self) {
        self.release_on_drop(&mut Mmio);
    }
}

/// A block with caller-managed DMA memory. A stop here proves local engine quiescence only.
#[must_use]
pub struct RawTransfer<'a> {
    n: u8,
    state: &'a mut State,
    result: Option<Result<(), Error>>,
    terminal_status: u32,
    _local: core::marker::PhantomData<core::cell::Cell<()>>,
}

impl<'a> RawTransfer<'a> {
    fn new(n: u8, state: &'a mut State) -> Self {
        Self {
            n,
            state,
            result: None,
            terminal_status: 0,
            _local: core::marker::PhantomData,
        }
    }

    /// Observe completion once, issuing a graceful stop on a bus error.
    pub fn try_complete(&mut self) -> Option<Result<(), Error>> {
        self.poll(&mut Mmio)
    }

    fn poll(&mut self, io: &mut impl Registers) -> Option<Result<(), Error>> {
        let state = &mut *self.state;
        if *state != State::Idle {
            let result = observe(io, self.n, state, &mut self.terminal_status);
            if let Some(Err(Error::Bus(status))) = result {
                self.stop_on_bus_error(io, status);
            }
            if self.result.is_none() {
                self.result = result;
            }
        }
        if matches!(*self.state, State::Idle | State::Aborted) {
            self.result
        } else {
            None
        }
    }

    #[rp1_rt::cold]
    fn stop_on_bus_error(&mut self, io: &mut impl Registers, status: u32) {
        let _ = self.stop(io, Error::Bus(status));
    }

    #[rp1_rt::cold]
    fn stop(&mut self, io: &mut impl Registers, reason: Error) -> Result<(), Error> {
        let state = &mut *self.state;
        if *state != State::Idle {
            let result = request_stop(io, self.n, state, reason, &mut self.terminal_status);
            self.result.get_or_insert(result);
        }
        match state {
            State::Idle => Ok(()),
            State::Aborted => Err(Error::Aborted),
            _ => Err(Error::Recovering),
        }
    }

    /// Request graceful disable and make one bounded stop check.
    pub fn try_cancel(&mut self) -> Result<(), Error> {
        self.stop(&mut Mmio, Error::Cancelled)
    }

    /// Request cancellation, then wait in thread mode for a timer-backed recheck.
    /// The recheck deadline is the earlier of `end` and 999 µs after entry. With IRQ40
    /// masked, a stop missed by the initial checks may be reported only at the alarm.
    #[inline(never)]
    pub fn cancel_until(&mut self, wait: &mut WaitTimer, end: Deadline) -> Result<(), Error> {
        __stop_until(wait, end, || self.try_cancel())
    }

    /// Explicitly poll against one absolute deadline in thread mode.
    pub fn wait_polling(&mut self, end: Deadline) -> Result<(), Error> {
        rp1_rt::interrupt::ThreadGuard::new().map_err(timer::Error::from)?;
        self.wait_accepted(&mut Mmio, end)
    }

    fn wait_accepted(&mut self, io: &mut impl Registers, end: Deadline) -> Result<(), Error> {
        // A retained outcome already has a completion proof or needs cleanup, so it
        // never enters the completion loop. Bus errors go straight to bounded cleanup.
        let result = self.result.unwrap_or_else(|| {
            wait_block(io, end, |io| {
                observe(io, self.n, self.state, &mut self.terminal_status)
            })
        });
        match result {
            Ok(()) => {
                self.result = Some(Ok(()));
                Ok(())
            }
            Err(error) => self.finish_error(io, error),
        }
    }

    #[rp1_rt::cold]
    fn finish_error(&mut self, io: &mut impl Registers, error: Error) -> Result<(), Error> {
        // Retain the operation's error even if completion races the cleanup.
        self.result.get_or_insert(Err(error));
        self.stop(io, error)?;
        Err(error)
    }
}

impl Drop for RawTransfer<'_> {
    fn drop(&mut self) {
        if *self.state == State::Active {
            let _ = self.try_cancel();
        }
    }
}

#[rp1_rt::isram]
fn start_block(
    io: &mut impl Registers,
    n: u8,
    state: &mut State,
    src: BusAddr,
    dst: BusAddr,
    len: u32,
) -> Result<(), Error> {
    let w = width_log2(src.0, dst.0, len);
    if len == 0 || len > max_block_bytes(w) {
        return Err(Error::Invalid);
    }
    src.range(len)?;
    dst.range(len)?;
    if src.0.abs_diff(dst.0) < u64::from(len) {
        return Err(Error::Invalid);
    }
    if *state != State::Idle {
        return Err(Error::Recovering);
    }
    if enabled(io, n) {
        return Err(Error::Busy);
    }
    let c = channel_base(n);
    for (offset, v) in [
        (ch::SAR, src.0),
        (ch::DAR, dst.0),
        (ch::BLOCK_TS, u64::from((len >> w) - 1)),
    ] {
        io.write_relaxed(c + offset, v as u32);
        io.write_relaxed(c + offset + 4, (v >> 32) as u32);
    }
    io.write_relaxed(
        c + ch::CTL_L,
        ctl_l::NONPOSTED_LAST_WRITE_EN
            | (ctl_l::AW_CACHE_NONBUFFERABLE << ctl_l::AW_CACHE_SHIFT)
            | (ctl_l::MSIZE_4 << ctl_l::DST_MSIZE_SHIFT)
            | (ctl_l::MSIZE_4 << ctl_l::SRC_MSIZE_SHIFT)
            | (w << ctl_l::DST_TR_WIDTH_SHIFT)
            | (w << ctl_l::SRC_TR_WIDTH_SHIFT),
    );
    let burst = BURST[n as usize];
    io.write_relaxed(
        c + ch::CTL_H,
        ctl_h::ARLEN_EN
            | ((burst - 1) << ctl_h::ARLEN_SHIFT)
            | ctl_h::AWLEN_EN
            | ((burst - 1) << ctl_h::AWLEN_SHIFT),
    );
    // Contiguous source and destination (single block), memory to memory with DMAC flow
    // control, priority = channel number (Linux `snps,priority`).
    io.write_relaxed(c + ch::CFG_L, 0);
    io.write_relaxed(
        c + ch::CFG_H,
        (u32::from(n) << cfg2::PRIORITY_SHIFT) | cfg2::HS_SEL_SRC | cfg2::HS_SEL_DST,
    );
    io.write_relaxed(c + ch::INTCLEAR, int::ALL);
    io.write_relaxed(c + ch::INTSIGNAL_ENA, 0);
    io.write_relaxed(
        c + ch::INTSTATUS_ENA,
        int::DMA_TFR_DONE | int::ALL_ERR | int::CH_SUSPENDED | int::CH_DISABLED | int::CH_ABORTED,
    );
    // The channel registers above are posted in order on the same AHB port. The barrier
    // completes earlier CPU stores to the source buffer (and keeps the compiler from sinking
    // them) before the enable; `write` is followed by `dsb` as well.
    rp1_proto::barrier();
    // If W1C has not retired the previous status, do not arm a new operation.
    if io.read(c + ch::INTSTATUS) != 0 {
        return Err(Error::Recovering);
    }
    *state = State::Active;
    io.write(
        BASE + common::CHEN,
        (1 << n) | (1 << (n as u32 + common::CHEN_WE_SHIFT)),
    );
    Ok(())
}

#[rp1_rt::isram]
fn observe(
    io: &mut impl Registers,
    n: u8,
    state: &mut State,
    terminal_status: &mut u32,
) -> Option<Result<(), Error>> {
    let c = channel_base(n);
    // Read EN before status. A completion is accepted only with both pieces of evidence
    // and never from an IRQ notification. IRQ40 stays masked.
    let idle = !enabled(io, n);
    let status = io.read(c + ch::INTSTATUS);
    if status & int::CH_ABORTED != 0 {
        // The databook's CH_ABORT field warns that AXI transactions may remain outstanding.
        *terminal_status = status;
        *state = State::Aborted;
    }
    if *state == State::Aborted {
        return Some(Err(Error::Aborted));
    }

    let fault = (status & int::ALL_ERR != 0).then_some(Err(Error::Bus(status)));
    if idle && status & (int::CH_DISABLED | int::DMA_TFR_DONE) != 0 {
        io.write_relaxed(c + ch::INTCLEAR, int::ALL);
        // All earlier writes by this channel to local SRAM precede later CPU accesses.
        rp1_proto::barrier();
        if io.read(c + ch::INTSTATUS) == 0 {
            *terminal_status = status;
            let completed = *state == State::Active && status & int::DMA_TFR_DONE != 0;
            *state = State::Idle;
            return Some(fault.unwrap_or(if completed {
                Ok(())
            } else {
                Err(Error::Cancelled)
            }));
        }
        // A failed ACK is ambiguous. Do not rearm on a later bare EN=0 observation.
    }
    fault
}

#[rp1_rt::cold]
fn request_stop(
    io: &mut impl Registers,
    n: u8,
    state: &mut State,
    reason: Error,
    terminal_status: &mut u32,
) -> Result<(), Error> {
    let result = observe(io, n, state, terminal_status);
    if *state != State::Active {
        return result.unwrap_or(Err(reason));
    }
    // CH_EN write enable affects this channel only. HW (2026-11-27): disable-only
    // produces a terminal status and stops local SRAM accesses (see docs/dma.md).
    // Never substitute CH_ABORT: the latter can leave work in the AXI fabric.
    let c = channel_base(n);
    io.write_relaxed(c + ch::INTSIGNAL_ENA, 0);
    io.write(BASE + common::CHEN, 1 << (n as u32 + common::CHEN_WE_SHIFT));
    *state = State::Stopping;
    let stopped = observe(io, n, state, terminal_status);
    result
        .or(stopped.filter(|result| *result != Err(Error::Cancelled)))
        .unwrap_or(Err(reason))
}

#[rp1_rt::isram]
fn wait_block<H: Registers, T>(
    io: &mut H,
    end: Deadline,
    mut probe: impl FnMut(&mut H) -> Option<Result<T, Error>>,
) -> Result<T, Error> {
    loop {
        let remaining = end.check(read_instant(io));
        if let Some(result) = probe(io) {
            return result;
        }
        remaining?;
        core::hint::spin_loop();
    }
}

fn enabled(io: &mut impl Registers, n: u8) -> bool {
    io.read(BASE + common::CHEN) & (1 << n) != 0
}

#[cfg(test)]
mod tests;
