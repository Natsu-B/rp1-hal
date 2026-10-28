use core::sync::atomic::Ordering;
use rp1_rt::sync::Spsc;

#[rp1_rt::shared]
static QUEUE: Spsc<u32, 4> = Spsc::new();

fn main() {
    QUEUE.head.fetch_add(1, Ordering::Relaxed);
}
