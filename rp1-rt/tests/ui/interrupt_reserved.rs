#[rp1_rt::interrupt(proc0)]
fn PCIE_OUT() {}
#[rp1_rt::interrupt(proc1)]
fn PROC_MISC() {}
#[rp1_rt::interrupt(proc0)]
fn CLOCKS_DEFAULT() {}
fn main() {}
