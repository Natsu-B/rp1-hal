use rp1_build::{
    generate,
    metadata::{Declaration, TypeInfo},
};
use rp1_proto::note::{ENTRY, Note};
use std::{fs, path::Path, process::Command};

fn declaration() -> Declaration {
    toml::from_str(
        r#"
schema=1
board="rpi-cm5"
log={sink="uart0",baudrate=115200}
[proc0]
pins=[22]
peripherals=["TIMER.ALARM0"]
[proc1]
pins=[27]
gpio_irq=[27]
peripherals=["TIMER.ALARM1"]
"#,
    )
    .unwrap()
}
#[test]
fn rejects_mistakes_before_building() {
    let mut d = declaration();
    d.validate().unwrap();
    d.proc0.pins.push(14);
    assert!(d.validate().is_err());
    d.proc0.pins.pop();
    // The log owns UART0: no driver may also take it (review #6).
    d.proc0.peripherals.push("UART0".into());
    assert!(d.validate().is_err());
    d.proc0.peripherals.pop();
    d.proc1.peripherals.push("TIMER.ALARM0".into());
    assert!(d.validate().is_err());
    d.proc1.peripherals.pop();
    d.proc1.gpio_irq.push(26);
    assert!(d.validate().is_err());
    d.proc1.gpio_irq.pop();
    // Review 3 #5: a second spelling of a declared block, on the other core or the same one.
    for (core, alias) in [
        (1, "TIMER.ALARM00"),
        (0, "TIMER.ALARM00"),
        (1, "TIMER.ALARM+0"),
        (1, "UART00"),
        (1, "UART+0"),
    ] {
        let c = if core == 0 {
            &mut d.proc0
        } else {
            &mut d.proc1
        };
        c.peripherals.push(alias.into());
        assert!(d.validate().is_err(), "{alias}");
        assert!(
            generate("test", "0.1.0", &d, [0; 32], [0; 20]).is_err(),
            "{alias}"
        );
        let c = if core == 0 {
            &mut d.proc0
        } else {
            &mut d.proc1
        };
        c.peripherals.pop();
    }
    // Non-canonical names are rejected even without a clash.
    for alias in ["TIMER.ALARM03", "UART05", "UART+5", "I2C01"] {
        assert!(
            rp1_build::metadata::peripheral_type(alias).is_err(),
            "{alias}"
        );
    }
    for name in ["TIMER.ALARM3", "UART5", "I2C1"] {
        rp1_build::metadata::peripheral_type(name).unwrap();
    }
    d.clock.authority = "linux".into();
    assert!(d.validate().is_err());
    assert!(toml::from_str::<Declaration>("typo=1").is_err());
}
#[test]
fn i2c_ownership_and_board_management_pins() {
    use rp1_build::metadata::{peripheral_id, peripheral_type};
    for n in 0..4 {
        assert_eq!(peripheral_id(&format!("I2C{n}")).unwrap(), (2, 32 + n));
    }
    for p in ["I2C4", "I2C5", "I2C6", "I2C7", "I2C01"] {
        assert!(peripheral_type(p).is_err());
    }
    let mut d = declaration();
    d.proc0.pins.extend([2, 3]);
    assert!(d.validate().is_err());
    d.proc1.peripherals.push("I2C1".into());
    assert!(
        d.validate().is_err(),
        "a different core cannot unlock GPIO2/3"
    );
    d.proc1.peripherals.clear();
    d.proc0.peripherals.push("I2C1".into());
    d.validate().unwrap();
    let a = generate("i2c", "0.1.0", &d, [0; 32], [0; 20]).unwrap();
    assert!(a.dts.contains("target = <&rp1_i2c1>"));
    let note = Note::parse(&a.note).unwrap();
    assert!(
        note.get(rp1_proto::note::OWNERSHIP).unwrap().value[4..]
            .as_chunks::<4>()
            .0
            .contains(&[2, 1, 33, 0])
    );
    d.proc1.peripherals.push("I2C1".into());
    assert!(d.validate().is_err());
    d.proc1.peripherals.clear();
    d.proc0.pins.push(0);
    assert!(d.validate().is_err(), "Linux UART1 retains GPIO0/1");
}
#[test]
fn adc_ownership_disables_linux_adc_and_keeps_its_reset() {
    use rp1_build::metadata::{BOARD_LINUX_RESETS, peripheral_id, peripheral_type};
    assert_eq!(peripheral_id("ADC").unwrap(), (2, 64));
    assert_eq!(peripheral_type("ADC").unwrap(), "::rp1_hal::adc::Token");
    assert!(peripheral_type("ADC0").is_err());
    let mut d = declaration();
    let a = generate("plain", "0.1.0", &d, [0; 32], [0; 20]).unwrap();
    assert!(!a.dts.contains("<&rp1_adc>; __overlay__ { status"));
    assert_eq!(d.linux_resets()[0] & 1, 1);
    d.proc0.peripherals.push("ADC".into());
    d.validate().unwrap();
    assert_eq!(d.linux_resets()[0], BOARD_LINUX_RESETS[0] & !1);
    let a = generate("adc", "0.1.0", &d, [0; 32], [0; 20]).unwrap();
    assert!(
        a.dts
            .contains("target = <&rp1_adc>; __overlay__ { status = \"disabled\"; }")
    );
    let note = Note::parse(&a.note).unwrap();
    assert!(
        note.get(rp1_proto::note::OWNERSHIP).unwrap().value[4..]
            .as_chunks::<4>()
            .0
            .contains(&[2, 1, 64, 0])
    );
    d.proc1.peripherals.push("ADC".into());
    assert!(d.validate().is_err(), "ADC claimed twice");
}
/// Review #7/#9: declarations v0.1 cannot deliver are build errors, not silent no-ops.
#[test]
fn rejects_unsupported_clock_authority_and_notify() {
    let mut d: Declaration = toml::from_str(
        r#"
[clock]
authority="linux"
[proc0]
pins=[22]
"#,
    )
    .unwrap();
    assert!(d.log.is_none() && d.proc0.peripherals.is_empty());
    assert!(
        d.validate().is_err(),
        "GPIO-only firmware must not take authority=linux"
    );
    d.clock.authority = "rp1".into();
    d.validate().unwrap();
    for (notify, ok) in [("poll", true), ("irq", false), ("doorbell", false)] {
        let d: Declaration = toml::from_str(&format!(
            r#"
[[shm]]
name="up"
kind="ring"
core="proc1"
message="u32"
direction="rp1-to-linux"
depth=16
notify="{notify}"
"#
        ))
        .unwrap();
        assert_eq!(d.validate().is_ok(), ok, "notify={notify}");
    }
}
#[test]
fn typed_regions_and_sram_budget() {
    let mut d: Declaration = toml::from_str(
        r#"
[[shm]]
name="commands"
kind="ring"
core="proc1"
message="u32"
direction="linux-to-rp1"
depth=16
"#,
    )
    .unwrap();
    d.validate().unwrap();
    assert!(d.regions().is_err());
    d.types.insert("u32".into(), TypeInfo::of::<u32>());
    let r = d.regions().unwrap();
    assert_eq!(r[0].sram_len, 384);
    assert_eq!(r[0].ddr_offset, 65536);
    let a = generate("test", "0.1.0", &d, [0; 32], [0; 20]).unwrap();
    assert!(Note::parse(&a.note).is_ok());
    let area = rp1_proto::note::ShmArea::parse(
        Note::parse(&a.note)
            .unwrap()
            .get(rp1_proto::note::SHM_AREA)
            .unwrap()
            .value,
    )
    .unwrap();
    assert_eq!((area.phys, area.size), (0x2200_0000, 0x10_0000));
    d.shm[0].depth = Some(4096);
    assert!(d.regions().is_err());
    d.shm[0].placement = Some("linux".into());
    assert!(d.validate().is_err());
}
#[test]
fn note_is_deterministic_and_rejects_corruption() {
    let d = declaration();
    let a = generate("test", "0.1.0", &d, [1; 32], [2; 20]).unwrap();
    assert_eq!(
        a.note,
        generate("test", "0.1.0", &d, [1; 32], [2; 20])
            .unwrap()
            .note
    );
    let n = Note::parse(&a.note).unwrap();
    assert_eq!(
        &n.get(ENTRY).unwrap().value[..4],
        &0x2000_0401u32.to_le_bytes()
    );
    let mut bad = a.note;
    bad[40] ^= 1;
    assert!(Note::parse(&bad).is_err());
}
#[test]
fn large_block_and_malformed_region() {
    let mut d: Declaration = toml::from_str(
        r#"
[[shm]]
name="bulk"
kind="block"
core="proc1"
type="Bulk"
writer="both"
"#,
    )
    .unwrap();
    d.types
        .insert("Bulk".into(), TypeInfo::of::<[u32; 262144]>());
    let mut a = generate("test", "0.1.0", &d, [0; 32], [0; 20]).unwrap();
    let n = Note::parse(&a.note).unwrap();
    let region = n.get(rp1_proto::note::SHM_REGION).unwrap().value;
    let at = region.as_ptr() as usize - a.note.as_ptr() as usize;
    assert_eq!(d.ddr_size().unwrap(), 0x20_0000);
    // A valid CRC cannot hide an allocation that overlaps the area header.
    a.note[at + 8..at + 12].copy_from_slice(&0u32.to_le_bytes());
    let crc = rp1_proto::note::crc32(&a.note[32..]);
    a.note[20..24].copy_from_slice(&crc.to_le_bytes());
    assert!(Note::parse(&a.note).is_err());
}
/// The stock board DTB as the firmware hands it over: `bcm2712-rpi-cm5l-cm5io.dtb` (what the
/// firmware loads for a CM5 Lite on the CM5 IO board), plus `overlays/bcm2712d0.dtbo` when
/// `RP1_TEST_DTB_OVERLAY` names it. Skips (returns `None`) when `RP1_TEST_DTB` is unset.
/// Review 3 #4 without a board DTB: every consumer whose assigned clock moves to SCMI also has
/// its stock assigned parent (audio_out: clk-rp1 PLL_AUDIO) replaced by a null phandle.
#[test]
fn scmi_assigned_clocks_drop_the_clk_rp1_parent() {
    let a = generate("test", "0.1.0", &declaration(), [0; 32], [0; 20]).unwrap();
    for (label, id) in rp1_build::overlay::BOARD_SCMI_ASSIGNED {
        let fragment = format!(
            "target = <&{label}>; __overlay__ {{ assigned-clocks = <&rp1_scmi_clocks {id}>; assigned-clock-parents = <0>; }}"
        );
        assert!(a.dts.contains(&fragment), "{label}");
    }
    assert!(
        rp1_build::overlay::BOARD_SCMI_ASSIGNED
            .iter()
            .any(|(l, _)| *l == "rp1_audio_out")
    );
}

