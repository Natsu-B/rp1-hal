use std::path::PathBuf;

use rp1_regdb::{load::load_database, model::*, validate::validate};

fn database() -> Database {
    load_database(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/device.toml")).unwrap()
}

#[test]
fn rejects_inferred_public_writer_with_source_context() {
    let mut db = database();
    let resets = db
        .peripherals
        .iter_mut()
        .find(|p| p.peripheral.name == "RESETS")
        .unwrap();
    let ctrl0 = resets
        .registers
        .iter_mut()
        .find(|r| r.name == "CTRL0")
        .unwrap();
    let candidate = ctrl0
        .fields
        .iter_mut()
        .find(|f| f.name == "CANDIDATE_BIT6")
        .unwrap();
    candidate.api_write = ApiExposure::Public;

    let errors = validate(&db).unwrap_err().join("\n");
    assert!(errors.contains("resets.toml"));
    assert!(errors.contains("RESETS register CTRL0 field CANDIDATE_BIT6"));
    assert!(errors.contains("inferred field"));
}

#[test]
fn rejects_atomic_alias_on_command_register() {
    let mut db = database();
    let watchdog = db
        .peripherals
        .iter_mut()
        .find(|p| p.peripheral.name == "WATCHDOG")
        .unwrap();
    let load = watchdog
        .registers
        .iter_mut()
        .find(|r| r.name == "LOAD")
        .unwrap();
    load.atomic.kind = AtomicKind::Rp1Native;
    load.atomic.set = AtomicExposure::Typed;

    let errors = validate(&db).unwrap_err().join("\n");
    assert!(errors.contains("WATCHDOG register LOAD"));
    assert!(errors.contains("ATOMIC_PLAIN_WRITE_ONLY"));
}

#[test]
fn rejects_missing_custom_view_and_duplicate_interrupt() {
    let mut db = database();
    let pcie = db
        .peripherals
        .iter_mut()
        .find(|p| p.peripheral.name == "PCIE_APBS")
        .unwrap();
    pcie.registers
        .iter_mut()
        .find(|register| register.name == "MSIX_CFG")
        .unwrap()
        .atomic
        .set_view = Some("missing".into());
    db.interrupts.push(
        db.interrupts
            .iter()
            .find(|i| i.name == "UART0")
            .unwrap()
            .clone(),
    );

    let errors = validate(&db).unwrap_err().join("\n");
    assert!(errors.contains("missing or wrong-kind custom Set view"));
    assert!(errors.contains("duplicate name"));
    assert!(errors.contains("duplicates UART0"));
}

#[test]
fn rejects_missing_rp1_interrupt_number() {
    let mut db = database();
    db.interrupts.retain(|interrupt| interrupt.number != 53);
    let errors = validate(&db).unwrap_err().join("\n");
    assert!(errors.contains("[INTERRUPT_NUMBER_ASSIGNED_RANGE] number 60"));
    assert!(errors.contains("[INTERRUPT_NUMBER_COMPLETE] missing named interrupt number 53"));
}

#[test]
fn rejects_interrupt_outside_named_assignment_range() {
    let mut db = database();
    db.interrupts
        .iter_mut()
        .find(|i| i.name == "PCIE_OUT")
        .unwrap()
        .number = 61;
    let errors = validate(&db).unwrap_err().join("\n");
    assert!(errors.contains("[INTERRUPT_NUMBER_ASSIGNED_RANGE] number 61"));
    assert!(!errors.contains("[INTERRUPT_NUMBER_NVIC_RANGE] number 61"));
    assert!(errors.contains("[INTERRUPT_NUMBER_COMPLETE] missing named interrupt number 53"));
}

#[test]
fn accepts_61_named_interrupts_in_64_nvic_slots() {
    let db = database();
    assert_eq!(db.interrupts.len(), 61);
    assert_eq!(db.core.nvic.external_slots, 64);
    validate(&db).unwrap();
}

#[test]
fn rejects_zero_nvic_interrupt_count() {
    let mut db = database();
    db.core.nvic.external_slots = 0;
    let errors = validate(&db).unwrap_err().join("\n");
    assert!(errors.contains("[NVIC_INTERRUPT_COUNT_ZERO]"));
    assert!(errors.contains("[NVIC_INTERRUPT_COUNT_TOO_SMALL]"));
}

#[test]
fn rejects_unaligned_nvic_interrupt_count() {
    let mut db = database();
    db.core.nvic.external_slots = 63;
    let errors = validate(&db).unwrap_err().join("\n");
    assert!(errors.contains("[NVIC_INTERRUPT_COUNT_ALIGNMENT]"));
}

#[test]
fn rejects_nvic_interrupt_count_smaller_than_named_interrupts() {
    let mut db = database();
    db.core.nvic.external_slots = 32;
    let errors = validate(&db).unwrap_err().join("\n");
    assert!(errors.contains("[NVIC_INTERRUPT_COUNT_TOO_SMALL]"));
}

#[test]
fn rejects_interrupt_outside_nvic_range() {
    let mut db = database();
    db.interrupts
        .iter_mut()
        .find(|i| i.name == "PCIE_OUT")
        .unwrap()
        .number = 64;
    let errors = validate(&db).unwrap_err().join("\n");
    assert!(errors.contains("[INTERRUPT_NUMBER_NVIC_RANGE] number 64"));
    assert!(errors.contains("[INTERRUPT_NUMBER_ASSIGNED_RANGE] number 64"));
}

#[test]
fn rejects_stride_and_public_count_mismatch() {
    let mut db = database();
    let pcie = db
        .peripherals
        .iter_mut()
        .find(|p| p.peripheral.name == "PCIE_APBS")
        .unwrap();
    pcie.registers[0].repeat_stride = 6;
    pcie.registers[0].public_count = Some(65);

    let errors = validate(&db).unwrap_err().join("\n");
    assert!(errors.contains("REPEAT_STRIDE"));
    assert!(errors.contains("PUBLIC_COUNT"));
}

#[test]
fn rejects_modify_on_mixed_semantics_register() {
    let mut db = database();
    let watchdog = db
        .peripherals
        .iter_mut()
        .find(|p| p.peripheral.name == "WATCHDOG")
        .unwrap();
    watchdog.registers[0].api.modify = ApiExposure::Public;

    let errors = validate(&db).unwrap_err().join("\n");
    assert!(errors.contains("MODIFY_MIXED_SEMANTICS"));
}

#[test]
fn watchdog_pause_fields_keep_debug_source_bit_mapping() {
    let db = database();
    let watchdog = db
        .peripherals
        .iter()
        .find(|p| p.peripheral.name == "WATCHDOG")
        .unwrap();

    for (name, lsb) in [("PAUSE_JTAG", 24), ("PAUSE_DBG0", 25), ("PAUSE_DBG1", 26)] {
        let field = watchdog.registers[0]
            .fields
            .iter()
            .find(|field| field.name == name)
            .unwrap();
        assert_eq!(field.lsb, lsb);
    }
}

#[test]
fn rejects_unverified_field_writer() {
    let mut db = database();
    let watchdog = db
        .peripherals
        .iter_mut()
        .find(|p| p.peripheral.name == "WATCHDOG")
        .unwrap();
    let pause = watchdog.registers[0]
        .fields
        .iter_mut()
        .find(|field| field.name == "PAUSE_DBG0")
        .unwrap();
    pause.api_write = ApiExposure::Public;

    let errors = validate(&db).unwrap_err().join("\n");
    assert!(errors.contains("UNVERIFIED_FIELD_SAFE_WRITER"));
}

#[test]
fn rejects_state_machine_and_unknown_raw_writers() {
    let mut db = database();
    let timer = db
        .peripherals
        .iter_mut()
        .find(|p| p.peripheral.name == "TIMER")
        .unwrap();
    timer.registers[0].api.raw_write = RawWriteExposure::Unsafe;
    let mut raw = timer.registers[0].clone();
    raw.id = "raw".into();
    raw.name = "RAW".into();
    raw.offset = 0x100;
    raw.repeat_count = 8;
    raw.hardware_write = WriteBehavior::Plain;
    raw.api.raw_write = RawWriteExposure::Unsafe;
    timer.registers.push(raw);

    let errors = validate(&db).unwrap_err().join("\n");
    assert!(errors.contains("RAW_WRITE_ON_NONWRITABLE"));
    assert!(errors.contains("UNKNOWN_RAW_PUBLIC_WRITE"));
}

#[test]
fn rejects_otp_writers_and_atomic_aliases() {
    let mut db = database();
    let resets = db
        .peripherals
        .iter_mut()
        .find(|p| p.peripheral.name == "RESETS")
        .unwrap();
    resets.peripheral.id = "otp".into();
    resets.peripheral.name = "OTP".into();

    let errors = validate(&db).unwrap_err().join("\n");
    assert!(errors.contains("OTP_EFUSE_WRITE_FORBIDDEN"));
    assert!(errors.contains("OTP_EFUSE_ATOMIC_FORBIDDEN"));
}

fn register_mut<'a>(db: &'a mut Database, peripheral: &str, register: &str) -> &'a mut Register {
    db.peripherals
        .iter_mut()
        .find(|definition| definition.peripheral.name == peripheral)
        .unwrap()
        .registers
        .iter_mut()
        .find(|candidate| candidate.name == register)
        .unwrap()
}

