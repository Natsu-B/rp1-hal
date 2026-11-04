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
use crate::rp1;
use core::cell::UnsafeCell;
use rp1_hal::{
    dma::{BusAddr, Channel, Dma, Error},
    log,
};

pub const WORDS: usize = 256;
const TIMEOUT_US: u32 = 100_000;
#[repr(C, align(16))]
struct Buf(UnsafeCell<[u32; WORDS]>);
unsafe impl Sync for Buf {}
/// Shared SRAM (DMA-visible). Only proc1 touches these, and only here.
#[unsafe(link_section = ".shared")]
static SRC: Buf = Buf(UnsafeCell::new([0; WORDS]));
#[unsafe(link_section = ".shared")]
static DST: Buf = Buf(UnsafeCell::new([0; WORDS]));

fn now() -> u32 {
    unsafe { (0x400a_c028 as *const u32).read_volatile() }
}
pub const fn pattern(i: usize, seed: u32) -> u32 {
    (i as u32).wrapping_mul(0x9e37_79b9) ^ seed
}
fn src() -> *mut u32 {
    SRC.0.get().cast()
}
fn dst() -> *mut u32 {
    DST.0.get().cast()
}
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
    let (s, d) = unsafe {
        (
            core::slice::from_raw_parts(s8, 1024),
            core::slice::from_raw_parts_mut(d8, 1024),
        )
    };
    let mut us = [0u32; 3];
    for (slot, u) in us.iter_mut().enumerate() {
        let t = now();
        for _ in 0..1000 {
            match slot {
                0 => ok &= dma.ch0.copy(s, d, TIMEOUT_US).is_ok(),
                1 => ok &= dma.ch2.copy(s, d, TIMEOUT_US).is_ok(),
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

/// Command 16. `data` = M3 pointer of the bulk block data (1 MiB, outbound window), `phys` =
/// its host physical address. Returns (errors, results for request[8..16]).
#[inline(never)]
pub fn ddr(dma: &mut Dma, data: *mut u32, phys: u64, seed: u32) -> (u32, [u32; 8]) {
    let mut out = [0u32; 8];
    let mut errors = 0;
    let atu = match BusAddr::from_proc(data as usize) {
        Ok(a) => a,
        Err(_) => return (1, out),
    };
    let direct = BusAddr::host_phys(phys);
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
