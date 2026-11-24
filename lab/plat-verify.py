#!/usr/bin/env python3
"""Platform stack and size verdict for one firmware build and one boot (contract PR-D, V1).

Usage: plat-verify.py --elf FILE [--uart0 FILE] [--uart10 FILE] [--stack-paths EXE]

It runs `stack-paths <elf> --size` (the host tool in lab/stack-paths; by default through
`cargo run --release -p stack-paths` in this repository, or the executable given with
--stack-paths or $STACK_PATHS) and reads the UART captures of one boot (raw, or with a prefix
on each line). It prints one line per row, each PASS, FAIL or INCOMPLETE:

- `stack_static`: stack-paths' static bound of the proc0 platform handlers. PASS only with
  `stack_static=PASS` and exit status 0 (S_plat within proc0's platform stack margin).
- `stack_watermark_free`: the smallest untouched stack space the firmware reported after its
  load, observed, not a bound. proc0: `IRQ_STACK p0_free= p0_size=`; proc1: `p1_free=`,
  `stack_free=`, `RINGDMA_STACK free_bytes=`. PASS if every value is at least 512 B. Without a
  proc0 value with its size the row is INCOMPLETE (the row is about proc0's headroom with the
  platform handlers on top). A `p0_size=`/`p1_size=` that differs from the ELF's
  `stack0`/`stack1` is FAIL `reason=other-image` (the capture is of another build).
- `size`: stack-paths' size budget line; PASS if every budget has room left.
- `plat_verify`: PASS only if every row is PASS; FAIL if any row is FAIL; else INCOMPLETE.

A row without evidence (no ELF, no capture, no line, a tool that did not run) is INCOMPLETE,
never PASS. The ELF and every capture are named with their sha256 first, and the stack-paths
that ran with its sha256 (--stack-paths) or this repository's commit and dirty flag (cargo run).
Exit 0 = PASS, 1 = FAIL, 3 = INCOMPLETE, 2 = usage error.
"""
import argparse
import hashlib
import os
import pathlib
import re
import subprocess
import sys

MIN_STACK_FREE = 512
PASS, FAIL, INCOMPLETE = "PASS", "FAIL", "INCOMPLETE"
REPO = pathlib.Path(__file__).resolve().parent.parent

# Watermark lines of the lab firmware: (marker, core, field). A line counts when it has both.
WATERMARKS = [
    (re.compile(r"\bIRQ_STACK\b"), 0, re.compile(r"\bp0_free=(\d+)")),
    (re.compile(r"\bIRQ_STACK\b"), 1, re.compile(r"\bp1_free=(\d+)")),
    (re.compile(r"\bstack_free="), 1, re.compile(r"\bstack_free=(\d+)")),
    (re.compile(r"\bRINGDMA_STACK\b"), 1, re.compile(r"\bfree_bytes=(\d+)")),
]
# The stack size a watermark line was measured against, per core.
SIZES = [re.compile(r"\bIRQ_STACK\b.*\bp0_size=(\d+)"), re.compile(r"\bIRQ_STACK\b.*\bp1_size=(\d+)")]
FIELD = re.compile(r"\b([a-z][a-z0-9_]*)=(-?\d+)(?:/(\d+))?")


def stack_row(output, status):
    """`stack_static` from stack-paths' output and exit status (None: it did not run)."""
    if output is None or status is None:
        return INCOMPLETE, "stack-paths did not run"
    lines = [l for l in output.splitlines() if l.startswith("stack_static=")]
    if len(lines) != 1:
        return INCOMPLETE, "no stack_static line"
    verdict, _, detail = lines[0][len("stack_static="):].partition(" ")
    if verdict == PASS and status == 0:
        return PASS, detail
    if verdict == FAIL and status == 1:
        return FAIL, detail
    if verdict in (PASS, "DIAG") and status in (0, 1):
        return FAIL, f"tool={verdict} exit={status} {detail}"
    return INCOMPLETE, f"stack-paths exit={status}"


def watermark_row(texts, stacks):
    """`stack_watermark_free` from the captures (a list of texts; None: not given) and the ELF's
    `(stack0, stack1)` from the size budget (None: unknown)."""
    free, sizes = ([], []), (set(), set())
    for text in texts:
        for line in (text or "").splitlines():
            for marker, core, field in WATERMARKS:
                if marker.search(line):
                    free[core].extend(int(v) for v in field.findall(line))
            for core, size in enumerate(SIZES):
                sizes[core].update(int(v) for v in size.findall(line))
    if not free[0]:
        return INCOMPLETE, f"no proc0 watermark (p0_free=), proc1 n={len(free[1])}"
    if not sizes[0]:
        return INCOMPLETE, "no proc0 stack size (p0_size=)"
    if stacks is None:
        return INCOMPLETE, "no ELF stack sizes to compare p0_size= with"
    for core in (0, 1):
        if sizes[core] - {stacks[core]}:
            seen = ",".join(map(str, sorted(sizes[core])))
            return FAIL, f"reason=other-image p{core}_size={seen} stack{core}={stacks[core]}"
    low = [min(v) if v else None for v in free]
    ok = all(v is None or v >= MIN_STACK_FREE for v in low)
    p1 = "none" if low[1] is None else low[1]
    detail = (f"p0_min={low[0]} p1_min={p1} n={len(free[0]) + len(free[1])} need={MIN_STACK_FREE} "
              f"stack0={stacks[0]}")
    return (PASS if ok else FAIL), detail


