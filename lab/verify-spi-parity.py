#!/usr/bin/env python3
"""Check SPI parity records and, when supplied, an independent wire capture."""

import argparse
import csv
from dataclasses import dataclass, field
from pathlib import Path
import re
import sys


def require(condition, reason):
    if not condition:
        raise ValueError(reason)


TRIALS = 100
TX_RATES = (1_000_000, 100_000)
FAULT_AT_US = 250
SLACK_US = 100
CS_DELAY_US = 4
MIN_SAMPLE_HZ = 100_000_000


def verify_uart(text):
    require(not re.search(r"SPI_PARITY_FAIL|panicked|panic at|EXAMPLE_FAIL|fault-continuation|HardFault", text),
            "firmware recorded a failure")
    records = iter(line for line in text.splitlines() if line.startswith("SPI_"))

    def read(pattern):
        line = next(records, "")
        match = re.fullmatch(pattern, line)
        require(match is not None, f"expected {pattern!r}, got {line!r}")
        return tuple(map(int, match.groups()))

    first = next(records, "")
    if re.fullmatch(r"SPI_INSTANCE instance=[013]", first):
        read(r"SPI_PARITY_READY")
    else:
        require(first == "SPI_PARITY_READY", f"unexpected first SPI record {first!r}")
    counts = []
    depths = set()
    for hz in (1_000_000, 10_000_000):
        depth, rx = read(
            rf"SPI_IRQ_RATE hz={hz} frames=100000 trials=100 depth=(\d+) rx_irq=(\d+) stray=0 errors=0")
        require(2 <= depth <= 256, "FIFO depth outside supported range")
        half_depth = (depth + 1) // 2
        upper = TRIALS * ((1000 + half_depth - 1) // half_depth + 2)
        require(TRIALS <= rx <= upper, "RX IRQ count outside FIFO-derived bound")
        counts.append(rx)
        depths.add(depth)
    require(len(depths) == 1, "different FIFO depths between bulk rates")
    depth, = depths

    flush_results = {}
    for hz in TX_RATES:
        burst = depth * (8_000_000 // hz)
        earliest = burst - (1_000_000 // hz) - 1
        samples = []
        for i in range(TRIALS):
            elapsed, launch, late, alarms = read(
                rf"SPI_TX_SAMPLE hz={hz} trial={i} flush_us=(\d+) launch_us=(\d+) "
                r"late_us=(\d+) tx_irq=1 tail_alarms=(\d+) idle=1")
            require(earliest <= launch and elapsed <= launch and elapsed <= burst + SLACK_US
                    and late == launch - earliest + 1 and late <= SLACK_US,
                    "TX flush outside launch-to-shifter timing bound")
            require((1 if hz == 100_000 else 0) <= alarms <= 1 + SLACK_US,
                    "TX flush tail alarm count outside bound")
            samples.append((elapsed, late, alarms))
        tails, flush_max, late_max = read(
            rf"SPI_TX_TRIALS hz={hz} tx_trials=100 depth={depth} tx_irq=100 tail_trials=(\d+) "
            r"flush_max_us=(\d+) flush_late_max_us=(\d+) stray=0 errors=0")
        require((tails, flush_max, late_max) == (sum(alarms > 0 for _, _, alarms in samples),
                max(t for t, _, _ in samples), max(t for _, t, _ in samples)),
                "TX flush aggregate disagrees with trial samples")
        flush_results[hz] = flush_max, late_max, tails

    timeout, = read(r"SPI_IRQ_TRIALS timeout=100 timeout_max_us=(\d+) rearm=100 tx_irq=0 stray=0 errors=0")
    require(500 <= timeout <= 600, "blocked transfer deadline outside bound")

    faults = []
    for i in range(TRIALS):
        inject, release, late = read(
            rf"SPI_FAULT_SAMPLE trial={i} inject_us=(\d+) release_us=(\d+) late_us=(\d+) "
            r"fault_irq=1 released=1")
        require(FAULT_AT_US <= inject <= release <= FAULT_AT_US + SLACK_US
                and FAULT_AT_US - 50 <= release and late == release - inject + 1 and late <= SLACK_US,
                "forced error did not release CS0 within injection +100 us")
        faults.append((release, late))
    delay_min, delay_upper, fault_min, fault_max, fault_late = read(
        r"SPI_BUS_TRIALS cs0=1000 cs1=1000 forced_error=100 recovered=100 cs_delay_min_us=(\d+) "
        r"cs_delay_upper_us=(\d+) fault_release_min_us=(\d+) fault_release_max_us=(\d+) "
        r"fault_late_max_us=(\d+) bad_cs=0 stray=0 errors=0")
    require(CS_DELAY_US <= delay_min <= delay_upper <= CS_DELAY_US + SLACK_US,
            "CS delay outside timing bound")
    require((fault_min, fault_max, fault_late) ==
            (min(t for t, _ in faults), max(t for t, _ in faults), max(t for _, t in faults)),
            "forced release aggregate disagrees with trial samples")
    irq_us, polling_us = read(
        r"SPI_POLL_COMPARISON hz=1000000 frames=32 irq_us=(\d+) polling_us=(\d+) stray=0 errors=0")
    require(min(irq_us, polling_us) >= 256, "comparison returned before minimum wire time")
    read(r"SPI_PARITY_FIRMWARE_END result=PASS")
    require(next(records, None) is None, "unexpected SPI marker after completion")
    return depth, counts, flush_results, timeout, delay_min, delay_upper, fault_min, fault_max


@dataclass
class Sample:
    period_us: float
    bits: int = 0
    words: int = 0
    word: int = 0
    last_rising: float = 0
    last_falling: float = 0
    first_fall_before_delay: float = 0
    delay: float = 0
    payload: bytearray = field(default_factory=bytearray)

    def rising(self, us, mosi, miso):
        require(mosi == miso, "MOSI/MISO data mismatch")
        if self.bits:
            require(abs(us - self.last_rising - self.period_us) <= max(0.02, self.period_us / 20),
                    "SCLK period within a frame is out of range")
        if self.words == 4 and self.bits == 0:
            self.delay = us - self.first_fall_before_delay
        self.last_rising = us
        self.word = (self.word << 1) | mosi
        self.bits += 1
        if self.bits == 8:
            self.payload.append(self.word)
            self.word = self.bits = 0
            self.words += 1

    def falling(self, us):
        self.last_falling = us
        if self.words == 4 and self.bits == 0:
            self.first_fall_before_delay = us

    def finish(self):
        require(self.bits == 0, "partial wire frame")


def verify_wire(stream, depth):
    samples = iter(stream)
    rate = re.fullmatch(r"# sample_hz=(\d+)", next(samples, "").strip())
    require(rate is not None and int(rate[1]) >= MIN_SAMPLE_HZ, "sample rate below 100 MS/s")
    reader = csv.reader(samples)
    require(next(reader, None) == ["us", "sck", "mosi", "miso", "cs0", "cs1", "gate"],
            "unexpected trace CSV header")
    counts = {"bulk": 0, "txflush": 0, "cs": 0, "fault": 0}
    rx_tail_max = flush_max = 0
    active, sample, started = None, None, None
    previous = None
    for row in reader:
        require(len(row) == 7 and all(bit in ("0", "1") for bit in row[1:]), "invalid trace row")
        us = float(row[0])
        sck, mosi, miso, cs0, cs1, gate = map(int, row[1:])
        require(us >= 0 and us < float("inf"), "invalid time")
        require(cs0 or cs1, "two chip selects simultaneously active")
        require(not gate or (cs0 and cs1), "flush strobe overlapped chip select")
        gate_kind = (active if active in ("bulk", "txflush") else
                     "bulk" if counts["bulk"] < 200 else "txflush")
        select = gate_kind if gate else 0 if not cs0 else 1 if not cs1 else None
        if previous is None:
            require(select is None and not sck, "trace must start idle")
            previous = us, sck
            continue
        previous_us, previous_sck = previous
        require(us > previous_us, "trace times must increase")
        previous = us, sck
        if select != active:
            # TX flush is strobed after FIFO submission, so its first observed
            # frame may already be in progress. Closing still requires idle SCLK.
            require((active is None and select == "txflush") or (not sck and not previous_sck),
                    "strobe or CS moved before SCLK returned idle")
            if sample is not None:
                if active != "txflush":
                    sample.finish()
                if active == "bulk":
                    count = counts["bulk"]
                    require(count < 200 and sample.words == 1_000, "wrong bulk frame count")
                    want = bytes(((i * 37) & 0xff) ^ (count % 100) for i in range(1_000))
                    require(sample.payload == want, "wrong bulk transfer payload")
                    delay = us - sample.last_falling
                    require(0 <= delay <= SLACK_US, "bulk RX completion later than final edge +100 us")
                    rx_tail_max = max(rx_tail_max, delay)
                    counts["bulk"] += 1
                elif active == "txflush":
                    require(0 < sample.words * 8 + sample.bits <= depth * 8,
                            "invalid TX flush frame count")
                    delay = us - sample.last_falling
                    require(sample.last_falling > started and 0 <= delay <= SLACK_US,
                            "flush later than wire completion +100 us")
                    flush_max = max(flush_max, delay)
                    counts["txflush"] += 1
                elif counts["cs"] < 2_000:
                    count = counts["cs"]
                    want = bytes([0x5a, count % 2]) + (count // 2).to_bytes(2, "little")
                    require(count % 2 == active and sample.payload == want + bytes(4),
                            "wrong CS, order or selected payload")
                    require(CS_DELAY_US <= sample.delay <= CS_DELAY_US + SLACK_US,
                            "CS delay outside 4–104 us")
                    counts["cs"] += 1
                else:
                    require(active == 0 and sample.words == 0
                            and FAULT_AT_US - 50 <= us - started <= FAULT_AT_US + SLACK_US,
                            "forced error did not release CS0 within injection +100 us")
                    counts["fault"] += 1
            require(active is None or select is None, "missing both-CS-idle interval")
            if select is not None:
                if select == "bulk":
                    period = 0.1 if counts["bulk"] >= TRIALS else 1
                elif select == "txflush":
                    require(counts["cs"] == 0 and counts["txflush"] < 2 * TRIALS,
                            "unexpected TX flush section")
                    period = 1 if counts["txflush"] < TRIALS else 10
                else:
                    require(counts["txflush"] == 2 * TRIALS, "CS before TX flush")
                    period = 1
                sample = Sample(period)
                started = us
            else:
                sample = None
            active = select
        if sample is not None and previous_sck != sck:
            if sck:
                sample.rising(us, mosi, miso)
            else:
                sample.falling(us)
    require(previous is not None and previous[1] == 0 and active is None, "trace must end idle")
    require(counts == {"bulk": 200, "txflush": 200, "cs": 2_000, "fault": 100},
            f"incorrect wire counts {counts}")
    return rx_tail_max, flush_max


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("uart", type=Path)
    parser.add_argument("wire", type=Path, nargs="?")
    args = parser.parse_args()
    try:
        depth, (rx1, rx10), flush, timeout, delay_min, delay_upper, fault_min, fault_max = verify_uart(
            args.uart.read_text())
        if args.wire:
            with args.wire.open(newline="") as stream:
                rx_tail, wire_flush = verify_wire(stream, depth)
    except (OSError, ValueError, csv.Error) as e:
        print(f"SPI_PARITY_VERIFY_FAIL {e}", file=sys.stderr)
        return 1
    if args.wire:
        print(f"SPI_EXTERNAL_TIMEBASE rx_completion_max_us={rx_tail:.3f} "
              f"flush_late_max_us={wire_flush:.3f} result=PASS")
    else:
        print("SPI_EXTERNAL_TIMEBASE result=NOT_RUN")
    flush_1m, late_1m, tails_1m = flush[1_000_000]
    flush_100k, late_100k, tails_100k = flush[100_000]
    print("PARITY_SPI_IRQ_END source=rp1_timer frames_1mhz=100000 frames_10mhz=100000 "
          f"depth={depth} rx_irq_1mhz={rx1} rx_irq_10mhz={rx10} tx_irq=200 timeouts=100 "
          f"timeout_max_us={timeout} flush_max_us={flush_1m} flush_late_max_us={late_1m} "
          f"tail_trials_1mhz={tails_1m} flush_100khz_max_us={flush_100k} "
          f"flush_100khz_late_max_us={late_100k} tail_trials_100khz={tails_100k} errors=0 result=PASS")
    print("PARITY_SPI_BUS_END source=rp1_timer_gpio cs0=1000 cs1=1000 forced_error=100 released=100 "
          f"cs_delay_min_us={delay_min} cs_delay_upper_us={delay_upper} fault_release_min_us={fault_min} "
          f"fault_release_max_us={fault_max} order_errors=0 duplicate_active=0 result=PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
