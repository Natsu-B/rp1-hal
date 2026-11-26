// proc1's Clocks handoff takes the proc1 token; proc0 has its own value from platform::init.
use rp1_hal::{clocks::Clocks, fugit::ExtU32};
fn proc0_side(core: &rp1_rt::Proc0) {
    let _ = Clocks::wait_from_proc0(core, 0.micros());
}
fn main() {}
