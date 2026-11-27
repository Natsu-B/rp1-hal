#!/usr/bin/env python3
"""Check the I2C1 fixture transcript and the powered ESP32's counter deltas."""

import argparse
import re
from pathlib import Path

FIELDS = (
    "count", "ok", "idle", "min_us", "max_us", "irq_bad",
    "rx", "tx", "stop", "abort", "stray", "other",
)
PHASES = [
    ("combined", 100_000, 1_000, 7_000, 10_000),
    ("combined", 400_000, 1_000, 1_750, 10_000),
    ("stop", 100_000, 100, 1_500, 2_000),
    ("nack", 100_000, 100, 75, 2_000),
    ("timeout", 100_000, 100, 2_000, 3_500),
    ("combined", 1_000_000, 1_000, 700, 10_000),
]
PHASE = re.compile(r"I2C_PHASE kind=(\w+) hz=(\d+)" + "".join(
    rf" {name}=(\d+)" for name in FIELDS
))
PEER = re.compile(
    r"i2c_addr=0x2d rx=(\d+) bytes=(\d+) bad_frames=(\d+) "
    r"reads=(\d+) preload_errors=(\d+)"
)


def peer_values(transcript):
    matches = [PEER.fullmatch(line) for line in transcript.splitlines()]
    records = [tuple(map(int, match.groups())) for match in matches if match]
    if len(records) != 1 or transcript.splitlines().count("OK") != 1:
        raise ValueError("expected one ESP32 counter record and OK")
    return records[0]


def verify(transcript, before, after):
    messages, failures = [], []
    records = [line.strip("\x00\r ") for line in transcript.splitlines()]
    markers = [line for line in records if line.startswith(
        ("I2C_PEER_READY", "I2C_PHASE", "I2C_REARM", "I2C_FIXTURE_END")
    )]
    if len(markers) != 10 or markers[0] != "I2C_PEER_READY addr=0x2d bytes=64":
        return messages, ["missing, duplicate or unordered fixture markers"]
    for index, (kind, hz, count, lower, upper) in enumerate(PHASES):
        line = markers[[1, 2, 3, 4, 6, 8][index]]
        match = PHASE.fullmatch(line)
        label = f"{kind} hz={hz}"
        if not match or match.groups()[:2] != (kind, str(hz)):
            failures.append(f"{label} invalid marker")
            continue
        v = dict(zip(FIELDS, map(int, match.groups()[2:])))
        good = (
            v["count"] == v["ok"] == count
            and lower <= v["min_us"] <= v["max_us"] <= upper
            and v["irq_bad"] == v["stray"] == v["other"] == 0
        )
        if kind == "combined":
            good &= (
                v["idle"] == count and count <= v["rx"] <= 64 * count
                and v["tx"] <= 17 * count and v["stop"] <= count and v["abort"] == 0
            )
        if kind == "stop":
            good &= (v["idle"] == v["stop"] == count
                     and v["abort"] == v["rx"] == v["tx"] == 0)
        if kind in {"nack", "timeout"}:
            good &= v["rx"] == v["tx"] == v["stop"] == 0
        if kind == "nack":
            good &= v["idle"] == count and v["abort"] == count
            good &= markers[5] == "I2C_REARM kind=nack count=100"
        if kind == "timeout":
            good &= v["abort"] == 0
            good &= markers[7] == (
                "I2C_REARM kind=timeout count=100 held_low=100 rejected=100 recovered=100"
            )
        if good:
            messages.append(f"I2C_GATE {label} result=PASS")
        else:
            failures.append(f"{label} out of bounds: {line}")
    try:
        a, b = peer_values(before), peer_values(after)
        delta = tuple(new - old for old, new in zip(a, b))
        if delta != (3_100, 49_600, 0, 3_302, 0):
            failures.append(f"ESP32 writes/bytes/bad/reads/preload delta {delta}")
        else:
            messages.append("I2C_PEER_GATE writes=3100 bytes=49600 bad=0 reads=3302 preload=0 result=PASS")
    except ValueError as error:
        failures.append(str(error))
    if markers[9] != "I2C_FIXTURE_END stray=0 other=0 result=Ok(())":
        failures.append(markers[9])
    return messages, failures


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("uart", type=Path)
    parser.add_argument("peer_before", type=Path)
    parser.add_argument("peer_after", type=Path)
    args = parser.parse_args()
    messages, failures = verify(*(p.read_text(errors="replace") for p in (
        args.uart, args.peer_before, args.peer_after
    )))
    for message in messages:
        print(message)
    for failure in failures:
        print(f"I2C_GATE_FAIL {failure}")
    if failures:
        return 1
    print("PARITY_I2C_CTRL_END result=PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