#[test]
fn verified_plain_field_does_not_authorize_command_or_w1c() {
    let mut command_db = database();
    register_mut(&mut command_db, "RESETS", "CTRL1").api.command = ApiExposure::Public;
    let command_errors = validate(&command_db).unwrap_err().join("\n");
    assert!(command_errors.contains("resets.toml"));
    assert!(command_errors.contains("RESETS register CTRL1"));
    assert!(command_errors.contains("operation command"));
    assert!(command_errors.contains("COMMAND_FIELD_REQUIRED"));

    let mut w1c_db = database();
    register_mut(&mut w1c_db, "RESETS", "CTRL1").api.w1c = ApiExposure::Public;
    let w1c_errors = validate(&w1c_db).unwrap_err().join("\n");
    assert!(w1c_errors.contains("operation w1c"));
    assert!(w1c_errors.contains("W1C_FIELD_REQUIRED"));
}

#[test]
fn verified_plain_field_does_not_authorize_reset() {
    let mut db = database();
    let ctrl1 = register_mut(&mut db, "RESETS", "CTRL1");
    ctrl1.verified_write = Verification::Unverified;
    ctrl1.reset_value = Some(0);
    ctrl1.api.reset = ApiExposure::Public;

    let errors = validate(&db).unwrap_err().join("\n");
    assert!(errors.contains("operation reset"));
    assert!(errors.contains("RESET_OPERATION_UNVERIFIED"));
}

