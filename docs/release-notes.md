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
- `Timer::now` returns a 64-bit microsecond instant. Use `Timer::now_32` for the absolute
  deadline passed to `Alarm::schedule`. Use `fugit::ExtU32` for durations such as `10.millis()`
  in `Timer::delay`, `Uart::set_read_timeout` and the DMA wait APIs.
- Proc0 gets its clocks from `rp1::platform::init`. Proc1 uses
  `Clocks::wait_from_proc0(&core, timeout)`. Pass `0.micros()` for a single attempt.