fn stock_board_dtb(dir: &Path) -> Option<std::path::PathBuf> {
    let Some(base) = std::env::var_os("RP1_TEST_DTB").map(std::path::PathBuf::from) else {
        eprintln!("STATIC fixture unavailable: set RP1_TEST_DTB for integration check");
        return None;
    };
    assert!(
        base.is_file(),
        "RP1_TEST_DTB is not a file: {}",
        base.display()
    );
    let copy = dir.join("stock.dtb");
    fs::write(&copy, fs::read(&base).unwrap()).unwrap();
    if let Some(d0) = std::env::var_os("RP1_TEST_DTB_OVERLAY") {
        let out = dir.join("stock-d0.dtb");
        rp1_build::overlay::apply(&copy, Path::new(&d0), &out).unwrap();
        return Some(out);
    }
    Some(copy)
}

#[test]
fn overlay_applies_to_a_copy_of_the_stock_board_dtb() {
    let tmp = tempfile::tempdir().unwrap();
    let Some(copy) = stock_board_dtb(tmp.path()) else {
        return;
    };
    let before = fs::read(&copy).unwrap();
    let source = tmp.path().join("app.dts");
    let overlay = tmp.path().join("app.dtbo");
    let merged = tmp.path().join("merged.dtb");
    let a = generate("test", "0.1.0", &declaration(), [0; 32], [0; 20]).unwrap();
    fs::write(&source, &a.dts).unwrap();
    rp1_build::overlay::compile(&source, &overlay).unwrap();
    rp1_build::overlay::apply(&copy, &overlay, &merged).unwrap();
    let decompile = |p: &Path| {
        let out = Command::new("dtc")
            .args(["-q", "-s", "-I", "dtb", "-O", "dts"])
            .arg(p)
            .output()
            .unwrap();
        assert!(out.status.success());
        String::from_utf8(out.stdout).unwrap()
    };
    // The loader's pure-Rust apply gives the same tree as libfdt's fdtoverlay.
    let mut fdt = rp1_build::fdt::Fdt::parse(&before).unwrap();
    rp1_dt::overlay::apply(&mut fdt, &fs::read(&overlay).unwrap()).unwrap();
    let ours = tmp.path().join("ours.dtb");
    fs::write(&ours, fdt.to_bytes()).unwrap();
    let dts = decompile(&merged);
    assert_eq!(decompile(&ours), dts, "rp1-dt differs from fdtoverlay");
    assert!(dts.contains("rp1:link"));
    // rp1-mailbox stays bound (SCMI transport); no UIO node takes an interrupt.
    let base_dts = decompile(&copy);
    let mbox = |dts: &str| {
        dts.split("mailbox@8000 {")
            .nth(1)
            .unwrap()
            .split("};")
            .next()
            .unwrap()
            .to_string()
    };
    assert_eq!(
        mbox(&dts),
        mbox(&base_dts),
        "overlay must not touch rp1-mailbox"
    );
    assert!(!dts.contains("rp1:events"));
    for uio in dts.split("\"generic-uio\"").skip(1) {
        let body = &uio[..uio.find("};").unwrap()];
        assert!(
            !body.contains("interrupts"),
            "UIO node with interrupts: {body}"
        );
    }
    // Linux never gains pins: the merged reservation is a superset of the base one.
    let base_pins = rp1_build::overlay::reserved_gpios(&copy).unwrap();
    let merged_pins = rp1_build::overlay::reserved_gpios(&merged).unwrap();
    assert!(merged_pins.is_superset(&base_pins));
    assert!([14, 15, 22, 27].iter().all(|p| merged_pins.contains(p)));
    assert!(!merged_pins.contains(&0) && !merged_pins.contains(&1));

    let m = rp1_build::fdt::Fdt::parse(&fs::read(&merged).unwrap()).unwrap();
    let label = |l: &str| m.label(l).unwrap_or_else(|| panic!("no label {l}"));
    // Root and /reserved-memory are <2 1>: regs are 3 cells.
    assert_eq!(
        m.reg(m.find("/rp1-link@1f0040c000").unwrap()).unwrap(),
        [(0x1f_0040_c000, 0x4000)]
    );
    assert_eq!(
        m.reg(m.find("/reserved-memory/rp1-shm@22000000").unwrap())
            .unwrap(),
        [(0x2200_0000, 0x10_0000)]
    );
    // SCMI over rp1-mailbox channel 0 + shmem in shared SRAM, added by the overlay itself.
    let scmi = m.find("/firmware/scmi").expect("no /firmware/scmi");
    assert!(m.enabled(scmi));
    assert_eq!(m.strings(scmi, "compatible"), ["arm,scmi"]);
    assert_eq!(
        m.cells(scmi, "mboxes").unwrap(),
        [m.phandle_of(label("rp1_mbox")).unwrap(), 0]
    );
    let shmem = m.phandle(m.cells(scmi, "shmem").unwrap()[0]).unwrap();
    assert_eq!(m.strings(shmem, "compatible"), ["arm,scmi-shmem"]);
    assert_eq!(
        m.path(shmem),
        "/axi/pcie@1000120000/rp1/sram@400000/scmi@d7c0"
    );
    assert_eq!(m.cells(shmem, "reg").unwrap(), [0xd7c0, 0x100]);
    let clocks = m.find("/firmware/scmi/protocol@14").unwrap();
    assert_eq!(label("rp1_scmi_clocks"), clocks);
    // Every RP1 UART takes uartclk/apb_pclk from SCMI (ids 1 and 0), none from clk-rp1.
    let scmi_clk = m.phandle_of(clocks).unwrap();
    for uart in 0..=5 {
        let n = label(&format!("rp1_uart{uart}"));
        assert_eq!(m.cells(n, "clocks").unwrap(), [scmi_clk, 1, scmi_clk, 0]);
        assert_eq!(m.strings(n, "clock-names"), ["uartclk", "apb_pclk"]);
    }
    // clk-rp1, the stock RP1 firmware client (same mailbox channel) and PIO (needs that
    // client) are off.
    for l in ["rp1_clocks", "rp1_firmware", "rp1_fw_shmem"]
        .into_iter()
        .chain(
            rp1_build::overlay::BOARD_NEEDS_STOCK_FIRMWARE
                .iter()
                .copied(),
        )
    {
        assert!(!m.enabled(label(l)), "{l} still enabled");
    }
    // The blocks Linux owns stay enabled as in the stock DT, with their clocks on SCMI.
    for l in [
        "rp1_eth", "rp1_dma", "rp1_adc", "rp1_usb0", "rp1_usb1", "led_pwr", "cam0_reg",
    ] {
        assert!(m.enabled(label(l)), "{l} disabled");
    }
    for (l, ids) in rp1_build::overlay::BOARD_SCMI_CONSUMERS {
        let cells: Vec<u32> = ids.iter().flat_map(|i| [scmi_clk, *i]).collect();
        assert_eq!(m.cells(label(l), "clocks").unwrap(), cells, "{l}");
    }
    assert_eq!(
        m.cells(label("rp1_eth"), "clocks").unwrap(),
        [scmi_clk, 2, scmi_clk, 2, scmi_clk, 6, scmi_clk, 5]
    );
    assert_eq!(
        m.cells(label("rp1_pwm0"), "assigned-clocks").unwrap(),
        [scmi_clk, 8]
    );
    // Ethernet's PHY reset/IRQ and USB VBUS lines (32, 37, 42) are Linux's.
    assert!(![32, 37, 42].iter().any(|p| merged_pins.contains(p)));
    // Phandles of a clock specifier list (`clocks`, `assigned-clock*`); null entries have no
    // argument cells.
    let providers = |cells: &[u32]| {
        let mut out = Vec::new();
        let mut i = 0;
        while i < cells.len() {
            let ph = cells[i];
            out.push(ph);
            let args = if ph == 0 {
                0
            } else {
                let n = m
                    .phandle(ph)
                    .unwrap_or_else(|| panic!("dangling phandle {ph:#x}"));
                m.cells(n, "#clock-cells")
                    .unwrap_or_else(|| panic!("{} has no #clock-cells", m.path(n)))[0]
                    as usize
            };
            i += 1 + args;
        }
        out
    };
    const CLOCK_PROPS: [&str; 3] = ["clocks", "assigned-clocks", "assigned-clock-parents"];
    // Review 3 #4: a consumer moved to SCMI (enabled or not; audio_out is off in the stock DT)
    // keeps no reference to clk-rp1 in any clock property.
    let rp1_clk = m.phandle_of(label("rp1_clocks")).unwrap();
    let moved = rp1_build::overlay::BOARD_SCMI_CONSUMERS
        .iter()
        .map(|(l, _)| *l)
        .chain(
            rp1_build::overlay::BOARD_SCMI_ASSIGNED
                .iter()
                .map(|(l, _)| *l),
        );
    for l in moved {
        for prop in CLOCK_PROPS {
            if let Some(c) = m.cells(label(l), prop) {
                assert!(!providers(&c).contains(&rp1_clk), "{l} {prop} uses clk-rp1");
            }
        }
    }
    assert_eq!(
        m.cells(label("rp1_audio_out"), "assigned-clocks").unwrap(),
        [scmi_clk, 10]
    );
    assert_eq!(
        m.cells(label("rp1_audio_out"), "assigned-clock-parents")
            .unwrap(),
        [0]
    );
    // No enabled node outside the unserved video/SDIO set still uses clk-rp1.
    for (i, _) in m.nodes.iter().enumerate() {
        // Label tables hold path strings under the same property names.
        let path = m.path(i);
        if !m.enabled(i)
            || i == label("rp1_clocks")
            || path.starts_with("/__")
            || path == "/aliases"
        {
            continue;
        }
        for prop in CLOCK_PROPS {
            if let Some(c) = m.cells(i, prop) {
                assert!(
                    !providers(&c).contains(&rp1_clk),
                    "{} {prop} uses clk-rp1",
                    m.path(i)
                );
            }
        }
    }
    assert_eq!(m.cells(label("pcie2"), "brcm,tperst-clk-ms").unwrap(), [20]);
    // No OpenAMP in the stock DT: the overlay leaves only an inert disabled stub.
    let oa = m.find("/openamp-address-space").unwrap();
    assert!(!m.enabled(oa) && m.prop(oa, "compatible").is_none());
    assert!(m.nodes[oa].children.is_empty());
    let out = Command::new("dtc")
        .args(["-I", "dtb", "-O", "dts"])
        .arg(&overlay)
        .output()
        .unwrap();
    assert!(
        String::from_utf8(out.stdout)
            .unwrap()
            .contains("__fixups__")
    );
    // The stock board DT has no ownership/pinctrl/reservation conflict (review #3), and the
    // loader's check (pins handed back + final conflicts) passes too.
    assert_eq!(
        rp1_build::overlay::final_conflicts(&fs::read(&merged).unwrap(), &a.note).unwrap(),
        Vec::<String>::new()
    );
    assert_eq!(
        rp1_dt::check::check(&rp1_build::fdt::Fdt::parse(&before).unwrap(), &m, &a.note).unwrap(),
        Vec::<String>::new()
    );
    assert_eq!(before, fs::read(&copy).unwrap());
}

