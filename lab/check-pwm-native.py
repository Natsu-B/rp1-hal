#!/usr/bin/env python3
"""Check the bounded native PWM probe and its exact RP1 MMIO addresses."""
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
    targets = set()
    for _, opcode, operands in instructions:
        if opcode in BRANCH:
            target = re.search(r"(?:^|,\s*)([0-9a-f]+)\s+<", operands)
            if target:
                targets.add(int(target[1], 16))

    constants = {}
    self_in_r0 = True
    loads = 0
    effects = []
    for address, opcode, operands in instructions:
        if address in targets:
            # A branch may skip instructions that establish a constant.
            constants.clear()
            self_in_r0 = False
        if opcode in BRANCH:
            target = re.search(r"(?:^|,\s*)([0-9a-f]+)\s+<", operands)
            if not target or int(target[1], 16) not in addresses or int(target[1], 16) <= address:
                raise ValueError(f"branch leaves the function or is backward at {address:x}")
        elif opcode == "bx" and operands == "lr":
            continue
        elif opcode not in STRAIGHT or re.search(r"\bpc\b", operands):
            raise ValueError(f"unexpected instruction at {address:x}: {opcode} {operands}")
        if opcode in ("ldr", "str"):
            access = re.fullmatch(r"(r\d+), \[(r\d+)(?:, #(\d+))?\]", operands)
            if not access:
                raise ValueError(f"unexpected memory operand at {address:x}: {operands}")
            reg, base, offset = access.groups()
            offset = int(offset or "0")
            if opcode == "ldr":
                loads += 1
                if base != "r0" or offset != 0 or not self_in_r0:
                    raise ValueError("hot path may load only the channel's cached range from self")
                constants.pop(reg, None)
                self_in_r0 &= reg != "r0"
            else:
                if base not in constants:
                    raise ValueError(f"store base is unknown at {address:x}")
                effects.append(constants[base] + offset)
        elif opcode in ("dsb", "dmb"):
            effects.append("barrier")
        elif opcode.startswith("mov"):
            reg = operands.split(",", 1)[0]
            imm = re.fullmatch(r"r\d+, #(\d+)", operands)
            prior = constants.pop(reg, None)
            self_in_r0 &= reg != "r0"
            if opcode == "movw" and imm:
                constants[reg] = int(imm[1])
            elif opcode == "movt" and imm and prior is not None:
                constants[reg] = (prior & 0xFFFF) | (int(imm[1]) << 16)
        elif opcode == "ldmia" and operands != "sp!, {r7, lr}":
            raise ValueError(f"unexpected stack restore at {address:x}: {operands}")

    if loads != 1:
        raise ValueError("expected exactly one load from self")
    if effects != [0x40098020, "barrier", 0x4009A000, "barrier"]:
        raise ValueError("expected ordered stores to CHAN0_DUTY and GLOBAL_CTRL SET")
    if instructions[-1][1:] != ("bx", "lr"):
        raise ValueError("expected return at end of probe")
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
