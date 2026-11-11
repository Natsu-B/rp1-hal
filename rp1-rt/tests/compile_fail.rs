//! Review #1: a `#[shared]` static must not hand safe code an RMW atomic on shared SRAM.
#[test]
fn interrupt_contracts_and_shared_safety() {
    let tests = trybuild::TestCases::new();
    tests.compile_fail("tests/ui/*.rs");
    tests.pass("tests/pass/*.rs");
}
