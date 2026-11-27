use rp1_hal::{dma::Channel, timer::Duration};

fn borrowed(channel: &mut Channel<2>, src: &mut [u8], dst: &mut [u8]) {
    let _ = channel.copy_polling(src, dst, Duration::millis(1));
}

fn main() {}
