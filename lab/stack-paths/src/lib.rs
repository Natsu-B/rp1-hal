//! Static upper bound of stack use for Thumb-2 (Cortex-M3) code, from the text of
//! `arm-none-eabi-objdump -d --no-show-raw-insn` (contract §5.5).
//!
//! The analysis is fail-closed. A function has a bound only if every reachable instruction
//! changes SP by a known amount: `push`/`pop`, `stmdb sp!`/`ldmia sp!`, `sub|add sp, #imm`
//! and immediate pre/post-indexed `ldr|str` with SP as the base. Any other SP write, a
//! conditional SP change, recursion, an indirect call or jump that is not a linker thunk,
//! a jump table, `svc`, a branch to anything but an instruction of the same function or the
//! start of another function, or a path that falls off the end of a function is a [`Fail`].
//!
//! Choices, all conservative:
//! - The bound of a function is the deepest SP reached on any path, or the depth at a call
//!   plus the callee's bound. `bl` pushes nothing (LR is a register).
//! - A tail call (`b` to the start of another function, conditional or not) counts the
//!   callee's bound on top of the depth at the branch, whatever the epilogue released. If the
//!   callee returns, that depth must be 0, because it returns to our caller.
//! - A call to a function without a return path (for example `panic`) ends the path. The
//!   instructions after it are not analysed, which is what lets a `bl` be the last
//!   instruction of a function. A call to a function that can return and is followed by the
//!   end of the function fails (`falls-off-end`), and so does any path that would run into
//!   data (`falls-into-data`).
//! - Every return (`bx lr`, `pop {.., pc}`, `ldr pc, [sp], #4`) must be at depth 0, and two
//!   paths that meet must have the same depth.
//! - `bx rN`/`blx rN` resolve only when the two instructions before them are `movw rN` and
//!   `movt rN` and neither the branch nor the `movt` is a branch target of the function: the
//!   long-branch thunks the linker inserts between ISRAM and SRAM code.
//! - `tbb`/`tbh [pc, rN]` jump tables: the entries are the data bytes objdump shows between
//!   the instruction and the next instruction (the compiler's `$d` mapping symbol). This relies
//!   on the compiler's own range check or enum validity for the index. An entry that points
//!   back into the table is the alignment padding of an odd `tbb` table and is skipped; any
//!   other entry must be an instruction of the same function.
//! - `udf` and `bkpt` end the path. The HardFault they raise is its own root.
//! - Exception entry frames are not part of a handler's bound; the caller adds them.

use std::collections::{BTreeMap, HashMap, HashSet};

pub mod size;

/// Exception entry frame: 32 B of state plus 4 B of alignment padding, no FPU (contract §5.5).
pub const FRAME: u32 = 36;

/// Why there is no bound, and where (`symbol+0xoffset`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fail {
    pub reason: &'static str,
    pub at: String,
}

/// Bound of one function: the deepest stack below its entry SP, and whether it can return.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bound {
    pub bytes: u32,
    pub returns: bool,
}

struct Insn {
    address: u32,
    op: String,
    args: String,
    /// Data (or an elided `...` block) lies between the previous instruction and this one.
    after_data: bool,
}

struct Function {
    name: String,
    /// Index range into `Program::insns`.
    start: usize,
    end: usize,
}

/// The disassembly, split into functions at each `<symbol>:` label.
pub struct Program {
    insns: Vec<Insn>,
    functions: Vec<Function>,
    index: HashMap<u32, usize>,
    by_start: HashMap<u32, usize>,
    /// Bytes of the `.word`/`.short`/`.byte` lines (literal pools and jump tables).
    data: BTreeMap<u32, u8>,
}

/// What an instruction does to SP and to control flow.
enum Flow {
    Next,
    Branch(u32),
    Call(u32),
    Return,
    Trap,
    /// `tbb` (`half = false`) or `tbh`.
    Table {
        half: bool,
    },
    /// `bx rN` (`call = false`) or `blx rN` (`call = true`).
    Indirect {
        reg: usize,
        call: bool,
    },
}

struct Effect {
    /// Bytes pushed (negative: released).
    push: i64,
    flow: Flow,
    conditional: bool,
}

const CONDITIONS: [&str; 17] = [
    "eq", "ne", "cs", "hs", "cc", "lo", "mi", "pl", "vs", "vc", "hi", "ls", "ge", "lt", "gt", "le",
    "al",
];
const SP: usize = 13;
const PC: usize = 15;

