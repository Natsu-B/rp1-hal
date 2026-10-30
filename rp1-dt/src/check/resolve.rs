//! STATIC: ownership resolution for a final, merged board DTB. No hardware access.
use super::{block_label, check, final_conflicts, reserved_gpios};
use crate::{Result, fdt::Fdt};
use alloc::{
    collections::{BTreeMap, BTreeSet},
    format,
    string::String,
    vec::Vec,
};
use rp1_proto::note::{self, Note};

/// What the loader does when an enabled Linux node conflicts with RP1 ownership
/// (`/rp1-firmware/rp1,on-conflict`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OnConflict {
    /// Disable the conflicting node and record it (default).
    #[default]
    Disable,
    /// Refuse to start RP1.
    Refuse,
}

/// Whether `/dev/gpiomem0` stays available to Linux (`/rp1-firmware/rp1,gpiomem`).
/// It maps every bank-0 pin's registers, so it cannot be restricted per pin.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Gpiomem {
    /// Disable it whenever RP1 owns a pin (default).
    #[default]
    Disable,
    /// Leave it enabled.
    Keep,
}

/// Missing policy properties use the schema-2 defaults. Unknown/malformed values fail closed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Policy {
    pub on_conflict: OnConflict,
    pub gpiomem: Gpiomem,
}

impl Policy {
    /// Read the policy from `/rp1-firmware`.
    pub fn from_fdt(fdt: &Fdt) -> Result<Self> {
        let n = fdt
            .find("/rp1-firmware")
            .ok_or("DTB has no /rp1-firmware")?;
        let on_conflict = match string(fdt, n, "rp1,on-conflict")? {
            None | Some("disable") => OnConflict::Disable,
            Some("refuse") => OnConflict::Refuse,
            Some(_) => return Err("unknown rp1,on-conflict policy".into()),
        };
        let gpiomem = match string(fdt, n, "rp1,gpiomem")? {
            None | Some("disable") => Gpiomem::Disable,
            Some("keep") => Gpiomem::Keep,
            Some(_) => return Err("unknown rp1,gpiomem policy".into()),
        };
        Ok(Self {
            on_conflict,
            gpiomem,
        })
    }
}

/// One Linux node to disable; reasons include paths and the specific GPIO/block involved.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Disable {
    pub path: String,
    pub label: Option<String>,
    pub reasons: Vec<String>,
}

/// A refusal never authorizes partial application of the disable list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Plan {
    /// Disable these nodes (empty: nothing conflicts).
    Disable(Vec<Disable>),
    /// Do not start RP1, for these reasons.
    Refuse(Vec<String>),
}

/// Plan disables without changing `after`. `note` is the expected ownership descriptor;
/// it must match `/rp1-firmware/rp1,note`. Read policy with [`Policy::from_fdt`].
/// Protected conflicts, strict policy, or invariants that disables cannot fix refuse.
/// Malformed data returns `Err`. A proposed plan is simulated and checked before return.
pub fn resolve(after: &Fdt, note: &[u8], policy: Policy) -> Result<Plan> {
    let parsed = Note::parse(note).map_err(|e| format!("note: {e:?}"))?;
    let own = parsed
        .get(note::OWNERSHIP)
        .ok_or("note has no OWNERSHIP")?
        .value;
    let mut conflicts = node_conflicts(after, own)?;
    if !conflicts.pins.is_empty() && policy.gpiomem == Gpiomem::Disable {
        for n in (0..after.nodes.len()).filter(|n| after.enabled(*n)) {
            if after
                .strings(n, "compatible")
                .iter()
                .any(|s| s == "raspberrypi,gpiomem")
            {
                // RP1 bus address, independent of node name, label or board path.
                if after.prop(n, "reg").is_none() || after.cells(n, "reg").is_none() {
                    return Err(format!(
                        "{}: missing or malformed gpiomem reg",
                        after.path(n)
                    ));
                }
                let reg = after.reg(n)?;
                let (address, _) = reg
                    .first()
                    .ok_or_else(|| format!("{}: empty gpiomem reg", after.path(n)))?;
                if *address == 0xc0_400d0000 {
                    conflicts.add(
                        n,
                        format!(
                            "{} exposes RP1 GPIO registers (gpiomem policy)",
                            after.path(n)
                        ),
                    );
                }
            }
        }
    }
    let protected = protected_nodes(after)?;
    let mut refused = conflicts.unresolved;
    let mut disable = Vec::new();
    for (n, reasons) in conflicts.nodes {
        if let Some(why) = protected.get(&n) {
            for reason in reasons {
                refused.push(format!("{reason}; protected: {why}"));
            }
        } else if policy.on_conflict == OnConflict::Refuse {
            for reason in reasons {
                refused.push(format!("{reason}; on_conflict=refuse"));
            }
        } else {
            let label = after.find("/__symbols__").and_then(|s| {
                after.nodes[s]
                    .props
                    .iter()
                    .filter(|(name, _)| after.label(name) == Some(n))
                    .min_by_key(|(name, _)| (!name.starts_with("rp1_"), name.as_str()))
                    .map(|(name, _)| name.clone())
            });
            disable.push(Disable {
                path: after.path(n),
                label,
                reasons,
            });
        }
    }
    if !refused.is_empty() {
        return Ok(Plan::Refuse(refused));
    }
    let candidate = with_disables(after, &disable)?;
    let remaining = final_conflicts(&candidate, note)?;
    if !remaining.is_empty() {
        return Ok(Plan::Refuse(remaining));
    }
    Ok(Plan::Disable(disable))
}

