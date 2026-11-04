# RP1 PWM driver — STATIC / HW 未検証

`rp1_hal::pwm` owns PWM0 or PWM1 as a whole, with four trailing-edge channels,
independent 32-bit periods and compares, and optional owned GPIO outputs. This implementation
has host tests only. Waveform timing, reset release and update completion are **HW 未検証**.

## Use and ownership

Schema 2 example (STATIC):

```toml
[package.metadata.rp1.proc1]
pwm0 = { channel0 = 12, channel1 = 13, channel2 = 18, channel3 = 19 }
gpio = [22, 23]
```

The generated `p.pwm0` is a block `pwm::Token<0>`. Optional output pins stay in `p.pins`;
`channelN` specifies a checked route and reserves the pin. Unlike fixed UART/I2C/SPI pin
bundles, callers attach each PWM pin explicitly:

```rust,no_run
# use rp1_hal::{clocks::Clocks, gpio::Pin, pwm::{Error, Token}};
# use embedded_hal::pwm::SetDutyCycle;
fn output(token: Token<0>, pin: Pin<12>, clocks: &Clocks) -> Result<(), Error> {
    let mut channels = token.init(clocks).map_err(|e| e.error)?;
    let mut output = channels.channel0.output(pin);
    output.set_period_ticks(50_000)?; // 50 MHz / 50000 = 1 kHz
    output.set_duty_cycle_percent(25)?;
    output.enable()?;
    // Later, stop and return the output and pin before returning the whole block.
    let (channel, pin) = output.free()?;
    channels.channel0 = channel;
    let _token = channels.free()?;
    let _pin = pin;
    Ok(())
}
```

STATIC: initialization selects XOSC / 1 = 50 MHz, disables all channels, DMA requests and
interrupts, and initializes each channel to period 65535, duty zero, phase zero. Enable is
explicit. One tick is 20 ns. `set_period_ticks` accepts 1..=u32::MAX only while disabled and
resets duty to zero. `set_duty_ticks` accepts 0..=period. Embedded-hal `SetDutyCycle` uses
normalized 0..=65535 fractions, rounded down to ticks; 65535 maps exactly to period, even
for periods larger than 65535. Small periods have correspondingly coarse resolution.

STATIC: enable/disable use peripheral SET/CLR aliases; every enable, disable, period and duty
write is followed by SET_UPDATE (GLOBAL_CTRL bit 31, self-clearing), as the Linux driver does
after its channel writes, and waits for it to clear with a 10 ms raw-timer deadline. Whether a
new compare appears immediately or at the end of the running period is HW 未検証. No synchronous waveform-observation guarantee is implied
by returning from a setter. Clock checks reject a changed rate or stopped clock before
channel writes. `InitError` returns the token; reset/update failures may leave partial
hardware state. `free` consumes ownership on error, without issuing a new token.
Dropping an output leaves its pin and channel configured; stop it explicitly first.

STATIC: the declaration reserves one block ID (48 or 49), one reset and one clock, regardless
of the number of output pins. It disables the Linux PWM node, excludes its reset from Linux
bring-up, and holds its clock against SCMI rate/disable requests. Boot checks reject
or disable Linux consumers of that owned block, including pinmux users, under the selected
conflict policy. Linux and the other core cannot declare another channel of the same block.
Channels are neither Send nor Sync, preventing their distribution between cores in safe Rust.
Reclaiming the token requires all four channels back. No shared-SRAM RMW atomics are used.

STATIC: supported PWM0 routes are C0=GPIO12/a0, C1=GPIO13/a0, C2=GPIO14/a0 or GPIO18/a3,
C3=GPIO15/a0 or GPIO19/a3. The UART0 logger owns GPIO14/15, so use GPIO18/19 for those
channels when logging. The current GPIO API owns only bank 0 (GPIO0..27); PWM1 has no
bank-0 output. `pwm1 = {}` allows internal channel diagnostics, with no fabricated pin
route. GPIO28..53 remain Linux board lines; routing PWM1 externally needs a separate board
ownership design (for example the fan on GPIO45). FIFO/DMA, interrupts, inversion, phase
control and alternate waveform modes are outside this driver's current API.

## Register evidence

Every new access below is STATIC evidence from the checked-in register database at this
revision, cross-checked with the public specifications. Prior HW provenance in the database
does not constitute a HW test of this implementation.

| Register/access | Value or field relied on | Evidence in `rp1-regdb/data/` |
|---|---|---|
| PWM0/1 base | 0x40098000 / 0x4009c000 | `peripherals/pwm0.toml`, `pwm1.toml`, peripheral base |
| GLOBAL_CTRL +0x00 | EN[3:0], SET_UPDATE[31], self-clearing | same, `global_ctrl`; peripheral aliases SET +0x2000, CLR +0x3000 |
| FIFO_CTRL +0x04 | write zero to disable DREQ[31] and flush[5] | same, `fifo_ctrl`; no FIFO data access |
| CHAN0..3_CTRL +0x14 + C*0x10 | MODE=1, FIFO_POP_MASK[8]=1; BIND/USEFIFO/SDM/INVERT clear | same, `chanN_ctrl` |
| CHAN0..3_RANGE +0x18 + C*0x10 | unsigned 32-bit period | same, `chanN_range` |
| CHAN0..3_PHASE +0x1c + C*0x10 | zero phase | same, `chanN_phase` |
| CHAN0..3_DUTY +0x20 + C*0x10 | unsigned 32-bit compare | same, `chanN_duty` |
| INTE +0x58; INTF +0x5c | zero masks interrupts and clears forced sources | same, `inte`, `intf` |
| RESETS CTRL1 CLR 0x40017004 | clear bit 4 (PWM0) / bit 5 (PWM1) | `peripherals/resets.toml`, `ctrl1`, alias layout |
| RESETS DONE1 0x4001401c | corresponding bit high before PWM access | same, `done1`; PWM1 fields have reverse-engineered STATIC provenance |
| CLK_PWM0_CTRL/1_CTRL 0x40018074 / 0x40018084 | AUXSRC[9:5]=2 (XOSC), ENABLE[11]=1 | `peripherals/clocks-main.toml`, matching CTRL fields; existing `scmi::RP1_CLOCKS` parent table |
| CLK_PWM0/1_DIV_INT +4, DIV_FRAC +8 | integer 1, fraction 0 | same, `clk_pwmN_div_int`, `clk_pwmN_div_frac`; existing SCMI generator |
| TIMER TIMERAWL 0x400ac028 | wrapping 1 MHz deadline | `peripherals/timer.toml`, `raw_low`; existing `linux_blocks::Mmio` |
| GPIO mux/pad and RIO OE | existing `Pin::into_alternate`: SYS_RIO0 reset bit 19; GPIO CTRL 0x400d0004+P*8; pad 0x400f0004+P*4; RIO OE CLR 0x400e3004 | `resets.toml`, `io-bank0.toml`, `pads-bank0.toml`, `sys-rio0.toml` under `peripherals/`; pin function in `pinmux/bank0.toml` |

