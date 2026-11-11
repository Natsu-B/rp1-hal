# RP1 DMA driver

`rp1_hal::dma` drives the RP1 DMA controller, a Synopsys DW AXI DMAC (1.02a-lp02, eight
channels, one 128-bit AXI master). v1 does single-block memory-to-memory transfers with polled
completion. Evidence strength: **HW** measured, **STATIC** code/tests, **DOC** datasheet or
Linux source, **HYP** hypothesis.

## Ownership

```toml
[package.metadata.rp1.proc1]
dma = {}          # the whole controller: reset, clk_dma, the DMA interrupt line, all 8 channels
```

- The controller is one block (one reset bit, one clock, one interrupt), so it is declared whole
  by one core, like `adc`. Pins: none. Two cores, or `dma = { .. }` with keys, are build errors
  (E3 / E0). STATIC: `rp1-build` tests.
- `DMA` is no longer in the board's Linux-required list (BREAKING for the loader's protected
  nodes). Without the declaration nothing changes: Linux keeps `rp1_dma` with SCMI clocks.
- With the declaration the generated overlay disables `rp1_dma`, the image's `linux_resets`
  leave RESETS CTRL0 bit 3 alone, and the SCMI server holds clk_dma (Linux cannot change it).
  STATIC: `rp1-build` artifacts test; HW (2026-11-05): `rp1_dma` reads `status=disabled` and
  no DMA interrupt appears for it in Linux.
