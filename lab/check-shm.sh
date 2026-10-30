#!/bin/sh
# Linux-side SHM / doorbell / I2C / ADC regression. Strict: every step has a time limit, a non-zero
# exit code or a missing reply is a failure, and the script ends with exactly one
# `CHECK_SHM_RESULT=PASS|FAIL`, then `DDR_INSPECT_END`, and exits non-zero on failure.
# The host decides with `lab/verify-shm.py` (UART0 + UART10 + capture completion); the boot
# marker report is not a pass for this regression.
export PATH=/bin
busybox --install -s /bin
FAILED=
fail() { echo "CHECK_FAIL $1"; FAILED="$FAILED $1"; }
# limit SECONDS CMD...: run CMD, kill it after SECONDS (rc 137). No busybox timeout: its
# syntax differs between versions.
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
echo HWRUN3_BEGIN
uname -a
dmesg | grep -E 'Kernel command line|SCMI Protocol|clk:|34000|chip_id'
insmod /bin/uio.ko
insmod /bin/uio_pdrv_genirq.ko of_id=generic-uio
echo UIO_RC=$?
L= D=
for x in /sys/class/uio/uio*; do
    n=$(cat "$x/name")
    echo "UIO $x $n addr=$(cat "$x/maps/map0/addr") size=$(cat "$x/maps/map0/size")"
    case "$n" in rp1:link) L=/dev/${x##*/};; rp1:doorbell) D=/dev/${x##*/};; esac
done
[ -n "$L" ] && [ -n "$D" ] || fail uio
cat /proc/iomem
echo ASPM_POLICY=$(cat /sys/module/pcie_aspm/parameters/policy 2>&1)
for d in /sys/bus/pci/devices/*; do echo "PCI $d $(cat $d/vendor) $(cat $d/device)"; ls $d/link 2>/dev/null && grep . $d/link/* 2>/dev/null; od -A x -t x4 -N 256 $d/config; done
limit 30 /bin/linux-doorbell "$L" "$D" 1000
rc=$?; echo DOORBELL_BEFORE_RC=$rc; [ $rc -eq 0 ] || fail doorbell_before
# shm-linux has a 180 s limit per request; the whole run normally takes well under a minute.
limit 600 /bin/shm-linux 1000000
rc=$?; echo SHM_RC=$rc; [ $rc -eq 0 ] || fail shm
limit 30 /bin/linux-doorbell "$L" "$D" 1000
rc=$?; echo DOORBELL_AFTER_RC=$rc; [ $rc -eq 0 ] || fail doorbell_after
mount -t debugfs none /sys/kernel/debug 2>/dev/null
grep -E 'uart|apb' /sys/kernel/debug/clk/clk_summary
dmesg | grep -E 'scmi|clk:|34000|SError|abort|fault|WARN|error'
echo HWRUN3_DONE
echo I2C_LINUX_BEGIN
for path in $(find /sys/firmware/devicetree/base -name i2c@74000); do
    echo I2C_DT="$path"
    cat "$path/status"
done
# Request 1 = the full I2C proof (lab/shm-firmware/src/i2c_proof.rs); the reply word is the
# request on success, 0xffffffff on failure, 0 while pending.
busybox devmem 0x1f0040c02c 32 0
busybox devmem 0x1f0040c028 32 1
reply=0
i=0
while [ $i -lt 20 ]; do
    sleep 1
    reply=$(busybox devmem 0x1f0040c02c 32)
    [ $((reply)) -ne 0 ] && break
    i=$((i + 1))
done
echo I2C_REPLY=$reply
[ $((reply)) -eq 1 ] || fail i2c_reply
echo I2C_LINUX_DONE
# request REQ: write it to link+0x28 (reply cleared first), poll link+0x2c up to 20 s and
# print the reply. Each runs only after the doorbell and shm steps have finished: the
# proc1 bursts block its doorbell and command handling for tens of ms.
request() {
    busybox devmem 0x1f0040c02c 32 0
    busybox devmem 0x1f0040c028 32 "$1"
    reply=0
    i=0
    while [ $i -lt 20 ]; do
        sleep 1
        reply=$(busybox devmem 0x1f0040c02c 32)
        [ $((reply)) -ne 0 ] && break
        i=$((i + 1))
    done
}
echo ADC_LINUX_BEGIN
for path in $(find /sys/firmware/devicetree/base -name adc@c8000); do
    echo ADC_DT="$path" status=$(cat "$path/status")
done
# 0xadc = proc1's ADC burst (lab/shm-firmware/src/adc_proof.rs).
request 0xadc
echo ADC_REPLY=$reply
[ $((reply)) -eq $((0xadc)) ] || fail adc_reply
echo ADC_LINUX_DONE
if [ -z "$FAILED" ]; then
    echo CHECK_SHM_RESULT=PASS
else
    echo "CHECK_SHM_RESULT=FAIL$FAILED"
fi
echo DDR_INSPECT_END
[ -z "$FAILED" ]
