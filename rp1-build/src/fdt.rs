//! Minimal read-only flattened device tree (DTB v17) reader for host-side checks.
use crate::Result;

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
}

fn be32(b: &[u8], at: usize) -> Result<u32> {
    Ok(u32::from_be_bytes(
        b.get(at..at.checked_add(4).ok_or("FDT offset overflow")?)
            .ok_or("truncated FDT")?
            .try_into()?,
    ))
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
    Ok(std::str::from_utf8(&rest[..end])?)
}

impl Fdt {
    pub fn parse(b: &[u8]) -> Result<Self> {
        if be32(b, 0)? != 0xd00d_feed {
            return Err("not a DTB".into());
        }
        let total = be32(b, 4)? as usize;
        let b = b.get(..total).ok_or("truncated DTB")?;
        let structs = be32(b, 8)? as usize;
        let strings = be32(b, 12)? as usize;
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
                t => return Err(format!("bad FDT token {t}").into()),
            }
        }
        if !stack.is_empty() || nodes.is_empty() {
            return Err("unbalanced FDT".into());
        }
        Ok(Self { nodes, memreserve })
    }
    pub fn prop(&self, node: usize, name: &str) -> Option<&[u8]> {
        self.nodes[node]
            .props
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v.as_slice())
    }
    pub fn cells(&self, node: usize, name: &str) -> Option<Vec<u32>> {
        let v = self.prop(node, name)?;
        if !v.len().is_multiple_of(4) {
            return None;
        }
        Some(
            v.chunks(4)
                .map(|c| u32::from_be_bytes(c.try_into().unwrap()))
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
            node = *self.nodes[node]
                .children
                .iter()
                .find(|c| self.nodes[**c].name == part)?;
        }
        Some(node)
    }
    /// Node for a `__symbols__` label.
    pub fn label(&self, label: &str) -> Option<usize> {
        let symbols = self.find("/__symbols__")?;
        let path = self.strings(symbols, label).into_iter().next()?;
        self.find(&path)
    }
    pub fn phandle(&self, handle: u32) -> Option<usize> {
        (0..self.nodes.len()).find(|n| {
            self.cells(*n, "phandle")
                .or_else(|| self.cells(*n, "linux,phandle"))
                .is_some_and(|c| c == [handle])
        })
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
            return Err(format!("{}: unsupported reg layout", self.path(node)).into());
        }
        let join = |c: &[u32]| c.iter().fold(0u64, |a, v| (a << 32) | u64::from(*v));
        Ok(cells
            .chunks(ac + sc)
            .map(|c| (join(&c[..ac]), join(&c[ac..])))
            .collect())
    }
}