STATIC: `Clocks::still_current` admission reuses the existing CLK_SYS/PLL_SYS/CLK_UART
register checks in `clocks.rs`; no new clock-tree register interpretation is introduced.
Host tests compare both PAC block layouts with the offsets above and exercise both instances
and all four channels against fake MMIO. These tests cannot verify clock-domain timing.

DOC sources: [RP1 peripherals datasheet](https://datasheets.raspberrypi.com/rp1/rp1-peripherals.pdf)
(2023-11-07, §2.4, Table 4, §3.4, Tables 24–50) specifies the bases, aliases, routing, mode
and update semantics. [Raspberry Pi Linux PWM driver](https://github.com/raspberrypi/linux/blob/rpi-6.12.y/drivers/pwm/pwm-rp1.c)
confirms the period-to-range convention and channel setup. They are register references;
the HAL does not depend on Linux kernel internal APIs.

## Host checks and example

STATIC: `cargo test --workspace` includes fake-MMIO reset/clock/update failures, full-scale
arithmetic, pin-route generation, compile-fail pin reuse and cross-core channel transfer,
whole-block declaration conflicts, overlay reservations and held SCMI clocks.

The diagnostic example is `lab/pwm-loopback` (HW 未検証). Build without deployment:

```sh
CARGO_BUILD_JOBS=1 cargo rp1 build -p pwm-loopback --target thumbv7m-none-eabi --release
```

On a memory-limited shared host, wait for any other cargo process before each invocation.
The example keeps proc0 servicing SCMI/endpoint events while proc1 counts loopback samples.
It exercises two channels, five duties, a disabled-only period change, pin return and whole
block token reinitialization. It prints observations, not an automatic HW pass verdict.

## Proposed HW acceptance — HW 未検証

1. Schedule with the hardware owner. Save the active boot tuple and hashes, then cold-boot
   the example twice. Retain UART0/UART10 captures through terminal completion. Require
   `PWM_BOOT`, Linux SCMI/UART1 probe and no panic, reset timeout or `PWM_FAIL`.
2. With power off, wire GPIO12 to GPIO22 and GPIO13 to GPIO23, inputs only on GPIO22/23,
   using the board's 3.3 V GPIO header and common ground. No ESP32 output drive is needed.
3. For each `PWM_SAMPLE`, divide high by total samples and rising edges by elapsed time.
   On A, 0/100% must have no edges and stable low/high after settling; at 25/50/75%, expect
   1 kHz within 5% and duty fraction within 0.05. B must remain 500 Hz and 50% within the
   same tolerances through A's changes. Poll sampling is approximate: record sample count
   and timing, and investigate aliasing before judging a marginal result.
4. `PWM_REPERIOD` should show A at 500 Hz, 25%. `PWM_DISABLED` must have no continuing
   edges. The example repeats 60 rounds (~90 s) across Linux startup and `clk_disable_unused`,
   then prints `PWM_DONE reinitialized=1`; measure the final A=1 kHz/B=500 Hz 50% outputs
   again. Require both cold boots to complete; a marker alone is not waveform evidence.
5. Inspect the final DT and reserved GPIOs: Linux PWM0 is disabled, GPIO12/13/22/23 reserved,
   PWM1 is untouched (the stock CM5 IO DT already disables both PWM nodes;
   a fan overlay would enable PWM1). Attempting a conflicting Linux PWM0 consumer or pinctrl use
   must invoke the established disable/refuse policy. Linux SCMI disable/rate requests for
   PWM0 must leave the waveform and 50 MHz clock intact; keep Ethernet/UART1 regression.
6. Extend the diagnostic with C2/C3 on GPIO18/19 and spare input pins, avoiding UART0. Repeat
   independent periods and duties including period 1, 65535, 65536 and a long u32 period;
   use a suitable instrument for frequencies beyond GPIO polling. Check for a one-tick
   period discrepancy before claiming exact frequency or glitch-free reconfiguration.
7. PWM1 internal-only test: declare `pwm1 = {}` on one core, initialize and enable each channel,
   observe its INTR reload bit (5+C, W1C per regdb/datasheet) with diagnostic code under `lab/`,
   clear between polling windows, and log nonzero progress to UART. This verifies reset,
   clock and counter activity, not an external PWM1 waveform. Do not mux GPIO45 or disturb
   the Linux fan without a separately reviewed board-line ownership change.
