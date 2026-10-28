//! EL2 CPU state: timer, identity-mapped caches, cache maintenance and the Linux handoff.
use core::arch::asm;
use core::sync::atomic::{AtomicU64, Ordering};

unsafe extern "C" {
    static __image_start: u8;
    static __heap_end: u8;
}

pub fn image_start() -> u64 {
    (&raw const __image_start) as u64
}
pub fn heap_end() -> u64 {
    (&raw const __heap_end) as u64
}

macro_rules! sysreg {
    ($name:ident, $reg:literal) => {
        pub fn $name() -> u64 {
            let v: u64;
            // SAFETY: reading a system register has no side effects.
            unsafe { asm!(concat!("mrs {}, ", $reg), out(reg) v, options(nomem, nostack)) };
            v
        }
    };
}
sysreg!(hcr_el2, "hcr_el2");
sysreg!(sctlr_el2, "sctlr_el2");
sysreg!(cntpct, "cntpct_el0");
sysreg!(cntfrq, "cntfrq_el0");
sysreg!(clidr, "clidr_el1");

pub fn current_el() -> u64 {
    let v: u64;
    // SAFETY: read-only.
    unsafe { asm!("mrs {}, CurrentEL", out(reg) v, options(nomem, nostack)) };
    (v >> 2) & 3
}

// Plain load/store only (no exclusives: they are unusable before the MMU is on).
static T0: AtomicU64 = AtomicU64::new(0);

pub fn start_clock() {
    T0.store(cntpct(), Ordering::Relaxed);
}
/// Milliseconds since the loader started.
pub fn ms() -> u64 {
    let f = cntfrq().max(1);
    (cntpct() - T0.load(Ordering::Relaxed)) * 1000 / f
}
pub fn delay_us(us: u64) {
    let end = cntpct() + us * cntfrq() / 1_000_000;
    while cntpct() < end {}
}

pub fn wfe() {
    // SAFETY: wait for event; no memory effects.
    unsafe { asm!("wfe", options(nomem, nostack)) };
}

#[repr(C, align(4096))]
struct Table([u64; 512]);
static mut L1: Table = Table([0; 512]);

const SCTLR_M: u64 = 1;
const SCTLR_A: u64 = 1 << 1;
const SCTLR_C: u64 = 1 << 2;
const SCTLR_I: u64 = 1 << 12;
const HCR_E2H: u64 = 1 << 34;

/// Invalidate every data/unified cache by set/way (nothing is dirty yet: the ARM ran with
/// caches off before the loader).
fn invalidate_dcache_all() {
    let clidr = clidr();
    let loc = (clidr >> 24) & 7;
    for level in 0..loc {
        if (clidr >> (level * 3)) & 7 < 2 {
            continue; // no data cache at this level
        }
        let ccsidr: u64;
        // SAFETY: selects the cache level for CCSIDR; read-only otherwise.
        unsafe {
            asm!("msr csselr_el1, {}", "isb", "mrs {}, ccsidr_el1",
                 in(reg) level << 1, out(reg) ccsidr, options(nostack));
        }
        let line_shift = (ccsidr & 7) + 4;
        let ways = ((ccsidr >> 3) & 0x3ff) + 1;
        let sets = ((ccsidr >> 13) & 0x7fff) + 1;
        let way_shift = (ways as u32 - 1).leading_zeros() % 32;
        for way in 0..ways {
            for set in 0..sets {
                let v = (way << way_shift) | (set << line_shift) | (level << 1);
                // SAFETY: set/way maintenance on this core's caches during single-core boot.
                unsafe { asm!("dc isw, {}", in(reg) v, options(nostack)) };
            }
        }
    }
    // SAFETY: barrier.
    unsafe { asm!("dsb sy", "isb", options(nostack)) };
}

