//! Contract §10.1 P2e/P5. Check the handwritten entry paths in the final ELF; the general
//! stack analysis tracks SP, but does not track the contents of LR or SCB register writes.

use super::{Fail, Insn, Program, immediate, target};

const FAULT: &str = "__rp1_fault";
const HARDFAULT: &str = "__rp1_hardfault";
const SETUP: &str = "__rp1_setup_exceptions";
const RESETS: [&str; 2] = ["__rp1_reset0", "__rp1_reset1"];

impl Program {
    /// Check the exception vectors and entry code for both cores. Each table starts at slot 0.
    pub fn check_exceptions(&self, vectors: [[u32; 7]; 2]) -> Result<(), Fail> {
        for (core, reset) in RESETS.into_iter().enumerate() {
            for (slot, name) in [
                (1, reset),
                (3, HARDFAULT),
                (4, FAULT),
                (5, FAULT),
                (6, FAULT),
            ] {
                if self.address_of(name).map(|a| a | 1) != Some(vectors[core][slot]) {
                    return Err(Fail {
                        reason: "exception-vector",
                        at: format!("proc{core}-slot{slot}"),
                    });
                }
            }
            self.check_reset(reset, core)?;
        }
        self.check_fault_entry()?;
        self.check_fault_setup()
    }

    fn check_fault_entry(&self) -> Result<(), Fail> {
        let fail = |name: &str| failure("fault-entry", name);
        let terminal = "__rp1_fault_terminal";
        let rust = "rp1_fault_rust";
        let [test, psp, frame, push, call, pop, ret] =
            self.prefix(FAULT).ok_or_else(|| fail(FAULT))?;
        // The exception LR is saved at entry SP-4, then restored from that exact word.
        // Only the ordinary Rust call lies between the matched stack transfers.
        if !(test.is("tst", "lr, #4")
            && self.to(psp, "bne", terminal)
            && frame.is("mov", "r0, sp")
            && (push.is("push", "{r0, lr}") || push.is("stmdb", "sp!, {r0, lr}"))
            && self.to(call, "bl", rust)
            && (pop.is("pop", "{r0, lr}")
                || pop.is("ldmia", "sp!, {r0, lr}")
                || pop.is("ldm", "sp!, {r0, lr}"))
            && ret.is("bx", "lr"))
        {
            return Err(fail(FAULT));
        }
        let [branch] = self.prefix(terminal).ok_or_else(|| fail(terminal))?;
        if !self.to(branch, "b", "rp1_fault_terminal") {
            return Err(fail(terminal));
        }
        if self.bound(branch.address)?.returns {
            return Err(failure("fault-terminal-returns", terminal));
        }
        if !self.bound(target(&call.args).unwrap())?.returns {
            return Err(failure("fault-rust-no-return", rust));
        }
        Ok(())
    }

    fn check_reset(&self, reset: &str, core: usize) -> Result<(), Fail> {
        let [mask, sp, msp, id, branch] = self
            .prefix(reset)
            .ok_or_else(|| failure("fault-reset-path", reset))?;
        if mask.is("cpsid", "i")
            && self.literal(sp, "r0") == Some(0x1000_4000)
            && msp.is("msr", "MSP, r0")
            && id.is("movs", if core == 0 { "r0, #0" } else { "r0, #1" })
            && self.to(branch, "b", SETUP)
        {
            Ok(())
        } else {
            Err(failure("fault-reset-path", reset))
        }
    }

    fn check_fault_setup(&self) -> Result<(), Fail> {
        // Fail closed on changes to this straight-line asm. Offsets are from AIRCR.
        let sequence = [
            ("str", "r2, [r1, #0]"),
            ("ldr", "r2, [r1, #8]"),
            ("orr", "r2, r2, #512"),
            ("str", "r2, [r1, #8]"), // CCR.STKALIGN.
            ("movs", "r2, #0"),
            ("msr", "BASEPRI, r2"),
            ("cpsie", "f"),
            ("dsb", "sy"),
            ("isb", "sy"),
            ("movs", "r2, #32"),
            ("strb", "r2, [r1, #12]"), // SHPR1, one byte per fault.
            ("strb", "r2, [r1, #13]"),
            ("strb", "r2, [r1, #14]"),
            ("ldr", "r2, [r1, #24]"),
            ("orr", "r2, r2, #458752"), // SHCSR.MEMFAULTENA | BUSFAULTENA | USGFAULTENA.
            ("str", "r2, [r1, #24]"),
            ("dsb", "sy"),
            ("isb", "sy"),
            ("cpsie", "i"),
            ("bx", "r0"),
        ];
        let fail = || failure("fault-startup-order", SETUP);
        let [init, aircr, value, body @ ..] = self.prefix::<23>(SETUP).ok_or_else(fail)?;
        if self.to(init, "bl", "__rp1_initialize")
            && self.literal(aircr, "r1") == Some(0xe000_ed0c)
            && self.literal(value, "r2") == Some(0x05fa_0000)
            && body
                .iter()
                .zip(sequence)
                .all(|(x, (op, args))| x.is(op, args))
        {
            Ok(())
        } else {
            Err(fail())
        }
    }

    /// The entry instructions up to the final transfer, excluding any unreachable padding.
    fn prefix<const N: usize>(&self, name: &str) -> Option<&[Insn; N]> {
        let f = self.functions.iter().find(|f| f.name == name)?;
        let prefix = self.insns[f.start..f.end].first_chunk::<N>()?;
        // Data at the end of the previous function is harmless.
        (!prefix[1..].iter().any(|x| x.after_data)).then_some(prefix)
    }

    fn to(&self, insn: &Insn, op: &str, name: &str) -> bool {
        insn.op == op
            && self
                .address_of(name)
                .is_some_and(|a| target(&insn.args) == Some(a))
    }

    fn literal(&self, insn: &Insn, reg: &str) -> Option<u32> {
        let (dest, operand) = insn.args.split_once(", ")?;
        if insn.op != "ldr" || dest != reg {
            return None;
        }
        let offset = operand.strip_prefix("[pc, ")?.strip_suffix(']')?;
        let address = ((insn.address + 4) & !3).wrapping_add(immediate(offset)?);
        let bytes = core::array::from_fn(|i| self.data.get(&(address + i as u32)).copied());
        let [Some(a), Some(b), Some(c), Some(d)] = bytes else {
            return None;
        };
        Some(u32::from_le_bytes([a, b, c, d]))
    }
}

impl Insn {
    fn is(&self, op: &str, args: &str) -> bool {
        self.op == op && self.args.eq_ignore_ascii_case(args)
    }
}

fn failure(reason: &'static str, symbol: &str) -> Fail {
    Fail {
        reason,
        at: symbol.into(),
    }
}
