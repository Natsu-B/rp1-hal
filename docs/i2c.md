# I2C master

STATIC. Declare I2C0–3 and a wait timer on the same core to
use `i2c.blocking(&mut p.wait_timer)` for embedded-hal 1.0 seven-bit
`I2c`. The declaration generates the instance handler and hands its capability to the parts.
`write`, `read`, and `write_read` use the shared `transaction` implementation.
Empty buffers are skipped, adjacent operations of the same direction continue,
direction changes issue repeated START, and the final byte issues STOP. Reads can
exceed the RX FIFO depth: outstanding read commands are bounded by its reported size.

```toml
[package.metadata.rp1]
schema = 2

[package.metadata.rp1.proc0]
i2c1 = { sda = 2, scl = 3 }
wait_timer = "ALARM1"
```

```rust,ignore
use embedded_hal::i2c::I2c;
use rp1_hal::i2c::Speed;

let mut bus = p.i2c1.init(&clocks, Speed::Standard)?;
bus.blocking(&mut p.wait_timer).write_read(0x2d, &request, &mut response)?;
```

STATIC: the generated token has one declared core owner. Initialization consumes
unconfigured pins and selects the function from the bank-0 pin table (3); reversed or
unrelated pins fail to compile.
The overlay disables the owned `rp1_i2cN` Linux node and reserves exactly the declared pins
(`gpio-reserved-ranges`). A Linux user of an owned pin from a config.txt overlay (an enabled
pinctrl group, `*-gpios`, a hog or an rp1_gpio interrupt), or a later overlay that re-enables
the controller, makes `cargo rp1 check --dtb` fail. At boot, rp1boot disables such a node by
default and logs `RP1BOOT_LINUX_DISABLED path=... why=...` (`linux.on_conflict = "refuse"`
stops instead); see [build.md](build.md) and [declaration-and-note.md](declaration-and-note.md).
GPIO2/3 are an I2C1 route. To use the I2C0 route on GPIO0/1, exclude UART1 from `linux.keep`.

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
| I2C4–6 | — | — |

Use explicit polling without a same-core wait timer. For an application IRQ on I2C1, declare
`custom_i2c1_handler = true` and define `#[rp1_rt::interrupt(procN)] fn I2C1()`
to own the raw IRQ in the application. This exposes `p.irq.I2C1` and the
raw mask, status and acknowledge methods.

DOC: the peripheral drives open drain. Pads use input enable, Schmitt input, weak
pull-up and 12 mA sink strength; **external pull-ups are required**. Initialization
disables the pad output during mux selection. It releases only the owned I2C reset;
cold bring-up must already have started GPIO bank0 and the raw microsecond timer.

