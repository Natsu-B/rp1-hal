# What Linux loses

This table covers CM5 with `clock.authority = "rp1"`. The declarations decide which blocks
and pins Linux keeps. Under this clock policy, `rp1boot` switches supported Linux consumers to
SCMI clocks served by rp1-hal. The stock RP1 mailbox service and clock provider are disabled.

**HW** means an operation was observed on hardware. **STATIC** means a code or device-tree
result. **NOT_CHECKED** means a function was outside the test. **HYP** marks a possible
explanation or path that still needs a test.

## Evidence runs

Stock 2712 means Linux `6.12.75+rpt-rpi-2712` (Debian `1:6.12.75-1+rpt1`). The runs below
are **HW** observations. Linux probe, bus scan, transfer and waveform results have different scopes.

| Run | Date | RP1 firmware | Linux kernel | Root | Tested |
| --- | --- | --- | --- | --- | --- |
| P0-2 | 2026-11-17 | Stock vs `bd6859d` | Stock 2712 | NFS | Linux device comparison |
| P0-3 | 2026-11-17 | `91292ee` | Stock 2712 | NFS | Linux device retest after the IRQ gate fix |
| P0-4 | 2026-11-17 | `91292ee` | Stock 2712 | SD (ext4) | Boot and SCMI without a Linux initramfs |
| realos v2 | 2026-11-24 | `fc75c24` | Stock 2712 | NFS | Linux device probes, including combined SPI1/PWM0 overlay |
| P1-9 B | 2026-11-27 | `e961f78` | Stock 2712 | NFS | Linux device probes and 1,000 template polling RPC replies |
| P1-9 C | 2026-11-27 | `e961f78` | `6.12.75-v8-16k` | Initramfs | RP1 PWM loopback and Linux UART1 |
| V7 B | 2026-11-27 | `aa6fe43` | Stock 2712 | NFS | RP1 peripheral examples |
| V7 C | 2026-11-27 | `c5ec941` | Stock 2712 | NFS | IRQ RPC, named polling channel and both-core health |

V7 B and C used TFTP and EEPROM bootloader `9cd61c53` (2026-04-14).

