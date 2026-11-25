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

Both DMA features below (`dma`, `ring-dma`) enable rp1-link's feature `unstable-ring-dma`
(`SlotCopy`, `rp1_link::ring_dma::RingDma`; docs/dma.md).

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
HW (2026-11-11, two candidate cold boots): the minimum free margin was 448 B.
Changes to stack use require a new measurement.

STATIC: the focused ring image omits generic DDR microbench command 5 and up/down batch
commands 10–13 to fit the 48 KiB image budget. Omitted commands return an error. Use the
ordinary image for `shm-linux`'s full suite; ring-irq commands 14/15 remain available,
but their stream loop runs from shared SRAM here (the ISRAM is needed by command 17).
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

STATIC: every image keeps the `host_irq = true` declaration (rp1-link feature
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
| ring-dma (2026-11-11, focused image) | 47840 | 1312 | 7912 | 16 |
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
The allocator starts at `0xc0` after the lifecycle words and skips the SCMI region
`0x17c0..0x18c0`.
Linux loads `uio.ko`, then `uio_pdrv_genirq.ko of_id=generic-uio`, and runs `lab/check-shm.sh`.
UIO is polled, or (after shm enable) proc1 also raises MSI-X vector 29 on the `rp1:irq` UIO after
each doorbell reply (unstable `host_irq = true`, `linux-doorbell <link> <doorbell> N
/dev/uio<rp1:irq>`, which re-checks the reply every 10 ms without a wake and reports
`missed=`); hwirq58 stays with SCMI's mailbox. No cached mapping, kernel patch or private
kernel API is used. Device mappings never go through libc memcpy/memset.

STATIC: proc0 calls generated `rp1::link::try_enable(&mut pcie)` once when Linux requests
publication. It requires the PERSTn detector to be `Up`. The lab uses the hidden
`rp1::link::__map_for_probe` for its separate, pre-publication window test.
Each owning core moves its generated `p.link` into `try_open()` to attach its typed fields.
Pending publication returns the token for retry. Window loss or attachment failure ends
the wait. Block writes require the declared writer ownership. Linux `Firmware::open` checks UIO names, physical ranges, note hash and generation;
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

## Interrupt proof image (HW 2026-11-14)

`irq-proof` is a focused firmware that proves the interrupt path on hardware: handlers entered
and left on both cores, the `+0x108` wrapper of the DesignWare blocks, the user interrupt
section, the default handler and `log!` from a handler. HW (2026-11-14, cold boot ×2): the
proof and its regressions passed on the image as first built; later changes to the image and
its checker are STATIC until the next run. `IRQ_MASKED` now measures BASEPRI sections
(STATIC). Its system part is `shm-firmware`'s without the
I2C/ADC/SPI proofs (same owners and the rings `shm-linux` uses), so the SHM, doorbell and UART1
regressions run in the same boot. The doorbell is answered by proc1's `IO_BANK0` handler; main
code never answers it. Both stacks are the defaults, which the proof measures (proc0 2 KiB + the
768 B platform margin since 2026-11-25, STATIC; proc1 2 KiB; the HW run above had 2 KiB on both).

Every marker line on UART0 ends with `nonce=<8 hex digits>`, a value made at boot; the firmware
also stores it in the link page (+0x28) for Linux to print. Items that need no Linux run right
after boot. The proof ends when Linux has rung the doorbell 1000 times.

| Marker (UART0) | Shows | Decides |
|---|---|---|
| `IRQ_PROOF_BEGIN sweep=0` | start of a boot's proof | cuts the capture; `sweep=0` (no estimated line is raised in this image) |
| `IRQ_P0_TIMER0`, `IRQ_P1_TIMER1` | 1000 alarm interrupts at 1 kHz enter the handler and return | `n=1000`, `ipsr=42` / `43`, `us` within 999000..1010000, `pre_pending=0`, `post_pending=0` |
| `IRQ_INTF_ONLY core= pin=` | whether `PROCn_INTF` alone (INTE = 0) raises INTS and the NVIC line | only `pre_pending=0`, `post_pending=0`; `ints`, `nvic`, `need_inte` are findings (`IRQ_INTF_INTE` follows if INTE had to be set) |
| `IRQ_P0_GPIO22` | proc0's `IO_BANK0` handler, entered by `PinIrq::force` | `n=100`, `ipsr=16`, pending 0 before and after |
| `IRQ_WRAP i2c1`, `IRQ_WRAP spi0` | wrapper closed: the unmasked source is active in the controller but not pending; opened by `enable_interrupt`: one handler entry | `pre_pending=0 off_stat=1 off_pending=0 on_n=1`, `ipsr=24` / `35`, `post_pending=0` |
| `IRQ_CS` | main updates a two-word pair in critical sections against a 10 kHz handler | `iters>=100000`, `isr_n>=1000`, `torn=0`, `control_torn>=1` (the same update unmasked must be caught), `max_masked_us<=100` |
| `IRQ_LOG`, `IRQ_LOG_MAIN i=`, `IRQ_LOG_ISR k=` | a handler's `log!` is dropped without waiting while main writes | 20 main lines in order, `isr_dropped>=1`, `drop_max_us<=100`, as many `IRQ_LOG_ISR` lines as `isr_logged` |
| `IRQ_DEFAULT` | a line enabled without a handler is masked by the default handler; the core goes on | `irq=28 count=1 masked=1 pre_count=0 pre_pending=0` |
| `IRQ_P1_GPIO27` | Linux's 1000 doorbells were answered by proc1's handler | `seen>=1000`, `ipsr=16`; the round-trip time is `linux-doorbell`'s line on UART10 |
| `IRQ_XCORE` | no handler ran on the core it was not written for | both 0 |
| `IRQ_UNHANDLED` | unhandled interrupts per core | `p0=0 p1=1` (the `IRQ_DEFAULT` one) |
| `IRQ_STACK` | untouched bytes at the bottom of each stack (painted at entry) | both at least 512 |
| `IRQ_MASKED` | longest interval with user IRQs masked by `critical_section` per core, timed from an initially clear BASEPRI | both at most 100 µs |
| `IRQ_PROOF_PASS` / `IRQ_PROOF_FAIL <item>` | verdict; a failed item also prints `IRQ_PROOF_FAIL <item>` at once | PASS is the last marker, no FAIL line |

After the proof each `p1 shm cmd=` line carries `stack_free=`, which must stay at 512 or more.
The lines whose number was only an estimate before the sweep are not raised here; `irq-sweep` (below) does that.

Build a boot set with `FIRMWARE_PKG=irq-proof bash lab/build-rp1boot.sh` (the initramfs needs
`shm-linux`, `linux-doorbell` and the UIO modules, as for `check-shm.sh`). On the Linux console
run `lab/check-irq.sh`; it prints `IRQ_NONCE=`, rings the doorbell, runs `shm-linux` and the
doorbell again, and ends with `IRQ_CHECK_RESULT=` and `IRQ_CHECK_END`. Capture UART0 and UART10
from power-on (the bootloader banner is part of the check). The host decides with

```sh
python3 lab/verify-irq.py --uart0 UART0_CAPTURE --uart10 UART10_CAPTURE
```

which checks one boot (the last, or `--nonce HEX`): the values above, the nonce on every
marker line, the same nonce on the Linux console, `usb_pd_init status 3` and `max-current 3000`
in that boot's banner, and the Linux-side results (`--proof-only` leaves the SHM regression
out). It prints `IRQ_PROOF_VERDICT=PASS|FAIL`. Host tests: `python3 lab/test_verify_irq.py`.

