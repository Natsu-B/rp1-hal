# Linux shared-memory proof

STATIC: `shm-firmware` retains the dual-core UART0, SCMI server and GPIO27 doorbell, and adds
reader-local rings, a 1 MiB block and a command block. `shm-linux` checks 1,000,000 messages
in each direction in reject mode, every word of the block in both directions, overwrite
accounting, M3 scalar DDR read latency/read bandwidth/posted-write bandwidth, and the isolated
M3 per-message push/pop cost (commands 7/8, Linux idle on the link page).
Rates include API/loop overhead; M3 writes include a same-window read to drain PCIe writes.

STATIC: the SRAM/DDR micro-profile (command 9) is opt-in. The default firmware and
`shm-linux` omit it; `shm-linux` prints `PROFILE skipped`. Set `SHM_PROFILE=1` when using
`lab/build-shm.sh` to enable `sram-profile` in both binaries, and pass `--profile` to
`lab/verify-shm.py`. For direct builds, pass `--features sram-profile` to both packages. The default + `sram-profile` image currently does not fit (below).
The firmware also has `i2c`, `adc` and `spi` features, all enabled by default for
`lab/check-shm.sh`. Use `--no-default-features` for SHM/doorbell/SCMI alone, or add
`--features i2c` (or `adc`, `spi`, `sram-profile`, comma-separated) for a focused image.
For example, build a profile-only image with:

```sh
cargo rp1 build -p shm-firmware --target thumbv7m-none-eabi --release \
    --no-default-features --features sram-profile
cargo build -p shm-linux --target aarch64-unknown-linux-musl --release --features sram-profile
```

