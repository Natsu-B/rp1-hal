use object::{Object, ObjectSection};
use std::path::{Path, PathBuf};
fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.first().is_some_and(|s| s == "rp1") {
        args.remove(0);
    }
    const USAGE: &str = "usage: cargo rp1 check <firmware.elf> [--dtb <base.dtb>]\n       \
        cargo rp1 bundle <firmware.elf> --kernel <Image[.gz]> [--initramfs <file>] \
        [--loader <rp1boot.img>] [--cmdline <args>] [--config-extra <line>]... --out <dir>";
    if args.first().is_some_and(|s| s == "bundle") {
        return bundle(&args[1..]).map_err(|e| format!("{e}\n{USAGE}").into());
    }
    if args.len() < 2 || args[0] != "check" {
        return Err(USAGE.into());
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
/// `cargo rp1 bundle`: check the ELF, then write the boot set for rp1boot.
fn bundle(args: &[std::ffi::OsString]) -> Result<(), Box<dyn std::error::Error>> {
    let (mut elf, mut kernel, mut initramfs, mut loader, mut out) = (None, None, None, None, None);
    let (mut cmdline, mut extra) = (String::new(), Vec::new());
    let mut it = args.iter();
    while let Some(a) = it.next() {
        let mut value = || it.next().ok_or(format!("{a:?} needs a value"));
        match a.to_str() {
            Some("--kernel") => kernel = Some(PathBuf::from(value()?)),
            Some("--initramfs") => initramfs = Some(PathBuf::from(value()?)),
            Some("--loader") => loader = Some(PathBuf::from(value()?)),
            Some("--out") => out = Some(PathBuf::from(value()?)),
            Some("--cmdline") => cmdline = value()?.to_str().ok_or("non-UTF-8 cmdline")?.into(),
            Some("--config-extra") => extra.push(
                value()?
                    .to_str()
                    .ok_or("non-UTF-8 config line")?
                    .to_string(),
            ),
            _ if elf.is_none() => elf = Some(PathBuf::from(a)),
            _ => return Err(format!("unexpected argument {a:?}").into()),
        }
    }
    let (elf, kernel, out) = (
        elf.ok_or("missing firmware ELF")?,
        kernel.ok_or("missing --kernel")?,
        out.ok_or("missing --out")?,
    );
    let objdump = std::env::var_os("RP1_OBJDUMP").unwrap_or_else(|| "arm-none-eabi-objdump".into());
    cargo_rp1::check(&elf, Path::new(&objdump))?;
    let firmware = std::fs::read(&elf)?;
    let initramfs = initramfs.map(std::fs::read).transpose()?;
    let set = cargo_rp1::boot::assemble(&cargo_rp1::boot::Input {
        firmware: &firmware,
        kernel: &std::fs::read(&kernel)?,
        initramfs: initramfs.as_deref(),
        cmdline: &cmdline,
        config_extra: &extra,
    })?;
    let loader = loader.map(std::fs::read).transpose()?;
    print!(
        "{}",
        cargo_rp1::boot::write(&out, &set, &firmware, loader.as_deref())?
    );
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("cargo-rp1: {e}");
        std::process::exit(1);
    }
}
