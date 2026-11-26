// Safe code cannot make a notifier through a bound on the public traits either.
use rp1_link::notify::{Doorbell, HostIrq, Notify, Wake};
fn claim_notify<N: Notify>() -> N {
    N::claim()
}
fn claim_wake<W: Wake>() -> W {
    W::claim()
}
fn main() {
    let _: HostIrq = claim_notify();
    let _: Doorbell<27, 1> = claim_wake();
}
