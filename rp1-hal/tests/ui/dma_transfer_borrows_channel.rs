use rp1_hal::dma::{Channel, Lease};

fn second_start(channel: &mut Channel<2>, source: Lease<'_>, destination: Lease<'_>) {
    let transfer = channel.try_start(source, destination).unwrap();
    let _ = channel.try_recover();
    drop(transfer);
}

fn main() {}
