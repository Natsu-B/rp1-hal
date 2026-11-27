use rp1_hal::dma::{Channel, Lease};

fn in_flight(channel: &mut Channel<2>, mut source: Lease<'_>, mut destination: Lease<'_>) {
    let transfer = channel.try_start(&mut source, &mut destination).unwrap();
    destination.as_mut_slice()[0] = 0;
    drop(transfer);
}

fn main() {}
