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
//! A transfer owns both pool leases while it holds exclusive access to the channel.
//! Cancellation requests graceful disable, then checks for a terminal state with a bounded
//! number of MMIO accesses. A running or ambiguous channel retains its leases. Drop requests
//! the same stop and quarantines the slots if it cannot prove quiescence. A forgotten transfer
//! leaves its pool claims and the channel's in-flight state intact.
//!
//! IRQ40 delivery is pending a separate hardware probe. Waiting for completion is available
//! through the explicit polling adapter. Thread-mode cancellation can use the timer IRQ.
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

/// A rejected DMA admission, with both CPU-owned leases.
pub type StartError<'p> = crate::InitError<(Lease<'p>, Lease<'p>), Error>;

impl<const N: u8> Channel<N> {
    /// Starts one local SRAM-to-SRAM block. Both leases are returned on admission failure.
    pub fn try_start<'ch, 'pool>(
        &'ch mut self,
        src: Lease<'pool>,
        dst: Lease<'pool>,
    ) -> Result<Transfer<'ch, 'pool, N>, StartError<'pool>> {
        self.start_leased(&mut Mmio, src, dst)
    }

    fn start_leased<'ch, 'pool>(
        &'ch mut self,
        io: &mut impl Registers,
        src: Lease<'pool>,
        dst: Lease<'pool>,
    ) -> Result<Transfer<'ch, 'pool, N>, StartError<'pool>> {
        if let Err(error) = leased_addresses(&src, &dst)
            .and_then(|(s, d)| start_block(io, N, &mut self.state, s, d, src.len()))
        {
            return Err(crate::InitError {
                error,
                resources: (src, dst),
            });
        }
        Ok(Transfer {
            raw: RawTransfer {
                channel: self,
                result: None,
            },
            buffers: Some((src, dst)),
        })
    }

    /// Explicitly select busy-polling in thread mode. DMA IRQ40 has no verified binding.
    pub fn polling(&mut self) -> Polling<'_, N> {
        Polling(self)
    }

    /// Start a single block with caller-managed bus addresses.
    ///
    /// # Safety
    /// Keep the complete, non-overlapping ranges valid and reserved even if the transfer is
    /// dropped or forgotten. The caller must independently establish remote drain and window
    /// stability before reusing or publishing remote memory. On an unconfirmed stop, preserve
    /// both ranges until external quiescence is established. Addresses must not change this
    /// controller, its clocks, firmware state, or any concurrently accessed memory.
    pub unsafe fn start_raw(
        &mut self,
        src: BusAddr,
        dst: BusAddr,
        len: u32,
    ) -> Result<RawTransfer<'_, N>, Error> {
        start_block(&mut Mmio, N, &mut self.state, src, dst, len)?;
        Ok(RawTransfer {
            channel: self,
            result: None,
        })
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
        let _ = request_stop(io, N, &mut self.state, Error::Cancelled);
        match self.state {
            State::Idle => Ok(()),
            State::Aborted => Err(Error::Aborted),
            _ => Err(Error::Recovering),
        }
    }
}

fn leased_addresses(src: &Lease<'_>, dst: &Lease<'_>) -> Result<(BusAddr, BusAddr), Error> {
    if src.len() != dst.len() {
        return Err(Error::Invalid);
    }
    Ok((src.address()?, dst.address()?))
}

/// Explicit polling adapter. Completion is checked against one absolute deadline.
pub struct Polling<'a, const N: u8>(&'a mut Channel<N>);

impl<const N: u8> Polling<'_, N> {
    /// Copies equal-length pool leases, returning both on success. On an error, proven idle
    /// slots return to the pool. Slots with an unconfirmed stop remain claimed.
    pub fn copy<'p>(
        &mut self,
        src: Lease<'p>,
        dst: Lease<'p>,
        timeout: Duration,
    ) -> Result<(Lease<'p>, Lease<'p>), Error> {
        rp1_rt::interrupt::ThreadGuard::new().map_err(timer::Error::from)?;
        let end = Deadline::after(timeout)?;
        clock_current(&mut Mmio)?;
        end.check(Timer::new().now())?;
        let mut transfer = self.0.try_start(src, dst)?;
        transfer.raw.wait_accepted(&mut Mmio, end)?;
        Ok(transfer.buffers.take().unwrap())
    }
}

/// Pool leases recovered after a terminal observation. A cancelled destination can contain
/// a prefix of the transfer. Inspect the result before consuming the destination.
#[non_exhaustive]
pub struct Completed<'p> {
    pub result: Result<(), Error>,
    pub src: Lease<'p>,
    pub dst: Lease<'p>,
}

/// A running or quarantined local transfer. Its leases are inaccessible until reclaimed.
#[must_use]
pub struct Transfer<'ch, 'p, const N: u8> {
    raw: RawTransfer<'ch, N>,
    buffers: Option<(Lease<'p>, Lease<'p>)>,
}