def budget(output):
    """The `size budget` line of `stack-paths --size` and its fields, or None."""
    lines = [l for l in (output or "").splitlines() if l.startswith("size budget ")]
    if len(lines) != 1:
        return None
    return lines[0], {k: int(v) for k, v, _ in FIELD.findall(lines[0])}


def elf_stacks(output):
    """`(stack0, stack1)` from the size budget line, or None."""
    fields = (budget(output) or (None, {}))[1]
    if "stack0" not in fields or "stack1" not in fields:
        return None
    return fields["stack0"], fields["stack1"]


def size_row(output):
    """`size` from the `size budget` line of `stack-paths --size`."""
    found = budget(output)
    if found is None:
        return INCOMPLETE, "no size budget line"
    line, fields = found
    free = ["sram_free", "isram_free", "dsram_free0", "dsram_free1"]
    if any(k not in fields for k in free):
        return INCOMPLETE, "incomplete size budget line"
    detail = line[len("size budget "):]
    return (PASS if all(fields[k] >= 0 for k in free) else FAIL), detail


def overall(rows):
    values = [v for v, _ in rows]
    if all(v == PASS for v in values):
        return PASS
    return FAIL if FAIL in values else INCOMPLETE


def tool_version(exe):
    """Which stack-paths gives the verdict: the executable's sha256, or this repository's commit
    and whether its tree is dirty (cargo run builds from it)."""
    if exe:
        path = pathlib.Path(exe)
        return f"{exe} sha256={sha256(path)}" if path.is_file() else f"{exe} sha256=unknown"

    def git(*args):
        try:
            done = subprocess.run(["git", *args], cwd=REPO, capture_output=True, text=True,
                                  check=False)
        except OSError:
            return None
        return done.stdout.strip() if done.returncode == 0 else None

    head, status = git("rev-parse", "HEAD"), git("status", "--porcelain")
    dirty = "unknown" if status is None else int(bool(status))
    return f"cargo-run git={head or 'unknown'} dirty={dirty}"


def run_stack_paths(elf, exe):
    print(f"input stack_paths={tool_version(exe)}")
    if exe:
        command = [exe]
    else:
        command = ["cargo", "run", "-q", "--release", "-p", "stack-paths", "--"]
    try:
        done = subprocess.run(command + [str(elf), "--size"], cwd=REPO, capture_output=True,
                              text=True, check=False)
    except OSError as e:
        print(f"INFO stack-paths: {e}")
        return None, None
    if done.stderr.strip():
        print(f"INFO stack-paths stderr: {done.stderr.strip().splitlines()[-1]}")
    return done.stdout, done.returncode


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def read(name, path):
    """The text of a capture, or None (printed as missing)."""
    if path is None:
        print(f"input {name}=none")
        return None
    path = pathlib.Path(path)
    if not path.is_file():
        print(f"input {name}={path} missing")
        return None
    print(f"input {name}={path} sha256={sha256(path)}")
    return path.read_bytes().decode(errors="replace")


def main(argv):
    p = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    p.add_argument("--elf", required=True)
    p.add_argument("--uart0")
    p.add_argument("--uart10")
    p.add_argument("--stack-paths", default=os.environ.get("STACK_PATHS"))
    a = p.parse_args(argv)

    elf = pathlib.Path(a.elf)
    output = status = None
    if elf.is_file():
        print(f"input elf={elf} sha256={sha256(elf)}")
        output, status = run_stack_paths(elf, a.stack_paths)
    else:
        print(f"input elf={elf} missing")
    texts = [read("uart0", a.uart0), read("uart10", a.uart10)]

    rows = [stack_row(output, status), watermark_row(texts, elf_stacks(output)), size_row(output)]
    for name, (value, detail) in zip(["stack_static", "stack_watermark_free", "size"], rows):
        print(f"{name}={value} {detail}")
    verdict = overall(rows)
    print(f"plat_verify={verdict}")
    return {PASS: 0, FAIL: 1, INCOMPLETE: 3}[verdict]


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
