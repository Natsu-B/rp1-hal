# ADC

**STATIC · 2026-11-28.** ADC0 does 12-bit one-shot conversions. The FIFO
holds one result per conversion, with DMA and free-running mode disabled.

| ADC0 declaration | Wait mode | Immediate API |
|---|---|---|
| ADC and `wait_timer` on the same core | `blocking` (ADC_FIFO IRQ52, alarm, WFE), or explicit polling | `try_start`, `try_sample`, `try_cancel` |
| ADC without a same-core `wait_timer`, or `custom_adc_handler = true` | Explicit polling | `try_start`, `try_sample`, `try_cancel` |

```toml
[package.metadata.rp1.proc0]
adc = {}
wait_timer = "ALARM1"
```

```rust,ignore
use rp1_hal::adc::TEMPERATURE;
let mut adc = p.adc.init(&sys.clocks)?;
let mut adc = adc.blocking(&mut p.wait_timer);
let raw = adc.read(TEMPERATURE)?;
let mc = adc.read_temperature()?;
```

Pass an absolute `Deadline` to `read_until`, or use `WaitConfig` to override
the default 1 ms timeout. Both the IRQ and explicit polling entry points
require unmasked thread mode. A caller without a wait alarm can use
`adc.read_polling(TEMPERATURE)` or `adc.read_temperature_polling()`.

Enable `rp1-hal/eh02` (the `eh02-adc` feature still works) for embedded-hal
0.2 `OneShot`. Use `let mut input = TEMPERATURE`. The first trait call starts
a conversion and returns `WouldBlock`. Subsequent calls for the same channel
return `WouldBlock` until the result is ready. A different channel gets
`Busy` while the conversion is owned. External AIN0–3 require board-issued
`Input<N>` tokens. The CM5 declaration provides the internal sensor.

## Ownership

STATIC: declaring `adc` assigns the block, clock and reset.

- The generated resource has one declared core owner, and the NOTE records
  OWNERSHIP block 64. A same-core wait timer selects `rp1::AdcParts` and the
  generated ISRAM `ADC_FIFO` handler. It owns and masks the interrupt source.
  A declaration without that timer, or with `custom_adc_handler = true`,
  exposes the raw `p.irq.ADC_FIFO` binding. A custom handler must be linked and
  service its own source.
- The overlay disables Linux's `rp1_adc` (the stock DT enables it with the `rp1-adc` hwmon
  driver). The final-DTB check rejects a later overlay that turns it back on.
- The ADC reset (RESETS CTRL0 bit 0) leaves the set that rp1-hal releases for Linux.
  `linux_blocks::bring_up` no longer touches the clk_adc generator. The SCMI server *holds*
  the `adc` clock (ID 7): it reports the generator's actual rate and state, denies
  `CLOCK_RATE_SET` to any other rate, and treats `CLOCK_CONFIG_SET` as a vote only.
  Linux's `clk_disable_unused` therefore cannot stop it.

Without the declaration, the ADC stays Linux's, as before.

Either core may own the ADC. On proc1, `init` needs proc0's clocks.
`Clocks::wait_from_proc0` returns the value proc0's `platform::init` published
(see [Clocks on proc1](#clocks-on-proc1)).

## Clocks on proc1

STATIC: only proc0 holds the platform token, so only proc0's `platform::init` creates a
`Clocks`. When its clock step (step 2: clk_sys and the blocks Linux owns) succeeds, and only
then, init stores the value in a rp1-hal-private `.shared` cell (frequencies first, then a
ready word with a release store). A later step that fails (log, warm start, quiesce,
endpoint, SYSCFG) does not withdraw it: a published value means the clocks are up, not that
the whole platform init succeeded. A failing clock step leaves the ready word 0.
The value is published once per image load: `init` consumes the one `platform::Token`, and
`.shared` is loaded with the image (ready word 0).
`Clocks::wait_from_proc0(&core, timeout)` reads the ready word with an acquire load,
checks the published rates and returns a copy. It retries `Error::NotReady` against the
64-bit 1 MHz timer until `timeout`. Pass `0.micros()` to try once.
A stale clock tree returns `Error::Precondition` immediately.
Only loads and stores are used (no LDREX/STREX in shared SRAM). Safe code cannot build a
`Clocks` from arbitrary numbers, so a proc1 `Clocks` still means "proc0's init ran".
Launching proc1 after init (the usual order) makes the wait return at once.


## Sequence

**STATIC · 2026-11-28.** CS and clock setup follow the Linux ADC driver.
The PAC also describes FCS, FIFO and INTR/INTE/INTF/INTS.

1. `init` starts clk_adc at XOSC / 1 = 50 MHz, releases reset (DONE stays
   zero, HW), powers the ADC and temperature sensor, and confirms READY
   and a clean FIFO. `InitFailure` retains the parts. Use `try_recover` if
   READY or FIFO cleanup was not confirmed.
2. Each start validates idle and removes stale FIFO data. It programs
   AINSEL and issues START_ONCE. The managed path enables the FIFO
   threshold interrupt before START_ONCE. The ISR masks the asserted source
   before signaling WFE. Thread mode checks READY and takes one FIFO word.
   FIFO overflow is `Overrun`; a sample error is `Conversion`.
3. Timeout and `try_cancel` retire the interrupt and discard the result.
   A pending one-shot can still complete. `Recovering` keeps ownership
   until `try_cancel` confirms READY, drains the FIFO within its fixed
   depth and clears sticky flags. `try_free` requires idle and carries the
   driver in its error when release is unsafe. The clock and reset stay
   configured across release.
4. Temperature uses the Linux scale. mV = 3300 × raw / 4095 (stock DT
   references the 3.3 V supply), and T = 27 − (mV − 706) / 1.721 °C.

## Hardware

Historical polling measurements, HW (2026-10-29, cold boot ×3, lab `shm-firmware` declaring `ADC` on proc0):

- `init` succeeds. The first burst runs before Linux starts.
- After Linux has booted, the burst is repeated on request through the lab link word. By
  then `clk: Disabling unused clocks` has run, and Linux has sent `CLOCK_CONFIG_SET(adc, off)`.
  The SCMI server answered success as a vote only. clk_adc stayed at XOSC / 1, enabled
  (CTRL 0x1000_0800, DIV 1), and every request still converted.
- Linux sees `rp1_adc` disabled and binds no rp1-adc hwmon. Its `clk_summary` shows `adc`
  at 50 MHz, hardware-enabled.
- Repeated reads are stable. Over 1000 conversions per input, the temperature sensor spans
  2–3 LSB, and an input near full scale (input 1, ≈ 4086, HYP tied to 3.3 V) spans 3 LSB.
  Inputs 2 and 3 wander by hundreds of LSB (HYP: floating).
- Temperature: about 68–69 °C right after the RP1 firmware starts, and about 76 °C once
  Linux is running (raw ≈ 772). BCM2712's `cpu-thermal` read 70.5 °C at the same time.
  The sensor law is uncalibrated (DOC formula), so the absolute accuracy is not established.
- One `read()` call takes 7.1 µs on proc0 at boot (1000 calls in 7.08–7.11 ms), and about
  34.4 µs while Linux is running. The call includes the register accesses around the
  conversion; the conversion itself is not isolated (DOC/HYP: 96 clk_adc cycles ≈ 1.9 µs).
  HYP for the slowdown: APB access latency on proc0 rises once Linux and proc1 are active.
  It was not investigated further.
- The first conversion after TS_EN reads about 5–7 LSB higher than the steady value.

The board wiring of inputs 0..=3 on the CM5 IO board is not documented here. Their values
are recorded but have no known meaning (HYP: unconnected or tied).
