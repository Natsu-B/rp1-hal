#!/bin/sh
# Linux-side check for the real Raspberry Pi OS environment (stock kernel, NFS root, systemd).
# Run as root on the target, e.g. from the serial console. Prints an inventory and runs the
# examples/dual-core doorbell polling check (linux-doorbell, 1000 round trips over the rp1:link and
# rp1:doorbell UIO nodes). Ends with exactly one REALOS_CHECK_RESULT=PASS|FAIL line.
# Everything is also written to OUTDIR (default /var/log/rp1-realos/<boot id>).
# Optional: DOORBELL (default linux-doorbell from PATH), ROUNDS (default 1000),
# EXPECTED_ROOT_FSTYPE (default nfs; set to e.g. ext4 for an SD root).
boot_id=$(cat /proc/sys/kernel/random/boot_id) || { echo "REALOS_CHECK_RESULT=FAIL boot_id"; exit 1; }
out=${OUTDIR:-/var/log/rp1-realos/$boot_id}
mkdir -p "$out" || { echo "REALOS_CHECK_RESULT=FAIL out_dir"; exit 1; }
doorbell=${DOORBELL:-linux-doorbell}
rounds=${ROUNDS:-1000}
FAILED=
fail() { echo "REALOS_FAIL $1"; FAILED="$FAILED $1"; }
section() { echo "=== $1"; }
save() { # save <name> <command...>: full output to a file, head on the console
    name=$1; shift
    "$@" > "$out/$name.txt" 2>&1
    saved_rc=$?
    echo "--- $name ($(wc -l < "$out/$name.txt") lines, $out/$name.txt)"
    return "$saved_rc"
}

echo "REALOS_CHECK_BEGIN boot_id=$boot_id out=$out"
section system
uname -a
cat /proc/cmdline
expected_root=${EXPECTED_ROOT_FSTYPE:-nfs}
if root_type=$(findmnt -n -o FSTYPE --mountpoint /); then
    echo "ROOT_FSTYPE=$root_type expected=$expected_root"
    case "$expected_root:$root_type" in
        "$expected_root:$expected_root"|nfs:nfs4) ;;
        *) fail root_fstype ;;
    esac
else
    fail root_mount
fi
systemctl is-active --quiet multi-user.target || fail multi-user.target
systemctl is-system-running
systemctl --failed --no-legend --plain | sed 's/^/FAILED_UNIT /'
save lsmod lsmod
save interrupts cat /proc/interrupts
save ip-addr ip -br addr
cat "$out/ip-addr.txt"
section buses
for p in /sys/class/i2c-dev /sys/class/spidev /dev/i2c-* /dev/spidev* /dev/uio* /dev/ttyAMA*; do
    [ -e "$p" ] && ls -d "$p" "$p"/* 2>/dev/null
done | sed 's/^/NODE /'
if command -v lsusb >/dev/null; then save lsusb lsusb; cat "$out/lsusb.txt"; else echo "lsusb absent"; fi
section rp1-platform
# Bound platform devices per driver (the RP1 ones sit under .../1f00000000.* or pcie).
for d in /sys/bus/platform/drivers/*; do
    for dev in "$d"/*; do
        case "$dev" in */bind|*/unbind|*/uevent|*/module|*/new_id|*/remove_id) continue;; esac
        [ -L "$dev" ] && echo "BOUND ${d##*/} ${dev##*/}"
    done
done > "$out/platform-bound.txt"
grep -E '1f000|rp1|scmi|pcie' "$out/platform-bound.txt"
if [ -r /sys/kernel/debug/devices_deferred ]; then
    sed 's/^/DEFERRED /' /sys/kernel/debug/devices_deferred | tee "$out/devices-deferred.txt"
else
    echo "devices_deferred unreadable"
fi
# RP1 device-tree nodes with status "okay" that have no driver bound.
for n in /sys/bus/platform/devices/1f*; do
    [ -e "$n" ] || continue
    [ -L "$n/driver" ] || echo "UNBOUND ${n##*/} $(cat "$n/of_node/compatible" 2>/dev/null | tr '\0' ' ')"
done | tee "$out/rp1-unbound.txt"

section uio
modprobe uio_pdrv_genirq || fail modprobe
L= D= H=
for x in /sys/class/uio/uio*; do
    [ -r "$x/name" ] || continue
    n=$(cat "$x/name") || continue
    echo "UIO $x $n"
    node=/dev/${x##*/}
    [ -e "$node" ] || continue
    case "$n" in
        rp1:link) L=$node ;;
        rp1:doorbell) D=$node ;;
        rp1:header) H=$node ;;
    esac
done
[ -n "$L" ] || fail uio_rp1:link
[ -n "$D" ] || fail uio_rp1:doorbell
[ -n "$H" ] || fail uio_rp1:header
section doorbell-polling
if [ -n "$L" ] && [ -n "$D" ]; then
    timeout 30 "$doorbell" "$L" "$D" "$rounds" > "$out/doorbell.txt" 2>&1
    rc=$?
    cat "$out/doorbell.txt"
    echo "DOORBELL_MODE=polling DOORBELL_RC=$rc"
    [ "$rc" -eq 0 ] || fail doorbell
fi
if save dmesg dmesg; then
    grep -iE 'scmi|macb|rp1|uio' "$out/dmesg.txt" | tail -40
    grep -E 'SError|Internal error|scmi.*(fail|timed out)' "$out/dmesg.txt"
    case $? in
        0) fail kernel_message ;;
        1) ;;
        *) fail dmesg ;;
    esac
else
    fail dmesg
fi
echo REALOS_CHECK_END
if [ -z "$FAILED" ]; then
    echo REALOS_CHECK_RESULT=PASS
else
    echo "REALOS_CHECK_RESULT=FAIL$FAILED"
    exit 1
fi
