//! Review #1: a `#[shared]` static must not hand safe code an RMW atomic on shared SRAM.
#[test]
fn shared_statics_expose_no_rmw() {
    let tests = trybuild::TestCases::new();
    tests.compile_fail("tests/ui/*.rs");
}
