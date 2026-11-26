#!/usr/bin/env python3
"""Strict pass/fail for one boot of the interrupt proof image (`lab/irq-proof`, `lab/check-irq.sh`),
or with --sweep of the sweep image (`lab/irq-sweep`, `lab/check-sweep.sh`).

Usage: verify-irq.py [--uart0 FILE] [--uart10 FILE] [--nonce HEX] [--proof-only]
       verify-irq.py --sweep [--uart0 FILE] [--uart10 FILE] [--nonce HEX] [--json FILE]
       (files default to the environment variables IRQ_UART0 and IRQ_UART10)

UART0 is the RP1 log, UART10 the bootloader banner and the Linux console; raw captures or
captures with a prefix on each line both work. A capture may hold several boots:

- UART0 is cut at each `IRQ_PROOF_BEGIN`; the boot checked is the last one, or the one whose
  nonce is given with --nonce. Every marker line of that boot must carry its nonce, so a line
  of another boot in the range is an error.
- UART10 is cut at each bootloader banner (`RPi: BOOTSYS`); the boot checked is the one where
  `check-irq.sh` printed the same nonce (`IRQ_NONCE=`), which it reads from the firmware.

PASS needs every marker once, with the values of the contract below (not just present), no
`IRQ_PROOF_FAIL`, `IRQ_PROOF_PASS` as the last marker, the USB-PD banner lines
(`usb_pd_init status 3`, `max-current 3000`), and the doorbell run on the Linux side. The SHM
regression and the notification proofs of the same boot (UART10_NOTIFY, in check-irq.sh's
order; any `<NAME>_STALL` fails) are required too unless --proof-only is given. Findings that
are not pass conditions (`IRQ_INTF_ONLY`, the record-only `CONTROL_J1` / `CONTROL_J2` lines)
are printed as `INFO`.

Terminal control sequences in a capture (a console's bracketed-paste switches, colours) are
removed before anything is matched.

--sweep PASS means every line of the table was routed by its expected stimulus: one
`IRQ_SWEEP ... result=ROUTED exit=left` line per table line, in table order, with the hits of
SWEEP_KINDS (I2C/SPI: wrapper closed 0/3, open 3/3; ADC: forced and real 3/3; PWM1 and the IO
banks: forced 3/3), and its `IRQ_SWEEP_TRIAL` lines must bear that out: all trials in order,
the source raised (`raised=1` and the status bit), the target bit clear before the stimulus and
after the deassert, no other new pending bit, and the same hit counts. Any other result
(NOT_ROUTED, OTHER_LINE, NEW_PENDING, SKIP, ABORT) is a FAIL that names it. The frame must hold
too: every line with the boot's nonce, no line enabled in the NVIC, and the whole sweep before
the PCIe link came up (`PLAT_INIT ok=1` after `IRQ_SWEEP_END`, with a fresh PERSTn wait). --json
writes the per-line results.
UART0 supplies the sweep measurements. The healthy-run verdict also needs the Linux side
(`check-sweep.sh`) with both cores Running, bound to the same boot value.
Exit 0 = PASS, 1 = FAIL, 2 = usage error, 3 = sweep measurements pass but Linux evidence is missing.
"""
import argparse
import json
import os
import re
import sys

MIN_STACK_FREE = 512
MAX_MASKED_US = 100
LOG_MAIN_LINES = 20
# shm-linux's default suite: commands 1..8 and 10..13, each answered with a `p1 shm cmd=` line.
SHM_COMMANDS = 12

MARKER = re.compile(r"\bIRQ_[A-Z0-9_]+\b")
FIELD = re.compile(r"\b([a-z][a-z0-9_]*)=(\S+)")


def between(low, high):
    return lambda v: low <= v <= high


def at_least(low):
    return lambda v: v >= low


def at_most(high):
    return lambda v: v <= high


