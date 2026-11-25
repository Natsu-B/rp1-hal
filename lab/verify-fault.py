#!/usr/bin/env python3
"""Judge PR-E containment from one realos bundle and a pair of raw UART captures per boot."""

import argparse
import hashlib
import importlib.util
import json
import os
import pathlib
import re
import struct
import subprocess
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
STAGES = ("BEGIN", "PRE", "REQUEST", "SITE", "STATUS", "TIMER", "PING", "USB", "SCMI", "DMESG", "END")
IDENTITY_ROWS = ("identity", "bundle", "client", "boot", "sequence")
POST_ROWS = ("timer", "peer", "ping", "usb", "scmi", "dmesg", "pcie")
BOOTSYS = "RPi: BOOTSYS"
BOOT_LINE = re.compile(r"(?m)^[\x00 \t]*(?:[0-9]+\.[0-9]+[ \t]+)?RPi: BOOTSYS\b")
SHUTDOWN = re.compile(r"(?m)^\s*\[ *[0-9.]+\] reboot: (Restarting system|Power down)\s*$")
OPERATING_BOOT = re.compile(r"\[rp1boot\s|Linux version|Run /init|root@|~ #")
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
            yield i, stage, fields if len(fields) == len(pairs) else {}


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


def reset_modes(text):
    return re.findall(r"power-on-reset ([01])\b", text)


def poweroff_transition(previous, force_at, final_boot):
    before_force, after_force = previous[:force_at], previous[force_at:]
    pre_halt = BOOT_LINE.split(before_force)
    if len(pre_halt) < 2:
        return False
    prior, halt = pre_halt[-2:]
    if not (SHUTDOWN.findall(prior) == ["Power down"]
            and reset_modes(halt) == ["0"]
            and re.search(r"Halt: power_off: 1[^\n]*\n[\s\S]*?\bPower off\b", halt)
            and not OPERATING_BOOT.search(halt)):
        return False
    # A force-boot can restart before Ethernet comes up. The skipped preamble must
    # end at link wait, before handing off to the loader or Linux.
    preambles = BOOT_LINE.split(after_force)
    return (not preambles[0].strip("\x00 \t\r\n")
            and all(reset_modes(boot) == ["1"]
                    and "wait for link" in boot.strip("\x00 \t\r\n").splitlines()[-1]
                    and not OPERATING_BOOT.search(boot) for boot in preambles[1:])
            and reset_modes(final_boot) == ["1"])


def boot_mode(prefix, force_at):
    """Return the transition immediately preceding the operating boot."""
    markers = list(BOOT_LINE.finditer(prefix))
    if not markers:
        return None
    last = markers[-1]
    previous, boot = prefix[:last.start()], prefix[last.end():]
    modes = reset_modes(boot)
    if len(modes) != 1:
        return None
    prior = previous[markers[-2].end():] if len(markers) > 1 else previous
    if modes == ["0"]:
        return "reboot" if SHUTDOWN.findall(prior) == ["Restarting system"] else None
    if force_at is not None:
        return "poweroff" if poweroff_transition(previous, force_at, boot) else None
    # A poweroff without its capture-side observations cannot count as a cold boot.
    return None if SHUTDOWN.search(previous) else "cold"


def bundle_firmware(bundle):
    """Parse the layout in rp1_proto::bundle and verify the payload digests."""
    if len(bundle) < 32:
        raise ValueError("short bundle")
    magic, version, header, count, total, reserved = struct.unpack_from("<8sIIIIQ", bundle)
    if (magic != b"RP1BNDL\0" or version != 1 or count not in (2, 3)
            or header != 32 + count * 48 or not header < total == len(bundle) or reserved):
        raise ValueError("invalid bundle header")
    entries = {}
    end = header
    for at in range(32, header, 48):
        kind, flags, offset, length, digest = struct.unpack_from("<IIII32s", bundle, at)
        if (kind not in (1, 2, 3) or kind in entries or flags or offset % 4096
                or offset < end or not length or offset + length > total):
            raise ValueError("invalid bundle entry")
        end = offset + length
        if hashlib.sha256(memoryview(bundle)[offset:end]).digest() != digest:
            raise ValueError("invalid bundle payload digest")
        entries[kind] = digest.hex()
    if not {1, 2} <= entries.keys():
        raise ValueError("missing firmware or kernel")
    return hashlib.sha256(bundle[:header]).hexdigest(), entries[1]


