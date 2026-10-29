//! Repository rule (AGENTS.md): no tracked file may point at places that exist only on the
//! developer's machine (lab tree, home directory, run logs, artifacts, TFTP root, backups).
//! Evidence is cited generically ("HW, 2026-10-29, cold boot ×2"); lab tools take locations
//! from environment variables or arguments.

use std::path::Path;
use std::process::Command;

/// Forbidden substrings. Built with `concat!` so this file does not match itself.
const FORBIDDEN: &[&str] = &[
    concat!("/opt/", "rpi-cm5-hack"),
    concat!("/home", "/"),
    concat!("/Users", "/"),
    concat!("~/", "TODO"),
    concat!("tftp", "-backup"),
    concat!("tftp", "boot"),
    concat!("deploy/", "tftp"),
    concat!("artifacts/", "20"),
    concat!("/dev/", "cm5-"),
    concat!("/dev/", "esp32-"),
    concat!("/tmp/", "codex"),
    concat!("HAND", "OFF-2"),
    concat!("CURRENT_", "STATUS"),
];

/// `prefix` followed by at least `digits` ASCII digits (a dated log directory, a numbered run ID).
const FORBIDDEN_NUMBERED: &[(&str, usize)] =
    &[(concat!("logs/", "20"), 6), (concat!("hw", "run"), 1)];

fn hits(line: &str) -> Vec<&'static str> {
    let mut out: Vec<&'static str> = FORBIDDEN
        .iter()
        .copied()
        .filter(|p| line.contains(p))
        .collect();
    for &(prefix, digits) in FORBIDDEN_NUMBERED {
        let numbered = line.match_indices(prefix).any(|(at, _)| {
            let rest = &line.as_bytes()[at + prefix.len()..];
            rest.len() >= digits && rest[..digits].iter().all(u8::is_ascii_digit)
        });
        if numbered {
            out.push(prefix);
        }
    }
    out
}

#[test]
fn tracked_files_contain_no_host_local_paths() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let Ok(listed) = Command::new("git")
        .arg("-C")
        .arg(&root)
        .args(["ls-files", "-z"])
        .output()
    else {
        eprintln!("git unavailable: local-path check skipped");
        return;
    };
    if !listed.status.success() {
        eprintln!("not a git checkout: local-path check skipped");
        return;
    }
    let mut bad = Vec::new();
    for name in listed.stdout.split(|&b| b == 0).filter(|n| !n.is_empty()) {
        let name = String::from_utf8_lossy(name);
        let Ok(text) = std::fs::read_to_string(root.join(&*name)) else {
            continue; // binary or deleted in the work tree
        };
        for (n, line) in text.lines().enumerate() {
            for p in hits(line) {
                bad.push(format!("{name}:{}: `{p}`: {}", n + 1, line.trim()));
            }
        }
    }
    assert!(
        bad.is_empty(),
        "host-local paths in tracked files:\n{}",
        bad.join("\n")
    );
}

#[test]
fn detector_matches_examples() {
    assert!(!hits(concat!("see /opt/", "rpi-cm5-hack/x")).is_empty());
    assert!(!hits(concat!("logs/", "20261029-0415")).is_empty());
    assert!(!hits(concat!("hw", "run5 06")).is_empty());
    assert!(hits("logs/ directory, hwrun, HW 2026-10-29").is_empty());
}
