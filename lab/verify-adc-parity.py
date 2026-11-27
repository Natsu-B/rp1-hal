#!/usr/bin/env python3
"""Verify the ADC one-shot measurements and exact marker order in the UART capture."""

import argparse
from pathlib import Path
import re
import sys


def require(condition, reason):
    if not condition:
        raise ValueError(reason)


def verify(text):
    require(not re.search(r"panicked|panic at|fault-continuation|HardFault", text),
            "ADC firmware recorded a failure")
    records = iter(line for line in text.splitlines() if line.startswith("ADC_"))

    def read(pattern):
        line = next(records, "")
        match = re.fullmatch(pattern, line)
        require(match is not None, f"expected {pattern!r}, got {line!r}")
        return tuple(map(int, match.groups()))

    def sample(pattern):
        raw, us = read(pattern.replace("@", r"raw=(\d+) us=(\d+)"))
        require(1 <= raw <= 4094, f"saturated sample {raw}")
        require(1 <= us <= 1000, f"conversion took {us} µs outside [1,1000]")
        return raw, us

    irq = r"irq=1 stray=0 alarm=0 dropped=0"
    none = r"irq=0 stray=0 alarm=0 dropped=0"
    read("ADC_PARITY_READY")
    samples = [sample(rf"ADC_IRQ_SAMPLE i={i} @ {irq}") for i in range(100)]
    legacy = [sample(rf"ADC_LEGACY_SAMPLE i={i} @") for i in range(100)]
    read(rf"ADC_LEGACY_END trials=100 would_block=100 {none}")

    stray, = read(r"ADC_STRAY us=(\d+) irq=0 stray=1 alarm=0 dropped=0 lowered=1")
    require(0 <= stray <= 100, f"forced IRQ retired in {stray} µs outside [0,100]")
    timeouts = []
    for i in range(10):
        us, = read(rf"ADC_TIMEOUT i={i} us=(\d+) irq=0 stray=0 alarm=1 dropped=1")
        require(999 <= us <= 1100, f"timeout took {us} µs outside [999,1100]")
        timeouts.append(us)
        sample(rf"ADC_REARM i={i} @ {irq}")

    read(rf"ADC_OVERRUN injected=1 cleared=1 busy=1 {none}")
    read(rf"ADC_ADMISSION expired=1 masked_irq=1 masked_poll=1 {none}")
    sample(rf"ADC_CANCEL @ {irq} retained=1")
    sample(rf"ADC_REINIT @ {irq}")
    _, poll = sample(rf"ADC_POLL @ {none}")
    read("ADC_PARITY_FIRMWARE_END result=PASS")
    require(next(records, None) is None, "unexpected ADC marker after completion")

    raws, times = zip(*samples)
    legacy_times = [us for _, us in legacy]
    return (f"PARITY_ADC_ONESHOT_END input=temperature trials=100 irq=100 unexpected_adc_irq=0 "
            f"raw_range={min(raws)}..{max(raws)} us_range={min(times)}..{max(times)} "
            f"timeout_us_range={min(timeouts)}..{max(timeouts)} rearm=10 "
            f"stray_retire_us={stray} poll_us={poll} overrun=1 cancel=1 result=PASS\n"
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
