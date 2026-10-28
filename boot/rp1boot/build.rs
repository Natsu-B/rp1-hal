fn main() {
    // Only the bare-metal loader uses the linker script; host builds (tests) do not.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("none") {
        let dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        println!("cargo:rustc-link-arg-bins=-T{dir}/link.ld");
    }
    println!("cargo:rerun-if-changed=link.ld");
}
