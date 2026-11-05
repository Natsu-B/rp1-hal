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
//! write enables, so no read-modify-write is shared) and may be moved to the other core.
//!
//! # Addresses
//! The DMAC is a 40-bit bus master and does not use the processor's 32-bit map (DOC: RP1
//! Peripherals §2.2, table 1). [`BusAddr::from_proc`] translates the processor addresses it can
//! reach; processor-local ISRAM/DSRAM (stacks, ordinary `static`s) and the ROM are not visible
//! to the DMAC. Linux DDR is reached either through the outbound ATU window that
//! [`crate::shm`] programs (processor `0x8000_0000 + x` = bus `0x80_0000_0000 + x`) or through
//! the PCIe direct-mapped space, where host physical address `p` is bus
//! `0x10_0000_0000 + p` ([`BusAddr::host_phys`], the RP1 `dma-ranges` Linux uses).
//!
//! # Ordering
//! The Cortex-M3 has no data cache, so no cache maintenance is needed. Channel setup writes are
//! posted without `dsb`; immediately before the channel-enable write a compiler fence + `dsb`
//! ([`rp1_proto::barrier`]) completes every earlier CPU store to the source buffer and keeps
//! the compiler from moving buffer stores past the enable. Completion is reported only after
//! `DMA_TFR_DONE` *and* the channel-enable bit cleared, i.e. after the last AXI write response,
//! and is followed by the same barrier, so later CPU accesses to either buffer (also the
//! caller's ordinary, non-volatile ones) happen after the DMA. The same holds after a
//! cancellation. For Linux DDR, a write response from the PCIe endpoint means the posted write
//! was accepted, and later CPU posted writes through the same endpoint (e.g. a ring head) are
//! ordered behind it by PCIe posted-write ordering (DOC/HYP, see `docs/dma.md`).
//!
//! # Cancellation
//! A transfer is stopped (timeout, bus error, [`Transfer::cancel`], dropping a running
//! [`Transfer`]) by: `CH_SUSP` and a bounded wait for `CH_SUSPENDED`, `CH_EN = 0` and a bounded
//! wait for the enable bit to clear (the databook's graceful disable; Linux
//! `dma_chan_terminate_all` writes only `CH_EN = 0`), then, if it is still set, `CH_ABORT` (in
//! the high half of `DMAC_CHEN` on this 8-channel DMAC) and another bounded wait. If the channel
//! is *still* enabled, the driver spins until it clears, without a deadline: no exit releases a
//! buffer borrow while the DMAC may still access the buffer. A wedged DMAC (e.g. a PCIe target
//! that never answers) therefore hangs the calling core instead of returning.
use crate::{
    clocks::Clocks,
    linux_blocks::{Hardware, Mmio},
    pac::dw_axi_dmac::{
        BASE, CHANNEL_BASE, CHANNEL_STRIDE, MAX_BLOCK_TS, cfg2, ch, common, ctl_h, ctl_l, int,
    },
    scmi::{Generator, Kind, PhysicalState, RP1_CLOCKS, Round, id},
};

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
/// Bounded wait of each cancellation step (suspend, disable, abort).
const CANCEL_STEP_US: u32 = 10_000;
/// Largest transfer width: the 128-bit master (`snps,data-width = <4>`).
const MAX_WIDTH_LOG2: u32 = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    /// Length zero, not a multiple of the transfer width, or larger than one block
    /// ([`Channel::start`] only).
    Invalid,
    /// The address is not visible to the DMAC (processor-local memory, ROM, unsupported range).
    NotDmaVisible,
    /// Stale `Clocks`, or clk_dma is no longer running at 100 MHz.
    Precondition,
    /// The transfer (or the reset) did not finish in time. A timed-out transfer has been
    /// stopped (the channel is disabled) before this is returned.
    Timeout,
    /// The channel reported an error interrupt (`CH_INTSTATUS` value, decode / slave errors).
    /// The channel has been stopped before this is returned.
    Bus(u32),
    /// The channel was found enabled when it should be idle.
    Busy,
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
        self.0.checked_add(u64::from(len)).ok_or(Error::Invalid)?;
        Ok(self)
    }
}

