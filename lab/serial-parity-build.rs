use std::{
    env, fs,
    path::{Path, PathBuf},
};

// Trace the generated dispatcher and preserve its vector, ownership and placement.
pub fn configure(
    block: &str,
    default: u8,
    variants: &[u8],
    types: &[(&str, rp1_build::metadata::TypeInfo)],
) -> u8 {
    let enabled: Vec<_> = variants
        .iter()
        .copied()
        .filter(|n| env::var_os(format!("CARGO_FEATURE_{}{n}", block.to_uppercase())).is_some())
        .collect();
    assert!(enabled.len() <= 1, "select one {block} instance");
    let n = enabled.first().copied().unwrap_or(default);
    let manifest = if n == default {
        "Cargo.toml".into()
    } else {
        format!("instances/{block}{n}.toml")
    };
    rp1_build::configure_with_manifest(Path::new(&manifest), types).unwrap();

    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let alias = format!("{}{}{n}", block[..1].to_uppercase(), &block[1..]);
    let pins = match (block, n) {
        ("spi", _) | ("i2c", 1) => format!("use rp1::{alias}Pins as DriverPins;"),
        _ => String::new(),
    };
    fs::write(out.join("instance.rs"), format!(
        "const N: u8 = {n};\n         use rp1::{alias} as Driver;\n{pins}\n         use rp1_hal::pac::{}{n} as Registers;\n         macro_rules! parts {{ ($p:ident) => {{ $p.{block}{n} }}; }}\n",
        block.to_uppercase()
    )).unwrap();

    let generated = out.join("rp1.rs");
    let mut source = fs::read_to_string(&generated).unwrap();
    let call = format!("unsafe {{ ::rp1_hal::{block}::__on_interrupt::<{n}, 0>() }}");
    assert_eq!(source.matches(&format!("{call};")).count(), 1);
    source = source.replace(
        &format!("{call};"),
        &format!("crate::observe_device(|| {call});"),
    );
    if block != "uart" || n != default {
        let timer = "unsafe { ::rp1_hal::timer::WaitTimer::__on_interrupt::<1>() };";
        assert_eq!(source.matches(timer).count(), 1);
        source = source.replace(timer, "crate::observe_wait_alarm();");
    }
    fs::write(generated, source).unwrap();
    n
}
