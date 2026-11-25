// Safe code cannot make a second HostIrq (one owner of the RP1 -> Linux interrupt).
use rp1_link::notify::HostIrq;
fn second() -> HostIrq {
    HostIrq::new()
}
fn main() {}