impl Program {
    pub fn parse(text: &str) -> Self {
        let mut insns = Vec::new();
        let mut labels: Vec<(String, usize)> = Vec::new();
        let mut data = BTreeMap::new();
        let mut gap = false;
        for line in text.lines() {
            let line = line.trim_end();
            if line.trim() == "..." {
                gap = true;
                continue;
            }
            if let Some(label) = line.strip_suffix(">:")
                && let Some((_, name)) = label.split_once(" <")
            {
                labels.push((name.to_string(), insns.len()));
                continue;
            }
            let Some((a, rest)) = line.trim_start().split_once(':') else {
                continue;
            };
            let Ok(address) = u32::from_str_radix(a, 16) else {
                continue;
            };
            let mut p = rest.trim().splitn(2, char::is_whitespace);
            let op = p.next().unwrap_or("");
            if let Some(width) = match op {
                ".word" => Some(4),
                ".short" => Some(2),
                ".byte" => Some(1),
                _ => None,
            } && let Some(value) = p
                .next()
                .and_then(|v| immediate(v.split('@').next().unwrap()))
            {
                for k in 0..width {
                    data.insert(address + k, (value >> (8 * k)) as u8);
                }
                gap = true;
                continue;
            }
            if op.starts_with('.') {
                gap = true;
            }
            if op.is_empty() || op.starts_with('.') || labels.is_empty() {
                continue;
            }
            let args = p.next().unwrap_or("").split('@').next().unwrap().trim();
            insns.push(Insn {
                address,
                op: op.split('.').next().unwrap().to_ascii_lowercase(),
                args: args.to_string(),
                after_data: std::mem::take(&mut gap),
            });
        }
        let mut functions = Vec::new();
        for (k, (name, start)) in labels.iter().enumerate() {
            let end = labels.get(k + 1).map_or(insns.len(), |l| l.1);
            if *start < end {
                functions.push(Function {
                    name: name.clone(),
                    start: *start,
                    end,
                });
            }
        }
        let index = insns
            .iter()
            .enumerate()
            .map(|(i, x)| (x.address, i))
            .collect();
        let by_start = functions
            .iter()
            .enumerate()
            .map(|(f, x)| (insns[x.start].address, f))
            .collect();
        Program {
            insns,
            functions,
            index,
            by_start,
            data,
        }
    }

    /// Targets of the jump table after the `tbb`/`tbh` at index `i`.
    fn table(&self, i: usize, half: bool) -> Option<Vec<u32>> {
        let start = self.insns[i].address + 4;
        let end = self.insns.get(i + 1)?.address;
        let bytes: Vec<u8> = (start..end)
            .map(|a| self.data.get(&a).copied())
            .collect::<Option<_>>()?;
        let entries: Vec<u32> = if half {
            if !bytes.len().is_multiple_of(2) {
                return None;
            }
            bytes
                .chunks(2)
                .map(|c| u32::from(u16::from_le_bytes([c[0], c[1]])))
                .collect()
        } else {
            bytes.iter().map(|&b| u32::from(b)).collect()
        };
        Some(
            entries
                .into_iter()
                .map(|e| start + 2 * e)
                .filter(|t| !(start..end).contains(t))
                .collect(),
        )
    }

    /// The symbol objdump shows for a function starting at `address` (Thumb bit ignored).
    pub fn name_at(&self, address: u32) -> Option<&str> {
        self.by_start
            .get(&(address & !1))
            .map(|&f| self.functions[f].name.as_str())
    }

    /// The function starting at `name`.
    pub fn address_of(&self, name: &str) -> Option<u32> {
        let f = self.functions.iter().find(|f| f.name == name)?;
        Some(self.insns[f.start].address)
    }

    /// The stack bound of the function starting at `address` (Thumb bit ignored).
    pub fn bound(&self, address: u32) -> Result<Bound, Fail> {
        self.bound_cut(address, &HashSet::new())
    }

    /// Diagnostic only, never a proof: like [`Program::bound`], but a call or tail call to a
    /// function in `cut` counts 0 bytes and does not return.
    pub fn bound_cut(&self, address: u32, cut: &HashSet<u32>) -> Result<Bound, Fail> {
        let f = *self.by_start.get(&(address & !1)).ok_or_else(|| Fail {
            reason: "unknown-root",
            at: format!("0x{address:08x}"),
        })?;
        Analyzer {
            program: self,
            done: HashMap::new(),
            active: Vec::new(),
            cut,
        }
        .function(f)
    }

