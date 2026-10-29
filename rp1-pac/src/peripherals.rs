// Generated from rp1-regdb TOML.
// Do not edit manually.
// Regenerate with: cargo run -p rp1-regdb -- generate

use core::sync::atomic::{AtomicBool, Ordering};

static TAKEN: AtomicBool = AtomicBool::new(false);

/// RP1 ADC_RW register block (12-bit SAR ADC: four external inputs and the temperature sensor). Bus=APB. Atomic alias: normal read/write access.
/// Atomic view: Rp1Native alias windows; ownership remains with this logical peripheral.
#[allow(non_camel_case_types)]
pub struct ADC {
    _private: (),
}
pub type Adc = ADC;
impl ADC {
    pub const PTR: *const u8 = 0x400c8000 as *const u8;
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

impl core::ops::Deref for ADC {
    type Target = crate::generated::adc::RegisterBlock;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*(Self::PTR.cast()) }
    }
}

unsafe impl Send for ADC {}

/// RP1 AUDIO_IN_RW register block. Bus=APB. Atomic alias: normal read/write access.
/// Atomic view: Rp1Native alias windows; ownership remains with this logical peripheral.
/// Address-only or externally delegated block; unknown register layout is intentionally hidden.
#[allow(non_camel_case_types)]
pub struct AUDIO_IN {
    _private: (),
}
pub type AudioIn = AUDIO_IN;
impl AUDIO_IN {
    pub const PTR: *const u8 = 0x40090000 as *const u8;
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

unsafe impl Send for AUDIO_IN {}

/// RP1 AUDIO_OUT_RW register block. Bus=APB. Atomic alias: normal read/write access.
/// Atomic view: Rp1Native alias windows; ownership remains with this logical peripheral.
/// Address-only or externally delegated block; unknown register layout is intentionally hidden.
#[allow(non_camel_case_types)]
pub struct AUDIO_OUT {
    _private: (),
}
pub type AudioOut = AUDIO_OUT;
impl AUDIO_OUT {
    pub const PTR: *const u8 = 0x40094000 as *const u8;
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

unsafe impl Send for AUDIO_OUT {}

/// RP1 BUSFABRIC_AXISHIM_RW register block. Bus=APB. Atomic alias: normal read/write access.
/// Atomic view: Rp1Native alias windows; ownership remains with this logical peripheral.
/// Address-only or externally delegated block; unknown register layout is intentionally hidden.
#[allow(non_camel_case_types)]
pub struct BUSFABRIC_AXISHIM {
    _private: (),
}
pub type BusfabricAxishim = BUSFABRIC_AXISHIM;
impl BUSFABRIC_AXISHIM {
    pub const PTR: *const u8 = 0x400c4000 as *const u8;
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

unsafe impl Send for BUSFABRIC_AXISHIM {}

/// RP1 BUSFABRIC_MONITOR_RW register block. Bus=APB. Atomic alias: normal read/write access.
/// Atomic view: Rp1Native alias windows; ownership remains with this logical peripheral.
/// Address-only or externally delegated block; unknown register layout is intentionally hidden.
#[allow(non_camel_case_types)]
pub struct BUSFABRIC_MONITOR {
    _private: (),
}
pub type BusfabricMonitor = BUSFABRIC_MONITOR;
impl BUSFABRIC_MONITOR {
    pub const PTR: *const u8 = 0x400c0000 as *const u8;
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

unsafe impl Send for BUSFABRIC_MONITOR {}

/// RP1 CLOCKS_MAIN register block with supplemental register semantics. Alias behavior: normal read/write access. Register layout recovered from the Linux RP1 common-clock driver. Unknown offsets remain split RAW arrays.
/// Atomic view: Rp1Native alias windows; ownership remains with this logical peripheral.
#[allow(non_camel_case_types)]
pub struct CLOCKS_MAIN {
    _private: (),
}
pub type ClocksMain = CLOCKS_MAIN;
impl CLOCKS_MAIN {
    pub const PTR: *const u8 = 0x40018000 as *const u8;
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

impl core::ops::Deref for CLOCKS_MAIN {
    type Target = crate::generated::clocks_main::RegisterBlock;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*(Self::PTR.cast()) }
    }
}

unsafe impl Send for CLOCKS_MAIN {}

/// RP1 CLOCKS_VIDEO register block with supplemental register semantics. Alias behavior: normal read/write access. Video clock layout recovered from the Linux RP1 common-clock driver. Unknown offsets remain split RAW arrays.
/// Atomic view: Rp1Native alias windows; ownership remains with this logical peripheral.
#[allow(non_camel_case_types)]
pub struct CLOCKS_VIDEO {
    _private: (),
}
pub type ClocksVideo = CLOCKS_VIDEO;
impl CLOCKS_VIDEO {
    pub const PTR: *const u8 = 0x4001c000 as *const u8;
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

impl core::ops::Deref for CLOCKS_VIDEO {
    type Target = crate::generated::clocks_video::RegisterBlock;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*(Self::PTR.cast()) }
    }
}

unsafe impl Send for CLOCKS_VIDEO {}

/// RP1 DMA register block. Bus=AHB. No RP1 atomic alias listed.
/// External IP instance delegated to `synopsys-dw-axi-dmac`; only RP1 instance metadata is provided.
/// Address-only or externally delegated block; unknown register layout is intentionally hidden.
#[allow(non_camel_case_types)]
pub struct DMA {
    _private: (),
}
pub type Dma = DMA;
impl DMA {
    pub const PTR: *const u8 = 0x40188000 as *const u8;
    pub const EXTERNAL_IP: &str = "synopsys-dw-axi-dmac";
    pub const CLOCK_DEPENDENCY: &str = "ClockId::Dma";
    pub const RESET_DEPENDENCY: &str = "RESETS field DMA when present";
    pub const PINMUX_DEPENDENCY: &str = "IO_BANK/PADS function selection where externally pinned";
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

unsafe impl Send for DMA {}

/// RP1 DMA_TICK_RW register block. Bus=APB. Atomic alias: normal read/write access.
/// Atomic view: Rp1Native alias windows; ownership remains with this logical peripheral.
#[allow(non_camel_case_types)]
pub struct DMA_TICK {
    _private: (),
}
pub type DmaTick = DMA_TICK;
impl DMA_TICK {
    pub const PTR: *const u8 = 0x40158000 as *const u8;
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

impl core::ops::Deref for DMA_TICK {
    type Target = crate::generated::dma_tick::RegisterBlock;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*(Self::PTR.cast()) }
    }
}

unsafe impl Send for DMA_TICK {}

/// RP1 ETH register block. Bus=APB. No RP1 atomic alias listed.
/// External IP instance delegated to `cadence-gem`; only RP1 instance metadata is provided.
/// Address-only or externally delegated block; unknown register layout is intentionally hidden.
#[allow(non_camel_case_types)]
pub struct ETH {
    _private: (),
}
pub type Eth = ETH;
impl ETH {
    pub const PTR: *const u8 = 0x40100000 as *const u8;
    pub const EXTERNAL_IP: &str = "cadence-gem";
    pub const CLOCK_DEPENDENCY: &str = "ClockId::Ethernet";
    pub const RESET_DEPENDENCY: &str = "RESETS field ETH when present";
    pub const PINMUX_DEPENDENCY: &str = "IO_BANK/PADS function selection where externally pinned";
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

unsafe impl Send for ETH {}

/// RP1 ETH_CFG_RW register block. Bus=APB. Atomic alias: normal read/write access.
/// Atomic view: Rp1Native alias windows; ownership remains with this logical peripheral.
#[allow(non_camel_case_types)]
pub struct ETH_CFG {
    _private: (),
}
pub type EthCfg = ETH_CFG;
impl ETH_CFG {
    pub const PTR: *const u8 = 0x40104000 as *const u8;
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

impl core::ops::Deref for ETH_CFG {
    type Target = crate::generated::eth_cfg::RegisterBlock;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*(Self::PTR.cast()) }
    }
}

unsafe impl Send for ETH_CFG {}

/// RP1 EXAC register block. Bus=AXI3. No RP1 atomic alias listed.
/// Address-only or externally delegated block; unknown register layout is intentionally hidden.
#[allow(non_camel_case_types)]
pub struct EXAC {
    _private: (),
}
pub type Exac = EXAC;
impl EXAC {
    pub const PTR: *const u8 = 0x40400000 as *const u8;
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

unsafe impl Send for EXAC {}

/// RP1 I2C0 register block. Bus=APB. No RP1 atomic alias listed.
/// External IP instance delegated to `synopsys-dw-apb-i2c`; only RP1 instance metadata is provided.
/// Address-only or externally delegated block; unknown register layout is intentionally hidden.
#[allow(non_camel_case_types)]
pub struct I2C0 {
    _private: (),
}
pub type I2c0 = I2C0;
impl I2C0 {
    pub const PTR: *const u8 = 0x40070000 as *const u8;
    pub const EXTERNAL_IP: &str = "synopsys-dw-apb-i2c";
    pub const CLOCK_DEPENDENCY: &str = "ClockId::Sys";
    pub const RESET_DEPENDENCY: &str = "RESETS field I2C0 when present";
    pub const PINMUX_DEPENDENCY: &str = "IO_BANK/PADS function selection where externally pinned";
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

unsafe impl Send for I2C0 {}

/// RP1 I2C1 register block. Bus=APB. No RP1 atomic alias listed.
/// External IP instance delegated to `synopsys-dw-apb-i2c`; only RP1 instance metadata is provided.
/// Address-only or externally delegated block; unknown register layout is intentionally hidden.
#[allow(non_camel_case_types)]
pub struct I2C1 {
    _private: (),
}
pub type I2c1 = I2C1;
impl I2C1 {
    pub const PTR: *const u8 = 0x40074000 as *const u8;
    pub const EXTERNAL_IP: &str = "synopsys-dw-apb-i2c";
    pub const CLOCK_DEPENDENCY: &str = "ClockId::Sys";
    pub const RESET_DEPENDENCY: &str = "RESETS field I2C1 when present";
    pub const PINMUX_DEPENDENCY: &str = "IO_BANK/PADS function selection where externally pinned";
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

unsafe impl Send for I2C1 {}

/// RP1 I2C2 register block. Bus=APB. No RP1 atomic alias listed.
/// External IP instance delegated to `synopsys-dw-apb-i2c`; only RP1 instance metadata is provided.
/// Address-only or externally delegated block; unknown register layout is intentionally hidden.
#[allow(non_camel_case_types)]
pub struct I2C2 {
    _private: (),
}
pub type I2c2 = I2C2;
impl I2C2 {
    pub const PTR: *const u8 = 0x40078000 as *const u8;
    pub const EXTERNAL_IP: &str = "synopsys-dw-apb-i2c";
    pub const CLOCK_DEPENDENCY: &str = "ClockId::Sys";
    pub const RESET_DEPENDENCY: &str = "RESETS field I2C2 when present";
    pub const PINMUX_DEPENDENCY: &str = "IO_BANK/PADS function selection where externally pinned";
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

unsafe impl Send for I2C2 {}

/// RP1 I2C3 register block. Bus=APB. No RP1 atomic alias listed.
/// External IP instance delegated to `synopsys-dw-apb-i2c`; only RP1 instance metadata is provided.
/// Address-only or externally delegated block; unknown register layout is intentionally hidden.
#[allow(non_camel_case_types)]
pub struct I2C3 {
    _private: (),
}
pub type I2c3 = I2C3;
impl I2C3 {
    pub const PTR: *const u8 = 0x4007c000 as *const u8;
    pub const EXTERNAL_IP: &str = "synopsys-dw-apb-i2c";
    pub const CLOCK_DEPENDENCY: &str = "ClockId::Sys";
    pub const RESET_DEPENDENCY: &str = "RESETS field I2C3 when present";
    pub const PINMUX_DEPENDENCY: &str = "IO_BANK/PADS function selection where externally pinned";
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

unsafe impl Send for I2C3 {}

/// RP1 I2C4 register block. Bus=APB. No RP1 atomic alias listed.
/// External IP instance delegated to `synopsys-dw-apb-i2c`; only RP1 instance metadata is provided.
/// Address-only or externally delegated block; unknown register layout is intentionally hidden.
#[allow(non_camel_case_types)]
pub struct I2C4 {
    _private: (),
}
pub type I2c4 = I2C4;
impl I2C4 {
    pub const PTR: *const u8 = 0x40080000 as *const u8;
    pub const EXTERNAL_IP: &str = "synopsys-dw-apb-i2c";
    pub const CLOCK_DEPENDENCY: &str = "ClockId::Sys";
    pub const RESET_DEPENDENCY: &str = "RESETS field I2C4 when present";
    pub const PINMUX_DEPENDENCY: &str = "IO_BANK/PADS function selection where externally pinned";
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

unsafe impl Send for I2C4 {}

/// RP1 I2C5 register block. Bus=APB. No RP1 atomic alias listed.
/// External IP instance delegated to `synopsys-dw-apb-i2c`; only RP1 instance metadata is provided.
/// Address-only or externally delegated block; unknown register layout is intentionally hidden.
#[allow(non_camel_case_types)]
pub struct I2C5 {
    _private: (),
}
pub type I2c5 = I2C5;
impl I2C5 {
    pub const PTR: *const u8 = 0x40084000 as *const u8;
    pub const EXTERNAL_IP: &str = "synopsys-dw-apb-i2c";
    pub const CLOCK_DEPENDENCY: &str = "ClockId::Sys";
    pub const RESET_DEPENDENCY: &str = "RESETS field I2C5 when present";
    pub const PINMUX_DEPENDENCY: &str = "IO_BANK/PADS function selection where externally pinned";
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

unsafe impl Send for I2C5 {}

/// RP1 I2C6 register block. Bus=APB. No RP1 atomic alias listed.
/// External IP instance delegated to `synopsys-dw-apb-i2c`; only RP1 instance metadata is provided.
/// Address-only or externally delegated block; unknown register layout is intentionally hidden.
#[allow(non_camel_case_types)]
pub struct I2C6 {
    _private: (),
}
pub type I2c6 = I2C6;
impl I2C6 {
    pub const PTR: *const u8 = 0x40088000 as *const u8;
    pub const EXTERNAL_IP: &str = "synopsys-dw-apb-i2c";
    pub const CLOCK_DEPENDENCY: &str = "ClockId::Sys";
    pub const RESET_DEPENDENCY: &str = "RESETS field I2C6 when present";
    pub const PINMUX_DEPENDENCY: &str = "IO_BANK/PADS function selection where externally pinned";
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

unsafe impl Send for I2C6 {}

/// RP1 I2S0 register block. Bus=APB. No RP1 atomic alias listed.
/// External IP instance delegated to `synopsys-designware-i2s`; only RP1 instance metadata is provided.
/// Address-only or externally delegated block; unknown register layout is intentionally hidden.
#[allow(non_camel_case_types)]
pub struct I2S0 {
    _private: (),
}
pub type I2s0 = I2S0;
impl I2S0 {
    pub const PTR: *const u8 = 0x400a0000 as *const u8;
    pub const EXTERNAL_IP: &str = "synopsys-designware-i2s";
    pub const CLOCK_DEPENDENCY: &str = "ClockId::I2s";
    pub const RESET_DEPENDENCY: &str = "RESETS field I2S0 when present";
    pub const PINMUX_DEPENDENCY: &str = "IO_BANK/PADS function selection where externally pinned";
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

unsafe impl Send for I2S0 {}

/// RP1 I2S1 register block. Bus=APB. No RP1 atomic alias listed.
/// External IP instance delegated to `synopsys-designware-i2s`; only RP1 instance metadata is provided.
/// Address-only or externally delegated block; unknown register layout is intentionally hidden.
#[allow(non_camel_case_types)]
pub struct I2S1 {
    _private: (),
}
pub type I2s1 = I2S1;
impl I2S1 {
    pub const PTR: *const u8 = 0x400a4000 as *const u8;
    pub const EXTERNAL_IP: &str = "synopsys-designware-i2s";
    pub const CLOCK_DEPENDENCY: &str = "ClockId::I2s";
    pub const RESET_DEPENDENCY: &str = "RESETS field I2S1 when present";
    pub const PINMUX_DEPENDENCY: &str = "IO_BANK/PADS function selection where externally pinned";
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

unsafe impl Send for I2S1 {}

/// RP1 I2S2 register block. Bus=APB. No RP1 atomic alias listed.
/// External IP instance delegated to `synopsys-designware-i2s`; only RP1 instance metadata is provided.
/// Address-only or externally delegated block; unknown register layout is intentionally hidden.
#[allow(non_camel_case_types)]
pub struct I2S2 {
    _private: (),
}
pub type I2s2 = I2S2;
impl I2S2 {
    pub const PTR: *const u8 = 0x400a8000 as *const u8;
    pub const EXTERNAL_IP: &str = "synopsys-designware-i2s";
    pub const CLOCK_DEPENDENCY: &str = "ClockId::I2s";
    pub const RESET_DEPENDENCY: &str = "RESETS field I2S2 when present";
    pub const PINMUX_DEPENDENCY: &str = "IO_BANK/PADS function selection where externally pinned";
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

unsafe impl Send for I2S2 {}

/// RP1 IO_BANK0_RW register block. Bus=APB. Atomic alias: normal read/write access.
/// Atomic view: Rp1Native alias windows; ownership remains with this logical peripheral.
#[allow(non_camel_case_types)]
pub struct IO_BANK0 {
    _private: (),
}
pub type IoBank0 = IO_BANK0;
impl IO_BANK0 {
    pub const PTR: *const u8 = 0x400d0000 as *const u8;
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

impl core::ops::Deref for IO_BANK0 {
    type Target = crate::generated::io_bank0::RegisterBlock;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*(Self::PTR.cast()) }
    }
}

unsafe impl Send for IO_BANK0 {}

/// RP1 IO_BANK1 register block with supplemental register semantics. Alias behavior: normal read/write access. Bank geometry and interrupt offsets are recovered from the Linux pinctrl driver; register fields are inherited from the public bank-0 layout. Unknown offsets remain split RAW arrays.
/// Atomic view: Rp1Native alias windows; ownership remains with this logical peripheral.
#[allow(non_camel_case_types)]
pub struct IO_BANK1 {
    _private: (),
}
pub type IoBank1 = IO_BANK1;
impl IO_BANK1 {
    pub const PTR: *const u8 = 0x400d4000 as *const u8;
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

impl core::ops::Deref for IO_BANK1 {
    type Target = crate::generated::io_bank1::RegisterBlock;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*(Self::PTR.cast()) }
    }
}

unsafe impl Send for IO_BANK1 {}

/// RP1 IO_BANK2 register block with supplemental register semantics. Alias behavior: normal read/write access. Bank geometry and interrupt offsets are recovered from the Linux pinctrl driver; register fields are inherited from the public bank-0 layout. Unknown offsets remain split RAW arrays.
/// Atomic view: Rp1Native alias windows; ownership remains with this logical peripheral.
#[allow(non_camel_case_types)]
pub struct IO_BANK2 {
    _private: (),
}
pub type IoBank2 = IO_BANK2;
impl IO_BANK2 {
    pub const PTR: *const u8 = 0x400d8000 as *const u8;
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

impl core::ops::Deref for IO_BANK2 {
    type Target = crate::generated::io_bank2::RegisterBlock;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*(Self::PTR.cast()) }
    }
}

unsafe impl Send for IO_BANK2 {}

/// RP1 MIPI0_CFG_RW register block. Bus=APB. Atomic alias: normal read/write access.
/// Atomic view: Rp1Native alias windows; ownership remains with this logical peripheral.
/// Address-only or externally delegated block; unknown register layout is intentionally hidden.
#[allow(non_camel_case_types)]
pub struct MIPI0_CFG {
    _private: (),
}
pub type Mipi0Cfg = MIPI0_CFG;
impl MIPI0_CFG {
    pub const PTR: *const u8 = 0x40120000 as *const u8;
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

unsafe impl Send for MIPI0_CFG {}

/// RP1 MIPI0_CSIDMA register block. Bus=APB. No RP1 atomic alias listed.
/// External IP instance delegated to `mipi-csi-dma`; only RP1 instance metadata is provided.
/// Address-only or externally delegated block; unknown register layout is intentionally hidden.
#[allow(non_camel_case_types)]
pub struct MIPI0_CSIDMA {
    _private: (),
}
pub type Mipi0Csidma = MIPI0_CSIDMA;
impl MIPI0_CSIDMA {
    pub const PTR: *const u8 = 0x40110000 as *const u8;
    pub const EXTERNAL_IP: &str = "mipi-csi-dma";
    pub const CLOCK_DEPENDENCY: &str = "platform clock dependency; see RP1 integration";
    pub const RESET_DEPENDENCY: &str = "RESETS field MIPI0_CSIDMA when present";
    pub const PINMUX_DEPENDENCY: &str = "IO_BANK/PADS function selection where externally pinned";
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

unsafe impl Send for MIPI0_CSIDMA {}

/// RP1 MIPI0_CSIHOST register block. Bus=APB. No RP1 atomic alias listed.
/// External IP instance delegated to `mipi-csi-host`; only RP1 instance metadata is provided.
/// Address-only or externally delegated block; unknown register layout is intentionally hidden.
#[allow(non_camel_case_types)]
pub struct MIPI0_CSIHOST {
    _private: (),
}
pub type Mipi0Csihost = MIPI0_CSIHOST;
impl MIPI0_CSIHOST {
    pub const PTR: *const u8 = 0x40114000 as *const u8;
    pub const EXTERNAL_IP: &str = "mipi-csi-host";
    pub const CLOCK_DEPENDENCY: &str = "platform clock dependency; see RP1 integration";
    pub const RESET_DEPENDENCY: &str = "RESETS field MIPI0_CSIHOST when present";
    pub const PINMUX_DEPENDENCY: &str = "IO_BANK/PADS function selection where externally pinned";
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

unsafe impl Send for MIPI0_CSIHOST {}

/// RP1 MIPI0_DSIDMA register block. Bus=APB. No RP1 atomic alias listed.
/// External IP instance delegated to `mipi-dsi-dma`; only RP1 instance metadata is provided.
/// Address-only or externally delegated block; unknown register layout is intentionally hidden.
#[allow(non_camel_case_types)]
pub struct MIPI0_DSIDMA {
    _private: (),
}
pub type Mipi0Dsidma = MIPI0_DSIDMA;
impl MIPI0_DSIDMA {
    pub const PTR: *const u8 = 0x40118000 as *const u8;
    pub const EXTERNAL_IP: &str = "mipi-dsi-dma";
    pub const CLOCK_DEPENDENCY: &str = "platform clock dependency; see RP1 integration";
    pub const RESET_DEPENDENCY: &str = "RESETS field MIPI0_DSIDMA when present";
    pub const PINMUX_DEPENDENCY: &str = "IO_BANK/PADS function selection where externally pinned";
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

unsafe impl Send for MIPI0_DSIDMA {}

/// RP1 MIPI0_DSIHOST register block. Bus=APB. No RP1 atomic alias listed.
/// External IP instance delegated to `mipi-dsi-host`; only RP1 instance metadata is provided.
/// Address-only or externally delegated block; unknown register layout is intentionally hidden.
#[allow(non_camel_case_types)]
pub struct MIPI0_DSIHOST {
    _private: (),
}
pub type Mipi0Dsihost = MIPI0_DSIHOST;
impl MIPI0_DSIHOST {
    pub const PTR: *const u8 = 0x4011c000 as *const u8;
    pub const EXTERNAL_IP: &str = "mipi-dsi-host";
    pub const CLOCK_DEPENDENCY: &str = "platform clock dependency; see RP1 integration";
    pub const RESET_DEPENDENCY: &str = "RESETS field MIPI0_DSIHOST when present";
    pub const PINMUX_DEPENDENCY: &str = "IO_BANK/PADS function selection where externally pinned";
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

unsafe impl Send for MIPI0_DSIHOST {}

/// RP1 MIPI0_ISP register block. Bus=APB. No RP1 atomic alias listed.
/// External IP instance delegated to `mipi-isp`; only RP1 instance metadata is provided.
/// Address-only or externally delegated block; unknown register layout is intentionally hidden.
#[allow(non_camel_case_types)]
pub struct MIPI0_ISP {
    _private: (),
}
pub type Mipi0Isp = MIPI0_ISP;
impl MIPI0_ISP {
    pub const PTR: *const u8 = 0x40124000 as *const u8;
    pub const EXTERNAL_IP: &str = "mipi-isp";
    pub const CLOCK_DEPENDENCY: &str = "platform clock dependency; see RP1 integration";
    pub const RESET_DEPENDENCY: &str = "RESETS field MIPI0_ISP when present";
    pub const PINMUX_DEPENDENCY: &str = "IO_BANK/PADS function selection where externally pinned";
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

unsafe impl Send for MIPI0_ISP {}

/// RP1 MIPI1_CFG_RW register block. Bus=APB. Atomic alias: normal read/write access.
/// Atomic view: Rp1Native alias windows; ownership remains with this logical peripheral.
/// Address-only or externally delegated block; unknown register layout is intentionally hidden.
#[allow(non_camel_case_types)]
pub struct MIPI1_CFG {
    _private: (),
}
pub type Mipi1Cfg = MIPI1_CFG;
impl MIPI1_CFG {
    pub const PTR: *const u8 = 0x40138000 as *const u8;
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

unsafe impl Send for MIPI1_CFG {}

/// RP1 MIPI1_CSIDMA register block. Bus=APB. No RP1 atomic alias listed.
/// External IP instance delegated to `mipi-csi-dma`; only RP1 instance metadata is provided.
/// Address-only or externally delegated block; unknown register layout is intentionally hidden.
#[allow(non_camel_case_types)]
pub struct MIPI1_CSIDMA {
    _private: (),
}
pub type Mipi1Csidma = MIPI1_CSIDMA;
impl MIPI1_CSIDMA {
    pub const PTR: *const u8 = 0x40128000 as *const u8;
    pub const EXTERNAL_IP: &str = "mipi-csi-dma";
    pub const CLOCK_DEPENDENCY: &str = "platform clock dependency; see RP1 integration";
    pub const RESET_DEPENDENCY: &str = "RESETS field MIPI1_CSIDMA when present";
    pub const PINMUX_DEPENDENCY: &str = "IO_BANK/PADS function selection where externally pinned";
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

unsafe impl Send for MIPI1_CSIDMA {}

/// RP1 MIPI1_CSIHOST register block. Bus=APB. No RP1 atomic alias listed.
/// External IP instance delegated to `mipi-csi-host`; only RP1 instance metadata is provided.
/// Address-only or externally delegated block; unknown register layout is intentionally hidden.
#[allow(non_camel_case_types)]
pub struct MIPI1_CSIHOST {
    _private: (),
}
pub type Mipi1Csihost = MIPI1_CSIHOST;
impl MIPI1_CSIHOST {
    pub const PTR: *const u8 = 0x4012c000 as *const u8;
    pub const EXTERNAL_IP: &str = "mipi-csi-host";
    pub const CLOCK_DEPENDENCY: &str = "platform clock dependency; see RP1 integration";
    pub const RESET_DEPENDENCY: &str = "RESETS field MIPI1_CSIHOST when present";
    pub const PINMUX_DEPENDENCY: &str = "IO_BANK/PADS function selection where externally pinned";
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

unsafe impl Send for MIPI1_CSIHOST {}

/// RP1 MIPI1_DSIDMA register block. Bus=APB. No RP1 atomic alias listed.
/// External IP instance delegated to `mipi-dsi-dma`; only RP1 instance metadata is provided.
/// Address-only or externally delegated block; unknown register layout is intentionally hidden.
#[allow(non_camel_case_types)]
pub struct MIPI1_DSIDMA {
    _private: (),
}
pub type Mipi1Dsidma = MIPI1_DSIDMA;
impl MIPI1_DSIDMA {
    pub const PTR: *const u8 = 0x40130000 as *const u8;
    pub const EXTERNAL_IP: &str = "mipi-dsi-dma";
    pub const CLOCK_DEPENDENCY: &str = "platform clock dependency; see RP1 integration";
    pub const RESET_DEPENDENCY: &str = "RESETS field MIPI1_DSIDMA when present";
    pub const PINMUX_DEPENDENCY: &str = "IO_BANK/PADS function selection where externally pinned";
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

unsafe impl Send for MIPI1_DSIDMA {}

/// RP1 MIPI1_DSIHOST register block. Bus=APB. No RP1 atomic alias listed.
/// External IP instance delegated to `mipi-dsi-host`; only RP1 instance metadata is provided.
/// Address-only or externally delegated block; unknown register layout is intentionally hidden.
#[allow(non_camel_case_types)]
pub struct MIPI1_DSIHOST {
    _private: (),
}
pub type Mipi1Dsihost = MIPI1_DSIHOST;
impl MIPI1_DSIHOST {
    pub const PTR: *const u8 = 0x40134000 as *const u8;
    pub const EXTERNAL_IP: &str = "mipi-dsi-host";
    pub const CLOCK_DEPENDENCY: &str = "platform clock dependency; see RP1 integration";
    pub const RESET_DEPENDENCY: &str = "RESETS field MIPI1_DSIHOST when present";
    pub const PINMUX_DEPENDENCY: &str = "IO_BANK/PADS function selection where externally pinned";
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

unsafe impl Send for MIPI1_DSIHOST {}

/// RP1 MIPI1_ISP register block. Bus=APB. No RP1 atomic alias listed.
/// External IP instance delegated to `mipi-isp`; only RP1 instance metadata is provided.
/// Address-only or externally delegated block; unknown register layout is intentionally hidden.
#[allow(non_camel_case_types)]
pub struct MIPI1_ISP {
    _private: (),
}
pub type Mipi1Isp = MIPI1_ISP;
impl MIPI1_ISP {
    pub const PTR: *const u8 = 0x4013c000 as *const u8;
    pub const EXTERNAL_IP: &str = "mipi-isp";
    pub const CLOCK_DEPENDENCY: &str = "platform clock dependency; see RP1 integration";
    pub const RESET_DEPENDENCY: &str = "RESETS field MIPI1_ISP when present";
    pub const PINMUX_DEPENDENCY: &str = "IO_BANK/PADS function selection where externally pinned";
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

unsafe impl Send for MIPI1_ISP {}

/// RP1 OTP block used by the Boot ROM read/configuration path. Unknown register bodies remain hidden; no programming, erase, lock, safe raw-write, modify, or atomic API is exposed.
/// Atomic view: Rp1Native alias windows; ownership remains with this logical peripheral.
/// Address-only or externally delegated block; unknown register layout is intentionally hidden.
#[allow(non_camel_case_types)]
pub struct OTP {
    _private: (),
}
pub type Otp = OTP;
impl OTP {
    pub const PTR: *const u8 = 0x4000c000 as *const u8;
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

unsafe impl Send for OTP {}

/// RP1 PADS_BANK0_RW register block. Bus=APB. Atomic alias: normal read/write access.
/// Atomic view: Rp1Native alias windows; ownership remains with this logical peripheral.
#[allow(non_camel_case_types)]
pub struct PADS_BANK0 {
    _private: (),
}
pub type PadsBank0 = PADS_BANK0;
impl PADS_BANK0 {
    pub const PTR: *const u8 = 0x400f0000 as *const u8;
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

impl core::ops::Deref for PADS_BANK0 {
    type Target = crate::generated::pads_bank0::RegisterBlock;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*(Self::PTR.cast()) }
    }
}

unsafe impl Send for PADS_BANK0 {}

/// RP1 PADS_BANK1 register block with supplemental register semantics. Alias behavior: normal read/write access. Bank geometry and pad fields are recovered from the Linux pinctrl driver; fields are inherited from the public bank-0 layout. Unknown offsets remain split RAW arrays.
/// Atomic view: Rp1Native alias windows; ownership remains with this logical peripheral.
#[allow(non_camel_case_types)]
pub struct PADS_BANK1 {
    _private: (),
}
pub type PadsBank1 = PADS_BANK1;
impl PADS_BANK1 {
    pub const PTR: *const u8 = 0x400f4000 as *const u8;
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

impl core::ops::Deref for PADS_BANK1 {
    type Target = crate::generated::pads_bank1::RegisterBlock;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*(Self::PTR.cast()) }
    }
}

unsafe impl Send for PADS_BANK1 {}

/// RP1 PADS_BANK2 register block with supplemental register semantics. Alias behavior: normal read/write access. Bank geometry and pad fields are recovered from the Linux pinctrl driver; fields are inherited from the public bank-0 layout. Unknown offsets remain split RAW arrays.
/// Atomic view: Rp1Native alias windows; ownership remains with this logical peripheral.
#[allow(non_camel_case_types)]
pub struct PADS_BANK2 {
    _private: (),
}
pub type PadsBank2 = PADS_BANK2;
impl PADS_BANK2 {
    pub const PTR: *const u8 = 0x400f8000 as *const u8;
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

impl core::ops::Deref for PADS_BANK2 {
    type Target = crate::generated::pads_bank2::RegisterBlock;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*(Self::PTR.cast()) }
    }
}

unsafe impl Send for PADS_BANK2 {}

/// RP1 PADS_ETH_RW register block. Bus=APB. Atomic alias: normal read/write access.
/// Atomic view: Rp1Native alias windows; ownership remains with this logical peripheral.
/// Address-only or externally delegated block; unknown register layout is intentionally hidden.
#[allow(non_camel_case_types)]
pub struct PADS_ETH {
    _private: (),
}
pub type PadsEth = PADS_ETH;
impl PADS_ETH {
    pub const PTR: *const u8 = 0x400fc000 as *const u8;
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

unsafe impl Send for PADS_ETH {}

/// Only independently understood pre-DBI controller registers are exposed; state-machine-only controls remain hidden.
/// Atomic view: Rp1Native alias windows; ownership remains with this logical peripheral.
#[allow(non_camel_case_types)]
pub struct PCIE_APBS {
    _private: (),
}
pub type PcieApbs = PCIE_APBS;
impl PCIE_APBS {
    pub const PTR: *const u8 = 0x40108000 as *const u8;
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

impl core::ops::Deref for PCIE_APBS {
    type Target = crate::generated::pcie_apbs::RegisterBlock;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*(Self::PTR.cast()) }
    }
}

unsafe impl Send for PCIE_APBS {}

/// RP1 PIO_RW register block. Bus=APB. Atomic alias: normal read/write access.
/// Atomic view: Rp1Native alias windows; ownership remains with this logical peripheral.
/// Address-only or externally delegated block; unknown register layout is intentionally hidden.
#[allow(non_camel_case_types)]
pub struct PIO {
    _private: (),
}
pub type Pio = PIO;
impl PIO {
    pub const PTR: *const u8 = 0x40178000 as *const u8;
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

unsafe impl Send for PIO {}

/// RP1 PLL_AUDIO register block with supplemental register semantics. Alias behavior: normal read/write access. PLL layout recovered from the Linux RP1 common-clock driver. Unknown offsets remain split RAW arrays.
/// Atomic view: Rp1Native alias windows; ownership remains with this logical peripheral.
#[allow(non_camel_case_types)]
pub struct PLL_AUDIO {
    _private: (),
}
pub type PllAudio = PLL_AUDIO;
impl PLL_AUDIO {
    pub const PTR: *const u8 = 0x40024000 as *const u8;
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

impl core::ops::Deref for PLL_AUDIO {
    type Target = crate::generated::pll_audio::RegisterBlock;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*(Self::PTR.cast()) }
    }
}

unsafe impl Send for PLL_AUDIO {}

/// Minimal clock block slice retained for read-only APB clock phase diagnostics.
/// Atomic view: Rp1Native alias windows; ownership remains with this logical peripheral.
#[allow(non_camel_case_types)]
pub struct PLL_SYS {
    _private: (),
}
pub type PllSys = PLL_SYS;
impl PLL_SYS {
    pub const PTR: *const u8 = 0x40020000 as *const u8;
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

impl core::ops::Deref for PLL_SYS {
    type Target = crate::generated::pll_sys::RegisterBlock;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*(Self::PTR.cast()) }
    }
}

unsafe impl Send for PLL_SYS {}

/// RP1 PLL_VIDEO register block with supplemental register semantics. Alias behavior: normal read/write access. PLL layout recovered from the Linux RP1 common-clock driver. Unknown offsets remain split RAW arrays.
/// Atomic view: Rp1Native alias windows; ownership remains with this logical peripheral.
#[allow(non_camel_case_types)]
pub struct PLL_VIDEO {
    _private: (),
}
pub type PllVideo = PLL_VIDEO;
impl PLL_VIDEO {
    pub const PTR: *const u8 = 0x40028000 as *const u8;
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

impl core::ops::Deref for PLL_VIDEO {
    type Target = crate::generated::pll_video::RegisterBlock;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*(Self::PTR.cast()) }
    }
}

unsafe impl Send for PLL_VIDEO {}

/// RP1 POWER_RW register block. Bus=APB. Atomic alias: normal read/write access.
/// Atomic view: Rp1Native alias windows; ownership remains with this logical peripheral.
/// Address-only or externally delegated block; unknown register layout is intentionally hidden.
#[allow(non_camel_case_types)]
pub struct POWER {
    _private: (),
}
pub type Power = POWER;
impl POWER {
    pub const PTR: *const u8 = 0x40010000 as *const u8;
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

unsafe impl Send for POWER {}

/// RP1 PWM0_RW register block. Bus=APB. Atomic alias: normal read/write access.
/// Atomic view: Rp1Native alias windows; ownership remains with this logical peripheral.
#[allow(non_camel_case_types)]
pub struct PWM0 {
    _private: (),
}
pub type Pwm0 = PWM0;
impl PWM0 {
    pub const PTR: *const u8 = 0x40098000 as *const u8;
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

impl core::ops::Deref for PWM0 {
    type Target = crate::generated::pwm0::RegisterBlock;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*(Self::PTR.cast()) }
    }
}

unsafe impl Send for PWM0 {}

/// RP1 PWM1_RW register block. Bus=APB. Atomic alias: normal read/write access.
/// Atomic view: Rp1Native alias windows; ownership remains with this logical peripheral.
#[allow(non_camel_case_types)]
pub struct PWM1 {
    _private: (),
}
pub type Pwm1 = PWM1;
impl PWM1 {
    pub const PTR: *const u8 = 0x4009c000 as *const u8;
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

impl core::ops::Deref for PWM1 {
    type Target = crate::generated::pwm1::RegisterBlock;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*(Self::PTR.cast()) }
    }
}

unsafe impl Send for PWM1 {}

/// Logical reset controller. XOR/SET/CLR are views of this ownership, not independent peripherals.
/// Atomic view: Rp1Native alias windows; ownership remains with this logical peripheral.
#[allow(non_camel_case_types)]
pub struct RESETS {
    _private: (),
}
pub type Resets = RESETS;
impl RESETS {
    pub const PTR: *const u8 = 0x40014000 as *const u8;
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

impl core::ops::Deref for RESETS {
    type Target = crate::generated::resets::RegisterBlock;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*(Self::PTR.cast()) }
    }
}

unsafe impl Send for RESETS {}

/// RP1 ROSC0_RW register block. Bus=APB. Atomic alias: normal read/write access.
/// Atomic view: Rp1Native alias windows; ownership remains with this logical peripheral.
/// Address-only or externally delegated block; unknown register layout is intentionally hidden.
#[allow(non_camel_case_types)]
pub struct ROSC0 {
    _private: (),
}
pub type Rosc0 = ROSC0;
impl ROSC0 {
    pub const PTR: *const u8 = 0x40168000 as *const u8;
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

unsafe impl Send for ROSC0 {}

/// RP1 ROSC1_RW register block. Bus=APB. Atomic alias: normal read/write access.
/// Atomic view: Rp1Native alias windows; ownership remains with this logical peripheral.
/// Address-only or externally delegated block; unknown register layout is intentionally hidden.
#[allow(non_camel_case_types)]
pub struct ROSC1 {
    _private: (),
}
pub type Rosc1 = ROSC1;
impl ROSC1 {
    pub const PTR: *const u8 = 0x4016c000 as *const u8;
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

unsafe impl Send for ROSC1 {}

/// RP1 SDIO0 register block. Bus=AHB. No RP1 atomic alias listed.
/// External IP instance delegated to `sdhci-compatible-host`; only RP1 instance metadata is provided.
/// Address-only or externally delegated block; unknown register layout is intentionally hidden.
#[allow(non_camel_case_types)]
pub struct SDIO0 {
    _private: (),
}
pub type Sdio0 = SDIO0;
impl SDIO0 {
    pub const PTR: *const u8 = 0x40180000 as *const u8;
    pub const EXTERNAL_IP: &str = "sdhci-compatible-host";
    pub const CLOCK_DEPENDENCY: &str = "platform clock dependency; see RP1 integration";
    pub const RESET_DEPENDENCY: &str = "RESETS field SDIO0 when present";
    pub const PINMUX_DEPENDENCY: &str = "IO_BANK/PADS function selection where externally pinned";
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

unsafe impl Send for SDIO0 {}

/// RP1 SDIO0_CFG_RW register block. Bus=APB. Atomic alias: normal read/write access.
/// Atomic view: Rp1Native alias windows; ownership remains with this logical peripheral.
/// Address-only or externally delegated block; unknown register layout is intentionally hidden.
#[allow(non_camel_case_types)]
pub struct SDIO0_CFG {
    _private: (),
}
pub type Sdio0Cfg = SDIO0_CFG;
impl SDIO0_CFG {
    pub const PTR: *const u8 = 0x400b0000 as *const u8;
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

unsafe impl Send for SDIO0_CFG {}

/// RP1 SDIO1 register block. Bus=AHB. No RP1 atomic alias listed.
/// External IP instance delegated to `sdhci-compatible-host`; only RP1 instance metadata is provided.
/// Address-only or externally delegated block; unknown register layout is intentionally hidden.
#[allow(non_camel_case_types)]
pub struct SDIO1 {
    _private: (),
}
pub type Sdio1 = SDIO1;
impl SDIO1 {
    pub const PTR: *const u8 = 0x40184000 as *const u8;
    pub const EXTERNAL_IP: &str = "sdhci-compatible-host";
    pub const CLOCK_DEPENDENCY: &str = "platform clock dependency; see RP1 integration";
    pub const RESET_DEPENDENCY: &str = "RESETS field SDIO1 when present";
    pub const PINMUX_DEPENDENCY: &str = "IO_BANK/PADS function selection where externally pinned";
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

unsafe impl Send for SDIO1 {}

/// RP1 SDIO1_CFG_RW register block. Bus=APB. Atomic alias: normal read/write access.
/// Atomic view: Rp1Native alias windows; ownership remains with this logical peripheral.
/// Address-only or externally delegated block; unknown register layout is intentionally hidden.
#[allow(non_camel_case_types)]
pub struct SDIO1_CFG {
    _private: (),
}
pub type Sdio1Cfg = SDIO1_CFG;
impl SDIO1_CFG {
    pub const PTR: *const u8 = 0x400b4000 as *const u8;
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

unsafe impl Send for SDIO1_CFG {}

/// RP1 SPI0 register block. Bus=APB. No RP1 atomic alias listed.
/// External IP instance delegated to `synopsys-dw-apb-ssi`; only RP1 instance metadata is provided.
/// Address-only or externally delegated block; unknown register layout is intentionally hidden.
#[allow(non_camel_case_types)]
pub struct SPI0 {
    _private: (),
}
pub type Spi0 = SPI0;
impl SPI0 {
    pub const PTR: *const u8 = 0x40050000 as *const u8;
    pub const EXTERNAL_IP: &str = "synopsys-dw-apb-ssi";
    pub const CLOCK_DEPENDENCY: &str = "ClockId::Sys";
    pub const RESET_DEPENDENCY: &str = "RESETS field SPI0 when present";
    pub const PINMUX_DEPENDENCY: &str = "IO_BANK/PADS function selection where externally pinned";
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

unsafe impl Send for SPI0 {}

/// RP1 SPI1 register block. Bus=APB. No RP1 atomic alias listed.
/// External IP instance delegated to `synopsys-dw-apb-ssi`; only RP1 instance metadata is provided.
/// Address-only or externally delegated block; unknown register layout is intentionally hidden.
#[allow(non_camel_case_types)]
pub struct SPI1 {
    _private: (),
}
pub type Spi1 = SPI1;
impl SPI1 {
    pub const PTR: *const u8 = 0x40054000 as *const u8;
    pub const EXTERNAL_IP: &str = "synopsys-dw-apb-ssi";
    pub const CLOCK_DEPENDENCY: &str = "ClockId::Sys";
    pub const RESET_DEPENDENCY: &str = "RESETS field SPI1 when present";
    pub const PINMUX_DEPENDENCY: &str = "IO_BANK/PADS function selection where externally pinned";
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

unsafe impl Send for SPI1 {}

/// RP1 SPI2 register block. Bus=APB. No RP1 atomic alias listed.
/// External IP instance delegated to `synopsys-dw-apb-ssi`; only RP1 instance metadata is provided.
/// Address-only or externally delegated block; unknown register layout is intentionally hidden.
#[allow(non_camel_case_types)]
pub struct SPI2 {
    _private: (),
}
pub type Spi2 = SPI2;
impl SPI2 {
    pub const PTR: *const u8 = 0x40058000 as *const u8;
    pub const EXTERNAL_IP: &str = "synopsys-dw-apb-ssi";
    pub const CLOCK_DEPENDENCY: &str = "ClockId::Sys";
    pub const RESET_DEPENDENCY: &str = "RESETS field SPI2 when present";
    pub const PINMUX_DEPENDENCY: &str = "IO_BANK/PADS function selection where externally pinned";
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

unsafe impl Send for SPI2 {}

/// RP1 SPI3 register block. Bus=APB. No RP1 atomic alias listed.
/// External IP instance delegated to `synopsys-dw-apb-ssi`; only RP1 instance metadata is provided.
/// Address-only or externally delegated block; unknown register layout is intentionally hidden.
#[allow(non_camel_case_types)]
pub struct SPI3 {
    _private: (),
}
pub type Spi3 = SPI3;
impl SPI3 {
    pub const PTR: *const u8 = 0x4005c000 as *const u8;
    pub const EXTERNAL_IP: &str = "synopsys-dw-apb-ssi";
    pub const CLOCK_DEPENDENCY: &str = "ClockId::Sys";
    pub const RESET_DEPENDENCY: &str = "RESETS field SPI3 when present";
    pub const PINMUX_DEPENDENCY: &str = "IO_BANK/PADS function selection where externally pinned";
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

unsafe impl Send for SPI3 {}

/// RP1 SPI4 register block. Bus=APB. No RP1 atomic alias listed.
/// External IP instance delegated to `synopsys-dw-apb-ssi`; only RP1 instance metadata is provided.
/// Address-only or externally delegated block; unknown register layout is intentionally hidden.
#[allow(non_camel_case_types)]
pub struct SPI4 {
    _private: (),
}
pub type Spi4 = SPI4;
impl SPI4 {
    pub const PTR: *const u8 = 0x40060000 as *const u8;
    pub const EXTERNAL_IP: &str = "synopsys-dw-apb-ssi";
    pub const CLOCK_DEPENDENCY: &str = "ClockId::Sys";
    pub const RESET_DEPENDENCY: &str = "RESETS field SPI4 when present";
    pub const PINMUX_DEPENDENCY: &str = "IO_BANK/PADS function selection where externally pinned";
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

unsafe impl Send for SPI4 {}

/// RP1 SPI5 register block. Bus=APB. No RP1 atomic alias listed.
/// External IP instance delegated to `synopsys-dw-apb-ssi`; only RP1 instance metadata is provided.
/// Address-only or externally delegated block; unknown register layout is intentionally hidden.
#[allow(non_camel_case_types)]
pub struct SPI5 {
    _private: (),
}
pub type Spi5 = SPI5;
impl SPI5 {
    pub const PTR: *const u8 = 0x40064000 as *const u8;
    pub const EXTERNAL_IP: &str = "synopsys-dw-apb-ssi";
    pub const CLOCK_DEPENDENCY: &str = "ClockId::Sys";
    pub const RESET_DEPENDENCY: &str = "RESETS field SPI5 when present";
    pub const PINMUX_DEPENDENCY: &str = "IO_BANK/PADS function selection where externally pinned";
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

unsafe impl Send for SPI5 {}

/// RP1 SPI6 register block. Bus=APB. No RP1 atomic alias listed.
/// External IP instance delegated to `synopsys-dw-apb-ssi`; only RP1 instance metadata is provided.
/// Address-only or externally delegated block; unknown register layout is intentionally hidden.
#[allow(non_camel_case_types)]
pub struct SPI6 {
    _private: (),
}
pub type Spi6 = SPI6;
impl SPI6 {
    pub const PTR: *const u8 = 0x40068000 as *const u8;
    pub const EXTERNAL_IP: &str = "synopsys-dw-apb-ssi";
    pub const CLOCK_DEPENDENCY: &str = "ClockId::Sys";
    pub const RESET_DEPENDENCY: &str = "RESETS field SPI6 when present";
    pub const PINMUX_DEPENDENCY: &str = "IO_BANK/PADS function selection where externally pinned";
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

unsafe impl Send for SPI6 {}

/// RP1 SPI7 register block. Bus=APB. No RP1 atomic alias listed.
/// External IP instance delegated to `synopsys-dw-apb-ssi`; only RP1 instance metadata is provided.
/// Address-only or externally delegated block; unknown register layout is intentionally hidden.
#[allow(non_camel_case_types)]
pub struct SPI7 {
    _private: (),
}
pub type Spi7 = SPI7;
impl SPI7 {
    pub const PTR: *const u8 = 0x4006c000 as *const u8;
    pub const EXTERNAL_IP: &str = "synopsys-dw-apb-ssi";
    pub const CLOCK_DEPENDENCY: &str = "ClockId::Sys";
    pub const RESET_DEPENDENCY: &str = "RESETS field SPI7 when present";
    pub const PINMUX_DEPENDENCY: &str = "IO_BANK/PADS function selection where externally pinned";
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

unsafe impl Send for SPI7 {}

/// RP1 SPI8 register block. Bus=APB. No RP1 atomic alias listed.
/// External IP instance delegated to `synopsys-dw-apb-ssi`; only RP1 instance metadata is provided.
/// Address-only or externally delegated block; unknown register layout is intentionally hidden.
#[allow(non_camel_case_types)]
pub struct SPI8 {
    _private: (),
}
pub type Spi8 = SPI8;
impl SPI8 {
    pub const PTR: *const u8 = 0x4004c000 as *const u8;
    pub const EXTERNAL_IP: &str = "synopsys-dw-apb-ssi";
    pub const CLOCK_DEPENDENCY: &str = "ClockId::Sys";
    pub const RESET_DEPENDENCY: &str = "RESETS field SPI8 when present";
    pub const PINMUX_DEPENDENCY: &str = "IO_BANK/PADS function selection where externally pinned";
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

unsafe impl Send for SPI8 {}

/// RP1 SYSCFG_RW register block. Bus=APB. Atomic alias: normal read/write access.
/// Atomic view: Rp1Native alias windows; ownership remains with this logical peripheral.
/// Address-only or externally delegated block; unknown register layout is intentionally hidden.
#[allow(non_camel_case_types)]
pub struct SYSCFG {
    _private: (),
}
pub type Syscfg = SYSCFG;
impl SYSCFG {
    pub const PTR: *const u8 = 0x40008000 as *const u8;
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

unsafe impl Send for SYSCFG {}

/// RP1 SYSINFO_RW register block. Bus=APB. Atomic alias: normal read/write access.
/// Atomic view: Rp1Native alias windows; ownership remains with this logical peripheral.
/// Address-only or externally delegated block; unknown register layout is intentionally hidden.
#[allow(non_camel_case_types)]
pub struct SYSINFO {
    _private: (),
}
pub type Sysinfo = SYSINFO;
impl SYSINFO {
    pub const PTR: *const u8 = 0x40000000 as *const u8;
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

unsafe impl Send for SYSINFO {}

/// RP1 SYS_RIO0_RW register block. Bus=APB. Atomic alias: normal read/write access.
/// Atomic view: Rp1Native alias windows; ownership remains with this logical peripheral.
#[allow(non_camel_case_types)]
pub struct SYS_RIO0 {
    _private: (),
}
pub type SysRio0 = SYS_RIO0;
impl SYS_RIO0 {
    pub const PTR: *const u8 = 0x400e0000 as *const u8;
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

impl core::ops::Deref for SYS_RIO0 {
    type Target = crate::generated::sys_rio0::RegisterBlock;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*(Self::PTR.cast()) }
    }
}

unsafe impl Send for SYS_RIO0 {}

/// RP1 SYS_RIO1 register block with supplemental register semantics. Alias behavior: normal read/write access. Bank geometry is recovered from the Linux pinctrl driver; register fields are inherited from the public bank-0 layout. Unknown offsets remain split RAW arrays.
/// Atomic view: Rp1Native alias windows; ownership remains with this logical peripheral.
#[allow(non_camel_case_types)]
pub struct SYS_RIO1 {
    _private: (),
}
pub type SysRio1 = SYS_RIO1;
impl SYS_RIO1 {
    pub const PTR: *const u8 = 0x400e4000 as *const u8;
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

impl core::ops::Deref for SYS_RIO1 {
    type Target = crate::generated::sys_rio1::RegisterBlock;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*(Self::PTR.cast()) }
    }
}

unsafe impl Send for SYS_RIO1 {}

/// RP1 SYS_RIO2 register block with supplemental register semantics. Alias behavior: normal read/write access. Bank geometry is recovered from the Linux pinctrl driver; register fields are inherited from the public bank-0 layout. Unknown offsets remain split RAW arrays.
/// Atomic view: Rp1Native alias windows; ownership remains with this logical peripheral.
#[allow(non_camel_case_types)]
pub struct SYS_RIO2 {
    _private: (),
}
pub type SysRio2 = SYS_RIO2;
impl SYS_RIO2 {
    pub const PTR: *const u8 = 0x400e8000 as *const u8;
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

impl core::ops::Deref for SYS_RIO2 {
    type Target = crate::generated::sys_rio2::RegisterBlock;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*(Self::PTR.cast()) }
    }
}

unsafe impl Send for SYS_RIO2 {}

/// RP1 TICKS block at 0x40174000: 8 groups, 12-byte group stride, and 3 registers per group. Boot ROM initialization writes cycle value 0x32 and enables each group; that software initialization is distinct from hardware reset values and does not authorize additional writers.
/// Atomic view: Rp1Native alias windows; ownership remains with this logical peripheral.
#[allow(non_camel_case_types)]
pub struct TICKS {
    _private: (),
}
pub type Ticks = TICKS;
impl TICKS {
    pub const PTR: *const u8 = 0x40174000 as *const u8;
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

impl core::ops::Deref for TICKS {
    type Target = crate::generated::ticks::RegisterBlock;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*(Self::PTR.cast()) }
    }
}

unsafe impl Send for TICKS {}

/// RP1 timer block. RAW_HIGH/RAW_LOW advance at one tick per microsecond; coherent raw reads use high-low-high and retry on rollover. Write protocols are exposed only where independently verified.
/// Atomic view: Rp1Native alias windows; ownership remains with this logical peripheral.
#[allow(non_camel_case_types)]
pub struct TIMER {
    _private: (),
}
pub type Timer = TIMER;
impl TIMER {
    pub const PTR: *const u8 = 0x400ac000 as *const u8;
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

impl core::ops::Deref for TIMER {
    type Target = crate::generated::timer::RegisterBlock;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*(Self::PTR.cast()) }
    }
}

unsafe impl Send for TIMER {}

/// PL011 register model is supplied by arm-pl011-uart.
/// External IP instance delegated to `arm-pl011`; only RP1 instance metadata is provided.
/// Address-only or externally delegated block; unknown register layout is intentionally hidden.
#[allow(non_camel_case_types)]
pub struct UART0 {
    _private: (),
}
pub type Uart0 = UART0;
impl UART0 {
    pub const PTR: *const u8 = 0x40030000 as *const u8;
    pub const EXTERNAL_IP: &str = "arm-pl011";
    pub const CLOCK_DEPENDENCY: &str = "uartclk and apb_pclk";
    pub const RESET_DEPENDENCY: &str = "RESETS.CTRL1.UART0";
    pub const PINMUX_DEPENDENCY: &str = "IO_BANK/PADS selection";
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

unsafe impl Send for UART0 {}

/// RP1 UART1 register block. Bus=APB. No RP1 atomic alias listed.
/// External IP instance delegated to `arm-pl011`; only RP1 instance metadata is provided.
/// Address-only or externally delegated block; unknown register layout is intentionally hidden.
#[allow(non_camel_case_types)]
pub struct UART1 {
    _private: (),
}
pub type Uart1 = UART1;
impl UART1 {
    pub const PTR: *const u8 = 0x40034000 as *const u8;
    pub const EXTERNAL_IP: &str = "arm-pl011";
    pub const CLOCK_DEPENDENCY: &str = "uartclk and apb_pclk";
    pub const RESET_DEPENDENCY: &str = "RESETS.CTRL1.UART1";
    pub const PINMUX_DEPENDENCY: &str = "IO_BANK/PADS selection";
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

unsafe impl Send for UART1 {}

/// RP1 UART2 register block. Bus=APB. No RP1 atomic alias listed.
/// External IP instance delegated to `arm-pl011`; only RP1 instance metadata is provided.
/// Address-only or externally delegated block; unknown register layout is intentionally hidden.
#[allow(non_camel_case_types)]
pub struct UART2 {
    _private: (),
}
pub type Uart2 = UART2;
impl UART2 {
    pub const PTR: *const u8 = 0x40038000 as *const u8;
    pub const EXTERNAL_IP: &str = "arm-pl011";
    pub const CLOCK_DEPENDENCY: &str = "uartclk and apb_pclk";
    pub const RESET_DEPENDENCY: &str = "RESETS.CTRL1.UART2";
    pub const PINMUX_DEPENDENCY: &str = "IO_BANK/PADS selection";
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

unsafe impl Send for UART2 {}

/// RP1 UART3 register block. Bus=APB. No RP1 atomic alias listed.
/// External IP instance delegated to `arm-pl011`; only RP1 instance metadata is provided.
/// Address-only or externally delegated block; unknown register layout is intentionally hidden.
#[allow(non_camel_case_types)]
pub struct UART3 {
    _private: (),
}
pub type Uart3 = UART3;
impl UART3 {
    pub const PTR: *const u8 = 0x4003c000 as *const u8;
    pub const EXTERNAL_IP: &str = "arm-pl011";
    pub const CLOCK_DEPENDENCY: &str = "uartclk and apb_pclk";
    pub const RESET_DEPENDENCY: &str = "RESETS.CTRL1.UART3";
    pub const PINMUX_DEPENDENCY: &str = "IO_BANK/PADS selection";
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

unsafe impl Send for UART3 {}

/// RP1 UART4 register block. Bus=APB. No RP1 atomic alias listed.
/// External IP instance delegated to `arm-pl011`; only RP1 instance metadata is provided.
/// Address-only or externally delegated block; unknown register layout is intentionally hidden.
#[allow(non_camel_case_types)]
pub struct UART4 {
    _private: (),
}
pub type Uart4 = UART4;
impl UART4 {
    pub const PTR: *const u8 = 0x40040000 as *const u8;
    pub const EXTERNAL_IP: &str = "arm-pl011";
    pub const CLOCK_DEPENDENCY: &str = "uartclk and apb_pclk";
    pub const RESET_DEPENDENCY: &str = "RESETS.CTRL1.UART4";
    pub const PINMUX_DEPENDENCY: &str = "IO_BANK/PADS selection";
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

unsafe impl Send for UART4 {}

/// RP1 UART5 register block. Bus=APB. No RP1 atomic alias listed.
/// External IP instance delegated to `arm-pl011`; only RP1 instance metadata is provided.
/// Address-only or externally delegated block; unknown register layout is intentionally hidden.
#[allow(non_camel_case_types)]
pub struct UART5 {
    _private: (),
}
pub type Uart5 = UART5;
impl UART5 {
    pub const PTR: *const u8 = 0x40044000 as *const u8;
    pub const EXTERNAL_IP: &str = "arm-pl011";
    pub const CLOCK_DEPENDENCY: &str = "uartclk and apb_pclk";
    pub const RESET_DEPENDENCY: &str = "RESETS.CTRL1.UART5";
    pub const PINMUX_DEPENDENCY: &str = "IO_BANK/PADS selection";
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

unsafe impl Send for UART5 {}

/// RP1 USBHOST0 register block. Bus=AXI3. No RP1 atomic alias listed.
/// External IP instance delegated to `usb-xhci-host`; only RP1 instance metadata is provided.
/// Address-only or externally delegated block; unknown register layout is intentionally hidden.
#[allow(non_camel_case_types)]
pub struct USBHOST0 {
    _private: (),
}
pub type Usbhost0 = USBHOST0;
impl USBHOST0 {
    pub const PTR: *const u8 = 0x40200000 as *const u8;
    pub const EXTERNAL_IP: &str = "usb-xhci-host";
    pub const CLOCK_DEPENDENCY: &str = "ClockId::Usbh0Microframe + ClockId::Usbh0Suspend";
    pub const RESET_DEPENDENCY: &str = "RESETS field USBHOST0 when present";
    pub const PINMUX_DEPENDENCY: &str = "IO_BANK/PADS function selection where externally pinned";
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

unsafe impl Send for USBHOST0 {}

/// RP1 USBHOST0_CFG_RW register block. Bus=APB. Atomic alias: normal read/write access.
/// Atomic view: Rp1Native alias windows; ownership remains with this logical peripheral.
/// Address-only or externally delegated block; unknown register layout is intentionally hidden.
#[allow(non_camel_case_types)]
pub struct USBHOST0_CFG {
    _private: (),
}
pub type Usbhost0Cfg = USBHOST0_CFG;
impl USBHOST0_CFG {
    pub const PTR: *const u8 = 0x40160000 as *const u8;
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

unsafe impl Send for USBHOST0_CFG {}

/// RP1 USBHOST1 register block. Bus=AXI3. No RP1 atomic alias listed.
/// External IP instance delegated to `usb-xhci-host`; only RP1 instance metadata is provided.
/// Address-only or externally delegated block; unknown register layout is intentionally hidden.
#[allow(non_camel_case_types)]
pub struct USBHOST1 {
    _private: (),
}
pub type Usbhost1 = USBHOST1;
impl USBHOST1 {
    pub const PTR: *const u8 = 0x40300000 as *const u8;
    pub const EXTERNAL_IP: &str = "usb-xhci-host";
    pub const CLOCK_DEPENDENCY: &str = "ClockId::Usbh1Microframe + ClockId::Usbh1Suspend";
    pub const RESET_DEPENDENCY: &str = "RESETS field USBHOST1 when present";
    pub const PINMUX_DEPENDENCY: &str = "IO_BANK/PADS function selection where externally pinned";
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

unsafe impl Send for USBHOST1 {}

/// RP1 USBHOST1_CFG_RW register block. Bus=APB. Atomic alias: normal read/write access.
/// Atomic view: Rp1Native alias windows; ownership remains with this logical peripheral.
/// Address-only or externally delegated block; unknown register layout is intentionally hidden.
#[allow(non_camel_case_types)]
pub struct USBHOST1_CFG {
    _private: (),
}
pub type Usbhost1Cfg = USBHOST1_CFG;
impl USBHOST1_CFG {
    pub const PTR: *const u8 = 0x40164000 as *const u8;
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

unsafe impl Send for USBHOST1_CFG {}

/// RP1 VBUSCTRL_RW register block. Bus=APB. Atomic alias: normal read/write access.
/// Atomic view: Rp1Native alias windows; ownership remains with this logical peripheral.
/// Address-only or externally delegated block; unknown register layout is intentionally hidden.
#[allow(non_camel_case_types)]
pub struct VBUSCTRL {
    _private: (),
}
pub type Vbusctrl = VBUSCTRL;
impl VBUSCTRL {
    pub const PTR: *const u8 = 0x40170000 as *const u8;
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

unsafe impl Send for VBUSCTRL {}

/// RP1 VIDEO_OUT_CFG_RW register block. Bus=APB. Atomic alias: normal read/write access.
/// Atomic view: Rp1Native alias windows; ownership remains with this logical peripheral.
/// Address-only or externally delegated block; unknown register layout is intentionally hidden.
#[allow(non_camel_case_types)]
pub struct VIDEO_OUT_CFG {
    _private: (),
}
pub type VideoOutCfg = VIDEO_OUT_CFG;
impl VIDEO_OUT_CFG {
    pub const PTR: *const u8 = 0x40140000 as *const u8;
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

unsafe impl Send for VIDEO_OUT_CFG {}

/// RP1 VIDEO_OUT_DPI_RW register block. Bus=APB. Atomic alias: normal read/write access.
/// Atomic view: Rp1Native alias windows; ownership remains with this logical peripheral.
/// Address-only or externally delegated block; unknown register layout is intentionally hidden.
#[allow(non_camel_case_types)]
pub struct VIDEO_OUT_DPI {
    _private: (),
}
pub type VideoOutDpi = VIDEO_OUT_DPI;
impl VIDEO_OUT_DPI {
    pub const PTR: *const u8 = 0x40148000 as *const u8;
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

unsafe impl Send for VIDEO_OUT_DPI {}

/// RP1 VIDEO_OUT_VEC_RW register block. Bus=APB. Atomic alias: normal read/write access.
/// Atomic view: Rp1Native alias windows; ownership remains with this logical peripheral.
/// Address-only or externally delegated block; unknown register layout is intentionally hidden.
#[allow(non_camel_case_types)]
pub struct VIDEO_OUT_VEC {
    _private: (),
}
pub type VideoOutVec = VIDEO_OUT_VEC;
impl VIDEO_OUT_VEC {
    pub const PTR: *const u8 = 0x40144000 as *const u8;
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

unsafe impl Send for VIDEO_OUT_VEC {}

/// RP2040-compatible layout where independently verified; no Rp1Lk provenance remains.
/// Atomic view: Rp1Native alias windows; ownership remains with this logical peripheral.
#[allow(non_camel_case_types)]
pub struct WATCHDOG {
    _private: (),
}
pub type Watchdog = WATCHDOG;
impl WATCHDOG {
    pub const PTR: *const u8 = 0x40154000 as *const u8;
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

impl core::ops::Deref for WATCHDOG {
    type Target = crate::generated::watchdog::RegisterBlock;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*(Self::PTR.cast()) }
    }
}

unsafe impl Send for WATCHDOG {}

/// RP1 XOSC_RW register block. Bus=APB. Atomic alias: normal read/write access.
/// Atomic view: Rp1Native alias windows; ownership remains with this logical peripheral.
/// Address-only or externally delegated block; unknown register layout is intentionally hidden.
#[allow(non_camel_case_types)]
pub struct XOSC {
    _private: (),
}
pub type Xosc = XOSC;
impl XOSC {
    pub const PTR: *const u8 = 0x40150000 as *const u8;
    #[inline(always)]
    pub const fn ptr() -> *const u8 {
        Self::PTR
    }
}

unsafe impl Send for XOSC {}

#[allow(non_snake_case)]
pub struct Peripherals {
    pub ADC: ADC,
    pub AUDIO_IN: AUDIO_IN,
    pub AUDIO_OUT: AUDIO_OUT,
    pub BUSFABRIC_AXISHIM: BUSFABRIC_AXISHIM,
    pub BUSFABRIC_MONITOR: BUSFABRIC_MONITOR,
    pub CLOCKS_MAIN: CLOCKS_MAIN,
    pub CLOCKS_VIDEO: CLOCKS_VIDEO,
    pub DMA: DMA,
    pub DMA_TICK: DMA_TICK,
    pub ETH: ETH,
    pub ETH_CFG: ETH_CFG,
    pub EXAC: EXAC,
    pub I2C0: I2C0,
    pub I2C1: I2C1,
    pub I2C2: I2C2,
    pub I2C3: I2C3,
    pub I2C4: I2C4,
    pub I2C5: I2C5,
    pub I2C6: I2C6,
    pub I2S0: I2S0,
    pub I2S1: I2S1,
    pub I2S2: I2S2,
    pub IO_BANK0: IO_BANK0,
    pub IO_BANK1: IO_BANK1,
    pub IO_BANK2: IO_BANK2,
    pub MIPI0_CFG: MIPI0_CFG,
    pub MIPI0_CSIDMA: MIPI0_CSIDMA,
    pub MIPI0_CSIHOST: MIPI0_CSIHOST,
    pub MIPI0_DSIDMA: MIPI0_DSIDMA,
    pub MIPI0_DSIHOST: MIPI0_DSIHOST,
    pub MIPI0_ISP: MIPI0_ISP,
    pub MIPI1_CFG: MIPI1_CFG,
    pub MIPI1_CSIDMA: MIPI1_CSIDMA,
    pub MIPI1_CSIHOST: MIPI1_CSIHOST,
    pub MIPI1_DSIDMA: MIPI1_DSIDMA,
    pub MIPI1_DSIHOST: MIPI1_DSIHOST,
    pub MIPI1_ISP: MIPI1_ISP,
    pub OTP: OTP,
    pub PADS_BANK0: PADS_BANK0,
    pub PADS_BANK1: PADS_BANK1,
    pub PADS_BANK2: PADS_BANK2,
    pub PADS_ETH: PADS_ETH,
    pub PCIE_APBS: PCIE_APBS,
    pub PIO: PIO,
    pub PLL_AUDIO: PLL_AUDIO,
    pub PLL_SYS: PLL_SYS,
    pub PLL_VIDEO: PLL_VIDEO,
    pub POWER: POWER,
    pub PWM0: PWM0,
    pub PWM1: PWM1,
    pub RESETS: RESETS,
    pub ROSC0: ROSC0,
    pub ROSC1: ROSC1,
    pub SDIO0: SDIO0,
    pub SDIO0_CFG: SDIO0_CFG,
    pub SDIO1: SDIO1,
    pub SDIO1_CFG: SDIO1_CFG,
    pub SPI0: SPI0,
    pub SPI1: SPI1,
    pub SPI2: SPI2,
    pub SPI3: SPI3,
    pub SPI4: SPI4,
    pub SPI5: SPI5,
    pub SPI6: SPI6,
    pub SPI7: SPI7,
    pub SPI8: SPI8,
    pub SYSCFG: SYSCFG,
    pub SYSINFO: SYSINFO,
    pub SYS_RIO0: SYS_RIO0,
    pub SYS_RIO1: SYS_RIO1,
    pub SYS_RIO2: SYS_RIO2,
    pub TICKS: TICKS,
    pub TIMER: TIMER,
    pub UART0: UART0,
    pub UART1: UART1,
    pub UART2: UART2,
    pub UART3: UART3,
    pub UART4: UART4,
    pub UART5: UART5,
    pub USBHOST0: USBHOST0,
    pub USBHOST0_CFG: USBHOST0_CFG,
    pub USBHOST1: USBHOST1,
    pub USBHOST1_CFG: USBHOST1_CFG,
    pub VBUSCTRL: VBUSCTRL,
    pub VIDEO_OUT_CFG: VIDEO_OUT_CFG,
    pub VIDEO_OUT_DPI: VIDEO_OUT_DPI,
    pub VIDEO_OUT_VEC: VIDEO_OUT_VEC,
    pub WATCHDOG: WATCHDOG,
    pub XOSC: XOSC,
}

impl Peripherals {
    /// # Safety
    /// Coordinate ownership across both cores. TAKEN is core-local, not a device-wide lock.
    pub unsafe fn take() -> Option<Self> {
        TAKEN
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .ok()
            .map(|_| unsafe { Self::steal() })
    }
    /// # Safety
    /// The caller must ensure no other `Peripherals` instance is in use.
    pub unsafe fn steal() -> Self {
        TAKEN.store(true, Ordering::Release);
        Self {
            ADC: ADC { _private: () },
            AUDIO_IN: AUDIO_IN { _private: () },
            AUDIO_OUT: AUDIO_OUT { _private: () },
            BUSFABRIC_AXISHIM: BUSFABRIC_AXISHIM { _private: () },
            BUSFABRIC_MONITOR: BUSFABRIC_MONITOR { _private: () },
            CLOCKS_MAIN: CLOCKS_MAIN { _private: () },
            CLOCKS_VIDEO: CLOCKS_VIDEO { _private: () },
            DMA: DMA { _private: () },
            DMA_TICK: DMA_TICK { _private: () },
            ETH: ETH { _private: () },
            ETH_CFG: ETH_CFG { _private: () },
            EXAC: EXAC { _private: () },
            I2C0: I2C0 { _private: () },
            I2C1: I2C1 { _private: () },
            I2C2: I2C2 { _private: () },
            I2C3: I2C3 { _private: () },
            I2C4: I2C4 { _private: () },
            I2C5: I2C5 { _private: () },
            I2C6: I2C6 { _private: () },
            I2S0: I2S0 { _private: () },
            I2S1: I2S1 { _private: () },
            I2S2: I2S2 { _private: () },
            IO_BANK0: IO_BANK0 { _private: () },
            IO_BANK1: IO_BANK1 { _private: () },
            IO_BANK2: IO_BANK2 { _private: () },
            MIPI0_CFG: MIPI0_CFG { _private: () },
            MIPI0_CSIDMA: MIPI0_CSIDMA { _private: () },
            MIPI0_CSIHOST: MIPI0_CSIHOST { _private: () },
            MIPI0_DSIDMA: MIPI0_DSIDMA { _private: () },
            MIPI0_DSIHOST: MIPI0_DSIHOST { _private: () },
            MIPI0_ISP: MIPI0_ISP { _private: () },
            MIPI1_CFG: MIPI1_CFG { _private: () },
            MIPI1_CSIDMA: MIPI1_CSIDMA { _private: () },
            MIPI1_CSIHOST: MIPI1_CSIHOST { _private: () },
            MIPI1_DSIDMA: MIPI1_DSIDMA { _private: () },
            MIPI1_DSIHOST: MIPI1_DSIHOST { _private: () },
            MIPI1_ISP: MIPI1_ISP { _private: () },
            OTP: OTP { _private: () },
            PADS_BANK0: PADS_BANK0 { _private: () },
            PADS_BANK1: PADS_BANK1 { _private: () },
            PADS_BANK2: PADS_BANK2 { _private: () },
            PADS_ETH: PADS_ETH { _private: () },
            PCIE_APBS: PCIE_APBS { _private: () },
            PIO: PIO { _private: () },
            PLL_AUDIO: PLL_AUDIO { _private: () },
            PLL_SYS: PLL_SYS { _private: () },
            PLL_VIDEO: PLL_VIDEO { _private: () },
            POWER: POWER { _private: () },
            PWM0: PWM0 { _private: () },
            PWM1: PWM1 { _private: () },
            RESETS: RESETS { _private: () },
            ROSC0: ROSC0 { _private: () },
            ROSC1: ROSC1 { _private: () },
            SDIO0: SDIO0 { _private: () },
            SDIO0_CFG: SDIO0_CFG { _private: () },
            SDIO1: SDIO1 { _private: () },
            SDIO1_CFG: SDIO1_CFG { _private: () },
            SPI0: SPI0 { _private: () },
            SPI1: SPI1 { _private: () },
            SPI2: SPI2 { _private: () },
            SPI3: SPI3 { _private: () },
            SPI4: SPI4 { _private: () },
            SPI5: SPI5 { _private: () },
            SPI6: SPI6 { _private: () },
            SPI7: SPI7 { _private: () },
            SPI8: SPI8 { _private: () },
            SYSCFG: SYSCFG { _private: () },
            SYSINFO: SYSINFO { _private: () },
            SYS_RIO0: SYS_RIO0 { _private: () },
            SYS_RIO1: SYS_RIO1 { _private: () },
            SYS_RIO2: SYS_RIO2 { _private: () },
            TICKS: TICKS { _private: () },
            TIMER: TIMER { _private: () },
            UART0: UART0 { _private: () },
            UART1: UART1 { _private: () },
            UART2: UART2 { _private: () },
            UART3: UART3 { _private: () },
            UART4: UART4 { _private: () },
            UART5: UART5 { _private: () },
            USBHOST0: USBHOST0 { _private: () },
            USBHOST0_CFG: USBHOST0_CFG { _private: () },
            USBHOST1: USBHOST1 { _private: () },
            USBHOST1_CFG: USBHOST1_CFG { _private: () },
            VBUSCTRL: VBUSCTRL { _private: () },
            VIDEO_OUT_CFG: VIDEO_OUT_CFG { _private: () },
            VIDEO_OUT_DPI: VIDEO_OUT_DPI { _private: () },
            VIDEO_OUT_VEC: VIDEO_OUT_VEC { _private: () },
            WATCHDOG: WATCHDOG { _private: () },
            XOSC: XOSC { _private: () },
        }
    }
}

#[cfg(test)]
mod layout_tests {
    #[test]
    fn adc_base() {
        assert_eq!(super::ADC::PTR as usize, 0x400c8000);
    }
    #[test]
    fn audio_in_base() {
        assert_eq!(super::AUDIO_IN::PTR as usize, 0x40090000);
    }
    #[test]
    fn audio_out_base() {
        assert_eq!(super::AUDIO_OUT::PTR as usize, 0x40094000);
    }
    #[test]
    fn busfabric_axishim_base() {
        assert_eq!(super::BUSFABRIC_AXISHIM::PTR as usize, 0x400c4000);
    }
    #[test]
    fn busfabric_monitor_base() {
        assert_eq!(super::BUSFABRIC_MONITOR::PTR as usize, 0x400c0000);
    }
    #[test]
    fn clocks_main_base() {
        assert_eq!(super::CLOCKS_MAIN::PTR as usize, 0x40018000);
    }
    #[test]
    fn clocks_video_base() {
        assert_eq!(super::CLOCKS_VIDEO::PTR as usize, 0x4001c000);
    }
    #[test]
    fn dma_base() {
        assert_eq!(super::DMA::PTR as usize, 0x40188000);
    }
    #[test]
    fn dma_tick_base() {
        assert_eq!(super::DMA_TICK::PTR as usize, 0x40158000);
    }
    #[test]
    fn eth_base() {
        assert_eq!(super::ETH::PTR as usize, 0x40100000);
    }
    #[test]
    fn eth_cfg_base() {
        assert_eq!(super::ETH_CFG::PTR as usize, 0x40104000);
    }
    #[test]
    fn exac_base() {
        assert_eq!(super::EXAC::PTR as usize, 0x40400000);
    }
    #[test]
    fn i2c0_base() {
        assert_eq!(super::I2C0::PTR as usize, 0x40070000);
    }
    #[test]
    fn i2c1_base() {
        assert_eq!(super::I2C1::PTR as usize, 0x40074000);
    }
    #[test]
    fn i2c2_base() {
        assert_eq!(super::I2C2::PTR as usize, 0x40078000);
    }
    #[test]
    fn i2c3_base() {
        assert_eq!(super::I2C3::PTR as usize, 0x4007c000);
    }
    #[test]
    fn i2c4_base() {
        assert_eq!(super::I2C4::PTR as usize, 0x40080000);
    }
    #[test]
    fn i2c5_base() {
        assert_eq!(super::I2C5::PTR as usize, 0x40084000);
    }
    #[test]
    fn i2c6_base() {
        assert_eq!(super::I2C6::PTR as usize, 0x40088000);
    }
    #[test]
    fn i2s0_base() {
        assert_eq!(super::I2S0::PTR as usize, 0x400a0000);
    }
    #[test]
    fn i2s1_base() {
        assert_eq!(super::I2S1::PTR as usize, 0x400a4000);
    }
    #[test]
    fn i2s2_base() {
        assert_eq!(super::I2S2::PTR as usize, 0x400a8000);
    }
    #[test]
    fn io_bank0_base() {
        assert_eq!(super::IO_BANK0::PTR as usize, 0x400d0000);
    }
    #[test]
    fn io_bank1_base() {
        assert_eq!(super::IO_BANK1::PTR as usize, 0x400d4000);
    }
    #[test]
    fn io_bank2_base() {
        assert_eq!(super::IO_BANK2::PTR as usize, 0x400d8000);
    }
    #[test]
    fn mipi0_cfg_base() {
        assert_eq!(super::MIPI0_CFG::PTR as usize, 0x40120000);
    }
    #[test]
    fn mipi0_csidma_base() {
        assert_eq!(super::MIPI0_CSIDMA::PTR as usize, 0x40110000);
    }
    #[test]
    fn mipi0_csihost_base() {
        assert_eq!(super::MIPI0_CSIHOST::PTR as usize, 0x40114000);
    }
    #[test]
    fn mipi0_dsidma_base() {
        assert_eq!(super::MIPI0_DSIDMA::PTR as usize, 0x40118000);
    }
    #[test]
    fn mipi0_dsihost_base() {
        assert_eq!(super::MIPI0_DSIHOST::PTR as usize, 0x4011c000);
    }
    #[test]
    fn mipi0_isp_base() {
        assert_eq!(super::MIPI0_ISP::PTR as usize, 0x40124000);
    }
    #[test]
    fn mipi1_cfg_base() {
        assert_eq!(super::MIPI1_CFG::PTR as usize, 0x40138000);
    }
    #[test]
    fn mipi1_csidma_base() {
        assert_eq!(super::MIPI1_CSIDMA::PTR as usize, 0x40128000);
    }
    #[test]
    fn mipi1_csihost_base() {
        assert_eq!(super::MIPI1_CSIHOST::PTR as usize, 0x4012c000);
    }
    #[test]
    fn mipi1_dsidma_base() {
        assert_eq!(super::MIPI1_DSIDMA::PTR as usize, 0x40130000);
    }
    #[test]
    fn mipi1_dsihost_base() {
        assert_eq!(super::MIPI1_DSIHOST::PTR as usize, 0x40134000);
    }
    #[test]
    fn mipi1_isp_base() {
        assert_eq!(super::MIPI1_ISP::PTR as usize, 0x4013c000);
    }
    #[test]
    fn otp_base() {
        assert_eq!(super::OTP::PTR as usize, 0x4000c000);
    }
    #[test]
    fn pads_bank0_base() {
        assert_eq!(super::PADS_BANK0::PTR as usize, 0x400f0000);
    }
    #[test]
    fn pads_bank1_base() {
        assert_eq!(super::PADS_BANK1::PTR as usize, 0x400f4000);
    }
    #[test]
    fn pads_bank2_base() {
        assert_eq!(super::PADS_BANK2::PTR as usize, 0x400f8000);
    }
    #[test]
    fn pads_eth_base() {
        assert_eq!(super::PADS_ETH::PTR as usize, 0x400fc000);
    }
    #[test]
    fn pcie_apbs_base() {
        assert_eq!(super::PCIE_APBS::PTR as usize, 0x40108000);
    }
    #[test]
    fn pio_base() {
        assert_eq!(super::PIO::PTR as usize, 0x40178000);
    }
    #[test]
    fn pll_audio_base() {
        assert_eq!(super::PLL_AUDIO::PTR as usize, 0x40024000);
    }
    #[test]
    fn pll_sys_base() {
        assert_eq!(super::PLL_SYS::PTR as usize, 0x40020000);
    }
    #[test]
    fn pll_video_base() {
        assert_eq!(super::PLL_VIDEO::PTR as usize, 0x40028000);
    }
    #[test]
    fn power_base() {
        assert_eq!(super::POWER::PTR as usize, 0x40010000);
    }
    #[test]
    fn pwm0_base() {
        assert_eq!(super::PWM0::PTR as usize, 0x40098000);
    }
    #[test]
    fn pwm1_base() {
        assert_eq!(super::PWM1::PTR as usize, 0x4009c000);
    }
    #[test]
    fn resets_base() {
        assert_eq!(super::RESETS::PTR as usize, 0x40014000);
    }
    #[test]
    fn rosc0_base() {
        assert_eq!(super::ROSC0::PTR as usize, 0x40168000);
    }
    #[test]
    fn rosc1_base() {
        assert_eq!(super::ROSC1::PTR as usize, 0x4016c000);
    }
    #[test]
    fn sdio0_base() {
        assert_eq!(super::SDIO0::PTR as usize, 0x40180000);
    }
    #[test]
    fn sdio0_cfg_base() {
        assert_eq!(super::SDIO0_CFG::PTR as usize, 0x400b0000);
    }
    #[test]
    fn sdio1_base() {
        assert_eq!(super::SDIO1::PTR as usize, 0x40184000);
    }
    #[test]
    fn sdio1_cfg_base() {
        assert_eq!(super::SDIO1_CFG::PTR as usize, 0x400b4000);
    }
    #[test]
    fn spi0_base() {
        assert_eq!(super::SPI0::PTR as usize, 0x40050000);
    }
    #[test]
    fn spi1_base() {
        assert_eq!(super::SPI1::PTR as usize, 0x40054000);
    }
    #[test]
    fn spi2_base() {
        assert_eq!(super::SPI2::PTR as usize, 0x40058000);
    }
    #[test]
    fn spi3_base() {
        assert_eq!(super::SPI3::PTR as usize, 0x4005c000);
    }
    #[test]
    fn spi4_base() {
        assert_eq!(super::SPI4::PTR as usize, 0x40060000);
    }
    #[test]
    fn spi5_base() {
        assert_eq!(super::SPI5::PTR as usize, 0x40064000);
    }
    #[test]
    fn spi6_base() {
        assert_eq!(super::SPI6::PTR as usize, 0x40068000);
    }
    #[test]
    fn spi7_base() {
        assert_eq!(super::SPI7::PTR as usize, 0x4006c000);
    }
    #[test]
    fn spi8_base() {
        assert_eq!(super::SPI8::PTR as usize, 0x4004c000);
    }
    #[test]
    fn syscfg_base() {
        assert_eq!(super::SYSCFG::PTR as usize, 0x40008000);
    }
    #[test]
    fn sysinfo_base() {
        assert_eq!(super::SYSINFO::PTR as usize, 0x40000000);
    }
    #[test]
    fn sys_rio0_base() {
        assert_eq!(super::SYS_RIO0::PTR as usize, 0x400e0000);
    }
    #[test]
    fn sys_rio1_base() {
        assert_eq!(super::SYS_RIO1::PTR as usize, 0x400e4000);
    }
    #[test]
    fn sys_rio2_base() {
        assert_eq!(super::SYS_RIO2::PTR as usize, 0x400e8000);
    }
    #[test]
    fn ticks_base() {
        assert_eq!(super::TICKS::PTR as usize, 0x40174000);
    }
    #[test]
    fn timer_base() {
        assert_eq!(super::TIMER::PTR as usize, 0x400ac000);
    }
    #[test]
    fn uart0_base() {
        assert_eq!(super::UART0::PTR as usize, 0x40030000);
    }
    #[test]
    fn uart1_base() {
        assert_eq!(super::UART1::PTR as usize, 0x40034000);
    }
    #[test]
    fn uart2_base() {
        assert_eq!(super::UART2::PTR as usize, 0x40038000);
    }
    #[test]
    fn uart3_base() {
        assert_eq!(super::UART3::PTR as usize, 0x4003c000);
    }
    #[test]
    fn uart4_base() {
        assert_eq!(super::UART4::PTR as usize, 0x40040000);
    }
    #[test]
    fn uart5_base() {
        assert_eq!(super::UART5::PTR as usize, 0x40044000);
    }
    #[test]
    fn usbhost0_base() {
        assert_eq!(super::USBHOST0::PTR as usize, 0x40200000);
    }
    #[test]
    fn usbhost0_cfg_base() {
        assert_eq!(super::USBHOST0_CFG::PTR as usize, 0x40160000);
    }
    #[test]
    fn usbhost1_base() {
        assert_eq!(super::USBHOST1::PTR as usize, 0x40300000);
    }
    #[test]
    fn usbhost1_cfg_base() {
        assert_eq!(super::USBHOST1_CFG::PTR as usize, 0x40164000);
    }
    #[test]
    fn vbusctrl_base() {
        assert_eq!(super::VBUSCTRL::PTR as usize, 0x40170000);
    }
    #[test]
    fn video_out_cfg_base() {
        assert_eq!(super::VIDEO_OUT_CFG::PTR as usize, 0x40140000);
    }
    #[test]
    fn video_out_dpi_base() {
        assert_eq!(super::VIDEO_OUT_DPI::PTR as usize, 0x40148000);
    }
    #[test]
    fn video_out_vec_base() {
        assert_eq!(super::VIDEO_OUT_VEC::PTR as usize, 0x40144000);
    }
    #[test]
    fn watchdog_base() {
        assert_eq!(super::WATCHDOG::PTR as usize, 0x40154000);
    }
    #[test]
    fn xosc_base() {
        assert_eq!(super::XOSC::PTR as usize, 0x40150000);
    }
}
