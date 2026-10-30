# ADC

STATIC: `rp1_hal::adc` does one-shot 12-bit conversions by polling (no FIFO, DMA,
free-running mode or interrupt). Inputs 0..=3 are the external inputs and 4 is the
temperature sensor.

```toml
[package.metadata.rp1.proc0]
peripherals = ["ADC"]
```

```rust,ignore
let mut adc = p.ADC.init(&clocks)?;          // after clocks::Authority::init
let raw: u16 = adc.read(0)?;                 // input 0, 0..=0xfff
let mc: i32 = adc.read_temperature()?;       // millidegrees Celsius

// Declared under [proc1] instead: proc1 takes the Clocks proc0's Authority::init published.
let clocks = rp1_hal::clocks::Clocks::wait_from_proc0(&core, 10_000)?;   // core: Proc1
let mut adc = p.ADC.init(&clocks)?;

// embedded-hal 0.2 (1.0 has no ADC trait):
use embedded_hal_0_2::adc::OneShot;
let t = nb::block!(OneShot::read(&mut adc, &mut rp1_hal::adc::TemperatureSensor {}))?;
```

## Ownership

STATIC: declaring `ADC` does three things.

- The generated `Token` has one declared core owner, and the note records OWNERSHIP block 64.
- The overlay disables Linux's `rp1_adc` (the stock DT enables it with the `rp1-adc` hwmon
  driver). The final-DTB check rejects a later overlay that turns it back on.
- The ADC reset (RESETS CTRL0 bit 0) leaves the set that rp1-hal releases for Linux.
  `linux_blocks::bring_up` no longer touches the clk_adc generator. The SCMI server *holds*
  the `adc` clock (ID 7): it reports the generator's actual rate and state, denies
  `CLOCK_RATE_SET` to any other rate, and treats `CLOCK_CONFIG_SET` as a vote only.
  Linux's `clk_disable_unused` therefore cannot stop it.

Without the declaration, the ADC stays Linux's, as before.

Either core may own the ADC. On proc1, `init` needs proc0's clocks: `Clocks::from_proc0` /
`Clocks::wait_from_proc0` return the value proc0's `clocks::Authority::init` published
(see [Clocks on proc1](#clocks-on-proc1)).

## Clocks on proc1

STATIC: only proc0 holds the clock authority, so only proc0's `Authority::init` creates a
`Clocks`. Just before it returns `Ok`, and only then, init stores the same value in a
rp1-hal-private `.shared` cell (frequencies first, then a ready word with a release store).
The value is published once per image load: `init` consumes the one `Authority`, and
`.shared` is loaded with the image (ready word 0). A failing init leaves the ready word 0.
`Clocks::from_proc0(&Proc1)` reads the ready word with an acquire load and returns that
copy. It returns `Error::NotReady` before init has succeeded, and `Error::Precondition` if
a value was published but the clock tree no longer matches it (as the drivers do).
`Clocks::wait_from_proc0(&Proc1, timeout_us)` retries only `NotReady`, against the 64-bit
1 MHz timer, trying at least once; it returns `Error::NotReady` once `timeout_us` has
elapsed (any `u32` value ends) and `Precondition` at once.
Only loads and stores are used (no LDREX/STREX in shared SRAM). Safe code cannot build a
`Clocks` from arbitrary numbers, so a proc1 `Clocks` still means "proc0's init ran".
Launching proc1 after init (the usual order) makes the wait return at once.


## Sequence

STATIC: the register contract and sequence come from the Linux RP1 ADC hwmon driver. The
registers are CS, RESULT, FCS, FIFO, DIV and INTR/INTE/INTF/INTS (rp1-pac `adc`). The PAC
write APIs are raw/unsafe only until the write semantics are verified.

1. `init` starts clk_adc at XOSC / 1 = 50 MHz. This is the rate Linux requests, and the SCMI
   table's `adc` entry has the same parent and maximum. Then it releases the reset (DONE is
   not waited for: it stays 0 even under the stock firmware, HW), writes INTE = 0 and
   CS = EN | ERR_STICKY, and waits for READY.
2. `read(ain)` selects AINSEL through the CLR/SET aliases (also clearing START_MANY and
   RROBIN), waits for READY, sets START_ONCE, and waits for READY again. It returns
   `Error::Conversion` if CS.ERR is set, and otherwise RESULT[11:0]. Reading input 4 powers
   the temperature sensor (TS_EN) first and leaves it on, as Linux does.
3. `read_temperature` converts as Linux does: mV = 3300 × raw / 4095 (the board's
   `vref-supply` is the 3.3 V rail, DOC), and T = 27 − (mV − 706) / 1.721 °C.

## Hardware

HW (2026-10-29, cold boot ×3, lab `shm-firmware` declaring `ADC` on proc0):

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
