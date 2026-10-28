fn main() {
    let peripherals = unsafe { rp1_pac::Peripherals::steal() };
    peripherals.TIMER.pause().write(|w| w);
}
