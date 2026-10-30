//! DesignWare APB SSI (DW_apb_ssi v4.02a) registers shared by SPI0..8.
//! DOC: RP1 peripherals §3.6 (instances, clk_sys) and Linux spi-dw.h (rpi-6.12.y) for offsets.
//! The regdb keeps SPI0..8 as delegated `synopsys-dw-apb-ssi` blocks (base, reset, IRQ).
use crate::{FifoPop, FifoPush, Readable, Reg, RegisterSpec, UnsafeWritable};

pub struct ReadOnly;
pub struct ReadWrite;
pub struct Data;
/// Reading acknowledges interrupt state; deliberately has no ordinary `read()` API.
pub struct ReadClear;
impl RegisterSpec for ReadOnly {
    type Ux = u32;
}
impl RegisterSpec for ReadWrite {
    type Ux = u32;
}
impl RegisterSpec for Data {
    type Ux = u32;
}
impl RegisterSpec for ReadClear {
    type Ux = u32;
}
impl Readable for ReadOnly {}
impl Readable for ReadWrite {}
impl UnsafeWritable for ReadWrite {}
impl FifoPop for Data {}
impl FifoPush for Data {}
impl FifoPop for ReadClear {}

/// CTRLR0 fields of the APB (PSSI) layout.
pub mod ctrlr0 {
    pub const DFS_MASK: u32 = 0xf;
    pub const DFS32_SHIFT: u32 = 16;
    pub const DFS32_MASK: u32 = 0x1f << DFS32_SHIFT;
    pub const SCPH: u32 = 1 << 6;
    pub const SCPOL: u32 = 1 << 7;
    pub const TMOD_MASK: u32 = 0x3 << 8;
    /// Shift register loop: TXD is fed back to RXD inside the block.
    pub const SRL: u32 = 1 << 11;
}
/// SR bits.
pub mod sr {
    pub const BUSY: u32 = 1 << 0;
    pub const TFNF: u32 = 1 << 1;
    pub const TFE: u32 = 1 << 2;
    pub const RFNE: u32 = 1 << 3;
}
/// RISR/ISR/IMR bits.
pub mod int {
    pub const TXOI: u32 = 1 << 1;
    pub const RXUI: u32 = 1 << 2;
    pub const RXOI: u32 = 1 << 3;
}

#[repr(C)]
pub struct RegisterBlock {
    pub ctrlr0: Reg<ReadWrite>,
    pub ctrlr1: Reg<ReadWrite>,
    pub ssienr: Reg<ReadWrite>,
    pub mwcr: Reg<ReadWrite>,
    pub ser: Reg<ReadWrite>,
    pub baudr: Reg<ReadWrite>,
    pub txftlr: Reg<ReadWrite>,
    pub rxftlr: Reg<ReadWrite>,
    pub txflr: Reg<ReadOnly>,
    pub rxflr: Reg<ReadOnly>,
    pub sr: Reg<ReadOnly>,
    pub imr: Reg<ReadWrite>,
    pub isr: Reg<ReadOnly>,
    pub risr: Reg<ReadOnly>,
    pub txoicr: Reg<ReadClear>,
    pub rxoicr: Reg<ReadClear>,
    pub rxuicr: Reg<ReadClear>,
    pub msticr: Reg<ReadClear>,
    pub icr: Reg<ReadClear>,
    pub dmacr: Reg<ReadWrite>,
    pub dmatdlr: Reg<ReadWrite>,
    pub dmardlr: Reg<ReadWrite>,
    pub idr: Reg<ReadOnly>,
    pub ssi_version_id: Reg<ReadOnly>,
    /// DR0; DR1..35 (+0x64..+0xec) alias the same FIFO.
    pub dr: Reg<Data>,
    _reserved0: [u32; 35],
    pub rx_sample_dly: Reg<ReadWrite>,
}

macro_rules! instances {
    ($($t:ty),*) => {$(impl $t {
        pub fn registers(&self) -> &RegisterBlock {
            unsafe { &*Self::PTR.cast::<RegisterBlock>() }
        }
    })*};
}
instances!(
    crate::SPI0,
    crate::SPI1,
    crate::SPI2,
    crate::SPI3,
    crate::SPI4,
    crate::SPI5,
    crate::SPI6,
    crate::SPI7,
    crate::SPI8
);

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn designware_offsets_and_instances() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, ser), 0x10);
        assert_eq!(core::mem::offset_of!(RegisterBlock, baudr), 0x14);
        assert_eq!(core::mem::offset_of!(RegisterBlock, txflr), 0x20);
        assert_eq!(core::mem::offset_of!(RegisterBlock, sr), 0x28);
        assert_eq!(core::mem::offset_of!(RegisterBlock, risr), 0x34);
        assert_eq!(core::mem::offset_of!(RegisterBlock, icr), 0x48);
        assert_eq!(core::mem::offset_of!(RegisterBlock, ssi_version_id), 0x5c);
        assert_eq!(core::mem::offset_of!(RegisterBlock, dr), 0x60);
        assert_eq!(core::mem::offset_of!(RegisterBlock, rx_sample_dly), 0xf0);
        assert_eq!(crate::SPI0::PTR as usize, 0x4005_0000);
        assert_eq!(crate::SPI5::PTR as usize, 0x4006_4000);
        assert_eq!(crate::SPI8::PTR as usize, 0x4004_c000);
    }
}
