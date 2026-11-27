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
  `Deadline::after`, `Uart::set_read_timeout` and the DMA wait APIs.
- `spi::Device` busy-waits for `Operation::DelayNs` and returns `Precondition` in a
  handler or masked context. IRQ-backed CS delays arrive with the SPI wait PR.
- Proc0 gets its clocks from `rp1::platform::init`. Proc1 uses
  `Clocks::wait_from_proc0(&core, timeout)`. Pass `0.micros()` for a single attempt.
- `Uart::split` gives a `Tx` and `Rx` for use in main and an ISR on the owning core.
  Each half exposes its own IRQ status and mask operations. Both `Uart::enable_rx_interrupt`
  and `Rx::enable_interrupt` enable the RX threshold, timeout and error interrupts.
  Acknowledge receive error interrupts with `clear_interrupt`. Use `Uart::join` before
  calling `free` or `abort`. The pin route traits are `TxPin` and `RxPin`.
- Enable `rp1-hal/eh02-adc` for the embedded-hal 0.2 `OneShot` ADC adapter or migrate to
  `Adc::read`.
