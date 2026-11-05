#!/usr/bin/env python3
"""Run a Linux console script with reset telemetry and timestamped raw UART capture.

Paths to the hardware lock, UARTs, lab root and output are explicit arguments.
The existing console transport is loaded from the lab root, without deployment.
"""
import argparse
import datetime
import fcntl
import json
from pathlib import Path
import runpy
import select
import subprocess
import time

import serial


def payload_for(script):
    telemetry = Path(__file__).with_name('reset-telemetry.sh').read_bytes()
    # The outer marker must follow telemetry shutdown, not the supplied script.
    body = script.replace(b'echo DDR_INSPECT_END', b': # outer observer emits end')
    return (telemetry + b'\nreset_telemetry_start\n' + body
            + b'\nreset_telemetry_stop\necho DDR_INSPECT_END\n')


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--root', type=Path, required=True)
    p.add_argument('--campaign', type=Path, required=True)
    p.add_argument('--script', type=Path, required=True)
    p.add_argument('--lock', type=Path, required=True)
    p.add_argument('--uart0', required=True)
    p.add_argument('--uart10', required=True)
    p.add_argument('--duration', type=int, default=300)
    args = p.parse_args()
    transport = runpy.run_path(str(args.root / 'scripts/linux-ddr-observe.py'))
    payload = payload_for(args.script.read_bytes())
    with args.lock.open('a') as lock:
        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        holders = subprocess.run(['fuser', args.uart0, args.uart10], capture_output=True)
        if holders.returncode != 1 or holders.stdout or holders.stderr:
            raise RuntimeError('UART holder or failed fuser check')
        out = args.campaign / (datetime.datetime.now().strftime('%Y%m%d-%H%M%S-%f') + '-ddr-live')
        out.mkdir(parents=True)
        (out / 'inspect.sh').write_bytes(payload)
        (out / 'command.txt').write_bytes(transport['command_for'](payload))
        ports, files = [], []
        complete = False
        started = time.monotonic()
        next_status = started
        journal = None
        try:
            with (out / 'host-kernel.raw').open('wb') as hostlog, (out / 'esp32-status.raw').open('ab') as statuslog, (out / 'timestamps.jsonl').open('w') as stamps:
                journal = subprocess.Popen(['journalctl', '-k', '-f', '-n', '20', '--no-pager', '-o', 'short-iso-precise'], stdout=hostlog, stderr=subprocess.STDOUT)
                for device, dtr, name in [(args.uart0, False, 'uart0'), (args.uart10, True, 'uart10')]:
                    port = serial.Serial(port=None, baudrate=115200, timeout=0, exclusive=True)
                    port.dtr, port.rts, port.port = dtr, False, device
                    port.open()
                    ports.append(port)
                    files.append((out / (name + '.raw')).open('wb'))
                command = transport['command_for'](payload)
                sent = 0
                tail = b''
                while time.monotonic() - started < args.duration:
                    now = time.monotonic()
                    if now >= next_status:
                        statuslog.write((datetime.datetime.now(datetime.timezone.utc).isoformat() + '\n').encode())
                        statuslog.flush()
                        subprocess.run(['python3', str(args.root / 'scripts/cm5ctl.py'), 'status'], stdout=statuslog, stderr=subprocess.STDOUT, timeout=10)
                        statuslog.flush()
                        next_status = now + 30
                    ready, writable, _ = select.select(ports, [ports[1]] if sent < len(command) else [], [], 0.25)
                    if writable:
                        # Drain echoed input while sending scripts larger than the TTY buffer.
                        sent += ports[1].write(command[sent:sent + 512])
                    for port in ready:
                        index = ports.index(port)
                        data = port.read(65536)
                        files[index].write(data)
                        files[index].flush()
                        stamps.write(json.dumps({'utc': datetime.datetime.now(datetime.timezone.utc).isoformat(), 'monotonic_ns': time.monotonic_ns(), 'port': index, 'bytes': len(data)}) + '\n')
                        stamps.flush()
                        if index == 1:
                            tail = (tail + data)[-4096:]
                            if b'\nDDR_INSPECT_END\r\n' in tail or b'\nDDR_INSPECT_END\n' in tail:
                                complete = True
                    if complete:
                        break
        finally:
            for file in files:
                file.close()
            for port in ports:
                port.close()
            if journal:
                journal.terminate()
                journal.wait(timeout=5)
            (out / 'result.json').write_text(json.dumps({'complete': complete, 'seconds': time.monotonic() - started}, indent=2) + '\n')
            print(out, flush=True)
        if not complete:
            raise SystemExit('FAIL: missing explicit end marker; raw retained')


if __name__ == '__main__':
    main()
