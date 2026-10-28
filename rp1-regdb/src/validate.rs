use std::collections::{BTreeMap, BTreeSet};

use crate::model::*;

pub fn validate(db: &Database) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    if db.schema != "rp1-regdb-v3" {
        errors.push(format!(
            "device.toml: [SCHEMA_VERSION] expected rp1-regdb-v3, found {}",
            db.schema
        ));
    }
    if db.device.address_unit_bits != 8 || db.device.register_width != 32 {
        errors.push(
            "device.toml: [DEVICE_WIDTH] generator requires 8-bit address units and 32-bit registers"
                .into(),
        );
    }

    let mut bases = BTreeMap::new();
    let mut names = BTreeSet::new();
    let mut ids = BTreeSet::new();
    let mut modules = BTreeSet::new();
    for definition in &db.peripherals {
        validate_peripheral_identity(
            definition,
            &mut names,
            &mut ids,
            &mut modules,
            &mut bases,
            &mut errors,
        );
        validate_peripheral(definition, &mut errors);
    }

    validate_core(db, &mut errors);
    validate_interrupts(db, &mut errors);
    validate_metadata(db, &mut errors);
    validate_rp1_register_contracts(db, &mut errors);
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn validate_peripheral_identity<'a>(
    definition: &'a PeripheralDefinition,
    names: &mut BTreeSet<String>,
    ids: &mut BTreeSet<String>,
    modules: &mut BTreeSet<String>,
    bases: &mut BTreeMap<u64, &'a str>,
    errors: &mut Vec<String>,
) {
    let peripheral = &definition.peripheral;
    let source = definition.source.display();
    if !names.insert(peripheral.name.clone()) {
        errors.push(format!(
            "{source}: peripheral {}: [PERIPHERAL_NAME_UNIQUE] duplicate logical name",
            peripheral.name
        ));
    }
    if !ids.insert(peripheral.id.clone()) {
        errors.push(format!(
            "{source}: peripheral {}: [PERIPHERAL_ID_UNIQUE] duplicate id {}",
            peripheral.name, peripheral.id
        ));
    }
    if !modules.insert(peripheral.module.clone()) {
        errors.push(format!(
            "{source}: peripheral {}: [PERIPHERAL_MODULE_UNIQUE] duplicate module {}",
            peripheral.name, peripheral.module
        ));
    }
    if let Some(previous) = bases.insert(peripheral.base, &peripheral.name) {
        errors.push(format!(
            "{source}: peripheral {}: [PERIPHERAL_BASE_UNIQUE] base {:#x} duplicates {previous}",
            peripheral.name, peripheral.base
        ));
    }
}

fn validate_peripheral(definition: &PeripheralDefinition, errors: &mut Vec<String>) {
    let peripheral = &definition.peripheral;
    let source = definition.source.display();
    if peripheral.ownership != Ownership::Singleton {
        errors.push(format!(
            "{source}: peripheral {}: [OWNERSHIP_SINGLETON] only singleton ownership is generated",
            peripheral.name
        ));
    }
    match peripheral.kind {
        PeripheralKind::ExternalIp => {
            if peripheral.external_ip.as_deref().is_none_or(str::is_empty) {
                errors.push(format!(
                    "{source}: peripheral {}: [EXTERNAL_IP_REFERENCE] missing external_ip identifier",
                    peripheral.name
                ));
            }
            if !definition.registers.is_empty() {
                errors.push(format!(
                    "{source}: peripheral {}: [EXTERNAL_IP_DELEGATION] external IP register bodies must not be duplicated",
                    peripheral.name
                ));
            }
        }
        _ if peripheral.external_ip.is_some() => errors.push(format!(
            "{source}: peripheral {}: [EXTERNAL_IP_REFERENCE] non-external peripheral has external_ip",
            peripheral.name
        )),
        _ => {}
    }
    if let Some(view) = &peripheral.atomic_view
        && (peripheral.kind != PeripheralKind::Rp1NativeApb || view.kind != AtomicKind::Rp1Native) {
            errors.push(format!(
                "{source}: peripheral {}: [ATOMIC_VIEW_KIND] native atomic view requires rp1-native-apb",
                peripheral.name
            ));
        }

    let mut custom = BTreeMap::new();
    for view in &peripheral.custom_views {
        if custom.insert(view.id.as_str(), view).is_some() {
            errors.push(format!(
                "{source}: peripheral {}: [CUSTOM_VIEW_UNIQUE] duplicate custom view {}",
                peripheral.name, view.id
            ));
        }
    }

    let mut register_names = BTreeSet::new();
    let mut register_ids = BTreeSet::new();
    let mut occupied = BTreeMap::new();
    for register in &definition.registers {
        let context = register_context(peripheral, register);
        if !register_names.insert(register.name.as_str()) {
            errors.push(format!(
                "{context}: [REGISTER_NAME_UNIQUE] duplicate register name"
            ));
        }
        if !register_ids.insert(register.id.as_str()) {
            errors.push(format!(
                "{context}: [REGISTER_ID_UNIQUE] duplicate register id {}",
                register.id
            ));
        }
        validate_layout(peripheral, register, &context, &mut occupied, errors);
        validate_access(peripheral, register, &context, &custom, errors);
        validate_fields(peripheral, register, &context, errors);
        if let Some(initialization) = &register.boot_rom_initialization {
            validate_observation(
                &format!("{context}: Boot ROM initialization"),
                &initialization.observation,
                errors,
            );
            if initialization.observation.result != ObservationResult::Confirmed {
                errors.push(format!(
                    "{context}: [BOOT_ROM_INITIALIZATION_UNCONFIRMED] initialization value requires a Confirmed observation"
                ));
            }
            if register.reset_value == Some(initialization.value) {
                errors.push(format!(
                    "{context}: [BOOT_ROM_INITIALIZATION_AS_RESET] Boot ROM initialization value {:#x} must not be used as the hardware reset value",
                    initialization.value
                ));
            }
        }
    }
}

