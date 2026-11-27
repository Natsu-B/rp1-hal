//! Lab-only DMA proof, run by proc1 (which declares DMA channels 0–2).
//!
//! - Boot: SRAM -> SRAM copies through [`rp1_hal::dma::Channel::copy`] (correctness of every
//!   byte, several sizes) and their throughput (`DMA_SRAM ...` on UART0).
//! - Command 16 (after shm enable, RP1 holds the `bulk` writer role): RP1 SRAM -> Linux DDR
//!   through the outbound ATU window and through the PCIe direct-mapped space, against CPU
//!   posted writes (`DMA_DDR ...`). Linux pre-fills the block with a canary and verifies the
//!   final layout (lab/shm-linux `dma`):
//!   words [0, 128K) = ATU copies, [128K, 256K) = direct copies, each 1 KiB copy `k` is the
//!   pattern of [`pattern`] with word 0 replaced by `k`.
//! - Command 16, before the final layout: cancellation proof ([`abort`], `DMA_ABORT ...`): a
//!   byte-wide 4 KiB SRAM -> DDR block on channel 2 is cancelled at 1/4, 1/2 and 3/4 of its
//!   measured duration; the channel must be disabled on return, the destination must hold a
//!   prefix of the copy followed by the untouched canary (and not change afterwards), and the
//!   channel must then copy the full block correctly. A last step writes `CH_ABORT` directly
//!   (`DMAC_CHEN` high word, 0x01C) mid-transfer to confirm the 8-channel abort layout on HW
//!   (`DMA_ABORT_RAW`): the enable bit must clear with `CH_ABORTED` set.
#![cfg_attr(not(feature = "dma"), allow(unused_imports))]
use crate::rp1;
use crate::rp1::Proc1DmaChannels as Dma;
use core::cell::UnsafeCell;
#[cfg(feature = "dma")]
use rp1_hal::{
    dma::PollingTimeout,
    timer::{Deadline, Duration, Timer, WaitTimer},
};
use rp1_hal::{
    dma::{BusAddr, Channel, Error, Pool},
    log,
};

#[cfg(feature = "dma")]
pub const WORDS: usize = 256;
#[cfg(feature = "dma")]
const TIMEOUT: rp1_hal::fugit::MicrosDurationU32 = rp1_hal::fugit::MicrosDurationU32::millis(100);
#[cfg(feature = "dma")]
#[repr(C, align(16))]
struct Buf(UnsafeCell<[u32; WORDS]>);
#[cfg(feature = "dma")]
unsafe impl Sync for Buf {}
/// Shared SRAM (DMA-visible). Only proc1 touches these, and only here.
#[cfg(feature = "dma")]
#[unsafe(link_section = ".shared")]
static SRC: Buf = Buf(UnsafeCell::new([0; WORDS]));
#[cfg(feature = "dma")]
#[unsafe(link_section = ".shared")]
static LOCAL: Local = Local(UnsafeCell::new([0; 2 * WORDS * 4]));
#[cfg(feature = "dma")]
#[repr(align(16))]
struct Local(UnsafeCell<[u8; 2 * WORDS * 4]>);
#[cfg(feature = "dma")]
unsafe impl Sync for Local {}

#[cfg(feature = "ring-dma")]
const BIG_BYTES: usize = crate::ring_dma_protocol::STAGE_BYTES;
#[cfg(not(feature = "ring-dma"))]
const BIG_BYTES: usize = 4096;
#[repr(C, align(16))]
struct Big(UnsafeCell<[u8; BIG_BYTES]>);
unsafe impl Sync for Big {}
/// Source of the cancellation proof (shared SRAM; bytes never equal the canary 0xa5).
#[unsafe(link_section = ".shared")]
static BIG: Big = Big(UnsafeCell::new([0; BIG_BYTES]));

