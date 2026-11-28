use rp1_proto::BlockId;
use rp1_rt::interrupt::{Binding, BlockIrq};

fn main() {
    let line = unsafe { Binding::<BlockIrq<{ BlockId::Pio as u16 }>, 50, 0>::new() };
    unsafe {
        rp1_hal::pio::__bind_irq::<0, 50, 0>(line);
        rp1_hal::pio::__on_interrupt::<0, 0>();
    }
}
