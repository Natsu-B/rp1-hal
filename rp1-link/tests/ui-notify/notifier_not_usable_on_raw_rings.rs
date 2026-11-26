// Safe code cannot use a notifier on a raw ring through a bound on the public traits.
use rp1_link::notify::{Notify, Wake};
use rp1_proto::{Message, shm};
fn wake<N: Notify, T: Message>(notify: N, ring: &mut shm::RingTx<T>) -> bool {
    notify.wake(ring)
}
fn opened<W: Wake, T: Message>(wake: W, ring: &mut shm::RingRx<T>) {
    wake.opened(ring)
}
fn drain<W: Wake, T: Message>(wake: W, ring: &mut shm::RingRx<T>) {
    let _ = wake.drain(ring, 1, |_| {});
}
fn main() {}
