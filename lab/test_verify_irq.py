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
        "PLAT_INIT ok=1 warm=0 ltssm_off=1 link_before_scmi=0 perstn_wait_us=1766406 link_wait_us=8204 "
        "monitor2=0x1b0000 clocks_programmed=1 sys_hz=200000000 uart_hz=50000000",
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
        "IRQ_P1_GPIO27 ipsr=16 seen=1000",
        "IRQ_XCORE p0_seen_27=0 p1_seen_26=0",
        "IRQ_UNHANDLED p0=0 p1=1",
        "IRQ_STACK p0_free=1320 p1_free=1104 p0_size=2048 p1_size=2048",
        "IRQ_MASKED p0_max_us=3 p1_max_us=2",
        "IRQ_PROOF_PASS",
        "p0 shm enable Ok(())",
    ]
    lines += [f"p1 shm cmd={c} count=1000000 errors=0 us=2000000 stack_free=880"
              for c in (1, 2, 3, 4, 5, 6, 7, 8, 10, 11, 12, 13)]
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
    f0, _, found = v.verify_uart0(u0, nonce, regression)
    if found is None:
        return f0
    steps = v.UART10_PROOF + (v.UART10_REGRESSION if regression else [])
    return f0 + v.verify_uart10(uart10() if u10 is None else u10, found, steps)[0]


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
        self.assertFails(failures(changed("IRQ_PROOF_BEGIN sweep=0", "IRQ_PROOF_BEGIN sweep=1")), "sweep=1")
        # The stack figure after each shm command is part of the regression.
        short = uart0().replace("p1 shm cmd=13 ", "p1 shm command 13 ")
        self.assertFails(failures(short), "11 lines, not 12")
        self.assertEqual(failures(short, proof_only, regression=False), [])

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
            fields = {field for f in printed for field in re.findall(r"([a-z][a-z0-9_]*)=[{\d]", f)}
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


BOOT = "5eed0001"
ZERO = "00000000,00000000"


def pair(bits, width=0):
    return f"{bits & 0xffffffff:0{width}x},{bits >> 32:0{width}x}"


def sweep_records(irq):
    """The PRE, TRIAL and result lines the image prints for a line routed as expected (the
    shape and values of the 2026-11-15 hardware boots)."""
    stages, wanted, source = v.SWEEP_KINDS[v.SWEEP_TABLE[irq].rstrip("0123456789")]
    target = 1 << irq
    fs = "-" if irq in (52, 1, 2) else "8:1f,9:1f"
    lines = [f"IRQ_SWEEP_PRE irq={irq} fs={fs} found=0,44570140"]
    for i in range(v.SWEEP_TRIALS * len(stages)):
        stage = stages[i % len(stages)]
        status = source | (1 << 16 if stage in ("wrap1", "real") else 0)
        post = target if wanted[i % len(stages)] else 0
        lines.append(f"IRQ_SWEEP_TRIAL irq={irq} stage={stage} trial={i // len(stages)} pre=0,0 base=0,0 "
                     f"raised=1 post={pair(post)} status={status:x} intstat=0,0 us=1 clr=0,0")
    routed_by = stages[wanted.index(v.SWEEP_TRIALS)]
    lines.append(f"IRQ_SWEEP irq={irq} block={v.SWEEP_TABLE[irq]} result=ROUTED reason={routed_by} detail=0 "
                 f"exit=left hits={v.expected_hits(stages, wanted)} ispr_before={ZERO} "
                 f"ispr_after={pair(target, 8)} new={ZERO} status={source:x}")
    return lines


def sweep_uart0(nonce=NONCE, results=None):
    """The UART0 lines of one passing boot of lab/irq-sweep. `results` maps an IRQ to the list
    of lines that replaces its own."""
    results = results or {}
    lines = ["proc0 clocks sys_hz=200000000 uart_hz=50000000",
             "p0 pwm1 init Some(true) adc init true",
             f"IRQ_SWEEP_BEGIN lines={len(v.SWEEP_TABLE)} iser={ZERO} ispr={ZERO}"]
    for irq in v.SWEEP_TABLE:
        lines += results.get(irq, sweep_records(irq))
    lines += [f"IRQ_SWEEP_END lines={len(v.SWEEP_TABLE)} iser={ZERO} us=731638",
              "PLAT_INIT ok=1 warm=0 ltssm_off=1 link_before_scmi=0 perstn_wait_us=1766406 "
              "link_wait_us=8204 monitor2=0x1b0000 clocks_programmed=1",
              f"IRQ_SWEEP_BOOT boot={BOOT}"]
    return "".join((f"{l} nonce={nonce}" if l.startswith("IRQ_") else l) + "\r\n" for l in lines)


