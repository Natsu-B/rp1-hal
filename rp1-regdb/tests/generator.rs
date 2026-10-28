use std::{fs, path::PathBuf, process::Command};

use rp1_regdb::{gen_rust, load::load_database, validate::validate};

fn database() -> rp1_regdb::model::Database {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/device.toml");
    let database = load_database(&path).unwrap();
    validate(&database).unwrap();
    database
}

fn generated() -> (tempfile::TempDir, PathBuf) {
    let directory = tempfile::tempdir().unwrap();
    let source = directory.path().join("rp1-pac/src");
    fs::create_dir_all(&source).unwrap();
    gen_rust::generate(&database(), &source).unwrap();
    (directory, source)
}

#[test]
fn timer_raw_high_is_readable_and_pause_is_not_writable() {
    let (_directory, source) = generated();
    let timer = fs::read_to_string(source.join("generated/timer.rs")).unwrap();
    assert!(timer.contains("impl crate::Readable for RAW_HIGH_SPEC"));
    assert!(!timer.contains("impl crate::Modifiable for PAUSE_SPEC"));
    assert!(!timer.contains("impl crate::Modifiable for PAUSE_SPEC"));
    assert!(!timer.contains("impl crate::WriteOneToClear for ARMED_SPEC"));
    assert!(timer.contains("impl crate::WriteOneToClear for INTR_SPEC"));
    assert!(!timer.contains("impl crate::Modifiable for INTR_SPEC"));
}

#[test]
fn watchdog_load_has_no_atomic_method() {
    let (_directory, source) = generated();
    let watchdog = fs::read_to_string(source.join("generated/watchdog.rs")).unwrap();
    let load = watchdog
        .split("pub mod load")
        .nth(1)
        .unwrap()
        .split("\n}\n")
        .next()
        .unwrap();
    assert!(load.contains("CommandWritable"));
    assert!(!load.contains("Modifiable"));
    assert!(!load.contains("AtomicSet"));
    assert!(!load.contains("AtomicClear"));
    assert!(!load.contains("AtomicXor"));
}

#[test]
fn watchdog_ctrl_only_exposes_verified_enable_writer() {
    let (_directory, source) = generated();
    let watchdog = fs::read_to_string(source.join("generated/watchdog.rs")).unwrap();
    assert!(watchdog.contains("impl crate::TypedAtomicSet for CTRL_SPEC"));
    assert!(watchdog.contains("impl crate::TypedAtomicClear for CTRL_SPEC"));
    assert!(watchdog.contains("pub fn enable(&mut self)"));
    assert!(!watchdog.contains("pub fn pause_dbg0(&mut self)"));
    assert!(!watchdog.contains("pub fn trigger(&mut self)"));
}

#[test]
fn resets_proc1_control_and_done_are_generated_safely() {
    let (_directory, source) = generated();
    let resets = fs::read_to_string(source.join("generated/resets.rs")).unwrap();
    assert!(resets.contains("impl crate::TypedAtomicSet for CTRL0_SPEC"));
    assert!(resets.contains("impl crate::TypedAtomicClear for CTRL0_SPEC"));
    assert!(resets.contains("pub fn proc1(&mut self)"));
    assert!(resets.contains("Processor 1 reset control"));
    assert!(resets.contains("Processor 1 reset completion"));
    assert!(!resets.contains("PROC1_OR_PLL_CANDIDATE"));
    assert!(!resets.contains("proc1_or_pll_candidate"));

    let done0 = resets
        .split("pub mod done0")
        .nth(1)
        .unwrap()
        .split("pub type DONE1")
        .next()
        .unwrap();
    assert!(done0.contains("pub type R"));
    assert!(!done0.contains("pub type W"));
    assert!(!done0.contains("pub fn proc1(&mut self)"));
}

