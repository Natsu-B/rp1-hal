fn main() {
    if std::env::var("CARGO_CFG_TARGET_ARCH").as_deref() == Ok("arm")
        && std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("none")
    {
        rp1_build::configure().unwrap();
    }
}