    fn location(&self, i: usize) -> String {
        let address = self.insns[i].address;
        let f = self
            .functions
            .iter()
            .find(|f| (f.start..f.end).contains(&i))
            .unwrap();
        format!("{}+0x{:x}", f.name, address - self.insns[f.start].address)
    }
}

struct Analyzer<'a> {
    program: &'a Program,
    done: HashMap<usize, Bound>,
    active: Vec<usize>,
    cut: &'a HashSet<u32>,
}

impl Analyzer<'_> {
    fn function(&mut self, f: usize) -> Result<Bound, Fail> {
        if let Some(b) = self.done.get(&f) {
            return Ok(*b);
        }
        self.active.push(f);
        let result = self.walk(f);
        self.active.pop();
        let bound = result?;
        self.done.insert(f, bound);
        Ok(bound)
    }

    /// Callee or tail-call target: a function start, analysed unless it is on the call path.
    fn callee(&mut self, target: u32, at: usize) -> Result<Bound, Fail> {
        let p = self.program;
        let fail = |reason| Fail {
            reason,
            at: p.location(at),
        };
        if self.cut.contains(&target) {
            return Ok(Bound {
                bytes: 0,
                returns: false,
            });
        }
        let f = *p
            .by_start
            .get(&target)
            .ok_or_else(|| fail("bad-call-target"))?;
        if self.active.contains(&f) {
            return Err(fail("recursion"));
        }
        self.function(f)
    }

    fn walk(&mut self, f: usize) -> Result<Bound, Fail> {
        let p = self.program;
        let func = &p.functions[f];
        let targets = local_targets(p, func);
        let mut depth: Vec<Option<u32>> = vec![None; func.end - func.start];
        let mut work = vec![(func.start, 0u32)];
        let mut bound = Bound {
            bytes: 0,
            returns: false,
        };
        while let Some((i, d)) = work.pop() {
            let fail = |reason| Fail {
                reason,
                at: p.location(i),
            };
            match depth[i - func.start] {
                Some(old) if old == d => continue,
                Some(_) => return Err(fail("sp-mismatch")),
                None => depth[i - func.start] = Some(d),
            }
            let mut e = effect(&p.insns[i]).map_err(fail)?;
            if let Flow::Indirect { reg, call } = e.flow {
                let t = thunk(p, func, &targets, i, reg).ok_or_else(|| {
                    fail(if call {
                        "unresolved-indirect-call"
                    } else {
                        "unresolved-indirect-jump"
                    })
                })?;
                e.flow = if call { Flow::Call(t) } else { Flow::Branch(t) };
            }
            if e.conditional && e.push != 0 && !matches!(e.flow, Flow::Return) {
                return Err(fail("conditional-sp"));
            }
            let after = i64::from(d) + e.push;
            if after < 0 {
                return Err(fail("sp-above-entry"));
            }
            let after = u32::try_from(after).map_err(|_| fail("sp-overflow"))?;
            bound.bytes = bound.bytes.max(after);
            // Depth on the fall-through path: a conditional return does not release.
            let next_depth = if e.conditional { d } else { after };
            let mut next = !matches!(e.flow, Flow::Return | Flow::Trap) || e.conditional;
            match e.flow {
                Flow::Next | Flow::Trap | Flow::Indirect { .. } => {}
                Flow::Table { half } => {
                    let targets = p.table(i, half).ok_or_else(|| fail("jump-table"))?;
                    if targets.is_empty() {
                        return Err(fail("jump-table"));
                    }
                    for t in targets {
                        match p.index.get(&t) {
                            Some(&j) if (func.start..func.end).contains(&j) => {
                                work.push((j, after))
                            }
                            _ => return Err(fail("jump-table")),
                        }
                    }
                    next = false;
                }
                Flow::Return => {
                    if after != 0 {
                        return Err(fail("unbalanced-return"));
                    }
                    bound.returns = true;
                }
                Flow::Call(target) => {
                    let c = self.callee(target, i)?;
                    bound.bytes = bound.bytes.max(after + c.bytes);
                    next = c.returns || e.conditional;
                }
                Flow::Branch(target) => {
                    if let Some(&j) = p.index.get(&target)
                        && (func.start..func.end).contains(&j)
                    {
                        work.push((j, after));
                    } else {
                        let c = self.callee(target, i)?;
                        bound.bytes = bound.bytes.max(after + c.bytes);
                        if c.returns {
                            if after != 0 {
                                return Err(fail("unbalanced-tail-call"));
                            }
                            bound.returns = true;
                        }
                    }
                    next = e.conditional;
                }
            }
            if next {
                if i + 1 >= func.end {
                    return Err(fail("falls-off-end"));
                }
                if p.insns[i + 1].after_data {
                    return Err(fail("falls-into-data"));
                }
                work.push((i + 1, next_depth));
            }
        }
        Ok(bound)
    }
}

