use std::{collections::BTreeMap, fmt::Write as _, fs, path::Path, process::Command};

use anyhow::{Context, Result, bail};

use crate::model::*;

pub fn generate(db: &Database, pac_src: &Path) -> Result<()> {
    let pac_root = pac_src
        .parent()
        .context("PAC src directory has no parent")?;
    let generated = pac_src.join("generated");
    if generated.exists() {
        fs::remove_dir_all(&generated)?;
    }
    fs::create_dir_all(&generated)?;

    let mut definitions: Vec<_> = db
        .peripherals
        .iter()
        .filter(|definition| !definition.registers.is_empty())
        .collect();
    definitions.sort_by(|a, b| a.peripheral.module.cmp(&b.peripheral.module));

    for definition in &definitions {
        let module = &definition.peripheral.module;
        fs::write(
            generated.join(format!("{module}.rs")),
            peripheral_module(definition),
        )?;
    }
    let modules = definitions
        .iter()
        .map(|definition| format!("pub mod {};\n", definition.peripheral.module))
        .collect::<String>();
    fs::write(generated.join("mod.rs"), modules)?;
    fs::write(pac_src.join("peripherals.rs"), peripherals(db))?;
    fs::write(pac_src.join("interrupts.rs"), interrupts(db))?;
    fs::write(pac_src.join("metadata.rs"), metadata(db))?;
    fs::write(pac_root.join("device.x"), device_x(db))?;
    format_generated(pac_src)?;
    Ok(())
}

fn format_generated(pac_src: &Path) -> Result<()> {
    let mut files = vec![
        pac_src.join("peripherals.rs"),
        pac_src.join("interrupts.rs"),
        pac_src.join("metadata.rs"),
        pac_src.join("generated/mod.rs"),
    ];
    let entries = fs::read_dir(pac_src.join("generated"))?;
    for entry in entries {
        let path = entry?.path();
        if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
            files.push(path);
        }
    }
    files.sort();
    files.dedup();
    let status = Command::new("rustfmt")
        .args(["--edition", "2024"])
        .args(&files)
        .status()
        .context("running rustfmt on generated PAC files")?;
    if !status.success() {
        bail!("rustfmt failed for generated PAC files");
    }
    Ok(())
}

fn source_label(definition: &PeripheralDefinition) -> String {
    let file = definition
        .source
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("unknown.toml");
    if definition
        .source
        .parent()
        .and_then(Path::file_name)
        .and_then(|name| name.to_str())
        == Some("peripherals")
    {
        format!("rp1-regdb/data/peripherals/{file}")
    } else {
        format!("rp1-regdb/data/{file}")
    }
}

fn rustdoc(out: &mut String, indent: &str, text: &str) {
    for line in text.lines().map(str::trim).filter(|line| !line.is_empty()) {
        let line = line.replace('[', "\\[").replace(']', "\\]");
        writeln!(out, "{indent}/// {line}").unwrap();
    }
}

fn peripheral_docs(out: &mut String, peripheral: &Peripheral, indent: &str) {
    if let Some(note) = &peripheral.note {
        rustdoc(out, indent, note);
    }
    if let Some(external_ip) = &peripheral.external_ip {
        rustdoc(
            out,
            indent,
            &format!(
                "External IP instance delegated to `{external_ip}`; only RP1 instance metadata is provided."
            ),
        );
    }
    if let Some(view) = &peripheral.atomic_view {
        rustdoc(
            out,
            indent,
            &format!(
                "Atomic view: {:?} alias windows; ownership remains with this logical peripheral.",
                view.kind
            ),
        );
    }
}

fn register_docs(out: &mut String, register: &Register, indent: &str) {
    if let Some(note) = &register.note {
        rustdoc(out, indent, note);
    }
    if let Some(initialization) = &register.boot_rom_initialization {
        rustdoc(
            out,
            indent,
            &format!(
                "Boot ROM initialization value {:#x} was {:?} in scope {:?}; this is not the hardware reset value.",
                initialization.value,
                initialization.observation.result,
                initialization
                    .observation
                    .scope
                    .as_deref()
                    .unwrap_or("unspecified")
            ),
        );
    }
    rustdoc(
        out,
        indent,
        &format!(
            "Hardware access: read {:?}; write {:?}.",
            register.hardware_read, register.hardware_write
        ),
    );
    match register.hardware_write {
        WriteBehavior::WriteOneToClear => rustdoc(
            out,
            indent,
            "Write-one-to-clear register; general write and modify APIs are not generated.",
        ),
        WriteBehavior::WriteOneToSet => rustdoc(
            out,
            indent,
            "Write-one-to-set register; general write and modify APIs are not generated.",
        ),
        WriteBehavior::WriteOneToToggle => rustdoc(
            out,
            indent,
            "Write-one-to-toggle register; general write and modify APIs are not generated.",
        ),
        WriteBehavior::Command | WriteBehavior::WriteOnly => rustdoc(
            out,
            indent,
            "Command write; ordinary read, write, and modify APIs are not generated.",
        ),
        WriteBehavior::Fifo => rustdoc(
            out,
            indent,
            "FIFO/data port; only dedicated FIFO operations are generated.",
        ),
        WriteBehavior::StateMachineOnly => rustdoc(
            out,
            indent,
            "State-machine-only register; unrestricted writers are intentionally hidden.",
        ),
        _ => {}
    }
    if register.hardware_write != WriteBehavior::None && !register.api.has_safe_write() {
        rustdoc(
            out,
            indent,
            "Write API is not exposed because the verified operation semantics are insufficient.",
        );
    }
    if register.repeat_count > 1 {
        rustdoc(
            out,
            indent,
            &format!(
                "Array layout: {} physical entries, {} public entries, stride {:#x} bytes.",
                register.repeat_count,
                register.public_count.unwrap_or(register.repeat_count),
                register.repeat_stride
            ),
        );
    }
}

fn field_docs(
    out: &mut String,
    peripheral: &Peripheral,
    register: &Register,
    field: &Field,
    indent: &str,
) {
    if let Some(note) = &field.note {
        rustdoc(out, indent, note);
    }
    let behavior = field.effective_write_behavior(register);
    rustdoc(out, indent, &format!("Field write behavior: {behavior:?}."));
    let origin = field
        .origin
        .or(register.origin)
        .unwrap_or(peripheral.default_origin);
    if field.api_write == ApiExposure::None && behavior != FieldWriteBehavior::ReadOnly {
        let reason = if origin == Origin::Inferred {
            "Typed write API is not exposed because this field is inferred."
        } else if !field.effective_verified_write(register).permits_write() {
            "Typed write API is not exposed because write behavior is not hardware/official verified."
        } else {
            "Typed write API is not exposed by register policy."
        };
        rustdoc(out, indent, reason);
    } else if behavior == FieldWriteBehavior::ReadOnly {
        rustdoc(out, indent, "Read-only field; no writer is generated.");
    }
}

