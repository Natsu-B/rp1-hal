#!/usr/bin/env python3
"""Host peer for `lab/check-uart1.sh`: the adapter wired to the CM5's GPIO 0/1 (Linux UART1).

Usage: uart1-peer.py --nonce HEX --duration SECONDS --out FILE [--port DEV] [--baud 115200]

For DURATION seconds it records every byte the CM5 sends (raw, to FILE) and repeats
`RP1SHARE_RX_<nonce>` every 0.25 s for the Linux side to read. The port comes from --port or
`CM5_UART1_PEER`; there is no default. The verdict is `lab/verify-shm.py --uart1-host FILE
--uart1-nonce HEX`. Exit 0 when the capture ran for the whole duration, 2 on usage errors.
"""
import argparse
import os
import re
import sys
import time


def main(argv):
    p = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    p.add_argument("--port", default=os.environ.get("CM5_UART1_PEER"))
    p.add_argument("--baud", type=int, default=115200)
    p.add_argument("--nonce", required=True)
    p.add_argument("--duration", type=float, required=True)
    p.add_argument("--out", required=True)
    a = p.parse_args(argv)
    if not a.port:
        p.error("set --port or CM5_UART1_PEER to the adapter wired to GPIO 0/1")
    if not re.fullmatch(r"[0-9a-f]{8,}", a.nonce):
        p.error("--nonce must be at least 8 lowercase hex digits")
    import serial  # pyserial; imported late so --help works without it

    s = serial.Serial(a.port, a.baud, timeout=0.05)
    s.dtr = False
    s.rts = False
    s.reset_input_buffer()
    line = f"RP1SHARE_RX_{a.nonce}\n".encode()
    sent = received = 0
    end = time.monotonic() + a.duration
    next_tx = 0.0
    with open(a.out, "wb") as out:
        while (now := time.monotonic()) < end:
            if now >= next_tx:
                s.write(line)
                sent += 1
                next_tx = now + 0.25
            data = s.read(4096)
            if data:
                out.write(data)
                out.flush()
                received += len(data)
    s.close()
    print(f"UART1_PEER port={a.port} baud={a.baud} sent_lines={sent} received_bytes={received}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
