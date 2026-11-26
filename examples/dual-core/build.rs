use rp1_build::metadata::TypeInfo;

fn main() {
    rp1_build::configure_with_types(&[
        ("Ping", TypeInfo::of::<u32>()),
        ("Pong", TypeInfo::of::<u32>()),
    ])
    .unwrap();
}
