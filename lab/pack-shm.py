#!/usr/bin/env python3
"""Keep the deployed initramfs; replace the two local test tools. No kernel changes.

Required environment: SHM_ARTIFACT_ROOT (build-shm.sh output), RP1_INITRAMFS (the deployed
initramfs to extend), RP1_INITRAMFS_HELPER (Python module providing parse_newc/add_newc).
"""
import gzip
import hashlib
import os
from pathlib import Path
import runpy

def env(name):
    value = os.environ.get(name)
    if not value:
        raise SystemExit(f"set {name}")
    return Path(value)

out = env("SHM_ARTIFACT_ROOT")
base_initramfs = env("RP1_INITRAMFS")
helper = runpy.run_path(str(env("RP1_INITRAMFS_HELPER")))
tools = {"bin/shm-linux": out / "shm-linux", "bin/linux-doorbell": out / "linux-doorbell"}
entries = [(n, m, bytes(d)) for n, m, d, _ in helper["parse_newc"](gzip.decompress(base_initramfs.read_bytes()))
           if n != "TRAILER!!!" and n not in tools]
entries.extend((n, 0o100755, path.read_bytes()) for n, path in tools.items())
assert len(entries) == len({n for n, _, _ in entries})
raw = bytearray()
for ino, (name, mode, data) in enumerate(entries, 1):
    pos = len(raw)
    helper["add_newc"](raw, name, mode, data, ino)
    if name == "dev/console":
        raw[pos + 78:pos + 94] = b"0000000500000001"
helper["add_newc"](raw, "TRAILER!!!", 0, b"", 0)
assert [(n, m, bytes(d)) for n, m, d, _ in helper["parse_newc"](raw) if n != "TRAILER!!!"] == entries
packed = gzip.compress(raw, compresslevel=9, mtime=0)
assert len(packed) < 8 * 1024 * 1024
(out / "initramfs_2712").write_bytes(packed)
print(f"members={len(entries)} bytes={len(packed)} sha256={hashlib.sha256(packed).hexdigest()}")
