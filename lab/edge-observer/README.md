# ESP32 edge observer lab

`edge-observer` produces bounded RP1 GPIO, PWM0, timer alarm and SPI0 stimuli.
It uses an RP1 link-page mailbox at offset `0x400`. Word 0 is the Linux
request, with a nonzero nonce and a mode in its low byte. Word 1 is the ready
cookie `0x45444745`. RP1 publishes the result flag, payload word count, first
alarm deadline and payload in words 3 onward, then acknowledges the request in
word 2. Use this image with the generated ownership device tree.

| Mode | Stimulus |
| --- | --- |
| 1 | Two groups of eight simultaneous GPIO12/13 transitions, 2 s apart |
| 2, 3 | PWM0 channels 0/1 at 1/10 kHz and 50% duty, about 130 cycles |
| 4 | GPIO13 toggled by 256 successive ALARM1 deadlines 1 ms apart, with marker groups before and after |
| 5 | One SPI0 mode-0 byte at 1 MHz, with a 4 µs HAL delay on each side, RP1 GPIO8 as software CS0 |
| 6 | Simultaneous RP1 GPIO8/11 transitions for the SPI observer's channel skew |
| 7 | Route GPIO12/13 to RIO and hold them low |
| 8, 9 | Prepare disabled PWM0 outputs for mode 2 or 3 |
| 10, 11 | Prepare GPIO8/11 low for mode 6, then restore the SPI pin routing |
| 12 | Release GPIO12/13 to inputs |

GPIO22/23 remain inputs. Run setup and cleanup modes outside observer capture
windows. Use mode 7 before changing PWM configuration or returning to markers.
Prepare SPI skew with mode 10, capture mode 6, then restore with mode 11.
Use mode 12 to release GPIO12/13 when finished. For modes 1, 4 and 6 each
marker payload is a triple
of raw timer counts before and after a GPIO write and pad readback, followed
by the GPIO input mask. Mode 4 appends 256 triples for GPIO13. Modes 2 and 3
return period ticks, duty ticks, PWM clock in Hz and run duration in µs.
Mode 5 returns the configured SPI rate in Hz.

## ESP32 control and analysis

Use firmware `0.8.1-rp1-hal-edge-observer` or later from `lab/esp32-i2c-peer`.
Hold the bench lock, verify the CM5 is powered and use an exclusive serial
connection. Install PySerial and set `CM5_ESP32_PORT` to the control port,
or pass `--port` before the serial subcommand.

```sh
python3 lab/edge-observer/observe.py caps
python3 lab/edge-observer/observe.py arm 42 34,4 1024
# Trigger the RP1 stimulus and wait for completion before dumping.
python3 lab/edge-observer/observe.py dump 42
python3 lab/edge-observer/observe.py analyze trials.jsonl
```

The ESP32 protocol adds `obs_caps`, `obs_arm <nonce> <profile> <limit>`,
`obs_status`, `obs_stop <nonce>`, `obs_read <nonce> <offset> <count>`,
`obs_abort <nonce>` to discard a capture, and `obs_pins` to read pin state.

A profile is one to three distinct pins from 34, 4, 27, 26, 32, 23, 33 or
`spi0` (CS0 on 26 and SCLK on 33). The observer configures inputs without
pulls and rejects `gpio_read` commands while capture is armed. Arm with a
fresh, nonzero nonce after the lines settle. Stop after the stimulus ends and
dump the capture before arming again. The observer expires at the next
serial loop iteration after 5 s.

The ring retains the last `limit` recorded edges. After 2049 serviced edges
the firmware masks capture interrupts to bound CPU load. The reported
`dropped` count is a lower bound and covers serviced records evicted from
the ring. PCNT totals are reported after stop.

MCPWM latches the edge against an 80 MHz APB counter
(12.5 ns nominal tick). Interrupt latency affects whether an edge is
retained. PCNT independently counts both polarities and can reveal
overwritten latches. For one SPI frame, MCPWM timestamps CS and latches the
last falling SCLK edge. RMT uses the same APB clock and captures the relative
widths needed to recover the first rising SCLK. A frame that exceeds the
single 64-item RMT block or spans separate RMT records cannot be reduced as
one burst. Reject captures with drops, mismatched counts, saturation, or
expired windows. [Espressif MCPWM capture](https://docs.espressif.com/projects/esp-idf/en/v4.4/esp32/api-reference/peripherals/mcpwm.html)

JSONL analysis input includes a `label`, the object returned by `dump`,
`rp1` containing the stimulus payload, and `deadline` for the alarm. Use
labels `calibration`, `pwm_1000`, `pwm_10000`, `jitter`, `spi_skew`,
and `spi` for 12 individual frames. Preserve the raw serial and RP1
mailbox records alongside the JSONL. The fit compares the two oscillators.
Absolute frequency accuracy requires an external calibrated reference.
Pass `--calibration-and-pwm` to analyze just the calibration and 1 kHz PWM row.
