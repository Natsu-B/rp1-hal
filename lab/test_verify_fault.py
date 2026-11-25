"""Synthetic raw UART captures for the PR-E verifier."""

import contextlib
import hashlib
import importlib.util
import io
import json
import struct
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
CLIENT_SHA256 = "c" * 64
UDF_PC = 0x20001234


def captures(kind="p0-thread-panic", phase="cold-1", nonce=NONCE, boot_id=BOOT_ID,
             bundle_id=BUNDLE_ID):
    code = vf.CODES[kind]
    core = int(kind == "p1-panic")
    state, reason, aux = "Panicked", 0, "0x0"
    if kind.endswith("udf"):
        state, reason, aux = "Fault", 6, "0x10000"
    if kind == "stkerr":
        state, reason, aux = "Fault", 3, "0x10010"

    ipsr = 42 if "-isr-" in kind else 0
    pc = UDF_PC if kind.endswith("udf") else 0
    reset = 0 if phase == "reboot" else 1
    previous = {
        "reboot": "[  42.23] reboot: Restarting system\n",
        "poweroff": "[  42.23] reboot: Power down\nRPi: BOOTSYS\npower-on-reset 0\nHalt: power_off: 1\nPower off\n",
    }.get(phase, "")
    tag = f"nonce={nonce} kind={kind}"
    uart0 = f"\x00FAULT_BOOT nonce={nonce} kinds={','.join(vf.CODES)} stkerr=1 irq57=unavailable\n"
    uart10 = (
        f"{previous}RPi: BOOTSYS release VERSION:abc\n"
        f"  1.00 POWER_OFF_ON_HALT: 1 WAIT_FOR_POWER_BUTTON 0 power-on-reset {reset}\n"
        f"[rp1boot     253] bundle ok: rp1.bundle={bundle_id} (3 elements, sha256 in 211 ms)\n"
        f"FAULT_BEGIN {tag} boot_id={boot_id} epoch=2 client_sha256={CLIENT_SHA256}\n"
        f"FAULT_PRE {tag} p0_before=25 p0_after=30 p1_before=26 p1_after=31\n"
        f"FAULT_REQUEST {tag} code={code}\n"
        f"FAULT_SITE {tag} code={code} core={core} ipsr={ipsr}\n"
        f"FAULT_STATUS {tag} core={core} ack={code} state={state} reason={reason} aux={aux} pc_or_line={pc:#x}"
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
            f"FAULT_DMESG {tag} scmi_timeouts=0 oops=0 link_errors=0\n"
            f"FAULT_END {tag} pcie=Up peer=Running\n"
        )
    return uart0, uart10.replace("\n", "\r\n")


def make_bundle(elf):
    kernel = b"synthetic kernel"
    bundle = bytearray(8192 + len(kernel))
    struct.pack_into("<8sIIIIQ", bundle, 0, b"RP1BNDL\0", 1, 128, 2, len(bundle), 0)
    for i, offset, data in ((1, 4096, elf), (2, 8192, kernel)):
        struct.pack_into("<IIII32s", bundle, 32 + (i - 1) * 48,
                         i, 0, offset, len(data), hashlib.sha256(data).digest())
        bundle[offset:offset + len(data)] = data
    return bytes(bundle), hashlib.sha256(bundle[:128]).hexdigest()


def transition_inputs(raw):
    at = raw.index(b"RPi: BOOTSYS release")
    boot = {"kind": "force", "complete": True, "pd_ok": True,
            "raw_start": at, "raw_end": len(raw)}
    events = [
        {"kind": "console_sendraw", "text": "systemctl poweroff\n", "mono": 1},
        {"kind": "carrier", "value": "0", "mono": 4},
        {"kind": "esp_command_start", "command": "force-boot", "mono": 5},
        {"kind": "esp_command_end", "command": "force-boot", "rc": 0, "mono": 7},
    ]
    stamps = [
        {"port": 1, "offset": raw.index(anchor), "bytes": len(anchor), "monotonic_ns": when * 10**9}
        for anchor, when in ((b"reboot: Power down", 2), (b"Power off", 3),
                             (b"RPi: BOOTSYS release", 6))
    ]
    return boot, events, stamps


