#!/usr/bin/env python3
"""Strict pass/fail for the Linux SHM / doorbell / I2C / ADC / SPI regression (`lab/check-shm.sh`).

Usage: verify-shm.py --uart0 FILE --uart10 FILE --result RESULT_JSON
       verify-shm.py --uart1-host FILE --uart1-nonce HEX --uart1-console FILE [--uart1-result JSON]
       (both groups may be given together)

PASS only if the capture completed (`"complete": true` in the observer's JSON), every required
marker is present on its UART (UART10 ones in order), and no failure marker appears. A missing
marker, a non-zero exit code, a timeout (no terminal marker / incomplete capture) is FAIL.
The generic boot marker report is not evidence for this regression. Exit 0 = PASS, 1 = FAIL,
2 = usage error. Paths come from the arguments only.

Linux UART1 (`lab/check-uart1.sh` with `lab/uart1-peer.py` on the host): UART1_TX_PASS when the
console shows `UART1_TX_SENT nonce=<nonce>` and the host capture holds `RP1SHARE_TX_<nonce>_<i>`
lines; UART1_RX_PASS when the console shows `UART1_RX_GOT=RP1SHARE_RX_<nonce>` (read from the
wire by Linux). Both need `UART1_CHECK_DONE` and, if given, a complete capture.
"""
import argparse
import json
import re
import sys

# UART10 (Linux console), in the order check-shm.sh prints them.
UART10_ORDERED = [
    ("doorbell before", r"\bDOORBELL_BEFORE_RC=0\b"),
    ("SHM_PASS", r"\bSHM_PASS\b"),
    ("SHM_RC=0", r"\bSHM_RC=0\b"),
    ("doorbell after", r"\bDOORBELL_AFTER_RC=0\b"),
    ("I2C reply", r"\bI2C_REPLY=0x0*1\b"),
    ("ADC node disabled", r"\bADC_DT=\S+ status=disabled(?!\S)"),
    ("ADC reply", r"\bADC_REPLY=0x0*[aA][dD][cC]\b"),
    ("SPI node disabled", r"\bSPI_DT=\S+ status=disabled(?!\S)"),
    ("SPI reply", r"\bSPI_REPLY=0x0*5[bB]1\b"),
    ("script result", r"\bCHECK_SHM_RESULT=PASS\b"),
    ("terminal marker", r"\bDDR_INSPECT_END\b"),
]
# Linux must not have any node of an RP1-owned block enabled (status other than disabled).
UART10_FORBIDDEN = [r"\bCHECK_FAIL\b", r"\bCHECK_SHM_RESULT=FAIL\b", r"Kernel panic", r"\bSError\b",
                    r"\b(ADC|SPI)_DT=(missing\b|\S+ status=(?!disabled(?!\S)))"]
# UART0 (RP1 firmware log).
UART0_REQUIRED = [
    ("I2C 100 kHz", r"\bI2C_PASS hz=100000\b"),
    ("I2C 400 kHz", r"\bI2C_PASS hz=400000\b"),
    ("I2C r64_w16", r"\bI2C_PASS r64_w16\b"),
    ("I2C result", r"\bI2C_RESULT request=1 Ok\b"),
    ("ADC boot burst", r"\bADC_P1BOOT first=\d+ temp_mc=-?\d+ PASS\b"),
    ("ADC boot result", r"\bADC_RESULT boot Ok\b"),
    ("ADC request burst", r"\bADC_P1REQ first=\d+ temp_mc=-?\d+ PASS\b"),
    ("ADC request result", r"\bADC_RESULT request=0xadc Ok\b"),
    ("SPI boot proof", r"\bSPI_P1BOOT depth=\d+ .*\bPASS\b"),
    ("SPI boot result", r"\bSPI_RESULT boot Ok\b"),
    ("SPI 1 MHz", r"\bSPI_PASS P1REQ hz=1000000 "),
    ("SPI 10 MHz", r"\bSPI_PASS P1REQ hz=10000000 "),
    ("SPI request proof", r"\bSPI_P1REQ depth=\d+ .*\bPASS\b"),
    ("SPI request result", r"\bSPI_RESULT request=0x5b1 Ok\b"),
]
UART0_PROFILE_REQUIRED = [
    ("profile done", r"\bPROF_DONE\b"),
    ("profile command", r"\bproc1 shm cmd=9 count=\d+ errors=0\b"),
]
UART0_FORBIDDEN = [r"\bI2C_FAIL\b", r"\bpanic", r"\bI2C_RESULT request=1 Err\b", r"\bADC_RESULT\b.*\bErr\b",
                   r"\bSPI_RESULT\b.*\bErr\b", r"\bspi0 init Err\b"]


