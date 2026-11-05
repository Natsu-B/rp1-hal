#!/bin/sh
# Source in the Linux observer shell; keep each sample in one console write.
export PATH=/bin
reset_telemetry_sample() {
    printf 'TELEMETRY_BEGIN uptime='
    busybox cat /proc/uptime
    for zone in /sys/class/thermal/thermal_zone*; do
        [ -r "$zone/temp" ] || continue
        printf 'THERMAL %s type=%s temp=%s\n' "$zone" "$(busybox cat "$zone/type")" "$(busybox cat "$zone/temp")"
    done
    for chip in /sys/class/hwmon/hwmon*; do
        [ -r "$chip/name" ] || continue
        printf 'HWMON %s name=%s\n' "$chip" "$(busybox cat "$chip/name")"
        for file in "$chip"/in*_input "$chip"/in*_alarm "$chip"/in*_label "$chip"/temp*_input "$chip"/temp*_alarm "$chip"/temp*_label; do
            [ -r "$file" ] && printf 'HWMON_VALUE %s=%s\n' "$file" "$(busybox cat "$file")"
        done
    done
    for file in $RESET_THROTTLED_FILES; do
        printf 'GET_THROTTLED %s=%s\n' "$file" "$(busybox cat "$file" 2>&1)"
    done
    if [ -n "$RESET_VCGENCMD" ]; then
        "$RESET_VCGENCMD" get_throttled
    fi
    printf 'DMESG_TAIL_BEGIN\n'
    busybox dmesg | busybox tail -5
    printf 'DMESG_TAIL_END\nTELEMETRY_END\n'
}
reset_telemetry_start() {
    RESET_THROTTLED_FILES=
    for file in /sys/devices/platform/soc/soc:firmware/get_throttled; do
        [ -r "$file" ] && RESET_THROTTLED_FILES="$RESET_THROTTLED_FILES $file"
    done
    # Only discover named firmware status files; never read arbitrary debugfs registers.
    for file in $(busybox find /sys/kernel/debug -type f -name 'get_throttled' 2>/dev/null); do
        [ -r "$file" ] && RESET_THROTTLED_FILES="$RESET_THROTTLED_FILES $file"
    done
    RESET_VCGENCMD=$(command -v vcgencmd)
    [ -n "$RESET_THROTTLED_FILES$RESET_VCGENCMD" ] || echo TELEMETRY_MISSING=get_throttled
    [ -d /sys/class/thermal/thermal_zone0 ] || echo TELEMETRY_MISSING=thermal
    busybox ls /sys/class/hwmon/hwmon*/name >/dev/null 2>&1 || echo TELEMETRY_MISSING=hwmon
    echo TELEMETRY_RP1=only_if_exposed_as_named_thermal_or_hwmon_sensor
    echo TELEMETRY_INTERVAL_SECONDS=5
    (
        while :; do
            sample=$(reset_telemetry_sample)
            printf '\n%s\n' "$sample"
            busybox sleep 5
        done
    ) &
    RESET_TELEMETRY_PID=$!
}
reset_telemetry_stop() {
    kill "$RESET_TELEMETRY_PID" 2>/dev/null
    wait "$RESET_TELEMETRY_PID" 2>/dev/null
    reset_telemetry_sample
    echo TELEMETRY_STOPPED
}
