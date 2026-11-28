extern crate std;

use super::*;
use crate::pio::{PIO, tests::block};

fn install<'a>(pio: &'a PIO, program: &::pio::Program<32>) -> InstalledProgram<'a> {
    pio.install_with(program, |_, _| {}).unwrap()
}

#[test]
fn requires_owned_and_declared_pins_in_the_correct_direction() {
    let pio = block((1 << 12) | (1 << 22));
    let p = ::pio::pio_asm!("in pins, 1", "set pins, 1");
    let p = install(&pio, &p.program);
    let mut b = PIOBuilder::from_installed_program(&p)
        .in_pins(22, 1)
        .set_pins(12, 1);
    assert_eq!(b.pinctrl(0), Err(BuildError::UnownedPin));
    let pin = unsafe { Pin::<12>::new() };
    b.pins = Pins::new()
        .input(pin, PadConfig::INPUT_DEFAULT)
        .input(unsafe { Pin::<22>::new() }, PadConfig::INPUT_DEFAULT);
    assert_eq!(b.pinctrl(0), Err(BuildError::UnownedPin));
    let pin = b.pins.take::<12>().unwrap();
    b.pins = b.pins.output(pin, PadConfig::OUTPUT_DEFAULT, false, true);
    assert_eq!(b.pinctrl(0), Ok(1 << 26 | 22 << 15 | 12 << 5));
    b.in_pins = PinRange { base: 27, count: 2 };
    assert_eq!(b.pinctrl(0), Err(BuildError::PinRange));
    b.in_pins = PinRange { base: 22, count: 1 };
    b.pins = b
        .pins
        .input(unsafe { Pin::<23>::new() }, PadConfig::INPUT_DEFAULT);
    assert_eq!(b.pinctrl(0), Err(BuildError::UnownedPin));
}

#[test]
fn restricts_internal_flags_to_each_sm() {
    let pio = block(0);
    let p = ::pio::pio_asm!("irq 1", "irq set 4 rel", "wait 0 irq 0 rel");
    let p = install(&pio, &p.program);
    let b = PIOBuilder::from_installed_program(&p);
    assert_eq!(b.pinctrl(0), Err(BuildError::IrqFlag));
    assert_eq!(b.pinctrl(1), Ok(0));
}

#[test]
fn divider_rounds_and_checks_fixed_point_limits() {
    let sys = 200_000_000;
    assert_eq!(Divider::Fixed(0, 0).bits(sys), Ok(0));
    assert_eq!(Divider::Fixed(0, 1).bits(sys), Err(BuildError::Divider));
    assert_eq!(Divider::Fixed(1, 128).bits(sys), Ok(0x18000));
    assert_eq!(
        Divider::Frequency(HertzU32::Hz(3_000_000)).bits(sys),
        Ok(0x0042_ab00)
    );
    for invalid in [0, 3_000, 200_000_001] {
        assert_eq!(
            Divider::Frequency(HertzU32::Hz(invalid)).bits(sys),
            Err(BuildError::Divider)
        );
    }
}

#[test]
fn fifo_reset_toggles_join_before_restoring_the_requested_configuration() {
    for mode in [Buffers::RxTx, Buffers::OnlyTx, Buffers::OnlyRx] {
        let config = mode.bits() | 0x00c8_0000;
        let mut writes = std::vec::Vec::new();
        clear_fifos(3, config, |address, value| writes.push((address, value)));
        let expected = if mode == Buffers::RxTx { 1 << 31 } else { 0 };
        assert_eq!(
            writes,
            [
                (sm_reg(3, SHIFTCTRL), expected | 0x00c8_0000),
                (sm_reg(3, SHIFTCTRL), config)
            ]
        );
    }
}

#[test]
fn forgetting_a_machine_keeps_its_program_reserved() {
    let pio = block(0);
    let p = ::pio::pio_asm!("nop").program;
    let program = install(&pio, &p);
    pio.active[0].set(program.mask);
    let machine = Machine {
        n: 0,
        program: &program,
        pins: Pins::new(),
        running: true,
    };
    core::mem::forget(machine);
    let error = program.uninstall().unwrap_err();
    assert_eq!(*error.error(), crate::pio::InstallError::InUse);
    assert_eq!(pio.used.get(), 1 << 31);
}
