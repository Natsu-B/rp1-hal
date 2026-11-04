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
[t] PROFILE skipped
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
UART0 = """[t] I2C_PASS hz=100000 write16_us=1
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

    def test_profile_is_explicit_and_requires_completion(self):
        profiled = UART10.replace("PROFILE skipped", "PROFILE rp1_us=99 (results on UART0)")
        done = UART0 + "[t] PROF_DONE\n[t] proc1 shm cmd=9 count=0 errors=0 us=99\n"
        self.assertEqual(v.verify(done, profiled, DONE, profile=True), [])
        self.assertTrue(v.verify(UART0, profiled, DONE, profile=True))
        self.assertTrue(v.verify(done.replace("PROF_DONE", ""), profiled, DONE, profile=True))
        self.assertTrue(v.verify(done.replace("errors=0", "errors=1"), profiled, DONE, profile=True))
        self.assertTrue(v.verify(done, UART10, DONE, profile=True))
        self.assertTrue(v.verify(done, profiled, DONE))
        self.assertTrue(v.verify(UART0, UART10.replace("PROFILE skipped", ""), DONE))

    def test_incomplete_capture_fails(self):
        self.assertTrue(v.verify(UART0, UART10, {"complete": False}))
        self.assertTrue(v.verify(UART0, UART10, {}))
        self.assertTrue(v.verify(UART0, UART10, None))

    def test_review3_stall_fails(self):
        # Review 3 #2: both UARTs stop inside command 9; boot markers were all present.
        uart10 = "[t] DOORBELL_BEFORE_RC=0\n[t] ISOLATED up_push_batch n=256 rp1_us=293\n"
        uart0 = "[t] PROF ring_data_ddr single push n=128 cyc=1 pop cyc=2\n"
        failures = v.verify(uart0, uart10, {"complete": False}, profile=True)
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



NONCE = "12536739b847"
CONSOLE = f"""[t] UART1_CHECK_BEGIN
[t] UART1_DEV=/dev/ttyAMA1
[t] UART1_TX_SENT nonce={NONCE} lines=10
[t] UART1_RX_BYTES=270
[t] UART1_RX_GOT=RP1SHARE_RX_{NONCE}
[t] UART1_CHECK_DONE
[t] DDR_INSPECT_END
"""
HOST = "".join(f"RP1SHARE_TX_{NONCE}_{i}\n" for i in range(10))


class Uart1(unittest.TestCase):
    def test_both_directions_pass(self):
        self.assertEqual(v.verify_uart1(CONSOLE, HOST, NONCE, DONE), ([], []))
        self.assertEqual(v.verify_uart1(CONSOLE, HOST, NONCE), ([], []))

    def test_directions_are_judged_separately(self):
        tx, rx = v.verify_uart1(CONSOLE, "", NONCE, DONE)
        self.assertTrue(tx)
        self.assertEqual(rx, [])
        tx, rx = v.verify_uart1(CONSOLE.replace(f"GOT=RP1SHARE_RX_{NONCE}", "GOT="), HOST, NONCE, DONE)
        self.assertEqual(tx, [])
        self.assertTrue(rx)

    def test_other_nonce_or_partial_token_fails(self):
        other = "0123456789ab"
        self.assertTrue(all(v.verify_uart1(CONSOLE, HOST, other, DONE)))
        for got in (f"RP1SHARE_RX_{NONCE[:-1]}", f"RP1SHARE_RX_{NONCE}0", f"RP1SHARE_TX_{NONCE}"):
            _, rx = v.verify_uart1(CONSOLE.replace(f"RP1SHARE_RX_{NONCE}", got), HOST, NONCE, DONE)
            self.assertTrue(rx, got)
        # The host must have received the lines, not merely Linux claiming it sent them.
        tx, _ = v.verify_uart1(CONSOLE, HOST.replace(NONCE, other), NONCE, DONE)
        self.assertTrue(tx)
        tx, _ = v.verify_uart1(CONSOLE.replace("UART1_TX_SENT", "UART1_TX_SKIPPED"), HOST, NONCE, DONE)
        self.assertTrue(tx)

    def test_incomplete_or_bad_nonce_fails_both(self):
        for console, nonce, result in ((CONSOLE.replace("UART1_CHECK_DONE", "X"), NONCE, DONE),
                                       (CONSOLE, NONCE, {"complete": False}),
                                       (CONSOLE.replace(NONCE, "1234"), "1234", DONE),
                                       (CONSOLE, "", DONE)):
            tx, rx = v.verify_uart1(console, HOST, nonce, result)
            self.assertTrue(tx and rx, (nonce, result))

    def test_cli_prints_markers(self):
        import contextlib, io, tempfile
        with tempfile.TemporaryDirectory() as d:
            c, h = pathlib.Path(d, "c"), pathlib.Path(d, "h")
            c.write_text(CONSOLE)
            h.write_text(HOST)
            out = io.StringIO()
            with contextlib.redirect_stdout(out):
                rc = v.main(["--uart1-host", str(h), "--uart1-nonce", NONCE, "--uart1-console", str(c)])
            self.assertEqual(rc, 0)
            self.assertIn("UART1_TX_PASS", out.getvalue())
            self.assertIn("UART1_RX_PASS", out.getvalue())
            h.write_text("")
            with contextlib.redirect_stdout(io.StringIO()) as out:
                rc = v.main(["--uart1-host", str(h), "--uart1-nonce", NONCE, "--uart1-console", str(c)])
            self.assertEqual(rc, 1)
            self.assertIn("UART1_TX_FAIL", out.getvalue())
            self.assertIn("UART1_RX_PASS", out.getvalue())


if __name__ == "__main__":
    unittest.main()
