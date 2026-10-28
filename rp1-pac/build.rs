use std::{env, fs, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=device.x");

    if env::var_os("CARGO_FEATURE_VECTORS").is_none() {
        return;
    }

    let out = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo must set OUT_DIR"));

    fs::copy("device.x", out.join("device.x")).expect("copy generated device.x");

    println!("cargo:rustc-link-search={}", out.display());
}
