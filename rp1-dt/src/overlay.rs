//! Device-tree overlay application with libfdt `fdt_overlay_apply` semantics:
//! renumber the overlay's phandles above the base's, resolve `__local_fixups__` and
//! `__fixups__` (base `__symbols__`), merge each `fragment@N/__overlay__` into its `target`
//! (phandle) or `target-path`, and export the overlay's `__symbols__` into the base.
use crate::{Result, fdt::Fdt};
use alloc::{format, string::String, vec::Vec};

fn special(name: &str) -> bool {
    matches!(name, "__symbols__" | "__fixups__" | "__local_fixups__")
}

fn put_be32(v: &mut [u8], at: usize, value: u32) -> Result<()> {
    v.get_mut(at..at + 4)
        .ok_or("fixup offset outside the property")?
        .copy_from_slice(&value.to_be_bytes());
    Ok(())
}

/// Add `delta` to every phandle the overlay defines and to every local reference to one.
fn renumber(ov: &mut Fdt, delta: u32) -> Result<()> {
    for n in 0..ov.nodes.len() {
        for (name, v) in ov.nodes[n].props.iter_mut() {
            if (name == "phandle" || name == "linux,phandle") && v.len() == 4 {
                let p = u32::from_be_bytes([v[0], v[1], v[2], v[3]]);
                if p != 0 && p != u32::MAX {
                    put_be32(v, 0, p.checked_add(delta).ok_or("phandle overflow")?)?;
                }
            }
        }
    }
    update_local_refs(ov, |p| {
        p.checked_add(delta)
            .ok_or_else(|| "phandle overflow".into())
    })
}

fn update_local_refs(ov: &mut Fdt, update: impl Fn(u32) -> Result<u32>) -> Result<()> {
    let Some(local) = ov.find("/__local_fixups__") else {
        return Ok(());
    };
    // Walk __local_fixups__ in parallel with the overlay tree.
    let mut work = alloc::vec![(local, 0usize)];
    while let Some((fix, node)) = work.pop() {
        let props = ov.nodes[fix].props.clone();
        for (name, offsets) in props {
            if offsets.len() % 4 != 0 {
                return Err("bad __local_fixups__ entry".into());
            }
            let target = ov.nodes[node]
                .props
                .iter_mut()
                .find(|(n, _)| *n == name)
                .ok_or_else(|| format!("__local_fixups__: no property {name}"))?;
            for o in offsets.chunks(4) {
                let at = u32::from_be_bytes([o[0], o[1], o[2], o[3]]) as usize;
                let v = target.1.get(at..at + 4).ok_or("local fixup out of range")?;
                let p = u32::from_be_bytes([v[0], v[1], v[2], v[3]]);
                put_be32(&mut target.1, at, update(p)?)?;
            }
        }
        for &c in &ov.nodes[fix].children.clone() {
            let name = ov.nodes[c].name.clone();
            let n = ov
                .child(node, &name)
                .ok_or_else(|| format!("__local_fixups__: no node {name}"))?;
            work.push((c, n));
        }
    }
    Ok(())
}

/// Resolve `__fixups__` (`label = "path:prop:offset", ...`) against the base's symbols.
fn resolve(base: &Fdt, ov: &mut Fdt) -> Result<()> {
    let Some(fixups) = ov.find("/__fixups__") else {
        return Ok(());
    };
    for (label, refs) in ov.nodes[fixups].props.clone() {
        let node = base
            .label(&label)
            .ok_or_else(|| format!("overlay references label {label} missing from the base DTB"))?;
        let handle = base
            .phandle_of(node)
            .ok_or_else(|| format!("base node {} for {label} has no phandle", base.path(node)))?;
        for r in refs.split(|c| *c == 0).filter(|r| !r.is_empty()) {
            let r = core::str::from_utf8(r).map_err(|_| "bad __fixups__ entry")?;
            let mut parts = r.rsplitn(3, ':');
            let (Some(off), Some(prop), Some(path)) = (parts.next(), parts.next(), parts.next())
            else {
                return Err(format!("bad __fixups__ entry {r}"));
            };
            let at: usize = off.parse().map_err(|_| "bad __fixups__ offset")?;
            let n = ov
                .find(path)
                .ok_or_else(|| format!("__fixups__: no node {path}"))?;
            let v = ov.nodes[n]
                .props
                .iter_mut()
                .find(|(name, _)| name == prop)
                .ok_or_else(|| format!("__fixups__: no property {path}:{prop}"))?;
            put_be32(&mut v.1, at, handle)?;
        }
    }
    Ok(())
}