#[test]
fn watchdog_boot_service_words_are_read_only_independent_registers() {
    let (_directory, source) = generated();
    let watchdog = fs::read_to_string(source.join("generated/watchdog.rs")).unwrap();
    for name in [
        "BOOT_MAGIC",
        "PROC0_ENCODED_ENTRY",
        "PROC1_ENCODED_ENTRY",
        "PROC0_INITIAL_SP",
        "PROC1_INITIAL_SP",
    ] {
        assert!(watchdog.contains(&format!("pub type {name} =")));
    }
    for module in [
        "boot_magic",
        "proc0_encoded_entry",
        "proc1_encoded_entry",
        "proc0_initial_sp",
        "proc1_initial_sp",
    ] {
        let body = watchdog
            .split(&format!("pub mod {module}"))
            .nth(1)
            .unwrap()
            .split("\n}\n")
            .next()
            .unwrap();
        assert!(body.contains("impl crate::Readable"));
        assert!(!body.contains("Modifiable"));
        assert!(!body.contains("AtomicSet"));
        assert!(!body.contains("AtomicClear"));
        assert!(!body.contains("AtomicXor"));
        assert!(!body.contains("UnsafeWritable"));
    }
    assert!(watchdog.contains("[SCRATCH; 3]"));
    assert!(!watchdog.contains("[SCRATCH; 8]"));
    assert!(watchdog.contains("0xb007c0de"));
    assert!(watchdog.contains("0x4ff83f2d"));
}

#[test]
fn resets_ctrl1_has_native_atomic_methods_without_alias_ownership() {
    let (_directory, source) = generated();
    let resets = fs::read_to_string(source.join("generated/resets.rs")).unwrap();
    let peripherals = fs::read_to_string(source.join("peripherals.rs")).unwrap();
    assert!(resets.contains("impl crate::TypedAtomicSet for CTRL1_SPEC"));
    assert!(resets.contains("impl crate::TypedAtomicClear for CTRL1_SPEC"));
    assert!(resets.contains("impl crate::TypedAtomicXor for CTRL1_SPEC"));
    assert!(resets.contains("impl crate::UnsafeAtomicSet for CTRL1_SPEC"));
    assert!(resets.contains("impl crate::Modifiable for CTRL1_SPEC"));
    assert!(resets.contains("pub type AtomicSetW = crate::AtomicSetW<CTRL1_SPEC>"));
    assert!(resets.contains("pub type AtomicClearW = crate::AtomicClearW<CTRL1_SPEC>"));
    assert!(resets.contains("pub type AtomicXorW = crate::AtomicXorW<CTRL1_SPEC>"));
    assert!(!peripherals.contains("RESETS_SET"));
    assert!(!peripherals.contains("RESETS_CLR"));
    assert!(!peripherals.contains("RESETS_XOR"));
}

#[test]
fn pcie_msix_uses_custom_windows() {
    let (_directory, source) = generated();
    let pcie = fs::read_to_string(source.join("generated/pcie_apbs.rs")).unwrap();
    assert!(pcie.contains("impl crate::TypedAtomicSet for MSIX_CFG_SPEC"));
    assert!(pcie.contains("const OFFSET: usize = 0x800"));
    assert!(pcie.contains("impl crate::TypedAtomicClear for MSIX_CFG_SPEC"));
    assert!(pcie.contains("const OFFSET: usize = 0xc00"));
    assert!(!pcie.contains("AtomicXor for MSIX_CFG_SPEC"));
    assert!(pcie.contains("assert!(index < 61)"));
    assert!(pcie.contains("[MSIX_CFG; 64]"));
}

#[test]
fn inferred_field_has_no_writer() {
    let (_directory, source) = generated();
    let resets = fs::read_to_string(source.join("generated/resets.rs")).unwrap();
    let candidate = resets
        .split("pub mod candidate_bit6")
        .nth(1)
        .unwrap()
        .split("    }")
        .next()
        .unwrap();
    assert!(candidate.contains("pub type R"));
    assert!(!candidate.contains("pub type W"));
    assert!(!candidate.contains("pub type ASW"));
    assert!(!candidate.contains("pub type ACW"));
    assert!(!candidate.contains("pub type AXW"));
}

#[test]
fn atomic_writer_vocabulary_is_operation_specific() {
    let (_directory, source) = generated();
    let resets = fs::read_to_string(source.join("generated/resets.rs")).unwrap();

    assert!(resets.contains("crate::AtomicSetBitWriter"));
    assert!(resets.contains("crate::AtomicClearBitWriter"));
    assert!(resets.contains("crate::AtomicXorBitWriter"));
    assert!(!resets.contains("pub type AtomicW"));
}

