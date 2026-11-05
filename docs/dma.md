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
- Completion = `DMA_TFR_DONE` and the channel's `CHEN` bit clear, i.e. after the last AXI write
  response, followed by the same barrier before returning (also after a cancellation), so
  later ordinary CPU reads/writes of either buffer cannot be moved before it. For SRAM the data
  is then visible to both cores and to Linux (BAR2).
- Linux DDR: the write response for a PCIe posted write comes from the endpoint when it accepts
  the write; later CPU posted writes through the same endpoint (e.g. a ring head) are ordered
  behind it by PCIe posted-write ordering (DOC/HYP: not separately tested; the lab check below
  reads the data only after RP1 reported completion).

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

### RP1 → Linux ring ceiling

The CPU-written ring is bound by posted writes (0.54 µs/word; batched ring 4.58 MB/s, HW
2026-10-29). Staging a 32-message batch (512 B) in shared SRAM and copying it with one DMA block
takes 7.1 µs here, i.e. about 72 MB/s of payload before the head update and staging cost, so
DMA lifts the ceiling by roughly an order of magnitude for batched rings (HYP until a ring
producer uses it and is measured end to end). It does not help single 16 B messages (5.0 µs
per DMA block vs about 2.7 µs for five posted words).

## Mem-to-peripheral plan (not implemented)

Peripherals with a DMA handshake (UART, SPI, I2C0..5, PWM FIFO, I2S, PIO, ADC FIFO, DMA_TICK)
connect to DMAC handshake interfaces (DOC: datasheet ch. 9; the handshake table is not in the
text we have, the Linux `rp1.h` `RP1_DMA_*` numbers are). Steps: (1) take the DREQ numbers from
the Linux DT binding header with evidence; (2) `CFG2.DST_PER`/`SRC_PER` = DREQ, `HS_SEL` = 0
(hardware), `TT_FC` = 1 / 2, no address increment on the peripheral side, MSIZE = the
peripheral FIFO threshold; (3) per-peripheral glue (SPI `DMACR`/`DMATDLR`, UART `UARTDMACR`);
(4) a lab loopback on SPI0 (TX DMA, RX by CPU) before any RX DMA. The ownership side is ready:
the consuming peripheral and the DMAC must be owned by the same image.