/// Contents of every watched file (directories recursively), as Cargo's
/// `rerun-if-changed` would see them.
fn snapshot(paths: &[std::path::PathBuf]) -> Vec<(std::path::PathBuf, Vec<u8>)> {
    fn walk(p: &Path, out: &mut Vec<(std::path::PathBuf, Vec<u8>)>) {
        if p.is_dir() {
            let mut entries: Vec<_> = fs::read_dir(p)
                .unwrap()
                .map(|e| e.unwrap().path())
                .collect();
            entries.sort();
            for e in entries {
                walk(&e, out);
            }
        } else if let Ok(b) = fs::read(p) {
            out.push((p.to_path_buf(), b));
        }
    }
    let mut out = Vec::new();
    for p in paths {
        walk(p, &mut out);
    }
    out
}

#[test]
fn branch_commits_change_watched_git_paths() {
    // REVIEW2 #6: on a branch a commit rewrites the ref, not HEAD.
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();
    let git = |args: &[&str]| {
        let st = Command::new("git")
            .args([
                "-c",
                "user.name=t",
                "-c",
                "user.email=t@t",
                "-c",
                "commit.gpgsign=false",
            ])
            .args(args)
            .current_dir(dir)
            .output()
            .unwrap();
        assert!(
            st.status.success(),
            "{args:?}: {}",
            String::from_utf8_lossy(&st.stderr)
        );
    };
    git(&["init", "-q", "-b", "feature/x"]);
    let mut n = 0;
    let mut commit = || {
        n += 1;
        fs::write(dir.join("main.rs"), format!("// {n}\n")).unwrap();
        git(&["add", "main.rs"]);
        git(&["commit", "-q", "-m", "c"]);
    };
    commit();
    for pack in [false, true] {
        if pack {
            git(&["pack-refs", "--all"]);
        }
        let paths = rp1_build::git_watch_paths(dir);
        let before = snapshot(&paths);
        commit();
        assert_ne!(before, snapshot(&paths), "pack={pack} paths={paths:?}");
    }
}