/// Apply an unchanged plan transactionally, mark each disabled node and record the path
/// string list in `/rp1-firmware/rp1,loader-disabled`. Recheck against the pre-overlay
/// `before` tree, including reservation shrinkage. On any error `after` is unchanged.
/// A stale or caller-modified plan is rejected, as is a refusal.
pub fn apply_plan(
    before: &Fdt,
    after: &mut Fdt,
    note: &[u8],
    policy: Policy,
    plan: &Plan,
) -> Result<()> {
    let Plan::Disable(disable) = plan else {
        return Err("cannot apply a refused plan".into());
    };
    if resolve(after, note, policy)? != *plan {
        return Err("disable plan no longer matches DTB, ownership or policy".into());
    }
    let candidate = with_disables(after, disable)?;
    let remaining = check(before, &candidate, note)?;
    if !remaining.is_empty() {
        return Err(format!("post-disable check: {}", remaining.join("; ")));
    }
    if resolve(&candidate, note, policy)? != Plan::Disable(Vec::new()) {
        return Err("post-disable resolution still has conflicts".into());
    }
    *after = candidate;
    Ok(())
}

fn with_disables(fdt: &Fdt, disable: &[Disable]) -> Result<Fdt> {
    let mut result = fdt.clone();
    if disable.is_empty() {
        return Ok(result);
    }
    let firmware = fdt
        .find("/rp1-firmware")
        .ok_or("DTB has no /rp1-firmware")?;
    let mut paths = fdt
        .prop(firmware, "rp1,loader-disabled")
        .unwrap_or_default()
        .to_vec();
    if !paths.is_empty() && (paths.last() != Some(&0) || core::str::from_utf8(&paths).is_err()) {
        return Err("malformed rp1,loader-disabled path list".into());
    }
    for item in disable {
        let n = result.find(&item.path).ok_or("disable node disappeared")?;
        result.set_prop(n, "status", b"disabled\0".to_vec());
        result.set_prop(n, "rp1,loader-disabled", Vec::new());
        if !paths.split(|b| *b == 0).any(|p| p == item.path.as_bytes()) {
            paths.extend_from_slice(item.path.as_bytes());
            paths.push(0);
        }
    }
    result.set_prop(firmware, "rp1,loader-disabled", paths);
    Ok(result)
}

fn string<'a>(fdt: &'a Fdt, n: usize, name: &str) -> Result<Option<&'a str>> {
    let Some(v) = fdt.prop(n, name) else {
        return Ok(None);
    };
    let raw = v
        .strip_suffix(&[0])
        .filter(|v| !v.is_empty() && !v.contains(&0))
        .ok_or_else(|| format!("{}: malformed {name}", fdt.path(n)))?;
    core::str::from_utf8(raw)
        .map(Some)
        .map_err(|_| format!("{}: non-UTF-8 {name}", fdt.path(n)))
}

fn cells(fdt: &Fdt, n: usize, name: &str) -> Result<Vec<u32>> {
    fdt.cells(n, name)
        .ok_or_else(|| format!("{}: missing or malformed {name}", fdt.path(n)))
}

