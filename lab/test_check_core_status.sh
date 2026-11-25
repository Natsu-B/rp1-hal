#!/usr/bin/env bash
set -Eeuo pipefail
lab=$(cd "$(dirname "$0")" && pwd)
fixture=$(mktemp -d)
trap 'rm -rf "$fixture"' EXIT
export PATH="$fixture:$PATH" STATUS_READS="$fixture/reads"
cat > "$fixture/busybox" <<'SH'
#!/bin/sh
[ "$1" = devmem ] && [ "$2" = "$WANT_ADDRESS" ] && [ "$3" = 32 ] || exit 2
if [ "${MODE:-}" = changed ] && [ -e "$STATUS_READS" ]; then
    echo 0x76543210
else
    echo 0x12AB5678
fi
touch "$STATUS_READS"
SH
cat > "$fixture/linux-status" <<'SH'
#!/bin/sh
case "${MODE:-}" in
    p0-panic) echo 'RP1_CORE core=0 state=Panicked' ;;
    missing) ;;
    *) echo 'RP1_CORE core=0 state=Running' ;;
esac
case "${MODE:-}" in
    p1-fault) echo 'RP1_CORE core=1 state=Fault' ;;
    *) echo 'RP1_CORE core=1 state=Running' ;;
esac
[ "${MODE:-}" != failed ]
SH
chmod +x "$fixture/busybox" "$fixture/linux-status"
reject() {
    if "$@"; then
        printf 'Unexpected success: %s\n' "$*" >&2
        exit 1
    fi
}
for check in sweep pwm scmi; do
    if [[ $check == sweep ]]; then
        export WANT_ADDRESS=0x1f0040c028
    else
        export WANT_ADDRESS=0x1f0040c030
    fi
    "$lab/check-core-status.sh" "$check" 12ab5678 > "$fixture/out"
    grep -qx "LAB_FINAL_END check=$check nonce=12ab5678 result=PASS" "$fixture/out"
    reject "$lab/check-core-status.sh" "$check" 76543210 > "$fixture/out"
    for mode in p0-panic p1-fault missing failed changed; do
        rm -f "$STATUS_READS"
        reject env MODE="$mode" "$lab/check-core-status.sh" "$check" 12ab5678 > "$fixture/out"
        grep -qx "LAB_FINAL_END check=$check nonce=12ab5678 result=FAIL" "$fixture/out"
    done
done
echo 'final core status fixtures passed'
