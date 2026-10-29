//! Lab-only M3 micro-profile (command 9): SysTick core cycles, no DWT on this M3.
//!
//! Every primitive runs from ISRAM so that instruction fetch from shared SRAM does not hide
//! the memory cost being measured. Results go to UART0 as `PROF <name> n=<n> cyc=<total>`.
use super::rp1;
use core::cell::UnsafeCell;
use rp1_hal::log;
use rp1_proto::shm::{Overflow, RingRx, RingTx};

const SYST_CSR: *mut u32 = 0xe000_e010usize as _;
const SYST_RVR: *mut u32 = 0xe000_e014usize as _;
const SYST_CVR: *mut u32 = 0xe000_e018usize as _;
/// Shared SRAM scratch for the scalar SRAM timings (one 4-word STM/LDM at most), allocated by
/// the linker inside the image (`.shared`) so it can never overlap code, constants or another
/// shared object. Only proc1 touches it, and only during command 9. A raw address above
/// `__image_end` is not reserved by anything (review 3 #1: it overwrote the Logger), so there is
/// no SRAM ring variant: a ring needs 512 bytes the 48 KiB image cannot spare.
#[repr(C, align(8))]
struct SramScratch(UnsafeCell<[u32; 4]>);
unsafe impl Sync for SramScratch {}
#[unsafe(link_section = ".shared")]
static SRAM_SCRATCH: SramScratch = SramScratch(UnsafeCell::new([0; 4]));

pub fn init() {
    unsafe {
        SYST_RVR.write_volatile(0x00ff_ffff);
        SYST_CVR.write_volatile(0);
        SYST_CSR.write_volatile(5); // enable, processor clock, no interrupt
    }
}
#[inline(always)]
fn cyc() -> u32 {
    unsafe { SYST_CVR.read_volatile() }
}
#[inline(always)]
fn since(start: u32) -> u32 {
    start.wrapping_sub(cyc()) & 0x00ff_ffff
}
#[inline(always)]
fn dsb() {
    unsafe { core::arch::asm!("dsb sy", options(nostack, preserves_flags)) };
}
#[inline(always)]
fn spin_us(us: u32) {
    let t = super::now();
    while super::now().wrapping_sub(t) < us {}
}

#[rp1_rt::isram]
fn empty(n: u32) -> u32 {
    let t = cyc();
    for i in 0..n {
        core::hint::black_box(i);
    }
    since(t)
}
#[rp1_rt::isram]
unsafe fn load(p: *const u32, n: u32) -> u32 {
    let t = cyc();
    for _ in 0..n {
        core::hint::black_box(unsafe { p.read_volatile() });
    }
    since(t)
}
#[rp1_rt::isram]
unsafe fn store(p: *mut u32, n: u32) -> u32 {
    let t = cyc();
    for i in 0..n {
        unsafe { p.write_volatile(i) };
    }
    dsb();
    since(t)
}
#[rp1_rt::isram]
unsafe fn store_dsb(p: *mut u32, n: u32) -> u32 {
    let t = cyc();
    for i in 0..n {
        unsafe { p.write_volatile(i) };
        dsb();
    }
    since(t)
}
#[rp1_rt::isram]
fn dsb_only(n: u32) -> u32 {
    let t = cyc();
    for _ in 0..n {
        dsb();
    }
    since(t)
}
/// One `store; dsb` (or one load) after `gap` µs of PCIe idle; returns the summed op cycles.
#[rp1_rt::isram]
unsafe fn store_dsb_gap(p: *mut u32, n: u32, gap: u32) -> u32 {
    let mut sum = 0;
    for i in 0..n {
        spin_us(gap);
        let t = cyc();
        unsafe { p.write_volatile(i) };
        dsb();
        sum += since(t);
    }
    sum
}
#[rp1_rt::isram]
unsafe fn store_gap(p: *mut u32, n: u32, gap: u32) -> u32 {
    let mut sum = 0;
    for i in 0..n {
        spin_us(gap);
        let t = cyc();
        unsafe { p.write_volatile(i) };
        sum += since(t);
    }
    sum
}
#[rp1_rt::isram]
unsafe fn load_gap(p: *const u32, n: u32, gap: u32) -> u32 {
    let mut sum = 0;
    for _ in 0..n {
        spin_us(gap);
        let t = cyc();
        core::hint::black_box(unsafe { p.read_volatile() });
        sum += since(t);
    }
    sum
}
/// `n` times: 7-word burst (push shape) then dsb.
#[rp1_rt::isram]
unsafe fn burst7_dsb(p: *mut u32, n: u32) -> u32 {
    let t = cyc();
    for i in 0..n {
        for k in 0..7 {
            unsafe { p.add(k).write_volatile(i) };
        }
        dsb();
    }
    since(t)
}

/// `n` times a 4-word STM (one AHB burst) then dsb at the end.
#[rp1_rt::isram]
unsafe fn stm4(p: *mut u32, n: u32) -> u32 {
    let t = cyc();
    for i in 0..n {
        unsafe {
            core::arch::asm!("stm r1, {{r2, r3, r4, r5}}", in("r1") p,
                in("r2") i, in("r3") i, in("r4") i, in("r5") i, options(nostack));
        }
    }
    dsb();
    since(t)
}
#[rp1_rt::isram]
unsafe fn ldm4(p: *const u32, n: u32) -> u32 {
    let t = cyc();
    for _ in 0..n {
        let (a, b, c, d): (u32, u32, u32, u32);
        unsafe {
            core::arch::asm!("ldm r1, {{r2, r3, r4, r5}}", in("r1") p,
                out("r2") a, out("r3") b, out("r4") c, out("r5") d, options(nostack));
        }
        core::hint::black_box((a, b, c, d));
    }
    since(t)
}

