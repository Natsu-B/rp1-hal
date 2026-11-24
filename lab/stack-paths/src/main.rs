//! `stack-paths <firmware.elf> [--root <symbol>]...`: static stack bounds of the proc0 platform
//! handlers and `S_plat` (contract §5.5). Exits non-zero unless every root has a bound.
//! `--cut <symbol>` (diagnostic) counts calls to that function as 0 bytes and noreturn; the
//! result is then `stack_static=DIAG`, never PASS, and the exit status is non-zero.
//! `RP1_OBJDUMP` overrides the disassembler (default `arm-none-eabi-objdump`).

use object::{Object, ObjectSection};
use stack_paths::{Program, s_plat};
use std::process::{Command, ExitCode};

const USAGE: &str = "usage: stack-paths <firmware.elf> [--root <symbol>]... [--cut <symbol>]...";

/// proc0 vector slots: HardFault, IRQ53 (PCIE_OUT), IRQ57 (PROC_MISC).
const HARDFAULT: usize = 3;
const IRQ53: usize = 16 + 53;
const IRQ57: usize = 16 + 57;

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
    while let Some(a) = args.next() {
        match a.as_str() {
            "--root" => extra.push(args.next().ok_or(USAGE)?),
            "--cut" => cut_names.push(args.next().ok_or(USAGE)?),
            _ if elf.is_none() && !a.starts_with('-') => elf = Some(a),
            _ => return Err(USAGE.into()),
        }
    }
    let elf = elf.ok_or(USAGE)?;
    let bytes = std::fs::read(&elf)?;
    let file = object::File::parse(&*bytes)?;
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
    for slot in [IRQ57, IRQ53, HARDFAULT] {
        roots.push(vector(slot)?);
    }
    for name in &extra {
        roots.push(
            program
                .address_of(name)
                .ok_or_else(|| format!("no function {name}"))?,
        );
    }
    let mut bytes = Vec::new();
    for &address in &roots {
        let name = program.name_at(address).unwrap_or("?");
        match program.bound_cut(address, &cut) {
            Ok(b) => {
                println!("stack root={name} bytes={}", b.bytes);
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
    let (h57, h53, hhard) = (bytes[0], bytes[1], bytes[2]);
    // Configurable fault handlers are not enabled yet, so Hfault = 0.
    let hfault = 0;
    let verdict = if cut.is_empty() { "PASS" } else { "DIAG" };
    println!(
        "stack_static={verdict} s_plat={} h57={h57} h53={h53} hhard={hhard} hfault={hfault}",
        s_plat(h57, hfault, h53, hhard)
    );
    Ok(cut.is_empty())
}
