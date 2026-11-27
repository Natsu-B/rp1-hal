use rp1_build::metadata::TypeInfo;
use std::path::Path;

fn main() {
    let manifest = if cfg!(feature = "link-wake") {
        assert!(
            !cfg!(feature = "ping-poll"),
            "link-wake and ping-poll are separate profiles"
        );
        "../../lab/ping/dual-core-wake.toml"
    } else if cfg!(feature = "ping-poll") {
        "../../lab/ping/dual-core-poll.toml"
    } else {
        "Cargo.toml"
    };
    rp1_build::configure_with_manifest(
        Path::new(manifest),
        &[
            ("Ping", TypeInfo::of::<u32>()),
            ("Pong", TypeInfo::of::<u32>()),
            ("Nonce", TypeInfo::of::<u32>()),
        ],
    )
    .unwrap();
}
