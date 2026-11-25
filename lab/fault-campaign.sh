# Bash functions used by the fault containment hardware procedure.

# The target shell wraps command output in bracketed-paste toggles (ESC[?2004l / ESC[?2004h),
# which break line-anchored checks. Only those two sequences are removed; "$log.raw" keeps
# the console text unchanged.
strip_paste_toggles() {
    sed 's/\x1b\[?2004[hl]//g'
}

remote_checked() {
    local dir=$1 timeout=$2 command=$3 log=$4 marker rc=0
    python3 "$tools/sh.py" "$dir" "$timeout" "( $command )" > "$log.raw" || rc=$?
    strip_paste_toggles < "$log.raw" | tee "$log" || return
    if (( rc )); then
        printf 'Console command did not complete\n' >&2
        return 75
    fi
    marker=$(awk '/^END_[0-9a-f]+ rc=[0-9]+$/ { marker=$0; count++ }
                  END { if (count == 1) print marker; else exit 1 }' "$log") || return 75
    if [[ ! $marker =~ ^END_[0-9a-f]{8}\ rc=([0-9]+)$ ]]; then
        return 75
    fi
    [[ ${BASH_REMATCH[1]} == 0 ]]
}

preflight_ok() {
    local log=$1
    grep -qx 'active' "$log" &&
        grep -Eqx 'nfs4?' "$log" &&
        grep -Eq '^RP1_STATUS .* pcie=Up scmi=Serving\(requests=[0-9]+\)$' "$log" &&
        awk '
            $1 == "RP1_CORE" { count[$2]++; if ($3 != "state=Running") bad = 1 }
            END { exit (bad || count["core=0"] != 1 || count["core=1"] != 1) }
        ' "$log" &&
        grep -Eq 'arm-scmi.*SCMI Protocol' "$log"
}

scmi_preflight_ok() {
    local log=$1 n=$2 counts
    local -a samples
    counts=$(sed -nE 's/^RP1_STATUS .* scmi=Serving\(requests=([0-9]+)\)$/\1/p' "$log") || return
    mapfile -t samples <<< "$counts"
    [[ ${#samples[@]} == 2 && ${samples[0]} =~ ^[0-9]{1,5}$ && ${samples[1]} =~ ^[0-9]{1,5}$ ]] || return 1
    (( 10#${samples[1]} < 65535 && 10#${samples[1]} - 10#${samples[0]} == n )) || return 1
    [[ $(grep -Ec '^SCMI_PREFLIGHT_RATE=[1-9][0-9]*$' "$log") == "$n" ]]
}

capture_start() {
    [[ -z ${active_capture:-} && ! -e $1 ]] || return 1
    active_capture=$1
    "$tools/phase.sh" start "$active_capture"
}

capture_stop() {
    local rc=0
    [[ -n ${active_capture:-} ]] || return 0
    if [[ -f $active_capture/capture.pid || -S $active_capture/capture.sock ]]; then
        "$tools/phase.sh" stop "$active_capture" || rc=$?
        [[ ! -S $active_capture/capture.sock ]] || return 1
    fi
    active_capture=
    return "$rc"
}

restore_once() {
    if ! capture_stop; then
        [[ -z $active_capture ]] || return 1
        echo 'Capture stopped with errors. Restoring the backup.' >&2
    fi
    [[ ${restored:-0} == 0 ]] || return 0
    rm -f "$RUN/restore-verify.txt" || return
    bash "$RUN/restore.sh" 2>&1 | tee "$RUN/restore.log" || return
    grep -qx RESTORE_VERIFY_OK "$RUN/restore-verify.txt" &&
        grep -qx ROOTFS_TOOLS_RESTORE_OK "$RUN/restore-verify.txt" &&
        cmp "$RUN/manifest-before.txt" "$T/manifest.txt" || return
    restored=1
}

restore_on_exit() {
    local rc=$?
    trap - EXIT
    trap '' INT TERM
    if ! restore_once; then
        echo 'Capture stop or restoration failed. Keep the backup and inspect restore.log.' >&2
        (( rc != 0 )) || rc=1
    fi
    exit "$rc"
}

install_restore_trap() {
    trap restore_on_exit EXIT
    trap 'exit 130' INT
    trap 'exit 143' TERM
}

preflight() {
    local d=$1 client=${LINUX_STATUS:-/usr/local/libexec/rp1-lab/linux-status} quoted_client
    if [[ $client != /* ]]; then
        printf 'LINUX_STATUS must be an absolute target path: %s\n' "$client" >&2
        return 2
    fi
    printf -v quoted_client '%q' "$client"
    remote_checked "$d" 50 \
        "systemctl is-active multi-user.target && findmnt -n -o FSTYPE / && $quoted_client && dmesg | grep -E 'arm-scmi.*SCMI Protocol' && (mountpoint -q /sys/kernel/debug || mount -t debugfs debugfs /sys/kernel/debug) && test -r /sys/kernel/debug/clk/uart/clk_rate && test -r /dev/kmsg" \
        "$d/preflight.log" &&
        preflight_ok "$d/preflight.log"
}

run_case() {
    local kind=$1 phase=$2 d="$RUN/$1-$2" nonce
    capture_start "$d" || return
    python3 "$tools/rpc.py" "$d" release-check | tee "$d/cm5-release.txt" || return
    case "$phase" in
        cold-1|cold-2)
            python3 "$tools/boot.py" "$d" force | tee "$d/boot.log" || return ;;
        reboot|poweroff)
            remote_checked "$d" 15 'true' "$d/pre-shutdown.log" || return
            if [[ $phase == reboot ]]; then
                python3 "$tools/boot.py" "$d" reboot | tee "$d/boot.log" || return
            else
                poweroff_boot "$d" || return
            fi ;;
    esac
    nonce=$(sed -nE 's/^.*FAULT_BOOT nonce=([0-9a-f]{8}) .*/\1/p' "$d/continuous/uart0.raw") || return
    [[ $nonce =~ ^[0-9a-f]{8}$ ]] || return 1
    printf '%s\n' "$nonce" > "$d/nonce.txt" || return
    preflight "$d" || return
    remote_checked "$d" 180 "fault-linux $kind $nonce $HOST $USB_PORT $SCMI_N" "$d/target.log" || return
    if [[ $kind != stkerr ]]; then
        grep -q "^FAULT_END nonce=$nonce kind=$kind " "$d/target.log" || return 1
    fi
    capture_stop
}

capture_matrix() {
    local kind phase rc
    for kind in "${kinds[@]}"; do
        for phase in "${phases[@]}"; do
            rc=0
            run_case "$kind" "$phase" || rc=$?
            if (( rc )); then
                printf 'Capture stopped: %s %s rc=%s\n' "$kind" "$phase" "$rc" |
                    tee -a "$RUN/capture-errors.txt"
                return "$rc"
            fi
        done
    done
}
