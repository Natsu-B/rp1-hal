// Review 8: the unchecked `spi::Bus` operations cannot be called outside rp1-hal, neither
// through a generic bound nor on `Spi`, so the frame-size and clk_sys admission is not skipped.
use rp1_hal::spi::{Bus, Spi};
fn generic<B: Bus<u8>>(bus: &mut B, rx: &mut [u8], tx: &[u8]) {
    let _ = bus.admit();
    let _ = bus.exchange(rx, tx);
    let _ = bus.exchange_in_place(rx);
    let _ = bus.wait_idle();
}
fn concrete(spi: &mut Spi<0, 11, 10, 9>, rx: &mut [u16], tx: &[u16]) {
    let _ = Bus::<u16>::exchange(spi, rx, tx);
    let _ = rp1_hal::spi::sealed::Key(());
}
fn main() {}
