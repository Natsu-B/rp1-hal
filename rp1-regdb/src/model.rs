use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DeviceFile {
    pub schema: String,
    pub device: Device,
    pub core: Core,
    pub boot_rom: BootRom,
    #[serde(default)]
    pub data_files: Vec<String>,
    #[serde(default)]
    pub interrupt: Vec<Interrupt>,
    #[serde(default)]
    pub clock: Vec<Clock>,
    #[serde(default)]
    pub gpio_bank: Vec<GpioBank>,
    #[serde(default)]
    pub address_space: Vec<AddressSpace>,
    #[serde(default)]
    pub pci_endpoint: Option<PciEndpoint>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Device {
    pub name: String,
    pub address_unit_bits: u8,
    pub register_width: u16,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Endianness {
    Little,
    Big,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Core {
    pub architecture: CoreArchitecture,
    pub implementation: CoreImplementation,
    pub revision: CoreRevision,
    pub cpuid: u32,
    pub processor_count: u8,
    pub endianness: Endianness,
    pub nvic: Nvic,
    pub mpu: Mpu,
    pub systick: SysTick,
    pub dwt: Dwt,
    pub fpb: Fpb,
    pub instructions: CoreInstructions,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum CoreArchitecture {
    Armv7M,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum CoreImplementation {
    CortexM3,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CoreRevision {
    R2p1,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Nvic {
    pub external_slots: u16,
    pub assigned_interrupts: u16,
    pub priority_bits: u8,
    pub verification: Verification,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Mpu {
    pub present: bool,
    pub kind: MpuKind,
    pub data_regions: u8,
    pub instruction_regions: u8,
    pub verification: Verification,
    pub proc0_privileged_shared_sram_data_read: MpuScopedEnforcement,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum MpuKind {
    Unified,
    Separate,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MpuScopedEnforcement {
    pub processor: u8,
    pub mode: MpuTestMode,
    pub access: MpuTestAccess,
    pub target: MpuTestTarget,
    pub region: u8,
    pub region_size_bytes: u16,
    pub permission: MpuTestPermission,
    pub exception_number: u8,
    pub precise_memmanage: Observation,
    pub data_access_violation: Observation,
    pub fault_address_valid: Observation,
    pub exact_fault_address: Observation,
    pub hardfault_escalation: Observation,
    pub exact_pc_recovery: Observation,
    pub single_handler_entry: Observation,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum MpuTestMode {
    PrivilegedThread,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum MpuTestAccess {
    DataRead,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum MpuTestTarget {
    SharedSram,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum MpuTestPermission {
    NoAccess,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SysTick {
    pub calib: u32,
    pub tenms: u32,
    pub reference_present: bool,
    pub skew: bool,
    pub nominal_reference_hz: u32,
    pub calib_verification: Verification,
    pub frequency_verification: Verification,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Dwt {
    pub present: bool,
    pub comparators: u8,
    pub cycle_counter: bool,
    pub profiling_counters: bool,
    pub external_match_trigger: bool,
    pub trace_packets: bool,
    pub verification: Verification,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Fpb {
    pub present: bool,
    pub code_comparators: u8,
    pub literal_comparators: u8,
    pub verification: Verification,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CoreInstructions {
    pub hardware_divide: Observation,
    pub exclusive_load: Observation,
    pub exclusive_store: Observation,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BootRom {
    pub present: bool,
    pub public_lookup_table: Observation,
    pub official_firmware_rom_calls: Observation,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ObservationClass {
    Capability,
    ScopedBehavior,
    RuntimeState,
    ArchitecturalInterpretation,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ObservationResult {
    Confirmed,
    NotObserved,
    Inconclusive,
    Contradicted,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Observation {
    pub verification: Verification,
    pub result: ObservationResult,
    pub class: ObservationClass,
    #[serde(default)]
    pub scope: Option<String>,
    #[serde(default)]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ObservedValue {
    pub value: u64,
    #[serde(flatten)]
    pub observation: Observation,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PeripheralFile {
    pub peripheral: Peripheral,
    #[serde(default, rename = "register")]
    pub registers: Vec<Register>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Peripheral {
    pub id: String,
    pub name: String,
    pub module: String,
    pub kind: PeripheralKind,
    pub base: u64,
    pub size: u64,
    pub ownership: Ownership,
    pub default_origin: Origin,
    #[serde(default)]
    pub legacy_svd_name: Option<String>,
    #[serde(default)]
    pub external_ip: Option<String>,
    #[serde(default)]
    pub clock_dependency: Option<String>,
    #[serde(default)]
    pub reset_dependency: Option<String>,
    #[serde(default)]
    pub pinmux_dependency: Option<String>,
    #[serde(default)]
    pub atomic_view: Option<AtomicView>,
    #[serde(default)]
    pub custom_views: Vec<CustomView>,
    #[serde(default)]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum PeripheralKind {
    Rp1NativeApb,
    ExternalIp,
    MemoryMapped,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Ownership {
    Singleton,
    Shared,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Origin {
    Official,
    Linux,
    OfficialFirmware,
    Hardware,
    ReverseEngineered,
    Inferred,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AtomicView {
    pub kind: AtomicKind,
    pub xor_offset: u64,
    pub set_offset: u64,
    pub clear_offset: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CustomView {
    pub id: String,
    pub kind: CustomViewKind,
    pub offset: u64,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum CustomViewKind {
    Set,
    Clear,
    Xor,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Register {
    pub id: String,
    pub name: String,
    pub offset: u64,
    pub width: u16,
    #[serde(default = "one")]
    pub repeat_count: u32,
    #[serde(default = "four")]
    pub repeat_stride: u64,
    #[serde(default)]
    pub public_count: Option<u32>,
    pub hardware_read: ReadBehavior,
    pub hardware_write: WriteBehavior,
    pub verified_read: Verification,
    pub verified_write: Verification,
    #[serde(default)]
    pub api: RegisterApi,
    #[serde(default)]
    pub atomic: RegisterAtomic,
    #[serde(default)]
    pub origin: Option<Origin>,
    #[serde(default)]
    pub reset_value: Option<u64>,
    #[serde(default)]
    pub boot_rom_initialization: Option<ObservedValue>,
    #[serde(default)]
    pub note: Option<String>,
    #[serde(default, rename = "field")]
    pub fields: Vec<Field>,
    #[serde(skip)]
    pub source: PathBuf,
}

const fn one() -> u32 {
    1
}
const fn four() -> u64 {
    4
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ReadBehavior {
    None,
    Plain,
    Fifo,
    StateMachineOnly,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum WriteBehavior {
    None,
    Plain,
    WriteOnly,
    WriteOneToClear,
    WriteOneToSet,
    WriteOneToToggle,
    SelfClearing,
    Command,
    Fifo,
    StateMachineOnly,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Verification {
    Official,
    Hardware,
    Static,
    Architectural,
    Unverified,
    NotApplicable,
}

impl Verification {
    pub fn permits_write(self) -> bool {
        matches!(self, Self::Official | Self::Hardware)
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
#[derive(Default)]
pub enum ApiExposure {
    #[default]
    None,
    Public,
    CratePrivate,
}


#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum RawWriteExposure {
    #[default]
    None,
    Unsafe,
    CratePrivate,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
#[derive(Default)]
pub enum AtomicKind {
    #[default]
    None,
    Rp1Native,
    Custom,
}


#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum AtomicExposure {
    #[default]
    None,
    Typed,
    Unsafe,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RegisterApi {
    #[serde(default)]
    pub read: ApiExposure,
    #[serde(default)]
    pub modify: ApiExposure,
    #[serde(default)]
    pub reset: ApiExposure,
    #[serde(default)]
    pub w1c: ApiExposure,
    #[serde(default)]
    pub w1s: ApiExposure,
    #[serde(default)]
    pub w1t: ApiExposure,
    #[serde(default)]
    pub command: ApiExposure,
    #[serde(default)]
    pub fifo_push: ApiExposure,
    #[serde(default)]
    pub fifo_pop: ApiExposure,
    #[serde(default)]
    pub raw_write: RawWriteExposure,
}

impl RegisterApi {
    pub fn has_safe_write(&self) -> bool {
        [
            self.modify,
            self.reset,
            self.w1c,
            self.w1s,
            self.w1t,
            self.command,
            self.fifo_push,
        ]
        .into_iter()
        .any(|exposure| exposure != ApiExposure::None)
    }

    pub fn is_visible(&self) -> bool {
        self.read != ApiExposure::None
            || self.fifo_pop != ApiExposure::None
            || self.has_safe_write()
            || self.raw_write != RawWriteExposure::None
    }

    pub fn is_public(&self) -> bool {
        [
            self.read,
            self.modify,
            self.reset,
            self.w1c,
            self.w1s,
            self.w1t,
            self.command,
            self.fifo_push,
            self.fifo_pop,
        ]
        .into_iter()
        .any(|exposure| exposure == ApiExposure::Public)
            || self.raw_write == RawWriteExposure::Unsafe
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct RegisterAtomic {
    #[serde(default)]
    pub kind: AtomicKind,
    #[serde(default)]
    pub set: AtomicExposure,
    #[serde(default)]
    pub clear: AtomicExposure,
    #[serde(default)]
    pub xor: AtomicExposure,
    #[serde(default)]
    pub set_view: Option<String>,
    #[serde(default)]
    pub clear_view: Option<String>,
    #[serde(default)]
    pub xor_view: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Field {
    pub id: String,
    pub name: String,
    pub lsb: u8,
    pub width: u8,
    #[serde(default)]
    pub api_read: ApiExposure,
    #[serde(default)]
    pub api_write: ApiExposure,
    #[serde(default)]
    pub write_behavior: FieldWriteBehavior,
    #[serde(default)]
    pub verified_read: Option<Verification>,
    #[serde(default)]
    pub verified_write: Option<Verification>,
    #[serde(default)]
    pub origin: Option<Origin>,
    #[serde(default)]
    pub note: Option<String>,
    #[serde(default, rename = "enum_value")]
    pub enum_values: Vec<EnumValue>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EnumValue {
    pub name: String,
    pub value: u64,
    #[serde(default)]
    pub note: Option<String>,
}

impl Field {
    pub fn effective_verified_read(&self, register: &Register) -> Verification {
        self.verified_read.unwrap_or(register.verified_read)
    }

    pub fn effective_verified_write(&self, register: &Register) -> Verification {
        self.verified_write.unwrap_or(register.verified_write)
    }

    pub fn effective_write_behavior(&self, register: &Register) -> FieldWriteBehavior {
        match self.write_behavior {
            FieldWriteBehavior::Inherit => match register.hardware_write {
                WriteBehavior::Plain => FieldWriteBehavior::Plain,
                WriteBehavior::WriteOneToClear => FieldWriteBehavior::WriteOneToClear,
                WriteBehavior::WriteOneToSet => FieldWriteBehavior::WriteOneToSet,
                WriteBehavior::WriteOneToToggle => FieldWriteBehavior::WriteOneToToggle,
                WriteBehavior::SelfClearing => FieldWriteBehavior::SelfClearing,
                WriteBehavior::Command | WriteBehavior::WriteOnly => FieldWriteBehavior::Command,
                WriteBehavior::None | WriteBehavior::Fifo | WriteBehavior::StateMachineOnly => {
                    FieldWriteBehavior::ReadOnly
                }
            },
            behavior => behavior,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum FieldWriteBehavior {
    #[default]
    Inherit,
    Plain,
    WriteOneToClear,
    WriteOneToSet,
    WriteOneToToggle,
    SelfClearing,
    Command,
    ReadOnly,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Interrupt {
    pub name: String,
    pub number: u16,
    pub origin: Origin,
    pub verified: Verification,
    #[serde(default)]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Clock {
    pub name: String,
    pub id: u8,
    pub origin: Origin,
    pub verified: Verification,
    #[serde(default)]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GpioBank {
    pub id: u8,
    pub first_gpio: u8,
    pub gpio_count: u8,
    pub io_offset: u64,
    pub interrupt_enable_offset: u64,
    pub interrupt_status_offset: u64,
    pub rio_offset: u64,
    pub pads_offset: u64,
    pub origin: Origin,
    pub verified: Verification,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AddressSpace {
    pub name: String,
    pub local_base: u64,
    pub size: u64,
    #[serde(default)]
    pub host_base: Option<u64>,
    pub origin: Origin,
    pub verified: Verification,
    #[serde(default)]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PciEndpoint {
    pub vendor_id: u16,
    pub device_id: u16,
    pub origin: Origin,
    pub verified: Verification,
    #[serde(default)]
    pub note: Option<String>,
}

#[derive(Debug)]
pub struct Database {
    pub schema: String,
    pub device: Device,
    pub core: Core,
    pub boot_rom: BootRom,
    pub peripherals: Vec<PeripheralDefinition>,
    pub interrupts: Vec<Interrupt>,
    pub clocks: Vec<Clock>,
    pub gpio_banks: Vec<GpioBank>,
    pub address_spaces: Vec<AddressSpace>,
    pub pci_endpoint: Option<PciEndpoint>,
    pub root: PathBuf,
}

#[derive(Debug)]
pub struct PeripheralDefinition {
    pub peripheral: Peripheral,
    pub registers: Vec<Register>,
    pub source: PathBuf,
}
