#!/usr/bin/env python3
"""Verify the ADC one-shot measurements and exact marker order in the UART capture."""

import argparse
from pathlib import Path
import re
import sys


# Calibration measured +19 µs; allow one 1 MHz timer tick.
IRQ_SLACK_US = 20


def require(condition, reason):
    if not condition:
        raise ValueError(reason)


def verify(text, irq_slack_us=IRQ_SLACK_US):
    require(not re.search(r"panicked|panic at|fault-continuation|HardFault", text),
            "ADC firmware recorded a failure")
    records = iter(line for line in text.splitlines() if line.startswith("ADC_"))

    def read(pattern):
        line = next(records, "")
        match = re.fullmatch(pattern, line)
        require(match is not None, f"expected {pattern!r}, got {line!r}")
        return tuple(map(int, match.groups()))

    def sample(pattern, upper):
        raw, us = read(pattern.replace("@", r"raw=(\d+) us=(\d+)"))
        require(1 <= raw <= 4094, f"saturated sample {raw}")
        require(3 <= us <= upper, f"conversion took {us} µs outside [3,{upper}]")
        return raw, us

    irq = r"irq=1 stray=0 alarm=0 dropped=0"
    none = r"irq=0 stray=0 alarm=0 dropped=0"
    read("ADC_PARITY_READY")
    samples = [sample(rf"ADC_IRQ_SAMPLE i={i} @ {irq}", 32) for i in range(100)]
    legacy = [sample(rf"ADC_LEGACY_SAMPLE i={i} @", 10) for i in range(100)]
    read(rf"ADC_LEGACY_END trials=100 would_block=100 {none}")

    stray, = read(r"ADC_STRAY us=(\d+) irq=0 stray=1 alarm=0 dropped=0 lowered=1")
    require(0 <= stray <= 5, f"forced IRQ retired in {stray} µs outside [0,5]")
    timeouts = []
    recovery_times = []
    for i in range(10):
        us, = read(rf"ADC_TIMEOUT i={i} us=(\d+) irq=0 stray=0 alarm=1 dropped=1")
        require(1000 <= us <= 1016, f"timeout took {us} µs outside [1000,1016]")
        timeouts.append(us)
        recovery_times.append(sample(rf"ADC_REARM i={i} @ {irq}", 32)[1])

    read(rf"ADC_OVERRUN injected=1 cleared=1 busy=1 {none}")
    read(rf"ADC_ADMISSION expired=1 masked_irq=1 masked_poll=1 {none}")
    recovery_times.append(sample(rf"ADC_CANCEL @ {irq} retained=1", 32)[1])
    recovery_times.append(sample(rf"ADC_REINIT @ {irq}", 32)[1])
    polls = [sample(rf"ADC_POLL i={i} @ {none}", 10)[1] for i in range(100)]
    read("ADC_PARITY_FIRMWARE_END result=PASS")
    require(next(records, None) is None, "unexpected ADC marker after completion")

    raws, times = zip(*samples)
    legacy_times = [us for _, us in legacy]
    require(max(times) - min(times) <= 3, f"IRQ sample spread {max(times) - min(times)} µs exceeds 3")
    irq_min = min(*times, *recovery_times)
    irq_max = max(*times, *recovery_times)
    poll_min, poll_max = min(polls), max(polls)
    require(irq_min >= poll_min - 1, f"IRQ min {irq_min} µs below polling min {poll_min} µs minus 1")
    delta = irq_max - poll_max
    require(delta <= irq_slack_us, f"IRQ max {irq_max} µs exceeds polling max {poll_max} µs + {irq_slack_us} µs")
    return (f"PARITY_ADC_ONESHOT_END input=temperature trials=100 irq=100 unexpected_adc_irq=0 "
            f"raw_range={min(raws)}..{max(raws)} us_range={min(times)}..{max(times)} "
            f"timeout_us_range={min(timeouts)}..{max(timeouts)} rearm=10 "
            f"stray_retire_us={stray} poll_us_range={poll_min}..{poll_max} recovery_us_range="
            f"{min(recovery_times)}..{max(recovery_times)} irq_poll_delta_us={delta} "
            f"irq_slack_us={irq_slack_us} overrun=1 cancel=1 result=PASS\n"
            f"PARITY_EH02_END driver=adc trials=100 would_block=100 "
            f"us_range={min(legacy_times)}..{max(legacy_times)} result=PASS")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("uart", type=Path)
    args = parser.parse_args()
    try:
        print(verify(args.uart.read_text(errors="replace")))
    except (OSError, ValueError) as error:
        print(f"PARITY_ADC_ONESHOT_FAIL {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
