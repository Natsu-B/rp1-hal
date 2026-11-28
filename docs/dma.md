# RP1 DMA driver

`rp1_hal::dma` drives the RP1 DMA controller, a Synopsys DW AXI DMAC (1.02a-lp02, eight
channels, one 128-bit AXI master). The v0.2 API does single-block memory-to-memory
transfers. **HW** denotes measurement, **STATIC** code or tests, **DOC** the datasheet
or Linux source, and **HYP** a hypothesis.

## Ownership

```toml
[package.metadata.rp1.proc1]
dma = { channels = [0, 2], pool = { slots = 2, bytes = 1024 } }
```

- STATIC, 2026-11-27. The controller is one block (one reset bit, one clock, one interrupt),
  so one core declares it whole, like `adc` (two owners: E3, bad keys or channels: E0).
  Without the declaration Linux keeps `rp1_dma` with SCMI clocks.
- `channels` selects the generated channel fields. Omission (`dma = {}`) gives all eight channels.
  An empty list reserves the controller. The NOTE still claims block 65.
- **STATIC · 2026-11-27.** The optional `pool` generates `p.dma_pool` on the DMA-owning
  core. `bytes` is the capacity per slot and must be a nonzero multiple of 16.
  The linker places the aligned reservation in shared SRAM and checks the image budget.
- **STATIC · 2026-11-27.** The complete generated set can `try_stop_all` or
  `stop_all_until(&mut wait, end)`. Release uses `try_free` or `free_until` and returns
  the original declaration token. A partially moved channel set cannot release the
  controller. Every issued channel must prove quiescence and retire its state. The driver
  also checks all hardware slots, including reserved channels, before disabling the block.
  `FreeError::driver` returns the owner if recovery has to be retried.
- A forgotten transfer retains its pool claims for the epoch. Recovery may return the
  controller after the channel drains and its terminal status is acknowledged. Controller
  reuse grants no access to the forgotten leases.
- With the declaration the generated overlay disables `rp1_dma`, the image's `linux_resets`
  leave RESETS CTRL0 bit 3 alone, and the SCMI server holds clk_dma (Linux cannot change it).
  STATIC: `rp1-build` artifacts test; HW (2026-11-05): `rp1_dma` reads `status=disabled` and
  no DMA interrupt appears for it in Linux.
