#!/usr/bin/env python3
"""Fixtures for the independent SPI wire and firmware verifier."""

from pathlib import Path
import runpy
import unittest

verify = runpy.run_path(str(Path(__file__).with_name("verify-spi-parity.py")))


def uart():
    yield "SPI_PARITY_READY"
    yield "SPI_IRQ_RATE hz=1000000 frames=100000 trials=100 depth=16 rx_irq=101 stray=0 errors=0"
    yield "SPI_IRQ_RATE hz=10000000 frames=100000 trials=100 depth=16 rx_irq=102 stray=0 errors=0"
    for hz, elapsed, late in [(1_000_000, 134, 10), (100_000, 1296, 29)]:
        for i in range(100):
            yield (f"SPI_TX_SAMPLE hz={hz} trial={i} flush_us={elapsed} launch_us={elapsed + 1} "
                   f"late_us={late} tx_irq=1 tail_alarms=1 idle=1")
        yield (f"SPI_TX_TRIALS hz={hz} tx_trials=100 depth=16 tx_irq=100 tail_trials=100 "
               f"flush_max_us={elapsed} flush_late_max_us={late} stray=0 errors=0")
    yield "SPI_IRQ_TRIALS timeout=100 timeout_max_us=510 rearm=100 tx_irq=0 stray=0 errors=0"
    for i in range(100):
        yield f"SPI_FAULT_SAMPLE trial={i} inject_us=251 release_us=260 late_us=10 fault_irq=1 released=1"
    yield ("SPI_BUS_TRIALS cs0=1000 cs1=1000 forced_error=100 recovered=100 cs_delay_min_us=6 "
           "cs_delay_upper_us=20 fault_release_min_us=260 fault_release_max_us=260 "
           "fault_late_max_us=10 bad_cs=0 stray=0 errors=0")
    yield "SPI_POLL_COMPARISON hz=1000000 frames=32 irq_us=264 polling_us=260 stray=0 errors=0"
    yield "SPI_PARITY_FIRMWARE_END result=PASS"


LOG = "\n".join(uart()) + "\n"


def wire(first_flush=50, cs_delay=5, fault_release=250, rate=100_000_000, bad_duplex=False):
    yield f"# sample_hz={rate}"
    yield "us,sck,mosi,miso,cs0,cs1,gate"
    us = 1
    yield f"{us},0,0,0,1,1,0"

    def section(cs0, cs1, gate, period, payload, flush, skip=0):
        nonlocal us
        us += 1
        yield f"{us:.3f},{int(skip > 0)},0,0,{cs0},{cs1},{gate}"
        if skip:
            us += period / 2
            yield f"{us:.3f},0,0,0,{cs0},{cs1},{gate}"
        for j, value in enumerate(payload):
            if not (cs0 and cs1) and j == 4:
                us += cs_delay - period / 2
            for bit in range(7 - skip, -1, -1):
                level = (value >> bit) & 1
                other = 1 - level if bad_duplex else level
                us += period / 2
                yield f"{us:.3f},1,{level},{other},{cs0},{cs1},{gate}"
                us += period / 2
                yield f"{us:.3f},0,{level},{level},{cs0},{cs1},{gate}"
            skip = 0
        us += flush
        yield f"{us:.3f},0,0,0,1,1,0"

    for i in range(200):
        period = 1 if i < 100 else 0.1
        payload = bytes(((j * 37) & 0xff) ^ (i % 100) for j in range(1_000))
        yield from section(1, 1, 1, period, payload, 1)
    for i in range(200):
        period = 1 if i < 100 else 10
        yield from section(1, 1, 1, period, bytes(16), first_flush if i == 0 else 1, skip=3)
    for i in range(2_000):
        cs = i % 2
        payload = bytes([0x5a, cs]) + (i // 2).to_bytes(2, "little") + bytes(4)
        yield from section(cs, 1 - cs, 0, 1, payload, 1)
    for _ in range(100):
        yield from section(0, 1, 0, 1, b"", fault_release)


class SpiParityTest(unittest.TestCase):
    def test_uart_counts_order_and_timing(self):
        self.assertEqual(verify["verify_uart"](LOG)[:2], (16, [101, 102]))
        for altered in [
            LOG.replace("rx_irq=101", "rx_irq=99"),
            LOG.replace("rx_irq=101", "rx_irq=12701"),
            LOG.replace("rx_irq=101 stray=0", "rx_irq=101 stray=1"),
            LOG.replace("tx_irq=100 tail_trials", "tx_irq=101 tail_trials"),
            LOG.replace("launch_us=135 late_us=10", "launch_us=226 late_us=101"),
            LOG.replace("launch_us=135 late_us=10", "launch_us=125 late_us=0"),
            LOG.replace("release_us=260 late_us=10", "release_us=100000 late_us=99750"),
            LOG.replace("release_us=260 late_us=10", "release_us=199 late_us=0"),
            LOG.replace("cs_delay_upper_us=20", "cs_delay_upper_us=105"),
            LOG.replace("timeout_max_us=510", "timeout_max_us=601"),
            LOG.replace("frames=100000", "frames=99999"),
            LOG + "SPI_PARITY_READY\n",
        ]:
            with self.assertRaises(ValueError):
                verify["verify_uart"](altered)

    def test_complete_wire_and_flush_bound(self):
        self.assertEqual(verify["verify_wire"](wire(), 16), (1, 50))
        with self.assertRaisesRegex(ValueError, "wire completion"):
            verify["verify_wire"](wire(first_flush=100.001), 16)

    def test_cs_delay_and_forced_release_bounds(self):
        with self.assertRaisesRegex(ValueError, "CS delay"):
            verify["verify_wire"](wire(cs_delay=104.5), 16)
        with self.assertRaisesRegex(ValueError, "forced error"):
            verify["verify_wire"](wire(fault_release=100_000), 16)

    def test_reject_overlap_rate_and_data_mismatch(self):
        with self.assertRaisesRegex(ValueError, "simultaneously active"):
            verify["verify_wire"]([
                "# sample_hz=100000000", "us,sck,mosi,miso,cs0,cs1,gate",
                "1,0,0,0,1,1,0", "2,0,0,0,0,0,0",
            ], 16)
        with self.assertRaisesRegex(ValueError, "MOSI/MISO"):
            verify["verify_wire"](wire(bad_duplex=True), 16)
        with self.assertRaisesRegex(ValueError, "sample rate"):
            verify["verify_wire"](wire(rate=99_999_999), 16)


if __name__ == "__main__":
    unittest.main()
