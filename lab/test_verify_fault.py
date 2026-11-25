"""Synthetic raw UART captures for the PR-E verifier."""

import contextlib
import hashlib
import importlib.util
import io
from pathlib import Path
from tempfile import TemporaryDirectory
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location("verify_fault", Path(__file__).with_name("verify-fault.py"))
vf = importlib.util.module_from_spec(spec)
spec.loader.exec_module(vf)

PASS, FAIL, INCOMPLETE = vf.PASS, vf.FAIL, vf.INCOMPLETE
NONCE = "12ab5678"
BOOT_ID = "deadbeef-1234-1234-1234-1234567890ab"
BUNDLE_ID = "0123456789abcdef"


def captures(kind="p0-thread-panic", phase="cold-1", nonce=NONCE, boot_id=BOOT_ID):
    code = vf.CODES[kind]
    core = int(kind == "p1-panic")
    state, reason, aux = "Panicked", 0, "0x0"
    if kind.endswith("udf"):
        state, reason, aux = "Fault", 6, "0x10000"
    if kind == "stkerr":
        state, reason, aux = "Fault", 3, "0x10010"

    reset = 0 if phase == "reboot" else 1
    previous = {
        "reboot": "[  42.23] reboot: Restarting system\n",
        "poweroff": "[  42.23] reboot: Power down\nRPi: BOOTSYS\nHalt: power_off: 1\nPower off\n",
    }.get(phase, "")
    tag = f"nonce={nonce} kind={kind}"
    uart0 = f"\x00FAULT_BOOT nonce={nonce} kinds={','.join(vf.CODES)} stkerr=1 irq57=unavailable\n"
    uart10 = (
        f"{previous}RPi: BOOTSYS release VERSION:abc\n"
        f"  1.00 POWER_OFF_ON_HALT: 1 WAIT_FOR_POWER_BUTTON 0 power-on-reset {reset}\n"
        f"[rp1boot     253] bundle ok: rp1.bundle={BUNDLE_ID} (3 elements, sha256 in 211 ms)\n"
        f"FAULT_BEGIN {tag} boot_id={boot_id} epoch=2\n"
        f"FAULT_PRE {tag} p0_before=25 p0_after=30 p1_before=26 p1_after=31\n"
        f"FAULT_REQUEST {tag} code={code}\n"
        f"FAULT_STATUS {tag} core={core} ack={code} state={state} reason={reason} aux={aux}"
        " peer=Running pcie=Up scmi=Serving(requests=126)\n"
    )
    if kind != "stkerr":
        uart10 += (
            f"FAULT_TIMER {tag} elapsed_ms=1000 p0_before=40 p0_after={60 if core else 40}"
            f" p1_before=41 p1_after={41 if core else 61}\n"
            f"FAULT_PING {tag} sent=20 received=20 rc=0\n"
            f"FAULT_USB {tag} probe=yes rc=0 result=PASS evidence=ok\n"
            f"FAULT_RATE {tag} hz=48000000\n"
            f"FAULT_RATE {tag} hz=48000000\n"
            f"FAULT_SCMI {tag} n=2 replies=2 before=126 after=128"
            " scmi_before=Serving(requests=126) scmi_after=Serving(requests=128)\n"
            f"FAULT_END {tag} pcie=Up\n"
        )
    return uart0, uart10.replace("\n", "\r\n")


