#!/bin/sh
export PATH=/bin
busybox --install -s /bin
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
cat /proc/iomem
/bin/linux-doorbell "$L" "$D" 1000
echo DOORBELL_BEFORE_RC=$?
/bin/shm-linux 1000000
echo SHM_RC=$?
/bin/linux-doorbell "$L" "$D" 1000
echo DOORBELL_AFTER_RC=$?
mount -t debugfs none /sys/kernel/debug 2>/dev/null
grep -E 'uart|apb' /sys/kernel/debug/clk/clk_summary
dmesg | grep -E 'scmi|clk:|34000|SError|abort|fault|WARN|error'
echo HWRUN3_DONE
echo DDR_INSPECT_END