/// The CPU stage stays reusable on every return from SlotCopy. The DMA engine reads a
/// separate scratch pool whose claim can survive a wedged DMAC.
#[cfg(feature = "ring-dma")]
#[unsafe(link_section = ".shared")]
static RING_SCRATCH: Scratch = Scratch(UnsafeCell::new([0; BIG_BYTES + 15]));
#[cfg(feature = "ring-dma")]
#[repr(align(16))]
struct Scratch(UnsafeCell<[u8; BIG_BYTES + 15]>);
#[cfg(feature = "ring-dma")]
unsafe impl Sync for Scratch {}

/// # Safety
/// Call once on proc1. No code may independently access RING_SCRATCH during this epoch.
#[cfg(feature = "ring-dma")]
pub unsafe fn ring_scratch() -> Pool<1> {
    Pool::new(unsafe { &mut *RING_SCRATCH.0.get() }).unwrap()
}

/// # Safety
/// Only between commands, with no surviving borrow or DMA access to BIG.
#[cfg(feature = "ring-dma")]
pub unsafe fn ring_stage() -> &'static mut [u8] {
    unsafe { &mut *BIG.0.get() }
}
#[cfg(feature = "dma")]
fn now() -> u32 {
    unsafe { (0x400a_c028 as *const u32).read_volatile() }
}
#[cfg(feature = "dma")]
pub const fn pattern(i: usize, seed: u32) -> u32 {
    (i as u32).wrapping_mul(0x9e37_79b9) ^ seed
}
#[cfg(feature = "dma")]
fn src() -> *mut u32 {
    SRC.0.get().cast()
}
#[cfg(feature = "dma")]
fn fill(p: *mut u32, seed: u32) {
    for i in 0..WORDS {
        unsafe { p.add(i).write_volatile(pattern(i, seed)) };
    }
}

pub fn init(token: crate::rp1::Proc1Dma, clocks: &rp1_hal::clocks::Clocks) -> Option<Dma> {
    let dma = token.init(clocks).map_err(|e| *e.error());
    log!(
        "p1 dma init rc=0x{:x}\n",
        code(dma.as_ref().map(|_| ()).map_err(|e| *e))
    )
    .unwrap();
    dma.ok()
}

/// Result as a log code: 0 = Ok, 1..=5 = Invalid, NotDmaVisible, Precondition, Timeout, Busy; a bus error is `0x8000_0000 | CH_INTSTATUS`.
fn code(r: Result<(), Error>) -> u32 {
    match r {
        Ok(()) => 0,
        Err(Error::Invalid) => 1,
        Err(Error::NotDmaVisible) => 2,
        Err(Error::Precondition) => 3,
        Err(Error::Timeout) => 4,
        Err(Error::Busy) => 5,
        Err(Error::Bus(s)) => 0x8000_0000 | s,
        Err(Error::Recovering) => 6,
        Err(Error::Aborted) => 7,
        Err(Error::Cancelled) => 8,
        Err(Error::Wait(_)) => 9,
        Err(_) => u32::MAX,
    }
}