## Interrupt sweep image (HW 2026-11-15)

`irq-sweep` answers one question for each line whose IRQ number was only an estimate before it: which NVIC
pending bit of proc0 does the block's interrupt output raise? No line is enabled, so no handler
runs; the image has no handlers at all.

HW (2026-11-15, 2 boots, identical results): all 9 lines ROUTED to the IRQ of the table
below, on the image as first built. The five DesignWare blocks pended only with the wrapper open
(`wrap0:0/3,wrap1:3/3`), the ADC with both stimuli (`force:3/3,real:3/3`), PWM1 and both IO
banks when forced (`force:3/3`); no other pending bit appeared, and the sweep took about 0.7 s.
Later changes to the image and its checker (the stricter verdict below, the reset and cleanup
handling) are STATIC until the next run.

| IRQ | Block | Stimulus | Runs only if |
|---|---|---|---|
| 7, 9, 10 | I2C0, I2C2, I2C3 | controller enabled and idle (`TX_EMPTY`), unmasked; wrapper `+0x108` closed (`wrap0`), then open (`wrap1`) | no pin selects the block's function |
| 20, 22 | SPI1, SPI3 | block enabled with an empty FIFO (`TXEI`), unmasked; `wrap0`, then `wrap1` | no pin selects the block's function |
| 52 | ADC | `INTF` forced (`force`); then one temperature conversion into the FIFO with `INTE` set (`real`) | the driver's `init` succeeded at startup |
| 41 | PWM1 | `INTF` forced; no channel is enabled | GPIO45 (fan) selects no function (FUNCSEL = NULL) |
| 1, 2 | IO_BANK1, IO_BANK2 | one bit of `PROC0_INTF` forced | `PROC0_INTE`, `PROC0_INTF`, `PROC0_INTS` all read 0 |

DMA (40), SPI5 (24), PIO (50/51), CLOCKS (59) and VBUS (60) have no safe stimulus and are not
in the table; the image fails to build, and the checker fails, if one is added.

For each line: prerequisites, then three trials of each stimulus. A trial checks the block's
baseline (explicit masks and values), clears the target pending bit, raises the source, reads
both pending words and the block's status, then deasserts: lowers the source, reads the block's
output back low, clears the target bit and checks that it stays clear. How the block is left
depends only on what was done to it (`exit=` on the result line): a block that was only read
(a prerequisite failed) gets no further access (`untouched`); a written one, however its line
ended, is deasserted once more, its interrupt enables and output must read 0, and it goes back
into reset (the ADC is powered down instead) (`left`); if a deassert fails, the block is not
touched again and the sweep stops (`exit=` names the failed step); a block whose reset does
not complete is treated the same way. Every wait is at most 1 ms. Pins are only read: the I2C and SPI drivers are never initialised. The sweep
runs only if both NVIC enable words read 0, and writes none.

