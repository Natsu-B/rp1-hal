# ESP32 I²C lab peer

Send line-oriented commands at 115200 baud. `gpio_read` or `gpio_read none`
returns one `gpio_read PIN=LEVEL ...` line followed by `OK`. The fixed set of
inputs is GPIO4, 23, 25, 26, 27, 32, 33, 34, and 35. `gpio_read down` temporarily
applies weak internal pull-downs on GPIO4 through GPIO33 in that set. GPIO34 and
GPIO35 have no internal pulls. The selected inputs return to no pull after
each read.

`gpio_read none scan` and `gpio_read down scan` first return
`GPIO_SCAN_READY`, then capture 256 samples at 1 ms intervals. The
`gpio_scan interval_us=1000 max_late_us=... samples=...` line contains one
three-digit hex mask per sample, in increasing time order. Bit 0 is GPIO4,
with the remaining bits following the input order above. The maximum sample
lateness is measured against the nominal schedule in microseconds. The
response ends with `OK`. Use the readiness line to trigger a short pulse
while the ESP32 captures locally.

While an edge capture is armed, all `gpio_read` forms return
`ERR observer_busy`. See the [observer lab](../edge-observer/README.md) for
capture and pin-state commands.
