//! `stack-paths <firmware.elf> [--root <symbol>]...`: static stack bounds of the proc0 platform
//! handlers and `S_plat` (contract §5.5). The roots are proc0's IRQ57, IRQ53, HardFault,
//! MemManage, BusFault, UsageFault and NMI vectors, plus any `--root`. Exits non-zero unless every root has a bound, `S_plat`
//! fits in the image's platform margin (`__stack_margin0`, which rp1-build adds to proc0's
//! stack) and proc0's stack (`__stack_size0`) is larger than that margin.
//! `--size` first prints the image's size report ([`stack_paths::size`]).
//! `--cut <symbol>` (diagnostic) counts calls to that function as 0 bytes and noreturn; the
//! result is then `stack_static=DIAG`, never PASS, and the exit status is non-zero.
//! `RP1_OBJDUMP` overrides the disassembler (default `arm-none-eabi-objdump`).

use object::{Object, ObjectSection};
use stack_paths::{Handlers, Proc0Stack, Program, verdict};
use std::process::{Command, ExitCode};

const USAGE: &str =
    "usage: stack-paths <firmware.elf> [--root <symbol>]... [--cut <symbol>]... [--size]";

/// The proc0 vector slots that are roots, with their names on the `stack root=` lines.
const VECTORS: [(usize, &str); 7] = [
    (16 + 57, "irq57"),
    (16 + 53, "irq53"),
    (3, "hardfault"),
    (4, "memmanage"),
    (5, "busfault"),
    (6, "usagefault"),
    (2, "nmi"),
];
/// rp1-rt's handler of every exception the platform does not handle.
const DEFAULT_HANDLER: &str = "__rp1_default";

fn main() -> ExitCode {
    match run() {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(e) => {
            eprintln!("stack-paths: {e}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<bool, Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let mut elf = None;
    let mut extra = Vec::new();
    let mut cut_names = Vec::new();
    let mut size = false;
    while let Some(a) = args.next() {
        match a.as_str() {
            "--root" => extra.push(args.next().ok_or(USAGE)?),
            "--cut" => cut_names.push(args.next().ok_or(USAGE)?),
            "--size" => size = true,
            _ if elf.is_none() && !a.starts_with('-') => elf = Some(a),
            _ => return Err(USAGE.into()),
        }
    }
    let elf = elf.ok_or(USAGE)?;
    let bytes = std::fs::read(&elf)?;
    let file = object::File::parse(&*bytes)?;
    let symbol = |name| -> Result<u32, String> {
        let v = stack_paths::size::symbol(&file, name)?;
        u32::try_from(v).map_err(|_| format!("{name} out of range"))
    };
    let stack = Proc0Stack {
        stack0: symbol("__stack_size0")?,
        margin: symbol("__stack_margin0")?,
    };
    if size {
        let report = stack_paths::size::Size::of(&file)?;
        for line in report.lines() {
            println!("{line}");
        }
    }
    let vectors = file
        .section_by_name(".vectors")
        .ok_or("missing .vectors")?
        .data()?;
    let vector = |slot: usize| -> Result<u32, String> {
        let b = vectors
            .get(slot * 4..slot * 4 + 4)
            .ok_or("short vector table")?;
        Ok(u32::from_le_bytes(b.try_into().unwrap()))
    };
    let objdump = std::env::var_os("RP1_OBJDUMP").unwrap_or_else(|| "arm-none-eabi-objdump".into());
    let out = Command::new(objdump)
        .args(["-d", "--no-show-raw-insn"])
        .arg(&elf)
        .output()?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).into_owned().into());
    }
    let program = Program::parse(&String::from_utf8(out.stdout)?);

    let mut cut = std::collections::HashSet::new();
    for name in &cut_names {
        cut.insert(
            program
                .address_of(name)
                .ok_or_else(|| format!("no function {name}"))?,
        );
    }
    let mut roots = Vec::new();
    for (slot, label) in VECTORS {
        roots.push((vector(slot)?, label));
    }
    for name in &extra {
        let address = program
            .address_of(name)
            .ok_or_else(|| format!("no function {name}"))?;
        roots.push((address, "extra"));
    }
    let mut bytes = Vec::new();
    for &(address, label) in &roots {
        let name = program.name_at(address).unwrap_or("?");
        match program.bound_cut(address, &cut) {
            Ok(b) => {
                println!("stack root={name} vector={label} bytes={}", b.bytes);
                bytes.push(b.bytes);
            }
            Err(f) => {
                println!(
                    "stack_static=FAIL reason={} at={} root={name}",
                    f.reason, f.at
                );
                return Ok(false);
            }
        }
    }
    // NMI preempts every other handler, so a real NMI handler adds its bound and one frame. The
    // default handler there is not counted: the platform enables no NMI source (HYP).
    let default = stack_paths::size::symbol(&file, DEFAULT_HANDLER).ok();
    let nmi = (default.map(|a| a & !1) != Some(u64::from(vector(2)? & !1))).then_some(bytes[6]);
    let handlers = Handlers {
        h57: bytes[0],
        h53: bytes[1],
        hhard: bytes[2],
        // MemManage, BusFault and UsageFault, whichever handler they hold: with the default
        // handler (the configurable faults disabled) this is at most H57, so S_plat stays.
        hfault: bytes[3].max(bytes[4]).max(bytes[5]),
        hnmi: nmi,
    };
    let (pass, line) = verdict(&handlers, stack, !cut.is_empty());
    println!("{line}");
    Ok(pass)
}