#[test]
fn generated_rustdoc_explains_restricted_operations() {
    let (_directory, source) = generated();
    let timer = fs::read_to_string(source.join("generated/timer.rs")).unwrap();
    let watchdog = fs::read_to_string(source.join("generated/watchdog.rs")).unwrap();
    let resets = fs::read_to_string(source.join("generated/resets.rs")).unwrap();

    assert!(timer.contains("PAUSE"));
    assert!(timer.contains("Write API is not exposed"));
    assert!(timer.contains("Write-one-to-clear"));
    assert!(watchdog.contains("Command write"));
    assert!(resets.contains("inferred"));
    assert!(resets.contains("Typed write API is not exposed"));
}

#[test]
fn generated_output_is_stable_and_has_source_banner() {
    let (_first_directory, first) = generated();
    let (_second_directory, second) = generated();
    for relative in [
        "generated/mod.rs",
        "generated/timer.rs",
        "generated/watchdog.rs",
        "generated/resets.rs",
        "peripherals.rs",
        "interrupts.rs",
        "metadata.rs",
    ] {
        assert_eq!(
            fs::read(first.join(relative)).unwrap(),
            fs::read(second.join(relative)).unwrap()
        );
    }
    assert_eq!(
        fs::read(first.parent().unwrap().join("device.x")).unwrap(),
        fs::read(second.parent().unwrap().join("device.x")).unwrap()
    );
    let timer = fs::read_to_string(first.join("generated/timer.rs")).unwrap();
    assert!(timer.contains("Generated from rp1-regdb/data/peripherals/timer.toml"));
}

