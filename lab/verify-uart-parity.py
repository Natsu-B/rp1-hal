#!/usr/bin/env python3
"""Verify UART1 parity from raw UART0, RP1 timer brackets and FT230X byte captures."""

import argparse
import json
import math
import re
from pathlib import Path

TRIALS = 1000
STREAM = 100_000
TX_SAMPLE = 34
STREAM_RX_IRQ_MAX = 7031
STREAM_TX_IRQ_MAX = 6250
FLUSH_MAX_US = 1_000_000 / 115_200 + 50 + 0.5
EDGE_MAX_US = 10
RX, TX, RT = 0x10, 0x20, 0x40
BUSY, RXFE, TXFF, TXFE = 8, 16, 32, 128
CLOCK_HZ = 50_000_000
DIVISOR = 1736
BIT = 250_000 * DIVISOR / CLOCK_HZ


def require(ok, message):
    if not ok:
        raise ValueError(message)


def pattern(nonce, length, offset=0):
    return bytes((((i * 73) ^ (i >> 8)) & 255) ^ nonce[i % 8]
                 for i in range(offset, offset + length))


def transcript(nonce, occupied):
    inbound = bytearray(nonce)
    outbound = bytearray(nonce)
    for command, width in [(b"R", 16), (b"S", 1)]:
        outbound.extend(command * TRIALS)
        inbound.extend(pattern(nonce, TRIALS * width))
    for i, count in enumerate(occupied):
        outbound.extend(b"T" + pattern(nonce, TX_SAMPLE, i * TX_SAMPLE) + bytes([count]))
    inbound.extend(pattern(nonce, STREAM) + nonce[:1] + nonce + nonce)
    outbound.extend(b"IO" + pattern(nonce, STREAM) + b"AQ" + nonce + b"P" + nonce)
    return inbound, outbound


def rows(text, name):
    for line in text.splitlines():
        if line.startswith(name + " "):
            pairs = [field.split("=", 1) for field in line.split()[1:]]
            fields = dict(pairs)
            require(len(fields) == len(pairs), f"duplicate field in {name}")
            yield fields


def single(text, name, **expected):
    found = list(rows(text, name))
    require(len(found) == 1, f"{name} missing or duplicate")
    require(all(found[0].get(key) == str(value) for key, value in expected.items()),
            f"{name} field mismatch")
    return found[0]


def elapsed(end, start):
    """Signed time difference for a measurement shorter than half the 32-bit period."""
    return ((end - start + (1 << 31)) % (1 << 32)) - (1 << 31)


def measure(kind, low, high, entry, ris, fr, serviced, empty_lo, empty_hi,
            busy_lo, busy_hi, begin, end, end_fr):
    source = {"rx": RX, "rt": RT, "tx": TX}[kind]
    rx = source != TX
    require(serviced & source and serviced & ~(RX | RT if rx else TX) == 0,
            "wrong service source")
    require(ris & source and not fr & (RXFE if rx else TXFF), "IRQ entry RIS/FIFO mismatch")
    span = elapsed(high, low) + 1
    require(1 <= span <= EDGE_MAX_US, f"RIS bracket {span} us")
    entry_upper = elapsed(entry, low) + 1
    require(1 <= entry_upper <= 50, f"IRQ entry upper bound {entry_upper} us")
    if rx:
        require(not any((empty_lo, empty_hi, busy_lo, busy_hi, begin, end, end_fr)),
                "unexpected TX sample in RX trial")
        return (entry_upper,)

    require(0 <= elapsed(begin, entry) and elapsed(empty_lo, begin) >= 0,
            "flush did not start before TX FIFO emptied")
    for label, lo, hi in [("TXFE", empty_lo, empty_hi), ("BUSY", busy_lo, busy_hi)]:
        require(1 <= elapsed(hi, lo) + 1 <= EDGE_MAX_US, f"{label} bracket width")
    # TXFE marks when the last frame enters the shifter. Allow one bit for
    # internal phase and one timer tick for quantization, with no external clock.
    model_lo = 9 * BIT
    model_hi = elapsed(empty_hi, empty_lo) + 1 + 11 * BIT
    busy_low = elapsed(busy_lo, empty_lo)
    busy_high = elapsed(busy_hi, empty_lo) + 1
    require(busy_low <= model_hi and busy_high >= model_lo,
            "BUSY clear outside baud-derived stop-bit interval")
    finish = elapsed(end, empty_lo)
    late_busy = elapsed(end, busy_lo) + 1
    late_model = finish + 1 - model_lo
    require(end_fr & (BUSY | TXFE) == TXFE and late_busy >= 1,
            "flush returned before idle")
    require(late_model >= 0, "flush returned before baud-derived physical end")
    require(max(late_busy, late_model) <= FLUSH_MAX_US,
            f"flush late upper bound BUSY={late_busy} model={late_model:.3f} us")
    return (entry_upper, late_busy, late_model, finish - 10 * BIT)


def distribution(values):
    values = sorted(values)
    return "/".join(f"{values[i]:.3f}" for i in
                    (0, math.ceil(0.50 * len(values)) - 1, math.ceil(0.99 * len(values)) - 1, -1))


