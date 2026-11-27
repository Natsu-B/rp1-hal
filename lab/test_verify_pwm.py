"""Fixtures for the PWM pull and waveform verifier."""
from pathlib import Path
import runpy
import unittest

verify = runpy.run_path(str(Path(__file__).with_name("verify-pwm.py")))["verify"]


def uart0():
    rows = [
        "PWM_BOOT nonce=3badbeef PLAT_INIT ok=1",
        "PWM_PAD nonce=3badbeef pin=22 pull=Up raw=0xca",
        "PWM_PAD nonce=3badbeef pin=22 pull=Down raw=0xc6",
        "PWM_PAD nonce=3badbeef pin=23 pull=Down raw=0xc6",
    ]
    for n in range(60):
        for duty in (0, 25, 50, 75, 100):
            rise = 100 if 0 < duty < 100 else 0
            rows.append(
                f"PWM_SAMPLE round={n} duty={duty} a_rise={rise} a_high={duty} a_total=100 a_us=100000"
                " b_rise=50 b_high=50 b_total=100 b_us=100000"
            )
        rows.extend([
            f"PWM_REPERIOD round={n} rise=50 high=25 total=100 us=100000",
            f"PWM_DISABLED round={n} rise=0 high=0 total=100",
        ])
    rows.append("PWM_REINIT a_rise=100 a_high=50 a_total=100 a_us=100000 "
                "b_rise=50 b_high=50 b_total=100 b_us=100000")
    return "\n".join([*rows, "PWM_DONE reinitialized=1", ""])


class TestPwm(unittest.TestCase):
    def test_success_with_timestamp_and_ansi(self):
        log = "".join(f"2026-11-27T03:00:00 \x1b[?2004h{line}\x1b[?2004l\r\n"
                      for line in uart0().splitlines())
        self.assertIsNone(verify(log))

    def test_missing_or_misordered_marker(self):
        good = uart0()
        lines = good.splitlines()
        self.assertIn("count or order", verify(good.replace(lines[1] + "\n", "")))
        lines[3], lines[4] = lines[4], lines[3]
        self.assertIn("count or order", verify("\n".join(lines)))

    def test_pad_mask(self):
        for raw in ("0xc6", "0x4a"):
            self.assertIn("invalid PWM_PAD", verify(uart0().replace("raw=0xca", f"raw={raw}")))

    def test_mixed_boot(self):
        good = uart0()
        self.assertIn("invalid PWM_PAD", verify(good.replace("nonce=3badbeef pin=23", "nonce=12345678 pin=23")))
        self.assertIn("count or order", verify(good + "PWM_BOOT nonce=12345678 PLAT_INIT ok=1\n"))

    def test_invalid_waveform(self):
        good = uart0()
        for valid, bad in (("a_rise=100", "a_rise=106"), ("b_high=50", "b_high=57"),
                           ("rise=0 high=0 total=100\n", "rise=0 high=100 total=100\n")):
            self.assertIn("waveform", verify(good.replace(valid, bad, 1)))


if __name__ == "__main__":
    unittest.main()
