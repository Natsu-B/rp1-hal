use rp1_hal::{
    dma::{Channel, Lease, PioFifo, PioRequests},
    pio::{Rx, StateMachine, Stopped, Tx},
};

fn wrong_direction(ch: &mut Channel<0, PioRequests>, data: &mut Lease<'_>, rx: &mut PioFifo<Rx<0>>) {
    let _ = ch.try_start_to_pio(data, rx);
}

fn channel_borrow(
    ch: &mut Channel<0, PioRequests>,
    a: &mut Lease<'_>,
    b: &mut Lease<'_>,
    tx: &mut PioFifo<Tx<0>>,
    rx: &mut PioFifo<Rx<0>>,
) {
    let transfer = ch.try_start_to_pio(a, tx).unwrap();
    let _ = ch.try_start_from_pio(rx, b);
    drop(transfer);
}

fn fifo_borrow(
    a: &mut Channel<0, PioRequests>,
    b: &mut Channel<1, PioRequests>,
    src: &mut Lease<'_>,
    other: &mut Lease<'_>,
    tx: &mut PioFifo<Tx<0>>,
) {
    let transfer = a.try_start_to_pio(src, tx).unwrap();
    let _ = b.try_start_to_pio(other, tx);
    drop(transfer);
}

fn lease_during_transfer(ch: &mut Channel<0, PioRequests>, mut dst: Lease<'_>, mut rx: PioFifo<Rx<0>>) {
    let transfer = ch.try_start_from_pio(&mut rx, &mut dst).unwrap();
    let _ = dst.as_mut_slice();
    drop(dst);
    drop(transfer);
}

fn fifo_during_transfer(ch: &mut Channel<0, PioRequests>, mut src: Lease<'_>, mut tx: PioFifo<Tx<0>>) {
    let transfer = ch.try_start_to_pio(&mut src, &mut tx).unwrap();
    let _ = tx.try_cpu();
    drop(transfer);
}

fn sm_during_transfer(
    ch: &mut Channel<0, PioRequests>,
    mut dst: Lease<'_>,
    mut rx: PioFifo<Rx<0>>,
    tx: Tx<0>,
    sm: StateMachine<'_, '_, 0, Stopped>,
) {
    let transfer = ch.try_start_from_pio(&mut rx, &mut dst).unwrap();
    let _ = sm.uninit(rx.try_into_inner().ok().unwrap(), tx);
    drop(transfer);
}

fn main() {}
