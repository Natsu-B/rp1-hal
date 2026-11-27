#!/usr/bin/env python3
"""Cross-check the UART1 peer, firmware and time-aligned logic analyzer transitions."""

import argparse
import csv
import json
import math
import re
from bisect import bisect_right
from pathlib import Path

TRIALS = 1000
STREAM = 100_000
TX_SAMPLE = 34
BIT = 1_000_000 / 115_200
CAPTURE_US = 0.5
FLUSH_SLACK_US = BIT + 50 + CAPTURE_US
MIN_SAMPLE_RATE_HZ = 10_000_000
STREAM_TX_IRQ_MAX = math.ceil(STREAM / 16)
STREAM_RX_IRQ_MAX = STREAM_TX_IRQ_MAX + STREAM // 128
COLUMNS = ["us", "rx", "tx", "irq", "flush", "gate"]


def require(condition, message):
    if not condition:
        raise ValueError(message)


def transition_data(path):
    times = []
    values = [[] for _ in COLUMNS[1:]]
    with path.open(newline="") as trace:
        reader = csv.reader(trace)
        require(next(reader) == COLUMNS, f"trace header must be {','.join(COLUMNS)}")
        for time, *bits in reader:
            instant = float(time)
            require(math.isfinite(instant) and (not times or instant > times[-1]), "unordered trace")
            require(len(bits) == 5 and all(bit in ("0", "1") for bit in bits), "invalid digital sample")
            times.append(instant)
            for column, bit in zip(values, bits):
                column.append(bit == "1")
    require(times and [column[0] for column in values] == [True, True, False, False, False],
            "trace must start idle")
    require([column[-1] for column in values] == [True, True, False, False, False],
            "trace must end idle")
    return times, values


def decode(times, wire):
    def high_at(instant):
        return wire[bisect_right(times, instant) - 1]

    frames = []
    earliest = times[0]
    for i, instant in enumerate(times[1:], 1):
        if instant < earliest or wire[i] or not wire[i - 1]:
            continue
        require(instant + 10 * BIT <= times[-1], "truncated UART frame")
        require(not high_at(instant + BIT / 2) and high_at(instant + 9.5 * BIT),
                f"invalid UART framing at {instant:.3f} us")
        data = sum(high_at(instant + (bit + 1.5) * BIT) << bit for bit in range(8))
        frames.append((instant, data))
        # Permit 0.2 bit time of clock skew, without interpreting in-frame edges as starts.
        earliest = instant + 9.8 * BIT
    return frames


def edges(times, signal, value):
    return [times[i] for i in range(1, len(times)) if signal[i] == value and signal[i - 1] != value]


def windows(times, signal, count, name):
    rising = edges(times, signal, True)
    falling = edges(times, signal, False)
    require(len(rising) == len(falling) == count, f"{name}: expected {count} complete strobes")
    return list(zip(rising, falling))


def pattern(length):
    return bytes(((i * 73) ^ (i >> 8)) & 255 for i in range(length))


def transcript(nonce, occupied):
    outbound = bytearray(nonce + b"R" * TRIALS + b"S" * TRIALS)
    tx_cases = []
    payload = pattern(TRIALS * TX_SAMPLE)
    for i, count in enumerate(occupied):
        start = i * TX_SAMPLE
        tx_cases.append((len(outbound) + 1, count))
        outbound.extend(b"T" + payload[start:start + TX_SAMPLE] + bytes([count]))
    stream = pattern(STREAM)
    outbound.extend(b"IO" + stream + b"AP" + nonce)
    inbound = nonce + pattern(TRIALS * 16) + pattern(TRIALS) + stream + b"a" + nonce
    return inbound, outbound, tx_cases


def latency(edge, reference, uncertainty):
    # Reject an early unrelated edge, then include measurement uncertainty in the bound.
    delay = edge - reference
    require(delay >= -uncertainty, "IRQ strobe precedes its wire event")
    require(delay + uncertainty <= 50, f"IRQ entry upper bound exceeds 50 us ({delay + uncertainty:.3f})")
    return delay + uncertainty


