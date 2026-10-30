//! Review #6: safe code cannot build a second UART0 logger outside the generated `rp1::log`.
//! Review 4 #3: `Clocks` cannot be forged, and its proc1 handoff needs the proc1 token.
//! Sharing T7: UART, I2C and SPI pins outside the bank-0 pin table do not compile.
#[test]
fn logger_needs_log_ownership() {
    let tests = trybuild::TestCases::new();
    tests.compile_fail("tests/ui/*.rs");
}
