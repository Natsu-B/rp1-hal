fn main() {
    let peripherals = unsafe { rp1_pac::Peripherals::steal() };
    peripherals.TIMER.armed().clear(|w| w.alarm().clear());
}