def artifact_identity(manifest, bundle, elf_sha256):
    ids = re.findall(r"^bundle-id ([0-9a-f]{64})$", manifest or "", re.MULTILINE)
    digests = re.findall(r"^([0-9a-f]{64})  rp1\.elf$", manifest or "", re.MULTILINE)
    if elf_sha256 is None or bundle is None or len(ids) != 1 or len(digests) != 1:
        return INCOMPLETE, None
    try:
        bundle_id, firmware = bundle_firmware(bundle)
    except ValueError:
        return FAIL, None
    return (PASS if ids[0] == bundle_id and firmware == digests[0] == elf_sha256 else FAIL,
            bundle_id[:16])


def stack_check(output, exit_status):
    value, detail = plat.stack_row(output, exit_status)
    if value != PASS:
        return value, detail
    setup = [line for line in output.splitlines() if line.startswith("stack_fault_setup=")]
    if not setup:
        return INCOMPLETE, "missing stack_fault_setup=PASS"
    if setup != ["stack_fault_setup=PASS"]:
        return FAIL, "contradictory or unsuccessful stack_fault_setup"
    fields = re.findall(r"\b(s_plat|margin)=(\d+)\b", detail)
    if sorted(key for key, _ in fields) != ["margin", "s_plat"]:
        return INCOMPLETE, detail
    return numbers(dict(fields), ("s_plat", "margin"),
                   lambda bound, margin: bound <= margin <= 768), detail


def udf_address(elf):
    """The linked Thumb UDF instruction, as reported by the same objdump as stack-paths."""
    try:
        run = subprocess.run(
            [os.environ.get("RP1_OBJDUMP", "arm-none-eabi-objdump"), "-d", "--no-show-raw-insn",
             "--disassemble=fault_lab_udf", str(elf)],
            capture_output=True, text=True, check=True,
        )
    except (OSError, subprocess.CalledProcessError):
        return None
    sites = re.findall(r"(?m)^([0-9a-f]+) <fault_lab_udf>:\n([0-9a-f]+):\s+udf(?:\.n)?\s+#0\b",
                       run.stdout)
    return int(sites[0][0], 16) if len(sites) == 1 and sites[0][0] == sites[0][1] else None


def site_row(kind, site):
    ipsr = 42 if "-isr-" in kind else 0  # Exception number = TIMER_0 IRQ26 + 16.
    return numbers(site, ("code", "core", "ipsr"),
                   lambda code, core, observed: code == CODES[kind]
                   and core == int(kind == "p1-panic") and observed == ipsr)


def terminal_row(kind, status, udf_pc):
    core = int(kind == "p1-panic")
    identity = numbers(status, ("core", "ack"), lambda c, ack: c == core and ack == CODES[kind])
    if kind == "stkerr":
        if not u32(status.get("ack")):
            return INCOMPLETE
        state = check(status, ("state",), lambda s: s == "Fault")
        aux = numbers(status, ("reason", "pc_or_line", "aux"),
                      lambda reason, pc, value: reason == 3 and pc == 0 and bool(value & STACKING))
        # A stacking failure may lock up without publishing Fault.
        if status.get("state") == "Running":
            return INCOMPLETE
        return overall((identity, state, aux))
    if kind.endswith("udf"):
        state = check(status, ("state",), lambda s: s == "Fault")
        details = overall((
            numbers(status, ("reason", "aux"),
                    lambda reason, aux: reason == 6 and bool(aux & UNDEFINSTR)),
            INCOMPLETE if udf_pc is None else numbers(
                status, ("pc_or_line",), lambda observed: observed == udf_pc),
        ))
    else:
        state = check(status, ("state",), lambda s: s == "Panicked")
        details = numbers(status, ("reason", "aux"), lambda reason, aux: reason == aux == 0)
    return overall((identity, state, details))


def counters(fields, core, live):
    before, after = (u32(fields.get(f"p{core}_{key}")) for key in ("before", "after"))
    if any(n in (None, 0xffffffff) for n in (before, after)):
        return INCOMPLETE
    return PASS if (after > before if live else before == after) else FAIL


def continuity(pre, timer, core):
    previous, later = (u32(fields.get(f"p{core}_{key}"))
                       for fields, key in ((pre, "after"), (timer, "before")))
    if any(n in (None, 0xffffffff) for n in (previous, later)):
        return INCOMPLETE
    # A fresh boot and a short 20 Hz run make a decrease ambiguous or contradictory.
    return PASS if previous <= later else FAIL


