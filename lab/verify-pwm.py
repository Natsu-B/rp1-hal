#!/usr/bin/env python3
"""Check one boot of pwm-loopback from its UART0 capture."""
import argparse
from pathlib import Path
import re

DUTIES = (0, 25, 50, 75, 100)
ROUNDS = 60
PADS = ((22, "Up", 0xc8), (22, "Down", 0xc4), (23, "Down", 0xc4))
ANSI = re.compile(r"\x1b\[[0-?]*[ -/]*[@-~]|\x1b[@-Z\\-_]")


def wave(fields, prefix, hz, duty):
    rise, high, total, elapsed = (fields[prefix + k] for k in ("rise", "high", "total", "us"))
    if not 0 <= high <= total or total == 0 or elapsed < 100_000:
        return False
    if duty in (0, 100):
        return rise == 0 and high == (total if duty else 0)
    return abs(rise * 1_000_000 / elapsed - hz) <= hz * 0.05 and abs(high * 100 / total - duty) <= 5


def verify(uart0):
    """Return the first failed condition or None on success."""
    uart0 = ANSI.sub("", uart0).replace("\r", "").replace("\0", "")
    if re.search(r"\b(?:PWM_FAIL|panic|panicked|hardfault|RP1_FAULT)\b", uart0, re.I):
        return "firmware reported a failure"
    rows = re.findall(r"\b(PWM_[A-Z]+)\b[ \t]*(.*)", uart0, re.M)
    round_markers = ["PWM_SAMPLE"] * len(DUTIES) + ["PWM_REPERIOD", "PWM_DISABLED"]
    if [name for name, _ in rows] != ["PWM_BOOT"] + ["PWM_PAD"] * 3 + round_markers * ROUNDS + ["PWM_REINIT", "PWM_DONE"]:
        return "wrong count or order of PWM markers"
    boot = re.fullmatch(r"nonce=([0-9a-f]{8}) PLAT_INIT ok=1(?: .*)?", rows[0][1].strip())
    if not boot or int(boot[1], 16) in (0, 0xffffffff):
        return "invalid boot nonce or platform status"
    for (_, fields), (pin, pull, bits) in zip(rows[1:4], PADS):
        pad = re.fullmatch(r"nonce=([0-9a-f]{8}) pin=(\d+) pull=(\w+) raw=0x([0-9a-f]+)", fields.strip())
        if not pad or (pad[1], int(pad[2]), pad[3], int(pad[4], 16) & 0xcc) != (boot[1], pin, pull, bits):
            return f"invalid PWM_PAD for GPIO{pin} {pull}"

    samples = iter(rows[4:-2])
    for n in range(ROUNDS):
        for duty in (*DUTIES, None, None):
            marker, data = next(samples)
            fields = {k: int(v) for k, v in re.findall(r"\b(\w+)=(\d+)(?=\s|$)", data)}
            try:
                ok = fields["round"] == n
                if marker == "PWM_SAMPLE":
                    ok &= fields["duty"] == duty and wave(fields, "a_", 1000, duty) and wave(fields, "b_", 500, 50)
                elif marker == "PWM_REPERIOD":
                    ok &= wave(fields, "", 500, 25)
                else:
                    ok &= fields["rise"] == fields["high"] == 0 and fields["total"] > 0
            except KeyError:
                ok = False
            if not ok:
                return f"invalid {marker} waveform for round {n}: {data}"
    try:
        fields = {k: int(v) for k, v in re.findall(r"\b(\w+)=(\d+)(?=\s|$)", rows[-2][1])}
        reinitialized = wave(fields, "a_", 1000, 50) and wave(fields, "b_", 500, 50)
    except KeyError:
        reinitialized = False
    if not reinitialized:
        return "invalid PWM_REINIT waveform"
    if rows[-1][1].strip() != "reinitialized=1":
        return "missing PWM reinitialization"
    return None


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--uart0", required=True, type=Path)
    args = parser.parse_args()
    failure = verify(args.uart0.read_text(errors="replace"))
    if failure:
        print(f"PWM_VERDICT=FAIL {failure}")
        return 1
    print("PWM_VERDICT=PASS pads=3 samples=300 reperiod=60 disabled=60")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
