use super::{Buffers, Rx, Tx};
use super::{IRQ, InstalledProgram, PioPin, UninitStateMachine, control, sm_reg};
use crate::{
    Error, FreeError,
    fugit::HertzU32,
    gpio::{self, PadConfig},
    read, write,
};
use core::marker::PhantomData;

const CLKDIV: usize = 0;
const EXECCTRL: usize = 4;
const SHIFTCTRL: usize = 8;
const ADDR: usize = 12;
const INSTR: usize = 16;
const PINCTRL: usize = 20;

/// Owned GPIOs and their electrical settings. Output GPIOs may also be sampled.
#[derive(Default)]
pub struct Pins {
    owned: u32,
    outputs: u32,
    initial_high: u32,
    initial_dirs: u32,
    pad_bits: [u8; 28],
    local: PhantomData<*mut ()>,
}

/// Initial latch and direction for a pin the program may drive.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InitialOutput {
    Low,
    High,
    /// Released, with a low latch for the next output-enable.
    HiZLow,
    /// Released, with a high latch for the next output-enable.
    HiZHigh,
}

impl Pins {
    pub fn new() -> Self {
        Self::default()
    }

    /// Claims a pin whose pad output stays disabled.
    pub fn input<const N: u8>(mut self, _pin: PioPin<N>, pad: PadConfig) -> Self {
        self.owned |= 1 << N;
        self.pad_bits[usize::from(N)] = pad.bits() as u8;
        self
    }

    /// Claims a pin that the program may drive.
    pub fn output<const N: u8>(
        mut self,
        _pin: PioPin<N>,
        pad: PadConfig,
        initial: InitialOutput,
    ) -> Self {
        let bit = 1 << N;
        self.owned |= bit;
        self.outputs |= bit;
        if matches!(initial, InitialOutput::High | InitialOutput::HiZHigh) {
            self.initial_high |= bit;
        }
        if matches!(initial, InitialOutput::Low | InitialOutput::High) {
            self.initial_dirs |= bit;
        }
        self.pad_bits[usize::from(N)] = pad.bits() as u8;
        self
    }

    /// Get a typed owner back from a pin set returned by the SM or a failed build.
    pub fn take<const N: u8>(&mut self) -> Option<PioPin<N>> {
        if N >= 28 || self.owned & (1 << N) == 0 {
            return None;
        }
        self.owned &= !(1 << N);
        self.outputs &= !(1 << N);
        self.initial_dirs &= !(1 << N);
        self.initial_high &= !(1 << N);
        // The bit was set by consuming this exact token and is now clear.
        Some(unsafe { PioPin::__new() })
    }
}

