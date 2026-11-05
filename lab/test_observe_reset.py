#!/usr/bin/env python3
"""Check telemetry wrapping without opening hardware."""
import importlib.util
from pathlib import Path
import subprocess
import unittest

spec = importlib.util.spec_from_file_location('observer', Path(__file__).with_name('observe-reset.py'))
observer = importlib.util.module_from_spec(spec)
spec.loader.exec_module(observer)


class PayloadTest(unittest.TestCase):
    def test_outer_end_follows_telemetry_shutdown(self):
        payload = observer.payload_for(b'echo RINGDMA_FIX_END\necho DDR_INSPECT_END\n')
        self.assertEqual(payload.count(b'echo DDR_INSPECT_END'), 1)
        self.assertLess(payload.index(b'echo RINGDMA_FIX_END'), payload.rindex(b'reset_telemetry_stop'))
        self.assertTrue(payload.endswith(b'reset_telemetry_stop\necho DDR_INSPECT_END\n'))
        subprocess.run(['sh', '-n'], input=payload, check=True)


if __name__ == '__main__':
    unittest.main()
