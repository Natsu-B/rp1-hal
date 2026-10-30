# SPI master

STATIC: `rp1_hal::spi` drives the RP1 DesignWare APB SSI blocks as polling SPI masters:
embedded-hal 1.0 `SpiBus<u8 | u16 | u32>` on `Spi`, and `SpiDevice` on `spi::Device`, which
adds a GPIO chip select (active low) and owns the bus exclusively, either an owned `Spi` or a
lent `&mut Spi` (`Device::new(&mut spi, cs)`; other buses are not accepted).

```toml
[package.metadata.rp1.proc0]
pins = [8, 9, 10, 11]
peripherals = ["SPI0"]
```

```rust,ignore
use embedded_hal::spi::{MODE_0, Operation, SpiDevice};
let bus = p.SPI0.init(p.pins.gpio11, p.pins.gpio10, p.pins.gpio9, &clocks,
    rp1_hal::spi::Config::new(10_000_000, MODE_0))?;
let mut dev = rp1_hal::spi::Device::new(bus, p.pins.gpio8);
dev.transaction(&mut [Operation::Write(&[0x9f]), Operation::Read(&mut id)])?;
```

`examples/dual-core` uses SPI0 on proc0 this way (compile-checked).

## Which blocks

DOC (RP1 peripherals §3.6.1, rp1.dtsi): nine DW_apb_ssi v4.02a instances. SPI4 and SPI7 are
slave-only (`spi-slave` in Linux's DT); SPI0/1/2/3/5/6/8 are masters. Only SPI0..5 are on GPIO
bank 0; SPI6 and SPI8 sit on the internal banks, which this HAL does not expose (as I2C4..6).
The HAL token accepts SPI0, 1, 2, 3 and 5. On the rpi-cm5 board `rp1-build` accepts
`SPI0`, `SPI1`, `SPI3`, `SPI5`: SPI2's only MISO route is GPIO1, which stays with Linux UART1.

| Controller | Function | SCLK | MOSI (SIO0) | MISO (SIO1) | Native CS0 / CS1 (not used) |
|---|---|---|---|---|---|
| SPI0 | 0 | 11 | 10 | 9 | 8 / 7 |
| SPI1 | 0 | 21 | 20 | 19 | 18 / 17 |
| SPI2 | 8 | 3 | 2 | 1 | 0 / 24 |
| SPI3 | 8 | 7 | 6 | 5 | 4 / 25 |
| SPI5 | 8 | 15 | 14 | 13 | 12 / 26 |

Wrong or swapped pins fail to compile (`tests/ui/spi_wrong_pins.rs`).

## Clock, reset, registers

- DOC §3.6.1 / §2.5.5: master `pclk` and `sclk` are clk_sys; Linux's nodes use
  `RP1_CLK_SYS` as `ssi_clk`. SCLK = `Clocks::sys_hz() / BAUDR`, BAUDR even, 2..=65534
  (Linux dw_spi rounds the same way: up, then to even). At 200 MHz: 100 MHz maximum,
  1 MHz = /200, 10 MHz = /20, 25 MHz = /8, 50 MHz = /4; the lowest rate is about 3.05 kHz.
  clk_sys belongs to RP1 (`clock.authority = "rp1"`): no SCMI clock is held or voted for
  SPI, and `Clocks` comes either from proc0's `Authority::init` or, on proc1, from
  `Clocks::wait_from_proc0`. A clk_sys that no longer matches the value returns
  `Precondition` before the FIFO is touched. SCLK depends on clk_sys only, so clk_uart is
  not compared. `init` and every reconfiguration read CLK_SYS CTRL/DIV/SEL and the PLL_SYS
  tuple twice (`Clocks::sys_current_strict`). Each public bus call checks once
  (`Clocks::sys_current`): CLK_SYS_SEL and DIV_INT once each, plus PLL_SYS CS (lock),
  FBDIV_INT and PRIM when SEL is PLL_SYS; one copy in ISRAM, shared with I2C.
  `transfer_in_place` checks once for the whole buffer, `Device::transaction` once before CS
  goes low (a stale clock never moves CS). A check before a transfer cannot catch a change
  made during it, and the per-call check does not re-read CTRL, PLL PWR or FBDIV_FRAC,
  which init verified and nothing in rp1-hal changes later (STATIC, host tests: a clk_uart
  change does not reject, a SEL/DIV or PLL change does; each register is read once).
- Reset: RESETS CTRL1 bit 10 + n, DONE1 the same bit (rp1-regdb `resets.toml`; SPI0 is
  hardware-proven there). `init` releases only that bit and waits for DONE (10 ms bound).
- Registers: `rp1_pac::dw_ssi` (offsets from Linux `spi-dw.h`), bases from the regdb.
  `init` detects the FIFO depths (TXFTLR and RXFTLR probes; DW_apb_ssi sizes them
  separately) and bounds the frames in flight by the shallower one, and detects the 32-bit
  frame field (CTRLR0 probe) as Linux does.
- Pins: outputs (SCLK, MOSI) get 8 mA, fast slew; MISO has input enable and the pad
  pull-down. The pad output is off while the function select changes.

## Linux

As for I2C: declaring `SPIn` removes CTRL1 bit 10 + n from the resets rp1-hal releases for
Linux, and the overlay sets `rp1_spiN` to `status = "disabled"`. The final DT check rejects a
later overlay that enables the node again (e.g. `dtparam=spi=on`) and enabled Linux pinctrl
users of the declared pins.

## Behaviour

STATIC (host model tests): `transfer` shifts `max(read, write)` frames; frames past `write`
send 0, received frames past `read` are dropped. At most `fifo_depth()` frames are in flight,
so neither FIFO can overflow. Any 100 ms without progress returns `Timeout`, an RX/TX overflow
or underflow flag returns `Overrun`; after an error the controller is disabled and enabled again
(flushing both FIFOs) with the same configuration. `flush` waits (bounded) for TX empty and not
busy. `frame_bits` is 4..=16, or 4..=32 when the block has the 32-bit field; `SpiBus<W>`
returns `Invalid` if the frame is wider than `W`. The transfer loop runs from ISRAM; all word
types share it.

The native chip selects are not routed: SER selects CS0 internally because the DW SSI only
shifts with a slave selected. `Device::transaction` checks the bus once (frame size,
clk_sys), then drives CS low, runs the operations (`DelayNs` flushes first), flushes, and
drives CS high even after an error. A failed check returns before CS moves. The unchecked
operations behind this (the hidden `spi::Bus` methods) take arguments only rp1-hal can
make, so code outside the crate cannot call them and skip the check (STATIC, trybuild and
doctest compile-fail cases, through a generic bound and on `Spi`).

`set_frequency`, `set_mode`, `set_frame_bits` and `set_loopback` (CTRLR0.SRL internal
loopback) wait for the bus to be idle and reprogram the disabled controller.

`init` checks clk_sys, the rate and the frame size (up to 32 bits; the instance's own
limit after the probe) before it releases the reset. On any error it returns `InitError`
with the token and the untouched pins, so the caller can retry (`?` converts it to `Error`).
STATIC (host model through an injected register interface): a stale clk_sys, a zero or too
low rate and 3- or 33-bit frames write no register, leave the reset asserted and route no
pin; 32-bit frames on a 16-bit instance fail after the probe, still without routing, and the
destructured `InitError`'s token and pins initialise the controller on retry.

`Config` is `#[non_exhaustive]`: build it with `Config::new(hz, mode)` and the `with_*`
setters.
`Spi` has no `Drop`: dropping it without `free` leaves the controller enabled and the pins
on the SPI function, and the token is gone.

`Config::rx_sample_dly` (default 0) is written to RX_SAMPLE_DLY: MISO is sampled that many
clk_sys cycles later. Only 0 is hardware-tested. It is written without a range check or
read-back, as Linux spi-dw-core does. The usable maximum is the DW_apb_ssi synthesis
parameter SSI_RX_DLY_SR_DEPTH, which is not known for RP1, and what the block does with a
larger value is unverified (recalled, not checked: the databook treats it as no delay).

STATIC (host tests): init, reconfiguration and transactions are checked against a model whose
fast and strict clock checks are counted apart and can disagree (a drift only the strict
check sees): init and reconfiguration use only the strict one and fail before the reset,
the pin mux or any register write; transactions use only the fast one. The real `Spi` bus
impls (owned and `&mut`) return `Invalid` for words narrower than the frame before any
register access and before CS moves.
`Device::transaction` checks clk_sys once, then drives CS low, runs the
operations, flushes, then drives CS high; with a stale clk_sys it returns `Precondition` with
no CS edge; a `DelayNs` flushes before it waits; after a failed operation it skips the
rest but still flushes and releases CS, and returns the operation's error, not the flush's.
The model keeps BUSY after the last frame is received, so `flush` must wait it out. The
delay itself waits at least the requested time (the 1 MHz timer is read for `us + 1` steps,
since the first step can come right after the start).

There is no interrupt, DMA, dual/quad mode, or slave support.

## Hardware proof

`lab/shm-firmware` declares SPI0 on proc1 (GPIO8 chip select). proc1 runs `spi_proof` at boot
and on Linux's request: with loopback off MISO reads all zeros (pad pull-down; the lab wiring
has only ESP32 inputs on these pins), then internal loopback at 1, 10, 25 and 50 MHz with 8-,
12- and 16-bit frames through `SpiDevice`, plus u8 words, in-place and a write/delay/read
transaction. `lab/check-shm.sh` and `lab/verify-shm.py` make it part of the regression; both
also fail unless Linux's `spi@50000` and `adc@c8000` nodes exist with `status = "disabled"`.