def markers(text, run):
    fw = single(text, "UART_PARITY_FW_END", run=run, rx_watermark=TRIALS, rx_timeout=TRIALS,
                tx=TRIALS, rx_bytes=STREAM, tx_bytes=STREAM, timeout=1, rearm=1, polling=8,
                flush=TRIALS, false_wake=0, result="PASS")
    stream_rx, stream_rt, stream_tx = (int(fw[k]) for k in
                                       ("rx_stream_rx", "rx_stream_rt", "tx_stream_tx"))
    require(all(int(fw[k]) == 0 for k in ("rx_stream_tx", "rx_stream_other", "tx_stream_rx",
                                          "tx_stream_rt", "tx_stream_other")),
            "unexpected stream IRQ direction")
    require(0 <= min(stream_rx, stream_rt) and 1 <= stream_rx + stream_rt <= STREAM_RX_IRQ_MAX,
            f"RX stream IRQ count outside 1..{STREAM_RX_IRQ_MAX}")
    require(1 <= stream_tx <= STREAM_TX_IRQ_MAX, f"TX stream IRQ count outside 1..{STREAM_TX_IRQ_MAX}")
    nb = single(text, "PARITY_UART_NB_END", run=run, full=TRIALS, empty=TRIALS,
                false_ready=0, result="PASS")
    samples = list(rows(text, "UART_PARITY_NB_SAMPLES"))
    require(len(samples) == 2, "expected two native try sample sets")
    for kind in ("full", "empty"):
        matching = [r["us"] for r in samples if r["kind"] == kind and r["run"] == run]
        require(len(matching) == 1, f"{kind} native try sample set missing or duplicate")
        values = bytes.fromhex(matching[0])
        require(len(values) == TRIALS and 1 <= min(values) <= max(values) == int(nb[f"{kind}_max_us"]) <= 10,
                f"{kind} native try samples outside 1..10 us or inconsistent maximum")
    rearm = single(text, "UART_PARITY_REARM", run=run, timeout_irq=0, rearm_rt=1, byte=ord(run[0]))
    require(10_000 <= int(rearm["timeout_us"]) <= 10_050, "cancel/timeout timer outside 10000..10050 us")
    compare = single(text, "UART_PARITY_COMPARE", run=run, irq_rt=1, poll_irq=0, bytes=8)
    require(min(int(compare["irq_us"]), int(compare["poll_us"])) > 0, "missing comparison duration")
    return stream_rx, stream_rt, stream_tx


def verify(uart0, peer):
    metadata = json.loads((peer / "peer.json").read_text())
    run, occupied = metadata["run"], metadata["prefill"]
    require(re.fullmatch(r"[0-9a-f]{8}", run), "invalid nonce")
    require(len(occupied) == TRIALS and all(n in (32, 33) for n in occupied), "invalid TX prefill")
    text = uart0.read_text(errors="replace").replace("\r", "")
    require(text.splitlines().count("UART_PARITY_READY") == 1, "UART_PARITY_READY missing or duplicate")
    single(text, "UART_PARITY_CLOCK", timer_hz=1_000_000, uart_hz=CLOCK_HZ, divisor=DIVISOR)
    stream_counts = markers(text, run)
    inbound, outbound = transcript(run.encode(), occupied)
    require((peer / "to-rp1.bin").read_bytes() == inbound, "UART1 sent transcript mismatch")
    require((peer / "from-rp1.bin").read_bytes() == outbound, "UART1 received transcript mismatch")

    trials = iter(rows(text, "UART_PARITY_TRIAL"))
    results = {}
    for kind in ("rx", "rt", "tx"):
        group = []
        for i in range(TRIALS):
            row = next(trials)
            require((row.pop("run"), row.pop("kind"), row.pop("i")) == (run, kind, str(i)),
                    f"trial sequence/nonce mismatch at {kind}[{i}]")
            try:
                words = {key: int(value) for key, value in row.items()}
                require(all(0 <= x < 1 << 32 for x in words.values()), "not a u32")
                group.append(measure(kind, **words))
            except (ValueError, TypeError) as error:
                raise ValueError(f"{kind}[{i}] {error}") from error
        results[kind] = group
    require(next(trials, None) is None, "extra trial")
    return run, results, stream_counts


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--uart0", type=Path, required=True)
    parser.add_argument("--peer", type=Path, required=True)
    args = parser.parse_args()
    try:
        run, groups, (rx_stream, rt_stream, tx_stream) = verify(args.uart0, args.peer)
    except (ValueError, OSError, KeyError, StopIteration) as error:
        raise SystemExit(f"PARITY_UART_IRQ_END result=FAIL reason={str(error) or 'missing trial'}") from error
    for kind, group in groups.items():
        print(f"UART_PARITY_TIMING run={run} kind={kind} n={len(group)} order=min/p50/p99/max "
              f"entry_upper_us={distribution([row[0] for row in group])}")
    for i, label in enumerate(("busy_late_upper_us", "model_late_upper_us", "txfe_lo_plus_10bit_late_us"), 1):
        print(f"UART_PARITY_FLUSH run={run} n={TRIALS} order=min/p50/p99/max "
              f"{label}={distribution([row[i] for row in groups['tx']])}")
    print(f"PARITY_UART_IRQ_END run={run} rx_watermark=1000 rx_timeout=1000 tx=1000 "
          f"rx_bytes=100000 tx_bytes=100000 false_wake=0 lost=0 "
          f"rx_stream_rx={rx_stream} rx_stream_rt={rt_stream} tx_stream_tx={tx_stream} "
          f"timeout=1 rearm=1 polling=8 flush=1000 result=PASS")


if __name__ == "__main__":
    main()
