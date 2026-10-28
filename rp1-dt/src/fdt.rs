//! Minimal flattened device tree (DTB v17) as an owned tree: parse, query, edit, serialize.
//! Used by host checks (`cargo rp1 check`) and by the BCM2712 loader (`rp1boot`) alike.
use crate::Result;
use alloc::{format, string::String, string::ToString, vec::Vec};

pub struct Node {
    pub name: String,
    pub parent: Option<usize>,
    pub children: Vec<usize>,
    pub props: Vec<(String, Vec<u8>)>,
}

pub struct Fdt {
    pub nodes: Vec<Node>,
    /// `/memreserve/` entries as (address, size).
    pub memreserve: Vec<(u64, u64)>,
    pub boot_cpuid: u32,
}

pub const MAGIC: u32 = 0xd00d_feed;

fn be32(b: &[u8], at: usize) -> Result<u32> {
    let end = at.checked_add(4).ok_or("FDT offset overflow")?;
    let s = b.get(at..end).ok_or("truncated FDT")?;
    Ok(u32::from_be_bytes([s[0], s[1], s[2], s[3]]))
}
fn be64(b: &[u8], at: usize) -> Result<u64> {
    Ok((u64::from(be32(b, at)?) << 32) | u64::from(be32(b, at + 4)?))
}
fn cstr(b: &[u8], at: usize) -> Result<&str> {
    let rest = b.get(at..).ok_or("truncated FDT string")?;
    let end = rest
        .iter()
        .position(|c| *c == 0)
        .ok_or("unterminated FDT string")?;
    core::str::from_utf8(&rest[..end]).map_err(|_| "non-UTF-8 FDT string".into())
}

/// `totalsize` of the DTB starting at `b` (checks the magic only).
pub fn total_size(b: &[u8]) -> Result<usize> {
    if be32(b, 0)? != MAGIC {
        return Err("not a DTB".into());
    }
    Ok(be32(b, 4)? as usize)
}

impl Fdt {
    pub fn parse(b: &[u8]) -> Result<Self> {
        let total = total_size(b)?;
        let b = b.get(..total).ok_or("truncated DTB")?;
        if be32(b, 20)? < 16 {
            return Err("DTB version < 16".into());
        }
        let structs = be32(b, 8)? as usize;
        let strings = be32(b, 12)? as usize;
        let boot_cpuid = be32(b, 28)?;
        let mut memreserve = Vec::new();
        let mut at = be32(b, 16)? as usize;
        loop {
            let (addr, size) = (be64(b, at)?, be64(b, at + 8)?);
            at += 16;
            if addr == 0 && size == 0 {
                break;
            }
            memreserve.push((addr, size));
        }
        let mut nodes: Vec<Node> = Vec::new();
        let mut stack: Vec<usize> = Vec::new();
        let mut at = structs;
        loop {
            let token = be32(b, at)?;
            at += 4;
            match token {
                1 => {
                    let name = cstr(b, at)?.to_string();
                    at = (at + name.len() + 1 + 3) & !3;
                    let parent = stack.last().copied();
                    if parent.is_none() && !nodes.is_empty() {
                        return Err("FDT has more than one root".into());
                    }
                    let id = nodes.len();
                    nodes.push(Node {
                        name,
                        parent,
                        children: Vec::new(),
                        props: Vec::new(),
                    });
                    if let Some(p) = parent {
                        nodes[p].children.push(id);
                    }
                    stack.push(id);
                }
                2 => {
                    stack.pop().ok_or("unbalanced FDT END_NODE")?;
                }
                3 => {
                    let len = be32(b, at)? as usize;
                    let name = cstr(b, strings + be32(b, at + 4)? as usize)?.to_string();
                    let data = b
                        .get(at + 8..at + 8 + len)
                        .ok_or("truncated FDT property")?
                        .to_vec();
                    at = (at + 8 + len + 3) & !3;
                    let node = *stack.last().ok_or("FDT property outside a node")?;
                    nodes[node].props.push((name, data));
                }
                4 => {}
                9 => break,
                t => return Err(format!("bad FDT token {t}")),
            }
        }
        if !stack.is_empty() || nodes.is_empty() {
            return Err("unbalanced FDT".into());
        }
        Ok(Self {
            nodes,
            memreserve,
            boot_cpuid,
        })
    }