/// Boot SRAM -> SRAM proof on channel 0: every byte of the copy and the untouched tail.
/// Logs `DMA_SRAM_PASS` when every size copied exactly.
#[cfg(feature = "dma")]
#[inline(never)]
pub fn sram(dma: &mut Dma) -> Result<(), Error> {
    // SAFETY: the boot proof is the sole owner of this reservation, and runs exactly once.
    let pool = Pool::<2>::new(unsafe { &mut *LOCAL.0.get() })?;
    let mut source = pool.lease(0, WORDS * 4)?;
    let mut destination = pool.lease(1, WORDS * 4)?;
    let mut ok = true;
    for (k, bytes) in [4usize, 12, 16, 64, 1000, 1024].into_iter().enumerate() {
        source.set_len(WORDS * 4)?;
        destination.set_len(WORDS * 4)?;
        for (i, (s, d)) in source
            .as_mut_slice()
            .as_chunks_mut::<4>()
            .0
            .iter_mut()
            .zip(destination.as_mut_slice().as_chunks_mut::<4>().0)
            .enumerate()
        {
            s.copy_from_slice(&pattern(i, 0x5a5a_0000 + k as u32).to_le_bytes());
            d.copy_from_slice(&pattern(i, 0).to_le_bytes());
        }
        source.set_len(bytes)?;
        destination.set_len(bytes)?;
        let result = dma
            .ch0
            .copy(source, destination, PollingTimeout::new(TIMEOUT));
        let rc = code(result.as_ref().map(|_| ()).map_err(|e| *e));
        (source, destination) = match result {
            Ok(buffers) => buffers,
            Err(error) => {
                log!(
                    "DMA_SRAM bytes={} rc=0x{:x} bad_bytes=0\nDMA_SRAM_FAIL\n",
                    bytes,
                    rc
                )
                .unwrap();
                return Err(error);
            }
        };
        destination.set_len(WORDS * 4)?;
        let mut bad = 0;
        for (i, &got) in destination.as_slice().iter().enumerate() {
            let want = if i < bytes {
                source.as_slice()[i]
            } else {
                pattern(i / 4, 0).to_le_bytes()[i % 4]
            };
            bad += u32::from(got != want);
        }
        ok &= bad == 0;
        log!("DMA_SRAM bytes={} rc=0x{:x} bad_bytes={}\n", bytes, rc, bad).unwrap();
    }
    // Throughput: 1 KiB SRAM -> SRAM, 1000 copies on channel 0 (8-beat FIFO) and 2
    // (4-beat FIFO), against a CPU word copy from code in shared SRAM.
    let mut us = [0u32; 3];
    for (slot, u) in us.iter_mut().enumerate() {
        let t = now();
        for _ in 0..1000 {
            match slot {
                0 => {
                    (source, destination) =
                        dma.ch0
                            .copy(source, destination, PollingTimeout::new(TIMEOUT))?
                }
                1 => {
                    (source, destination) =
                        dma.ch2
                            .copy(source, destination, PollingTimeout::new(TIMEOUT))?
                }
                _ => {
                    let s = source.as_slice().as_ptr().cast::<u32>();
                    let d = destination.as_mut_slice().as_mut_ptr().cast::<u32>();
                    for i in 0..WORDS {
                        // SAFETY: the aligned pool slots each hold 1024 bytes.
                        unsafe { d.add(i).write_volatile(s.add(i).read_volatile()) };
                    }
                }
            }
        }
        *u = now().wrapping_sub(t);
    }
    log!(
        "DMA_SRAM_RATE bytes=1024 n=1000 ch0_us={} ch2_us={} cpu_us={}\n",
        us[0],
        us[1],
        us[2]
    )
    .unwrap();
    log!("{}\n", if ok { "DMA_SRAM_PASS" } else { "DMA_SRAM_FAIL" }).unwrap();
    if ok { Ok(()) } else { Err(Error::Invalid) }
}

/// One block of `bytes` from SRC to `dst` on `ch`, `n` times; elapsed µs or the first error.
#[cfg(feature = "dma")]
fn repeat<const N: u8>(
    ch: &mut Channel<N>,
    dst: BusAddr,
    bytes: u32,
    n: u32,
) -> Result<u32, Error> {
    let s = BusAddr::from_proc(src() as usize)?;
    let t = now();
    for _ in 0..n {
        let end = Deadline::after(TIMEOUT)?;
        unsafe { ch.start_raw(s, dst, bytes) }?.wait_polling(end)?;
    }
    Ok(now().wrapping_sub(t))
}

#[cfg(feature = "dma")]
fn raw_copy<const N: u8>(
    channel: &mut Channel<N>,
    src: BusAddr,
    dst: BusAddr,
    len: u32,
) -> Result<(), Error> {
    let end = Deadline::after(TIMEOUT)?;
    // SAFETY: command 16 owns the SRAM sources and outbound reservation. It fences all
    // subsequent DMA commands on error. Successful readback is the experiment's remote check.
    unsafe { channel.start_raw(src, dst, len) }?.wait_polling(end)
}

