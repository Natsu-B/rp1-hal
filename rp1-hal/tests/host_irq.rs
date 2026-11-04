//! Review (irq-path) major 1: notification needs exclusive ownership of the one HostIrq.
#![cfg(feature = "unstable-host-irq")]
#[test]
fn host_irq_needs_exclusive_ownership() {
    let tests = trybuild::TestCases::new();
    tests.compile_fail("tests/ui-host-irq/*.rs");
}