HALT = ("[  40.69] reboot: Power down\r\n  0.98 RPi: BOOTSYS release VERSION:abc\r\n"
        "  1.00 POWER_OFF_ON_HALT: 1 WAIT_FOR_POWER_BUTTON 0 power-on-reset 0\r\n"
        "  1.02 Halt: power_off: 1\r\n\r\n  1.02 Power off\r\n\x00\r\n")


def cold_shutdown_inputs(raw):
    """boot.py and capture records of a cold force-boot that Linux answered with a halt."""
    boot = {"kind": "force", "complete": True, "pd_ok": True, "raw_start": 0, "raw_end": len(raw)}
    events = [
        {"kind": "esp_command_start", "command": "force-boot", "mono": 1},
        {"kind": "carrier", "value": "0", "mono": 3},
        {"kind": "esp_command_end", "command": "force-boot", "rc": 0, "mono": 6},
        {"kind": "console_sendraw", "text": "( true ); echo END_''12345678 rc=$?\n", "mono": 9},
    ]
    final = raw.index(b"RPi: BOOTSYS release", raw.index(b"Power off"))
    stamps = [
        {"port": 1, "offset": at, "bytes": 8, "monotonic_ns": when * 10**9}
        for at, when in ((raw.index(b"reboot: Power down"), 2), (raw.index(b"Power off"), 4),
                         (final, 5))
    ]
    return boot, events, stamps


