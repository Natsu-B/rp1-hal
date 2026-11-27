import importlib.util
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location("motor", Path(__file__).with_name("verify-motor-api.py"))
motor = importlib.util.module_from_spec(spec)
spec.loader.exec_module(motor)


def capture():
    nonce = "3badbeef"
    rows = [f"PWM_BOOT nonce={nonce} PLAT_INIT ok=1",
            f"MOTOR_API_START run={nonce} updates=100000 range=50000"]
    for phase, duty in (("duty", 25), ("duty", 50), ("duty", 75), ("invalid", 50)):
        for i in range(100):
            rows.append(f"MOTOR_API_SAMPLE run={nonce} phase={phase} duty={duty} i={i}"
                        f" period_us=1000 high_us={duty * 10}")
    for duty in (0, 100):
        rows.append(f"MOTOR_API_LEVEL run={nonce} duty={duty} samples=50000 wrong=0 us=100000")
    rows.append(f"MOTOR_API_END run={nonce} updates=100000 missed=0 errors=0 arm_failed=0"
                " max_late_us=4 max_call_us=9 max_call_upper_us=10 invalid=100 invalid_native=100"
                " register_changes=0 elapsed_us=100000001 stopped=1 result=PASS")
    return "\n".join(rows)


class VerifyMotor(unittest.TestCase):
    def test_full_capture(self):
        self.assertEqual(motor.verify(capture()), "3badbeef")

    def test_missing_and_duplicate_markers(self):
        good = capture()
        with self.assertRaises(ValueError):
            motor.verify(good.rsplit("\n", 1)[0])
        with self.assertRaises(ValueError):
            motor.verify(good + "\n" + good.splitlines()[-1])

    def test_rejects_wrong_nonce_and_out_of_range_evidence(self):
        good = capture()
        for before, after in [
            ("run=3badbeef", "run=3badbee0"),
            ("period_us=1000", "period_us=1011"),
            ("period_us=1000", "period_us=989"),
            ("high_us=250", "high_us=261"),
            ("high_us=250", "high_us=239"),
            ("wrong=0", "wrong=1"),
            ("invalid=100", "invalid=99"),
            ("max_call_us=9 max_call_upper_us=10", "max_call_us=10 max_call_upper_us=11"),
            ("elapsed_us=100000001", "elapsed_us=120000001"),
        ]:
            with self.subTest(after=after), self.assertRaises(ValueError):
                motor.verify(good.replace(before, after, 1))


if __name__ == "__main__":
    unittest.main()
