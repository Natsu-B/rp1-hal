import contextlib
import importlib.util
import io
from pathlib import Path
import unittest


spec = importlib.util.spec_from_file_location(
    "verify_pio_dma", Path(__file__).parent / "pio-dma-parity" / "verify.py"
)
verifier = importlib.util.module_from_spec(spec)
spec.loader.exec_module(verifier)


def fixture():
    lines = ["PIO_D_READY core=1 clk_hz=200000000 divider=200:0 quantization_us=1 irq40=masked"]
    for run in (1, 2):
        for sm in range(4):
            for rx in (0, 1):
                for n in (1, 7, 8, 9, 257):
                    lines.append(
                        f"PIO_D_FIFO run={run} sm={sm} rx={rx} words={n} count=100 total_words={100*n}"
                        " blocked_min=1000 blocked_max=1002 b_start=1 b_stop=1002 masked_min=1000 masked_max=1004 m_start=2 m_stop=1003 min=0 max=10000 start=1 stop=10000"
                        " progress=0 content=0 guard=0 fdebug=0"
                    )
            lines.append(
                f"PIO_D_ABORT run={run} sm={sm} count=25 words=257 min=0 max=257 partials=24"
                " max_us=1000 reuse=25 guard=0 fdebug=0"
            )
        lines.append(f"PIO_D_END run={run} pass=1 start=0 end=20000000 stack_free=64 error=None")
    lines += [
        "PIO_D_QUARANTINE sm=3 active=1 disabled=1 masked=1 terminal=0 start=1 stop=1000"
        " decision=Err(Recovering) retry=Err(Recovering) held=1 claimed=1 fifo=1",
        "PIO_D_QUARANTINE_END pass=1 error=None",
    ]
    return "\n".join(lines) + "\n"


class GateD(unittest.TestCase):
    def test_printk_interleaving(self):
        with contextlib.redirect_stdout(io.StringIO()):
            verifier.verify(fixture().replace("QUARANTINE_END pass", "QUARAN[ 54.002] random: crng init\nTINE_END pass"))

    def test_rejects_bad_or_incomplete_evidence(self):
        valid = fixture()
        for old, new in (
            ("count=100", "count=99"),
            ("blocked_min=1000", "blocked_min=999"),
            ("masked_min=1000", "masked_min=999"),
            ("max=10000", "max=10001"),
            ("progress=0", "progress=1"),
            ("content=0", "content=1"),
            ("guard=0", "guard=1"),
            ("fdebug=0", "fdebug=1"),
            ("reuse=25", "reuse=24"),
            ("held=1", "held=0"),
            ("PIO_D_QUARANTINE_END", "missing"),
        ):
            with self.subTest(new=new), self.assertRaises((AssertionError, KeyError)):
                verifier.verify(valid.replace(old, new, 1))


if __name__ == "__main__":
    unittest.main()
