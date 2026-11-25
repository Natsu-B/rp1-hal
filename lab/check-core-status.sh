#!/bin/sh
# Run on Linux after the matching firmware's workload ends.
# Usage: check-core-status.sh sweep|pwm|scmi NONCE (UART0's eight hex digits).
# LINUX_STATUS overrides the absolute path to the build-matched client.
mode=${1:-}
case "$mode" in
    sweep) address=0x1f0040c028 ;;
    pwm|scmi) address=0x1f0040c030 ;;
    *) echo 'usage: check-core-status.sh sweep|pwm|scmi NONCE' >&2; exit 2 ;;
esac
nonce_word() {
    raw=$(busybox devmem "$address" 32) || return 1
    raw=${raw#0x}
    raw=${raw#0X}
    normalize "$raw"
}
normalize() {
    [ "${#1}" = 8 ] || return 1
    case "$1" in
        *[!0-9a-fA-F]*|FFFFFFFF|ffffffff) return 1 ;;
    esac
    printf '%s\n' "$1" | tr 'A-F' 'a-f'
}
expected=${2:-}
nonce=$(normalize "${expected#0x}") || exit 2
client=${LINUX_STATUS:-/usr/local/libexec/rp1-lab/linux-status}
case "$client" in
    /*) ;;
    *) printf 'LINUX_STATUS must be an absolute path: %s\n' "$client" >&2; exit 2 ;;
esac
result=FAIL
printf 'LAB_FINAL_BEGIN check=%s nonce=%s\n' "$mode" "$nonce"
if [ ! -f "$client" ] || [ ! -x "$client" ]; then
    printf 'Missing or non-executable linux-status client: %s\n' "$client" >&2
elif before=$(nonce_word) && [ "$before" = "$nonce" ]; then
    rc=0
    status=$("$client") || rc=$?
    printf '%s\n' "$status"
    if [ "$rc" = 0 ] && after=$(nonce_word) && [ "$after" = "$nonce" ] &&
        printf '%s\n' "$status" | awk '
            $1 == "RP1_CORE" {
                count[$2]++; if ($2 !~ /^core=[01]$/ || $3 != "state=Running") bad = 1
            }
            END { exit (bad || count["core=0"] != 1 || count["core=1"] != 1) }
        '
    then
        result=PASS
    fi
fi
printf 'LAB_FINAL_END check=%s nonce=%s result=%s\n' "$mode" "$nonce" "$result"
[ "$result" = PASS ]
