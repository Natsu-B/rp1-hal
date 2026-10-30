# SPI master

STATIC: `rp1_hal::spi` drives the RP1 DesignWare APB SSI blocks as polling SPI masters:
embedded-hal 1.0 `SpiBus<u8 | u16 | u32>` on `Spi`, and `SpiDevice` on `spi::Device`, which
adds a GPIO chip select (active low) and owns the bus exclusively.

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
  `Clocks::wait_from_proc0`. A clock tree that no longer matches the value returns
  `Precondition`.
- Reset: RESETS CTRL1 bit 10 + n, DONE1 the same bit (rp1-regdb `resets.toml`; SPI0 is
  hardware-proven there). `init` releases only that bit and waits for DONE (10 ms bound).
- Registers: `rp1_pac::dw_ssi` (offsets from Linux `spi-dw.h`), bases from the regdb.
  `init` detects the FIFO depth (TXFTLR probe) and the 32-bit frame field (CTRLR0 probe) as
  Linux does.
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
shifts with a slave selected. `Device::transaction` drives CS low, runs the operations
(`DelayNs` flushes first), flushes, and drives CS high even after an error.

`set_frequency`, `set_mode`, `set_frame_bits` and `set_loopback` (CTRLR0.SRL internal
loopback) wait for the bus to be idle and reprogram the disabled controller.

There is no interrupt, DMA, dual/quad mode, or slave support.

## Hardware proof

`lab/shm-firmware` declares SPI0 on proc1 (GPIO8 chip select). proc1 runs `spi_proof` at boot
and on Linux's request: with loopback off MISO reads all zeros (pad pull-down; the lab wiring
has only ESP32 inputs on these pins), then internal loopback at 1, 10, 25 and 50 MHz with 8-,
12- and 16-bit frames through `SpiDevice`, plus u8 words, in-place and a write/delay/read
transaction. `lab/check-shm.sh` and `lab/verify-shm.py` make it part of the regression.

HW (2026-10-30, two cold boots, clk_sys 200 MHz, at boot and on request while Linux runs):
`init` found FIFO depth 64 and the 32-bit frame field; with loopback off MISO read all zeros;
every loopback pattern matched at all four rates and three frame sizes, and the u8, in-place
and transaction checks passed. Measured throughput of 1024 frames in 32-frame `SpiDevice`
transfers (one boot; the other boot is within 2 %):

| SCLK (BAUDR) | 8-bit | 12-bit | 16-bit |
|---|---|---|---|
| 1 MHz (/200) | 0.95 Mbit/s (95 %) | 0.97 Mbit/s (96 %) | 0.98 Mbit/s (97 %) |
| 10 MHz (/20) | 6.8 Mbit/s (68 %) | 7.7 Mbit/s (76 %) | 8.1 Mbit/s (81 %) |
| 25 MHz (/8) | 11.3 Mbit/s (45 %) | 14.1 Mbit/s (56 %) | 15.9 Mbit/s (63 %) |
| 50 MHz (/4) | 12.1 Mbit/s (24 %) | 18.1 Mbit/s (36 %) | 22.7 Mbit/s (45 %) |

At 1 MHz the wire dominates; above about 10 MHz the polling loop (one APB FIFO access per
frame each way, plus the CS and call overhead per 32 frames) is the limit, about 0.66 µs
per 8-bit frame at 50 MHz. Longer transfers or wider frames get closer to the wire rate.

The rate is measured on the 1 MHz timer around each 32-frame transfer call (CS handling
included), so it is a throughput figure, not an SCLK measurement. SCLK itself was not probed
externally: no SPI peer is wired.