# Marker -> the fields that decide, each with the value or range it must have. A marker's key
# is its name plus the words before the first field (`IRQ_WRAP i2c1`), or plus the fields that
# only tell instances apart (`IRQ_INTF_ONLY core=0`).
TIMER = {"n": 1000, "us": between(999_000, 1_010_000), "pre_pending": 0, "post_pending": 0}
WRAP = {"pre_pending": 0, "off_stat": 1, "off_pending": 0, "on_n": 1, "post_pending": 0}
INTF = {"pre_pending": 0, "post_pending": 0}
DBELL = {"n": 1, "ipsr": 16, "pre_pending": 0, "post_pending": 0}
CONTRACT = {
    "IRQ_PROOF_BEGIN": {"sweep": 0},
    "IRQ_P0_TIMER0": {**TIMER, "ipsr": 42},
    "IRQ_P1_TIMER1": {**TIMER, "ipsr": 43},
    "IRQ_INTF_ONLY core=0": {**INTF, "pin": 22},
    "IRQ_P0_GPIO22": {"n": 100, "ipsr": 16, "pre_pending": 0, "post_pending": 0},
    "IRQ_P0_DBELL": {**DBELL, "pin": 26},
    "IRQ_P1_DBELL": {**DBELL, "pin": 27},
    "IRQ_WRAP i2c1": {**WRAP, "ipsr": 24},
    "IRQ_WRAP spi0": {**WRAP, "ipsr": 35},
    "IRQ_CS": {"iters": at_least(100_000), "isr_n": at_least(1000), "torn": 0,
               "control_torn": at_least(1), "max_masked_us": at_most(MAX_MASKED_US)},
    "IRQ_LOG": {"main_lines": LOG_MAIN_LINES, "isr_dropped": at_least(1),
                "drop_max_us": at_most(MAX_MASKED_US)},
    "IRQ_DEFAULT": {"irq": 28, "count": 1, "masked": 1, "pre_count": 0, "pre_pending": 0},
    "IRQ_P1_GPIO27": {"ipsr": 16, "seen": at_least(1000)},
    "IRQ_XCORE": {"p0_seen_27": 0, "p1_seen_26": 0},
    "IRQ_UNHANDLED": {"p0": 0, "p1": 1},
    "IRQ_STACK": {"p0_free": at_least(MIN_STACK_FREE), "p1_free": at_least(MIN_STACK_FREE)},
    "IRQ_MASKED": {"p0_max_us": at_most(MAX_MASKED_US), "p1_max_us": at_most(MAX_MASKED_US)},
    "IRQ_PROOF_PASS": {},
}
# Lines that may repeat or be absent; they never decide by themselves.
INFORMATIONAL = ("IRQ_LOG_MAIN", "IRQ_LOG_ISR", "IRQ_INTF_INTE")


# CSI sequences (`ESC [ ... final`, e.g. the bracketed-paste `ESC [ ? 2004 h` and
# `ESC [ 200 ~`) and the other two-byte escapes.
CONTROL_SEQUENCE = re.compile(r"\x1b\[[0-?]*[ -/]*[@-~]|\x1b[@-Z\\-_]")


def clean(text):
    return CONTROL_SEQUENCE.sub("", text).replace("\r", "").replace("\x00", "")


def read_text(path):
    with open(path, "rb") as f:
        return clean(f.read().decode("utf-8", errors="replace"))


def number(text):
    try:
        return int(text, 0)
    except ValueError:
        return None


def parse(line):
    """(key, fields) of the marker in `line`, or None."""
    m = MARKER.search(line)
    if m is None:
        return None
    name, rest = m.group(0), line[m.end():]
    fields = dict(FIELD.findall(rest))
    words = rest.split("=", 1)[0].split()[:-1] if "=" in rest else rest.split()
    key = " ".join([name] + words)
    if name == "IRQ_INTF_ONLY":
        key += f" core={fields.get('core')}"
    return key, fields


def boots(lines, start):
    """The line ranges that begin at each line matching `start`."""
    at = [i for i, line in enumerate(lines) if re.search(start, line)]
    return [lines[a:b] for a, b in zip(at, at[1:] + [len(lines)])]


