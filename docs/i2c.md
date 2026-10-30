# I2C master

STATIC: `rp1_hal::i2c` implements embedded-hal 1.0 seven-bit `I2c` with polling.
`write`, `read`, and `write_read` use the shared `transaction` implementation.
Empty buffers are skipped, adjacent operations of the same direction continue,
direction changes issue repeated START, and the final byte issues STOP. Reads can
exceed the RX FIFO depth: outstanding read commands are bounded by its reported size.

```toml
[package.metadata.rp1.proc0]
pins = [2, 3]
peripherals = ["I2C1"]
```

```rust,ignore
use embedded_hal::i2c::I2c;
let mut bus = p.I2C1.init(p.pins.gpio2, p.pins.gpio3, &clocks,
    rp1_hal::i2c::Speed::Standard)?;
bus.write_read(0x2d, &request, &mut response)?;
```

STATIC: the generated token has one declared core owner. Initialization consumes
unconfigured pins and selects function 3; reversed or unrelated pins fail to compile.
The overlay disables the owned `rp1_i2cN` Linux node and reserves the declared pins.
The final DT checker rejects a later overlay that re-enables the controller or an
enabled Linux pinctrl user of an owned pin. GPIO2/3 require I2C1 on the same core;
GPIO0/1 remain with Linux UART1 on the CM5 profile.

DOC: the [RP1 GPIO table](https://datasheets.raspberrypi.com/rp1/rp1-peripherals.pdf)
exposes the following bank-0 routes. SDA and SCL can independently select a listed
route for the same controller. IO_BANK1/2 are internal and deliberately unavailable
through this board's safe HAL and ownership declaration. The PAC provides the
DesignWare register slice for all seven instances, I2C0..6.

| Controller | SDA | SCL |
|---|---|---|
| I2C0 | 0, 8 | 1, 9 |
| I2C1 | 2, 10 | 3, 11 |
| I2C2 | 4, 12 | 5, 13 |
| I2C3 | 6, 14, 22 | 7, 15, 23 |

DOC: the peripheral drives open drain. Pads use input enable, Schmitt input, weak
pull-up and 12 mA sink strength; **external pull-ups are required**. Initialization
disables the pad output during mux selection. It releases only the owned I2C reset;
cold bring-up must already have started GPIO bank0 and the raw microsecond timer.

HW: the controller clock (`ic_clk`) is clk_sys. With clk_sys on XOSC it measured
50 MHz (high period exactly HCNT + SPKLEN + 7 cycles of 20 ns); since
`clocks::Authority::init` selects PLL_SYS it is 200 MHz. The driver computes the counts
from `Clocks::sys_hz()` at init and `set_speed`: SCL high/low targets 4.6/5.4 us,
0.9/1.6 us and 0.4/0.6 us (I2C minimums kept, nominal 100 kHz, 400 kHz, 1 MHz), a 50 ns
spike filter (3 cycles at 50 MHz, 10 at 200 MHz) and 100 ns of pull-up rise taken off
the low period. HW at 200 MHz (2026-10-29, ESP32
RMT, 100 ns ticks): 100.1 kHz, 400.0 kHz, 1011 kHz (NACK probe only). Actual timing
depends on pull-ups, capacitance, stretching and FIFO service. No shared clock is
programmed and SCMI ownership is unchanged. A clock tree that no longer matches the
`Clocks` value fails with `Precondition`: `init`, `set_speed` and every transaction compare
clk_sys and clk_uart with it before touching the controller (STATIC, host model test; the
transaction check used to cover only PLL_SYS and CLK_UART, so clk_sys moved to XOSC went
unnoticed). `init` checks the clocks and the speed before it releases the reset; on any
error it returns `InitError` with the token and the untouched pins, so the caller can retry
(`?` converts it to `Error`).
A controller declared under `[proc1]` takes its `Clocks` from
`Clocks::wait_from_proc0(&core, timeout_us)` (the value proc0's `Authority::init` published;
see `docs/adc.md`, "Clocks on proc1"). STATIC: compile-checked in `examples/dual-core`.

STATIC: address NACK, data NACK and arbitration loss map to embedded-hal's matching
error kinds. Other aborts retain `IC_TX_ABRT_SOURCE`. FIFO errors are reported as
overrun. A 100 ms interval without transfer progress returns `Timeout`; cleanup
has its own bounded disable wait. Error cleanup requests abort/STOP, then disables
the controller and acknowledges interrupt state. A failed disable poisons the
handle; further calls return `Precondition`, requiring a fresh boot/unsafe owner
reinitialization. There is no GPIO bus-clear sequence or interrupt/DMA implementation.

STATIC: the polling loop is an `#[isram]` root; `cargo rp1 build` places its callees
within the 8 KiB budget. Host tests cover FIFO boundaries, operation directions,
NACK/abort mapping, timer wrap, stalled transfers, pin types and DT ownership.

The hardware fixture is `lab/shm-firmware/src/i2c_proof.rs` with the dedicated
`lab/esp32-i2c-peer` firmware. It only writes the ESP32 at 0x2d; negative probes at
0x2e are reads. The peer reloads its reply only after a read that ends in STOP, so the
proof starts with an unchecked resync read and runs read-then-write last.
`lab/check-shm.sh` requests the proof after the existing Linux/SHM
checks and fails unless the reply word is 1; `lab/verify-shm.py` also requires the
three `I2C_PASS` lines and `I2C_RESULT request=1 Ok` on UART0. `lab/measure-i2c.py` uses passive ESP32 RMT capture for SCL periods. Its
`--self-test` checks extraction without hardware. The peer temporarily replaces the
general ESP32 trace firmware, retaining CM5 boot controls and leaving other pins as
inputs; the previous flash must be backed up before deployment.

DOC: Espressif's [ESP32 I2C guide](https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/peripherals/i2c.html)
documents standard/fast operation up to 400 kHz. Payload proofs therefore use
100/400 kHz; the 1 MHz mode is exercised only by read-only NACK probes with the
ESP32 slave stopped and its RMT input measuring SCL. A 1 MHz-capable data peer is
still needed to qualify payload transfers at that speed.

Hardware results and limits are recorded separately after each run, in run notes kept
outside this repository (HW, 2026-10-29).