/// Addresses that a direct branch of the function jumps to.
fn local_targets(p: &Program, func: &Function) -> HashSet<u32> {
    p.insns[func.start..func.end]
        .iter()
        .filter_map(|x| match effect(x) {
            Ok(Effect {
                flow: Flow::Branch(t),
                ..
            }) => Some(t),
            _ => None,
        })
        .collect()
}

/// `movw rN, #lo; movt rN, #hi; bx|blx rN` with no branch into the sequence: the target.
fn thunk(
    p: &Program,
    func: &Function,
    targets: &HashSet<u32>,
    i: usize,
    reg: usize,
) -> Option<u32> {
    if i < func.start + 2 {
        return None;
    }
    let (w, t) = (&p.insns[i - 2], &p.insns[i - 1]);
    if targets.contains(&t.address) || targets.contains(&p.insns[i].address) {
        return None;
    }
    let half = |x: &Insn, op: &str| {
        let (r, v) = x.args.split_once(',')?;
        (x.op == op && parse_reg(r) == Some(reg)).then(|| immediate(v))?
    };
    let value = half(t, "movt")? << 16 | half(w, "movw")?;
    (value & 1 == 1).then_some(value & !1)
}

fn condition(op: &str, base: &str) -> Option<bool> {
    let rest = op.strip_prefix(base)?;
    if rest.is_empty() {
        Some(false)
    } else {
        CONDITIONS.contains(&rest).then_some(true)
    }
}

fn parse_reg(s: &str) -> Option<usize> {
    match s.trim().trim_end_matches('!') {
        // objdump's aliases for r9..r12.
        "sb" => Some(9),
        "sl" => Some(10),
        "fp" => Some(11),
        "ip" => Some(12),
        "sp" => Some(SP),
        "lr" => Some(14),
        "pc" => Some(PC),
        s => s.strip_prefix('r')?.parse().ok().filter(|n| *n < 16),
    }
}

fn immediate(s: &str) -> Option<u32> {
    let s = s.trim().trim_start_matches('#');
    let (negative, s) = match s.strip_prefix('-') {
        Some(s) => (true, s),
        None => (false, s),
    };
    let v = match s.strip_prefix("0x") {
        Some(h) => u32::from_str_radix(h, 16).ok()?,
        None => s.parse().ok()?,
    };
    Some(if negative { v.wrapping_neg() } else { v })
}

fn target(s: &str) -> Option<u32> {
    u32::from_str_radix(s.split_whitespace().next()?, 16).ok()
}

/// Registers in `{r4, r5-r7, lr}`.
fn reg_list(args: &str) -> Result<Vec<usize>, &'static str> {
    let list = args
        .split_once('{')
        .and_then(|(_, l)| l.split_once('}'))
        .ok_or("unknown-instruction")?
        .0;
    let mut regs = Vec::new();
    for item in list.split(',') {
        match item.split_once('-') {
            Some((a, b)) => {
                let (a, b) = (parse_reg(a), parse_reg(b));
                regs.extend(a.zip(b).map(|(a, b)| a..=b).ok_or("unknown-instruction")?);
            }
            None => regs.push(parse_reg(item).ok_or("unknown-instruction")?),
        }
    }
    Ok(regs)
}

