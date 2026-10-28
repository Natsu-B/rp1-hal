use crate::{Result, metadata::Declaration};
use std::{path::Path, process::Command};

pub fn compile(source: &Path, destination: &Path) -> Result<()> {
    let output = Command::new(std::env::var_os("RP1_DTC").unwrap_or_else(|| "dtc".into()))
        .args(["-@", "-I", "dts", "-O", "dtb", "-o"])
        .arg(destination)
        .arg(source)
        .output()?;
    if !output.status.success() {
        return Err(format!("dtc failed: {}", String::from_utf8_lossy(&output.stderr)).into());
    }
    Ok(())
}
pub fn apply(base: &Path, overlay: &Path, destination: &Path) -> Result<()> {
    if base == destination {
        return Err("base DTB must remain read-only; choose another output".into());
    }
    let output = Command::new("fdtoverlay")
        .arg("-i")
        .arg(base)
        .arg("-o")
        .arg(destination)
        .arg(overlay)
        .output()?;
    if !output.status.success() {
        return Err(format!(
            "fdtoverlay failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    Ok(())
}
/// RP1 GPIO numbers that `dtb` withholds from Linux (`gpio-reserved-ranges` of the node
/// labelled `rp1_gpio`), read with the host `fdtget`.
pub fn reserved_gpios(dtb: &Path) -> Result<std::collections::BTreeSet<u32>> {
    let fdtget = |args: &[&str]| -> Result<String> {
        let out = Command::new("fdtget").arg(dtb).args(args).output()?;
        if !out.status.success() {
            return Err(format!("fdtget failed: {}", String::from_utf8_lossy(&out.stderr)).into());
        }
        Ok(String::from_utf8(out.stdout)?.trim().to_string())
    };
    let path = fdtget(&["/__symbols__", "rp1_gpio"])?;
    let cells = match fdtget(&["-t", "u", &path, "gpio-reserved-ranges"]) {
        Ok(cells) => cells,
        Err(_) => return Ok(Default::default()),
    };
    let cells: Vec<u32> = cells
        .split_whitespace()
        .map(str::parse)
        .collect::<std::result::Result<_, _>>()?;
    if !cells.len().is_multiple_of(2) {
        return Err("gpio-reserved-ranges must be <start count> pairs".into());
    }
    Ok(cells.chunks(2).flat_map(|c| c[0]..c[0] + c[1]).collect())
}

/// rpi-cm5 lab profile (scmi_linux.dtb): Linux keeps only GPIO 0..1 on the RP1 bank.
pub const BOARD_GPIO_RESERVED: std::ops::RangeInclusive<u8> = 2..=53;

pub fn dts(d: &Declaration, note: &[u8]) -> Result<String> {
    let regions = d.regions()?;
    let ddr_end = d.ddr_size()?;
    let phys = rp1_proto::SHM_PHYS;
    let bytes = note
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<Vec<_>>()
        .join(" ");
    // BCM2712 DTBs use #address-cells = <2>, #size-cells = <1> at "/" and "/reserved-memory",
    // so every reg below is <addr_hi addr_lo size> (3 cells).
    let mut s = String::from("/dts-v1/;\n/plugin/;\n/ {\n");
    let mut n = 0;
    let mut fragment = |target: &str, content: &str| {
        s.push_str(&format!(
            "fragment@{n} {{ {target}; __overlay__ {{ {content} }}; }};\n"
        ));
        n += 1;
    };
    // gpio-reserved-ranges is replaced, not merged, by an overlay. Emit the union of the board
    // profile's reservation and the declared pins so Linux never gets more pins than the base DT
    // gives it (`cargo rp1 check --dtb` verifies this against the actual DTB).
    let mut pins: Vec<_> = d.proc0.pins.iter().chain(&d.proc1.pins).copied().collect();
    if d.log.is_some() {
        pins.extend([14, 15]);
    }
    pins.extend(BOARD_GPIO_RESERVED);
    pins.sort();
    pins.dedup();
    let ranges = pins
        .iter()
        .map(|p| format!("{p} 1"))
        .collect::<Vec<_>>()
        .join(" ");
    if !pins.is_empty() {
        fragment(
            "target = <&rp1_gpio>",
            &format!("gpio-reserved-ranges = <{ranges}>;"),
        );
    }
    let mut uarts: Vec<_> = d
        .proc0
        .peripherals
        .iter()
        .chain(&d.proc1.peripherals)
        .filter(|p| p.starts_with("UART"))
        .map(|p| p[4..].parse::<u8>().unwrap())
        .collect();
    if d.log.is_some() {
        uarts.push(0);
    }
    for uart in uarts {
        fragment(
            &format!("target = <&rp1_uart{uart}>"),
            "status = \"disabled\";",
        );
    }
    fragment(
        "target-path = \"/\"",
        &format!(
            r#"
        rp1-firmware {{ rp1,note = [{bytes}]; rp1,abi = <2>; rp1,static-only; }};
        rp1_link: rp1-link@1f0040c000 {{ compatible = "generic-uio"; linux,uio-name = "rp1:link";
            reg = <0x1f 0x0040c000 0x4000>; status = "okay"; }};
        rp1-doorbell@1f000d0000 {{ compatible = "generic-uio"; linux,uio-name = "rp1:doorbell";
            reg = <0x1f 0x000d0000 0x4000>; status = "okay"; }};
        rp1-header@{phys:x} {{ compatible = "generic-uio"; linux,uio-name = "rp1:header";
            reg = <0 0x{phys:x} 0x10000>; status = "okay"; }};
    "#
        ),
    );
    // Keep the PoC scratch/openamp reservations intact; use the independent board-table range.
    fragment(
        "target-path = \"/reserved-memory\"",
        &format!("rp1-shm@{phys:x} {{ no-map; reg = <0 0x{phys:x} 0x{ddr_end:x}>; }};"),
    );
    for r in regions {
        let addr = phys + u64::from(r.ddr_offset);
        fragment(
            "target-path = \"/\"",
            &format!(
                "rp1-{}@{addr:x} {{ compatible = \"generic-uio\"; linux,uio-name = \"rp1:{}\"; reg = <0 0x{addr:x} 0x{:x}>; status = \"okay\"; }};",
                r.spec.name, r.spec.name, r.ddr_len
            ),
        );
    }
    // rp1-mailbox keeps HOST_EVENTS (hwirq 58) and stays SCMI's transport; UIO cannot share that
    // IRQ (HW 2026-10-28: -EBUSY). v0.1 has no proc1 -> Linux interrupt: the UIO nodes above carry
    // no interrupts and Linux polls the link page. Only the OpenAMP remoteproc (mailbox channels
    // 1..3, not SCMI's channel 0) is disabled; it has no label in the lab profile, so by path.
    fragment(
        "target-path = \"/openamp-address-space\"",
        "status = \"disabled\";",
    );
    if d.clock.authority == "rp1" {
        // scmi_linux profiles already supply the clock-consumer rewrites and mailbox/shmem wiring.
        fragment("target = <&rp1_clocks>", "status = \"disabled\";");
        fragment("target = <&rp1_scmi>", "status = \"okay\";");
        fragment("target = <&rp1_scmi_clocks>", "status = \"okay\";");
    }
    s.push_str("};\n");
    Ok(s)
}

/// Board-table label of the Linux node for an OWNERSHIP block id (`rp1_proto::BlockId`).
fn block_label(id: u16) -> Option<String> {
    Some(match id {
        0..=5 => format!("rp1_uart{id}"),
        16..=24 => format!("rp1_spi{}", id - 16),
        32..=38 => format!("rp1_i2c{}", id - 32),
        48..=49 => format!("rp1_pwm{}", id - 48),
        56..=58 => format!("rp1_i2s{}", id - 56),
        64 => "rp1_adc".into(),
        65 => "rp1_dma".into(),
        66 => "rp1_pio".into(),
        _ => return None,
    })
}

/// RP1 bank-0 pins a pinctrl group (and its subnodes) configures.
fn group_pins(fdt: &crate::fdt::Fdt, group: usize, out: &mut Vec<u32>) {
    for p in fdt.strings(group, "pins") {
        if let Some(n) = p.strip_prefix("gpio").and_then(|n| n.parse().ok()) {
            out.push(n);
        }
    }
    out.extend(fdt.cells(group, "brcm,pins").unwrap_or_default());
    for &c in &fdt.nodes[group].children {
        group_pins(fdt, c, out);
    }
}

/// Final-DTB ownership check (declaration-and-note.md §3.2 step 3, design §9): every conflict
/// between the note's OWNERSHIP/SHM records and what Linux would see in `dtb`.
/// An empty list means no conflict was found; a missing board-table label fails closed.
pub fn final_conflicts(dtb: &[u8], note: &[u8]) -> Result<Vec<String>> {
    use rp1_proto::note::{self, Note, ShmArea, ShmRegion};
    let fdt = crate::fdt::Fdt::parse(dtb)?;
    let parsed = Note::parse(note).map_err(|e| format!("note: {e:?}"))?;
    let mut bad = Vec::new();

    // The Linux-visible note must be this ELF's note (another overlay may have replaced it).
    match fdt
        .find("/rp1-firmware")
        .and_then(|n| fdt.prop(n, "rp1,note"))
    {
        Some(v) if v == note => {}
        Some(_) => bad.push("/rp1-firmware rp1,note differs from the ELF note".into()),
        None => bad.push("/rp1-firmware rp1,note is missing".into()),
    }

    let own = parsed
        .get(note::OWNERSHIP)
        .ok_or("note has no OWNERSHIP")?
        .value;
    let count = note::u32_at(own, 0).map_err(|_| "OWNERSHIP truncated")? as usize;
    let mut pins = std::collections::BTreeSet::new();
    for i in 0..count {
        let at = 4 + i * 4;
        let (kind, id) = (
            own[at],
            note::u16_at(own, at + 2).map_err(|_| "OWNERSHIP truncated")?,
        );
        match kind {
            1 => {
                pins.insert(u32::from(id));
            }
            2 => {
                let label = block_label(id).ok_or(format!("unknown OWNERSHIP block id {id}"))?;
                match fdt.label(&label) {
                    Some(n) if fdt.enabled(n) => bad.push(format!(
                        "RP1-owned block {label} ({}) is enabled for Linux",
                        fdt.path(n)
                    )),
                    Some(_) => {}
                    None => bad.push(format!("board DTB has no node labelled {label}")),
                }
            }
            5 => match fdt.label("rp1_clocks") {
                Some(n) if fdt.enabled(n) => {
                    bad.push("clock authority is RP1 but rp1_clocks is enabled".into())
                }
                Some(_) => {}
                None => bad.push("board DTB has no node labelled rp1_clocks".into()),
            },
            _ => {}
        }
    }

    // pinctrl: an enabled node selecting an RP1 pin group that touches an RP1-owned pin.
    if !pins.is_empty() {
        let gpio = fdt
            .label("rp1_gpio")
            .ok_or("board DTB has no node labelled rp1_gpio")?;
        for n in (0..fdt.nodes.len()).filter(|n| fdt.enabled(*n)) {
            for (name, _) in &fdt.nodes[n].props {
                let Some(state) = name.strip_prefix("pinctrl-") else {
                    continue;
                };
                if state.parse::<u32>().is_err() {
                    continue;
                }
                for handle in fdt.cells(n, name).unwrap_or_default() {
                    let Some(group) = fdt.phandle(handle) else {
                        bad.push(format!(
                            "{} {name}: phandle {handle:#x} not found",
                            fdt.path(n)
                        ));
                        continue;
                    };
                    if !fdt.is_descendant(group, gpio) {
                        continue;
                    }
                    let mut used = Vec::new();
                    group_pins(&fdt, group, &mut used);
                    for p in used.into_iter().filter(|p| pins.contains(p)) {
                        bad.push(format!(
                            "{} {name} -> {} configures RP1-owned GPIO{p}",
                            fdt.path(n),
                            fdt.path(group)
                        ));
                    }
                }
            }
        }
    }

    // Shared-memory reservation: exactly the note's area, overlapping nothing else.
    let area = parsed
        .get(note::SHM_AREA)
        .map(|r| ShmArea::parse(r.value))
        .transpose()
        .map_err(|e| format!("SHM_AREA: {e:?}"))?;
    if let Some(area) = area {
        let (start, end) = (area.phys, area.phys + u64::from(area.size));
        let reserved = fdt
            .find("/reserved-memory")
            .ok_or("DTB has no /reserved-memory")?;
        let mut own_node = false;
        for &c in &fdt.nodes[reserved].children {
            if !fdt.enabled(c) {
                continue;
            }
            for (a, s) in fdt.reg(c)? {
                if fdt.nodes[c].name.starts_with("rp1-shm@") {
                    if (a, s) == (start, area.size.into()) && fdt.prop(c, "no-map").is_some() {
                        own_node = true;
                    } else {
                        bad.push(format!(
                            "{} is {a:#x}+{s:#x} (no-map {}), note wants {start:#x}+{:#x} no-map",
                            fdt.path(c),
                            fdt.prop(c, "no-map").is_some(),
                            area.size
                        ));
                    }
                } else if a < end && a.saturating_add(s) > start {
                    bad.push(format!(
                        "{} {a:#x}+{s:#x} overlaps the RP1 shared memory {start:#x}..{end:#x}",
                        fdt.path(c)
                    ));
                }
            }
        }
        if !own_node {
            bad.push(format!(
                "no enabled no-map rp1-shm reservation at {start:#x}"
            ));
        }
        for (a, s) in &fdt.memreserve {
            if *a < end && a.saturating_add(*s) > start {
                bad.push(format!(
                    "/memreserve/ {a:#x}+{s:#x} overlaps the RP1 shared memory"
                ));
            }
        }
        // Every per-region UIO node must expose exactly its note region.
        for r in parsed
            .records()
            .filter_map(std::result::Result::ok)
            .filter(|r| r.kind == note::SHM_REGION)
        {
            let region = ShmRegion::parse(r.value).map_err(|e| format!("SHM_REGION: {e:?}"))?;
            let name = region.name;
            let want = (
                start + u64::from(region.ddr_offset),
                u64::from(region.ddr_len),
            );
            let found: Vec<_> = (0..fdt.nodes.len())
                .filter(|n| {
                    fdt.enabled(*n) && fdt.strings(*n, "linux,uio-name") == [format!("rp1:{name}")]
                })
                .collect();
            if found.len() != 1 || fdt.reg(found[0])? != [want] {
                bad.push(format!(
                    "UIO rp1:{name} must be one enabled node with reg {:#x}+{:#x}",
                    want.0, want.1
                ));
            }
        }
    }
    Ok(bad)
}
