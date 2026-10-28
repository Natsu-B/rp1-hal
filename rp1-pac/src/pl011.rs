//! Standard PL011 slice shared by UART0..5 (DOC: ARM DDI0183; STATIC: reference PoC uart.rs).
use crate::{FifoPop, FifoPush, Readable, Reg, RegisterSpec, UnsafeWritable, WriteOneToClear};
pub struct ReadOnly;
pub struct ReadWrite;
pub struct Data;
pub struct Clear;
impl RegisterSpec for ReadOnly {
    type Ux = u32;
}
impl RegisterSpec for ReadWrite {
    type Ux = u32;
}
impl RegisterSpec for Data {
    type Ux = u32;
}
impl RegisterSpec for Clear {
    type Ux = u32;
}
impl Readable for ReadOnly {}
impl Readable for ReadWrite {}
impl UnsafeWritable for ReadWrite {}
impl FifoPop for Data {}
impl FifoPush for Data {}
impl WriteOneToClear for Clear {}
#[repr(C)]
pub struct RegisterBlock {
    pub dr: Reg<Data>,
    pub rsr_ecr: Reg<ReadWrite>,
    _reserved0: [u32; 4],
    pub fr: Reg<ReadOnly>,
    _reserved1: u32,
    pub ilpr: Reg<ReadWrite>,
    pub ibrd: Reg<ReadWrite>,
    pub fbrd: Reg<ReadWrite>,
    pub lcr_h: Reg<ReadWrite>,
    pub cr: Reg<ReadWrite>,
    pub ifls: Reg<ReadWrite>,
    pub imsc: Reg<ReadWrite>,
    pub ris: Reg<ReadOnly>,
    pub mis: Reg<ReadOnly>,
    pub icr: Reg<Clear>,
    pub dmacr: Reg<ReadWrite>,
}
macro_rules! instance {
    ($($t:ty),*)=>{$(impl $t {
        pub fn registers(&self)->&RegisterBlock {unsafe {&*Self::PTR.cast::<RegisterBlock>()}}
    })*};
}
instance!(
    crate::UART0,
    crate::UART1,
    crate::UART2,
    crate::UART3,
    crate::UART4,
    crate::UART5
);
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pl011_offsets() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, fr), 0x18);
        assert_eq!(core::mem::offset_of!(RegisterBlock, ibrd), 0x24);
        assert_eq!(core::mem::offset_of!(RegisterBlock, cr), 0x30);
        assert_eq!(core::mem::offset_of!(RegisterBlock, icr), 0x44);
    }
}
