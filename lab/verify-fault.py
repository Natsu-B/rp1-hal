#!/usr/bin/env python3
"""Judge PR-E containment from one realos bundle and a pair of raw UART captures per boot."""

import argparse
import importlib.util
import pathlib
import re
import sys

# Use the same stack-paths invocation, tri-state and input hashes as the platform verifier.
_spec = importlib.util.spec_from_file_location(
    "plat_verify", pathlib.Path(__file__).with_name("plat-verify.py"))
plat = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(plat)
PASS, FAIL, INCOMPLETE = plat.PASS, plat.FAIL, plat.INCOMPLETE

KINDS = ("p0-thread-panic", "p0-isr-panic", "p0-thread-udf", "p0-isr-udf", "p1-panic")
CODES = {kind: i for i, kind in enumerate((*KINDS, "stkerr"), 1)}
PHASES = ("cold-1", "cold-2", "reboot", "poweroff")
STAGES = ("BEGIN", "PRE", "REQUEST", "STATUS", "TIMER", "PING", "USB", "SCMI", "END")
MARKER = re.compile(r"^(?:\[[^\]]*\]\s*)?FAULT_([A-Z]+) (.+)$")
BUNDLE = re.compile(r"\[rp1boot\s+\d+\] bundle ok: rp1\.bundle=([0-9a-f]{16})\b")
HEX = re.compile(r"[0-9a-f]+")
BOOT_ID = re.compile(r"[0-9a-f]{8}(?:-[0-9a-f]{4}){3}-[0-9a-f]{12}")
UNDEFINSTR = 1 << 16
STACKING = (1 << 4) | (1 << 12)  # MSTKERR or STKERR


def markers(text):
    for i, line in enumerate((text or "").splitlines()):
        match = MARKER.fullmatch(line.lstrip("\x00"))
        if match:
            stage, data = match.groups()
            pairs = [tuple(token.split("=", 1)) for token in data.split() if "=" in token]
            fields = dict(pairs)
            if len(fields) == len(pairs):
                yield i, stage, fields


def u32(value):
    if value is not None and re.fullmatch(r"(?:\d{1,10}|0x[0-9a-fA-F]{1,8})", value):
        n = int(value, 16 if value.startswith("0x") else 10)
        if n <= 0xffffffff:
            return n
    return None


def check(fields, keys, predicate):
    if any(k not in fields or fields[k] is None for k in keys):
        return INCOMPLETE
    return PASS if predicate(*(fields[k] for k in keys)) else FAIL


def numbers(fields, keys, predicate):
    return check({k: u32(fields.get(k)) for k in keys}, keys, predicate)


def boot_mode(text, begin):
    """Use bootloader reset provenance and the preceding Linux shutdown, where required."""
    prefix = "\n".join(text.splitlines()[:begin])
    previous, marker, boot = prefix.rpartition("RPi: BOOTSYS")
    modes = re.findall(r"power-on-reset ([01])\b", boot)
    if not marker or len(modes) != 1:
        return None
    shutdowns = re.findall(r"reboot: (Restarting system|Power down)", previous)
    last = shutdowns[-1] if shutdowns else None
    if modes[0] == "0":
        return "reboot" if last == "Restarting system" else None
    if last == "Power down":
        poweroff = previous.rsplit("reboot: Power down", 1)[1]
        return "poweroff" if re.search(r"Halt: power_off: 1[\s\S]*\bPower off\b", poweroff) else None
    return "cold" if last is None else None


def manifest_tuple(text, elf_sha256):
    """The deployed manifest binds the ELF SHA-256 to the loader's bundle ID."""
    bundle = re.findall(r"^bundle-id ([0-9a-f]{64})$", text or "", re.MULTILINE)
    digests = re.findall(r"^([0-9a-f]{64})  rp1\.elf$", text or "", re.MULTILINE)
    if elf_sha256 is None or len(bundle) != 1 or len(digests) != 1:
        return INCOMPLETE, None
    return (PASS if elf_sha256 == digests[0] else FAIL), bundle[0][:16]


def stack_check(output, exit_status):
    value, detail = plat.stack_row(output, exit_status)
    if value == PASS:
        fields = dict(re.findall(r"\b(s_plat|margin)=(\d+)\b", detail))
        value = numbers(fields, ("s_plat", "margin"),
                        lambda bound, margin: bound <= margin <= 768)
    return value, detail


def terminal_row(kind, status):
    core = int(kind == "p1-panic")
    identity = numbers(status, ("core", "ack"), lambda c, ack: c == core and ack == CODES[kind])
    if kind == "stkerr":
        if not u32(status.get("ack")):
            return INCOMPLETE
        state = check(status, ("state",), lambda s: s == "Fault")
        aux = numbers(status, ("aux",), lambda value: bool(value & STACKING))
        # A stacking failure may lock up without publishing Fault.
        if status.get("state") == "Running":
            return INCOMPLETE
        return overall((identity, state, aux))
    if kind.endswith("udf"):
        state = check(status, ("state",), lambda s: s == "Fault")
        details = numbers(status, ("reason", "aux"),
                          lambda reason, aux: reason == 6 and bool(aux & UNDEFINSTR))
    else:
        state = check(status, ("state",), lambda s: s == "Panicked")
        details = numbers(status, ("reason", "aux"), lambda reason, aux: reason == aux == 0)
    return overall((identity, state, details))


