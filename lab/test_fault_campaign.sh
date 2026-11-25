#!/usr/bin/env bash
set -Eeuo pipefail
lab=$(cd "$(dirname "$0")" && pwd)
source "$lab/fault-campaign.sh"
unset LINUX_STATUS
fixture=$(mktemp -d)
trap 'rm -rf "$fixture"' EXIT
export TRACE="$fixture/trace"
tools="$fixture/tools"
mkdir "$tools"
cat > "$tools/sh.py" <<'PY'
import os
import shlex
import sys

command = sys.argv[3]
mode = os.environ["CASE_MODE"]
with open(os.environ["TRACE"], "a") as trace:
    print(command, file=trace)
# The target shell emits bracketed-paste toggles around command output.
paste = "\x1b[?2004l" if mode == "bracketed-paste" else ""
if "findmnt" in command:
    client = os.environ.get("LINUX_STATUS", "/usr/local/libexec/rp1-lab/linux-status")
    words = shlex.split(command)
    dmesg = words.index("dmesg")
    assert words[dmesg - 3:dmesg] == ["&&", client, "&&"]
    print(paste + "active\nnfs4\nRP1_STATUS epoch=2 pcie=Up scmi=Serving(requests=12)")
    print("RP1_CORE core=0 state=Running\nRP1_CORE core=1 state=" +
          ("Panicked" if mode == "preflight-state" else "Running"))
    print("arm-scmi: SCMI Protocol")
    rc = 42 if mode == "preflight-rc" else 0
else:
    if mode in ("collect-timeout", "shutdown-timeout"):
        print("SH_TIMEOUT END_1234abcd")
        sys.exit(1)
    print("FAULT_END nonce=12ab5678 kind=p0-thread-panic pcie=Up peer=Running")
    rc = 42 if mode == "collect-rc" else 0
print(f"END_1234abcd rc={rc}")
if paste:
    print("\x1b[?2004hroot@target:~# ")
PY
cat > "$tools/phase.sh" <<'SH'
#!/usr/bin/env bash
set -eu
case $1 in
    start)
        mkdir -p "$2/continuous"
        printf 'FAULT_BOOT nonce=12ab5678 kinds=p0-thread-panic\n' > "$2/continuous/uart0.raw"
        printf "%s\n" "$$" > "$2/capture.pid"
        ;;
    stop)
        echo stop >> "$TRACE"
        rm "$2/capture.pid"
        if [[ ${STOP_ERROR:-0} == 1 ]]; then exit 1; fi
        ;;
esac
SH
chmod +x "$tools/phase.sh"
cat > "$tools/boot.py" <<'PY'
import os
import sys
with open(os.environ["TRACE"], "a") as trace:
    print(sys.argv[2], file=trace)
PY
printf 'print("RELEASED")\n' > "$tools/rpc.py"

if LINUX_STATUS=linux-status preflight "$fixture" 2> "$fixture/path-error"; then exit 1; fi
grep -Fq 'must be an absolute target path' "$fixture/path-error"

export HOST=192.0.2.1 USB_PORT=3-1 SCMI_N=2
kinds=(p0-thread-panic)
phases=(cold-1 reboot)
for mode in preflight-rc preflight-state collect-timeout collect-rc; do
    export CASE_MODE=$mode
    RUN="$fixture/$mode"
    mkdir "$RUN"
    : > "$TRACE"
    rc=0
    capture_matrix > "$RUN/matrix.log" 2>&1 || rc=$?
    if [[ $mode == collect-timeout ]]; then
        [[ $rc == 75 ]]
    else
        [[ $rc == 1 ]]
    fi
    [[ ! -e "$RUN/p0-thread-panic-reboot" ]]
    if grep -qx reboot "$TRACE"; then exit 1; fi
    case $mode in
        preflight-*) if grep -q 'fault-linux' "$TRACE"; then exit 1; fi ;;
        *) grep -q 'fault-linux' "$TRACE" ;;
    esac
    capture_stop
done

export CASE_MODE=shutdown-timeout
RUN="$fixture/shutdown-timeout"
mkdir "$RUN"
: > "$TRACE"
rc=0
run_case p0-thread-panic reboot > "$RUN/case.log" 2>&1 || rc=$?
[[ $rc == 75 ]]
if grep -Eq 'reboot|fault-linux|findmnt' "$TRACE"; then exit 1; fi
capture_stop

cat > "$fixture/scmi.log" <<'SCMI'
RP1_STATUS epoch=2 pcie=Up scmi=Serving(requests=25)
SCMI_PREFLIGHT_RATE=48000000
SCMI_PREFLIGHT_RATE=48000000
RP1_STATUS epoch=2 pcie=Up scmi=Serving(requests=27)
SCMI
scmi_preflight_ok "$fixture/scmi.log" 2
sed -i 's/requests=27/requests=25/' "$fixture/scmi.log"
if scmi_preflight_ok "$fixture/scmi.log" 2; then exit 1; fi

export CASE_MODE=success
RUN="$fixture/success"
mkdir "$RUN"
LINUX_STATUS="$fixture/staged linux-status" run_case p0-thread-panic cold-1 > "$RUN/case.log"

export CASE_MODE=bracketed-paste
RUN="$fixture/bracketed-paste"
mkdir "$RUN"
run_case p0-thread-panic cold-1 > "$RUN/case.log"
log="$RUN/p0-thread-panic-cold-1/preflight.log"
grep -q $'\e\\[?2004l' "$log.raw"
if grep -q $'\e' "$log"; then exit 1; fi
grep -qx 'active' "$log"

export T="$fixture/tftp"
mkdir "$T"
export RUN="$fixture/restore"
mkdir "$RUN"
printf 'original\n' > "$RUN/manifest-before.txt"
printf 'test\n' > "$T/manifest.txt"
cat > "$RUN/restore.sh" <<'SH'
set -eu
echo restore >> "$TRACE"
cp "$RUN/manifest-before.txt" "$T/manifest.txt"
echo RESTORE_VERIFY_OK > "$RUN/restore-verify.txt"
if [[ ${MISSING_VERIFY:-} != 1 ]]; then
    echo ROOTFS_TOOLS_RESTORE_OK >> "$RUN/restore-verify.txt"
fi
SH
: > "$TRACE"
rc=0
(
    set -e
    install_restore_trap
    export STOP_ERROR=1
    capture_start "$RUN/partial"
    exit 42
) || rc=$?
[[ $rc == 42 && $(cat "$TRACE") == $'stop\nrestore' ]]
cmp "$RUN/manifest-before.txt" "$T/manifest.txt"

: > "$TRACE"
export MISSING_VERIFY=1
if restore_once; then exit 1; fi
[[ ${restored:-0} == 0 ]]
unset MISSING_VERIFY
restore_once
restore_once
[[ $(cat "$TRACE") == $'restore\nrestore' ]]

echo 'fault campaign fixtures passed'
