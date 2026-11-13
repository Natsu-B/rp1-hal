#!/bin/sh
# Linux side of the interrupt proof (lab/irq-proof): print the firmware's nonce, ring the GPIO27
# doorbell 1000 times (proc1 answers from its interrupt handler; the firmware then prints its
# final markers on UART0), and run the SHM and doorbell regressions in the same boot.
# Strict like check-shm.sh: every step has a time limit, and the script ends with exactly one
# `IRQ_CHECK_RESULT=PASS|FAIL`, then `IRQ_CHECK_END`. The host decides with lab/verify-irq.py.
export PATH=/bin
busybox --install -s /bin
FAILED=
fail() { echo "CHECK_FAIL $1"; FAILED="$FAILED $1"; }
# limit SECONDS CMD...: run CMD, kill it after SECONDS (rc 137).
limit() {
    secs=$1; shift
    "$@" &
    pid=$!
    ( sleep "$secs"; kill -9 "$pid" 2>/dev/null ) &
    watchdog=$!
    wait "$pid"
    rc=$?
    kill "$watchdog" 2>/dev/null
    wait "$watchdog" 2>/dev/null
    return $rc
}
echo IRQ_CHECK_BEGIN
uname -a
insmod /bin/uio.ko
insmod /bin/uio_pdrv_genirq.ko of_id=generic-uio
L= D=
for x in /sys/class/uio/uio*; do
    n=$(cat "$x/name")
    echo "UIO $x $n"
    case "$n" in rp1:link) L=/dev/${x##*/};; rp1:doorbell) D=/dev/${x##*/};; esac
done
[ -n "$L" ] && [ -n "$D" ] || fail uio
# The firmware keeps its boot nonce in the link page (+0x28); it ties this console to the
# UART0 lines of the same boot.
echo IRQ_NONCE=$(busybox devmem 0x1f0040c028 32)
# The blocks whose interrupts the proof raises must be RP1's: Linux's nodes disabled.
for node in i2c@74000 spi@50000; do
    for path in $(find /sys/firmware/devicetree/base -name "$node"); do
        st=$(cat "$path/status" 2>/dev/null)
        echo "IRQ_DT=$path status=${st:-none}"
        [ "$st" = disabled ] || fail "dt_$node"
    done
done
limit 30 /bin/linux-doorbell "$L" "$D" 1000
rc=$?; echo DOORBELL_BEFORE_RC=$rc; [ $rc -eq 0 ] || fail doorbell_before
# shm-linux has a 180 s limit per request; the whole run normally takes well under a minute.
limit 600 /bin/shm-linux 1000000
rc=$?; echo SHM_RC=$rc; [ $rc -eq 0 ] || fail shm
limit 30 /bin/linux-doorbell "$L" "$D" 1000
rc=$?; echo DOORBELL_AFTER_RC=$rc; [ $rc -eq 0 ] || fail doorbell_after
grep -E 'rp1|uio' /proc/interrupts
dmesg | grep -E 'SError|abort|fault|WARN'
if [ -z "$FAILED" ]; then
    echo IRQ_CHECK_RESULT=PASS
else
    echo "IRQ_CHECK_RESULT=FAIL$FAILED"
fi
echo IRQ_CHECK_END
[ -z "$FAILED" ]
