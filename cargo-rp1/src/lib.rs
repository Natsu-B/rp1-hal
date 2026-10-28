use object::{Object, ObjectSection, ObjectSymbol};
use rp1_proto::note::{self, Note};
use std::{
    collections::{BTreeMap, VecDeque},
    path::Path,
    process::Command,
};
pub mod boot;
pub mod isram;
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
#[derive(Debug)]
pub struct Report {
    pub image_bytes: u64,
    pub isram_bytes: u64,
    pub dsram_bytes: u64,
    pub exclusive_instructions: usize,
}
fn word(b: &[u8], p: usize) -> Result<u32> {
    note::u32_at(b, p).map_err(|_| "truncated ELF".into())
}
fn short(b: &[u8], p: usize) -> Result<u16> {
    note::u16_at(b, p).map_err(|_| "truncated ELF".into())
}
fn within(base: u64, len: u64, start: u64, end: u64) -> bool {
    base >= start && base.checked_add(len).is_some_and(|e| e <= end)
}

pub fn descriptor(bytes: &[u8]) -> Result<&[u8]> {
    let desc = note::descriptor(bytes).map_err(|_| "invalid ELF note envelope/length/padding")?;
    Note::parse(desc).map_err(|e| format!("invalid note: {e:?}"))?;
    Ok(desc)
}
pub fn check(path: &Path, objdump: &Path) -> Result<Report> {
    let bytes = std::fs::read(path)?;
    if bytes.get(..7) != Some(&b"\x7fELF\x01\x01\x01"[..])
        || short(&bytes, 16)? != 2
        || short(&bytes, 18)? != 40
    {
        return Err("expected an ARM ELF32 little-endian executable".into());
    }
    let file = object::File::parse(&*bytes)?;
    let sec = file
        .section_by_name(".note.rp1")
        .ok_or("missing .note.rp1")?;
    if sec.kind() != object::SectionKind::Note {
        return Err(".note.rp1 must be SHT_NOTE".into());
    }
    let note = Note::parse(descriptor(sec.data()?)?).map_err(|e| format!("note parse: {e:?}"))?;
    let entry = note.get(note::ENTRY).ok_or("missing entry")?.value;
    if file.entry() != u64::from(word(entry, 0)?)
        || word(entry, 4)? != 0x1000_4000
        || word(entry, 8)? != 0x2000_0000
        || word(entry, 12)? != rp1_proto::SRAM_START
        || word(entry, 16)? != rp1_proto::IMAGE_END
    {
        return Err("note ENTRY disagrees with ELF memory contract".into());
    }
    let vectors = file.section_by_name(".vectors").ok_or("missing vectors")?;
    if vectors.address() != u64::from(word(entry, 8)?)
        || word(vectors.data()?, 0)? != word(entry, 4)?
        || word(vectors.data()?, 4)? != word(entry, 0)?
    {
        return Err("initial vectors disagree with note".into());
    }
    let phoff = word(&bytes, 28)? as usize;
    let phsize = short(&bytes, 42)? as usize;
    let phnum = short(&bytes, 44)? as usize;
    if phsize != 32 || phnum == 0 {
        return Err("invalid program header table".into());
    }
    let mut loads = Vec::new();
    let mut image_end = u64::from(rp1_proto::SRAM_START);
    let mut isram = 0;
    let mut dsram = 0;
    for i in 0..phnum {
        let h = phoff
            .checked_add(i.checked_mul(phsize).ok_or("program header overflow")?)
            .ok_or("program header overflow")?;
        let typ = word(&bytes, h)?;
        if typ == 2 || typ == 3 {
            return Err("dynamic ELF is not supported".into());
        }
        if typ != 1 {
            continue;
        }
        let offset = u64::from(word(&bytes, h + 4)?);
        let va = u64::from(word(&bytes, h + 8)?);
        let pa = u64::from(word(&bytes, h + 12)?);
        let size = u64::from(word(&bytes, h + 16)?);
        let mem = u64::from(word(&bytes, h + 20)?);
        if size > mem
            || offset
                .checked_add(size)
                .is_none_or(|end| end > bytes.len() as u64)
        {
            return Err("invalid load extent".into());
        }
        if mem == 0 {
            continue;
        }
        if within(va, mem, 0x1000_0000, 0x1000_2000) {
            isram = isram.max(va + mem - 0x1000_0000);
        } else if within(va, mem, 0x1000_2000, 0x1000_4000) {
            dsram = dsram.max(va + mem - 0x1000_2000);
        } else if !within(va, mem, 0x2000_0000, 0x2000_c000) {
            return Err(format!("load VMA 0x{va:x}+0x{mem:x} exceeds memory budget").into());
        }
        // The loader materializes every PT_LOAD at p_paddr for p_memsz bytes (zero-filling the tail)
        // and rejects overlapping ranges, so the LMA footprint is memsz, not filesz.
        if pa != va && mem != size {
            return Err(format!("relocated load 0x{va:x} has memsz != filesz; the loader would zero shared SRAM at its LMA").into());
        }
        if size != 0 || pa == va {
            if !within(pa, mem, 0x2000_0000, 0x2000_c000) {
                return Err("load LMA exceeds shared SRAM image window".into());
            }
            for &(start, end) in &loads {
                if pa < end && pa + mem > start {
                    return Err("overlapping load images".into());
                }
            }
            loads.push((pa, pa + mem));
            image_end = image_end.max(pa + mem);
            if let Some((start, len)) = sec.file_range()
                && offset < start + len
                && offset + size > start
            {
                return Err("note must not be in a PT_LOAD".into());
            }
        }
    }
    // .bss is NOLOAD and outside every PT_LOAD; take the DSRAM high-water mark from the linker symbol.
    let ebss = file
        .symbols()
        .find(|s| s.name() == Ok("__ebss"))
        .ok_or("missing __ebss")?
        .address();
    if !within(ebss, 0, 0x1000_2000, 0x1000_4000) {
        return Err("__ebss outside DSRAM".into());
    }
    dsram = dsram.max(ebss - 0x1000_2000);
    for name in ["__stack_size0", "__stack_size1"] {
        let stack = file
            .symbols()
            .find(|s| s.name() == Ok(name))
            .ok_or("missing stack budget symbol")?
            .address();
        if stack == 0 || stack % 8 != 0 || dsram + stack > 8192 {
            return Err("DSRAM + stack budget exceeded".into());
        }
    }
    validate_regions(&note)?;
    let overlay = file
        .section_by_name(".rp1.dtbo")
        .ok_or("missing embedded overlay")?;
    if overlay.data()?.get(..4) != Some(&[0xd0, 0x0d, 0xfe, 0xed]) {
        return Err("invalid embedded overlay".into());
    }
    let out = Command::new(objdump)
        .args(["-d", "--no-show-raw-insn"])
        .arg(path)
        .output()?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).to_string().into());
    }
    let exclusive_instructions = check_exclusives(&String::from_utf8(out.stdout)?, |address| {
        // Only immutable bytes count as constants: a literal in .data/.shared/.bss has an
        // initial value in the ELF but can be changed at run time (review #2).
        for section in file.sections().filter(|s| literal_section(s)) {
            let start = section.address();
            if within(
                u64::from(address),
                4,
                start,
                start.checked_add(section.size())?,
            ) {
                return note::u32_at(section.data().ok()?, (u64::from(address) - start) as usize)
                    .ok();
            }
        }
        None
    })?;
    Ok(Report {
        image_bytes: image_end - 0x2000_0000,
        isram_bytes: isram,
        dsram_bytes: dsram,
        exclusive_instructions,
    })
}
/// A section whose contents cannot change at run time: allocated, not writable and not NOBITS.
fn literal_section(section: &object::Section<'_, '_>) -> bool {
    use object::{SectionFlags, SectionKind, elf};
    matches!(
        section.kind(),
        SectionKind::Text | SectionKind::ReadOnlyData | SectionKind::ReadOnlyString
    ) && match section.flags() {
        SectionFlags::Elf { sh_flags } => {
            sh_flags & u64::from(elf::SHF_ALLOC) != 0 && sh_flags & u64::from(elf::SHF_WRITE) == 0
        }
        _ => false,
    }
}
fn validate_regions(note: &Note<'_>) -> Result<()> {
    let area = note.get(note::SHM_AREA).ok_or("missing SHM_AREA")?.value;
    let ddr = u64::from(word(area, 0)?);
    let sram = u64::from(short(area, 12)?);
    if ddr == 0 || ddr % 65536 != 0 || word(area, 8)? != rp1_proto::HANDOFF {
        return Err("invalid SHM_AREA".into());
    }
    let mut ddr_ranges = Vec::new();
    let mut sram_ranges = Vec::new();
    let mut names = std::collections::BTreeSet::new();
    for r in note
        .records()
        .filter_map(std::result::Result::ok)
        .filter(|r| r.kind == note::SHM_REGION)
    {
        let v = r.value;
        let doff = u64::from(word(v, 8)?);
        let dlen = u64::from(word(v, 12)?);
        let soff = u64::from(short(v, 16)?);
        let slen = u64::from(short(v, 18)?);
        let elem = u64::from(word(v, 20)?);
        let depth = u64::from(word(v, 24)?);
        let body = 128 + depth * (8 + ((elem + 7) & !7));
        if elem == 0
            || (v[0] != 3 && elem > 65535)
            || doff % 65536 != 0
            || !within(doff, dlen, 65536, ddr)
        {
            return Err("SHM DDR layout is invalid".into());
        }
        if v[0] == 1
            && ((v[1] == 1 && (body > dlen || slen < 128))
                || (v[1] == 2 && (body > slen || dlen < 128)))
        {
            return Err("ring storage is too short".into());
        }
        if v[0] == 3 && 8 + ((elem + 7) & !7) > dlen {
            return Err("block storage is too short".into());
        }
        if slen != 0
            && (!within(soff, slen, 64, sram)
                || soff % 64 != 0
                || (soff < 0x18c0 && soff + slen > 0x17c0))
        {
            return Err("SHM SRAM overlaps header, SCMI or reservation".into());
        }
        let name_len = v[44] as usize;
        let name = &v[45..45 + name_len];
        if !names.insert(name) {
            return Err("duplicate SHM name".into());
        }
        for (start, end) in &ddr_ranges {
            if doff < *end && doff + dlen > *start {
                return Err("overlapping DDR regions".into());
            }
        }
        ddr_ranges.push((doff, doff + dlen));
        if slen != 0 {
            for (start, end) in &sram_ranges {
                if soff < *end && soff + slen > *start {
                    return Err("overlapping SRAM regions".into());
                }
            }
            sram_ranges.push((soff, soff + slen));
        }
    }
    Ok(())
}

