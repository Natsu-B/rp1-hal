#!/usr/bin/env python3
"""Exercise the UART parity image using a single live 115200 8N1 peer."""

import argparse
import json
import re
import secrets
import time
from pathlib import Path

import serial

TRIALS = 1000
STREAM = 100_000
TX_SAMPLE = 34


def pattern(nonce, length, offset=0):
    return bytes((((i * 73) ^ (i >> 8)) & 255) ^ nonce[i % 8]
                 for i in range(offset, offset + length))


def run(port, raw, incoming, nonce):
    def read(count):
        data = port.read(count)
        raw.write(data)
        if len(data) != count:
            raise ValueError(f"UART1 short read ({len(data)} of {count})")
        return data

    def expect(expected):
        data = read(len(expected))
        if data != expected:
            pos = next(i for i, (a, b) in enumerate(zip(data, expected)) if a != b)
            raise ValueError(f"UART1 mismatch at byte {pos} of {len(data)}")

    def send(data):
        written = port.write(data)
        incoming.write(data[:written])
        if written != len(data):
            raise ValueError("UART1 short write")
        port.flush()

    def reply(data):
        # Give thread mode time to arm after physically flushing the command.
        time.sleep(0.004)
        send(data)

    send(nonce)
    expect(nonce)
    for command, width in [(b"R", 16), (b"S", 1)]:
        for i in range(TRIALS):
            expect(command)
            reply(pattern(nonce, width, i * width))

    occupied = []
    for i in range(TRIALS):
        expect(b"T")
        expect(pattern(nonce, TX_SAMPLE, i * TX_SAMPLE))
        count = read(1)[0]
        if count not in (32, 33):
            raise ValueError(f"invalid TX prefill count {count}")
        occupied.append(count)

    expect(b"I")
    reply(pattern(nonce, STREAM))
    expect(b"O")
    expect(pattern(nonce, STREAM))
    expect(b"A")
    reply(nonce[:1])
    for command in (b"Q", b"P"):
        expect(command)
        reply(nonce)
        expect(nonce)
    port.timeout = 0.25
    extra = port.read(1)
    raw.write(extra)
    if extra:
        raise ValueError("unexpected trailing UART1 byte")
    return occupied


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--port", required=True)
    parser.add_argument("--uart0", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--nonce", default=secrets.token_hex(4))
    args = parser.parse_args()
    if not re.fullmatch("[0-9a-f]{8}", args.nonce):
        parser.error("nonce must be eight lowercase hex digits")
    args.out.mkdir()
    port = serial.Serial(baudrate=115200, timeout=25, write_timeout=25, exclusive=True)
    port.dtr = False
    port.rts = False
    port.port = args.port
    with port:
        print("UART_PARITY_PEER_ARMED", flush=True)
        ready_by = time.monotonic() + 60
        ready = re.compile(rb"(?:^|\n)UART_PARITY_READY\r?\n")
        while time.monotonic() < ready_by:
            if args.uart0.exists() and ready.search(args.uart0.read_bytes()):
                break
            time.sleep(0.02)
        else:
            raise SystemExit("UART_PARITY_READY timeout")
        port.reset_input_buffer()
        with (args.out / "from-rp1.bin").open("xb") as raw, (args.out / "to-rp1.bin").open("xb") as sent:
            occupied = run(port, raw, sent, args.nonce.encode())
    metadata = {"run": args.nonce, "prefill": occupied}
    (args.out / "peer.json").write_text(json.dumps(metadata) + "\n")
    print(f"UART_PARITY_PEER_END run={args.nonce} result=PASS", flush=True)


if __name__ == "__main__":
    main()
