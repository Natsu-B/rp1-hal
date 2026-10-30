use crate::{read, write};
use core::{convert::Infallible, marker::PhantomData};
use embedded_hal::digital::{ErrorType, InputPin, OutputPin, StatefulOutputPin};

pub struct Unconfigured;
pub struct Input;
pub struct Output;
pub struct Function<const F: u8>;
/// Muxed to the peripheral signal named by a driver's pin trait (for example [`crate::uart::Tx`]),
/// with the function select from the bank-0 pin table.
pub struct Alternate;
pub struct Pin<const N: u8, Mode = Unconfigured> {
    _mode: PhantomData<Mode>,
}
impl<const N: u8, M> Pin<N, M> {
    /// Back to the unconfigured state type (the pad and mux stay as they are).
    pub(crate) fn release(self) -> Pin<N> {
        Pin { _mode: PhantomData }
    }
    /// Host tests: the state type without touching the registers.
    #[cfg(test)]
    pub(crate) fn assume<M2>(self) -> Pin<N, M2> {
        Pin { _mode: PhantomData }
    }
}
impl<const N: u8> Pin<N> {
    /// # Safety
    /// The declaration grants this core sole ownership of bank-0 pin N (0..27).
    pub const unsafe fn new() -> Self {
        assert!(N < 28);
        Self { _mode: PhantomData }
    }
    /// Selects function `fsel` with the pad output disabled (`OD | idle`) while the mux switches,
    /// then sets the pad to `pad` (PADS_BANK0: OD 7, IE 6, DRIVE 5:4, PUE 3, PDE 2, SCHMITT 1,
    /// SLEWFAST 0).
    pub(crate) fn into_alternate(self, fsel: u8, idle: u32, pad: u32) -> Pin<N, Alternate> {
        write(0x4001_7004, 1 << 19);
        write(0x400f_0004 + N as usize * 4, 0x80 | idle);
        write(0x400e_3004, 1 << N);
        write(0x400d_0004 + N as usize * 8, u32::from(fsel));
        write(0x400f_0004 + N as usize * 4, pad);
        Pin { _mode: PhantomData }
    }
    pub fn into_output(self, high: bool) -> Pin<N, Output> {
        write(0x4001_7004, 1 << 19);
        write(if high { 0x400e_2000 } else { 0x400e_3000 }, 1 << N);
        write(0x400d_0004 + N as usize * 8, 0x85);
        write(0x400f_0004 + N as usize * 4, 0x56);
        write(0x400e_2004, 1 << N);
        Pin { _mode: PhantomData }
    }
    pub fn into_input(self) -> Pin<N, Input> {
        write(0x4001_7004, 1 << 19);
        write(0x400e_3004, 1 << N);
        write(0x400d_0004 + N as usize * 8, 5);
        write(0x400f_0004 + N as usize * 4, 0x46);
        Pin { _mode: PhantomData }
    }
    pub fn into_function<const F: u8>(self) -> Pin<N, Function<F>> {
        assert!(F <= 31);
        write(0x4001_7004, 1 << 19);
        write(0x400d_0004 + N as usize * 8, u32::from(F));
        write(0x400f_0004 + N as usize * 4, 0x46);
        Pin { _mode: PhantomData }
    }
}
impl<const N: u8, M> ErrorType for Pin<N, M> {
    type Error = Infallible;
}
impl<const N: u8> OutputPin for Pin<N, Output> {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        write(0x400e_3000, 1 << N);
        Ok(())
    }
    fn set_high(&mut self) -> Result<(), Self::Error> {
        write(0x400e_2000, 1 << N);
        Ok(())
    }
}
impl<const N: u8> StatefulOutputPin for Pin<N, Output> {
    fn is_set_high(&mut self) -> Result<bool, Self::Error> {
        Ok(read(0x400e_0000) & (1 << N) != 0)
    }
    fn is_set_low(&mut self) -> Result<bool, Self::Error> {
        self.is_set_high().map(|v| !v)
    }
}
impl<const N: u8> InputPin for Pin<N, Input> {
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        Ok(read(0x400e_0008) & (1 << N) != 0)
    }
    fn is_low(&mut self) -> Result<bool, Self::Error> {
        self.is_high().map(|v| !v)
    }
}

/// GPIO27 force-event doorbell for proc1 (Linux -> RP1 only). There is no RP1 -> Linux
/// interrupt in v0.1: HOST_EVENTS / hwirq58 belongs to rp1-mailbox (SCMI), so Linux polls.
/// The GPIO itself must be reserved by the declaration.
pub struct Doorbell {
    _pin: Pin<27>,
}
impl Doorbell {
    pub fn new(pin: Pin<27>) -> Self {
        Self { _pin: pin }
    }
    #[inline(always)]
    pub fn pending(&self) -> bool {
        read(0x400d_0114) & (1 << 27) != 0
    }
    #[inline(always)]
    pub fn acknowledge(&mut self) {
        write(0x400d_3114, 1 << 27);
    }
}
