use std::{env, fs, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=irq.x");

    if env::var_os("CARGO_FEATURE_RT").is_none() {
        return;
    }

    let out = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo must set OUT_DIR"));

    fs::copy("irq.x", out.join("irq.x")).expect("copy generated irq.x");

    println!("cargo:rustc-link-search={}", out.display());
}
