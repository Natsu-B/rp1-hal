#!/usr/bin/env bash
# Build an rp1boot boot set that starts the stock Raspberry Pi OS kernel with an NFS root.
# The firmware (RP1.elf) and the loader (rp1boot.img) come from an existing boot set or a build;
# the kernel is the root file system's own vmlinuz, and the initramfs is from build-initramfs.sh.
#
# Required: ROOTFS, RP1_ELF, RP1BOOT_IMG, INITRAMFS, NFS_ROOT (server:/path as exported), OUT.
# OUT must name a new directory whose parent exists.
# Optional: KERNEL_RELEASE (default: the only release prepared in ROOTFS), RP1_BOARD_CHECK_DTB (run
#           `cargo rp1 check` against it), CMDLINE_EXTRA, CONSOLE (default ttyAMA10).
# On failure OUT is removed and the `cargo rp1 check` output, if any, is printed.
set -euo pipefail
repo=$(cd "$(dirname "$0")/../.." && pwd)
root=$(realpath -e -- "${ROOTFS:?set ROOTFS}")
elf=$(realpath -e -- "${RP1_ELF:?set RP1_ELF}")
loader=$(realpath -e -- "${RP1BOOT_IMG:?set RP1BOOT_IMG}")
initramfs=$(realpath -e -- "${INITRAMFS:?set INITRAMFS}")
nfs=${NFS_ROOT:?set NFS_ROOT to server:/exported/path}
parent=$(realpath -e -- "$(dirname -- "${OUT:?set OUT}")")
out=$parent/$(basename -- "$OUT")
dtb=
if [[ -n ${RP1_BOARD_CHECK_DTB:-} ]]; then
    dtb=$(realpath -e -- "$RP1_BOARD_CHECK_DTB")
fi
console=${CONSOLE:-ttyAMA10}
release=${KERNEL_RELEASE:-}
if [[ -z $release ]]; then
    shopt -s nullglob
    prepared=("$root"/lib/modules/*/updates/rp1-hal/)
    if (( ${#prepared[@]} != 1 )); then
        echo "expected one prepared kernel release in ROOTFS; set KERNEL_RELEASE" >&2
        exit 1
    fi
    release=${prepared[0]%/updates/rp1-hal/}
    release=${release##*/}
fi
kernel=$(realpath -e -- "$root/boot/vmlinuz-$release")
entries=$(lsinitramfs "$initramfs")
archive_release=$(awk -F/ '
    { sub(/^\.\//, ""); sub(/^usr\//, "") }
    /^lib\/modules\/[^/]+\/.+\.ko(\.(xz|zst|gz))?$/ { print $3 }
' <<< "$entries" | LC_ALL=C sort -u)
if [[ $archive_release != "$release" ]]; then
    printf 'expected initramfs modules for %s; found %s\n' "$release" "${archive_release:-none}" >&2
    exit 1
fi
export CARGO_BUILD_JOBS=${CARGO_BUILD_JOBS:-2}
# No ip=: the kernel's own IP autoconfiguration would start before the SCMI clock modules
# (and so the Ethernet) exist. The initramfs (BOOT=nfs, DEVICE=eth0) does DHCP instead.
cmdline="earlycon=pl011,0x107d001000,115200n8 console=$console,115200 root=/dev/nfs nfsroot=$nfs,nfsvers=3 rw uio_pdrv_genirq.of_id=generic-uio panic=0${CMDLINE_EXTRA:+ $CMDLINE_EXTRA}"

mkdir -- "$out"
trap 'rc=$?
if (( rc )); then
    [[ -f $out/check.log ]] && cat -- "$out/check.log" >&2
    rm -rf -- "$out"
fi' EXIT
cd "$repo"
if [[ -n $dtb ]]; then
    cargo run --offline -q -p cargo-rp1 -- rp1 check "$elf" --dtb "$dtb" \
        > "$out/check.log" 2>&1
fi
cargo run --offline -q -p cargo-rp1 -- rp1 bundle "$elf" --kernel "$kernel" \
    --initramfs "$initramfs" --loader "$loader" --cmdline "$cmdline" \
    --config-extra dtoverlay=uart1-pi5 --config-extra device_tree_address=0x20000000 \
    --out "$out/boot"
echo "$release" > "$out/KERNEL_RELEASE"
(cd "$out" && find . -type f ! -name SHA256SUMS -printf '%P\0' | LC_ALL=C sort -z | xargs -0 sha256sum -- > SHA256SUMS)
cat "$out/boot/manifest.txt"
