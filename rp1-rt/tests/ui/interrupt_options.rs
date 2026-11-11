#[rp1_rt::interrupt(proc0, sram, sram)]
fn TIMER_0() {}
#[rp1_rt::interrupt(proc0, ram)]
fn TIMER_1() {}
fn main() {}
