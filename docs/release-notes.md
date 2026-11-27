# Firmware API changes for v0.2

STATIC

- Owned bank-0 GPIO pins expose `pad_config` and `set_pad_config` for pull, drive strength,
  slew rate and Schmitt trigger settings. Use `into_input_with_pad` or `into_output_with_pad`
  to supply a configuration at mode transition. PWM outputs expose the routed pin via
  `pin_mut`.
- Firmware rates use `rp1_hal::fugit::HertzU32`. Import `fugit::RateExtU32` from the HAL for
  literals such as `10.MHz()` and `115_200.Hz()`. Read the platform rates with `Clocks::sys`
  and `Clocks::uart`, SPI SCLK with `Spi::frequency`, and the ADC and PWM clocks with
  `clock`. Use `Config::with_frequency` for SPI's rate limit. TOML rates and SCMI values
  remain integer hertz.
- `Timer::now` returns a 64-bit microsecond instant. Use `Deadline::after(timeout)?`
  or `Deadline::at(instant)` for an absolute deadline with a maximum usable horizon
  of `i32::MAX` µs. For raw alarm ISRs, use `alarm.try_arm(timer.now_32() + 10.millis())?`
  and `alarm.try_expired()` in place of `schedule` and `finished`. `try_arm` also accepts
  a full-width instant or deadline. Timer operations use the flat `timer::Error`.
- Declare `wait_timer = "ALARM1"` on the owning core. Replace
  `Timer::new().delay(10.millis())` with `p.wait_timer.delay_ms(10)` after importing
  `embedded_hal::delay::DelayNs`. The IRQ delay rejects handler and masked contexts,
  including Slot borrows. Use `p.wait_timer.delay(10.millis())?` to handle errors or
  `Timer::delay_polling` to choose a busy wait explicitly.
- Use `fugit::ExtU32` for durations such as `10.millis()` in `WaitTimer::delay`,
  `Deadline::after`, `uart.polling().set_read_timeout` and the DMA wait APIs.
- Proc0 gets its clocks from `rp1::platform::init`. Proc1 uses
  `Clocks::wait_from_proc0(&core, timeout)`. Pass `0.micros()` for a single attempt.
- `Uart::split` gives a `Tx` and `Rx` for use in main and an ISR on the owning core.
  Each half exposes its own IRQ status and mask operations. Both `Uart::enable_rx_interrupt`
  and `Rx::enable_interrupt` enable the RX threshold, timeout and error interrupts.
  Acknowledge receive error interrupts with `clear_interrupt`. Use `Uart::join` before
  releasing or cancelling the UART. The pin route traits are `TxPin` and `RxPin`.
- Declare `dma = { channels = [0, 2], pool = { slots = 2, bytes = 1024 } }` to get
  `p.dma_pool` and the selected channels. Use
  `ch.copy_polling(&mut src, &mut dst, duration)` with borrowed pool leases.
  `Transfer::try_complete` replaces `poll`. `wait_polling(end)` replaces `wait`.
- `try_cancel` and timer-backed `cancel_until` replace `cancel` and `dma::Stopped`.
  `try_reclaim` ends a transfer borrow after a proven stop. Drop is bounded and can
  quarantine both slots for the rest of the epoch. `dma::Error` is `#[non_exhaustive]`.
- SRAM-to-DDR copies require `unsafe Channel::start_raw` and an independent outbound
  drain and mapping proof.
- The experimental ring DMA adapter uses its own scratch pool and internal timeout.
  The public `dma::RING_TIMEOUT_US` constant was removed.


## Peripheral migration

STATIC (2026-11-27). UART, SPI and I²C expose the existing synchronous engines
through `.polling()`. The controller types and `Parts` name the required pin tuple.
Declarations generate aliases such as `rp1::Uart1` and `rp1::Uart1Pins`.
UART and I²C accept a baud rate or bus speed directly as well as a `Config`.

| Before | After |
| --- | --- |
| `p.uart1.init(&clocks, 115_200.Hz())?` | `p.uart1.init(&clocks, 115_200.Hz())?` (with an owner-preserving error type) |
| `tx.write_all(bytes)?` and `rx.read_byte()?` | `tx.polling().write_all(bytes)?` and `rx.try_read_byte()?` |
| `if tx.try_write_byte(byte)` and `if tx.try_flush()` | `if tx.try_write_byte(byte)?` and `if tx.try_flush()?` (`Busy` after cancellation) |
| `spi::Device::new(bus, cs)` | `spi::Device::new(bus.polling(), cs)` |
| `p.i2c1.init(&clocks, Speed::Standard)?` and `bus.write(addr, bytes)?` | `p.i2c1.init(&clocks, Speed::Standard)?` and `bus.polling().write(addr, bytes)?` |
| `spi::Parts<0, 11, 10, 9>` | `spi::Parts<0, rp1::Spi0Pins>` for a declared instance |
| `token.init(tx, rx, &clocks, baud)` | `uart::Parts::new(token, uart::Pins::builder().tx(tx).rx(rx)).init(&clocks, baud)` |
| `driver.free()` and `driver.abort()` | `driver.try_free()` for an immediate check or `driver.free_polling()` for the bounded wait. Use `driver.try_cancel()` to request stop before collecting parts. |
| `e.resources` and `e.driver` | `e.try_into_parts()` or `e.try_recover()` on a peripheral initialization error. On a release error, `e.into_driver()` returns the driver and `e.into_parts()` returns it with the cause. |
| `PadConfig { pull: Pull::Up, ..PadConfig::INPUT_DEFAULT }` | `PadConfig::INPUT_DEFAULT.with_pull(Pull::Up)` |

