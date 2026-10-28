//! Bare-metal side of rp1boot: entry, UART10 log, caches, RP1 over I2C, Linux handoff.
mod cpu;
mod heap;
mod rp1;
mod uart;

use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};
use rp1_dt::fdt::Fdt;
use rp1_proto::{bundle, note};
use rp1boot::{Elf, Range, hex, kernel_header, overlap, verify_bundle};
use sha2::{Digest, Sha256};

/// Kernel placement (2 MiB aligned; the PoC loader's HW-proven address) and the final DTB.
const KERNEL_BASE: u64 = 0x0600_0000;
const DTB_OUT: u64 = 0x2020_0000;
const DTB_OUT_MAX: u64 = 0x0020_0000;
/// RP1 image window in shared SRAM (rp1-proto: image 48 KiB from 0x2000_0000).
const RP1_IMAGE_MAX: usize = (rp1_proto::IMAGE_END - rp1_proto::SRAM_START) as usize;

core::arch::global_asm!(
    r#"
    .section .text.boot, "ax"
    .global _start
_start:
    mov x19, x0
    msr spsel, #1
    ldr x1, =__stack_top
    mov sp, x1
    ldr x1, =__bss_start
    ldr x2, =__bss_end
1:  cmp x1, x2
    b.hs 2f
    str xzr, [x1], #8
    b 1b
2:  adr x1, rp1boot_vectors
    msr vbar_el2, x1
    isb
    mov x0, x19
    bl rp1boot_main
3:  wfe
    b 3b

    .balign 2048
rp1boot_vectors:
    .irp n, 0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15
    .balign 128
    mov x0, #\n
    b rp1boot_exception
    .endr
"#
);

/// Any exception taken to EL2 while the loader runs is fatal: report it and stop.
#[unsafe(no_mangle)]
extern "C" fn rp1boot_exception(vector: u64) -> ! {
    let (esr, elr, far): (u64, u64, u64);
    // SAFETY: reading exception syndrome registers.
    unsafe {
        core::arch::asm!("mrs {}, esr_el2", "mrs {}, elr_el2", "mrs {}, far_el2",
            out(reg) esr, out(reg) elr, out(reg) far, options(nomem, nostack));
    }
    // No allocation here: the heap may be what faulted.
    log!("FAIL: exception vector {vector} ESR_EL2={esr:#x} ELR_EL2={elr:#x} FAR_EL2={far:#x}");
    loop {
        cpu::wfe();
    }
}

macro_rules! log {
    ($($t:tt)*) => { $crate::hw::uart::print(format_args!("[rp1boot {:>7}] {}\n", $crate::hw::cpu::ms(), format_args!($($t)*))) };
}
pub(crate) use log;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo<'_>) -> ! {
    fail(&format!("panic: {info}"))
}

/// v0.1 failure policy (loader.md §3): print the reason on UART10 and stop here.
fn fail(reason: &str) -> ! {
    log!("FAIL: {reason}");
    log!("halted: fix the boot set (cargo rp1 bundle) and power-cycle; no fallback in v0.1");
    loop {
        cpu::wfe();
    }
}

#[unsafe(no_mangle)]
extern "C" fn rp1boot_main(dtb: usize) -> ! {
    cpu::start_clock();
    log!(
        "v{} start EL{} dtb={dtb:#x} HCR_EL2={:#x} SCTLR_EL2={:#x}",
        env!("CARGO_PKG_VERSION"),
        cpu::current_el(),
        cpu::hcr_el2(),
        cpu::sctlr_el2()
    );
    match cpu::enable_caches() {
        Ok(()) => log!("MMU + caches on (identity map, 0..1 GiB normal, 64..128 GiB device)"),
        Err(e) => log!("caches stay off: {e}"),
    }
    match run(dtb) {
        Ok(never) => match never {},
        Err(e) => fail(&e),
    }
}

/// Reads a firmware-owned range (DTB, bundle) as a byte slice.
fn mem(start: u64, len: u64) -> &'static [u8] {
    // SAFETY: the VPU firmware placed these ranges in RAM before entering the loader; they are
    // only read, and nothing else runs on the system.
    unsafe { core::slice::from_raw_parts(start as *const u8, len as usize) }
}

enum Never {}

