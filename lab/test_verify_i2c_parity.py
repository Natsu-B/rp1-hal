#!/usr/bin/env python3
"""Host fixtures for the fixed timing, IRQ and peer checks."""

import runpy
import unittest
from pathlib import Path

verify = runpy.run_path(str(Path(__file__).with_name("verify-i2c-parity.py")))["verify"]

BEFORE = "i2c_addr=0x2d rx=1 bytes=16 bad_frames=0 reads=2 preload_errors=0\nOK\n"
AFTER = "i2c_addr=0x2d rx=3101 bytes=49616 bad_frames=0 reads=3304 preload_errors=0\nOK\n"
RAW = """\x00I2C_PEER_READY addr=0x2d bytes=64
I2C_PHASE kind=combined hz=100000 count=1000 ok=1000 idle=1000 min_us=7000 max_us=10000 irq_bad=0 rx=1000 tx=0 stop=1000 abort=0 stray=0 other=0
I2C_PHASE kind=combined hz=400000 count=1000 ok=1000 idle=1000 min_us=1750 max_us=10000 irq_bad=0 rx=64000 tx=17000 stop=0 abort=0 stray=0 other=0
I2C_PHASE kind=stop hz=100000 count=100 ok=100 idle=100 min_us=1500 max_us=2000 irq_bad=0 rx=0 tx=0 stop=100 abort=0 stray=0 other=0
I2C_PHASE kind=nack hz=100000 count=100 ok=100 idle=100 min_us=75 max_us=2000 irq_bad=0 rx=0 tx=0 stop=0 abort=100 stray=0 other=0
I2C_REARM kind=nack count=100
I2C_PHASE kind=timeout hz=100000 count=100 ok=100 idle=0 min_us=2000 max_us=3500 irq_bad=0 rx=0 tx=0 stop=0 abort=0 stray=0 other=0
I2C_REARM kind=timeout count=100 held_low=100 rejected=100 recovered=100
I2C_PHASE kind=combined hz=1000000 count=1000 ok=1000 idle=1000 min_us=700 max_us=10000 irq_bad=0 rx=1000 tx=0 stop=0 abort=0 stray=0 other=0
I2C_FIXTURE_END stray=0 other=0 result=Ok(())
"""


class TestGate(unittest.TestCase):
    def test_all_fixed_bounds(self):
        self.assertEqual(verify(RAW, BEFORE, AFTER)[1], [])

    def test_fails_in_both_directions(self):
        for old, new in [
            ("min_us=7000", "min_us=6999"),
            ("max_us=10000", "max_us=10001"),
            ("min_us=2000", "min_us=1999"),
            ("max_us=3500", "max_us=3501"),
            ("rx=64000", "rx=64001"),
            ("rx=1000", "rx=999"),
            ("tx=17000", "tx=17001"),
            ("abort=100", "abort=101"),
            ("stray=0", "stray=1"),
            ("irq_bad=0", "irq_bad=1"),
            ("held_low=100", "held_low=99"),
            ("rejected=100", "rejected=99"),
            ("recovered=100", "recovered=99"),
            ("I2C_REARM kind=nack count=100", ""),
        ]:
            with self.subTest(old=old, new=new):
                self.assertTrue(verify(RAW.replace(old, new, 1), BEFORE, AFTER)[1])

    def test_peer_counters_and_markers_are_mandatory(self):
        self.assertTrue(verify(RAW, BEFORE, AFTER.replace("49616", "49600"))[1])
        self.assertTrue(verify(RAW, BEFORE, AFTER.replace("bad_frames=0", "bad_frames=1"))[1])
        self.assertTrue(verify(RAW + RAW, BEFORE, AFTER)[1])
        self.assertTrue(verify(RAW, BEFORE, AFTER.replace("\nOK", ""))[1])


if __name__ == "__main__":
    unittest.main()
