#!/bin/sh
# Run on the initramfs console after cold booting a dual-core ping-test bundle.
export PATH=/bin
mode=$1 bundle=$2 client_sha=$3
case "$mode" in irq|poll) ;; *) exit 2 ;; esac

ready() {
    line=$(busybox timeout -s KILL 15 /bin/linux-ping --test "$mode" ready) || return
    echo "$line phase=$1"
    /bin/linux-status
}

irq_count() {
    awk '$NF == "rp1:irq" {
        n++
        for (i = 2; $i ~ /^[0-9]+$/; i++) sum += $i
    } END {
        if (n != 1) exit 1
        print sum + 0
    }' /proc/interrupts
}

run() (
    set -e
    busybox --install -s /bin
    boot_id=$(cat /proc/sys/kernel/random/boot_id)
    grep -Eq "(^| )rp1\.bundle=$bundle( |$)" /proc/cmdline
    hash=$(sha256sum /bin/linux-ping)
    test "$hash" = "$client_sha  /bin/linux-ping"
    echo "PING_CHECK_BEGIN mode=$mode boot_id=$boot_id bundle=$bundle client_sha256=$client_sha"

    insmod /bin/uio.ko
    insmod /bin/uio_pdrv_genirq.ko of_id=generic-uio
    required='rp1:link rp1:header rp1:ping-dn rp1:ping-up'
    if [ "$mode" = irq ]; then
        required="$required rp1:doorbell rp1:irq"
    fi
    seen=
    for node in /sys/class/uio/uio*; do
        name=$(cat "$node/name")
        dev=/dev/${node##*/}
        ls -l "$dev"
        echo "PING_UIO $name $dev"
        if [ -r "$dev" ] && [ -w "$dev" ]; then
            seen="$seen $name"
        fi
    done
    for name in $required; do
        case "$seen " in *" $name "*) ;; *) echo "missing usable $name"; exit 1 ;; esac
    done

    ready before
    if [ "$mode" = irq ]; then
        cat /proc/interrupts
        before=$(irq_count)
    fi
    busybox timeout -s KILL 60 /bin/linux-ping --test "$mode" rtt
    if [ "$mode" = irq ]; then
        cat /proc/interrupts
        after=$(irq_count)
        echo "PING_IRQ before=$before after=$after"
        test "$after" -gt "$before"
    fi
    busybox timeout -s KILL 120 /bin/linux-ping --test "$mode" late
    ready after
    if [ "$mode" = irq ]; then
        busybox timeout -s KILL 5 /bin/linux-ping --test irq panic
        /bin/linux-status
    fi
    end_id=$(cat /proc/sys/kernel/random/boot_id)
    test "$end_id" = "$boot_id"
    echo "PING_BOOT_END_ID=$end_id"
)
run
rc=$?
result=PASS
if [ "$rc" -ne 0 ]; then
    result=FAIL
fi
echo "PING_CHECK_RESULT=$result mode=$mode"
echo PING_CHECK_END
exit "$rc"
