#!/usr/bin/env python3
"""Check a single normalized UART0 capture from timer-parity."""
import argparse
import re
from pathlib import Path

MAX_US = {1: 23, 10: 32, 100: 122, 1000: 1022, 10000: 10022}
PHASES = ("control", "isolated")
SAMPLES = 100
WAKE_REQUIRED_FROM_US = 100
BOOT = re.compile(r"PARITY_DELAY_BOOT nonce=([0-9a-f]{8}) PLAT_INIT ok=1 .+")
SAMPLE = re.compile(
    r"PARITY_DELAY_SAMPLE nonce=(?P<nonce>[0-9a-f]{8}) phase=(?P<phase>\w+) "
    r"request_us=(?P<request>\d+) n=100 under=0 over_limit=0 "
    r"min_us=(?P<minimum>\d+) max_us=(?P<maximum>\d+) "
    r"wait_wakes=(?P<wakes>\d+) wake_misses=0 isr_spans=(?P<spans>\d+)"
)
END = re.compile(
    r"PARITY_DELAY_END nonce=([0-9a-f]{8}) settings=5 samples=1000 under=0 over_limit=0 "
    r"wait_wakes=(\d+) wake_misses=0 isr_spans=(\d+) period_us=1000 ticks=1500 "
    r"during=(\d+) missed=0 max_late_us=(\d+) arm_failed=0 ok=1"
)


def verify(output):
    rows = iter(line for line in output.splitlines() if line.startswith("PARITY_DELAY_"))
    boot = BOOT.fullmatch(next(rows, ""))
    if boot is None:
        raise ValueError("missing boot identity")
    nonce = boot[1]
    total_wakes = total_spans = 0
    for phase in PHASES:
        if next(rows, "") != f"PARITY_DELAY_PHASE nonce={nonce} phase={phase}":
            raise ValueError(f"missing {phase} phase")
        for requested, limit in MAX_US.items():
            sample = SAMPLE.fullmatch(next(rows, ""))
            setting = f"{phase} {requested} us"
            if (
                sample is None
                or sample["nonce"] != nonce
                or sample["phase"] != phase
                or int(sample["request"]) != requested
            ):
                raise ValueError(f"invalid sample for {setting}")
            if not requested < int(sample["minimum"]) <= int(sample["maximum"]) <= limit:
                raise ValueError(f"tick range exceeds the bounds for {setting}")
            wakes = int(sample["wakes"])
            minimum_wakes = SAMPLES if requested >= WAKE_REQUIRED_FROM_US else 0
            if not minimum_wakes <= wakes <= SAMPLES:
                raise ValueError(f"wait IRQ count out of range for {setting}")
            spans = int(sample["spans"])
            if spans > (SAMPLES if phase == "control" else 0):
                raise ValueError(f"control deadline crossings out of range for {setting}")
            total_wakes += wakes
            total_spans += spans
    end = END.fullmatch(next(rows, ""))
    if end is None or end[1] != nonce or next(rows, None) is not None:
        raise ValueError("invalid end or nonzero under-wait, overshoot, wake miss, miss, or arm failure")
    wakes, spans, during, late = map(int, end.groups()[1:])
    if (wakes, spans) != (total_wakes, total_spans):
        raise ValueError("end totals disagree with the phase samples")
    if not 1000 <= during < 1500 or late > 50:
        raise ValueError("control IRQ count during waits or IRQ lateness out of range")
    return nonce, late, wakes, spans


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("uart0", type=Path)
    args = parser.parse_args()
    try:
        nonce, late, wakes, spans = verify(args.uart0.read_text())
    except (OSError, ValueError) as error:
        parser.exit(1, f"timer-parity: {error}\n")
    print(
        f"timer-parity nonce={nonce} samples=1000 under=0 over_limit=0 "
        f"wait_wakes={wakes} isr_spans={spans} max_late_us={late}"
    )


if __name__ == "__main__":
    main()
