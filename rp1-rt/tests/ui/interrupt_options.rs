#[rp1_rt::interrupt(proc0, sram, sram)]
fn TIMER_0() {}
#[rp1_rt::interrupt(proc0, ram)]
fn TIMER_1() {}
#[rp1_rt::interrupt(proc1, __platform)]
fn PROC_MISC() {}
fn main() {}
