// The token that the notifier methods take cannot be named or built outside rp1-link.
use rp1_link::notify::{HostIrq, Notify};
fn claim<N: Notify>() -> N {
    N::claim(rp1_link::notify::sealed::Token(()))
}
fn main() {
    let _: HostIrq = claim();
}