/// Identity map with 1 GiB blocks: 0..1 GiB Normal write-back (all loader, DTB, bundle and
/// kernel addresses), 64..128 GiB Device-nGnRE (BCM2712 peripherals at 0x10_xxxx_xxxx).
pub fn enable_caches() -> Result<(), &'static str> {
    if current_el() != 2 {
        return Err("not at EL2");
    }
    if hcr_el2() & HCR_E2H != 0 {
        return Err("HCR_EL2.E2H set (VHE translation regime not supported)");
    }
    const AF: u64 = 1 << 10;
    const BLOCK: u64 = 1;
    let normal = BLOCK | AF | (3 << 8) | (1 << 2); // AttrIndx 1, inner shareable
    let device = BLOCK | AF | (1 << 54); // AttrIndx 0, XN
    let table = (&raw mut L1).cast::<u64>();
    // SAFETY: single-core boot; the table is only written here, before the MMU uses it.
    unsafe {
        table.write(normal);
        for i in 64..128u64 {
            table.add(i as usize).write((i << 30) | device);
        }
    }
    let mair: u64 = 0x04 | (0xff << 8);
    // T0SZ=25 (39-bit VA, start at level 1), WBWA inner/outer, inner shareable, 4K granule,
    // PS=40-bit, RES1 bits 23 and 31.
    let tcr: u64 = 25 | (1 << 8) | (1 << 10) | (3 << 12) | (2 << 16) | (1 << 23) | (1 << 31);
    invalidate_dcache_all();
    // SAFETY: valid identity tables for every address the loader touches; caches were
    // invalidated so no stale line can shadow what the firmware wrote to RAM.
    unsafe {
        asm!(
            "dsb sy",
            "msr mair_el2, {mair}",
            "msr tcr_el2, {tcr}",
            "msr ttbr0_el2, {ttbr}",
            "isb",
            "tlbi alle2",
            "dsb sy",
            "ic iallu",
            "dsb sy",
            "isb",
            "mrs {t}, sctlr_el2",
            "bic {t}, {t}, {a}",
            "orr {t}, {t}, {on}",
            "msr sctlr_el2, {t}",
            "isb",
            mair = in(reg) mair,
            tcr = in(reg) tcr,
            ttbr = in(reg) (&raw const L1) as u64,
            a = in(reg) SCTLR_A,
            on = in(reg) SCTLR_M | SCTLR_C | SCTLR_I,
            t = out(reg) _,
            options(nostack),
        );
    }
    Ok(())
}

/// Clean and invalidate `[start, start + len)` to the point of coherency.
pub fn clean_dcache(start: u64, len: u64) {
    let mut p = start & !63;
    while p < start + len {
        // SAFETY: cache maintenance by VA on identity-mapped RAM.
        unsafe { asm!("dc civac, {}", in(reg) p, options(nostack)) };
        p += 64;
    }
    // SAFETY: barrier.
    unsafe { asm!("dsb sy", options(nostack)) };
}

/// Reset the SDHCI command/data state the firmware may leave behind (PoC `quiesce_sdhc`).
pub fn quiesce_sdhc() {
    const HOST: usize = 0x10_00ff_f000;
    let w32 = |o: usize, v: u32| unsafe { core::ptr::write_volatile((HOST + o) as *mut u32, v) };
    let r32 = |o: usize| unsafe { core::ptr::read_volatile((HOST + o) as *const u32) };
    // SAFETY (closures above): fixed BCM2712 SDHCI MMIO, device-mapped.
    w32(0x34, 0);
    w32(0x38, 0);
    for _ in 0..100_000 {
        if r32(0x24) & 3 == 0 {
            break;
        }
    }
    w32(0x30, u32::MAX);
    // SAFETY: byte register of the same block.
    unsafe { core::ptr::write_volatile((HOST + 0x2f) as *mut u8, 0x06) };
    for _ in 0..100_000 {
        // SAFETY: as above.
        if unsafe { core::ptr::read_volatile((HOST + 0x2f) as *const u8) } & 0x06 == 0 {
            break;
        }
    }
}

/// arm64 boot protocol at EL2: MMU and D-cache off, x0 = DTB, x1..x3 = 0.
///
/// # Safety
/// The kernel Image and DTB must be in place and cleaned to the point of coherency.
pub unsafe fn enter_linux(entry: u64, dtb: u64) -> ! {
    // SAFETY: terminal handoff; the sequence uses registers only.
    unsafe {
        asm!(
            "msr daifset, #0xf",
            "mrs x2, sctlr_el2",
            "bic x2, x2, #1",
            "bic x2, x2, #(1 << 2)",
            "bic x2, x2, #(1 << 12)",
            "msr sctlr_el2, x2",
            "isb",
            "ic iallu",
            "tlbi alle2",
            "dsb sy",
            "isb",
            "msr vttbr_el2, xzr",
            "msr cntvoff_el2, xzr",
            "msr cptr_el2, xzr",
            "mov x2, #(1 << 31)",
            "msr hcr_el2, x2",
            "isb",
            "mov x0, x17",
            "mov x1, xzr",
            "mov x2, xzr",
            "mov x3, xzr",
            "br x16",
            in("x16") entry,
            in("x17") dtb,
            options(noreturn)
        )
    }
}
