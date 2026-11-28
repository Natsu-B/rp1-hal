#[path = "../serial-parity-build.rs"]
mod build;

fn main() {
    build::configure("spi", 0, &[1, 3], &[]);
}
