# RP1 PWM driver

`rp1_hal::pwm` owns PWM0 or PWM1 as a whole, with four trailing-edge channels,
independent 32-bit counters and compares, and optional owned GPIO outputs. This implementation
has static gates (2026-11-27) and the PWM0 hardware results below. PWM1 and exact
update timing remain **HW 未検証**.

## PWM0 loopback results (HW, 2026-11-05)

Two cold boots of `lab/pwm-loopback`, with 1 kΩ series connections GPIO12→22 and GPIO13→23,
each completed 60 rounds: 300 `PWM_SAMPLE`, 60 `PWM_REPERIOD`, 60 `PWM_DISABLED` and
`PWM_DONE reinitialized=1`, with no `PWM_FAIL`. GPIO input configuration sets both IE and
OD; Linux-side readback after boot confirmed both input pads retained those bits.

Every A sample met the 0/100% constant-level checks, or 1 kHz ±5% and requested
25/50/75% high fraction ±0.05. Every B sample met 500 Hz ±5% and 50% ±0.05, including
Linux boot and rounds after `clk_disable_unused`. Reperiod samples met 500 Hz ±5% and
25% ±0.05; disabled samples had zero rising edges. Linux reached its root shell, SCMI
v2.0 and UART1 bidirectional nonce checks passed, and both PWM DT nodes remained disabled
with no Linux PWM probe. These are GPIO-polling observations, not oscilloscope measurements.

