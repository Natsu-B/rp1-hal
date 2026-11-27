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
- `spi::Device` busy-waits for `Operation::DelayNs` and returns `Precondition` in a
  handler or masked context. IRQ-backed CS delays arrive with the SPI wait PR.
- Proc0 gets its clocks from `rp1::platform::init`. Proc1 uses
  `Clocks::wait_from_proc0(&core, timeout)`. Pass `0.micros()` for a single attempt.
- `Uart::split` gives a `Tx` and `Rx` for use in main and an ISR on the owning core.
  Each half exposes its own IRQ status and mask operations. Both `Uart::enable_rx_interrupt`
  and `Rx::enable_interrupt` enable the RX threshold, timeout and error interrupts.
  Acknowledge receive error interrupts with `clear_interrupt`. Use `Uart::join` before
  releasing or cancelling the UART. The pin route traits are `TxPin` and `RxPin`.
- Enable `rp1-hal/eh02-adc` for the embedded-hal 0.2 `OneShot` ADC adapter or migrate to
  `Adc::read`.


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
