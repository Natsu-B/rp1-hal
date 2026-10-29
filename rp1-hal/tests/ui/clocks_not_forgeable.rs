// A Clocks is proof that proc0's Authority::init ran: safe code cannot build one from numbers.
use rp1_hal::clocks::Clocks;
fn forged() -> Clocks {
    Clocks { sys_hz: 200_000_000, uart_hz: 50_000_000 }
}
fn main() {}
