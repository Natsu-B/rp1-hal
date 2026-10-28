fn main() {
    let peripherals = unsafe { rp1_pac::Peripherals::steal() };
    peripherals
        .WATCHDOG
        .boot_magic()
        .modify(|_, w| w.value().bits(0x5250_3100));
}
