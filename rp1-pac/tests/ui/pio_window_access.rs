fn access(p: &rp1_pac::Peripherals) {
    let _ = p.PIO_M3;
    let _ = p.PIO.txf(0).read();
    p.PIO.rxf(0).push(0);
    let pio = p.PIO.pio_m3();
    let _ = pio.instr_mem(0).read();
    unsafe { pio.sm_addr(0).write_bits_unchecked(0) };
    unsafe { pio.ctrl().atomic_set_bits_unchecked(1) };
    unsafe { pio.fdebug().write_bits_unchecked(1) };
}

fn main() {}