def verify_uart0(text, nonce=None, regression=True):
    """Returns (failures, info, nonce) for the selected boot of a UART0 capture. With
    `regression`, proc1's stack must also have been reported after each shm command."""
    found = boots(text.splitlines(), r"\bIRQ_PROOF_BEGIN\b")
    if not found:
        return ["UART0: no IRQ_PROOF_BEGIN"], [], None
    if nonce is None:
        boot = found[-1]
        nonce = parse(boot[0])[1].get("nonce")
        if nonce is None:
            return ["UART0: IRQ_PROOF_BEGIN without a nonce"], [], None
    else:
        matching = [b for b in found if parse(b[0])[1].get("nonce") == nonce]
        if len(matching) != 1:
            return [f"UART0: {len(matching)} boots with nonce {nonce}"], [], nonce
        boot = matching[0]

    failures, info, seen, order, commands = [], [], {}, [], 0
    for line in boot:
        parsed = parse(line)
        if parsed is None:
            m = re.search(r"\bp1 shm cmd=\d+ .*\bstack_free=(\d+)", line)
            if m:
                commands += 1
                if int(m.group(1)) < MIN_STACK_FREE:
                    failures.append(f"UART0 stack after a shm command: {line.strip()}")
            continue
        key, fields = parsed
        name = key.split()[0]
        if name == "IRQ_LOG_ISR":
            seen.setdefault(key.split()[0], []).append(fields)
            continue
        if fields.get("nonce") != nonce:
            failures.append(f"UART0 line of another boot or cut short: {line.strip()[:160]}")
            continue
        if name == "IRQ_PROOF_FAIL":
            failures.append(f"UART0 failure marker: {line.strip()[:160]}")
            continue
        if name in INFORMATIONAL:
            seen.setdefault(name, []).append(fields)
            if name not in ("IRQ_LOG_MAIN",):
                info.append(line[MARKER.search(line).start():].strip())
            continue
        order.append(key)
        if key not in CONTRACT:
            failures.append(f"UART0 unknown marker: {line.strip()[:160]}")
        elif key in seen:
            failures.append(f"UART0 {key}: more than once")
        else:
            seen[key] = fields
            if name == "IRQ_INTF_ONLY":
                info.append(line[MARKER.search(line).start():].strip())

    for key, wanted in CONTRACT.items():
        fields = seen.get(key)
        if fields is None:
            failures.append(f"UART0 {key}: missing")
            continue
        for field, want in wanted.items():
            value = number(fields.get(field, ""))
            ok = value is not None and (want(value) if callable(want) else value == want)
            if not ok:
                failures.append(f"UART0 {key}: {field}={fields.get(field)}")
    if regression and commands < SHM_COMMANDS:
        failures.append(f"UART0 stack after shm commands: {commands} lines, not {SHM_COMMANDS}")
    if order and order[-1] != "IRQ_PROOF_PASS" and "IRQ_PROOF_PASS" in order:
        failures.append(f"UART0 {order[-1]}: after IRQ_PROOF_PASS")

    main = [number(f.get("i", "")) for f in seen.get("IRQ_LOG_MAIN", [])]
    if main != list(range(LOG_MAIN_LINES)):
        failures.append(f"UART0 IRQ_LOG_MAIN: lines {main}, not 0..{LOG_MAIN_LINES - 1} in order")
    log = seen.get("IRQ_LOG", {})
    tries, dropped, logged = (number(log.get(k, "")) for k in ("isr_tries", "isr_dropped", "isr_logged"))
    if None not in (tries, dropped, logged):
        if logged != tries - dropped:
            failures.append(f"UART0 IRQ_LOG: isr_logged={logged} is not isr_tries - isr_dropped")
        if len(seen.get("IRQ_LOG_ISR", [])) != logged:
            failures.append(f"UART0 IRQ_LOG_ISR: {len(seen.get('IRQ_LOG_ISR', []))} lines, isr_logged={logged}")
    return failures, info, nonce


