fn main() {
    let peripherals = unsafe { rp1_pac::Peripherals::steal() };
    peripherals.WATCHDOG.load().modify(|_, w| w);
}
