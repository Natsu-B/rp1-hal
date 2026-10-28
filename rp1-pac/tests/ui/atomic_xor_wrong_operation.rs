fn misuse(resets: &rp1_pac::RESETS) {
    resets.ctrl1().atomic_xor(|w| w.uart0().set());
}

fn main() {}
