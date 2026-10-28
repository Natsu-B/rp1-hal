fn misuse(resets: &rp1_pac::RESETS) {
    resets.ctrl0().atomic_set(|w| w.candidate_bit6().set());
}

fn main() {}
