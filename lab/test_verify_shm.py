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
[t] ADC_DT=/sys/firmware/devicetree/base/axi/pcie@1000120000/rp1/adc@c8000 status=disabled
[t] ADC_REPLY=0x00000ADC
[t] ADC_LINUX_DONE
[t] SPI_DT=/sys/firmware/devicetree/base/axi/pcie@1000120000/rp1/spi@50000 status=disabled
[t] SPI_REPLY=0x000005B1
[t] SPI_LINUX_DONE
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
[t] SPI_P1BOOT depth=64 dfs32=true loopback_off=zeros u8=ok in_place=ok transaction=ok PASS
[t] SPI_RESULT boot Ok
[t] SPI_PASS P1REQ hz=1000000 div=200 bits=8 frames=1024 us=8300 kbps=987 wire_pct=98
[t] SPI_PASS P1REQ hz=10000000 div=20 bits=8 frames=1024 us=900 kbps=9102 wire_pct=91
[t] SPI_P1REQ depth=64 dfs32=true loopback_off=zeros u8=ok in_place=ok transaction=ok PASS
[t] SPI_RESULT request=0x5b1 Ok
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
        self.assertTrue(v.verify(UART0, UART10.replace("SPI_REPLY=0x000005B1", "SPI_REPLY=0xFFFFFFFF"), DONE))
        self.assertTrue(v.verify(UART0 + "[t] SPI_RESULT request=0x5b1 Err data hz=1000000 bits=8 code=0x3\n", UART10, DONE))
        self.assertTrue(v.verify(UART0 + "[t] proc1 spi0 init Err 0x102\n", UART10, DONE))
        # 100 MHz must not satisfy the 10 MHz marker.
        self.assertTrue(v.verify(UART0.replace("hz=10000000 ", "hz=100000000 "), UART10, DONE))
        self.assertTrue(v.verify(UART0 + "[t] I2C_FAIL Bus(\"read\", Timeout)\n", UART10, DONE))

    def test_linux_nodes_of_owned_blocks_must_be_disabled(self):
        # Review 6 #4: an enabled, status-less or missing node is a failure.
        for block in ("ADC", "SPI"):
            line = next(l for l in UART10.splitlines() if f"{block}_DT=" in l)
            for bad in (line.replace("status=disabled", "status=okay"),
                        line.replace("status=disabled", "status=none"),
                        line.replace("status=disabled", "status=disabledx"),
                        f"[t] {block}_DT=missing"):
                self.assertTrue(v.verify(UART0, UART10.replace(line, bad), DONE), bad)
                # Also when a second, disabled node line is present.
                self.assertTrue(v.verify(UART0, UART10.replace(line, line + "\n" + bad), DONE), bad)

    def test_order_matters(self):
        swapped = UART10.replace("[t] SHM_PASS\n", "").replace("[t] DDR_INSPECT_END\n", "[t] DDR_INSPECT_END\n[t] SHM_PASS\n")
        self.assertTrue(any("out of order" in f for f in v.verify(UART0, swapped, DONE)))


if __name__ == "__main__":
    unittest.main()