def markers(text, run):
    fw = re.findall(
        r"^UART_PARITY_FW_END run=([0-9a-f]{8}) rx_watermark=1000 rx_timeout=1000 tx=1000 "
        r"rx_bytes=100000 tx_bytes=100000 "
        r"rx_stream_rx=(\d+) rx_stream_rt=(\d+) rx_stream_tx=(\d+) rx_stream_other=(\d+) "
        r"tx_stream_rx=(\d+) tx_stream_rt=(\d+) tx_stream_tx=(\d+) tx_stream_other=(\d+) "
        r"timeout=1 rearm=1 polling=8 flush=1000 false_wake=0 result=PASS$",
        text, re.M,
    )
    nb = re.findall(
        r"^PARITY_UART_NB_END run=([0-9a-f]{8}) full=1000 empty=1000 full_max_us=(\d+) "
        r"empty_max_us=(\d+) false_ready=0 result=PASS$",
        text, re.M,
    )
    require(text.count("UART_PARITY_FW_END ") == 1 and len(fw) == 1 and fw[0][0] == run,
            "UART_PARITY_FW_END mismatch")
    rx, rt, rx_tx, rx_other, tx_rx, tx_rt, tx, tx_other = map(int, fw[0][1:])
    require(rx_tx + rx_other + tx_rx + tx_rt + tx_other == 0, "unexpected stream IRQ direction")
    require(1 <= rx + rt <= STREAM_RX_IRQ_MAX, f"RX stream IRQ count outside 1..{STREAM_RX_IRQ_MAX}")
    require(1 <= tx <= STREAM_TX_IRQ_MAX, f"TX stream IRQ count outside 1..{STREAM_TX_IRQ_MAX}")
    require(text.count("PARITY_UART_NB_END ") == 1 and len(nb) == 1 and nb[0][0] == run, "PARITY_UART_NB_END mismatch")
    maxima = dict(zip(("full", "empty"), map(int, nb[0][1:])))
    require(max(maxima.values()) <= 10, "native try exceeds 10 us")
    require(text.count("UART_PARITY_NB_SAMPLES ") == 2, "missing or duplicate native samples")
    for kind, maximum in maxima.items():
        rows = re.findall(rf"^UART_PARITY_NB_SAMPLES run={run} kind={kind} us=([0-9a-f]{{2000}})$",
                          text, re.M)
        require(len(rows) == 1, f"{kind} sample count or nonce mismatch")
        samples = bytes.fromhex(rows[0])
        require(1 <= min(samples) <= max(samples) == maximum, f"{kind} sample maximum mismatch")
    return rx, rt, tx


