#!/usr/bin/env python3
"""Check the configurable-fault priority written by the linked reset path."""
import re
import subprocess
import sys


def check(elf):
    assembly = subprocess.check_output(
        [
            "arm-none-eabi-objdump",
            "--no-show-raw-insn",
            "--disassemble=__rp1_setup_exceptions",
            elf,
        ],
        text=True,
    )
    instructions = re.findall(r"^\s*[0-9a-f]+:\s+([^\n@]+)", assembly, re.M)
    instructions = [" ".join(insn.split()) for insn in instructions]
    stores = [f"strb r2, [r1, #{offset}]" for offset in (12, 13, 14)]
    first = instructions.index(stores[0])
    actual = instructions[first - 1 : first + len(stores)]
    if actual != ["movs r2, #32", *stores]:
        raise ValueError(f"SHPR1 must configure all three faults at priority 1: {actual}")
    print("STARTUP_FAULT_PRIORITY=PASS")


if __name__ == "__main__":
    check(sys.argv[1])
