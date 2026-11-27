#!/usr/bin/env python3
"""Verify normalized UART captures for the DMA parity and shm DMA proofs."""

import argparse
import re
from pathlib import Path

BOOT = re.compile(r"PARITY_DMA_BOOT nonce=([0-9a-f]{8}) PLAT_INIT ok=1 .+")
CANCEL = re.compile(
    r"PARITY_DMA_CANCEL nonce=([0-9a-f]{8}) ch=(\d+) count=(\d+) "
    r"stopped=(\d+) quarantined=(\d+) max_us=(\d+) partial=(\d+) "
    r"early=0 range_diff=0 late_diff=0 payload_diff=0"
)


def parity(uart0):
    rows = iter(line for line in uart0.splitlines() if line.startswith("PARITY_DMA_"))
    boot = BOOT.fullmatch(next(rows, ""))
    if not boot or int(boot[1], 16) in (0, 0xffffffff):
        raise ValueError("missing parity boot identity")
    nonce = boot[1]
    stopped = quarantined = partial = maximum = 0
    for channel in range(8):
        for size in (4, 64, 4096):
            if next(rows, "") != (
                f"PARITY_DMA_COPY nonce={nonce} ch={channel} bytes={size} "
                "count=1000 crc_diff=0 range_diff=0"
            ):
                raise ValueError(f"invalid parity copy results for channel {channel}, {size} B")
        cancel = CANCEL.fullmatch(next(rows, ""))
        if not cancel or cancel[1] != nonce or int(cancel[2]) != channel:
            raise ValueError(f"invalid cancellation results for channel {channel}")
        count, idle, held, max_us, prefix = map(int, cancel.groups()[2:])
        if count != 12 + int(channel < 4) or idle + held != count or max_us > 1000 or prefix > count:
            raise ValueError(f"cancellation counts or time outside bounds for channel {channel}")
        stopped += idle
        quarantined += held
        partial += prefix
        maximum = max(maximum, max_us)
    if partial == 0:
        raise ValueError("no cancelled transfer wrote a proper prefix")
    if next(rows, "") != f"PARITY_DMA_RECOVERY nonce={nonce} forgotten=1 leases_claimed=2 reinit=1":
        raise ValueError("missing forgotten-transfer recovery result")
    if next(rows, "") != (
        f"PARITY_DMA_SINGLE_END nonce={nonce} channels=8 copies=24000 cancels=100 ok=1"
    ) or next(rows, None) is not None:
        raise ValueError("missing, failed, duplicated or mixed parity end")
    return nonce, stopped, quarantined, maximum


def ordered(log, patterns):
    rows = iter(log.splitlines())
    for pattern in patterns:
        matches = (match for line in rows if (match := re.fullmatch(pattern, line)))
        match = next(matches, None)
        if match is None:
            raise ValueError(f"missing ordered marker matching {pattern}")
        yield match


def shm(uart0, uart10):
    if any(re.search(pattern, uart0) for pattern in (
        r"\bDMA_SRAM_FAIL\b", r"\bDMA_ABORT_FAIL\b", r"\bDMA_DDR_ERR\b"
    )):
        raise ValueError("firmware reported DMA regression failure")
    patterns = [r"p1 dma init rc=0x0"]
    patterns += [fr"DMA_SRAM bytes={size} rc=0x0 bad_bytes=0" for size in (4, 12, 16, 64, 1000, 1024)]
    patterns += [r"DMA_SRAM_RATE bytes=1024 n=1000 ch0_us=[1-9]\d* ch2_us=[1-9]\d* cpu_us=[1-9]\d*", r"DMA_SRAM_PASS"]
    for size in (768, 2304, 2112, 3072, 4032, 3960, 4080):
        for width in ((16, 8) if size % 16 == 0 else (8, 8)):
            patterns.append(fr"RINGDMA_RAW bytes={size} width={width} count=1000 total_us=[1-9]\d* bad=0")
    patterns.append(r"DMA_ABORT_FULL bytes=4094 width=1 rc=0x0 us=[1-9]\d* copied=4094 ok=1")
    for quarter in range(1, 4):
        patterns += [
            fr"DMA_ABORT q={quarter} delay_us=[1-9]\d* how=1 chen_after=0 copied=([0-9]+) "
            r"len=4094 tail_bad=0 guard_bad=0 late_change=0 elapsed_us=\d+ ok=1",
            fr"DMA_ABORT_REUSE q={quarter} rc=0x0 us=[1-9]\d* copied=4094 ok=1",
        ]
    patterns += [
        r"DMA_ABORT_RAW reg=0x01c chen_after=0 clear_us=\d+ intstatus=0x[0-9a-f]{8} "
        r"aborted=1 copied=\d+ tail_bad=\d+ guard_bad=0 reuse_rc=0x0 reuse_ok=1 ok=1",
        r"DMA_ABORT_PASS midway=[1-3]",
        r"DMA_DDR small16_n1000_us=[1-9]\d* small512_n1000_us=[1-9]\d* cpu_512k_us=[1-9]\d* "
        r"atu_512k_us=[1-9]\d* direct_512k_us=[1-9]\d* readback_bad=0 "
        r"atu_64k_x8_us=[1-9]\d* direct_64k_x8_us=[1-9]\d* errors=0",
    ]
    for match in ordered(uart0, patterns):
        if match.lastindex and int(match[1]) >= 4094:
            raise ValueError("cancelled remote block finished before the stimulus")
    linux = [
        r"V7_DMA_BEGIN",
        r"WINDOW_PASS bytes=[1-9]\d* all_words errors=0",
        r"SHM_READY generation=[1-9]\d*",
        r"V7_DMA_PROBE_RC=0",
        r"PING_RTT .+",
        r"V7_DMA_PING_BEFORE_RC=0",
        r"DMA_DDR_LINUX rp1_errors=0 .+ rp1_readback_bad=0",
        r"DMA_DDR_VERIFY words=262144 bad=0 canary_left=0 first_bad=None",
        r"DMA_DDR_PASS",
        r"V7_DMA_COMMAND_RC=0",
        r"SHM_PASS.*",
        r"V7_DMA_WORKLOAD_RC=0",
        r"PING_RTT .+",
        r"V7_DMA_PING_AFTER_RC=0",
        r"V7_DMA_END",
    ]
    # The shell prints the end marker once and only after all client exits are captured.
    if uart10.splitlines().count("V7_DMA_END") != 1 or uart10.splitlines().count("V7_DMA_BEGIN") != 1:
        raise ValueError("missing or duplicated V7 DMA capture endpoints")
    if uart0.splitlines().count("DMA_SRAM_PASS") != 1:
        raise ValueError("missing or duplicated firmware boot result")
    for _ in ordered(uart10, linux):
        pass


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("mode", choices=("parity", "shm"))
    parser.add_argument("uart0", type=Path)
    parser.add_argument("uart10", type=Path, nargs="?")
    args = parser.parse_args()
    try:
        uart0 = args.uart0.read_text()
        if args.mode == "parity":
            if args.uart10:
                raise ValueError("parity expects one normalized UART0 stream")
            nonce, stopped, quarantined, maximum = parity(uart0)
            print(f"VERIFY_DMA_PARITY_PASS nonce={nonce} stopped={stopped} quarantined={quarantined} max_us={maximum}")
        else:
            if not args.uart10:
                raise ValueError("shm expects normalized UART0 and UART10 streams")
            shm(uart0, args.uart10.read_text())
            print("VERIFY_DMA_SHM_PASS words=262144")
    except (OSError, ValueError) as error:
        parser.exit(1, f"DMA capture: {error}\n")


if __name__ == "__main__":
    main()