def edited(irq, *edits):
    """The lines of `irq`, with each (index, old, new) replacement applied."""
    lines = sweep_records(irq)
    for index, old, new in edits:
        assert old in lines[index], (old, lines[index])
        lines[index] = lines[index].replace(old, new)
    return lines


def sweep_failures(results):
    return v.verify_sweep(sweep_uart0(results=results))[0]


def sweep_uart10(boot=BOOT):
    return f"""
  1.00 RPi: BOOTSYS release VERSION:9cd61c53 DATE: 2026/04/14 TIME: 22:20:24
  4.15 usb_pd_init status 3
 16.42 usb_max_current_enable default 0 max-current 3000
SWEEP_CHECK_BEGIN
IRQ_NONCE=0x{boot.upper()}
LAB_FINAL_BEGIN check=sweep nonce={boot}
RP1_CORE core=0 state=Running reason=0
RP1_CORE core=1 state=Running reason=0
LAB_FINAL_END check=sweep nonce={boot} result=PASS
SWEEP_CHECK_RESULT=PASS
SWEEP_CHECK_END
"""


def sweep_changed(old, new):
    text = sweep_uart0()
    assert old in text, old
    return text.replace(old, new, 1)


class VerifySweep(unittest.TestCase):
    assertFails = VerifyIrq.assertFails

    def test_all_lines_routed_pass(self):
        found, summary = v.verify_sweep(sweep_uart0())
        self.assertEqual(found, [])
        self.assertEqual((summary["nonce"], summary["boot"]), (NONCE, BOOT))
        self.assertEqual([r["irq"] for r in summary["lines"]], list(v.SWEEP_TABLE))
        self.assertEqual({r["result"] for r in summary["lines"]}, {"ROUTED"})
        stamped = "".join(f"2026-11-15T22:38:00+0900 {l}\n" for l in sweep_uart0().splitlines())
        self.assertEqual(v.verify_sweep(stamped)[0], [])

    def test_any_other_result_fails(self):
        # What the image prints when I2C0's output lands on IRQ 8: the first wrap1 trial shows
        # a foreign bit and the line stops.
        other = sweep_records(7)[:3]
        other[2] = other[2].replace("post=80,0", "post=100,0")
        other.append(f"IRQ_SWEEP irq=7 block=i2c0 result=OTHER_LINE reason=- detail=0 exit=left "
                     f"hits=wrap0:0/3,wrap1:0/3 ispr_before={ZERO} ispr_after=00000100,00000000 "
                     f"new=00000100,00000000 status=10010")
        self.assertFails(sweep_failures({7: other}), "irq=7: result=OTHER_LINE", "not ROUTED by wrap1")
        # PWM1 never pending.
        dead = [l.replace("post=0,200", "post=0,0") for l in sweep_records(41)]
        dead[-1] = dead[-1].replace("result=ROUTED reason=force", "result=NOT_ROUTED reason=-").replace(
            "force:3/3", "force:0/3")
        self.assertFails(sweep_failures({41: dead}), "irq=41: result=NOT_ROUTED")
        for result in ("SKIP reason=pin_routed", "ABORT reason=rearm", "NEW_PENDING reason=-"):
            line = edited(20, (-1, "ROUTED reason=wrap1", result))
            with self.subTest(result=result):
                self.assertFails(sweep_failures({20: line}), f"irq=20: result={result.split()[0]}")

    def test_routed_by_the_wrong_stimulus_fails(self):
        # The wrapper did not gate the line: the bit is not shown to be this block's output.
        ungated = [l.replace("post=0,0", "post=80,0") for l in sweep_records(7)]
        ungated[-1] = ungated[-1].replace("wrap0:0/3", "wrap0:3/3").replace("reason=wrap1", "reason=wrap0")
        self.assertFails(sweep_failures({7: ungated}), "irq=7: result=ROUTED reason=wrap0")
        # ADC routed by force only.
        forced = edited(52, *[(i, "post=0,100000", "post=0,0") for i in (2, 4, 6)],
                        (-1, "real:3/3", "real:0/3"))
        self.assertFails(sweep_failures({52: forced}), "irq=52: result=ROUTED reason=force hits=force:3/3,real:0/3")

    def test_trials_must_bear_out_the_result(self):
        cases = [
            (edited(7, (2, "post=80,0", "post=0,0")), "the trials say wrap0:0/3,wrap1:2/3"),
            (sweep_records(7)[:-2] + sweep_records(7)[-1:], "5 trials, not 6"),
            (edited(7, (1, "stage=wrap0", "stage=wrap1")), "trial 0 (trial=0 stage=wrap1): out of order"),
            (edited(7, (1, "raised=1", "raised=0")), "trial 0 (trial=0 stage=wrap0): raised=0"),
            (edited(7, (1, "status=10 ", "status=0 ")), "does not show the wrap0 source"),
            (edited(7, (2, "status=10010", "status=10")), "does not show the wrap1 source"),
            (edited(7, (1, "base=0,0", "base=80,0")), "target bit pending before the stimulus"),
            (edited(7, (2, "clr=0,0", "clr=80,0")), "or after the deassert"),
            (edited(7, (2, "post=80,0", "post=180,0")), "another line became pending: post=180,0"),
            (edited(7, (1, " post=0,0", "")), "base, post, clr or status missing"),
            (edited(7, (-1, "exit=left", "exit=cleanup")), "exit=cleanup"),
        ]
        for lines, needle in cases:
            with self.subTest(needle=needle):
                self.assertFails(sweep_failures({7: lines}), needle)
        moved = sweep_records(9)
        moved.insert(1, sweep_records(7)[1])
        self.assertFails(sweep_failures({9: moved}), "preceded by another line's trials")
        orphan = sweep_changed("IRQ_SWEEP_END lines=9", f"{sweep_records(7)[1]} nonce={NONCE}\r\nIRQ_SWEEP_END lines=9")
        self.assertFails(v.verify_sweep(orphan)[0], "without their result")

    def test_frame(self):
        self.assertFails(v.verify_sweep("PLAT_INIT ok=1\n")[0], "no IRQ_SWEEP_BEGIN")
        end = f"IRQ_SWEEP_END lines=9 iser={ZERO} us=731638 nonce={NONCE}\r\n"
        self.assertFails(v.verify_sweep(sweep_changed(end, ""))[0], "no IRQ_SWEEP_END", "before IRQ_SWEEP_END")
        self.assertFails(v.verify_sweep(sweep_changed(end, end + end))[0], "more than once")
        self.assertFails(sweep_failures({20: []}), "irq=20: missing", "lines=9")
        self.assertFails(sweep_failures({20: sweep_records(20) * 2}), "irq=20: more than once")
        self.assertFails(v.verify_sweep(sweep_changed(f"iser={ZERO} ispr", "iser=00000080,00000000 ispr"))[0],
                         "IRQ_SWEEP_BEGIN: iser=00000080")
        self.assertFails(v.verify_sweep(sweep_changed(f"iser={ZERO} us", "iser=00000080,00000000 us"))[0],
                         "IRQ_SWEEP_END: iser=00000080")
        self.assertFails(v.verify_sweep(sweep_uart0() + f"IRQ_SWEEP_PANIC nonce={NONCE}\r\n")[0], "IRQ_SWEEP_PANIC")

    def test_table(self):
        dma = [l.replace("irq=7 block=i2c0", "irq=40 block=dma").replace("irq=7 ", "irq=40 ")
               for l in sweep_records(7)]
        self.assertFails(sweep_failures({7: dma}), "must not be swept")
        self.assertFails(sweep_failures({7: edited(7, (-1, "block=i2c0", "block=i2c1"))}), "not in the sweep table")
        # A line without its IRQ and block is reported, not a crash.
        self.assertFails(sweep_failures({7: edited(7, (-1, "irq=7 block=i2c0 ", ""))}),
                         "irq=None block=None: not in the sweep table", "irq=7: missing")
        self.assertFails(sweep_failures({7: sweep_records(9), 9: sweep_records(7)}), "out of table order")

    def test_lines_of_another_boot(self):
        stale = f"{sweep_records(22)[-1]} nonce={OTHER}\r\n"
        mine = f"{sweep_records(22)[-1]} nonce={NONCE}\r\n"
        self.assertFails(v.verify_sweep(sweep_changed(mine, stale + mine))[0], "another boot")
        # Two boots in one capture: the last by default, either by nonce.
        both = sweep_uart0(OTHER) + sweep_uart0()
        self.assertEqual(v.verify_sweep(both)[0], [])
        self.assertEqual(v.verify_sweep(both, OTHER)[0], [])
        self.assertFails(v.verify_sweep(both, "ffffffff")[0], "0 boots")

    def test_sweep_ends_before_the_link(self):
        pcie = "PLAT_INIT ok=1 warm=0 ltssm_off=1 link_before_scmi=0 perstn_wait_us=1766406"
        self.assertFails(v.verify_sweep(sweep_changed(pcie, pcie.replace("1766406", "4294967295")))[0],
                         "no fresh PCIe bring-up")
        self.assertFails(v.verify_sweep(sweep_changed("PLAT_INIT ok=1", "PLAT_INIT ok=0"))[0],
                         "no fresh PCIe bring-up")
        early = sweep_changed("IRQ_SWEEP_PRE irq=7", pcie + " link_wait_us=1 monitor2=0x0\r\nIRQ_SWEEP_PRE irq=7")
        self.assertFails(v.verify_sweep(early)[0], "before IRQ_SWEEP_END")

    def test_command_line_and_json(self):
        import json
        import tempfile
        with tempfile.TemporaryDirectory() as d:
            u0, u10, out = (pathlib.Path(d, n) for n in ("uart0.raw", "uart10.raw", "sweep.json"))
            u0.write_bytes(sweep_uart0().encode())
            u10.write_bytes(sweep_uart10().encode())
            args = ["--sweep", "--uart0", str(u0), "--json", str(out)]
            self.assertEqual(v.main(args), 3)
            self.assertEqual(v.main(args + ["--uart10", str(u10)]), 0)
            summary = json.loads(out.read_text())
            self.assertEqual(summary["verdict"], "PASS")
            self.assertEqual(len(summary["lines"]), len(v.SWEEP_TABLE))
            u10.write_bytes(sweep_uart10("0badf00d").encode())
            self.assertEqual(v.main(args + ["--uart10", str(u10)]), 1)
            self.assertEqual(json.loads(out.read_text())["verdict"], "FAIL")
            u0.write_bytes(sweep_uart0(results={41: edited(41, (-1, "result=ROUTED", "result=NOT_ROUTED"))}).encode())
            self.assertEqual(v.main(args), 1)

    def test_sweep_requires_nonce_bound_final_running_status(self):
        for edit in (
            lambda text: text.replace("core=0 state=Running", "core=0 state=Panicked"),
            lambda text: text.replace("core=1 state=Running", "core=1 state=Fault"),
            lambda text: text.replace(f"check=sweep nonce={BOOT}", "check=sweep nonce=00000001"),
            lambda text: "\n".join(l for l in text.splitlines() if not l.startswith("LAB_FINAL")),
        ):
            failures, _ = v.verify_uart10(edit(sweep_uart10()), BOOT, v.UART10_SWEEP,
                                          final_check="sweep")
            self.assertTrue(failures)

    def test_table_matches_the_firmware_source(self):
        import re
        source = pathlib.Path(__file__).with_name("irq-sweep").joinpath("src/sweep.rs").read_text()
        table = {int(irq): name for irq, name in re.findall(r'irq: (\d+),\s+name: "(\w+)"', source)}
        self.assertEqual(table, v.SWEEP_TABLE)

    def test_records_match_the_firmware_source(self):
        import re
        src = pathlib.Path(__file__).with_name("irq-sweep").joinpath("src")
        sweep, blocks = (src.joinpath(n).read_text() for n in ("sweep.rs", "blocks.rs"))
        for name in ("IRQ_SWEEP_TRIAL", "IRQ_SWEEP"):
            fmt = re.search(r'"' + name + r' (irq=[^"]*)"', sweep).group(1)
            line = next(l for l in sweep_records(9) if l.startswith(name + " "))
            self.assertEqual(re.findall(r"(\w+)=", fmt), re.findall(r"(\w+)=", line), name)
        kinds = {"I2c": "i2c", "Spi": "spi", "Adc": "adc", "Pwm1": "pwm", "Bank": "io_bank"}
        found = re.findall(r'impl Source for (\w+) \{.*?fn stages\(&self\)[^{]*\{\s*&\[([^\]]*)\]', blocks, re.S)
        self.assertEqual({kinds[k]: tuple(re.findall(r'"(\w+)"', s)) for k, s in found},
                         {k: stages for k, (stages, _, _) in v.SWEEP_KINDS.items()})
        self.assertEqual(int(re.search(r"const TRIALS: usize = (\d+);", sweep).group(1)), v.SWEEP_TRIALS)


if __name__ == "__main__":
    unittest.main()