def counters(fields, core, live):
    return numbers(fields, (f"p{core}_before", f"p{core}_after"),
                   lambda a, b: b > a if live else a == b)


def scmi_row(scmi, status, rates):
    states = (status.get("scmi"), scmi.get("scmi_before"), scmi.get("scmi_after"))
    if any(s in ("NotStarted", "Quarantined") or (s or "").startswith("Unknown(") for s in states):
        return FAIL
    if None in states or u32(scmi.get("after")) == 65535:
        return INCOMPLETE  # A saturated counter cannot establish an exact delta.
    delta = numbers(scmi, ("n", "replies", "before", "after"),
                    lambda n, replies, before, after: 0 < n <= 100
                    and n == replies == after - before and 0 <= before < after < 65535
                    and states[1] == f"Serving(requests={before})"
                    and states[2] == f"Serving(requests={after})"
                    and re.fullmatch(r"Serving\(requests=\d+\)", states[0]))
    if delta != PASS:
        return delta
    n = u32(scmi["n"])
    if len(rates) < n:
        return INCOMPLETE
    if len(rates) != n:
        return FAIL
    return overall(check(fields, ("hz",),
                         lambda hz: bool(re.fullmatch(r"[0-9]{1,20}", hz)) and int(hz) > 0)
                   for fields in rates)


def ping_row(ping):
    rc = u32(ping.get("rc"))
    if rc is not None and rc != 0:
        return FAIL
    return numbers(ping, ("sent", "received", "rc"),
                   lambda sent, received, rc: sent == received == 20 and rc == 0)


def usb_row(usb):
    if usb.get("probe") != "yes" or usb.get("evidence") in ("no_cursor", "log_lost", "io_error"):
        return INCOMPLETE
    rc = u32(usb.get("rc"))
    if rc is None:
        return INCOMPLETE
    return check(usb, ("result", "evidence"),
                 lambda result, evidence: result == PASS and evidence == "ok" and rc == 0)


def overall(values):
    return plat.overall([(v, "") for v in values])


def verify_run(kind, phase, nonce, uart0, uart10, bundle_id):
    """Return (rows, Linux boot ID), with one nonce-bound target invocation per capture."""
    boot = [fields for _, stage, fields in markers(uart0) if stage == "BOOT"]
    records = list(markers(uart10))
    begins = [(i, f) for i, s, f in records if s == "BEGIN"]
    if len(boot) != 1 or len(begins) != 1 or bundle_id is None:
        return {"identity": INCOMPLETE}, None
    start, begin = begins[0]
    boot_id = begin.get("boot_id", "")
    if (boot[0].get("nonce") != nonce or begin.get("nonce") != nonce or begin.get("kind") != kind
            or kind not in boot[0].get("kinds", "").split(",") or not BOOT_ID.fullmatch(boot_id)):
        return {"identity": INCOMPLETE}, None

    prefix = "\n".join(uart10.splitlines()[:start])
    boot_prefix = prefix.rsplit("RPi: BOOTSYS", 1)[-1]
    bundles = BUNDLE.findall(boot_prefix)
    if len(bundles) != 1:
        return {"bundle": INCOMPLETE}, boot_id
    if bundles[0] != bundle_id:
        return {"bundle": FAIL}, boot_id

    observed = boot_mode(uart10, start)
    expected = phase if phase in ("reboot", "poweroff") else "cold"
    rows = {"boot": INCOMPLETE if observed is None else PASS if observed == expected else FAIL}
    trace = [(i, s, f) for i, s, f in records if s in STAGES]
    stages = [STAGES.index(s) for _, s, _ in trace]
    if stages != sorted(set(stages)) or any(
        f.get("nonce") != nonce or f.get("kind") != kind for _, _, f in trace
    ):
        return {"sequence": INCOMPLETE}, boot_id
    # Do not accept result records captured after a second reset.
    end = trace[-1][0]
    if "RPi: BOOTSYS" in "\n".join(uart10.splitlines()[start:end + 1]):
        return {"sequence": INCOMPLETE}, boot_id
    data = {s: f for _, s, f in trace}
    status = data.get("STATUS", {})
    rows["request"] = numbers(data.get("REQUEST", {}), ("code",), lambda code: code == CODES[kind])
    if rows["request"] != PASS:
        return rows, boot_id
    rows["terminal"] = terminal_row(kind, status)
    if kind == "stkerr":
        if boot[0].get("stkerr") != "1":
            rows["terminal"] = INCOMPLETE
        return rows, boot_id

    core = int(kind == "p1-panic")
    pre, timer = data.get("PRE", {}), data.get("TIMER", {})
    rows["timer_live_before"] = overall(counters(pre, c, live=True) for c in range(2))
    rows["timer"] = overall((numbers(timer, ("elapsed_ms",), lambda elapsed: elapsed >= 1000),
                             counters(timer, core, live=False)))
    if kind == "p1-panic":
        rows["proc0_continues"] = overall((counters(timer, 0, live=True),
                                          check(status, ("peer",), lambda peer: peer == "Running")))
    rows["ping"] = ping_row(data.get("PING", {}))
    rows["usb"] = usb_row(data.get("USB", {}))
    before_rates = max(i for i, s, _ in trace if s in STAGES[:STAGES.index("SCMI")])
    before_scmi = next((i for i, s, _ in trace if s == "SCMI"), None)
    rates = [f for i, s, f in records if s == "RATE" and before_scmi is not None
             and before_rates < i < before_scmi
             and f.get("nonce") == nonce and f.get("kind") == kind]
    rows["scmi"] = scmi_row(data.get("SCMI", {}), status, rates)
    rows["pcie"] = overall(check(fields, ("pcie",), lambda pcie: pcie == "Up")
                           for fields in (status, data.get("END", {})))
    return rows, boot_id


