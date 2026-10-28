fn main() {
    let peripherals = unsafe { rp1_pac::Peripherals::steal() };
    peripherals.RESETS.ctrl0().atomic_set_bits(1 << 6);
}
