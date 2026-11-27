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
FW = (f"UART_PARITY_FW_END run={NONCE} rx_watermark=2 rx_timeout=2 tx=2 "
      "rx_bytes=60 tx_bytes=60 rx_stream_rx=2 rx_stream_rt=1 rx_stream_tx=0 rx_stream_other=0 "
      "tx_stream_rx=0 tx_stream_rt=0 tx_stream_tx=2 tx_stream_other=0 "
      "timeout=1 rearm=1 polling=8 flush=2 false_wake=0 result=PASS")
TAIL = (f"UART_PARITY_NB_SAMPLES run={NONCE} kind=full us=030a\n"
        f"UART_PARITY_NB_SAMPLES run={NONCE} kind=empty us=0209\n"
        f"PARITY_UART_NB_END run={NONCE} full=2 empty=2 full_max_us=10 "
        "empty_max_us=9 false_ready=0 result=PASS\n"
        f"UART_PARITY_REARM run={NONCE} timeout_us=10002 timeout_irq=0 rearm_rt=1 byte=97\n"
        f"UART_PARITY_COMPARE run={NONCE} irq_us=6000 irq_rt=1 poll_us=5500 poll_irq=0 bytes=8\n")
TX = dict(low=100, high=101, entry=102, ris=32, fr=8, serviced=32,
          empty_lo=500, empty_hi=501, busy_lo=587, busy_hi=588, begin=130, end=600, end_fr=128)


def capture():
    lines = ["UART_PARITY_CLOCK timer_hz=1000000 uart_hz=50000000 divisor=1736", "UART_PARITY_READY"]
    for kind, source in [("rx", 0x50), ("rt", 0x40), ("tx", 0x20)]:
        data = TX if kind == "tx" else {
            **dict.fromkeys(TX, 0),
            "low": 100, "high": 101, "entry": 102, "ris": source, "serviced": source,
        }
        for i in range(2):
            lines.append(f"UART_PARITY_TRIAL run={NONCE} kind={kind} i={i} " +
                         " ".join(f"{key}={value}" for key, value in data.items()))
    return "\n".join(lines + [TAIL, FW, ""])


class VerifyUartParity(unittest.TestCase):
    def test_two_sided_timing_sources_and_wrap(self):
        self.assertEqual(v.measure("tx", **TX)[:2], (3, 14))
        self.assertEqual(v.elapsed(1, 0xffffffff), 2)
        for field, value, message in [
            ("entry", 150, "entry upper bound"),
            ("entry", 99, "entry upper bound"),
            ("high", 110, "RIS bracket"),
            ("serviced", 0, "service source"),
            ("fr", v.TXFF, "FIFO mismatch"),
            ("end_fr", v.BUSY, "before idle"),
            ("end", 586, "before idle"),
            ("end", 646, "flush late"),
            ("busy_lo", 570, "bracket width"),
        ]:
            with self.subTest(field=field, value=value), self.assertRaisesRegex(ValueError, message):
                v.measure("tx", **{**TX, field: value})
        with self.assertRaisesRegex(ValueError, "stop-bit interval"):
            v.measure("tx", **{**TX, "busy_lo": 560, "busy_hi": 561})

    @patch.object(v, "TRIALS", 2)
    @patch.object(v, "STREAM", 60)
    def test_candidate_and_corrupted_capture(self):
        with tempfile.TemporaryDirectory() as tmp:
            directory = Path(tmp)
            uart0 = directory / "uart0"
            occupied = [32, 33]
            (directory / "peer.json").write_text(json.dumps({"run": NONCE, "prefill": occupied}))
            inbound, outbound = v.transcript(NONCE.encode(), occupied)
            (directory / "to-rp1.bin").write_bytes(inbound)
            (directory / "from-rp1.bin").write_bytes(outbound)
            log = capture()
            uart0.write_text(log)
            self.assertEqual(v.verify(uart0, directory)[0], NONCE)
            for before, after, rejection in [
                ("full_max_us=10", "full_max_us=11", "native try"),
                ("rx_stream_rt=1", "rx_stream_rt=7030", "RX stream"),
                ("tx_stream_tx=2", "tx_stream_tx=6251", "TX stream"),
                ("tx_stream_tx=2", "tx_stream_tx=0", "TX stream"),
                ("tx_stream_other=0", "tx_stream_other=1", "IRQ direction"),
                (f"run={NONCE} kind=rt i=0", "run=deadbeef kind=rt i=0", "sequence/nonce"),
                (FW, FW + "\n" + FW, "duplicate"),
            ]:
                uart0.write_text(log.replace(before, after))
                with self.subTest(rejection=rejection), self.assertRaisesRegex(ValueError, rejection):
                    v.verify(uart0, directory)
            uart0.write_text(log)
            (directory / "to-rp1.bin").write_bytes(inbound[1:])
            with self.assertRaisesRegex(ValueError, "sent transcript mismatch"):
                v.verify(uart0, directory)


if __name__ == "__main__":
    unittest.main()
