use rp1_rt::interrupt::{Priority, Slot};

const _: Priority = Slot::<(), 0, 4>::control_priority::<2>();

fn main() {}
