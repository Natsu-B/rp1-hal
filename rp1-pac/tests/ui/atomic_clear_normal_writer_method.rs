fn misuse(resets: &rp1_pac::RESETS) {
    resets.ctrl1().atomic_clear(|w| w.uart0().clear_bit());
}

fn main() {}
