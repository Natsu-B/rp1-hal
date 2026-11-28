fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("none") {
        let word = rp1_build::metadata::TypeInfo::of::<u32>();
        rp1_build::__configure_delivery_fixture(&[("u32", word)]).unwrap();
    }
}
