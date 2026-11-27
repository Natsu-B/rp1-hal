import importlib.util
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location("motor", Path(__file__).with_name("verify-motor-api.py"))
motor = importlib.util.module_from_spec(spec)
spec.loader.exec_module(motor)


def capture():
    nonce = "3badbeef"
    rows = [f"PWM_BOOT nonce={nonce} PLAT_INIT ok=1",
            f"MOTOR_API_START run={nonce} updates=100000 range=50000",
            f"MOTOR_API_CLOCK run={nonce} before=3,50,840,1,0 after=3,50,840,1,0",
            f"MOTOR_API_CONVENTION run={nonce} cycles=200000 span_us=10000000"
            " min_period_us=49 max_period_us=51 shared=1 stopped=1"]

    def waveform(phase, duty, start):
        for i in range(100):
            rows.append(f"MOTOR_API_SAMPLE run={nonce} phase={phase} duty={duty} i={i}"
                        f" period_us=1000 high_us={duty * 10} t_us={start + i * 1000}")

    for plateau, duty in enumerate((25, 100, 50, 0, 75)):
        start = plateau * 20_000_000 + 5_000
        if duty in (0, 100):
            rows.append(f"MOTOR_API_LEVEL run={nonce} duty={duty} samples=50000"
                        f" wrong=0 us=100000 max_gap_us=10 t_us={start}")
        else:
            waveform("duty", duty, start)
    waveform("invalid", 50, -200_000)
    rows.append(f"MOTOR_API_SWEEP run={nonce} cycles=1050 low25=525 high75=525 other=0"
                " min_period_us=990 max_period_us=1010 start_us=100000100 end_us=101050100")
    rows.append(f"MOTOR_API_END run={nonce} updates=100000 sweep_updates=1100 missed=0 errors=0 arm_failed=0"
                " max_late_us=4 max_call_us=9 max_call_upper_us=10 invalid=100 invalid_native=100"
                " register_changes=0 elapsed_us=111111000 stopped=1 result=PASS")
    return "\n".join(rows)


class VerifyMotor(unittest.TestCase):
    def test_full_capture(self):
        self.assertEqual(motor.verify(capture()), ("3badbeef", True))

    def test_clock_source_unverified_does_not_fail_other_gates(self):
        changed = capture().replace("3,50,840,1,0", "3,49,840,1,0")
        changed = changed.replace("shared=1", "shared=0").replace("span_us=10000000", "span_us=10004000")
        self.assertEqual(motor.verify(changed), ("3badbeef", False))

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
            ("samples=50000", "samples=1"),
            ("max_gap_us=10", "max_gap_us=11"),
            ("t_us=20005000", "t_us=20004999"),
            ("t_us=5000", "t_us=4999"),
            ("cycles=1050", "cycles=999"),
            ("low25=525 high75=525", "low25=399 high75=651"),
            ("high75=525 other=0", "high75=524 other=1"),
            ("min_period_us=990", "min_period_us=989"),
            ("max_period_us=1010", "max_period_us=1011"),
            ("span_us=10000000", "span_us=10000101"),
            ("span_us=10000000", "span_us=10004000"),
            ("shared=1", "shared=0"),
            ("invalid=100", "invalid=99"),
            ("max_call_us=9 max_call_upper_us=10", "max_call_us=10 max_call_upper_us=11"),
            ("elapsed_us=111111000", "elapsed_us=120000001"),
        ]:
            with self.subTest(after=after), self.assertRaises(ValueError):
                motor.verify(good.replace(before, after, 1))


if __name__ == "__main__":
    unittest.main()
