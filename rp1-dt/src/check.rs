//! Final-DTB checks shared by `cargo rp1 check --dtb` and the loader (declaration-and-note.md §3.2).
use crate::{Result, fdt::Fdt};
use alloc::{collections::BTreeSet, format, string::String, vec::Vec};

/// RP1 GPIO numbers that `fdt` withholds from Linux (`gpio-reserved-ranges` of the node
/// labelled `rp1_gpio`). No label or no property means none.
pub fn reserved_gpios(fdt: &Fdt) -> Result<BTreeSet<u32>> {
    let Some(gpio) = fdt.label("rp1_gpio") else {
        return Ok(BTreeSet::new());
    };
    let Some(cells) = fdt.cells(gpio, "gpio-reserved-ranges") else {
        return Ok(BTreeSet::new());
    };
    if !cells.len().is_multiple_of(2) {
        return Err("gpio-reserved-ranges must be <start count> pairs".into());
    }
    Ok(cells
        .chunks(2)
        .flat_map(|c| c[0]..c[0].saturating_add(c[1]))
        .collect())
}

/// Every conflict between `before` (the DTB the overlay was applied to), the merged `after`
/// and the ELF `note`: pins handed back to Linux plus [`final_conflicts`].
pub fn check(before: &Fdt, after: &Fdt, note: &[u8]) -> Result<Vec<String>> {
    let freed: Vec<u32> = reserved_gpios(before)?
        .difference(&reserved_gpios(after)?)
        .copied()
        .collect();
    let mut bad = Vec::new();
    if !freed.is_empty() {
        bad.push(format!(
            "overlay would hand RP1 GPIOs {freed:?} back to Linux"
        ));
    }
    bad.extend(final_conflicts(after, note)?);
    Ok(bad)
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
fn group_pins(fdt: &Fdt, group: usize, out: &mut Vec<u32>) {
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
pub fn final_conflicts(fdt: &Fdt, note: &[u8]) -> Result<Vec<String>> {
    use rp1_proto::note::{self, Note, ShmArea, ShmRegion};
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
        .ok_or_else(|| String::from("note has no OWNERSHIP"))?
        .value;
    let count = note::u32_at(own, 0).map_err(|_| "OWNERSHIP truncated")? as usize;
    let mut pins = alloc::collections::BTreeSet::new();
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

    // An enabled gpio-hog on a line withheld from Linux fails the RP1 gpiochip (HYP, gpiolib).
    if let Some(gpio) = fdt.label("rp1_gpio") {
        let reserved = reserved_gpios(fdt)?;
        for &c in &fdt.nodes[gpio].children {
            if !fdt.enabled(c) || fdt.prop(c, "gpio-hog").is_none() {
                continue;
            }
            for line in fdt.cells(c, "gpios").unwrap_or_default().chunks(2) {
                if reserved.contains(&line[0]) {
                    bad.push(format!(
                        "{} hogs GPIO{} that gpio-reserved-ranges withholds from Linux",
                        fdt.path(c),
                        line[0]
                    ));
                }
            }
        }
    }

    // An enabled consumer (`gpio(s)`, `*-gpio(s)`) of a withheld RP1 line fails to probe.
    if let Some(gpio) = fdt.label("rp1_gpio") {
        let reserved = reserved_gpios(fdt)?;
        for n in (0..fdt.nodes.len()).filter(|n| fdt.enabled(*n)) {
            for (name, _) in &fdt.nodes[n].props {
                let consumer = name == "gpio"
                    || name == "gpios"
                    || name.ends_with("-gpio")
                    || name.ends_with("-gpios");
                if !consumer || fdt.prop(n, "gpio-hog").is_some() {
                    continue;
                }
                let cells = fdt.cells(n, name).unwrap_or_default();
                let mut i = 0;
                while i < cells.len() {
                    // A 0 phandle is an empty entry (e.g. a native chip select).
                    let Some(ctrl) = fdt.phandle(cells[i]).filter(|_| cells[i] != 0) else {
                        i += 1;
                        continue;
                    };
                    let args = fdt.number(ctrl, "#gpio-cells").unwrap_or(2) as usize;
                    if ctrl == gpio
                        && let Some(&line) = cells.get(i + 1)
                        && reserved.contains(&line)
                    {
                        bad.push(format!(
                            "{} {name} uses GPIO{line} that gpio-reserved-ranges withholds from Linux",
                            fdt.path(n)
                        ));
                    }
                    i += 1 + args;
                }
            }
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
                    group_pins(fdt, group, &mut used);
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
            .filter_map(core::result::Result::ok)
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