/// Pre-generated single-block control for the ring batch sizes, on the same channel 1.
/// Offset 8 forces 8-byte beats; aligned even-length blocks also exercise 16-byte beats.
#[cfg(feature = "dma")]
fn ring_raw(dma: &mut Dma, data: *mut u32) -> bool {
    // SAFETY: command 16 exclusively owns BIG and the bulk block until its response.
    let src = BIG.0.get().cast::<u8>();
    for i in 0..BIG_BYTES {
        unsafe { src.add(i).write_volatile((i as u8).wrapping_mul(7)) };
    }
    let dst = ((data as usize + 15) & !15) as *mut u8;
    for bytes in [768usize, 2304, 2112, 3072, 4032, 3960, 4080] {
        for offset in [0, 8] {
            let source = unsafe { src.add(offset) };
            let destination = unsafe { dst.add(offset) };
            let (Ok(s), Ok(d)) = (
                BusAddr::from_proc(source as usize),
                BusAddr::from_proc(destination as usize),
            ) else {
                return false;
            };
            let expected = |i: usize| {
                u32::from_le_bytes(core::array::from_fn(|j| {
                    ((offset + i + j) as u8).wrapping_mul(7)
                }))
            };
            // Every word differs from its expected value, even after a preceding condition.
            for i in (0..bytes).step_by(4) {
                let expected = expected(i);
                unsafe { destination.add(i).cast::<u32>().write_volatile(!expected) };
            }
            rp1_link::flush_window();
            let t = now();
            for _ in 0..1000 {
                if raw_copy(&mut dma.ch1, s, d, bytes as u32).is_err() {
                    return false;
                }
            }
            let us = now().wrapping_sub(t);
            rp1_link::flush_window();
            let mut bad = 0;
            for i in (0..bytes).step_by(4) {
                let expected = expected(i);
                if unsafe { destination.add(i).cast::<u32>().read_volatile() } != expected {
                    bad += 1;
                }
            }
            let width =
                1 << rp1_hal::dma::width_log2(source as u64, destination as u64, bytes as u32);
            log!(
                "RINGDMA_RAW bytes={} width={} count=1000 total_us={} bad={}\n",
                bytes,
                width,
                us,
                bad
            )
            .unwrap();
            if bad != 0 {
                return false;
            }
        }
    }
    true
}