type Sample = super::Sample;
const DEPTH: u32 = 16;
const BODY: usize = 128 + DEPTH as usize * 24;
#[repr(C, align(8))]
struct Local(UnsafeCell<[u8; BODY]>, UnsafeCell<[u8; 128]>);
unsafe impl Sync for Local {}
static LOCAL: Local = Local(UnsafeCell::new([0; BODY]), UnsafeCell::new([0; 128]));

/// Ring code cost with chosen placement: `data`/`ctrl` may be local (DSRAM) or remote (DDR).
fn ring_pair(data: *mut u8, ctrl: *mut u8, rounds: u32, batch: bool) -> (u32, u32) {
    unsafe {
        for i in 0..32 {
            data.cast::<u32>().add(i).write_volatile(0);
            ctrl.cast::<u32>().add(i).write_volatile(0);
        }
    }
    dsb();
    let mut tx: RingTx<Sample> =
        unsafe { RingTx::attach(data, BODY, ctrl, 128, DEPTH, Overflow::Reject) }.unwrap();
    let mut rx: RingRx<Sample> =
        unsafe { RingRx::attach(data, BODY, ctrl, 128, DEPTH, Overflow::Reject) }.unwrap();
    let (mut push, mut pop) = (0, 0);
    if batch {
        let mut buf = [[0u32; 4]; DEPTH as usize];
        for r in 0..rounds {
            for (i, b) in buf.iter_mut().enumerate() {
                *b = super::sample(r * DEPTH + i as u32);
            }
            let t = cyc();
            assert_eq!(tx.push_slice(&buf), DEPTH as usize);
            push += since(t);
            let t = cyc();
            assert_eq!(rx.pop_into(&mut buf), Ok(DEPTH as usize));
            pop += since(t);
            core::hint::black_box(&buf);
        }
        return (push, pop);
    }
    for r in 0..rounds {
        let t = cyc();
        for i in 0..DEPTH {
            tx.push(&super::sample(r * DEPTH + i)).unwrap();
        }
        push += since(t);
        let t = cyc();
        for _ in 0..DEPTH {
            core::hint::black_box(rx.pop().unwrap());
        }
        pop += since(t);
    }
    (push, pop)
}

/// `ddr` is a scratch area of the outbound window that nobody else uses during the command.
pub fn run(ddr: *mut u32) {
    init();
    let t0 = cyc();
    let u0 = super::now();
    spin_us(1000);
    let (c, u) = (since(t0), super::now().wrapping_sub(u0));
    log!("PROF clock cyc={} us={}\n", c, u).unwrap();
    let local = LOCAL.1.get().cast::<u32>();
    let sram = SRAM_SCRATCH.0.get().cast::<u32>();
    let n = 256;
    let r = |name: &str, n: u32, cyc: u32| log!("PROF {} n={} cyc={}\n", name, n, cyc).unwrap();
    r("empty", n, empty(n));
    r("dsb", n, dsb_only(n));
    unsafe {
        r("load_dsram", n, load(local, n));
        r("load_sram", n, load(sram, n));
        r("load_ddr", n, load(ddr, n));
        r("store_dsram", n, store(local, n));
        r("store_sram", n, store(sram, n));
        r("store_ddr", n, store(ddr, n));
        r("store_dsb_sram", n, store_dsb(sram, n));
        r("store_dsb_ddr", n, store_dsb(ddr, n));
        r("burst7_dsb_ddr", n, burst7_dsb(ddr, n));
        r("stm4_ddr", n, stm4(ddr, n));
        r("stm4_sram", n, stm4(sram, n));
        r("ldm4_ddr", n, ldm4(ddr, n));
        r("ldm4_sram", n, ldm4(sram, n));
        for gap in [2, 10, 50, 200] {
            log!(
                "PROF gap={} store_dsb_ddr={} store_ddr={} load_ddr={} n=64\n",
                gap,
                store_dsb_gap(ddr, 64, gap),
                store_gap(ddr, 64, gap),
                load_gap(ddr, 64, gap)
            )
            .unwrap();
        }
    }
    let data = LOCAL.0.get().cast::<u8>();
    let ctrl = LOCAL.1.get().cast::<u8>();
    let rounds = 8;
    let m = rounds * DEPTH;
    for batch in [false, true] {
        let b = if batch { "batch" } else { "single" };
        let (push, pop) = ring_pair(data, ctrl, rounds, batch);
        log!(
            "PROF ring_local {} push n={} cyc={} pop cyc={}\n",
            b,
            m,
            push,
            pop
        )
        .unwrap();
        let (push, pop) = ring_pair(data, ddr.cast(), rounds, batch);
        log!(
            "PROF ring_ctrl_ddr {} push n={} cyc={} pop cyc={}\n",
            b,
            m,
            push,
            pop
        )
        .unwrap();
        let (push, pop) = ring_pair(ddr.cast::<u8>().wrapping_add(4096), ctrl, rounds, batch);
        log!(
            "PROF ring_data_ddr {} push n={} cyc={} pop cyc={}\n",
            b,
            m,
            push,
            pop
        )
        .unwrap();
    }
    log!("PROF_DONE\n").unwrap();
}
