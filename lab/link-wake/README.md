# Link-wake delivery fixture

**STATIC · 2026-11-27.** This image uses experimental NOTE v3 and the single
vector owner on proc1. Its build script selects the internal profile for
testing terminal delivery and raw vector pulses. The build and ELF checks run in CI.

```sh
cargo rp1 build --locked -p link-wake-fixture --features firmware --target thumbv7m-none-eabi --release
cargo rp1 check target/thumbv7m-none-eabi/release/link-wake-fixture
```

**HYP · 2026-11-27.** For a future cold-boot delivery test, the fixture's
link SRAM scratch fields are as follows. The host reads the same link window
after each command store, then rings that core's GPIO force (26 on proc0 and
27 on proc1). Allow one outstanding command per core. The host observes a
terminal command through the existing status block.

| Offset | Purpose |
| --- | --- |
| `0x24` | proc0 command |
| `0x28` | proc1 command |
| `0x2c` | Vector state (0 starting, 1 low and ready, 2 a source check or pulse failed) |

Commands 1, 2 and 3 select stop, panic and UsageFault. Command 4 asks proc1
for a test pulse without a terminal transition. Proc1 scans terminal SRAM
on entry and after events. A proc0 terminal can wake proc1 even before any
ring is attached. Proc1's deliberate stop can publish and pulse before
parking. A fault or panic in proc1 can leave the vector without an owner.
If PCIe is down, the terminal stays eligible for a later scan after a
doorbell or terminal event. IRQ53 signals loss, and the fixture has no
automatic transition back to Up or dedicated retry event.
Start from a cold boot with old writers and posted accesses drained.

A legacy executable needs the matching schema-2 boot set. With the
`linux-ping` lab build, run
`lab/ping/legacy-launch.sh ./linux-ping irq ./old-client`. The launcher
waits for Ready, releases its mappings and then executes the client.

## Follow-up on DDR store width and coverage

**STATIC.** The schema-2 initialization store loop still runs from shared SRAM.
**HW · 2026-11-28.** A two-byte shift of the shared-SRAM v3 loop changed fill
time by about 8 ms in a controlled build. Account for this placement sensitivity
when comparing schema-2 builds. The v3 ISRAM loop took 40.572 ms to fill the
1 MiB reservation, including its per-chunk checks. Use that measurement as the
v3 reference when evaluating store width or which bytes need initialization.
