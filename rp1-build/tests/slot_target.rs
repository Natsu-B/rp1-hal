//! Check the same Slot trait fixtures as the host on the actual firmware target.
use std::{fs, path::Path, process::Command};

#[test]
fn thumb_slot_requires_send_and_accepts_a_uart_half() {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let project = tempfile::tempdir().unwrap();
    let manifest = project.path().join("Cargo.toml");
    fs::write(
        &manifest,
        format!(
            "[package]\nname = 'slot-target-ui'\nversion = '0.0.0'\nedition = '2024'\n\
             [lib]\npath = 'fixture.rs'\n\
             [dependencies]\nrp1-rt = {{ path = {:?} }}\nrp1-hal = {{ path = {:?} }}\n",
            workspace.join("rp1-rt"),
            workspace.join("rp1-hal")
        ),
    )
    .unwrap();
    fs::copy(
        workspace.join("Cargo.lock"),
        project.path().join("Cargo.lock"),
    )
    .unwrap();
    let check = |fixture: &str| {
        fs::copy(workspace.join(fixture), project.path().join("fixture.rs")).unwrap();
        Command::new(env!("CARGO"))
            .args([
                "check",
                "--offline",
                "--target",
                "thumbv7m-none-eabi",
                "--manifest-path",
            ])
            .arg(&manifest)
            .env("CARGO_TARGET_DIR", project.path().join("target"))
            .output()
            .unwrap()
    };

    let pass = check("rp1-hal/tests/pass/slot_uart_half.rs");
    assert!(
        pass.status.success(),
        "{}",
        String::from_utf8_lossy(&pass.stderr)
    );
    let fail = check("rp1-rt/tests/ui/slot_cell_ref.rs");
    let errors = String::from_utf8_lossy(&fail.stderr);
    assert!(
        !fail.status.success(),
        "static Slot accepted a Cell reference"
    );
    assert!(
        errors.contains("Cell<u32>` cannot be shared between threads safely"),
        "{errors}"
    );
    assert!(
        errors.contains("required for `Slot<Option<&'static Cell<u32>>, 0>` to implement `Sync`"),
        "{errors}"
    );
}
