#!/usr/bin/env python3
"""Verify M6 loopback and ISR evidence from a single raw UART0 capture."""
import argparse
from pathlib import Path
import re


def verify(capture):
    text = re.sub(r"\x1b\[[0-?]*[ -/]*[@-~]", "", capture).replace("\r", "")
    if re.search(r"\b(?:MOTOR_API_FAIL|PWM_FAIL|panic|panicked|hardfault|RP1_FAULT)\b", text, re.I):
        raise ValueError("firmware fault")
    rows = iter(line for line in text.splitlines() if line.startswith(("MOTOR_API_", "PWM_BOOT ")))

    def row(pattern):
        line = next(rows, "")
        match = re.fullmatch(pattern, line)
        if match is None:
            raise ValueError(f"expected {pattern}, got {line!r}")
        return match.groups()

    nonce, = row(r"PWM_BOOT nonce=([0-9a-f]{8}) PLAT_INIT ok=1(?: .*)?")
    if nonce in ("00000000", "ffffffff"):
        raise ValueError("invalid boot nonce")
    row(fr"MOTOR_API_START run={nonce} updates=100000 range=50000")
    for phase, duty in (("duty", 25), ("duty", 50), ("duty", 75), ("invalid", 50)):
        for idx in range(100):
            period, high = map(int, row(
                fr"MOTOR_API_SAMPLE run={nonce} phase={phase} duty={duty} i={idx}"
                r" period_us=(\d+) high_us=(\d+)"
            ))
            if not (990 <= period <= 1010 and 0 <= high <= period
                    and abs(100 * high - duty * period) <= period):
                raise ValueError(f"{phase}/{duty}% sample {idx} outside waveform bounds")

    for duty in (0, 100):
        samples, elapsed = map(int, row(
            fr"MOTOR_API_LEVEL run={nonce} duty={duty} samples=(\d+) wrong=0 us=(\d+)"
        ))
        if not (samples > 0 and 100_000 <= elapsed <= 100_010):
            raise ValueError(f"invalid {duty}% observation window")
    late, raw, upper, elapsed = map(int, row(
        fr"MOTOR_API_END run={nonce} updates=100000 missed=0 errors=0 arm_failed=0"
        r" max_late_us=(\d+) max_call_us=(\d+) max_call_upper_us=(\d+) invalid=100"
        r" invalid_native=100 register_changes=0 elapsed_us=(\d+) stopped=1 result=PASS"
    ))
    if not (late < 1_000 and upper == raw + 1 and upper <= 10
            and 100_000_000 <= elapsed <= 120_000_000):
        raise ValueError("update cadence or latency outside bounds")
    if next(rows, None) is not None:
        raise ValueError("duplicate or unexpected marker")
    return nonce


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--uart0", type=Path, required=True)
    args = parser.parse_args()
    try:
        nonce = verify(args.uart0.read_text(errors="replace"))
    except ValueError as exc:
        print(f"MOTOR_API_VERDICT=FAIL {exc}")
        return 1
    print(f"MOTOR_API_VERDICT=PASS run={nonce} updates=100000")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
