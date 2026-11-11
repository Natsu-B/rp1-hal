//! T6 (STATIC): the loader's ownership step on synthetic merged DTBs compiled by dtc.
use rp1_dt::fdt::Fdt;
use rp1_proto::note::{self, ownership_kind};
use rp1boot::{DTB_OUT_MAX, Ownership, enforce_linux_ownership, final_dtb};
use std::{
    io::Write,
    process::{Command, Stdio},
};

const RP1: &str = "/pcie@1000120000/rp1";
const UART3: &str = "&rp1_uart3 { status = \"okay\"; };";
const GPIOMEM: &str =
    "/ { gpiomem { compatible = \"raspberrypi,gpiomem\"; reg = <0xc0 0x400d0000 0x30000>; }; };";

/// Note v2 descriptor owning GPIO 8..11 and 22 and SPI0 (block 16).
fn descriptor() -> Vec<u8> {
    let mut bytes = vec![0; 32];
    fn record(bytes: &mut Vec<u8>, kind: u16, v: &[u8]) {
        bytes.extend(kind.to_le_bytes());
        bytes.extend(1u16.to_le_bytes());
        bytes.extend((v.len() as u32).to_le_bytes());
        bytes.extend(v);
        while !bytes.len().is_multiple_of(4) {
            bytes.push(0);
        }
    }
    let mut firmware = vec![1, b't', 1, b'1'];
    firmware.extend([0; 52]);
    record(&mut bytes, note::FIRMWARE, &firmware);
    record(&mut bytes, note::ENTRY, &[0; 20]);
    let entries = [
        (ownership_kind::PIN, 8u16),
        (ownership_kind::PIN, 9),
        (ownership_kind::PIN, 10),
        (ownership_kind::PIN, 11),
        (ownership_kind::PIN, 22),
        (ownership_kind::BLOCK, 16),
    ];
    let mut own = (entries.len() as u32).to_le_bytes().to_vec();
    for (kind, id) in entries {
        own.extend([kind, 1]);
        own.extend(id.to_le_bytes());
    }
    record(&mut bytes, note::OWNERSHIP, &own);
    record(&mut bytes, note::CLOCK, &[1, 0, 0, 0, 0]);
    record(&mut bytes, note::BOOT, &[0; 65]);
    bytes[..8].copy_from_slice(note::MAGIC);
    bytes[8..12].copy_from_slice(&[2, 0, 32, 0]);
    let len = bytes.len() as u32;
    bytes[12..16].copy_from_slice(&len.to_le_bytes());
    bytes[24] = 1;
    let crc = note::crc32(&bytes[32..]);
    bytes[20..24].copy_from_slice(&crc.to_le_bytes());
    note::Note::parse(&bytes).unwrap();
    bytes
}

