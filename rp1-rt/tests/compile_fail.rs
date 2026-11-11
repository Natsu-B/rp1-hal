//! trybuild: #[interrupt] contract and #[shared] safety.
#[test]
fn interrupt_contracts_and_shared_safety() {
    let tests = trybuild::TestCases::new();
    tests.compile_fail("tests/ui/*.rs");
    tests.pass("tests/pass/*.rs");
}
