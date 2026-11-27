use std::{env, fs, path::PathBuf};

fn main() {
    rp1_build::configure().unwrap();

    // Keep the generated IRQ bindings and ISRAM placement. This fixture wraps
    // their call sites to count delivery and inject a lost completion.
    let generated = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("rp1.rs");
    let source = fs::read_to_string(&generated).unwrap();
    let adc = "unsafe { ::rp1_hal::adc::__on_interrupt::<0, 0>() };";
    let timer = "unsafe { ::rp1_hal::timer::WaitTimer::__on_interrupt::<1>() };";
    assert_eq!(source.matches(adc).count(), 1);
    assert_eq!(source.matches(timer).count(), 1);
    fs::write(
        generated,
        source
            .replace(
                adc,
                "crate::observe_adc(|| unsafe { ::rp1_hal::adc::__on_interrupt::<0, 0>() });",
            )
            .replace(timer, "crate::observe_wait_alarm();"),
    )
    .unwrap();
}
