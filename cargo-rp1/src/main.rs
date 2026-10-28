use object::{Object, ObjectSection};
use std::path::{Path, PathBuf};
fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.first().is_some_and(|s| s == "rp1") {
        args.remove(0);
    }
    if args.len() < 2 || args[0] != "check" {
        return Err("usage: cargo rp1 check <firmware.elf> [--dtb <base.dtb>]".into());
    }
    let elf = PathBuf::from(&args[1]);
    let base = if args.len() == 4 && args[2] == "--dtb" {
        Some(PathBuf::from(&args[3]))
    } else if args.len() == 2 {
        None
    } else {
        return Err("unexpected check arguments".into());
    };
    let objdump = std::env::var_os("RP1_OBJDUMP").unwrap_or_else(|| "arm-none-eabi-objdump".into());
    let report = cargo_rp1::check(&elf, Path::new(&objdump))?;
    if let Some(base) = base {
        let bytes = std::fs::read(&elf)?;
        let file = object::File::parse(&*bytes)?;
        // Same code as the loader (rp1boot): libfdt-compatible apply, then the final checks.
        let base = std::fs::read(&base)?;
        let before = rp1_dt::fdt::Fdt::parse(&base)?;
        let mut after = rp1_dt::fdt::Fdt::parse(&base)?;
        rp1_dt::overlay::apply(
            &mut after,
            file.section_by_name(".rp1.dtbo")
                .ok_or("missing overlay")?
                .data()?,
        )?;
        let note = cargo_rp1::descriptor(
            file.section_by_name(".note.rp1")
                .ok_or("missing .note.rp1")?
                .data()?,
        )?;
        let conflicts = rp1_dt::check::check(&before, &after, note)?;
        if !conflicts.is_empty() {
            return Err(format!(
                "final DTB conflicts with the RP1 note:\n  {}",
                conflicts.join("\n  ")
            )
            .into());
        }
    }
    println!(
        "STATIC: image {}/49152 B, ISRAM {}/8192 B, DSRAM {} B + declared stack; {} exclusive instructions checked",
        report.image_bytes, report.isram_bytes, report.dsram_bytes, report.exclusive_instructions
    );
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("cargo-rp1: {e}");
        std::process::exit(1);
    }
}
