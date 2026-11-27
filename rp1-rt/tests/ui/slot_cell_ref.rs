#![cfg_attr(target_os = "none", no_std)]

use core::cell::Cell;
use rp1_rt::interrupt::Slot;

static SLOT: Slot<Option<&'static Cell<u32>>, 0> = Slot::empty();

#[cfg(not(target_os = "none"))]
fn main() {}