/// Command 16. `data` = M3 pointer of the bulk block data (1 MiB, outbound window), `phys` =
/// its host physical address. Returns (errors, results for request[8..16]).
#[cfg(feature = "dma")]
#[inline(never)]
pub fn ddr(
    dma: &mut Dma,
    wait: &mut WaitTimer,
    data: *mut u32,
    phys: u64,
    seed: u32,
) -> (u32, [u32; 8]) {
    let mut out = [0u32; 8];
    let atu = match BusAddr::from_proc(data as usize) {
        Ok(a) => a,
        Err(_) => return (1, out),
    };
    let direct = BusAddr::host_phys(phys);
    if !ring_raw(dma, data) {
        return (1, out);
    }
    fill(src(), seed);
    // Small-transfer latency (the ring case): 16 B and 512 B blocks, 1000 each, ATU window.
    for (slot, bytes) in [(0, 16u32), (1, 512)] {
        match repeat(&mut dma.ch0, atu, bytes, 1000) {
            Ok(us) => out[slot] = us,
            Err(e) => {
                log!(
                    "DMA_DDR_ERR small bytes={} rc=0x{:x}\n",
                    bytes,
                    code(Err(e))
                )
                .unwrap();
                return (1, out);
            }
        }
    }
    // CPU baseline: 512 KiB of posted writes (same pattern) into the first half.
    let t = now();
    for k in 0..512 {
        for i in 0..WORDS {
            unsafe {
                data.add(k * WORDS + i)
                    .write_volatile(src().add(i).read_volatile())
            };
        }
    }
    rp1_link::flush_window();
    out[2] = now().wrapping_sub(t);
    // Large blocks: 64 KiB of the SRAM image, 8 times per path (timing only; overwritten below).
    for (slot, base) in [(6usize, atu), (7, direct)] {
        let t = now();
        for k in 0..8u64 {
            let r = raw_copy(
                &mut dma.ch0,
                BusAddr(BusAddr::SRAM),
                BusAddr(base.0 + k * 0x1_0000),
                0x1_0000,
            );
            if let Err(e) = r {
                log!("DMA_DDR_ERR large slot={} rc=0x{:x}\n", slot, code(Err(e))).unwrap();
                return (1, out);
            }
        }
        out[slot] = now().wrapping_sub(t);
    }
    // Cancellation proof in the first 4 KiB of the block (overwritten by the layout below).
    if !abort(dma, wait, data, atu) {
        return (1, out);
    }
    // 512 KiB per path in 1 KiB blocks, copy k carries k in word 0.
    for (half, base) in [(0usize, atu), (1, direct)] {
        let t = now();
        let mut failed = None;
        for k in 0..512u32 {
            unsafe { src().write_volatile(k | (half as u32) << 16) };
            let at = BusAddr(base.0 + u64::from(k) * 1024 + half as u64 * 512 * 1024);
            if let Err(e) = repeat(&mut dma.ch0, at, 1024, 1) {
                failed = Some(e);
                break;
            }
        }
        out[3 + half] = now().wrapping_sub(t);
        if let Some(e) = failed {
            log!("DMA_DDR_ERR half={} rc=0x{:x}\n", half, code(Err(e))).unwrap();
            return (1, out);
        }
    }
    unsafe { src().write_volatile(pattern(0, seed)) };
    // Readback through the window (M3 reads from Linux DDR), every word.
    let mut bad = 0;
    for w in 0..WORDS * 1024 {
        let i = w % WORDS;
        let want = if i == 0 {
            (w / WORDS % 512) as u32 | ((w / (WORDS * 512)) as u32) << 16
        } else {
            pattern(i, seed)
        };
        if unsafe { data.add(w).read_volatile() } != want {
            bad += 1;
        }
    }
    out[5] = bad;
    let errors = u32::from(bad != 0);
    log!(
        "DMA_DDR small16_n1000_us={} small512_n1000_us={} cpu_512k_us={} atu_512k_us={} direct_512k_us={} readback_bad={} atu_64k_x8_us={} direct_64k_x8_us={} errors={}\n",
        out[0],
        out[1],
        out[2],
        out[3],
        out[4],
        bad,
        out[6],
        out[7],
        errors
    )
    .unwrap();
    (errors, out)
}

