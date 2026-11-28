#!/usr/bin/env python3
"""Verify Gate B summaries and recompute timer bounds from raw endpoints."""

from pathlib import Path
import re
import sys


# (pulse count, divider INT:FRAC, cycles, nominal high ns, nominal period ns,
#  bounds for 1,000 rising intervals in us)
CASES = {
    "1khz": (10000, "10000:0", 20, 500000, 1000000, 990000, 1010000),
    "10khz": (10000, "1000:0", 20, 50000, 100000, 99000, 101000),
    "int": (1001, "800:0", 20, 40000, 80000, 79200, 80800),
    "frac": (1001, "800:128", 20, 40025, 80050, 79250, 80850),
    "side": (1000, "1000:0", 20, 50000, 100000, 99000, 101000),
    "opt_side": (1000, "2000:0", 10, 50000, 100000, 99000, 101000),
    "pindirs": (1000, "1000:0", 20, 50000, 100000, 99000, 101000),
    "phase": (10001, "1000:0", 20, 50000, 100000, 99000, 101000),
}


def verify(path):
    runs = []
    for line_no, line in enumerate(path.read_text(errors="replace").splitlines(), 1):
        marker = re.search(r"\bPIO_B_(\w+) (.*)", line)
        if marker is None:
            continue
        kind, data = marker.groups()
        fields = dict(re.findall(r"(\w+)=([^ ]+)", data))
        if kind == "READY":
            assert int(fields["safety"], 0) == 15 and int(fields["clk_hz"]) == 200_000_000
        if kind == "BEGIN":
            runs.append({})
        if kind not in ("READY", "BOOT"):
            runs[-1].setdefault(kind, []).append((line_no, fields))
    assert len(runs) == 2, "expected idle and loaded runs"

    for i, rows in enumerate(runs, 1):
        def entries(kind):
            return [f for _, f in rows[kind]]

        def value(row, key):
            return int(row[key], 0)

        def elapsed(a, b):
            return (a - b) & 0xFFFFFFFF

        assert len(entries("BEGIN")) == len(entries("END")) == len(entries("STACK")) == 1
        end, = entries("END")
        assert end["result"] == "Ok(())" and value(end, "run") == i
        assert 0 < elapsed(value(end, "end"), value(end, "start")) < 30_000_000
        assert value(end, "safety") == 15 and value(end, "dma") == 0
        assert value(end, "stack_free") > 0 and value(entries("STACK")[0], "free") > 0
        assert len(entries("FIFO")) == 4
        for n, row in enumerate(entries("FIFO")):
            assert value(row, "sm") == n
            assert tuple(value(row, key) for key in ("split", "tx", "rx", "tx_16", "rx_16")) == (34, 33, 33, 33, 33)
            assert value(row, "dreq") == 0
            assert elapsed(value(row, "end"), value(row, "start")) < 30_000_000
        assert len(entries("WAVE")) == 9
        assert len(entries("WAVE_END")) == 7
        assert len(entries("WINDOW")) == 40
        for name, (pulses, divider, cycles, high, period, lower, upper) in CASES.items():
            for ch in range(2 if name == "phase" else 1):
                wave, = [r for r in entries("WAVE") if r["case"] == name and value(r, "channel") == ch]
                assert value(wave, "edges") == 2 * pulses, (name, ch, wave)
                assert wave["clkdiv"] == divider and value(wave, "cycles") == cycles
                assert value(wave, "quantum_ns") == 100
                high_tolerance = 5000 if name == "pindirs" else 1000
                for prefix, nominal, tolerance in [("high", high, high_tolerance), ("period", period, 1000)]:
                    lo = value(wave, f"{prefix}_min_ns")
                    hi = value(wave, f"{prefix}_max_ns")
                    assert nominal - tolerance <= lo <= hi <= nominal + tolerance, (name, prefix, lo, hi)
                windows = [r for r in entries("WINDOW") if r["case"] == name and value(r, "channel") == ch]
                assert len(windows) == (pulses - 1) // 1000, (name, ch, windows)
                for j, row in enumerate(windows):
                    assert value(row, "i") == j and value(row, "intervals") == 1000
                    lo = elapsed(value(row, "last_lo"), value(row, "first_hi")) - 1
                    hi = elapsed(value(row, "last_hi"), value(row, "first_lo")) + 1
                    assert lower <= lo <= hi <= upper, (name, ch, row)
                    assert (lo, hi) == (value(row, "sum_lo"), value(row, "sum_hi"))
            if name != "phase":
                finish, = [r for r in entries("WAVE_END") if r["case"] == name]
                assert value(finish, "pass") == 1 and value(finish, "elapsed") < 30_000_000
                assert value(finish, "safety") == 15 and value(finish, "dma") == 0

        phase, = entries("PHASE")
        assert value(phase, "pulses") == 10001 and value(phase, "uncertainty_ns") == 100
        assert value(phase, "pass") == 1 and value(phase, "elapsed") < 30_000_000
        assert value(phase, "safety") == 15 and value(phase, "dma") == 0
        for field in ("first", "last"):
            a, b = [r for r in entries("WAVE") if r["case"] == "phase"]
            difference = (elapsed(value(b, field), value(a, field)) + (1 << 31)) % (1 << 32) - (1 << 31)
            assert difference * 100 == value(phase, f"{field}_ns")
            assert abs(difference * 100) + 100 <= 2000
        assert len(entries("GROUP")) == 2
        for count, row in zip((3, 4), entries("GROUP")):
            assert value(row, "sms") == value(row, "completions") == count
            assert value(row, "flags") == (1 << count) - 1
            assert value(row, "remaining") == value(row, "dma") == 0
            assert value(row, "elapsed") <= 1_000

        print(f"PASS run={i} fifo=400 waves=7 phase=2 groups=2 "
              f"begin_line={rows['BEGIN'][0][0]} end_line={rows['END'][0][0]}")


if __name__ == "__main__":
    verify(Path(sys.argv[1]))
