use rp1_hal::dma::Pool;

fn stack() {
    let mut memory = [0; 16];
    let _ = Pool::<2>::new(&mut memory);
}

fn main() {}
