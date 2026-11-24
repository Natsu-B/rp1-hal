//! K1 (contract §10.1 PR-D): synthetic `objdump -d --no-show-raw-insn` fragments.

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

#[test]
fn platform_share() {
    assert_eq!(s_plat(440, 0, 8, 8), 440 + 8 + 8 + 108);
    assert_eq!(s_plat(10, 20, 0, 0), 20 + 108);
}

#[test]
fn s_plat_must_fit_the_margin() {
    let line = |diagnostic, margin| verdict(440, 0, 8, 8, margin, diagnostic);
    assert_eq!(
        line(false, 768),
        (
            true,
            "stack_static=PASS s_plat=564 h57=440 h53=8 hhard=8 hfault=0 margin=768".into()
        )
    );
    assert!(line(false, 564).0);
    let (pass, text) = line(false, 512);
    assert!(!pass && text.starts_with("stack_static=FAIL reason=margin s_plat=564 "));
    let (pass, text) = line(true, 768);
    assert!(!pass && text.starts_with("stack_static=DIAG "));
}
