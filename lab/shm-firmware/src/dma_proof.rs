//! Lab-only DMA proof, run by proc1 (which declares `dma = {}`).
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
use core::cell::UnsafeCell;
#[cfg(feature = "dma")]
use rp1_hal::shm::SlotCopy;
use rp1_hal::{
    dma::{BusAddr, Channel, Dma, Error, Stopped},
    log,
};

#[cfg(feature = "dma")]
pub const WORDS: usize = 256;
#[cfg(feature = "dma")]
const TIMEOUT_US: u32 = 100_000;
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
static DST: Buf = Buf(UnsafeCell::new([0; WORDS]));

const BIG_BYTES: usize = 4096;
#[repr(C, align(16))]
struct Big(UnsafeCell<[u8; BIG_BYTES]>);
unsafe impl Sync for Big {}
/// Source of the cancellation proof (shared SRAM; bytes never equal the canary 0xa5).
#[unsafe(link_section = ".shared")]
static BIG: Big = Big(UnsafeCell::new([0; BIG_BYTES]));

/// The cancellation proof's source buffer, lent to the DMA ring proof (command 17) as its
/// stage.
///
/// # Safety
/// Only between commands (command 16 is not running) and only one borrow at a time.
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
fn dst() -> *mut u32 {
    DST.0.get().cast()
}
#[cfg(feature = "dma")]
fn fill(p: *mut u32, seed: u32) {
    for i in 0..WORDS {
        unsafe { p.add(i).write_volatile(pattern(i, seed)) };
    }
}

pub fn init(token: rp1_hal::dma::Token, clocks: &rp1_hal::clocks::Clocks) -> Option<Dma> {
    let dma = token.init(clocks).map_err(|e| e.error);
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
    }
}

