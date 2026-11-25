//! P2e checks the return values in the fault helper's reachable epilogues.
//! Track the original LR, each BL's link, and writes through stack aliases.

use super::*;

const LR: usize = 14;
const HELPER_LINK: usize = usize::MAX;
const EXCEPTION_LR: Value = Value::Link(usize::MAX - 1);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
enum Value {
    #[default]
    Unknown,
    Link(usize),
    Stack(i32), // Offset from the fault helper's entry SP.
    Constant(u32),
}

impl Value {
    fn add(self, other: Self) -> Self {
        match (self, other) {
            (Self::Constant(a), Self::Constant(b)) => Self::Constant(a.wrapping_add(b)),
            (Self::Stack(a), Self::Constant(b)) | (Self::Constant(b), Self::Stack(a)) => {
                a.checked_add(b as i32).map_or(Self::Unknown, Self::Stack)
            }
            _ => Self::Unknown,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
struct State {
    regs: [Value; 16],
    stack: BTreeMap<i32, Value>,
    // Caller instruction after BL and its SP. A tail call leaves this frame intact.
    calls: Vec<(usize, i32)>,
}

impl State {
    fn sp(&self) -> i32 {
        let Value::Stack(sp) = self.regs[SP] else {
            unreachable!("SP always has a checked offset");
        };
        sp
    }

    fn operand(&self, operand: &str) -> Value {
        if let Some(r) = parse_reg(operand) {
            self.regs[r]
        } else {
            immediate(operand).map_or(Value::Unknown, Value::Constant)
        }
    }

    fn load(&self, address: Value, width: i32) -> Value {
        match address {
            Value::Stack(offset) if width == 4 => {
                self.stack.get(&offset).copied().unwrap_or_default()
            }
            _ => Value::Unknown,
        }
    }

    fn store(&mut self, address: Value, width: i32, value: Value) -> Result<(), &'static str> {
        match address {
            Value::Stack(offset) => {
                let end = offset.checked_add(width).ok_or("fault-return-store")?;
                self.stack
                    .retain(|&slot, _| slot.saturating_add(4) <= offset || slot >= end);
                if width == 4 && value != Value::Unknown {
                    self.stack.insert(offset, value);
                }
                Ok(())
            }
            // Only DSRAM can hold the checked MSP frames. Unknown stores could alias them.
            Value::Constant(address)
                if address
                    .checked_add(width as u32)
                    .is_some_and(|end| address >= 0x1000_4000 || end <= 0x1000_0000) =>
            {
                Ok(())
            }
            _ => Err("fault-return-store"),
        }
    }

    fn transfer(
        &mut self,
        address: Value,
        reg: usize,
        width: i32,
        load: bool,
    ) -> Result<(), &'static str> {
        if load {
            self.regs[reg] = self.load(address, width);
            Ok(())
        } else {
            self.store(address, width, self.regs[reg])
        }
    }
}

impl Program {
    pub(super) fn check_return_address(&self, address: u32) -> Result<(), Fail> {
        let start = self.index[&address];
        let mut initial = State::default();
        initial.regs[0] = Value::Stack(8); // The entry stub passes the frame above its two words.
        initial.regs[SP] = Value::Stack(0);
        initial.regs[LR] = Value::Link(HELPER_LINK);
        initial.stack.insert(4, EXCEPTION_LR);
        initial.calls.push((HELPER_LINK, 0));
        let mut pending = vec![(start, initial)];
        let mut seen = HashSet::new();
        let functions: Vec<_> = self
            .functions
            .iter()
            .map(|f| local_targets(self, f))
            .collect();
        let mut returning = HashMap::new();

        while let Some((i, mut state)) = pending.pop() {
            if !seen.insert((i, state.clone())) {
                continue;
            }
            let fail = |reason| Fail {
                reason,
                at: self.location(i),
            };
            if seen.len() > 16_384 {
                return Err(fail("fault-return-state-limit"));
            }
            let f = self.functions.partition_point(|f| f.end <= i);
            let func = &self.functions[f];
            let e = resolved_effect(self, func, &functions[f], i).map_err(fail)?;
            if e.conditional {
                pending.push((i + 1, state.clone()));
            }
            let old_sp = state.sp();
            let sp = i32::try_from(i64::from(old_sp) - e.push)
                .map_err(|_| fail("fault-return-stack"))?;
            match e.flow {
                Flow::Next | Flow::Return => {
                    let link = self.apply_return_effect(i, sp, &mut state).map_err(fail)?;
                    state.regs[SP] = Value::Stack(sp);
                    if sp > old_sp {
                        state.stack.retain(|&offset, _| offset >= sp);
                    }
                    if matches!(e.flow, Flow::Return) {
                        let (resume, caller_sp) = state.calls.pop().unwrap();
                        if link != Value::Link(resume) || sp != caller_sp {
                            return Err(fail("fault-return-address"));
                        }
                        if resume == HELPER_LINK {
                            if state.stack.get(&4) != Some(&EXCEPTION_LR) {
                                return Err(fail("fault-exception-lr"));
                            }
                        } else {
                            pending.push((resume, state));
                        }
                    } else {
                        pending.push((i + 1, state));
                    }
                }
                Flow::Call(t) | Flow::Branch(t) => {
                    let j = self.index[&t];
                    let call = matches!(e.flow, Flow::Call(_));
                    if call || !(func.start..func.end).contains(&j) {
                        let returns = match returning.get(&t) {
                            Some(&returns) => returns,
                            None => {
                                let returns = self.bound(t)?.returns;
                                returning.insert(t, returns);
                                returns
                            }
                        };
                        if !returns {
                            continue;
                        }
                        if call {
                            state.calls.push((i + 1, sp));
                            state.regs[LR] = Value::Link(i + 1);
                        }
                    }
                    pending.push((j, state));
                }
                Flow::Table { half } => {
                    for j in table_targets(self, func, i, half).map_err(fail)? {
                        pending.push((j, state.clone()));
                    }
                }
                Flow::Trap => {}
                Flow::Indirect { .. } => unreachable!("already resolved"),
            }
        }
        Ok(())
    }