def scmi_row(scmi, status, rates):
    states = (status.get("scmi"), scmi.get("scmi_before"), scmi.get("scmi_after"))
    if any(s in ("NotStarted", "Quarantined") or (s or "").startswith("Unknown(") for s in states):
        return FAIL
    if None in states or u32(scmi.get("after")) == 65535:
        return INCOMPLETE  # A saturated counter cannot establish an exact delta.
    observed = re.fullmatch(r"Serving\(requests=(\d{1,5})\)", states[0])
    if not observed:
        return FAIL
    delta = numbers(scmi, ("n", "replies", "before", "after"),
                    lambda n, replies, before, after: 0 < n <= 100
                    and n == replies == after - before and 0 <= before < after < 65535
                    and states[1] == f"Serving(requests={before})"
                    and states[2] == f"Serving(requests={after})"
                    and int(observed[1]) <= before)
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


def capture_identity(kind, phase, nonce, uart0, prefix, begin, bundle_id, client_sha256, force_at):
    boots = [fields for _, stage, fields in markers(uart0) if stage == "BOOT"]
    boot = boots[0] if len(boots) == 1 else {}
    boot_id = begin.get("boot_id", "")
    identity = (boot.get("nonce") == begin.get("nonce") == nonce and begin.get("kind") == kind
                and kind in boot.get("kinds", "").split(",") and BOOT_ID.fullmatch(boot_id))
    boot_segments = BOOT_LINE.split(prefix)
    banners = BUNDLE.findall(boot_segments[-1]) if len(boot_segments) > 1 else []
    observed = boot_mode(prefix, force_at)
    expected = phase if phase in ("reboot", "poweroff") else "cold"
    rows = {
        "identity": PASS if identity else INCOMPLETE,
        "bundle": INCOMPLETE if len(banners) != 1 or bundle_id is None
        else PASS if banners[0] == bundle_id else FAIL,
        "client": check(begin, ("client_sha256",), lambda digest: digest == client_sha256),
        "boot": INCOMPLETE if observed is None else PASS if observed == expected else FAIL,
    }
    return rows, boot_id if identity else None, boot


def sequence(kind, nonce, lines, records):
    trace = [(i, s, f) for i, s, f in records if s in STAGES]
    stages = [STAGES.index(s) for _, s, _ in trace]
    if not stages or stages[0] != 0 or stages != sorted(set(stages)) or any(
        f.get("nonce") != nonce or f.get("kind") != kind for _, _, f in trace
    ):
        return INCOMPLETE, {}, []
    start, end = trace[0][0], trace[-1][0]
    if any(BOOTSYS in line for line in lines[start:end + 1]):
        return INCOMPLETE, {}, []
    data = {s: f for _, s, f in trace}
    scmi = next((i for i, s, _ in trace if s == "SCMI"), None)
    samples = [(i, f) for i, s, f in records if s == "RATE"]
    if scmi is None:
        return PASS, data, []
    previous = max(i for i, _, _ in trace if i < scmi)
    if any(not previous < i < scmi or f.get("nonce") != nonce or f.get("kind") != kind
           for i, f in samples):
        return INCOMPLETE, {}, []
    return PASS, data, [f for _, f in samples]


def judge(kind, data, rates, boot, udf_pc):
    request, site, status, end = (data.get(s, {}) for s in ("REQUEST", "SITE", "STATUS", "END"))
    rows = {
        "request": numbers(request, ("code",), lambda code: code == CODES[kind]),
        "site": site_row(kind, site),
        "terminal": terminal_row(kind, status, udf_pc),
    }
    if kind == "stkerr":
        if boot.get("stkerr") != "1":
            rows["terminal"] = INCOMPLETE
        return rows

    core = int(kind == "p1-panic")
    pre, timer = data.get("PRE", {}), data.get("TIMER", {})
    rows["timer_live_before"] = overall(counters(pre, c, live=True) for c in range(2))
    rows["timer"] = overall((
        numbers(timer, ("elapsed_ms",), lambda elapsed: elapsed >= 1000),
        counters(timer, core, live=False),
        *(continuity(pre, timer, c) for c in range(2)),
    ))
    peer = [check(fields, ("peer",), lambda state: state == "Running") for fields in (status, end)]
    if core:
        peer.append(counters(timer, 0, live=True))
    rows["peer"] = overall(peer)
    rows["ping"] = ping_row(data.get("PING", {}))
    rows["usb"] = usb_row(data.get("USB", {}))
    rows["scmi"] = scmi_row(data.get("SCMI", {}), status, rates)
    rows["dmesg"] = numbers(data.get("DMESG", {}), ("scmi_timeouts", "oops", "link_errors"),
                            lambda *counts: all(n == 0 for n in counts))
    rows["pcie"] = overall(check(fields, ("pcie",), lambda pcie: pcie == "Up")
                           for fields in (status, end))
    return rows


