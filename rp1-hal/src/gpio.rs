use crate::{linux_blocks::Mmio, scmi::Registers};
use crate::{read, write};
use core::{convert::Infallible, marker::PhantomData};
use embedded_hal::digital::{ErrorType, InputPin, OutputPin, StatefulOutputPin};

/// An interrupt event of a bank-0 pin, numbered by its IRQMASK bit in GPIOn_CTRL. Edges latch
/// until [`PinIrq::clear`]; levels follow the pin.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    EdgeLow = 20,
    EdgeHigh = 21,
    LevelLow = 22,
    LevelHigh = 23,
}
const IRQRESET: u32 = 1 << 28;
const SET: usize = 0x2000;
const CLR: usize = 0x3000;
pub(crate) const fn ctrl(n: u8) -> usize {
    0x400d_0004 + n as usize * 8
}
const fn inte(core: u8) -> usize {
    0x400d_0104 + core as usize * 0xc
}
const fn intf(core: u8) -> usize {
    inte(core) + 4
}
const fn ints(core: u8) -> usize {
    inte(core) + 8
}

/// Ownership of bank-0 pin N's interrupt events on core CORE. The NVIC line is `irq.IO_BANK0`
/// in the generated resources, shared by all of this core's pins.
///
/// Writes go through the SET/CLR aliases: the enable and force registers are shared with this
/// core's other pins, and GPIOn_CTRL also holds the pin's function select. Configure the pin
/// (for example [`Pin::into_input`]) before enabling events: that rewrites GPIOn_CTRL and clears
/// the event masks.
pub struct PinIrq<const N: u8, const CORE: u8> {
    _private: core::marker::PhantomData<*mut ()>,
}
impl<const N: u8, const CORE: u8> PinIrq<N, CORE> {
    /// # Safety
    /// Call once on core CORE (0 or 1), which must own pin N (0..27) in `gpio_irq`.
    pub const unsafe fn new() -> Self {
        Self {
            _private: core::marker::PhantomData,
        }
    }
    /// Lets `event` interrupt this core. Call [`PinIrq::clear`] first to discard an edge latched
    /// before enabling.
    pub fn enable(&mut self, event: Event) {
        write(SET + ctrl(N), 1 << event as u32);
        write(SET + inte(CORE), 1 << N);
    }
    /// Stops `event` from interrupting; other enabled events of the pin still do.
    pub fn disable(&mut self, event: Event) {
        write(CLR + ctrl(N), 1 << event as u32);
    }
    /// Whether this pin interrupts this core now (an enabled event, or [`PinIrq::force`]).
    pub fn is_pending(&self) -> bool {
        read(ints(CORE)) & (1 << N) != 0
    }
    /// Clears latched edges. A level event stays pending while the pin is at that level.
    pub fn clear(&mut self) {
        write(SET + ctrl(N), IRQRESET);
    }
    /// Raises this pin's interrupt on this core whatever its events, until [`PinIrq::unforce`].
    pub fn force(&mut self) {
        write(SET + intf(CORE), 1 << N);
    }
    pub fn unforce(&mut self) {
        write(CLR + intf(CORE), 1 << N);
    }
    pub fn is_forced(&self) -> bool {
        read(intf(CORE)) & (1 << N) != 0
    }
}

/// Pull resistors for a bank-0 pad, encoded as PADS_BANK0 bits PUE (3) and PDE (2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pull {
    None = 0,
    Up = 8,
    Down = 4,
    /// Enables both PUE and PDE to hold the last driven level.
    BusKeep = 12,
}

/// Output drive strength.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DriveStrength {
    Ma2,
    Ma4,
    Ma8,
    Ma12,
}

/// Output slew rate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SlewRate {
    Slow,
    Fast,
}

/// Electrical settings of a bank-0 pad.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PadConfig {
    /// Pull resistor or bus keeper setting.
    pub pull: Pull,
    /// Output drive strength.
    pub drive: DriveStrength,
    /// Output slew rate.
    pub slew: SlewRate,
    /// Enables Schmitt-trigger input hysteresis.
    pub schmitt: bool,
}

impl PadConfig {
    /// Electrical settings used by [`Pin::into_input`].
    pub const INPUT_DEFAULT: Self = Self {
        pull: Pull::Down,
        drive: DriveStrength::Ma2,
        slew: SlewRate::Slow,
        schmitt: true,
    };
    /// Electrical settings used by [`Pin::into_output`].
    pub const OUTPUT_DEFAULT: Self = Self {
        drive: DriveStrength::Ma4,
        ..Self::INPUT_DEFAULT
    };

    fn bits(self) -> u32 {
        self.pull as u32 | (self.drive as u32) << 4 | (self.schmitt as u32) << 1 | self.slew as u32
    }

    fn from_bits(bits: u32) -> Self {
        Self {
            pull: [Pull::None, Pull::Down, Pull::Up, Pull::BusKeep][(bits as usize >> 2) & 3],
            drive: [
                DriveStrength::Ma2,
                DriveStrength::Ma4,
                DriveStrength::Ma8,
                DriveStrength::Ma12,
            ][(bits as usize >> 4) & 3],
            slew: if bits & 1 == 0 {
                SlewRate::Slow
            } else {
                SlewRate::Fast
            },
            schmitt: bits & 2 != 0,
        }
    }
}

pub(crate) const fn pad(n: u8) -> usize {
    0x400f_0004 + n as usize * 4
}