class TestVerifyFault(unittest.TestCase):
    def run_result(self, kind="p0-thread-panic", phase="cold-1", edit=lambda a, b: (a, b), **kwargs):
        uart0, uart10 = edit(*captures(kind, phase))
        if phase == "poweroff" and "force_at" not in kwargs:
            kwargs["force_at"] = uart10.find("RPi: BOOTSYS release")
        return vf.verify_run(kind, phase, NONCE, uart0, uart10, BUNDLE_ID,
                             CLIENT_SHA256, udf_pc=UDF_PC, **kwargs)

    def run_rows(self, *args, **kwargs):
        return self.run_result(*args, **kwargs)[0]

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
        rows, _, _ = vf.verify_run("p0-thread-panic", "cold-1", NONCE, uart0, uart10,
                                   BUNDLE_ID, CLIENT_SHA256)
        self.assertEqual(rows["boot"], FAIL)

    def test_all_required_evidence_must_be_present(self):
        cases = (
            ("identity", "BOOT", "uart0"),
            ("identity", "BEGIN", "uart10"),
            ("boot", "power-on-reset", "uart10"),
            ("bundle", "bundle ok", "uart10"),
            ("request", "REQUEST", "uart10"),
            ("site", "SITE", "uart10"),
            ("terminal", "STATUS", "uart10"),
            ("timer_live_before", "PRE", "uart10"),
            ("timer", "TIMER", "uart10"),
            ("ping", "PING", "uart10"),
            ("usb", "USB", "uart10"),
            ("scmi", "SCMI", "uart10"),
            ("scmi", "RATE", "uart10"),
            ("dmesg", "DMESG", "uart10"),
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
            (f"pc_or_line={UDF_PC:#x}", "pc_or_line=0x1234", FAIL),
        ):
            rows = self.run_rows("p0-isr-udf", edit=lambda a, b: (a, b.replace(old, new)))
            self.assertEqual(rows["terminal"], want)

    def test_proc1_requires_proc0_to_continue(self):
        for old, new, want in (
            ("p0_after=60", "p0_after=40", FAIL),
            ("p0_after=60", "", INCOMPLETE),
            ("peer=Running", "peer=Panicked", FAIL),
            ("pcie=Up peer=Running", "pcie=Up peer=Panicked", FAIL),
        ):
            rows = self.run_rows("p1-panic", edit=lambda a, b: (a, b.replace(old, new)))
            self.assertEqual(rows["peer"], want)

    def test_irq57_and_stacking_diagnostics(self):
        self.assertEqual(self.run_rows("stkerr")["terminal"], PASS)
        for old, new, want in (
            ("state=Fault", "state=Running", INCOMPLETE),
            ("reason=3", "reason=6", FAIL),
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

    def test_manifest_requires_matching_deployed_payload_and_header(self):
        elf = b"ELF A"
        bundle, bundle_id = make_bundle(elf)
        a = hashlib.sha256(elf).hexdigest()
        b = hashlib.sha256(b"ELF B").hexdigest()
        manifest = f"bundle-id {bundle_id}\n{a}  rp1.elf\n"
        match = (PASS, bundle_id[:16])
        self.assertEqual(vf.artifact_identity(manifest, bundle, a), match)
        # The manifest's ELF hash and the standalone ELF may come from build B.
        self.assertEqual(vf.artifact_identity(manifest.replace(a, b), bundle, b)[0], FAIL)
        self.assertEqual(vf.artifact_identity(manifest.replace(bundle_id, "f" * 64), bundle, a)[0], FAIL)
        self.assertEqual(vf.artifact_identity(manifest, bundle.replace(b"ELF A", b"ELF B"), a)[0], FAIL)
        self.assertEqual(vf.artifact_identity(manifest, bundle[:-1], a)[0], FAIL)
        self.assertEqual(vf.artifact_identity(manifest, None, a), (INCOMPLETE, None))
        self.assertEqual(vf.artifact_identity(None, bundle, a), (INCOMPLETE, None))

    def test_post_terminal_rows_require_status_even_on_measured_failure(self):
        def remove_status(a, b):
            b = "\n".join(line for line in b.splitlines() if not line.startswith("FAULT_STATUS"))
            return a, b.replace("received=20 rc=0", "received=19 rc=1")
        rows, _, notes = self.run_result(edit=remove_status)
        self.assertTrue(all(rows[row] == INCOMPLETE for row in vf.POST_ROWS))
        self.assertEqual(notes, {"ping": "measured=FAIL"})
        for edit, options in (
            (lambda a, b: (a, b), {"artifact": FAIL}),
            (lambda a, b: (a.replace(NONCE, "badbaaad"), b), {}),
            (lambda a, b: (a, b.replace(CLIENT_SHA256, "d" * 64)), {}),
        ):
            rows = self.run_rows(edit=edit, **options)
            self.assertTrue(all(rows[row] == INCOMPLETE for row in vf.POST_ROWS))

    def test_counter_history_and_peer_survival(self):
        for row, old, new, want in (
            ("timer", "p0_before=40 p0_after=40", "p0_before=0 p0_after=0", FAIL),
            ("timer", "p0_before=40 p0_after=40", "p0_before=4294967295 p0_after=4294967295", INCOMPLETE),
            ("timer_live_before", "p1_after=31", "p1_after=0xffffffff", INCOMPLETE),
            ("scmi", "scmi=Serving(requests=126)", "scmi=Serving(requests=200)", FAIL),
            ("peer", "peer=Running pcie=Up", "peer=Fault pcie=Up", FAIL),
            ("peer", "pcie=Up peer=Running", "pcie=Up peer=Panicked", FAIL),
        ):
            with self.subTest(row=row, new=new):
                self.assertEqual(self.run_rows(edit=lambda a, b: (a, b.replace(old, new)))[row], want)

    def test_site_requires_actual_context_and_kind(self):
        for old, new in (("ipsr=42", "ipsr=0"), ("code=2 core=0", "code=1 core=0"),
                         ("code=2 core=0", "code=2 core=1")):
            rows = self.run_rows("p0-isr-panic", edit=lambda a, b: (a, b.replace(old, new)))
            self.assertEqual((rows["request"], rows["site"], rows["terminal"]), (PASS, FAIL, INCOMPLETE))

    def test_kernel_diagnostics_fail_even_when_other_probes_pass(self):
        for field in ("scmi_timeouts", "oops", "link_errors"):
            rows = self.run_rows(edit=lambda a, b: (a, b.replace(f"{field}=0", f"{field}=1")))
            self.assertEqual(rows["dmesg"], FAIL)
            self.assertEqual(vf.overall(rows.values()), FAIL)
        rows = self.run_rows(edit=lambda a, b: (a, b.replace("FAULT_BEGIN",
                            "arm-scmi: SCMI max-rx-timeout: 30ms\nFAULT_BEGIN")))
        self.assertEqual(rows["dmesg"], PASS)

    def test_shutdown_must_immediately_precede_operating_boot(self):
        extra_boot = "RPi: BOOTSYS unrelated\nLinux version 6.12\n"
        for phase in ("reboot", "poweroff"):
            rows = self.run_rows(phase=phase, edit=lambda a, b: (a, b.replace(
                "RPi: BOOTSYS release", extra_boot + "RPi: BOOTSYS release", 1)))
            self.assertEqual(rows["boot"], INCOMPLETE)
        self.assertEqual(self.run_rows(phase="poweroff", force_at=None)["boot"], INCOMPLETE)

    def test_poweroff_accepts_timestamped_bootloader_link_wait_preamble(self):
        uart0, uart10 = captures(phase="poweroff")
        boundary = uart10.index("RPi: BOOTSYS release")
        preamble = ("\x00\r\n  0.98 RPi: BOOTSYS release VERSION:abc\r\n"
                    "  1.00 power-on-reset 1\r\n"
                    "  4.88 NETWORK: wait for link TFTP: 0.0.0.0\r\n")
        prefix, final_boot = uart10[:boundary], "  0.98 " + uart10[boundary:]
        for extra, want in (("", PASS), (preamble, PASS),
                            (preamble + "Boot mode: SD (01)\r\n", INCOMPLETE),
                            (preamble + "Linux version 6.12\r\n", INCOMPLETE)):
            rows, _, _ = vf.verify_run("p0-thread-panic", "poweroff", NONCE, uart0,
                prefix + extra + final_boot, BUNDLE_ID, CLIENT_SHA256, force_at=boundary)
            self.assertEqual(rows["boot"], want)

    def test_poweroff_needs_carrier_and_successful_force_in_same_capture(self):
        _, uart10 = captures(phase="poweroff")
        raw = uart10.encode()
        boot, events, stamps = transition_inputs(raw)
        self.assertIsNotNone(vf.force_boundary(raw, boot, events, stamps))
        self.assertIsNone(vf.force_boundary(raw, boot, [], stamps))
        self.assertIsNone(vf.force_boundary(raw, boot, [events[0], *events[2:]], stamps))
        self.assertIsNone(vf.force_boundary(raw, boot, [*events[:-1], dict(events[-1], rc=1)], stamps))
        self.assertIsNone(vf.force_boundary(raw, boot, events, stamps[:-1]))
        self.assertIsNone(vf.force_boundary(raw, boot, events, [*stamps[:-1], dict(stamps[-1], monotonic_ns=3)]))

    def test_cold_accepts_halt_caused_by_its_own_force_boot(self):
        uart0, uart10 = captures(phase="cold-1")
        uart10 = HALT + uart10
        raw = uart10.encode()
        boot, events, stamps = cold_shutdown_inputs(raw)

        def boot_row(boot=boot, events=events, stamps=stamps, text=uart10):
            at = vf.shutdown_boundary(text.encode(), boot, events, stamps)
            rows, _, notes = vf.verify_run("p0-thread-panic", "cold-1", NONCE, uart0, text,
                                           BUNDLE_ID, CLIENT_SHA256, shutdown_at=at)
            return rows["boot"], notes.get("boot")

        self.assertEqual(boot_row(), (PASS, "detail=cold-via-shutdown"))
        self.assertEqual(self.run_rows()["boot"], PASS)
        self.assertNotIn("boot", self.run_result()[2])
        early = [dict(e, mono=e["mono"] + 10) if e.get("command") == "force-boot" else e
                 for e in events]
        commanded = [*events, {"kind": "console_sendraw", "text": "systemctl poweroff\n", "mono": 0.5}]
        rebooted = [*events, {"kind": "console_sendraw", "text": "systemctl reboot\n", "mono": 9}]
        failed = [dict(e, rc=1) if e.get("kind") == "esp_command_end" else e for e in events]
        for name, case in (
            ("shutdown before force start", dict(events=early)),
            ("missing stamps", dict(stamps=stamps[1:])),
            ("no stamps", dict(stamps=[])),
            ("no force record", dict(events=[])),
            ("force failed", dict(events=failed)),
            ("incomplete boot.py record", dict(boot=dict(boot, complete=False))),
            ("recorded poweroff command", dict(events=commanded)),
            ("recorded reboot command", dict(events=rebooted)),
            ("reset mode 0 after halt", dict(text=uart10.replace(
                "power-on-reset 1", "power-on-reset 0"))),
            ("no halt banner", dict(text=uart10.replace("Halt: power_off: 1", "Halt"))),
        ):
            with self.subTest(name=name):
                self.assertEqual(boot_row(**case), (INCOMPLETE, None))
        rows, _, _ = vf.verify_run("p0-thread-panic", "cold-1", NONCE, uart0, uart10,
                                   BUNDLE_ID, CLIENT_SHA256)
        self.assertEqual(rows["boot"], INCOMPLETE)

    def test_static_bound_needs_stack_output_and_768_byte_budget(self):
        for output, rc, want in (
            ("stack_fault_setup=PASS\nstack_static=PASS s_plat=580 margin=768", 0, PASS),
            ("stack_static=PASS s_plat=572 margin=768", 0, INCOMPLETE),
            ("stack_fault_setup=PASS\nstack_fault_setup=FAIL\nstack_static=PASS s_plat=580 margin=768", 0, FAIL),
            ("stack_fault_setup=PASS\nstack_fault_setup=PASS\nstack_static=PASS s_plat=580 margin=768", 0, FAIL),
            ("stack_fault_setup=PASS\nstack_static=PASS s_plat=769 margin=769", 0, FAIL),
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
            bundle, header = make_bundle(elf.read_bytes())
            bundle_path = root / "rp1-bundle.img"
            bundle_path.write_bytes(bundle)
            manifest.write_text(f"bundle-id {header}\n{digest}  rp1.elf\n")
            args = ["--elf", str(elf), "--manifest", str(manifest),
                    "--bundle", str(bundle_path), "--client-sha256", CLIENT_SHA256]
            for i, (kind, phase) in enumerate((k, p) for k in vf.KINDS for p in vf.PHASES):
                nonce = f"{i + 1:08x}"
                uart0, uart10 = captures(kind, phase, nonce, f"{BOOT_ID[:-2]}{i:02x}", header[:16])
                case = root / nonce / "continuous"
                case.mkdir(parents=True)
                paths = [case / f"uart{port}.raw" for port in (0, 10)]
                for path, uart in zip(paths, (uart0, uart10)):
                    path.write_bytes(uart.encode())
                if phase == "poweroff":
                    boot, events, stamps = transition_inputs(uart10.encode())
                    (case.parent / "boot.json").write_text(json.dumps(boot))
                    for name, records in (("events", events), ("timestamps", stamps)):
                        (case / f"{name}.jsonl").write_text("\n".join(map(json.dumps, records)))
                args.extend(["--run", kind, phase, nonce, *(str(p) for p in paths)])
            output = io.StringIO()
            stack = "stack_fault_setup=PASS\nstack_static=PASS s_plat=580 margin=768"
            with (contextlib.redirect_stdout(output), patch.object(vf.plat, "run_stack_paths",
                    return_value=(stack, 0)), patch.object(vf, "udf_address", return_value=UDF_PC)):
                self.assertEqual(vf.main(args), 0)
            self.assertIn("PLAT_VERDICT=PASS", output.getvalue())
            self.assertIn("PLAT_LOST=INCOMPLETE", output.getvalue())
            self.assertIn("FAULT_DIAG kind=irq57-panic result=INCOMPLETE", output.getvalue())

    def test_matrix_requires_every_unique_cell_and_static_pass(self):
        runs = [(kind, phase, f"{i:08x}", str(i), PASS, i)
                for i, (kind, phase) in enumerate((k, p) for k in vf.KINDS for p in vf.PHASES)]
        self.assertEqual(vf.matrix(runs, PASS)[1], PASS)
        self.assertEqual(vf.matrix(runs, FAIL)[1], FAIL)
        self.assertEqual(vf.matrix(runs, INCOMPLETE)[1], INCOMPLETE)
        self.assertEqual(vf.matrix(runs[:-1], PASS)[1], INCOMPLETE)
        self.assertEqual(vf.matrix(runs + [runs[0]], PASS)[1], FAIL)
        for duplicate_field in (3, 5):
            first = list(runs[0])
            first[duplicate_field] = runs[1][duplicate_field]
            self.assertEqual(vf.matrix([first, *runs[1:]], PASS)[1], FAIL)
        colliding = (*runs[0][:2], runs[1][2], *runs[0][3:])
        self.assertEqual(vf.matrix([colliding, *runs[1:]], PASS)[1:], (INCOMPLETE, "duplicate-nonce"))
        diagnostic = ("stkerr", "cold-1", "a", "b", FAIL, 30)
        self.assertEqual(vf.matrix(runs + [diagnostic], PASS)[1], PASS)
        failed = (*runs[0][:4], FAIL, runs[0][5])
        per_kind, verdict, _ = vf.matrix([failed, *runs[1:]], PASS)
        self.assertEqual((per_kind[vf.KINDS[0]], verdict), (FAIL, FAIL))


if __name__ == "__main__":
    unittest.main()