#[test]
fn all_rp1_interrupts_are_generated() {
    let (_directory, source) = generated();
    let interrupts = fs::read_to_string(source.join("interrupts.rs")).unwrap();
    let peripherals = fs::read_to_string(source.join("peripherals.rs")).unwrap();
    let expected = [
        ("IO_BANK0", 0),
        ("IO_BANK1", 1),
        ("IO_BANK2", 2),
        ("AUDIO_IN", 3),
        ("AUDIO_OUT", 4),
        ("PWM0", 5),
        ("ETH", 6),
        ("I2C0", 7),
        ("I2C1", 8),
        ("I2C2", 9),
        ("I2C3", 10),
        ("I2C4", 11),
        ("I2C5", 12),
        ("I2C6", 13),
        ("I2S0", 14),
        ("I2S1", 15),
        ("I2S2", 16),
        ("SDIO0", 17),
        ("SDIO1", 18),
        ("SPI0", 19),
        ("SPI1", 20),
        ("SPI2", 21),
        ("SPI3", 22),
        ("SPI4", 23),
        ("SPI5", 24),
        ("UART0", 25),
        ("TIMER_0", 26),
        ("TIMER_1", 27),
        ("TIMER_2", 28),
        ("TIMER_3", 29),
        ("USBHOST0", 30),
        ("USBHOST0_0", 31),
        ("USBHOST0_1", 32),
        ("USBHOST0_2", 33),
        ("USBHOST0_3", 34),
        ("USBHOST1", 35),
        ("USBHOST1_0", 36),
        ("USBHOST1_1", 37),
        ("USBHOST1_2", 38),
        ("USBHOST1_3", 39),
        ("DMA", 40),
        ("PWM1", 41),
        ("UART1", 42),
        ("UART2", 43),
        ("UART3", 44),
        ("UART4", 45),
        ("UART5", 46),
        ("MIPI0", 47),
        ("MIPI1", 48),
        ("VIDEO_OUT", 49),
        ("PIO_0", 50),
        ("PIO_1", 51),
        ("ADC_FIFO", 52),
        ("PCIE_OUT", 53),
        ("SPI6", 54),
        ("SPI7", 55),
        ("SPI8", 56),
        ("PROC_MISC", 57),
        ("SYSCFG", 58),
        ("CLOCKS_DEFAULT", 59),
        ("VBUSCTRL", 60),
    ];
    for (name, number) in expected {
        assert!(
            interrupts.contains(&format!("{name} = {number}")),
            "missing interrupt {name} = {number}"
        );
    }
    assert_eq!(
        interrupts
            .lines()
            .filter(|line| line.starts_with("    ") && line.contains(" = "))
            .count(),
        61
    );
    assert!(
        interrupts
            .lines()
            .any(|line| line == "pub const NVIC_PRIO_BITS: u8 = 3;")
    );
    assert!(
        interrupts
            .lines()
            .any(|line| line == "pub const NVIC_INTERRUPT_COUNT: usize = 64;")
    );
    assert!(
        interrupts
            .lines()
            .any(|line| line == "pub const DEFINED_INTERRUPT_COUNT: usize = 61;")
    );
    assert!(interrupts.contains("pub static __INTERRUPTS: [Vector; NVIC_INTERRUPT_COUNT] = ["));
    assert!(interrupts.contains("#[unsafe(no_mangle)]"));
    assert!(interrupts.contains("#[unsafe(link_section = \".vector_table.interrupts\")]"));
    assert_eq!(
        interrupts.matches("#[cfg(feature = \"vectors\")]").count(),
        3
    );
    assert!(!interrupts.contains("#[cfg(feature = \"rt\")]"));

    let vectors = interrupts
        .split("pub static __INTERRUPTS")
        .nth(1)
        .expect("generated vector table");
    let irq_comments: Vec<_> = vectors
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with("// IRQ "))
        .collect();
    assert_eq!(irq_comments.len(), 64);
    assert_eq!(irq_comments[60], "// IRQ 60: VBUSCTRL");
    assert_eq!(irq_comments[61], "// IRQ 61: reserved");
    assert_eq!(irq_comments[62], "// IRQ 62: reserved");
    assert_eq!(irq_comments[63], "// IRQ 63: reserved");
    assert_eq!(
        vectors
            .lines()
            .filter(|line| line.contains("_handler:"))
            .count(),
        61
    );
    assert_eq!(
        vectors
            .lines()
            .filter(|line| line.contains("_reserved: 0"))
            .count(),
        3
    );

    let device_x = fs::read_to_string(source.parent().unwrap().join("device.x")).unwrap();
    let provide_lines: Vec<_> = device_x
        .lines()
        .filter(|line| line.starts_with("PROVIDE("))
        .collect();
    let expected_provides: Vec<_> = expected
        .iter()
        .map(|(name, _)| format!("PROVIDE({name} = DefaultHandler);"))
        .collect();
    assert_eq!(provide_lines.len(), 61);
    assert_eq!(provide_lines, expected_provides);
    assert!(device_x.contains("PROVIDE(UART0 = DefaultHandler);"));
    assert!(device_x.ends_with('\n'));
    assert!(peripherals.contains("pub type Uart0 = UART0;"));
    assert!(peripherals.contains("pub type Uart5 = UART5;"));
}

#[test]
fn generate_check_includes_device_x() {
    let (_directory, source) = generated();
    let device = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/device.toml");
    let binary = env!("CARGO_BIN_EXE_rp1-regdb");

    let current = Command::new(binary)
        .arg("generate")
        .arg("--check")
        .arg(&device)
        .arg(&source)
        .status()
        .unwrap();
    assert!(current.success());

    fs::write(
        source.parent().unwrap().join("device.x"),
        "/* deliberately stale */\n",
    )
    .unwrap();
    let stale = Command::new(binary)
        .arg("generate")
        .arg("--check")
        .arg(device)
        .arg(source)
        .status()
        .unwrap();
    assert!(!stale.success());
}

#[test]
fn linux_device_metadata_is_generated() {
    let (_directory, source) = generated();
    let metadata = fs::read_to_string(source.join("metadata.rs")).unwrap();
    assert!(metadata.contains("PllSysPrimaryPhase = 6"));
    assert!(metadata.contains("pub const PCI_VENDOR_ID: u16 = 0x1de4"));
    assert!(metadata.contains("pub const PCI_DEVICE_ID: u16 = 0x0001"));
    assert!(metadata.contains("gpio_count: 28"));
    assert!(metadata.contains("host_base: Some(0x1f00000000)"));
}

