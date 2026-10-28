# rp1-regdb-v3 schema notes

`rp1-regdb-v3` keeps RP1 peripheral registers and Arm core capabilities in
separate parts of the model. TOML plus direct Rust generation are authoritative;
compatibility outputs may omit capability metadata that they cannot represent
without inventing a peripheral.

## Device and core metadata

`[device]` contains RP1 device-wide register-bus facts. `[core]` describes the
two internal Cortex-M3 r2p1 processors and their common implementation:
architecture, implementation, revision, CPUID, processor count, endianness,
NVIC, MPU, SysTick, DWT, FPB, and instruction observations.

These values are capabilities, not MMIO ownership. `rp1-regdb` does not own Arm
system control registers as RP1 peripherals, and generation must not add MPU,
SCB, NVIC, SysTick, DWT, or FPB singleton owners.

The validator checks CPUID/implementation consistency, processor count, IRQ
capacity and bounds, NVIC priority width, MPU region shape, SysTick CALIB/TENMS
consistency, and representable comparator counts. IRQ 0 through 60 are assigned;
the 64 external slots cover IRQn 0 through 63, with 61 through 63 unassigned.

## Verification and observations

Verification records the source of a claim. An observation independently has:

- `result`: `confirmed`, `not-observed`, `inconclusive`, or `contradicted`;
- `class`: capability, scoped behavior, runtime state, or architectural
  interpretation;
- optional scope and note.

`not-observed` never means confirmed absence. An inconclusive or scoped result
does not authorize a positive global API. Contradicted observations stop
generation. Runtime state is not hardware capability, and architectural
interpretation is not a raw hardware value.

The MPU enforcement data is deliberately scoped to one proc0 privileged
Thread-mode shared-SRAM data-read test: region 7, 32 bytes, no access. It does
not generalize to instruction/XN behavior, unprivileged execution, other AP or
memory attributes, overlap/subregions, local memories, MMIO, proc1, nested or
stacking faults, DMA, or peripheral-memory enforcement.

The Boot ROM lookup table and official firmware ROM calls are `not-observed`
within their analysis scope. They are not encoded as absent booleans.

## Register safety and initialization

WATCHDOG `BOOT_MAGIC`, encoded entry, and initial-SP registers are Boot ROM
start-service state-machine words. They are independent registers rather than
members of the generic scratch array and cannot expose safe write, modify,
reset, typed atomic, or unsafe raw atomic operations. The generic scratch array
contains only offsets `0x20`, `0x24`, and `0x28`; unverified writes remain
hidden.

RESETS `CTRL0.PROC1` bit 31 controls proc1 reset and has typed atomic SET/CLEAR
operations. `DONE0.PROC1` bit 31 only reports release completion and is
read-only.

A Boot ROM initialization value is not a hardware reset value. TICKS cycle
registers therefore keep reset value zero while separately recording the
scoped observed Boot ROM value `0x32`. Such metadata cannot create writers or
fields. Unknown or unverified OTP contents likewise cannot gain safe writers,
modify, programming, erase, lock, raw-write, or atomic APIs.

## Evidence scope

Version 3 contains no machine-readable evidence list or evidence-reference
mechanism. It does not store report identifiers, commit/artifact hashes,
hardware run history, or imported test logs.
