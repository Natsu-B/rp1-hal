use rp1_hal::{gpio::Pin, spi::Pins};

fn wrong(mosi: Pin<10>, sck: Pin<11>, wrong_miso: Pin<5>) {
    // SPI3's MISO pin cannot be used on SPI0.
    let _ = Pins::<0>::builder().sck(mosi);
    let _ = Pins::<0>::builder().sck(sck).mosi(mosi).miso(wrong_miso);
}
fn main() {}
