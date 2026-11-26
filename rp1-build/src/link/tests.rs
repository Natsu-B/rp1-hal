use super::*;
use crate::{
    metadata::{Declaration, TypeInfo},
    resources, wire,
};
use std::{fs, path::Path};

fn fixture() -> Declaration {
    let mut d: Declaration = toml::from_str(
        r#"
schema = 2
[proc0]
host_irq = true
[proc1]
doorbell = 27

[[shm]]
name = "type"
kind = "ring"
core = "proc0"
direction = "rp1-to-linux"
notify = "irq"
overflow = "overwrite"
message = "Word"
depth = 32

[[shm]]
name = "down"
kind = "ring"
core = "proc1"
direction = "linux-to-rp1"
notify = "irq"
message = "crate::types::Word"
depth = 256

[[shm]]
name = "trace"
kind = "block"
core = "proc1"
type = "core::primitive::u32"
writer = "linux"
"#,
    )
    .unwrap();
    d.types.extend([
        ("Word".into(), TypeInfo::of::<[u32; 4]>()),
        ("crate::types::Word".into(), TypeInfo::of::<[u32; 4]>()),
        ("core::primitive::u32".into(), TypeInfo::of::<u32>()),
    ]);
    d.validate().unwrap();
    d
}

#[test]
fn generated_resources_only_reference_their_link_features() {
    let plain = Declaration::default();
    let source = resources(&plain, &[], &plain.shm_layout().unwrap()).unwrap();
    assert!(!source.contains("rp1_link"));
    assert!(!source.contains("pub mod link"));

    let mut d = fixture();
    let full = resources(&d, &[], &d.shm_layout().unwrap()).unwrap();
    assert!(full.contains("pub link: link::Proc0LinkInit"));
    assert!(full.contains("pub link: link::Proc1LinkInit"));

    d.shm.remove(0);
    let source = resources(&d, &[], &d.shm_layout().unwrap()).unwrap();
    assert!(!source.contains("Proc0Link"));
    assert!(source.contains("pub link: link::Proc1LinkInit"));
    assert!(source.contains("pub host_irq: ::rp1_link::notify::HostIrq"));

    d.shm.clear();
    let irq_only = resources(&d, &[], &d.shm_layout().unwrap()).unwrap();
    assert!(!irq_only.contains("pub mod link"));
    assert!(irq_only.contains("pub host_irq: ::rp1_link::notify::HostIrq"));
    // A doorbell exists without shm too: pre-Ready users answer it.
    assert!(irq_only.contains("pub type Doorbell1 = ::rp1_link::notify::Doorbell<27, 1>;"));
    assert!(irq_only.contains("pub doorbell: Doorbell1,"));
    assert!(irq_only.contains("doorbell: unsafe { Doorbell1::new() },"));
    assert!(!irq_only.contains("Doorbell0"));

    d.proc1.doorbell = None;
    d.proc0.host_irq = false;
    let no_notifier = resources(&d, &[], &d.shm_layout().unwrap()).unwrap();
    assert!(!no_notifier.contains("rp1_link"));
}

#[test]
fn every_region_has_an_alias_that_names_its_notifier() {
    let d = fixture();
    let (body, _) = fragment(&d);
    for alias in [
        "pub type Type = transport::RingTx<crate::Word, transport::notify::HostIrq>;",
        "pub type Down = transport::RingRx<crate::types::Word, super::Doorbell1>;",
        "pub type Trace = transport::Block<::core::primitive::u32>;",
        "pub r#type: Type,",
        "pub down: Down,",
        "r#type: generated::ring_tx(REGIONS[0])?,",
        "down: generated::ring_rx(REGIONS[1])?,",
    ] {
        assert!(body.contains(alias), "{alias}\n{body}");
    }
    let mut poll = d.clone();
    for r in &mut poll.shm {
        r.notify = None;
    }
    poll.proc0.host_irq = false;
    let (body, _) = fragment(&poll);
    assert!(
        body.contains("pub type Type = transport::RingTx<crate::Word>;"),
        "{body}"
    );
    assert!(
        body.contains("pub type Down = transport::RingRx<crate::types::Word>;"),
        "{body}"
    );
}

