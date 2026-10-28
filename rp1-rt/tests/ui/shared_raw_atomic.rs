use core::sync::atomic::{AtomicU32, Ordering};

#[rp1_rt::shared]
static COUNTER: AtomicU32 = AtomicU32::new(0);

fn main() {
    COUNTER.fetch_add(1, Ordering::Relaxed);
}
