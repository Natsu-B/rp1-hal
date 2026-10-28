//! Automatic ISRAM placement (design.md §3.3, v0.2): a call-graph plan for the second link.
//!
//! Roots are the functions already in `.isram` (manual `#[isram]`) and the exception handlers
//! of both vector tables. Everything statically reachable from a root by a direct branch is
//! planned into the common ISRAM image, except:
//! - boundaries: `#[cold]` functions (`.text.cold`) and panic / formatting paths. They stay in
//!   shared SRAM and the search does not continue through them;
//! - pre-ISRAM code: everything the reset path runs before the per-core ISRAM copy exists
//!   (`__rp1_reset*`, `__rp1_initialize` and their callees up to the core entry points).
//!
//! Indirect calls (function pointers, `dyn`) are not visible in the ELF and stay where they
//! are; that only costs speed, never correctness, because both cores carry the same ISRAM
//! image and shared SRAM code stays callable from anywhere.
use crate::Result;
use object::{Object, ObjectSection, ObjectSymbol, SymbolKind};
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    path::Path,
    process::Command,
};

pub const ISRAM_START: u64 = 0x1000_0000;
pub const ISRAM_SIZE: u64 = 8192;
/// Estimated size of one long-branch thunk lld adds per ISRAM → shared SRAM callee.
const THUNK_BYTES: u64 = 10;
const THUNK_PREFIXES: [&str; 3] = [
    "__Thumbv7ABSLongThunk_",
    "__ThumbV7PILongThunk_",
    "__ThumbV7ABSLongThunk_",
];
/// Startup symbols that run before ISRAM is populated, and where that path ends.
const PRE_ISRAM_ROOTS: [&str; 3] = ["__rp1_reset0", "__rp1_reset1", "__rp1_initialize"];
const PRE_ISRAM_STOP: [&str; 2] = ["__rp1_proc0_main", "__rp1_proc1_main"];

#[derive(Clone, Debug)]
pub struct Function {
    pub address: u64,
    pub size: u64,
    pub section: String,
}

#[derive(Debug, Default)]
pub struct Plan {
    /// Functions to move, in a stable order, with their sizes.
    pub functions: Vec<(String, u64)>,
    /// Bytes already in `.isram` (manual placement and thunks).
    pub current_bytes: u64,
    /// Predicted bytes added by the plan (functions, alignment, thunk estimate).
    pub planned_bytes: u64,
    /// Reachable functions left in shared SRAM because they are boundaries.
    pub boundaries: BTreeSet<String>,
    /// Reachable functions left in shared SRAM because the reset path needs them first.
    pub pre_isram: BTreeSet<String>,
}

impl Plan {
    /// The linker fragment included inside `.isram` (`rp1-isram-auto.x`).
    pub fn fragment(&self) -> String {
        let mut s = String::from("/* cargo rp1 build: automatic ISRAM plan */\n");
        for (name, _) in &self.functions {
            s.push_str(&format!("    *(.text.{name})\n"));
        }
        s
    }
    pub fn total_bytes(&self) -> u64 {
        self.current_bytes + self.planned_bytes
    }
}

/// Boundary by name: panics and formatting (large, cold, and pulling in `core::fmt`).
pub fn is_boundary_name(name: &str) -> bool {
    name.contains("panic")
        || name.contains("3fmt")
        || name.contains("_fail")
        || name.contains("failed")
        || name == "rust_begin_unwind"
}

fn strip_thunk(name: &str) -> &str {
    THUNK_PREFIXES
        .iter()
        .find_map(|p| name.strip_prefix(p))
        .unwrap_or(name)
}

/// Direct-branch call graph from `objdump -d` text: caller → callees (thunks resolved).
pub fn call_graph(text: &str) -> BTreeMap<String, BTreeSet<String>> {
    let mut graph: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut current: Option<String> = None;
    for line in text.lines() {
        let line = line.trim_end();
        if let Some(head) = line.strip_suffix(">:") {
            current = head
                .split_once(" <")
                .map(|(_, n)| strip_thunk(n).to_string());
            if let Some(c) = &current {
                graph.entry(c.clone()).or_default();
            }
            continue;
        }
        let Some(caller) = &current else { continue };
        let Some((_, rest)) = line.trim_start().split_once(':') else {
            continue;
        };
        let mut parts = rest.trim().splitn(2, char::is_whitespace);
        let op = parts.next().unwrap_or("");
        let args = parts.next().unwrap_or("");
        let branch = (op.starts_with('b')
            && !op.starts_with("bic")
            && !op.starts_with("bf")
            && !op.starts_with("bkpt"))
            || op.starts_with("cbz")
            || op.starts_with("cbnz");
        if !branch {
            continue;
        }
        let Some(start) = args.find('<') else {
            continue;
        };
        let Some(end) = args[start..].find('>') else {
            continue;
        };
        let target = &args[start + 1..start + end];
        let target = target.split('+').next().unwrap_or(target);
        let target = strip_thunk(target);
        if target != caller.as_str() && !target.starts_with(".L") {
            graph.get_mut(caller).unwrap().insert(target.to_string());
        }
    }
    graph
}