fn gpio_cells(fdt: &Fdt, n: usize) -> Result<usize> {
    let c = cells(fdt, n, "#gpio-cells")?;
    match c.as_slice() {
        [count] if *count > 0 => Ok(*count as usize),
        _ => Err(format!("{}: unsupported #gpio-cells", fdt.path(n))),
    }
}

pub(super) struct Conflicts {
    pub(super) nodes: BTreeMap<usize, Vec<String>>,
    pub(super) unresolved: Vec<String>,
    pins: BTreeSet<u32>,
}
impl Conflicts {
    fn add(&mut self, n: usize, reason: String) {
        self.nodes.entry(n).or_default().push(reason);
    }
}

/// Shared with final_conflicts so the post-application check cannot miss a different
/// class of GPIO consumer than the planner. `own` has been validated by Note::parse.
pub(super) fn node_conflicts(fdt: &Fdt, own: &[u8]) -> Result<Conflicts> {
    let mut out = Conflicts {
        nodes: BTreeMap::new(),
        unresolved: Vec::new(),
        pins: BTreeSet::new(),
    };
    for entry in own[4..].as_chunks::<4>().0 {
        let id = u16::from_le_bytes([entry[2], entry[3]]);
        match entry[0] {
            1 => {
                out.pins.insert(u32::from(id));
            }
            2 | 5 => {
                let label = if entry[0] == 5 {
                    String::from("rp1_clocks")
                } else {
                    block_label(id).ok_or_else(|| format!("unknown OWNERSHIP block id {id}"))?
                };
                match fdt.label(&label) {
                    Some(n) if fdt.enabled(n) => out.add(
                        n,
                        format!(
                            "RP1-owned block {label} ({}) is enabled for Linux",
                            fdt.path(n)
                        ),
                    ),
                    Some(_) => {}
                    None => out
                        .unresolved
                        .push(format!("board DTB has no node labelled {label}")),
                }
            }
            _ => {}
        }
    }
    let reserved = reserved_gpios(fdt)?;
    let withheld: BTreeSet<u32> = reserved.union(&out.pins).copied().collect();
    if withheld.is_empty() {
        return Ok(out);
    }
    let gpio = fdt
        .label("rp1_gpio")
        .ok_or("board DTB has no node labelled rp1_gpio")?;
    for pin in out.pins.difference(&reserved) {
        out.unresolved.push(format!(
            "RP1-owned GPIO{pin} is not in gpio-reserved-ranges"
        ));
    }
    let args = gpio_cells(fdt, gpio)?;
    if args != 2 {
        return Err("rp1_gpio must have #gpio-cells = <2>".into());
    }
    for n in (0..fdt.nodes.len()).filter(|n| fdt.enabled(*n) && !metadata_node(fdt, *n)) {
        let hog = fdt.prop(n, "gpio-hog").is_some();
        if hog && fdt.nodes[n].parent == Some(gpio) {
            let lines = cells(fdt, n, "gpios")?;
            if !lines.len().is_multiple_of(args) {
                return Err(format!("{}: truncated hog gpios", fdt.path(n)));
            }
            for line in lines.chunks_exact(args) {
                if withheld.contains(&line[0]) {
                    out.add(
                        n,
                        format!(
                            "{} hogs GPIO{} that gpio-reserved-ranges withholds from Linux",
                            fdt.path(n),
                            line[0]
                        ),
                    );
                }
            }
        }
        for (name, _) in &fdt.nodes[n].props {
            if !hog
                && (name == "gpio"
                    || name == "gpios"
                    || name.ends_with("-gpio")
                    || name.ends_with("-gpios"))
            {
                let values = cells(fdt, n, name)?;
                let mut rest = values.as_slice();
                while let Some((&handle, tail)) = rest.split_first() {
                    if handle == 0 {
                        rest = tail;
                        continue;
                    }
                    let ctrl = fdt.phandle(handle).ok_or_else(|| {
                        format!("{} {name}: GPIO phandle {handle:#x} not found", fdt.path(n))
                    })?;
                    let count = gpio_cells(fdt, ctrl)?;
                    let spec = tail.get(..count).ok_or_else(|| {
                        format!("{} {name}: truncated GPIO specifier", fdt.path(n))
                    })?;
                    if ctrl == gpio && withheld.contains(&spec[0]) {
                        out.add(n, format!("{} {name} uses GPIO{} that gpio-reserved-ranges withholds from Linux", fdt.path(n), spec[0]));
                    }
                    rest = &tail[count..];
                }
            }
            if name
                .strip_prefix("pinctrl-")
                .is_some_and(|state| state.parse::<u32>().is_ok())
            {
                for handle in cells(fdt, n, name)? {
                    let group = fdt.phandle(handle).ok_or_else(|| {
                        format!("{} {name}: phandle {handle:#x} not found", fdt.path(n))
                    })?;
                    if !fdt.is_descendant(group, gpio) {
                        continue;
                    }
                    // Iterative traversal: arbitrarily deep DTBs must not exhaust the stack.
                    let mut pending = alloc::vec![group];
                    while let Some(g) = pending.pop() {
                        let mut pins = Vec::new();
                        if let Some(v) = fdt.prop(g, "pins") {
                            if v.last() != Some(&0) || core::str::from_utf8(v).is_err() {
                                return Err(format!("{}: malformed pins", fdt.path(g)));
                            }
                            for p in fdt.strings(g, "pins") {
                                pins.push(
                                    p.strip_prefix("gpio")
                                        .and_then(|s| s.parse::<u32>().ok())
                                        .ok_or_else(|| {
                                            format!("{}: invalid RP1 pin {p}", fdt.path(g))
                                        })?,
                                );
                            }
                        }
                        if fdt.prop(g, "brcm,pins").is_some() {
                            pins.extend(cells(fdt, g, "brcm,pins")?);
                        }
                        for pin in pins {
                            if !out.pins.contains(&pin) {
                                continue;
                            }
                            out.add(
                                n,
                                format!(
                                    "{} {name} -> {} configures RP1-owned GPIO{pin}",
                                    fdt.path(n),
                                    fdt.path(group)
                                ),
                            );
                        }
                        pending.extend(&fdt.nodes[g].children);
                    }
                }
            }
        }
    }
    Ok(out)
}