pub struct Unconfigured;
pub struct Input;
pub struct Output;
/// Muxed to the peripheral signal named by a driver's pin trait (for example [`crate::uart::TxPin`]),
/// with the function select from the bank-0 pin table.
pub struct Alternate;
pub struct Pin<const N: u8, Mode = Unconfigured> {
    _mode: PhantomData<(Mode, *mut ())>,
}
impl<const N: u8, M> Pin<N, M> {
    /// Reads the pad's current electrical settings.
    pub fn pad_config(&self) -> PadConfig {
        PadConfig::from_bits(read(pad(N)))
    }

    /// Changes the electrical settings while preserving the pin's direction, mux and IRQs.
    /// Configure these settings after routing the pin, which applies the new mode's defaults.
    pub fn set_pad_config(&mut self, config: PadConfig) {
        set_pad_config(&mut Mmio, N, config);
    }

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
    /// then writes `pad_bits` (PADS_BANK0: OD 7, IE 6, DRIVE 5:4, PUE 3, PDE 2, SCHMITT 1,
    /// SLEWFAST 0).
    pub(crate) fn into_alternate(self, fsel: u8, idle: u32, pad_bits: u32) -> Pin<N, Alternate> {
        write(0x4001_7004, 1 << 19);
        write(pad(N), 0x80 | idle);
        write(0x400e_3004, 1 << N);
        write(ctrl(N), u32::from(fsel));
        write(pad(N), pad_bits);
        Pin { _mode: PhantomData }
    }
    /// Routes the pin as a GPIO output with the default pad settings.
    pub fn into_output(self, high: bool) -> Pin<N, Output> {
        self.into_output_with_pad(high, PadConfig::OUTPUT_DEFAULT)
    }

    /// Routes the pin as a GPIO output with the specified initial level and pad settings.
    pub fn into_output_with_pad(self, high: bool, config: PadConfig) -> Pin<N, Output> {
        configure_output(&mut Mmio, N, high, config);
        Pin { _mode: PhantomData }
    }

    /// Routes the pin as a GPIO input with the default pad settings.
    pub fn into_input(self) -> Pin<N, Input> {
        self.into_input_with_pad(PadConfig::INPUT_DEFAULT)
    }

    /// Routes the pin as a GPIO input with the specified pad settings.
    pub fn into_input_with_pad(self, config: PadConfig) -> Pin<N, Input> {
        configure_input(&mut Mmio, N, config);
        Pin { _mode: PhantomData }
    }
}

fn configure_output(io: &mut impl Registers, n: u8, high: bool, config: PadConfig) {
    io.write(0x4001_7004, 1 << 19);
    // Keep the pad disconnected until RIO is driving the initial level.
    io.write(pad(n), 0xc0 | config.bits());
    io.write(if high { 0x400e_2000 } else { 0x400e_3000 }, 1 << n);
    io.write(ctrl(n), 0x85);
    io.write(0x400e_2004, 1 << n);
    io.write(pad(n), 0x40 | config.bits());
}

fn configure_input(io: &mut impl Registers, n: u8, config: PadConfig) {
    io.write(0x4001_7004, 1 << 19);
    // Enable the input buffer and force the pad output off before switching the mux.
    io.write(pad(n), 0xc0 | config.bits());
    io.write(0x400e_3004, 1 << n);
    io.write(ctrl(n), 5);
}

fn set_pad_config(io: &mut impl Registers, n: u8, config: PadConfig) {
    let current = io.read(pad(n));
    io.write(pad(n), (current & !0x3f) | config.bits());
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

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use std::{collections::BTreeMap, vec::Vec};

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

    #[test]
    fn pad_settings_round_trip() {
        for raw in 0..64 {
            assert_eq!(PadConfig::from_bits(raw).bits(), raw);
        }
    }

    #[test]
    fn output_drives_initial_level_before_enabling_pad() {
        let mut io = Fake {
            regs: BTreeMap::new(),
            writes: Vec::new(),
        };
        configure_output(&mut io, 22, true, PadConfig::OUTPUT_DEFAULT);
        assert_eq!(
            io.writes,
            [
                (0x4001_7004, 1 << 19),
                (pad(22), 0xd6),
                (0x400e_2000, 1 << 22),
                (ctrl(22), 0x85),
                (0x400e_2004, 1 << 22),
                (pad(22), 0x56),
            ]
        );
    }

    #[test]
    fn pad_update_preserves_direction_and_irq_config() {
        let mut io = Fake {
            regs: [(pad(22), 0x1c8), (ctrl(22), 0x0020_0005)].into(),
            writes: Vec::new(),
        };
        assert_eq!(PadConfig::from_bits(io.read(pad(22))).pull, Pull::Up);
        set_pad_config(&mut io, 22, PadConfig::INPUT_DEFAULT);
        assert_eq!(io.writes, [(pad(22), 0x1c6)]);
    }

    #[test]
    fn input_enables_buffer_and_disables_pad_output_before_mux() {
        for n in [0, 22, 23, 27] {
            let pad = 0x400f_0004 + n as usize * 4;
            let mux = 0x400d_0004 + n as usize * 8;
            // Linux leaves a reserved input with IE=0, OD=1.
            let mut io = Fake {
                regs: [(pad, 0x96)].into(),
                writes: Vec::new(),
            };
            configure_input(&mut io, n, PadConfig::INPUT_DEFAULT);
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

    #[test]
    fn pin_interrupt_registers() {
        assert_eq!(SET + ctrl(27), 0x400d_20dc);
        assert_eq!(
            (inte(0), intf(0), ints(0)),
            (0x400d_0104, 0x400d_0108, 0x400d_010c)
        );
        assert_eq!(
            (inte(1), intf(1), ints(1)),
            (0x400d_0110, 0x400d_0114, 0x400d_0118)
        );
    }
}
