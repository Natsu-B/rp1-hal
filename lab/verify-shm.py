#!/usr/bin/env python3
"""Strict pass/fail for the Linux SHM / doorbell / I2C / ADC / SPI regression (`lab/check-shm.sh`).

Usage: verify-shm.py --uart0 FILE --uart10 FILE --result RESULT_JSON

PASS only if the capture completed (`"complete": true` in the observer's JSON), every required
marker is present on its UART (UART10 ones in order), and no failure marker appears. A missing
marker, a non-zero exit code, a timeout (no terminal marker / incomplete capture) is FAIL.
The generic boot marker report is not evidence for this regression. Exit 0 = PASS, 1 = FAIL,
2 = usage error. Paths come from the arguments only.
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
    ("profile done", r"\bPROF_DONE\b"),
    ("profile command", r"\bproc1 shm cmd=9 count=\d+ errors=0\b"),
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
UART0_FORBIDDEN = [r"\bI2C_FAIL\b", r"\bpanic", r"\bI2C_RESULT request=1 Err\b", r"\bADC_RESULT\b.*\bErr\b",
                   r"\bSPI_RESULT\b.*\bErr\b", r"\bspi0 init Err\b"]


def verify(uart0, uart10, result):
    """Returns the list of failures (empty = PASS)."""
    failures = []
    if not isinstance(result, dict) or result.get("complete") is not True:
        failures.append("capture not complete (timeout or missing terminal marker)")
    at = 0
    for name, pattern in UART10_ORDERED:
        m = re.compile(pattern).search(uart10, at)
        if m is None:
            missing = "missing" if re.search(pattern, uart10) is None else "out of order"
            failures.append(f"UART10 {name}: {missing} ({pattern})")
        else:
            at = m.end()
    for name, pattern in UART0_REQUIRED:
        if re.search(pattern, uart0) is None:
            failures.append(f"UART0 {name}: missing ({pattern})")
    for text, label, patterns in ((uart10, "UART10", UART10_FORBIDDEN), (uart0, "UART0", UART0_FORBIDDEN)):
        for pattern in patterns:
            line = next((l for l in text.splitlines() if re.search(pattern, l)), None)
            if line is not None:
                failures.append(f"{label} failure marker: {line.strip()[:160]}")
    return failures


def read_text(path):
    with open(path, "rb") as f:
        return f.read().decode("utf-8", errors="replace").replace("\r", "")


def main(argv):
    p = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    p.add_argument("--uart0", required=True)
    p.add_argument("--uart10", required=True)
    p.add_argument("--result", required=True, help="observer JSON with the `complete` flag")
    a = p.parse_args(argv)
    try:
        with open(a.result) as f:
            result = json.load(f)
    except (OSError, ValueError) as e:
        result = None
        print(f"FAIL result JSON unreadable: {e}")
    failures = verify(read_text(a.uart0), read_text(a.uart10), result)
    for f in failures:
        print(f"FAIL {f}")
    print("SHM_REGRESSION=" + ("FAIL" if failures else "PASS"))
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