Propagate UART, SPI and I²C initialization errors with an outer error that
retains the owner, or explicitly handle recovery.

`Pin::erase()` moves ownership to an `ErasedPin` of the same mode. A failed
number match in `try_into_pin` returns that owner.

## Experimental link-wake wire profile

**STATIC · 2026-11-27.** Existing schema 2 manifests keep the NOTE v2 and SHM
header v1 path. The lab can opt into NOTE v3 and exercise the v2 SHM header
codec, boot-scoped mailbox tickets, generation-scoped ring tickets and the
single ALARM3/MSI-X vector owner.

Two validation rules also apply to NOTE v2. `SHM_AREA.sram_end` must cover
the status block through `0xbf`, including in a descriptor with no rings.
A DT `rp1,abi` cell, when present, must be exactly `<2>`. Older generated
v2 notes and overlays already satisfy both rules.

Before, using the schema 2 link:

```toml
[package.metadata.rp1]
schema = 2
[package.metadata.rp1.proc1]
host_irq = true
```

The delivery fixture selects the experimental wire and enables the runtime feature.

```toml
[package.metadata.rp1]
schema = 2
link_wake = "experimental"
[package.metadata.rp1.proc1]
host_irq = true
[dependencies]
rp1-rt = { version = "0.1", features = ["experimental-link-wake"] }
```

NOTE v3 uses ELF type `0x52503103`, DT `rp1,abi = <3>` and the critical
`LINK_WAKE` TLV. It reserves link SRAM `0xc0..0xff` for the mailbox.
Region allocation starts at `0x100`. Ring controls and slots keep their
sizes and use reserved words for notification. The fixture gets a single
`VectorOwner` and the raw generated layout. Application endpoints continue
to use schema 2 without the opt-in. The v3 loader and firmware must travel
together, and a v3 client requires the upcoming link-wake wait backend.

**HYP · 2026-11-27.** Physical delivery through MSI-X/UIO and the posted
window ordering still require hardware verification.

## UART1 synchronous waits

**STATIC · 2026-11-27.** Declare UART1 and `wait_timer` on the same core.
The declaration generates the UART1 handler and keeps the IRQ in
`rp1::Uart1Parts`. The initialized driver and both split halves can use
`blocking(&mut p.wait_timer)`. For a raw handler, set
`custom_uart1_handler = true` and supply `#[rp1_rt::interrupt(procN)] fn UART1()`.
This gives the app `p.irq.UART1` and a UART with immediate I/O and explicit polling.
See the [peripherals UART example](../examples/peripherals/src/uart.rs).

| Before | v0.2 |
|---|---|
| `uart.polling().read(&mut buf)?;` | `uart.blocking(&mut p.wait_timer).read(&mut buf)?;` |
| `tx.polling().write_all(bytes)?;` | `tx.blocking(&mut p.wait_timer).write_all(bytes)?;` |
| Manual time checks around `try_read_byte` | Create one `Deadline` and share it across `read_exact_until`, `write_all_until` and `flush_until`. |
| Polling from an ISR or critical section | **Breaking.** `polling()` read/write/flush, the explicit `*_polling` waits and `free_polling` require unmasked thread mode on every instance. They report `WrongContext` or `Masked` when a wait would run. Use bounded `try_read_byte`, `try_write_byte` and `try_flush` in an ISR. |

UART1's blocking adapter waits for RX watermark, RX timeout or receive error,
or for the TX watermark when the FIFO is full. Below the RX watermark, a lone
byte wakes on the PL011 RX timeout about 32 bit times after its stop bit
(about 280 µs at 115200 baud). Lower IRQ latency needs the planned FIFO and
watermark configuration. Physical flush uses a baud-derived alarm and confirms
BUSY has cleared. A TX timeout leaves the queued bytes
transmitting. Retry flush, or rejoin the halves and request reset. An RX timeout
retires its wait while keeping incoming bytes and their status available.
Handler or masked calls fail, and a disabled IRQ or alarm is reported as an
error. The default RX budget is 100 ms and TX uses its baud-derived FIFO budget.
Use `with_timeout` to override the adapter budget. Zero expires immediately.

Both synchronous adapters expose embedded-io 0.6 I/O and readiness. Add
`embedded-io-07` for both 0.6 and 0.7 implementations on the same adapter.
UART read, write and flush also implement the native nonblocking serial traits
on the raw driver and its corresponding halves with `eh-nb` or `eh02`.
Overrun maps to `InvalidData` in both embedded-io versions.

