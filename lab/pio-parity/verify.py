#!/usr/bin/env python3
"""Verify both Gate A cohorts in one cold boot's UART0 capture."""
import argparse
import re
from pathlib import Path


# Bits 31:30 per direction, packed into pairs for TX0–3 and then RX0–3.
DREQ_ENABLE = 0xAAAA
DREQ_RX_STATUS = 0x5500
DREQ_TX_STATUS = 0x0055


def require(ok, message):
    if not ok:
        raise SystemExit(message)


def fields(line):
    return {key: int(value, 0) for key, value in re.findall(r"(\w+)=(0x[0-9a-f]+|[0-9]+)", line)}


def verify(path, legacy_verdict=False):
    rows, ends, stacks, ready = [], [], [], []
    for n, raw in enumerate(path.read_bytes().split(b"\n"), 1):
        line = raw.decode(errors="replace").lstrip("\0").rstrip("\r")
        for marker, table in (
            ("PIO_TRIAL ", rows), ("PIO_END ", ends), ("PIO_STACK ", stacks), ("PIO_READY ", ready)
        ):
            if line.startswith(marker):
                table.append((n, fields(line), line))
    require(len(ready) == 1 and ready[0][2] ==
            "PIO_READY core=1 pads_ie_od_oeoff=[true, true, true, true] clk_hz=200000000 clkdiv=1.0",
            "proc1 readiness or pad readback failed")
    require(not legacy_verdict or all("reset_dreq" in row for _, row, _ in rows),
            f"legacy capture has {len(rows)} trials with aggregate DREQ status only; "
            "expected two 100-trial cohorts with per-direction status")
    require(len(rows) == 200 and len(ends) == len(stacks) == 2, "expected two 100-trial cohorts")
    for run, ((end_line, end, _), (stack_line, stack, _)) in enumerate(zip(ends, stacks), 1):
        cohort = rows[(run - 1) * 100:run * 100]
        require(ready[0][0] < cohort[0][0] < cohort[-1][0] < end_line < stack_line,
                f"run {run}: marker order")
        for i, (n, row, _) in enumerate(cohort):
            expect = dict(run=run, i=i, seen=1, pc=3, rx=21, push=1, pop=1, empty=1, irq=0)
            require(all(row.get(k) == v for k, v in expect.items()), f"trial rejected at line {n}")
            for key in ("dreq", "reset_dreq"):
                flags = row.get(key, -1)
                require(0 <= flags <= 0xFFFF, f"invalid {key} at line {n}")
                require(flags & DREQ_ENABLE == 0, f"DREQ enabled in {key} at line {n}")
                require(flags & DREQ_RX_STATUS == 0, f"RX status set in {key} at line {n}")
            upper = ((row["done"] - row["release"]) & 0xffffffff) + 1
            require(0 < upper <= 1000 and row["upper"] == upper, f"reset bound failed at line {n}")
        upper = [row["upper"] for _, row, _ in cohort]
        elapsed = (end["end"] - end["start"]) & 0xffffffff
        # The archived probe also required quiet TX status to be clear for its verdict.
        expected_pass = int(not legacy_verdict or all(row["dreq"] == 0 for _, row, _ in cohort))
        require(end["run"] == run and end["pass"] == expected_pass and end["trials"] == 100
                and end["restored"] == 1 and 0 < elapsed < 30_000_000
                and end["stack_free"] > 0 and end["reset_upper_min"] == min(upper)
                and end["reset_upper_max"] == max(upper) and stack["core"] == 0
                and stack["run"] == run and stack["free"] > 0, f"run rejected at line {end_line}")
        tx = sorted({row["dreq"] & DREQ_TX_STATUS for _, row, _ in cohort})
        reset_tx = sorted({row["reset_dreq"] & DREQ_TX_STATUS for _, row, _ in cohort})
        print(f"PIO_A_PASS run={run} rows=100 elapsed_us={elapsed} "
              f"reset_upper_min_us={min(upper)} reset_upper_max_us={max(upper)} "
              f"tx_status={','.join(map(hex, tx))} reset_tx_status={','.join(map(hex, reset_tx))} "
              f"stack_free_proc0={stack['free']} stack_free_proc1={end['stack_free']} "
              f"first_line={cohort[0][0]} last_line={cohort[-1][0]} end_line={end_line} "
              f"stack_line={stack_line}")


if __name__ == "__main__":
    cli = argparse.ArgumentParser(description=__doc__)
    cli.add_argument("uart0", type=Path)
    cli.add_argument("--legacy-verdict", action="store_true",
                     help="Interpret the archived per-direction probe's original pass flag.")
    args = cli.parse_args()
    verify(args.uart0, args.legacy_verdict)