fn padded_array_group<'a>(register: &'a Register, registers: &[&'a Register]) -> Vec<&'a Register> {
    let byte_width = u64::from(register.width / 8);
    if register.repeat_count == 1 || register.repeat_stride <= byte_width {
        return vec![register];
    }
    let bucket = register.offset / register.repeat_stride;
    let mut group: Vec<_> = registers
        .iter()
        .copied()
        .filter(|candidate| {
            candidate.repeat_count == register.repeat_count
                && candidate.repeat_stride == register.repeat_stride
                && candidate.width == register.width
                && candidate.offset / candidate.repeat_stride == bucket
        })
        .collect();
    group.sort_by_key(|candidate| candidate.offset);
    group
}

fn padded_element_name(group: &[&Register]) -> String {
    format!("{}ArrayElement", rust_type(&group[0].name))
}

fn peripheral_module(definition: &PeripheralDefinition) -> String {
    let p = &definition.peripheral;
    let mut out = String::new();
    writeln!(
        out,
        "// Generated from {}.\n// Do not edit manually.\n// Regenerate with: cargo run -p rp1-regdb -- generate\n",
        source_label(definition)
    )
    .unwrap();
    writeln!(out, "use crate::Reg;\n").unwrap();

    let mut registers: Vec<_> = definition.registers.iter().collect();
    registers.sort_by_key(|register| register.offset);
    for register in &registers {
        let byte_width = u64::from(register.width / 8);
        if register.repeat_count <= 1 || register.repeat_stride <= byte_width {
            continue;
        }
        let group = padded_array_group(register, &registers);
        if group[0].id != register.id {
            continue;
        }
        let element = padded_element_name(&group);
        writeln!(out, "#[repr(C)]\nstruct {element} {{").unwrap();
        let base = group[0].offset;
        let mut element_cursor = base;
        let mut element_reserved = 0usize;
        for member in &group {
            if member.offset > element_cursor {
                writeln!(
                    out,
                    "    _reserved{element_reserved}: [u8; {}],",
                    member.offset - element_cursor
                )
                .unwrap();
                element_reserved += 1;
            }
            writeln!(
                out,
                "    {}: {},",
                ident(&member.id),
                rust_type(&member.name)
            )
            .unwrap();
            element_cursor = member.offset + u64::from(member.width / 8);
        }
        let element_end = base + register.repeat_stride;
        if element_cursor < element_end {
            writeln!(
                out,
                "    _reserved{element_reserved}: [u8; {}],",
                element_end - element_cursor
            )
            .unwrap();
        }
        writeln!(out, "}}\n").unwrap();
    }
    peripheral_docs(&mut out, p, "");
    writeln!(out, "#[repr(C)]\npub struct RegisterBlock {{").unwrap();

    let mut cursor = 0u64;
    let mut reserved = 0usize;
    for register in &registers {
        let byte_width = u64::from(register.width / 8);
        let padded = register.repeat_count > 1 && register.repeat_stride > byte_width;
        let group = padded_array_group(register, &registers);
        if padded && group[0].id != register.id {
            continue;
        }
        if register.offset > cursor {
            writeln!(
                out,
                "    _reserved{reserved}: [u8; {}],",
                register.offset - cursor
            )
            .unwrap();
            reserved += 1;
        }
        let id = ident(&register.id);
        let ty = rust_type(&register.name);
        if register.repeat_count == 1 {
            writeln!(out, "    {id}: {ty},").unwrap();
        } else if padded {
            let element = padded_element_name(&group);
            writeln!(out, "    {id}: [{element}; {}],", register.repeat_count).unwrap();
        } else {
            writeln!(out, "    {id}: [{ty}; {}],", register.repeat_count).unwrap();
        }
        cursor = if padded {
            register.offset + u64::from(register.repeat_count) * register.repeat_stride
        } else {
            register.offset
                + u64::from(register.repeat_count - 1) * register.repeat_stride
                + byte_width
        };
    }
    writeln!(out, "}}\n").unwrap();

    writeln!(out, "impl RegisterBlock {{").unwrap();
    for register in &registers {
        if !register_visible(register) {
            continue;
        }
        let id = ident(&register.id);
        let ty = rust_type(&register.name);
        let visibility = register_visibility(register);
        register_docs(&mut out, register, "    ");
        if register.repeat_count == 1 {
            writeln!(
                out,
                "    #[inline(always)]\n    {visibility} const fn {id}(&self) -> &{ty} {{ &self.{id} }}"
            )
            .unwrap();
        } else {
            let public_count = register.public_count.unwrap_or(register.repeat_count);
            if register.repeat_stride > u64::from(register.width / 8) {
                let group = padded_array_group(register, &registers);
                let owner = ident(&group[0].id);
                writeln!(out, "    #[inline(always)]\n    {visibility} fn {id}(&self, index: usize) -> &{ty} {{ assert!(index < {public_count}); &self.{owner}[index].{id} }}").unwrap();
            } else {
                writeln!(out, "    #[inline(always)]\n    {visibility} fn {id}(&self, index: usize) -> &{ty} {{ assert!(index < {public_count}); &self.{id}[index] }}").unwrap();
                writeln!(out, "    #[inline(always)]\n    {visibility} fn {id}_array(&self) -> &[{ty}] {{ &self.{id}[..{public_count}] }}").unwrap();
            }
        }
    }
    writeln!(out, "}}\n").unwrap();

    for register in &registers {
        register_module(&mut out, p, register);
    }

    writeln!(out, "#[cfg(test)]\nmod layout_tests {{").unwrap();
    writeln!(out, "    use super::RegisterBlock;").unwrap();
    for register in &registers {
        let id = ident(&register.id);
        if register.repeat_count > 1 && register.repeat_stride > u64::from(register.width / 8) {
            let group = padded_array_group(register, &registers);
            let owner = ident(&group[0].id);
            let element = padded_element_name(&group);
            writeln!(out, "    #[test]\n    fn {id}_offset() {{ assert_eq!(core::mem::offset_of!(RegisterBlock, {owner}) + core::mem::offset_of!(super::{element}, {id}), {:#x}); }}", register.offset).unwrap();
            writeln!(out, "    #[test]\n    fn {id}_stride() {{ assert_eq!(core::mem::size_of::<super::{element}>(), {:#x}); }}", register.repeat_stride).unwrap();
        } else {
            writeln!(out, "    #[test]\n    fn {id}_offset() {{ assert_eq!(core::mem::offset_of!(RegisterBlock, {id}), {:#x}); }}", register.offset).unwrap();
        }
    }
    writeln!(out, "    #[test]\n    fn register_block_size() {{ assert_eq!(core::mem::size_of::<RegisterBlock>(), {cursor:#x}); }}").unwrap();
    writeln!(out, "}}\n").unwrap();
    out
}

fn register_module(out: &mut String, peripheral: &Peripheral, register: &Register) {
    let module = ident(&register.id);
    let spec = format!("{}_SPEC", register.name.to_ascii_uppercase());
    let ty = rust_type(&register.name);
    register_docs(out, register, "");
    writeln!(out, "pub type {ty} = Reg<{module}::{spec}>;").unwrap();
    register_docs(out, register, "");
    writeln!(out, "pub mod {module} {{").unwrap();
    writeln!(out, "    pub struct {spec};").unwrap();
    writeln!(
        out,
        "    impl crate::RegisterSpec for {spec} {{ type Ux = u32; }}"
    )
    .unwrap();

    if register.api.read == ApiExposure::Public {
        writeln!(out, "    impl crate::Readable for {spec} {{}}").unwrap();
        writeln!(out, "    pub type R = crate::R<{spec}>;").unwrap();
    }
    if register.api.modify == ApiExposure::Public {
        writeln!(out, "    impl crate::Modifiable for {spec} {{}}").unwrap();
    }
    if register.api.w1c == ApiExposure::Public {
        writeln!(out, "    impl crate::WriteOneToClear for {spec} {{}}").unwrap();
    }
    if register.api.w1s == ApiExposure::Public {
        writeln!(out, "    impl crate::WriteOneToSet for {spec} {{}}").unwrap();
    }
    if register.api.w1t == ApiExposure::Public {
        writeln!(out, "    impl crate::WriteOneToToggle for {spec} {{}}").unwrap();
    }
    if register.api.command == ApiExposure::Public {
        writeln!(out, "    impl crate::CommandWritable for {spec} {{}}").unwrap();
    }
    if register.api.fifo_push == ApiExposure::Public {
        writeln!(out, "    impl crate::FifoPush for {spec} {{}}").unwrap();
    }
    if register.api.fifo_pop == ApiExposure::Public {
        writeln!(out, "    impl crate::FifoPop for {spec} {{}}").unwrap();
    }
    if register.api.raw_write == RawWriteExposure::Unsafe {
        writeln!(out, "    impl crate::UnsafeWritable for {spec} {{}}").unwrap();
    }
    if let Some(reset) = register.reset_value {
        writeln!(
            out,
            "    impl crate::ResetValue for {spec} {{ const RESET_VALUE: u32 = {reset:#x}; }}"
        )
        .unwrap();
        if register.api.reset == ApiExposure::Public {
            writeln!(out, "    impl crate::Resettable for {spec} {{}}").unwrap();
        }
    }

    generate_atomic_traits(out, peripheral, register, &spec);

    let has_plain_writer = register.api.modify == ApiExposure::Public;
    let has_atomic_set = register.atomic.set == AtomicExposure::Typed;
    let has_atomic_clear = register.atomic.clear == AtomicExposure::Typed;
    let has_atomic_xor = register.atomic.xor == AtomicExposure::Typed;

    if has_plain_writer {
        writeln!(out, "    pub type W = crate::W<{spec}>;").unwrap();
    }
    if register.api.w1c == ApiExposure::Public {
        writeln!(out, "    pub type ClearW = crate::ClearW<{spec}>;").unwrap();
    }
    if register.api.w1s == ApiExposure::Public {
        writeln!(out, "    pub type SetW = crate::SetW<{spec}>;").unwrap();
    }
    if register.api.w1t == ApiExposure::Public {
        writeln!(out, "    pub type ToggleW = crate::ToggleW<{spec}>;").unwrap();
    }
    if register.api.command == ApiExposure::Public {
        writeln!(out, "    pub type CommandW = crate::CommandW<{spec}>;").unwrap();
    }
    if has_atomic_set {
        writeln!(out, "    pub type AtomicSetW = crate::AtomicSetW<{spec}>;").unwrap();
    }
    if has_atomic_clear {
        writeln!(
            out,
            "    pub type AtomicClearW = crate::AtomicClearW<{spec}>;"
        )
        .unwrap();
    }
    if has_atomic_xor {
        writeln!(out, "    pub type AtomicXorW = crate::AtomicXorW<{spec}>;").unwrap();
    }

    for field in &register.fields {
        field_module(out, peripheral, register, field, &spec, has_plain_writer);
    }
    generate_reader_impl(out, register);
    if has_plain_writer {
        generate_writer_impl(out, register, "W", FieldWriteBehavior::Plain);
    }
    if register.api.w1c == ApiExposure::Public {
        generate_writer_impl(out, register, "ClearW", FieldWriteBehavior::WriteOneToClear);
    }
    if register.api.w1s == ApiExposure::Public {
        generate_writer_impl(out, register, "SetW", FieldWriteBehavior::WriteOneToSet);
    }
    if register.api.w1t == ApiExposure::Public {
        generate_writer_impl(
            out,
            register,
            "ToggleW",
            FieldWriteBehavior::WriteOneToToggle,
        );
    }
    if register.api.command == ApiExposure::Public {
        generate_command_writer_impl(out, register);
    }
    if has_atomic_set {
        generate_atomic_writer_impl(out, peripheral, register, "AtomicSetW", "ASW");
    }
    if has_atomic_clear {
        generate_atomic_writer_impl(out, peripheral, register, "AtomicClearW", "ACW");
    }
    if has_atomic_xor {
        generate_atomic_writer_impl(out, peripheral, register, "AtomicXorW", "AXW");
    }
    writeln!(out, "}}\n").unwrap();
}

fn generate_atomic_traits(
    out: &mut String,
    peripheral: &Peripheral,
    register: &Register,
    spec: &str,
) {
    for (operation, exposure, typed_trait, unsafe_trait) in [
        (
            CustomViewKind::Set,
            register.atomic.set,
            "TypedAtomicSet",
            "UnsafeAtomicSet",
        ),
        (
            CustomViewKind::Clear,
            register.atomic.clear,
            "TypedAtomicClear",
            "UnsafeAtomicClear",
        ),
        (
            CustomViewKind::Xor,
            register.atomic.xor,
            "TypedAtomicXor",
            "UnsafeAtomicXor",
        ),
    ] {
        if exposure == AtomicExposure::None {
            continue;
        }
        let offset = atomic_offset(peripheral, register, operation);
        if exposure == AtomicExposure::Typed {
            writeln!(
                out,
                "    impl crate::{typed_trait} for {spec} {{ const OFFSET: usize = {offset:#x}; }}"
            )
            .unwrap();
        }
        writeln!(
            out,
            "    impl crate::{unsafe_trait} for {spec} {{ const OFFSET: usize = {offset:#x}; }}"
        )
        .unwrap();
    }
}

fn atomic_offset(peripheral: &Peripheral, register: &Register, operation: CustomViewKind) -> u64 {
    match register.atomic.kind {
        AtomicKind::Rp1Native => {
            let view = peripheral
                .atomic_view
                .as_ref()
                .expect("validated native atomic view");
            match operation {
                CustomViewKind::Set => view.set_offset,
                CustomViewKind::Clear => view.clear_offset,
                CustomViewKind::Xor => view.xor_offset,
            }
        }
        AtomicKind::Custom => {
            let reference = match operation {
                CustomViewKind::Set => register.atomic.set_view.as_deref(),
                CustomViewKind::Clear => register.atomic.clear_view.as_deref(),
                CustomViewKind::Xor => register.atomic.xor_view.as_deref(),
            }
            .expect("validated custom atomic view reference");
            peripheral
                .custom_views
                .iter()
                .find(|view| view.id == reference && view.kind == operation)
                .expect("validated custom atomic view")
                .offset
        }
        AtomicKind::None => unreachable!("validated atomic exposure without an atomic kind"),
    }
}

fn field_module(
    out: &mut String,
    peripheral: &Peripheral,
    register: &Register,
    field: &Field,
    spec: &str,
    has_plain_writer: bool,
) {
    let field_module = ident(&field.id);
    let behavior = field.effective_write_behavior(register);
    field_docs(out, peripheral, register, field, "    ");
    writeln!(out, "    pub mod {field_module} {{").unwrap();
    if field.api_read != ApiExposure::None {
        if field.width == 1 {
            writeln!(out, "        pub type R = crate::BitReader;").unwrap();
        } else {
            writeln!(out, "        pub type R = crate::FieldReader<u32>;").unwrap();
        }
    }
    if field.api_write != ApiExposure::None {
        if has_plain_writer && behavior == FieldWriteBehavior::Plain {
            writer_alias(out, "W", "W", field, spec);
        }
        if typed_atomic_field(peripheral, register, field) {
            if register.atomic.set == AtomicExposure::Typed {
                writer_alias(out, "AtomicSetW", "ASW", field, spec);
            }
            if register.atomic.clear == AtomicExposure::Typed {
                writer_alias(out, "AtomicClearW", "ACW", field, spec);
            }
            if register.atomic.xor == AtomicExposure::Typed {
                writer_alias(out, "AtomicXorW", "AXW", field, spec);
            }
        }
        match behavior {
            FieldWriteBehavior::WriteOneToClear if register.api.w1c == ApiExposure::Public => {
                writer_alias(out, "ClearW", "CW", field, spec)
            }
            FieldWriteBehavior::WriteOneToSet if register.api.w1s == ApiExposure::Public => {
                writer_alias(out, "SetW", "SW", field, spec)
            }
            FieldWriteBehavior::WriteOneToToggle if register.api.w1t == ApiExposure::Public => {
                writer_alias(out, "ToggleW", "TW", field, spec)
            }
            FieldWriteBehavior::Command | FieldWriteBehavior::SelfClearing
                if register.api.command == ApiExposure::Public =>
            {
                writer_alias(out, "CommandW", "CMDW", field, spec)
            }
            _ => {}
        }
    }
    for value in &field.enum_values {
        writeln!(
            out,
            "        pub const {}: u32 = {:#x};",
            value.name.to_ascii_uppercase().replace(['-', '.'], "_"),
            value.value
        )
        .unwrap();
    }
    writeln!(out, "    }}").unwrap();
}

fn writer_alias(out: &mut String, writer: &str, alias: &str, field: &Field, spec: &str) {
    let (field_writer, bit_writer) = match writer {
        "ClearW" => ("ClearFieldWriter", "ClearBitWriter"),
        "SetW" => ("SetFieldWriter", "SetBitWriter"),
        "ToggleW" => ("ToggleFieldWriter", "ToggleBitWriter"),
        "CommandW" => ("CommandFieldWriter", "CommandBitWriter"),
        "AtomicSetW" => ("AtomicSetFieldWriter", "AtomicSetBitWriter"),
        "AtomicClearW" => ("AtomicClearFieldWriter", "AtomicClearBitWriter"),
        "AtomicXorW" => ("AtomicXorFieldWriter", "AtomicXorBitWriter"),
        _ => ("FieldWriter", "BitWriter"),
    };
    if field.width == 1 {
        writeln!(
            out,
            "        pub type {alias}<'a> = crate::{bit_writer}<'a, super::{writer}, {}>;",
            field.lsb
        )
        .unwrap();
    } else {
        writeln!(
            out,
            "        pub type {alias}<'a> = crate::{field_writer}<'a, super::{writer}, {}, {}>;",
            field.lsb, field.width
        )
        .unwrap();
    }
    let _ = spec;
}

fn generate_reader_impl(out: &mut String, register: &Register) {
    if register.api.read != ApiExposure::Public
        || !register
            .fields
            .iter()
            .any(|field| field.api_read != ApiExposure::None)
    {
        return;
    }
    writeln!(out, "    impl R {{").unwrap();
    for field in &register.fields {
        if field.api_read == ApiExposure::None {
            continue;
        }
        let field_name = ident(&field.id);
        let visibility = visibility(field.api_read);
        if field.width == 1 {
            writeln!(out, "        #[inline(always)]\n        {visibility} fn {field_name}(&self) -> {field_name}::R {{ crate::BitReader::new(((self.bits() >> {}) & 1) != 0) }}", field.lsb).unwrap();
        } else {
            let mask = if field.width == 32 {
                u32::MAX
            } else {
                (1u32 << field.width) - 1
            };
            writeln!(out, "        #[inline(always)]\n        {visibility} fn {field_name}(&self) -> {field_name}::R {{ crate::FieldReader::new((self.bits() >> {}) & {mask:#x}) }}", field.lsb).unwrap();
        }
    }
    writeln!(out, "    }}").unwrap();
}

fn generate_writer_impl(
    out: &mut String,
    register: &Register,
    writer: &str,
    behavior: FieldWriteBehavior,
) {
    let alias = match writer {
        "W" => "W",
        "ClearW" => "CW",
        "SetW" => "SW",
        "ToggleW" => "TW",
        _ => unreachable!(),
    };
    let fields: Vec<_> = register
        .fields
        .iter()
        .filter(|field| {
            field.api_write != ApiExposure::None
                && field.effective_write_behavior(register) == behavior
        })
        .collect();
    if fields.is_empty() {
        return;
    }
    writeln!(out, "    impl {writer} {{").unwrap();
    for field in fields {
        let name = ident(&field.id);
        let visibility = visibility(field.api_write);
        writeln!(out, "        #[inline(always)]\n        {visibility} fn {name}(&mut self) -> {name}::{alias}<'_> {{ {name}::{alias}::new(self) }}").unwrap();
    }
    writeln!(out, "    }}").unwrap();
}

fn typed_atomic_field(peripheral: &Peripheral, register: &Register, field: &Field) -> bool {
    field.api_write == ApiExposure::Public
        && field.effective_write_behavior(register) == FieldWriteBehavior::Plain
        && field.effective_verified_write(register).permits_write()
        && field
            .origin
            .or(register.origin)
            .unwrap_or(peripheral.default_origin)
            != Origin::Inferred
}

fn generate_atomic_writer_impl(
    out: &mut String,
    peripheral: &Peripheral,
    register: &Register,
    writer: &str,
    alias: &str,
) {
    let fields: Vec<_> = register
        .fields
        .iter()
        .filter(|field| typed_atomic_field(peripheral, register, field))
        .collect();
    if fields.is_empty() {
        return;
    }
    writeln!(out, "    impl {writer} {{").unwrap();
    for field in fields {
        let name = ident(&field.id);
        field_docs(out, peripheral, register, field, "        ");
        writeln!(out, "        #[inline(always)]\n        pub fn {name}(&mut self) -> {name}::{alias}<'_> {{ {name}::{alias}::new(self) }}").unwrap();
    }
    writeln!(out, "    }}").unwrap();
}

fn generate_command_writer_impl(out: &mut String, register: &Register) {
    let fields: Vec<_> = register
        .fields
        .iter()
        .filter(|field| {
            field.api_write != ApiExposure::None
                && matches!(
                    field.effective_write_behavior(register),
                    FieldWriteBehavior::Command | FieldWriteBehavior::SelfClearing
                )
        })
        .collect();
    if fields.is_empty() {
        return;
    }
    writeln!(out, "    impl CommandW {{").unwrap();
    for field in fields {
        let name = ident(&field.id);
        let visibility = visibility(field.api_write);
        writeln!(out, "        #[inline(always)]\n        {visibility} fn {name}(&mut self) -> {name}::CMDW<'_> {{ {name}::CMDW::new(self) }}").unwrap();
    }
    writeln!(out, "    }}").unwrap();
}

fn peripherals(db: &Database) -> String {
    let mut definitions: Vec<_> = db.peripherals.iter().collect();
    definitions.sort_by(|a, b| a.peripheral.name.cmp(&b.peripheral.name));
    let mut out = String::from(
        "// Generated from rp1-regdb TOML.\n// Do not edit manually.\n// Regenerate with: cargo run -p rp1-regdb -- generate\n\n",
    );
    writeln!(out, "use core::sync::atomic::{{AtomicBool, Ordering}};\n").unwrap();
    writeln!(out, "static TAKEN: AtomicBool = AtomicBool::new(false);\n").unwrap();
    for definition in &definitions {
        let peripheral = &definition.peripheral;
        peripheral_docs(&mut out, peripheral, "");
        if definition.registers.is_empty() {
            rustdoc(
                &mut out,
                "",
                "Address-only or externally delegated block; unknown register layout is intentionally hidden.",
            );
        }
        writeln!(
            out,
            "#[allow(non_camel_case_types)]\npub struct {} {{ _private: () }}",
            peripheral.name
        )
        .unwrap();
        let compatibility_alias = owner_type_alias(&peripheral.name);
        if compatibility_alias != peripheral.name {
            writeln!(out, "pub type {compatibility_alias} = {};", peripheral.name).unwrap();
        }
        writeln!(out, "impl {} {{", peripheral.name).unwrap();
        writeln!(
            out,
            "    pub const PTR: *const u8 = {:#x} as *const u8;",
            peripheral.base
        )
        .unwrap();
        if let Some(external_ip) = &peripheral.external_ip {
            writeln!(out, "    pub const EXTERNAL_IP: &str = {external_ip:?};").unwrap();
        }
        if let Some(clock) = &peripheral.clock_dependency {
            writeln!(out, "    pub const CLOCK_DEPENDENCY: &str = {clock:?};").unwrap();
        }
        if let Some(reset) = &peripheral.reset_dependency {
            writeln!(out, "    pub const RESET_DEPENDENCY: &str = {reset:?};").unwrap();
        }
        if let Some(pinmux) = &peripheral.pinmux_dependency {
            writeln!(out, "    pub const PINMUX_DEPENDENCY: &str = {pinmux:?};").unwrap();
        }
        writeln!(
            out,
            "    #[inline(always)]\n    pub const fn ptr() -> *const u8 {{ Self::PTR }}"
        )
        .unwrap();
        writeln!(out, "}}\n").unwrap();
        if !definition.registers.is_empty() {
            writeln!(out, "impl core::ops::Deref for {} {{", peripheral.name).unwrap();
            writeln!(
                out,
                "    type Target = crate::generated::{}::RegisterBlock;",
                peripheral.module
            )
            .unwrap();
            writeln!(out, "    #[inline(always)]\n    fn deref(&self) -> &Self::Target {{ unsafe {{ &*(Self::PTR.cast()) }} }}").unwrap();
            writeln!(out, "}}\n").unwrap();
        }
        writeln!(out, "unsafe impl Send for {} {{}}\n", peripheral.name).unwrap();
    }
    writeln!(out, "#[allow(non_snake_case)]\npub struct Peripherals {{").unwrap();
    for definition in &definitions {
        writeln!(
            out,
            "    pub {}: {},",
            definition.peripheral.name, definition.peripheral.name
        )
        .unwrap();
    }
    writeln!(out, "}}\n").unwrap();
    writeln!(out, "impl Peripherals {{").unwrap();
    writeln!(out, "    /// # Safety\n    /// Coordinate ownership across both cores. TAKEN is core-local, not a device-wide lock.\n    pub unsafe fn take() -> Option<Self> {{\n        TAKEN.compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire).ok().map(|_| unsafe {{ Self::steal() }})\n    }}").unwrap();
    writeln!(out, "    /// # Safety\n    /// The caller must ensure no other `Peripherals` instance is in use.\n    pub unsafe fn steal() -> Self {{\n        TAKEN.store(true, Ordering::Release);\n        Self {{").unwrap();
    for definition in &definitions {
        writeln!(
            out,
            "            {}: {} {{ _private: () }},",
            definition.peripheral.name, definition.peripheral.name
        )
        .unwrap();
    }
    writeln!(out, "        }}\n    }}\n}}\n").unwrap();
    writeln!(out, "#[cfg(test)]\nmod layout_tests {{").unwrap();
    for definition in &definitions {
        writeln!(
            out,
            "    #[test]\n    fn {}_base() {{ assert_eq!(super::{}::PTR as usize, {:#x}); }}",
            ident(&definition.peripheral.name),
            definition.peripheral.name,
            definition.peripheral.base
        )
        .unwrap();
    }
    writeln!(out, "}}").unwrap();
    out
}

fn interrupts(db: &Database) -> String {
    let mut interrupts: Vec<_> = db.interrupts.iter().collect();
    interrupts.sort_by_key(|interrupt| (interrupt.number, interrupt.name.as_str()));
    let interrupts_by_number: BTreeMap<_, _> = interrupts
        .iter()
        .map(|interrupt| (interrupt.number, *interrupt))
        .collect();
    let mut out = String::from(
        "// Generated from rp1-regdb/data/device.toml.\n// Do not edit manually.\n// Regenerate with: cargo run -p rp1-regdb -- generate\n\n",
    );
    writeln!(
        out,
        "pub const NVIC_PRIO_BITS: u8 = {};",
        db.core.nvic.priority_bits
    )
    .unwrap();
    writeln!(
        out,
        "pub const NVIC_INTERRUPT_COUNT: usize = {};",
        db.core.nvic.external_slots
    )
    .unwrap();
    writeln!(
        out,
        "pub const DEFINED_INTERRUPT_COUNT: usize = {};\n",
        db.interrupts.len()
    )
    .unwrap();
    writeln!(
        out,
        "#[derive(Clone, Copy, Debug, Eq, PartialEq)]\n#[repr(u16)]\npub enum Interrupt {{"
    )
    .unwrap();
    for interrupt in &interrupts {
        writeln!(out, "    {} = {},", interrupt.name, interrupt.number).unwrap();
    }
    writeln!(out, "}}\n").unwrap();
    writeln!(out, "unsafe impl cortex_m::interrupt::InterruptNumber for Interrupt {{\n    #[inline(always)]\n    fn number(self) -> u16 {{ self as u16 }}\n}}\n").unwrap();

    writeln!(
        out,
        "#[cfg(feature = \"vectors\")]\n#[allow(non_snake_case)]\nunsafe extern \"C\" {{"
    )
    .unwrap();
    for interrupt in &interrupts {
        writeln!(out, "    fn {}();", interrupt.name).unwrap();
    }
    writeln!(out, "}}\n").unwrap();

    writeln!(out, "#[cfg(feature = \"vectors\")]\n#[doc(hidden)]\n#[repr(C)]\npub union Vector {{\n    _handler: unsafe extern \"C\" fn(),\n    _reserved: u32,\n}}\n").unwrap();
    writeln!(out, "#[cfg(feature = \"vectors\")]\n#[doc(hidden)]\n#[used]\n#[unsafe(no_mangle)]\n#[unsafe(link_section = \".vector_table.interrupts\")]\npub static __INTERRUPTS: [Vector; NVIC_INTERRUPT_COUNT] = [").unwrap();
    for number in 0..db.core.nvic.external_slots {
        if let Some(interrupt) = interrupts_by_number.get(&number) {
            writeln!(
                out,
                "    // IRQ {number}: {}\n    Vector {{ _handler: {} }},",
                interrupt.name, interrupt.name
            )
            .unwrap();
        } else {
            writeln!(
                out,
                "    // IRQ {number}: reserved\n    Vector {{ _reserved: 0 }},"
            )
            .unwrap();
        }
    }
    writeln!(out, "];\n").unwrap();
    out
}

fn device_x(db: &Database) -> String {
    let mut interrupts: Vec<_> = db.interrupts.iter().collect();
    interrupts.sort_by_key(|interrupt| (interrupt.number, interrupt.name.as_str()));
    let mut out = String::from(
        "/* Generated from rp1-regdb/data/device.toml. */\n/* Do not edit manually. */\n/* Regenerate with: cargo run -p rp1-regdb -- generate */\n\n",
    );
    for interrupt in interrupts {
        writeln!(out, "PROVIDE({} = DefaultHandler);", interrupt.name).unwrap();
    }
    out
}

fn metadata(db: &Database) -> String {
    let mut out = String::from(
        "// Generated from rp1-regdb/data/device.toml.\n// Do not edit manually.\n// Regenerate with: cargo run -p rp1-regdb -- generate\n\n",
    );
    writeln!(out, "#[derive(Clone, Copy, Debug, Eq, PartialEq)]\npub enum CoreArchitecture {{\n    Armv7M,\n}}\n").unwrap();
    writeln!(out, "#[derive(Clone, Copy, Debug, Eq, PartialEq)]\npub enum CoreImplementation {{\n    CortexM3R2P1,\n}}\n").unwrap();
    writeln!(out, "#[derive(Clone, Copy, Debug, Eq, PartialEq)]\npub enum CoreEndianness {{\n    Little,\n    Big,\n}}\n").unwrap();
    writeln!(out, "#[derive(Clone, Copy, Debug, Eq, PartialEq)]\npub enum MetadataVerification {{\n    Official,\n    Hardware,\n    Static,\n    Architectural,\n    Unverified,\n    NotApplicable,\n}}\n").unwrap();
    writeln!(out, "#[derive(Clone, Copy, Debug, Eq, PartialEq)]\npub enum ObservationClass {{\n    Capability,\n    ScopedBehavior,\n    RuntimeState,\n    ArchitecturalInterpretation,\n}}\n").unwrap();
    writeln!(out, "#[derive(Clone, Copy, Debug, Eq, PartialEq)]\npub enum ObservationResult {{\n    Confirmed,\n    NotObserved,\n    Inconclusive,\n    Contradicted,\n}}\n").unwrap();
    writeln!(out, "#[derive(Clone, Copy, Debug, Eq, PartialEq)]\npub struct Observation {{\n    pub verification: MetadataVerification,\n    pub result: ObservationResult,\n    pub class: ObservationClass,\n    pub scope: Option<&'static str>,\n    pub note: Option<&'static str>,\n}}\n").unwrap();

    writeln!(
        out,
        "pub const CORE_ARCHITECTURE: CoreArchitecture = CoreArchitecture::{};",
        core_architecture_variant(db.core.architecture)
    )
    .unwrap();
    writeln!(
        out,
        "pub const CORE_IMPLEMENTATION: CoreImplementation = CoreImplementation::{};",
        core_implementation_variant(db.core.implementation, db.core.revision)
    )
    .unwrap();
    writeln!(out, "pub const CORE_CPUID: u32 = {:#010x};", db.core.cpuid).unwrap();
    writeln!(
        out,
        "pub const PROCESSOR_COUNT: u8 = {};",
        db.core.processor_count
    )
    .unwrap();
    writeln!(
        out,
        "pub const CORE_ENDIANNESS: CoreEndianness = CoreEndianness::{};\n",
        endianness_variant(db.core.endianness)
    )
    .unwrap();

    writeln!(
        out,
        "pub const NVIC_EXTERNAL_SLOTS: u16 = {};",
        db.core.nvic.external_slots
    )
    .unwrap();
    writeln!(
        out,
        "pub const NVIC_ASSIGNED_INTERRUPTS: u16 = {};",
        db.core.nvic.assigned_interrupts
    )
    .unwrap();
    writeln!(
        out,
        "pub const NVIC_PRIORITY_BITS: u8 = {};\n",
        db.core.nvic.priority_bits
    )
    .unwrap();
    writeln!(
        out,
        "pub const NVIC_VERIFICATION: MetadataVerification = MetadataVerification::{};\n",
        verification_variant(db.core.nvic.verification)
    )
    .unwrap();

    writeln!(
        out,
        "pub const MPU_PRESENT: bool = {};",
        db.core.mpu.present
    )
    .unwrap();
    writeln!(
        out,
        "pub const MPU_UNIFIED: bool = {};",
        db.core.mpu.kind == MpuKind::Unified
    )
    .unwrap();
    writeln!(
        out,
        "pub const MPU_DATA_REGIONS: u8 = {};",
        db.core.mpu.data_regions
    )
    .unwrap();
    writeln!(
        out,
        "pub const MPU_INSTRUCTION_REGIONS: u8 = {};",
        db.core.mpu.instruction_regions
    )
    .unwrap();
    writeln!(
        out,
        "pub const MPU_VERIFICATION: MetadataVerification = MetadataVerification::{};",
        verification_variant(db.core.mpu.verification)
    )
    .unwrap();
    let enforcement = &db.core.mpu.proc0_privileged_shared_sram_data_read;
    writeln!(
        out,
        "pub const MPU_PROC0_TEST_REGION: u8 = {};",
        enforcement.region
    )
    .unwrap();
    writeln!(
        out,
        "pub const MPU_PROC0_TEST_REGION_SIZE_BYTES: u16 = {};",
        enforcement.region_size_bytes
    )
    .unwrap();
    writeln!(
        out,
        "pub const MPU_PROC0_TEST_EXCEPTION_NUMBER: u8 = {};\n",
        enforcement.exception_number
    )
    .unwrap();
    write_observation(
        &mut out,
        "MPU_PROC0_PRECISE_MEMMANAGE",
        &enforcement.precise_memmanage,
    );
    write_observation(
        &mut out,
        "MPU_PROC0_DATA_ACCESS_VIOLATION",
        &enforcement.data_access_violation,
    );
    write_observation(
        &mut out,
        "MPU_PROC0_FAULT_ADDRESS_VALID",
        &enforcement.fault_address_valid,
    );
    write_observation(
        &mut out,
        "MPU_PROC0_EXACT_FAULT_ADDRESS",
        &enforcement.exact_fault_address,
    );
    write_observation(
        &mut out,
        "MPU_PROC0_HARDFAULT_ESCALATION",
        &enforcement.hardfault_escalation,
    );
    write_observation(
        &mut out,
        "MPU_PROC0_EXACT_PC_RECOVERY",
        &enforcement.exact_pc_recovery,
    );
    write_observation(
        &mut out,
        "MPU_PROC0_SINGLE_HANDLER_ENTRY",
        &enforcement.single_handler_entry,
    );

    writeln!(
        out,
        "pub const SYSTICK_CALIB: u32 = {:#010x};",
        db.core.systick.calib
    )
    .unwrap();
    writeln!(
        out,
        "pub const SYSTICK_TENMS: u32 = {};",
        db.core.systick.tenms
    )
    .unwrap();
    writeln!(
        out,
        "pub const SYSTICK_HAS_REFERENCE: bool = {};",
        db.core.systick.reference_present
    )
    .unwrap();
    writeln!(
        out,
        "pub const SYSTICK_SKEW: bool = {};",
        db.core.systick.skew
    )
    .unwrap();
    writeln!(
        out,
        "pub const SYSTICK_NOMINAL_REFERENCE_HZ: u32 = {};\n",
        db.core.systick.nominal_reference_hz
    )
    .unwrap();
    writeln!(
        out,
        "pub const SYSTICK_CALIB_VERIFICATION: MetadataVerification = MetadataVerification::{};",
        verification_variant(db.core.systick.calib_verification)
    )
    .unwrap();
    writeln!(out, "pub const SYSTICK_FREQUENCY_VERIFICATION: MetadataVerification = MetadataVerification::{};\n", verification_variant(db.core.systick.frequency_verification)).unwrap();

    writeln!(
        out,
        "pub const DWT_PRESENT: bool = {};",
        db.core.dwt.present
    )
    .unwrap();
    writeln!(
        out,
        "pub const DWT_COMPARATORS: u8 = {};",
        db.core.dwt.comparators
    )
    .unwrap();
    writeln!(
        out,
        "pub const DWT_HAS_CYCCNT: bool = {};",
        db.core.dwt.cycle_counter
    )
    .unwrap();
    writeln!(
        out,
        "pub const DWT_HAS_PROFILING_COUNTERS: bool = {};",
        db.core.dwt.profiling_counters
    )
    .unwrap();
    writeln!(
        out,
        "pub const DWT_HAS_EXTERNAL_MATCH_TRIGGER: bool = {};",
        db.core.dwt.external_match_trigger
    )
    .unwrap();
    writeln!(
        out,
        "pub const DWT_HAS_TRACE_PACKETS: bool = {};\n",
        db.core.dwt.trace_packets
    )
    .unwrap();
    writeln!(
        out,
        "pub const DWT_VERIFICATION: MetadataVerification = MetadataVerification::{};\n",
        verification_variant(db.core.dwt.verification)
    )
    .unwrap();

    writeln!(
        out,
        "pub const FPB_PRESENT: bool = {};",
        db.core.fpb.present
    )
    .unwrap();
    writeln!(
        out,
        "pub const FPB_CODE_COMPARATORS: u8 = {};",
        db.core.fpb.code_comparators
    )
    .unwrap();
    writeln!(
        out,
        "pub const FPB_LITERAL_COMPARATORS: u8 = {};\n",
        db.core.fpb.literal_comparators
    )
    .unwrap();
    writeln!(
        out,
        "pub const FPB_VERIFICATION: MetadataVerification = MetadataVerification::{};\n",
        verification_variant(db.core.fpb.verification)
    )
    .unwrap();

    write_observation(
        &mut out,
        "HARDWARE_DIVIDE",
        &db.core.instructions.hardware_divide,
    );
    write_observation(
        &mut out,
        "EXCLUSIVE_LOAD",
        &db.core.instructions.exclusive_load,
    );
    write_observation(
        &mut out,
        "EXCLUSIVE_STORE",
        &db.core.instructions.exclusive_store,
    );
    writeln!(
        out,
        "pub const BOOT_ROM_PRESENT: bool = {};",
        db.boot_rom.present
    )
    .unwrap();
    write_observation(
        &mut out,
        "BOOT_ROM_PUBLIC_LOOKUP_TABLE",
        &db.boot_rom.public_lookup_table,
    );
    write_observation(
        &mut out,
        "BOOT_ROM_OFFICIAL_FIRMWARE_CALLS",
        &db.boot_rom.official_firmware_rom_calls,
    );

    writeln!(
        out,
        "#[derive(Clone, Copy, Debug, Eq, PartialEq)]\n#[repr(u8)]\npub enum ClockId {{"
    )
    .unwrap();
    let mut clocks: Vec<_> = db.clocks.iter().collect();
    clocks.sort_by_key(|clock| (clock.id, clock.name.as_str()));
    for clock in clocks {
        writeln!(out, "    {} = {},", clock.name, clock.id).unwrap();
    }
    writeln!(out, "}}\n").unwrap();

    writeln!(out, "#[derive(Clone, Copy, Debug, Eq, PartialEq)]\npub struct GpioBank {{\n    pub id: u8,\n    pub first_gpio: u8,\n    pub gpio_count: u8,\n    pub io_offset: usize,\n    pub interrupt_enable_offset: usize,\n    pub interrupt_status_offset: usize,\n    pub rio_offset: usize,\n    pub pads_offset: usize,\n}}\n").unwrap();
    writeln!(out, "pub const GPIO_BANKS: &[GpioBank] = &[").unwrap();
    let mut banks: Vec<_> = db.gpio_banks.iter().collect();
    banks.sort_by_key(|bank| bank.id);
    for bank in banks {
        writeln!(out, "    GpioBank {{ id: {}, first_gpio: {}, gpio_count: {}, io_offset: {:#x}, interrupt_enable_offset: {:#x}, interrupt_status_offset: {:#x}, rio_offset: {:#x}, pads_offset: {:#x} }},", bank.id, bank.first_gpio, bank.gpio_count, bank.io_offset, bank.interrupt_enable_offset, bank.interrupt_status_offset, bank.rio_offset, bank.pads_offset).unwrap();
    }
    writeln!(out, "];\n").unwrap();

    writeln!(out, "#[derive(Clone, Copy, Debug, Eq, PartialEq)]\npub struct AddressSpace {{\n    pub name: &'static str,\n    pub local_base: usize,\n    pub size: usize,\n    pub host_base: Option<u64>,\n}}\n").unwrap();
    writeln!(out, "pub const ADDRESS_SPACES: &[AddressSpace] = &[").unwrap();
    let mut spaces: Vec<_> = db.address_spaces.iter().collect();
    spaces.sort_by(|a, b| a.name.cmp(&b.name));
    for space in spaces {
        match space.host_base {
            Some(host) => writeln!(out, "    AddressSpace {{ name: {:?}, local_base: {:#x}, size: {:#x}, host_base: Some({host:#x}) }},", space.name, space.local_base, space.size).unwrap(),
            None => writeln!(out, "    AddressSpace {{ name: {:?}, local_base: {:#x}, size: {:#x}, host_base: None }},", space.name, space.local_base, space.size).unwrap(),
        }
    }
    writeln!(out, "];\n").unwrap();
    if let Some(endpoint) = &db.pci_endpoint {
        writeln!(
            out,
            "pub const PCI_VENDOR_ID: u16 = {:#06x};\npub const PCI_DEVICE_ID: u16 = {:#06x};\n",
            endpoint.vendor_id, endpoint.device_id
        )
        .unwrap();
    }
    out
}

fn write_observation(out: &mut String, name: &str, observation: &Observation) {
    writeln!(
        out,
        "pub const {name}: Observation = Observation {{ verification: MetadataVerification::{}, result: ObservationResult::{}, class: ObservationClass::{}, scope: {}, note: {} }};",
        verification_variant(observation.verification),
        observation_result_variant(observation.result),
        observation_class_variant(observation.class),
        rust_string_option(observation.scope.as_deref()),
        rust_string_option(observation.note.as_deref()),
    )
    .unwrap();
}

fn core_architecture_variant(architecture: CoreArchitecture) -> &'static str {
    match architecture {
        CoreArchitecture::Armv7M => "Armv7M",
    }
}