impl<'p, const N: u8> Transfer<'_, 'p, N> {
    /// Observe completion once, issuing a graceful stop on a bus error.
    pub fn try_complete(&mut self) -> Option<Result<(), Error>> {
        self.raw.try_complete()
    }

    /// Request graceful disable and make one bounded stop check.
    /// On `Recovering`, this handle still owns both slots.
    pub fn try_cancel(&mut self) -> Result<(), Error> {
        self.raw.try_cancel()
    }

    /// Request cancellation, then use the timer alarm for a bounded recheck in thread mode.
    pub fn cancel_until(&mut self, wait: &mut WaitTimer, end: Deadline) -> Result<(), Error> {
        self.raw.cancel_until(wait, end)
    }

    /// Explicitly poll against one absolute deadline in thread mode.
    pub fn wait_polling(&mut self, end: Deadline) -> Result<(), Error> {
        self.raw.wait_polling(end)
    }

    /// Return the leases only after a terminal observation and ACK.
    pub fn try_reclaim(mut self) -> Result<Completed<'p>, Self> {
        if self.try_complete().is_some() && self.raw.channel.state == State::Idle {
            let (src, dst) = self.buffers.take().unwrap();
            Ok(Completed {
                result: self.raw.result.unwrap(),
                src,
                dst,
            })
        } else {
            Err(self)
        }
    }

    fn release_on_drop(&mut self, io: &mut impl Registers) {
        if let Some(buffers) = self.buffers.take() {
            // On an unconfirmed stop the pool claims survive, including if a check unwinds.
            let mut buffers = core::mem::ManuallyDrop::new(buffers);
            if self.raw.stop(io, Error::Cancelled).is_ok() {
                // SAFETY: the terminal observation and ACK establish quiescence.
                unsafe { core::mem::ManuallyDrop::drop(&mut buffers) };
            }
        }
    }
}

impl<const N: u8> Drop for Transfer<'_, '_, N> {
    fn drop(&mut self) {
        self.release_on_drop(&mut Mmio);
    }
}

/// A block with caller-managed DMA memory. A stop here proves local engine quiescence only.
#[must_use]
pub struct RawTransfer<'a, const N: u8> {
    channel: &'a mut Channel<N>,
    result: Option<Result<(), Error>>,
}

impl<const N: u8> RawTransfer<'_, N> {
    pub fn try_complete(&mut self) -> Option<Result<(), Error>> {
        self.poll(&mut Mmio)
    }

    fn poll(&mut self, io: &mut impl Registers) -> Option<Result<(), Error>> {
        let state = &mut self.channel.state;
        if *state != State::Idle {
            let result = observe(io, N, state);
            if let Some(Err(Error::Bus(status))) = result {
                let _ = request_stop(io, N, state, Error::Bus(status));
            }
            if self.result.is_none() {
                self.result = result;
            }
        }
        if *state == State::Idle || *state == State::Aborted {
            self.result
        } else {
            None
        }
    }

    #[rp1_rt::cold]
    fn stop(&mut self, io: &mut impl Registers, reason: Error) -> Result<(), Error> {
        let state = &mut self.channel.state;
        if *state != State::Idle {
            let result = request_stop(io, N, state, reason);
            self.result.get_or_insert(result);
        }
        match state {
            State::Idle => Ok(()),
            State::Aborted => Err(Error::Aborted),
            _ => Err(Error::Recovering),
        }
    }

    pub fn try_cancel(&mut self) -> Result<(), Error> {
        self.stop(&mut Mmio, Error::Cancelled)
    }

    pub fn cancel_until(&mut self, wait: &mut WaitTimer, end: Deadline) -> Result<(), Error> {
        let mut context = wait.thread_context()?;
        let now = Timer::new().now();
        end.check(now)?;
        let limit = Deadline::after_ticks(now, CLEANUP_US)?;
        let end = Deadline::at(end.instant().min(limit.instant()));
        match self.try_cancel() {
            Err(Error::Recovering) => {}
            result => return result,
        }
        match context.wait_until(end, || self.try_complete()) {
            Ok(_) if self.channel.state == State::Idle => Ok(()),
            Ok(_) => Err(Error::Aborted),
            Err(timer::Error::Timeout) => self.try_cancel(),
            Err(error) => Err(error.into()),
        }
    }

    pub fn wait_polling(&mut self, end: Deadline) -> Result<(), Error> {
        rp1_rt::interrupt::ThreadGuard::new().map_err(timer::Error::from)?;
        self.wait_accepted(&mut Mmio, end)
    }

    fn wait_accepted(&mut self, io: &mut impl Registers, end: Deadline) -> Result<(), Error> {
        match wait_block(io, end, |io| {
            self.poll(io).or_else(|| self.result.filter(Result::is_err))
        }) {
            Ok(()) => Ok(()),
            Err(error) => self.finish_error(io, error),
        }
    }

    #[rp1_rt::cold]
    fn finish_error(&mut self, io: &mut impl Registers, error: Error) -> Result<(), Error> {
        self.stop(io, error)?;
        Err(error)
    }
}

impl<const N: u8> Drop for RawTransfer<'_, N> {
    fn drop(&mut self) {
        if self.channel.state == State::Active {
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
fn observe(io: &mut impl Registers, n: u8, state: &mut State) -> Option<Result<(), Error>> {
    let c = channel_base(n);
    // Read EN before status. A completion is accepted only with both pieces of evidence
    // and never from an IRQ notification. IRQ40 stays masked.
    let idle = !enabled(io, n);
    let status = io.read(c + ch::INTSTATUS);
    if status & int::CH_ABORTED != 0 {
        // The databook's CH_ABORT field warns that AXI transactions may remain outstanding.
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
            *state = State::Idle;
            return Some(fault.unwrap_or(if status & int::DMA_TFR_DONE != 0 {
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
) -> Result<(), Error> {
    let result = observe(io, n, state);
    if *state != State::Active {
        return result.unwrap_or(Err(reason));
    }
    // CH_EN write enable affects this channel only. Graceful disable drains its AXI work.
    // Never substitute CH_ABORT: the latter can leave work in the AXI fabric.
    let c = channel_base(n);
    io.write_relaxed(c + ch::INTSIGNAL_ENA, 0);
    io.write(BASE + common::CHEN, 1 << (n as u32 + common::CHEN_WE_SHIFT));
    *state = State::Stopping;
    let stopped = observe(io, n, state);
    result
        .or(stopped.filter(|result| *result != Err(Error::Cancelled)))
        .unwrap_or(Err(reason))
}

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