HW (2026-10-30, two cold boots, clk_sys 200 MHz, at boot and on request while Linux runs):
`init` found FIFO depth 64 (the shallower of TX and RX since review 6) and the 32-bit frame
field; with loopback off MISO read all zeros; every loopback pattern matched at all four rates
and three frame sizes, and the u8, in-place and transaction checks passed. The same proof
passed again on two more cold boots after the review 6 changes, on two more after the
review 7 changes, and on two more after the review 8 changes (50 MHz 8-bit: 600 µs on both,
the same as review 7). Measured throughput of 1024 frames in 32-frame `SpiDevice` transfers after
review 7 (one boot; the other is within 0.5 %), with the review 6 figure in brackets:

| SCLK (BAUDR) | 8-bit | 12-bit | 16-bit |
|---|---|---|---|
| 1 MHz (/200) | 0.96 (0.93) Mbit/s, 96 % | 0.97 (0.95) Mbit/s, 96 % | 0.98 (0.96) Mbit/s, 98 % |
| 10 MHz (/20) | 7.3 (5.8) Mbit/s, 73 % | 8.1 (6.7) Mbit/s, 80 % | 8.4 (7.3) Mbit/s, 84 % |
| 25 MHz (/8) | 12.9 (8.7) Mbit/s, 51 % | 15.7 (11.3) Mbit/s, 62 % | 17.2 (13.1) Mbit/s, 68 % |
| 50 MHz (/4) | 13.7 (9.1) Mbit/s, 27 % | 20.5 (13.7) Mbit/s, 40 % | 26.0 (17.5) Mbit/s, 51 % |

