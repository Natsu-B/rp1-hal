#[test]
fn unsafe_register_operations_are_not_safe_api() {
    let tests = trybuild::TestCases::new();
    tests.compile_fail("tests/ui/*.rs");
}