UART0 and UART2–4 offer immediate operations and explicit `polling()`.
UART0's boot log sink continues to use its dedicated bounded path. UART5 is
unrouted on bank 0. Polling methods accept an absolute deadline with
`read_polling`, `write_polling` and `flush_polling`.


## SPI0 synchronous waits

**STATIC · 2026-11-27.** Declare `spi0` and `wait_timer` on the same core for
`rp1::Spi0Parts` and a generated handler. Initialize as before with
`p.spi0.init(&clocks, Config::new(1.MHz(), MODE_0))?`. For a raw handler,
set `custom_spi0_handler = true` and provide
`#[rp1_rt::interrupt(procN)] fn SPI0()`. This exposes `p.irq.SPI0` and the
raw mask and status methods.

| Before | v0.2 |
|---|---|
| `bus.polling().transfer(&mut rx, &tx)?` | `bus.blocking(&mut p.wait_timer).transfer(&mut rx, &tx)?` on managed SPI0 |
| `Device::new(bus.polling(), cs)` | `Device::new(bus.blocking(&mut p.wait_timer), cs)` on managed SPI0 |
| Polling timeout restarted on each FIFO progress event | One absolute deadline for each call or transaction, including CS delays and flush. The default is 100 ms. Share a deadline with `transfer_until`, `flush_until` and `transaction_until`. |
| An exchange error re-enabled the controller implicitly | Call `try_recover` after an admitted transfer faults or times out. The driver confirms stop before clearing stale state and re-enabling. |
| Polling bus operations from a handler or critical section | **Breaking.** `polling()` transfer, flush and reconfiguration, and `free_polling` require unmasked thread mode on every instance and otherwise return `WrongContext`. Use `try_read_word`, `try_write_word` or `try_start_transfer` in an ISR. |

SPI0's IRQ adapter uses WFE for RX and TX FIFO waits and the dedicated timer
for shifter and CS-delay waits. Explicit `polling()` remains available on all
supported controllers. The native transfer handle borrows both buffers.
With `eh-nb`, the native driver also implements
`embedded_hal_nb::spi::FullDuplex`.

## I2C1 synchronous waits

**STATIC · 2026-11-28.** Declare `i2c1` and `wait_timer` on the same core
for the generated `rp1::I2c1Parts`. Initialize with
`p.i2c1.init(&clocks, Speed::Standard)?`.

| Before | v0.2 |
|---|---|
| `bus.polling().write_read(addr, tx, rx)?` | `bus.blocking(&mut p.wait_timer).write_read(addr, tx, rx)?` on managed I2C1 |
| `i2c::Parts<1, rp1::I2c1Pins>` in application arguments | `rp1::I2c1Parts` on managed I2C1 |
| Each FIFO step could restart the 100 ms timer | One 100 ms transaction budget. Override it with `with_wait_config` or supply an absolute deadline. |
| Polling operations, speed changes or release in an ISR or critical section | **Breaking.** Explicit polling requires unmasked thread mode and returns `WrongContext` otherwise. Use the native `try_start_transaction` and bounded `try_complete` from a handler. |
| A successful disable after any transfer error permitted reuse | After timeout, arbitration loss, other non-NACK errors or cancellation, call `bus.try_recover()?` before starting a new transaction. NACK permits reuse if the controller stopped. |
| Application-owned raw I2C1 mask and status methods | Set `custom_i2c1_handler = true`, define `#[rp1_rt::interrupt(procN)] fn I2C1()`, and use `p.irq.I2C1`. |

| Instances | Wait modes |
|---|---|
| I2C1 with the generated handler | IRQ/WFE, explicit polling |
| I2C0, unmanaged I2C1, I2C2–3 | Explicit polling |
| I2C4–6 | No bank-0 route |

## ADC0 one-shot waits

**STATIC · 2026-11-28.** A same-core `wait_timer` and `adc = {}` generate
the IRQ52 handler, `rp1::AdcParts` and `rp1::Adc`. Declare
`custom_adc_handler = true` to retain `p.irq.ADC_FIFO` and link a raw handler.
When `wait_timer` is conditional, its feature controls the parts type and
raw IRQ field. The temperature sensor is powered during setup.

| Before | After |
|---|---|
| `adc.read(TEMPERATURE)?` | `adc.blocking(&mut p.wait_timer).read(TEMPERATURE)?` |
| `adc.read_temperature()?` | `adc.blocking(&mut p.wait_timer).read_temperature()?`, or `adc.read_temperature_polling()?` for an explicit busy wait |
| `adc.read(ain)?` | `adc.blocking(&mut p.wait_timer).read(&ain)?` using a typed board input, or `adc.read_polling(&ain)?` |
| `let mut input = Input::<4>` and a completing `OneShot::read` | `let mut input = TEMPERATURE` and handle `WouldBlock` with `eh02` or `eh02-adc` |
| `adc.free()` | `adc.try_free()`, with ownership retained on error. If needed, use `adc.try_cancel()` until recovery succeeds. |
| `token.init(clocks)?` converting to `adc::Error` | Preserve `adc::InitFailure` or its recovery owner in the caller's error. |

The synchronous IRQ and explicit polling methods reject handler or
masked calls. Use `try_start` and `try_sample` for immediate I/O.
