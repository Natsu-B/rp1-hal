//! Build-time declaration validation and reproducible static artifacts.
pub use rp1_dt::fdt;
use serde::Deserialize;
use std::{
    collections::BTreeMap,
    env,
    fmt::Write as _,
    fs,
    path::{Path, PathBuf},
    process::Command,
};
pub mod board;
mod declaration;
mod link;
pub mod metadata;
pub mod overlay;
pub mod wire;
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Deserialize)]
struct Manifest {
    package: Package,
    #[serde(default)]
    features: BTreeMap<String, Vec<String>>,
}
#[derive(Deserialize)]
struct Package {
    name: String,
    version: toml::Value,
    metadata: Metadata,
}
#[derive(Deserialize)]
struct Metadata {
    rp1: toml::Value,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ConditionalWaitTimer {
    alarm: String,
    feature: String,
}

/// Read and validate the package declaration with feature-conditional wait timers
/// reserved as raw alarms.
pub fn declaration(path: &Path) -> Result<(String, String, metadata::Declaration)> {
    declaration_for_features(path, &[])
}

/// Read and validate the package declaration for the given active Cargo features.
pub fn declaration_for_features(
    path: &Path,
    features: &[&str],
) -> Result<(String, String, metadata::Declaration)> {
    load_declaration(path, |feature| features.contains(&feature))
}

fn load_declaration(
    path: &Path,
    feature_enabled: impl Fn(&str) -> bool,
) -> Result<(String, String, metadata::Declaration)> {
    let mut m: Manifest = toml::from_str(&fs::read_to_string(path)?)?;
    for owner in ["proc0", "proc1"] {
        let Some(core) = m
            .package
            .metadata
            .rp1
            .get_mut(owner)
            .and_then(toml::Value::as_table_mut)
        else {
            continue;
        };
        let Some(wait_timer) = core.get_mut("wait_timer").filter(|value| value.is_table()) else {
            continue;
        };
        let ConditionalWaitTimer { alarm, feature } = wait_timer.clone().try_into()?;
        if !m.features.contains_key(&feature) {
            return Err(
                format!("E0: {owner}.wait_timer: unknown Cargo feature '{feature}'").into(),
            );
        }
        if feature_enabled(&feature) {
            *wait_timer = toml::Value::String(alarm);
        } else {
            core.remove("wait_timer");
            core.entry("timer")
                .or_insert_with(|| toml::Value::Array(Vec::new()))
                .as_array_mut()
                .ok_or_else(|| format!("E0: {owner}.timer must be an array"))?
                .push(toml::Value::String(alarm));
        }
    }
    let d: metadata::Declaration = m.package.metadata.rp1.try_into()?;
    d.validate()?;
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
    Ok((m.package.name, version, d))
}

/// One-line build.rs entry point. Does not touch hardware or the boot directory.
pub fn configure() -> Result<()> {
    configure_with_types(&[])
}

pub fn configure_with_types(types: &[(&str, metadata::TypeInfo)]) -> Result<()> {
    configure_with_manifest(Path::new("Cargo.toml"), types)
}

/// Configure from an RP1 manifest path relative to the firmware package.
pub fn configure_with_manifest(path: &Path, types: &[(&str, metadata::TypeInfo)]) -> Result<()> {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let path = manifest_dir.join(path);
    let (name, version, mut d) = load_declaration(&path, |feature| {
        // Cargo supplies one CARGO_FEATURE_<NAME> to this build script per active feature.
        let key = format!("CARGO_FEATURE_{}", feature.to_uppercase().replace('-', "_"));
        env::var_os(key).is_some()
    })?;
    d.types = types.iter().map(|(n, t)| (n.to_string(), *t)).collect();
    d.validate()?;
    let out = PathBuf::from(env::var("OUT_DIR")?);
    for warning in d.warnings() {
        println!("cargo:warning={warning}");
    }
    let declaration_hash = d.declaration_hash()?;
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
    let shm = d.shm_layout()?;
    let artifacts = generate_with_layout(&name, &version, &d, declaration_hash, source, &shm)?;
    fs::create_dir_all(&out)?;
    fs::write(out.join("rp1-app.dts"), &artifacts.dts)?;
    overlay::compile(&out.join("rp1-app.dts"), &out.join("rp1-app.dtbo"))?;
    // E6 (optional): predict rp1boot on the given board DTB; no host path is assumed.
    println!("cargo:rerun-if-env-changed=RP1_BOARD_DTB");
    if let Some(board) = env::var_os("RP1_BOARD_DTB").filter(|p| !p.is_empty()) {
        let board = PathBuf::from(board);
        println!("cargo:rerun-if-changed={}", board.display());
        let dtbo = fs::read(out.join("rp1-app.dtbo"))?;
        let prediction = overlay::board_dtb_check(&board, &dtbo, &artifacts.note)?;
        for line in prediction.report() {
            println!("cargo:warning={line}");
        }
    }
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
    fs::write(out.join("rp1.rs"), resources(&d, &artifacts.note, &shm)?)?;
    println!("cargo:rerun-if-changed={}", path.display());
    println!(
        "cargo:rerun-if-changed={}",
        manifest_dir.join("../../rp1-rt/link.x").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        manifest_dir.join("../../rp1-pac/irq.x").display()
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
    generate_with_layout(name, version, d, hash, source, &d.shm_layout()?)
}

fn generate_with_layout(
    name: &str,
    version: &str,
    d: &metadata::Declaration,
    hash: [u8; 32],
    source: [u8; 20],
    shm: &metadata::ShmLayout<'_>,
) -> Result<Artifacts> {
    let note = wire::descriptor_with_layout(name, version, d, hash, source, shm)?;
    let layout = format!(
        "__stack_size0 = {};\n__stack_size1 = {};\n__stack_margin0 = {};\n{}\n{}{}",
        d.stack_size(0)?,
        d.stack_size(1)?,
        metadata::Declaration::stack_margin(0),
        include_str!("../../rp1-rt/link.x"),
        custom_irq_assertions(d),
        include_str!("../../rp1-pac/irq.x")
    );
    let dts = overlay::dts_with_layout(d, &note, shm)?;
    Ok(Artifacts { note, layout, dts })
}

fn custom_irq_assertions(d: &metadata::Declaration) -> String {
    let mut checks = String::new();
    for (core, c) in [&d.proc0, &d.proc1].into_iter().enumerate() {
        if c.custom_doorbell_handler {
            checks.push_str(&format!(
                "ASSERT(__rp1_p{core}_IO_BANK0 != __rp1_default_irq, \"proc{core} needs a custom IO_BANK0 handler\");\n"
            ));
        }
    }
    checks
}

fn resources(
    d: &metadata::Declaration,
    note: &[u8],
    shm: &metadata::ShmLayout<'_>,
) -> Result<String> {
    let elf_note = wire::elf_note(note);
    let mut s = format!(
        "pub mod rp1 {{\n#[used] #[unsafe(link_section = \".note.rp1\")] static NOTE: [u8;{}] = {:?};\n#[used] #[unsafe(link_section = \".rp1.dtbo\")] static OVERLAY: [u8;include_bytes!(concat!(env!(\"OUT_DIR\"),\"/rp1-app.dtbo\")).len()] = *include_bytes!(concat!(env!(\"OUT_DIR\"),\"/rp1-app.dtbo\"));\n",
        elf_note.len(),
        elf_note
    );
    let experimental_wake = d.link_wake.is_some();
    if experimental_wake {
        s.push_str("const _: () = ::rp1_rt::status::__REQUIRES_EXPERIMENTAL_LINK_WAKE;\n");
    }
    if experimental_wake || !shm.regions.is_empty() {
        s.push_str("pub mod link {\n");
        s.push_str(&link::generate(shm, note, experimental_wake)?);
        s.push_str("}\n");
    }
    for (index, c) in [&d.proc0, &d.proc1].into_iter().enumerate() {
        if let Some(pin) = c.doorbell {
            s.push_str(&format!(
                "/// proc{index}'s doorbell, which Linux rings (`doorbell = {pin}`).\n\
                 pub type Doorbell{index} = ::rp1_link::notify::Doorbell<{pin}, {index}>;\n"
            ));
        }
    }
    for (index, c) in [&d.proc0, &d.proc1].into_iter().enumerate() {
        let ty = format!("Proc{index}Resources");
        let has_link = !experimental_wake
            && shm
                .regions
                .iter()
                .any(|r| usize::from(r.core()) == index + 1);
        let doorbell = !c.custom_doorbell_handler
            && shm.regions.iter().any(|r| r.receives_doorbell(index as u8));
        if doorbell {
            let ack = if experimental_wake {
                "__ack_and_signal_wake"
            } else {
                "ack_and_signal"
            };
            writeln!(
                s,
                "#[unsafe(export_name = \"__rp1_p{index}_IO_BANK0\")]\n\
                #[unsafe(link_section = \".isram.text.__rp1_p{index}_IO_BANK0\")]\n\
                unsafe extern \"C\" fn doorbell_proc{index}() {{\n\
                    unsafe {{ Doorbell{index}::new() }}.{ack}();\n\
                }}"
            )?;
        }
        let wait_alarm = c
            .wait_timer
            .as_deref()
            .map(|alarm| alarm.strip_prefix("ALARM").unwrap().parse::<u8>())
            .transpose()?;
        if let Some(alarm) = wait_alarm {
            writeln!(
                s,
                "#[unsafe(export_name = \"__rp1_p{index}_TIMER_{alarm}\")]\n\
                #[unsafe(link_section = \".isram.text.__rp1_p{index}_TIMER_{alarm}\")]\n\
                unsafe extern \"C\" fn wait_timer_proc{index}() {{\n\
                    unsafe {{ ::rp1_hal::timer::WaitTimer::__on_interrupt::<{alarm}>() }};\n\
                }}"
            )?;
        }
        let dma_pool = c.instances.get("dma").and_then(|instance| match instance {
            metadata::Instance::Dma(dma) => dma.pool,
            _ => None,
        });
        if let Some(metadata::DmaPool { slots, bytes }) = dma_pool {
            writeln!(
                s,
                "#[repr(C, align(16))]\nstruct Proc{index}DmaMemory([[u8; {bytes}]; {slots}]);\n\
                 #[unsafe(link_section = \".shared\")]\n\
                 static mut PROC{index}_DMA_MEMORY: Proc{index}DmaMemory =\n\
                 Proc{index}DmaMemory([[0; {bytes}]; {slots}]);"
            )?;
        }
        // Instances come as bundles of their token and pins; `pins` keeps the rest.
        let parts = c
            .instances
            .iter()
            .map(|(name, instance)| match instance {
                metadata::Instance::Dma(channels) => {
                    dma_resource(&mut s, index, channels.selected())
                }
                _ => instance_resource(&mut s, name, instance),
            })
            .collect::<Result<Vec<_>>>()?;
        // The doorbell pin's force bit belongs to the doorbell, not to a `Pin`.
        let pins: Vec<u8> = c
            .pins()
            .into_iter()
            .filter(|p| Some(*p) != c.doorbell)
            .filter(|p| !parts.iter().any(|(_, _, used)| used.contains(p)))
            .collect();
        s.push_str(&format!("pub struct Proc{index}Pins {{\n_private: ::core::marker::PhantomData<::core::cell::Cell<()>>,\n"));
        for pin in &pins {
            s.push_str(&format!("pub gpio{pin}: ::rp1_hal::gpio::Pin<{pin}>,\n"));
        }
        let lines: Vec<_> = board::IRQ_VERIFIED
            .iter()
            .filter(|line| match line.owner {
                board::IrqOwner::GpioBank0 => !c.gpio_irq.is_empty() || c.doorbell.is_some(),
                board::IrqOwner::Alarm(alarm) => c.timer.contains(&format!("ALARM{alarm}")),
                // By block, not name: the ADC's line is ADC_FIFO.
                board::IrqOwner::Instance(_) => c.instances.keys().any(|key| {
                    metadata::peripheral_id(&key.to_ascii_uppercase()).ok()
                        == Some(line.owner.ownership())
                }),
            })
            .collect();
        let undeclared = format!(
            "{{Self}} is not declared for proc{index}. Add a supported source to [package.metadata.rp1.proc{index}]"
        );
        let bank0_error = if doorbell {
            "IO_BANK0 is handled by the generated doorbell dispatcher. Set custom_doorbell_handler = true"
        } else {
            &undeclared
        };
        writeln!(
            s,
            "}}\n#[doc(hidden)]\n#[allow(non_camel_case_types)]\npub mod __irq_proc{index} {{\n\
             pub trait Handler {{}}\n\
             mod sealed {{ pub trait Admitted {{}} }}\n\
             #[diagnostic::on_unimplemented(message = {undeclared:?})]\n\
             pub trait Declared: sealed::Admitted {{}}\n\
             impl<T: sealed::Admitted> Declared for T {{}}\n\
             #[diagnostic::on_unimplemented(message = {bank0_error:?})]\n\
             pub trait Bank0Declared: sealed::Admitted {{}}\n\
             impl<T: sealed::Admitted> Bank0Declared for T {{}}"
        )?;
        for name in rp1_pac::INTERRUPT_NAMES {
            writeln!(s, "pub enum {name} {{}}")?;
        }
        for line in &lines {
            if !(doorbell && line.irq == 0) {
                writeln!(s, "impl sealed::Admitted for {} {{}}", line.name)?;
            }
        }
        writeln!(
            s,
            "}}\n/// Binding types for IRQ resources declared by proc{index}.\n\
             #[allow(non_camel_case_types)]\npub mod proc{index}_irq {{"
        )?;
        for line in &lines {
            let board::VerifiedIrq { name, irq, owner } = line;
            let source = owner.source_type();
            writeln!(
                s,
                "pub type {name} = ::rp1_rt::interrupt::Binding<::rp1_rt::interrupt::{source}, {irq}, {index}>;"
            )?;
        }
        writeln!(
            s,
            "}}\n#[allow(non_snake_case)]\npub struct Proc{index}Irq {{\n_private: ::core::marker::PhantomData<::core::cell::Cell<()>>,"
        )?;
        for line in &lines {
            let name = line.name;
            writeln!(s, "pub {name}: proc{index}_irq::{name},")?;
        }
        for pin in &c.gpio_irq {
            s.push_str(&format!(
                "pub gpio{pin}: ::rp1_hal::gpio::PinIrq<{pin}, {index}>,\n"
            ));
        }
        s.push_str("}\n#[allow(non_snake_case)]\n");
        s.push_str(&format!(
            "#[non_exhaustive]\npub struct {ty} {{ pub pins: Proc{index}Pins, pub irq: Proc{index}Irq,\n"
        ));
        for alarm in &c.timer {
            let p = format!("TIMER.{alarm}");
            s.push_str(&format!(
                "pub TIMER_{alarm}: {},\n",
                metadata::peripheral_type(&p)?
            ));
        }
        if wait_alarm.is_some() {
            s.push_str("pub wait_timer: ::rp1_hal::timer::WaitTimer,\n");
        }
        if let Some(pool) = dma_pool {
            writeln!(s, "pub dma_pool: ::rp1_hal::dma::Pool<{}>,", pool.slots)?;
        }
        for (name, (field_ty, _, _)) in c.instances.keys().zip(&parts) {
            s.push_str(&format!("pub {name}: {field_ty},\n"));
        }
        if c.host_irq {
            if experimental_wake {
                s.push_str("pub host_irq: ::rp1_link::notify::VectorOwner,\n");
            } else {
                s.push_str("pub host_irq: ::rp1_link::notify::HostIrq,\n");
            }
        }
        if c.doorbell.is_some() {
            s.push_str(&format!("pub doorbell: Doorbell{index},\n"));
        }
        if has_link {
            s.push_str(&format!("pub link: link::Proc{index}LinkInit,\n"));
        }
        if index == 0 {
            s.push_str("pub proc1: ::rp1_rt::Proc1Launcher,\n");
            if d.clock.authority == "rp1" {
                s.push_str("pub platform: ::rp1_hal::platform::Token,\n");
            }
        }
        s.push_str(&format!("_core: ::core::marker::PhantomData<::rp1_rt::Proc{index}>,\n}}\nimpl {ty} {{\n/// # Safety\n/// Runtime only, once on the owning core.\npub(crate) unsafe fn new() -> Self {{ Self {{ _core: ::core::marker::PhantomData, pins: Proc{index}Pins {{\n_private: ::core::marker::PhantomData,\n"));
        for pin in &pins {
            s.push_str(&format!(
                "gpio{pin}: unsafe {{ ::rp1_hal::gpio::Pin::new() }},\n"
            ));
        }
        s.push_str("},\n");
        s.push_str(&format!(
            "irq: Proc{index}Irq {{\n_private: ::core::marker::PhantomData,\n"
        ));
        for line in &lines {
            let name = line.name;
            s.push_str(&format!(
                "{name}: unsafe {{ ::rp1_rt::interrupt::Binding::new() }},\n"
            ));
        }
        for pin in &c.gpio_irq {
            s.push_str(&format!(
                "gpio{pin}: unsafe {{ ::rp1_hal::gpio::PinIrq::new() }},\n"
            ));
        }
        s.push_str("},\n");
        for alarm in &c.timer {
            let ty = metadata::peripheral_type(&format!("TIMER.{alarm}"))?;
            s.push_str(&format!("TIMER_{alarm}: unsafe {{ <{ty}>::new() }},\n"));
        }
        if let Some(alarm) = wait_alarm {
            writeln!(
                s,
                "wait_timer: unsafe {{ ::rp1_hal::timer::WaitTimer::__bind::<{alarm}, {}, {index}>(\n\
                    ::rp1_hal::timer::Alarm::new(), ::rp1_rt::interrupt::Binding::new()) }},",
                26 + alarm
            )?;
        }
        if dma_pool.is_some() {
            // SAFETY in the generated code: only the owning core's once-per-epoch
            // constructor can borrow this private, linker-aligned shared SRAM static.
            writeln!(
                s,
                "dma_pool: unsafe {{ ::rp1_hal::dma::Pool::__from_declared(\n\
                    &raw mut PROC{index}_DMA_MEMORY.0) }},"
            )?;
        }
        for (name, (_, init, _)) in c.instances.keys().zip(&parts) {
            s.push_str(&format!("{name}: {init},\n"));
        }
        if c.host_irq {
            if experimental_wake {
                s.push_str("host_irq: unsafe { ::rp1_link::notify::VectorOwner::__new() },\n");
            } else {
                s.push_str("host_irq: unsafe { ::rp1_link::notify::HostIrq::new() },\n");
            }
        }
        if c.doorbell.is_some() {
            s.push_str(&format!("doorbell: unsafe {{ Doorbell{index}::new() }},\n"));
        }
        if has_link {
            s.push_str(&format!("link: link::Proc{index}LinkInit::new(),\n"));
        }
        if index == 0 {
            s.push_str("proc1: unsafe { ::rp1_rt::Proc1Launcher::new() },\n");
            if d.clock.authority == "rp1" {
                let [r0, r1, r2] = d.linux_resets();
                // The only clk_sys rate v0.1 accepts, also when the declaration leaves it out.
                let sys_hz = d.clock.clk_sys_hz.unwrap_or(200_000_000);
                s.push_str(&format!("platform: unsafe {{ ::rp1_hal::platform::Token::new(::rp1_hal::linux_blocks::LinuxBlocks::new([{r0:#x}, {r1:#x}, {r2:#x}]), {sys_hz}) }},\n"));
            }
        }
        s.push_str("} } }\n");
    }
    if d.clock.authority == "rp1" {
        // Step 3 of the platform init is the declared log, if there is one.
        let log = if d.log.is_some() {
            "super::log::init"
        } else {
            "|_| Ok(())"
        };
        s.push_str(&format!("pub mod platform {{\n/// Clocks, the blocks Linux owns, the log, the PCIe endpoint and the SCMI server (`rp1_hal::platform`). proc0, once, before any driver; returns when the link is up.\npub fn init(core: &mut ::rp1_rt::Proc0, token: ::rp1_hal::platform::Token) -> ::core::result::Result<::rp1_hal::platform::System, ::rp1_hal::platform::InitError> {{ token.init(core, {log}, |_| {{}}) }}\n/// [`init`] that runs `before_link` after the log, before the PCIe endpoint: lab only.\n#[doc(hidden)]\npub fn __init_before_link(core: &mut ::rp1_rt::Proc0, token: ::rp1_hal::platform::Token, before_link: impl FnOnce(&::rp1_hal::clocks::Clocks)) -> ::core::result::Result<::rp1_hal::platform::System, ::rp1_hal::platform::InitError> {{ token.init(core, {log}, before_link) }}\n}}\n"));
    }
    if let Some(log) = &d.log {
        s.push_str(&format!("pub mod log {{\n#[unsafe(link_section=\".shared\")] static LOGGER: ::rp1_hal::logging::Logger = unsafe {{ ::rp1_hal::logging::Logger::new() }};\npub fn init(core: &mut ::rp1_rt::Proc0) -> Result<(),::rp1_hal::Error> {{ LOGGER.init(core,::rp1_hal::fugit::HertzU32::from_raw({})) }}\npub fn write(args: ::core::fmt::Arguments<'_>) -> ::core::fmt::Result {{ LOGGER.write(args) }}\npub fn dropped() -> u32 {{ ::rp1_hal::logging::dropped() }}\n}}\n",log.baudrate));
    }
    s.push_str("}\n");
    Ok(s)
}

/// Field type, initializer and bundled pins of an instance's resource.
/// UART, SPI and I²C also emit aliases for the required pin tuple and driver in `rp1`.
/// UART CTS/RTS stay in `pins` (rp1-hal has no flow control).
fn instance_resource(
    source: &mut String,
    name: &str,
    instance: &metadata::Instance,
) -> Result<(String, String, Vec<u8>)> {
    use metadata::Instance;
    let n = board::find_instance(name)
        .ok_or_else(|| format!("unknown instance {name}"))?
        .number;
    let (module, driver, signals) = match instance {
        Instance::Uart(p) => ("uart", "Uart", [("tx", p.tx), ("rx", p.rx)].to_vec()),
        Instance::I2c(p) => ("i2c", "I2c", [("sda", p.sda), ("scl", p.scl)].to_vec()),
        Instance::Spi(p) => (
            "spi",
            "Spi",
            [("sck", p.sck), ("mosi", p.mosi), ("miso", p.miso)].to_vec(),
        ),
        // PWM outputs are optional: keep the declared pins in ProcNPins, with a block token.
        Instance::Pwm(_) => {
            let ty = format!("::rp1_hal::pwm::Token<{n}>");
            return Ok((
                ty.clone(),
                format!("unsafe {{ <{ty}>::new() }}"),
                Vec::new(),
            ));
        }
        Instance::Dma(_) => unreachable!("DMA has a declaration-specific channel set"),
        Instance::Adc {} => {
            let ty = "::rp1_hal::adc::Token";
            return Ok((ty.into(), format!("unsafe {{ {ty}::new() }}"), Vec::new()));
        }
    };
    let pins = signals
        .iter()
        .map(|(role, pin)| pin.ok_or_else(|| format!("{name}.{role} is missing").into()))
        .collect::<Result<Vec<u8>>>()?;
    let pin_types = pins
        .iter()
        .map(|pin| format!("::rp1_hal::gpio::Pin<{pin}>"))
        .collect::<Vec<_>>();
    let alias = format!("{driver}{n}");
    let required = format!("{alias}Pins");
    writeln!(
        source,
        "/// Required pin routes for the declared `{name}`.\n\
         pub type {required} = ({});\n\
         /// Initialized driver for the declared `{name}`.\n\
         pub type {alias} = ::rp1_hal::{module}::{driver}<{n}, {required}>;",
        pin_types.join(", "),
    )?;
    let ty = format!("::rp1_hal::{module}::Parts<{n}, {required}>");
    let mut init = format!(
        "::rp1_hal::{module}::Parts::new(unsafe {{ ::rp1_hal::{module}::Token::new() }}, ::rp1_hal::{module}::Pins::<{n}>::builder()"
    );
    for (role, _) in &signals {
        write!(init, ".{role}(unsafe {{ ::rp1_hal::gpio::Pin::new() }})")?;
    }
    init.push(')');
    Ok((ty, init, pins))
}

fn dma_resource(
    source: &mut String,
    core: usize,
    channels: &[u8],
) -> Result<(String, String, Vec<u8>)> {
    let ty = format!("Proc{core}Dma");
    let initialized = format!("Proc{core}DmaChannels");
    writeln!(
        source,
        "/// Initialized channels issued by the proc{core} DMA declaration.\n\
         #[non_exhaustive]\npub struct {initialized} {{"
    )?;
    for n in channels {
        writeln!(source, "pub ch{n}: ::rp1_hal::dma::Channel<{n}>,")?;
    }
    writeln!(source, "_controller: ::rp1_hal::dma::Dma,\n}}")?;
    writeln!(
        source,
        "/// Controller owned by `proc{core}.dma`.\npub struct {ty}(::rp1_hal::dma::Token);\n\
        impl {ty} {{\n\
            pub fn init(self, clocks: &::rp1_hal::clocks::Clocks) -> ::core::result::Result<\
                {initialized}, ::rp1_hal::InitError<Self, ::rp1_hal::dma::Error>> {{\n\
                #[allow(unused_mut)]\n\
                let mut controller = self.0.init(clocks).map_err(|e| e.map_parts(Self))?;\n\
                Ok({initialized} {{"
    )?;
    for n in channels {
        writeln!(source, "ch{n}: unsafe {{ controller.channel() }},")?;
    }
    writeln!(
        source,
        "_controller: controller,\n}})\n}}\n}}\n\
         impl {initialized} {{\n\
             /// Request graceful disable on every issued channel, then check all eight slots.\n\
             pub fn try_stop_all(&mut self) -> Result<(), ::rp1_hal::dma::Error> {{\n\
                 let result = Ok(());"
    )?;
    for n in channels {
        // Evaluate every recovery even if an earlier channel is still draining.
        writeln!(source, "let result = result.and(self.ch{n}.try_recover());")?;
    }
    writeln!(
        source,
        "        result.and_then(|()| self._controller.__check_idle())\n}}\n\
             /// Request stops, then recheck by timer IRQ until the earlier of end and 999 µs after entry.\n\
             pub fn stop_all_until(&mut self, wait: &mut ::rp1_hal::timer::WaitTimer,\n\
                 end: ::rp1_hal::timer::Deadline) -> Result<(), ::rp1_hal::dma::Error> {{\n\
                 ::rp1_hal::dma::__stop_until(wait, end, || self.try_stop_all())\n}}\n\
             /// Return the declaration's token after all channels prove idle.\n\
             pub fn try_free(mut self) -> Result<{ty}, ::rp1_hal::FreeError<Self, ::rp1_hal::dma::Error>> {{\n\
                 if let Err(error) = self.try_stop_all() {{\n\
                     return Err(::rp1_hal::FreeError::new(self, error));\n\
                 }}\n\
                 // SAFETY: this set owns all issued channels and just proved them idle.\n\
                 match unsafe {{ self._controller.__try_free() }} {{\n\
                     Ok(token) => Ok({ty}(token)),\n\
                     Err(e) => {{\n\
                         let (controller, error) = e.into_parts();\n\
                         Err(::rp1_hal::FreeError::new(Self {{ _controller: controller, ..self }}, error))\n\
                     }},\n\
                 }}\n}}\n\
             /// Wait in thread mode for a bounded stop before returning the token.\n\
             pub fn free_until(mut self, wait: &mut ::rp1_hal::timer::WaitTimer,\n\
                 end: ::rp1_hal::timer::Deadline) -> Result<{ty}, ::rp1_hal::FreeError<Self, ::rp1_hal::dma::Error>> {{\n\
                 match self.stop_all_until(wait, end) {{\n\
                     Ok(()) => self.try_free(),\n\
                     Err(error) => Err(::rp1_hal::FreeError::new(self, error)),\n\
                 }}\n}}\n}}"
    )?;
    Ok((
        ty.clone(),
        format!("{ty}(unsafe {{ ::rp1_hal::dma::Token::new() }})"),
        Vec::new(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn irq_fields(source: &str, core: usize) -> &str {
        source
            .split(&format!("pub struct Proc{core}Irq {{\n"))
            .nth(1)
            .unwrap()
            .split_once('\n')
            .unwrap()
            .1
            .split('}')
            .next()
            .unwrap()
    }

    #[test]
    fn irq_resources_follow_verified_instance_and_alarm_ownership() {
        let cases = [
            ("uart0={tx=14,rx=15}", None),
            ("uart1={tx=0,rx=1}", Some(("UART1", 42))),
            ("uart2={tx=4,rx=5}", Some(("UART2", 43))),
            ("uart3={tx=8,rx=9}", Some(("UART3", 44))),
            ("uart4={tx=12,rx=13}", Some(("UART4", 45))),
            ("i2c0={sda=0,scl=1}", Some(("I2C0", 7))),
            ("i2c1={sda=2,scl=3}", Some(("I2C1", 8))),
            ("i2c2={sda=4,scl=5}", Some(("I2C2", 9))),
            ("i2c3={sda=6,scl=7}", Some(("I2C3", 10))),
            ("spi0={sck=11,mosi=10,miso=9}", Some(("SPI0", 19))),
            ("spi1={sck=21,mosi=20,miso=19}", Some(("SPI1", 20))),
            ("spi3={sck=7,mosi=6,miso=5}", Some(("SPI3", 22))),
            ("spi5={sck=15,mosi=14,miso=13}", None),
            ("pwm0={}", Some(("PWM0", 5))),
            ("pwm1={}", Some(("PWM1", 41))),
            ("adc={}", Some(("ADC_FIFO", 52))),
            ("dma={}", None),
            ("host_irq=true", None),
        ];
        for (declaration, expected) in cases {
            for core in 0..2 {
                let d: metadata::Declaration = toml::from_str(&format!(
                    "schema=2\n[linux]\nkeep={{}}\n[proc{core}]\n{declaration}"
                ))
                .unwrap();
                d.validate().unwrap();
                let source = resources(&d, &[], &d.shm_layout().unwrap()).unwrap();
                if let Some((name, irq)) = expected {
                    let (kind, id) = metadata::peripheral_id(
                        &declaration.split_once('=').unwrap().0.to_ascii_uppercase(),
                    )
                    .unwrap();
                    assert_eq!(kind, rp1_proto::note::ownership_kind::BLOCK);
                    assert_eq!(
                        irq_fields(&source, core),
                        format!("pub {name}: proc{core}_irq::{name},\n")
                    );
                    assert!(source.contains(&format!(
                        "pub type {name} = ::rp1_rt::interrupt::Binding<::rp1_rt::interrupt::BlockIrq<{id}>, {irq}, {core}>;"
                    )));
                } else {
                    assert_eq!(irq_fields(&source, core), "");
                }
                assert_eq!(irq_fields(&source, 1 - core), "");
            }
        }
        for core in 0..2 {
            for alarm in 0..4 {
                let d: metadata::Declaration =
                    toml::from_str(&format!("schema=2\n[proc{core}]\ntimer=['ALARM{alarm}']"))
                        .unwrap();
                d.validate().unwrap();
                let source = resources(&d, &[], &d.shm_layout().unwrap()).unwrap();
                assert_eq!(
                    irq_fields(&source, core),
                    format!("pub TIMER_{alarm}: proc{core}_irq::TIMER_{alarm},\n")
                );
                assert!(source.contains(&format!(
                    "pub type TIMER_{alarm} = ::rp1_rt::interrupt::Binding<::rp1_rt::interrupt::AlarmIrq<{alarm}>, {}, {core}>;",
                    26 + alarm
                )));
                assert_eq!(irq_fields(&source, 1 - core), "");
            }
        }
    }

    #[test]
    fn wait_timer_keeps_alarm_ownership_but_reserves_its_handler() {
        let d: metadata::Declaration =
            toml::from_str("schema=2\n[proc1]\nwait_timer='ALARM2'").unwrap();
        let raw: metadata::Declaration =
            toml::from_str("schema=2\n[proc1]\ntimer=['ALARM2']").unwrap();
        let generated = generate("wait", "0.1", &d, [0; 32], [0; 20]).unwrap();
        let raw = generate("wait", "0.1", &raw, [0; 32], [0; 20]).unwrap();
        assert_eq!(generated.note, raw.note);
        assert_eq!(generated.dts, raw.dts);
        let source = resources(&d, &[], &d.shm_layout().unwrap()).unwrap();
        assert!(source.contains("export_name = \"__rp1_p1_TIMER_2\""));
        assert!(source.contains("WaitTimer::__bind::<2, 28, 1>"));
        assert!(source.contains("pub wait_timer: ::rp1_hal::timer::WaitTimer"));
        assert!(!source.contains("pub TIMER_ALARM2:"));
        assert!(!source.contains("pub TIMER_2:"));
        // All vectors have declaration markers. The wait vector
        // has neither a public binding nor permission for a user handler.
        assert!(!source.contains("pub type TIMER_2 ="));
        assert!(!source.contains("impl sealed::Admitted for TIMER_2"));

        for declaration in [
            "[proc0]\nwait_timer='ALARM2'\ntimer=['ALARM2']",
            "[proc0]\nwait_timer='ALARM2'\n[proc1]\ntimer=['ALARM2']",
            "[proc0]\nwait_timer='ALARM3'\n[proc1]\nhost_irq=true",
            "[proc0]\nwait_timer='ALARM02'",
        ] {
            let d: metadata::Declaration =
                toml::from_str(&format!("schema=2\n{declaration}")).unwrap();
            assert!(d.validate().is_err(), "{declaration}");
        }
    }

    #[test]
    fn gpio_irq_resources_are_separate_from_pin_state_and_core_local() {
        let d: metadata::Declaration = toml::from_str(
            "schema=2\n[proc0]\ngpio=[22]\ngpio_irq=[22]\n[proc1]\ngpio=[26,27]\ngpio_irq=[26,27]",
        )
        .unwrap();
        d.validate().unwrap();
        let source = resources(&d, &[], &d.shm_layout().unwrap()).unwrap();
        for (core, pins) in [(0, &[22][..]), (1, &[26, 27][..])] {
            let mut expected = format!("pub IO_BANK0: proc{core}_irq::IO_BANK0,\n");
            for pin in pins {
                expected.push_str(&format!(
                    "pub gpio{pin}: ::rp1_hal::gpio::PinIrq<{pin}, {core}>,\n"
                ));
                assert!(source.contains(&format!("pub gpio{pin}: ::rp1_hal::gpio::Pin<{pin}>,")));
                assert!(source.contains(&format!(
                    "gpio{pin}: unsafe {{ ::rp1_hal::gpio::PinIrq::new() }},"
                )));
            }
            assert_eq!(irq_fields(&source, core), expected);
        }
    }

    #[test]
    fn doorbell_adds_one_bank_line_and_no_pin_resource() {
        let d: metadata::Declaration =
            toml::from_str("schema=2\n[proc1]\ngpio=[22]\ngpio_irq=[22]\ndoorbell=27").unwrap();
        d.validate().unwrap();
        let source = resources(&d, &[], &d.shm_layout().unwrap()).unwrap();
        assert_eq!(
            irq_fields(&source, 1),
            "pub IO_BANK0: proc1_irq::IO_BANK0,\n\
             pub gpio22: ::rp1_hal::gpio::PinIrq<22, 1>,\n"
        );
        assert!(!source.contains("gpio27"), "{source}");
        let d: metadata::Declaration = toml::from_str("schema=2\n[proc0]\ndoorbell=26").unwrap();
        d.validate().unwrap();
        let source = resources(&d, &[], &d.shm_layout().unwrap()).unwrap();
        assert_eq!(
            irq_fields(&source, 0),
            "pub IO_BANK0: proc0_irq::IO_BANK0,\n"
        );
        assert!(!source.contains("gpio26"), "{source}");
    }

    #[test]
    fn instances_are_handed_out_as_typed_parts_and_pins_keep_only_the_rest() {
        let d: metadata::Declaration = toml::from_str(
            r#"
schema = 2
[proc0]
gpio = [6, 22]
timer = ["ALARM0"]
i2c1 = { sda = 2, scl = 3 }
uart2 = { tx = 4, rx = 5 }
[proc1]
gpio = [8, 27]
adc = {}
spi0 = { sck = 11, mosi = 10, miso = 9 }
"#,
        )
        .unwrap();
        d.validate().unwrap();
        let s = resources(&d, &[], &d.shm_layout().unwrap()).unwrap();
        for field in [
            "pub TIMER_ALARM0: ::rp1_hal::timer::Alarm<0>,",
            "pub type I2c1Pins = (::rp1_hal::gpio::Pin<2>, ::rp1_hal::gpio::Pin<3>);",
            "pub type Uart2Pins = (::rp1_hal::gpio::Pin<4>, ::rp1_hal::gpio::Pin<5>);",
            "pub type Spi0Pins = (::rp1_hal::gpio::Pin<11>, ::rp1_hal::gpio::Pin<10>, ::rp1_hal::gpio::Pin<9>);",
            "pub type I2c1 = ::rp1_hal::i2c::I2c<1, I2c1Pins>;",
            "pub type Uart2 = ::rp1_hal::uart::Uart<2, Uart2Pins>;",
            "pub type Spi0 = ::rp1_hal::spi::Spi<0, Spi0Pins>;",
            "pub i2c1: ::rp1_hal::i2c::Parts<1, I2c1Pins>,",
            "pub uart2: ::rp1_hal::uart::Parts<2, Uart2Pins>,",
            "pub adc: ::rp1_hal::adc::Token,",
            "pub spi0: ::rp1_hal::spi::Parts<0, Spi0Pins>,",
            "uart2: ::rp1_hal::uart::Parts::new(unsafe { ::rp1_hal::uart::Token::new() }, \
             ::rp1_hal::uart::Pins::<2>::builder().tx(unsafe { ::rp1_hal::gpio::Pin::new() }).rx(unsafe { ::rp1_hal::gpio::Pin::new() })),",
        ] {
            assert!(s.contains(field), "{field}\n{s}");
        }
        let pins = |core: &str| -> Vec<&str> {
            let start = s.find(&format!("pub struct {core}Pins {{")).unwrap();
            let body = &s[start..start + s[start..].find('}').unwrap()];
            body.lines()
                .skip(1)
                .filter(|line| line.starts_with("pub "))
                .collect()
        };
        // A GPIO next to the instance's pins stays a plain pin.
        assert_eq!(
            pins("Proc0"),
            [
                "pub gpio6: ::rp1_hal::gpio::Pin<6>,",
                "pub gpio22: ::rp1_hal::gpio::Pin<22>,"
            ]
        );
        assert_eq!(
            pins("Proc1"),
            [
                "pub gpio8: ::rp1_hal::gpio::Pin<8>,",
                "pub gpio27: ::rp1_hal::gpio::Pin<27>,"
            ]
        );
    }
}

#[cfg(test)]
mod pwm_tests {
    #[test]
    fn resources_issue_block_tokens_and_retain_optional_pins() {
        let d =
            toml::from_str("schema=2\n[proc0]\npwm0={channel0=12,channel2=18}\n[proc1]\npwm1={}")
                .unwrap();
        let s = super::resources(&d, &[], &d.shm_layout().unwrap()).unwrap();
        for field in [
            "pub pwm0: ::rp1_hal::pwm::Token<0>",
            "pub pwm1: ::rp1_hal::pwm::Token<1>",
            "pub gpio12: ::rp1_hal::gpio::Pin<12>",
            "pub gpio18: ::rp1_hal::gpio::Pin<18>",
        ] {
            assert!(s.contains(field), "{field}");
        }
    }
}
