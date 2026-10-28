# RP1 register database

`rp1-regdb` is the canonical RP1 register model and direct Rust PAC generator.
The supported pipeline is:

```text
TOML -> validator -> Rust generator -> rp1-pac
```

There is no SVD or `svd2rust` step. Historical SVD, Linux-audit, and unofficial
Rust tables were one-time migration inputs and are not build dependencies.

## Layout

- `data/device.toml`: device/core capability, interrupt, clock-ID, GPIO-bank,
  address-space, PCI identity, Boot ROM observation, and peripheral-file index.
- `data/peripherals/*.toml`: one logical peripheral or external-IP instance per
  file.
- `src/model.rs`: strongly typed schema.
- `src/validate.rs`: safety and layout rules.
- `src/gen_rust.rs`: deterministic Rust PAC generation.
- `data/COVERAGE.generated.md`: current generated coverage.
- `SCHEMA.md`: `rp1-regdb-v3` metadata and safety semantics.
- `MIGRATION-COVERAGE.md`: final legacy migration accounting.

## Generate

From the workspace root:

```bash
cargo run -p rp1-regdb -- validate
cargo run -p rp1-regdb -- generate
cargo run -p rp1-regdb -- generate --check
```

`generate --check` regenerates a temporary `rp1-pac` package layout and fails
if any checked-in generated Rust file or `rp1-pac/device.x` differs.

## Schema

The schema is `rp1-regdb-v3`. Hardware behavior, verification, and generated
API exposure are independent:

```toml
hardware_read = "plain"
hardware_write = "plain"
verified_read = "hardware"
verified_write = "unverified"

[register.api]
read = "public"
modify = "none"
raw_write = "none"
```

Register API capabilities are operation-specific: `read`, `modify`, `reset`,
`w1c`, `w1s`, `w1t`, `command`, `fifo_push`, `fifo_pop`, and `raw_write`.
Omitted operations default to none. Generic full-register and zero-based write
APIs are intentionally absent; unsafe raw writes require an explicit TOML
policy. Known reset-value metadata does not imply a generated `reset()`
operation.

Field write behavior is also explicit: `plain`, `write-one-to-clear`,
`write-one-to-set`, `write-one-to-toggle`, `self-clearing`, `command`, or
`read-only`. Mixed-semantics registers cannot expose general `modify()`.
Inferred or write-unverified fields cannot expose safe writers.

`[core]` describes implementation capabilities of the two RP1-internal
Cortex-M3 processors. It is device metadata, not an RP1 memory-mapped
peripheral owner: the PAC does not create `Peripherals::MPU`, `SCB`, `NVIC`,
`SYSTICK`, `DWT`, or `FPB` entries. `rp1-pac::metadata` exposes these values
without claiming ownership of Arm system control registers.

Verification source and observation result are separate. `hardware` says how
an observation was obtained; `confirmed`, `not-observed`, `inconclusive`, and
`contradicted` say what happened. `not-observed` is not confirmed absence.
Observation class and scope additionally separate global capability,
condition-limited behavior, probe-time runtime state, and architectural
interpretation. Runtime state and scoped observations are never flattened into
global capability booleans.

The MPU enforcement observations cover only proc0 privileged Thread-mode data
reads from a 32-byte shared-SRAM region configured as region 7 with no access.
They do not guarantee other MPU modes, regions, attributes, targets, proc1, or
DMA behavior. The Boot ROM public lookup table and official firmware ROM calls
are likewise recorded as not observed in the stated analysis scope, not absent.

## Interrupts

RP1 implements 64 NVIC external interrupt slots: the valid local IRQn range is
0..63. The database defines 61 named interrupts at IRQn 0..60, leaving IRQn
61..63 reserved. Generation uses the RP1 MSI-X source ID as the identical local
NVIC IRQn.

The existing evidence classification remains independent from that numbering
assumption. UART0 through UART5 are `hardware`/`hardware`; the other named IRQs
remain `linux`/`static`. Validation requires a positive 32-aligned NVIC slot
count, unique and verified named interrupts, a complete contiguous named range,
and all named IRQn values to fit both that range and the implemented NVIC slots.

The generator owns both `rp1-pac/src/interrupts.rs` (including the optional
runtime vector table) and `rp1-pac/device.x`.

## Atomic aliases

RP1-native APB alias windows are peripheral metadata, but capability is granted
per register:

```toml
[peripheral.atomic_view]
kind = "rp1-native"
xor_offset = 0x1000
set_offset = 0x2000
clear_offset = 0x3000

[register.atomic]
kind = "rp1-native"
set = "typed"
clear = "typed"
xor = "typed"
```

The PAC owns one logical peripheral. It never creates `*_SET`, `*_CLR`, or
`*_XOR` singleton owners. Safe atomic methods accept generated typed writers,
so unpublished bits cannot be selected. SET, CLEAR, and XOR use distinct mask
builders with `set`/`set_bits`, `clear`/`clear_bits`, and
`toggle`/`toggle_bits` vocabulary. Raw mask methods are explicitly `unsafe`
and end in `_unchecked`.

Custom windows name each view. PCIe MSI-X uses SET `+0x0800` and CLEAR
`+0x0c00`; it never inherits native RP1 offsets.

Atomic operations are rejected for command, FIFO, W1C/W1S, self-clearing,
state-machine-only, unknown, OTP/eFuse, and other unsafe registers.

WATCHDOG offsets `0x0c..0x1c` are Boot ROM proc0/proc1 start-service state,
not generic scratch memory. They expose no safe writer, modify, reset, typed
atomic, or unsafe raw atomic API. Only the three words at `0x20..0x28` remain a
scratch array, and their unverified write behavior keeps safe writers hidden.

TICKS retains hardware reset value zero. Its observed Boot ROM initialization
of cycle value `0x32` and enable state is separate scoped metadata and neither a
reset value nor evidence for a new write API. OTP remains read/configuration
metadata only; the Boot ROM usage note does not add programming, erase, lock,
modify, raw-write, or atomic surfaces.

## Ownership

`rp1-pac::Peripherals` contains exactly one owner per logical peripheral or
external-IP instance. `Peripherals::take()` uses an atomic singleton flag;
`unsafe Peripherals::steal()` requires caller-proven uniqueness. Alias and host
address views do not receive separate ownership. CamelCase owner type aliases
such as `Uart0` are generated for compatibility with existing HAL type bounds;
they do not create a second owner.

## Arrays

Contiguous arrays use `[Reg<SPEC>; count]`. Strided or interleaved arrays use a
generated `#[repr(C)]` padded element wrapper. `public_count` may hide reserved
physical entries. The validator rejects under-sized, unaligned, or out-of-range
strides and counts.

## External IP

Public third-party IP register bodies are not duplicated. TOML keeps RP1
instance integration facts such as base, interrupt, clock, reset, and pinmux
dependencies. This includes Arm PL011, DesignWare I2C/SPI/I2S/DMA, SDHCI,
Cadence GEM, xHCI, and MIPI blocks.

## Adding data

1. Add or edit one `data/peripherals/*.toml` file.
2. Add its relative path to `data/device.toml` if it is new.
3. Record only `origin`, verification, and a short generation-relevant note.
4. Keep uncertain or state-machine-only controls hidden or read-only.
5. Run validation, generation, deterministic check, tests, and target check.
6. Review generated PAC and coverage diffs.

Detailed firmware-analysis logs and hardware transcripts belong in the
separate `rpi-cm5-hack` repository, not this database. Version 3 deliberately
does not add a machine-readable evidence registry, external report IDs,
artifact hashes, or hardware-run history.
