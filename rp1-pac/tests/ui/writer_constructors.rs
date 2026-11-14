// A hand-built field writer would reach bits that the register's generated writer does not offer.
fn misuse(resets: &rp1_pac::RESETS, timer: &rp1_pac::TIMER) {
    resets
        .ctrl0()
        .modify(|_, w| rp1_pac::FieldWriter::<_, 0, 32>::new(w).bits(u32::MAX));
    resets
        .ctrl0()
        .modify(|_, w| rp1_pac::BitWriter::<_, 6>::new(w).set_bit());
    resets
        .ctrl0()
        .atomic_set(|w| rp1_pac::AtomicSetFieldWriter::<_, 0, 32>::new(w).set_bits(u32::MAX));
    resets
        .ctrl0()
        .atomic_clear(|w| rp1_pac::AtomicClearBitWriter::<_, 6>::new(w).clear());
    timer
        .intr()
        .clear(|w| rp1_pac::ClearFieldWriter::<_, 0, 32>::new(w).clear_bits(u32::MAX));
    timer
        .alarm(0)
        .command(|w| rp1_pac::CommandBitWriter::<_, 31>::new(w).trigger());
}

fn main() {}
