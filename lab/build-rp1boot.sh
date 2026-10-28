#!/usr/bin/env bash
# Build the rp1boot boot set for the shm lab firmware (hash-recorded; deploying is separate).
# Inputs (overridable): the stock board DTB the firmware loads, the Linux kernel and the shm initramfs.
# The firmware boots the stock DTB (no device_tree=); rp1boot applies RP1.elf's .rp1.dtbo onto it.
set -euo pipefail
repo=$(cd "$(dirname "$0")/.." && pwd)
root=${CM5_HACK_ROOT:-<lab>}
out=${RP1BOOT_ARTIFACT_ROOT:-$root/<lab-artifact>
tftp=${RP1_TFTP_ROOT:-$root/tftpboot}
base=${RP1_BASE_DTB:-$tftp/bcm2712-rpi-cm5l-cm5io.dtb}
kernel=${RP1_KERNEL:-$root/tftpboot/linux_2712.img}
initramfs=${RP1_INITRAMFS:-$root/<lab-artifact>
cmdline=${RP1_CMDLINE:-earlycon=pl011,0x107d001000,115200n8 console=ttyAMA10,115200 rdinit=/init panic=0 ftrace=nop trace_clock=mono_raw trace_buf_size=256K trace_options=nooverwrite}
mkdir -p "$out"
cd "$repo"
cargo build --offline -p shm-firmware --target thumbv7m-none-eabi --release
cargo build --offline -p rp1boot --target aarch64-unknown-none-softfloat --release
cp target/thumbv7m-none-eabi/release/shm-firmware "$out/RP1.elf"
llvm-objcopy -O binary target/aarch64-unknown-none-softfloat/release/rp1boot "$out/rp1boot.img"
# Check against what the firmware hands over: the stock DTB + bcm2712d0.dtbo (D0 stepping) +
# the config.txt overlay below (Linux's UART1 on GPIO 0/1 = ttyAMA1).
fdtoverlay -i "$base" -o "$out/board-check.dtb" "$tftp/overlays/bcm2712d0.dtbo" "$tftp/overlays/uart1-pi5.dtbo"
cargo run --offline -q -p cargo-rp1 -- rp1 check "$out/RP1.elf" --dtb "$out/board-check.dtb"
rm -rf "$out/boot"
cargo run --offline -q -p cargo-rp1 -- rp1 bundle "$out/RP1.elf" --kernel "$kernel" \
    --initramfs "$initramfs" --loader "$out/rp1boot.img" --cmdline "$cmdline" \
    --config-extra dtoverlay=uart1-pi5 --config-extra device_tree_address=0x20000000 \
    --out "$out/boot"
sha256sum "$out/boot/"* "$base" "$kernel" "$initramfs"