#[test]
fn verified_command_and_w1c_fields_authorize_matching_operations() {
    let db = database();
    let watchdog = db
        .peripherals
        .iter()
        .find(|definition| definition.peripheral.name == "WATCHDOG")
        .unwrap();
    assert!(
        watchdog
            .registers
            .iter()
            .any(|register| register.name == "LOAD" && register.api.command == ApiExposure::Public)
    );
    let timer = db
        .peripherals
        .iter()
        .find(|definition| definition.peripheral.name == "TIMER")
        .unwrap();
    assert!(
        timer
            .registers
            .iter()
            .any(|register| register.name == "INTR" && register.api.w1c == ApiExposure::Public)
    );
    validate(&db).unwrap();
}

#[test]
fn typed_atomic_rejects_unverified_selected_field() {
    let mut db = database();
    let ctrl1 = register_mut(&mut db, "RESETS", "CTRL1");
    let uart0 = ctrl1
        .fields
        .iter_mut()
        .find(|field| field.name == "UART0")
        .unwrap();
    uart0.verified_write = Some(Verification::Unverified);

    let errors = validate(&db).unwrap_err().join("\n");
    assert!(errors.contains("RESETS register CTRL1 field UART0"));
    assert!(errors.contains("operation atomic"));
}

#[test]
fn removed_full_and_zero_write_schema_is_rejected() {
    assert!(toml::from_str::<RegisterApi>("write_full = \"public\"").is_err());
    assert!(toml::from_str::<RegisterApi>("write_zero = \"public\"").is_err());
}

#[test]
fn rejects_inconsistent_core_capabilities() {
    let mut cpuid_db = database();
    cpuid_db.core.cpuid ^= 1;
    assert!(
        validate(&cpuid_db)
            .unwrap_err()
            .join("\n")
            .contains("CORE_CPUID_IMPLEMENTATION")
    );

    let mut processor_db = database();
    processor_db.core.processor_count = 0;
    assert!(
        validate(&processor_db)
            .unwrap_err()
            .join("\n")
            .contains("CORE_PROCESSOR_COUNT")
    );

    let mut priority_db = database();
    priority_db.core.nvic.priority_bits = 0;
    assert!(
        validate(&priority_db)
            .unwrap_err()
            .join("\n")
            .contains("NVIC_PRIORITY_BITS")
    );
}

#[test]
fn rejects_assigned_interrupt_metadata_over_capacity_or_out_of_sync() {
    let mut db = database();
    db.core.nvic.assigned_interrupts = 65;
    let errors = validate(&db).unwrap_err().join("\n");
    assert!(errors.contains("NVIC_ASSIGNED_CAPACITY"));
    assert!(errors.contains("NVIC_ASSIGNED_COUNT"));
}

