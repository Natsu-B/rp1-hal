#!/bin/sh
# Linux side of the DMA regression. Run once on a fresh boot with shm-firmware --features dma.
# Usage: check-dma-shm.sh /path/to/shm-linux
set -eu
client=${1:?pass the candidate shm-linux executable}

step() {
    label=$1
    seconds=$2
    shift 2
    rc=0
    timeout "$seconds" "$client" "$@" || rc=$?
    printf 'V7_DMA_%s_RC=%s\n' "$label" "$rc"
    return "$rc"
}

echo V7_DMA_BEGIN
step PROBE 30s probe-only
step PING_BEFORE 30s ping
step COMMAND 60s dma
step WORKLOAD 600s workload 1000000
step PING_AFTER 30s ping
echo V7_DMA_END
