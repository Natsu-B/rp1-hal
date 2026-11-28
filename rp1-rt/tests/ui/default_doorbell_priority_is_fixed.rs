use rp1_rt::interrupt::{Binding, Bank0Irq, Priority, priority};

fn raise(line: &mut Binding<Bank0Irq, 0, 0, { priority::DOORBELL }>) {
    line.set_priority(Priority::control::<2>());
}

fn main() {}
