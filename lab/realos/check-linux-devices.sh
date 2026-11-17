#!/usr/bin/env bash
# Linux-side check of RP1 devices that Linux owns (I2C, SPI, PWM, fan, Ethernet, USB) on the
# real Raspberry Pi OS environment. Run as root on the target with device arguments, e.g.
#   PING_HOST=<peer> rp1-device-check i2c2 i2c6 spi1.0 pwm0 fan eth:eth0 usb:3-1
# Use the attached USB device's sysfs port name, or usb-storage:sda for a direct read.
#
# Prints one row per probe and continues after failures:
#   DEVCHECK dev=<device> op=<operation> probe=yes|no rc=<status> result=PASS|FAIL evidence=<code> ...
# A probe passes when its command succeeds and its evidence is ok (an IRQ count rise and a
# readable kernel log without matching timeouts), sysfs (PWM state read back) or tach (fan
# speed). Failing evidence codes: no_cursor (no kernel log cursor or /proc/interrupts before
# the probe), log_lost (the kernel log cursor scrolled away), timeout (a matching timeout in the
# kernel log), no_irq (the required IRQ count did not rise), io_error (/proc/interrupts after
# the probe or the probe output file), unavailable (probe=no; see reason=). The first failing
# code is reported. Ends with exactly one DEVCHECK_END result=PASS|FAIL line.
#
# Optional: PING_HOST (required for eth:*), I2C_FIRST/I2C_LAST (short scan range, default
# 0x08..0x0f), FULL_SCAN=1 (also scan the whole bus when the short scan took less than five
# seconds), LABEL (default run), OUTDIR (default /var/log/rp1-device-check/<LABEL>-<boot id>).
# Needs: i2c-tools, python3-spidev, usbutils, iputils-ping, and busybox with the devmem applet.
set -uo pipefail
(( $# )) || { echo "usage: ${0##*/} DEVICE..." >&2; exit 1; }
shopt -s extglob
label=${LABEL:-run}
boot_id=$(cat /proc/sys/kernel/random/boot_id) ||
    { echo "DEVCHECK_END result=FAIL reason=boot_id"; exit 1; }
out=${OUTDIR:-/var/log/rp1-device-check/$label-$boot_id}
mkdir -p "$out" || { echo "DEVCHECK_END result=FAIL reason=out_dir"; exit 1; }
echo "DEVCHECK_BEGIN label=$label boot_id=$boot_id out=$out"
uname -r
cat /proc/cmdline

unavailable() { # device, operation, reason
    echo "DEVCHECK dev=$1 op=$2 probe=no rc=1 result=FAIL evidence=unavailable reason=$3"
    return 1
}

result() { # device, operation, command status, evidence code, optional IRQ counts
    local verdict=FAIL
    if (( $3 == 0 )); then
        case $4 in ok|sysfs|tach) verdict=PASS ;; esac
    fi
    echo "DEVCHECK dev=$1 op=$2 probe=yes rc=$3 result=$verdict evidence=$4${5:+ $5}"
    [[ $verdict == PASS ]]
}

note() { # failing evidence code; sets the caller's evidence unless it already failed
    [[ $evidence != ok ]] || evidence=$1
}

# Compare two /proc/interrupts snapshots, summing only the CPU columns. With show_dma=1 the
# shared DMA lines are listed separately from the controller IRQ. require is controller or dma:
# exit 1 unless a line of that kind rose. A rise is necessary, not attributed to the probe:
# background traffic (an NFS root on eth0, other users of the shared DMA) also counts.
irqdelta() { # before, after, controller IRQ pattern, require, show_dma
    awk -v pat="$3" -v require="$4" -v show_dma="$5" '
        FNR == 1 { cpus = NF; next }
        $1 ~ /^[0-9]+:$/ && ($0 ~ pat || (show_dma && /dma/)) {
            total = 0
            for (i = 2; i <= cpus + 1; i++) total += $i
            if (NR == FNR) before[$1] = total
            else if ($1 in before) {
                delta = total - before[$1]
                line = sprintf("%s(%s)%+d ", $NF, $1, delta)
                if ($0 ~ pat) { irq = irq line; if (delta > 0) rose["controller"] = 1 }
                if (show_dma && /dma/) { dma = dma line; if (delta > 0) rose["dma"] = 1 }
            }
        }
        END {
            printf "irq=\"%s\" shared_dma_irq=\"%s\"", irq, dma
            exit !(require in rose)
        }
    ' "$1" "$2"
}

irq_probe() { # device, operation, IRQ pattern, required IRQ (controller/dma), kernel log pattern, command
    local device=$1 op=$2 irq_pat=$3 need_irq=$4 log_pat=$5 tag=$out/$1-$2
    local cursor before after output counts= rc=0 log_status=0 irq_status=0 evidence=ok show_dma=0
    shift 5
    # Use the last timestamped record as a cursor. A lost cursor means incomplete evidence.
    if ! cursor=$(dmesg --time-format=raw --force-prefix | tail -n 1) || [[ $cursor != \[* ]] ||
        ! before=$(cat /proc/interrupts) || [[ -z $before ]]; then
        result "$device" "$op" 1 no_cursor
        return
    fi
    output=$("$@" 2>&1) || rc=$?
    after=$(cat /proc/interrupts) || after=
    dmesg --time-format=raw --force-prefix | awk -v pat="$log_pat|$irq_pat" '
        NR == FNR { cursor = $0; next }
        found { print; if ($0 ~ pat && /timed out|timeout/) timed_out = 1 }
        $0 == cursor { found = 1 }
        END { exit (!found ? 1 : timed_out ? 2 : 0) }
    ' <(printf '%s\n' "$cursor") - > "$tag.dmesg" || log_status=$?
    case $log_status in 0) ;; 2) note timeout ;; *) note log_lost ;; esac
    [[ $device == spi* ]] && show_dma=1
    if [[ -n $after ]]; then
        counts=$(irqdelta <(printf '%s\n' "$before") <(printf '%s\n' "$after") \
            "$irq_pat" "$need_irq" "$show_dma") || irq_status=$?
        case $irq_status in 0) ;; 1) note no_irq ;; *) note io_error ;; esac
    else
        note io_error
    fi
    printf '%s\n' "$output" > "$tag.out" || note io_error
    result "$device" "$op" "$rc" "$evidence" "$counts"
}

