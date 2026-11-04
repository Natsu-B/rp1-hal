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
//! The Cortex-M3 has no data cache, so no cache maintenance is needed. Every register write of
//! this driver is followed by `dsb` ([`crate::write`]), so CPU stores to a source buffer are
//! complete before the channel is enabled. Completion is reported only after `DMA_TFR_DONE`
//! *and* the channel-enable bit cleared, i.e. after the last AXI write response; reads of the
//! destination after that see the data (SRAM). For Linux DDR, a write response from the PCIe
//! endpoint means the posted write was accepted, and later CPU posted writes through the same
//! endpoint (e.g. a ring head) are ordered behind it by PCIe posted-write ordering (DOC/HYP,
//! see `docs/dma.md`).
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
/// Linux `snps,axi-max-burst-len` of rp1.dtsi, written to ARLEN/AWLEN as the Linux driver does.
const BURST: [u32; 8] = [8, 8, 4, 4, 4, 4, 4, 4];
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
    /// The transfer (or the reset / abort) did not finish in time. A timed-out transfer has
    /// been aborted.
    Timeout,
    /// The channel reported an error interrupt (`CH_INTSTATUS` value, decode / slave errors).
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

/// One DMA channel. Idle whenever no [`Transfer`] borrows it (a dropped transfer is aborted).
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
    /// [`max_block_bytes`]) and returns a handle to poll it.
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
    /// as needed. Both slices must be in DMAC-visible memory (shared SRAM, a peripheral, or the
    /// outbound window); a timed-out block is aborted before this returns.
    pub fn copy(&mut self, src: &[u8], dst: &mut [u8], timeout_us: u32) -> Result<(), Error> {
        if src.len() != dst.len() || src.is_empty() || src.len() > u32::MAX as usize {
            return Err(Error::Invalid);
        }
        let len = src.len() as u32;
        let s = BusAddr::from_proc(src.as_ptr() as usize)?;
        let d = BusAddr::from_proc(dst.as_mut_ptr() as usize)?;
        BusAddr::from_proc(src.as_ptr() as usize + src.len() - 1)?;
        BusAddr::from_proc(dst.as_ptr() as usize + dst.len() - 1)?;
        // The borrows of `src` and `dst` outlive every block: each is finished (or aborted)
        // before the next starts and before return.
        unsafe { copy_blocking(&mut Mmio, N, s, d, len, timeout_us) }
    }
}

/// A running block on channel `N`. Dropping it aborts the transfer if it has not finished.
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
    /// Polls until done or `timeout_us` passed (then aborts and returns [`Error::Timeout`]).
    pub fn wait(mut self, timeout_us: u32) -> Result<(), Error> {
        let r = match self.result {
            Some(r) => r,
            None => wait_block(&mut Mmio, N, timeout_us),
        };
        self.result = Some(r);
        r
    }
}
impl<const N: u8> Drop for Transfer<'_, N> {
    fn drop(&mut self) {
        if self.poll().is_none() {
            let _ = abort(&mut Mmio, N);
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
    clock_current(io)?;
    if io.read(BASE + common::CHEN) & (1 << n) != 0 {
        return Err(Error::Busy);
    }
    let c = channel_base(n);
    for (offset, v) in [
        (ch::SAR, src.0),
        (ch::DAR, dst.0),
        (ch::BLOCK_TS, u64::from((len >> w) - 1)),
    ] {
        io.write(c + offset, v as u32);
        io.write(c + offset + 4, (v >> 32) as u32);
    }
    io.write(
        c + ch::CTL_L,
        (ctl_l::MSIZE_4 << ctl_l::DST_MSIZE_SHIFT)
            | (ctl_l::MSIZE_4 << ctl_l::SRC_MSIZE_SHIFT)
            | (w << ctl_l::DST_TR_WIDTH_SHIFT)
            | (w << ctl_l::SRC_TR_WIDTH_SHIFT),
    );
    let burst = BURST[n as usize];
    io.write(
        c + ch::CTL_H,
        ctl_h::ARLEN_EN
            | (burst << ctl_h::ARLEN_SHIFT)
            | ctl_h::AWLEN_EN
            | (burst << ctl_h::AWLEN_SHIFT),
    );
    // Contiguous source and destination (single block), memory to memory with DMAC flow
    // control, priority = channel number (Linux `snps,priority`).
    io.write(c + ch::CFG_L, 0);
    io.write(
        c + ch::CFG_H,
        (u32::from(n) << cfg2::PRIORITY_SHIFT) | cfg2::HS_SEL_SRC | cfg2::HS_SEL_DST,
    );
    io.write(c + ch::INTCLEAR, int::ALL);
    io.write(c + ch::INTSIGNAL_ENA, 0);
    io.write(c + ch::INTSTATUS_ENA, int::DMA_TFR_DONE | int::ALL_ERR);
    io.write(
        BASE + common::CHEN,
        (1 << n) | (1 << (n as u32 + common::CHEN_WE_SHIFT)),
    );
    Ok(())
}

fn poll_block(io: &mut impl Hardware, n: u8) -> Option<Result<(), Error>> {
    let c = channel_base(n);
    let status = io.read(c + ch::INTSTATUS);
    if status & int::ALL_ERR != 0 {
        let _ = abort(io, n);
        io.write(c + ch::INTCLEAR, int::ALL);
        return Some(Err(Error::Bus(status)));
    }
    if status & int::DMA_TFR_DONE != 0 && io.read(BASE + common::CHEN) & (1 << n) == 0 {
        io.write(c + ch::INTCLEAR, int::ALL);
        return Some(Ok(()));
    }
    None
}

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
            abort(io, n)?;
            return Err(Error::Timeout);
        }
    }
}