The `dma` feature (not default) adds the DMA proof (`src/dma_proof.rs`, proc1 declares
`dma = {}` in every profile, so Linux's `rp1_dma` is always disabled in the lab): at boot an
SRAM->SRAM check (`DMA_SRAM ... rc=0x0 bad_bytes=0`, `DMA_SRAM_RATE`, `DMA_SRAM_PASS` on
UART0), and command 16 (`shm-linux dma`, after `shm-linux`): RP1 SRAM -> Linux DDR through
the ATU window and the direct-mapped PCIe space, against CPU posted writes, verified word by
word by Linux and by an RP1 read-back (`DMA_DDR_LINUX`, `DMA_DDR_VERIFY ... bad=0
canary_left=0`, `DMA_DDR_PASS` on UART10). The default + `dma` image does not fit (about
3.3 KiB over); use the focused image:

```sh
cargo rp1 build -p shm-firmware --target thumbv7m-none-eabi --release \
    --no-default-features --features dma
```

With it, I2C/ADC/SPI requests are refused (`0xffffffff`) as for any focused image, so the
full `check-shm.sh`/verifier does not apply; check SHM, doorbell, ring-irq and UART1 directly.

The `ring-dma` feature (not default; use `--no-default-features --features ring-dma`) adds
command 17 (`shm-linux ring-dma [count]`): RP1 -> Linux streams on `events` (16 B), `wide64`
(64 B), `wide256` (256 B), with channel 1 and the 4 KiB cancellation buffer as stage.
Modes 0/1/2 are CPU slice/DMA slice/auto; 3/4 are deliberate early-head/bit30-zero ordering
controls; 5/6/7 are generated DMA/CPU/auto. Generation uses identical message functions;
Linux verifies every payload word, newest-first and when consumed. The preferred larger
batches are 128/56/15; former 32/32/8 batches remain selectable. The generated path has no
batch array; the slice comparison has a separate stack frame (maximum 3840 B), within
proc1's declared 6 KiB stack (STATIC: build and disassembly). The focused image paints unused
proc1 stack words at entry and scans after each command 17, outside the measured interval.
UART0 `RINGDMA_STACK run_id=... free_bytes=...` reports cumulative untouched stack space
since entry, in bytes (4-byte granularity). Used bytes = 6144 - free_bytes, including the
unpainted active entry frame; the scan never counts that frame as free.
Retain these lines for every slice-comparison condition, especially batches 128/56/15, and
report the minimum free margin per boot. Require nonzero free_bytes before accepting a
speed comparison. A watermark is diagnostic, not an overflow guard;
no HW margin has been measured yet.

STATIC: the focused ring image omits generic DDR microbench command 5 and up/down batch
commands 10–13 to fit the 48 KiB image budget. Omitted commands return an error. Use the
ordinary image for `shm-linux`'s full suite; ring-irq commands 14/15 remain available.
Command 18 fetches detailed timing from the last command 17 without changing its response.
The separate `dma` image's command 16 also emits pre-generated `RINGDMA_RAW` controls
(channel 1, 768/2304/2112/3072/4032/3960/4080 B, aligned and +8 offsets, 1000 copies each,
width and final data verification), before its existing bulk proof. Each condition poisons
its destination with the complement of the expected words and drains posted writes before
starting its timer, so a condition that writes nothing cannot reuse an earlier success.

STATIC: feature selection removes proof initialization, handlers and driver code from the
linked image. The metadata, note ownership and overlay keep the existing I2C1/ADC/SPI0/pin
reservations in every profile; disabling a proof does not return its resources to Linux.
Peripheral requests for an omitted proof return `0xffffffff` instead of hanging. Focused
images use `shm-linux` directly; the full `check-shm.sh`/verifier still requires all three
peripheral proofs. A profile-enabled client with a default firmware gets a command error;
it must not be reported as a successful profile. A disabled command 9 follows the existing
unknown-command path (`errors=1`); it never touches the bulk scratch or starts SysTick.

STATIC: every image keeps the `host_irq = true` declaration (rp1-hal feature
`unstable-host-irq`): it is part of the default lab profile, not a cargo feature of
`shm-firmware`, because the declaration (and the `rp1:irq` overlay node) is metadata that
features cannot select. Its code is the `notify_sram()` call in proc1's ISRAM command loop.
A firmware built with `sram-profile` logs `p0 sram-profile` on UART0; `verify-shm.py`
fails a run without `--profile` that has this line, and a run with `--profile` that lacks it,
so a firmware/client profile mismatch is caught in both directions.

STATIC size measurement (rustc 1.99.1, release, `cargo rp1 build` two-stage link, with the
host interrupt):

| Firmware features | SRAM image / 49152 B | Free | ISRAM / 8192 B | DSRAM static |
|---|---:|---:|---:|---:|
| i2c, adc, spi (default) | 45400 | 3752 | 3128 | 16 |
| none (SHM only) | 31648 | 17504 | 2116 | 16 |
| i2c | 36736 | 12416 | 2812 | 16 |
| adc | 35840 | 13312 | 2116 | 16 |
| spi | 37352 | 11800 | 2532 | 16 |
| sram-profile | 35440 | 13712 | 2708 | 656 |
| dma (2026-11-05, rustc of that date) | 39728 | 9424 | 3092 | 16 |
| i2c, adc, spi, sram-profile | does not link (16 B over) | - | - | - |
| i2c, adc, spi, dma | does not link (about 3.3 KiB over) | - | - | - |

The SRAM image includes ISRAM/DSRAM load copies and linker alignment; ELF file length is
not the SRAM budget. Numbers are compiler-dependent. The all-features image (default +
`sram-profile`) is best-effort and excluded from the matrix: it currently overflows the
shared SRAM, so `SHM_PROFILE=1 lab/build-shm.sh` and `verify-shm.py --profile` (which also
needs the I2C/ADC/SPI proofs) only work if it fits again. Profile with the focused
`sram-profile` image and the profile client instead (commands above), running `shm-linux`
directly and checking `p0 sram-profile`, `PROF_DONE`, `p1 shm cmd=9 ... errors=0` on UART0
and `PROFILE rp1_us=` before `SHM_PASS` on UART10.

HW 未検証: cold-boot the default tuple, run `lab/check-shm.sh`, and require `PROFILE skipped`, `SHM_PASS`, the peripheral replies
and the normal verifier's pass. Also pair a profile client
with a default firmware once and require a non-zero client exit for command 9. Cold-boot
each focused image, run `shm-linux` (with the matching `sram-profile` setting) plus a doorbell check before/after it, then send
peripheral requests 1/0xadc/0x5b1 one at a time through link+0x28 (clear link+0x2c first).
Require the usual reply and UART0 proof markers for enabled proofs; omitted proofs must
clear the request and reply `0xffffffff`, both before and after SHM enable. Verify SCMI
and Linux UART1 still probe, and the owned-node/pin reservations stay the same.

Build with `bash lab/build-shm.sh` and `python3 lab/pack-shm.py`. Locations come from the
environment: `SHM_ARTIFACT_ROOT`, `RP1_LOADER_ROOT`, `RP1_BASE_DTB` (build), and
`SHM_ARTIFACT_ROOT`, `RP1_INITRAMFS`, `RP1_INITRAMFS_HELPER` (pack); see each script header.
The patch is for the PoC chainboot loader (its rp1-hal dev branch): it publishes the fixed
board-table PA and generated DT reservation size over I2C before starting RP1. ELF and merged
DTB remain SHA-256 pinned. This is a dev-loader integration, not the full rp1boot bundle loader.

STATIC memory contract: PA `0x22000000`, minimum 1 MiB (lab: 2 MiB), maximum 16 MiB,
`reserved-memory/no-map`, 64 KiB per-region alignment. The earlier scratch/openamp reservations
remain intact. Loader maximum placements are kernel `0x06000000..0x0e000000`, initramfs
`0x10000000..0x18000000`, scratch `0x1ffe0000..0x20000000`, original/copy DTB below `0x20400000`.
No range intersects even the maximum shared reservation `0x22000000..0x23000000`.
M3 `0x81000000+x` translates through ATU region 0 to PCIe `0x1022000000+x`.

The link-page first 64 bytes hold handoff magic/PA/size (offsets 0/4/8/12), existing doorbell
request/reply (16/20), published generation/status (24/28), and Linux enable request (32).
SCMI occupies `0x17c0..0x18c0` relative to the link page; allocations skip it and the ABI tail.
Linux loads `uio.ko`, then `uio_pdrv_genirq.ko of_id=generic-uio`, and runs `lab/check-shm.sh`.
UIO is polled, or (after shm enable) proc1 also raises MSI-X vector 29 on the `rp1:irq` UIO after
each doorbell reply (unstable `host_irq = true`, `linux-doorbell <link> <doorbell> N
/dev/uio<rp1:irq>`, which re-checks the reply every 10 ms without a wake and reports
`missed=`); hwirq58 stays with SCMI's mailbox. No cached mapping, kernel patch or private
kernel API is used. Device mappings never go through libc memcpy/memset.

RP1 calls unsafe `shm::enable` once after the final Linux PERSTn, then one unsafe `Area::attach`
per owning core. `ring_tx`, `ring_rx` and `block` check the note's type/core/direction and reject
duplicate endpoint acquisition. Returned low-level block writers require the declared writer
ownership. Linux `Firmware::open` checks UIO names, physical ranges, note hash and generation;
handles retain mappings and advisory exclusive UIO locks. Other processes must respect those
locks and must not mmap aliases. Only cold boot is supported; a live PCIe reset invalidates ATU.
For writer=both the lab command handshake transfers writer ownership; the SeqLock alone does
not arbitrate two writers. Blocks support word sections, so RP1 needs no 1 MiB local copy.

Pass/fail is strict (`lab/check-shm.sh` + `lab/verify-shm.py`): each Linux step has a time
limit, any non-zero exit or wrong I2C reply prints `CHECK_FAIL`, the script ends with
`CHECK_SHM_RESULT=PASS|FAIL` then `DDR_INSPECT_END` and exits non-zero on failure. The host
runs `python3 lab/verify-shm.py --uart0 U0 --uart10 U10 --result OBSERVER_JSON`, which needs
the capture's `"complete": true`, the UART10 markers in order (`SHM_PASS`, `SHM_RC=0`, both
doorbells, `I2C_REPLY=0x00000001`, `ADC_REPLY=0x00000ADC`, `SPI_REPLY=0x000005B1`, `CHECK_SHM_RESULT=PASS`, `DDR_INSPECT_END`) and on UART0
the three `I2C_PASS` lines and
`I2C_RESULT request=1 Ok` (with `--profile`, also `PROF_DONE` and
`p1 shm cmd=9 ... errors=0`). The ADC is declared on proc1 in `shm-firmware`: proc1 gets its
`Clocks` with `Clocks::wait_from_proc0`, runs the boot burst (`p1 adc init Ok`,
`ADC_P1BOOT ... PASS`) and answers Linux's 0xadc request (`ADC_P1REQ ... PASS`).
`check-shm.sh` issues that request itself, after the doorbell, shm and I2C steps (the burst
blocks proc1's doorbell and command handling for tens of ms, so it must not overlap them), and
fails unless the reply is 0xadc. `verify-shm.py` requires `ADC_REPLY=0x00000ADC` on UART10 (after
`I2C_REPLY`) and on UART0 `ADC_P1BOOT ... PASS`, `ADC_RESULT boot Ok`, `ADC_P1REQ ... PASS` and
`ADC_RESULT request=0xadc Ok`; any `ADC_RESULT ... Err` fails the run. The UART0 boot markers are
in the boot capture, so pass the boot capture and the observer's UART0 log together.
SPI0 is also declared on proc1 (GPIO8 chip select as a plain output, GPIO9/10/11 MISO/MOSI/SCLK;
on the lab board these go to ESP32 inputs only). proc1 runs `spi_proof` at boot and on Linux's
0x5b1 request, which `check-shm.sh` issues after the ADC request: loopback off first (MISO is
pulled down, all zeros expected), then the internal loopback (CTRLR0.SRL) at 1, 10, 25 and
50 MHz with 8-, 12- and 16-bit frames through `SpiDevice`, u8 words on the bus, in-place and a
write/delay/read transaction. Each rate and size logs `SPI_PASS <tag> hz= div= bits= frames= us=
kbps= wire_pct=`. `verify-shm.py` requires `SPI_REPLY=0x000005B1`, `SPI_P1BOOT ... PASS`,
`SPI_RESULT boot Ok`, the 1 MHz and 10 MHz `SPI_PASS P1REQ` lines, `SPI_P1REQ ... PASS` and
`SPI_RESULT request=0x5b1 Ok`; `SPI_RESULT ... Err` and `spi0 init Err` fail the run. Failures
print `Display` codes, not `Debug`: the image is near the 48 KiB shared SRAM limit.
A boot-marker report (`full_success`) is not a pass for this regression. Host tests: `python3 lab/test_verify_shm.py`.

HW results belong in the run notes kept outside this repository; build success alone is STATIC evidence.

## Linux UART1 next to the RP1 firmware

Linux keeps UART1 on GPIO 0/1 (`dtoverlay=uart1-pi5`, ttyAMA*) while the firmware owns UART0,
I2C1 and SPI0. `lab/check-uart1.sh` runs on the Linux console with `UART1_NONCE=<hex>` prepended;
`lab/uart1-peer.py` runs on the host at the same time on the adapter wired to GPIO 0/1 (`--port`
or `CM5_UART1_PEER`, no default). Linux writes `RP1SHARE_TX_<nonce>_<i>` lines and the peer
records them; the peer repeats `RP1SHARE_RX_<nonce>` and Linux prints what it read as
`UART1_RX_GOT=`. `python3 lab/verify-shm.py --uart1-host PEER_OUT --uart1-nonce HEX
--uart1-console UART10 [--uart1-result OBSERVER_JSON]` prints `UART1_TX_PASS|FAIL` and
`UART1_RX_PASS|FAIL` (it can run together with the SHM regression arguments). The script also
prints `UART1_DIAG` lines: GPIO 0/1 CTRL/STATUS/PADS, the PL011 baud and control registers and the
driver's tx/rx counters, to tell a wiring or pinmux fault from a script fault. It keeps the port
open while it reads, because closing it shuts the PL011 down and drops received bytes.

## PWM loopback diagnostic (STATIC / HW 未検証)

`pwm-loopback` is a separate, focused firmware example. See [the PWM API, register evidence
and HW test plan](../docs/pwm.md). It reserves PWM0 as one block and reports two GPIO loopbacks
through UART counters while proc0 services SCMI. It has not been deployed or run on hardware.

Interrupt-notified ring: the `events` ring (`notify = "irq"`, proc1 has `host_irq`) is
streamed by commands 14 (with `HostIrq::push_slice_notify` for every batch) and 15 (without).
`shm-linux ring-irq [count]` runs burst / paced / single-message streams with a polling and
an `RingRx::wait` consumer, checks every message and prints one `RINGIRQ` line per run
(measured `missing` / `corrupt`, `check=`). Notified irq runs must be woken by interrupts
(`woken > 0`, `late` at most 1 % of the sleeps); the designated `paced-irq` run must also have
slept (`sleeps > 0`, `woken > 0`), else it is `INCONCLUSIVE` and no `RINGIRQ_PASS` is printed. So a broken notification path fails although
the 10 ms re-check still delivers the data; `suppressed-irq` (command 15, irq consumer) must
recover through the re-check (`pulses=0`, `late > 0`, every wait under RECHECK + 40 ms).
`RINGIRQ_PASS` only if every run passes, else `RINGIRQ_FAIL`. HW (2026-11-05, cold boot x3): docs/design.md §6.2.

`shm-linux ring-dma-order [count]` runs the three production DMA sizes with both
legacy and enlarged odd-wrap batches, plus the six ordering controls. `count` applies to each production stream; each control is capped at
32,000 items. `RINGDMA_PASS` also requires early-head violations for every size. The latest
slot and batch-prefix errors are checked and deduplicated by logical slot index.

## Ring DMA speed measurement

STATIC: `lab/build-ring-dma.sh` stages a boot set containing the focused firmware, loader,
overlay and a rootfs with the current client, and records `SHA256SUMS`. Set
`RP1BOOT_ARTIFACT_ROOT`, `RP1_KERNEL` and `RP1_INITRAMFS` (base rootfs). Supply
`RP1_BOARD_CHECK_DTB` as an offline board DTB including the stepping/UART overlays, or the
firmware inputs documented by `build-rp1boot.sh`. It only builds local output; deployment
and hardware control are separate. `CARGO_BUILD_JOBS` defaults to 2. Use the same kernel,
rootfs, DTB, client and loader for baseline and candidate; only RP1.elf/bundle identifiers
should change. Build baseline firmware from `368854d`, not from the historical guard image.

HW plan (not executed here): fix clk_sys=200 MHz, clk_dma=100 MHz, channel 1, Linux mappings
and consumer. Record exact artifact hashes, toolchain and clocks. Use two cold boots for
each firmware. Every throughput condition warms 100,000 messages, then measures
1,000,000 messages five times. Compare per-boot median and range of Linux payload MB/s and
RP1 `total_us/count`; `push_us` alone excludes slice generation but includes direct generation.
`shm-linux ring-dma-perf` implements that plan and fails if any response/phase is missing,
corrupt, stale or erroneous. Each phase has a 180 s limit. Run IDs, size, requested batch,
start head, count, mode and timing flag appear in `RINGDMA_PERF_BEGIN`; every completed
phase has `RINGDMA_PERF_END ... status=PASS`. An absent end marker is failure.

```sh
# Baseline firmware: this uses only modes 0/1 and batches the original firmware accepts.
shm-linux ring-dma-perf --baseline
# Candidate: modes 0/1/5/6, same small batches plus the large candidates.
shm-linux ring-dma-perf
# Same baseline conditions on candidate, to isolate changes in the slice path.
shm-linux ring-dma-perf --baseline
# Repeat candidate sweep starting each phase at an odd logical head.
shm-linux ring-dma-perf --odd-head
```

The default batch sweeps are 16 B: 1/2/4/8/12/16/31/32/64/127/128/170;
64 B: 1/2/4/8/12/16/31/32/48/55/56; 256 B: 1/2/4/7/8/12/14/15.
`--ring 0|1|2`, `--batch N`, `--mode 0|1|2|5|6|7`, `--count N`, `--warmup N`,
`--repeat N` narrow the plan. Invalid or firmware-clamped batch sizes are rejected.
Run poll first; repeat selected conditions with `--irq`. For sleep/wake validation run
`--irq --gap-us 100` on each size's old and large batch, modes 1 and 5 (mode 1 on baseline).
Paced runs require sleeps > 0, woken > 0 and late=0; never sleeping is inconclusive/failing.

Add `--timed` on the candidate to repeat selected conditions with lab-only `TimedCopy`.
It records block copy time, last-copy-return to API-return time (`publish_us`), residual
prepare time, no-progress push time (`blocked_us`), total blocks, two-block batches and
8/16-byte width counts per block. Direct generation is timed inside each callback; slice
generation is timed per batch. Preparation is elapsed push time minus measured generation,
copy and publication; CPU publication is included in that residual. Timer reads and wrapper
bookkeeping cost time (1 µs resolution); compare timer-on/off e2e and total time, and treat
components as diagnostics. Generated payloads start zeroed on every path; the in-place DMA
path zeroes each whole slot before filling it. The HW speed run must measure that cost in
end-to-end/total and preparation time. DMA/CPU batch counts use actual copier invocation,
also without timing. Auto modes 2/7 require at least one DMA batch when the requested wire
batch reaches 256 B; an entirely CPU run fails the path check even under backpressure.
Without `--timed`, direct `gen_us=0` means generation is included in push time, and detailed fields are unavailable (zero). The shared lab loop uses the same
per-batch dispatch for each payload type and mode.

At each candidate cold boot, run `shm-linux ring-dma-order 32000` first. Each size's early-head
control must detect a violation; bit30-zero controls are reported, not judged by violation
presence. Then run the following ordering stress, using the same full-word observer:

```sh
for pair in '0 31' '0 127' '1 31' '1 55' '2 7' '2 15'; do
    set -- $pair
    repeat=10
    if [ "$1" = 2 ]; then repeat=5; fi
    shm-linux ring-dma-perf --ring "$1" --batch "$2" --mode 5 \
        --count 1000000 --warmup 0 --repeat "$repeat" --odd-head || exit 1
done
```

This is 20M/20M/10M messages per boot, 100M over two candidate boots. Split further if any
phase approaches 180 s. Require got=count, missing/corrupt/stale/errors=0 and all end markers.
A separate `dma`-feature image plus `shm-linux dma` supplies the raw-copy controls; retain
UART0 `RINGDMA_RAW ... bad=0` for each size/width. No ring publication occurs in that control.
Copy-failure injection is a separate transport experiment: fail/stop the second wrap block,
verify unchanged head and fencing on all retries, source stopped, and no destination reuse
until an external reset/drain. Host tests cover both block failures; this image does not add
a hardware failure-injection command.

Adopt only after direct generation improves e2e at the same batch, larger batches improve
selected workloads, and all validation completes. Finite HW runs qualify only those measured
conditions; they do not prove the HYP AXI-to-PCIe ordering chain. Larger stages (8 KiB),
double buffering and threshold retuning are outside this implementation.