Results, one `IRQ_SWEEP irq= block= result= ...` line each, between `IRQ_SWEEP_BEGIN` and
`IRQ_SWEEP_END lines=`: `ROUTED` (every trial of a stimulus set the target bit; `reason=` names
the stimulus), `NOT_ROUTED`, `OTHER_LINE` / `NEW_PENDING` (a pending bit that is not the
target's appeared, without / with the target bit; it is left pending and the line stops),
`SKIP reason=` (a prerequisite did not hold, or the source did not come up) and `ABORT reason=`
(the block or the NVIC did not return to the baseline; the remaining lines are skipped).
`IRQ_SWEEP_PRE` (function selects read, values found) and `IRQ_SWEEP_TRIAL` (every observation)
precede each result. A result says where the block's output is wired, nothing about a driver.

Ownership: the swept blocks are declared for proc0, so Linux's nodes are disabled, and the
sweep runs before the image brings the PCIe endpoint up. `rp1boot` starts Linux right after
RP1, but until the link is up Linux cannot reach an RP1 register. The price is that the sweep
(about 0.7 s) must end before Linux releases PERSTn.

Build a boot set with `FIRMWARE_PKG=irq-sweep bash lab/build-rp1boot.sh`.
Stage the [matching Linux status client](#linux-status-client) and place
`lab/check-core-status.sh` next to `lab/check-sweep.sh` on the target.
Run `check-sweep.sh` after the sweep. If the matching client was packed in the
initramfs by `pack-shm.py`, run `LINUX_STATUS=/bin/linux-status sh lab/check-sweep.sh`.
The checker records the boot value, device-tree status of all swept blocks,
and both cores' final state. The host decides with

```sh
python3 lab/verify-irq.py --sweep --uart0 UART0_CAPTURE [--uart10 UART10_CAPTURE] [--json OUT]
```

which checks one boot (the last, or `--nonce HEX`). A measurement PASS means every table line was routed by
its expected stimulus: `result=ROUTED exit=left` with `wrap0:0/3,wrap1:3/3` for I2C and SPI,
`force:3/3,real:3/3` for the ADC and `force:3/3` for PWM1 and the banks, borne out by its
`IRQ_SWEEP_TRIAL` lines (all trials in order, the source raised, the target bit clear before the
stimulus and after the deassert, no other new pending bit, the same hit counts). Any other result
(`NOT_ROUTED`, `OTHER_LINE`, `NEW_PENDING`, `SKIP`, `ABORT`) fails and is named. The frame must
hold as well: lines in table order with the boot's nonce, both enable words 0, and
`IRQ_SWEEP_END` before a `PLAT_INIT ok=1` line with a fresh PERSTn wait. It prints
`IRQ_SWEEP_MEASUREMENTS=PASS|FAIL` and `IRQ_SWEEP_VERDICT=PASS|FAIL|INCOMPLETE`.
A verdict PASS requires the matching UART10 check and both cores `Running`.
With UART0 alone, successful measurements produce an INCOMPLETE verdict (exit 3).
`--json` writes the per-line results.

## Platform stack and size checks (`stack-paths`, `plat-verify`, STATIC)

The platform handlers run on proc0's MSP on top of the user's code. With `T` the thread's
own use, `H_p` the bound of each user handler priority `p`, and `F = 36` B per exception
frame (32 B of state + 4 B alignment, no FPU), proc0's deepest chain is

```text
need0  = T + Σ H_p + max(H57, Hfault) + H53 + Hhard + (|U| + 3) · F
S_plat = max(H57, Hfault) + H53 + Hhard + 3 · F          (the platform's share)
stack0 = user + roundup(S_plat, 256)                     (user: declared, default 2 KiB)
```

`Hfault` is the deepest of the MemManage, BusFault and UsageFault handlers, which share
rp1-rt's configurable fault stub. `Hhard` is the bound of rp1-rt's HardFault path.
An NMI handler other than the default one would add
`Hnmi + F` on top; the platform enables no NMI source (HYP), so the default handler in the NMI
slot is not counted. The platform bounds only `S_plat`; the user part `T + Σ H_p` has no bound
here. rp1-build's `PLATFORM_STACK_MARGIN` (768 B) is that roundup for the largest `S_plat` of
this workspace's firmware (732 B, STATIC 2026-11-25). rp1-build adds it to proc0's user stack,
declared or default, and gives it to the linker as `__stack_margin0`: proc0's default stack is
2816 B, and `stack = { proc0 = "1KiB" }` gives 1792 B.

### Static bound (`stack-paths`)

`stack-paths` is a host tool. It disassembles a firmware ELF with `arm-none-eabi-objdump`
(`RP1_OBJDUMP` overrides it) and bounds the stack use of the proc0 platform roots, which it
takes from the vector table: IRQ57, IRQ53, HardFault, MemManage, BusFault, UsageFault and NMI.
It also checks both cores' fault vectors and reset paths (P2e/P5). A passing check verifies
that the fault stub restores its saved EXC_RETURN and entry SP after the ordinary Rust call,
and that startup sets CCR.STKALIGN before enabling the faults and clearing PRIMASK.
It reads proc0's stack and its
platform margin from the ELF (`__stack_size0`, `__stack_margin0`), prints `S_plat` and fails
when `S_plat` exceeds the margin, or the margin leaves no user stack (`reason=proc0-stack`). An
ELF without `__stack_margin0` (built before rp1-build had it) is an error (exit status 2):

```sh
cargo rp1 build -p irq-proof --target thumbv7m-none-eabi --release
cargo run -p stack-paths -- target/thumbv7m-none-eabi/release/irq-proof
# stack_fault_setup=PASS
# stack root=<symbol> vector=<slot> bytes=N   (one line per root)
# stack_static=PASS s_plat=N h57=N h53=N hhard=N hfault=N hnmi=none margin=768 stack0=2816
# stack_static=FAIL reason=<code> at=<symbol+offset> root=<symbol>        (exit status 1)
# stack_static=FAIL reason=margin|proc0-stack s_plat=N ... stack0=N        (exit status 1)
```

The rules and the conservative choices (tail calls, noreturn calls, jump tables, linker
thunks) are documented in `stack-paths/src/lib.rs`. Anything the tool cannot bound is a FAIL:
an SP write other than push/pop, `sub|add sp, #imm` or immediate SP-indexed writeback,
recursion, or an unresolved indirect call or jump. `--root <symbol>` adds a root and
`--cut <symbol>` treats calls to a function as 0 bytes and noreturn. A cut run prints
`stack_static=DIAG` and exits non-zero: it is a diagnostic, never a PASS.

A panic in an interrupt handler runs on that handler's stack, so the panic path is part of
the bound. The default rp1-rt handler records `Panicked` in the link page if the core was
running, sets BASEPRI to `0x40`, clears PRIMASK and waits in a `wfi` loop (STATIC / HW 未検証).
Both shm-firmware profiles, irq-proof, irq-sweep, pwm-loopback and scmi-starvation use this
handler. irq-proof selects `default-panic-handler` explicitly alongside its timed critical section.

irq-sweep, pwm-loopback and scmi-starvation report panics through the link page.
The sweep verifier also rejects `IRQ_SWEEP_PANIC` in legacy captures. Stage the
[matching Linux status client](#linux-status-client) before the final check. For PWM,
run `check-core-status.sh pwm "$NONCE"` after `PWM_DONE`, using `PWM_BOOT`'s
nonce. For SCMI starvation, run `check-core-status.sh scmi "$NONCE"` after
`SCMI_STARVE_END`, using `SCMI_STARVE_READY`'s nonce. Require
`LAB_FINAL_END` with `result=PASS` and both cores `Running` in the captured
Linux output. These checks are STATIC until the next hardware run.

STATIC (2026-11-25, release builds, rustc 1.99.1, bytes):

| Firmware | `stack_static` | H57 | H53 | Hhard | Hfault | Hnmi | `S_plat` |
|---|---|---:|---:|---:|---:|---:|---:|
| examples/dual-core | PASS | 592 | 8 | 24 | 56 | none | 732 |
| irq-proof | PASS | 440 | 8 | 24 | 56 | none | 580 |
| irq-sweep | PASS | 440 | 8 | 24 | 56 | none | 580 |
| pwm-loopback | PASS | 440 | 8 | 24 | 56 | none | 580 |
| scmi-starvation | PASS | 440 | 8 | 24 | 56 | none | 580 |
| shm-firmware (default) | PASS | 440 | 8 | 24 | 56 | none | 580 |
| shm-firmware (ring-dma) | PASS | 440 | 8 | 24 | 56 | none | 580 |

Each image has `stack0=2816` and `margin=768`. Removing panic-time formatting resolved the
`core::fmt::write` indirect-call failures in irq-sweep, pwm-loopback and scmi-starvation.

### Verdict (`plat-verify.py`)

`plat-verify.py` takes the ELF and the UART captures of one boot of that ELF, records their
sha256, runs `stack-paths --size` (through `cargo run` in this repository, or `--stack-paths
EXE` / `STACK_PATHS`), records which stack-paths ran (the executable's sha256, or the
repository's commit and a dirty flag) and prints one row each, PASS, FAIL or INCOMPLETE:

```sh
python3 lab/plat-verify.py --elf target/thumbv7m-none-eabi/release/irq-proof \
    --uart0 UART0_LOG --uart10 UART10_LOG
# stack_static=PASS s_plat=580 ...       static bound (stack-paths)
# stack_watermark_free=PASS p0_min=N p1_min=N n=N need=512 stack0=N   observed, separate row
# size=PASS image=N/49152 ...            every budget has room left
# plat_verify=PASS|FAIL|INCOMPLETE
```

`stack_watermark_free` is the smallest untouched stack space the firmware reported after its
load, per core: proc0 from `IRQ_STACK p0_free= p0_size=`, proc1 from `p1_free=` and
`p1 shm cmd= ... stack_free=` (irq-proof) or `RINGDMA_STACK free_bytes=` (ring-dma). It is an
observation, not a bound or an overflow guard. The row is about proc0's headroom with the
platform handlers on top, so it needs a proc0 value and its size: proc1 lines alone (ring-dma,
shm-firmware) are INCOMPLETE. A `p0_size=` or `p1_size=` that differs from the ELF's `stack0`
or `stack1` is FAIL `reason=other-image`: the capture is of another build. Give the capture of
one boot only. A row without evidence (no ELF, no capture, no line, the tool did not run) is
INCOMPLETE, never PASS. Exit status 0 = PASS, 1 = FAIL, 3 = INCOMPLETE. Host tests:
`python3 lab/test_plat_verify.py`. HW 未検証: no boot of these images has been checked with
`plat-verify` yet.

### Platform size (`stack-paths --size`, STATIC)

`--size` first prints what the platform costs in the image. It is host-side only: the ELF
holds everything (no UART line; the firmware does not report its own size).

```sh
cargo run -p stack-paths -- <firmware.elf> --size
# size budget image=N/49152 sram_free=N isram=N/8192 isram_free=N dsram_static=N stack0=N stack1=N margin=768 dsram_free0=N dsram_free1=N
# size owner=platform|runtime|user|unattributed code=N data=N bss=N
```

`image` is the shared-SRAM image (`__image_end`, as `cargo rp1 build` reports it), `stack0`
includes the platform `margin` (both from the ELF), and `dsram_freeN = 8192 - dsram_static - stackN`. Owners
come from symbol names: platform = `rp1_rt`, `rp1_hal`, `rp1_link`, `rp1_proto`, `rp1_pac`, `__rp1_*`,
the vector tables, the reset code and the `rp1` module that rp1-build generates; runtime =
`core` and compiler-builtins; user = the firmware crate and other named symbols;
unattributed = anonymous constants, literal pools and padding. LTO inlines platform code into
user functions (for example `platform::init` into `main`), so the platform figure is a lower
bound and the user figure an upper bound. The compiler's panic entry symbol is attributed to
user code by name, including when rp1-rt supplies the handler.

STATIC (2026-11-25, rustc 1.99.1, release `cargo rp1 build`; bytes):

| Firmware | Image / 49152 | SRAM free | ISRAM / 8192 | Platform code | Runtime code | User code | Unattributed | stack0 (margin) / stack1 | DSRAM free p0 / p1 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| examples/dual-core | 27200 | 21952 | 1136 | 11290 | 6202 | 6176 | 3260 | 2816 (768) / 2048 | 5264 / 6032 |
| irq-proof | 43768 | 5384 | 4224 | 16604 | 6374 | 15936 | 4514 | 2816 (768) / 2048 | 5104 / 5872 |
| irq-sweep | 27400 | 21752 | 316 | 8018 | 4746 | 10832 | 3590 | 2816 (768) / 2048 | 5296 / 6064 |
| pwm-loopback | 18304 | 30848 | 324 | 8134 | 2892 | 5472 | 1578 | 2816 (768) / 2048 | 5304 / 6072 |
| scmi-starvation | 16264 | 32888 | 536 | 7190 | 2696 | 4482 | 1688 | 2816 (768) / 2048 | 5288 / 6056 |
| shm-firmware (default) | 44096 | 5056 | 3664 | 17418 | 9110 | 12902 | 4402 | 2816 (768) / 6144 | 5288 / 1960 |
| shm-firmware (ring-dma) | 44944 | 4208 | 8044 | 20266 | 5622 | 10974 | 3712 | 2816 (768) / 6144 | 5288 / 1960 |

Data and bss are below 130 B per owner in every image except ring-dma's 4136 B of user
`.shared` data. The stack margin lives in DSRAM, so the SRAM images did not grow with it.

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
8/16-byte width counts per block. Slice generation is timed per batch; direct generation
is not timed separately (`gen_us=0`, also with `--timed`: it is part of push and prepare
time). Preparation is elapsed push time minus copy and publication; CPU publication is
included in that residual. Timer reads and wrapper
bookkeeping cost time (1 µs resolution); compare timer-on/off e2e and total time, and treat
components as diagnostics. Generated payloads are written in place over the previous stage
contents (nothing is zeroed per message). DMA/CPU batch counts use actual copier invocation,
also without timing. Auto modes 2/7 require at least one DMA batch when the requested wire
batch reaches 256 B; an entirely CPU run fails the path check even under backpressure.
Without `--timed` the detailed fields are unavailable (zero). One batch loop serves every
payload type and mode. It runs from ISRAM, like the slice generator and the ring API; only
the per-ring dispatch of each push (about 35 instructions) runs from shared SRAM.

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

## Real Raspberry Pi OS environment (`realos/`)

Use the **stock** Raspberry Pi OS kernel (6.12.75+rpt-rpi-2712, trixie) in an rp1boot
bundle with an NFS root on a disposable copy of the Raspberry Pi OS root file system.
The configuration selects `multi-user.target` and root autologin on ttyAMA10.

HW: the stock kernel has no SCMI (`# CONFIG_ARM_SCMI_PROTOCOL is not set`). The overlay of
`clock.authority = "rp1"` moves every RP1 clock consumer Linux keeps (Ethernet included) to
`/firmware/scmi`, so with the stock kernel alone no RP1 device that needs a clock probes and
there is no network for the root file system. `build-scmi-modules.sh` builds the SCMI core,
the mailbox transport and `clk-scmi` as modules from the `linux-source-6.12` package of the
same Debian version (the kernel image itself is unchanged); the initramfs loads them before
it waits for `eth0`.

| Step | Script | Runs as |
|---|---|---|
| SCMI modules | `build-scmi-modules.sh` (`KSRC`, `KHEADERS`, `OUT`) | user (cross gcc-14) |
| Root file system | `prepare-rootfs.sh` (`ROOTFS`, `SCMI_MODULES`, `CLIENT_BIN`) | root |
| initramfs | `build-initramfs.sh` (`ROOTFS`, `OUT`): the root file system's initramfs-tools in a chroot, `MODULES=list`, `BOOT=nfs`, xz. Requires `unmkinitramfs` on the host | root |
| Boot set | `build-bootset.sh` (`ROOTFS`, `RP1_ELF`, `RP1BOOT_IMG`, `INITRAMFS`, `NFS_ROOT`, `OUT`). Requires `lsinitramfs` and a new output directory with an existing parent | user |
| Target check | `check-realos.sh`, installed as `rp1-realos-check` | root on the target |
| Device check | `check-linux-devices.sh`, installed as `rp1-device-check` | root on the target |

- Command line: `console=ttyAMA10,115200 root=/dev/nfs nfsroot=<server>:<path>,nfsvers=3 rw
  uio_pdrv_genirq.of_id=generic-uio`. No `ip=`: the kernel's own IP autoconfiguration would run
  before the SCMI modules exist; the initramfs does DHCP on `eth0`.
- The NFS export needs `rw,no_root_squash`; NFSv3 (klibc `nfsmount`).
- `prepare-rootfs.sh` saves selected configuration files and module indexes as
  `<file>.orig-realos`. Discard the prepared copy to undo all changes, including module
  and client installs.
- Before replacing `OUT`, `build-initramfs.sh` runs the unpacked `/bin/sh` in a chroot
  and checks for executable entry points and the required NFS and module files.
  `build-bootset.sh` checks the archive's module release against the selected kernel.
  Both accept `KERNEL_RELEASE` when the root copy has several prepared releases.
  HW (2026-10-28) verified bundle loading up to 32 MiB. Measure the new bundle before
  relying on that boot range. When reusing a copy from the earlier experiment, remove its
  `etc/initramfs-tools-rp1nfs/hooks/zz-rp1nfs-prune` hook first.
- `check-realos.sh` prints an inventory (dmesg, lsmod, interrupts, bound platform devices,
  deferred probes, i2c/spi/uio nodes, lsusb). PASS requires a mounted NFS root, active
  `multi-user.target`, the `rp1:link`, `rp1:doorbell`, and `rp1:header` UIO nodes, and
  successful `linux-doorbell` polling (1000 round trips by default with `examples/dual-core`).
  For an SD boot with the same UIO setup, set `EXPECTED_ROOT_FSTYPE=ext4`.

HW (2026-11-16, `examples/dual-core`, cold boot ×2 + `systemctl reboot` ×1; firmware from
before SCMI moved to IRQ 57, not repeated since): multi-user in
14–16 s of systemd time, three UIO nodes, doorbell rtt p50 about 2.3 µs, check PASS on each
boot. `systemctl reboot` resets the whole board (new bootloader banner, rp1boot reloads the RP1
firmware). `systemctl poweroff` powers down (no UART output, Ethernet link down); the power
button sequence brings it back.

### Linux status client

Build the client from the same checkout and commit as the firmware.

```sh
cargo build --locked -p linux-status --target aarch64-unknown-linux-musl --release
```

For realos, include the resulting `linux-status` in `CLIENT_BIN` when running
`prepare-rootfs.sh`. It installs the client at
`/usr/local/libexec/rp1-lab/linux-status` on the target, the default for
`check-core-status.sh` and `fault-campaign.sh` preflight.

For a one-off check, transfer the client into a private directory on an
executable target filesystem such as `/tmp`. Verify its SHA-256 against
the build artifact, set its execute bit, and set `LINUX_STATUS` to its
absolute target path. Export it on the target for `check-core-status.sh`
or on the host for `fault-campaign.sh`. Stage the client again after a
reboot if the scratch directory is gone. The checks fail if the selected
client cannot run. HW (2026-11-25, realos) observed `/run` mounted with
`noexec`.

### Linux-side RP1 devices (`check-linux-devices.sh`)

`build-bootset.sh` takes `CONFIG_EXTRA` (extra config.txt lines, one per line) and `OVERLAYS`
(`.dts` files compiled with `dtc` into the bundle's `boot/overlays/` and enabled with
`dtoverlay=` lines) to enable Linux devices. Copy `boot/overlays/*.dtbo` to the boot server's
`overlays/` together with the rest of `boot/`. Run the installed checker as root on the target:

```sh
PING_HOST=192.0.2.10 rp1-device-check \
    i2c2 i2c6 spi1.0 pwm0 fan eth:eth0 usb:3-1
```

Replace the example address with a peer reachable through the selected interface and use
the bus numbers and USB port from your setup. Use `usb-storage:sda` for a direct disk read.
The short I2C scan defaults to 0x08–0x0f. Set `FULL_SCAN=1` to opt into the full scan. The
full scan runs only when the short scan took less than five seconds; otherwise its row fails
with `reason=short_scan_slow`. The full scan uses i2cdetect's default quick-write/read-byte
probing, so enable it only on buses without write-sensitive devices.

The script emits a `DEVCHECK` row per probe, with the same keys on every row, and continues
after failures. It ends with one `DEVCHECK_END result=PASS|FAIL` line. I2C scans, the 16-byte
SPI transfer, Ethernet ping, and USB operations require an IRQ count increase for the
selected controller. The 4096-byte SPI transfer goes through DMA and requires an increase on a
shared DMA line. An increase is necessary, not attributed to the probe: background traffic
(the NFS root on `eth0`, other xHCI devices, other DMA users) also counts, and the ping replies,
the USB status reply and the disk read are the operation evidence. The I2C, SPI, Ethernet, and
USB probes also require readable kernel logs with no matching timeouts during the probe.
Failing `evidence=` codes are `no_cursor`, `log_lost`, `timeout`, `no_irq`, `io_error` and
`unavailable` (see the script header). PWM reads its sysfs state back (`evidence=sysfs`: the PWM
core accepted the setting, not a measured GPIO12 output), and an
already-exported PWM channel produces a failed probe. The fan probe requires a nonzero tach
reading (`evidence=tach`), so it needs a fan with its tach wire connected; leave `fan` out
otherwise. The register dump uses read-only `busybox devmem` on bound
blocks. Record the source revision and SHA-256 of the external `RP1_ELF` when comparing with
stock RP1 firmware.

`overlays/linux-spi1-pwm0.dts` enables SPI1 (GPIO18–21, spidev) and PWM0 channel 0
(GPIO12) with pin groups in the string form. The stock `spi1-1cs` and `pwm` overlays use the
legacy numeric `brcm,pins` / `brcm,function` form. rp1boot and `cargo rp1 check --dtb`
translate it as pinctrl-rp1 does and refuse it only on RP1-owned pins or signals (STATIC).
Before 2026-11-17 the numeric form was refused on any RP1 GPIO and the boot stopped
(`RP1BOOT_LINUX_REFUSED`, HW); the user replaced that policy. The stock `pwm` overlay and
`spi1-1cs` both default to GPIO18, so use them together as `dtoverlay=pwm,pin=12,func=4`.

Pre-fix history (HW, 2026-11-17): I2C2/I2C6 and the 16-byte SPI1 transfer timed out with
I2C/SPI wrapper `+0x108` bit 0 clear. The 4096-byte SPI1 transfer completed using DMA.
USBHOST `+0x98` was zero while xHCI interrupts arrived.

STATIC: rp1-hal enables the Linux-owned I2C/SPI wrapper gates and programs the USBHOST
wrapper words before Linux starts. HW (2026-11, [interrupts](../docs/interrupts.md#4-根拠)):
Linux-side IRQ delivery observed for I2C2, I2C6, SPI1, Ethernet, and USB. HW (2026-11-24, cold
boot, firmware with SCMI on IRQ 57): this checker passes for I2C2, I2C6, SPI1 (16 and 4096 bytes),
PWM0, Ethernet, and USB, and `rp1-realos-check` passes. The fan row was not verified: the test
board reads no tach pulses, also with stock RP1 firmware.

## PR-E fault injection (`fault-inject`)

STATIC / 実機未検証. This lab image uses the same RP1 clock authority and
`platform::init` as the realos firmware. The standard kernel, SCMI modules,
Ethernet and USB setup are described in [Real Raspberry Pi OS environment](#real-raspberry-pi-os-environment-realos).
Deploy the lab image in its own realos boot set. It owns timer alarms 0 and 1
and accepts one request per firmware boot through the link page's lab scratch
range. The RL v3 status layout is shared with the regular firmware.

```sh
cargo rp1 build --locked -p fault-inject --bin fault-inject --features firmware \
    --target thumbv7m-none-eabi --release
cargo build --locked -p fault-inject --bin fault-linux --features linux \
    --target aarch64-unknown-linux-musl --release
cargo build --locked --release -p stack-paths
```

Install `fault-linux` and `rp1-device-check` on the prepared realos target.
Stage the [matching Linux status client](#linux-status-client) for campaign preflight.
Start both UART captures before boot or the Linux shutdown command. Run
`fault-linux KIND NONCE HOST_IPV4 USB_PORT SCMI_N` from the root UART10 console once
per boot. Supply the boot nonce from UART0, the host's Ethernet address,
the attached USB port (for example `3-1`), and an SCMI request count from
1 to 100. The debugfs clock
`/sys/kernel/debug/clk/uart/clk_rate` must be available. Use a quiet target
during the SCMI samples so every increase can be attributed to the test.
Before the matrix, verify on one boot that N clock reads increment SCMI's
request count by N. Cached reads cannot establish this evidence.

| Kind | Expected RL v3 result |
|---|---|
| `p0-thread-panic`, `p0-isr-panic` | proc0 `Panicked` |
| `p0-thread-udf`, `p0-isr-udf` | proc0 `Fault`, reason 6, CFSR.UNDEFINSTR |
| `p1-panic` | proc1 `Panicked`, proc0 stays `Running` |
| `stkerr` | HardFault diagnostic with a PSP stacking error. Requires an MPU, reason 3, a stacking-error CFSR bit and PC 0. |
| IRQ57 panic | Diagnostic unavailable. IRQ57 belongs to the platform and has no lab injection hook. |

The two 20 Hz, priority-4 timer counters must advance before the request. The
target core's count must stay fixed over at least one second after its terminal
record. For proc1 panic the proc0 timer must keep advancing. The peer must
remain `Running` at both the terminal observation and the final sample.
The firmware publishes the stimulus kind, core and IPSR immediately before
the injection. The target reports that site, ping 20/20 over `eth0`, the
USB control probe, the SCMI replies and counter delta, PCIe state, and
`FAULT_DMESG` counts from the invocation's kernel log window. UART0 supplies
the pre-injection `FAULT_BOOT` nonce.

Pass the deployed boot-set ELF, bundle, manifest and fault-linux SHA-256 to
the host verifier. Supply one
`--run KIND PHASE NONCE UART0 UART10` per capture pair. The nonce is the
eight-digit hex value from `FAULT_BOOT`. Use `cold-1`, `cold-2`, `reboot` and
`poweroff` for each of the five required kinds. Keep each pair from a distinct
boot. For `reboot` and `poweroff`, keep the Linux shutdown and bootloader
markers in the same UART10 capture. For poweroff also supply the capture's
`continuous/events.jsonl`, `continuous/timestamps.jsonl` and `boot.json`
(one directory above `continuous/`). They tie the carrier-low and force-boot
events to the UART transition.

Cold cells are a force-boot from a running Linux. The power-key press of the
force-boot can let Linux power down gracefully (`reboot: Power down`, then the
bootloader's `Halt: power_off: 1` / `Power off`) before the hard cut. Supply the
same three records for cold captures too. The verifier accepts such a halt as a
cold boot only when both halt lines are stamped after the force-boot command
started, the force-boot succeeded, the cell sent no reboot or poweroff command,
and a `power-on-reset 1` boot follows. It reports the boot row as
`detail=cold-via-shutdown`. A halt without these records stays INCOMPLETE.

The target shell wraps command output in bracketed-paste toggles
(`ESC[?2004l` / `ESC[?2004h`). `lab/fault-campaign.sh` removes only these two
sequences before its line-anchored checks and keeps the console text in
`LOG.raw`.

Do not open the ESP32 control port while the CM5 is halted or powered off.
Opening the port resets the ESP32. The lab I2C-peer firmware
(`lab/esp32-i2c-peer`) then asserts at `Wire.begin` on the unpowered CM5 I2C bus
and boot-loops, so it cannot send force-boot (HW, 2026-11). Recovery needs a physical CM5
power-on (IO-board power button or supply replug). Do not start a cold cell
from a halted CM5. Start it from a running Linux.

```sh
python3 lab/verify-fault.py --elf "$BOOT_SET/rp1.elf" \
    --manifest "$BOOT_SET/manifest.txt" --bundle "$BOOT_SET/rp1-bundle.img" \
    --client-sha256 "$FAULT_LINUX_SHA256" --stack-paths target/release/stack-paths \
    --run p0-thread-panic cold-1 "$NONCE" "$UART0" "$UART10"
```

The verifier checks the bundle firmware payload against the ELF, the header
ID against the loader banner, the client hash, the nonce and the boot
identity before combining evidence. A UDF observation must report the
linked stimulus address as its stacked PC. It prints `FAULT_RUN` per boot, `FAULT_KIND` per
required kind and `PLAT_VERDICT` for the full matrix and static stack bound.
Exit 0 is PASS, 1 is FAIL, and 3 is INCOMPLETE. Missing evidence or a
nonce collision across distinct captures needs a fresh capture. `FAULT_DIAG` and `PLAT_LOST=INCOMPLETE` are observational.
