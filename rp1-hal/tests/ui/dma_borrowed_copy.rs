use rp1_hal::{dma::{Channel, PollingTimeout}, timer::Duration};

fn borrowed(channel: &mut Channel<2>, src: &[u8], dst: &mut [u8]) {
    let _ = channel.copy(src, dst, PollingTimeout::new(Duration::millis(1)));
}

fn main() {}
