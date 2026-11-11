#[rp1_rt::interrupt(proc1)]
fn TIMER_1(value: u32) { let _ = value; }
fn main() {}
