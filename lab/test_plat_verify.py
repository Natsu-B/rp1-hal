#!/usr/bin/env python3
"""Host tests for plat-verify.py (V1): `python3 lab/test_plat_verify.py`."""
import contextlib
import importlib.util
import io
import os
import pathlib
import stat
import tempfile
import unittest

spec = importlib.util.spec_from_file_location(
    "plat_verify", pathlib.Path(__file__).with_name("plat-verify.py"))
v = importlib.util.module_from_spec(spec)
spec.loader.exec_module(v)

BUDGET = ("size budget image=43536/49152 sram_free=5616 isram=3876/8192 isram_free=4316 "
          "dsram_static=272 stack0=2816 stack1=2048 margin=768 dsram_free0=5104 "
          "dsram_free1=5872")
STACK_PASS = "stack_static=PASS s_plat=572 h57=448 h53=8 hhard=8 hfault=0 margin=768 stack0=2816"


def tool(stack=STACK_PASS, budget=BUDGET):
    """Synthetic `stack-paths --size` output."""
    lines = [budget, "size owner=platform code=16366 data=28 bss=70",
             "stack root=__rp1_p0_PROC_MISC bytes=448", stack]
    return "\n".join(l for l in lines if l) + "\n"


UART0 = (
    "[12:00:01.000] IRQ_STACK p0_free=1320 p1_free=1104 p0_size=2816 p1_size=2048 nonce=1a2b3c4d\r\n"
    "[12:00:02.000] p1 shm cmd=1 count=1000000 errors=0 us=2000000 stack_free=880\r\n"
    "[12:00:03.000] RINGDMA_STACK run_id=3 free_bytes=600\r\n"
)


class Rows(unittest.TestCase):
    def test_stack_static(self):
        self.assertEqual(v.stack_row(tool(), 0)[0], v.PASS)
        self.assertIn("s_plat=572", v.stack_row(tool(), 0)[1])
        # A tool verdict and its exit status must agree.
        self.assertEqual(v.stack_row(tool(), 1)[0], v.FAIL)
        fail = "stack_static=FAIL reason=recursion at=f+0x2 root=__rp1_p0_PROC_MISC"
        self.assertEqual(v.stack_row(tool(stack=fail), 1), (v.FAIL, fail[len("stack_static=FAIL "):]))
        # FAIL with exit 0 is a tool inconsistency, not a verdict.
        self.assertEqual(v.stack_row(tool(stack=fail), 0)[0], v.INCOMPLETE)
        margin = "stack_static=FAIL reason=margin s_plat=900 h57=776 h53=8 hhard=8 hfault=0 margin=768 stack0=2816"
        self.assertEqual(v.stack_row(tool(stack=margin), 1)[0], v.FAIL)
        diag = "stack_static=DIAG s_plat=564 h57=440 h53=8 hhard=8 hfault=0 margin=768 stack0=2816"
        self.assertEqual(v.stack_row(tool(stack=diag), 1)[0], v.FAIL)
        self.assertEqual(v.stack_row(None, None)[0], v.INCOMPLETE)
        self.assertEqual(v.stack_row(tool(stack=""), 0)[0], v.INCOMPLETE)
        # Usage or ELF error (exit 2) is no verdict.
        self.assertEqual(v.stack_row(tool(stack=fail), 2)[0], v.INCOMPLETE)
        self.assertEqual(v.stack_row(tool() + STACK_PASS + "\n", 0)[0], v.INCOMPLETE)

    def test_stack_watermark_free(self):
        self.assertEqual(v.watermark_row([UART0, None]), (v.PASS, "min=600 n=4 need=512"))
        self.assertEqual(v.watermark_row([UART0.replace("free_bytes=600", "free_bytes=448"), ""])[0],
                         v.FAIL)
        self.assertEqual(v.watermark_row([UART0.replace("p0_free=1320", "p0_free=512")])[0], v.PASS)
        self.assertEqual(v.watermark_row([UART0.replace("p0_free=1320", "p0_free=511")])[0], v.FAIL)
        self.assertEqual(v.watermark_row([None, None])[0], v.INCOMPLETE)
        self.assertEqual(v.watermark_row(["PLAT_INIT ok=1\n", "Linux version\n"])[0], v.INCOMPLETE)
        # A size field of a watermark line is not a free value.
        self.assertEqual(v.watermark_row(["IRQ_STACK p0_size=100 p1_size=100\n"])[0], v.INCOMPLETE)

    def test_size(self):
        self.assertEqual(v.size_row(tool())[0], v.PASS)
        self.assertEqual(v.size_row(tool(budget=BUDGET.replace("dsram_free1=5872", "dsram_free1=-8")))[0],
                         v.FAIL)
        self.assertEqual(v.size_row(tool(budget=BUDGET.replace("sram_free=5616", "sram_free=-1")))[0],
                         v.FAIL)
        self.assertEqual(v.size_row(None)[0], v.INCOMPLETE)
        self.assertEqual(v.size_row(tool(budget=""))[0], v.INCOMPLETE)
        self.assertEqual(v.size_row(tool(budget="size budget image=1/49152"))[0], v.INCOMPLETE)

    def test_overall(self):
        ok = (v.PASS, "")
        self.assertEqual(v.overall([ok, ok, ok]), v.PASS)
        self.assertEqual(v.overall([ok, (v.INCOMPLETE, ""), ok]), v.INCOMPLETE)
        self.assertEqual(v.overall([(v.FAIL, ""), (v.INCOMPLETE, ""), ok]), v.FAIL)


