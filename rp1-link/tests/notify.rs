//! Only generated code makes notifiers: safe code can neither construct nor forge them, and
//! cannot reach their methods through a bound on the public traits.
#[test]
fn notifiers_are_not_constructible() {
    let tests = trybuild::TestCases::new();
    tests.compile_fail("tests/ui-notify/*.rs");
}
