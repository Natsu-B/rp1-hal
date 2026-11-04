// A shared reference cannot pulse: two interleaved pulses could lose a wake.
use rp1_hal::host_irq::HostIrq;
fn shared(irq: &HostIrq) {
    irq.notify_sram();
}
fn main() {}