fn register_context(peripheral: &Peripheral, register: &Register) -> String {
    format!(
        "{}: peripheral {} register {}",
        register.source.display(),
        peripheral.name,
        register.name
    )
}

fn validate_layout<'a>(
    peripheral: &Peripheral,
    register: &'a Register,
    context: &str,
    occupied: &mut BTreeMap<u64, &'a str>,
    errors: &mut Vec<String>,
) {
    if register.width != 32 {
        errors.push(format!(
            "{context}: [REGISTER_WIDTH] only 32-bit registers are supported"
        ));
    }
    let byte_width = u64::from(register.width / 8);
    if register.repeat_count == 0 {
        errors.push(format!(
            "{context}: [REPEAT_COUNT] repeat_count must be nonzero"
        ));
        return;
    }
    if register.repeat_stride < byte_width || !register.repeat_stride.is_multiple_of(byte_width) {
        errors.push(format!(
            "{context}: [REPEAT_STRIDE] repeat_stride ({:#x}) must be at least and aligned to register width in bytes ({byte_width:#x})",
            register.repeat_stride
        ));
    }
    let public_count = register.public_count.unwrap_or(register.repeat_count);
    if public_count == 0 || public_count > register.repeat_count {
        errors.push(format!(
            "{context}: [PUBLIC_COUNT] public_count {public_count} must be in 1..={} ",
            register.repeat_count
        ));
    }
    if !register.offset.is_multiple_of(byte_width) {
        errors.push(format!(
            "{context}: [REGISTER_ALIGNMENT] offset {:#x} is not {byte_width}-byte aligned",
            register.offset
        ));
    }
    let end = register.offset
        + u64::from(register.repeat_count - 1) * register.repeat_stride
        + byte_width;
    if end > peripheral.size {
        errors.push(format!(
            "{context}: [REGISTER_BLOCK_RANGE] end {end:#x} exceeds block size {:#x}",
            peripheral.size
        ));
    }
    for index in 0..register.repeat_count {
        let offset = register.offset + u64::from(index) * register.repeat_stride;
        if let Some(previous) = occupied.insert(offset, register.name.as_str()) {
            errors.push(format!(
                "{context}: [REGISTER_OFFSET_UNIQUE] offset {offset:#x} overlaps {previous}"
            ));
        }
    }
}

