#!/usr/bin/env python3
"""Check both Gate C runs from one cold boot."""

import re
import sys
from pathlib import Path

TRIALS = 1_000
TIMEOUTS = 100
LINES = (1 << (50 - 32), 1 << (51 - 32))


def delta(a, b):
    return (a - b) & 0xFFFF_FFFF


def verify(data):
    records = {}
    for marker, body in re.findall(r"\bPIO_C_(\w+) ([^\n]*)", data):
        fields = dict(re.findall(r"(\w+)=([^\s,]+)", body))
        if marker not in ("FORCE", "FIFO", "END", "STACK"):
            continue
        run = int(fields["run"])
        key = (run, marker)
        if marker == "FORCE":
            key += (int(fields["line"]),)
        elif marker == "FIFO":
            key += (int(fields["sm"]), int(fields["line"]), fields["dir"])
        elif marker == "STACK":
            key += (int(fields["core"]),)
        assert key not in records, key
        records[key] = fields

    for run in (1, 2):
        def get(marker, *ids):
            return records.pop((run, marker, *ids))

        def integer(row, name):
            return int(row[name], 0)

        end = get("END")
        assert integer(end, "pass") == integer(end, "parked") == 1, end
        assert integer(end, "masks") == 0, end
        assert delta(integer(end, "end"), integer(end, "start")) < 30_000_000, end
        assert integer(end, "stack_free") > 0 and integer(get("STACK", 0), "free") > 0
        for line, bit in enumerate(LINES):
            row = get("FORCE", line)
            assert integer(row, "pre0") == integer(row, "pre1") == 0, row
            assert integer(row, "p0") in (0, bit) and integer(row, "p1") == bit, row
            assert integer(row, "quiet0") == integer(row, "quiet1") == 0, row
            assert integer(row, "entries") == 1 and integer(row, "cross") == 0, row
            assert integer(row, "status") == 0x100, row
            assert delta(integer(row, "isr"), integer(row, "start")) <= 1_000, row
            for sm in range(4):
                for direction in ("rx", "tx"):
                    row = get("FIFO", sm, line, direction)
                    assert row["result"] == "Ok(())", row
                    assert integer(row, "n") == integer(row, "entries") == TRIALS, row
                    assert 0 <= integer(row, "min") <= integer(row, "max") <= 50, row
                    lo, hi, isr = (integer(row, name) for name in ("lo", "hi", "isr"))
                    assert delta(hi, lo) < 50 and delta(isr, lo) + 1 == integer(row, "max"), row
                    assert integer(row, "timeouts") == TIMEOUTS, row
                    assert 1_000 <= integer(row, "tmin") <= integer(row, "tmax") <= 1_050, row
                    start, deadline, finish = (integer(row, name) for name in ("start", "deadline", "finish"))
                    assert delta(deadline, start) == 1_000, row
                    assert delta(finish, start) + 1 == integer(row, "tmax"), row
    assert not records, records
    print("PIO_C_VERIFY runs=2 force=4 transfers=32000 timeouts=3200")


if __name__ == "__main__":
    verify(Path(sys.argv[1]).read_text(errors="replace"))