fn generator() -> Generator {
    let Kind::Linux(clk) = RP1_CLOCKS[id::DMA as usize].kind else {
        unreachable!()
    };
    clk
}
fn clock_current(io: &mut impl Hardware) -> Result<(), Error> {
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
fn wait(io: &mut impl Hardware, address: usize, mask: u32, value: u32) -> Result<(), Error> {
    let start = io.now_us();
    loop {
        if io.read(address) & mask == value {
            return Ok(());
        }
        if io.now_us().wrapping_sub(start) > INIT_TIMEOUT_US {
            return if io.read(address) & mask == value {
                Ok(())
            } else {
                Err(Error::Timeout)
            };
        }
    }
}

/// Sole ownership of the DMA controller, issued by generated resources.
#[derive(Debug)]
pub struct Token {
    _private: (),
}
/// Initialization failed; ownership is retained for a retry.
#[derive(Debug)]
pub struct InitError {
    pub error: Error,
    pub token: Token,
}
impl Token {
    /// # Safety
    /// Generated resources only: one core owns the DMAC, its reset and clock. Linux's
    /// `rp1_dma` node and its `dmas` consumers must be disabled; SCMI must hold clk_dma.
    pub const unsafe fn new() -> Self {
        Self { _private: () }
    }
    /// Starts clk_dma at 100 MHz, releases the DMA reset, soft-resets the DMAC, disables every
    /// channel and its interrupts and enables the controller (interrupt output off).
    pub fn init(self, clocks: &Clocks) -> Result<Dma, InitError> {
        if !clocks.still_current() {
            return Err(InitError {
                error: Error::Precondition,
                token: self,
            });
        }
        match start(&mut Mmio) {
            Ok(()) => Ok(Dma::new()),
            Err(error) => Err(InitError { error, token: self }),
        }
    }
}

fn start(io: &mut impl Hardware) -> Result<(), Error> {
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
    wait(io, RESETS_DONE0, RESET_BIT, RESET_BIT)?;
    io.write(BASE + common::RESET, 1);
    wait(io, BASE + common::RESET, 1, 0)?;
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

/// The eight channels of an initialized controller. Linux numbering: channels 0 and 1 have the
/// 8-beat FIFOs (datasheet "channels 1 and 2").
#[derive(Debug)]
pub struct Dma {
    pub ch0: Channel<0>,
    pub ch1: Channel<1>,
    pub ch2: Channel<2>,
    pub ch3: Channel<3>,
    pub ch4: Channel<4>,
    pub ch5: Channel<5>,
    pub ch6: Channel<6>,
    pub ch7: Channel<7>,
}
impl Dma {
    fn new() -> Self {
        Self {
            ch0: Channel(()),
            ch1: Channel(()),
            ch2: Channel(()),
            ch3: Channel(()),
            ch4: Channel(()),
            ch5: Channel(()),
            ch6: Channel(()),
            ch7: Channel(()),
        }
    }
    /// Returns the controller token once every channel is back. Clock and reset stay on.
    pub fn free(self) -> Token {
        Token { _private: () }
    }
}

/// One DMA channel. Idle whenever no [`Transfer`] borrows it (a dropped transfer is stopped).
#[derive(Debug)]
pub struct Channel<const N: u8>(());

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
    /// Starts one memory-to-memory block of `len` bytes (at most one block,
    /// [`max_block_bytes`]) and returns a handle to poll it. Unlike [`Self::copy`] it does not
    /// re-check clk_dma (the SCMI server holds it for a declared DMA block), so the setup is
    /// only the channel registers.
    ///
    /// # Safety
    /// `src..src+len` must be readable and `dst..dst+len` writable by the DMAC for as long as
    /// the transfer runs, nothing else may access `dst` meanwhile, and neither may be memory the
    /// program relies on through Rust references. If the returned [`Transfer`] is leaked
    /// (`mem::forget`), the transfer keeps running after the borrow ends.
    pub unsafe fn start(
        &mut self,
        src: BusAddr,
        dst: BusAddr,
        len: u32,
    ) -> Result<Transfer<'_, N>, Error> {
        unsafe { start_block(&mut Mmio, N, src, dst, len) }?;
        Ok(Transfer {
            _channel: self,
            result: None,
        })
    }

    /// Copies `src` into `dst` (equal lengths) and waits for completion, splitting into blocks
    /// as needed. Both slices must each be one contiguous range of shared SRAM or of the
    /// outbound window; anything else (ISRAM, DSRAM, stack, peripherals) is refused with
    /// `NotDmaVisible` — peripheral registers need the unsafe [`Self::start`]. A timed-out or
    /// failed block is stopped before this returns; if the DMAC cannot be stopped this never
    /// returns (module docs, Cancellation). On error the contents of `dst` are unspecified
    /// (a prefix may have been written).
    pub fn copy(&mut self, src: &[u8], dst: &mut [u8], timeout_us: u32) -> Result<(), Error> {
        if src.len() != dst.len() || src.is_empty() || src.len() > u32::MAX as usize {
            return Err(Error::Invalid);
        }
        let len = src.len() as u32;
        clock_current(&mut Mmio)?;
        let s = memory(src.as_ptr() as usize, len)?;
        let d = memory(dst.as_mut_ptr() as usize, len)?;
        // The borrows of `src` and `dst` outlive every block: each is finished (or stopped)
        // before the next starts and before return.
        unsafe { copy_blocking(&mut Mmio, N, s, d, len, timeout_us) }
    }
}