fn validate_access(
    peripheral: &Peripheral,
    register: &Register,
    context: &str,
    custom: &BTreeMap<&str, &CustomView>,
    errors: &mut Vec<String>,
) {
    let api = &register.api;
    let safe_operations = [
        ("modify", api.modify),
        ("reset", api.reset),
        ("w1c", api.w1c),
        ("w1s", api.w1s),
        ("w1t", api.w1t),
        ("command", api.command),
        ("fifo_push", api.fifo_push),
    ];
    if api.read != ApiExposure::None && register.hardware_read == ReadBehavior::None {
        errors.push(format!(
            "{context} field <register>: operation read: [READ_API_ON_NONREADABLE] read API exposed for non-readable hardware"
        ));
    }
    if api.fifo_pop != ApiExposure::None && register.hardware_read != ReadBehavior::Fifo {
        errors.push(format!(
            "{context} field <register>: operation fifo_pop: [FIFO_POP_BEHAVIOR] fifo_pop requires FIFO read behavior"
        ));
    }
    for (operation, exposure) in safe_operations {
        if exposure != ApiExposure::None && register.hardware_write == WriteBehavior::None {
            errors.push(format!(
                "{context} field <register>: operation {operation}: [WRITE_API_ON_NONWRITABLE] safe write API exposed for non-writable hardware"
            ));
        }
        if exposure == ApiExposure::CratePrivate {
            errors.push(format!(
                "{context} field <register>: operation {operation}: [CRATE_PRIVATE_OPERATION_UNSUPPORTED] operation must be public or none in generator v2"
            ));
        }
    }
    if api.read == ApiExposure::Public && register.verified_read == Verification::Unverified {
        errors.push(format!(
            "{context} field <register>: operation read: [UNVERIFIED_PUBLIC_READ] public read requires static, official, or hardware verification"
        ));
    }
    if api.modify != ApiExposure::None {
        if register.hardware_read != ReadBehavior::Plain
            || register.hardware_write != WriteBehavior::Plain
        {
            errors.push(format!(
                "{context} field <register>: operation modify: [MODIFY_PLAIN_STATE_ONLY] modify requires plain read and plain write"
            ));
        }
        if register.fields.iter().any(|field| {
            !matches!(
                field.effective_write_behavior(register),
                FieldWriteBehavior::Plain | FieldWriteBehavior::ReadOnly
            )
        }) {
            errors.push(format!(
                "{context} field <register>: operation modify: [MODIFY_MIXED_SEMANTICS] modify forbidden on mixed-semantics register"
            ));
        }
        if !has_verified_operation_field(register, &[FieldWriteBehavior::Plain]) {
            errors.push(format!(
                "{context} field <none>: operation modify: [MODIFY_VERIFIED_PLAIN_FIELD_REQUIRED] public modify requires a public, verified plain field"
            ));
        }
    }
    if api.reset != ApiExposure::None {
        if register.reset_value.is_none() {
            errors.push(format!(
                "{context} field <register>: operation reset: [RESET_VALUE_REQUIRED] reset API requires reset_value metadata"
            ));
        }
        if register.hardware_write != WriteBehavior::Plain {
            errors.push(format!(
                "{context} field <register>: operation reset: [RESET_PLAIN_WRITE_REQUIRED] reset requires plain register write behavior"
            ));
        }
        if !register.verified_write.permits_write() {
            errors.push(format!(
                "{context} field <register>: operation reset: [RESET_OPERATION_UNVERIFIED] reset requires register-level official or hardware write verification"
            ));
        }
    }
    validate_field_operation(
        register,
        context,
        "w1c",
        api.w1c,
        &[FieldWriteBehavior::WriteOneToClear],
        "W1C_FIELD_REQUIRED",
        errors,
    );
    validate_field_operation(
        register,
        context,
        "w1s",
        api.w1s,
        &[FieldWriteBehavior::WriteOneToSet],
        "W1S_FIELD_REQUIRED",
        errors,
    );
    validate_field_operation(
        register,
        context,
        "w1t",
        api.w1t,
        &[FieldWriteBehavior::WriteOneToToggle],
        "W1T_FIELD_REQUIRED",
        errors,
    );
    validate_field_operation(
        register,
        context,
        "command",
        api.command,
        &[
            FieldWriteBehavior::Command,
            FieldWriteBehavior::SelfClearing,
        ],
        "COMMAND_FIELD_REQUIRED",
        errors,
    );
    if api.fifo_push != ApiExposure::None {
        if register.hardware_write != WriteBehavior::Fifo {
            errors.push(format!(
                "{context} field <register>: operation fifo_push: [FIFO_PUSH_BEHAVIOR] fifo_push requires FIFO write behavior"
            ));
        }
        if !register.verified_write.permits_write() {
            errors.push(format!(
                "{context} field <register>: operation fifo_push: [FIFO_PUSH_UNVERIFIED] fifo_push requires register-level write verification"
            ));
        }
    }
    if register.hardware_write == WriteBehavior::StateMachineOnly && api.has_safe_write() {
        errors.push(format!(
            "{context}: [STATE_MACHINE_PUBLIC_WRITE] state-machine-only register cannot expose safe writes"
        ));
    }
    if register.hardware_write == WriteBehavior::StateMachineOnly
        && api.raw_write != RawWriteExposure::None
    {
        errors.push(format!(
            "{context}: [STATE_MACHINE_RAW_WRITE_FORBIDDEN] state-machine-only register cannot expose raw writes"
        ));
    }
    if register.hardware_read == ReadBehavior::StateMachineOnly && api.read != ApiExposure::None {
        errors.push(format!(
            "{context}: [STATE_MACHINE_PUBLIC_READ] state-machine-only register cannot expose general read"
        ));
    }
    if api.raw_write == RawWriteExposure::CratePrivate {
        errors.push(format!(
            "{context}: [CRATE_PRIVATE_RAW_UNSUPPORTED] crate-private raw writes are not generated"
        ));
    }
    if api.raw_write != RawWriteExposure::None
        && matches!(
            register.hardware_write,
            WriteBehavior::None | WriteBehavior::StateMachineOnly
        )
    {
        errors.push(format!(
            "{context}: [RAW_WRITE_ON_NONWRITABLE] raw write requires writable hardware"
        ));
    }
    let inferred_register =
        register.origin.or(Some(peripheral.default_origin)) == Some(Origin::Inferred);
    if inferred_register && api.has_safe_write() {
        errors.push(format!(
            "{context}: [INFERRED_REGISTER_SAFE_WRITE] inferred register cannot expose safe writes"
        ));
    }
    let protected = {
        let identity = format!("{} {}", peripheral.id, peripheral.name).to_ascii_lowercase();
        identity.contains("otp") || identity.contains("efuse") || identity.contains("e_fuse")
    };
    if protected && (api.has_safe_write() || api.raw_write != RawWriteExposure::None) {
        errors.push(format!(
            "{context}: [OTP_EFUSE_WRITE_FORBIDDEN] OTP/eFuse register cannot expose writes"
        ));
    }
    if register.name.eq_ignore_ascii_case("RAW")
        && register.repeat_count > 1
        && (api.has_safe_write() || api.raw_write != RawWriteExposure::None)
    {
        errors.push(format!(
            "{context}: [UNKNOWN_RAW_PUBLIC_WRITE] unknown RAW arrays cannot expose writers"
        ));
    }

    validate_atomic(peripheral, register, context, custom, protected, errors);
}

fn has_verified_operation_field(register: &Register, behaviors: &[FieldWriteBehavior]) -> bool {
    register.fields.iter().any(|field| {
        field.api_write == ApiExposure::Public
            && behaviors.contains(&field.effective_write_behavior(register))
            && field.effective_verified_write(register).permits_write()
    })
}

fn validate_field_operation(
    register: &Register,
    context: &str,
    operation: &str,
    exposure: ApiExposure,
    behaviors: &[FieldWriteBehavior],
    rule: &str,
    errors: &mut Vec<String>,
) {
    if exposure == ApiExposure::None {
        return;
    }
    if !has_verified_operation_field(register, behaviors) {
        errors.push(format!(
            "{context} field <none>: operation {operation}: [{rule}] requires a public, verified field with matching write behavior"
        ));
    }
}