#[test]
fn core_capability_and_observation_metadata_is_generated() {
    let (_directory, source) = generated();
    let metadata = fs::read_to_string(source.join("metadata.rs")).unwrap();
    let peripherals = fs::read_to_string(source.join("peripherals.rs")).unwrap();

    for expected in [
        "pub const CORE_CPUID: u32 = 0x412fc231;",
        "pub const PROCESSOR_COUNT: u8 = 2;",
        "pub const NVIC_EXTERNAL_SLOTS: u16 = 64;",
        "pub const NVIC_ASSIGNED_INTERRUPTS: u16 = 61;",
        "pub const NVIC_PRIORITY_BITS: u8 = 3;",
        "pub const MPU_DATA_REGIONS: u8 = 8;",
        "pub const MPU_INSTRUCTION_REGIONS: u8 = 0;",
        "pub const SYSTICK_CALIB: u32 = 0x00001387;",
        "pub const SYSTICK_TENMS: u32 = 4999;",
        "pub const DWT_COMPARATORS: u8 = 4;",
        "pub const FPB_CODE_COMPARATORS: u8 = 6;",
        "pub const FPB_LITERAL_COMPARATORS: u8 = 2;",
    ] {
        assert!(metadata.contains(expected), "missing {expected}");
    }
    assert!(metadata.contains("pub const EXCLUSIVE_STORE: Observation"));
    assert!(metadata.contains("result: ObservationResult::NotObserved"));
    assert!(metadata.contains("scope: Some(\"tested-firmware-sram-path\")"));
    assert!(metadata.contains("pub const BOOT_ROM_PUBLIC_LOOKUP_TABLE: Observation"));
    assert!(!metadata.contains("ConfirmedAbsent"));
    assert!(!metadata.contains("EXCLUSIVE_STORE_SUPPORTED"));
    assert!(!metadata.contains("BOOT_ROM_PUBLIC_LOOKUP_TABLE_ABSENT"));

    for owner in ["MPU", "SCB", "NVIC", "SYSTICK", "DWT", "FPB"] {
        assert!(
            !peripherals
                .lines()
                .any(|line| line.trim_start().starts_with(&format!("pub {owner}:"))),
            "unexpected core-system owner {owner}"
        );
    }
}

#[test]
fn inconclusive_observation_stays_typed_and_is_not_a_positive_bool() {
    let mut database = database();
    database.boot_rom.public_lookup_table.result =
        rp1_regdb::model::ObservationResult::Inconclusive;
    let directory = tempfile::tempdir().unwrap();
    let source = directory.path().join("rp1-pac/src");
    fs::create_dir_all(&source).unwrap();
    gen_rust::generate(&database, &source).unwrap();
    let metadata = fs::read_to_string(source.join("metadata.rs")).unwrap();
    let lookup = metadata
        .split("pub const BOOT_ROM_PUBLIC_LOOKUP_TABLE")
        .nth(1)
        .unwrap()
        .split(';')
        .next()
        .unwrap();
    assert!(lookup.contains("ObservationResult::Inconclusive"));
    assert!(!lookup.contains("bool"));
}

#[test]
fn padded_register_arrays_use_stride_elements() {
    let mut database = database();
    let timer = database
        .peripherals
        .iter_mut()
        .find(|p| p.peripheral.name == "TIMER")
        .unwrap();
    let alarm = timer
        .registers
        .iter_mut()
        .find(|register| register.name == "ALARM")
        .unwrap();
    alarm.repeat_stride = 8;

    let directory = tempfile::tempdir().unwrap();
    let source = directory.path().join("src");
    fs::create_dir_all(&source).unwrap();
    gen_rust::generate(&database, &source).unwrap();
    let timer = fs::read_to_string(source.join("generated/timer.rs")).unwrap();
    assert!(timer.contains("struct ALARMArrayElement"));
    assert!(timer.contains("[ALARMArrayElement; 4]"));
    assert!(timer.contains("&self.alarm[index].alarm"));
    assert!(!timer.contains("fn alarm_array"));
}