struct Instruction {
    address: u32,
    op: String,
    args: String,
}
type State = [Option<u32>; 16];
fn reg(s: &str) -> Option<usize> {
    match s.trim() {
        "ip" => Some(12),
        "sp" => Some(13),
        "lr" => Some(14),
        "pc" => Some(15),
        s => s
            .strip_prefix('r')?
            .parse::<usize>()
            .ok()
            .filter(|n| *n < 16),
    }
}
fn immediate(s: &str) -> Option<u32> {
    let s = s.trim().trim_start_matches('#');
    if let Some(n) = s.strip_prefix("0x") {
        u32::from_str_radix(n, 16).ok()
    } else {
        s.parse().ok()
    }
}
fn target(args: &str) -> Option<u32> {
    u32::from_str_radix(args.split_whitespace().next()?, 16).ok()
}
fn bracket(args: &str, state: &State, pc: u32) -> Option<u32> {
    let b = args.split('[').nth(1)?.split(']').next()?;
    let mut p = b.split(',');
    let r = reg(p.next()?)?;
    let value = if r == 15 { (pc + 4) & !3 } else { state[r]? };
    let offset = p
        .next()
        .map(|v| immediate(v).or_else(|| state[reg(v)?]))
        .unwrap_or(Some(0))?;
    if p.next().is_some() {
        return None;
    }
    value.checked_add(offset)
}
/// `[pc]` or `[pc, #imm]`: an immediate offset from the instruction's own address.
fn pc_literal(args: &str) -> bool {
    let Some(b) = args.split('[').nth(1).and_then(|b| b.split(']').next()) else {
        return false;
    };
    let mut p = b.split(',');
    reg(p.next().unwrap_or("")) == Some(15)
        && p.next().is_none_or(|v| v.trim().starts_with('#'))
        && p.next().is_none()
}
/// Conservative control-flow constant propagation. Unknown exclusive targets fail closed.
pub fn check_exclusives(text: &str, read_word: impl Fn(u32) -> Option<u32>) -> Result<usize> {
    let mut instructions = Vec::new();
    let mut roots = Vec::new();
    for line in text.lines() {
        if line.trim_end().ends_with(">:") {
            roots.push(instructions.len());
            continue;
        }
        let Some((a, rest)) = line.trim().split_once(':') else {
            continue;
        };
        let Ok(address) = u32::from_str_radix(a.trim(), 16) else {
            continue;
        };
        let mut p = rest.trim().splitn(2, char::is_whitespace);
        let op = p.next().unwrap_or("");
        if op.is_empty() || op.starts_with('.') {
            continue;
        }
        instructions.push(Instruction {
            address,
            op: op.split('.').next().unwrap().to_string(),
            args: p
                .next()
                .unwrap_or("")
                .split('@')
                .next()
                .unwrap()
                .trim()
                .to_string(),
        });
    }
    let by_address: BTreeMap<_, _> = instructions
        .iter()
        .enumerate()
        .map(|(i, x)| (x.address, i))
        .collect();
    let mut states: Vec<Option<State>> = vec![None; instructions.len()];
    let mut queue = VecDeque::new();
    for i in roots
        .into_iter()
        .chain(std::iter::once(0))
        .filter(|i| *i < instructions.len())
    {
        states[i] = Some([None; 16]);
        queue.push_back(i);
    }
    while let Some(i) = queue.pop_front() {
        let ins = &instructions[i];
        let mut state = states[i].unwrap();
        let parts: Vec<_> = ins.args.split(',').map(str::trim).collect();
        let dest = parts.first().and_then(|s| reg(s));
        let op = ins.op.as_str();
        let mut successors = Vec::new();
        match op {
            "mov" | "movs" | "movw" => {
                if let Some(r) = dest {
                    state[r] = parts
                        .get(1)
                        .and_then(|s| immediate(s).or_else(|| state[reg(s)?]));
                }
            }
            "movt" => {
                if let Some(r) = dest {
                    state[r] = state[r]
                        .and_then(|lo| Some((lo & 0xffff) | (immediate(parts.get(1)?)? << 16)));
                }
            }
            "add" | "adds" | "sub" | "subs" => {
                if let Some(r) = dest {
                    state[r] = (|| {
                        let (base, offset) = if parts.len() == 2 {
                            (state[r]?, immediate(parts[1])?)
                        } else {
                            (state[reg(parts[1])?]?, immediate(parts.get(2)?)?)
                        };
                        if op.starts_with("add") {
                            base.checked_add(offset)
                        } else {
                            base.checked_sub(offset)
                        }
                    })();
                }
            }
            "ldr" => {
                if let Some(r) = dest {
                    // Fold only PC-relative literal-pool loads. A load through any other register
                    // reads memory whose run-time value the checker cannot know (review #2).
                    state[r] = if pc_literal(&ins.args) {
                        bracket(&ins.args, &state, ins.address).and_then(&read_word)
                    } else {
                        None
                    };
                }
            }
            "bl" | "blx" => {
                for r in [0, 1, 2, 3, 12, 14] {
                    state[r] = None;
                }
            }
            "b" => {
                if let Some(j) = target(&ins.args).and_then(|a| by_address.get(&a)).copied() {
                    successors.push(j);
                }
            }
            "bx" | "tbb" | "tbh" => {}
            op if op.starts_with('b') && !matches!(op, "bic" | "bics" | "bfi" | "bfc" | "bkpt") => {
                if let Some(j) = target(&ins.args).and_then(|a| by_address.get(&a)).copied() {
                    successors.push(j);
                }
            }
            "cbz" | "cbnz" => {
                if let Some(j) = parts
                    .get(1)
                    .and_then(|a| target(a))
                    .and_then(|a| by_address.get(&a))
                    .copied()
                {
                    successors.push(j);
                }
            }
            "cmp" | "cmn" | "tst" | "teq" | "str" | "strb" | "strh" | "dmb" | "dsb" | "isb"
            | "nop" | "cpsid" | "cpsie" | "sev" | "wfe" | "wfi" | "clrex" => {}
            op if op.starts_with("ldrex") || op.starts_with("strex") => {
                if let Some(r) = dest {
                    state[r] = None;
                }
            }
            _ => state.fill(None),
        }
        if ins.args.contains('!') || ins.args.contains("],") {
            state.fill(None);
        }
        let is_return =
            matches!(op, "bx" | "tbb" | "tbh") || (op == "pop" && ins.args.contains("pc"));
        if op != "b" && !is_return && i + 1 < instructions.len() {
            successors.push(i + 1);
        }
        for next in successors {
            let merged = if let Some(old) = states[next] {
                core::array::from_fn(|r| if old[r] == state[r] { old[r] } else { None })
            } else {
                state
            };
            if states[next] != Some(merged) {
                states[next] = Some(merged);
                queue.push_back(next);
            }
        }
    }
    let mut count = 0;
    for (i, ins) in instructions.iter().enumerate() {
        if ins.op.starts_with("ldrex") || ins.op.starts_with("strex") {
            count += 1;
            let address = states[i].and_then(|s| bracket(&ins.args, &s, ins.address));
            if address.is_none_or(|a| {
                !within(
                    u64::from(a),
                    if ins.op.ends_with('d') { 8 } else { 4 },
                    0x1000_2000,
                    0x1000_4000,
                )
            }) {
                return Err(format!("exclusive at 0x{:08x} ({}) cannot be proven to target private DSRAM; target={address:?}",ins.address,ins.op).into());
            }
        }
    }
    Ok(count)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exclusive_targets_and_control_flow() {
        let text = "00001000 <f>:\n1000: ldr r1, [pc, #12]\n1002: ldrex r0, [r1]\n1006: strex r2, r0, [r1]\n100a: bx lr\n";
        assert_eq!(check_exclusives(text, |_| Some(0x1000_2000)).unwrap(), 2);
        assert!(check_exclusives(text, |_| Some(0x2000_0000)).is_err());
        assert!(check_exclusives(text, |_| None).is_err());
        let bypass =
            "1000: cbz r0, 1004\n1002: ldr r1, [pc, #12]\n1004: ldrex r0, [r1]\n1008: bx lr";
        assert!(check_exclusives(bypass, |_| Some(0x1000_2000)).is_err());
        let call = "1000: ldr r1, [pc, #12]\n1002: bl 2000\n1006: ldrex r0, [r1]\n100a: bx lr";
        assert!(check_exclusives(call, |_| Some(0x1000_2000)).is_err());
    }
    /// Review #2: a literal points at mutable shared SRAM whose *initial* value is a DSRAM
    /// address. The run-time value can differ, so the second load must be unknown.
    #[test]
    fn two_stage_load_through_mutable_memory_is_rejected() {
        let text = "00001000 <f>:\n1000: ldr r1, [pc, #12]\n1002: ldr r1, [r1]\n1004: ldrex r0, [r1]\n1008: strex r2, r0, [r1]\n100c: bx lr\n";
        let table = |a| match a {
            0x1010 => Some(0x2000_0000),
            0x2000_0000 => Some(0x1000_2000),
            _ => None,
        };
        assert!(check_exclusives(text, table).is_err());
        // Register-offset and register-based forms are not literals either.
        assert!(!pc_literal("r1, [pc, r2]"));
        assert!(!pc_literal("r1, [r1, #4]"));
        assert!(pc_literal("r1, [pc, #12]"));
        assert!(pc_literal("r1, [pc]"));
    }
}