def verify(uart0, peer, trace):
    run = json.loads((peer / "peer.json").read_text())
    nonce = run["run"]
    occupied = run["prefill"]
    require(re.fullmatch(r"[0-9a-f]{8}", nonce), "invalid nonce")
    require(len(occupied) == TRIALS and all(n in (32, 33) for n in occupied), "invalid TX prefill")
    rate = run["sample_rate_hz"]
    require(type(rate) is int and rate >= MIN_SAMPLE_RATE_HZ,
            f"analyzer sample rate must be at least {MIN_SAMPLE_RATE_HZ} Hz")
    stream_counts = markers(uart0.read_text(errors="replace"), nonce)
    inbound, outbound, tx_cases = transcript(nonce.encode(), occupied)
    require((peer / "to-rp1.bin").read_bytes() == inbound, "UART1 sent transcript mismatch")
    require((peer / "from-rp1.bin").read_bytes() == outbound, "UART1 received transcript mismatch")

    times, (rx, tx, irq, flush, gate) = transition_data(trace)
    received, sent = decode(times, rx), decode(times, tx)
    require(bytes(b for _, b in received) == inbound, "RX wire transcript mismatch")
    require(bytes(b for _, b in sent) == outbound, "TX wire transcript mismatch")
    gates = windows(times, gate, 3 * TRIALS, "wait")
    flushes = windows(times, flush, TRIALS, "flush")
    irq_edges = edges(times, irq, True)

    def irq_in(window):
        enter, leave = window
        first, last = bisect_right(irq_edges, enter), bisect_right(irq_edges, leave)
        require(last - first == 1, "wait window is missing an IRQ or has multiple IRQs")
        return irq_edges[first]

    peaks = [0., 0., 0.]
    flush_late_max = 0.
    for i in range(TRIALS):
        first = 8 + i * 16
        r, s, t = gates[i], gates[TRIALS + i], gates[2 * TRIALS + i]
        require(r[0] < received[first][0] and s[0] < received[8 + 16 * TRIALS + i][0],
                "peer transmitted before wait was armed")
        watermark = received[first + 15][0] + 9.5 * BIT
        timeout = received[8 + 16 * TRIALS + i][0] + 41.5 * BIT
        start, count = tx_cases[i]
        threshold = sent[start + count - 17][0]
        # PL011 oversampling, bit-period error and capture resolution have bounded
        # uncertainty. Budget one bit for timeout, half a bit for the RX watermark.
        for kind, edge, ref, bound in [
            (0, irq_in(r), watermark, 0.5 * BIT + CAPTURE_US),
            (1, irq_in(s), timeout, BIT + CAPTURE_US),
            (2, irq_in(t), threshold, CAPTURE_US),
        ]:
            peaks[kind] = max(peaks[kind], latency(edge, ref, bound))
        physical_end = sent[start + TX_SAMPLE - 1][0] + 10 * BIT
        begin, finish = flushes[i]
        require(t[1] <= begin < physical_end and finish >= physical_end + CAPTURE_US,
                "flush strobe did not cover physical stop-bit completion")
        late = finish - physical_end
        require(late <= FLUSH_SLACK_US,
                f"flush returned {late:.1f} us after the stop bit (limit {FLUSH_SLACK_US:.3f} us)")
        flush_late_max = max(flush_late_max, late)
        require(finish < sent[start + TX_SAMPLE][0], "flush strobe did not precede count byte")

    for name, frames, start in [
        ("RX", received, 8 + 17 * TRIALS),
        ("TX", sent, len(sent) - STREAM - 10),
    ]:
        segment = frames[start:start + STREAM]
        require(all(b[0] - a[0] <= 12 * BIT for a, b in zip(segment, segment[1:])),
                f"{name} stream idle gap exceeds two bit times")
    return nonce, peaks, flush_late_max, stream_counts


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for option in ("uart0", "peer", "trace"):
        parser.add_argument(f"--{option}", type=Path, required=True)
    args = parser.parse_args()
    try:
        run, (rx, rt, tx), flush_late_max, (stream_rx, stream_rt, stream_tx) = verify(
            args.uart0, args.peer, args.trace
        )
    except (ValueError, OSError, KeyError, StopIteration) as error:
        raise SystemExit(f"PARITY_UART_IRQ_END result=FAIL reason={error}") from error
    print(f"PARITY_UART_IRQ_END run={run} rx_watermark=1000 rx_timeout=1000 tx=1000 "
          f"rx_bytes=100000 tx_bytes=100000 false_wake=0 lost=0 "
          f"rx_stream_rx={stream_rx} rx_stream_rt={stream_rt} tx_stream_tx={stream_tx} "
          f"rx_entry_us={rx:.3f} rt_entry_us={rt:.3f} tx_entry_us={tx:.3f} "
          f"timeout=1 rearm=1 flush=1000 flush_late_max_us={flush_late_max:.3f} result=PASS")


if __name__ == "__main__":
    main()