- Linux consumers: an enabled node whose `dmas` names `rp1_dma` would defer forever (of-dma
  returns -EPROBE_DEFER for an unregistered provider, DOC). The loader's resolver treats it like
  a `pwms` consumer of an owned PWM: `on_conflict = "disable"` disables the node with
  `why=dmas uses RP1-owned block rp1_dma`, `"refuse"` stops. Specifiers are parsed with the
  provider's `#dma-cells` (a DREQ number equal to a phandle is not a reference). STATIC:
  `rp1-dt` resolve test. HW (2026-11-05): with the standard CM5 IO DTB + `uart1-pi5` no enabled
  node uses `rp1_dma`; the loader disabled only `gpiomem` (as before). An overlay enabling an
  RP1 SPI/I2S node (they carry `dmas = <&rp1_dma ..>`) would now lose that node. `rp1_pio` is
  different: it is a protected node (in the board's Linux-required list), so a `dmas` conflict
  on an enabled `rp1_pio` makes the resolver **refuse** the plan even with
  `on_conflict = "disable"` (STATIC: `rp1-dt` resolve, protected nodes are never disabled).

## API

- `dma::Token::init(&Clocks)` (once): clk_dma 100 MHz (DOC: datasheet ch. 9), release the reset,
  `DMAC_RESET` soft reset, every channel disabled with interrupts masked, `DMAC_CFG.DMAC_EN`.
  Returns `Dma { ch0..ch7 }`.
- `Channel<N>::copy(&[u8], &mut [u8], timeout_us)`: safe, blocking. Both slices must be
  contiguous DMAC-visible memory (shared SRAM or the outbound window; peripheral registers are
  refused, use `start`); splits into blocks. A timed-out or failed block is stopped before
  return (see Cancellation); `copy` never returns while the channel can still access a buffer.
- `unsafe Channel<N>::start(src: BusAddr, dst: BusAddr, len) -> Transfer`: one block,
  `Transfer::poll()` / `wait(timeout_us)` / `cancel() -> Stopped`; dropping an unfinished
  transfer stops it the same way. The caller guarantees the memory (a leaked `Transfer` keeps
  the DMA running).
- Errors: `Bus(CH_INTSTATUS)` on decode/slave errors, `Timeout`, `Busy`, `Invalid`,
  `NotDmaVisible`, `Precondition` (clk_dma not 100 MHz). With `Bus` and `Timeout` the channel
  has already been stopped.
- Programming (DOC, Linux `dw-axi-dmac` with the RP1 DT properties): 8-channel register map
  (`CHEN` write enables at bit 8), `CFG2` layout (`snps,dma-targets = <64>`), contiguous
  source/destination, memory-to-memory with DMAC flow control, priority = channel, MSIZE 4,
  width = largest power of two dividing addresses and length (max 16 B). AXI burst length from
  `snps,axi-max-burst-len = <8 8 4 4 4 4 4 4>` (= the datasheet FIFO depths, 8 / 4 beats),
  written as `beats - 1`: ARLEN/AWLEN use the AXI encoding (Linux `DWAXIDMAC_ARWLEN_8 = 7`).
  Linux validates the DT value as a beat count (`<= ARWLEN_MAX + 1`) but writes it unchanged,
  i.e. one beat more than the FIFO (DOC: `dw-axi-dmac-platform.c`); this driver does not copy
  that. `HS_SEL_SRC/DST = 1` (software) where Linux leaves 0; for memory-to-memory with DMAC
  flow control no handshake is used either way (HW: both copy correctly). Single-block contiguous mode
  (no LLI) is not what Linux uses; it works on RP1 (HW below).
- Not in v1: linked lists, interrupts (the line is RP1 IRQ 40, unused), peripheral handshakes.
- Ring producer: `shm::RingDma` (below, "DMA ring producer").

## Cancellation and the abort register

Linux 6.12 (`drivers/dma/dw-axi-dmac/`) picks the register map from `dma-channels`:
`parse_device_properties` sets `reg_map_8_channels = true` for `dma-channels <= 8` (`DMA_REG_MAP_CH_REF`).
With that map `CH_EN`/`CH_EN_WE` are `DMAC_CHEN[7:0]`/`[15:8]` and `CH_SUSP`/`CH_SUSP_WE` are
`DMAC_CHEN[23:16]`/`[31:24]` (`axi_chan_enable`, `dma_chan_pause`); only without it does
Linux use `DMAC_CHSUSPREG` (0x020) and the `EN2` write-enable at bit 16. RP1's `rp1.dtsi` has
`dma-channels = <8>` and the datasheet says eight channels, so RP1 uses the 8-channel map. Linux
never aborts a channel (`dma_chan_terminate_all` writes `CH_EN = 0` and polls). By the same
layout (the Synopsys databook as cited in review; not read here) `CH_ABORT`/`CH_ABORT_WE` are
`DMAC_CHEN[39:32]`/`[47:40]`, i.e. bits 0..7 / 8..15 of the high word at 0x01C;
`DMAC_CHABORTREG` (0x028) belongs to the >8-channel map. The first version wrote 0x028, which
on RP1 does not request an abort (that path was never exercised on HW).

The driver now stops a channel with: `CH_SUSP` and up to 10 ms for `CH_SUSPENDED`; `CH_EN = 0`
and up to 10 ms for the enable bit to clear; only then `CH_ABORT` (0x01C) and another 10 ms. If
the channel is still enabled it spins without a deadline: a safe API must not end the buffer
borrows while the DMAC can still access them, and spinning is sound under any panic strategy
(a panic could unwind on a host target). A wedged DMAC therefore hangs the calling core. The
suspend request is cleared afterwards so the next enable runs. This applies to `wait`/`copy`
timeouts, bus errors, `Transfer::cancel` and `Drop for Transfer`. STATIC: unit tests with a
fake DMAC whose disable, abort, or both take effect late (the call returns only after the
enable bit cleared). HW: lab abort proof below.

## Address map

The DMAC is a 40-bit bus master and does not see the processors' 32-bit map (DOC: RP1
Peripherals §2.2 table 1, §2.3.1). `BusAddr::from_proc` translates:

| Processor address | Bus address (DMAC) | Evidence |
|---|---|---|
| shared SRAM `0x2000_0000..0x2001_0000` | `0xC0_2000_0000 + x` | DOC; HW (SRAM→SRAM, SRAM source to DDR) |
| APB/AHB peripherals `0x4000_0000..0x4019_0000` | `0xC0_4000_0000 + x` | DOC (only the DMAC may access peripheral registers); not exercised |
| outbound window `0x8000_0000..0xc000_0000` | `0x80_0000_0000 + x` (ATU space) | DOC; HW: through ATU region 0 that `shm` programs |
| host physical `p` (`BusAddr::host_phys`) | `0x10_0000_0000 + p` (direct-mapped PCIe) | DOC (RP1 `dma-ranges`); HW: works without any ATU region |
| ISRAM / DSRAM `0x1000_0000..`, ROM | not visible | DOC (processor-local); `NotDmaVisible` |

