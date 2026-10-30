use rp1_hal::{clocks::Clocks, gpio::Pin, spi::{Config, Token}};
fn wrong(clocks: &Clocks) {
    // SCLK and MOSI swapped; SPI3's MISO pin given to SPI0.
    let token = unsafe { Token::<0>::new() };
    let config = Config::new(1_000_000, embedded_hal::spi::MODE_0);
    let _ = token.init(
        unsafe { Pin::<10>::new() },
        unsafe { Pin::<11>::new() },
        unsafe { Pin::<5>::new() },
        clocks,
        config,
    );
}
fn main() {}