At 1 MHz the wire dominates; above about 10 MHz the polling loop (one APB FIFO access per
frame each way) and the per-call overhead are the limit. HW, the 32 calls of 32 frames at
50 MHz 8-bit: 676 and 679 µs (12.1 Mbit/s) before review 6, which had no clock check on bus
calls; 896 and 928 µs (9.1 Mbit/s) with review 6, which compared clk_sys and clk_uart
(22 register reads, from shared SRAM) on every bus call; 600 µs on both boots (13.7 Mbit/s)
with review 7, which checks clk_sys only, once per transaction, from ISRAM (5 reads). The
review 7 time is lower than the time without any check, so the cheaper check does not explain
the whole gain: review 6 and 7 also changed the transfer path (for example `Device` now
admits once and calls the exchange directly instead of going through the `SpiBus` methods).
How the gain splits between those changes was not measured. Longer transfers or wider frames
get closer to the wire rate.

The rate is measured on the 1 MHz timer around each 32-frame transfer call (CS handling
included), so it is a throughput figure, not an SCLK measurement. SCLK itself was not probed
externally: no SPI peer is wired.

Only the internal loopback is proven, and at 25 and 50 MHz only that: the data never
crossed the pads, the wiring or a device, and RX_SAMPLE_DLY was 0. An external device may
need `Config::rx_sample_dly` or a lower SCLK; at 25 MHz and above it is untested (HYP: the
MISO round trip through pads and wiring takes a large part of a 20–40 ns bit).
