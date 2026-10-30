//! Final-DTB checks shared by `cargo rp1 check --dtb` and the loader (declaration-and-note.md §3.2).
use crate::{Result, fdt::Fdt};
use alloc::{collections::BTreeSet, format, string::String, vec::Vec};

mod resolve;
pub use resolve::{Disable, Gpiomem, OnConflict, Plan, Policy, apply_plan, enforce, resolve};

/// RP1 GPIO numbers that `fdt` withholds from Linux (`gpio-reserved-ranges` of the node
/// labelled `rp1_gpio`). No label or no property means none.
pub fn reserved_gpios(fdt: &Fdt) -> Result<BTreeSet<u32>> {
    let Some(gpio) = fdt.label("rp1_gpio") else {
        return Ok(BTreeSet::new());
    };
    if fdt.prop(gpio, "gpio-reserved-ranges").is_none() {
        return Ok(BTreeSet::new());
    }
    let cells = fdt
        .cells(gpio, "gpio-reserved-ranges")
        .ok_or("malformed gpio-reserved-ranges")?;
    if !cells.len().is_multiple_of(2) {
        return Err("gpio-reserved-ranges must be <start count> pairs".into());
    }
    let mut pins = BTreeSet::new();
    for pair in cells.as_chunks::<2>().0 {
        let end = pair[0]
            .checked_add(pair[1])
            .filter(|end| *end <= 54)
            .ok_or("gpio-reserved-ranges exceeds RP1 GPIO0..53")?;
        pins.extend(pair[0]..end);
    }
    Ok(pins)
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
    let conflicts = resolve::node_conflicts(fdt, own)?;
    bad.extend(conflicts.unresolved);
    bad.extend(conflicts.nodes.into_values().flatten());

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
