use rp1_build::metadata::TypeInfo;
use std::path::Path;

fn main() {
    let manifest = if cfg!(feature = "ping-poll") {
        "../../lab/ping/dual-core-poll.toml"
    } else {
        "Cargo.toml"
    };
    rp1_build::configure_with_manifest(
        Path::new(manifest),
        &[
            ("Ping", TypeInfo::of::<u32>()),
            ("Pong", TypeInfo::of::<u32>()),
        ],
    )
    .unwrap();
}
