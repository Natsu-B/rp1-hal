#!/usr/bin/env python3
"""Fixture logs for the dual-core ping acceptance verifier."""
import pathlib
import runpy
import subprocess
import sys
import tempfile
import unittest

VERIFIER = pathlib.Path(__file__).with_name("verify-ping.py")
verify = runpy.run_path(str(VERIFIER))["verify"]
A_ID = "12345678-1234-4321-9999-123456789abc"
B_ID = "12345678-1234-4321-9999-123456789abd"
BUNDLE = "123456789abcdef0"
CLIENT = "ab" * 32
UART0 = "PLAT_INIT ok=1 warm=0\nproc0 ready private=7\nproc1 ready private=7\n"
STATUS = """RP1_STATUS epoch=2 shm=Ready shm_generation=2 pcie=Up scmi=Serving(requests=17)
RP1_CORE core=0 state=Running reason=0
RP1_CORE core=1 state=Running reason=0
"""


def console(mode):
    irq = mode == "irq"
    boot = A_ID if irq else B_ID
    uios = ["link", "header", "ping-dn", "ping-up"] + (["irq", "doorbell"] if irq else [])
    return (
        "RPi: BOOTSYS release\n"
        f"PING_CHECK_BEGIN mode={mode} boot_id={boot} bundle={BUNDLE} client_sha256={CLIENT}\n"
        + "".join(f"PING_UIO rp1:{name} /dev/uio{i}\n" for i, name in enumerate(uios))
        + f"PING_READY mode={mode} generation=2 phase=before\n" + STATUS
        + f"PING_RTT mode={mode} n=5000 failed=0 p50_us=6 p99_us=12 max_us=700\n"
        + ("PING_IRQ before=8 after=91\n" if irq else "")
        + f"PING_LATE mode={mode} n=5000 injected=50 timed_out=50 replied=50 wrong=0 failed=0\n"
        + f"PING_READY mode={mode} generation=2 phase=after\n" + STATUS
        + ("PING_PEER_DOWN core=1 state=Panicked elapsed_us=15000\n"
           + STATUS.replace("core=1 state=Running", "core=1 state=Panicked") if irq else "")
        + f"PING_BOOT_END_ID={boot}\nPING_CHECK_RESULT=PASS mode={mode}\nPING_CHECK_END\nPING_CHECK_EXIT=0\n"
    )


class PingFixtures(unittest.TestCase):
    def test_both_modes_and_ansi(self):
        for mode, boot in [("irq", A_ID), ("poll", B_ID)]:
            raw = "".join(f"2026-11-26T10:05:00+0900 \x1b[?2004h\x1b[200~{line}\x1b[201~\x1b[?2004l\r\n"
                          for line in console(mode).splitlines())
            self.assertEqual(verify(UART0, raw, mode, BUNDLE, CLIENT, A_ID if mode == "poll" else None), boot)

    def test_capture_started_before_force_boot(self):
        shutdown = ("[  42.23] reboot: Power down\r\n"
                    "  0.98 RPi: BOOTSYS release\r\n"
                    "  1.00 power-on-reset 0\r\n"
                    "  1.12 Power off\r\n")
        raw = shutdown + "  0.99 " + console("irq").replace(
            "RPi: BOOTSYS release\n", "RPi: BOOTSYS release\r\n  1.01 power-on-reset 1\r\n", 1)
        self.assertEqual(verify(UART0, raw, "irq", BUNDLE, CLIENT), A_ID)

    def test_rejects_partial_failed_and_mixed_runs(self):
        valid = console("irq")
        for old, new in [
            ("PING_CHECK_END\n", ""), ("PING_CHECK_EXIT=0", "PING_CHECK_EXIT=1"),
            ("injected=50", "injected=49"), ("replied=50", "replied=49"), ("wrong=0", "wrong=1"),
            ("elapsed_us=15000", "elapsed_us=15001"),
            ("core=1 state=Panicked elapsed", "core=0 state=Panicked elapsed"),
            ("before=8 after=91", "before=8 after=8"),
            ("generation=2 phase=after", "generation=4 phase=after"),
            (f"PING_BOOT_END_ID={A_ID}", f"PING_BOOT_END_ID={B_ID}"),
            (BUNDLE, "0" * 16), (CLIENT, "0" * 64),
            ("PING_CHECK_END", "PING_CHECK_END\nOops: fault"),
            ("PING_CHECK_END", "PING_CHECK_END\n[rp1boot    234] FAIL: bad image"),
        ]:
            with self.subTest(new=new), self.assertRaises(ValueError):
                verify(UART0, valid.replace(old, new), "irq", BUNDLE, CLIENT)
        with self.assertRaises(ValueError):
            verify(UART0, valid + console("poll"), "irq", BUNDLE, CLIENT)
        for raw in [valid.replace("RPi: BOOTSYS release\n", ""),
                    valid.replace("PING_CHECK_EXIT=0", "RPi: BOOTSYS release\nPING_CHECK_EXIT=0"),
                    "PING_CHECK_EXIT=0\n" + valid,
                    valid + "RPi: BOOTSYS release\nPING_CHECK_EXIT=0\n"]:
            with self.subTest(raw=raw), self.assertRaises(ValueError):
                verify(UART0, raw, "irq", BUNDLE, CLIENT)
        with self.assertRaises(ValueError):
            verify(UART0, console("poll"), "poll", BUNDLE, CLIENT, B_ID)
        with self.assertRaises(ValueError):
            verify(UART0, console("poll") + "PING_PEER_DOWN core=1 state=Panicked elapsed_us=1\n",
                   "poll", BUNDLE, CLIENT, A_ID)

    def test_verifier_exit_status(self):
        with tempfile.TemporaryDirectory() as temp:
            uart0, uart10 = (pathlib.Path(temp) / name for name in ("uart0.log", "uart10.log"))
            uart0.write_text(UART0)
            args = [sys.executable, str(VERIFIER), "--uart0", str(uart0), "--uart10", str(uart10),
                    "--mode", "irq", "--bundle", BUNDLE, "--client-sha", CLIENT]
            for raw, rc, verdict in [(console("irq"), 0, "PASS"),
                                     (console("irq").replace("PING_CHECK_END", ""), 1, "FAIL")]:
                uart10.write_text(raw)
                result = subprocess.run(args, capture_output=True, text=True)
                self.assertEqual(result.returncode, rc, result.stderr)
                self.assertIn(f"PING_VERDICT={verdict}", result.stdout)


if __name__ == "__main__":
    unittest.main()
