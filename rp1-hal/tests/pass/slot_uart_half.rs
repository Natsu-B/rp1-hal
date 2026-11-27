#![cfg_attr(target_os = "none", no_std)]

use rp1_hal::uart::{Irq, Rx};
use rp1_rt::interrupt::Slot;

pub static RX: Slot<Option<Rx<2, 5>>, 0> = Slot::empty();
pub static MANAGED_RX: Slot<Option<Rx<1, 1, Irq<1, 0>>>, 0> = Slot::empty();

#[cfg(not(target_os = "none"))]
fn main() {}
