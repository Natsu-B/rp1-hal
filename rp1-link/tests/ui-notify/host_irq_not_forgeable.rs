// Safe code cannot forge a HostIrq.
use rp1_link::notify::HostIrq;
fn forged() -> HostIrq {
    HostIrq { _private: () }
}
fn main() {}
