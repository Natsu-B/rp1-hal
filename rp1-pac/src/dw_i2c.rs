//! DesignWare APB I2C master registers shared by I2C0..6.
//! DOC: RP1 peripherals §9; Linux i2c-designware-core.h (rpi-6.12.y).
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

/// INTR_STAT/INTR_MASK/RAW_INTR_STAT bits.
pub mod intr {
    pub const RX_UNDER: u32 = 1 << 0;
    pub const RX_OVER: u32 = 1 << 1;
    pub const RX_FULL: u32 = 1 << 2;
    pub const TX_OVER: u32 = 1 << 3;
    pub const TX_EMPTY: u32 = 1 << 4;
    pub const RD_REQ: u32 = 1 << 5;
    pub const TX_ABRT: u32 = 1 << 6;
    pub const RX_DONE: u32 = 1 << 7;
    pub const ACTIVITY: u32 = 1 << 8;
    pub const STOP_DET: u32 = 1 << 9;
    pub const START_DET: u32 = 1 << 10;
    pub const GEN_CALL: u32 = 1 << 11;
    pub const RESTART_DET: u32 = 1 << 12;
}
/// IRQ_WRAP (+0x108) bits.
pub mod irq_wrap {
    pub const IRQ_EN: u32 = 1 << 0;
}

#[repr(C)]
pub struct RegisterBlock {
    pub con: Reg<ReadWrite>,
    pub tar: Reg<ReadWrite>,
    pub sar: Reg<ReadWrite>,
    _reserved0: u32,
    pub data_cmd: Reg<Data>,
    pub ss_scl_hcnt: Reg<ReadWrite>,
    pub ss_scl_lcnt: Reg<ReadWrite>,
    pub fs_scl_hcnt: Reg<ReadWrite>,
    pub fs_scl_lcnt: Reg<ReadWrite>,
    _reserved1: [u32; 2],
    pub intr_stat: Reg<ReadOnly>,
    pub intr_mask: Reg<ReadWrite>,
    pub raw_intr_stat: Reg<ReadOnly>,
    pub rx_tl: Reg<ReadWrite>,
    pub tx_tl: Reg<ReadWrite>,
    pub clr_intr: Reg<ReadClear>,
    pub clr_rx_under: Reg<ReadClear>,
    pub clr_rx_over: Reg<ReadClear>,
    pub clr_tx_over: Reg<ReadClear>,
    pub clr_rd_req: Reg<ReadClear>,
    pub clr_tx_abrt: Reg<ReadClear>,
    pub clr_rx_done: Reg<ReadClear>,
    pub clr_activity: Reg<ReadClear>,
    pub clr_stop_det: Reg<ReadClear>,
    pub clr_start_det: Reg<ReadClear>,
    pub clr_gen_call: Reg<ReadClear>,
    pub enable: Reg<ReadWrite>,
    pub status: Reg<ReadOnly>,
    pub txflr: Reg<ReadOnly>,
    pub rxflr: Reg<ReadOnly>,
    pub sda_hold: Reg<ReadWrite>,
    pub tx_abrt_source: Reg<ReadOnly>,
    _reserved2: [u32; 6],
    pub enable_status: Reg<ReadOnly>,
    pub fs_spklen: Reg<ReadWrite>,
    _reserved3: [u32; 20],
    pub comp_param_1: Reg<ReadOnly>,
    pub comp_version: Reg<ReadOnly>,
    pub comp_type: Reg<ReadOnly>,
    _reserved4: [u32; 2],
    /// RP1 wrapper outside the DesignWare block: [`irq_wrap::IRQ_EN`] routes the controller
    /// interrupt to the RP1 cores (HW on I2C1; STATIC from the standard firmware on the other
    /// instances). Its reset value and any effect on Linux's MSI-X vector are not verified.
    pub irq_wrap: Reg<ReadWrite>,
}

macro_rules! instances {
    ($($t:ty),*) => {$(impl $t {
        pub fn registers(&self) -> &RegisterBlock {
            unsafe { &*Self::PTR.cast::<RegisterBlock>() }
        }
    })*};
}
instances!(
    crate::I2C0,
    crate::I2C1,
    crate::I2C2,
    crate::I2C3,
    crate::I2C4,
    crate::I2C5,
    crate::I2C6
);

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn designware_offsets_and_instances() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, data_cmd), 0x10);
        assert_eq!(core::mem::offset_of!(RegisterBlock, raw_intr_stat), 0x34);
        assert_eq!(core::mem::offset_of!(RegisterBlock, enable), 0x6c);
        assert_eq!(core::mem::offset_of!(RegisterBlock, tx_abrt_source), 0x80);
        assert_eq!(core::mem::offset_of!(RegisterBlock, enable_status), 0x9c);
        assert_eq!(core::mem::offset_of!(RegisterBlock, comp_type), 0xfc);
        assert_eq!(core::mem::offset_of!(RegisterBlock, irq_wrap), 0x108);
        assert_eq!(crate::I2C0::PTR as usize, 0x4007_0000);
        assert_eq!(crate::I2C6::PTR as usize, 0x4008_8000);
    }
}
