fn main() {
    let peripherals = unsafe { rp1_pac::Peripherals::steal() };
    peripherals
        .WATCHDOG
        .boot_magic()
        .atomic_set(|w| w.value().set_bits(1));
}