- Linux consumers: an enabled node whose `dmas` names `rp1_dma` would defer forever (of-dma
  returns -EPROBE_DEFER for an unregistered provider, DOC). The loader refuses an enabled
  consumer unless its absolute DT path appears in `linux.dma_disable` and
  `on_conflict = "disable"`. Protected nodes always cause refusal. Specifiers use the
  provider's `#dma-cells` (a DREQ number equal to a phandle is not a reference). STATIC:
  `rp1-dt` resolve test. HW (2026-11-05): with the standard CM5 IO DTB + `uart1-pi5` no enabled
  node uses `rp1_dma`; the loader disabled only `gpiomem` (as before). An overlay enabling an
  RP1 SPI/I2S node with `dmas = <&rp1_dma ..>` needs an explicit profile decision. `rp1_pio` is
  different: it is a protected node (in the board's Linux-required list), so a `dmas` conflict
  on an enabled `rp1_pio` makes the resolver **refuse** the plan even with
  `on_conflict = "disable"` (STATIC: `rp1-dt` resolve, protected nodes are never disabled).

## API

- `p.dma.init(&clocks)` starts clk_dma at 100 MHz (DOC: datasheet ch. 9), releases reset,
  soft-resets the DMAC and leaves all channels disabled with interrupts masked. The returned
  channel set has one field per selected channel (`ch0` and `ch2` above). On failure,
  `InitError::resources` returns the declaration's controller token for a retry.
- **STATIC · 2026-11-27.** Safe copies borrow two equal-length leases from a fixed
  shared-SRAM pool. A lease selects a range of one slot. Safe admission excludes ISRAM,
  DSRAM, peripherals, stack and the outbound window.
- `try_start(&mut src, &mut dst)` borrows the channel and leases exclusively. An
  admission error preserves both leases. `try_complete` observes once. `try_reclaim`
  consumes the handle and returns the transfer outcome after quiescence and ACK. If
  the proof is pending, it returns the handle for retry. Inspect the outcome before
  using the destination. A cancelled transfer can write a prefix or the whole block.
- `copy_polling(&mut src, &mut dst, duration)` and `Transfer::wait_polling(end)` explicitly
  poll in thread mode while IRQ40 delivery awaits verification. `wait_polling` uses
  an absolute `Deadline`. Completion already visible at that deadline wins.
- If a transfer is forgotten or dropped without a stop proof, both leases yield empty
  slices and reject new ranges and DMA admission. Dropping those leases preserves the
  claims. Use `is_quarantined` to inspect a lease after its borrow ends.
- `unsafe Channel::start_raw(src, dst, len)` accepts caller-managed bus addresses.
  Its caller owns the entire lifetime of both ranges, including a lost handle. A graceful
  stop proves local engine quiescence. Remote write drain, outbound window stability and
  publish ordering need independent proof. Retire the remote mapping for the epoch if
  that proof is unavailable.
- `Bus(CH_INTSTATUS)` reports a bus fault. `Timeout` reports an elapsed transfer deadline.
  If stop remains uncertain, polling reports `Recovering`; an external forced abort
  reports `Aborted`. An explicit transfer retains its first fault or deadline error
  for `try_reclaim`. `try_recover` requests graceful disable if forgotten work is
  active and checks the channel before reuse.
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
- Deferred capabilities include linked lists and IRQ40 delivery.
- Ring producer: `rp1_link::ring_dma::RingDma`, feature `rp1-link/unstable-ring-dma`
  (below, "DMA ring producer").

### v0.2 migration (STATIC · 2026-11-27)

| Before | After |
|---|---|
| `ch.copy(&src, &mut dst, timeout)` | `ch.copy_polling(&mut src, &mut dst, duration)` with pool leases |
| Safe SRAM→DDR copies | Use `unsafe start_raw` with an independently managed remote mapping and drain proof |
| `unsafe { ch.start(src, dst, len) }` | `try_start(&mut src, &mut dst)` for leases or `unsafe { ch.start_raw(src, dst, len) }` for caller-managed ranges |
| `Transfer::poll`, `Transfer::wait` | `try_complete`, `wait_polling(end)` |
| `transfer.cancel()`, `dma::Stopped` and blocking Drop | `try_cancel` or timer-backed `cancel_until`. `try_reclaim` ends the borrow after a proven stop. Drop makes a bounded stop attempt. |
| Exhaustive matches on `dma::Error` | Include a fallback arm for the `#[non_exhaustive]` error type |
| `Dma::free()` (removed in PR 1a) | `channels.try_free()` or `channels.free_until(&mut wait, end)` after assembling the declared set |
| A channel implements `SlotCopy`, `dma::RING_TIMEOUT_US` | `unsafe { ch.experimental_ring_dma(&scratch) }` selects the polling adapter with private scratch and a fixed internal timeout |

With the declaration above and initialized clocks, a local SRAM copy uses the generated
pool and retains the channel set for further operations.

```rust
use rp1_hal::timer::Duration;

let mut dma = p.dma.init(&clocks)?;
let pool = p.dma_pool;
let [mut src, mut dst] = pool.leases(1024)?;
src.as_mut_slice().fill(0x5a);
dma.ch2.copy_polling(&mut src, &mut dst, Duration::millis(10))?;
assert_eq!(src.as_slice(), dst.as_slice());
```

### PIO FIFOs (STATIC)

Declare `dma` with a pool and `pio` on the same core. The generated `init_pio`
method provides channels for SRAM copies and PIO transfers. Use `into_dma()` on a
TX or RX FIFO to reserve it for transfers. The selected SRAM lease range must contain a whole number
of 32-bit words and start at a 4-byte boundary.

```rust
let mut dma = p.dma.init_pio(&clocks)?;
let mut tx = tx.into_dma();
let mut transfer = dma.ch0.try_start_to_pio(&mut src, &mut tx)?;
transfer.wait_polling(end)?;
// Dropping a completed transfer returns both endpoints.
drop(transfer);
```

Use `try_start_from_pio(&mut rx, &mut dst)` with a reserved RX FIFO. Like rp-hal's
single-buffer DMA, the FIFO type determines direction and pacing. RP1 uses one
fixed APB FIFO address, an incrementing SRAM address and one 32-bit beat per
handshake. Request IDs are 56–63 with consecutive TX/RX pairs for SM0–SM3.
The driver opens only the selected SM's DREQ after enabling the channel. TX DMA
completion means the words reached the FIFO. Check the SM or output separately
for waveform completion. [RP1 DMA bindings](https://github.com/raspberrypi/linux/blob/rpi-6.12.y/include/dt-bindings/mfd/rp1.h) ·
[RP1 PIO DMA thresholds](https://github.com/raspberrypi/linux/commit/f31ed0ba33) ·
[rp-hal single-buffer DMA](https://github.com/rp-rs/rp-hal/blob/main/rp2040-hal/src/dma/single_buffer.rs)

The borrowed FIFO excludes CPU access and FIFO IRQ waits. Once the transfer has
retired, `try_cpu` borrows the FIFO for CPU use and `try_into_inner` returns it
for SM reconfiguration or release. The opposite direction stays available
throughout a transfer.

Cancellation masks DREQ before requesting graceful disable. Use `try_cancel`
for one check or `cancel_until` for a timer-backed check bounded to 1 ms.
`try_reclaim` returns the transfer if the stop proof is pending. A dropped or
forgotten transfer with an outstanding claim quarantines that lease and FIFO for
the firmware epoch. `try_recover` can retire the channel independently, after
the request is masked and the channel stop is proven. Inspect the outcome before
using received data. A cancelled RX transfer may have written a prefix.

IRQ40 delivery has its own hardware gate. Completion uses `try_complete` or
explicit `wait_polling` while that gate is pending.

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

**DOC · 2026-11-27.** The [Synopsys DW_axi_dmac Databook 1.02a, §2.15.4 and
§2.16.3](https://picture.iczhiku.com/resource/eetop/wyKEEOUFJDggQnvB.pdf#page=91) describes
graceful disable and error handling. The channel-enable register descriptions explicitly
allow an abnormal abort to leave AXI operations outstanding.

**HW · 2026-11-27 · v0.2 candidate `8eba065`.** Boot P ran 100 mid-transfer,
disable-only SRAM cancellations across all eight channels. Every attempt stopped on
the first `try_cancel` (maximum 6 µs) and left a proper prefix, intact guards and no
later destination changes. The stop logic required EN clear plus `CH_DISABLED` or
`DMA_TFR_DONE` and W1C readback. Each attempt interrupted the block before it wrote the
whole payload. Terminal-bit identity and full decision/Drop timing are instrumented
in the refreshed candidate.

**STATIC · 2026-11-27.** `try_cancel` masks the channel signal, writes only the channel's
enable write-enable bit to request disable and checks for a terminal state. `cancel_until`
issues that request before checking thread context or deadline. In thread mode the
recheck deadline is the earlier of the caller's deadline and 999 µs from entry. With
IRQ40 masked, a stop missed by the initial checks can be reported by the timer alarm
nearly 1 ms after entry. Drop performs a fixed number of MMIO accesses.

The `Transfer` handle itself carries the quarantined claims while it can retry. If
the handle is dropped without proof or forgotten, the claims remain reserved for
the firmware epoch. Reuse requires EN clear, terminal status and a confirmed W1C ACK.
`CH_ABORTED` leaves quiescence unproven. The 1 ms real-time bound includes
preemption and MMIO stalls and is a hardware acceptance criterion.

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

Safe lease storage uses a `#[unsafe(link_section = ".shared")]` reservation issued once
by the owning core. Outbound shm uses the experimental raw path. The PCIe AXI APB peripherals at
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
- Even after graceful disable establishes local engine quiescence, accepted outbound writes
  can remain in flight. A CPU barrier orders CPU accesses and cannot drain the DMA's PCIe writes.

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

### Previous driver cancellation proof and burst-length change

**HW · 2026-11-05 · three cold boots.** The previous driver used suspend followed
by disable. Lab step in command 16 (channel 2, byte-wide 4094 B SRAM→DDR block through the ATU window,
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

Unstable: behind the cargo feature `rp1-link/unstable-ring-dma` (STATIC). It enables
the rp1-hal channel implementation and the staged ring methods in rp1-proto. The ordering chain below is HYP
and the hardware campaign after the review fixes FAILED (sections below). Default builds
contain none of it; Linux needs nothing, the wire format is the CPU ring's.

`rp1_link::ring_dma::RingDma` moves batches of a `reject` RP1 → Linux ring endpoint
(`rp1_link::RingTx`) into Linux DDR by DMA.
The wire format is unchanged: a ring can mix DMA and CPU batches, and the Linux reader is the
same (STATIC: rp1-proto tests).

```rust
#[unsafe(link_section = ".shared")]
static STAGE: Stage = Stage(UnsafeCell::new([0; 4096])); // 8-byte aligned, shared SRAM
let mut ring_dma = RingDma::new(unsafe { &mut *STAGE.0.get() })?;
// scratch is a separate, 16-byte-aligned Pool<1> with room for the batch plus 15 bytes.
let mut copy = unsafe { dma.ch1.experimental_ring_dma(&scratch) };
let n = tx.send_slice_dma(&mut ring_dma, &mut copy, &batch)?;
```

- `RingTx::send_slice_dma(&mut ring_dma, &mut copy, values)` returns
  `Result<usize, Staged<C::Error>>`: batches of at least the threshold
  (`DEFAULT_THRESHOLD` = 256 staged bytes, `RingDma::set_threshold`) go by DMA, smaller ones by
  the CPU (`RingTx::push_slice`). Then it wakes the reader as `send_slice` does (on a
  `notify = "irq"` endpoint; also after an error or a full ring, since the reader may have
  armed meanwhile). `push_slice_dma` is the same without the wake (follow with `flush`).
  The copier is lent per call (`C: SlotCopy`). Wrappers can implement the same contract, so lab timing
  needs no HAL diagnostics. One channel can serve several rings; nothing else may use it
  during the call. A lost shm window (PERSTn) gives `Staged::Reset` before any access.
- DMA path (`rp1_proto::shm::RingTx::push_slice_staged` with the experimental `SlotCopy`
  adapter): stage whole slot images (`seq`, length, payload, zero padding) in the shared-SRAM
  stage, copy them in one block, or two when the batch wraps around the end of the ring
  (explicit polling with a 10 ms deadline per copy). The experimental adapter copies the CPU
  stage into its own SRAM scratch lease before enabling DMA. A failed stop quarantines the
  scratch so the CPU stage is reusable on every return. Success is followed by the barrier
  and publication of `head`. The batch is bounded by `RingTx::writable()` and stage capacity;
  the threshold uses that actual writable count, including when only one slot is free.
  The CPU path takes the full input slice: reject rings stop at the writable capacity,
  while overwrite rings process every input, even beyond the ring depth (STATIC: host tests).
  Stage capacity limits only DMA; the CPU also handles a stage smaller than one slot, even
  at threshold 0. `usize::MAX` always selects the CPU path.
- `unsafe trait SlotCopy`: on success ALL copied bytes must reach the ordering point shared
  with subsequent CPU stores to the same mapping; no such store may overtake them. Source
  access must stop on every return. The experimental adapter relies on the HYP chain above.
  Value comparison is not an ordering guarantee, including repeated tails and zero padding;
  the former last-word guard and its errors/counters/unguarded API have been removed.
- `Staged::Invalid` publishes nothing. `Staged::Copy` also fences that writer permanently:
  `push` returns `NotReady`, `push_slice` returns zero, staged pushes return `Staged::Fenced`,
  and `writable()` returns zero. Pending remote writes could otherwise corrupt a retry.
  Re-attaching requires an external drain/reset; merely dropping the handle is insufficient.
  Overwrite rings use the CPU path (no copy is attempted).

### Direct generation (STATIC)

```rust
// Requires a pinned outbound mapping and the ordering proof described above.
let mut copy = unsafe { dma.ch1.experimental_ring_dma(&scratch) };
let n = tx.send_with_dma(&mut ring_dma, &mut copy, count, |offset, payload| {
    fill_message(first + offset as u32, payload)
})?;
// Retry from first + n, using the remaining count.
```

`RingTx::send_with_dma` (and `push_with_dma`, without the wake) uses the same threshold
decision as `send_slice_dma`, based on the actual
writable count and stage capacity. Small batches use `RingTx::push_with`; DMA batches use
`RingTx::push_with_staged`. Overwrite always uses CPU and accepts the full requested count.
Both return `Result<usize, Staged<C::Error>>`; `send_with_dma` wakes under the same
conditions as `send_slice_dma`.

The callback follows the contract in `rp1-proto`'s `RingTx::push_with`. Payloads aligned
up to 8 bytes are generated directly in ordinary stage memory; higher alignments use one
aligned temporary. All callbacks finish before the shared commit routine copies the one
or two physical blocks, fences on error, then barriers and publishes head. The adapter's **HYP** ordering qualification above still applies.

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

Mode 1 is the experimental ring adapter path. Mode 3 deliberately publishes head BEFORE DMA enable
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

## Other peripheral handshakes (HYP)

Peripherals with a DMA handshake (UART, SPI, I2C0..5, PWM FIFO, I2S, ADC FIFO, DMA_TICK)
connect to DMAC handshake interfaces (DOC: datasheet ch. 9; the handshake table is not in the
text we have, the Linux `rp1.h` `RP1_DMA_*` numbers are). Steps: (1) take the DREQ numbers from
the Linux DT binding header with evidence; (2) `CFG2.DST_PER`/`SRC_PER` = DREQ, `HS_SEL` = 0
(hardware), `TT_FC` = 1 / 2, no address increment on the peripheral side, MSIZE = the
peripheral FIFO threshold; (3) per-peripheral glue (SPI `DMACR`/`DMATDLR`, UART `UARTDMACR`);
(4) a lab loopback on SPI0 (TX DMA, RX by CPU) before any RX DMA. The ownership side is ready:
the consuming peripheral and the DMAC must be owned by the same image.
