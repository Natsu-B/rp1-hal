#!/usr/bin/env python3
"""Verify one cold boot of the dual-core ping test from raw UART captures."""
import argparse
from pathlib import Path
import re
import sys

ANSI = re.compile(r"\x1b\[[0-?]*[ -/]*[@-~]|\x1b[@-Z\\-_]")
TIMESTAMP = re.compile(r"^\d{4}-\d\d-\d\dT\d\d:\d\d:\d\d[+-]\d{4} ")
BOOT_ID = r"[0-9a-f]{8}(?:-[0-9a-f]{4}){3}-[0-9a-f]{12}"
FORBIDDEN = re.compile(r"\b(?:Kernel panic|Oops|SError|Internal error|RP1BOOT_LINUX_REFUSED)\b"
                       r"|\[rp1boot[^]]*\] FAIL:|PLAT_INIT ok=0", re.I)
PEER_LIMIT_US = 15_000


def lines(raw):
    return [TIMESTAMP.sub("", line).strip() for line in
            ANSI.sub("", raw).replace("\r", "").replace("\0", "").splitlines()]


def verify(uart0, uart10, mode, bundle, client_sha, previous_boot=None):
    fw, console = lines(uart0), lines(uart10)
    if sum("RPi: BOOTSYS" in line for line in console) != 1:
        raise ValueError("expected one bootloader banner")
    if sum(line.startswith("PLAT_INIT ok=1 ") for line in fw) != 1:
        raise ValueError("expected one successful firmware boot")
    if any(re.search(r"\b(?:panic|fault)\b", line, re.I) for line in fw):
        raise ValueError("unexpected firmware terminal message")
    if bad := next((line for line in console if FORBIDDEN.search(line)), None):
        raise ValueError(f"boot failure: {bad}")
    cursor = 0

    def required(pattern):
        nonlocal cursor
        matches = [(i, m) for i, line in enumerate(console) if (m := re.fullmatch(pattern, line))]
        if len(matches) != 1 or matches[0][0] < cursor:
            raise ValueError(f"missing, duplicate or out-of-order line: {pattern}")
        cursor = matches[0][0] + 1
        return matches[0][1]

    begin = required(rf"PING_CHECK_BEGIN mode={mode} boot_id=({BOOT_ID}) "
                     rf"bundle={re.escape(bundle)} client_sha256={re.escape(client_sha)}")
    boot_id = begin[1]
    if boot_id == previous_boot:
        raise ValueError("poll run reused the IRQ boot id")
    wanted = {"rp1:link", "rp1:header", "rp1:ping-dn", "rp1:ping-up"}
    if mode == "irq":
        wanted |= {"rp1:irq", "rp1:doorbell"}
    if not wanted.issubset({m[1] for line in console[cursor:]
                            if (m := re.fullmatch(r"PING_UIO (\S+) /dev/uio\d+", line))}):
        raise ValueError("missing required UIO nodes")

    def status(phase):
        ready = required(rf"PING_READY mode={mode} generation=(\d+) phase={phase}")
        return ready[1]

    generation = status("before")
    rtt = required(rf"PING_RTT mode={mode} n=5000 failed=0 p50_us=(\d+) p99_us=(\d+) max_us=(\d+)")
    if not int(rtt[1]) <= int(rtt[2]) <= int(rtt[3]):
        raise ValueError("RTT percentiles are out of order")
    if mode == "irq":
        counts = required(r"PING_IRQ before=(\d+) after=(\d+)")
        if int(counts[2]) <= int(counts[1]):
            raise ValueError("rp1:irq count did not increase")
    required(rf"PING_LATE mode={mode} n=5000 injected=50 timed_out=50 replied=50 wrong=0 failed=0")
    if status("after") != generation:
        raise ValueError("Ready generation changed")
    if mode == "irq":
        peer = required(r"PING_PEER_DOWN core=1 state=Panicked elapsed_us=(\d+)")
        if int(peer[1]) > PEER_LIMIT_US:
            raise ValueError("PeerDown exceeded 15000 us")
    required(rf"PING_BOOT_END_ID={boot_id}")
    required(rf"PING_CHECK_RESULT=PASS mode={mode}")
    required("PING_CHECK_END")
    required("PING_CHECK_EXIT=0")
    cores = [m.groups() for line in console
             if (m := re.match(r"RP1_CORE core=(\d+) state=(\S+)", line))]
    expected_states = [("0", "Running"), ("1", "Running")] * 2
    if mode == "irq":
        expected_states += [("0", "Running"), ("1", "Panicked")]
    if cores != expected_states:
        raise ValueError(f"unexpected core states: {cores}")
    states = [m.groups() for line in console if (m := re.fullmatch(
        r"RP1_STATUS epoch=\d+ shm=(\S+) shm_generation=(\d+) pcie=(\S+) scmi=(.*)", line))]
    if len(states) != len(cores) // 2 or any(
        state[:3] != ("Ready", generation, "Up")
        or not re.fullmatch(r"Serving\(requests=\d+\)", state[3]) for state in states
    ):
        raise ValueError(f"unexpected SHM, PCIe or SCMI state: {states}")
    expected_markers = {
        "PING_CHECK_BEGIN": 1, "PING_RTT": 1, "PING_LATE": 1, "PING_READY": 2,
        "PING_IRQ": int(mode == "irq"), "PING_PEER_DOWN": int(mode == "irq"),
        "PING_BOOT_END_ID": 1, "PING_CHECK_RESULT": 1, "PING_CHECK_END": 1, "PING_CHECK_EXIT": 1,
    }
    for name, count in expected_markers.items():
        if sum(re.match(rf"{name}(?:[ =]|$)", line) is not None for line in console) != count:
            raise ValueError(f"unexpected {name} count")
    return boot_id


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--uart0", required=True, type=Path)
    parser.add_argument("--uart10", required=True, type=Path)
    parser.add_argument("--mode", required=True, choices=("irq", "poll"))
    parser.add_argument("--bundle", required=True)
    parser.add_argument("--client-sha", required=True)
    parser.add_argument("--previous-boot-id")
    args = parser.parse_args()
    if args.mode == "poll" and not re.fullmatch(BOOT_ID, args.previous_boot_id or ""):
        parser.error("poll needs --previous-boot-id from the passing IRQ boot")
    try:
        boot_id = verify(args.uart0.read_text(errors="replace"), args.uart10.read_text(errors="replace"),
                         args.mode, args.bundle, args.client_sha, args.previous_boot_id)
    except (OSError, ValueError) as error:
        print(f"PING_VERDICT=FAIL {error}")
        return 1
    print(f"PING_VERDICT=PASS mode={args.mode} boot_id={boot_id}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
