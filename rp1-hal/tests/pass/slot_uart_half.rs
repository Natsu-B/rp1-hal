#![cfg_attr(target_os = "none", no_std)]

use rp1_hal::uart::Rx;
use rp1_rt::interrupt::Slot;

pub static RX: Slot<Option<Rx<2, 5>>, 0> = Slot::empty();

#[cfg(not(target_os = "none"))]
fn main() {}
