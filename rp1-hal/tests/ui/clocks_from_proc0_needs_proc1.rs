// proc1's Clocks handoff takes the proc1 token; proc0 has its own value from Authority::init.
use rp1_hal::clocks::Clocks;
fn proc0_side(core: &rp1_rt::Proc0) {
    let _ = Clocks::from_proc0(core);
}
fn main() {}
