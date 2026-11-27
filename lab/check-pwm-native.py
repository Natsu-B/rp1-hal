#!/usr/bin/env python3
"""Reject calls, loops, unexpected instructions or extra stores in the RP1 duty probe."""
import argparse
import re
import subprocess

SYMBOL = "motor_native_duty"

# Cortex-M3 instructions for one 32-bit range comparison, the two ordered MMIO
# stores, and a return. Admit normal compiler choices for bool materialization
# and frame setup, but reject divisions, sleep, traps, and all subroutine calls.
STRAIGHT = {
    "cmp", "ldr", "mov", "movt", "movw", "movs", "movls", "movhi", "it",
    "itt", "ite", "push", "ldmia", "dsb", "dmb", "str", "nop",
}
BRANCH = {
    "b", "beq", "bne", "bhi", "bls", "bcc", "bcs", "blo", "bhs", "cbz", "cbnz",
}


def check(assembly):
    match = re.search(
        fr"(?m)^[0-9a-f]+ <{SYMBOL}>:\n((?:^[ \t]*[0-9a-f]+:.*\n)+)", assembly
    )
    if not match:
        raise ValueError(f"missing {SYMBOL} disassembly")
    instructions = []
    for line in match[1].splitlines():
        addr, insn = line.split(":", 1)
        opcode, *operands = insn.split("@", 1)[0].strip().split(None, 1)
        instructions.append((int(addr, 16), opcode.split(".")[0], operands[0] if operands else ""))
    addresses = {address for address, _, _ in instructions}
    stores = barriers = 0
    for address, opcode, operands in instructions:
        if opcode in BRANCH:
            target = re.search(r"(?:^|,\s*)([0-9a-f]+)\s+<", operands)
            if not target or int(target[1], 16) not in addresses or int(target[1], 16) <= address:
                raise ValueError(f"branch leaves the function or is backward at {address:x}")
        elif opcode == "bx" and operands == "lr":
            continue
        elif opcode not in STRAIGHT or re.search(r"\bpc\b", operands):
            raise ValueError(f"unexpected instruction at {address:x}: {opcode} {operands}")
        stores += opcode == "str"
        barriers += opcode in ("dsb", "dmb")

    if (stores, barriers, instructions[-1][1:]) != (2, 2, ("bx", "lr")):
        raise ValueError("expected two ordered stores and a return")
    return match[0]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("elf")
    parser.add_argument("--objdump", default="arm-none-eabi-objdump")
    args = parser.parse_args()
    disassembly = subprocess.check_output(
        [args.objdump, "-d", "--no-show-raw-insn", f"--disassemble={SYMBOL}", args.elf],
        text=True,
    )
    try:
        print(check(disassembly), end="")
    except ValueError as exc:
        parser.exit(1, f"PWM_NATIVE=FAIL {exc}\n")
    print("PWM_NATIVE=PASS two MMIO stores, bounded call-free path")


if __name__ == "__main__":
    main()