#[test]
fn rejects_invalid_mpu_shape() {
    let mut absent_db = database();
    absent_db.core.mpu.present = false;
    let absent_errors = validate(&absent_db).unwrap_err().join("\n");
    assert!(absent_errors.contains("MPU_ABSENT_REGIONS"));

    let mut unified_db = database();
    unified_db.core.mpu.instruction_regions = 1;
    let unified_errors = validate(&unified_db).unwrap_err().join("\n");
    assert!(unified_errors.contains("MPU_UNIFIED_INSTRUCTION_REGIONS"));
}

#[test]
fn rejects_systick_calib_and_interpretation_mismatch() {
    let mut db = database();
    db.core.systick.tenms = 5000;
    db.core.systick.frequency_verification = Verification::Hardware;
    let errors = validate(&db).unwrap_err().join("\n");
    assert!(errors.contains("SYSTICK_TENMS_CALIB"));
    assert!(errors.contains("SYSTICK_NOMINAL_FREQUENCY"));
    assert!(errors.contains("SYSTICK_VERIFICATION_CLASS"));
}

#[test]
fn rejects_unsafe_observation_generalization() {
    let mut contradicted = database();
    contradicted.core.instructions.hardware_divide.result = ObservationResult::Contradicted;
    assert!(
        validate(&contradicted)
            .unwrap_err()
            .join("\n")
            .contains("OBSERVATION_CONTRADICTED")
    );

    let mut absent = database();
    absent.boot_rom.public_lookup_table.class = ObservationClass::Capability;
    absent.boot_rom.public_lookup_table.scope = None;
    let absent_errors = validate(&absent).unwrap_err().join("\n");
    assert!(absent_errors.contains("OBSERVATION_NOT_OBSERVED_CAPABILITY"));

    let mut unscoped = database();
    unscoped.core.instructions.exclusive_store.scope = None;
    assert!(
        validate(&unscoped)
            .unwrap_err()
            .join("\n")
            .contains("OBSERVATION_SCOPE_REQUIRED")
    );
}

#[test]
fn confirmed_absent_is_not_an_observation_result() {
    assert!(
        toml::from_str::<Observation>(
            "verification = \"hardware\"\nresult = \"confirmed-absent\"\nclass = \"capability\""
        )
        .is_err()
    );
}

#[test]
fn rejects_state_machine_writers_and_atomic_operations() {
    let mut db = database();
    let boot_magic = register_mut(&mut db, "WATCHDOG", "BOOT_MAGIC");
    boot_magic.api.modify = ApiExposure::Public;
    boot_magic.api.raw_write = RawWriteExposure::Unsafe;
    boot_magic.atomic.kind = AtomicKind::Rp1Native;
    boot_magic.atomic.set = AtomicExposure::Typed;
    let errors = validate(&db).unwrap_err().join("\n");
    assert!(errors.contains("STATE_MACHINE_PUBLIC_WRITE"));
    assert!(errors.contains("STATE_MACHINE_RAW_WRITE_FORBIDDEN"));
    assert!(errors.contains("STATE_MACHINE_ATOMIC_FORBIDDEN"));
}

#[test]
fn rejects_proc1_done_writer_and_bit_mismatch() {
    let mut db = database();
    let done0 = register_mut(&mut db, "RESETS", "DONE0");
    let proc1 = done0
        .fields
        .iter_mut()
        .find(|field| field.name == "PROC1")
        .unwrap();
    proc1.api_write = ApiExposure::Public;
    proc1.lsb = 30;
    let errors = validate(&db).unwrap_err().join("\n");
    assert!(errors.contains("RESETS_PROC1_BIT_MATCH"));
    assert!(errors.contains("RESETS_PROC1_DONE_WRITE"));
}

#[test]
fn rejects_boot_rom_initialization_as_hardware_reset_value() {
    let mut db = database();
    register_mut(&mut db, "TICKS", "TIMER_CYCLES").reset_value = Some(0x32);
    let errors = validate(&db).unwrap_err().join("\n");
    assert!(errors.contains("BOOT_ROM_INITIALIZATION_AS_RESET"));
}

#[test]
fn rejects_boot_service_words_in_generic_scratch_array() {
    let mut db = database();
    let scratch = register_mut(&mut db, "WATCHDOG", "SCRATCH");
    scratch.offset = 0x0c;
    scratch.repeat_count = 8;
    let errors = validate(&db).unwrap_err().join("\n");
    assert!(errors.contains("WATCHDOG_GENERIC_SCRATCH_RANGE"));
}
