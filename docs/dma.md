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
  RP1 SPI/I2S/PIO node (all carry `dmas = <&rp1_dma ..>`) would now lose that node.

## API

- `dma::Token::init(&Clocks)` (once): clk_dma 100 MHz (DOC: datasheet ch. 9), release the reset,
  `DMAC_RESET` soft reset, every channel disabled with interrupts masked, `DMAC_CFG.DMAC_EN`.
  Returns `Dma { ch0..ch7 }`.
- `Channel<N>::copy(&[u8], &mut [u8], timeout_us)`: safe, blocking. Both slices must be
  DMAC-visible (shared SRAM, APB/AHB peripherals, outbound window); splits into blocks; a
  timed-out block is aborted (`DMAC_CHABORTREG`) before return.
- `unsafe Channel<N>::start(src: BusAddr, dst: BusAddr, len) -> Transfer`: one block,
  `Transfer::poll()` / `wait(timeout_us)`; dropping an unfinished transfer aborts it. The
  caller guarantees the memory (a leaked `Transfer` keeps the DMA running).
- Errors: `Bus(CH_INTSTATUS)` on decode/slave errors (the channel is aborted), `Timeout`,
  `Busy`, `Invalid`, `NotDmaVisible`, `Precondition` (clk_dma not 100 MHz).
- Programming (DOC, Linux `dw-axi-dmac` with the RP1 DT properties): 8-channel register map
  (`CHEN` write enables at bit 8), `CFG2` layout (`snps,dma-targets = <64>`), contiguous
  source/destination, memory-to-memory with DMAC flow control, priority = channel, MSIZE 4,
  width = largest power of two dividing addresses and length (max 16 B), ARLEN/AWLEN from
  `snps,axi-max-burst-len = <8 8 4 4 4 4 4 4>` as Linux writes them. Single-block contiguous mode
  (no LLI) is not what Linux uses; it works on RP1 (HW below).
- Not in v1: linked lists, interrupts (the line is RP1 IRQ 40, unused), peripheral handshakes.

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
- Channel setup writes are posted in program order to the DMAC (Device memory, one AHB port);
  the channel-enable write is followed by `dsb`, which also completes earlier CPU stores to the
  source buffer.
- Completion = `DMA_TFR_DONE` and the channel's `CHEN` bit clear, i.e. after the last AXI write
  response. For SRAM the data is then visible to both cores and to Linux (BAR2).
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