#[cfg(feature = "dma")]
const CANARY: u32 = 0xa5a5_a5a5;
#[cfg(feature = "dma")]
const fn big(i: usize) -> u8 {
    ((i * 13 + 1) % 0xa5) as u8
}
/// Canary-fills the first `BIG_BYTES` of the DDR block (CPU posted writes, completed).
#[cfg(feature = "dma")]
fn canary(data: *mut u32) {
    for w in 0..BIG_BYTES / 4 {
        unsafe { data.add(w).write_volatile(CANARY) };
    }
    rp1_link::flush_window();
}
/// DDR bytes `1..BIG_BYTES - 1` against BIG: (copied prefix length, canary bytes missing after
/// the prefix, guard bytes 0 / BIG_BYTES - 1 changed, FNV hash of the whole range).
#[cfg(feature = "dma")]
fn inspect(data: *mut u32) -> (u32, u32, u32, u32) {
    let (mut copied, mut tail_bad, mut guard_bad, mut hash) = (0u32, 0u32, 0u32, 0x811c_9dc5u32);
    let mut in_prefix = true;
    for w in 0..BIG_BYTES / 4 {
        let word = unsafe { data.add(w).read_volatile() }.to_le_bytes();
        for (k, b) in word.into_iter().enumerate() {
            let i = w * 4 + k;
            hash = (hash ^ u32::from(b)).wrapping_mul(0x0100_0193);
            if i == 0 || i == BIG_BYTES - 1 {
                guard_bad += u32::from(b != 0xa5);
            } else if in_prefix && b == big(i) {
                copied += 1;
            } else {
                in_prefix = false;
                tail_bad += u32::from(b != 0xa5);
            }
        }
    }
    (copied, tail_bad, guard_bad, hash)
}
#[cfg(feature = "dma")]
fn chen() -> u32 {
    unsafe { (0x4018_8018 as *const u32).read_volatile() }
}
/// Cancellation proof on channel 2 (4-beat FIFO): byte-wide SRAM -> DDR (ATU window), so the
/// block is slow enough to be cancelled mid-way. Logs `DMA_ABORT_PASS` or `DMA_ABORT_FAIL`.
#[cfg(feature = "dma")]
#[inline(never)]
fn abort(dma: &mut Dma, wait: &mut WaitTimer, data: *mut u32, atu: BusAddr) -> bool {
    let b = BIG.0.get().cast::<u8>();
    for i in 0..BIG_BYTES {
        unsafe { b.add(i).write_volatile(big(i)) };
    }
    let len = BIG_BYTES as u32 - 2;
    let (src, dst) = match BusAddr::from_proc(b as usize + 1) {
        Ok(s) => (s, BusAddr(atu.0 + 1)),
        Err(_) => return false,
    };
    // Full block: timing and the reference result.
    let full = |dma: &mut Dma| {
        canary(data);
        let t = now();
        let r = code(raw_copy(&mut dma.ch2, src, dst, len));
        let us = now().wrapping_sub(t);
        let (copied, tail_bad, guard_bad, _) = inspect(data);
        (
            r,
            us,
            r == 0 && copied == len && tail_bad == 0 && guard_bad == 0,
            copied,
        )
    };
    let (r, full_us, mut ok, copied) = full(dma);
    log!(
        "DMA_ABORT_FULL bytes={} width=1 rc=0x{:x} us={} copied={} ok={}\n",
        len,
        r,
        full_us,
        copied,
        ok as u32
    )
    .unwrap();
    if !ok {
        return false;
    }
    let mut midway = 0;
    for quarter in 1..4u32 {
        canary(data);
        let delay = full_us * quarter / 4;
        let t = now();
        let cancelled = (|| {
            let mut x = unsafe { dma.ch2.start_raw(src, dst, len) }?;
            // Explicit polling is used for this mid-transfer control's stimulus time.
            Timer::new().delay_polling(Duration::micros(delay))?;
            x.cancel_until(wait, Deadline::after(Duration::micros(200))?)?;
            Ok::<_, Error>(x.try_complete() == Some(Err(Error::Cancelled)))
        })();
        let how = match cancelled {
            Ok(true) => 1,
            Ok(false) => 0,
            Err(error) => {
                log!("DMA_ABORT_FAIL q={} rc={}\n", quarter, code(Err(error))).unwrap();
                return false;
            }
        };
        let elapsed = now().wrapping_sub(t);
        let en = chen() & (1 << 2);
        let (copied, tail_bad, guard_bad, hash) = inspect(data);
        // Nothing may change after cancel() returned.
        if wait.delay(Duration::micros(2 * full_us + 100)).is_err() {
            return false;
        }
        let (_, _, _, hash2) = inspect(data);
        let late_change = u32::from(hash != hash2);
        let good = how != 0
            && en == 0
            && copied < len
            && tail_bad == 0
            && guard_bad == 0
            && late_change == 0;
        midway += u32::from(good && copied > 0);
        log!(
            "DMA_ABORT q={} delay_us={} how={} chen_after={} copied={} len={} tail_bad={} guard_bad={} late_change={} elapsed_us={} ok={}\n",
            quarter,
            delay,
            how,
            en,
            copied,
            len,
            tail_bad,
            guard_bad,
            late_change,
            elapsed,
            good as u32
        )
        .unwrap();
        // The same channel copies the whole block correctly afterwards.
        let (r, us, reuse_ok, copied) = full(dma);
        log!(
            "DMA_ABORT_REUSE q={} rc=0x{:x} us={} copied={} ok={}\n",
            quarter,
            r,
            us,
            copied,
            reuse_ok as u32
        )
        .unwrap();
        if !good || !reuse_ok {
            return false;
        }
    }
    ok &= midway > 0;
    ok &= raw_abort(dma, wait, data, src, dst, len, full_us);
    log!(
        "{} midway={}\n",
        if ok {
            "DMA_ABORT_PASS"
        } else {
            "DMA_ABORT_FAIL"
        },
        midway
    )
    .unwrap();
    ok
}

