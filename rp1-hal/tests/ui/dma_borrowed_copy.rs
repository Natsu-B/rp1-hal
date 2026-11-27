use rp1_hal::{dma::Channel, timer::Duration};

fn borrowed(channel: &mut Channel<2>, src: &[u8], dst: &mut [u8]) {
    let _ = channel.polling().copy(src, dst, Duration::millis(1));
}

fn main() {}