fn validate_atomic(
    peripheral: &Peripheral,
    register: &Register,
    context: &str,
    custom: &BTreeMap<&str, &CustomView>,
    protected: bool,
    errors: &mut Vec<String>,
) {
    let atomic = &register.atomic;
    let exposures = [
        (CustomViewKind::Set, atomic.set, atomic.set_view.as_deref()),
        (
            CustomViewKind::Clear,
            atomic.clear,
            atomic.clear_view.as_deref(),
        ),
        (CustomViewKind::Xor, atomic.xor, atomic.xor_view.as_deref()),
    ];
    let enabled = exposures
        .iter()
        .any(|(_, exposure, _)| *exposure != AtomicExposure::None);
    if enabled && register.hardware_write == WriteBehavior::StateMachineOnly {
        errors.push(format!(
            "{context}: [STATE_MACHINE_ATOMIC_FORBIDDEN] state-machine-only register cannot expose atomic operations"
        ));
    }
    if !enabled {
        if atomic.kind != AtomicKind::None {
            errors.push(format!(
                "{context}: [ATOMIC_UNUSED_KIND] atomic kind set without any operation"
            ));
        }
        return;
    }
    if protected {
        errors.push(format!(
            "{context}: [OTP_EFUSE_ATOMIC_FORBIDDEN] OTP/eFuse register cannot expose atomic aliases"
        ));
    }
    if register.hardware_write != WriteBehavior::Plain {
        errors.push(format!(
            "{context}: [ATOMIC_PLAIN_WRITE_ONLY] atomic aliases require plain register write behavior"
        ));
    }
    match atomic.kind {
        AtomicKind::None => errors.push(format!(
            "{context}: [ATOMIC_KIND_REQUIRED] atomic operation requires a kind"
        )),
        AtomicKind::Rp1Native => {
            if peripheral.kind != PeripheralKind::Rp1NativeApb || peripheral.atomic_view.is_none() {
                errors.push(format!(
                    "{context}: [NATIVE_ATOMIC_VIEW_REQUIRED] rp1-native atomic requires native APB view"
                ));
            }
            if exposures
                .iter()
                .any(|(_, _, reference)| reference.is_some())
            {
                errors.push(format!(
                    "{context}: [NATIVE_ATOMIC_CUSTOM_REFERENCE] native atomic cannot reference custom views"
                ));
            }
        }
        AtomicKind::Custom => {
            for (kind, exposure, reference) in exposures {
                if exposure == AtomicExposure::None {
                    if reference.is_some() {
                        errors.push(format!(
                            "{context}: [CUSTOM_VIEW_UNUSED_REFERENCE] disabled {kind:?} has a view reference"
                        ));
                    }
                    continue;
                }
                let valid = reference
                    .and_then(|name| custom.get(name))
                    .is_some_and(|view| view.kind == kind);
                if !valid {
                    errors.push(format!(
                        "{context}: [CUSTOM_VIEW_REFERENCE] missing or wrong-kind custom {kind:?} view"
                    ));
                }
            }
        }
    }
    for (operation, exposure, _) in exposures {
        if exposure != AtomicExposure::Typed {
            continue;
        }
        let operation_name = match operation {
            CustomViewKind::Set => "atomic_set",
            CustomViewKind::Clear => "atomic_clear",
            CustomViewKind::Xor => "atomic_xor",
        };
        let eligible = register.fields.iter().any(|field| {
            field.api_write == ApiExposure::Public
                && field.effective_write_behavior(register) == FieldWriteBehavior::Plain
                && field.effective_verified_write(register).permits_write()
                && field
                    .origin
                    .or(register.origin)
                    .unwrap_or(peripheral.default_origin)
                    != Origin::Inferred
        });
        if !eligible {
            errors.push(format!(
                "{context} field <none>: operation {operation_name}: [TYPED_ATOMIC_EMPTY_MASK] typed atomic operation has no public, verified plain fields"
            ));
        }
        for field in register.fields.iter().filter(|field| {
            field.api_write == ApiExposure::Public
                && field.effective_write_behavior(register) == FieldWriteBehavior::Plain
                && (!field.effective_verified_write(register).permits_write()
                    || field
                        .origin
                        .or(register.origin)
                        .unwrap_or(peripheral.default_origin)
                        == Origin::Inferred)
        }) {
            errors.push(format!(
                "{context} field {}: operation atomic: [TYPED_ATOMIC_FIELD_UNSAFE] typed atomic fields must be public, plain, verified, and non-inferred",
                field.name
            ));
        }
    }
}

fn validate_fields(
    peripheral: &Peripheral,
    register: &Register,
    context: &str,
    errors: &mut Vec<String>,
) {
    let mut bits = 0u64;
    for field in &register.fields {
        let field_context = format!("{context} field {}", field.name);
        if field.width == 0 || u16::from(field.lsb) + u16::from(field.width) > register.width {
            errors.push(format!(
                "{field_context}: [FIELD_RANGE] bit range exceeds register width"
            ));
            continue;
        }
        let mask = if field.width == 64 {
            u64::MAX
        } else {
            ((1u64 << field.width) - 1) << field.lsb
        };
        if bits & mask != 0 {
            errors.push(format!(
                "{field_context}: [FIELD_OVERLAP] field overlaps another field"
            ));
        }
        bits |= mask;

        let origin = field
            .origin
            .or(register.origin)
            .unwrap_or(peripheral.default_origin);
        let behavior = field.effective_write_behavior(register);
        if field.api_read != ApiExposure::None && register.api.read == ApiExposure::None {
            errors.push(format!(
                "{field_context}: [FIELD_READ_WITHOUT_REGISTER_READ] readable field belongs to non-readable API register"
            ));
        }
        if field.api_read == ApiExposure::Public
            && field.effective_verified_read(register) == Verification::Unverified
        {
            errors.push(format!(
                "{field_context}: [UNVERIFIED_FIELD_READ] public reader requires verification"
            ));
        }
        if field.api_write == ApiExposure::Public {
            if origin == Origin::Inferred {
                errors.push(format!(
                    "{field_context}: [INFERRED_FIELD_SAFE_WRITER] inferred field cannot expose a public writer"
                ));
            }
            if !field.effective_verified_write(register).permits_write() {
                errors.push(format!(
                    "{field_context}: [UNVERIFIED_FIELD_SAFE_WRITER] public field writer requires official or hardware verification"
                ));
            }
            if behavior == FieldWriteBehavior::ReadOnly {
                errors.push(format!(
                    "{field_context}: [READ_ONLY_FIELD_WRITER] read-only field cannot expose a writer"
                ));
            }
            if !operation_supports_behavior(register, behavior) {
                errors.push(format!(
                    "{field_context}: [FIELD_OPERATION_MISSING] no register operation supports {behavior:?}"
                ));
            }
        }
        let mut enum_names = BTreeSet::new();
        let mut enum_values = BTreeSet::new();
        let value_limit = if field.width == 64 {
            u64::MAX
        } else {
            (1u64 << field.width) - 1
        };
        for value in &field.enum_values {
            if !enum_names.insert(value.name.as_str()) {
                errors.push(format!(
                    "{field_context} enum {}: [ENUM_NAME_UNIQUE] duplicate name",
                    value.name
                ));
            }
            if !enum_values.insert(value.value) {
                errors.push(format!(
                    "{field_context} enum {}: [ENUM_VALUE_UNIQUE] duplicate value {:#x}",
                    value.name, value.value
                ));
            }
            if value.value > value_limit {
                errors.push(format!(
                    "{field_context} enum {}: [ENUM_VALUE_RANGE] value {:#x} exceeds field width",
                    value.name, value.value
                ));
            }
        }
    }
}

