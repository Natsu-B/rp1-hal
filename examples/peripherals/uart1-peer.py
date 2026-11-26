#!/usr/bin/env python3
"""Send a nonce on the UART1 adapter after READY appears in a fresh UART0 capture."""
import argparse
from pathlib import Path
import re
import secrets
import sys
import time

import serial


ANSI = re.compile(rb"\x1b\][^\x07\x1b]*(?:\x07|\x1b\\)|\x1b\[[0-?]*[ -/]*[@-~]|\x1b[ -/]*[0-~]")
READY = re.compile(rb"^UART1 READY\n", re.M)


def wait_ready(capture):
    deadline = time.monotonic() + 70
    while time.monotonic() < deadline:
        if capture.exists():
            data = ANSI.sub(b"", capture.read_bytes()).replace(b"\r", b"").replace(b"\0", b"")
            if READY.search(data):
                return
        time.sleep(0.05)
    sys.exit("timed out after 70 s waiting for UART1 READY")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--port", required=True, help="UART1 adapter device path")
    parser.add_argument("--uart0", type=Path, required=True, help="new boot's UART0 raw capture")
    parser.add_argument("--out", type=Path, required=True, help="new file for raw UART1 RX bytes")
    args = parser.parse_args()

    nonce = secrets.token_bytes(8)
    port = serial.Serial(baudrate=115200, timeout=2, write_timeout=0.5, exclusive=True)
    port.dtr = False
    port.rts = False
    port.port = args.port
    with port, args.out.open("xb") as raw:
        print("UART_PEER_ARMED", flush=True)
        wait_ready(args.uart0)
        port.reset_input_buffer()
        print(f"UART_PEER_TX nonce={nonce.hex()}", flush=True)
        if port.write(nonce) != len(nonce):
            sys.exit("UART1 nonce write was short")
        reply = port.read(len(nonce))
        raw.write(reply)
        if reply != nonce:
            sys.exit(f"UART1 echo mismatch: got {reply.hex()}")
        port.timeout = 0.1
        extra = port.read(1)
        raw.write(extra)
        if extra:
            sys.exit("UART1 echo contained extra bytes")
        if port.write(b"\x06") != 1:
            sys.exit("UART1 ACK write was short")
    print("UART_PEER_END rc=0")


if __name__ == "__main__":
    main()