| Function | Stock RP1 firmware | Kept by Linux with rp1-hal | Assigned to RP1 or limited by the implementation | Evidence and conditions |
| --- | --- | --- | --- | --- |
| I2C2 and I2C6 | Full bus scans completed with IRQs. | Full scans completed after the IRQ gate fix. P1-9 B short scans each raised the controller IRQ count by 16. I2C6 was Linux bus 10. | Linux loses a declared bus. V7 B's RP1-owned I2C1 peer exchange is a separate test. The Linux scan leaves device-specific transactions **NOT_CHECKED**. | **HW · 2026-11-17** · P0-2 `bd6859d`: probes succeeded, short scans timed out, IRQ delta 0. P0-3 `91292ee`: both scans completed. **HW · 2026-11-27** · P1-9 B `e961f78`, GPIO4/5 and GPIO38/39. V7 B `aa6fe43`, GPIO2/3 with a known I2C peer. [IRQ gates](../rp1-hal/src/linux_blocks.rs) · [I2C example](../examples/peripherals/README.md) |
| SPI1 | 16 B controller IRQ and 4,096 B DMA transfer completed. | 16 B transfer completed after the fix. P1-9 B `spidev1.0` completed both lengths with controller IRQ +2 and shared DMA IRQ +2 respectively. | Linux loses a declared controller. V7 B tested RP1-owned SPI0 internal loopback. External SPI device data integrity is **NOT_CHECKED**. | **HW · 2026-11-17** · P0-2 `bd6859d`: 16 B timeout with IRQ 0, 4,096 B still completed via DMA. P0-3 `91292ee`: 16 B IRQ +2. **HW · 2026-11-27** · P1-9 B `e961f78`, SPI1 on GPIO18–21. V7 B `aa6fe43`, SPI0 loopback. [IRQ gates](../rp1-hal/src/linux_blocks.rs) · [Linux probe](../lab/realos/check-linux-devices.sh) |
| PWM0 | Channel 0 register and pin function checked at GPIO12. | Channel 0 configured and read back via sysfs, with PWM routed to GPIO12 alongside SPI1. | Linux loses a declared PWM block. V7 B measured the RP1-owned PWM0 ch0 waveform with a GPIO12 → GPIO22 loopback. | **HW · 2026-11-17** · P0-2 stock and `bd6859d`. **HW · 2026-11-24/27** · realos v2 `fc75c24` and P1-9 B `e961f78`, sysfs. **HW · 2026-11-27** · V7 B `aa6fe43`, 1 kΩ loopback. [Combined overlay](../lab/realos/overlays/linux-spi1-pwm0.dts) · [PWM example](../examples/peripherals/README.md) |
| Fan on PWM1 | Cooling state and PWM read back. Tach read 0. | Cooling state and PWM changed. Tach read 0 and the realos v2 fan probe failed. | Tach function is **NOT_CHECKED** with a known tach signal. **HYP** · the bench may lack a routed tach signal. | **HW · 2026-11-17/24** · P0-2 stock and `bd6859d`, realos v2 `fc75c24`. PWM1 ch3, GPIO45. [Fan probe](../lab/realos/check-linux-devices.sh) |
| Ethernet | Ping 20/20. | Peer ping completed. V7 B and C also mounted their NFS roots over this link. | With RP1 authority, Linux Ethernet relies on the SCMI provider for its clocks. | **HW · 2026-11-17/24/27** · P0-2 stock and `bd6859d`, P0-3 `91292ee`, realos v2 `fc75c24`, P1-9 B `e961f78`, V7 B/C `aa6fe43`/`c5ec941`. [Clock assignments](../rp1-build/src/overlay.rs) |
| USBHOST1 | A Logitech receiver enumerated and completed a control transfer. | The receiver enumerated and completed a control transfer. P0-2 also observed hotplug. | Results cover this receiver and host. Other USB classes and workloads are **NOT_CHECKED** here. | **HW · 2026-11-17/24/27** · P0-2 stock and `bd6859d`, P0-3 `91292ee`, realos v2 `fc75c24`, P1-9 B `e961f78`; receiver 046d:c539 on USBHOST1. [USB probe](../lab/realos/check-linux-devices.sh) |
| USBHOST0 | Root hub present. | Root hub present. | External device enumeration and transfers are **NOT_CHECKED** on USBHOST0. | **HW · 2026-11-17** · P0-2 stock and `bd6859d`, no attached USBHOST0 device. [Linux block setup](../rp1-hal/src/linux_blocks.rs) |
| UART, ADC and declared peripheral blocks | Linux can bind enabled stock nodes. | P1-9 C checked Linux-owned UART1 TX and RX with a 3.3 V peer. RP1 ADC has a clock mapping when kept. Linux ADC use is **NOT_CHECKED** here. | The generated overlay disables the declared UART, ADC, PWM, I2C and SPI nodes and reserves their pins. V7 B tested RP1-owned UART1 echo and internal temperature conversion. | **STATIC · 2026-11-27** · `c5ec941`. **HW · 2026-11-27** · P1-9 C lab PWM build, Linux UART1 on GPIO0/1; V7 B `aa6fe43`, RP1 UART1 and ADC. [Overlay generation](../rp1-build/src/overlay.rs) · [Peripheral examples](../examples/peripherals/README.md) |
| GPIO, gpiomem and stock overlays | Stock pinmux and gpiomem apply. Stock `spi1-1cs` and `pwm` configurations were exercised. | Current checks understand supported legacy numeric pinmux forms. Stock `spi1-1cs` and `pwm` both default to GPIO18. P1-9 B used `dtoverlay=pwm,pin=12,func=4` alongside `spi1-1cs`. realos v2 used the combined overlay. | Linux GPIO excludes reserved pins. The default disables RP1 gpiomem when pins are reserved. Conflicting nodes are disabled by default. A protected conflict or `linux.on_conflict = "refuse"` can stop boot. | **HW · 2026-11-17** · P0-2 `bd6859d` refused old numeric overlays. **HW · 2026-11-24/27** · realos v2 `fc75c24` and P1-9 B `e961f78`, adjusted PWM routing. **STATIC · 2026-11-27** · `c5ec941`. [DT checks](../rp1-dt/src/check.rs) · [Conflict policy](../rp1-dt/src/check/resolve.rs) |
| Stock mailbox services, clocks and Linux PIO | Stock mailbox and `clk-rp1` provide the services. Linux PIO uses the stock mailbox service. | SCMI provides the clocks listed in `BOARD_SCMI_CONSUMERS` and the UART clock policy. | The stock mailbox client, `clk-rp1` and `rp1_pio` nodes are disabled under RP1 clock authority. | **STATIC · 2026-11-27** · `c5ec941`. [Stock service and clock lists](../rp1-build/src/overlay.rs) |
| Camera and display | Enabled stock paths can use RP1 video and MIPI clocks. | Sensor I2C can use a retained bus. Camera data via `rp1-cfe`/CSI and display via DSI, DPI or VEC still need clocks outside the SCMI list. | Those RP1 clock references still point to disabled `clk-rp1`, which prevents probe in this configuration. V7 camera and display function are **NOT_CHECKED**. | **STATIC · 2026-11-27** · `c5ec941`, stock CM5 DT clock references. [Clock list](../rp1-build/src/overlay.rs) |
| I2S and audio | Enabled paths use stock clocks. | The overlay maps RP1 I2S and audio output to SCMI and adjusts audio output's assigned clocks. | The firmware sets the stock I2S register bits for Linux-owned I2S. Audio streaming and codecs are **NOT_CHECKED**. | **STATIC · 2026-11-27** · `c5ec941`. [Clock assignments](../rp1-build/src/overlay.rs) · [I2S setup](../rp1-hal/src/linux_blocks.rs) |
| RP1 SDIO | Stock DT can reference the stock RP1 SDIO clocks. | RP1 SDIO clocks are outside the SCMI list. The RP1 SDIO reset bits are outside the Linux bring-up mask. | RP1 SDIO operation is **NOT_CHECKED**. The CM5 Lite boot card uses the BCM2712 controller. | **STATIC · 2026-11-27** · `c5ec941`. [Clock list](../rp1-build/src/overlay.rs) · [Linux reset mask](../rp1-hal/src/linux_blocks.rs) |

On each Linux boot, read the final device tree. The loader's disable list is NUL-separated.
It records the loader's conflict resolution. Check the affected nodes' `status` for disables
already applied by the generated overlay. Each RP1 `gpio-reserved-ranges` pair is a start pin and
count, stored as big-endian 32-bit cells. Check the path and use the RP1 GPIO controller's property.

```sh
p=/proc/device-tree/rp1-firmware/rp1,loader-disabled
if [ -r "$p" ]; then tr '\0' '\n' < "$p"; fi
find /proc/device-tree/ -name gpio-reserved-ranges -print -exec od -An -tu4 --endian=big {} \;
```

See [Getting started](getting-started.md) for the boot inputs and first run.