/// The rp1-dt resolve fixture (merged DTB shape) plus `extra`, with this note and `policy`
/// properties in `/rp1-firmware`.
fn merged(extra: &str, policy: &[(&str, &str)]) -> (Fdt, Vec<u8>) {
    let source = format!(
        "{}\n{extra}\n",
        include_str!("../../../rp1-dt/tests/fixtures/resolve.dts")
    );
    let mut child = Command::new("dtc")
        .args(["-@", "-q", "-I", "dts", "-O", "dtb", "-o", "-", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("dtc required for synthetic DTB tests");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(source.as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let mut fdt = Fdt::parse(&out.stdout).unwrap();
    let note = descriptor();
    let firmware = fdt.find("/rp1-firmware").unwrap();
    fdt.set_prop(firmware, "rp1,note", note.clone());
    for (name, value) in policy {
        fdt.set_prop(firmware, name, format!("{value}\0").into_bytes());
    }
    (fdt, note)
}

fn applied(o: Ownership) -> Vec<String> {
    match o {
        Ownership::Applied { markers } => markers,
        Ownership::Refused { lines, reason } => panic!("refused: {reason} {lines:?}"),
    }
}

#[test]
fn clean_tree_logs_a_zero_count_and_is_unchanged() {
    let (mut fdt, note) = merged("", &[]);
    let before = fdt.clone();
    let markers = applied(enforce_linux_ownership(&before, &mut fdt, &note));
    assert_eq!(markers, ["RP1BOOT_LINUX_DISABLED_COUNT=0"]);
    assert_eq!(fdt.to_bytes(), before.to_bytes());
}

#[test]
fn default_policy_disables_marks_and_logs_conflicts_and_gpiomem() {
    let (mut fdt, note) = merged(&format!("{UART3}\n{GPIOMEM}"), &[]);
    let before = fdt.clone();
    let markers = applied(enforce_linux_ownership(&before, &mut fdt, &note));
    assert_eq!(markers.len(), 3, "{markers:?}");
    let uart = format!("RP1BOOT_LINUX_DISABLED path={RP1}/serial@3c000 label=rp1_uart3 why=");
    assert!(
        markers
            .iter()
            .any(|m| m.starts_with(&uart) && m.contains("GPIO8") && m.contains("GPIO9")),
        "{markers:?}"
    );
    assert!(
        markers.iter().any(
            |m| m.starts_with("RP1BOOT_LINUX_DISABLED path=/gpiomem label=- why=")
                && m.contains("gpiomem policy")
        ),
        "{markers:?}"
    );
    assert_eq!(markers[2], "RP1BOOT_LINUX_DISABLED_COUNT=2");
    for path in [format!("{RP1}/serial@3c000"), "/gpiomem".into()] {
        let n = fdt.find(&path).unwrap();
        assert_eq!(fdt.prop(n, "status"), Some(b"disabled\0".as_slice()));
        assert_eq!(fdt.prop(n, "rp1,loader-disabled"), Some([].as_slice()));
    }
    let firmware = fdt.find("/rp1-firmware").unwrap();
    let mut listed = fdt.strings(firmware, "rp1,loader-disabled");
    listed.sort();
    assert_eq!(
        listed,
        ["/gpiomem".to_string(), format!("{RP1}/serial@3c000")]
    );
    // What Linux gets parses back clean and is recorded.
    let linux = Fdt::parse(&final_dtb(&fdt).unwrap()).unwrap();
    assert!(
        rp1_dt::check::check(&before, &linux, &note)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn gpiomem_keep_policy_leaves_it_enabled() {
    let (mut fdt, note) = merged(GPIOMEM, &[("rp1,gpiomem", "keep")]);
    let before = fdt.clone();
    let markers = applied(enforce_linux_ownership(&before, &mut fdt, &note));
    assert_eq!(markers, ["RP1BOOT_LINUX_DISABLED_COUNT=0"]);
    assert!(fdt.enabled(fdt.find("/gpiomem").unwrap()));
}

#[test]
fn refuse_policy_and_protected_nodes_stop_without_changing_the_dtb() {
    for (extra, policy, why) in [
        (
            UART3.to_string(),
            &[("rp1,on-conflict", "refuse")][..],
            "on_conflict=refuse",
        ),
        (
            format!("{UART3} / {{ chosen {{ stdout-path = \"serial3:115200n8\"; }}; }};"),
            &[][..],
            "protected node",
        ),
    ] {
        let (mut fdt, note) = merged(&extra, policy);
        let before = fdt.clone();
        match enforce_linux_ownership(&before, &mut fdt, &note) {
            Ownership::Refused { lines, reason } => {
                assert!(
                    reason.contains(why) && reason.ends_with("RP1 not started"),
                    "{reason}"
                );
                assert!(!lines.is_empty());
                assert!(
                    lines
                        .iter()
                        .all(|l| l.starts_with("RP1BOOT_LINUX_REFUSED why=")),
                    "{lines:?}"
                );
                assert!(lines.iter().any(|l| l.contains("GPIO8")), "{lines:?}");
            }
            o => panic!("expected refusal, got {o:?}"),
        }
        assert_eq!(fdt.to_bytes(), before.to_bytes());
    }
}

#[test]
fn errors_stop_the_boot_without_changing_the_dtb() {
    // Unknown policy value.
    let (mut fdt, note) = merged(UART3, &[("rp1,on-conflict", "ignore")]);
    let before = fdt.clone();
    let (lines, reason) = refused(enforce_linux_ownership(&before, &mut fdt, &note));
    assert_eq!(lines.len(), 1, "{lines:?}");
    assert!(
        lines[0].starts_with("RP1BOOT_LINUX_REFUSED why=") && lines[0].contains("policy"),
        "{lines:?}"
    );
    assert!(reason.ends_with("RP1 not started"), "{reason}");
    assert_eq!(fdt.to_bytes(), before.to_bytes());
    // The overlay handed a reserved GPIO back to Linux (recheck against the firmware DTB).
    let (mut fdt, note) = merged(UART3, &[]);
    let mut firmware_dtb = fdt.clone();
    let gpio = firmware_dtb.label("rp1_gpio").unwrap();
    firmware_dtb.set_prop(
        gpio,
        "gpio-reserved-ranges",
        [8u32, 4, 22, 2]
            .into_iter()
            .flat_map(u32::to_be_bytes)
            .collect(),
    );
    let original = fdt.to_bytes();
    let (lines, reason) = refused(enforce_linux_ownership(&firmware_dtb, &mut fdt, &note));
    assert!(
        lines.len() == 1
            && lines[0].starts_with("RP1BOOT_LINUX_REFUSED why=")
            && lines[0].contains("back to Linux"),
        "{lines:?}"
    );
    assert!(reason.contains("back to Linux"), "{reason}");
    assert_eq!(fdt.to_bytes(), original);
}

#[test]
fn markers_stay_on_one_line() {
    let (mut fdt, note) = merged(UART3, &[]);
    // A label (the __symbols__ property name) with a control character; it sorts first.
    let symbols = fdt.find("/__symbols__").unwrap();
    fdt.set_prop(
        symbols,
        "rp1_\nX",
        format!("{RP1}/serial@3c000\0").into_bytes(),
    );
    let before = fdt.clone();
    let markers = applied(enforce_linux_ownership(&before, &mut fdt, &note));
    assert!(markers[0].contains("label=rp1_?X "), "{markers:?}");
    assert!(markers.iter().all(|m| !m.chars().any(char::is_control)));
}

/// Marker growth is bounded by the final-DTB window, and the window is enforced.
#[test]
fn marker_growth_fits_the_final_dtb_window_and_oversize_is_refused() {
    let (mut fdt, note) = merged(&format!("{UART3}\n{GPIOMEM}"), &[]);
    let before = fdt.clone();
    let size = before.to_bytes().len();
    applied(enforce_linux_ownership(&before, &mut fdt, &note));
    let grown = final_dtb(&fdt).unwrap().len();
    // Per node: status (new or rewritten, <= 12 B header + 12 B "disabled") + empty marker
    // (12 B); the path list (12 B header + NUL-terminated paths, padded) and one new
    // string-table entry for the property name.
    let paths = format!("{RP1}/serial@3c000").len() + 1 + "/gpiomem".len() + 1;
    let bound = 2 * (24 + 12) + 12 + paths.next_multiple_of(4) + "rp1,loader-disabled\0".len();
    assert!(grown > size && grown - size <= bound, "{size} -> {grown}");

    // A firmware DTB that fits, but not with the markers added, stops the boot.
    let pad = DTB_OUT_MAX as usize - size - 12 - "rp1,pad\0".len() - (grown - size) + 4;
    let (mut fdt, note) = merged(&format!("{UART3}\n{GPIOMEM}"), &[]);
    let root = 0;
    fdt.set_prop(root, "rp1,pad", vec![0; pad.next_multiple_of(4)]);
    assert!(final_dtb(&fdt).is_ok());
    let before = fdt.clone();
    applied(enforce_linux_ownership(&before, &mut fdt, &note));
    let e = final_dtb(&fdt).unwrap_err();
    assert!(
        e.starts_with("final DTB is ") && e.ends_with("(> 0x200000)"),
        "{e}"
    );
}

/// A real merged board DTB (firmware DTB + config.txt overlays + `.rp1.dtbo`), when given
/// (not committed; the committed stock-tree cases are in `rp1-build/tests/stock_dtb.rs`).
/// rp1boot must apply its plan, print one marker per disabled node and the count, and keep the
/// final DTB in the window. `RP1BOOT_TEST_EXPECT_DISABLED` (comma-separated node paths), when
/// set, must be exactly the disabled set.
#[test]
fn real_merged_dtb() {
    let Ok(path) = std::env::var("RP1BOOT_TEST_MERGED_DTB") else {
        return;
    };
    let mut fdt = Fdt::parse(&std::fs::read(path).unwrap()).unwrap();
    let firmware = fdt.find("/rp1-firmware").unwrap();
    let note = fdt.prop(firmware, "rp1,note").unwrap().to_vec();
    let before = fdt.clone();
    let markers = match enforce_linux_ownership(&before, &mut fdt, &note) {
        Ownership::Applied { markers } => markers,
        Ownership::Refused { lines, reason } => panic!("refused: {lines:?} {reason}"),
    };
    markers.iter().for_each(|m| eprintln!("{m}"));
    let (count, nodes) = markers.split_last().unwrap();
    assert_eq!(
        *count,
        format!("RP1BOOT_LINUX_DISABLED_COUNT={}", nodes.len())
    );
    let mut paths: Vec<&str> = nodes
        .iter()
        .map(|m| {
            m.strip_prefix("RP1BOOT_LINUX_DISABLED path=")
                .and_then(|m| m.split(' ').next())
                .unwrap_or_else(|| panic!("bad marker {m}"))
        })
        .collect();
    for p in &paths {
        let n = fdt.find(p).unwrap();
        assert!(!fdt.enabled(n), "{p} still enabled");
    }
    if let Ok(expected) = std::env::var("RP1BOOT_TEST_EXPECT_DISABLED") {
        let mut expected: Vec<&str> = expected.split(',').filter(|p| !p.is_empty()).collect();
        expected.sort();
        paths.sort();
        assert_eq!(paths, expected);
    }
    let size = final_dtb(&fdt).unwrap().len();
    assert!(size as u64 <= DTB_OUT_MAX, "{size}");
    eprintln!(
        "final DTB {} -> {size} B (max {DTB_OUT_MAX})",
        before.to_bytes().len()
    );
}

fn refused(o: Ownership) -> (Vec<String>, String) {
    match o {
        Ownership::Refused { lines, reason } => (lines, reason),
        o => panic!("expected refusal, got {o:?}"),
    }
}
