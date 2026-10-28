//! Build-time declaration validation and reproducible static artifacts.
pub use rp1_dt::fdt;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};
pub mod metadata;
pub mod overlay;
pub mod wire;
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Deserialize)]
struct Manifest {
    package: Package,
}
#[derive(Deserialize)]
struct Package {
    name: String,
    version: toml::Value,
    metadata: Metadata,
}
#[derive(Deserialize)]
struct Metadata {
    rp1: metadata::Declaration,
}

pub fn declaration(path: &Path) -> Result<(String, String, metadata::Declaration)> {
    let m: Manifest = toml::from_str(&fs::read_to_string(path)?)?;
    m.package.metadata.rp1.validate()?;
    let version = if let Some(v) = m.package.version.as_str() {
        v.to_string()
    } else if m
        .package
        .version
        .get("workspace")
        .and_then(toml::Value::as_bool)
        == Some(true)
    {
        path.parent()
            .ok_or("manifest has no parent")?
            .ancestors()
            .find_map(|dir| {
                let v: toml::Value =
                    toml::from_str(&fs::read_to_string(dir.join("Cargo.toml")).ok()?).ok()?;
                v.get("workspace")?
                    .get("package")?
                    .get("version")?
                    .as_str()
                    .map(str::to_string)
            })
            .ok_or("workspace package version not found")?
    } else {
        return Err("invalid package version".into());
    };
    Ok((m.package.name, version, m.package.metadata.rp1))
}

/// One-line build.rs entry point. Does not touch hardware or the boot directory.
pub fn configure() -> Result<()> {
    configure_with_types(&[])
}

pub fn configure_with_types(types: &[(&str, metadata::TypeInfo)]) -> Result<()> {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let path = manifest_dir.join("Cargo.toml");
    let (name, version, mut d) = declaration(&path)?;
    d.types = types.iter().map(|(n, t)| (n.to_string(), *t)).collect();
    let out = PathBuf::from(env::var("OUT_DIR")?);
    let declaration_hash: [u8; 32] = Sha256::digest(toml::to_string(&d)?.as_bytes()).into();
    let git = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(&manifest_dir)
        .output()?;
    let git = String::from_utf8_lossy(&git.stdout);
    let mut source = [0u8; 20];
    if git.trim().len() == 40 {
        for (i, b) in source.iter_mut().enumerate() {
            *b = u8::from_str_radix(&git[i * 2..i * 2 + 2], 16)?;
        }
    }
    let artifacts = generate(&name, &version, &d, declaration_hash, source)?;
    fs::create_dir_all(&out)?;
    fs::write(out.join("rp1-app.dts"), &artifacts.dts)?;
    overlay::compile(&out.join("rp1-app.dts"), &out.join("rp1-app.dtbo"))?;
    fs::write(out.join("rp1-layout.x"), &artifacts.layout)?;
    // Automatic ISRAM placement (`cargo rp1 build`, second link): the plan is a list of
    // input-section rules included inside `.isram`. Without a plan the include is empty.
    println!("cargo:rerun-if-env-changed=RP1_ISRAM_PLAN");
    let plan = match env::var_os("RP1_ISRAM_PLAN") {
        Some(p) if !p.is_empty() => {
            println!("cargo:rerun-if-changed={}", PathBuf::from(&p).display());
            fs::read_to_string(&p)?
        }
        _ => String::from("/* no automatic ISRAM plan */\n"),
    };
    fs::write(out.join("rp1-isram-auto.x"), plan)?;
    fs::write(out.join("rp1-note-desc.bin"), &artifacts.note)?;
    fs::write(
        out.join("70-rp1.rules"),
        format!(
            "SUBSYSTEM==\"uio\", ATTR{{name}}==\"rp1:*\", GROUP=\"{}\", MODE=\"0660\"\n",
            d.linux.uio_group
        ),
    )?;
    fs::write(out.join("rp1.rs"), resources(&d, &artifacts.note)?)?;
    println!("cargo:rerun-if-changed={}", path.display());
    println!(
        "cargo:rerun-if-changed={}",
        manifest_dir.join("../../rp1-rt/link.x").display()
    );
    println!("cargo:rerun-if-env-changed=RP1_DTC");
    // A new commit changes the provenance record even when Cargo.toml stays identical.
    for p in git_watch_paths(&manifest_dir) {
        println!("cargo:rerun-if-changed={}", p.display());
    }
    println!("cargo:rustc-check-cfg=cfg(rp1_log)");
    if d.log.is_some() {
        println!("cargo:rustc-cfg=rp1_log");
    }
    if env::var("TARGET")?.starts_with("thumbv7m-") {
        println!("cargo:rustc-link-search={}", out.display());
        println!("cargo:rustc-link-arg=-Trp1-layout.x");
        println!("cargo:rustc-link-arg=--nmagic");
    }
    Ok(())
}