fn operation_supports_behavior(register: &Register, behavior: FieldWriteBehavior) -> bool {
    match behavior {
        FieldWriteBehavior::Plain => {
            register.api.modify != ApiExposure::None
                || [
                    register.atomic.set,
                    register.atomic.clear,
                    register.atomic.xor,
                ]
                .into_iter()
                .any(|exposure| exposure == AtomicExposure::Typed)
        }
        FieldWriteBehavior::WriteOneToClear => register.api.w1c != ApiExposure::None,
        FieldWriteBehavior::WriteOneToSet => register.api.w1s != ApiExposure::None,
        FieldWriteBehavior::WriteOneToToggle => register.api.w1t != ApiExposure::None,
        FieldWriteBehavior::SelfClearing | FieldWriteBehavior::Command => {
            register.api.command != ApiExposure::None
        }
        FieldWriteBehavior::ReadOnly | FieldWriteBehavior::Inherit => false,
    }
}

fn validate_core(db: &Database, errors: &mut Vec<String>) {
    let core = &db.core;
    if core.cpuid != 0x412f_c231
        || core.architecture != CoreArchitecture::Armv7M
        || core.implementation != CoreImplementation::CortexM3
        || core.revision != CoreRevision::R2p1
    {
        errors.push(format!(
            "device.toml: core: [CORE_CPUID_IMPLEMENTATION] CPUID {:#010x} is inconsistent with Armv7-M Cortex-M3 r2p1",
            core.cpuid
        ));
    }
    if core.processor_count == 0 {
        errors.push(
            "device.toml: core: [CORE_PROCESSOR_COUNT] processor_count must be at least one".into(),
        );
    }

    let nvic = &core.nvic;
    if nvic.assigned_interrupts > nvic.external_slots {
        errors.push(format!(
            "device.toml: core.nvic: [NVIC_ASSIGNED_CAPACITY] {} assigned interrupts exceed {} external slots",
            nvic.assigned_interrupts, nvic.external_slots
        ));
    }
    if usize::from(nvic.assigned_interrupts) != db.interrupts.len() {
        errors.push(format!(
            "device.toml: core.nvic: [NVIC_ASSIGNED_COUNT] assigned_interrupts {} does not match {} named interrupts",
            nvic.assigned_interrupts,
            db.interrupts.len()
        ));
    }
    if !(1..=8).contains(&nvic.priority_bits) {
        errors.push(format!(
            "device.toml: core.nvic: [NVIC_PRIORITY_BITS] priority_bits {} is outside 1..=8",
            nvic.priority_bits
        ));
    }
    if nvic.verification == Verification::Unverified {
        errors.push(
            "device.toml: core.nvic: [NVIC_VERIFICATION] generated NVIC capability metadata must be verified"
                .into(),
        );
    }

    let mpu = &core.mpu;
    if !mpu.present && (mpu.data_regions != 0 || mpu.instruction_regions != 0) {
        errors.push(
            "device.toml: core.mpu: [MPU_ABSENT_REGIONS] absent MPU must have zero data and instruction regions"
                .into(),
        );
    }
    if mpu.kind == MpuKind::Unified && mpu.instruction_regions != 0 {
        errors.push(
            "device.toml: core.mpu: [MPU_UNIFIED_INSTRUCTION_REGIONS] unified MPU must have zero separate instruction regions"
                .into(),
        );
    }
    if mpu.verification == Verification::Unverified
        || core.dwt.verification == Verification::Unverified
        || core.fpb.verification == Verification::Unverified
    {
        errors.push(
            "device.toml: core: [CORE_CAPABILITY_VERIFICATION] generated MPU, DWT, and FPB capabilities must be verified"
                .into(),
        );
    }
    let enforcement = &mpu.proc0_privileged_shared_sram_data_read;
    if enforcement.processor != 0
        || enforcement.mode != MpuTestMode::PrivilegedThread
        || enforcement.access != MpuTestAccess::DataRead
        || enforcement.target != MpuTestTarget::SharedSram
        || enforcement.region != 7
        || enforcement.region_size_bytes != 32
        || enforcement.permission != MpuTestPermission::NoAccess
        || enforcement.exception_number != 4
    {
        errors.push(
            "device.toml: core.mpu.proc0_privileged_shared_sram_data_read: [MPU_ENFORCEMENT_SCOPE] enforcement metadata must remain scoped to the confirmed proc0 privileged shared-SRAM data-read test"
                .into(),
        );
    }
    for (name, observation) in [
        ("precise_memmanage", &enforcement.precise_memmanage),
        ("data_access_violation", &enforcement.data_access_violation),
        ("fault_address_valid", &enforcement.fault_address_valid),
        ("exact_fault_address", &enforcement.exact_fault_address),
        ("hardfault_escalation", &enforcement.hardfault_escalation),
        ("exact_pc_recovery", &enforcement.exact_pc_recovery),
        ("single_handler_entry", &enforcement.single_handler_entry),
    ] {
        validate_observation(
            &format!("device.toml: core.mpu.proc0_privileged_shared_sram_data_read.{name}"),
            observation,
            errors,
        );
        if observation.class != ObservationClass::ScopedBehavior {
            errors.push(format!(
                "device.toml: core.mpu.proc0_privileged_shared_sram_data_read.{name}: [MPU_ENFORCEMENT_GENERALIZATION] limited enforcement result must be scoped behavior"
            ));
        }
    }

    let systick = &core.systick;
    if systick.calib & 0x00ff_ffff != systick.tenms {
        errors.push(format!(
            "device.toml: core.systick: [SYSTICK_TENMS_CALIB] TENMS {} does not match CALIB[23:0] {}",
            systick.tenms,
            systick.calib & 0x00ff_ffff
        ));
    }
    if systick.reference_present == (systick.calib & (1 << 31) != 0) {
        errors.push(
            "device.toml: core.systick: [SYSTICK_REFERENCE_CALIB] reference_present is inconsistent with CALIB.NOREF"
                .into(),
        );
    }
    if systick.skew != (systick.calib & (1 << 30) != 0) {
        errors.push(
            "device.toml: core.systick: [SYSTICK_SKEW_CALIB] skew is inconsistent with CALIB.SKEW"
                .into(),
        );
    }
    if systick.nominal_reference_hz != (systick.tenms + 1) * 100 {
        errors.push(
            "device.toml: core.systick: [SYSTICK_NOMINAL_FREQUENCY] nominal reference frequency must be the architectural TENMS interpretation"
                .into(),
        );
    }
    if systick.calib_verification != Verification::Hardware
        || systick.frequency_verification != Verification::Architectural
    {
        errors.push(
            "device.toml: core.systick: [SYSTICK_VERIFICATION_CLASS] raw CALIB must be hardware-verified and nominal frequency must be architectural"
                .into(),
        );
    }

    if core.dwt.comparators > 16 {
        errors.push(
            "device.toml: core.dwt: [DWT_COMPARATOR_RANGE] comparator count exceeds the supported metadata range 0..=16"
                .into(),
        );
    }
    if !core.dwt.present
        && (core.dwt.comparators != 0
            || core.dwt.cycle_counter
            || core.dwt.profiling_counters
            || core.dwt.external_match_trigger
            || core.dwt.trace_packets)
    {
        errors.push(
            "device.toml: core.dwt: [DWT_ABSENT_CAPABILITIES] absent DWT cannot declare capabilities"
                .into(),
        );
    }
    if core.fpb.code_comparators > 16 || core.fpb.literal_comparators > 16 {
        errors.push(
            "device.toml: core.fpb: [FPB_COMPARATOR_RANGE] comparator count exceeds the supported metadata range 0..=16"
                .into(),
        );
    }
    if !core.fpb.present && (core.fpb.code_comparators != 0 || core.fpb.literal_comparators != 0) {
        errors.push(
            "device.toml: core.fpb: [FPB_ABSENT_CAPABILITIES] absent FPB cannot declare comparators"
                .into(),
        );
    }

    for (name, observation) in [
        ("hardware_divide", &core.instructions.hardware_divide),
        ("exclusive_load", &core.instructions.exclusive_load),
        ("exclusive_store", &core.instructions.exclusive_store),
        ("public_lookup_table", &db.boot_rom.public_lookup_table),
        (
            "official_firmware_rom_calls",
            &db.boot_rom.official_firmware_rom_calls,
        ),
    ] {
        validate_observation(
            &format!("device.toml: observation {name}"),
            observation,
            errors,
        );
    }
}