    fn apply_return_effect(
        &self,
        i: usize,
        sp: i32,
        state: &mut State,
    ) -> Result<Value, &'static str> {
        let insn = &self.insns[i];
        let (op, args) = (insn.op.as_str(), insn.args.as_str());
        if condition(op, "bx").is_some() {
            return Ok(state.regs[LR]);
        }
        if args.contains('{') {
            return multiple_transfer(op, args, sp, state);
        }
        if let Some(base) = LOAD_STORE.iter().find(|base| condition(op, base).is_some()) {
            return self.transfer(insn, base, state);
        }
        let operands: Vec<_> = args.split(',').map(str::trim).collect();
        let Some(dest) = parse_reg(operands[0]) else {
            let safe = ["dsb", "isb", "dmb", "cpsid", "nop", "wfe", "wfi", "sev"];
            if safe.contains(&op)
                || op.starts_with("it")
                || (op.starts_with("msr") && !operands[0].eq_ignore_ascii_case("control"))
            {
                return Ok(Value::Unknown);
            }
            return Err("fault-return-instruction");
        };
        let is = |base| condition(op, base).is_some();
        if dest == SP || ["cmp", "cmn", "tst", "teq"].into_iter().any(is) {
            return Ok(Value::Unknown);
        }
        let right = operands.get(1).map_or(Value::Unknown, |x| state.operand(x));
        state.regs[dest] = if ["mov", "movs", "movw"].into_iter().any(is) {
            right
        } else if is("movt") {
            match (state.regs[dest], right) {
                (Value::Constant(low), Value::Constant(high)) => {
                    Value::Constant((low & 0xffff) | (high << 16))
                }
                _ => Value::Unknown,
            }
        } else if [
            "add", "adds", "addw", "sub", "subs", "subw", "orr", "ands", "and", "bic", "bics",
        ]
        .into_iter()
        .any(is)
        {
            let (left, right) = match operands.as_slice() {
                [_, _, right_operand] => (right, state.operand(right_operand)),
                [_, _] => (state.regs[dest], right),
                _ => (Value::Unknown, Value::Unknown),
            };
            arithmetic(op, left, right)
        } else if ["mrs", "bfi", "ubfx", "uxtb", "uxth"].into_iter().any(is) {
            Value::Unknown
        } else {
            return Err("fault-return-instruction");
        };
        Ok(Value::Unknown)
    }

