#[path = "../serial-parity-build.rs"]
mod build;

fn main() {
    build::configure("i2c", 1, &[2, 3], &[]);
}