/// Plan from the parts (host-testable): symbols, call graph, vector handler names.
pub fn plan_from(
    functions: &BTreeMap<String, Function>,
    graph: &BTreeMap<String, BTreeSet<String>>,
    handlers: &BTreeSet<String>,
    current_bytes: u64,
) -> Plan {
    let empty = BTreeSet::new();
    let callees = |n: &str| graph.get(n).unwrap_or(&empty);
    // Pre-ISRAM closure of the reset path.
    let mut pre = BTreeSet::new();
    let mut queue: VecDeque<String> = PRE_ISRAM_ROOTS.iter().map(|s| s.to_string()).collect();
    while let Some(n) = queue.pop_front() {
        if PRE_ISRAM_STOP.contains(&n.as_str()) || !pre.insert(n.clone()) {
            continue;
        }
        queue.extend(callees(&n).iter().cloned());
    }
    let mut plan = Plan {
        current_bytes,
        ..Plan::default()
    };
    let mut seen = BTreeSet::new();
    let mut queue: VecDeque<String> = functions
        .iter()
        .filter(|(_, f)| f.section == ".isram")
        .map(|(n, _)| n.clone())
        .chain(handlers.iter().cloned())
        .collect();
    let mut planned = BTreeSet::new();
    let mut outgoing = BTreeSet::new();
    while let Some(n) = queue.pop_front() {
        if !seen.insert(n.clone()) {
            continue;
        }
        let Some(f) = functions.get(&n) else { continue };
        let in_isram = f.section == ".isram";
        if !in_isram {
            if f.section == ".text.cold" || is_boundary_name(&n) {
                plan.boundaries.insert(n);
                continue;
            }
            if pre.contains(&n) {
                plan.pre_isram.insert(n);
                continue;
            }
            if f.section != ".text" {
                continue;
            }
            planned.insert(n.clone());
        }
        queue.extend(callees(&n).iter().cloned());
    }
    // Edges that will still leave ISRAM after the move need a thunk each.
    for n in functions
        .iter()
        .filter(|(n, f)| f.section == ".isram" || planned.contains(*n))
        .map(|(n, _)| n)
    {
        for c in callees(n) {
            if functions.contains_key(c) && !planned.contains(c) {
                let in_isram = functions.get(c).is_some_and(|f| f.section == ".isram");
                if !in_isram {
                    outgoing.insert(c.clone());
                }
            }
        }
    }
    for n in planned {
        let size = functions[&n].size;
        plan.planned_bytes += size.next_multiple_of(4);
        plan.functions.push((n, size));
    }
    plan.planned_bytes += outgoing.len() as u64 * THUNK_BYTES;
    plan
}

/// Plan for a linked RP1 ELF.
pub fn plan(path: &Path, objdump: &Path) -> Result<Plan> {
    let bytes = std::fs::read(path)?;
    let file = object::File::parse(&*bytes)?;
    let mut functions = BTreeMap::new();
    for s in file.symbols() {
        if s.kind() != SymbolKind::Text || s.size() == 0 {
            continue;
        }
        let Some(index) = s.section_index() else {
            continue;
        };
        let section = file.section_by_index(index)?.name()?.to_string();
        functions.insert(
            s.name()?.to_string(),
            Function {
                address: s.address() & !1,
                size: s.size(),
                section,
            },
        );
    }
    let isram = file.section_by_name(".isram").ok_or("missing .isram")?;
    let vectors = file.section_by_name(".vectors").ok_or("missing .vectors")?;
    let data = vectors.data()?;
    let by_address: BTreeMap<u64, &str> = functions
        .iter()
        .map(|(n, f)| (f.address, n.as_str()))
        .collect();
    let mut handlers = BTreeSet::new();
    // Two 80-word tables, 512-byte aligned: skip the initial SP and the reset entry.
    for table in [0usize, 512] {
        for i in 2..80 {
            let Some(w) = data.get(table + i * 4..table + i * 4 + 4) else {
                break;
            };
            let a = u64::from(u32::from_le_bytes(w.try_into().unwrap()) & !1);
            if let Some(n) = by_address.get(&a) {
                handlers.insert(n.to_string());
            }
        }
    }
    let out = Command::new(objdump)
        .args(["-d", "--no-show-raw-insn"])
        .arg(path)
        .output()?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).to_string().into());
    }
    let graph = call_graph(&String::from_utf8(out.stdout)?);
    Ok(plan_from(&functions, &graph, &handlers, isram.size()))
}

