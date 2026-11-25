//! Contract §10.1 K1, P2e and P5: synthetic `objdump -d --no-show-raw-insn` fragments.

use super::*;

/// Disassembly text from `(symbol, address, instructions)`; each instruction takes 4 bytes.
fn dis(functions: &[(&str, u32, &[&str])]) -> String {
    let mut text = String::from("\nDisassembly of section .text:\n");
    for (name, address, body) in functions {
        text += &format!("\n{address:08x} <{name}>:\n");
        for (k, insn) in body.iter().enumerate() {
            let (op, args) = insn.split_once(' ').unwrap_or((insn, ""));
            text += &format!("{:08x}:\t{op}\t{args}\n", address + 4 * k as u32);
        }
    }
    text
}

fn bound(functions: &[(&str, u32, &[&str])]) -> Result<Bound, Fail> {
    let program = Program::parse(&dis(functions));
    program.bound(program.address_of(functions[0].0).unwrap())
}

fn bytes(functions: &[(&str, u32, &[&str])]) -> u32 {
    let b = bound(functions).unwrap();
    assert!(b.returns);
    b.bytes
}

fn fails(functions: &[(&str, u32, &[&str])], reason: &str, at: &str) {
    let f = bound(functions).unwrap_err();
    assert_eq!((f.reason, f.at.as_str()), (reason, at));
}

const LEAF: (&str, u32, &[&str]) = ("leaf", 0x3000, &["push {r4, lr}", "pop {r4, pc}"]);

const EXCEPTION_VECTORS: [[u32; 7]; 2] = [
    [0, 0x401, 0, 0x1001, 0x1101, 0x1101, 0x1101],
    [0, 0x40d, 0, 0x1001, 0x1101, 0x1101, 0x1101],
];

