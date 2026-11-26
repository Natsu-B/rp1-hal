#!/bin/sh
# Linux side of the interrupt proof (lab/irq-proof): print the firmware's nonce, ring the GPIO27
# doorbell 1000 times (proc1 answers from its interrupt handler; the firmware then prints its
# final markers on UART0), and run the SHM and doorbell regressions in the same boot. Then the
# notification proofs of shm-linux (down_irq.rs, up_irq.rs): Linux -> RP1 on both cores'
# doorbells, RP1 -> Linux from proc0, a writer killed at random moments, and the record-only
# controls last (they may stall a ring by design).
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
dt_disabled I2C1 i2c@74000
dt_disabled SPI0 spi@50000
limit 30 /bin/linux-doorbell "$L" "$D" 1000
rc=$?; echo DOORBELL_BEFORE_RC=$rc; [ $rc -eq 0 ] || fail doorbell_before
# shm-linux has a 180 s limit per request; the whole run normally takes well under a minute.
limit 600 /bin/shm-linux 1000000
rc=$?; echo SHM_RC=$rc; [ $rc -eq 0 ] || fail shm
limit 30 /bin/linux-doorbell "$L" "$D" 1000
rc=$?; echo DOORBELL_AFTER_RC=$rc; [ $rc -eq 0 ] || fail doorbell_after
# step NAME SECONDS ARGS...: one shm-linux proof with a time limit.
step() {
    name=$1 secs=$2; shift 2
    limit "$secs" /bin/shm-linux "$@"
    rc=$?; echo "${name}_RC=$rc"; [ $rc -eq 0 ] || fail "$name"
}
step DOWNIRQ 300 down-irq 1000000
step DOWNIRQ_MORE 120 down-irq --max 1 --burst 128 100000
step DOWN0IRQ 60 down0-irq 10000
/bin/linux-status
step RINGIRQ0 180 host0-irq 100000
# proc0 served SCMI all along: still serving, both cores healthy.
limit 10 /bin/linux-status --health 1000
rc=$?; echo LINUX_STATUS_RC=$rc; [ $rc -eq 0 ] || fail linux_status
step MIXIRQ 120 mix-irq 100000
step IRQPAIR 120 irq-pair
# Writers killed at random moments (5..200 ms into their streams) must do no harm: the next
# full run still goes through. That run sends, so it does not show the opening notify; the
# open-only step below does.
i=0
while [ $i -lt 50 ]; do
    /bin/shm-linux down-irq --stream-only 100000000 &
    pid=$!
    usleep $((5000 + RANDOM % 195000))
    kill -9 "$pid" 2>/dev/null
    wait "$pid" 2>/dev/null
    i=$((i + 1))
done
echo "DOWNIRQ_KILLS=$i"
step DOWNIRQ_KILL 60 down-irq 10000
[ $rc -eq 0 ] && echo DOWNIRQ_KILL_PASS
# A writer that dies between its publish and its doorbell leaves messages that the sleeping
# reader was never told about (UNRUNG checks they stay unread for the stall window). The next
# writer only opens the ring and sends nothing: its opening notify alone must get the reader
# through them within the stall window.
step DOWNIRQ_UNRUNG 30 down-irq --publish-only 8
step DOWNIRQ_REOPEN 30 down-irq --open-only 8
# Record only: not part of the result.
limit 30 /bin/shm-linux controls
echo "CONTROLS_RC=$?"
grep -E 'rp1|uio' /proc/interrupts
dmesg | grep -E 'SError|abort|fault|WARN'
if [ -z "$FAILED" ]; then
    echo IRQ_CHECK_RESULT=PASS
else
    echo "IRQ_CHECK_RESULT=FAIL$FAILED"
fi
echo IRQ_CHECK_END
[ -z "$FAILED" ]
