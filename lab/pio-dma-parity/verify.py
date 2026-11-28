#!/usr/bin/env python3
"""Verify both Gate D phases and the lost-stop-proof test on one cold boot."""

import re
import sys
from pathlib import Path


def without_printk(output):
    # UART printk can interrupt a marker mid-field.
    parts = []
    for line in output.replace("\r", "").splitlines(keepends=True):
        stamp = re.search(r"\[ *\d+\.\d+\][ \t]", line)
        if stamp:
            prefix = line[:stamp.start()]
            if prefix.strip():
                parts.append(prefix)
        else:
            parts.append(line)
    return "".join(parts)


def elapsed(start, stop):
    return (stop - start) & 0xFFFF_FFFF


def verify(output):
    records = {}
    for marker, body in re.findall(r"\bPIO_D_(\w+) ([^\n]*)", without_printk(output)):
        if marker == "PLAT":
            continue
        assert marker not in ("TRIAL_FAIL", "DMA_ERROR"), body
        row = dict(re.findall(r"(\w+)=([^\s]+)", body))
        key = (marker,)
        if marker in ("FIFO", "ABORT", "END"):
            key += (int(row["run"]),)
        if marker in ("FIFO", "ABORT"):
            key += (int(row["sm"]),)
        if marker == "FIFO":
            key += (int(row["rx"]), int(row["words"]))
        assert key not in records, key
        records[key] = row

    def take(key, **expected):
        row = records.pop(key)
        for field, value in expected.items():
            assert row[field] == str(value), (key, field, row[field], value)
        return row

    take(("INIT",), clocks=1, pio=1, dma=1, echo=1)
    take(("READY",), core=1, clk_hz=200000000, divider="200:0", quantization_us=1, irq40="masked")
    min_us = 10_000
    max_us = stop_max = 0
    for run in (1, 2):
        end = take(("END", run), **{"pass": 1}, error="None")
        assert elapsed(int(end["start"]), int(end["end"])) < 30_000_000, end
        assert int(end["stack_free"]) > 0, end
        partials = 0
        for sm in range(4):
            abort = take(("ABORT", run, sm), count=25, words=257, reuse=25, guard=0, fdebug=0)
            assert 0 <= int(abort["min"]) <= int(abort["max"]) <= 257, abort
            assert 0 <= int(abort["partials"]) <= 25, abort
            partials += int(abort["partials"])
            stop = int(abort["max_us"])
            assert 0 <= stop <= 1_000, abort
            stop_max = max(stop_max, stop)
            for rx in (0, 1):
                for words in (1, 7, 8, 9, 257):
                    fifo = take(("FIFO", run, sm, rx, words), count=100, total_words=100 * words,
                                progress=0, content=0, guard=0, fdebug=0)
                    for key, prefix in (("blocked", "b"), ("masked", "m")):
                        low, high = int(fifo[f"{key}_min"]), int(fifo[f"{key}_max"])
                        start, stop = int(fifo[f"{prefix}_start"]), int(fifo[f"{prefix}_stop"])
                        assert 1_000 <= low <= high and elapsed(start, stop) - 1 == low, fifo
                    lower, upper = int(fifo["min"]), int(fifo["max"])
                    assert 0 <= lower <= upper <= 10_000, fifo
                    assert elapsed(int(fifo["start"]), int(fifo["stop"])) + 1 == upper, fifo
                    min_us = min(min_us, lower)
                    max_us = max(max_us, upper)
        assert partials > 0, ("no transfers stopped after a proper prefix", run)
    lost = take(("QUARANTINE",), sm=3, active=1, disabled=1, masked=1, terminal=0,
                decision="Err(Recovering)", retry="Err(Recovering)", held=1, claimed=1, fifo=1)
    assert elapsed(int(lost["start"]), int(lost["stop"])) + 1 <= 1_000, lost
    take(("QUARANTINE_END",), **{"pass": 1}, error="None")
    assert not records, records
    print(f"PIO_D_VERIFY runs=2 transfers=8000 blocked=8000 masked=8000 aborts=200 "
          f"reuse=200 quarantine=1 completion_us={min_us}..{max_us} stop_max_us={stop_max}")


if __name__ == "__main__":
    verify(Path(sys.argv[1]).read_text(errors="replace"))