class TestVerifyFault(unittest.TestCase):
    def run_rows(self, kind="p0-thread-panic", phase="cold-1", edit=lambda a, b: (a, b)):
        uart0, uart10 = edit(*captures(kind, phase))
        rows, _ = vf.verify_run(kind, phase, NONCE, uart0, uart10, BUNDLE_ID)
        return rows

    def test_all_kinds_and_boot_phases_pass(self):
        for kind in vf.KINDS:
            for phase in vf.PHASES:
                with self.subTest(kind=kind, phase=phase):
                    self.assertEqual(vf.overall(self.run_rows(kind, phase).values()), PASS)

    def test_required_rows_fail_when_observed_wrong(self):
        cases = (
            ("request", "cold-1", " code=1", " code=2", FAIL),
            ("terminal", "cold-1", "state=Panicked", "state=Running", FAIL),
            ("timer_live_before", "cold-1", "p1_after=31", "p1_after=26", FAIL),
            ("timer", "cold-1", "p0_after=40", "p0_after=41", FAIL),
            ("timer", "cold-1", "elapsed_ms=1000", "elapsed_ms=999", FAIL),
            ("ping", "cold-1", "received=20 rc=0", "received=19 rc=1", FAIL),
            ("usb", "cold-1", "rc=0 result=PASS evidence=ok", "rc=1 result=FAIL evidence=timeout", FAIL),
            ("scmi", "cold-1", "after=128", "after=129", FAIL),
            ("scmi", "cold-1", "replies=2", "replies=1", FAIL),
            ("scmi", "cold-1", "scmi_after=Serving(requests=128)", "scmi_after=Quarantined", FAIL),
            ("pcie", "cold-1", "pcie=Up", "pcie=Lost(monitor2_hi=0x0)", FAIL),
        )
        for row, phase, old, new, want in cases:
            with self.subTest(row=row, evidence=new):
                rows = self.run_rows(phase=phase, edit=lambda a, b: (a, b.replace(old, new)))
                self.assertEqual(rows[row], want)
        uart0, uart10 = captures(phase="reboot")
        rows, _ = vf.verify_run("p0-thread-panic", "cold-1", NONCE, uart0, uart10, BUNDLE_ID)
        self.assertEqual(rows["boot"], FAIL)

    def test_all_required_evidence_must_be_present(self):
        cases = (
            ("identity", "BOOT", "uart0"),
            ("identity", "BEGIN", "uart10"),
            ("boot", "power-on-reset", "uart10"),
            ("bundle", "bundle ok", "uart10"),
            ("request", "REQUEST", "uart10"),
            ("terminal", "STATUS", "uart10"),
            ("timer_live_before", "PRE", "uart10"),
            ("timer", "TIMER", "uart10"),
            ("ping", "PING", "uart10"),
            ("usb", "USB", "uart10"),
            ("scmi", "SCMI", "uart10"),
            ("scmi", "RATE", "uart10"),
            ("pcie", "END", "uart10"),
        )
        for row, marker, stream in cases:
            with self.subTest(row=row, marker=marker):
                def edit(a, b):
                    def remove(text):
                        return "\n".join(line for line in text.splitlines() if marker not in line)
                    return (remove(a), b) if stream == "uart0" else (a, remove(b))
                rows = self.run_rows(edit=edit)
                self.assertEqual(rows[row], INCOMPLETE)
                self.assertEqual(vf.overall(rows.values()), INCOMPLETE)

    def test_udf_requires_usagefault_and_undefinstr(self):
        for old, new, want in (
            ("reason=6", "reason=3", FAIL),
            ("aux=0x10000", "aux=0x0", FAIL),
            ("aux=0x10000", "aux=0x20010000", PASS),
            ("aux=0x10000", "", INCOMPLETE),
        ):
            rows = self.run_rows("p0-isr-udf", edit=lambda a, b: (a, b.replace(old, new)))
            self.assertEqual(rows["terminal"], want)

    def test_proc1_requires_proc0_to_continue(self):
        for old, new, want in (
            ("p0_after=60", "p0_after=40", FAIL),
            ("p0_after=60", "", INCOMPLETE),
            ("peer=Running", "peer=Panicked", FAIL),
        ):
            rows = self.run_rows("p1-panic", edit=lambda a, b: (a, b.replace(old, new)))
            self.assertEqual(rows["proc0_continues"], want)

    def test_irq57_and_stacking_diagnostics(self):
        self.assertEqual(self.run_rows("stkerr")["terminal"], PASS)
        for old, new, want in (
            ("state=Fault", "state=Running", INCOMPLETE),
            ("aux=0x10010", "aux=0x10000", FAIL),
            ("aux=0x10010", "aux=0x11000", PASS),
            ("aux=0x10010", "", INCOMPLETE),
        ):
            rows = self.run_rows("stkerr", edit=lambda a, b: (a, b.replace(old, new)))
            self.assertEqual(rows["terminal"], want)
        rows = self.run_rows("stkerr", edit=lambda a, b: (a.replace("stkerr=1", "stkerr=0"), b))
        self.assertEqual(rows["terminal"], INCOMPLETE)

    def test_incomplete_or_cross_boot_evidence_cannot_pass(self):
        for name, edit in (
            ("identity", lambda a, b: (a.replace(NONCE, "55667788"), b)),
            ("sequence", lambda a, b: (a, b.replace("FAULT_END " + f"nonce={NONCE}", "FAULT_END nonce=ffeeddcc"))),
            ("sequence", lambda a, b: (a, b + "FAULT_END " + f"nonce={NONCE} kind=p0-thread-panic pcie=Up")),
            ("sequence", lambda a, b: (a, b.replace("FAULT_TIMER", "RPi: BOOTSYS\nFAULT_TIMER"))),
            ("boot", lambda a, b: (a, b.replace("Halt: power_off: 1", ""))),
        ):
            with self.subTest(name=name):
                self.assertEqual(self.run_rows(phase="poweroff", edit=edit)[name], INCOMPLETE)
        rows = self.run_rows(edit=lambda a, b: (a, b.replace(BUNDLE_ID, "abcd" * 4)))
        self.assertEqual(rows["bundle"], FAIL)

    def test_saturation_and_unavailable_usb_are_incomplete(self):
        def edit(a, b):
            b = b.replace("after=128", "after=65535")
            return a, b.replace("probe=yes", "probe=no")
        rows = self.run_rows(edit=edit)
        self.assertEqual((rows["scmi"], rows["usb"]), (INCOMPLETE, INCOMPLETE))

    def test_manifest_requires_matching_elf(self):
        digest = "c" * 64
        manifest = f"bundle-id {BUNDLE_ID * 4}\n{digest}  rp1.elf\n"
        self.assertEqual(vf.manifest_tuple(manifest, digest), (PASS, BUNDLE_ID))
        self.assertEqual(vf.manifest_tuple(manifest, "a" * 64), (FAIL, BUNDLE_ID))
        self.assertEqual(vf.manifest_tuple(manifest, None), (INCOMPLETE, None))
        self.assertEqual(vf.manifest_tuple(None, digest), (INCOMPLETE, None))

    def test_static_bound_needs_stack_output_and_768_byte_budget(self):
        for output, rc, want in (
            ("stack_static=PASS s_plat=580 margin=768", 0, PASS),
            ("stack_static=PASS s_plat=769 margin=769", 0, FAIL),
            ("stack_static=FAIL reason=margin s_plat=769 margin=768", 1, FAIL),
            ("stack_static=PASS s_plat=580", 0, INCOMPLETE),
            (None, None, INCOMPLETE),
        ):
            self.assertEqual(vf.stack_check(output, rc)[0], want)

    def test_cli_judges_a_complete_matrix_from_files(self):
        with TemporaryDirectory() as tmp:
            root = Path(tmp)
            elf = root / "rp1.elf"
            elf.write_bytes(b"synthetic ELF")
            manifest = root / "manifest.txt"
            digest = hashlib.sha256(elf.read_bytes()).hexdigest()
            manifest.write_text(f"bundle-id {BUNDLE_ID * 4}\n{digest}  rp1.elf\n")
            args = ["--elf", str(elf), "--manifest", str(manifest)]
            for i, (kind, phase) in enumerate((k, p) for k in vf.KINDS for p in vf.PHASES):
                nonce = f"{i + 1:08x}"
                uart0, uart10 = captures(kind, phase, nonce, f"{BOOT_ID[:-2]}{i:02x}")
                paths = [root / f"{nonce}-uart{port}.raw" for port in (0, 10)]
                for path, uart in zip(paths, (uart0, uart10)):
                    path.write_text(uart)
                args.extend(["--run", kind, phase, nonce, *(str(p) for p in paths)])
            output = io.StringIO()
            stack = "stack_static=PASS s_plat=580 margin=768"
            with contextlib.redirect_stdout(output), patch.object(vf.plat, "run_stack_paths",
                                                                 return_value=(stack, 0)):
                self.assertEqual(vf.main(args), 0)
            self.assertIn("PLAT_VERDICT=PASS", output.getvalue())
            self.assertIn("PLAT_LOST=INCOMPLETE", output.getvalue())
            self.assertIn("FAULT_DIAG kind=irq57-panic result=INCOMPLETE", output.getvalue())

    def test_matrix_requires_every_unique_cell_and_static_pass(self):
        runs = [(kind, phase, f"{i:08x}", str(i), PASS)
                for i, (kind, phase) in enumerate((k, p) for k in vf.KINDS for p in vf.PHASES)]
        self.assertEqual(vf.matrix(runs, PASS)[1], PASS)
        self.assertEqual(vf.matrix(runs, FAIL)[1], FAIL)
        self.assertEqual(vf.matrix(runs, INCOMPLETE)[1], INCOMPLETE)
        self.assertEqual(vf.matrix(runs[:-1], PASS)[1], INCOMPLETE)
        self.assertEqual(vf.matrix(runs + [runs[0]], PASS)[1], FAIL)
        for duplicate_field in (2, 3):
            first = list(runs[0])
            first[duplicate_field] = runs[1][duplicate_field]
            self.assertEqual(vf.matrix([first, *runs[1:]], PASS)[1], FAIL)
        self.assertEqual(vf.matrix(runs + [("stkerr", "cold-1", "a", "b", FAIL)], PASS)[1], PASS)
        failed = (*runs[0][:4], FAIL)
        per_kind, verdict = vf.matrix([failed, *runs[1:]], PASS)
        self.assertEqual((per_kind[vf.KINDS[0]], verdict), (FAIL, FAIL))


if __name__ == "__main__":
    unittest.main()