    /// Serialize as DTB v17 (strings de-duplicated, nodes in tree order).
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut structs = Vec::new();
        let mut strings: Vec<u8> = Vec::new();
        let mut offsets: Vec<(String, u32)> = Vec::new();
        let mut stack = alloc::vec![(0usize, false)];
        while let Some((n, closing)) = stack.pop() {
            if closing {
                structs.extend_from_slice(&2u32.to_be_bytes());
                continue;
            }
            structs.extend_from_slice(&1u32.to_be_bytes());
            structs.extend_from_slice(self.nodes[n].name.as_bytes());
            structs.push(0);
            while structs.len() % 4 != 0 {
                structs.push(0);
            }
            for (name, value) in &self.nodes[n].props {
                let off = match offsets.iter().find(|(s, _)| s == name) {
                    Some((_, o)) => *o,
                    None => {
                        let o = strings.len() as u32;
                        strings.extend_from_slice(name.as_bytes());
                        strings.push(0);
                        offsets.push((name.clone(), o));
                        o
                    }
                };
                structs.extend_from_slice(&3u32.to_be_bytes());
                structs.extend_from_slice(&(value.len() as u32).to_be_bytes());
                structs.extend_from_slice(&off.to_be_bytes());
                structs.extend_from_slice(value);
                while structs.len() % 4 != 0 {
                    structs.push(0);
                }
            }
            stack.push((n, true));
            for &c in self.nodes[n].children.iter().rev() {
                stack.push((c, false));
            }
        }
        structs.extend_from_slice(&9u32.to_be_bytes());
        let rsv_off = 40usize;
        let rsv_len = (self.memreserve.len() + 1) * 16;
        let struct_off = rsv_off + rsv_len;
        let strings_off = struct_off + structs.len();
        let total = strings_off + strings.len();
        let mut out = Vec::with_capacity(total);
        for v in [
            MAGIC,
            total as u32,
            struct_off as u32,
            strings_off as u32,
            rsv_off as u32,
            17,
            16,
            self.boot_cpuid,
            strings.len() as u32,
            structs.len() as u32,
        ] {
            out.extend_from_slice(&v.to_be_bytes());
        }
        for (a, s) in self.memreserve.iter().chain([(0, 0)].iter()) {
            out.extend_from_slice(&a.to_be_bytes());
            out.extend_from_slice(&s.to_be_bytes());
        }
        out.extend_from_slice(&structs);
        out.extend_from_slice(&strings);
        out
    }

    pub fn prop(&self, node: usize, name: &str) -> Option<&[u8]> {
        self.nodes[node]
            .props
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v.as_slice())
    }
    /// Replace or append a property.
    pub fn set_prop(&mut self, node: usize, name: &str, value: Vec<u8>) {
        let props = &mut self.nodes[node].props;
        match props.iter_mut().find(|(n, _)| n == name) {
            Some((_, v)) => *v = value,
            None => props.push((name.to_string(), value)),
        }
    }
    pub fn remove_prop(&mut self, node: usize, name: &str) {
        self.nodes[node].props.retain(|(n, _)| n != name);
    }
    pub fn child(&self, node: usize, name: &str) -> Option<usize> {
        self.nodes[node]
            .children
            .iter()
            .copied()
            .find(|c| self.nodes[*c].name == name)
    }
    /// Existing child `name` of `node`, or a new empty one.
    pub fn child_or_add(&mut self, node: usize, name: &str) -> usize {
        if let Some(c) = self.child(node, name) {
            return c;
        }
        let id = self.nodes.len();
        self.nodes.push(Node {
            name: name.to_string(),
            parent: Some(node),
            children: Vec::new(),
            props: Vec::new(),
        });
        self.nodes[node].children.push(id);
        id
    }
    pub fn cells(&self, node: usize, name: &str) -> Option<Vec<u32>> {
        let v = self.prop(node, name)?;
        if !v.len().is_multiple_of(4) {
            return None;
        }
        Some(
            v.chunks(4)
                .map(|c| u32::from_be_bytes([c[0], c[1], c[2], c[3]]))
                .collect(),
        )
    }
    pub fn strings(&self, node: usize, name: &str) -> Vec<String> {
        self.prop(node, name)
            .map(|v| {
                v.split(|c| *c == 0)
                    .filter(|s| !s.is_empty())
                    .map(|s| String::from_utf8_lossy(s).into_owned())
                    .collect()
            })
            .unwrap_or_default()
    }
    pub fn path(&self, node: usize) -> String {
        let mut parts = Vec::new();
        let mut n = Some(node);
        while let Some(i) = n {
            if self.nodes[i].parent.is_some() {
                parts.push(self.nodes[i].name.as_str());
            }
            n = self.nodes[i].parent;
        }
        parts.reverse();
        format!("/{}", parts.join("/"))
    }
    pub fn find(&self, path: &str) -> Option<usize> {
        let mut node = 0;
        for part in path.split('/').filter(|p| !p.is_empty()) {
            node = self.child(node, part)?;
        }
        Some(node)
    }
    /// Node for a `__symbols__` label.
    pub fn label(&self, label: &str) -> Option<usize> {
        let symbols = self.find("/__symbols__")?;
        let path = self.strings(symbols, label).into_iter().next()?;
        self.find(&path)
    }
    pub fn phandle_of(&self, node: usize) -> Option<u32> {
        let c = self
            .cells(node, "phandle")
            .or_else(|| self.cells(node, "linux,phandle"))?;
        (c.len() == 1).then(|| c[0])
    }
    pub fn phandle(&self, handle: u32) -> Option<usize> {
        (0..self.nodes.len()).find(|n| self.phandle_of(*n) == Some(handle))
    }
    pub fn max_phandle(&self) -> u32 {
        (0..self.nodes.len())
            .filter_map(|n| self.phandle_of(n))
            .filter(|p| *p != u32::MAX)
            .max()
            .unwrap_or(0)
    }
    /// The node and all its ancestors have no status or status "okay"/"ok".
    pub fn enabled(&self, node: usize) -> bool {
        let mut n = Some(node);
        while let Some(i) = n {
            if let Some(s) = self.strings(i, "status").first()
                && s != "okay"
                && s != "ok"
            {
                return false;
            }
            n = self.nodes[i].parent;
        }
        true
    }
    pub fn is_descendant(&self, node: usize, ancestor: usize) -> bool {
        let mut n = Some(node);
        while let Some(i) = n {
            if i == ancestor {
                return true;
            }
            n = self.nodes[i].parent;
        }
        false
    }
    /// `reg` of `node` as (address, size) pairs, using the parent's cell counts.
    pub fn reg(&self, node: usize) -> Result<Vec<(u64, u64)>> {
        let parent = self.nodes[node].parent.ok_or("root has no reg")?;
        let ac = self
            .cells(parent, "#address-cells")
            .and_then(|c| c.first().copied())
            .unwrap_or(2) as usize;
        let sc = self
            .cells(parent, "#size-cells")
            .and_then(|c| c.first().copied())
            .unwrap_or(1) as usize;
        let cells = self.cells(node, "reg").unwrap_or_default();
        if ac == 0 || ac > 2 || sc > 2 || !cells.len().is_multiple_of(ac + sc) {
            return Err(format!("{}: unsupported reg layout", self.path(node)));
        }
        let join = |c: &[u32]| c.iter().fold(0u64, |a, v| (a << 32) | u64::from(*v));
        Ok(cells
            .chunks(ac + sc)
            .map(|c| (join(&c[..ac]), join(&c[ac..])))
            .collect())
    }
    /// A big-endian number stored in 1 or 2 cells (e.g. `/chosen/linux,initrd-start`).
    pub fn number(&self, node: usize, name: &str) -> Option<u64> {
        let v = self.prop(node, name)?;
        match v.len() {
            4 => Some(u64::from(u32::from_be_bytes([v[0], v[1], v[2], v[3]]))),
            8 => Some(u64::from_be_bytes([
                v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7],
            ])),
            _ => None,
        }
    }
}
