#!/bin/sh
# Linux side of the interrupt sweep (lab/irq-sweep). The sweep itself is over before Linux has
# a PCIe link to RP1; this script only records that the same boot reached Linux in order:
# the firmware's boot value (ties this console to its UART0 lines) and the swept blocks'
# Linux nodes, which must exist and be disabled. Ends with exactly one
# `SWEEP_CHECK_RESULT=PASS|FAIL`, then `SWEEP_CHECK_END`. The host decides with
# lab/verify-irq.py --sweep.
export PATH=/bin
busybox --install -s /bin
FAILED=
fail() { echo "CHECK_FAIL $1"; FAILED="$FAILED $1"; }
# dt_disabled NAME NODE: RP1 owns the block, so Linux's node must exist and be disabled
# (a missing status property means okay).
dt_disabled() {
    found=
    for path in $(find /sys/firmware/devicetree/base -name "$2"); do
        st=$(cat "$path/status" 2>/dev/null)
        echo "$1_DT=$path status=${st:-none}"
        found=1
        [ "$st" = disabled ] || fail "$1_dt"
    done
    [ -n "$found" ] || { echo "$1_DT=missing"; fail "$1_dt"; }
}
echo SWEEP_CHECK_BEGIN
uname -a
# The firmware keeps its boot value in the link page (+0x28).
echo IRQ_NONCE=$(busybox devmem 0x1f0040c028 32)
dt_disabled I2C0 i2c@70000
dt_disabled I2C2 i2c@78000
dt_disabled I2C3 i2c@7c000
dt_disabled SPI1 spi@54000
dt_disabled SPI3 spi@5c000
dt_disabled PWM1 pwm@9c000
dt_disabled ADC adc@c8000
grep rp1 /proc/interrupts
dmesg | grep -E 'SError|abort|fault|WARN'
if [ -z "$FAILED" ]; then
    echo SWEEP_CHECK_RESULT=PASS
else
    echo "SWEEP_CHECK_RESULT=FAIL$FAILED"
fi
echo SWEEP_CHECK_END
[ -z "$FAILED" ]