/// Register-layout probe: `CH_ABORT` for channel 2 at `DMAC_CHEN[39:32]` (+ write enable at
/// `[47:40]`) only, no suspend/disable. Then the Transfer is dropped (finds the channel idle).
#[cfg(feature = "dma")]
fn raw_abort(
    dma: &mut Dma,
    wait: &mut WaitTimer,
    data: *mut u32,
    src: BusAddr,
    dst: BusAddr,
    len: u32,
    full_us: u32,
) -> bool {
    const CHEN_H: usize = 0x4018_801c;
    const CH2_INTSTATUS: usize = 0x4018_8000 + 0x100 + 2 * 0x100 + 0x88;
    canary(data);
    let (mut en, mut clear_us, mut status) = (1, 0, 0);
    if let Ok(mut x) = unsafe { dma.ch2.start_raw(src, dst, len) } {
        if Timer::new()
            .delay_polling(Duration::micros(full_us / 2))
            .is_err()
        {
            return false;
        }
        unsafe { (CHEN_H as *mut u32).write_volatile((1 << 2) | (1 << 10)) };
        let t = now();
        if wait_raw_abort_polling().is_err() {
            return false;
        }
        clear_us = now().wrapping_sub(t);
        en = chen() & (1 << 2);
        status = unsafe { (CH2_INTSTATUS as *const u32).read_volatile() };
        let _ = x.try_complete();
    }
    if en != 0 || status & (1 << 31) == 0 {
        log!(
            "DMA_ABORT_RAW reg=0x01c chen_after={} clear_us={} intstatus=0x{:08x} ok=0\n",
            en,
            clear_us,
            status
        )
        .unwrap();
        return false;
    }
    // Experimental legacy control: use only in the reserved DDR test region. Settling and
    // checking a stable observed copy are evidence for the lab's reuse, not a safe HAL proof.
    if wait.delay(Duration::micros(2 * full_us + 100)).is_err() {
        return false;
    }
    let before = inspect(data).3;
    if wait.delay(Duration::micros(2 * full_us + 100)).is_err() || inspect(data).3 != before {
        return false;
    }
    // SAFETY: the raw-only lab assumes the previous readback and settle drained this dedicated
    // test range. It may not use this assumption to reclaim safe pool leases or user memory.
    if unsafe { dma.ch2.acknowledge_external_abort() }.is_err() {
        return false;
    }
    let (copied, tail_bad, guard_bad, _) = inspect(data);
    let (r, _, reuse_ok, _) = {
        canary(data);
        let r = code(raw_copy(&mut dma.ch2, src, dst, len));
        let (c, t, g, _) = inspect(data);
        (r, 0, r == 0 && c == len && t == 0 && g == 0, c)
    };
    let good = en == 0 && status & (1 << 31) != 0 && copied < len && guard_bad == 0 && reuse_ok;
    log!(
        "DMA_ABORT_RAW reg=0x01c chen_after={} clear_us={} intstatus=0x{:08x} aborted={} copied={} tail_bad={} guard_bad={} reuse_rc=0x{:x} reuse_ok={} ok={}\n",
        en,
        clear_us,
        status,
        status >> 31,
        copied,
        tail_bad,
        guard_bad,
        r,
        reuse_ok as u32,
        good as u32
    )
    .unwrap();
    good
}

/// Explicit polling for the raw register control, isolated from the bounded HAL stop path.
#[cfg(feature = "dma")]
fn wait_raw_abort_polling() -> Result<(), Error> {
    const CHANNEL: u32 = 1 << 2;
    let end = Deadline::after(Duration::micros(200))?;
    let timer = Timer::new();
    while chen() & CHANNEL != 0 && timer.now() < end.instant() {
        core::hint::spin_loop();
    }
    Ok(())
}
