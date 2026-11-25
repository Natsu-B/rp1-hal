//! Notification requires exclusive ownership of the generated HostIrq.
#![cfg(feature = "unstable-host-irq")]
#[test]
fn host_irq_needs_exclusive_ownership() {
    let tests = trybuild::TestCases::new();
    tests.compile_fail("tests/ui-host-irq/*.rs");
}
