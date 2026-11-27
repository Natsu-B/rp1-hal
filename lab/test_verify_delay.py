import runpy
import unittest
from pathlib import Path

verify = runpy.run_path(str(Path(__file__).with_name("verify-delay.py")))["verify"]
BOOT = "PARITY_DELAY_BOOT nonce=123456ab PLAT_INIT ok=1 sys_hz=200000000\n"


def phase_samples(phase):
    return f"PARITY_DELAY_PHASE nonce=123456ab phase={phase}\n" + "".join(
        f"PARITY_DELAY_SAMPLE nonce=123456ab phase={phase} request_us={us} n=100 "
        f"under=0 over_limit=0 min_us={us + 1} max_us={us + 22} "
        f"wait_wakes={100 if us >= 100 else 0} wake_misses=0 "
        f"isr_spans={int(phase == 'control' and us == 1000)}\n"
        for us in (1, 10, 100, 1000, 10000)
    )


CONTROL = phase_samples("control")
ISOLATED = phase_samples("isolated")
END = (
    "PARITY_DELAY_END nonce=123456ab settings=5 samples=1000 under=0 over_limit=0 "
    "wait_wakes=600 wake_misses=0 isr_spans=1 period_us=1000 "
    "ticks=1500 during=1100 missed=0 max_late_us=50 arm_failed=0 ok=1\n"
)
GOOD = BOOT + CONTROL + ISOLATED + END


class DelayTest(unittest.TestCase):
    def test_bound(self):
        self.assertEqual(verify(GOOD), ("123456ab", 50, 600, 1))

    def test_reject_bad_capture(self):
        for bad in [
            CONTROL + ISOLATED + END,
            BOOT + CONTROL + END,
            GOOD + END,
            GOOD.replace("request_us=10 n=100", "request_us=10 n=99", 1),
            GOOD.replace("min_us=11 ", "min_us=10 ", 1),
            GOOD.replace("max_us=32 ", "max_us=33 ", 1),
            GOOD.replace("over_limit=0", "over_limit=1", 1),
            GOOD.replace("wait_wakes=100", "wait_wakes=99", 1),
            GOOD.replace("wait_wakes=100", "wait_wakes=101", 1),
            GOOD.replace("wake_misses=0", "wake_misses=1", 1),
            GOOD.replace("wait_wakes=600", "wait_wakes=599"),
            GOOD.replace("isr_spans=1 period", "isr_spans=0 period"),
            GOOD.replace(ISOLATED, ISOLATED.replace("isr_spans=0", "isr_spans=1", 1)),
            GOOD.replace("max_late_us=50", "max_late_us=51"),
            GOOD.replace("during=1100", "during=999"),
            GOOD.replace("missed=0", "missed=1"),
            GOOD.replace("arm_failed=0", "arm_failed=1"),
            GOOD.replace(END, END.replace("123456ab", "123456ac")),
        ]:
            with self.subTest(bad=bad), self.assertRaises(ValueError):
                verify(bad)


if __name__ == "__main__":
    unittest.main()