fn gpio_numbers(mut mask: u32) -> impl Iterator<Item = u8> {
    core::iter::from_fn(move || {
        if mask == 0 {
            return None;
        }
        let n = mask.trailing_zeros() as u8;
        mask &= mask - 1;
        Some(n)
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShiftDirection {
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BuildError {
    Clock,
    Divider,
    Threshold,
    PinRange,
    UnownedPin,
    IrqFlag,
}

#[derive(Clone, Copy, Default)]
struct PinRange {
    base: u8,
    count: u8,
}

impl PinRange {
    fn mask(self, max: u8) -> Result<u32, BuildError> {
        if self.base >= 28 || self.count > max || u16::from(self.base) + u16::from(self.count) > 28
        {
            return Err(BuildError::PinRange);
        }
        if self.count == 0 {
            Ok(0)
        } else {
            Ok((u32::MAX >> (32 - self.count)) << self.base)
        }
    }
}

#[derive(Clone, Copy)]
enum Divider {
    Fixed(u16, u8),
    Frequency(HertzU32),
}

impl Divider {
    fn bits(self, sys: u32) -> Result<u32, BuildError> {
        let fixed = match self {
            Self::Fixed(0, 0) => return Ok(0), // 65536 has a special encoding.
            Self::Fixed(0, _) => return Err(BuildError::Divider),
            Self::Fixed(integer, frac) => u64::from(integer) * 256 + u64::from(frac),
            Self::Frequency(hz) => {
                let rate = u64::from(hz.raw());
                if rate == 0 || rate > u64::from(sys) {
                    return Err(BuildError::Divider);
                }
                (u64::from(sys) * 256 + rate / 2) / rate
            }
        };
        if fixed > 65536 * 256 {
            return Err(BuildError::Divider);
        }
        Ok((fixed as u32) << 8)
    }
}

/// Configures a stopped state machine from an installed program.
pub struct PIOBuilder<'program, 'pio> {
    program: &'program InstalledProgram<'pio>,
    pins: Pins,
    in_pins: PinRange,
    out_pins: PinRange,
    set_pins: PinRange,
    side_set_pin_base: u8,
    jmp_pin: Option<u8>,
    divider: Divider,
    push_threshold: u8,
    pull_threshold: u8,
    in_shift_direction: ShiftDirection,
    out_shift_direction: ShiftDirection,
    autopush: bool,
    autopull: bool,
    buffers: Buffers,
}

macro_rules! builder_options {
    ($($(#[$doc:meta])* $field:ident: $type:ty),+ $(,)?) => {
        $(
            $(#[$doc])*
            pub fn $field(mut self, $field: $type) -> Self {
                self.$field = $field;
                self
            }
        )+
    };
}

impl<'program, 'pio> PIOBuilder<'program, 'pio> {
    pub fn from_installed_program(program: &'program InstalledProgram<'pio>) -> Self {
        Self {
            program,
            pins: Pins::default(),
            in_pins: PinRange::default(),
            out_pins: PinRange::default(),
            set_pins: PinRange::default(),
            side_set_pin_base: 0,
            jmp_pin: None,
            divider: Divider::Fixed(1, 0),
            push_threshold: 32,
            pull_threshold: 32,
            in_shift_direction: ShiftDirection::Right,
            out_shift_direction: ShiftDirection::Right,
            autopush: false,
            autopull: false,
            buffers: Buffers::RxTx,
        }
    }

    builder_options! {
        /// Set the owned pins and their pad configuration.
        pins: Pins,
        /// Set the first side-set pin.
        side_set_pin_base: u8,
        /// Set the ISR threshold in bits (1–32).
        push_threshold: u8,
        /// Set the OSR threshold in bits (1–32).
        pull_threshold: u8,
        /// Select the ISR shift direction.
        in_shift_direction: ShiftDirection,
        /// Select the OSR shift direction.
        out_shift_direction: ShiftDirection,
        /// Enable automatic RX pushes at the ISR threshold.
        autopush: bool,
        /// Enable automatic TX pulls at the OSR threshold.
        autopull: bool,
        /// Set the split or joined FIFO layout.
        buffers: Buffers,
    }

    /// Set the first relative input pin.
    pub fn in_pin_base(mut self, base: u8) -> Self {
        self.in_pins.base = base;
        self
    }

    /// Bound the relative inputs the program may read.
    pub fn in_count(mut self, count: u8) -> Self {
        self.in_pins.count = count;
        self
    }

    /// Select the contiguous pins for OUT and MOV to PINS.
    pub fn out_pins(mut self, base: u8, count: u8) -> Self {
        self.out_pins = PinRange { base, count };
        self
    }

    /// Select the contiguous pins for SET.
    pub fn set_pins(mut self, base: u8, count: u8) -> Self {
        self.set_pins = PinRange { base, count };
        self
    }

    /// Select the pin tested by JMP PIN.
    pub fn jmp_pin(mut self, pin: u8) -> Self {
        self.jmp_pin = Some(pin);
        self
    }

    /// Set the 16.8 fixed-point divider. `(0, 0)` encodes 65536.
    pub fn clock_divisor_fixed_point(mut self, integer: u16, fractional: u8) -> Self {
        self.divider = Divider::Fixed(integer, fractional);
        self
    }

    /// Round to the closest representable divider using the verified clk_sys.
    pub fn clock_frequency(mut self, rate: HertzU32) -> Self {
        self.divider = Divider::Frequency(rate);
        self
    }

    fn pinctrl(&self, n: u8) -> Result<u32, BuildError> {
        let input = self.in_pins.mask(28)?;
        let out = self.out_pins.mask(28)?;
        let set = self.set_pins.mask(5)?;
        let side = self.program.side_set;
        let side_count = side.bits() - u8::from(side.optional());
        let sides = PinRange {
            base: self.side_set_pin_base,
            count: side_count,
        }
        .mask(5)?;
        let jmp = self
            .jmp_pin
            .map_or(Ok(0), |base| PinRange { base, count: 1 }.mask(1))?;
        let req = &self.program.requirements;
        let relative_limit = input >> self.in_pins.base;
        if req.relative_in & !relative_limit != 0
            || (req.jmp_pin && jmp == 0)
            || (req.set_pins && set == 0)
            || (req.out_pins && out == 0)
        {
            return Err(BuildError::PinRange);
        }
        if ((input | jmp | req.absolute_in) & !self.pins.owned != 0)
            || ((out | set | sides) & !self.pins.outputs != 0)
        {
            return Err(BuildError::UnownedPin);
        }
        if req.flags & !((1 << n) | (1 << (n + 4))) != 0 || req.relative_flags & !0x11 != 0 {
            return Err(BuildError::IrqFlag);
        }
        Ok(u32::from(side.bits()) << 29
            | u32::from(self.set_pins.count) << 26
            | u32::from(self.out_pins.count) << 20
            | u32::from(self.in_pins.base) << 15
            | u32::from(self.side_set_pin_base) << 10
            | u32::from(self.set_pins.base) << 5
            | u32::from(self.out_pins.base))
    }

    fn shiftctrl(&self) -> Result<u32, BuildError> {
        if !(1..=32).contains(&self.push_threshold) || !(1..=32).contains(&self.pull_threshold) {
            return Err(BuildError::Threshold);
        }
        Ok(self.buffers.bits()
            | u32::from(self.pull_threshold % 32) << 25
            | u32::from(self.push_threshold % 32) << 20
            | u32::from(self.out_shift_direction == ShiftDirection::Right) << 19
            | u32::from(self.in_shift_direction == ShiftDirection::Right) << 18
            | u32::from(self.autopull) << 17
            | u32::from(self.autopush) << 16)
    }

    pub fn build<const N: u8>(
        self,
        sm: UninitStateMachine<N>,
    ) -> Result<Configured<'program, 'pio, N>, BuildFailure<N>> {
        let buffers = self.buffers;
        match self.configure(N) {
            Ok(machine) => Ok((
                StateMachine {
                    machine,
                    state: PhantomData,
                },
                Rx::new(buffers),
                Tx::new(buffers),
            )),
            Err(error) => {
                let (pins, error) = error.into_parts();
                Err(FreeError::new((sm, pins), error))
            }
        }
    }

    fn config_bits(&self, n: u8) -> Result<(u32, u32, u32), BuildError> {
        if !self.program.pio.clocks.sys_current_strict() {
            return Err(BuildError::Clock);
        }
        Ok((
            self.divider.bits(self.program.pio.clocks.sys().raw())?,
            self.pinctrl(n)?,
            self.shiftctrl()?,
        ))
    }

    #[inline(never)]
    fn configure(self, n: u8) -> Result<Machine<'program, 'pio>, FreeError<Pins, BuildError>> {
        let (divider, pinctrl, shiftctrl) = match self.config_bits(n) {
            Ok(config) => config,
            Err(error) => return Err(FreeError::new(self.pins, error)),
        };

        for pin in gpio_numbers(self.pins.owned) {
            gpio::pio_prepare(pin, self.pins.pad_bits[usize::from(pin)]);
        }
        let program = self.program;
        let side = program.side_set;
        let execctrl = u32::from(side.optional()) << 30
            | u32::from(side.pindirs()) << 29
            | u32::from(self.jmp_pin.unwrap_or_default()) << 24
            | u32::from(program.wrap_top) << 12
            | u32::from(program.wrap_bottom) << 7;

        program.pio.active[usize::from(n)].set(program.mask);
        write(sm_reg(n, CLKDIV), divider);
        clear_fifos(n, shiftctrl, write);
        let mut machine = Machine {
            n,
            program,
            pins: self.pins,
            running: false,
        };
        machine.reset(pinctrl, execctrl);
        Ok(machine)
    }
}

pub type BuildFailure<const N: u8> = FreeError<(UninitStateMachine<N>, Pins), BuildError>;

pub type Configured<'program, 'pio, const N: u8> =
    (StateMachine<'program, 'pio, N, Stopped>, Rx<N>, Tx<N>);

fn clear_fifos(n: u8, shiftctrl: u32, mut write: impl FnMut(usize, u32)) {
    // Changing either JOIN bit resets both FIFOs. Keep the intermediate mode valid.
    let other = if shiftctrl & (3 << 30) == 0 {
        Buffers::OnlyRx.bits()
    } else {
        0
    };
    write(sm_reg(n, SHIFTCTRL), (shiftctrl & !(3 << 30)) | other);
    write(sm_reg(n, SHIFTCTRL), shiftctrl);
}

pub struct Stopped;
pub struct Running;

pub struct StateMachine<'program, 'pio, const N: u8, State> {
    machine: Machine<'program, 'pio>,
    state: PhantomData<State>,
}

struct Machine<'program, 'pio> {
    n: u8,
    program: &'program InstalledProgram<'pio>,
    pins: Pins,
    running: bool,
}

impl Machine<'_, '_> {
    #[inline(never)]
    fn output(&self, enable: bool) {
        for n in gpio_numbers(self.pins.outputs) {
            gpio::pio_output(n, enable);
        }
    }

    #[inline(never)]
    fn stop(&mut self) {
        self.output(false);
        control(0, 1 << self.n, 0, 0);
        self.running = false;
    }

    fn clear_flags(&self) {
        // This SM is the only owner of these internal flags.
        write(IRQ, (1 << self.n) | (1 << (self.n + 4)));
    }

    #[inline(never)]
    fn reset(&mut self, pinctrl: u32, execctrl: u32) {
        control(0, 1 << self.n, 1 << self.n, 1 << self.n);
        self.clear_flags();
        // Forced SET/JMP must be independent of the program's side-set.
        write(sm_reg(self.n, EXECCTRL), 0);
        write(sm_reg(self.n, PINCTRL), 0);
        write(sm_reg(self.n, INSTR), 0xe020); // SET X, 0
        write(sm_reg(self.n, INSTR), 0xe040); // SET Y, 0
        for n in gpio_numbers(self.pins.outputs) {
            write(sm_reg(self.n, PINCTRL), (1 << 26) | (u32::from(n) << 5));
            write(
                sm_reg(self.n, INSTR),
                0xe000 | ((self.pins.initial_high >> n) & 1), // SET PINS
            );
            write(
                sm_reg(self.n, INSTR),
                0xe080 | ((self.pins.initial_dirs >> n) & 1), // SET PINDIRS
            );
        }
        write(sm_reg(self.n, PINCTRL), 0);
        // Entry precedes wrap target when the program has an initialization preamble.
        write(sm_reg(self.n, INSTR), u32::from(self.program.offset));
        write(sm_reg(self.n, PINCTRL), pinctrl);
        write(sm_reg(self.n, EXECCTRL), execctrl);
    }
    #[inline(never)]
    fn release(&mut self) -> Pins {
        let n = self.n;
        self.output(false);
        for pin in gpio_numbers(self.pins.owned) {
            gpio::pio_release(pin);
        }
        self.clear_flags();
        clear_fifos(n, read(sm_reg(n, SHIFTCTRL)), write);
        write(sm_reg(n, SHIFTCTRL), 0);
        self.program.pio.active[usize::from(n)].set(0);
        core::mem::take(&mut self.pins)
    }
}

impl Drop for Machine<'_, '_> {
    fn drop(&mut self) {
        if self.running {
            self.stop();
        }
        // Without both FIFO handles, leave the program allocation quarantined.
    }
}

impl<'program, 'pio, const N: u8, State> StateMachine<'program, 'pio, N, State> {
    /// Current instruction address in the block's instruction memory.
    pub fn instruction_address(&self) -> u8 {
        read(sm_reg(N, ADDR)) as u8
    }

    /// The two internal flags assigned to this SM, packed into bits 0 and 1.
    pub fn flags(&self) -> u8 {
        let flags = read(IRQ) >> N;
        ((flags & 1) | ((flags >> 3) & 2)) as u8
    }

    fn into_state<Next>(self) -> StateMachine<'program, 'pio, N, Next> {
        StateMachine {
            machine: self.machine,
            state: PhantomData,
        }
    }
}

impl<'program, 'pio, const N: u8> StateMachine<'program, 'pio, N, Stopped> {
    /// Start or resume the SM, preserving its PC and FIFO contents.
    pub fn start(
        mut self,
    ) -> Result<StateMachine<'program, 'pio, N, Running>, FreeError<Self, Error>> {
        if !self.machine.program.pio.clocks.sys_current_strict() {
            return Err(FreeError::new(self, Error::Precondition));
        }
        self.machine.output(true);
        control(1 << N, 0, 0, 1 << N);
        self.machine.running = true;
        Ok(self.into_state())
    }

    /// Restart at the first instruction, restore initial pin levels and reset the divider phase.
    /// FIFO contents are retained.
    pub fn restart(&mut self) {
        let pinctrl = read(sm_reg(N, PINCTRL));
        let execctrl = read(sm_reg(N, EXECCTRL)) & 0x7fff_ffff;
        self.machine.reset(pinctrl, execctrl);
    }

    /// Reset both FIFOs while this SM is stopped.
    pub fn clear_fifos(&mut self, _rx: &mut Rx<N>, _tx: &mut Tx<N>) {
        clear_fifos(N, read(sm_reg(N, SHIFTCTRL)), write);
    }

    /// Release a stopped SM and its GPIOs after taking back both FIFO handles.
    pub fn uninit(mut self, _rx: Rx<N>, _tx: Tx<N>) -> (UninitStateMachine<N>, Pins) {
        let pins = self.machine.release();
        // The typed StateMachine consumed the original token for exactly N.
        (unsafe { UninitStateMachine::__new() }, pins)
    }
}

impl<'program, 'pio, const N: u8> StateMachine<'program, 'pio, N, Running> {
    /// Disconnect outputs and halt the SM. Call restart before starting at the entry again.
    pub fn stop(mut self) -> StateMachine<'program, 'pio, N, Stopped> {
        self.machine.stop();
        self.into_state()
    }
}

mod sealed {
    pub trait Group {}
}

/// Enable two, three or four stopped SMs and restart their dividers in one CTRL write.
pub trait SyncStart: sealed::Group + Sized {
    type Running;
    fn sync_start(self) -> Result<Self::Running, FreeError<Self, Error>>;
}

macro_rules! group {
    ($($sm:ident: $n:ident),+) => {
        impl<'p, 'pio, $(const $n: u8),+> sealed::Group for ($(StateMachine<'p, 'pio, $n, Stopped>,)+) {}
        impl<'p, 'pio, $(const $n: u8),+> SyncStart for ($(StateMachine<'p, 'pio, $n, Stopped>,)+) {
            type Running = ($(StateMachine<'p, 'pio, $n, Running>,)+);

            fn sync_start(mut self) -> Result<Self::Running, FreeError<Self, Error>> {
                let pio = self.0.machine.program.pio;
                let ($($sm,)+) = &mut self;
                if !pio.clocks.sys_current_strict() $(
                    || !core::ptr::eq(pio, $sm.machine.program.pio)
                )+ {
                    return Err(FreeError::new(self, Error::Precondition));
                }
                $($sm.machine.output(true);)+
                let mask = 0 $(| (1 << $n))+;
                control(mask, 0, 0, mask);
                $($sm.machine.running = true;)+
                let ($($sm,)+) = self;
                Ok(($($sm.into_state(),)+))
            }
        }
    }
}

group!(a: A, b: B);
group!(a: A, b: B, c: C);
group!(a: A, b: B, c: C, d: D);

#[cfg(test)]
#[path = "sm_tests.rs"]
mod tests;