Consequences: stacks and ordinary `static`s (DSRAM) cannot be DMA buffers; use a
`#[unsafe(link_section = ".shared")]` buffer or shm memory. The PCIe AXI APB peripherals at
processor `0x4040_0000` map to bus `0xC0_4020_0000` and are not translated (rejected).

## Ordering and caches

- The Cortex-M3 has no data cache: no clean/invalidate is needed for DMA buffers.
- Channel setup writes are posted in program order to the DMAC (Device memory, one AHB port).
  Right before the channel-enable write, `rp1_proto::barrier()` (compiler fence + `dsb`)
  completes earlier CPU stores to the source buffer and keeps the compiler from moving
  ordinary buffer stores past the enable.
- Completion is the observed `DMA_TFR_DONE` plus cleared `CHEN`, followed by a barrier
  (STATIC). Every block explicitly sets `CTL_L.NonPosted_LastWrite_En` (bit30) and `AW_CACHE=0`
  (Device Non-bufferable). The earlier claim that DONE alone implied the final AXI write
  response was unsupported and is withdrawn. Linux names bit30 `CH_CTL_L_LAST_WRITE_EN` (DOC).
- [ARM IHI 0022](https://documentation-service.arm.com/static/63ff0ebd56ea36189d4e7ee7) specifies that a non-bufferable write response comes from the final AXI
  destination (DOC); that destination is the endpoint bridge, not host DDR. The interpretation
  that bit30 waits for the last response, that it covers prior writes of the channel, and
  that acceptance at the endpoint orders later M3 window stores behind them is HYP for RP1.
  Register readback and finite error-free streams cannot prove those internal connections.
- DOC cross-check: [CVITEK datasheet, pp. 152–153](https://github.com/milkv-duo/duo-files/blob/main/duo/datasheet/CV1800B-CV1801B-Preliminary-Datasheet-full-en.pdf)
  documents AW_CACHE[29:26] and bit30 synchronizing completion with the last write in another
  DW AXI DMAC integration. This supports the field interpretation, not RP1's internal wiring.
- Error/cancellation stops the engine accessing the source; accepted outbound writes can
  still be in flight. A barrier after cancellation is not a remote drain.

## HW results (2026-11-05, lab `shm-firmware --no-default-features --features dma`, clk_sys 200 MHz)

Correctness: SRAM→SRAM copies of 4, 12, 16, 64, 1000 and 1024 bytes exact, untouched tail
intact; SRAM→Linux DDR: 2 × 512 one-KiB copies (one half through the ATU window, one through the
direct-mapped space) into a canary-filled 1 MiB block, every word verified by Linux and by an
RP1 read-back over PCIe (0 bad, no canary left).

Throughput and latency (RP1 timer, 1 µs; per block = setup + transfer + completion poll):

| Measurement | v1 before the perf commit (1 boot) | v1 (relaxed setup + ISRAM, cold boots) |
|---|---:|---:|
| SRAM→SRAM 1 KiB block, ch0 / ch2 | 21.9 / 22.3 µs | 15.0 / 14.6 µs (68 MB/s; CPU word copy 84.4 µs) |
| SRAM→DDR 16 B block | 14.8 µs | 5.0 µs |
| SRAM→DDR 512 B block | 16.4 µs | 7.1 µs (72 MB/s) |
| SRAM→DDR 1 KiB blocks, ATU / direct | 46.2 / 46.2 MB/s | 86.4 / 86.4 MB/s |
| SRAM→DDR 64 KiB blocks, ATU / direct | 203 / 203 MB/s | 210 / 210 MB/s |
| CPU posted writes to DDR (same data, word stores) | 8.54 MB/s | 8.54 MB/s |

(MB = 10^6 bytes; the right column is from two cold boots, which agree within 1 %.) The ATU window and the direct-mapped space perform the
same. Fixed per-block cost is about 5 µs (16 B block), so DMA beats CPU posted writes
(0.54 µs per word) from about 10 words (40 B) per block upward.

### Cancellation proof and burst-length change (2026-11-05, 3 cold boots)

Lab step in command 16 (channel 2, byte-wide 4094 B SRAM→DDR block through the ATU window,
full block 272 µs): cancelled at 68 / 136 / 204 µs, every time stopped by suspend + disable
(`CH_SUSPENDED` seen, no abort needed) about 7 µs after the request; `CHEN` bit clear on
return; the destination held a prefix (≈1070 / 2110 / 3150 bytes) followed by the untouched
canary, guard bytes intact, no change during a further 2 × 272 µs; the same channel then copied
the full block exactly. A separate step wrote only `CH_ABORT`/`CH_ABORT_WE` for channel 2 to
0x01C mid-transfer: the enable bit cleared within 1 µs with `CH_INTSTATUS = 0x8000_0000`
(`CH_ABORTED`), and the channel copied the full block afterwards. This confirms the 8-channel
abort layout on HW (two cold boots with the probe).

With ARLEN/AWLEN = `beats - 1` (8 / 4 beats instead of Linux's 9 / 5) SRAM→SRAM 1 KiB blocks
take 16.9 µs (ch0) and 18.3 µs (ch2) instead of 15.0 / 14.6 µs; SRAM→DDR numbers are unchanged
(16 B 5.0 µs, 512 B 7.0 µs, 1 KiB blocks 86 MB/s, 64 KiB blocks 210 MB/s). The slowdown is
attributed to the shorter bursts (HYP: not isolated from the other changes of that commit).

## DMA ring producer (RP1 → Linux)

`rp1_hal::shm::RingDma` moves batches of a `reject` RP1 → Linux ring into Linux DDR by DMA.
The wire format is unchanged: a ring can mix DMA and CPU batches, and the Linux reader is the
same (STATIC: rp1-proto tests).

```rust
#[unsafe(link_section = ".shared")]
static STAGE: Stage = Stage(UnsafeCell::new([0; 4096])); // 8-byte aligned, shared SRAM
let mut ring_dma = RingDma::new(unsafe { &mut *STAGE.0.get() })?;
let (n, pulsed) = host_irq.push_slice_dma_notify(&mut ring_dma, &mut tx, &mut dma.ch1, &batch);
```

- `RingDma::push_slice(&mut ring, &mut channel, values)`: batches of at least the threshold
  (`DEFAULT_THRESHOLD` = 256 staged bytes, `set_threshold`) go by DMA, smaller ones by the
  CPU (`RingTx::push_slice`). The copier is lent per call (`C: SlotCopy`, normally
  `&mut Channel<N>` of the declared controller, `dma = {}`). Wrappers can implement the
  same blocking contract, so lab timing needs no HAL diagnostics. One channel can serve
  several rings; nothing else may use it during the call. `HostIrq::push_slice_dma_notify` adds `notify_ring` for `notify = "irq"`.
- DMA path (`rp1_proto::shm::RingTx::push_slice_staged` with the `SlotCopy` the channel
  implements): stage whole slot images (`seq`, length, payload, zero padding) in the shared-SRAM
  stage, copy them in one block, or two when the batch wraps around the end of the ring
  (blocking, `dma::RING_TIMEOUT_US` per block, a failed block is stopped before return), then the
  barrier, then `head`. The batch is bounded by `RingTx::writable()` and stage capacity;
  the threshold uses that actual writable count, including when only one slot is free.
  The CPU path takes the full input slice: reject rings stop at the writable capacity,
  while overwrite rings process every input, even beyond the ring depth (STATIC: host tests).
  Stage capacity limits only DMA; the CPU also handles a stage smaller than one slot, even
  at threshold 0. `usize::MAX` always selects the CPU path.
- `unsafe trait SlotCopy`: on success ALL copied bytes must reach the ordering point shared
  with subsequent CPU stores to the same mapping; no such store may overtake them. Source
  access must stop on every return. The Channel implementation relies on the HYP chain above.
  Value comparison is not an ordering guarantee, including repeated tails and zero padding;
  the former last-word guard and its errors/counters/unguarded API have been removed.
- `Staged::Invalid` publishes nothing. `Staged::Copy` also fences that writer permanently:
  `push` returns `NotReady`, `push_slice` returns zero, staged pushes return `Staged::Fenced`,
  and `writable()` returns zero. Pending remote writes could otherwise corrupt a retry.
  Re-attaching requires an external drain/reset; merely dropping the handle is insufficient.
  Overwrite rings use the CPU path (no copy is attempted).

### Direct generation (STATIC)

```rust
let (n, pulsed) = host_irq.push_with_dma_notify(
    &mut ring_dma, &mut tx, &mut dma.ch1, count,
    |offset, payload| fill_message(first + offset as u32, payload),
);
// Retry from first + n, using the remaining count.
```

`RingDma::push_with` uses the same threshold decision as `push_slice`, based on the actual
writable count and stage capacity. Small batches use `RingTx::push_with`; DMA batches use
`RingTx::push_with_staged`. Overwrite always uses CPU and accepts the full requested count.
Both notification helpers accept any `C: SlotCopy` and return `Staged<C::Error>`.
They call `notify_ring` on full rings and errors as well.

The callback follows the contract in `rp1-proto`'s `RingTx::push_with`. Payloads aligned
up to 8 bytes are generated directly in ordinary stage memory; higher alignments use one
aligned temporary. All callbacks finish before the shared commit routine copies the one
or two physical blocks, fences on error, then barriers and publishes head. The Channel
implementation's **HYP** ordering qualification above still applies.

Measured (HW, 2026-11-11): generate-DMA achieved 1.07–1.53× slice-DMA; generate-CPU was
slower at 23/26 conditions, worst 0.82×. Slice paths were at least baseline at every batch.
See the [measurement plan](../lab/README.md#ring-dma-speed-measurement).

### Historical HW throughput (before review fixes, 2026-11-05, lab `--no-default-features --features ring-dma`, 2 cold boots)

Lab command 17 (`shm-linux ring-dma`): 10^6 messages per run, every one checked by Linux;
batches of 32 (16 B, 64 B) or 8 (256 B) messages, channel 1, the stage in shared SRAM.
`ring` = payload rate of the RP1 time inside the historical ring API (CPU push, or DMA
staging, copy, the former guard and `head`); `end-to-end` = Linux-measured payload rate including the lab's message
generation on RP1 (from shared-SRAM code; not part of the ring). Polling consumer; the irq
consumer gives the same RP1 rates (within 1 %) with 13–29 % consumer CPU. The two boots agree
within 0.1 %.

| Message | CPU ring | DMA ring | CPU end-to-end | DMA end-to-end |
|---|---:|---:|---:|---:|
| 16 B (24 B slots, 768 B per batch) | 14.6 MB/s | 22.1 MB/s | 8.9 MB/s | 11.3 MB/s |
| 64 B (72 B slots, 2304 B) | 17.7 MB/s | 40.0 MB/s | 12.7 MB/s | 21.1 MB/s |
| 256 B (264 B slots, 2112 B) | 18.4 MB/s | 43.3 MB/s | 14.4 MB/s | 26.2 MB/s |

Historical counters reported missing 0, corrupt 0, stale 0, guard retries 0, guard failures 0.
The old batch reader could hide a transient stale slot after a valid prefix; its stale=0
therefore does not establish absence of premature publication. These rates include the removed guard. The `irq`
runs were notified (one pulse and one wake per batch, no 10 ms re-check found data), a paced
DMA run (100 µs between batches) slept and was woken by interrupts. With the default
threshold a single 16 B message stayed on the CPU (10^6 CPU batches) and a 32-message batch
used DMA.

Historical crossover with the former guard (RP1 µs per batch inside the ring API, CPU / DMA): 16 B messages 10.1 / 12.0 at
8 slots (192 B), 14.3 / 14.0 at 12 (288 B), 35.2 / 23.2 at 32; 64 B messages 9.0 / 11.2 at
2 (144 B), 16.1 / 13.7 at 4 (288 B); 256 B messages 15.5 / 14.1 at 1 (264 B). Hence the
256 B default was selected from these historical measurements. The historical cost model was
about 9 µs fixed (block setup/completion, the former guard and `head`) plus about 0.45 µs per
24 B slot. The current path stages and copies the slots, waits for completion, then performs
the barrier and publishes `head`; it has no guard read. Its fixed cost and crossover have
not been remeasured, so the retained 256 B default is provisional. Slots are 8-byte multiples,
so the DMAC moves 64-bit beats rather than the bulk tests' 128-bit beats (STATIC); attributing
the rate difference mainly to beat width is HYP.

Staging uses volatile word stores: a plain copy loop became a `memcpy` call, which on RP1 runs
from shared SRAM and made 256 B slots cost more by DMA than by CPU (HW, an earlier build:
17.0 MB/s DMA vs 18.4 MB/s CPU).

### Ordering experiment (DMA completion vs a later CPU `head` store)

The previous guard-off experiment reported zero violations, but `pop_into` could hide an
error after a valid prefix. Those counts are withdrawn as ordering evidence (STATIC review finding).

The revised lab command `shm-linux ring-dma-order [count]` checks the newest published slot
first (seq, length, every payload word), then consumes using `pop_into_report` so a transient
error after a prefix is retained. It counts each logical offending slot once across newest,
prefix and repeated retry observations. Deduplication assumes ring depths <=256 (events and
wide64: 256, wide256: 128). A slot is classified by its first observed violation; later stale
or corrupt observations of the same index are suppressed. Only violation presence is meaningful,
not the stale/corrupt breakdown. The 16 B `events` payload ends in constant
`0x5250_3148`; the 64/256 B payloads have word 0 = `i`, intermediate words
`i ^ k * 0x9e37_79b9`, final word = `!i`. Seq alone cannot detect delayed interior payload.

Mode 1 is the production Channel path. Mode 3 deliberately publishes head BEFORE DMA enable
and delays 100 µs, with bit30=1, as a positive control; it MUST produce a violation for every
size or the experiment is insensitive. Mode 4 uses the same lab register-copy path with
bit30=0, AW_CACHE=0 and normal publication after DONE, as a comparison. These deliberate
contract violations are confined to `lab/`. Every run requires a response and explicit end
marker; timeout/truncated capture fails. Zero production errors support only the measured
conditions, not a universal proof of the endpoint ordering chain.

### Review-fix HW attempt (2026-11-05, two cold boots): FAIL

HW: production CTL_L readback `0x40046400` (bit30=1, AW_CACHE=0), comparison
`0x00046400` (bit30=0, AW_CACHE=0); endpoint and root port DevCtl2 both IDOReq-/IDOCompl-.
All 21 completed early-head controls detected violations (32,000 slots per case, all sizes).
Bit30=0 comparisons observed no violations in 672,000 slots; necessity of bit30 is not proved.

Production streams used batches 31/31/7 for 16/64/256 B, forcing physical ring wraps including
two-block copies. Completed phases cover 90,000,000 items, zero missing/corrupt/stale. The
last 256 B stream of boot2 reset unexpectedly, followed by a new BOOTSYS in UART, without
the experiment's end markers. That phase is FAIL and its partial data is excluded; the
requested 100,000,000-item complete campaign is therefore NOT VERIFIED. Reset cause remains
unknown. Neither these finite observations nor the positive control prove the internal
AXI-to-PCIe ordering chain. Error fencing remains STATIC-tested, not HW fault-injected.

HW rerun (2026-11-05/06): FAIL as well. A spontaneous restart had power-on-reset telemetry;
another phase was interrupted without end markers. The requested two-boot complete campaign
remains NOT VERIFIED. Separate 6M-item wide256 DMA and CPU diagnostics completed, but do not
replace the failed campaign or establish a reset cause. The ordering chain remains HYP.

## Mem-to-peripheral plan (not implemented)

Peripherals with a DMA handshake (UART, SPI, I2C0..5, PWM FIFO, I2S, PIO, ADC FIFO, DMA_TICK)
connect to DMAC handshake interfaces (DOC: datasheet ch. 9; the handshake table is not in the
text we have, the Linux `rp1.h` `RP1_DMA_*` numbers are). Steps: (1) take the DREQ numbers from
the Linux DT binding header with evidence; (2) `CFG2.DST_PER`/`SRC_PER` = DREQ, `HS_SEL` = 0
(hardware), `TT_FC` = 1 / 2, no address increment on the peripheral side, MSIZE = the
peripheral FIFO threshold; (3) per-peripheral glue (SPI `DMACR`/`DMATDLR`, UART `UARTDMACR`);
(4) a lab loopback on SPI0 (TX DMA, RX by CPU) before any RX DMA. The ownership side is ready:
the consuming peripheral and the DMAC must be owned by the same image.