# Linux console, in the order check-irq.sh prints them.
UART10_PROOF = [
    ("doorbell replies", r"\bdoorbell rtt n=1000 "),
    ("doorbell before", r"\bDOORBELL_BEFORE_RC=0\b"),
]
# The notification proofs (shm-linux down_irq.rs / up_irq.rs), in check-irq.sh's order.
UART10_NOTIFY = [
    ("(b) DOWNIRQ", r"\bDOWNIRQ ring=down1 n=1000000 missing=0 corrupt=0 stalls=0 "),
    ("(b) DOWNIRQ_PASS", r"\bDOWNIRQ_PASS\b"),
    ("(h) DOWNIRQ_MORE", r"\bDOWNIRQ_MORE raises=[1-9]\d* missing=0 corrupt=0 stalls=0\b"),
    ("(h) DOWNIRQ_MORE_PASS", r"\bDOWNIRQ_MORE_PASS\b"),
    ("(c)(d) DOWN0IRQ", r"\bDOWN0IRQ ring=down0 n=10000 missing=0 corrupt=0 stalls=0 "),
    ("(c)(d) DOWN0IRQ_PASS", r"\bDOWN0IRQ_PASS\b"),
    ("(g) RINGIRQ0_PASS", r"\bRINGIRQ0_PASS\b"),
    ("(g) SCMI still serving", r"\bRP1_STATUS .*\bpcie=Up scmi=Serving\(requests=[1-9]"),
    ("(g) cores healthy", r"\bRP1_HEALTH Ok\b"),
    ("(e) MIXIRQ", r"\bMIXIRQ .*\bmissing=0 corrupt=0 .*\blate_pct=(?:0\.\d+|1\.00)\b"),
    ("(e) MIXIRQ_PASS", r"\bMIXIRQ_PASS\b"),
    ("(f) IRQPAIR_PASS", r"\bIRQPAIR_PASS\b"),
    ("(i) 50 killed writers", r"\bDOWNIRQ_KILLS=50\b"),
    ("(i) DOWNIRQ after them", r"\bDOWNIRQ ring=down1 n=10000 missing=0 corrupt=0 stalls=0 "),
    ("(i) DOWNIRQ_KILL_PASS", r"\bDOWNIRQ_KILL_PASS\b"),
    ("(i) unrung messages left", r"\bDOWNIRQ_UNRUNG ring=down1 sent=8 pending=8\b"),
    ("(i) DOWNIRQ_UNRUNG_PASS", r"\bDOWNIRQ_UNRUNG_PASS\b"),
    ("(i) reopen drained them", r"\bDOWNIRQ_REOPEN ring=down1 n=8 received=8 corrupt=0 \S+ errors=0 "),
    ("(i) DOWNIRQ_REOPEN_PASS", r"\bDOWNIRQ_REOPEN_PASS\b"),
]
UART10_REGRESSION = [
    ("SHM_PASS", r"\bSHM_PASS\b"),
    ("SHM_RC=0", r"\bSHM_RC=0\b"),
    ("doorbell after", r"\bDOORBELL_AFTER_RC=0\b"),
    *UART10_NOTIFY,
    ("script result", r"\bIRQ_CHECK_RESULT=PASS\b"),
    ("terminal marker", r"\bIRQ_CHECK_END\b"),
]
# (j): printed as INFO, never a pass condition.
UART10_RECORD = ("CONTROL_J1", "CONTROL_J2")
UART10_POWER = [
    ("USB-PD negotiated", r"\busb_pd_init status 3\b"),
    ("3 A supply", r"\bmax-current 3000\b"),
]
UART10_FORBIDDEN = [r"\bCHECK_FAIL\b", r"\bIRQ_CHECK_RESULT=FAIL\b", r"Kernel panic", r"\bSError\b",
                    r"\b(?:DOWNIRQ|DOWN0IRQ|RINGIRQ0|MIXIRQ|IRQPAIR)(?:_UNRUNG|_REOPEN)?_(?:STALL|FAIL)\b"]


