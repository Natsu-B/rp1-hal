#!/usr/bin/env bash
# Build the rp1boot boot set for the shm lab firmware (hash-recorded; deploying is separate).
# Inputs (overridable): the lab's SCMI base DTB, the Linux kernel and the shm initramfs.
set -euo pipefail
repo=$(cd "$(dirname "$0")/.." && pwd)
root=${CM5_HACK_ROOT:-<lab>}
out=${RP1BOOT_ARTIFACT_ROOT:-$root/<lab-artifact>
base=${RP1_BASE_DTB:-$root/<lab-backup>
kernel=${RP1_KERNEL:-$root/tftpboot/linux_2712.img}
initramfs=${RP1_INITRAMFS:-$root/<lab-artifact>
cmdline=${RP1_CMDLINE:-earlycon=pl011,0x107d001000,115200n8 console=ttyAMA10,115200 rdinit=/init panic=0 ftrace=nop trace_clock=mono_raw trace_buf_size=256K trace_options=nooverwrite}
mkdir -p "$out"
cd "$repo"
cargo build --offline -p shm-firmware --target thumbv7m-none-eabi --release
cargo build --offline -p rp1boot --target aarch64-unknown-none-softfloat --release
cp target/thumbv7m-none-eabi/release/shm-firmware "$out/RP1.elf"
llvm-objcopy -O binary target/aarch64-unknown-none-softfloat/release/rp1boot "$out/rp1boot.img"
# The SCMI transport/clock policy still comes from the PoC-generated base DTB (not yet in
# .rp1.dtbo). It is a runtime dump, so drop its stale /chosen values: the firmware prepends
# an existing bootargs to cmdline.txt (HW hwrun5 08-boot7).
cp "$base" "$out/rp1-base.dtb"
for p in bootargs linux,initrd-start linux,initrd-end; do
    fdtput -d "$out/rp1-base.dtb" /chosen "$p" 2>/dev/null || true
done
cargo run --offline -q -p cargo-rp1 -- rp1 check "$out/RP1.elf" --dtb "$out/rp1-base.dtb"
rm -rf "$out/boot"
cargo run --offline -q -p cargo-rp1 -- rp1 bundle "$out/RP1.elf" --kernel "$kernel" \
    --initramfs "$initramfs" --loader "$out/rp1boot.img" --cmdline "$cmdline" \
    --config-extra device_tree=rp1-base.dtb --config-extra device_tree_address=0x20000000 \
    --out "$out/boot"
cp "$out/rp1-base.dtb" "$out/boot/"
sha256sum "$out/boot/"* "$base" "$kernel" "$initramfs"
