#!/usr/bin/env python3
"""Host tests for verify-shm.py: `python3 lab/test_verify_shm.py`."""
import importlib.util
import pathlib
import unittest

spec = importlib.util.spec_from_file_location(
    "verify_shm", pathlib.Path(__file__).with_name("verify-shm.py"))
v = importlib.util.module_from_spec(spec)
spec.loader.exec_module(v)

UART10 = """[t] DOORBELL_BEFORE_RC=0
[t] RING up_batch n=1000000 missing=0 corrupt=0
[t] PROFILE rp1_us=1234 (results on UART0)
[t] SHM_PASS
[t] SHM_RC=0
[t] DOORBELL_AFTER_RC=0
[t] I2C_REPLY=0x00000001
[t] I2C_LINUX_DONE
[t] ADC_REPLY=0x00000ADC
[t] ADC_LINUX_DONE
[t] CHECK_SHM_RESULT=PASS
[t] DDR_INSPECT_END
"""
UART0 = """[t] PROF ring_data_ddr batch push n=128 cyc=1 pop cyc=2
[t] PROF_DONE
[t] proc1 shm cmd=9 count=0 errors=0 us=99
[t] I2C_PASS hz=100000 write16_us=1
[t] I2C_PASS hz=400000 write16_us=1
[t] I2C_PASS r64_w16 hz=400000
[t] I2C_RESULT request=1 Ok(())
[t] ADC_P1BOOT first=791 temp_mc=70579 PASS
[t] ADC_RESULT boot Ok(())
[t] ADC_P1REQ first=778 temp_mc=75809 PASS
[t] ADC_RESULT request=0xadc Ok(())
"""
DONE = {"complete": True}


class Verify(unittest.TestCase):
    def test_full_run_passes(self):
        self.assertEqual(v.verify(UART0, UART10, DONE), [])

    def test_incomplete_capture_fails(self):
        self.assertTrue(v.verify(UART0, UART10, {"complete": False}))
        self.assertTrue(v.verify(UART0, UART10, {}))
        self.assertTrue(v.verify(UART0, UART10, None))

    def test_review3_stall_fails(self):
        # Review 3 #2: both UARTs stop inside command 9; boot markers were all present.
        uart10 = "[t] DOORBELL_BEFORE_RC=0\n[t] ISOLATED up_push_batch n=256 rp1_us=293\n"
        uart0 = "[t] PROF ring_data_ddr single push n=128 cyc=1 pop cyc=2\n"
        failures = v.verify(uart0, uart10, {"complete": False})
        for marker in ("SHM_PASS", "SHM_RC=0", "terminal marker", "I2C reply",
                       "profile done", "I2C 100 kHz"):
            self.assertTrue(any(marker in f for f in failures), (marker, failures))

    def test_each_required_marker_is_needed(self):
        for text, required in ((UART10, v.UART10_ORDERED), (UART0, v.UART0_REQUIRED)):
            for name, pattern in required:
                line = next(l for l in text.splitlines() if v.re.search(pattern, l))
                cut = text.replace(line + "\n", "")
                args = (UART0, cut) if text is UART10 else (cut, UART10)
                self.assertTrue(v.verify(*args, DONE), name)

    def test_nonzero_exit_and_failure_markers_fail(self):
        for bad in ("SHM_RC=1", "SHM_RC=137", "SHM_RC=01"):
            self.assertTrue(v.verify(UART0, UART10.replace("SHM_RC=0", bad), DONE), bad)
        self.assertTrue(v.verify(UART0, UART10.replace("I2C_REPLY=0x00000001", "I2C_REPLY=0xFFFFFFFF"), DONE))
        self.assertTrue(v.verify(UART0, UART10.replace("PASS\n[t] DDR", "FAIL shm\n[t] DDR"), DONE))
        self.assertTrue(v.verify(UART0, "[t] CHECK_FAIL shm\n" + UART10, DONE))
        self.assertTrue(v.verify(UART0, UART10.replace("ADC_REPLY=0x00000ADC", "ADC_REPLY=0x00000000"), DONE))
        self.assertTrue(v.verify(UART0 + "[t] ADC_RESULT request=0xadc Err(Timeout)\n", UART10, DONE))
        self.assertTrue(v.verify(UART0 + "[t] ADC_RESULT boot Err(Precondition)\n", UART10, DONE))
        self.assertTrue(v.verify(UART0 + "[t] I2C_FAIL Bus(\"read\", Timeout)\n", UART10, DONE))

    def test_order_matters(self):
        swapped = UART10.replace("[t] SHM_PASS\n", "").replace("[t] DDR_INSPECT_END\n", "[t] DDR_INSPECT_END\n[t] SHM_PASS\n")
        self.assertTrue(any("out of order" in f for f in v.verify(UART0, swapped, DONE)))


if __name__ == "__main__":
    unittest.main()
