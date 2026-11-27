# SPI master

STATIC. SPI0/1/2/3/5 use `spi.blocking(&mut p.wait_timer)` for synchronous
embedded-hal 1.0 `SpiBus<u8 | u16 | u32>` and `SpiDevice`. The IRQ binding
and wait alarm belong to the declaring core. When the same core declares
`spi0` and `wait_timer`, the generator emits the ISRAM dispatcher and
returns `rp1::Spi0Parts` with the IRQ capability. All supported controllers offer
the same synchronous traits through explicit `spi.polling()`.

```toml
[package.metadata.rp1]
schema = 2

[package.metadata.rp1.proc0]
gpio = [8]                                 # chip select, driven as a GPIO
spi0 = { sck = 11, mosi = 10, miso = 9 }
wait_timer = "ALARM1"
```

```rust,ignore
use embedded_hal::spi::{MODE_0, Operation, SpiDevice};
use rp1_hal::{fugit::RateExtU32, spi::{Config, Device}};

// In the proc0 entry:
let mut bus = p.spi0.init(&clocks, Config::new(10.MHz(), MODE_0))?;
let mut dev = Device::new(bus.blocking(&mut p.wait_timer), p.pins.gpio8);
dev.transaction(&mut [Operation::Write(&[0x9f]), Operation::Read(&mut id)])?;
```

`examples/peripherals` uses SPI0 on proc0. The SHM SPI proof uses proc1.
The SPI ISR masks the armed FIFO or error source, publishes readiness and
sends an event. Thread mode does FIFO service. The Slot ceiling and IRQ
priority are both 4. The one-shot wait alarm bounds missing device delivery.
Use the explicit polling adapter when comparing serviced wire latency.

## Which blocks

