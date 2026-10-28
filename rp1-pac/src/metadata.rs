// Generated from rp1-regdb/data/device.toml.
// Do not edit manually.
// Regenerate with: cargo run -p rp1-regdb -- generate

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CoreArchitecture {
    Armv7M,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CoreImplementation {
    CortexM3R2P1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CoreEndianness {
    Little,
    Big,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MetadataVerification {
    Official,
    Hardware,
    Static,
    Architectural,
    Unverified,
    NotApplicable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObservationClass {
    Capability,
    ScopedBehavior,
    RuntimeState,
    ArchitecturalInterpretation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObservationResult {
    Confirmed,
    NotObserved,
    Inconclusive,
    Contradicted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Observation {
    pub verification: MetadataVerification,
    pub result: ObservationResult,
    pub class: ObservationClass,
    pub scope: Option<&'static str>,
    pub note: Option<&'static str>,
}

pub const CORE_ARCHITECTURE: CoreArchitecture = CoreArchitecture::Armv7M;
pub const CORE_IMPLEMENTATION: CoreImplementation = CoreImplementation::CortexM3R2P1;
pub const CORE_CPUID: u32 = 0x412fc231;
pub const PROCESSOR_COUNT: u8 = 2;
pub const CORE_ENDIANNESS: CoreEndianness = CoreEndianness::Little;

pub const NVIC_EXTERNAL_SLOTS: u16 = 64;
pub const NVIC_ASSIGNED_INTERRUPTS: u16 = 61;
pub const NVIC_PRIORITY_BITS: u8 = 3;

pub const NVIC_VERIFICATION: MetadataVerification = MetadataVerification::Hardware;

pub const MPU_PRESENT: bool = true;
pub const MPU_UNIFIED: bool = true;
pub const MPU_DATA_REGIONS: u8 = 8;
pub const MPU_INSTRUCTION_REGIONS: u8 = 0;
pub const MPU_VERIFICATION: MetadataVerification = MetadataVerification::Hardware;
pub const MPU_PROC0_TEST_REGION: u8 = 7;
pub const MPU_PROC0_TEST_REGION_SIZE_BYTES: u16 = 32;
pub const MPU_PROC0_TEST_EXCEPTION_NUMBER: u8 = 4;

pub const MPU_PROC0_PRECISE_MEMMANAGE: Observation = Observation {
    verification: MetadataVerification::Hardware,
    result: ObservationResult::Confirmed,
    class: ObservationClass::ScopedBehavior,
    scope: Some(
        "proc0 privileged Thread-mode data read from region-7 32-byte shared-SRAM no-access mapping",
    ),
    note: None,
};
pub const MPU_PROC0_DATA_ACCESS_VIOLATION: Observation = Observation {
    verification: MetadataVerification::Hardware,
    result: ObservationResult::Confirmed,
    class: ObservationClass::ScopedBehavior,
    scope: Some(
        "proc0 privileged Thread-mode data read from region-7 32-byte shared-SRAM no-access mapping",
    ),
    note: Some("CFSR.DACCVIOL was set."),
};
pub const MPU_PROC0_FAULT_ADDRESS_VALID: Observation = Observation {
    verification: MetadataVerification::Hardware,
    result: ObservationResult::Confirmed,
    class: ObservationClass::ScopedBehavior,
    scope: Some(
        "proc0 privileged Thread-mode data read from region-7 32-byte shared-SRAM no-access mapping",
    ),
    note: Some("CFSR.MMARVALID was set."),
};
pub const MPU_PROC0_EXACT_FAULT_ADDRESS: Observation = Observation {
    verification: MetadataVerification::Hardware,
    result: ObservationResult::Confirmed,
    class: ObservationClass::ScopedBehavior,
    scope: Some(
        "proc0 privileged Thread-mode data read from region-7 32-byte shared-SRAM no-access mapping",
    ),
    note: Some("MMFAR exactly matched the tested address."),
};
pub const MPU_PROC0_HARDFAULT_ESCALATION: Observation = Observation {
    verification: MetadataVerification::Hardware,
    result: ObservationResult::NotObserved,
    class: ObservationClass::ScopedBehavior,
    scope: Some(
        "proc0 privileged Thread-mode data read from region-7 32-byte shared-SRAM no-access mapping",
    ),
    note: Some("HFSR remained zero; this is not a global no-escalation guarantee."),
};
pub const MPU_PROC0_EXACT_PC_RECOVERY: Observation = Observation {
    verification: MetadataVerification::Hardware,
    result: ObservationResult::Confirmed,
    class: ObservationClass::ScopedBehavior,
    scope: Some(
        "proc0 privileged Thread-mode data read from region-7 32-byte shared-SRAM no-access mapping",
    ),
    note: None,
};
pub const MPU_PROC0_SINGLE_HANDLER_ENTRY: Observation = Observation {
    verification: MetadataVerification::Hardware,
    result: ObservationResult::Confirmed,
    class: ObservationClass::ScopedBehavior,
    scope: Some(
        "proc0 privileged Thread-mode data read from region-7 32-byte shared-SRAM no-access mapping",
    ),
    note: None,
};
pub const SYSTICK_CALIB: u32 = 0x00001387;
pub const SYSTICK_TENMS: u32 = 4999;
pub const SYSTICK_HAS_REFERENCE: bool = true;
pub const SYSTICK_SKEW: bool = false;
pub const SYSTICK_NOMINAL_REFERENCE_HZ: u32 = 500000;

pub const SYSTICK_CALIB_VERIFICATION: MetadataVerification = MetadataVerification::Hardware;
pub const SYSTICK_FREQUENCY_VERIFICATION: MetadataVerification =
    MetadataVerification::Architectural;

pub const DWT_PRESENT: bool = true;
pub const DWT_COMPARATORS: u8 = 4;
pub const DWT_HAS_CYCCNT: bool = false;
pub const DWT_HAS_PROFILING_COUNTERS: bool = false;
pub const DWT_HAS_EXTERNAL_MATCH_TRIGGER: bool = false;
pub const DWT_HAS_TRACE_PACKETS: bool = false;

pub const DWT_VERIFICATION: MetadataVerification = MetadataVerification::Hardware;

pub const FPB_PRESENT: bool = true;
pub const FPB_CODE_COMPARATORS: u8 = 6;
pub const FPB_LITERAL_COMPARATORS: u8 = 2;

pub const FPB_VERIFICATION: MetadataVerification = MetadataVerification::Hardware;

pub const HARDWARE_DIVIDE: Observation = Observation {
    verification: MetadataVerification::Hardware,
    result: ObservationResult::Confirmed,
    class: ObservationClass::Capability,
    scope: None,
    note: None,
};
pub const EXCLUSIVE_LOAD: Observation = Observation {
    verification: MetadataVerification::Hardware,
    result: ObservationResult::Confirmed,
    class: ObservationClass::Capability,
    scope: None,
    note: None,
};
pub const EXCLUSIVE_STORE: Observation = Observation {
    verification: MetadataVerification::Hardware,
    result: ObservationResult::NotObserved,
    class: ObservationClass::ScopedBehavior,
    scope: Some("tested-firmware-sram-path"),
    note: Some(
        "No successful STREX was observed on the tested firmware SRAM path; instruction absence is not claimed.",
    ),
};
pub const BOOT_ROM_PRESENT: bool = true;
pub const BOOT_ROM_PUBLIC_LOOKUP_TABLE: Observation = Observation {
    verification: MetadataVerification::Static,
    result: ObservationResult::NotObserved,
    class: ObservationClass::ScopedBehavior,
    scope: Some("current-boot-rom-analysis"),
    note: Some("No public lookup table was found in the analyzed scope; absence is not claimed."),
};
pub const BOOT_ROM_OFFICIAL_FIRMWARE_CALLS: Observation = Observation {
    verification: MetadataVerification::Static,
    result: ObservationResult::NotObserved,
    class: ObservationClass::ScopedBehavior,
    scope: Some("current-official-firmware-analysis"),
    note: Some(
        "No official-firmware ROM calls were found in the analyzed scope; absence is not claimed.",
    ),
};
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum ClockId {
    PllSysCore = 0,
    PllAudioCore = 1,
    PllVideoCore = 2,
    PllSys = 3,
    PllAudio = 4,
    PllVideo = 5,
    PllSysPrimaryPhase = 6,
    PllSysSecondaryPhase = 7,
    PllAudioPrimaryPhase = 8,
    PllSysSecondary = 9,
    PllAudioSecondary = 10,
    PllVideoSecondary = 11,
    Sys = 12,
    SlowSys = 13,
    Dma = 14,
    Uart = 15,
    Ethernet = 16,
    Pwm0 = 17,
    Pwm1 = 18,
    AudioIn = 19,
    AudioOut = 20,
    I2s = 21,
    Mipi0Config = 22,
    Mipi1Config = 23,
    PcieAux = 24,
    Usbh0Microframe = 25,
    Usbh1Microframe = 26,
    Usbh0Suspend = 27,
    Usbh1Suspend = 28,
    EthernetTsu = 29,
    Adc = 30,
    SdioTimer = 31,
    SdioAltSource = 32,
    Gp0 = 33,
    Gp1 = 34,
    Gp2 = 35,
    Gp3 = 36,
    Gp4 = 37,
    Gp5 = 38,
    Vec = 39,
    Dpi = 40,
    Mipi0Dpi = 41,
    Mipi1Dpi = 42,
    PllVideoPrimaryPhase = 43,
    PllAudioTertiary = 44,
    Mipi0DsiByteclock = 45,
    Mipi1DsiByteclock = 46,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GpioBank {
    pub id: u8,
    pub first_gpio: u8,
    pub gpio_count: u8,
    pub io_offset: usize,
    pub interrupt_enable_offset: usize,
    pub interrupt_status_offset: usize,
    pub rio_offset: usize,
    pub pads_offset: usize,
}

pub const GPIO_BANKS: &[GpioBank] = &[
    GpioBank {
        id: 0,
        first_gpio: 0,
        gpio_count: 28,
        io_offset: 0x0,
        interrupt_enable_offset: 0x11c,
        interrupt_status_offset: 0x124,
        rio_offset: 0x0,
        pads_offset: 0x4,
    },
    GpioBank {
        id: 1,
        first_gpio: 28,
        gpio_count: 6,
        io_offset: 0x4000,
        interrupt_enable_offset: 0x411c,
        interrupt_status_offset: 0x4124,
        rio_offset: 0x4000,
        pads_offset: 0x4004,
    },
    GpioBank {
        id: 2,
        first_gpio: 34,
        gpio_count: 20,
        io_offset: 0x8000,
        interrupt_enable_offset: 0x811c,
        interrupt_status_offset: 0x8124,
        rio_offset: 0x8000,
        pads_offset: 0x8004,
    },
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AddressSpace {
    pub name: &'static str,
    pub local_base: usize,
    pub size: usize,
    pub host_base: Option<u64>,
}

pub const ADDRESS_SPACES: &[AddressSpace] = &[
    AddressSpace {
        name: "PERIPHERAL",
        local_base: 0x40000000,
        size: 0x400000,
        host_base: Some(0x1f00000000),
    },
    AddressSpace {
        name: "SRAM",
        local_base: 0x20000000,
        size: 0x10000,
        host_base: Some(0x1f00400000),
    },
];

pub const PCI_VENDOR_ID: u16 = 0x1de4;
pub const PCI_DEVICE_ID: u16 = 0x0001;
