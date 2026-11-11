use core::cell::Cell;
use rp1_rt::interrupt::{Mutex, Priority, free};

static COUNT: Mutex<Cell<u32>> = Mutex::new(Cell::new(0));
#[rp1_rt::interrupt(proc0)]
fn TIMER_0() { free(|cs| COUNT.borrow(cs).set(COUNT.borrow(cs).get() + 1)); }
mod proc1 {
    #[rp1_rt::interrupt(proc1, sram)]
    fn TIMER_0() -> () {}
}
#[rp1_rt::interrupt(proc0, __platform)]
fn PCIE_OUT() {}
#[rp1_rt::interrupt(proc0)]
#[cfg(any())]
fn UNKNOWN_BUT_DISABLED() {}
fn main() { assert_eq!(Priority::new(0), None); }