def verify_uart10(text, nonce, steps, final_check=None):
    """Returns (failures, info) for the boot of a UART10 capture that printed `nonce`, where
    the Linux side must have printed `steps` in order."""
    def printed(boot):
        return [number(m) for line in boot for m in re.findall(r"\bIRQ_NONCE=(\S+)", line)]

    matching = [b for b in boots(text.splitlines(), r"\bRPi: BOOTSYS\b")
                if int(nonce, 16) in printed(b)]
    if len(matching) != 1:
        return [f"UART10: {len(matching)} boots with IRQ_NONCE={nonce} after a bootloader banner"], []
    boot = "\n".join(matching[0])
    failures, info = [], []
    for name, pattern in UART10_POWER:
        if re.search(pattern, boot) is None:
            failures.append(f"UART10 power {name}: missing ({pattern})")
    at = 0
    for name, pattern in steps:
        m = re.compile(pattern).search(boot, at)
        if m is None:
            missing = "missing" if re.search(pattern, boot) is None else "out of order"
            failures.append(f"UART10 {name}: {missing} ({pattern})")
        else:
            at = m.end()
    for pattern in UART10_FORBIDDEN:
        line = next((l for l in matching[0] if re.search(pattern, l)), None)
        if line is not None:
            failures.append(f"UART10 failure marker: {line.strip()[:160]}")
    if final_check:
        failures += verify_final_cores(boot, final_check, nonce)
    rtt = re.search(r"\bdoorbell rtt n=1000 .*", boot)
    if rtt:
        info.append(rtt.group(0).strip())
    if any(name.startswith("(b)") for name, _ in steps):
        for name in UART10_RECORD:
            line = re.search(rf"\b{name} .*", boot)
            info.append(line.group(0).strip() if line else f"{name}: not recorded")
    return failures, info


def verify_final_cores(boot, check, nonce):
    tag = f"check={check} nonce={nonce}"
    begin, end = f"LAB_FINAL_BEGIN {tag}", f"LAB_FINAL_END {tag} result=PASS"
    starts = list(re.finditer(r"\bLAB_FINAL_BEGIN\b[^\r\n]*", boot))
    ends = list(re.finditer(r"\bLAB_FINAL_END\b[^\r\n]*", boot))
    if (len(starts) != 1 or len(ends) != 1 or starts[0][0].strip() != begin
            or ends[0][0].strip() != end or starts[0].end() >= ends[0].start()):
        return ["UART10: missing or invalid final nonce-bound status"]
    status = boot[starts[0].end():ends[0].start()]
    cores = re.findall(r"\bRP1_CORE core=(\d+) state=(\S+)", status)
    if sorted(cores) != [("0", "Running"), ("1", "Running")]:
        return [f"UART10: expected both cores Running at final status, got {cores}"]
    if re.search(r"\bRP1_CORE core=\d+ state=(?:Panicked|Fault)\b", boot):
        return ["UART10: terminal core state"]
    result = boot.find("SWEEP_CHECK_RESULT=")
    if result != -1 and result < ends[0].end():
        return ["UART10: final status must precede the sweep check result"]
    return []


# The sweep table of lab/irq-sweep: IRQ number -> block. Nothing else may be swept; the lines
# in SWEEP_FORBIDDEN have no safe stimulus.
SWEEP_TABLE = {7: "i2c0", 9: "i2c2", 10: "i2c3", 20: "spi1", 22: "spi3", 52: "adc",
               41: "pwm1", 1: "io_bank1", 2: "io_bank2"}
SWEEP_FORBIDDEN = (24, 40, 50, 51, 59, 60)
SWEEP_TRIALS = 3
# Each kind of block (its name without the number): its stimuli in the order the image runs
# them, the hits each must have for the line to count as routed, and the bit of its `status`
# that shows the source raised (lab/irq-sweep/src/blocks.rs). For the DesignWare blocks the
# closed wrapper must hide the source (wrap0 0/3) and the open one pass it (wrap1 3/3): that is
# what shows the bit is this block's output and not something else's.
SWEEP_KINDS = {
    "i2c": (("wrap0", "wrap1"), (0, 3), 0x10),
    "spi": (("wrap0", "wrap1"), (0, 3), 0x1),
    "adc": (("force", "real"), (3, 3), 0x1),
    "pwm": (("force",), (3,), 0x20),
    "io_bank": (("force",), (3,), 0x1),
}
WRAPPER_STATUS = 1 << 16
NO_FRESH_PERSTN = 0xFFFF_FFFF
ZERO_WORDS = "00000000,00000000"
UART10_SWEEP = [
    ("script result", r"\bSWEEP_CHECK_RESULT=PASS\b"),
    ("terminal marker", r"\bSWEEP_CHECK_END\b"),
]


