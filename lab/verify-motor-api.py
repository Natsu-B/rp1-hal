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
    row(fr"MOTOR_API_START run={nonce} updates=100000 range=49999")
    clock_fields = r"([0-9a-f]+,\d+,[0-9a-f]+,\d+,\d+)"
    before, after = row(fr"MOTOR_API_CLOCK run={nonce} before={clock_fields} after={clock_fields}")
    ctrl, divider, pwm_ctrl, pwm_int, pwm_frac = before.split(",")
    shared = (before == after and int(ctrl, 16) & 3 == 3 and int(divider) == 50
              and int(pwm_ctrl, 16) & 0xBE0 == 0x840 and int(pwm_int) == 1 and int(pwm_frac) == 0)
    span, minimum, maximum, reported_shared = map(int, row(
        fr"MOTOR_API_CONVENTION run={nonce} cycles=200000 range=2500 span_us=(\d+)"
        r" min_period_us=(\d+) max_period_us=(\d+) shared=([01]) stopped=1"
    ))
    if reported_shared != shared or (shared and not (
            40 <= minimum <= maximum <= 60 and abs(span - 10_004_000) <= 100)):
        raise ValueError(f"invalid RANGE convention observation: {span} us")

    def waveform(phase, duty, start, end):
        previous_end = None
        for idx in range(100):
            period, high, time = map(int, row(
                fr"MOTOR_API_SAMPLE run={nonce} phase={phase} duty={duty} i={idx}"
                r" period_us=(\d+) high_us=(\d+) t_us=(-?\d+)"
            ))
            if not (990 <= period <= 1010 and 0 <= high <= period
                    and abs(100 * high - duty * period) <= period):
                raise ValueError(f"{phase}/{duty}% sample {idx} outside waveform bounds")

            if not (start <= time and time + period < end
                    and (previous_end is None or time == previous_end)):
                raise ValueError(f"{phase}/{duty}% sample {idx} outside time window")
            previous_end = time + period

    for plateau, duty in enumerate((25, 100, 50, 0, 75)):
        start, end = plateau * 20_000_000 + 5_000, (plateau + 1) * 20_000_000
        if duty not in (0, 100):
            waveform("duty", duty, start, end)
            continue
        samples, elapsed, gap, time = map(int, row(
            fr"MOTOR_API_LEVEL run={nonce} duty={duty} samples=(\d+) wrong=0 us=(\d+)"
            r" max_gap_us=(\d+) t_us=(\d+)"
        ))
        if not (samples >= 50_000 and gap <= 10 and 100_000 <= elapsed <= 100_000 + gap
                and start <= time and time + elapsed < end):
            raise ValueError(f"invalid {duty}% observation window")
    waveform("invalid", 50, -2**31, 0)

    cycles, low25, high75, other, minimum, maximum, start, end = map(int, row(
        fr"MOTOR_API_SWEEP run={nonce} cycles=(\d+) low25=(\d+) high75=(\d+) other=(\d+)"
        r" min_period_us=(\d+) max_period_us=(\d+) start_us=(\d+) end_us=(\d+)"
    ))
    if not (cycles >= 1_000 and cycles == low25 + high75 + other
            and low25 >= 400 and high75 >= 400 and other == 0
            and 990 <= minimum <= maximum <= 1010
            and 100_000_000 <= start < end < 101_100_099
            and minimum * cycles <= end - start <= maximum * cycles):
        raise ValueError("phase sweep outside waveform bounds")
    late, raw, upper, elapsed = map(int, row(
        fr"MOTOR_API_END run={nonce} updates=100000 sweep_updates=1100 missed=0 errors=0 arm_failed=0"
        r" max_late_us=(\d+) max_call_us=(\d+) max_call_upper_us=(\d+) invalid=100"
        r" invalid_native=100 register_changes=0 elapsed_us=(\d+) stopped=1 result=PASS"
    ))
    if not (late < 1_000 and upper == raw + 1 and upper <= 2
            and 100_000_000 <= elapsed <= 120_000_000):
        raise ValueError("update cadence or latency outside bounds")
    if next(rows, None) is not None:
        raise ValueError("duplicate or unexpected marker")
    return nonce, shared


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--uart0", type=Path, required=True)
    args = parser.parse_args()
    try:
        nonce, shared = verify(args.uart0.read_text(errors="replace"))
    except ValueError as exc:
        print(f"MOTOR_API_VERDICT=FAIL {exc}")
        return 1
    convention = "RANGE+1" if shared else "NOT_VERIFIED"
    print(f"MOTOR_API_VERDICT=PASS run={nonce} updates=100000 sweep_updates=1100")
    print(f"MOTOR_API_CYCLE_TICKS={convention}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
