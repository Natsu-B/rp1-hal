use object::{Object, ObjectSection};
use std::{
    path::{Path, PathBuf},
    process::Command,
};
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
        let parent = elf.parent().ok_or("ELF needs a parent directory")?;
        let overlay = parent.join("rp1-check.dtbo");
        let merged = parent.join("rp1-check-merged.dtb");
        if overlay.exists() || merged.exists() {
            return Err("check output already exists; remove rp1-check artifacts first".into());
        }
        std::fs::write(
            &overlay,
            file.section_by_name(".rp1.dtbo")
                .ok_or("missing overlay")?
                .data()?,
        )?;
        let result = rp1_build::overlay::apply(&base, &overlay, &merged).and_then(|()| {
            let decoded = Command::new("dtc")
                .args(["-I", "dtb", "-O", "dts"])
                .arg(&merged)
                .output()?;
            if !decoded.status.success() {
                return Err("merged DTB failed decoding".into());
            }
            // Overlays replace gpio-reserved-ranges; Linux must not end up with more pins.
            let before = rp1_build::overlay::reserved_gpios(&base)?;
            let after = rp1_build::overlay::reserved_gpios(&merged)?;
            let freed: Vec<_> = before.difference(&after).collect();
            if !freed.is_empty() {
                return Err(format!("overlay would hand RP1 GPIOs {freed:?} back to Linux").into());
            }
            Ok(())
        });
        let _ = std::fs::remove_file(&overlay);
        let _ = std::fs::remove_file(&merged);
        result?;
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