fn effect(x: &Insn) -> Result<Effect, &'static str> {
    let op = x.op.as_str();
    let args = x.args.as_str();
    let ops: Vec<&str> = args.split(',').map(str::trim).collect();
    let first = parse_reg(ops[0]);
    let simple = |push: i64, flow: Flow, conditional: bool| {
        Ok(Effect {
            push,
            flow,
            conditional,
        })
    };
    if op.starts_with("it") && op[2..].chars().all(|c| c == 't' || c == 'e') {
        return simple(0, Flow::Next, false);
    }
    // Longest name first: `bls` is `b` + `ls`, `bleq` is `bl` + `eq`.
    if let Some(c) = condition(op, "blx") {
        let reg = first.ok_or("blx-immediate")?;
        return simple(0, Flow::Indirect { reg, call: true }, c);
    }
    if let Some(c) = condition(op, "bx") {
        let reg = first.ok_or("unknown-instruction")?;
        let flow = if reg == 14 {
            Flow::Return
        } else {
            Flow::Indirect { reg, call: false }
        };
        return simple(0, flow, c);
    }
    if let Some(c) = condition(op, "bl") {
        return simple(0, Flow::Call(target(args).ok_or("unknown-instruction")?), c);
    }
    if let Some(c) = condition(op, "b") {
        return simple(
            0,
            Flow::Branch(target(args).ok_or("unknown-instruction")?),
            c,
        );
    }
    if op == "cbz" || op == "cbnz" {
        let t = ops
            .get(1)
            .and_then(|t| target(t))
            .ok_or("unknown-instruction")?;
        return simple(0, Flow::Branch(t), true);
    }
    if op == "tbb" || op == "tbh" {
        if !args.trim_start().starts_with("[pc,") {
            return Err("jump-table");
        }
        return simple(0, Flow::Table { half: op == "tbh" }, false);
    }
    if op == "svc" {
        return Err("svc");
    }
    if op == "udf" || op == "bkpt" {
        return simple(0, Flow::Trap, false);
    }
    let stack_list = |base: &str| args.split_once(',').is_some_and(|(b, _)| b.trim() == base);
    let pushes = condition(op, "push").or_else(|| {
        ["stmdb", "stmfd"]
            .iter()
            .find_map(|b| condition(op, b))
            .filter(|_| stack_list("sp!"))
    });
    if let Some(c) = pushes {
        let regs = reg_list(args)?;
        if regs.contains(&SP) || regs.contains(&PC) {
            return Err("unknown-instruction");
        }
        return simple(4 * regs.len() as i64, Flow::Next, c);
    }
    let pops = condition(op, "pop").or_else(|| {
        ["ldmia", "ldm", "ldmfd"]
            .iter()
            .find_map(|b| condition(op, b))
            .filter(|_| stack_list("sp!"))
    });
    if let Some(c) = pops {
        let regs = reg_list(args)?;
        if regs.contains(&SP) {
            return Err("sp-write");
        }
        let flow = if regs.contains(&PC) {
            Flow::Return
        } else {
            Flow::Next
        };
        return simple(-4 * regs.len() as i64, flow, c);
    }
    if op.starts_with("ldm") || op.starts_with("stm") {
        // Any other multiple transfer: SP may be neither the written-back base nor in the list.
        let (base, _) = args.split_once(',').ok_or("unknown-instruction")?;
        let regs = reg_list(args)?;
        if parse_reg(base) == Some(SP) && base.trim().ends_with('!') {
            return Err("variable-sp");
        }
        if op.starts_with("ldm") && regs.contains(&SP) {
            return Err("sp-write");
        }
        if op.starts_with("ldm") && regs.contains(&PC) {
            return Err("unresolved-indirect-jump");
        }
        return simple(0, Flow::Next, false);
    }
    if op.starts_with("ldr") || op.starts_with("str") {
        return load_store(op, args);
    }
    if op.starts_with("msr") {
        let special = ops[0].to_ascii_lowercase();
        if special.starts_with("msp") || special.starts_with("psp") {
            return Err("sp-write");
        }
        return simple(0, Flow::Next, false);
    }
    // Instructions whose first operand is read, not written.
    let reads_first = ["cmp", "cmn", "tst", "teq", "pld", "pli"]
        .iter()
        .any(|b| condition(op, b).is_some());
    if first == Some(SP) && !reads_first {
        let add = condition(op, "add").or_else(|| condition(op, "addw"));
        let sub = condition(op, "sub").or_else(|| condition(op, "subw"));
        let (sign, c) = match (add, sub) {
            (Some(c), _) => (-1, c),
            (_, Some(c)) => (1, c),
            _ => return Err("sp-write"),
        };
        // `sub sp, #imm` or `sub sp, sp, #imm`.
        let imm = match ops.as_slice() {
            [_, i] => immediate(i),
            [_, s, i] if parse_reg(s) == Some(SP) => immediate(i),
            _ => None,
        }
        .ok_or("variable-sp")?;
        return simple(sign * i64::from(imm as i32), Flow::Next, c);
    }
    if first == Some(PC) && !reads_first {
        return Err("unresolved-indirect-jump");
    }
    simple(0, Flow::Next, false)
}

