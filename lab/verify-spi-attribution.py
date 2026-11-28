#!/usr/bin/env python3
"""Cross-check the proc1 idle brackets and stamped proc0 ISR overlap."""

import argparse
from pathlib import Path
import re
import sys

TRIALS = 1000
RATES = (1000000, 100000)
MASK = (1 << 32) - 1


def require(ok, reason):
    if not ok:
        raise ValueError(reason)


def verify_phase(records, phase, spi_irq):
    inventory = {}
    data = {hz: bytearray() for hz in RATES}
    samples = {}
    spans = {}
    summary = {}
    platform = {}
    flushes = set()
    for line in records:
        tag, *fields = line.split()
        f = dict(field.split("=", 1) for field in fields)
        if tag == "ATTR_IRQ":
            irq, priority, enabled = (int(f[key]) for key in ("irq", "priority", "enabled"))
            inventory.setdefault(irq, []).append((priority, enabled))
        elif tag == "ATTR_DATA":
            hz = int(f["hz"])
            at = int(f["offset"])
            require(at == len(data[hz]), f"{phase} {hz} wrong sample offset {at}")
            data[hz].extend(bytes.fromhex(f["upper"]))
        elif tag in ("ATTR_SAMPLE", "ATTR_ISR"):
            key = int(f["hz"]), int(f["trial"])
            require(key[0] in RATES and 0 <= key[1] < TRIALS, "sample outside cohort")
            if tag == "ATTR_SAMPLE":
                require(key not in samples, "duplicate sample")
                samples[key] = f
            else:
                spans.setdefault(key, []).append(tuple(int(f[k]) for k in ("irq", "entered", "exited")))
        elif tag == "ATTR_RATE":
            hz = int(f["hz"])
            require(hz in RATES and hz not in summary, "duplicate or unexpected rate")
            summary[hz] = tuple(int(f[k]) for k in
                                ("trials", "preempted", "max_plain", "max_preempted", "failed"))
        elif tag == "ATTR_PLATFORM":
            irq = int(f["irq"])
            require(irq in (53, 57) and irq not in platform, "unexpected or duplicate platform IRQ")
            platform[irq] = int(f["entries"]), int(f["worst_us"])
        elif tag == "SPI_TX_TRIALS":
            hz = int(f["hz"])
            require(hz in RATES and hz not in flushes and int(f["tx_trials"]) == TRIALS
                    and int(f["tx_irq"]) == TRIALS and f["stray"] == f["errors"] == "0",
                    "missing device IRQ or failed flush trial")
            require(hz != 100000 or int(f["tail_trials"]) == TRIALS, "missing timer tail")
            flushes.add(hz)
        else:
            raise ValueError(f"unexpected record {line}")

    priorities = {53: 0, 57: 1, 26: 4, 27: 4, spi_irq: 4}
    require(set(inventory) == set(priorities), "unknown or missing NVIC line")
    for irq, priority in priorities.items():
        require(len(inventory[irq]) == 2 and all(p == priority and en in (0, 1)
                for p, en in inventory[irq]), "changed IRQ priority or invalid inventory")
        require(inventory[irq][-1][1] == 1 and (irq == 27 or inventory[irq][0][1] == 1),
                "required NVIC line disabled")
    require(set(platform) == {53, 57}, "missing platform counters")
    require(flushes == set(RATES) == set(summary), "missing rate cohort")
    require(set(spans) <= set(samples), "ISR stamps without sample")
    for hz in RATES:
        require(len(data[hz]) == TRIALS, f"{phase} {hz} missing flush samples")
        plain, preempted, fails = [], [], 0
        for i, compact in enumerate(data[hz]):
            key = hz, i
            if key not in samples:
                require(1 <= compact <= 3, f"{phase} {hz} trial {i} missing excess-latency stamps")
                plain.append(compact)
                continue
            f = samples[key]
            start = int(f["start"])
            elapsed = lambda field: (int(f[field]) - start) & MASK
            busy, idle, returned = map(elapsed, ("busy", "idle", "returned"))
            upper = (returned - busy + 1) & MASK
            require(min(upper, 255) == compact and upper == int(f["upper"]), "idle upper mismatch")
            credited = 0
            through = busy
            for irq, entered, exited in sorted(spans.get(key, []), key=lambda s: (s[1] - start) & MASK):
                begin, end = (entered - start) & MASK, (exited - start) & MASK
                require(irq in platform and 0 <= begin <= end <= returned
                        and end - begin <= platform[irq][1] and platform[irq][0],
                        "invalid ISR stamp or platform duration")
                credited += max(0, end - max(through, begin))
                through = max(through, end)
            require(credited == int(f["credit"]), "incorrect preemption credit")
            passed = (int(f["busy"]) != 0 and 0 <= idle - busy <= 6 and 0 <= busy <= returned
                      and f["overflow"] == "0" and upper <= 3 + credited)
            require(int(f["pass"]) == int(passed), "incorrect per-sample gate verdict")
            fails += not passed
            (preempted if credited else plain).append(upper)
        values = TRIALS, len(preempted), max(plain, default=0), max(preempted, default=0), fails
        require(summary[hz] == values, "summary disagrees with raw trials")
        print(f"SPI_LATENCY phase={phase} hz={hz} trials={TRIALS} preempted={values[1]} "
              f"max_plain_us={values[2]} max_preempted_us={values[3]} failed={fails} "
              f"result={'FAIL' if fails else 'PASS'}")
    for irq, (entries, worst) in platform.items():
        require(entries or worst == 0, "duration without ISR entry")
        print(f"SPI_PLATFORM through={phase} irq={irq} entries={entries} worst_us={worst}")
    return sum(s[-1] for s in summary.values()), platform


def verify(text, instance):
    require(not re.search(r"SPI_PARITY_FAIL|panicked|panic at|HardFault", text), "firmware failed")
    require(re.findall(r"^SPI_INSTANCE instance=(\d+)$", text, re.M) == [str(instance)],
            "unexpected SPI image")
    require(text.count("SPI_PARITY_FIRMWARE_END result=PASS") == 1, "firmware did not complete")
    records = iter(line for line in text.splitlines()
                   if line.startswith(("ATTR_", "SPI_TX_TRIALS ")))
    failures = 0
    prior = {53: (0, 0), 57: (0, 0)}
    for phase in ("early", "linux"):
        begin = f"ATTR_BEGIN instance={instance} phase={phase}"
        end = f"ATTR_END instance={instance} phase={phase}"
        require(next(records, None) == begin, f"missing {begin}")
        batch = []
        for line in records:
            if line == end:
                break
            batch.append(line)
        else:
            raise ValueError(f"missing {end}")
        failed, platform = verify_phase(batch, phase, 19 if instance == 0 else 22)
        failures += failed
        require(all(n >= prior[irq][0] and worst >= prior[irq][1]
                    for irq, (n, worst) in platform.items()), "platform counters regressed")
        prior = platform
        if phase == "early":
            require(next(records, None) == "ATTR_AWAIT_LINUX", "missing Linux start barrier")
    require(next(records, None) is None, "extra attribution records")
    return failures


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--instance", required=True, type=int, choices=(0, 3))
    parser.add_argument("uart", type=Path)
    args = parser.parse_args()
    try:
        return bool(verify(args.uart.read_text(), args.instance))
    except (OSError, ValueError, KeyError) as error:
        print(f"SPI_ATTR_VERIFY_FAIL {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
