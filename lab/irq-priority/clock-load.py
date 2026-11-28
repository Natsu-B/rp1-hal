#!/usr/bin/env python3
"""Sweep Linux clocks at the marked device waits and then throughout the run."""

import argparse
import mmap
import os
import struct
import time
from pathlib import Path


def sweep(fd):
    size = 0
    while chunk := os.read(fd, 65536):
        size += len(chunk)
    if size == 0:
        raise RuntimeError("empty clock summary")


def wait_for_marker(window, offset, marker):
    end = time.monotonic() + 0.5
    while struct.unpack_from("<I", window, offset)[0] != marker:
        if time.monotonic() >= end:
            raise TimeoutError(f"RP1 clock probe marker {marker:#x}")
        time.sleep(0.00001)


def load(window, offset, trigger, summary):
    # These trial indices match the markers in the RP1 fixture. Open the first
    # summary before releasing RP1, so reading can start as soon as it signals.
    probes = [
        (label, tag | trial, trial)
        for label, tag, trials in [
            ("spi", 0x1000_0000, range(0, 10, 3)),
            ("uart", 0x2000_0000, range(12, 22, 3)),
        ]
        for trial in trials
    ]
    completed = 0
    for label, marker, trial in probes:
        with summary.open("rb") as stream:
            if completed == 0:
                struct.pack_into("<I", window, offset, trigger)
            wait_for_marker(window, offset, marker)
            sweep(stream.fileno())
        completed += 1
        print(f"CLOCK_PROBE driver={label} trial={trial}", flush=True)

    for _ in range(100 - completed):
        with summary.open("rb") as stream:
            sweep(stream.fileno())
        completed += 1
        time.sleep(0.15)
    print(f"CLOCK_SWEEP_COUNT={completed}", flush=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("address", type=lambda value: int(value, 0))
    parser.add_argument("trigger", type=lambda value: int(value, 0))
    args = parser.parse_args()
    page_size = mmap.PAGESIZE
    offset = args.address % page_size
    if offset % 4:
        parser.error("trigger address must be word-aligned")
    summary = Path("/sys/kernel/debug/clk/clk_summary")
    memory = os.open("/dev/mem", os.O_RDWR | os.O_SYNC)
    try:
        with mmap.mmap(memory, page_size, offset=args.address - offset) as window:
            load(window, offset, args.trigger, summary)
    finally:
        os.close(memory)


if __name__ == "__main__":
    main()
