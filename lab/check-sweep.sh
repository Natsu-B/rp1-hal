#!/bin/sh
# Linux side of the interrupt sweep (lab/irq-sweep). The sweep itself is over before Linux has
# a PCIe link to RP1; this script only records that the same boot reached Linux in order:
# the firmware's boot value (ties this console to its UART0 lines) and the swept blocks'
# Linux nodes, which must exist and be disabled. Ends with exactly one
# `SWEEP_CHECK_RESULT=PASS|FAIL`, then `SWEEP_CHECK_END`. The host decides with
# lab/verify-irq.py --sweep.
export PATH=/bin:/usr/local/bin
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
# The firmware keeps its boot value in the link page (+0x28): UART0's `IRQ_SWEEP_BOOT boot=`,
# not the `nonce=` of its marker lines. It is printed under check-irq.sh's label so that
# verify-irq.py ties the console to the boot the same way for both images.
boot=$(busybox devmem 0x1f0040c028 32) || fail nonce
echo "IRQ_NONCE=$boot"
dt_disabled I2C0 i2c@70000
dt_disabled I2C2 i2c@78000
dt_disabled I2C3 i2c@7c000
dt_disabled SPI1 spi@54000
dt_disabled SPI3 spi@5c000
dt_disabled PWM1 pwm@9c000
dt_disabled ADC adc@c8000
grep rp1 /proc/interrupts
dmesg | grep -E 'SError|abort|fault|WARN'
sh "$(dirname "$0")/check-core-status.sh" sweep "$boot" || fail core_status
if [ -z "$FAILED" ]; then
    echo SWEEP_CHECK_RESULT=PASS
else
    echo "SWEEP_CHECK_RESULT=FAIL$FAILED"
fi
echo SWEEP_CHECK_END
[ -z "$FAILED" ]
