use object::{Object, ObjectSection};
use std::path::{Path, PathBuf};
fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.first().is_some_and(|s| s == "rp1") {
        args.remove(0);
    }
    const USAGE: &str = "usage: cargo rp1 check <firmware.elf> [--dtb <base.dtb>]\n       \
        cargo rp1 bundle <firmware.elf> --kernel <Image[.gz]> [--initramfs <file>] \
        [--loader <rp1boot.img>] [--cmdline <args>] [--config-extra <line>]... --out <dir>\n       \
        cargo rp1 isram-plan <firmware.elf> [--out <plan.x>]\n       \
        cargo rp1 build <cargo build arguments> (two-stage link with automatic ISRAM placement)";
    if args.first().is_some_and(|s| s == "isram-plan") {
        return isram_plan(&args[1..]).map_err(|e| format!("{e}\n{USAGE}").into());
    }
    if args.first().is_some_and(|s| s == "build") {
        return build(&args[1..]);
    }
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

fn objdump() -> std::ffi::OsString {
    std::env::var_os("RP1_OBJDUMP").unwrap_or_else(|| "arm-none-eabi-objdump".into())
}
fn print_plan(plan: &cargo_rp1::isram::Plan) {
    for (name, size) in &plan.functions {
        println!("  isram+ {size:5} {name}");
    }
    for name in &plan.boundaries {
        println!("  stays (cold/panic/fmt) {name}");
    }
    for name in &plan.pre_isram {
        println!("  stays (runs before ISRAM is copied) {name}");
    }
    println!(
        "STATIC: ISRAM plan {} functions, {} B now + {} B planned = {}/{} B",
        plan.functions.len(),
        plan.current_bytes,
        plan.planned_bytes,
        plan.total_bytes(),
        cargo_rp1::isram::ISRAM_SIZE
    );
}
/// `cargo rp1 isram-plan`: print the automatic placement for a linked ELF.
fn isram_plan(args: &[std::ffi::OsString]) -> Result<(), Box<dyn std::error::Error>> {
    let elf = PathBuf::from(args.first().ok_or("missing ELF")?);
    let plan = cargo_rp1::isram::plan(&elf, Path::new(&objdump()))?;
    print_plan(&plan);
    match (args.get(1).and_then(|a| a.to_str()), args.get(2)) {
        (Some("--out"), Some(out)) => std::fs::write(out, plan.fragment())?,
        (None, None) => {}
        _ => return Err("unexpected isram-plan arguments".into()),
    }
    Ok(())
}
/// Run `cargo build` and return the RP1 executable it produced (exactly one).
fn cargo_build(
    args: &[std::ffi::OsString],
    plan: Option<&Path>,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let mut cmd = std::process::Command::new(cargo);
    cmd.arg("build")
        .arg("--message-format=json-render-diagnostics")
        .args(args);
    match plan {
        Some(p) => cmd.env("RP1_ISRAM_PLAN", p),
        None => cmd.env_remove("RP1_ISRAM_PLAN"),
    };
    let out = cmd.stderr(std::process::Stdio::inherit()).output()?;
    if !out.status.success() {
        return Err("cargo build failed".into());
    }
    let mut elves = Vec::new();
    for line in String::from_utf8(out.stdout)?.lines() {
        if let Some(i) = line.find("\"executable\":\"") {
            let rest = &line[i + 14..];
            let path = &rest[..rest.find('"').ok_or("bad cargo message")?];
            if path.contains("thumbv7m-none-eabi") {
                elves.push(PathBuf::from(path));
            }
        }
    }
    match elves.as_slice() {
        [one] => Ok(one.clone()),
        [] => Err("no thumbv7m-none-eabi executable built (pass --target thumbv7m-none-eabi -p <firmware>)".into()),
        _ => Err("more than one RP1 executable; select one with -p".into()),
    }
}
/// `cargo rp1 build`: first link, plan from its call graph, second link with the plan, then
/// check that every planned function landed in ISRAM and the budgets hold.
fn build(args: &[std::ffi::OsString]) -> Result<(), Box<dyn std::error::Error>> {
    let objdump = objdump();
    let first = cargo_build(args, None)?;
    let plan = cargo_rp1::isram::plan(&first, Path::new(&objdump))?;
    print_plan(&plan);
    if plan.total_bytes() > cargo_rp1::isram::ISRAM_SIZE {
        return Err(format!(
            "ISRAM plan {} B exceeds {} B; mark large callees #[rp1_rt::cold] or drop #[isram] roots",
            plan.total_bytes(),
            cargo_rp1::isram::ISRAM_SIZE
        )
        .into());
    }
    let fragment = first.with_extension("isram.x");
    std::fs::write(&fragment, plan.fragment())?;
    let second = cargo_build(args, Some(&fragment))?;
    cargo_rp1::isram::verify(&second, &plan)?;
    let report = cargo_rp1::check(&second, Path::new(&objdump))?;
    println!(
        "STATIC: {} built with automatic ISRAM placement (plan {}): image {}/49152 B, ISRAM {}/8192 B, DSRAM {} B + declared stack",
        second.display(),
        fragment.display(),
        report.image_bytes,
        report.isram_bytes,
        report.dsram_bytes
    );
    Ok(())
}
