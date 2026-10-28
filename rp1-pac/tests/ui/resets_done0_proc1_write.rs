fn main() {
    let peripherals = unsafe { rp1_pac::Peripherals::steal() };
    peripherals
        .RESETS
        .done0()
        .modify(|_, w| w.proc1().set_bit());
}
