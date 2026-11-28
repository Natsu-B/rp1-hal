fn main() {
    rp1_build::configure_with_types(&[("PingWord", rp1_build::metadata::TypeInfo::of::<u32>())])
        .unwrap();
}
