extern crate std;
use super::*;
use crate::cold::RESETS_DONE0;
use ::pio::pio_asm;
use core::cell::RefCell;

#[test]
fn reset_does_not_release_until_assertion_is_seen() {
    assert_eq!(
        (RESET_SET, RESET_CLEAR, RESET_DONE),
        (0x4001_6000, 0x4001_7000, RESETS_DONE0)
    );
    let state = RefCell::new((100, RESET, std::vec::Vec::new()));
    let mut reset = Reset::Fresh;
    let mut step = || {
        reset.step(
            |address| {
                let state = state.borrow();
                if address == TIMER_RAWL {
                    state.0
                } else {
                    assert_eq!(address, RESET_DONE);
                    state.1
                }
            },
            |address, value| state.borrow_mut().2.push((address, value)),
        )
    };
    assert_eq!(step(), Err(Error::NotReady));
    assert_eq!(state.borrow().2, [(RESET_SET, RESET)]);
    state.borrow_mut().1 = 0;
    assert_eq!(step(), Err(Error::NotReady));
    assert_eq!(state.borrow().2, [(RESET_SET, RESET), (RESET_CLEAR, RESET)]);
    state.borrow_mut().1 = RESET;
    assert_eq!(step(), Ok(()));
}

#[test]
fn reset_timeout_reasserts_reset_even_when_the_timer_wraps() {
    let time = Cell::new(u32::MAX - 7);
    let writes = RefCell::new(std::vec::Vec::new());
    let mut reset = Reset::Fresh;
    let mut step = || {
        reset.step(
            |address| if address == TIMER_RAWL { time.get() } else { 0 },
            |address, value| writes.borrow_mut().push((address, value)),
        )
    };
    assert_eq!(step(), Err(Error::NotReady));
    time.set(time.get().wrapping_add(1_000));
    assert_eq!(step(), Err(Error::Timeout));
    assert_eq!(step(), Err(Error::Timeout));
    assert_eq!(
        *writes.borrow(),
        [(RESET_SET, RESET), (RESET_CLEAR, RESET), (RESET_SET, RESET)]
    );
}

pub(super) fn block() -> PIO {
    PIO {
        clocks: Clocks::for_test(200_000_000, 50_000_000),
        used: Cell::new(0),
        active: [const { Cell::new(0) }; 4],
        local: PhantomData,
    }
}

#[test]
fn lifecycle_never_writes_dma_thresholds() {
    use crate::{
        gpio::{PadConfig, ctrl},
        test_mmio::{Trace, record},
    };

    let mut trace = Trace::default();
    trace.registers.extend([
        (0x4001_8018, 1), // CLK_SYS_DIV_INT
        (0x4001_8020, 1), // CLK_SYS_SEL: XOSC
        (RESET_DONE, RESET),
        (ctrl(12), 0x0090_001f), // Two IRQ event masks, disconnected mux.
    ]);
    let trace = record(trace, || {
        let clocks = Clocks::for_test(50_000_000, 50_000_000);
        let parts = unsafe { PioParts::<4, _>::__new(UninitStateMachine::<2>::__new()) };
        let (pio, sm) = parts.split_wait(&clocks).unwrap();
        let program = pio_asm!("set pins, 1").program;
        let program = pio.install(&program).unwrap();
        let pin = unsafe { PioPin::<12>::__new() };
        let (mut sm, rx, tx) = PIOBuilder::from_installed_program(&program)
            .set_pins(12, 1)
            .pins(Pins::new().output(pin, PadConfig::OUTPUT_DEFAULT, InitialOutput::HiZHigh))
            .build(sm)
            .unwrap();
        sm.restart();
        let running = sm.start().unwrap();
        let (_, mut pins) = running.stop().uninit(rx, tx);
        assert!(pins.take::<12>().is_some());
        program.uninstall().unwrap();
    });

    assert_eq!(
        &trace.writes[..7],
        &[
            (RESET_SET, RESET),
            (RESET_CLEAR, RESET),
            (CTRL, 0),
            (IRQ0_INTE, 0),
            (IRQ0_INTF, 0),
            (IRQ1_INTE, 0),
            (IRQ1_INTF, 0),
        ]
    );
    assert_eq!(trace.registers[&ctrl(12)], 0x0090_8005);
    let sm_writes = |offset| {
        trace
            .writes
            .iter()
            .filter_map(move |&(address, value)| (address == sm_reg(2, offset)).then_some(value))
    };
    // Configure and restart both force X/Y, the latch and direction, then JMP.
    let instructions: std::vec::Vec<_> = sm_writes(0x10).collect();
    let reset = [0xe020, 0xe040, 0xe001, 0xe080, 31];
    assert_eq!(instructions, [reset, reset].concat());
    assert_eq!(sm_writes(0x08).count(), 5); // FIFO JOIN transitions at configure and release.
    assert_eq!(
        trace
            .writes
            .iter()
            .filter(|&&(address, _)| address == FDEBUG)
            .copied()
            .collect::<std::vec::Vec<_>>(),
        [(FDEBUG, 0x0404_0404)]
    );
    for n in 0..4 {
        for offset in [0x18, 0x1c] {
            let register = sm_reg(n, offset);
            assert!(
                trace.writes.iter().all(|&(address, _)| address != register),
                "unexpected write to {register:#x}"
            );
        }
    }
}
