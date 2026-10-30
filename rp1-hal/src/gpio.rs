use crate::{read, write};
use core::{convert::Infallible, marker::PhantomData};
use embedded_hal::digital::{ErrorType, InputPin, OutputPin, StatefulOutputPin};

pub struct Unconfigured;
pub struct Input;
pub struct Output;
pub struct Function<const F: u8>;
/// Muxed to an SPI controller (function 0 or 8, see [`crate::spi`]).
pub struct SpiFunction;
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
    pub(crate) fn into_i2c(self) -> Pin<N, Function<3>> {
        write(0x4001_7004, 1 << 19);
        // Disable the pad while switching; only the I2C peripheral may sink the line.
        write(0x400f_0004 + N as usize * 4, 0xfa);
        write(0x400e_3004, 1 << N);
        write(0x400d_0004 + N as usize * 8, 3);
        // Schmitt, input enable, 12 mA sink, weak pull-up; external pull-ups are required.
        write(0x400f_0004 + N as usize * 4, 0x7a);
        Pin { _mode: PhantomData }
    }
    /// Selects SPI function `fsel` (0 or 8 on bank 0). The pad output is disabled while the mux
    /// switches; then outputs get 8 mA fast slew, the input (MISO) a pull-down.
    pub(crate) fn into_spi(self, fsel: u8, output: bool) -> Pin<N, SpiFunction> {
        write(0x4001_7004, 1 << 19);
        write(0x400f_0004 + N as usize * 4, 0xc6);
        write(0x400e_3004, 1 << N);
        write(0x400d_0004 + N as usize * 8, u32::from(fsel));
        // IE (6), DRIVE 8 mA (5:4), Schmitt (1), SLEWFAST (0) | IE, PDE (2), Schmitt.
        write(
            0x400f_0004 + N as usize * 4,
            if output { 0x63 } else { 0x46 },
        );
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
