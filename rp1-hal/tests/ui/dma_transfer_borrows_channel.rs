use rp1_hal::dma::{Channel, Lease};

fn second_start(channel: &mut Channel<2>, mut source: Lease<'_>, mut destination: Lease<'_>) {
    let transfer = channel.try_start(&mut source, &mut destination).unwrap();
    let _ = channel.try_recover();
    drop(transfer);
}

fn main() {}
