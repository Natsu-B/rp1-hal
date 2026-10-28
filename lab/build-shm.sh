#!/usr/bin/env bash
# Build a hash-pinned dev tuple; deployment is a separate, backed-up step.
set -euo pipefail
repo=$(cd "$(dirname "$0")/.." && pwd)
root=${CM5_HACK_ROOT:-<lab>}
out=${SHM_ARTIFACT_ROOT:-$root/<lab-artifact>
loader=${RP1_LOADER_ROOT:-<local>
base=${RP1_BASE_DTB:-$root/<lab-backup>
mkdir -p "$out/fw" "$out/dtb" "$out/loader"
cd "$repo"
cargo build --offline -p shm-firmware --target thumbv7m-none-eabi --release
cargo build --offline -p shm-linux -p linux-doorbell --target aarch64-unknown-linux-musl --release
cp target/thumbv7m-none-eabi/release/shm-firmware "$out/fw/RP1.elf"
# The bss smoke test needs a real .bss in the image (review #10).
arm-none-eabi-size -A "$out/fw/RP1.elf" | awk '$1==".bss" { found = $2 > 0 } END { exit !found }'
cp target/aarch64-unknown-linux-musl/release/{shm-linux,linux-doorbell} "$out/"
llvm-objcopy --dump-section .rp1.dtbo="$out/dtb/app.dtbo" "$out/fw/RP1.elf"
fdtoverlay -i "$base" -o "$out/dtb/scmi_linux.dtb" "$out/dtb/app.dtbo"
args=$(fdtget -t s "$out/dtb/scmi_linux.dtb" /chosen bootargs)
fdtput -t s "$out/dtb/scmi_linux.dtb" /chosen bootargs "${args// clk_ignore_unused/}"
cargo rp1 check "$out/fw/RP1.elf" --dtb "$out/dtb/scmi_linux.dtb"
export RP1_HAL_DEV_ELF_SHA256 RP1_HAL_DEV_DTB_SHA256 RP1_HAL_DEV_SHM_SIZE
RP1_HAL_DEV_ELF_SHA256=$(sha256sum "$out/fw/RP1.elf" | cut -c1-64)
RP1_HAL_DEV_DTB_SHA256=$(sha256sum "$out/dtb/scmi_linux.dtb" | cut -c1-64)
RP1_HAL_DEV_SHM_SIZE=$(fdtget -t x "$out/dtb/scmi_linux.dtb" /reserved-memory/rp1-shm@22000000 reg | awk '{print $3}')
cd "$loader"
if git apply --reverse --check "$repo/lab/loader-handoff.patch" 2>/dev/null; then
    :
else
    git apply --check "$repo/lab/loader-handoff.patch"
    git apply "$repo/lab/loader-handoff.patch"
fi
git rev-parse HEAD
git diff -- rp1_chainboot_poc/src/rp1_bootstrap.rs
printf 'ELF=%s DTB=%s SHM_SIZE=%s\n' "$RP1_HAL_DEV_ELF_SHA256" "$RP1_HAL_DEV_DTB_SHA256" "$RP1_HAL_DEV_SHM_SIZE"
CARGO_TARGET_DIR="$out/loader-target" CARGO_INCREMENTAL=0 CARGO_ENCODED_RUSTFLAGS='' CARGO_PROFILE_RELEASE_DEBUG=0 CARGO_PROFILE_RELEASE_STRIP=debuginfo \
 cargo +nightly-2026-08-27 build --offline --locked -j4 -Z build-std=core,alloc,compiler_builtins -Z build-std-features=compiler-builtins-mem \
 -p rp1_chainboot_poc --target aarch64-unknown-none-softfloat --release --no-default-features \
 --features 'log-uart,tftp-boot,require-rp1-img,rp1-hal-dev'
cp "$out/loader-target/aarch64-unknown-none-softfloat/release/rp1_chainboot_poc" "$out/loader/reader.elf"
llvm-objcopy -O binary "$out/loader/reader.elf" "$out/loader/kernel_2712.img"
sha256sum "$out/fw/RP1.elf" "$out/dtb/scmi_linux.dtb" "$out/loader/kernel_2712.img" "$out/shm-linux"
