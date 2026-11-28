#[path = "../serial-parity-build.rs"]
mod build;

fn main() {
    if cfg!(feature = "uart0-report") {
        return;
    }
    build::configure(
        "uart",
        1,
        &[0, 2, 3, 4],
        &[("Report", rp1_build::metadata::TypeInfo::of::<[u32; 16]>())],
    );
}
