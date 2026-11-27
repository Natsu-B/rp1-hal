use core::cell::Cell;
use rp1_rt::interrupt::{Mutex, free};

mod rp1 {
    pub mod __irq_proc0 { pub trait Handler {} pub trait Declared {} #[allow(non_camel_case_types)] pub enum TIMER_0 {} impl Declared for TIMER_0 {} }
    pub mod __irq_proc1 { pub trait Handler {} pub trait Declared {} #[allow(non_camel_case_types)] pub enum TIMER_0 {} impl Declared for TIMER_0 {} }
}

static COUNT: Mutex<Cell<u32>> = Mutex::new(Cell::new(0));
#[rp1_rt::interrupt(proc0)]
fn r#TIMER_0() { free(|cs| COUNT.borrow(cs).set(COUNT.borrow(cs).get() + 1)); }
mod proc1 {
    #[rp1_rt::interrupt(proc1, sram)]
    fn TIMER_0() -> () {}
}
#[rp1_rt::interrupt(proc0)]
#[cfg(any())]
fn UNKNOWN_BUT_DISABLED() {}
fn main() {}
