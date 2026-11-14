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
regression of the same boot is required too unless --proof-only is given. Findings that are
not pass conditions (`IRQ_INTF_ONLY`) are printed as `INFO`.

--sweep checks that the sweep ran by its contract, not what it found: one `IRQ_SWEEP` line for
each line of the table between `IRQ_SWEEP_BEGIN` and `IRQ_SWEEP_END`, every line with the
boot's nonce, no line enabled in the NVIC, and the whole sweep before the PCIe link came up
(`p0 pcie Ok` after `IRQ_SWEEP_END`, with a fresh PERSTn wait). `NOT_ROUTED`, `OTHER_LINE`,
`NEW_PENDING` and `SKIP` are results; `ABORT` is a failure. --json writes the per-line results.
UART10 is optional with --sweep; if given, the Linux side (`check-sweep.sh`) must have passed
in the boot that printed the firmware's boot value.
Exit 0 = PASS, 1 = FAIL, 2 = usage error.
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
CONTRACT = {
    "IRQ_PROOF_BEGIN": {},
    "IRQ_P0_TIMER0": {**TIMER, "ipsr": 42},
    "IRQ_P1_TIMER1": {**TIMER, "ipsr": 43},
    "IRQ_INTF_ONLY core=0": {**INTF, "pin": 22},
    "IRQ_INTF_ONLY core=1": {**INTF, "pin": 27},
    "IRQ_P0_GPIO22": {"n": 100, "ipsr": 16, "pre_pending": 0, "post_pending": 0},
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


def read_text(path):
    with open(path, "rb") as f:
        return f.read().decode("utf-8", errors="replace").replace("\r", "").replace("\x00", "")


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
UART10_REGRESSION = [
    ("SHM_PASS", r"\bSHM_PASS\b"),
    ("SHM_RC=0", r"\bSHM_RC=0\b"),
    ("doorbell after", r"\bDOORBELL_AFTER_RC=0\b"),
    ("script result", r"\bIRQ_CHECK_RESULT=PASS\b"),
    ("terminal marker", r"\bIRQ_CHECK_END\b"),
]
UART10_POWER = [
    ("USB-PD negotiated", r"\busb_pd_init status 3\b"),
    ("3 A supply", r"\bmax-current 3000\b"),
]
UART10_FORBIDDEN = [r"\bCHECK_FAIL\b", r"\bIRQ_CHECK_RESULT=FAIL\b", r"Kernel panic", r"\bSError\b"]


def verify_uart10(text, nonce, steps):
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
    rtt = re.search(r"\bdoorbell rtt n=1000 .*", boot)
    if rtt:
        info.append(rtt.group(0).strip())
    return failures, info


# The sweep table of lab/irq-sweep: IRQ number -> block. Nothing else may be swept; the lines
# in SWEEP_FORBIDDEN have no safe stimulus.
SWEEP_TABLE = {7: "i2c0", 9: "i2c2", 10: "i2c3", 20: "spi1", 22: "spi3", 52: "adc",
               41: "pwm1", 1: "io_bank1", 2: "io_bank2"}
SWEEP_FORBIDDEN = (24, 40, 50, 51, 59, 60)
SWEEP_RESULTS = ("ROUTED", "NOT_ROUTED", "OTHER_LINE", "NEW_PENDING", "SKIP", "ABORT")
NO_FRESH_PERSTN = 0xFFFF_FFFF
UART10_SWEEP = [
    ("script result", r"\bSWEEP_CHECK_RESULT=PASS\b"),
    ("terminal marker", r"\bSWEEP_CHECK_END\b"),
]


def word_pair(text):
    """`a,b` in hexadecimal -> the 64 pending bits as one number, or None."""
    parts = (text or "").split(",")
    if len(parts) != 2:
        return None
    try:
        return int(parts[0], 16) | int(parts[1], 16) << 32
    except ValueError:
        return None


def sweep_line(fields):
    """(failures, record) for one `IRQ_SWEEP` line."""
    irq, block, result = number(fields.get("irq", "")), fields.get("block"), fields.get("result")
    record = {"irq": irq, "block": block, "result": result, "reason": fields.get("reason"),
              "detail": fields.get("detail"), "hits": fields.get("hits"),
              "ispr_before": fields.get("ispr_before"), "ispr_after": fields.get("ispr_after"),
              "new": fields.get("new"), "status": fields.get("status")}
    if irq in SWEEP_FORBIDDEN:
        return [f"irq={irq}: must not be swept"], record
    if SWEEP_TABLE.get(irq) != block:
        return [f"irq={irq} block={block}: not in the sweep table"], record
    if result not in SWEEP_RESULTS:
        return [f"irq={irq}: result={result}"], record
    before, after, new = (word_pair(fields.get(k)) for k in ("ispr_before", "ispr_after", "new"))
    if None in (before, after, new):
        return [f"irq={irq}: pending words missing or not hexadecimal"], record
    failures, target = [], 1 << irq
    if result == "ABORT":
        failures.append(f"irq={irq}: ABORT reason={fields.get('reason')}")
    # The words must say what the result says.
    if result in ("ROUTED", "NOT_ROUTED", "OTHER_LINE", "NEW_PENDING"):
        if before & target:
            failures.append(f"irq={irq}: result={result} but the target bit was pending before")
        if bool(after & target) != (result in ("ROUTED", "NEW_PENDING")):
            failures.append(f"irq={irq}: result={result} does not match ispr_after")
        if bool(new) != (result in ("OTHER_LINE", "NEW_PENDING")):
            failures.append(f"irq={irq}: result={result} does not match new={fields.get('new')}")
        if new != after & ~before & ~target:
            failures.append(f"irq={irq}: new={fields.get('new')} is not ispr_after minus ispr_before")
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

    failures, records, extra, ended, link = [], {}, {}, None, None
    for line in boot:
        if re.search(r"\bp0 pcie ", line):
            m = re.search(r"\bp0 pcie Ok .*\bendpoint_programmed=true perstn_wait_us=(\d+)", line)
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
        elif name == "IRQ_SWEEP":
            more, record = sweep_line(fields)
            failures += [f"UART0 IRQ_SWEEP {f}" for f in more]
            if record["irq"] in records:
                failures.append(f"UART0 IRQ_SWEEP irq={record['irq']}: more than once")
            records[record["irq"]] = record
            summary["lines"].append(record)
        elif name in ("IRQ_SWEEP_PRE", "IRQ_SWEEP_TRIAL"):
            irq = number(fields.get("irq", ""))
            if irq not in SWEEP_TABLE or irq in records:
                failures.append(f"UART0 {name}: irq={irq} is not the line being swept")
            extra.setdefault(irq, []).append((name, fields))
        else:
            failures.append(f"UART0 unknown marker: {line.strip()[:160]}")

    for record in summary["lines"]:
        mine = extra.get(record["irq"], [])
        record["funcsel"] = next((f.get("fs") for n, f in mine if n == "IRQ_SWEEP_PRE"), None)
        record["trials"] = [f for n, f in mine if n == "IRQ_SWEEP_TRIAL"]
    for irq in SWEEP_TABLE:
        if irq not in records:
            failures.append(f"UART0 IRQ_SWEEP irq={irq}: missing")
    begin = summary.get("begin", {})
    if begin.get("iser") != "00000000,00000000":
        failures.append(f"UART0 IRQ_SWEEP_BEGIN: iser={begin.get('iser')} (a line was enabled)")
    if ended is None:
        failures.append("UART0: no IRQ_SWEEP_END")
    else:
        if ended.get("iser") != "00000000,00000000":
            failures.append(f"UART0 IRQ_SWEEP_END: iser={ended.get('iser')} (a line was enabled)")
        if not number(ended.get("lines", "")) == number(begin.get("lines", "")) == len(summary["lines"]):
            failures.append(f"UART0 IRQ_SWEEP_END: lines={ended.get('lines')}, "
                            f"{len(summary['lines'])} IRQ_SWEEP lines, BEGIN lines={begin.get('lines')}")
        summary["us"] = number(ended.get("us", ""))
    if link is None:
        failures.append("UART0: no `p0 pcie` line after the sweep")
    if summary["boot"] is None:
        failures.append("UART0: no IRQ_SWEEP_BOOT")
    if re.search(r"\bIRQ_SWEEP_PANIC\b", "\n".join(boot)):
        failures.append("UART0: IRQ_SWEEP_PANIC")
    return failures, summary


def main_sweep(a):
    failures, summary = verify_sweep(read_text(a.uart0), a.nonce)
    boot = summary["boot"]
    if a.uart10 and boot is not None and number("0x" + boot) is not None:
        more, _ = verify_uart10(read_text(a.uart10), boot, UART10_SWEEP)
        failures += more
    elif a.uart10:
        failures.append("UART10: not checked, the firmware printed no boot value")
    for record in summary["lines"]:
        print("INFO irq={irq} block={block} result={result} reason={reason} hits={hits}".format(**record))
    for line in failures:
        print(f"FAIL {line}")
    summary["verdict"] = "FAIL" if failures else "PASS"
    summary["failures"] = failures
    if a.json:
        with open(a.json, "w") as f:
            json.dump(summary, f, indent=2)
            f.write("\n")
    print(f"IRQ_SWEEP_VERDICT={summary['verdict']} nonce={summary['nonce']} boot={boot}")
    return 1 if failures else 0


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