# Read-only dump of the RP1 interrupt output gates of the blocks Linux has bound. RP1 addresses
# (0x40xxxxxx) are read through the PCIe BAR at 0x1f00000000 + (address - 0x40000000).
reg() { # bound platform device, register name, RP1 address
    [[ -L /sys/bus/platform/devices/$1/driver ]] || return 0
    local address value
    address=$(printf '0x1f%08x' "$(( $3 - 0x40000000 ))")
    value=$(busybox devmem "$address" 32 2>&1) || value="unavailable: $value"
    echo "REG $1 $2 $address = $value"
}
{
    reg 1f00100000.ethernet ETH_CFG.INTE 0x40104020
    reg 1f00200000.usb USBHOST0_CFG+0x98 0x40160098
    reg 1f00300000.usb USBHOST1_CFG+0x98 0x40164098
    # I2C/SPI wrapper +0x108, bit 0 = interrupt output enable.
    for d in /sys/bus/platform/devices/1f000*.{i2c,spi}; do
        [[ -L $d/driver ]] || continue
        d=${d##*/}
        reg "$d" IRQ_WRAP+0x108 "$(( 0x${d%%.*} - 0x1f00000000 + 0x40000108 ))"
    done
} | tee "$out/registers.txt"

short_scan() {
    local timing=$1 TIMEFORMAT=%R
    shift
    { time timeout 120 i2cdetect -y "$@" 2>&1; } 2>"$timing"
}

check_i2c() {
    local device=$1 bus=${1#i2c} path irq timing=$out/$1-scan.seconds secs
    path=$(readlink -f "/sys/bus/i2c/devices/i2c-$bus")
    [[ -e /dev/i2c-$bus && $path =~ /(1f000[0-9a-f]+\.i2c)/i2c-[0-9]+$ ]] ||
        { unavailable "$device" scan no_rp1_i2c_adapter; return; }
    irq=${BASH_REMATCH[1]/./[.]}
    irq_probe "$device" scan "$irq" controller " $irq:" short_scan "$timing" "$bus" \
        "${I2C_FIRST:-0x08}" "${I2C_LAST:-0x0f}" || return 1
    [[ ${FULL_SCAN:-0} == 1 ]] || return 0
    # A slow short scan means slow probing (timeouts), so the full scan would take minutes.
    if ! read -r secs < "$timing" || [[ ! $secs =~ ^[0-9]+[.,][0-9]+$ ]]; then
        unavailable "$device" scan-full missing_scan_time
    elif (( ${secs%%[.,]*} < 5 )); then
        irq_probe "$device" scan-full "$irq" controller " $irq:" timeout 300 i2cdetect -y "$bus"
    else
        unavailable "$device" scan-full "short_scan_slow secs=$secs"
    fi
}

spi_transfer() {
    timeout 30 python3 -c '
import spidev, sys
bus, cs, length = map(int, sys.argv[1:])
spi = spidev.SpiDev()
spi.open(bus, cs)
spi.max_speed_hz = 1000000
spi.mode = 0
rx = spi.xfer2([(i * 7) & 0xff for i in range(length)])
spi.close()
print("len", len(rx), "rx_head", rx[:8])
' "$@"
}

check_spi() {
    local device=$1 bus_cs=${1#spi} path irq failed=0
    path=$(readlink -f "/sys/class/spidev/spidev$bus_cs")
    [[ -e /dev/spidev$bus_cs && $path =~ /(1f000[0-9a-f]+\.spi)/ ]] ||
        { unavailable "$device" xfer16 no_rp1_spidev; return; }
    irq=${BASH_REMATCH[1]/./[.]}
    irq_probe "$device" xfer16 "$irq" controller " spidev ${device/./[.]}:" \
        spi_transfer "${bus_cs%.*}" "${bus_cs#*.}" 16 || failed=1
    # spi-dw moves transfers longer than its FIFO by DMA, so the evidence is a shared DMA IRQ rise.
    irq_probe "$device" xfer4096 "$irq" dma " spidev ${device/./[.]}:" \
        spi_transfer "${bus_cs%.*}" "${bus_cs#*.}" 4096 || failed=1
    return "$failed"
}

readback_probe() { # device, operation, evidence code, sysfs test
    local device=$1 op=$2 evidence=$3 rc=0
    shift 3
    "$@" > "$out/$device-$op.out" 2>&1 || rc=$?
    result "$device" "$op" "$rc" "$evidence"
}

pwm_test() (
    local chip channel period duty enabled
    for chip in /sys/class/pwm/pwmchip*; do
        if [[ $(readlink -f "$chip/device") == */1f00098000.pwm ]]; then channel=$chip/pwm0; break; fi
    done
    [[ -v channel ]] || { echo no_rp1_pwm_chip; return 1; }
    # Only a successful sysfs export confers ownership.
    echo 0 > "$chip/export" || return 1
    trap '
        rc=$?
        if [[ -e $channel/enable ]]; then echo 0 > "$channel/enable" || rc=1; fi
        echo 0 > "$chip/unexport" || rc=1
        exit "$rc"
    ' EXIT
    trap 'exit 1' HUP INT TERM
    sleep 0.2 && echo 1000000 > "$channel/period" && echo 500000 > "$channel/duty_cycle" &&
        echo 1 > "$channel/enable" || return 1
    period=$(cat "$channel/period") && duty=$(cat "$channel/duty_cycle") &&
        enabled=$(cat "$channel/enable") || return 1
    echo "period=$period duty=$duty enable=$enabled"
    [[ $period == 1000000 && $duty == 500000 && $enabled == 1 ]]
)

fan_test() (
    local h d hwmon= cooling= state saved max forced pwm rpm restored
    for h in /sys/class/hwmon/hwmon*; do
        if [[ $(cat "$h/name" 2>/dev/null) == pwmfan ]]; then hwmon=$h; break; fi
    done
    for d in /sys/class/thermal/cooling_device*; do
        if [[ $(cat "$d/type" 2>/dev/null) == pwm-fan ]]; then cooling=$d; break; fi
    done
    [[ -n $hwmon && -n $cooling ]] || { echo no_pwm_fan; return 1; }
    state=$cooling/cur_state
    saved=$(cat "$state") && max=$(cat "$cooling/max_state") || return 1
    [[ $saved =~ ^[0-9]+$ && $max =~ ^[0-9]+$ && $max -ge 2 ]] || return 1
    trap '
        rc=$?
        if ! { echo "$saved" > "$state" && restored=$(cat "$state") && [[ $restored == "$saved" ]]; }; then
            echo "FAN restore failed (state=$saved)" >&2
            rc=1
        fi
        exit "$rc"
    ' EXIT
    trap 'exit 1' HUP INT TERM
    echo 2 > "$state" && sleep 3 || return 1
    forced=$(cat "$state") && pwm=$(cat "$hwmon/pwm1") &&
        rpm=$(cat "$hwmon/fan1_input") || return 1
    echo "FAN before=$saved forced=$forced pwm1=$pwm fan1_input=$rpm"
    [[ $forced == 2 && $pwm =~ ^[1-9][0-9]*$ && $rpm =~ ^[1-9][0-9]*$ ]]
)

check_eth() {
    local device=$1 iface=${1#eth:}
    [[ $(readlink -f "/sys/class/net/$iface/device") == */1f00100000.ethernet ]] ||
        { unavailable "$device" ping no_rp1_net_interface; return; }
    [[ -n ${PING_HOST:-} ]] || { unavailable "$device" ping no_ping_host; return; }
    irq_probe "$device" ping "[[:space:]]$iface([[:space:],-]|$)" controller \
        " 1f00100000[.]ethernet([[:space:]]|:)| $iface:" ping -I "$iface" -c 20 -i 0.2 -q -- "$PING_HOST"
}

expect_output() { # required output pattern, command
    local pattern=$1 detail rc=0
    shift
    detail=$("$@") || rc=$?
    printf '%s\n' "$detail"
    (( rc == 0 )) || return "$rc"
    grep -qE "$pattern" <<< "$detail"
}

usb_read() {
    LC_ALL=C timeout 30 dd if="$1" of=/dev/null bs=1M count=32 iflag=direct,fullblock 2>&1
}

check_usb() {
    local device=$1 op=control path bus address irq controller log_pat primary= port
    case $device in
        usb:*) path=$(readlink -f "/sys/bus/usb/devices/${device#usb:}") ;;
        usb-storage:*) op=read path=$(readlink -f "/sys/block/${device#usb-storage:}") ;;
    esac
    [[ -e $path && $path =~ ^(.*/1f00[23]00000\.usb/.+)/usb[0-9]+/ ]] ||
        { unavailable "$device" "$op" no_rp1_usb_device; return; }
    controller=${BASH_REMATCH[1]}
    # xHCI registers its IRQ on the primary root hub, including for its shared USB3 hub.
    for port in "$controller"/usb[0-9]*; do
        [[ ${port##*/usb} =~ ^[0-9]+$ && -d $port ]] || continue
        bus=${port##*/usb}
        if [[ -z $primary || $bus -lt $primary ]]; then primary=$bus; fi
    done
    [[ -z $primary ]] && { unavailable "$device" "$op" no_xhci_root_hub; return; }
    irq="xhci.*:usb$primary([[:space:],]|$)"
    controller=${controller##*/}
    log_pat=" ${controller//./[.]}:"
    if [[ $path =~ /([0-9]+-[0-9]+(\.[0-9]+)*)(:|$) ]]; then
        port=${BASH_REMATCH[1]}
        log_pat+="| usb ${port//./[.]}:"
    fi
    if [[ $device == usb-storage:* ]]; then
        irq_probe "$device" read "$irq" controller "$log_pat" expect_output '^32[+]0 records in$' \
            usb_read "/dev/${device#usb-storage:}"
    else
        bus=$(cat "$path/busnum") && address=$(cat "$path/devnum") ||
            { unavailable "$device" control no_usb_address; return; }
        irq_probe "$device" control "$irq" controller "$log_pat" expect_output '^[[:space:]]*Device Status:' \
            timeout 30 lsusb -v -s "$bus:$address"
    fi
}

# Keep extended globs in variables so ordinary bash -n can parse this case.
i2c='i2c+([0-9])' spi='spi+([0-9]).+([0-9])' eth='eth:+([a-zA-Z0-9_-])'
usb='usb:+([0-9])-+([0-9])*(.+([0-9]))' storage='usb-storage:sd+([a-z])'
failed=0
for device do
    case $device in
        $i2c) check_i2c "$device" ;;
        $spi) check_spi "$device" ;;
        pwm0) readback_probe pwm0 ch0 sysfs pwm_test ;;
        fan) readback_probe fan cool tach fan_test ;;
        $eth) check_eth "$device" ;;
        $usb|$storage) check_usb "$device" ;;
        *) unavailable "$device" - unknown_device ;;
    esac || failed=1
done
verdict=PASS
(( failed )) && verdict=FAIL
echo "DEVCHECK_END result=$verdict out=$out"
exit "$failed"
