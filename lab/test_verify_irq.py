#!/usr/bin/env python3
"""Host tests for verify-irq.py: `python3 lab/test_verify_irq.py`."""
import importlib.util
import pathlib
import unittest

spec = importlib.util.spec_from_file_location(
    "verify_irq", pathlib.Path(__file__).with_name("verify-irq.py"))
v = importlib.util.module_from_spec(spec)
spec.loader.exec_module(v)

NONCE = "1a2b3c4d"
OTHER = "0badf00d"


def uart0(nonce=NONCE):
    """The UART0 lines of one passing boot (values invented; the shape is the firmware's)."""
    lines = [
        "proc0 clocks programmed=true sys_hz=200000000 uart_hz=50000000",
        "IRQ_PROOF_BEGIN sweep=0",
        "p0 i2c1 init Ok(())",
        "p1 spi0 init true",
        "IRQ_P1_TIMER1 n=1000 ipsr=43 us=1000004 late_max_us=3 resync=0 pre_pending=0 post_pending=0",
        "IRQ_P0_TIMER0 n=1000 ipsr=42 us=1000006 late_max_us=5 resync=0 pre_pending=0 post_pending=0",
        "IRQ_INTF_ONLY core=0 pin=22 pre_pending=0 ints=1 nvic=1 post_pending=0 need_inte=0",
        "IRQ_CS iters=412345 isr_n=1000 torn=0 control_torn=57 max_masked_us=2",
        "IRQ_P0_GPIO22 n=100 ipsr=16 pre_pending=0 post_pending=0",
        "IRQ_WRAP i2c1 pre_pending=0 off_stat=1 off_pending=0 on_n=1 ipsr=24 post_pending=0",
    ]
    for i in range(v.LOG_MAIN_LINES):
        lines.append(f"IRQ_LOG_MAIN i={i}")
        if i == 7:
            lines.append("IRQ_LOG_ISR k=31")
    lines += [
        "IRQ_LOG main_lines=20 isr_tries=75 isr_dropped=74 isr_logged=1 drop_max_us=4",
        "IRQ_DEFAULT irq=28 count=1 masked=1 pre_count=0 pre_pending=0",
        "IRQ_INTF_ONLY core=1 pin=27 pre_pending=0 ints=1 nvic=1 post_pending=0 need_inte=0",
        "IRQ_WRAP spi0 pre_pending=0 off_stat=1 off_pending=0 on_n=1 ipsr=35 post_pending=0",
        "proc0 scmi p14 m7 [3, 0] -> 0 votes=8",
        "IRQ_P1_GPIO27 n=1000 ipsr=16 seen=1000",
        "IRQ_XCORE p0_seen_27=0 p1_seen_26=0",
        "IRQ_UNHANDLED p0=0 p1=1",
        "IRQ_STACK p0_free=1320 p1_free=1104 p0_size=2048 p1_size=2048",
        "IRQ_MASKED p0_max_us=3 p1_max_us=2",
        "IRQ_PROOF_PASS",
        "p0 shm enable Ok(())",
        "p1 shm cmd=1 count=1000000 errors=0 us=2000000 stack_free=880",
    ]
    # The firmware ends every marker line with the nonce, except a handler's log line.
    return "".join(
        (f"{l} nonce={nonce}" if l.startswith("IRQ_") and "LOG_ISR" not in l else l) + "\r\n"
        for l in lines)


def uart10(nonce=NONCE):
    return f"""
  1.00 RPi: BOOTSYS release VERSION:9cd61c53 DATE: 2026/04/14 TIME: 22:20:24
  4.13 RPi: BOOTLOADER release VERSION:9cd61c53 DATE: 2026/04/14 TIME: 22:20:24
  4.15 usb_pd_init status 3
 16.42 usb_max_current_enable default 0 max-current 3000
[    0.000000] Linux version 6.12.75-v8-16k
IRQ_CHECK_BEGIN
IRQ_NONCE=0x{nonce.upper()}
doorbell rtt n=1000 min=1.611µs p50=3.481µs p99=3.519µs p99.9=20µs max=36.259µs
DOORBELL_BEFORE_RC=0
SHM_PASS
SHM_RC=0
doorbell rtt n=1000 min=1.148µs p50=2.333µs p99=2.352µs p99.9=20µs max=2.358111ms
DOORBELL_AFTER_RC=0
IRQ_CHECK_RESULT=PASS
IRQ_CHECK_END
"""


def failures(u0, u10=None, nonce=None, regression=True):
    f0, _, found = v.verify_uart0(u0, nonce)
    if found is None:
        return f0
    return f0 + v.verify_uart10(uart10() if u10 is None else u10, found, regression)[0]


