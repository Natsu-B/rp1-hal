# rp1-pac

This crate is generated directly from `../rp1-regdb/data/device.toml` and the
split TOML files under `../rp1-regdb/data/peripherals/`.

Handwritten code:

- `src/lib.rs`
- `src/generic.rs`
- `build.rs`

Generated code:

- `src/peripherals.rs`
- `src/interrupts.rs`
- `src/metadata.rs`
- `src/generated/*.rs`
- `device.x`

Do not edit generated files manually. Regenerate and verify with:

```bash
cargo run -p rp1-regdb -- generate
cargo run -p rp1-regdb -- generate --check
cargo test -p rp1-pac
```

The PAC exposes operation-specific register capabilities. Safe atomic aliases
are typed; unrestricted raw writes and raw atomic masks are available only as
explicitly `unsafe` `_unchecked` operations when TOML permits them. External IP
files expose RP1 instance ownership and integration metadata without copying
the third-party register body.

The generated public `rp1_pac::metadata` module describes the RP1-internal
Cortex-M3 r2p1 implementation and its NVIC, MPU, SysTick, DWT, FPB, instruction,
and Boot ROM observations. These are capability values, not owners: the PAC
does not add MPU, SCB, NVIC, SysTick, DWT, or FPB to `Peripherals`, and it does
not provide accessors for Arm system control registers. Typed observation
values preserve `Confirmed`, `NotObserved`, `Inconclusive`, and `Contradicted`
instead of flattening limited or negative observations into booleans.

RESETS `CTRL0.PROC1` has typed atomic SET/CLEAR operations, while
`DONE0.PROC1` is read-only. WATCHDOG Boot ROM start-service words are separate
read-only state-machine registers rather than generic scratch entries; only
the three words at `0x20..0x28` form the remaining scratch array. No new OTP
write surface is generated. TICKS Boot ROM initialization observations remain
distinct from hardware reset values.

Atomic SET, CLEAR, and XOR use separate mask builders:

```rust,ignore
resets.ctrl1().atomic_set(|w| w.uart0().set());
resets.ctrl1().atomic_clear(|w| w.uart0().clear());
resets.ctrl1().atomic_xor(|w| w.uart0().toggle());
```

RP1 has 64 NVIC external interrupt slots (IRQn 0..63). The generated
`Interrupt` enum contains the 61 named IRQs at 0..60; IRQn 61..63 are reserved.
Generation treats each RP1 MSI-X source ID as the identical local NVIC IRQn.
UART0 through UART5 retain `hardware`/`hardware` evidence, while the other
named IRQs retain `linux`/`static` evidence.

The optional, non-default `vectors` feature makes this PAC own the 64-entry
`__INTERRUPTS` table and supplies generated `device.x` weak aliases for all 61
named handlers. Reserved entries are zero. Attribute macros and core exception
vectors belong to the consuming runtime rather than the PAC.