/// Bound of one block of a ring copy ([`rp1_proto::shm::SlotCopy`]); a 64 KiB block takes
/// about 0.3 ms (HW, docs/dma.md).
pub const RING_TIMEOUT_US: u32 = 10_000;

/// A channel moves staged ring slots into the reader's memory for
/// [`rp1_proto::shm::RingTx::push_slice_staged`] (see `shm::RingDma`): blocking like
/// [`Channel::copy`] (blocks of [`RING_TIMEOUT_US`], stopped before return on error), without
/// the clk_dma check (the SCMI server holds the clock of a declared controller).
///
/// SAFETY: returns only after every block finished or was stopped (`copy_blocking`), and writes
/// nothing but `dst..dst + src.len()`.
unsafe impl<const N: u8> rp1_proto::shm::SlotCopy for Channel<N> {
    type Error = Error;
    unsafe fn copy_to_remote(&mut self, src: &[u8], dst: *mut u8) -> Result<(), Error> {
        unsafe { ring_copy(&mut Mmio, N, src.as_ptr() as usize, dst as usize, src.len()) }
    }
}

/// # Safety
/// `src..src+len` readable and `dst..dst+len` writable by the DMAC, nothing else writes `dst`.
unsafe fn ring_copy(
    io: &mut impl Hardware,
    n: u8,
    src: usize,
    dst: usize,
    len: usize,
) -> Result<(), Error> {
    if len == 0 || len > u32::MAX as usize {
        return Err(Error::Invalid);
    }
    let len = len as u32;
    let s = memory(src, len)?;
    let d = memory(dst, len)?;
    unsafe { copy_blocking(io, n, s, d, len, RING_TIMEOUT_US) }
}

/// Translates a slice of memory for [`Channel::copy`]: shared SRAM or the outbound window, one
/// contiguous range (16 B wide bursts are not for peripheral registers).
fn memory(start: usize, len: u32) -> Result<BusAddr, Error> {
    let first = BusAddr::from_proc(start)?;
    let last = BusAddr::from_proc(start + (len as usize - 1))?;
    if first.0 >= BusAddr::PERIPHERALS && first.0 < BusAddr::PERIPHERALS + 0x4000_0000
        || last.0 != first.0 + u64::from(len - 1)
    {
        return Err(Error::NotDmaVisible);
    }
    Ok(first)
}

/// How [`Transfer::cancel`] found or stopped the channel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stopped {
    /// The block had already finished (with this result) before the cancellation.
    Finished(Result<(), Error>),
    /// Stopped by `CH_EN = 0`; `suspended` = `CH_SUSPENDED` was seen first.
    Disabled { suspended: bool },
    /// Disable did not take effect in time; stopped by `CH_ABORT`.
    Aborted,
    /// Neither took effect within the bounded waits; the channel cleared later.
    Late,
}

