use rp1_build::{
    declaration,
    metadata::{Declaration, Instance, TypeInfo},
    wire,
};
use sha2::{Digest, Sha256};
use std::path::Path;

#[test]
fn schema1_infers_signals_and_keeps_remaining_gpio() {
    let d: Declaration =
        toml::from_str("schema=1\n[proc0]\npins=[2,3,22]\nperipherals=['TIMER.ALARM0','I2C1']")
            .unwrap();
    d.validate().unwrap();
    assert_eq!(d.schema, 2);
    assert_eq!(d.proc0.gpio, [22]);
    assert_eq!(d.proc0.timer, ["ALARM0"]);
    assert_eq!(
        d.proc0.instances["i2c1"].pins("i2c1"),
        [("sda", 2), ("scl", 3)]
    );
    assert!(
        d.warnings()
            .contains(&"schema 1 is deprecated; use schema 2")
    );
}
#[test]
fn schema1_ambiguous_inference_is_an_error() {
    for (block, pins) in [("I2C1", "2,3,10,11"), ("I2C3", "6,7,14,15,22,23")] {
        let message = toml::from_str::<Declaration>(&format!(
            "schema=1\n[proc0]\npins=[{pins}]\nperipherals=['{block}']"
        ))
        .unwrap_err()
        .to_string();
        assert!(
            message.contains(&format!("schema 1: cannot infer {block} pins")),
            "{message}"
        );
        assert!(message.contains("(schema 2)"));
    }
}
#[test]
fn schema1_without_complete_route_keeps_gpio() {
    let d: Declaration =
        toml::from_str("schema=1\n[proc0]\npins=[2,22]\nperipherals=['I2C1']").unwrap();
    d.validate().unwrap();
    assert_eq!(d.proc0.gpio, [2, 22]);
    assert!(d.proc0.instances["i2c1"].pins("i2c1").is_empty());
}
#[test]
fn schema1_preserves_claim_order_and_infers_optional_signals() {
    let d: Declaration = toml::from_str(
        "schema=1\n[proc0]\npins=[7,6,5,4,22]\nperipherals=['UART2','TIMER.ALARM1','ADC']",
    )
    .unwrap();
    d.validate().unwrap();
    assert_eq!(d.proc0.gpio, [22]);
    assert_eq!(d.proc0.pins(), [7, 6, 5, 4, 22]);
    assert_eq!(d.proc0.peripherals(), ["UART2", "TIMER.ALARM1", "ADC"]);
    assert_eq!(
        d.proc0.instances["uart2"].pins("uart2"),
        [("tx", 4), ("rx", 5), ("cts", 6), ("rts", 7)]
    );

    let d: Declaration =
        toml::from_str("schema=1\n[proc0]\npins=[11,10]\nperipherals=['I2C1']").unwrap();
    d.validate().unwrap();
    assert_eq!(
        d.proc0.instances["i2c1"].pins("i2c1"),
        [("sda", 10), ("scl", 11)]
    );
}
#[test]
fn schema1_unknown_fields_and_duplicate_pins_are_rejected() {
    for body in [
        "[proc0]\npins=[22,22]",
        "[proc0]\ngpio=[22]",
        "[proc0]\nperipherals=['UART00']",
        "[proc0]\nperipherals=['ADC','ADC']",
    ] {
        assert!(toml::from_str::<Declaration>(&format!("schema=1\n{body}")).is_err());
    }
}
#[test]
fn existing_declarations_have_byte_identical_notes_after_conversion() {
    // STATIC: SHA-256 of complete ELF notes captured before schema 2 was added, using the
    // original declaration hash and a zero source commit. Includes ordering, CRC and provenance.
    let cases = [
        (
            "examples/dual-core/Cargo.toml",
            "38307b294f0e9e14a06e7bd00818e22ccc4391cca9aa8d820d0195dcfc2043a5",
        ),
        (
            "lab/shm-firmware/Cargo.toml",
            "262f81b0ad563985ec1dd6e2c1a79b2e6b56b2148247bb7c3d92fcc1456dd6ba",
        ),
    ];
    for (manifest, expected) in cases {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join(manifest);
        let (name, version, mut d) = declaration(&path).unwrap();
        d.types.extend([
            ("Sample".into(), TypeInfo::of::<[u32; 4]>()),
            ("Control".into(), TypeInfo::of::<[u32; 16]>()),
            ("Bulk".into(), TypeInfo::of::<[u32; 262144]>()),
        ]);
        let hash = d.declaration_hash().unwrap();
        let note = wire::descriptor(&name, &version, &d, hash, [0; 20]).unwrap();
        assert_eq!(
            format!("{:x}", Sha256::digest(wire::elf_note(&note))),
            expected,
            "{manifest}"
        );

        // An explicit schema 2 migration has the same note with the same provenance inputs.
        let normalized = toml::to_string(&d).unwrap();
        let mut schema2: Declaration = toml::from_str(&normalized).unwrap();
        schema2.types = d.types.clone();
        schema2.validate().unwrap();
        assert_eq!(
            wire::descriptor(&name, &version, &schema2, hash, [0; 20]).unwrap(),
            note
        );
    }
}
#[test]
fn schema1_hash_tracks_edits_after_parsing() {
    let mut d: Declaration = toml::from_str("schema=1\n[proc0]\npins=[22]").unwrap();
    let original = d.declaration_hash().unwrap();
    d.proc0.gpio.push(23);
    assert_ne!(d.declaration_hash().unwrap(), original);
    let d: Declaration =
        toml::from_str("schema=1\n[proc0]\npins=[14,15]\nperipherals=['UART0']").unwrap();
    assert!(matches!(d.proc0.instances["uart0"], Instance::Uart(_)));
}