/// Boot SRAM -> SRAM proof on channel 0: every byte of the copy and the untouched tail.
/// Logs `DMA_SRAM_PASS` when every size copied exactly.
#[cfg(feature = "dma")]
#[inline(never)]
pub fn sram(dma: &mut Dma) {
    let mut ok = true;
    let (s8, d8) = (src().cast::<u8>(), dst().cast::<u8>());
    for (k, bytes) in [4usize, 12, 16, 64, 1000, 1024].into_iter().enumerate() {
        fill(src(), 0x5a5a_0000 + k as u32);
        fill(dst(), 0);
        let r = code(dma.ch0.copy(
            unsafe { core::slice::from_raw_parts(s8, bytes) },
            unsafe { core::slice::from_raw_parts_mut(d8, bytes) },
            TIMEOUT_US,
        ));
        let mut bad = 0;
        for i in 0..WORDS * 4 {
            let got = unsafe { d8.add(i).read_volatile() };
            // The tail keeps fill(dst, 0): byte i % 4 of pattern(i / 4, 0).
            let want = if i < bytes {
                unsafe { s8.add(i).read_volatile() }
            } else {
                pattern(i / 4, 0).to_le_bytes()[i % 4]
            };
            bad += u32::from(got != want);
        }
        ok &= r == 0 && bad == 0;
        log!("DMA_SRAM bytes={} rc=0x{:x} bad_bytes={}\n", bytes, r, bad).unwrap();
    }
    // Throughput: 1 KiB SRAM -> SRAM, 1000 copies on channel 0 (8-beat FIFO) and 2 (4-beat),
    // against a CPU word copy (code in shared SRAM).
    // The slices are created per copy, so no `&mut` to DST is live during the CPU loop.
    let s = || unsafe { core::slice::from_raw_parts(s8, 1024) };
    let d = || unsafe { core::slice::from_raw_parts_mut(d8, 1024) };
    let mut us = [0u32; 3];
    for (slot, u) in us.iter_mut().enumerate() {
        let t = now();
        for _ in 0..1000 {
            match slot {
                0 => ok &= dma.ch0.copy(s(), d(), TIMEOUT_US).is_ok(),
                1 => ok &= dma.ch2.copy(s(), d(), TIMEOUT_US).is_ok(),
                _ => {
                    for i in 0..WORDS {
                        unsafe { dst().add(i).write_volatile(src().add(i).read_volatile()) };
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
        unsafe { ch.start(s, dst, bytes) }?.wait(TIMEOUT_US)?;
    }
    Ok(now().wrapping_sub(t))
}

/// Pre-generated single-block control for the ring batch sizes, on the same channel 1.
/// Offset 8 forces 8-byte beats; aligned even-length blocks also exercise 16-byte beats.
#[cfg(feature = "dma")]
fn ring_raw(dma: &mut Dma, data: *mut u32) -> bool {
    // SAFETY: command 16 exclusively owns BIG and the bulk block until its response.
    let src = unsafe { &mut *BIG.0.get() };
    for (i, b) in src.iter_mut().enumerate() {
        *b = (i as u8).wrapping_mul(7);
    }
    let dst = ((data as usize + 15) & !15) as *mut u8;
    for bytes in [768usize, 2304, 2112, 3072, 4032, 3960, 4080] {
        for offset in [0, 8] {
            let source = &src[offset..offset + bytes];
            let destination = unsafe { dst.add(offset) };
            let t = now();
            for _ in 0..1000 {
                if unsafe { dma.ch1.copy_to_remote(source, destination) }.is_err() {
                    return false;
                }
            }
            let us = now().wrapping_sub(t);
            rp1_hal::shm::complete_posted_writes();
            let mut bad = 0;
            for i in (0..bytes).step_by(4) {
                let expected = u32::from_le_bytes(source[i..i + 4].try_into().unwrap());
                if unsafe { destination.add(i).cast::<u32>().read_volatile() } != expected {
                    bad += 1;
                }
            }
            let width = 1
                << rp1_hal::dma::width_log2(
                    source.as_ptr() as u64,
                    destination as u64,
                    bytes as u32,
                );
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
pub fn ddr(dma: &mut Dma, data: *mut u32, phys: u64, seed: u32) -> (u32, [u32; 8]) {
    let mut out = [0u32; 8];
    let mut errors = 0;
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
                errors += 1;
                log!(
                    "DMA_DDR_ERR small bytes={} rc=0x{:x}\n",
                    bytes,
                    code(Err(e))
                )
                .unwrap();
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
    rp1_hal::shm::complete_posted_writes();
    out[2] = now().wrapping_sub(t);
    // Large blocks: 64 KiB of the SRAM image, 8 times per path (timing only; overwritten below).
    for (slot, base) in [(6usize, atu), (7, direct)] {
        let t = now();
        for k in 0..8u64 {
            let r = unsafe {
                dma.ch0.start(
                    BusAddr(BusAddr::SRAM),
                    BusAddr(base.0 + k * 0x1_0000),
                    0x1_0000,
                )
            }
            .and_then(|x| x.wait(TIMEOUT_US));
            if let Err(e) = r {
                errors += 1;
                log!("DMA_DDR_ERR large slot={} rc=0x{:x}\n", slot, code(Err(e))).unwrap();
                break;
            }
        }
        out[slot] = now().wrapping_sub(t);
    }
    // Cancellation proof in the first 4 KiB of the block (overwritten by the layout below).
    if !abort(dma, data, atu) {
        errors += 1;
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
            errors += 1;
            log!("DMA_DDR_ERR half={} rc=0x{:x}\n", half, code(Err(e))).unwrap();
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
    errors += u32::from(bad != 0);
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
    rp1_hal::shm::complete_posted_writes();
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
#[cfg(feature = "dma")]
fn stopped_code(s: Stopped) -> u32 {
    match s {
        Stopped::Finished(_) => 0,
        Stopped::Disabled { suspended: false } => 1,
        Stopped::Disabled { suspended: true } => 2,
        Stopped::Aborted => 3,
        Stopped::Late => 4,
    }
}

/// Cancellation proof on channel 2 (4-beat FIFO): byte-wide SRAM -> DDR (ATU window), so the
/// block is slow enough to be cancelled mid-way. Logs `DMA_ABORT_PASS` or `DMA_ABORT_FAIL`.
#[cfg(feature = "dma")]
#[inline(never)]
fn abort(dma: &mut Dma, data: *mut u32, atu: BusAddr) -> bool {
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
        let r = code(unsafe { dma.ch2.start(src, dst, len) }.and_then(|x| x.wait(TIMEOUT_US)));
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
    let mut midway = 0;
    for quarter in 1..4u32 {
        canary(data);
        let delay = full_us * quarter / 4;
        let t = now();
        let how = match unsafe { dma.ch2.start(src, dst, len) } {
            Ok(x) => {
                while now().wrapping_sub(t) < delay {}
                x.cancel()
            }
            Err(e) => Stopped::Finished(Err(e)),
        };
        let elapsed = now().wrapping_sub(t);
        let en = chen() & (1 << 2);
        let (copied, tail_bad, guard_bad, hash) = inspect(data);
        // Nothing may change after cancel() returned.
        let t2 = now();
        while now().wrapping_sub(t2) < 2 * full_us + 100 {}
        let (_, _, _, hash2) = inspect(data);
        let late_change = u32::from(hash != hash2);
        let good = !matches!(how, Stopped::Finished(_))
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
            stopped_code(how),
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
        ok &= good && reuse_ok;
    }
    ok &= midway > 0;
    ok &= raw_abort(dma, data, src, dst, len, full_us);
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
    if let Ok(x) = unsafe { dma.ch2.start(src, dst, len) } {
        let t = now();
        while now().wrapping_sub(t) < full_us / 2 {}
        unsafe { (CHEN_H as *mut u32).write_volatile((1 << 2) | (1 << 10)) };
        let t = now();
        while chen() & (1 << 2) != 0 && now().wrapping_sub(t) < 10_000 {}
        clear_us = now().wrapping_sub(t);
        en = chen() & (1 << 2);
        status = unsafe { (CH2_INTSTATUS as *const u32).read_volatile() };
        drop(x);
    }
    let (copied, tail_bad, guard_bad, _) = inspect(data);
    let (r, _, reuse_ok, _) = {
        canary(data);
        let r = code(unsafe { dma.ch2.start(src, dst, len) }.and_then(|x| x.wait(TIMEOUT_US)));
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