/// A running block on channel `N`. Dropping it stops the transfer if it has not finished.
#[derive(Debug)]
pub struct Transfer<'a, const N: u8> {
    _channel: &'a mut Channel<N>,
    result: Option<Result<(), Error>>,
}
impl<const N: u8> Transfer<'_, N> {
    /// `None` while running; `Some(Ok)` once the last write completed (and from then on).
    pub fn poll(&mut self) -> Option<Result<(), Error>> {
        if self.result.is_none() {
            self.result = poll_block(&mut Mmio, N);
        }
        self.result
    }
    /// Polls until done or `timeout_us` passed (then stops it and returns [`Error::Timeout`]).
    pub fn wait(mut self, timeout_us: u32) -> Result<(), Error> {
        let r = match self.result {
            Some(r) => r,
            None => wait_block(&mut Mmio, N, timeout_us),
        };
        self.result = Some(r);
        r
    }
    /// Stops the block now (module docs, Cancellation) and reports how. The destination then
    /// holds a prefix of the copy; the channel is idle and reusable.
    pub fn cancel(mut self) -> Stopped {
        if let Some(r) = self.poll() {
            return Stopped::Finished(r);
        }
        let s = cancel(&mut Mmio, N);
        self.result = Some(Err(Error::Timeout));
        s
    }
}
impl<const N: u8> Drop for Transfer<'_, N> {
    fn drop(&mut self) {
        if self.poll().is_none() {
            cancel(&mut Mmio, N);
        }
    }
}

/// # Safety
/// See [`Channel::start`].
unsafe fn copy_blocking(
    io: &mut impl Hardware,
    n: u8,
    mut src: BusAddr,
    mut dst: BusAddr,
    mut len: u32,
    timeout_us: u32,
) -> Result<(), Error> {
    while len > 0 {
        let block = len.min(max_block_bytes(width_log2(src.0, dst.0, len)));
        unsafe { start_block(io, n, src, dst, block) }?;
        wait_block(io, n, timeout_us)?;
        src.0 += u64::from(block);
        dst.0 += u64::from(block);
        len -= block;
    }
    Ok(())
}

