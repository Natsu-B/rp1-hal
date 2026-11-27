#!/usr/bin/env python3
"""Check I2C1 parity through 400 kHz and report Fm+ observations."""

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
    # 9 * (1 address + 96 payload) clocks: 8730 / 2182.5 us nominal.
    # Allow about 5% faster SCL and keep 1270 / 817.5 us for FIFO and bus service.
    ("tx_refill", 100_000, 1_000, 8_300, 10_000),
    ("tx_refill", 400_000, 1_000, 2_075, 3_000),
    ("stop", 100_000, 100, 1_500, 2_000),
    ("nack", 100_000, 100, 75, 2_000),
    ("timeout", 100_000, 100, 2_000, 3_500),
]
REARM = {
    "nack": "I2C_REARM kind=nack count=100",
    "timeout": "I2C_REARM kind=timeout count=100 held_low=100 rejected=100 recovered=100",
}
PHASE = re.compile(r"I2C_PHASE kind=(\w+) hz=(\d+)" + "".join(
    rf" {name}=(\d+)" for name in FIELDS
))
TX_FIFO = re.compile(r"I2C_TX_FIFO depth=(\d+) frame=96")
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


def phase_values(line, kind, hz):
    match = PHASE.fullmatch(line)
    if match and match.groups()[:2] == (kind, str(hz)):
        return dict(zip(FIELDS, map(int, match.groups()[2:])))
    return None


def verify(transcript, before, supported, after):
    messages, failures = [], []
    records = [line.strip("\x00\r ") for line in transcript.splitlines()]
    markers = [line for line in records if line.startswith((
        "I2C_PEER_READY", "I2C_TX_FIFO", "I2C_PHASE", "I2C_REARM",
        "I2C_FMP_BEGIN", "I2C_FIXTURE_END",
    ))]
    if len(markers) != 14 or markers[0] != "I2C_PEER_READY addr=0x2d bytes=64":
        return messages, ["missing, duplicate or unordered fixture markers"]
    depth = TX_FIFO.fullmatch(markers[1])
    if not depth or not 1 <= int(depth[1]) < 96:
        failures.append("96-byte frame must exceed the reported TX FIFO depth")
    rows = iter(markers[2:])
    for kind, hz, count, lower, upper in PHASES:
        line = next(rows)
        rearm_ok = kind not in REARM or next(rows) == REARM[kind]
        v = phase_values(line, kind, hz)
        label = f"{kind} hz={hz}"
        if v is None:
            failures.append(f"{label} invalid marker")
            continue
        good = (
            v["count"] == v["ok"] == count
            and lower <= v["min_us"] <= v["max_us"] <= upper
            and v["irq_bad"] == v["stray"] == v["other"] == 0
            and rearm_ok
        )
        if kind == "combined":
            good &= (
                v["idle"] == count and count <= v["rx"] <= 64 * count
                and v["tx"] <= 17 * count and v["stop"] <= count and v["abort"] == 0
            )
        if kind == "tx_refill":
            good &= (
                v["idle"] == count and count <= v["tx"] <= 95 * count
                and v["rx"] == v["abort"] == 0 and v["stop"] <= count
            )
        if kind == "stop":
            good &= (v["idle"] == v["stop"] == count
                     and v["abort"] == v["rx"] == v["tx"] == 0)
        if kind in REARM:
            good &= v["rx"] == v["tx"] == v["stop"] == 0
        if kind == "nack":
            good &= v["idle"] == count and v["abort"] == count
        if kind == "timeout":
            good &= v["abort"] == 0
        if good:
            messages.append(f"I2C_GATE {label} result=PASS")
        else:
            failures.append(f"{label} out of bounds: {line}")
    boundary = next(rows)
    fmp_line = next(rows)
    fmp = phase_values(fmp_line, "combined", 1_000_000)
    if boundary != "I2C_FMP_BEGIN" or fmp is None or fmp["count"] != 1_000:
        failures.append("missing 1 MHz trial evidence")
    else:
        messages.append(f"I2C_FMP result=OPEN reason=requires_Fm+_peer {fmp_line}")
    try:
        a, b, c = map(peer_values, (before, supported, after))
        delta = tuple(new - old for old, new in zip(a, b))
        # Both frame lengths are checked byte-for-byte by the peer.
        if delta != (4_100, 2_100 * 16 + 2_000 * 96, 0, 2_302, 0):
            failures.append(f"ESP32 writes/bytes/bad/reads/preload delta {delta}")
        else:
            messages.append(
                "I2C_PEER_GATE writes=4100 bytes=225600 bad=0 reads=2302 preload=0 result=PASS"
            )
        fmp_delta = tuple(new - old for old, new in zip(b, c))
        if any(value < 0 for value in fmp_delta):
            failures.append(f"ESP32 counters reset during 1 MHz trials: {fmp_delta}")
        messages.append(f"I2C_FMP_PEER writes/bytes/bad/reads/preload delta {fmp_delta}")
    except ValueError as error:
        failures.append(str(error))
    end = next(rows)
    if end != "I2C_FIXTURE_END stray=0 other=0 result=Ok(())":
        failures.append(end)
    return messages, failures


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("uart", type=Path)
    parser.add_argument("peer_before", type=Path)
    parser.add_argument("peer_before_fmp", type=Path)
    parser.add_argument("peer_after", type=Path)
    args = parser.parse_args()
    messages, failures = verify(*(p.read_text(errors="replace") for p in (
        args.uart, args.peer_before, args.peer_before_fmp, args.peer_after
    )))
    for message in messages:
        print(message)
    for failure in failures:
        print(f"I2C_GATE_FAIL {failure}")
    if failures:
        return 1
    print("PARITY_I2C_CTRL_END max_hz=400000 result=PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
