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
  load (`IRQ_STACK p0_free= p1_free=`, `stack_free=`, `RINGDMA_STACK free_bytes=`), observed,
  not a bound. PASS if at least 512 B; no such line is INCOMPLETE.
- `size`: stack-paths' size budget line; PASS if every budget has room left.
- `plat_verify`: PASS only if every row is PASS; FAIL if any row is FAIL; else INCOMPLETE.

A row without evidence (no ELF, no capture, no line, a tool that did not run) is INCOMPLETE,
never PASS. The ELF and every capture are named with their sha256 first.
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

# Watermark lines of the lab firmware: (marker, field) pairs. A line counts when it has both.
WATERMARKS = [
    (re.compile(r"\bIRQ_STACK\b"), re.compile(r"\bp[01]_free=(\d+)")),
    (re.compile(r"\bstack_free="), re.compile(r"\bstack_free=(\d+)")),
    (re.compile(r"\bRINGDMA_STACK\b"), re.compile(r"\bfree_bytes=(\d+)")),
]
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


def watermark_row(texts):
    """`stack_watermark_free` from the captures (a list of texts; None: not given)."""
    values = []
    for text in texts:
        for line in (text or "").splitlines():
            for marker, field in WATERMARKS:
                if marker.search(line):
                    values += [int(v) for v in field.findall(line)]
    if not values:
        return INCOMPLETE, "no watermark line"
    low = min(values)
    return (PASS if low >= MIN_STACK_FREE else FAIL), f"min={low} n={len(values)} need={MIN_STACK_FREE}"


def size_row(output):
    """`size` from the `size budget` line of `stack-paths --size`."""
    lines = [l for l in (output or "").splitlines() if l.startswith("size budget ")]
    if len(lines) != 1:
        return INCOMPLETE, "no size budget line"
    fields = {k: int(v) for k, v, _ in FIELD.findall(lines[0])}
    free = ["sram_free", "isram_free", "dsram_free0", "dsram_free1"]
    if any(k not in fields for k in free):
        return INCOMPLETE, "incomplete size budget line"
    detail = lines[0][len("size budget "):]
    return (PASS if all(fields[k] >= 0 for k in free) else FAIL), detail


def overall(rows):
    values = [v for v, _ in rows]
    if all(v == PASS for v in values):
        return PASS
    return FAIL if FAIL in values else INCOMPLETE


def run_stack_paths(elf, exe):
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

    rows = [stack_row(output, status), watermark_row(texts), size_row(output)]
    for name, (value, detail) in zip(["stack_static", "stack_watermark_free", "size"], rows):
        print(f"{name}={value} {detail}")
    verdict = overall(rows)
    print(f"plat_verify={verdict}")
    return {PASS: 0, FAIL: 1, INCOMPLETE: 3}[verdict]


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