def changed(old, new, text=None):
    text = uart0() if text is None else text
    assert old in text, old
    return text.replace(old, new)


class VerifyIrq(unittest.TestCase):
    def assertFails(self, found, *needles):
        self.assertTrue(found, "expected a failure")
        for needle in needles:
            self.assertTrue(any(needle in f for f in found), f"{needle!r} not in {found}")

    def test_passing_boot(self):
        self.assertEqual(failures(uart0()), [])
        # A capture with a timestamp before every line.
        stamped = "".join(f"2026-11-14T10:00:00+0900 {l}\n" for l in uart0().splitlines())
        self.assertEqual(failures(stamped), [])
        _, info, _ = v.verify_uart0(uart0())
        self.assertEqual(sum("IRQ_INTF_ONLY" in i for i in info), 2)

    def test_missing_marker(self):
        without = "".join(l for l in uart0().splitlines(True) if "IRQ_WRAP spi0" not in l)
        self.assertFails(failures(without), "IRQ_WRAP spi0: missing")
        without = "".join(l for l in uart0().splitlines(True) if "IRQ_PROOF_PASS" not in l)
        self.assertFails(failures(without), "IRQ_PROOF_PASS: missing")
        self.assertFails(failures("p0 ready\n"), "no IRQ_PROOF_BEGIN")

    def test_wrong_values(self):
        for old, new, needle in [
            ("IRQ_P0_TIMER0 n=1000", "IRQ_P0_TIMER0 n=999", "IRQ_P0_TIMER0: n=999"),
            ("ipsr=43", "ipsr=42", "IRQ_P1_TIMER1: ipsr=42"),
            ("us=1000004", "us=1200000", "IRQ_P1_TIMER1: us=1200000"),
            ("IRQ_P0_GPIO22 n=100 ipsr=16 pre_pending=0", "IRQ_P0_GPIO22 n=100 ipsr=16 pre_pending=1",
             "IRQ_P0_GPIO22: pre_pending=1"),
            ("off_stat=1 off_pending=0 on_n=1 ipsr=24", "off_stat=1 off_pending=1 on_n=1 ipsr=24",
             "IRQ_WRAP i2c1: off_pending=1"),
            ("off_stat=1 off_pending=0 on_n=1 ipsr=35", "off_stat=0 off_pending=0 on_n=1 ipsr=35",
             "IRQ_WRAP spi0: off_stat=0"),
            ("torn=0", "torn=1", "IRQ_CS: torn=1"),
            ("control_torn=57", "control_torn=0", "IRQ_CS: control_torn=0"),
            ("max_masked_us=2", "max_masked_us=101", "IRQ_CS: max_masked_us=101"),
            ("count=1 masked=1", "count=2 masked=0", "IRQ_DEFAULT: count=2"),
            ("p1_seen_26=0", "p1_seen_26=3", "IRQ_XCORE: p1_seen_26=3"),
            ("IRQ_UNHANDLED p0=0", "IRQ_UNHANDLED p0=1", "IRQ_UNHANDLED: p0=1"),
            ("p1_free=1104", "p1_free=508", "IRQ_STACK: p1_free=508"),
            ("p0_max_us=3", "p0_max_us=250", "IRQ_MASKED: p0_max_us=250"),
            ("isr_dropped=74 isr_logged=1", "isr_dropped=0 isr_logged=75", "IRQ_LOG: isr_dropped=0"),
            ("stack_free=880", "stack_free=96", "stack after a shm command"),
            ("iters=412345", "iters=", "IRQ_CS: iters="),
        ]:
            with self.subTest(new=new):
                self.assertFails(failures(changed(old, new)), needle)

    def test_log_lines(self):
        self.assertFails(failures(changed(f"IRQ_LOG_MAIN i=5 nonce={NONCE}\r\n", "")), "IRQ_LOG_MAIN")
        self.assertFails(failures(changed("IRQ_LOG_ISR k=31\r\n", "")), "IRQ_LOG_ISR: 0 lines")

    def test_lines_of_another_boot(self):
        # A marker of another boot inside this boot's range, even with passing values.
        stray = f"IRQ_CS iters=100000 isr_n=1000 torn=0 control_torn=9 max_masked_us=1 nonce={OTHER}\r\n"
        self.assertFails(failures(changed("IRQ_DEFAULT", stray + "IRQ_DEFAULT")), "another boot")
        # It cannot stand in for this boot's own line either.
        mixed = changed(f"max_masked_us=2 nonce={NONCE}", f"max_masked_us=2 nonce={OTHER}")
        self.assertFails(failures(mixed), "another boot", "IRQ_CS: missing")

    def test_several_boots_in_one_capture(self):
        both = uart0(OTHER) + uart0()
        consoles = uart10(OTHER) + uart10()
        self.assertEqual(failures(both, consoles), [])
        self.assertEqual(failures(both, consoles, nonce=OTHER), [])
        # The earlier boot failed; the default (last) boot does not hide it when asked for.
        bad = changed("torn=0", "torn=4", uart0(OTHER)) + uart0()
        self.assertEqual(failures(bad, consoles), [])
        self.assertFails(failures(bad, consoles, nonce=OTHER), "IRQ_CS: torn=4")
        self.assertFails(failures(both, consoles, nonce="ffffffff"), "0 boots with nonce")
        # The Linux console of a different boot does not count for this one.
        self.assertFails(failures(uart0(), uart10(OTHER)), "0 boots with IRQ_NONCE")
        # Same nonce twice (a capture pasted together): refuse to choose.
        self.assertFails(failures(uart0() + uart0(), nonce=NONCE), "2 boots with nonce")

    def test_fail_marker_and_order(self):
        after = uart0() + f"IRQ_PROOF_FAIL p1_gpio27 nonce={NONCE}\r\n"
        self.assertFails(failures(after), "failure marker")
        late = uart0() + f"IRQ_MASKED p0_max_us=3 p1_max_us=2 nonce={NONCE}\r\n"
        self.assertFails(failures(late), "more than once", "after IRQ_PROOF_PASS")
        summary = changed("IRQ_PROOF_PASS", "IRQ_PROOF_FAIL summary p0_fails=0 p1_fails=1")
        self.assertFails(failures(summary), "failure marker", "IRQ_PROOF_PASS: missing")
        self.assertFails(failures(changed("IRQ_XCORE", "IRQ_XCORES")), "unknown marker", "IRQ_XCORE: missing")

    def test_power_banner(self):
        self.assertFails(failures(uart0(), uart10().replace("usb_pd_init status 3", "usb_pd_init status 1")),
                         "USB-PD negotiated")
        self.assertFails(failures(uart0(), uart10().replace("max-current 3000", "max-current 900")),
                         "3 A supply")
        # A capture that starts after the banner cannot show the supply.
        headless = uart10()[uart10().index("[    0.000000]"):]
        self.assertFails(failures(uart0(), headless), "0 boots with IRQ_NONCE")

    def test_linux_side(self):
        self.assertFails(failures(uart0(), uart10().replace("DOORBELL_BEFORE_RC=0", "DOORBELL_BEFORE_RC=137")),
                         "doorbell before")
        no_shm = uart10().replace("SHM_PASS\n", "").replace("SHM_RC=0", "SHM_RC=1").replace(
            "IRQ_CHECK_RESULT=PASS", "CHECK_FAIL shm\nIRQ_CHECK_RESULT=FAIL shm")
        self.assertFails(failures(uart0(), no_shm), "SHM_PASS", "failure marker")
        # --proof-only drops the regression lines, not the failure markers.
        self.assertFails(failures(uart0(), no_shm, regression=False), "failure marker")
        proof_only = uart10().replace("SHM_PASS\nSHM_RC=0\n", "")
        self.assertFails(failures(uart0(), proof_only), "SHM_PASS")
        self.assertEqual(failures(uart0(), proof_only, regression=False), [])
        self.assertFails(failures(uart0(), uart10() + "Kernel panic - not syncing\n"), "Kernel panic")

    def test_contract_matches_the_firmware_source(self):
        # Every field the checker decides on is one the firmware prints in that marker.
        import re
        source = "".join(p.read_text() for p in sorted(
            pathlib.Path(__file__).with_name("irq-proof").glob("src/*.rs")))
        formats = re.findall(r'mark!\(\s*"([^"]+)"', source)
        for key, wanted in v.CONTRACT.items():
            name = key.split()[0]
            printed = [f for f in formats
                       if re.fullmatch(f.split()[0].replace("{}", r"\d"), name)]
            self.assertTrue(printed, f"{name} is not in the firmware")
            fields = {field for f in printed for field in re.findall(r"([a-z][a-z0-9_]*)=\{", f)}
            self.assertLessEqual(set(wanted), fields, key)

    def test_command_line(self):
        import tempfile
        with tempfile.TemporaryDirectory() as d:
            u0, u10 = pathlib.Path(d, "uart0.raw"), pathlib.Path(d, "uart10.raw")
            u0.write_bytes(uart0().encode())
            u10.write_bytes(uart10().encode())
            self.assertEqual(v.main(["--uart0", str(u0), "--uart10", str(u10)]), 0)
            u0.write_bytes(changed("torn=0", "torn=1").encode())
            self.assertEqual(v.main(["--uart0", str(u0), "--uart10", str(u10)]), 1)


if __name__ == "__main__":
    unittest.main()
