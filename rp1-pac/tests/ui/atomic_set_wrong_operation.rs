fn misuse(resets: &rp1_pac::RESETS) {
    resets.ctrl1().atomic_set(|w| w.uart0().clear());
}

fn main() {}
