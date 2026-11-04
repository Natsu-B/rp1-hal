// Safe code cannot forge a HostIrq.
use rp1_hal::host_irq::HostIrq;
fn forged() -> HostIrq {
    HostIrq { _private: () }
}
fn main() {}