fn run(dtb_addr: usize) -> Result<Never, String> {
    // 1. The firmware DTB.
    let dtb_len = rp1_dt::fdt::total_size(mem(dtb_addr as u64, 8))? as u64;
    let dtb_in = mem(dtb_addr as u64, dtb_len);
    let mut fdt = Fdt::parse(dtb_in)?;
    log!("firmware DTB parsed");
    let chosen = fdt.find("/chosen").ok_or("DTB has no /chosen")?;
    if let Some(b) = fdt.find("/chosen/bootloader") {
        log!(
            "firmware boot-mode={:?} dtb {dtb_len} B, {} nodes",
            fdt.cells(b, "boot-mode"),
            fdt.nodes.len()
        );
    }
    let bootargs = fdt
        .strings(chosen, "bootargs")
        .into_iter()
        .next()
        .unwrap_or_default();
    log!(
        "bootargs {} B: ...{}",
        bootargs.len(),
        bootargs.get(bootargs.len().saturating_sub(48)..).unwrap_or("")
    );
    let (start, end) = (
        fdt.number(chosen, "linux,initrd-start"),
        fdt.number(chosen, "linux,initrd-end"),
    );
    let (Some(start), Some(end)) = (start, end) else {
        return Err(
            "no /chosen/linux,initrd-start/end: config.txt must load rp1-bundle.img \
                    with `initramfs rp1-bundle.img followkernel`"
                .into(),
        );
    };
    if end <= start {
        return Err(format!("empty initrd range {start:#x}..{end:#x}"));
    }
    log!("bundle at {start:#x}..{end:#x} ({} B)", end - start);

    // 2. Bundle: id against the cmdline token, then every element hash.
    let t = cpu::ms();
    let b = verify_bundle(mem(start, end - start), &bootargs, |d| {
        Sha256::digest(d).into()
    })?;
    log!(
        "bundle ok: rp1.bundle={} ({} elements, sha256 in {} ms)",
        core::str::from_utf8(&bundle::id_hex(&Sha256::digest(b.header()).into())).unwrap_or("?"),
        b.entries().count(),
        cpu::ms() - t
    );
    let fw = b.data(&b.get(bundle::FIRMWARE).ok_or("no rp1.elf")?);
    let kernel = b.data(&b.get(bundle::KERNEL).ok_or("no kernel")?);
    let initramfs = b.get(bundle::INITRAMFS);

    // 3. The firmware's note and overlay.
    let elf = Elf::parse(fw)?;
    let desc = note::descriptor(elf.section(".note.rp1")?)
        .map_err(|_| "bad .note.rp1 envelope".to_string())?;
    let parsed = note::Note::parse(desc).map_err(|e| {
        format!("invalid .note.rp1 ({e}): CRC, version or an unknown critical record")
    })?;
    let entry = parsed.get(note::ENTRY).ok_or("note has no ENTRY")?.value;
    let (entry_pc, entry_sp) = (
        note::u32_at(entry, 0).map_err(|_| "ENTRY truncated")?,
        note::u32_at(entry, 4).map_err(|_| "ENTRY truncated")?,
    );
    if entry_pc != elf.entry() {
        return Err(format!(
            "note ENTRY {entry_pc:#x} != ELF entry {:#x}",
            elf.entry()
        ));
    }
    let area = parsed
        .get(note::SHM_AREA)
        .map(|r| note::ShmArea::parse(r.value))
        .transpose()
        .map_err(|_| "bad SHM_AREA")?;
    log!(
        "rp1.elf {} B: note v2 ok, entry={entry_pc:#x} sp={entry_sp:#x} shm={}",
        fw.len(),
        area.map_or(String::from("none"), |a| format!(
            "{:#x}+{:#x}",
            a.phys, a.size
        ))
    );

    // 4. Overlay last (after config.txt's), then the same checks as `cargo rp1 check --dtb`.
    let t = cpu::ms();
    let before = Fdt::parse(dtb_in)?;
    rp1_dt::overlay::apply(&mut fdt, elf.section(".rp1.dtbo")?)
        .map_err(|e| format!("applying .rp1.dtbo: {e}"))?;
    let conflicts = rp1_dt::check::check(&before, &fdt, desc)?;
    drop(before);
    if !conflicts.is_empty() {
        for c in &conflicts {
            log!("conflict: {c}");
        }
        return Err(format!(
            "{} conflict(s) between the final DTB and the RP1 note; RP1 not started",
            conflicts.len()
        ));
    }
    log!(
        "overlay applied, final-DTB checks passed ({} ms)",
        cpu::ms() - t
    );

    // 5. Linux initrd = the bundle's initramfs, in place.
    let chosen = fdt.find("/chosen").ok_or("DTB lost /chosen")?;
    match initramfs {
        Some(e) => {
            let s = start + u64::from(e.offset);
            fdt.set_prop(chosen, "linux,initrd-start", s.to_be_bytes().to_vec());
            fdt.set_prop(
                chosen,
                "linux,initrd-end",
                (s + u64::from(e.len)).to_be_bytes().to_vec(),
            );
            log!("initrd = bundle initramfs {s:#x}+{:#x}", e.len);
        }
        None => {
            fdt.remove_prop(chosen, "linux,initrd-start");
            fdt.remove_prop(chosen, "linux,initrd-end");
        }
    }
    let (text_offset, image_size) = kernel_header(kernel)?;
    let load = KERNEL_BASE + text_offset;
    let out = fdt.to_bytes();
    if out.len() as u64 > DTB_OUT_MAX {
        return Err(format!("final DTB is {} B (> {DTB_OUT_MAX:#x})", out.len()));
    }
    let mut ranges = Vec::from([
        Range {
            name: "rp1boot",
            start: cpu::image_start(),
            end: cpu::heap_end(),
        },
        Range {
            name: "kernel",
            start: KERNEL_BASE,
            end: load + image_size,
        },
        Range {
            name: "bundle",
            start,
            end,
        },
        Range {
            name: "firmware DTB",
            start: dtb_addr as u64,
            end: dtb_addr as u64 + dtb_len,
        },
        Range {
            name: "final DTB",
            start: DTB_OUT,
            end: DTB_OUT + out.len() as u64,
        },
    ]);
    if let Some(a) = area {
        ranges.push(Range {
            name: "rp1 shm",
            start: a.phys,
            end: a.phys + u64::from(a.size),
        });
    }
    if let Some((a, b)) = overlap(&ranges) {
        return Err(format!(
            "placement: {} {:#x}..{:#x} overlaps {} {:#x}..{:#x}",
            a.name, a.start, a.end, b.name, b.start, b.end
        ));
    }
    // What Linux must not get: none of our writes may land in a reservation.
    let mut reserved: Vec<(u64, u64)> = fdt.memreserve.clone();
    if let Some(rm) = fdt.find("/reserved-memory") {
        for &c in &fdt.nodes[rm].children {
            if fdt.enabled(c) && fdt.prop(c, "reg").is_some() {
                reserved.extend(fdt.reg(c)?);
            }
        }
    }
    for r in &ranges[1..] {
        if matches!(r.name, "kernel" | "final DTB")
            && let Some((a, s)) = reserved
                .iter()
                .find(|(a, s)| *a < r.end && a.saturating_add(*s) > r.start)
        {
            return Err(format!(
                "{} {:#x}..{:#x} overlaps reserved memory {a:#x}+{s:#x}",
                r.name, r.start, r.end
            ));
        }
    }

    // 6. RP1: reset into the bootrom, load the image, handoff word, start (PoC sequence).
    let image = elf.image(rp1_proto::SRAM_START, RP1_IMAGE_MAX)?;
    let t = cpu::ms();
    rp1::boot(&image, entry_pc, entry_sp, area.as_ref())?;
    log!(
        "RP1 started: {} B over I2C in {} ms",
        image.len(),
        cpu::ms() - t
    );

    // 7. Kernel and DTB into place, quiesce, enter Linux at EL2.
    // SAFETY: both destinations were checked above against the loader, bundle, firmware DTB,
    // shared memory and every reservation in the final DTB.
    unsafe {
        core::ptr::copy_nonoverlapping(kernel.as_ptr(), load as *mut u8, kernel.len());
        core::ptr::write_bytes(
            (load + kernel.len() as u64) as *mut u8,
            0,
            (image_size - kernel.len() as u64) as usize,
        );
        core::ptr::copy_nonoverlapping(out.as_ptr(), DTB_OUT as *mut u8, out.len());
    }
    cpu::quiesce_sdhc();
    log!(
        "kernel {:#x}+{image_size:#x} (sha256 {}..), dtb {DTB_OUT:#x}+{:#x}; jumping to Linux at EL2",
        load,
        &hex(&b.get(bundle::KERNEL).unwrap().sha256)[..16],
        out.len()
    );
    cpu::clean_dcache(load, image_size);
    cpu::clean_dcache(DTB_OUT, out.len() as u64);
    cpu::clean_dcache(cpu::image_start(), cpu::heap_end() - cpu::image_start());
    // SAFETY: Image and DTB are in place and cleaned to PoC; this does not return.
    unsafe { cpu::enter_linux(load, DTB_OUT) }
}
