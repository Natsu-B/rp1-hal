#!/bin/sh
# Configure a dedicated CM5 boot. Run as root before starting the RTT client.
set -eu

cpu=/sys/devices/system/cpu
irq=/proc/irq
housekeeping=0-1,3
isolated=2
cat /proc/cmdline
test "$(cat "$cpu/isolated")" = "$isolated"
test "$(cat "$cpu/nohz_full")" = "$isolated"
test "$(cat "$irq/default_smp_affinity")" = b
printf 'isolated=%s nohz_full=%s default_irq_mask=%s\n' \
    "$(cat "$cpu/isolated")" "$(cat "$cpu/nohz_full")" "$(cat "$irq/default_smp_affinity")"

if command -v systemctl >/dev/null && systemctl is-active --quiet irqbalance; then
    systemctl stop irqbalance
fi

workqueue=/sys/devices/virtual/workqueue/cpumask
if [ -w "$workqueue" ]; then
    echo b > "$workqueue"
    printf 'unbound_workqueue_mask=%s\n' "$(cat "$workqueue")"
fi
for policy in "$cpu"/cpufreq/policy*; do
    [ -d "$policy" ] || continue
    echo performance > "$policy/scaling_governor"
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