HW: the controller clock (`ic_clk`) is clk_sys. With clk_sys on XOSC it measured
50 MHz (high period exactly HCNT + SPKLEN + 7 cycles of 20 ns); since
`platform::init` selects PLL_SYS it is 200 MHz. The driver computes the counts
from `Clocks::sys()` at init and `set_speed`: SCL high/low targets 4.6/5.4 us,
0.9/1.6 us and 0.4/0.6 us (I2C minimums kept, nominal 100 kHz, 400 kHz, 1 MHz), a 50 ns
spike filter (3 cycles at 50 MHz, 10 at 200 MHz) and 100 ns of pull-up rise taken off
the low period. HW at 200 MHz (2026-10-29, ESP32
RMT, 100 ns ticks): 100.1 kHz, 400.0 kHz, 1011 kHz (NACK probe only). Actual timing
depends on pull-ups, capacitance, stretching and FIFO service. No shared clock is
programmed and SCMI ownership is unchanged. A clock tree that no longer matches the
`Clocks` value fails with `Precondition` before the controller is touched. `ic_clk` is
clk_sys, so clk_uart is not compared. `init` and `set_speed` read CLK_SYS CTRL/DIV/SEL and
the PLL_SYS tuple twice (`Clocks::sys_current_strict`); every transaction checks once with
`Clocks::sys_current` (CLK_SYS_SEL and DIV_INT, plus PLL_SYS CS/FBDIV_INT/PRIM when SEL is
PLL_SYS; see `docs/spi.md`) (STATIC, host model tests whose fast and strict checks are
counted apart and can disagree: init and `set_speed` use only the strict one, transactions
only the fast one). The counts are computed in whole MHz
in 32-bit arithmetic: an `ic_clk` that is not a whole number of MHz is `Invalid`; for every
whole-MHz `ic_clk` the result (counts or `Invalid`) equals the earlier 64-bit formula's
(STATIC, host test over all whole-MHz u32 values). `Clocks` only gives 50 or 200 MHz.
`init` checks clk_sys and the speed before releasing reset. Use `error.try_into_parts()`
when the token and untouched pins can be reused. If recovery is pending,
`error.try_recover()` asserts only this I²C block's reset and checks that its
`RESETS_DONE0` bit cleared. This can recover after an incomplete reset release,
component ID mismatch or disable timeout with a target holding SCL. The pins
remain untouched. HYP (2026-11-27): when DONE clears after asserting reset.
STATIC (host model). A stale clock or impossible speed writes no register and routes
no pin. The returned parts initialize on retry.
A controller declared under `[proc1]` takes its `Clocks` from
`Clocks::wait_from_proc0(&core, timeout)` (the value proc0's `platform::init` published;
see `docs/adc.md`, "Clocks on proc1"). STATIC: compile-checked in `examples/dual-core`.

STATIC (2026-11-28). Both synchronous adapters require unmasked thread
mode. Each transaction uses one absolute deadline across every operation,
with a 100 ms default. Use `with_wait_config`, `transaction_until`, or
`transaction_polling_until` to supply a different budget. The IRQ handler
masks qualified FIFO, STOP or error sources and signals the waiter. The
calling thread owns the FIFO and buffers. Once STOP arrives, the adapter
confirms the controller is disabled and master idle. A separate 1 ms
budget bounds this check after success or an abort/disable request.

Address NACK, data NACK and arbitration loss map to embedded-hal's matching
error kinds. Other aborts retain `IC_TX_ABRT_SOURCE`, and FIFO errors map to
overrun. A NACK allows another transaction once disable and idle have
been confirmed. After timeout, arbitration loss, other faults or manual
cancellation, call `try_recover()` to confirm stop and discard stale
status before reuse. If the target holds SCL and the controller is still
active, recovery returns `Busy`. A failed transaction may have transferred
a prefix. Recovery leaves the pins muxed and never drives them as GPIO.
Transfers are CPU-driven, with no DMA support.

`try_start_transaction` borrows a driver and its operation buffers. Each
`try_complete` call scans or services a fixed number of entries. The handle
requests stop on cancellation or Drop with a bounded register sequence.
The ISR never owns the buffers. `try_free` (or managed `free_until`) returns the still-muxed pins,
block token and any generated IRQ capability only after disable and idle.
Disable and unpend a separately owned raw NVIC line first, or ensure its
handler accepts the returned controller. `free_polling` is a thread-only
disable wait of up to 100 ms. It retains the driver and mask on failure.

STATIC (host model). The FIFO step is an `#[isram]` root. Host tests cover
FIFO boundaries, direction changes, STOP, abort details, arm races,
stale interrupts, missing delivery, timer wrap, cancellation and re-arm.

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
documents standard/fast operation up to 400 kHz. Payload acceptance covers 100/400 kHz.
The 1 MHz SCL measurement used a read-only NACK probe and the ESP32 RMT input.
The parity fixture reports 1 MHz payload trials as open pending an Fm+-capable peer.

Hardware results and limits are recorded separately after each run, in run notes kept
outside this repository (HW, 2026-10-29).
