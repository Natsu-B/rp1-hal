#!/usr/bin/env bash
# Build the SCMI client modules for a stock Raspberry Pi OS kernel that has no SCMI built in.
#
# Use the source and headers matching the installed kernel's Debian version.
# The build updates the extracted KSRC tree. Re-extract it to start clean.
#
# Required: KSRC (extracted linux-source tree), KHEADERS (linux-headers-<release> directory),
#           OUT (output directory for the .ko files).
# Optional: CROSS_COMPILE (default aarch64-linux-gnu-), KCC (default ${CROSS_COMPILE}gcc-14).
set -euo pipefail
ksrc=${KSRC:?set KSRC to the extracted linux-source tree}
headers=${KHEADERS:?set KHEADERS to the linux-headers-<release> directory}
out=${OUT:?set OUT to the output directory}
cross=${CROSS_COMPILE:-aarch64-linux-gnu-}
cc=${KCC:-${cross}gcc-14}
release=$(cat "$headers/include/config/kernel.release")
jobs=${CARGO_BUILD_JOBS:-2}
k=(make -C "$ksrc" ARCH=arm64 CROSS_COMPILE="$cross" CC="$cc" KERNELRELEASE="$release" -j"$jobs")

cp "$headers/.config" "$ksrc/.config"
# Only what the overlay's /firmware/scmi node needs: the core, the mailbox transport, clocks.
"$ksrc/scripts/config" --file "$ksrc/.config" \
    -m ARM_SCMI_PROTOCOL -m ARM_SCMI_TRANSPORT_MAILBOX -m COMMON_CLK_SCMI \
    -d ARM_SCMI_TRANSPORT_SMC -d ARM_SCMI_PERF_DOMAIN -d ARM_SCMI_POWER_DOMAIN \
    -d RESET_SCMI -d ARM_SCMI_POWER_CONTROL -d ARM_SCMI_RAW_MODE_SUPPORT
"${k[@]}" olddefconfig
"${k[@]}" modules_prepare
# The running kernel's symbol CRCs.
cp "$headers/Module.symvers" "$ksrc/Module.symvers"
"${k[@]}" M=drivers/firmware/arm_scmi modules

# clk-scmi lives in drivers/clk; build it alone as an external module.
clk=$(mktemp -d)
trap 'rm -rf "$clk"' EXIT
cp "$ksrc/drivers/clk/clk-scmi.c" "$clk/"
echo 'obj-m := clk-scmi.o' > "$clk/Kbuild"
"${k[@]}" M="$clk" KBUILD_EXTRA_SYMBOLS="$ksrc/drivers/firmware/arm_scmi/Module.symvers" modules

mkdir -p "$out"
cp "$ksrc/drivers/firmware/arm_scmi/scmi-core.ko" "$ksrc/drivers/firmware/arm_scmi/scmi-module.ko" \
    "$ksrc/drivers/firmware/arm_scmi/transports/scmi_transport_mailbox.ko" "$clk/clk-scmi.ko" "$out/"
for m in "$out"/*.ko; do
    magic=$(modinfo -F vermagic "$m")
    [[ $magic == "$release "* ]] || { echo "vermagic mismatch in $m: $magic" >&2; exit 1; }
done
(cd "$out" && sha256sum ./*.ko)
