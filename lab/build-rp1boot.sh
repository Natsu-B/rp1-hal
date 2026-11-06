#!/usr/bin/env bash
# Build an rp1boot boot set for a lab firmware (hash-recorded; deploying is separate).
# Inputs (overridable): the stock board DTB the firmware loads, the Linux kernel and the shm initramfs.
# The firmware boots the stock DTB (no device_tree=); rp1boot applies RP1.elf's .rp1.dtbo onto it.
# Required: RP1BOOT_ARTIFACT_ROOT (output dir), RP1_TFTP_ROOT (firmware boot dir with the stock
# DTB and overlays/), RP1_KERNEL, RP1_INITRAMFS. Alternatively RP1_BOARD_CHECK_DTB
# supplies an already overlaid board DTB without a firmware boot directory.
# Optional: RP1_BASE_DTB, RP1_CMDLINE,
# FIRMWARE_PKG (default shm-firmware; package and binary names must match).
# FIRMWARE_FEATURES: cargo feature arguments (e.g. "--no-default-features
# --features sram-profile"); use the same ones for the `cargo rp1 build` that made RP1_ISRAM_PLAN.
# SHM_FIRMWARE_FEATURES remains a fallback for existing shm-firmware callers.
set -euo pipefail
repo=$(cd "$(dirname "$0")/.." && pwd)
out=${RP1BOOT_ARTIFACT_ROOT:?set RP1BOOT_ARTIFACT_ROOT to the output directory}
board_check=${RP1_BOARD_CHECK_DTB:-}
if [[ -z $board_check ]]; then
    tftp=${RP1_TFTP_ROOT:?set RP1_TFTP_ROOT or RP1_BOARD_CHECK_DTB}
    base=${RP1_BASE_DTB:-$tftp/bcm2712-rpi-cm5l-cm5io.dtb}
fi
export CARGO_BUILD_JOBS=${CARGO_BUILD_JOBS:-2}
kernel=${RP1_KERNEL:?set RP1_KERNEL to the Linux Image}
initramfs=${RP1_INITRAMFS:?set RP1_INITRAMFS to the initramfs}
firmware=${FIRMWARE_PKG:-shm-firmware}
features=${FIRMWARE_FEATURES-${SHM_FIRMWARE_FEATURES:-}}
cmdline=${RP1_CMDLINE:-earlycon=pl011,0x107d001000,115200n8 console=ttyAMA10,115200 rdinit=/init panic=0 ftrace=nop trace_clock=mono_raw trace_buf_size=256K trace_options=nooverwrite}
mkdir -p "$out"
cd "$repo"
# shellcheck disable=SC2086 # word-split feature arguments
cargo rp1 build --offline -p "$firmware" --target thumbv7m-none-eabi --release $features
cargo build --offline -p rp1boot --target aarch64-unknown-none-softfloat --release
cp "target/thumbv7m-none-eabi/release/$firmware" "$out/RP1.elf"
llvm-objcopy -O binary target/aarch64-unknown-none-softfloat/release/rp1boot "$out/rp1boot.img"
# Check against what the firmware hands over: the stock DTB + bcm2712d0.dtbo (D0 stepping) +
# the config.txt overlay below (Linux's UART1 on GPIO 0/1 = ttyAMA1).
if [[ -n $board_check ]]; then
    cp "$board_check" "$out/board-check.dtb"
else
    fdtoverlay -i "$base" -o "$out/board-check.dtb" "$tftp/overlays/bcm2712d0.dtbo" "$tftp/overlays/uart1-pi5.dtbo"
fi
cargo run --offline -q -p cargo-rp1 -- rp1 check "$out/RP1.elf" --dtb "$out/board-check.dtb"
rm -rf "$out/boot"
cargo run --offline -q -p cargo-rp1 -- rp1 bundle "$out/RP1.elf" --kernel "$kernel" \
    --initramfs "$initramfs" --loader "$out/rp1boot.img" --cmdline "$cmdline" \
    --config-extra dtoverlay=uart1-pi5 --config-extra device_tree_address=0x20000000 \
    --out "$out/boot"
sha256sum "$out/boot/"* "$out/RP1.elf" "$out/board-check.dtb" "$kernel" "$initramfs"
