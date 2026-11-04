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
    /// Disabled only by the gpiomem policy (the expected default), not by an ownership
    /// conflict. Host checks report these without failing.
    pub policy_only: bool,
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
    let mut gpiomem = BTreeSet::new();
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
                    gpiomem.insert(n);
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
    let mut refused = conflicts.unresolved;
    // The console must stay Linux's: a stdout-path on an RP1-owned block (which the overlay
    // has already disabled) would silently remove it (sharing design §7, risk 5).
    for (prop, found) in console_nodes(after) {
        if let Ok(n) = found
            && let Some(label) = conflicts.owned_blocks.get(&n)
        {
            refused.push(format!(
                "console {prop} is {} ({label}), which RP1 owns",
                after.path(n)
            ));
        }
    }
    // The gpiomem policy is not an ownership conflict: on_conflict=refuse still disables it.
    let policy_only = |n: &usize, reasons: &Vec<String>| gpiomem.contains(n) && reasons.len() == 1;
    // Only a planned disable needs the protected list: DT defects unrelated to RP1 (e.g. an
    // unresolvable stdout-path alias) must not stop a boot without conflicts. The gpiomem node
    // (identified by compatible and reg above) is never the console, so with the default policy
    // (always one disable) an unknown console stops the boot only next to a real conflict.
    let protected = if conflicts.nodes.is_empty() {
        BTreeMap::new()
    } else {
        let (protected, console) = protected_nodes(after)?;
        if let Some(why) = console
            && conflicts.nodes.iter().any(|(n, r)| !policy_only(n, r))
        {
            refused.push(format!(
                "{why}; cannot tell whether a disable would remove the Linux console"
            ));
        }
        protected
    };
    let mut disable = Vec::new();
    for (n, reasons) in conflicts.nodes {
        let policy_only = policy_only(&n, &reasons);
        if let Some(why) = protected.get(&n) {
            for reason in reasons {
                refused.push(format!("{reason}; protected: {why}"));
            }
        } else if policy.on_conflict == OnConflict::Refuse && !policy_only {
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
                policy_only,
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

/// The loader's step after applying `.rp1.dtbo` (sharing design §4, 4b..4e), shared with the
/// host predictions (`cargo rp1 check --dtb`, `RP1_BOARD_DTB`): read the policy from
/// `/rp1-firmware`, [`resolve`], and apply a disable plan with [`apply_plan`] (which
/// rechecks against `before`).
///
/// `Ok(Plan::Disable(nodes))`: `after` now has `nodes` disabled and marked.
/// `Ok(Plan::Refuse(reasons))`: RP1 must not start; `after` is unchanged.
/// `Err`: malformed input or a failed post-disable check; `after` is unchanged.
pub fn enforce(before: &Fdt, after: &mut Fdt, note: &[u8]) -> Result<(Policy, Plan)> {
    let policy = Policy::from_fdt(after)?;
    let plan = resolve(after, note, policy)?;
    if matches!(plan, Plan::Disable(_)) {
        apply_plan(before, after, note, policy, &plan)?;
    }
    Ok((policy, plan))
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
    /// OWNERSHIP block ids (`rp1_proto::BlockId`).
    blocks: BTreeSet<u16>,
    /// Linux nodes of the RP1-owned blocks, enabled or not.
    owned_blocks: BTreeMap<usize, String>,
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
        blocks: BTreeSet::new(),
        owned_blocks: BTreeMap::new(),
    };
    for entry in own[4..].as_chunks::<4>().0 {
        let id = u16::from_le_bytes([entry[2], entry[3]]);
        match entry[0] {
            1 => {
                out.pins.insert(u32::from(id));
            }
            2 | 5 => {
                if entry[0] == 2 {
                    out.blocks.insert(id);
                }
                let label = if entry[0] == 5 {
                    String::from("rp1_clocks")
                } else {
                    block_label(id).ok_or_else(|| format!("unknown OWNERSHIP block id {id}"))?
                };
                if let Some(n) = fdt.label(&label) {
                    out.owned_blocks.insert(n, label.clone());
                }
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
    pwm_consumers(fdt, &mut out)?;
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
    let gpio_handle = fdt.phandle_of(gpio);
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
                // Strict only for rp1_gpio specifiers. Linux ignores a malformed list on another
                // controller, so it must not stop the boot; it refuses only when the part that
                // cannot be parsed may still name rp1_gpio.
                let Some(values) = fdt.cells(n, name) else {
                    let raw = fdt.prop(n, name).unwrap_or_default();
                    if names_handle(raw.as_chunks::<4>().0.iter().map(be32), gpio_handle) {
                        out.unresolved.push(format!(
                            "{} {name}: malformed GPIO list may name rp1_gpio",
                            fdt.path(n)
                        ));
                    }
                    continue;
                };
                let mut rest = values.as_slice();
                while let Some((&handle, tail)) = rest.split_first() {
                    if handle == 0 {
                        rest = tail;
                        continue;
                    }
                    let layout = fdt
                        .phandle(handle)
                        .and_then(|ctrl| Some((ctrl, gpio_cells(fdt, ctrl).ok()?)));
                    let Some((ctrl, count)) = layout else {
                        // Unknown controller or #gpio-cells: the rest cannot be split.
                        if names_handle(tail.iter().copied(), gpio_handle) {
                            out.unresolved.push(format!(
                                "{} {name}: GPIO phandle {handle:#x} cannot be parsed and a later entry may name rp1_gpio",
                                fdt.path(n)
                            ));
                        }
                        break;
                    };
                    let Some(spec) = tail.get(..count) else {
                        if ctrl == gpio {
                            return Err(format!(
                                "{} {name}: truncated GPIO specifier",
                                fdt.path(n)
                            ));
                        }
                        break;
                    };
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
                // A dangling or malformed pinctrl entry cannot select an RP1 group: Linux fails
                // that device's pinctrl, nothing more. A malformed list that has an RP1 group's
                // phandle in it cannot be checked, so it refuses.
                let words: Vec<u32> = match fdt.cells(n, name) {
                    Some(words) => words,
                    None => {
                        let raw = fdt.prop(n, name).unwrap_or_default();
                        if raw
                            .as_chunks::<4>()
                            .0
                            .iter()
                            .map(be32)
                            .any(|h| fdt.phandle(h).is_some_and(|g| fdt.is_descendant(g, gpio)))
                        {
                            out.unresolved.push(format!(
                                "{} {name}: malformed pinctrl list names an RP1 pin group",
                                fdt.path(n)
                            ));
                        }
                        continue;
                    }
                };
                for handle in words {
                    let Some(group) = fdt.phandle(handle) else {
                        continue;
                    };
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
                        // Generic binding without `pins`: pinctrl-rp1 has one group per pin,
                        // named gpio0..gpio53, and muxes the group's function onto it. All
                        // three selectors are read, so the set covers whichever Linux uses.
                        if let Some(v) = fdt.prop(g, "groups") {
                            if v.last() != Some(&0) || core::str::from_utf8(v).is_err() {
                                return Err(format!("{}: malformed groups", fdt.path(g)));
                            }
                            for group_name in fdt.strings(g, "groups") {
                                match group_name
                                    .strip_prefix("gpio")
                                    .and_then(|s| s.parse::<u32>().ok())
                                    .filter(|pin| *pin < 54)
                                {
                                    Some(pin) => pins.push(pin),
                                    None => out.unresolved.push(format!(
                                        "{} {name} -> {}: unknown RP1 pin group {group_name}",
                                        fdt.path(n),
                                        fdt.path(g)
                                    )),
                                }
                            }
                        }
                        // A pin RP1 does not own, muxed to a signal of an RP1-owned instance,
                        // still feeds that instance: inputs selected on several pins are ORed
                        // (RP1 Peripherals §3.1), so e.g. a high GPIO10 as i2c1 holds SDA.
                        // A malformed `function` must not abort the whole check: treat the
                        // group as unverifiable instead.
                        let function = match string(fdt, g, "function") {
                            Ok(f) => f,
                            Err(e) => {
                                if !pins.is_empty() {
                                    out.unresolved.push(format!(
                                        "{} {name} -> {}: unverifiable function ({e})",
                                        fdt.path(n),
                                        fdt.path(g)
                                    ));
                                }
                                None
                            }
                        };
                        // Legacy numeric `brcm,function` (single value or per-pin array) is not
                        // translated; refuse it for pins RP1 does not own (owned pins already
                        // conflict below).
                        let numeric = fdt.prop(g, "brcm,function").is_some();
                        for pin in pins {
                            if numeric && !out.pins.contains(&pin) {
                                out.unresolved.push(format!(
                                    "{} {name} -> {}: legacy numeric brcm,function on GPIO{pin} cannot be verified",
                                    fdt.path(n),
                                    fdt.path(group)
                                ));
                            }
                            if out.pins.contains(&pin) {
                                out.add(
                                    n,
                                    format!(
                                        "{} {name} -> {} configures RP1-owned GPIO{pin}",
                                        fdt.path(n),
                                        fdt.path(group)
                                    ),
                                );
                                continue;
                            }
                            let block = function
                                .zip(u8::try_from(pin).ok())
                                .and_then(|(f, pin)| rp1_pinmux::linux_signal(pin, f))
                                .and_then(signal_block)
                                .filter(|id| out.blocks.contains(id));
                            if let Some(id) = block {
                                out.add(
                                    n,
                                    format!(
                                        "{} {name} -> {} muxes GPIO{pin} to {}, a signal of RP1-owned {}",
                                        fdt.path(n),
                                        fdt.path(group),
                                        function.unwrap_or_default(),
                                        block_label(id).unwrap_or_default()
                                    ),
                                );
                            }
                        }
                        pending.extend(&fdt.nodes[g].children);
                    }
                }
            }
        }
        // Interrupt users: pinctrl-rp1 rewrites the pin's CTRL event bits when Linux maps
        // the IRQ, which would clear an RP1 gpio_irq's setup and pending events.
        for (prop, pin) in rp1_irq_lines(fdt, n, gpio, gpio_handle, &mut out.unresolved)? {
            if withheld.contains(&pin) {
                out.add(
                    n,
                    format!(
                        "{} {prop} uses GPIO{pin} that gpio-reserved-ranges withholds from Linux",
                        fdt.path(n)
                    ),
                );
            }
        }
    }
    Ok(out)
}

/// RP1 GPIO lines `n` takes as interrupts: `interrupts-extended` entries on rp1_gpio, or
/// `interrupts` when the (inherited) `interrupt-parent` is rp1_gpio. Strict for rp1_gpio
/// specifiers; another controller's entries that cannot be parsed are skipped unless the
/// rest may name rp1_gpio (then a reason goes to `unresolved`).
fn rp1_irq_lines(
    fdt: &Fdt,
    n: usize,
    gpio: usize,
    gpio_handle: Option<u32>,
    unresolved: &mut Vec<String>,
) -> Result<Vec<(&'static str, u32)>> {
    let Some(handle) = gpio_handle else {
        return Ok(Vec::new()); // nothing can refer to rp1_gpio
    };
    let irq_cells = |ctrl: usize| -> Option<usize> {
        match fdt.cells(ctrl, "#interrupt-cells")?.as_slice() {
            [c] if *c > 0 => Some(*c as usize),
            _ => None,
        }
    };
    let rp1_cells = || {
        irq_cells(gpio)
            .ok_or_else(|| String::from("rp1_gpio: missing or unsupported #interrupt-cells"))
    };
    let mut lines = Vec::new();
    if fdt.prop(n, "interrupts-extended").is_some() {
        let prop = "interrupts-extended";
        let Some(values) = fdt.cells(n, prop) else {
            let raw = fdt.prop(n, prop).unwrap_or_default();
            if names_handle(raw.as_chunks::<4>().0.iter().map(be32), Some(handle)) {
                unresolved.push(format!(
                    "{} {prop}: malformed list may name rp1_gpio",
                    fdt.path(n)
                ));
            }
            return Ok(lines);
        };
        let mut rest = values.as_slice();
        while let Some((&h, tail)) = rest.split_first() {
            let layout = if h == handle {
                Some((true, rp1_cells()?))
            } else {
                fdt.phandle(h).and_then(irq_cells).map(|c| (false, c))
            };
            let Some((is_rp1, count)) = layout else {
                if names_handle(tail.iter().copied(), Some(handle)) {
                    unresolved.push(format!(
                        "{} {prop}: phandle {h:#x} cannot be parsed and a later entry may name rp1_gpio",
                        fdt.path(n)
                    ));
                }
                break;
            };
            let Some(spec) = tail.get(..count) else {
                if is_rp1 {
                    return Err(format!(
                        "{} {prop}: truncated rp1_gpio specifier",
                        fdt.path(n)
                    ));
                }
                break;
            };
            if is_rp1 {
                lines.push((prop, spec[0]));
            }
            rest = &tail[count..];
        }
        return Ok(lines);
    }
    if fdt.prop(n, "interrupts").is_none() {
        return Ok(lines);
    }
    if irq_parent(fdt, n) != Some(gpio) {
        return Ok(lines);
    }
    let count = rp1_cells()?;
    let values = cells(fdt, n, "interrupts")?;
    if values.is_empty() || !values.len().is_multiple_of(count) {
        return Err(format!(
            "{} interrupts: malformed rp1_gpio specifier",
            fdt.path(n)
        ));
    }
    lines.extend(
        values
            .chunks_exact(count)
            .map(|spec| ("interrupts", spec[0])),
    );
    Ok(lines)
}

/// A disabled PWM provider must not leave its Linux consumers eligible to probe.
/// Parse each specifier with its provider's #pwm-cells; raw cell matching could mistake a
/// period or duty argument for an owned provider's phandle.
fn pwm_consumers(fdt: &Fdt, out: &mut Conflicts) -> Result<()> {
    if !out.blocks.iter().any(|id| (48..=49).contains(id)) {
        return Ok(());
    }
    for n in 0..fdt.nodes.len() {
        if !fdt.enabled(n) || metadata_node(fdt, n) || fdt.prop(n, "pwms").is_none() {
            continue;
        }
        let values = cells(fdt, n, "pwms")?;
        let mut at = 0;
        while at < values.len() {
            let handle = values[at];
            at += 1;
            if handle == 0 {
                continue;
            } // Empty phandle slot (of_parse_phandle_with_args).
            let provider = fdt
                .phandle(handle)
                .ok_or_else(|| format!("{} pwms: unresolved provider {handle:#x}", fdt.path(n)))?;
            let count = cells(fdt, provider, "#pwm-cells")?;
            let count = match count.as_slice() {
                [count] if *count > 0 => *count as usize,
                _ => return Err(format!("{}: unsupported #pwm-cells", fdt.path(provider))),
            };
            if count > values.len() - at {
                return Err(format!("{} pwms: truncated specifier", fdt.path(n)));
            }
            if let Some(label) = out.owned_blocks.get(&provider) {
                out.add(n, format!("pwms uses RP1-owned block {label}"));
            }
            at += count;
        }
    }
    Ok(())
}

/// OWNERSHIP block id (`rp1_proto::BlockId`) of the instance a bank-0 signal belongs to.
fn signal_block(signal: rp1_pinmux::Signal) -> Option<u16> {
    use rp1_pinmux::Signal;
    Some(match signal {
        Signal::Uart(n, _) => u16::from(n),
        Signal::Spi(n, _) => 16 + u16::from(n),
        Signal::I2c(n, _) => 32 + u16::from(n),
        Signal::Pwm(n, _) => 48 + u16::from(n),
        Signal::I2s(n, _) => 56 + u16::from(n),
        Signal::Pio(_) => 66,
        _ => return None,
    })
}

/// The interrupt parent of `n` as Linux's `of_irq_find_parent` finds it: follow the node's
/// `interrupt-parent` phandle, or else its DT parent, until a node with `#interrupt-cells`.
/// A dangling phandle means none. A `interrupt-parent` shorter than a cell is read as absent
/// (`of_property_read_u32` fails), so the walk goes to the DT parent.
fn irq_parent(fdt: &Fdt, n: usize) -> Option<usize> {
    let mut child = n;
    // Bounded: a phandle cycle without #interrupt-cells must not loop forever.
    for _ in 0..=fdt.nodes.len() {
        let next = match fdt.prop(child, "interrupt-parent") {
            Some(v) if v.len() >= 4 => fdt.phandle(u32::from_be_bytes([v[0], v[1], v[2], v[3]]))?,
            _ => fdt.nodes[child].parent?,
        };
        if fdt.prop(next, "#interrupt-cells").is_some() {
            return Some(next);
        }
        child = next;
    }
    None
}

fn be32(word: &[u8; 4]) -> u32 {
    u32::from_be_bytes(*word)
}

/// Whether any word may be the phandle `handle` (false when the controller has none).
fn names_handle(mut words: impl Iterator<Item = u32>, handle: Option<u32>) -> bool {
    handle.is_some_and(|h| words.any(|w| w == h))
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

/// The nodes a disable must not touch, and why the console could not be found, if so.
fn protected_nodes(fdt: &Fdt) -> Result<(BTreeMap<usize, String>, Option<String>)> {
    let mut protected = BTreeMap::new();
    // Protect ancestors too: disabling a bus would otherwise hide a protected child.
    let mut protect = |n: usize, why: String| {
        let mut at = Some(n);
        while let Some(n) = at {
            protected.entry(n).or_insert_with(|| why.clone());
            at = fdt.nodes[n].parent;
        }
    };
    // The board's Linux blocks (one table with rp1-build's declaration check), plus the RP1
    // GPIO controller and the SCMI transport the generated overlay depends on.
    let board = rp1_proto::board::LINUX_BLOCKS
        .iter()
        .map(|(_, label)| *label);
    for label in board.chain(["rp1_gpio", "rp1_scmi", "rp1_scmi_clocks", "rp1_scmi_shmem"]) {
        if let Some(n) = fdt.label(label) {
            protect(n, format!("board Linux block {label}"));
        }
    }
    let mut console_unknown = None;
    for (prop, found) in console_nodes(fdt) {
        match found {
            Ok(n) => protect(n, format!("console {prop}")),
            Err(why) => console_unknown = Some(why),
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
    Ok((protected, console_unknown))
}

/// The Linux console nodes named by `/chosen` (`stdout-path`, `linux,stdout-path`), each
/// resolved through `/aliases` if needed, or why it could not be resolved.
pub(super) fn console_nodes(fdt: &Fdt) -> Vec<(&'static str, core::result::Result<usize, String>)> {
    let Some(chosen) = fdt.find("/chosen") else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for prop in ["stdout-path", "linux,stdout-path"] {
        let value = match string(fdt, chosen, prop) {
            Ok(None) => continue,
            Ok(Some(v)) => v,
            Err(e) => {
                out.push((prop, Err(e)));
                continue;
            }
        };
        let path = value.split(':').next().unwrap_or_default();
        let node = if path.starts_with('/') {
            fdt.find(path)
        } else {
            fdt.find("/aliases")
                .and_then(|a| string(fdt, a, path).ok().flatten())
                .and_then(|p| fdt.find(p))
        };
        out.push((
            prop,
            node.ok_or_else(|| format!("console {prop}={value} does not resolve to a node")),
        ));
    }
    out
}
