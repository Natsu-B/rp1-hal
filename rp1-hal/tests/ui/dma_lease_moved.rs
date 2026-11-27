use rp1_hal::dma::{Channel, Lease};

fn moved(channel: &mut Channel<2>, source: Lease<'_>, mut destination: Lease<'_>) {
    let _transfer = channel.try_start(source, destination).unwrap();
    destination.as_mut_slice()[0] = 0;
}

fn main() {}