/// Aborts channel `n` and waits for its enable bit to clear (the DMAC finishes the current AXI
/// burst first). Error only if the channel stays enabled for 10 ms.
fn abort(io: &mut impl Hardware, n: u8) -> Result<(), Error> {
    if io.read(BASE + common::CHEN) & (1 << n) == 0 {
        return Ok(());
    }
    io.write(BASE + common::CHABORT, (1 << n) | (1 << (n as u32 + 8)));
    let r = wait(io, BASE + common::CHEN, 1 << n, 0);
    io.write(channel_base(n) + ch::INTCLEAR, int::ALL);
    r
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use crate::scmi::Registers;
    use std::{collections::BTreeMap, vec::Vec};

    /// Fake DMAC: enabling a channel completes it on the next CHEN read (or reports `fault`).
    #[derive(Default)]
    struct Fake {
        regs: BTreeMap<usize, u32>,
        writes: Vec<(usize, u32)>,
        now: u32,
        fault: u32,
        stuck: bool,
    }
    impl Registers for Fake {
        fn read(&mut self, a: usize) -> u32 {
            if a == BASE + common::RESET {
                return 0;
            }
            if a == RESETS_DONE0 {
                return RESET_BIT;
            }
            *self.regs.get(&a).unwrap_or(&0)
        }
        fn write(&mut self, a: usize, v: u32) {
            self.writes.push((a, v));
            match a {
                _ if a == BASE + common::CHEN => {
                    let we = v >> 8 & 0xff;
                    let old = self.regs.get(&a).copied().unwrap_or(0);
                    let new = (old & !we) | (v & we);
                    self.regs.insert(a, new);
                    for n in 0..8u8 {
                        if new & (1 << n) != 0 && !self.stuck {
                            let status = if self.fault != 0 {
                                self.fault
                            } else {
                                int::DMA_TFR_DONE | int::BLOCK_TFR_DONE
                            };
                            self.regs.insert(channel_base(n) + ch::INTSTATUS, status);
                            if self.fault == 0 {
                                self.regs.insert(a, new & !(1 << n));
                            }
                        }
                    }
                }
                _ if a == BASE + common::CHABORT => {
                    let we = v >> 8 & 0xff;
                    let old = self.regs.get(&(BASE + common::CHEN)).copied().unwrap_or(0);
                    if !self.stuck {
                        self.regs.insert(BASE + common::CHEN, old & !(v & we));
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
            (1 << 6) | (4 << 7) | (1 << 15) | (4 << 16)
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

    #[test]
    fn bus_errors_abort_and_report_status() {
        let mut f = running();
        f.fault = int::DST_DEC_ERR;
        let s = BusAddr(0xc0_2000_0000);
        unsafe { start_block(&mut f, 5, s, BusAddr(0x80_0000_0000), 64) }.unwrap();
        assert_eq!(
            wait_block(&mut f, 5, 100),
            Err(Error::Bus(int::DST_DEC_ERR))
        );
        assert!(
            f.writes
                .contains(&(BASE + common::CHABORT, (1 << 5) | (1 << 13)))
        );
        assert_eq!(f.regs[&(BASE + common::CHEN)] & (1 << 5), 0);
    }

    #[test]
    fn timeouts_abort_and_a_stuck_abort_is_reported() {
        let mut f = running();
        f.stuck = true;
        let s = BusAddr(0xc0_2000_0000);
        unsafe { start_block(&mut f, 1, s, s, 64) }.unwrap();
        assert_eq!(wait_block(&mut f, 1, 50), Err(Error::Timeout));
        assert!(f.writes.iter().any(|w| w.0 == BASE + common::CHABORT));
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
        let ctrl = generator().ctrl;
        f.regs.insert(ctrl, 0);
        let s = BusAddr(0xc0_2000_0000);
        assert_eq!(
            unsafe { start_block(&mut f, 0, s, s, 16) },
            Err(Error::Precondition)
        );
    }
}
