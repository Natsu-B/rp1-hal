fn main() {
    let peripherals = unsafe { rp1_pac::Peripherals::steal() };
    peripherals
        .RESETS
        .ctrl1()
        .write(|writer| writer.uart0().set_bit());
}
