fn main() {
    let peripherals = unsafe { rp1_pac::Peripherals::steal() };
    peripherals
        .RESETS
        .ctrl0()
        .modify(|_, w| w.candidate_bit6().set_bit());
}
