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
        format!("type Word = [u32; 4];\nmod types {{ pub type Word = super::Word; }}\n{source}"),
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