fn core_implementation_variant(
    implementation: CoreImplementation,
    revision: CoreRevision,
) -> &'static str {
    match (implementation, revision) {
        (CoreImplementation::CortexM3, CoreRevision::R2p1) => "CortexM3R2P1",
    }
}

fn endianness_variant(endianness: Endianness) -> &'static str {
    match endianness {
        Endianness::Little => "Little",
        Endianness::Big => "Big",
    }
}

fn verification_variant(verification: Verification) -> &'static str {
    match verification {
        Verification::Official => "Official",
        Verification::Hardware => "Hardware",
        Verification::Static => "Static",
        Verification::Architectural => "Architectural",
        Verification::Unverified => "Unverified",
        Verification::NotApplicable => "NotApplicable",
    }
}

fn observation_result_variant(result: ObservationResult) -> &'static str {
    match result {
        ObservationResult::Confirmed => "Confirmed",
        ObservationResult::NotObserved => "NotObserved",
        ObservationResult::Inconclusive => "Inconclusive",
        ObservationResult::Contradicted => "Contradicted",
    }
}

fn observation_class_variant(class: ObservationClass) -> &'static str {
    match class {
        ObservationClass::Capability => "Capability",
        ObservationClass::ScopedBehavior => "ScopedBehavior",
        ObservationClass::RuntimeState => "RuntimeState",
        ObservationClass::ArchitecturalInterpretation => "ArchitecturalInterpretation",
    }
}