def verify(uart0, uart10, result, profile=False):
    """Returns the list of failures (empty = PASS)."""
    failures = []
    if not isinstance(result, dict) or result.get("complete") is not True:
        failures.append("capture not complete (timeout or missing terminal marker)")
    ordered = list(UART10_ORDERED)
    profile_pattern = r"\bPROFILE rp1_us=\d+ \(results on UART0\)" if profile else r"\bPROFILE skipped\b"
    ordered.insert(1, ("profile mode", profile_pattern))
    at = 0
    for name, pattern in ordered:
        m = re.compile(pattern).search(uart10, at)
        if m is None:
            missing = "missing" if re.search(pattern, uart10) is None else "out of order"
            failures.append(f"UART10 {name}: {missing} ({pattern})")
        else:
            at = m.end()
    for name, pattern in UART0_REQUIRED + (UART0_PROFILE_REQUIRED if profile else []):
        if re.search(pattern, uart0) is None:
            failures.append(f"UART0 {name}: missing ({pattern})")
    for text, label, patterns in ((uart10, "UART10", UART10_FORBIDDEN), (uart0, "UART0", UART0_FORBIDDEN)):
        for pattern in patterns:
            line = next((l for l in text.splitlines() if re.search(pattern, l)), None)
            if line is not None:
                failures.append(f"{label} failure marker: {line.strip()[:160]}")
    return failures


def verify_uart1(console, host, nonce, result=None):
    """Returns (tx_failures, rx_failures) for the Linux UART1 check (empty = PASS)."""
    common = []
    if not re.fullmatch(r"[0-9a-f]{8,}", nonce or ""):
        common.append(f"nonce {nonce!r} is not at least 8 lowercase hex digits")
    if result is not None and (not isinstance(result, dict) or result.get("complete") is not True):
        common.append("UART1 capture not complete (timeout or missing terminal marker)")
    if re.search(r"\bUART1_CHECK_DONE\b", console) is None:
        common.append("UART1_CHECK_DONE missing")
    tx, rx = list(common), list(common)
    n = re.escape(nonce or "")
    if re.search(rf"\bUART1_TX_SENT nonce={n} ", console) is None:
        tx.append("Linux did not report UART1_TX_SENT for this nonce")
    if re.search(rf"^RP1SHARE_TX_{n}_\d+$", host, re.M) is None:
        tx.append(f"host capture has no RP1SHARE_TX_{nonce}_<i> line")
    got = re.search(r"\bUART1_RX_GOT=(\S*)", console)
    if got is None or got.group(1) != f"RP1SHARE_RX_{nonce}":
        rx.append(f"Linux read {got.group(1) if got else None!r}, not RP1SHARE_RX_{nonce}")
    return tx, rx


def read_text(path):
    with open(path, "rb") as f:
        return f.read().decode("utf-8", errors="replace").replace("\r", "")


def main(argv):
    p = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    p.add_argument("--profile", action="store_true", help="require SRAM profile completion (both binaries built with sram-profile)")
    p.add_argument("--uart0")
    p.add_argument("--uart10")
    p.add_argument("--result", help="observer JSON with the `complete` flag")
    p.add_argument("--uart1-host", help="host peer capture (lab/uart1-peer.py --out)")
    p.add_argument("--uart1-nonce")
    p.add_argument("--uart1-console", help="UART10 capture of lab/check-uart1.sh")
    p.add_argument("--uart1-result", help="observer JSON of that capture")
    a = p.parse_args(argv)
    shm = [a.uart0, a.uart10, a.result]
    uart1 = [a.uart1_host, a.uart1_nonce, a.uart1_console]
    if any(shm) and not all(shm) or any(uart1) and not all(uart1) or not any(shm + uart1):
        p.error("give --uart0/--uart10/--result and/or --uart1-host/--uart1-nonce/--uart1-console")
    if a.profile and not all(shm):
        p.error("--profile needs --uart0/--uart10/--result")
    failed = False
    if all(shm):
        result = load_result(a.result)
        failures = verify(read_text(a.uart0), read_text(a.uart10), result, profile=a.profile)
        for f in failures:
            print(f"FAIL {f}")
        print("SHM_REGRESSION=" + ("FAIL" if failures else "PASS"))
        failed |= bool(failures)
    if all(uart1):
        result = load_result(a.uart1_result) if a.uart1_result else None
        tx, rx = verify_uart1(read_text(a.uart1_console), read_text(a.uart1_host), a.uart1_nonce, result)
        for name, failures in (("TX", tx), ("RX", rx)):
            for f in failures:
                print(f"FAIL UART1 {name}: {f}")
            print(f"UART1_{name}_" + ("FAIL" if failures else "PASS"))
            failed |= bool(failures)
    return 1 if failed else 0


def load_result(path):
    try:
        with open(path) as f:
            return json.load(f)
    except (OSError, ValueError) as e:
        print(f"FAIL result JSON unreadable: {e}")
        return {}


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
