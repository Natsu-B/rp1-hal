#!/bin/sh
# Configure a dedicated CM5 boot. Run as root before starting the RTT client.
# The stock CM5 kernel used by this fixture has no CONFIG_NO_HZ_FULL. CPU 2
# still has a scheduler tick. FIFO priority and isolation limit runnable interference.
set -eu

cpu=/sys/devices/system/cpu
irq=/proc/irq
housekeeping=0-1,3
housekeeping_mask=b # Bits 0, 1 and 3.
isolated=2

require() {
    message=$1
    shift
    if ! "$@"; then
        printf '%s\n' "$message" >&2
        exit 1
    fi
}

cat /proc/cmdline
require "expected isolated CPU $isolated" test "$(cat "$cpu/isolated")" = "$isolated"
mask=$(cat "$irq/default_smp_affinity")
require "expected default IRQ mask $housekeeping_mask, got $mask" \
    test "$((0x$mask))" -eq "$((0x$housekeeping_mask))"
printf 'isolated=%s default_irq_mask=%s\n' "$isolated" "$mask"

if command -v systemctl >/dev/null && systemctl is-active --quiet irqbalance; then
    systemctl stop irqbalance
fi

workqueue=/sys/devices/virtual/workqueue/cpumask
if [ -w "$workqueue" ]; then
    echo "$housekeeping_mask" > "$workqueue"
    printf 'unbound_workqueue_mask=%s\n' "$(cat "$workqueue")"
fi
for policy in "$cpu"/cpufreq/policy*; do
    [ -d "$policy" ] || continue
    echo performance > "$policy/scaling_governor"
    require "expected performance governor in $policy" \
        test "$(cat "$policy/scaling_governor")" = performance
    printf '%s governor=performance\n' "$policy"
done

for f in "$irq"/[0-9]*/effective_affinity; do
    mask=$(cat "$f")
    if [ "$((0x$mask & 4))" -eq 0 ]; then
        continue
    fi
    dir=${f%/*}
    if (echo "$housekeeping" > "$dir/smp_affinity_list"); then
        printf 'IRQ %s affinity=%s\n' "${dir##*/}" "$(cat "$f")"
    else
        printf 'IRQ %s fixed affinity=%s\n' "${dir##*/}" "$mask"
    fi
done