class Main(unittest.TestCase):
    """The whole script with a fake stack-paths executable."""

    def run_main(self, args, output=tool(), status=0):
        with tempfile.TemporaryDirectory() as d:
            d = pathlib.Path(d)
            fake = d / "stack-paths"
            (d / "out.txt").write_text(output)
            fake.write_text(f"#!/bin/sh\ncat '{d / 'out.txt'}'\nexit {status}\n")
            fake.chmod(fake.stat().st_mode | stat.S_IEXEC)
            (d / "fw.elf").write_bytes(b"\x7fELF")
            (d / "uart0.log").write_text(UART0)
            argv = [a.replace("@", str(d) + os.sep) for a in args] + ["--stack-paths", str(fake)]
            out = io.StringIO()
            with contextlib.redirect_stdout(out):
                code = v.main(argv)
            return code, out.getvalue()

    def test_pass(self):
        code, out = self.run_main(["--elf", "@fw.elf", "--uart0", "@uart0.log"])
        self.assertEqual(code, 0, out)
        self.assertIn("sha256=", out.splitlines()[0])
        self.assertTrue(out.rstrip().endswith("plat_verify=PASS"), out)

    def test_missing_capture_is_incomplete(self):
        code, out = self.run_main(["--elf", "@fw.elf", "--uart0", "@nothing.log"])
        self.assertEqual(code, 3, out)
        self.assertIn("stack_watermark_free=INCOMPLETE", out)
        self.assertIn("plat_verify=INCOMPLETE", out)

    def test_missing_elf_is_incomplete(self):
        code, out = self.run_main(["--elf", "@none.elf", "--uart0", "@uart0.log"])
        self.assertEqual(code, 3, out)
        self.assertIn("stack_static=INCOMPLETE", out)
        self.assertIn("size=INCOMPLETE", out)

    def test_fail(self):
        fail = "stack_static=FAIL reason=recursion at=f+0x2 root=__rp1_p0_PROC_MISC"
        code, out = self.run_main(["--elf", "@fw.elf", "--uart0", "@uart0.log"],
                                  output=tool(stack=fail), status=1)
        self.assertEqual(code, 1, out)
        self.assertIn("plat_verify=FAIL", out)


if __name__ == "__main__":
    unittest.main()