/// `ldr*`/`str*`: SP changes only through an immediate pre- or post-indexed SP base.
fn load_store(op: &str, args: &str) -> Result<Effect, &'static str> {
    let (regs, address) = args.split_once('[').ok_or("unknown-instruction")?;
    let (inside, after) = address.split_once(']').ok_or("unknown-instruction")?;
    let mut inside = inside.split(',').map(str::trim);
    let base = parse_reg(inside.next().unwrap_or("")).ok_or("unknown-instruction")?;
    let offset = inside.next();
    let after = after.trim();
    let load = op.starts_with("ldr");
    let dests: Vec<usize> = regs.split(',').filter_map(parse_reg).collect();
    // strex writes its status to the first register.
    let written: &[usize] = if load {
        &dests
    } else if op.starts_with("strex") {
        &dests[..dests.len().min(1)]
    } else {
        &[]
    };
    if written.contains(&SP) {
        return Err("sp-write");
    }
    let conditional = CONDITIONS.iter().any(|c| op.ends_with(c));
    let push = if base != SP {
        0
    } else if after == "!" {
        // Pre-indexed: `str r4, [sp, #-4]!`.
        -i64::from(offset.and_then(immediate).ok_or("variable-sp")? as i32)
    } else if let Some(post) = after.strip_prefix(',') {
        // Post-indexed: `ldr r4, [sp], #4`.
        -i64::from(immediate(post).ok_or("variable-sp")? as i32)
    } else {
        0
    };
    if written.contains(&PC) {
        // `ldr pc, [sp], #4` is `pop {pc}`; any other load into PC is an indirect jump.
        if base == SP && push == -4 && dests.len() == 1 {
            return Ok(Effect {
                push,
                flow: Flow::Return,
                conditional,
            });
        }
        return Err("unresolved-indirect-jump");
    }
    Ok(Effect {
        push,
        flow: Flow::Next,
        conditional,
    })
}

/// Stack bounds of the proc0 platform handlers (contract §5.5), entry frames excluded.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Handlers {
    pub h57: u32,
    pub h53: u32,
    pub hhard: u32,
    /// The deepest configurable fault handler (MemManage, BusFault, UsageFault).
    pub hfault: u32,
    /// The NMI handler, if the NMI vector holds one other than the default handler.
    pub hnmi: Option<u32>,
}

impl Handlers {
    /// The platform's share of the proc0 stack: `max(H57, Hfault) + H53 + Hhard + 3 * F`, plus
    /// `Hnmi + F` for an NMI handler, which can preempt all of them.
    pub fn s_plat(&self) -> u32 {
        let nmi = self.hnmi.map_or(0, |h| h + FRAME);
        self.h57.max(self.hfault) + self.h53 + self.hhard + 3 * FRAME + nmi
    }
}

/// proc0's stack as the image declares it: `__stack_size0` (the whole stack) and
/// `__stack_margin0` (the platform's part of it, from rp1-build).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Proc0Stack {
    pub stack0: u32,
    pub margin: u32,
}

/// The `stack_static=` line for the handler bounds, and whether it is a PASS: `S_plat` must fit
/// in the image's platform margin, and the margin must leave room for the user's stack. A
/// `diagnostic` run (`--cut`) is never a PASS.
pub fn verdict(h: &Handlers, stack: Proc0Stack, diagnostic: bool) -> (bool, String) {
    let s = h.s_plat();
    let Proc0Stack { stack0, margin } = stack;
    let hnmi = h.hnmi.map_or("none".into(), |n| n.to_string());
    let terms = format!(
        "s_plat={s} h57={} h53={} hhard={} hfault={} hnmi={hnmi} margin={margin} stack0={stack0}",
        h.h57, h.h53, h.hhard, h.hfault
    );
    if diagnostic {
        (false, format!("stack_static=DIAG {terms}"))
    } else if margin >= stack0 {
        (
            false,
            format!("stack_static=FAIL reason=proc0-stack {terms}"),
        )
    } else if s > margin {
        (false, format!("stack_static=FAIL reason=margin {terms}"))
    } else {
        (true, format!("stack_static=PASS {terms}"))
    }
}

#[cfg(test)]
mod tests;
