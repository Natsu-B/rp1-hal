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
fn schema1_error(body: &str) -> String {
    toml::from_str::<Declaration>(&format!("schema=1\n{body}"))
        .unwrap_err()
        .to_string()
}
#[test]
fn schema1_without_complete_route_is_an_error() {
    // Schema 2 cannot express a pinless UART/I2C/SPI, so conversion cannot keep one.
    for body in [
        "[proc0]\npins=[2,22]\nperipherals=['I2C1']",
        "[proc0]\npins=[22]\nperipherals=['UART2']",
    ] {
        let message = schema1_error(body);
        assert!(message.contains("schema 1: cannot infer"), "{message}");
        assert!(message.contains("(schema 2)"), "{message}");
    }
}
#[test]
fn schema1_gpio_irq_pins_stay_gpio() {
    let d: Declaration =
        toml::from_str("schema=1\n[proc0]\npins=[2,3,10]\ngpio_irq=[10]\nperipherals=['I2C1']")
            .unwrap();
    d.validate().unwrap();
    assert_eq!(d.proc0.gpio, [10]);
    assert_eq!(
        d.proc0.instances["i2c1"].pins("i2c1"),
        [("sda", 2), ("scl", 3)]
    );
    // The only RX pin is an IRQ line: it stays GPIO, so UART2 has no RX.
    let message = schema1_error("[proc0]\npins=[4,5]\ngpio_irq=[5]\nperipherals=['UART2']");
    assert!(
        message
            .contains("schema 1: GPIO 5 is in gpio_irq, so it stays GPIO and cannot be UART2_RX"),
        "{message}"
    );
    assert!(
        message.contains("write uart2 = { tx = .., rx = .. }"),
        "{message}"
    );
}
#[test]
fn schema1_uart5_has_no_bank0_pins() {
    let message = schema1_error("[proc0]\nperipherals=['UART5']");
    assert!(
        message.contains("E2: UART5 has no bank-0 pins"),
        "{message}"
    );
}
#[test]
fn schema1_preserves_claim_order_and_infers_only_required_signals() {
    let d: Declaration = toml::from_str(
        "schema=1\n[proc0]\npins=[7,6,5,4,22]\nperipherals=['UART2','TIMER.ALARM1','ADC']",
    )
    .unwrap();
    d.validate().unwrap();
    // CTS/RTS are optional: schema 1 listed 6/7 as pins, so they stay GPIO.
    assert_eq!(d.proc0.gpio, [7, 6, 22]);
    assert_eq!(d.proc0.pins(), [7, 6, 5, 4, 22]);
    assert_eq!(d.proc0.peripherals(), ["UART2", "TIMER.ALARM1", "ADC"]);
    assert_eq!(
        d.proc0.instances["uart2"].pins("uart2"),
        [("tx", 4), ("rx", 5)]
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

        // Rewriting as schema 2 changes the declaration hash; the rest of the note is identical.
        let normalized = toml::to_string(&d).unwrap();
        let mut schema2: Declaration = toml::from_str(&normalized).unwrap();
        schema2.types = d.types.clone();
        schema2.validate().unwrap();
        let hash2 = schema2.declaration_hash().unwrap();
        assert_ne!(hash2, hash, "{manifest}");
        let note2 = wire::descriptor(&name, &version, &schema2, hash2, [0; 20]).unwrap();
        assert_eq!(
            without_hash(&note2, hash2),
            without_hash(&note, hash),
            "{manifest}"
        );
    }
}
/// The note with its declaration hash and the header CRC (which covers it) zeroed.
fn without_hash(note: &[u8], hash: [u8; 32]) -> Vec<u8> {
    let mut note = note.to_vec();
    let at = note.windows(32).position(|w| w == hash).unwrap();
    note[at..at + 32].fill(0);
    note[20..24].fill(0);
    note
}
#[test]
fn converted_models_serialize_to_equivalent_schema2() {
    let mut converted: Vec<Declaration> = [
        "[proc0]\npins=[2,3,22]\nperipherals=['TIMER.ALARM0','I2C1']",
        "[proc0]\npins=[7,6,5,4,22]\ngpio_irq=[22]\nperipherals=['UART2','ADC']",
        "log={sink='uart0',baudrate=115200}\n[proc1]\npins=[8,9,10,11,27]\nperipherals=['SPI0']",
    ]
    .map(|body| toml::from_str(&format!("schema=1\n{body}")).unwrap())
    .into();
    for manifest in [
        "examples/dual-core/Cargo.toml",
        "lab/shm-firmware/Cargo.toml",
    ] {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join(manifest);
        converted.push(declaration(&path).unwrap().2);
    }
    for d in converted {
        let schema2 = toml::to_string(&d).unwrap();
        assert!(schema2.starts_with("schema = 2\n"), "{schema2}");
        let back: Declaration = toml::from_str(&schema2).unwrap();
        back.validate().unwrap();
        assert_eq!(toml::to_string(&back).unwrap(), schema2);
        for (a, b) in [(&d.proc0, &back.proc0), (&d.proc1, &back.proc1)] {
            assert_eq!(a.gpio, b.gpio);
            assert_eq!(a.gpio_irq, b.gpio_irq);
            assert_eq!(a.timer, b.timer);
            assert_eq!(a.instances, b.instances);
        }
        assert_eq!(d.linux, back.linux);
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