fn git(dir: &Path, args: &[&str]) -> Option<String> {
    let out = Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// Files whose change means `git rev-parse HEAD` may have changed: `HEAD` itself and, on a
/// branch, the ref it points to. A commit on a branch rewrites only that ref (loose file, or
/// `packed-refs` after `git pack-refs`), never `HEAD`. A loose ref that does not exist yet is
/// watched through its nearest existing directory so its creation is noticed. Dirty working
/// trees are not part of the record (it names the commit only).
pub fn git_watch_paths(dir: &Path) -> Vec<PathBuf> {
    let git_path = |p: &str| git(dir, &["rev-parse", "--git-path", p]).map(|p| dir.join(p));
    let mut paths: Vec<PathBuf> = git_path("HEAD").into_iter().collect();
    if let Some(r) = git(dir, &["symbolic-ref", "-q", "HEAD"]) {
        if let Some(loose) = git_path(&r) {
            paths.extend(
                loose
                    .ancestors()
                    .find(|p| p.exists())
                    .map(Path::to_path_buf),
            );
        }
        paths.extend(git_path("packed-refs").filter(|p| p.exists()));
    }
    paths
}

pub struct Artifacts {
    pub note: Vec<u8>,
    pub layout: String,
    pub dts: String,
}
pub fn generate(
    name: &str,
    version: &str,
    d: &metadata::Declaration,
    hash: [u8; 32],
    source: [u8; 20],
) -> Result<Artifacts> {
    d.validate()?;
    let note = wire::descriptor(name, version, d, hash, source)?;
    let layout = format!(
        "__stack_size0 = {};\n__stack_size1 = {};\n{}",
        d.stack_size(0)?,
        d.stack_size(1)?,
        include_str!("../../rp1-rt/link.x")
    );
    let dts = overlay::dts(d, &note)?;
    Ok(Artifacts { note, layout, dts })
}

fn resources(d: &metadata::Declaration, note: &[u8]) -> Result<String> {
    let elf_note = wire::elf_note(note);
    let mut s = format!(
        "pub mod rp1 {{\n#[used] #[unsafe(link_section = \".note.rp1\")] static NOTE: [u8;{}] = {:?};\n#[used] #[unsafe(link_section = \".rp1.dtbo\")] static OVERLAY: [u8;include_bytes!(concat!(env!(\"OUT_DIR\"),\"/rp1-app.dtbo\")).len()] = *include_bytes!(concat!(env!(\"OUT_DIR\"),\"/rp1-app.dtbo\"));\n",
        elf_note.len(),
        elf_note
    );
    let digest: [u8; 32] = Sha256::digest(note).into();
    s.push_str(&format!(
        "pub const SHM_NOTE: &[u8] = &{:?};\npub const SHM_NOTE_SHA256: [u8;32] = {:?};\n",
        note, digest
    ));
    for (index, c) in [&d.proc0, &d.proc1].into_iter().enumerate() {
        let ty = format!("Proc{index}Resources");
        s.push_str(&format!("pub struct Proc{index}Pins {{\n"));
        for pin in &c.pins {
            s.push_str(&format!("pub gpio{pin}: ::rp1_hal::gpio::Pin<{pin}>,\n"));
        }
        s.push_str("}\n#[allow(non_snake_case)]\n");
        s.push_str(&format!("pub struct {ty} {{ pub pins: Proc{index}Pins,\n"));
        for p in &c.peripherals {
            s.push_str(&format!(
                "pub {}: {},\n",
                p.replace('.', "_"),
                metadata::peripheral_type(p)?
            ));
        }
        if index == 0 {
            s.push_str("pub proc1: ::rp1_rt::Proc1Launcher,\n");
            if d.clock.authority == "rp1" {
                s.push_str("pub clock_authority: ::rp1_hal::clocks::Authority,\n");
            }
        }
        s.push_str(&format!("}}\nimpl {ty} {{\n/// # Safety\n/// Runtime only, once on the owning core.\npub unsafe fn new() -> Self {{ Self {{ pins: Proc{index}Pins {{\n"));
        for pin in &c.pins {
            s.push_str(&format!(
                "gpio{pin}: unsafe {{ ::rp1_hal::gpio::Pin::new() }},\n"
            ));
        }
        s.push_str("},\n");
        for p in &c.peripherals {
            let ty = metadata::peripheral_type(p)?;
            s.push_str(&format!(
                "{}: unsafe {{ <{ty}>::new() }},\n",
                p.replace('.', "_")
            ));
        }
        if index == 0 {
            s.push_str("proc1: unsafe { ::rp1_rt::Proc1Launcher::new() },\n");
            if d.clock.authority == "rp1" {
                s.push_str("clock_authority: unsafe { ::rp1_hal::clocks::Authority::new() },\n");
            }
        }
        s.push_str("} } }\n");
    }
    if let Some(log) = &d.log {
        s.push_str(&format!("pub mod log {{\n#[unsafe(link_section=\".shared\")] static LOGGER: ::rp1_hal::logging::Logger = unsafe {{ ::rp1_hal::logging::Logger::new() }};\npub fn init(core: &mut ::rp1_rt::Proc0) -> Result<(),::rp1_hal::Error> {{ LOGGER.init(core,{}) }}\npub fn write(args: ::core::fmt::Arguments<'_>) -> ::core::fmt::Result {{ LOGGER.write(args) }}\n}}\n",log.baudrate));
    }
    s.push_str("}\n");
    Ok(s)
}
