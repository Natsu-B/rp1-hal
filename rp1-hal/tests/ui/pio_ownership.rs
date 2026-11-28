use rp1_hal::gpio::{PadConfig, Pin};
use rp1_hal::pio::{InstalledProgram, PIOBuilder, Pins, PioPin, Running, Rx, StateMachine, Stopped, Tx, UninitStateMachine};

fn unowned_pin(gpio: Pin<22>) {
    let _ = Pins::new().input(gpio, PadConfig::INPUT_DEFAULT);
}

fn reused_sm(program: &InstalledProgram<'_>, sm: UninitStateMachine<0>) {
    let _ = PIOBuilder::from_installed_program(program).build(sm);
    let _ = PIOBuilder::from_installed_program(program).build(sm);
}

fn in_use(program: InstalledProgram<'_>, sm: UninitStateMachine<0>) {
    let (sm, _, _) = PIOBuilder::from_installed_program(&program).build(sm).unwrap();
    let running = sm.start().unwrap();
    let _ = program.uninstall();
    drop(running);
}

fn wrong_core() {
    fn send<T: Send>() {}
    send::<UninitStateMachine<0>>();
    send::<Tx<0>>();
    send::<PioPin<22>>();
    send::<Pins>();
}

fn stopped_with_one_half(sm: StateMachine<'_, '_, 0, Stopped>, rx: Rx<0>) {
    let _ = sm.uninit(rx);
}

fn mismatched(sm: StateMachine<'_, '_, 0, Stopped>, rx: Rx<1>, tx: Tx<0>) {
    let _ = sm.uninit(rx, tx);
}

fn still_running(sm: StateMachine<'_, '_, 0, Running>, rx: Rx<0>, tx: Tx<0>) {
    let _ = sm.uninit(rx, tx);
}

fn main() {}