fn validate_observation(context: &str, observation: &Observation, errors: &mut Vec<String>) {
    if observation.result == ObservationResult::Contradicted {
        errors.push(format!(
            "{context}: [OBSERVATION_CONTRADICTED] contradicted metadata must block generation"
        ));
    }
    if observation.result == ObservationResult::NotObserved
        && observation.class == ObservationClass::Capability
    {
        errors.push(format!(
            "{context}: [OBSERVATION_NOT_OBSERVED_CAPABILITY] NotObserved cannot be encoded as a global capability or confirmed absence"
        ));
    }
    if observation.class == ObservationClass::Capability && observation.scope.is_some() {
        errors.push(format!(
            "{context}: [OBSERVATION_SCOPED_CAPABILITY] scoped results cannot be generated as global capabilities"
        ));
    }
    if matches!(
        observation.class,
        ObservationClass::ScopedBehavior | ObservationClass::RuntimeState
    ) && observation.scope.as_deref().is_none_or(str::is_empty)
    {
        errors.push(format!(
            "{context}: [OBSERVATION_SCOPE_REQUIRED] scoped behavior and runtime state require an explicit scope"
        ));
    }
    if (observation.class == ObservationClass::ArchitecturalInterpretation)
        != (observation.verification == Verification::Architectural)
    {
        errors.push(format!(
            "{context}: [OBSERVATION_ARCHITECTURAL_CLASS] architectural interpretation and raw hardware verification must remain distinct"
        ));
    }
}

