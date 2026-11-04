// Safe code cannot make a second HostIrq (one owner of the RP1 -> Linux interrupt).
use rp1_hal::host_irq::HostIrq;
fn second() -> HostIrq {
    HostIrq::new()
}
fn main() {}