fn rust_string_option(value: Option<&str>) -> String {
    value.map_or_else(|| "None".into(), |value| format!("Some({value:?})"))
}

fn ident(value: &str) -> String {
    let mut out = value.to_ascii_lowercase().replace(['-', '.'], "_");
    if matches!(out.as_str(), "type" | "match" | "loop" | "in" | "ref") {
        out.push('_');
    }
    out
}

fn rust_type(value: &str) -> String {
    value
        .to_ascii_uppercase()
        .replace(['-', '.'], "_")
        .replace("%S", "")
}

fn owner_type_alias(value: &str) -> String {
    value
        .split('_')
        .map(|part| {
            let mut characters = part.chars();
            match characters.next() {
                Some(first) => {
                    first.to_ascii_uppercase().to_string()
                        + &characters.as_str().to_ascii_lowercase()
                }
                None => String::new(),
            }
        })
        .collect()
}

fn visibility(exposure: ApiExposure) -> &'static str {
    match exposure {
        ApiExposure::Public => "pub",
        ApiExposure::CratePrivate => "pub(crate)",
        ApiExposure::None => "",
    }
}

fn register_visible(register: &Register) -> bool {
    register.api.is_visible()
        || [
            register.atomic.set,
            register.atomic.clear,
            register.atomic.xor,
        ]
        .into_iter()
        .any(|exposure| exposure != AtomicExposure::None)
}

