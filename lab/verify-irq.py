#!/usr/bin/env python3
"""Strict pass/fail for one boot of the interrupt proof image (`lab/irq-proof`, `lab/check-irq.sh`).

Usage: verify-irq.py [--uart0 FILE] [--uart10 FILE] [--nonce HEX] [--proof-only]
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
not pass conditions (`IRQ_INTF_ONLY`, `IRQ_SWEEP`) are printed as `INFO`.
Exit 0 = PASS, 1 = FAIL, 2 = usage error.
"""
import argparse
import os
import re
import sys

MIN_STACK_FREE = 512
MAX_MASKED_US = 100
LOG_MAIN_LINES = 20

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
    "IRQ_P1_GPIO27": {"n": 1000, "ipsr": 16, "seen": at_least(1000)},
    "IRQ_XCORE": {"p0_seen_27": 0, "p1_seen_26": 0},
    "IRQ_UNHANDLED": {"p0": 0, "p1": 1},
    "IRQ_STACK": {"p0_free": at_least(MIN_STACK_FREE), "p1_free": at_least(MIN_STACK_FREE)},
    "IRQ_MASKED": {"p0_max_us": at_most(MAX_MASKED_US), "p1_max_us": at_most(MAX_MASKED_US)},
    "IRQ_PROOF_PASS": {},
}
# Lines that may repeat or be absent; they never decide by themselves.
INFORMATIONAL = ("IRQ_LOG_MAIN", "IRQ_LOG_ISR", "IRQ_INTF_INTE", "IRQ_SWEEP", "IRQ_SWEEP_DONE")


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


def verify_uart0(text, nonce=None):
    """Returns (failures, info, nonce) for the selected boot of a UART0 capture."""
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

    failures, info, seen, order = [], [], {}, []
    for line in boot:
        parsed = parse(line)
        if parsed is None:
            m = re.search(r"\bp1 shm cmd=\d+ .*\bstack_free=(\d+)", line)
            if m and int(m.group(1)) < MIN_STACK_FREE:
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


def verify_uart10(text, nonce, regression=True):
    """Returns (failures, info) for the boot of a UART10 capture that printed `nonce`."""
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
    for name, pattern in UART10_PROOF + (UART10_REGRESSION if regression else []):
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


def main(argv):
    p = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    p.add_argument("--uart0", default=os.environ.get("IRQ_UART0"))
    p.add_argument("--uart10", default=os.environ.get("IRQ_UART10"))
    p.add_argument("--nonce", help="check the boot with this nonce, not the last one")
    p.add_argument("--proof-only", action="store_true",
                   help="do not require the SHM regression of the same boot")
    a = p.parse_args(argv)
    if not a.uart0 or not a.uart10:
        p.error("give --uart0 and --uart10 (or set IRQ_UART0 and IRQ_UART10)")
    failures, info, nonce = verify_uart0(read_text(a.uart0), a.nonce)
    if nonce is not None and number("0x" + nonce) is not None:
        more, more_info = verify_uart10(read_text(a.uart10), nonce, regression=not a.proof_only)
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
