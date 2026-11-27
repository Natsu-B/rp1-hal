"""Fixtures for the ADC UART acceptance gate."""

from pathlib import Path
import runpy
import unittest

verify = runpy.run_path(str(Path(__file__).with_name("verify-adc-parity.py")))["verify"]

IRQ = "irq=1 stray=0 alarm=0 dropped=0"
NONE = "irq=0 stray=0 alarm=0 dropped=0"


def transcript():
    rows = ["ADC_PARITY_READY"]
    rows += [f"ADC_IRQ_SAMPLE i={i} raw=777 us=19 {IRQ}" for i in range(100)]
    rows += [f"ADC_LEGACY_SAMPLE i={i} raw=777 us=8" for i in range(100)]
    rows += [f"ADC_LEGACY_END trials=100 would_block=100 {NONE}"]
    rows += ["ADC_STRAY us=2 irq=0 stray=1 alarm=0 dropped=0 lowered=1"]
    for i in range(10):
        rows += [f"ADC_TIMEOUT i={i} us=1012 irq=0 stray=0 alarm=1 dropped=1",
                 f"ADC_REARM i={i} raw=778 us=19 {IRQ}"]
    rows += [f"ADC_OVERRUN injected=1 cleared=1 busy=1 {NONE}",
             f"ADC_ADMISSION expired=1 masked_irq=1 masked_poll=1 {NONE}",
             f"ADC_CANCEL raw=777 us=20 {IRQ} retained=1",
             f"ADC_REINIT raw=777 us=20 {IRQ}"]
    rows += [f"ADC_POLL i={i} raw=777 us=8 {NONE}" for i in range(100)]
    rows += ["ADC_PARITY_FIRMWARE_END result=PASS"]
    return "\n".join(rows) + "\n"


class VerifyAdc(unittest.TestCase):
    def test_complete_trace(self):
        result = verify(transcript())
        self.assertIn("trials=100 irq=100 unexpected_adc_irq=0", result)
        self.assertIn("PARITY_EH02_END driver=adc", result)

    def test_two_sided_timing_bounds(self):
        for original, replacement in [("us=19", "us=2"), ("us=19", "us=33"),
                                      ("us=1012", "us=999"), ("us=1012", "us=1017"),
                                      ("ADC_STRAY us=2", "ADC_STRAY us=6"),
                                      ("raw=777 us=8", "raw=777 us=2"),
                                      ("raw=777 us=8", "raw=777 us=11"),
                                      ("ADC_POLL i=0 raw=777 us=8", "ADC_POLL i=0 raw=777 us=11")]:
            with self.subTest(replacement=replacement), self.assertRaises(ValueError):
                verify(transcript().replace(original, replacement, 1))

    def test_spread_and_relative_limit(self):
        with self.assertRaisesRegex(ValueError, "spread"):
            verify(transcript().replace("us=19", "us=23", 1))
        with self.assertRaisesRegex(ValueError, "below polling min"):
            verify(transcript().replace("us=20", "us=6", 1))
        with self.assertRaisesRegex(ValueError, "exceeds polling max"):
            verify(transcript(), irq_slack_us=11)
        self.assertIn("irq_poll_delta_us=12", verify(transcript(), irq_slack_us=12))

    def test_count_and_recovery_failures(self):
        for original, replacement in [(IRQ, NONE), (IRQ, IRQ.replace("stray=0", "stray=1")),
                                      ("raw=777", "raw=4095"), ("cleared=1", "cleared=0"),
                                      ("retained=1", "retained=0"), ("alarm=1", "alarm=0"),
                                      ("ADC_REARM i=0", "ADC_REARM i=1")]:
            with self.subTest(replacement=replacement), self.assertRaises(ValueError):
                verify(transcript().replace(original, replacement, 1))

    def test_unexpected_markers(self):
        for appended in ("ADC_PARITY_READY\n", "RP1 panicked\n"):
            with self.subTest(appended=appended), self.assertRaises(ValueError):
                verify(transcript() + appended)


if __name__ == "__main__":
    unittest.main()