DOC (RP1 peripherals §3.6.1, rp1.dtsi): nine DW_apb_ssi v4.02a instances. SPI4 and SPI7 are
slave-only (`spi-slave` in Linux's DT); SPI0/1/2/3/5/6/8 are masters. Only SPI0..5 are on GPIO
bank 0; SPI6 and SPI8 sit on the internal banks, which this HAL does not expose (as I2C4..6).
On rpi-cm5, SPI2 requires excluding UART1 from `linux.keep` so GPIO1 can route
MISO. SPI5 uses GPIO14/15, which are reserved when the UART0 log sink is declared.

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
  `RP1_CLK_SYS` as `ssi_clk`. SCLK = `Clocks::sys().raw() / BAUDR`, BAUDR even, 2..=65534
  (Linux dw_spi rounds the same way: up, then to even). At 200 MHz: 100 MHz maximum,
  1 MHz = /200, 10 MHz = /20, 25 MHz = /8, 50 MHz = /4; the lowest rate is about 3.05 kHz.
  clk_sys belongs to RP1 (`clock.authority = "rp1"`): no SCMI clock is held or voted for
  SPI, and `Clocks` comes either from proc0's `platform::init` or, on proc1, from
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
Linux, and the overlay sets `rp1_spiN` to `status = "disabled"`. A later overlay that enables
the node again (e.g. `dtparam=spi=on`) or a Linux user of a declared pin (e.g.
`dtoverlay=uart3-pi5` on GPIO 8/9) makes `cargo rp1 check --dtb` fail; at boot rp1boot disables
that node by default and logs `RP1BOOT_LINUX_DISABLED` (`linux.on_conflict = "refuse"` stops
instead).

## Behaviour

STATIC (host model tests, 2026-11-27). `transfer` shifts `max(read, write)` frames.
Frames past `write` send zero, and the driver discards receives past `read`. At
most `fifo_depth()` frames are in flight. One absolute deadline covers the
call, including all chunks of an in-place transfer. A device transaction shares
its deadline with each transfer, flush and chip-select delay. The adapters
default to 100 ms, configurable via `WaitConfig`, and support explicit
`transfer_until`, `flush_until` and `Device::transaction_until` deadlines.

`flush` waits for TX FIFO empty and wire idle. SPI0 uses a frame-time one-shot
alarm to check the shifter after TX FIFO empty. An error or elapsed deadline
after admission retires the wait and requests stop. The transaction releases
CS, and `try_recover` verifies stop before restoring the configuration and
clearing stale FIFO and fault state. A failed or cancelled call may transmit
or receive a prefix. FIFO overflow and underflow return `Overrun`. The DW
multi-master indication maps to `ChipSelectFault`. Motorola controller mode
does not expose a distinct peer framing error. Frame width remains 4–16 bits,
or 4–32 on a controller with the 32-bit frame field. A word narrower than the
configured width is rejected before FIFO access.
All word types share the FIFO service step in ISRAM.

The native chip selects are not routed: SER selects CS0 internally because the DW SSI only
shifts with a slave selected. `Device::transaction` checks the bus once (frame size,
clk_sys, context and deadline) before asserting CS. It flushes before every delay and
before deselecting on success. An error stops the controller and deselects. The unchecked
operations behind this (the hidden `spi::Bus` methods) take arguments only rp1-hal can
make, so code outside the crate cannot call them and skip the check (STATIC, trybuild and
doctest compile-fail cases, through a generic bound and on the polling adapter).

`set_frequency`, `set_mode`, `set_frame_bits` and `set_loopback` (CTRLR0.SRL internal
loopback) wait for the bus to be idle and reprogram the disabled controller.

`init` checks clk_sys, the rate and the frame size before releasing reset. The controller's
own frame limit is checked with SSI disabled after the probe. Use `error.try_into_parts()`
to get the token and untouched pins when they are ready for reuse. If reset or the probe
failed, `error.try_recover()` requests reset and returns the parts once its
`RESETS_DONE1` bit clears. HYP (2026-11-27): when DONE clears after asserting reset.

STATIC (host model). A stale clk_sys, a zero or too-low rate and 3- or 33-bit frames
write no register and route no pin. A 32-bit frame on a 16-bit instance fails after the
probe, leaving the controller disabled and the pins untouched for a retry.

Build `Config` with `Config::new(frequency, mode)` and the `with_*` setters. The frequency
is a `HertzU32`. Dropping the controller leaves it enabled and the pins muxed.
`try_free()` checks TX FIFO empty and BUSY clear before disabling. `free_polling()`
waits up to 100 ms for the same wire-idle condition. IRQ-capable controllers also have `free_until()`. All return the controller in `FreeError` when stop cannot be
confirmed. `try_cancel()` requests disable immediately and may truncate a
frame and discard both FIFOs. Both synchronous adapters report `Busy` during
recovery, and raw interrupt controls cannot reopen the wrapper. Restore the
configuration with `try_recover()`, or collect the parts with `try_free()`
once stop is confirmed.

`Device::free` returns the bus and the chip select as an unconfigured `Pin<CS>` (still driven
high), so the same pin can make a new `Device` (STATIC: a host compile test).

`Config::rx_sample_dly` (default 0) is written to RX_SAMPLE_DLY: MISO is sampled that many
clk_sys cycles later. Only 0 is hardware-tested. It is written without a range check or
read-back, as Linux spi-dw-core does. The usable maximum is the DW_apb_ssi synthesis
parameter SSI_RX_DLY_SR_DEPTH, which is not known for RP1, and what the block does with a
larger value is unverified (recalled, not checked: the databook treats it as no delay).

STATIC (host tests, 2026-11-27). The model separates fast and strict clock
checks. Init and reconfiguration use the strict check and reject a stale clock
before touching reset, mux or controller registers. Bus and device transactions
use the fast check. The model also keeps BUSY asserted after the final receive so
physical flush must wait for the shifter. For `DelayNs` each engine rounds up to
the next microsecond and allows one timer tick for phase.

For an application-managed IRQ, use the instance-specific key and handler.
For SPI0, declare `custom_spi0_handler = true`
and provide `#[rp1_rt::interrupt(procN)] fn SPI0()`. This gives the application
`p.irq.SPI0` and the raw mask and status methods. Raw `enable_interrupt` opens
the RP1 `+0x108` output gate and unmasks the selected sources.
`clear_interrupt` is available on both resource types. See [Interrupts](interrupts.md).

## Hardware proof

STATIC (2026-11-27). `lab/spi-parity` checks internal loopback, RX/TX delivery,
timeouts, shifter idle and sensed CS levels with the RP1 timer.
`lab/verify-spi-parity.py` checks the firmware transcript. It can also check
an optional wire CSV from a bench with a MOSI/MISO loopback and a sample rate
of at least 100 MS/s. The measurements below used the previous polling driver.

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
