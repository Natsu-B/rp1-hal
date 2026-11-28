use rp1_rt::interrupt::{Bank0Irq, Binding, FixedDoorbell, Priority};

fn raise(line: &mut Binding<Bank0Irq, 0, 0, FixedDoorbell>) {
    line.set_priority(Priority::control::<2>());
}

fn main() {}