// Labels, aliases and firmware overlay parameters are not Linux GPIO consumers.
fn metadata_node(fdt: &Fdt, node: usize) -> bool {
    let mut at = Some(node);
    while let Some(n) = at {
        let name = &fdt.nodes[n].name;
        if name.starts_with("__") || (fdt.nodes[n].parent == Some(0) && name == "aliases") {
            return true;
        }
        at = fdt.nodes[n].parent;
    }
    false
}

fn protected_nodes(fdt: &Fdt) -> Result<BTreeMap<usize, String>> {
    let mut protected = BTreeMap::new();
    // Protect ancestors too: disabling a bus would otherwise hide a protected child.
    let mut protect = |n: usize, why: String| {
        let mut at = Some(n);
        while let Some(n) = at {
            protected.entry(n).or_insert_with(|| why.clone());
            at = fdt.nodes[n].parent;
        }
    };
    for label in [
        "rp1_gpio",
        "rp1_eth",
        "rp1_usb0",
        "rp1_usb1",
        "rp1_dma",
        "rp1_pio",
        "pcie2",
        "rp1_scmi",
        "rp1_scmi_clocks",
        "rp1_scmi_shmem",
    ] {
        if let Some(n) = fdt.label(label) {
            protect(n, format!("board Linux block {label}"));
        }
    }
    if let Some(chosen) = fdt.find("/chosen") {
        for prop in ["stdout-path", "linux,stdout-path"] {
            if let Some(value) = string(fdt, chosen, prop)? {
                let path = value.split(':').next().ok_or("empty stdout-path")?;
                let path = if path.starts_with('/') {
                    String::from(path)
                } else {
                    let aliases = fdt
                        .find("/aliases")
                        .ok_or("stdout-path alias has no /aliases")?;
                    String::from(string(fdt, aliases, path)?.ok_or("stdout-path alias not found")?)
                };
                let n = fdt.find(&path).ok_or("stdout-path node not found")?;
                protect(n, format!("console {prop}={value}"));
            }
        }
    }
    for n in 0..fdt.nodes.len() {
        let name = fdt.nodes[n].name.split('@').next().unwrap_or_default();
        let reserved = fdt
            .find("/reserved-memory")
            .is_some_and(|r| fdt.is_descendant(n, r));
        if reserved || name.starts_with("rp1-") || name == "rp1" || name.starts_with("pcie") {
            protect(n, format!("protected DT node {}", fdt.path(n)));
        }
    }
    Ok(protected)
}
