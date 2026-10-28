#!/usr/bin/env python3
"""Read-only NACK probes while the ESP32 RMT observes SCL; no bus writes."""
import datetime
import fcntl
import os
from pathlib import Path
import re
import select
import subprocess
import sys
import time
import serial

def periods(text):
    # Initial idle HIGH and terminal idle LOW are not complete clock periods.
    pulses = [(int(level), int(ticks)) for level, ticks in re.findall(r"([01]):(\d+)", text)]
    return [pulses[i][1] + pulses[i + 1][1] for i in range(len(pulses) - 1)
            if pulses[i][0] == 0 and pulses[i + 1][0] == 1
            and 0 < pulses[i][1] < 500 and 0 < pulses[i + 1][1] < 500]

def main():
    if sys.argv[1:] == ["--self-test"]:
        assert periods("RMT items=3 1:800,0:60 1:40,0:60 1:40,0:0") == [100, 100]
        assert periods("RMT empty") == []
        print("PASS RMT period extraction")
        return
    out = Path(sys.argv[1]); out.mkdir(parents=True, exist_ok=True)
    root = Path(os.environ.get("CM5_HACK_ROOT", "<lab>"))
    with open("/tmp/codex-rp1-priority-hardware.lock", "a") as lock:
        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        holders = subprocess.run(["fuser", "/dev/cm5-uart0", "/dev/cm5-uart10"], capture_output=True)
        if holders.returncode != 1:
            raise RuntimeError("UARTs busy or fuser failed")
        with (out / "commands.txt").open("w") as commands:
            def ctl(command):
                result = subprocess.run([str(root / "scripts/cm5ctl.py"), command], capture_output=True, text=True, check=True)
                commands.write(f"{datetime.datetime.now().isoformat()} {command}\n{result.stdout}{result.stderr}")
                commands.flush()
                return result.stdout
            with serial.Serial(None, baudrate=115200, timeout=0, exclusive=True) as uart0, \
                 serial.Serial(None, baudrate=115200, timeout=0, exclusive=True) as uart10, \
                 (out / "uart0.raw").open("wb") as log0, (out / "uart10.raw").open("wb") as log10, \
                 (out / "timestamps.txt").open("w") as stamps:
                for uart, port, dtr in [(uart0, "/dev/cm5-uart0", False), (uart10, "/dev/cm5-uart10", True)]:
                    uart.dtr = dtr; uart.rts = False; uart.port = port; uart.open()
                try:
                    for request in [101, 102, 103]:
                        ctl("wiring-arm")
                        for address, value in [(0x1f0040c02c, 0), (0x1f0040c028, request)]:
                            uart10.write(f"busybox devmem 0x{address:x} 32 {value}\n".encode())
                            uart10.flush(); time.sleep(0.05)
                        deadline = time.monotonic() + 5
                        output = b""
                        while time.monotonic() < deadline:
                            ready, _, _ = select.select([uart0, uart10], [], [], 0.2)
                            for uart in ready:
                                data = uart.read(65536)
                                (log0 if uart is uart0 else log10).write(data)
                                stamps.write(f"{time.monotonic_ns()} uart{0 if uart is uart0 else 10} bytes={len(data)}\n")
                                if uart is uart0: output += data
                            if f"I2C_RESULT request={request} Ok(())".encode() in output: break
                        else: raise RuntimeError(f"probe {request} did not finish")
                        measured = ctl("wiring-status")
                        sample = periods(measured)
                        if len(sample) < 7: raise RuntimeError(f"incomplete RMT capture: {sample}")
                        mean = sum(sample) / len(sample)
                        expected = {101: 100_000, 102: 400_000, 103: 1_000_000}[request]
                        assert abs(10_000_000 / mean - expected) < expected * 0.1
                        print(f"request={request} periods={len(sample)} ticks={sample} SCL_hz={10_000_000/mean:.1f}", flush=True)
                        ctl("wiring-disarm")
                finally:
                    ctl("wiring-disarm"); ctl("debug-disarm"); ctl("status")

if __name__ == "__main__": main()