fn merge(base: &mut Fdt, target: usize, ov: &Fdt, src: usize) {
    for (name, value) in &ov.nodes[src].props {
        base.set_prop(target, name, value.clone());
    }
    for &c in &ov.nodes[src].children {
        let t = base.child_or_add(target, &ov.nodes[c].name);
        merge(base, t, ov, c);
    }
}

fn preserve_phandles(base: &Fdt, target: usize, ov: &mut Fdt, src: usize) -> Result<()> {
    if let (Some(old), Some(new)) = (base.phandle_of(target), ov.phandle_of(src)) {
        for name in ["phandle", "linux,phandle"] {
            if ov.prop(src, name).is_some() {
                ov.set_prop(src, name, old.to_be_bytes().to_vec());
            }
        }
        update_local_refs(ov, |p| Ok(if p == new { old } else { p }))?;
    }
    for c in ov.nodes[src].children.clone() {
        if let Some(t) = base.child(target, &ov.nodes[c].name) {
            preserve_phandles(base, t, ov, c)?;
        }
    }
    Ok(())
}

fn fragment_target(base: &Fdt, ov: &Fdt, frag: usize) -> Result<usize> {
    if let Some(c) = ov.cells(frag, "target") {
        let [h] = c[..] else {
            return Err(format!("{}: bad target", ov.path(frag)));
        };
        return base
            .phandle(h)
            .ok_or_else(|| format!("{}: target phandle {h:#x} not in the base", ov.path(frag)));
    }
    let path = ov
        .strings(frag, "target-path")
        .into_iter()
        .next()
        .ok_or_else(|| format!("{}: no target", ov.path(frag)))?;
    let node = if path.starts_with('/') {
        base.find(&path)
    } else {
        base.find("/aliases")
            .and_then(|a| base.strings(a, &path).into_iter().next())
            .and_then(|p| base.find(&p))
    };
    node.ok_or_else(|| format!("{}: target-path {path} not in the base", ov.path(frag)))
}

/// Apply `overlay` (a compiled `/plugin/` DTB with `-@` symbols) to `base` in place.
pub fn apply(base: &mut Fdt, overlay: &[u8]) -> Result<()> {
    let mut ov = Fdt::parse(overlay)?;
    renumber(&mut ov, base.max_phandle())?;
    resolve(base, &mut ov)?;
    // Resolve every target before touching the base (no partial application).
    let mut fragments = Vec::new();
    for &frag in &ov.nodes[0].children {
        if special(&ov.nodes[frag].name) {
            continue;
        }
        let Some(body) = ov.child(frag, "__overlay__") else {
            continue;
        };
        fragments.push((frag, fragment_target(base, &ov, frag)?, body));
    }
    let mut targets: Vec<(String, String)> = Vec::new();
    for (frag, target, body) in fragments {
        preserve_phandles(base, target, &mut ov, body)?;
        merge(base, target, &ov, body);
        targets.push((ov.path(frag), base.path(target)));
    }
    if let Some(symbols) = ov.find("/__symbols__") {
        let base_symbols = base.child_or_add(0, "__symbols__");
        for (label, value) in ov.nodes[symbols].props.clone() {
            let path = core::str::from_utf8(value.split(|c| *c == 0).next().unwrap_or(&[]))
                .map_err(|_| "bad __symbols__ entry")?;
            let Some((frag, rest)) = path
                .split_once("/__overlay__")
                .and_then(|(f, r)| targets.iter().find(|(p, _)| p == f).map(|(_, t)| (t, r)))
            else {
                continue;
            };
            let mut full = if frag == "/" {
                String::new()
            } else {
                frag.clone()
            };
            full.push_str(rest);
            if full.is_empty() {
                full.push('/');
            }
            let mut v = full.into_bytes();
            v.push(0);
            base.set_prop(base_symbols, &label, v);
        }
    }
    Ok(())
}
