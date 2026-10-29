//! Review 3 #1: raw-address scratch must never overlap the image. The linker script names the
//! fixed scratch floor and asserts the image ends below it; keep both tied to rp1-proto.

const LINK_X: &str = include_str!("../link.x");

fn hex(s: &str) -> u32 {
    u32::from_str_radix(s.trim().trim_start_matches("0x"), 16).unwrap()
}

#[test]
fn image_is_asserted_below_the_fixed_scratch_floor() {
    let link = LINK_X
        .lines()
        .find(|l| l.trim_start().starts_with("LINK "))
        .expect("LINK memory region");
    let origin = link
        .split("ORIGIN =")
        .nth(1)
        .unwrap()
        .split(',')
        .next()
        .unwrap();
    assert_eq!(hex(origin), rp1_proto::LINK_START);
    assert!(LINK_X.contains("__rp1_fixed_scratch_floor = ORIGIN(LINK);"));
    assert!(LINK_X.contains("ASSERT(__image_end <= __rp1_fixed_scratch_floor,"));
}
