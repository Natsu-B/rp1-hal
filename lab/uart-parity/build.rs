use std::{env, fs, path::PathBuf};

fn main() {
    rp1_build::configure().unwrap();

    // Keep the generated handler, IRQ binding and ISRAM placement. Wrap only
    // this lab image's dispatch call so the trace covers the production path.
    let generated = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("rp1.rs");
    let source = fs::read_to_string(&generated).unwrap();
    let dispatch = "unsafe { ::rp1_hal::uart::__on_interrupt::<0>() };";
    assert_eq!(source.matches(dispatch).count(), 1);
    fs::write(
        generated,
        source.replace(
            dispatch,
            "crate::observe_uart1(|| unsafe { ::rp1_hal::uart::__on_interrupt::<0>() });",
        ),
    )
    .unwrap();
}