fn validate_rp1_register_contracts(db: &Database, errors: &mut Vec<String>) {
    if let Some(resets) = db
        .peripherals
        .iter()
        .find(|definition| definition.peripheral.name == "RESETS")
    {
        let ctrl0 = resets
            .registers
            .iter()
            .find(|register| register.id == "ctrl0");
        let done0 = resets
            .registers
            .iter()
            .find(|register| register.id == "done0");
        let ctrl_proc1 = ctrl0.and_then(|register| {
            register
                .fields
                .iter()
                .find(|field| field.id == "proc1")
                .map(|field| (register, field))
        });
        let done_proc1 = done0.and_then(|register| {
            register
                .fields
                .iter()
                .find(|field| field.id == "proc1")
                .map(|field| (register, field))
        });
        match (ctrl_proc1, done_proc1) {
            (Some((ctrl, ctrl_field)), Some((done, done_field))) => {
                if ctrl_field.lsb != done_field.lsb || ctrl_field.lsb != 31 {
                    errors.push(
                        "resets.toml: [RESETS_PROC1_BIT_MATCH] CTRL0 PROC1 and DONE0 PROC1 must both use bit 31"
                            .into(),
                    );
                }
                if ctrl_field.api_write != ApiExposure::Public
                    || ctrl_field.effective_write_behavior(ctrl) != FieldWriteBehavior::Plain
                    || !ctrl_field.effective_verified_write(ctrl).permits_write()
                    || ctrl.atomic.set != AtomicExposure::Typed
                    || ctrl.atomic.clear != AtomicExposure::Typed
                {
                    errors.push(
                        "resets.toml: [RESETS_PROC1_CONTROL_WRITE] CTRL0 PROC1 must have verified typed SET/CLEAR control"
                            .into(),
                    );
                }
                if done.hardware_write != WriteBehavior::None
                    || done_field.api_write != ApiExposure::None
                    || done_field.effective_write_behavior(done) != FieldWriteBehavior::ReadOnly
                {
                    errors.push(
                        "resets.toml: [RESETS_PROC1_DONE_WRITE] DONE0 PROC1 must remain read-only"
                            .into(),
                    );
                }
            }
            _ => errors.push(
                "resets.toml: [RESETS_PROC1_REQUIRED] CTRL0 and DONE0 must both define PROC1"
                    .into(),
            ),
        }
    }

    if let Some(watchdog) = db
        .peripherals
        .iter()
        .find(|definition| definition.peripheral.name == "WATCHDOG")
    {
        for (id, offset) in [
            ("boot_magic", 0x0c),
            ("proc0_encoded_entry", 0x10),
            ("proc1_encoded_entry", 0x14),
            ("proc0_initial_sp", 0x18),
            ("proc1_initial_sp", 0x1c),
        ] {
            match watchdog.registers.iter().find(|register| register.id == id) {
                Some(register)
                    if register.offset == offset
                        && register.repeat_count == 1
                        && register.hardware_read == ReadBehavior::Plain
                        && register.hardware_write == WriteBehavior::StateMachineOnly
                        && register.api.read == ApiExposure::Public
                        && !register.api.has_safe_write()
                        && register.api.raw_write == RawWriteExposure::None
                        && register.atomic.set == AtomicExposure::None
                        && register.atomic.clear == AtomicExposure::None
                        && register.atomic.xor == AtomicExposure::None => {}
                _ => errors.push(format!(
                    "watchdog.toml: [WATCHDOG_BOOT_SERVICE_REGISTER] {id} must be an independent state-machine-only register at {offset:#x} without writers or atomic API"
                )),
            }
        }
        match watchdog
            .registers
            .iter()
            .find(|register| register.id == "scratch")
        {
            Some(register)
                if register.offset == 0x20
                    && register.repeat_count == 3
                    && !register.api.has_safe_write()
                    && register.api.raw_write == RawWriteExposure::None
                    && register.atomic.set == AtomicExposure::None
                    && register.atomic.clear == AtomicExposure::None
                    && register.atomic.xor == AtomicExposure::None => {}
            _ => errors.push(
                "watchdog.toml: [WATCHDOG_GENERIC_SCRATCH_RANGE] generic SCRATCH must contain only the three words at 0x20..0x28 and expose no writer"
                    .into(),
            ),
        }
    }

    if let Some(ticks) = db
        .peripherals
        .iter()
        .find(|definition| definition.peripheral.name == "TICKS")
    {
        let mut offsets: Vec<_> = ticks
            .registers
            .iter()
            .map(|register| register.offset)
            .collect();
        offsets.sort_unstable();
        let expected: Vec<_> = (0..8)
            .flat_map(|group| [group * 12, group * 12 + 4, group * 12 + 8])
            .collect();
        if ticks.peripheral.base != 0x4017_4000 || offsets != expected {
            errors.push(
                "ticks.toml: [TICKS_GROUP_LAYOUT] TICKS must be 8 groups of 3 registers with a 12-byte group stride at 0x40174000"
                .into(),
            );
        }
        let initialized_cycles = ticks
            .registers
            .iter()
            .filter(|register| register.offset % 12 == 4)
            .filter(|register| {
                register
                    .boot_rom_initialization
                    .as_ref()
                    .is_some_and(|initialization| initialization.value == 0x32)
            })
            .count();
        if initialized_cycles != 8 {
            errors.push(
                "ticks.toml: [TICKS_BOOT_ROM_INITIALIZATION] all eight cycle registers must record Boot ROM initialization value 0x32 separately from reset_value"
                    .into(),
            );
        }
    }
}

