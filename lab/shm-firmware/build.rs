use rp1_build::metadata::TypeInfo;
fn main() {
    rp1_build::configure_with_types(&[
        ("Sample", TypeInfo::of::<[u32; 4]>()),
        ("Control", TypeInfo::of::<[u32; 16]>()),
        ("Bulk", TypeInfo::of::<[u32; 262144]>()),
    ])
    .unwrap();
}