def verify_run(kind, phase, nonce, uart0, uart10, bundle_id, client_sha256,
               udf_pc=None, artifact=PASS, force_at=None):
    """Judge a single nonce-bound invocation, retaining failed measurements behind a gate."""
    lines = (uart10 or "").splitlines(keepends=True)
    records = list(markers(uart10))
    begins = [(i, f) for i, s, f in records if s == "BEGIN"]
    start, begin = begins[0] if len(begins) == 1 else (0, {})
    prefix = "".join(lines[:start])
    rows, boot_id, boot = capture_identity(
        kind, phase, nonce, uart0, prefix, begin, bundle_id, client_sha256, force_at)
    rows["sequence"], data, rates = sequence(kind, nonce, lines, records)
    measured = judge(kind, data, rates, boot, udf_pc)
    usable = artifact == PASS and all(rows[name] == PASS for name in IDENTITY_ROWS)
    for row, verdict in measured.items():
        allowed = usable
        if row == "terminal":
            allowed &= rows["request"] == rows["site"] == PASS
        if row in POST_ROWS:
            allowed &= rows["terminal"] == PASS
        rows[row] = verdict if allowed else INCOMPLETE
    diagnostics = {row: FAIL for row, verdict in measured.items()
                   if verdict == FAIL and rows[row] != FAIL}
    return rows, boot_id, diagnostics


def load_bytes(name, path):
    if not path.is_file():
        print(f"input {name}={path} missing")
        return None
    data = path.read_bytes()
    print(f"input {name}={path} sha256={hashlib.sha256(data).hexdigest()}")
    return data


def force_boundary(raw, boot, events, stamps):
    """Locate the force command in the same capture after poweroff and carrier loss."""
    if (not raw or boot.get("kind") != "force" or boot.get("complete") is not True
            or boot.get("pd_ok") is not True):
        return None
    start, end = boot.get("raw_start"), boot.get("raw_end")
    if type(start) is not int or type(end) is not int or not 0 < start < end <= len(raw):
        return None
    command, carrier, forced = [], [], []
    for event in events:
        if event.get("kind") == "console_sendraw" and event.get("text") == "systemctl poweroff\n":
            command.append(event)
        elif event.get("kind") == "carrier":
            carrier.append(event)
        elif event.get("command") == "force-boot":
            forced.append(event)
    if len(command) != 1 or [e.get("kind") for e in forced] != [
        "esp_command_start", "esp_command_end",
    ] or forced[1].get("rc") != 0:
        return None
    try:
        shutdown, force, done = (float(e["mono"]) for e in (*command, *forced))
        states = sorted((float(e["mono"]), e["value"]) for e in carrier
                        if shutdown < float(e["mono"]) < force)
    except (KeyError, TypeError, ValueError):
        return None
    if not shutdown < force < done or not states or states[-1][1] != "0":
        return None
    power_down = raw.rfind(b"reboot: Power down", 0, start)
    halted = raw.rfind(b"Power off", 0, start)
    next_boot = raw.find(BOOTSYS.encode(), start, end)
    if not 0 <= power_down < halted < start <= next_boot < end:
        return None
    times = [capture_time(stamps, offset) for offset in (power_down, halted, next_boot)]
    if None in times or not shutdown <= times[0] <= times[1] < force <= times[2]:
        return None
    return len(raw[:start].decode(errors="replace"))


def capture_time(stamps, offset):
    hits = [record["monotonic_ns"] / 1e9 for record in stamps if record.get("port") == 1
            and record["offset"] <= offset < record["offset"] + record["bytes"]]
    return hits[0] if len(hits) == 1 else None


def capture_force_boundary(path, raw):
    event_data = load_bytes("events", path.with_name("events.jsonl"))
    boot_data = load_bytes("force_boot", path.parent.parent / "boot.json")
    stamp_data = load_bytes("timestamps", path.with_name("timestamps.jsonl"))
    if None in (event_data, boot_data, stamp_data):
        return None
    try:
        return force_boundary(raw, json.loads(boot_data),
                              [json.loads(line) for line in event_data.splitlines()],
                              [json.loads(line) for line in stamp_data.splitlines()])
    except (ValueError, AttributeError, TypeError, KeyError):
        return None