/// After the second link: every planned function must now be in ISRAM.
pub fn verify(path: &Path, plan: &Plan) -> Result<()> {
    let bytes = std::fs::read(path)?;
    let file = object::File::parse(&*bytes)?;
    let placed: BTreeMap<String, u64> = file
        .symbols()
        .filter(|s| s.kind() == SymbolKind::Text)
        .filter_map(|s| Some((s.name().ok()?.to_string(), s.address() & !1)))
        .collect();
    let missing: Vec<_> = plan
        .functions
        .iter()
        .filter(|(n, _)| {
            !placed
                .get(n)
                .is_some_and(|a| (ISRAM_START..ISRAM_START + ISRAM_SIZE).contains(a))
        })
        .map(|(n, _)| n.as_str())
        .collect();
    if missing.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "planned functions not placed in ISRAM (no .text.<symbol> input section?): {}",
            missing.join(", ")
        )
        .into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DUMP: &str = "
10000000 <hot>:
10000000:\tpush\t{r7, lr}
10000002:\tbl\t10000010 <__Thumbv7ABSLongThunk_helper>
10000006:\tbne.w\t10000000 <hot+0x6>
1000000a:\tbl\t10000020 <__Thumbv7ABSLongThunk_slow_cold>
1000000e:\tb.w\t10000030 <__Thumbv7ABSLongThunk_core9panicking5panic>

20000400 <__rp1_reset0>:
20000400:\tb.w\t20000500 <__rp1_initialize>

20000500 <__rp1_initialize>:
20000500:\tbl\t20000600 <memcpy>
20000504:\tbl\t20000700 <__rp1_proc0_main>

20000600 <memcpy>:
20000600:\tbx\tlr

20000700 <__rp1_proc0_main>:
20000700:\tbl\t20000800 <helper>
20000704:\tbl\t20000900 <memset>

20000800 <helper>:
20000800:\tbl\t20000600 <memcpy>
20000804:\tcbz\tr0, 20000a00 <leaf>
20000806:\tbic\tr0, r0, #1
20000808:\tbx\tlr

20000a00 <leaf>:
20000a00:\tbx\tlr
";

    fn f(address: u64, size: u64, section: &str) -> Function {
        Function {
            address,
            size,
            section: section.into(),
        }
    }

    #[test]
    fn plan_follows_direct_calls_and_stops_at_boundaries() {
        let graph = call_graph(DUMP);
        assert_eq!(
            graph["hot"],
            ["core9panicking5panic", "helper", "slow_cold"]
                .into_iter()
                .map(String::from)
                .collect()
        );
        let functions: BTreeMap<String, Function> = [
            ("hot", f(0x1000_0000, 0x12, ".isram")),
            ("helper", f(0x2000_0800, 0xa, ".text")),
            ("leaf", f(0x2000_0a00, 2, ".text")),
            ("memcpy", f(0x2000_0600, 2, ".text")),
            ("memset", f(0x2000_0900, 2, ".text")),
            ("slow_cold", f(0x2000_0b00, 0x40, ".text.cold")),
            ("core9panicking5panic", f(0x2000_0c00, 0x40, ".text")),
            ("__rp1_reset0", f(0x2000_0400, 4, ".boot")),
            ("__rp1_initialize", f(0x2000_0500, 8, ".text")),
            ("__rp1_proc0_main", f(0x2000_0700, 8, ".text")),
        ]
        .into_iter()
        .map(|(n, f)| (n.to_string(), f))
        .collect();
        let plan = plan_from(&functions, &graph, &BTreeSet::new(), 0x14);
        let names: Vec<_> = plan.functions.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(names, ["helper", "leaf"]);
        assert!(plan.boundaries.contains("slow_cold"));
        assert!(plan.boundaries.contains("core9panicking5panic"));
        // memcpy runs before ISRAM is populated, so it stays in shared SRAM.
        assert_eq!(
            plan.pre_isram.iter().collect::<Vec<_>>(),
            [&"memcpy".to_string()]
        );
        // helper (12) + leaf (4) + thunks to memcpy, slow_cold, panic.
        assert_eq!(plan.planned_bytes, 12 + 4 + 3 * THUNK_BYTES);
        assert_eq!(
            plan.fragment().lines().skip(1).collect::<Vec<_>>(),
            ["    *(.text.helper)", "    *(.text.leaf)"]
        );
    }
}