def hexadecimal(text):
    """One 32-bit word as the image prints it (lowercase hexadecimal, no sign), or None."""
    return int(text, 16) if re.fullmatch(r"[0-9a-f]{1,8}", text or "") else None


def word_pair(text):
    """`a,b`, two 32-bit words in hexadecimal -> the 64 pending bits as one number, or None."""
    parts = (text or "").split(",")
    if len(parts) != 2 or None in (low := hexadecimal(parts[0]), high := hexadecimal(parts[1])):
        return None
    return low | high << 32


def expected_hits(stages, wanted):
    return ",".join(f"{stage}:{n}/{SWEEP_TRIALS}" for stage, n in zip(stages, wanted))


def sweep_line(fields, trials):
    """(failures, record) for one `IRQ_SWEEP` line and the `IRQ_SWEEP_TRIAL` lines before it.
    The line passes only if it is ROUTED by the expected stimulus and its trials bear that out."""
    irq, block = number(fields.get("irq", "")), fields.get("block")
    record = {k: fields.get(k) for k in ("block", "result", "reason", "detail", "exit", "hits",
                                         "ispr_before", "ispr_after", "new", "status")}
    record["irq"] = irq
    if irq in SWEEP_FORBIDDEN:
        return [f"irq={irq}: must not be swept"], record
    if (irq, block) not in SWEEP_TABLE.items():
        return [f"irq={irq} block={block}: not in the sweep table"], record
    stages, wanted, source_bit = SWEEP_KINDS[block.rstrip("0123456789")]
    routed_by = stages[wanted.index(SWEEP_TRIALS)]
    want = expected_hits(stages, wanted)
    if (record["result"], record["reason"], record["hits"]) != ("ROUTED", routed_by, want):
        # NOT_ROUTED, OTHER_LINE, NEW_PENDING, SKIP and ABORT all end here.
        return [f"irq={irq}: result={record['result']} reason={record['reason']} hits={record['hits']},"
                f" not ROUTED by {routed_by} with {want}"], record
    failures = []
    if record["exit"] != "left":
        failures.append(f"irq={irq}: exit={record['exit']}, the block was not left cleanly")

    # The trials must be the image's full run, in order, each a clean observation: source
    # raised, target bit clear before the stimulus and after the deassert, no other new bit.
    target, hits = 1 << irq, [0] * len(stages)
    if len(trials) != SWEEP_TRIALS * len(stages):
        failures.append(f"irq={irq}: {len(trials)} trials, not {SWEEP_TRIALS * len(stages)}")
    for i, trial in enumerate(trials):
        stage = stages[i % len(stages)]
        where = f"irq={irq}: trial {i} (trial={trial.get('trial')} stage={trial.get('stage')})"
        if (trial.get("trial"), trial.get("stage")) != (str(i // len(stages)), stage):
            failures.append(f"{where}: out of order")
            break
        base, post, clr = (word_pair(trial.get(k)) for k in ("base", "post", "clr"))
        status = hexadecimal(trial.get("status"))
        if None in (base, post, clr, status):
            failures.append(f"{where}: base, post, clr or status missing")
            continue
        wrapper_open = bool(status & WRAPPER_STATUS)
        if trial.get("raised") != "1" or not status & source_bit or (
                stage.startswith("wrap") and wrapper_open != (stage == "wrap1")):
            failures.append(f"{where}: raised={trial.get('raised')} status={trial['status']}"
                            f" does not show the {stage} source")
        if (base | clr) & target:
            failures.append(f"{where}: target bit pending before the stimulus or after the deassert")
        if post & ~base & ~target:
            failures.append(f"{where}: another line became pending: post={trial['post']}")
        hits[i % len(stages)] += bool(post & target)
    if expected_hits(stages, hits) != want:
        failures.append(f"irq={irq}: the trials say {expected_hits(stages, hits)}, not {want}")
    return failures, record


def verify_sweep(text, nonce=None):
    """Returns (failures, summary) for the selected boot of a UART0 capture of lab/irq-sweep."""
    summary = {"nonce": nonce, "boot": None, "lines": []}
    found = boots(text.splitlines(), r"\bIRQ_SWEEP_BEGIN\b")
    if not found:
        return ["UART0: no IRQ_SWEEP_BEGIN"], summary
    if nonce is None:
        boot = found[-1]
        nonce = summary["nonce"] = parse(boot[0])[1].get("nonce")
        if nonce is None:
            return ["UART0: IRQ_SWEEP_BEGIN without a nonce"], summary
    else:
        matching = [b for b in found if parse(b[0])[1].get("nonce") == nonce]
        if len(matching) != 1:
            return [f"UART0: {len(matching)} boots with nonce {nonce}"], summary
        boot = matching[0]

    failures, seen, trials, ended, link = [], set(), [], None, None
    for line in boot:
        if re.search(r"\bPLAT_INIT ", line):
            m = re.search(r"\bPLAT_INIT ok=1 .*\bperstn_wait_us=(\d+)", line)
            # The link may only come up after the sweep, on a PERSTn release seen by the image.
            if ended is None:
                failures.append("UART0: the PCIe endpoint was brought up before IRQ_SWEEP_END")
            elif link is None and (m is None or int(m.group(1)) == NO_FRESH_PERSTN):
                failures.append(f"UART0: no fresh PCIe bring-up after the sweep: {line.strip()[:160]}")
            link = link or line
            continue
        parsed = parse(line)
        if parsed is None:
            continue
        name, fields = parsed[0].split()[0], parsed[1]
        if not name.startswith("IRQ_SWEEP"):
            failures.append(f"UART0 unknown marker: {line.strip()[:160]}")
        elif fields.get("nonce") != nonce:
            failures.append(f"UART0 line of another boot or cut short: {line.strip()[:160]}")
        elif name == "IRQ_SWEEP_BEGIN":
            summary["begin"] = fields
        elif name == "IRQ_SWEEP_BOOT":
            if summary["boot"] is not None or link is None:
                failures.append("UART0 IRQ_SWEEP_BOOT: more than once, or before the PCIe link")
            summary["boot"] = fields.get("boot")
        elif name == "IRQ_SWEEP_END":
            if ended is not None:
                failures.append("UART0 IRQ_SWEEP_END: more than once")
            ended = fields
        elif ended is not None:
            failures.append(f"UART0 sweep line after IRQ_SWEEP_END: {line.strip()[:160]}")
        elif name == "IRQ_SWEEP_TRIAL":
            trials.append(fields)
        elif name == "IRQ_SWEEP":
            # The TRIAL lines just before a result are that line's.
            irq = number(fields.get("irq", ""))
            if any(number(t.get("irq", "")) != irq for t in trials):
                failures.append(f"UART0 IRQ_SWEEP irq={irq}: preceded by another line's trials")
            more, record = sweep_line(fields, trials)
            failures += [f"UART0 IRQ_SWEEP {f}" for f in more]
            if irq in seen:
                failures.append(f"UART0 IRQ_SWEEP irq={irq}: more than once")
            seen.add(irq)
            trials = []
            summary["lines"].append(record)
        elif name != "IRQ_SWEEP_PRE":
            # PRE (function selects and identity words) is for the reader, not the verdict.
            failures.append(f"UART0 unknown marker: {line.strip()[:160]}")

    if trials:
        failures.append(f"UART0: {len(trials)} IRQ_SWEEP_TRIAL lines without their result")
    order = [r["irq"] for r in summary["lines"]]
    for irq in SWEEP_TABLE:
        if irq not in seen:
            failures.append(f"UART0 IRQ_SWEEP irq={irq}: missing")
    if sorted(map(str, order)) == sorted(map(str, SWEEP_TABLE)) and order != list(SWEEP_TABLE):
        failures.append(f"UART0 IRQ_SWEEP lines out of table order: {order}")
    begin = summary.get("begin", {})
    if begin.get("iser") != ZERO_WORDS:
        failures.append(f"UART0 IRQ_SWEEP_BEGIN: iser={begin.get('iser')} (a line was enabled)")
    if ended is None:
        failures.append("UART0: no IRQ_SWEEP_END")
    else:
        if ended.get("iser") != ZERO_WORDS:
            failures.append(f"UART0 IRQ_SWEEP_END: iser={ended.get('iser')} (a line was enabled)")
        if not number(ended.get("lines", "")) == number(begin.get("lines", "")) == len(summary["lines"]):
            failures.append(f"UART0 IRQ_SWEEP_END: lines={ended.get('lines')}, "
                            f"{len(summary['lines'])} IRQ_SWEEP lines, BEGIN lines={begin.get('lines')}")
        summary["us"] = number(ended.get("us", ""))
    if link is None:
        failures.append("UART0: no `PLAT_INIT` line after the sweep")
    if summary["boot"] is None:
        failures.append("UART0: no IRQ_SWEEP_BOOT")
    # Legacy captures had a UART0 panic marker.
    if re.search(r"\bIRQ_SWEEP_PANIC\b", "\n".join(boot)):
        failures.append("UART0: IRQ_SWEEP_PANIC")
    return failures, summary


def main_sweep(a):
    failures, summary = verify_sweep(read_text(a.uart0), a.nonce)
    boot = summary["boot"]
    measurements = "FAIL" if failures else "PASS"
    print(f"IRQ_SWEEP_MEASUREMENTS={measurements}")
    if a.uart10 and boot is not None and number("0x" + boot) is not None:
        more, _ = verify_uart10(read_text(a.uart10), boot, UART10_SWEEP, final_check="sweep")
        failures += more
    elif a.uart10:
        failures.append("UART10: not checked, the firmware printed no boot value")
    for record in summary["lines"]:
        print("INFO irq={irq} block={block} result={result} reason={reason} hits={hits}".format(**record))
    for line in failures:
        print(f"FAIL {line}")
    summary["verdict"] = "FAIL" if failures else "PASS" if a.uart10 else "INCOMPLETE"
    summary["failures"] = failures
    if a.json:
        with open(a.json, "w") as f:
            json.dump(summary, f, indent=2)
            f.write("\n")
    print(f"IRQ_SWEEP_VERDICT={summary['verdict']} nonce={summary['nonce']} boot={boot}")
    return {"PASS": 0, "FAIL": 1, "INCOMPLETE": 3}[summary["verdict"]]


def main(argv):
    p = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    p.add_argument("--uart0", default=os.environ.get("IRQ_UART0"))
    p.add_argument("--uart10", default=os.environ.get("IRQ_UART10"))
    p.add_argument("--nonce", help="check the boot with this nonce, not the last one")
    p.add_argument("--proof-only", action="store_true",
                   help="do not require the SHM regression of the same boot")
    p.add_argument("--sweep", action="store_true", help="check a boot of lab/irq-sweep")
    p.add_argument("--json", help="with --sweep: write the per-line results to this file")
    a = p.parse_args(argv)
    if a.sweep:
        if not a.uart0:
            p.error("give --uart0 (or set IRQ_UART0)")
        return main_sweep(a)
    if not a.uart0 or not a.uart10:
        p.error("give --uart0 and --uart10 (or set IRQ_UART0 and IRQ_UART10)")
    failures, info, nonce = verify_uart0(read_text(a.uart0), a.nonce, regression=not a.proof_only)
    if nonce is not None and number("0x" + nonce) is not None:
        steps = UART10_PROOF + ([] if a.proof_only else UART10_REGRESSION)
        more, more_info = verify_uart10(read_text(a.uart10), nonce, steps)
        failures += more
        info += more_info
    elif nonce is not None:
        failures.append(f"nonce {nonce!r} is not hexadecimal")
    for line in info:
        print(f"INFO {line}")
    for line in failures:
        print(f"FAIL {line}")
    print(f"IRQ_PROOF_VERDICT={'FAIL' if failures else 'PASS'} nonce={nonce}")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