/// # Safety
/// See [`Channel::start`].
#[rp1_rt::isram]
unsafe fn start_block(
    io: &mut impl Hardware,
    n: u8,
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
    if io.read(BASE + common::CHEN) & (1 << n) != 0 {
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
        (ctl_l::MSIZE_4 << ctl_l::DST_MSIZE_SHIFT)
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
    io.write(
        BASE + common::CHEN,
        (1 << n) | (1 << (n as u32 + common::CHEN_WE_SHIFT)),
    );
    Ok(())
}

#[rp1_rt::isram]
fn poll_block(io: &mut impl Hardware, n: u8) -> Option<Result<(), Error>> {
    let c = channel_base(n);
    let status = io.read(c + ch::INTSTATUS);
    if status & int::ALL_ERR != 0 {
        cancel(io, n);
        return Some(Err(Error::Bus(status)));
    }
    if status & int::DMA_TFR_DONE != 0 && io.read(BASE + common::CHEN) & (1 << n) == 0 {
        io.write_relaxed(c + ch::INTCLEAR, int::ALL);
        // The DMA's writes are complete; later CPU buffer accesses must not move above this.
        rp1_proto::barrier();
        return Some(Ok(()));
    }
    None
}

#[rp1_rt::isram]
fn wait_block(io: &mut impl Hardware, n: u8, timeout_us: u32) -> Result<(), Error> {
    let start = io.now_us();
    loop {
        if let Some(r) = poll_block(io, n) {
            return r;
        }
        if io.now_us().wrapping_sub(start) > timeout_us {
            if let Some(r) = poll_block(io, n) {
                return r;
            }
            cancel(io, n);
            return Err(Error::Timeout);
        }
    }
}

fn enabled(io: &mut impl Hardware, n: u8) -> bool {
    io.read(BASE + common::CHEN) & (1 << n) != 0
}
/// Bounded wait (`CANCEL_STEP_US`) until channel `n` is disabled or `status & mask` is set.
fn settle(io: &mut impl Hardware, n: u8, mask: u32) -> bool {
    let start = io.now_us();
    loop {
        if !enabled(io, n) {
            return true;
        }
        if mask != 0 && io.read(channel_base(n) + ch::INTSTATUS) & mask != 0 {
            return false;
        }
        if io.now_us().wrapping_sub(start) > CANCEL_STEP_US {
            return !enabled(io, n);
        }
    }
}

/// Stops channel `n` (module docs, Cancellation) and returns only once its enable bit is
/// clear, i.e. the DMAC no longer accesses the transfer's buffers. Leaves the channel idle
/// with its status cleared.
#[rp1_rt::isram]
fn cancel(io: &mut impl Hardware, n: u8) -> Stopped {
    let c = channel_base(n);
    let susp = (1 << (n as u32 + common::CHEN_SUSP_SHIFT))
        | (1 << (n as u32 + common::CHEN_SUSP_WE_SHIFT));
    let how = if !enabled(io, n) {
        Stopped::Disabled { suspended: false }
    } else {
        io.write(BASE + common::CHEN, susp);
        settle(io, n, int::CH_SUSPENDED);
        let suspended = io.read(c + ch::INTSTATUS) & int::CH_SUSPENDED != 0;
        io.write(BASE + common::CHEN, 1 << (n as u32 + common::CHEN_WE_SHIFT));
        if settle(io, n, 0) {
            Stopped::Disabled { suspended }
        } else {
            io.write(
                BASE + common::CHEN_H,
                (1 << n) | (1 << (n as u32 + common::CHEN_H_ABORT_WE_SHIFT)),
            );
            if settle(io, n, 0) {
                Stopped::Aborted
            } else {
                // No deadline from here: returning would release buffers the DMAC still uses.
                while enabled(io, n) {}
                Stopped::Late
            }
        }
    };
    // Clear the suspend request (write-enabled) so the next enable starts running.
    io.write(
        BASE + common::CHEN,
        susp & !(0xff << common::CHEN_SUSP_SHIFT),
    );
    io.write(c + ch::INTCLEAR, int::ALL);
    rp1_proto::barrier();
    how
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use crate::scmi::Registers;
    use std::{collections::BTreeMap, vec::Vec};

    /// Fake DMAC: enabling a channel completes it on the next CHEN read (or reports `fault`, or
    /// keeps running if `stuck`). `resist`: 0 = suspend and disable work, 1 = only abort works,
    /// 2 = nothing works and the channel clears `late` CHEN reads after the abort write.
    #[derive(Default)]
    struct Fake {
        regs: BTreeMap<usize, u32>,
        writes: Vec<(usize, u32)>,
        now: u32,
        fault: u32,
        stuck: bool,
        resist: u8,
        late: u32,
        after_abort: Option<(u32, u32)>,
    }
    impl Fake {
        fn stop(&mut self, n: u32, status: u32) {
            let chen = BASE + common::CHEN;
            let v = self.regs.get(&chen).copied().unwrap_or(0) & !(1 << n);
            self.regs.insert(chen, v);
            let s = channel_base(n as u8) + ch::INTSTATUS;
            let old = self.regs.get(&s).copied().unwrap_or(0);
            self.regs.insert(s, old | status);
        }
    }
    impl Registers for Fake {
        fn read(&mut self, a: usize) -> u32 {
            if a == BASE + common::RESET {
                return 0;
            }
            if a == RESETS_DONE0 {
                return RESET_BIT;
            }
            if a == BASE + common::CHEN
                && let Some((n, k)) = self.after_abort
            {
                if k >= self.late {
                    self.stop(n, int::CH_ABORTED);
                    self.after_abort = None;
                } else {
                    self.after_abort = Some((n, k + 1));
                }
            }
            *self.regs.get(&a).unwrap_or(&0)
        }
        fn write(&mut self, a: usize, v: u32) {
            self.writes.push((a, v));
            match a {
                _ if a == BASE + common::CHEN => {
                    let old = self.regs.get(&a).copied().unwrap_or(0);
                    for n in 0..8u32 {
                        let (en, susp) = (1 << n, 1 << (n + 16));
                        if v & (susp << 8) != 0
                            && v & susp != 0
                            && old & en != 0
                            && self.resist == 0
                        {
                            let s = channel_base(n as u8) + ch::INTSTATUS;
                            let o = self.regs.get(&s).copied().unwrap_or(0);
                            self.regs.insert(s, o | int::CH_SUSPENDED);
                        }
                        if v & (en << 8) == 0 {
                            continue;
                        }
                        if v & en == 0 {
                            if self.resist == 0 {
                                self.stop(n, int::CH_DISABLED);
                            }
                        } else {
                            let now = self.regs.get(&a).copied().unwrap_or(0);
                            self.regs.insert(a, now | en);
                            if !self.stuck {
                                let status = if self.fault != 0 {
                                    self.fault
                                } else {
                                    int::DMA_TFR_DONE | int::BLOCK_TFR_DONE
                                };
                                self.regs
                                    .insert(channel_base(n as u8) + ch::INTSTATUS, status);
                                if self.fault == 0 {
                                    self.regs.insert(a, now);
                                }
                            }
                        }
                    }
                }
                _ if a == BASE + common::CHEN_H => {
                    for n in 0..8u32 {
                        if v & (1 << n) != 0 && v & (1 << (n + 8)) != 0 {
                            match self.resist {
                                0 | 1 => self.stop(n, int::CH_ABORTED),
                                _ => self.after_abort = Some((n, 0)),
                            }
                        }
                    }
                }
                _ if a > BASE + CHANNEL_BASE && (a - BASE) % CHANNEL_STRIDE == ch::INTCLEAR => {
                    let s = a - ch::INTCLEAR + ch::INTSTATUS;
                    let old = self.regs.get(&s).copied().unwrap_or(0);
                    self.regs.insert(s, old & !v);
                }
                _ => {
                    self.regs.insert(a, v);
                }
            }
        }
    }
    impl Hardware for Fake {
        fn now_us(&mut self) -> u32 {
            self.now += 1;
            self.now
        }
    }
    fn running() -> Fake {
        let mut f = Fake::default();
        start(&mut f).unwrap();
        f.writes.clear();
        f
    }
    fn reg(f: &Fake, a: usize) -> u32 {
        f.writes.iter().rev().find(|w| w.0 == a).unwrap().1
    }

    #[test]
    fn ring_copy_moves_staged_slots_from_sram_to_the_window() {
        let mut f = running();
        // 5 slots of 24 B from shared SRAM into ring slots in the outbound window.
        unsafe { ring_copy(&mut f, 1, 0x2000_1000, 0x8100_00c8, 120) }.unwrap();
        let c = channel_base(1);
        assert_eq!(reg(&f, c + ch::SAR), 0x2000_1000);
        assert_eq!(reg(&f, c + ch::SAR + 4), 0xc0);
        assert_eq!(reg(&f, c + ch::DAR), 0x0100_00c8);
        assert_eq!(reg(&f, c + ch::DAR + 4), 0x80);
        // 8-byte aligned destination: 64-bit beats, 15 of them.
        assert_eq!(reg(&f, c + ch::BLOCK_TS), 14);
        assert_eq!(f.regs[&(BASE + common::CHEN)] & 0xff, 0);
        // Only shared SRAM / the outbound window; nothing is started otherwise.
        f.writes.clear();
        for (s, d) in [(0x1000_0000, 0x8100_0000), (0x2000_1000, 0x4005_0000)] {
            assert_eq!(
                unsafe { ring_copy(&mut f, 1, s, d, 16) },
                Err(Error::NotDmaVisible)
            );
        }
        assert_eq!(
            unsafe { ring_copy(&mut f, 1, 0x2000_1000, 0x8100_0000, 0) },
            Err(Error::Invalid)
        );
        assert!(f.writes.is_empty());
        // A stuck block is stopped before the call returns.
        f.stuck = true;
        assert_eq!(
            unsafe { ring_copy(&mut f, 1, 0x2000_1000, 0x8100_0000, 64) },
            Err(Error::Timeout)
        );
        assert_eq!(f.regs[&(BASE + common::CHEN)] & 2, 0);
    }

    #[test]
    fn address_map_follows_the_datasheet() {
        assert_eq!(BusAddr::from_proc(0x2000_0000), Ok(BusAddr(0xc0_2000_0000)));
        assert_eq!(BusAddr::from_proc(0x2000_fffc), Ok(BusAddr(0xc0_2000_fffc)));
        assert_eq!(BusAddr::from_proc(0x4005_0060), Ok(BusAddr(0xc0_4005_0060)));
        assert_eq!(BusAddr::from_proc(0x8100_0000), Ok(BusAddr(0x80_0100_0000)));
        assert_eq!(BusAddr::host_phys(0x2200_0000), BusAddr(0x10_2200_0000));
        for local in [
            0,
            0x1000_0000,
            0x1000_2000,
            0x2001_0000,
            0x4040_0000,
            0xc000_0000,
        ] {
            assert_eq!(
                BusAddr::from_proc(local),
                Err(Error::NotDmaVisible),
                "{local:#x}"
            );
        }
    }

    #[test]
    fn width_follows_alignment_and_caps_at_128_bits() {
        assert_eq!(width_log2(0, 0, 4096), 4);
        assert_eq!(width_log2(4, 0, 4096), 2);
        assert_eq!(width_log2(0, 0, 6), 1);
        assert_eq!(width_log2(1, 0, 16), 0);
        assert_eq!(max_block_bytes(4), MAX_BLOCK_TS << 4);
    }

    #[test]
    fn init_starts_clock_releases_reset_and_enables_controller() {
        let mut f = Fake::default();
        start(&mut f).unwrap();
        assert!(f.writes.contains(&(RESETS_CLR0, RESET_BIT)));
        assert!(f.writes.contains(&(BASE + common::RESET, 1)));
        assert_eq!(reg(&f, BASE + common::CFG), common::CFG_DMAC_EN);
        assert_eq!(reg(&f, BASE + common::CHEN), 0xff00);
        clock_current(&mut f).unwrap();
    }

    #[test]
    fn single_block_programs_linux_compatible_fields() {
        let mut f = running();
        unsafe {
            start_block(
                &mut f,
                2,
                BusAddr(0xc0_2000_1000),
                BusAddr(0x80_0100_0040),
                0x400,
            )
        }
        .unwrap();
        let c = channel_base(2);
        assert_eq!(c, 0x4018_8300);
        assert_eq!(reg(&f, c + ch::SAR), 0x2000_1000);
        assert_eq!(reg(&f, c + ch::SAR + 4), 0xc0);
        assert_eq!(reg(&f, c + ch::DAR + 4), 0x80);
        assert_eq!(reg(&f, c + ch::BLOCK_TS), 0x400 / 16 - 1);
        assert_eq!(
            reg(&f, c + ch::CTL_L),
            (1 << 18) | (1 << 14) | (4 << 11) | (4 << 8)
        );
        assert_eq!(
            reg(&f, c + ch::CTL_H),
            (1 << 6) | (3 << 7) | (1 << 15) | (3 << 16)
        );
        assert_eq!(reg(&f, c + ch::CFG_L), 0);
        assert_eq!(reg(&f, c + ch::CFG_H), (2 << 20) | (1 << 3) | (1 << 4));
        assert_eq!(reg(&f, BASE + common::CHEN), (1 << 2) | (1 << 10));
        assert_eq!(poll_block(&mut f, 2), Some(Ok(())));
    }

    #[test]
    fn invalid_lengths_and_busy_channels_are_rejected_before_enable() {
        let mut f = running();
        let s = BusAddr(0xc0_2000_0000);
        assert_eq!(
            unsafe { start_block(&mut f, 0, s, s, 0) },
            Err(Error::Invalid)
        );
        assert_eq!(
            unsafe { start_block(&mut f, 0, s, s, max_block_bytes(4) + 16) },
            Err(Error::Invalid)
        );
        f.regs.insert(BASE + common::CHEN, 1);
        assert_eq!(
            unsafe { start_block(&mut f, 0, s, s, 16) },
            Err(Error::Busy)
        );
        assert!(!f.writes.iter().any(|w| w.0 == BASE + common::CHEN));
    }

    const SUSP1: u32 = (1 << 17) | (1 << 25);

    #[test]
    fn timeouts_suspend_then_disable_and_leave_the_channel_reusable() {
        let mut f = running();
        f.stuck = true;
        let s = BusAddr(0xc0_2000_0000);
        unsafe { start_block(&mut f, 1, s, s, 64) }.unwrap();
        assert_eq!(wait_block(&mut f, 1, 50), Err(Error::Timeout));
        let chen: Vec<_> = f
            .writes
            .iter()
            .filter(|w| w.0 == BASE + common::CHEN)
            .map(|w| w.1)
            .collect();
        // enable, suspend, disable (EN = 0 with its write enable), clear the suspend request.
        assert_eq!(chen, [(1 << 1) | (1 << 9), SUSP1, 1 << 9, 1 << 25]);
        assert!(!f.writes.iter().any(|w| w.0 == BASE + common::CHEN_H));
        assert_eq!(f.regs[&(BASE + common::CHEN)] & 2, 0);
        assert_eq!(f.regs[&(channel_base(1) + ch::INTSTATUS)], 0);
        f.stuck = false;
        unsafe { start_block(&mut f, 1, s, s, 64) }.unwrap();
        assert_eq!(wait_block(&mut f, 1, 50), Ok(()));
    }

    #[test]
    fn a_disable_that_does_not_take_effect_falls_back_to_abort_in_chen_high() {
        let mut f = running();
        f.stuck = true;
        f.resist = 1;
        let s = BusAddr(0xc0_2000_0000);
        unsafe { start_block(&mut f, 1, s, s, 64) }.unwrap();
        let mut c = f;
        assert_eq!(cancel(&mut c, 1), Stopped::Aborted);
        assert!(
            c.writes
                .contains(&(BASE + common::CHEN_H, (1 << 1) | (1 << 9)))
        );
        assert_eq!(c.regs[&(BASE + common::CHEN)] & 2, 0);
    }

    #[test]
    fn a_timeout_never_returns_while_the_channel_is_still_enabled() {
        let mut f = running();
        f.stuck = true;
        f.resist = 2;
        f.late = 100_000;
        let s = BusAddr(0xc0_2000_0000);
        unsafe { start_block(&mut f, 1, s, s, 64) }.unwrap();
        assert_eq!(wait_block(&mut f, 1, 50), Err(Error::Timeout));
        // Returned only after the (late) abort cleared the enable bit.
        assert!(f.after_abort.is_none());
        assert_eq!(f.regs[&(BASE + common::CHEN)] & 2, 0);
    }

    #[test]
    fn bus_errors_stop_the_channel_before_reporting() {
        let mut f = running();
        f.fault = int::DST_DEC_ERR;
        let s = BusAddr(0xc0_2000_0000);
        unsafe { start_block(&mut f, 5, s, BusAddr(0x80_0000_0000), 64) }.unwrap();
        assert_eq!(
            wait_block(&mut f, 5, 100),
            Err(Error::Bus(int::DST_DEC_ERR))
        );
        assert!(f.writes.contains(&(BASE + common::CHEN, 1 << 13)));
        assert_eq!(f.regs[&(BASE + common::CHEN)] & (1 << 5), 0);
    }

    #[test]
    fn a_bus_error_never_returns_while_the_channel_is_still_enabled() {
        let mut f = running();
        f.fault = int::SRC_SLV_ERR;
        f.resist = 2;
        f.late = 100_000;
        let s = BusAddr(0xc0_2000_0000);
        unsafe { start_block(&mut f, 3, s, s, 64) }.unwrap();
        assert_eq!(
            poll_block(&mut f, 3),
            Some(Err(Error::Bus(int::SRC_SLV_ERR)))
        );
        assert!(f.after_abort.is_none());
        assert_eq!(f.regs[&(BASE + common::CHEN)] & (1 << 3), 0);
    }

    #[test]
    fn copy_accepts_only_contiguous_memory() {
        assert_eq!(memory(0x2000_0010, 16), Ok(BusAddr(0xc0_2000_0010)));
        assert_eq!(memory(0x8000_0000, 4096), Ok(BusAddr(0x80_0000_0000)));
        assert_eq!(memory(0x4005_0060, 4), Err(Error::NotDmaVisible));
        assert_eq!(memory(0x2000_fff0, 0x20), Err(Error::NotDmaVisible));
    }

    #[test]
    fn blocking_copy_splits_into_blocks() {
        let mut f = running();
        let len = max_block_bytes(4) + 32;
        unsafe {
            copy_blocking(
                &mut f,
                0,
                BusAddr(0xc0_2000_0000),
                BusAddr(0x80_0000_0000),
                len,
                100,
            )
        }
        .unwrap();
        let sars: Vec<_> = f
            .writes
            .iter()
            .filter(|w| w.0 == channel_base(0) + ch::SAR)
            .map(|w| w.1)
            .collect();
        assert_eq!(sars, [0x2000_0000, 0x2000_0000 + max_block_bytes(4)]);
        assert_eq!(reg(&f, channel_base(0) + ch::BLOCK_TS), 1);
    }

    #[test]
    fn stale_clock_is_a_precondition_error() {
        let mut f = running();
        clock_current(&mut f).unwrap();
        let ctrl = generator().ctrl;
        f.regs.insert(ctrl, 0);
        assert_eq!(clock_current(&mut f), Err(Error::Precondition));
    }
}
