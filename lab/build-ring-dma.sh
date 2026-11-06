#!/usr/bin/env bash
# Build only: stage a ring-DMA boot set and a client with a reproducible measurement plan.
# Required: RP1BOOT_ARTIFACT_ROOT, RP1_KERNEL, RP1_INITRAMFS (base rootfs), plus
# RP1_BOARD_CHECK_DTB or the firmware inputs of build-rp1boot.sh. No deployment.
set -euo pipefail
repo=$(cd "$(dirname "$0")/.." && pwd)
out=${RP1BOOT_ARTIFACT_ROOT:?set RP1BOOT_ARTIFACT_ROOT}
base_initramfs=${RP1_INITRAMFS:?set RP1_INITRAMFS to the base rootfs}
mkdir -p "$out"
out=$(cd "$out" && pwd)
base_initramfs=$(realpath "$base_initramfs")
[[ $base_initramfs != "$out/initramfs_2712" ]] || { echo 'base rootfs must differ from output' >&2; exit 1; }
export CARGO_BUILD_JOBS=${CARGO_BUILD_JOBS:-2}
cd "$repo"
cargo build --offline -p shm-linux --target aarch64-unknown-linux-musl --release
mkdir -p "$out/client/bin"
cp target/aarch64-unknown-linux-musl/release/shm-linux "$out/client/bin/"
(cd "$out/client" && printf 'bin/shm-linux\n' | cpio -o -H newc --owner=0:0) > "$out/client.cpio"
# Linux accepts concatenated compressed/uncompressed newc archives; the last member wins.
cat "$base_initramfs" > "$out/initramfs_2712"
pad=$(( (512 - $(stat -c %s "$base_initramfs") % 512) % 512 ))
head -c "$pad" /dev/zero >> "$out/initramfs_2712"
cat "$out/client.cpio" >> "$out/initramfs_2712"
RP1BOOT_ARTIFACT_ROOT="$out" RP1_INITRAMFS="$out/initramfs_2712" \
 FIRMWARE_FEATURES='--no-default-features --features ring-dma' "$repo/lab/build-rp1boot.sh"
llvm-objcopy --dump-section .rp1.dtbo="$out/app.dtbo" "$out/RP1.elf" /dev/null
(cd "$out" && sha256sum RP1.elf rp1boot.img app.dtbo initramfs_2712 client/bin/shm-linux boot/* > SHA256SUMS)
