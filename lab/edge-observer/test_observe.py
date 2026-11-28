import math
import unittest

from observe import Observer, WRAP, fit, series, spi_frame


class ObserveTest(unittest.TestCase):
    def test_marker_wrap_and_scale(self):
        origin = WRAP - 8000
        before = WRAP - 100
        elapsed = [0, 2000, 4000, 2_002_000, 2_004_000]
        ticks = [(origin + round(us * 80 * 1.00005)) % WRAP for us in elapsed]
        brackets = [((before + us) % WRAP, (before + us) % WRAP, 0) for us in elapsed]
        result = fit(ticks, brackets)
        self.assertTrue(math.isclose(result["esp_vs_rp1_ppm"], 50, abs_tol=0.01))
        self.assertLess(result["esp_vs_rp1_ppm_bounds"][0], 50)
        self.assertGreater(result["esp_vs_rp1_ppm_bounds"][1], 50)

    def test_spi_widths_anchored_to_last_falling_edge(self):
        trial = dict(rp1=[1_000_000], pcnt=[2, 16, 0],
                     state=dict(expired=0, dropped=0, saturated=0),
                     edges=[[120, 26, 0], [1560, 26, 1]],
                     spi=dict(latched=1, records=1, last=1120,
                              raw=[0x00288028] * 7 + [0x80000000 | 0x8028]))
        result = spi_frame(trial, dict(us_per_tick=0.0125, rate_error_ppm=1),
                           dict(secondary_minus_primary_us=[0, 0]))
        self.assertEqual(result["setup_us"], 5)
        self.assertEqual(result["hold_us"], 5.5)
        self.assertTrue(result["accepted"])

    def test_reject_missing_edges(self):
        trial = dict(pcnt=[4, 0, 0], state=dict(expired=0, dropped=0, saturated=0),
                     edges=[[WRAP - 10, 34, 1], [10, 34, 0]])
        with self.assertRaisesRegex(ValueError, "MCPWM count 2 != PCNT 4"):
            series(trial, [34])

    def test_nonce_paged_read(self):
        calls = []
        lines = iter(["obs nonce=8 active=0 expired=0 count=1 dropped=0 saturated=0 pcnt=1,0,0",
                      "OK", "obs_data nonce=8 offset=0 count=1", "edge 20 34 1", "OK"])
        data = Observer(calls.append, lambda: next(lines)).dump(8)
        self.assertEqual(data["edges"], [[20, 34, 1]])
        self.assertEqual(calls, ["obs_stop 8", "obs_read 8 0 1"])


if __name__ == "__main__":
    unittest.main()