fn register_visibility(register: &Register) -> &'static str {
    if register.api.is_public()
        || [
            register.atomic.set,
            register.atomic.clear,
            register.atomic.xor,
        ]
        .into_iter()
        .any(|exposure| exposure != AtomicExposure::None)
    {
        "pub"
    } else {
        "pub(crate)"
    }
}

pub fn write_coverage(db: &Database, path: &Path) -> Result<()> {
    let register_count: usize = db.peripherals.iter().map(|p| p.registers.len()).sum();
    let field_count: usize = db
        .peripherals
        .iter()
        .flat_map(|p| &p.registers)
        .map(|r| r.fields.len())
        .sum();
    let atomic_count = db
        .peripherals
        .iter()
        .flat_map(|p| &p.registers)
        .filter(|r| r.atomic.kind != AtomicKind::None)
        .count();
    let external_count = db
        .peripherals
        .iter()
        .filter(|p| p.peripheral.kind == PeripheralKind::ExternalIp)
        .count();
    let hidden_count = db
        .peripherals
        .iter()
        .flat_map(|p| &p.registers)
        .filter(|r| !register_visible(r))
        .count();
    let hardware_interrupt_count = db
        .interrupts
        .iter()
        .filter(|interrupt| {
            interrupt.origin == Origin::Hardware && interrupt.verified == Verification::Hardware
        })
        .count();
    let linux_interrupt_count = db
        .interrupts
        .iter()
        .filter(|interrupt| {
            interrupt.origin == Origin::Linux && interrupt.verified == Verification::Static
        })
        .count();
    let mut text = String::new();
    writeln!(text, "# RP1 regdb coverage\n").unwrap();
    writeln!(text, "- schema: `{}`", db.schema).unwrap();
    writeln!(text, "- logical peripherals: {}", db.peripherals.len()).unwrap();
    writeln!(text, "- external-IP instances: {external_count}").unwrap();
    writeln!(text, "- registers: {register_count}").unwrap();
    writeln!(text, "- hidden registers: {hidden_count}").unwrap();
    writeln!(text, "- fields: {field_count}").unwrap();
    writeln!(text, "- atomic-enabled registers: {atomic_count}").unwrap();
    writeln!(
        text,
        "- NVIC external interrupt slots: {} (IRQn 0..{})",
        db.core.nvic.external_slots,
        db.core.nvic.external_slots - 1
    )
    .unwrap();
    writeln!(
        text,
        "- named interrupts: {} (IRQn 0..{})",
        db.interrupts.len(),
        db.interrupts.len() - 1
    )
    .unwrap();
    writeln!(
        text,
        "- reserved NVIC external interrupt slots: {} (IRQn {}..{})",
        usize::from(db.core.nvic.external_slots) - db.interrupts.len(),
        db.interrupts.len(),
        db.core.nvic.external_slots - 1
    )
    .unwrap();
    writeln!(
        text,
        "- interrupt numbering: MSI-X source ID equals local NVIC IRQn"
    )
    .unwrap();
    writeln!(
        text,
        "- interrupt evidence: hardware/hardware {hardware_interrupt_count}; linux/static {linux_interrupt_count}"
    )
    .unwrap();
    writeln!(text, "- clock IDs: {}", db.clocks.len()).unwrap();
    writeln!(text, "- GPIO banks: {}", db.gpio_banks.len()).unwrap();
    writeln!(text, "- address spaces: {}\n", db.address_spaces.len()).unwrap();
    writeln!(
        text,
        "- PCI endpoint identity: {}\n",
        if db.pci_endpoint.is_some() {
            "present"
        } else {
            "absent"
        }
    )
    .unwrap();
    writeln!(text, "## Migrated peripherals\n").unwrap();
    let mut definitions: Vec<_> = db.peripherals.iter().collect();
    definitions.sort_by(|a, b| a.peripheral.name.cmp(&b.peripheral.name));
    for definition in definitions {
        writeln!(
            text,
            "- `{}`: {} registers ({})",
            definition.peripheral.name,
            definition.registers.len(),
            source_label(definition)
        )
        .unwrap();
    }
    fs::write(path, text).with_context(|| format!("writing {}", path.display()))
}
