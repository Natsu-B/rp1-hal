#!/bin/sh
# Run on the isolated bench's Linux target. The archive destination is supplied by the caller.
set -eu
client=${1:?linux-ping path}
mode=${2:?test mode}
step=${3:?test step}
archive=${4:?output archive}
tmp=$(mktemp -d)
trace=
cleanup() {
    if [ -n "$trace" ] && [ -d "$trace" ]; then
        echo 0 > "$trace/tracing_on"
        rmdir "$trace"
    fi
    rm -r "$tmp"
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

snapshot() {
    for name in interrupts softirqs; do
        cat "/proc/$name" > "$tmp/$1.$name"
    done
    cpu=/sys/devices/system/cpu
    for f in "$cpu"/cpu[0-3]/cpufreq/scaling_cur_freq "$cpu"/cpu[0-3]/cpufreq/scaling_governor \
        "$cpu"/cpu[0-3]/cpuidle/state*/name "$cpu"/cpu[0-3]/cpuidle/state*/disable \
        "$cpu"/cpu[0-3]/cpuidle/state*/usage "$cpu"/cpu[0-3]/cpuidle/state*/time; do
        [ -r "$f" ] || continue
        printf '%s %s\n' "$f" "$(cat "$f")"
    done > "$tmp/$1.cpu"
}

setup_trace() {
    root=/sys/kernel/tracing
    if ! mountpoint -q "$root"; then
        mount -t tracefs tracefs "$root" || return 1
    fi
    instance="$root/instances/rp1-rtt"
    mkdir "$instance" || return 1
    trace=$instance
    echo 0 > "$trace/tracing_on" || return 1
    echo mono > "$trace/trace_clock" || return 1
    echo 4096 > "$trace/buffer_size_kb" || return 1
    echo 0 > "$trace/options/overwrite" || return 1
    for event in sched/sched_switch irq/irq_handler_entry irq/irq_handler_exit; do
        echo 1 > "$trace/events/$event/enable" || return 1
        printf '%s\n' "$event"
    done
    for event in sched/sched_waking irq/softirq_entry irq/softirq_exit \
        ipi/ipi_entry ipi/ipi_exit power/cpu_frequency power/cpu_idle \
        timer/hrtimer_expire_entry; do
        if [ -e "$trace/events/$event/enable" ]; then
            echo 1 > "$trace/events/$event/enable" || return 1
            printf '%s\n' "$event"
        fi
    done
    cat "$trace/trace_clock" "$trace/tracing_cpumask"
}

if setup_trace > "$tmp/trace-setup.txt" 2>&1; then
    RP1_RTT_TRACE=tracefs
else
    RP1_RTT_TRACE=cpu
fi
export RP1_RTT_TRACE

snapshot before
if [ "$RP1_RTT_TRACE" = tracefs ]; then
    echo 1 > "$trace/tracing_on"
fi
rc=0
if [ "${RP1_RTT_RT:-0}" = 1 ]; then
    RP1_PING_MLOCK=1 taskset -c 2 chrt -f 50 "$client" --test "$mode" "$step" || rc=$?
else
    taskset -c 2 "$client" --test "$mode" "$step" || rc=$?
fi
if [ "$RP1_RTT_TRACE" = tracefs ]; then
    echo 0 > "$trace/tracing_on"
fi
snapshot after
if [ "$RP1_RTT_TRACE" = tracefs ]; then
    cat "$trace/trace" > "$tmp/trace.txt"
    for f in "$trace"/per_cpu/cpu*/stats; do
        printf '%s\n' "$f"
        cat "$f"
    done > "$tmp/trace-stats.txt"
fi
tar -czf "$archive" -C "$tmp" .
exit "$rc"
