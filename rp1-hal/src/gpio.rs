use crate::{linux_blocks::Mmio, scmi::Registers};
use crate::{read, write};
use core::{convert::Infallible, marker::PhantomData};
use embedded_hal::digital::{ErrorType, InputPin, OutputPin, StatefulOutputPin};

/// Ownership of bank-0 pin N's interrupt events on core CORE.
pub struct PinIrq<const N: u8, const CORE: u8> {
    _private: (),
}
impl<const N: u8, const CORE: u8> PinIrq<N, CORE> {
    /// # Safety
    /// Call once on core CORE (0 or 1), which must own pin N (0..27) in `gpio_irq`.
    pub const unsafe fn new() -> Self {
        Self { _private: () }
    }
}

pub struct Unconfigured;
pub struct Input;
pub struct Output;
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
        configure_input(&mut Mmio, N);
        Pin { _mode: PhantomData }
    }
}
fn configure_input(io: &mut impl Registers, n: u8) {
    io.write(0x4001_7004, 1 << 19);
    // Enable the input buffer and force the pad output off before switching the mux.
    io.write(0x400f_0004 + n as usize * 4, 0xc6);
    io.write(0x400e_3004, 1 << n);
    io.write(0x400d_0004 + n as usize * 8, 5);
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

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use std::{collections::BTreeMap, vec::Vec};

    #[test]
    fn input_enables_buffer_and_disables_pad_output_before_mux() {
        struct Fake {
            regs: BTreeMap<usize, u32>,
            writes: Vec<(usize, u32)>,
        }
        impl Registers for Fake {
            fn read(&mut self, address: usize) -> u32 {
                self.regs.get(&address).copied().unwrap_or(0)
            }
            fn write(&mut self, address: usize, value: u32) {
                self.regs.insert(address, value);
                self.writes.push((address, value));
            }
        }
        for n in [0, 22, 23, 27] {
            let pad = 0x400f_0004 + n as usize * 4;
            let mux = 0x400d_0004 + n as usize * 8;
            // Linux leaves a reserved input with IE=0, OD=1.
            let mut io = Fake {
                regs: [(pad, 0x96)].into(),
                writes: Vec::new(),
            };
            configure_input(&mut io, n);
            assert_eq!(io.read(pad) & 0xc0, 0xc0);
            assert_eq!(io.read(mux), 5);
            assert_eq!(
                io.writes,
                [
                    (0x4001_7004, 1 << 19),
                    (pad, 0xc6),
                    (0x400e_3004, 1 << n),
                    (mux, 5),
                ]
            );
        }
    }
}