fn validate_interrupts(db: &Database, errors: &mut Vec<String>) {
    let defined_interrupt_count =
        u16::try_from(db.interrupts.len()).expect("interrupt count fits in u16");

    if db.core.nvic.external_slots == 0 {
        errors.push(
            "device.toml: device: [NVIC_INTERRUPT_COUNT_ZERO] NVIC interrupt count must be greater than zero"
                .into(),
        );
    }
    if !db.core.nvic.external_slots.is_multiple_of(32) {
        errors.push(format!(
            "device.toml: device: [NVIC_INTERRUPT_COUNT_ALIGNMENT] NVIC interrupt count {} is not a multiple of 32",
            db.core.nvic.external_slots
        ));
    }
    if defined_interrupt_count > db.core.nvic.external_slots {
        errors.push(format!(
            "device.toml: device: [NVIC_INTERRUPT_COUNT_TOO_SMALL] NVIC interrupt count {} is smaller than the {defined_interrupt_count} defined interrupts",
            db.core.nvic.external_slots
        ));
    }

    let mut numbers = BTreeMap::new();
    let mut names = BTreeSet::new();
    for interrupt in &db.interrupts {
        if !names.insert(interrupt.name.as_str()) {
            errors.push(format!(
                "device.toml: interrupt {}: [INTERRUPT_NAME_UNIQUE] duplicate name",
                interrupt.name
            ));
        }
        if let Some(previous) = numbers.insert(interrupt.number, interrupt.name.as_str()) {
            errors.push(format!(
                "device.toml: interrupt {}: [INTERRUPT_NUMBER_UNIQUE] number {} duplicates {previous}",
                interrupt.name, interrupt.number
            ));
        }
        if interrupt.verified == Verification::Unverified {
            errors.push(format!(
                "device.toml: interrupt {}: [INTERRUPT_VERIFIED] unverified interrupt cannot be generated",
                interrupt.name
            ));
        }
        if interrupt.number >= db.core.nvic.external_slots {
            errors.push(format!(
                "device.toml: interrupt {}: [INTERRUPT_NUMBER_NVIC_RANGE] number {} is outside the {} implemented NVIC external interrupt slots",
                interrupt.name,
                interrupt.number,
                db.core.nvic.external_slots
            ));
        }
        if interrupt.number >= defined_interrupt_count {
            errors.push(format!(
                "device.toml: interrupt {}: [INTERRUPT_NUMBER_ASSIGNED_RANGE] number {} is outside the contiguous named interrupt range 0..{}",
                interrupt.name,
                interrupt.number,
                defined_interrupt_count
            ));
        }
    }

    for number in 0..defined_interrupt_count {
        if !numbers.contains_key(&number) {
            errors.push(format!(
                "device.toml: [INTERRUPT_NUMBER_COMPLETE] missing named interrupt number {number}"
            ));
        }
    }
}

fn validate_metadata(db: &Database, errors: &mut Vec<String>) {
    let mut clock_names = BTreeSet::new();
    let mut clock_ids = BTreeMap::new();
    for clock in &db.clocks {
        if !clock_names.insert(clock.name.as_str()) {
            errors.push(format!(
                "device.toml: clock {}: [CLOCK_NAME_UNIQUE] duplicate name",
                clock.name
            ));
        }
        if let Some(previous) = clock_ids.insert(clock.id, clock.name.as_str()) {
            errors.push(format!(
                "device.toml: clock {}: [CLOCK_ID_UNIQUE] id {} duplicates {previous}",
                clock.name, clock.id
            ));
        }
        if clock.verified == Verification::Unverified {
            errors.push(format!(
                "device.toml: clock {}: [CLOCK_VERIFIED] unverified clock ID cannot be generated",
                clock.name
            ));
        }
    }

    let mut bank_ids = BTreeSet::new();
    let mut gpio_ranges = Vec::new();
    for bank in &db.gpio_banks {
        if !bank_ids.insert(bank.id) {
            errors.push(format!(
                "device.toml: GPIO bank {}: [GPIO_BANK_ID_UNIQUE] duplicate bank id",
                bank.id
            ));
        }
        if bank.gpio_count == 0 {
            errors.push(format!(
                "device.toml: GPIO bank {}: [GPIO_BANK_COUNT] gpio_count must be nonzero",
                bank.id
            ));
        }
        let end = u16::from(bank.first_gpio) + u16::from(bank.gpio_count);
        for (previous_start, previous_end, previous_id) in &gpio_ranges {
            if u16::from(bank.first_gpio) < *previous_end && *previous_start < end {
                errors.push(format!(
                    "device.toml: GPIO bank {}: [GPIO_BANK_RANGE] overlaps bank {previous_id}",
                    bank.id
                ));
            }
        }
        gpio_ranges.push((u16::from(bank.first_gpio), end, bank.id));
    }

    let mut spaces = Vec::new();
    let mut space_names = BTreeSet::new();
    for space in &db.address_spaces {
        if !space_names.insert(space.name.as_str()) {
            errors.push(format!(
                "device.toml: address space {}: [ADDRESS_SPACE_NAME_UNIQUE] duplicate name",
                space.name
            ));
        }
        if space.size == 0 {
            errors.push(format!(
                "device.toml: address space {}: [ADDRESS_SPACE_SIZE] size must be nonzero",
                space.name
            ));
            continue;
        }
        let Some(end) = space.local_base.checked_add(space.size) else {
            errors.push(format!(
                "device.toml: address space {}: [ADDRESS_SPACE_RANGE] range overflows",
                space.name
            ));
            continue;
        };
        for (previous_start, previous_end, previous_name) in &spaces {
            if space.local_base < *previous_end && *previous_start < end {
                errors.push(format!(
                    "device.toml: address space {}: [ADDRESS_SPACE_OVERLAP] overlaps {previous_name}",
                    space.name
                ));
            }
        }
        spaces.push((space.local_base, end, space.name.as_str()));
    }

    if let Some(endpoint) = &db.pci_endpoint {
        if endpoint.vendor_id == 0 || endpoint.device_id == 0 {
            errors.push(
                "device.toml: PCI endpoint: [PCI_ID_NONZERO] vendor_id and device_id must be nonzero"
                    .into(),
            );
        }
        if endpoint.verified == Verification::Unverified {
            errors.push(
                "device.toml: PCI endpoint: [PCI_ID_VERIFIED] unverified identity cannot be generated"
                    .into(),
            );
        }
    }
}