const EXCEPTIONS: &str = "+00000400 <__rp1_reset0>:
 400: cpsid i
 402: ldr r0, [pc, #84]  @ (458 <__rp1_setup_exceptions+0x40>)
 404: msr MSP, r0
 408: movs r0, #0
 40a: b.n 418 <__rp1_setup_exceptions>
0000040c <__rp1_reset1>:
 40c: cpsid i
 40e: ldr r0, [pc, #72]  @ (458 <__rp1_setup_exceptions+0x40>)
 410: msr MSP, r0
 414: movs r0, #1
 416: b.n 418 <__rp1_setup_exceptions>
00000418 <__rp1_setup_exceptions>:
 418: ldr r1, [pc, #64]
 41a: ldr r2, [pc, #68]
 420: str r2, [r1, #0]
 422: ldr r2, [r1, #8]
 424: orr.w r2, r2, #512  @ 0x200
 428: str r2, [r1, #8]
 42a: movs r2, #0
 42c: msr BASEPRI, r2
 430: cpsie f
 432: dsb sy
 436: isb sy
 43a: movs r2, #32
 43c: strb r2, [r1, #12]
 43e: strb r2, [r1, #13]
 440: strb r2, [r1, #14]
 442: ldr r2, [r1, #24]
 444: orr.w r2, r2, #458752  @ 0x70000
 448: str r2, [r1, #24]
 44a: dsb sy
 44e: isb sy
 450: bl 2000 <__rp1_initialize>
 452: cpsie i
 454: bx r0
 456: movs r0, r0
 458: .word 0x10004000
 45c: .word 0xe000ed0c
 460: .word 0x05fa0000
00001000 <__rp1_hardfault>:
 1000: mov r0, sp
 1002: mov r1, lr
 1004: b.w 1400 <rp1_hardfault>
00001100 <__rp1_fault>:
 1100: tst.w lr, #4
 1104: bne.n 1114 <__rp1_fault_terminal>
 1106: mov r0, sp
 1108: push {r0, lr}
 110a: bl 1200 <rp1_fault_rust>
 110e: ldmia.w sp!, {r0, lr}
 1112: bx lr
00001114 <__rp1_fault_terminal>:
 1114: b.w 1300 <rp1_fault_terminal>
00001200 <rp1_fault_rust>:
 1200: push {r4, lr}
 1202: pop {r4, pc}
00001300 <rp1_fault_terminal>:
 1300: wfe
 1302: b.n 1300 <rp1_fault_terminal>
00001400 <rp1_hardfault>:
 1400: wfe
 1402: b.n 1400 <rp1_hardfault>
00002000 <__rp1_initialize>:
 2000: bx lr
";

#[test]
fn p2e_fault_vectors_and_return() {
    let p = Program::parse(EXCEPTIONS);
    p.check_exceptions(EXCEPTION_VECTORS).unwrap();
    assert_eq!(
        p.bound(0x1101).unwrap(),
        Bound {
            bytes: 16,
            returns: true
        }
    );
    for core in 0..2 {
        for slot in 3..7 {
            let mut wrong = EXCEPTION_VECTORS;
            wrong[core][slot] &= !1;
            assert_eq!(
                p.check_exceptions(wrong).unwrap_err().reason,
                "exception-vector"
            );
        }
    }
}

#[test]
fn p2e_checks_exc_return_provenance_and_the_rust_path() {
    for (from, to, reason) in [
        ("tst.w lr, #4", "tst.w lr, #8", "fault-entry"),
        ("bne.n 1114", "beq.n 1114", "fault-entry"),
        ("bl 1200", "b.w 1200", "fault-entry"),
        ("push {r0, lr}", "push {r0, r1, lr}", "fault-entry"),
        (
            "ldmia.w sp!, {r0, lr}",
            "ldmia.w sp!, {r0, r1}",
            "fault-entry",
        ),
        (
            "1112: bx lr",
            "1110: mov lr, r0\n 1112: bx lr",
            "fault-entry",
        ),
        ("pop {r4, pc}", "bx lr", "unbalanced-return"),
        ("pop {r4, pc}", "blx r4", "unresolved-indirect-call"),
        ("pop {r4, pc}", "b.n 1202", "fault-rust-no-return"),
        ("wfe\n 1302:", "bx lr\n 1302:", "fault-terminal-returns"),
    ] {
        let p = Program::parse(&EXCEPTIONS.replacen(from, to, 1));
        assert_eq!(
            p.check_exceptions(EXCEPTION_VECTORS).unwrap_err().reason,
            reason,
            "{from} -> {to}"
        );
    }
}

#[test]
fn p2e_rejects_balanced_returns_with_wrong_pc_or_overwritten_lr() {
    for (body, expected) in [
        (
            "1200: push {r0, r1}\n 1202: pop {r0, pc}",
            "fault-return-address",
        ),
        ("1200: mov lr, r0\n 1202: bx lr", "fault-return-address"),
        ("1200: str r1, [sp, #4]\n 1202: bx lr", "fault-exception-lr"),
        (
            "1200: push {lr}\n 1202: strb r0, [sp, #1]\n 1204: pop {pc}",
            "fault-return-address",
        ),
        (
            "1200: push {lr}\n 1202: bl 3000 <leaf>\n 1206: pop {pc}",
            "fault-return-address",
        ),
        ("1200: bl 3002 <leaf+0x2>\n 1204: bx lr", "bad-call-target"),
        (
            "1200: bl 3100 <clean>\n 1204: bx lr",
            "fault-return-address",
        ),
    ] {
        let mut source = EXCEPTIONS.replace("1200: push {r4, lr}\n 1202: pop {r4, pc}", body);
        // The callee can clobber the caller's saved LR through a pointer alias.
        source.push_str("00003000 <leaf>:\n 3000: mov r2, sp\n 3002: str r0, [r2]\n 3004: bx lr\n00003100 <clean>:\n 3100: bx lr\n");
        let p = Program::parse(&source);
        assert_eq!(
            p.check_exceptions(EXCEPTION_VECTORS).unwrap_err().reason,
            expected,
            "{body}"
        );
    }
}

#[test]
fn p2e_checks_return_addresses_through_nested_and_tail_calls() {
    let source = EXCEPTIONS.replace(
        "1202: pop {r4, pc}",
        "1202: bl 3000 <callee>\n 1206: pop {r4, lr}\n 120a: b 3100 <tail>",
    ) + "00003000 <callee>:\n 3000: push {lr}\n 3002: ldr pc, [sp], #4\n"
        + "00003100 <tail>:\n 3100: bx lr\n";
    Program::parse(&source)
        .check_exceptions(EXCEPTION_VECTORS)
        .unwrap();
    let wrong = source.replace("3100: bx lr", "3100: mov lr, r0\n 3102: bx lr");
    assert_eq!(
        Program::parse(&wrong)
            .check_exceptions(EXCEPTION_VECTORS)
            .unwrap_err()
            .reason,
        "fault-return-address"
    );
}

#[test]
fn hardfault_stub_passes_msp_and_lr_and_cannot_return() {
    for (from, to, reason) in [
        ("1000: mov r0, sp", "1000: bx lr", "hardfault-entry"),
        ("1002: mov r1, lr", "1002: mov r1, r2", "hardfault-entry"),
        ("1400: wfe", "1400: bx lr", "hardfault-returns"),
    ] {
        let p = Program::parse(&EXCEPTIONS.replace(from, to));
        assert_eq!(
            p.check_exceptions(EXCEPTION_VECTORS).unwrap_err().reason,
            reason
        );
    }
}

#[test]
fn startup_rejects_reachable_early_unmask_through_initialization_calls() {
    for early in ["cpsie i", "cpsie if", "msr PRIMASK, r0"] {
        let direct = EXCEPTIONS.replace("2000: bx lr", &format!("2000: {early}\n 2002: bx lr"));
        assert_eq!(
            Program::parse(&direct)
                .check_exceptions(EXCEPTION_VECTORS)
                .unwrap_err()
                .reason,
            "fault-startup-mask"
        );
        let called = EXCEPTIONS.replace(
            "2000: bx lr",
            &format!("2000: push {{lr}}\n 2002: bl 2100 <init_child>\n 2006: pop {{pc}}\n00002100 <init_child>:\n 2100: {early}\n 2102: bx lr"),
        );
        assert_eq!(
            Program::parse(&called)
                .check_exceptions(EXCEPTION_VECTORS)
                .unwrap_err()
                .reason,
            "fault-startup-mask"
        );
    }
    let unreachable = EXCEPTIONS.to_owned() + " 2002: cpsie i\n";
    Program::parse(&unreachable)
        .check_exceptions(EXCEPTION_VECTORS)
        .unwrap();
}

#[test]
fn p5_checks_fault_priority_enables_and_write_order_on_both_cores() {
    for (from, to) in [
        ("#32", "#64"), // SHPR1 priority.
        ("strb r2, [r1, #12]", "strb r2, [r1, #15]"),
        ("strb r2, [r1, #13]", "strb r2, [r1, #15]"),
        ("strb r2, [r1, #14]", "strb r2, [r1, #15]"),
        ("#458752", "#196608"), // Missing a SHCSR enable bit.
        ("0xe000ed0c", "0xe000ed08"),
        ("orr.w r2, r2, #512", "orr.w r2, r2, #256"),
        ("428: str r2, [r1, #8]", "428: cpsie i"), // PRIMASK cleared early.
        ("42a: movs r2, #0", "42a: str r2, [r1, #24]"),
        ("428: str r2, [r1, #8]", "428: str r2, [r1, #24]"), // Faults before STKALIGN.
    ] {
        let p = Program::parse(&EXCEPTIONS.replacen(from, to, 1));
        assert_eq!(
            p.check_exceptions(EXCEPTION_VECTORS).unwrap_err().reason,
            "fault-startup-order"
        );
    }
    let p = Program::parse(&EXCEPTIONS.replace("416: b.n 418", "416: b.n 2000"));
    assert_eq!(
        p.check_exceptions(EXCEPTION_VECTORS).unwrap_err(),
        Fail {
            reason: "fault-reset-path",
            at: "__rp1_reset1".into()
        }
    );
}

#[test]
fn nested_calls_add_frames() {
    let mid: &[&str] = &[
        "push {r7, lr}",
        "sub sp, #16",
        "bl 3000 <leaf>",
        "add sp, #16",
        "pop {r7, pc}",
    ];
    let root: &[&str] = &[
        "push {r4, r5, r6, r7, lr}",
        "bl 2000 <mid>",
        "bl 3000 <leaf>",
        "pop {r4, r5, r6, r7, pc}",
    ];
    // root 20 + mid (8 + 16) + leaf 8.
    assert_eq!(
        bytes(&[("root", 0x1000, root), ("mid", 0x2000, mid), LEAF]),
        52
    );
}

#[test]
fn push_sub_and_writeback_forms() {
    let f: &[&str] = &[
        "push {r4, r5, r6, r7, lr}",
        "add r7, sp, #12",
        "stmdb sp!, {r8, r9, sl, fp}",
        "sub.w sp, sp, #520",
        "str r0, [sp, #-8]!",
        "ldr r0, [sp], #8",
        "addw sp, sp, #520",
        "ldmia.w sp!, {r8, r9, sl, fp}",
        "pop {r4, r5, r6, r7, pc}",
    ];
    assert_eq!(bytes(&[("f", 0x1000, f)]), 20 + 16 + 520 + 8);
    let single: &[&str] = &["str lr, [sp, #-4]!", "ldr pc, [sp], #4"];
    assert_eq!(bytes(&[("single", 0x1000, single)]), 4);
}

#[test]
fn tail_call_counts_callee_on_current_depth() {
    let f: &[&str] = &["push {r7, lr}", "pop {r7, lr}", "b.w 3000 <leaf>"];
    assert_eq!(bytes(&[("f", 0x1000, f), LEAF]), 8);
    // A conditional tail call before the epilogue stacks the callee on the live frame.
    let g: &[&str] = &[
        "push {r4, r5, r6, r7, lr}",
        "cbz r0, 1010 <g+0x10>",
        "pop {r4, r5, r6, r7, pc}",
        "nop",
        "bl 3000 <leaf>",
        "udf #254",
    ];
    assert_eq!(bytes(&[("g", 0x1000, g), LEAF]), 28);
}

#[test]
fn conditional_return_and_branches() {
    let f: &[&str] = &[
        "push {r7, lr}",
        "cmp r0, #15",
        "it ls",
        "popls {r7, pc}",
        "sub sp, #8",
        "bne.n 1018 <f+0x18>",
        "add sp, #8",
        "pop {r7, pc}",
    ];
    assert_eq!(bytes(&[("f", 0x1000, f)]), 16);
}

#[test]
fn noreturn_call_ends_the_path() {
    let panic: &[&str] = &["push {r7, lr}", "wfe", "b.n 3004 <panic+0x4>"];
    let f: &[&str] = &[
        "push {r7, lr}",
        "cmp r0, #2",
        "bcs.n 1010 <f+0x10>",
        "pop {r7, pc}",
        "bl 3000 <panic>",
    ];
    let b = bound(&[("f", 0x1000, f), ("panic", 0x3000, panic)]).unwrap();
    assert_eq!(
        b,
        Bound {
            bytes: 16,
            returns: true
        }
    );
    let p = Program::parse(&dis(&[("panic", 0x3000, panic)]));
    assert!(!p.bound(0x3000).unwrap().returns);
}

#[test]
fn linker_thunk_resolves() {
    let thunk: &[&str] = &["movw ip, #12289", "movt ip, #0", "bx ip"];
    let f: &[&str] = &["push {r7, lr}", "bl 2000 <thunk>", "pop {r7, pc}"];
    assert_eq!(
        bytes(&[("f", 0x1000, f), ("thunk", 0x2000, thunk), LEAF]),
        16
    );
    let call: &[&str] = &[
        "push {r7, lr}",
        "movw r3, #12289",
        "movt r3, #0",
        "blx r3",
        "pop {r7, pc}",
    ];
    assert_eq!(bytes(&[("call", 0x1000, call), LEAF]), 16);
}

#[test]
fn jump_table_follows_every_entry() {
    // tbb at 0x1004; table at 0x1008 = {0x02, 0x03, 0x04, pad 0x00}: 0x100c, 0x100e, 0x1010.
    let text = "
00001000 <f>:
00001000:\tpush\t{r7, lr}
00001002:\tsub\tsp, #8
00001004:\ttbb\t[pc, r0]
00001008:\t.word\t0x00040302
0000100c:\tb.n\t1012 <f+0x12>
0000100e:\tb.n\t1012 <f+0x12>
00001010:\tsub\tsp, #16
00001012:\tbl\t3000 <leaf>
00001016:\tadd\tsp, #8
00001018:\tpop\t{r7, pc}

00003000 <leaf>:
00003000:\tpush\t{r4, lr}
00003002:\tpop\t{r4, pc}
";
    let p = Program::parse(text);
    // The 0x1010 path reaches the join 16 B deeper: whichever path is walked first, the
    // other fails (at the join, or at the return of the deeper one).
    let f = p.bound(0x1000).unwrap_err();
    assert!(
        matches!(f.reason, "sp-mismatch" | "unbalanced-return"),
        "{f:?}"
    );
    let p = Program::parse(&text.replace("sub\tsp, #16", "nop"));
    assert_eq!(p.bound(0x1000).unwrap().bytes, 8 + 8 + 8);
    // A jump table entry into a movw/movt/bx sequence breaks the thunk pattern too.
    let thunk = "
00001000 <g>:
00001000:\ttbb\t[pc, r0]
00001004:\t.short\t0x0301
00001006:\tmovw\tip, #12289
0000100a:\tmovt\tip, #0
0000100e:\tbx\tip

00003000 <leaf>:
00003000:\tpush\t{r4, lr}
00003002:\tpop\t{r4, pc}
";
    let f = Program::parse(thunk).bound(0x1000).unwrap_err();
    assert_eq!(
        (f.reason, f.at.as_str()),
        ("unresolved-indirect-jump", "g+0xe")
    );
    let p = Program::parse(&thunk.replace("0x0301", "0x0101"));
    assert_eq!(p.bound(0x1000).unwrap().bytes, 8);
    // An entry outside the function fails.
    let p = Program::parse(
        &text
            .replace("0x00040302", "0x00f00302")
            .replace("sub\tsp, #16", "nop"),
    );
    assert_eq!(p.bound(0x1000).unwrap_err().reason, "jump-table");
}

#[test]
fn recursion_fails() {
    let a: &[&str] = &["push {r7, lr}", "bl 2000 <b>", "pop {r7, pc}"];
    let b: &[&str] = &["push {r7, lr}", "bl 1000 <a>", "pop {r7, pc}"];
    fails(&[("a", 0x1000, a), ("b", 0x2000, b)], "recursion", "b+0x4");
    let own: &[&str] = &[
        "push {r7, lr}",
        "cbz r0, 100c <own+0xc>",
        "bl 1000 <own>",
        "pop {r7, pc}",
    ];
    fails(&[("own", 0x1000, own)], "recursion", "own+0x8");
    // Through tail calls too. (A branch to the function's own start is a loop, not a call.)
    let ta: &[&str] = &["b.w 2000 <tb>"];
    let tb: &[&str] = &["push {r7, lr}", "pop {r7, lr}", "b.w 1000 <ta>"];
    fails(
        &[("ta", 0x1000, ta), ("tb", 0x2000, tb)],
        "recursion",
        "tb+0x8",
    );
}

#[test]
fn variable_sp_fails() {
    let mov: &[&str] = &["push {r7, lr}", "mov sp, r0", "pop {r7, pc}"];
    fails(&[("f", 0x1000, mov)], "sp-write", "f+0x4");
    let sub: &[&str] = &["push {r7, lr}", "sub sp, r3", "pop {r7, pc}"];
    fails(&[("f", 0x1000, sub)], "variable-sp", "f+0x4");
    let sub3: &[&str] = &["sub.w sp, r7, #8", "bx lr"];
    fails(&[("f", 0x1000, sub3)], "variable-sp", "f+0x0");
    let post: &[&str] = &["ldr r0, [sp], r1", "bx lr"];
    fails(&[("f", 0x1000, post)], "variable-sp", "f+0x0");
}

#[test]
fn unknown_sp_instructions_fail() {
    for (insn, reason) in [
        ("msr MSP, r0", "sp-write"),
        ("ldr sp, [r0, #4]", "sp-write"),
        ("lsl sp, r0, #2", "sp-write"),
        ("adds sp, #8", "sp-write"),
        ("stmia sp!, {r0, r1}", "variable-sp"),
        ("ldm r0, {r1, sp}", "sp-write"),
        ("addeq sp, #8", "conditional-sp"),
        ("ldreq r4, [sp], #4", "conditional-sp"),
        ("strbeq r0, [sp, #-8]!", "conditional-sp"),
        ("ldrfoo r0, [sp], #4", "unknown-instruction"),
        ("svc 0", "svc"),
    ] {
        let f: &[&str] = &["it eq", insn, "bx lr"];
        fails(&[("f", 0x1000, f)], reason, "f+0x4");
    }
}

#[test]
fn unresolved_indirect_control_fails() {
    let blx: &[&str] = &["push {r7, lr}", "blx r3", "pop {r7, pc}"];
    fails(&[("f", 0x1000, blx)], "unresolved-indirect-call", "f+0x4");
    let bx: &[&str] = &["bx r2"];
    fails(&[("f", 0x1000, bx)], "unresolved-indirect-jump", "f+0x0");
    let ldr: &[&str] = &["ldr pc, [r0, #4]"];
    fails(&[("f", 0x1000, ldr)], "unresolved-indirect-jump", "f+0x0");
    let mov: &[&str] = &["mov pc, r1"];
    fails(&[("f", 0x1000, mov)], "unresolved-indirect-jump", "f+0x0");
    // Only the post-indexed form pops the return address.
    let pre: &[&str] = &["str lr, [sp, #-4]!", "ldr pc, [sp, #4]!"];
    fails(&[("f", 0x1000, pre)], "unresolved-indirect-jump", "f+0x4");
    // A branch into the movw/movt pair breaks the thunk pattern.
    let joined: &[&str] = &[
        "cbz r0, 1008 <f+0x8>",
        "movw ip, #12289",
        "movt ip, #0",
        "bx ip",
    ];
    fails(
        &[("f", 0x1000, joined), LEAF],
        "unresolved-indirect-jump",
        "f+0xc",
    );
}

#[test]
fn unbalanced_paths_fail() {
    let ret: &[&str] = &["push {r4, r7, lr}", "pop {r7, pc}"];
    fails(&[("f", 0x1000, ret)], "unbalanced-return", "f+0x4");
    let above: &[&str] = &["pop {r7, pc}"];
    fails(&[("f", 0x1000, above)], "sp-above-entry", "f+0x0");
    // Two depths meet at the `wfe` loop.
    let join: &[&str] = &[
        "push {r7, lr}",
        "cbz r0, 100c <f+0xc>",
        "sub sp, #8",
        "wfe",
        "b.n 100c <f+0xc>",
    ];
    let f = bound(&[("f", 0x1000, join)]).unwrap_err();
    assert_eq!((f.reason, f.at.as_str()), ("sp-mismatch", "f+0xc"));
    let off: &[&str] = &["push {r7, lr}", "bl 3000 <leaf>"];
    fails(&[("f", 0x1000, off), LEAF], "falls-off-end", "f+0x4");
    let tail: &[&str] = &["push {r7, lr}", "b.w 3000 <leaf>"];
    fails(
        &[("f", 0x1000, tail), LEAF],
        "unbalanced-tail-call",
        "f+0x4",
    );
    let data = "\n00001000 <f>:\n00001000:\tpush\t{r7, lr}\n00001002:\t.word\t0x0\n00001006:\tpop\t{r7, pc}\n";
    let f = Program::parse(data).bound(0x1000).unwrap_err();
    assert_eq!((f.reason, f.at.as_str()), ("falls-into-data", "f+0x0"));
    let middle: &[&str] = &["push {r7, lr}", "bl 3002 <leaf+0x2>", "pop {r7, pc}"];
    fails(&[("f", 0x1000, middle), LEAF], "bad-call-target", "f+0x4");
}

#[test]
fn cut_is_diagnostic_only() {
    let a: &[&str] = &[
        "push {r7, lr}",
        "cbz r0, 100c <a+0xc>",
        "pop {r7, pc}",
        "bl 2000 <b>",
    ];
    let b: &[&str] = &[
        "push {r4, r5, r6, r7, lr}",
        "bl 1000 <a>",
        "pop {r4, r5, r6, r7, pc}",
    ];
    let p = Program::parse(&dis(&[("a", 0x1000, a), ("b", 0x2000, b)]));
    assert_eq!(p.bound(0x1000).unwrap_err().reason, "recursion");
    let cut = HashSet::from([0x2000]);
    assert_eq!(p.bound_cut(0x1000, &cut).unwrap().bytes, 8);
}

#[test]
fn objdump_text_forms() {
    // Raw objdump lines: comments, `.w` suffixes, aliases, data words.
    let text = "
20001370 <__rp1_p0_PCIE_OUT>:
20001370:\tpush\t{r7, lr}
20001372:\tmov\tr7, sp
20001374:\tmovw\tr0, #8432\t@ 0x20f0
20001378:\tcbz\tr2, 2000137e <__rp1_p0_PCIE_OUT+0xe>
2000137a:\tstr.w\tr1, [ip, #44]\t@ 0x2c
2000137c:\tpop\t{r7, pc}
2000137e:\tsub.w\tsp, sp, #0x100
20001382:\tadd.w\tsp, sp, #256\t@ 0x100
20001386:\tpop\t{r7, pc}
20001388:\t.word\t0x20001234
";
    let p = Program::parse(text);
    assert_eq!(p.name_at(0x2000_1371), Some("__rp1_p0_PCIE_OUT"));
    assert_eq!(p.bound(0x2000_1371).unwrap().bytes, 8 + 256);
}

const HANDLERS: Handlers = Handlers {
    h57: 440,
    h53: 8,
    hhard: 8,
    hfault: 0,
    hnmi: None,
};

#[test]
fn platform_share() {
    assert_eq!(HANDLERS.s_plat(), 440 + 8 + 8 + 108);
    let faults = Handlers {
        h57: 10,
        hfault: 20,
        h53: 0,
        hhard: 0,
        hnmi: None,
    };
    assert_eq!(faults.s_plat(), 20 + 108);
    // An NMI handler stacks on top of everything, with its own frame.
    let nmi = Handlers {
        hnmi: Some(8),
        ..HANDLERS
    };
    assert_eq!(nmi.s_plat(), HANDLERS.s_plat() + 8 + 36);
}

#[test]
fn s_plat_must_fit_the_margin() {
    let line =
        |diagnostic, margin, stack0| verdict(&HANDLERS, Proc0Stack { stack0, margin }, diagnostic);
    assert_eq!(
        line(false, 768, 2816),
        (
            true,
            "stack_static=PASS s_plat=564 h57=440 h53=8 hhard=8 hfault=0 hnmi=none margin=768 \
             stack0=2816"
                .into()
        )
    );
    assert!(line(false, 564, 2816).0);
    let (pass, text) = line(false, 512, 2816);
    assert!(!pass && text.starts_with("stack_static=FAIL reason=margin s_plat=564 "));
    let (pass, text) = line(true, 768, 2816);
    assert!(!pass && text.starts_with("stack_static=DIAG "));
    let nmi = Handlers {
        hnmi: Some(200),
        ..HANDLERS
    };
    let (pass, text) = verdict(
        &nmi,
        Proc0Stack {
            stack0: 2816,
            margin: 768,
        },
        false,
    );
    assert!(
        !pass && text.contains("s_plat=800 ") && text.contains("hnmi=200 "),
        "{text}"
    );
}

#[test]
fn margin_must_leave_a_user_stack() {
    // An image without a user part above the margin (or a margin of the whole stack) fails,
    // whatever S_plat is.
    let line = |margin, stack0| verdict(&HANDLERS, Proc0Stack { stack0, margin }, false);
    assert!(line(768, 776).0);
    for (margin, stack0) in [(768, 768), (2048, 2048), (4096, 2048)] {
        let (pass, text) = line(margin, stack0);
        assert!(
            !pass && text.starts_with("stack_static=FAIL reason=proc0-stack "),
            "{text}"
        );
    }
}
