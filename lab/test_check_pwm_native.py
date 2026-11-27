import importlib.util
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location(
    "native", Path(__file__).with_name("check-pwm-native.py")
)
native = importlib.util.module_from_spec(spec)
spec.loader.exec_module(native)

# Objdump body from the Cortex-M3 release probe.
ASSEMBLY = """\
10000262 <motor_native_duty>:
10000262:	ldr	r2, [r0, #0]
10000264:	cmp	r1, r2
10000266:	bhi.n	10000290 <motor_native_duty+0x2e>
10000268:	push	{r7, lr}
1000026a:	mov	r7, sp
1000026c:	movw	r0, #32800	@ 0x8020
10000270:	mov.w	r3, #2147483648	@ 0x80000000
10000274:	movt	r0, #16393	@ 0x4009
10000278:	str	r1, [r0, #0]
1000027a:	movw	r0, #40960	@ 0xa000
1000027e:	movt	r0, #16393	@ 0x4009
10000282:	dsb	sy
10000286:	str	r3, [r0, #0]
10000288:	dsb	sy
1000028c:	ldmia.w	sp!, {r7, lr}
10000290:	movs	r0, #0
10000292:	cmp	r1, r2
10000294:	it	ls
10000296:	movls	r0, #1
10000298:	bx	lr
"""


class CheckPwmNative(unittest.TestCase):
    def test_release_probe(self):
        self.assertEqual(native.check(ASSEMBLY), ASSEMBLY)

    def test_rejects_mutated_probe(self):
        for before, after in [
            ("str\tr1, [r0, #0]", "bl\t10000300 <callee>"),
            ("mov\tr7, sp", "udiv\tr7, r1, r2"),
            ("10000282:\tdsb\tsy", "10000282:\tb\t10000264 <motor_native_duty+0x2>"),
            ("10000282:\tdsb\tsy", "10000281:\tstr\tr3, [r0, #0]\n10000282:\tdsb\tsy"),
            ("10000282:\tdsb\tsy", "10000281:\tldr\tr7, [r3]\n10000282:\tdsb\tsy"),
            ("ldr\tr2, [r0, #0]", "ldr\tr2, [r3, #0]"),
            ("10000274:\tmovt\tr0, #16393", "10000274:\tmovt\tr0, #16394"),
            ("1000027e:\tmovt\tr0, #16393", "1000027e:\tmovt\tr0, #16394"),
            ("str\tr1, [r0, #0]", "str\tr1, [r0, #16]"),
        ]:
            with self.subTest(after=after), self.assertRaises(ValueError):
                self.assertIn(before, ASSEMBLY)
                native.check(ASSEMBLY.replace(before, after, 1))


if __name__ == "__main__":
    unittest.main()