    fn transfer(&self, insn: &Insn, op: &str, state: &mut State) -> Result<Value, &'static str> {
        let (regs, operand) = insn
            .args
            .split_once('[')
            .ok_or("fault-return-instruction")?;
        let (address, update) = operand.split_once(']').ok_or("fault-return-instruction")?;
        let parts: Vec<_> = address.split(',').map(str::trim).collect();
        let base = parse_reg(parts[0]).ok_or("fault-return-instruction")?;
        let offset = match parts.as_slice() {
            [_] => Value::Constant(0),
            [_, off] => state.operand(off),
            _ => Value::Unknown,
        };
        let update = update.trim();
        let mut address = state.regs[base];
        if !update.starts_with(',') {
            address = address.add(offset);
        }
        let load = op.starts_with("ldr");
        let regs: Vec<_> = regs.split(',').filter_map(parse_reg).collect();
        let width = if op.ends_with('b') || op == "ldrsb" {
            1
        } else if op.ends_with('h') || op == "ldrsh" {
            2
        } else {
            4
        };
        if !matches!(
            op,
            "ldr" | "str" | "ldrb" | "strb" | "ldrh" | "strh" | "ldrsb" | "ldrsh" | "ldrd" | "strd"
        ) || regs.len() != if op.ends_with('d') { 2 } else { 1 }
        {
            return Err("fault-return-instruction");
        }
        for (n, reg) in regs.into_iter().enumerate() {
            if base == PC && load && op == "ldr" {
                let dest = insn.args.split(',').next().unwrap_or("");
                state.regs[reg] = self
                    .literal(insn, dest)
                    .map_or(Value::Unknown, Value::Constant);
            } else {
                state.transfer(address.add(Value::Constant(4 * n as u32)), reg, width, load)?;
            }
        }
        if base != SP {
            if let Some(post) = update.strip_prefix(',') {
                state.regs[base] = state.regs[base].add(state.operand(post));
            } else if update == "!" {
                state.regs[base] = address;
            }
        }
        Ok(state.regs[PC])
    }
}

fn arithmetic(op: &str, left: Value, right: Value) -> Value {
    if op.starts_with("add") {
        return left.add(right);
    }
    let Value::Constant(b) = right else {
        return Value::Unknown;
    };
    if op.starts_with("sub") {
        return left.add(Value::Constant(b.wrapping_neg()));
    }
    let Value::Constant(a) = left else {
        return Value::Unknown;
    };
    Value::Constant(if op.starts_with("orr") {
        a | b
    } else if op.starts_with("bic") {
        a & !b
    } else {
        a & b
    })
}

fn multiple_transfer(
    op: &str,
    args: &str,
    sp: i32,
    state: &mut State,
) -> Result<Value, &'static str> {
    let mut regs = reg_list(args)?;
    regs.sort_unstable();
    let load = op.starts_with("pop") || op.starts_with("ldm");
    let (mut address, writeback) = if op.starts_with("push") {
        (Value::Stack(sp), None)
    } else if op.starts_with("pop") {
        (state.regs[SP], None)
    } else {
        let base = args.split(',').next().unwrap_or("");
        let reg = parse_reg(base).ok_or("fault-return-instruction")?;
        let mut address = state.regs[reg];
        if op.starts_with("stmdb") || op.starts_with("stmfd") {
            address = address.add(Value::Constant((4 * regs.len() as u32).wrapping_neg()));
        } else if !(op.starts_with("ldm") || op.starts_with("stm"))
            || ["ldmdb", "ldmda", "ldmib", "stmda", "stmib"]
                .iter()
                .any(|m| op.starts_with(m))
        {
            return Err("fault-return-instruction");
        }
        (address, (reg != SP && base.ends_with('!')).then_some(reg))
    };
    let final_address = if op.starts_with("stmdb") || op.starts_with("stmfd") {
        address
    } else {
        address.add(Value::Constant(4 * regs.len() as u32))
    };
    for reg in regs {
        state.transfer(address, reg, 4, load)?;
        address = address.add(Value::Constant(4));
    }
    if let Some(reg) = writeback {
        state.regs[reg] = final_address;
    }
    Ok(state.regs[PC])
}
