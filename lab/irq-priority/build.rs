use std::{env, fs, path::Path};

fn main() {
    rp1_build::configure_with_manifest(Path::new("Cargo.toml"), &[]).unwrap();
    let generated = Path::new(&env::var_os("OUT_DIR").unwrap()).join("rp1.rs");
    let mut source = fs::read_to_string(&generated).unwrap();
    for (irq, call) in [
        (19, "::rp1_hal::spi::__on_interrupt::<0, 0>()"),
        (28, "::rp1_hal::timer::WaitTimer::__on_interrupt::<2>()"),
        (42, "::rp1_hal::uart::__on_interrupt::<1, 0>()"),
    ] {
        let call = format!("unsafe {{ {call} }}");
        assert_eq!(source.matches(&format!("{call};")).count(), 1);
        source = source.replace(
            &format!("{call};"),
            &format!("crate::stamp::<{irq}>(|| {{ {call}; }});"),
        );
    }
    fs::write(generated, source).unwrap();
}
