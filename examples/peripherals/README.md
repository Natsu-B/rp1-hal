# RP1 peripheral examples

One CM5 firmware runs seven finite examples on proc0. Each module lists the
declaration it uses from [Cargo.toml](Cargo.toml) and takes the corresponding
typed resources. After each check, UART0 prints
`EXAMPLE_END name=<name> result=PASS` on success or `EXAMPLE_FAIL` with the cause.
The next example runs either way.

| Run order and source | Wiring and expected result |
|---|---|
| [blinky](src/blinky.rs) | GPIO13 → 1 kΩ → GPIO23. Three low/high cycles read back on GPIO23. |
| [interrupt](src/interrupt.rs) | No wiring. Three ALARM0 interrupts approximately 20 ms apart. |
| [spi](src/spi.rs) | GPIO8 CS, GPIO9 MISO, GPIO10 MOSI, GPIO11 SCLK. Keep conflicting devices disconnected. The internal loopback returns all four bytes. |
| [adc](src/adc.rs) | No wiring. Three unsaturated internal temperature ADC samples and a temperature reading in millidegrees Celsius. |
| [pwm](src/pwm.rs) | GPIO12 → 1 kΩ → GPIO22. About 100 rising edges in 100 ms, 45–55% high, and no pulses after disable. |
| [i2c](src/i2c.rs) | GPIO2 SDA and GPIO3 SCL to ESP32 GPIO21/22 with 3.3 V pull-ups. The [lab peer](../../lab/esp32-i2c-peer) at 0x2d returns the expected 64 bytes after a 16-byte write. |
| [uart](src/uart.rs) | GPIO0 TX to adapter RX and GPIO1 RX to adapter TX, 3.3 V, 115200 8N1. After `UART1 READY` on UART0, send eight nonce bytes, compare the echo byte for byte and send 0x06 within 15 seconds. |

Share ground with each external peer. Capture UART0 on GPIO14 TX at 115200
8N1. Leave Linux UART1 and PWM overlays disabled for this firmware and keep
Linux pin configurations off the example pins.

**STATIC (2026-11-27)** Build and check the integrated image from the repository
root. Use the board DTB with the Linux boot overlays applied.

```sh
cargo rp1 build --locked -p peripherals --target thumbv7m-none-eabi --release
cargo rp1 check target/thumbv7m-none-eabi/release/peripherals --dtb board.dtb
```

**HYP (2026-11-27)** The checks in the table describe expected bench results.