def matrix(results, static):
    """All 20 cells need their own RP1 nonce and Linux boot ID."""
    nonces, ids = set(), set()
    cells = {}
    for kind, phase, nonce, boot_id, value in results:
        if kind not in KINDS:
            continue
        cell = kind, phase
        if cell in cells or nonce in nonces or (boot_id is not None and boot_id in ids):
            return {}, FAIL
        cells[cell] = value
        nonces.add(nonce)
        if boot_id is not None:
            ids.add(boot_id)
    kinds = {kind: overall(cells.get((kind, phase), INCOMPLETE) for phase in PHASES)
             for kind in KINDS}
    return kinds, overall((*kinds.values(), static))


def main(argv):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--elf", required=True, type=pathlib.Path)
    parser.add_argument("--manifest", required=True, help="manifest.txt of the deployed boot set")
    parser.add_argument("--stack-paths", help="prebuilt stack-paths executable")
    parser.add_argument("--run", action="append", nargs=5, default=[],
                        metavar=("KIND", "PHASE", "NONCE", "UART0", "UART10"))
    args = parser.parse_args(argv)
    for kind, phase, nonce, _, _ in args.run:
        if kind not in CODES or phase not in PHASES or len(nonce) != 8 or not HEX.fullmatch(nonce):
            parser.error("invalid kind, phase or eight-digit hex nonce in --run")

    digest = plat.sha256(args.elf) if args.elf.is_file() else None
    print(f"input elf={args.elf} sha256={digest or 'missing'}")
    manifest = plat.read("manifest", args.manifest)
    artifact, bundle_id = manifest_tuple(manifest, digest)
    print(f"FAULT_ARTIFACT={artifact} bundle={bundle_id or 'unknown'}")
    stack = INCOMPLETE
    if artifact == PASS:
        output, code = plat.run_stack_paths(args.elf.resolve(), args.stack_paths)
        stack, detail = stack_check(output, code)
        print(f"stack_static={stack} {detail}")
    else:
        print(f"stack_static={stack} missing matching ELF and manifest")
    results = []
    irq57_unavailable = False
    for kind, phase, nonce, uart0, uart10 in args.run:
        tag = f"kind={kind} phase={phase} nonce={nonce}"
        text0, text10 = plat.read("uart0", uart0), plat.read("uart10", uart10)
        irq57_unavailable |= any(s == "BOOT" and f.get("irq57") == "unavailable"
                                  for _, s, f in markers(text0))
        rows, boot_id = verify_run(kind, phase, nonce, text0, text10, bundle_id)
        value = overall((*rows.values(), artifact))
        for row, verdict in rows.items():
            print(f"FAULT_ROW {tag} row={row} result={verdict}")
        event = "FAULT_DIAG" if kind == "stkerr" else "FAULT_RUN"
        print(f"{event} {tag} result={value}")
        results.append((kind, phase, nonce, boot_id, value))
    if all(kind != "stkerr" for kind, *_ in args.run):
        print("FAULT_DIAG kind=stkerr result=INCOMPLETE reason=no-capture")
    reason = "platform-owned" if irq57_unavailable else "no-capture"
    print(f"FAULT_DIAG kind=irq57-panic result=INCOMPLETE reason={reason}")
    kinds, verdict = matrix(results, overall((artifact, stack)))
    for kind, status in kinds.items():
        print(f"FAULT_KIND kind={kind} result={status}")
    if not kinds:
        print("FAULT_MATRIX=FAIL reason=duplicate-cell-nonce-or-boot-id")
    print("PLAT_LOST=INCOMPLETE reason=no-stimulus")
    print(f"PLAT_VERDICT={verdict}")
    return {PASS: 0, FAIL: 1, INCOMPLETE: 3}[verdict]


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
