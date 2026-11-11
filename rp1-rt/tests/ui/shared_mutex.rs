use core::cell::Cell;
use rp1_rt::interrupt::Mutex;
#[rp1_rt::shared]
static VALUE: Mutex<Cell<u32>> = Mutex::new(Cell::new(0));
fn main() {}