#[test]
fn aliases_must_be_distinct_and_free() {
    let mut d = fixture();
    d.shm[1].name = "ty_pe".into();
    d.shm[0].name = "ty__pe".into();
    let error = generate(&d.shm_layout().unwrap(), &[]).unwrap_err();
    assert_eq!(
        error.to_string(),
        "SHM names \"ty__pe\" and \"ty_pe\" both give the type alias TyPe; rename one"
    );
    let mut d = fixture();
    d.shm[1].name = "proc1_link".into();
    let error = generate(&d.shm_layout().unwrap(), &[]).unwrap_err();
    assert!(
        error.to_string().contains("type alias Proc1Link"),
        "{error}"
    );
}

#[test]
fn unrepresentable_field_names_point_to_the_declaration() {
    let mut d = fixture();
    d.shm[0].name = "self".into();
    let error = generate(&d.shm_layout().unwrap(), &[]).unwrap_err();
    assert_eq!(
        error.to_string(),
        "SHM name \"self\" cannot be a Rust field; rename this region"
    );
}

#[test]
fn reserved_field_names_become_raw_identifiers() {
    for name in [
        "abstract", "become", "box", "final", "macro", "priv", "try", "yield",
    ] {
        assert_eq!(field_name(name).unwrap(), format!("r#{name}"));
    }
    assert_eq!(field_name("control").unwrap(), "control");
}

fn fragment(d: &Declaration) -> (String, Vec<u8>) {
    let shm = d.shm_layout().unwrap();
    let note =
        wire::descriptor_with_layout("fixture", "0.1.0", d, [0x45; 32], [0x67; 20], &shm).unwrap();
    (generate(&shm, &note).unwrap(), note)
}

fn write_case(dir: &Path, name: &str, source: String) -> std::path::PathBuf {
    let path = dir.join(format!("{name}.rs"));
    fs::write(
        &path,
        format!(
            "type Word = [u32; 4];\nmod types {{ pub type Word = super::Word; }}\n\
             type Doorbell1 = ::rp1_link::notify::Doorbell<27, 1>;\n{source}"
        ),
    )
    .unwrap();
    let snapshot = Path::new("tests/ui-link").join(format!("{name}.stderr"));
    if name != "pass" {
        let expected = fs::read_to_string(snapshot).unwrap();
        fs::write(
            path.with_extension("stderr"),
            expected.replace("$CASE", path.to_str().unwrap()),
        )
        .unwrap();
    }
    path
}

#[test]
fn generated_types_and_layout() {
    let d = fixture();
    let (body, note) = fragment(&d);
    let dir = tempfile::tempdir().unwrap();
    let t = trybuild::TestCases::new();
    let pass = include_str!("../../tests/ui-link/pass.inc");
    t.pass(write_case(
        dir.path(),
        "pass",
        format!("mod link {{\n{body}\nconst NOTE: &[u8] = &{note:?};\n{pass}"),
    ));

    let mut wrong_size = d.clone();
    wrong_size.types.get_mut("Word").unwrap().size = 24;
    let mut wrong_id = d.clone();
    wrong_id.types.get_mut("Word").unwrap().id ^= 1;
    for (name, declaration) in [("wrong_size", wrong_size), ("wrong_id", wrong_id)] {
        let (body, _) = fragment(&declaration);
        t.compile_fail(write_case(
            dir.path(),
            name,
            format!("mod link {{\n{body}}}\nfn main() {{}}"),
        ));
    }

    t.compile_fail(write_case(
        dir.path(),
        "open_twice",
        format!(
            "mod link {{\n{body}}}\n\
         fn reopen(init: link::Proc1LinkInit) {{\n\
             let _ = init.try_open();\n\
             let _ = init.try_open();\n\
         }}\n\
         fn main() {{}}",
        ),
    ));
}
