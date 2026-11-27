#!/usr/bin/env python3

import importlib.util
import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

SPEC = importlib.util.spec_from_file_location("verify", Path(__file__).with_name("verify-uart-parity.py"))
v = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(v)

NONCE = "a112bb34"
FW = (f"UART_PARITY_FW_END run={NONCE} rx_watermark=1000 rx_timeout=1000 tx=1000 "
      "rx_bytes=100000 tx_bytes=100000 rx_stream_rx=6250 rx_stream_rt=781 rx_stream_tx=0 rx_stream_other=0 "
      "tx_stream_rx=0 tx_stream_rt=0 tx_stream_tx=6250 tx_stream_other=0 "
      "timeout=1 rearm=1 polling=8 flush=1000 false_wake=0 result=PASS")
SAMPLES = (f"UART_PARITY_NB_SAMPLES run={NONCE} kind=full us=" + "0a" * 1000 + "\n"
           + f"UART_PARITY_NB_SAMPLES run={NONCE} kind=empty us=" + "09" * 1000 + "\n")
NB = (f"PARITY_UART_NB_END run={NONCE} full=1000 empty=1000 full_max_us=10 "
      "empty_max_us=9 false_ready=0 result=PASS")


class Trace:
    def __init__(self):
        self.samples = {}
        self.time = 500.

    def pulse(self, channel, start, end):
        self.samples.setdefault(start, {})[channel] = 1
        self.samples.setdefault(end, {})[channel] = 0

    def uart(self, channel, values):
        start = self.time
        for i, value in enumerate(values):
            at = start + i * 10 * v.BIT
            for bit, level in enumerate([0] + [(value >> j) & 1 for j in range(8)] + [1]):
                self.samples.setdefault(at + bit * v.BIT, {})[channel] = level
        self.time += len(values) * 10 * v.BIT
        return start

    def write(self, path):
        levels = [1, 1, 0, 0, 0]
        with path.open("w") as out:
            out.write("us,rx,tx,irq,flush,gate\n0,1,1,0,0,0\n")
            for at, changed in sorted(self.samples.items()):
                for channel, value in changed.items():
                    levels[channel] = value
                out.write(",".join(map(str, [at, *levels])) + "\n")
            out.write(f"{self.time + 1000},1,1,0,0,0\n")


def build_trace(occupied, late=False, flush_delay=5, missing=False):
    trace = Trace()
    nonce = NONCE.encode()
    trace.uart(0, nonce)
    trace.uart(1, nonce)
    for command, width in [(b"R", 16), (b"S", 1)]:
        for i in range(v.TRIALS):
            trace.uart(1, command)
            gate = trace.time + 100
            trace.time += 900
            data = v.pattern(v.TRIALS * width)[i * width:(i + 1) * width]
            start = trace.uart(0, data)
            reference = start + (159.5 if width == 16 else 41.5) * v.BIT
            edge = reference + (55 if late else 3)
            trace.pulse(4, gate, edge + 9)
            if not missing:
                trace.pulse(2, edge, edge + 1)
            trace.time = edge + 200
    tx = v.pattern(v.TX_SAMPLE * v.TRIALS)
    for i, count in enumerate(occupied):
        trace.uart(1, b"T")
        trace.time += 150
        start = trace.uart(1, tx[i * v.TX_SAMPLE:(i + 1) * v.TX_SAMPLE])
        edge = start + (count - 17) * 10 * v.BIT + 2
        trace.pulse(4, start + 50, edge + 5)
        trace.pulse(2, edge, edge + 1)
        trace.pulse(3, edge + 7, trace.time + flush_delay)
        trace.time += 150
        trace.uart(1, bytes([count]))
    trace.uart(1, b"I")
    trace.uart(0, v.pattern(v.STREAM))
    trace.uart(1, b"O" + v.pattern(v.STREAM) + b"A")
    trace.uart(0, b"a")
    trace.uart(1, b"P")
    trace.uart(0, nonce)
    trace.uart(1, nonce)
    return trace


class VerifyUartParity(unittest.TestCase):
    def test_firmware_end_markers_reject_stale_run_duplicates_and_bound(self):
        v.markers(FW + "\n" + NB + "\n" + SAMPLES, NONCE)
        for fw, nb in [(FW, NB.replace("us=10", "us=11")), (FW + "\n" + FW, NB), (FW, NB[1:])]:
            with self.assertRaises(ValueError):
                v.markers(fw + "\n" + nb + "\n" + SAMPLES, NONCE)
        with self.assertRaises(ValueError):
            v.markers(FW + "\n" + NB + "\n" + SAMPLES, "b112bb34")

    def test_rejects_stream_irq_excess_and_wrong_directions(self):
        for field, before, after, message in [
            ("rx_stream_rt", 781, 782, "RX stream IRQ count"),
            ("tx_stream_tx", 6250, 6251, "TX stream IRQ count"),
            ("rx_stream_tx", 0, 1, "unexpected stream IRQ direction"),
            ("tx_stream_other", 0, 1, "unexpected stream IRQ direction"),
        ]:
            with self.subTest(field=field), self.assertRaisesRegex(ValueError, message):
                fw = FW.replace(f"{field}={before}", f"{field}={after}")
                v.markers(fw + "\n" + NB + "\n" + SAMPLES, NONCE)

    @patch.object(v, "TRIALS", 2)
    @patch.object(v, "STREAM", 60)
    def test_correlates_wire_frames_and_rejects_invalid_timing(self):
        with tempfile.TemporaryDirectory() as tmp:
            directory = Path(tmp)
            uart0, trace = directory / "uart0", directory / "logic.csv"
            uart0.write_text(FW + "\n" + NB + "\n" + SAMPLES)
            occupied = [32, 33]
            metadata = {"run": NONCE, "prefill": occupied, "sample_rate_hz": 10_000_000}
            peer_json = directory / "peer.json"
            peer_json.write_text(json.dumps(metadata))
            incoming, outgoing, _ = v.transcript(NONCE.encode(), occupied)
            (directory / "to-rp1.bin").write_bytes(incoming)
            (directory / "from-rp1.bin").write_bytes(outgoing)
            build_trace(occupied).write(trace)
            self.assertEqual(v.verify(uart0, directory, trace)[0], NONCE)
            for options, rejection in [
                ({"missing": True}, "missing an IRQ"),
                ({"late": True}, "exceeds 50"),
                ({"flush_delay": -5}, "physical stop-bit"),
                ({"flush_delay": v.FLUSH_SLACK_US + 1}, "flush returned"),
            ]:
                build_trace(occupied, **options).write(trace)
                with self.assertRaisesRegex(ValueError, rejection):
                    v.verify(uart0, directory, trace)
            build_trace(occupied).write(trace)
            peer_json.write_text(json.dumps({**metadata, "sample_rate_hz": 9_999_999}))
            with self.assertRaisesRegex(ValueError, "analyzer sample rate"):
                v.verify(uart0, directory, trace)
            peer_json.write_text(json.dumps(metadata))
            (directory / "to-rp1.bin").write_bytes(incoming[1:])
            with self.assertRaisesRegex(ValueError, "sent transcript mismatch"):
                v.verify(uart0, directory, trace)


if __name__ == "__main__":
    unittest.main()
