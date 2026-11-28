use std::{env, fs, path::Path};

fn main() {
    rp1_build::configure_with_manifest(Path::new("Cargo.toml"), &[]).unwrap();
    let generated = Path::new(&env::var_os("OUT_DIR").unwrap()).join("rp1.rs");
    let mut source = fs::read_to_string(&generated).unwrap();
    for n in [0, 1, 3] {
        let call = format!("unsafe {{ ::rp1_hal::spi::__on_interrupt::<{n}, 0>() }}");
        assert_eq!(source.matches(&format!("{call};")).count(), 1);
        source = source.replace(&format!("{call};"), &format!("crate::device(|| {call});"));
    }
    let timer = "unsafe { ::rp1_hal::timer::WaitTimer::__on_interrupt::<1>() };";
    assert_eq!(source.matches(timer).count(), 1);
    source = source.replace(timer, "crate::alarm();");
    fs::write(generated, source).unwrap();
}