Duty observations start after a 3 ms settling delay and reperiod observations after 5 ms.
They establish settled behavior, but do not distinguish immediate from overflow latching
after SET_UPDATE. The 100 ms edge-count windows and microsecond timer cannot resolve
RANGE versus RANGE+1 at these periods (one tick is 20 ns). The longer measurement
below resolves that difference for PWM0 trailing-edge mode.

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
# use rp1_hal::{clocks::Clocks, fugit::RateExtU32, gpio::Pin, pwm::{Config, DutyTicks, Error, Token}};
fn output(token: Token<0>, pin: Pin<12>, clocks: &Clocks) -> Result<(), Error> {
    let mut channels = token.init(clocks)?;
    let actual = channels.channel0.configure(Config::from_frequency(1.kHz()))?;
    let duty = DutyTicks::new((actual.cycle_ticks() / 4) as u32);
    let output = channels.channel0.output(pin);
    let mut output = output.enable().map_err(|e| *e.error())?;
    output.try_set_duty_ticks(duty)?;
    let output = output.disable().map_err(|e| *e.error())?;
    let (channel, _pin) = output.try_free().map_err(|e| *e.error())?;
    channels.channel0 = channel;
    let _token = channels.free().map_err(|e| *e.error())?;
    Ok(())
}
```

### Period and duty

**STATIC · 2026-11-27.** Initialization selects XOSC / 1 = 50 MHz, disables all channels,
DMA requests and interrupts, and programs a 65,535-tick cycle, duty zero and phase zero.
While disabled, use `Config::from_frequency(HertzU32)`, `Config::from_period(NanosDurationU64)`
or exact `Config::from_ticks` for cycle lengths. Frequency accepts 1–50,000,000 Hz,
period accepts 20–85,899,345,900 ns, and exact cycle lengths accept 1..=u32::MAX.
Configuration rounds to the nearest cycle length with ties toward the longer period,
then writes RANGE = cycle ticks − 1 and resets duty to zero. The largest admitted RANGE
is u32::MAX − 1 so the full-high compare still fits in the 32-bit DUTY register. Invalid
requests, a stopped or changed PWM clock, and a channel still enabled in hardware all
fail before any configuration write. Configure before routing to defer pin drive until
configuration succeeds. Normal trailing-edge polarity is fixed.

`Actual::range_ticks()` reports the programmed counter limit. `Actual::cycle_ticks()`
reports RANGE+1, the denominator of the exact ratio `clock().raw() / cycle_ticks()`.
`period()` reports the resulting nanoseconds and `frequency_hz()` rounds to the nearest
whole Hz with ties up. Sub-half-Hz values round to zero.

**HW · 2026-11-27.** PWM0 at RANGE=2,500 took 10,004,000 µs across 200,000 cycles
against the RP1 timer. The shared XOSC clock readbacks imply 2,501 PWM clock ticks per
cycle. **DOC · checked 2026-11-27.** The RP1 datasheet §3.4.4.1 describes counting from
zero to RANGE and driving high while COUNT < DUTY. The regdb describes RANGE as the
counter limit and DUTY as the compare.

`try_set_duty_ticks(DutyTicks)` accepts 0..=cycle_ticks in either state. Zero drives
low and cycle_ticks drives continuously high. Under the declared, held PWM clock the
setter checks a single 32-bit bound and issues two ordered MMIO writes. The Cortex-M3
disassembly gate checks the `motor-api` release build for one load from the channel,
the exact store addresses, and the absence of calls, loops, sleep, allocation, logging,
and division. The stores use the normal RP1 MMIO barriers, so bus service time
contributes to call latency. The embedded-hal `SetDutyCycle` checks the clock and
scales normalized duty by cycle_ticks / 65535, rounding the result down. Both
endpoints are exact. Small periods have coarse resolution.

Use `check_clock()` in a slow supervisor loop, for example at 10–100 Hz. It
returns `Error::Precondition` when the clock is disabled or its selected rate has
changed. The ISR setter can still return success for an in-range compare after
loss of the clock hold. With the clock stopped, writes may remain pending and the
output may hold its last level. At a different rate, the waveform may run at the
wrong frequency. These fault behaviors are HW unverified.

### State transitions and release

**STATIC · 2026-11-27.** `enable` consumes `Disabled` and returns `Running`.
`disable` consumes `Running` and returns `Disabled`. Both check the PWM clock and return
the original owner on error via `TransitionError::into_driver()`. Period configuration,
pin routing and mutable pad access require `Disabled`.

Enable/disable use peripheral SET/CLR aliases. Enable, disable, period and duty changes
request SET_UPDATE (GLOBAL_CTRL bit 31, self-clearing), following the Linux driver.
DUTY and RANGE have separate write strobes. The RP1 datasheet (Table 26) places their
running-channel updates at counter overflow. The setter returns after issuing the writes.
Waveform application occurs in hardware. Its precise timing is **HW 未検証**.
The driver returns immediately because waiting for a period boundary could take
about 86 s at the maximum RANGE.

SET_UPDATE applies to the whole PWM block. Each `try_set_duty_ticks` call issues
one DUTY write and one SET_UPDATE strobe. A period boundary between two calls can
expose the new duty on one channel and the old duty on another for one period.
Synchronized multi-channel commits are deferred to the M2 `Group` API.

Stop a running output with `disable` before `try_free`. The latter clears MODE
(0 = generate 0) and returns the channel and pin. The pad stays muxed and is expected
to read low (DOC/STATIC, HW 未検証). `enable` restores the trailing-edge CTRL value.
`Channels::free` and `Output::try_free` check the clock first and return the owner on
`Precondition` without issuing any writes. `abort` is available on an output in
either state, a running channel, or the whole block. It issues the stop without a
clock check. With the clock stopped, the disable may fail to latch. The pin
returned by `Output::abort` stays muxed to PWM until the caller reconfigures it,
for example as an input.
`Token::init` starts the clock and disables all channels. Its bounded reset wait may
return `Timeout` with partial initialization and the original token in `InitError`.
Dropping a running output leaves the hardware configured. Fault handling that requires
external de-energization needs a separate EN/BRAKE or watchdog path.

STATIC: the declaration reserves one block ID (48 or 49), one reset and one clock, regardless
of the number of output pins. It disables the Linux PWM node, excludes its reset from Linux
bring-up, and holds its clock against SCMI rate/disable requests. Boot checks reject
or disable Linux consumers of that owned block, including pinmux users, under the selected
conflict policy. Linux and the other core cannot declare another channel of the same block.
Channels can move into an interrupt slot for their owner core. They implement neither
`Sync` nor the RP1 link's `Message` trait.
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
| FIFO_CTRL +0x04 | written zero: DREQ_EN[31] off, FLUSH[5] not asserted (the FIFO is unused) | same, `fifo_ctrl`; no FIFO data access |
| CHAN0..3_CTRL +0x14 + C*0x10 | MODE=1, FIFO_POP_MASK[8]=1; BIND/USEFIFO/SDM/INVERT clear | same, `chanN_ctrl` |
| CHAN0..3_RANGE +0x18 + C*0x10 | unsigned 32-bit counter limit | same, `chanN_range` |
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
uses the same channel setup and writes the rounded period directly to RANGE in
`rpi-6.12.y`. The HAL follows the measured RANGE+1 convention above and has no dependency
on Linux kernel internal APIs.

## Host checks and example

STATIC: `cargo test --workspace` includes fake-MMIO reset/clock/update failures, full-scale
arithmetic, pin-route generation, compile-fail pin reuse and cross-core channel transfer,
whole-block declaration conflicts, overlay reservations and held SCMI clocks.

The diagnostic example is `lab/pwm-loopback` (scoped HW results above). Build without deployment:

```sh
CARGO_BUILD_JOBS=1 cargo rp1 build -p pwm-loopback --target thumbv7m-none-eabi --release
```

Package it with `FIRMWARE_PKG=pwm-loopback lab/build-rp1boot.sh`, supplying the required
input/output environment variables documented in that script and `RP1_ISRAM_PLAN` from
the two-stage build. The default package remains `shm-firmware`; `FIRMWARE_FEATURES`
selects feature arguments, with `SHM_FIRMWARE_FEATURES` retained as a compatibility fallback.

On a memory-limited shared host, wait for any other cargo process before each invocation.
The example keeps proc0 servicing SCMI/endpoint events while proc1 counts loopback samples.
It exercises two channels, five duties, a disabled-only period change, pin return and whole
block token reinitialization. It prints observations for the hardware verifier.

## HW acceptance and extended checks

STATIC: the loopback firmware reports GPIO22 pull-up, GPIO22 pull-down and GPIO23's
default pull-down before enabling PWM. Run `python3 lab/verify-pwm.py --uart0 CAPTURE`
to check the three readbacks against the boot nonce and verify the waveform sequence.

**HYP · 2026-11-27.** Build the same package with `--features motor-api` for the M6 campaign.
ALARM1 schedules 100,000 native updates at 1 ms intervals across five duty plateaus
in the order 25%, 100%, 50%, 0%, 75%. It then alternates 25% and 75% at 1,001 µs
intervals to sweep through a 1,000 µs PWM period.
GPIO22 and GPIO23 are inputs on the 1 kΩ loopbacks. Use
`python3 lab/verify-motor-api.py --uart0 CAPTURE` to check each 25%, 50%, and 75% cycle
against 990–1010 µs and ±1 percentage point. The 0% and 100% windows each need
50,000 samples or more with gaps of 10 µs or less and no opposite levels. The
sweep needs 1,000 or more complete cycles at 990–1010 µs, at least 400 high times
in each of 250 ±10 µs and 750 ±10 µs, and zero outside these bands.
The verifier also checks 100 rejected configurations, 100 rejected native compares,
and a 2 µs latency bound with a 1 µs timer quantization allowance.

With ALARM1 off, the fixture counts 200,000 cycles at RANGE=2,500. The RP1
[datasheet](https://datasheets.raspberrypi.com/rp1/rp1-peripherals.pdf) §3.8 describes
the timer tick source as `clk_ref` at nominal XOSC frequency. The fixture records
the running timer's divider of 50 and PWM0's XOSC / 1 configuration alongside the
span and a RANGE readback of 2,500. The RANGE+1 gate expects
10,004,000 ±100 µs on that timebase. If the common clock basis cannot be established,
the convention result is unverified. Absolute frequency, exact latch timing and
pulses shorter than the GPIO sampling interval require separate instrumentation.

The results above cover the two-channel loopback subset. The remaining checks below are
an extended plan, not a claim that all routes, periods or PWM1 have been verified.

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
   then reports the final A=1 kHz/B=500 Hz 50% outputs in `PWM_REINIT`
   followed by `PWM_DONE reinitialized=1`. In each cold boot, run `lab/check-core-status.sh pwm "$NONCE"` from Linux
   after `PWM_DONE`, with the nonce from `PWM_BOOT` and the
   [matching Linux status client](../lab/README.md#linux-status-client) staged.
   Require both cores to report `Running` and the matching `LAB_FINAL_END` to report
   `result=PASS`. Keep the waveform measurements and UART records for both boots.
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