def matrix(results, static):
    """All 20 cells need their own RP1 nonce, Linux boot ID and capture pair."""
    nonces, ids, captures = set(), set(), set()
    cells = {}
    duplicate_nonce = False
    for kind, phase, nonce, boot_id, value, capture in results:
        if kind not in KINDS:
            continue
        cell = kind, phase
        if cell in cells or capture in captures or (boot_id is not None and boot_id in ids):
            return {}, FAIL, "duplicate-cell-capture-or-boot-id"
        cells[cell] = value
        captures.add(capture)
        duplicate_nonce |= nonce in nonces
        nonces.add(nonce)
        if boot_id is not None:
            ids.add(boot_id)
    if duplicate_nonce:
        return {}, overall((static, INCOMPLETE, *cells.values())), "duplicate-nonce"
    kinds = {kind: overall(cells.get((kind, phase), INCOMPLETE) for phase in PHASES)
             for kind in KINDS}
    return kinds, overall((*kinds.values(), static)), None


def main(argv):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--elf", required=True, type=pathlib.Path)
    parser.add_argument("--manifest", required=True, help="manifest.txt of the deployed boot set")
    parser.add_argument("--bundle", type=pathlib.Path, required=True, help="deployed rp1-bundle.img")
    parser.add_argument("--client-sha256", required=True, help="SHA-256 of deployed fault-linux")
    parser.add_argument("--stack-paths", help="prebuilt stack-paths executable")
    parser.add_argument("--run", action="append", nargs=5, default=[],
                        metavar=("KIND", "PHASE", "NONCE", "UART0", "UART10"))
    args = parser.parse_args(argv)
    if not re.fullmatch(r"[0-9a-f]{64}", args.client_sha256):
        parser.error("--client-sha256 requires 64 lowercase hex digits")
    for kind, phase, nonce, _, _ in args.run:
        if kind not in CODES or phase not in PHASES or len(nonce) != 8 or not HEX.fullmatch(nonce):
            parser.error("invalid kind, phase or eight-digit hex nonce in --run")

    digest = plat.sha256(args.elf) if args.elf.is_file() else None
    print(f"input elf={args.elf} sha256={digest or 'missing'}")
    manifest = plat.read("manifest", args.manifest)
    artifact, bundle_id = artifact_identity(manifest, load_bytes("bundle", args.bundle), digest)
    udf_pc = udf_address(args.elf) if artifact == PASS else None
    print(f"FAULT_ARTIFACT={artifact} bundle={bundle_id or 'unknown'}")
    stack = INCOMPLETE
    if artifact == PASS:
        output, code = plat.run_stack_paths(args.elf.resolve(), args.stack_paths)
        stack, detail = stack_check(output, code)
        print(f"stack_static={stack} {detail}")
    else:
        print(f"stack_static={stack} missing matching ELF, bundle and manifest")
    results = []
    irq57_unavailable = False
    for kind, phase, nonce, uart0, uart10 in args.run:
        tag = f"kind={kind} phase={phase} nonce={nonce}"
        text0 = plat.read("uart0", uart0)
        path = pathlib.Path(uart10)
        raw = load_bytes("uart10", path)
        text10 = raw.decode(errors="replace") if raw else None
        force_at = capture_force_boundary(path, raw) if phase == "poweroff" else None
        irq57_unavailable |= any(s == "BOOT" and f.get("irq57") == "unavailable"
                                  for _, s, f in markers(text0))
        rows, boot_id, diagnostics = verify_run(
            kind, phase, nonce, text0, text10, bundle_id, args.client_sha256,
            udf_pc=udf_pc, artifact=artifact, force_at=force_at)
        value = overall((*rows.values(), artifact))
        for row, verdict in rows.items():
            detail = " measured=FAIL" if row in diagnostics else ""
            print(f"FAULT_ROW {tag} row={row} result={verdict}{detail}")
        event = "FAULT_DIAG" if kind == "stkerr" else "FAULT_RUN"
        label = " diagnostic=hardfault-psp-stacking-error" if kind == "stkerr" else ""
        print(f"{event} {tag} result={value}{label}")
        capture = (pathlib.Path(uart0).resolve(), path.resolve())
        results.append((kind, phase, nonce, boot_id, value, capture))
    if all(kind != "stkerr" for kind, *_ in args.run):
        print("FAULT_DIAG kind=stkerr result=INCOMPLETE reason=no-capture")
    reason = "platform-owned" if irq57_unavailable else "no-capture"
    print(f"FAULT_DIAG kind=irq57-panic result=INCOMPLETE reason={reason}")
    kinds, verdict, matrix_reason = matrix(results, overall((artifact, stack)))
    for kind, status in kinds.items():
        print(f"FAULT_KIND kind={kind} result={status}")
    if matrix_reason:
        print(f"FAULT_MATRIX={verdict} reason={matrix_reason}")
    print("PLAT_LOST=INCOMPLETE reason=no-stimulus")
    print(f"PLAT_VERDICT={verdict}")
    return {PASS: 0, FAIL: 1, INCOMPLETE: 3}[verdict]


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
