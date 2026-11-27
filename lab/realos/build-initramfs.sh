#!/usr/bin/env bash
# Build the NFS-root initramfs with the offline rootfs's own initramfs-tools, inside a chroot
# (an aarch64 host, or qemu-user binfmt with the F flag on another host). Run as root.
# Uses etc/initramfs-tools-rp1nfs written by prepare-rootfs.sh (MODULES=list, BOOT=nfs).
#
# Required: ROOTFS, OUT (output initramfs file outside ROOTFS), and unmkinitramfs on the host.
# Optional: KERNEL_RELEASE (default: the only release prepared in ROOTFS).
set -euo pipefail
root=$(realpath -e -- "${ROOTFS:?set ROOTFS to the root file system copy}")
if [[ $root -ef / ]]; then
    echo "ROOTFS must be an offline copy, not the running root file system" >&2
    exit 1
fi
out=${OUT:?set OUT to the output initramfs path}
parent=$(realpath -e -- "$(dirname -- "$out")")
out=$parent/$(basename -- "$out")
if [[ $out == "$root" || $out == "$root/"* ]]; then
    echo "OUT must be outside ROOTFS" >&2
    exit 1
fi
# Keep temporary chroot mounts clear of concurrent preparation or rollback.
exec {root_lock}<"$root"
flock -n "$root_lock"
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
[[ -d $root/etc/initramfs-tools-rp1nfs ]] || { echo "run prepare-rootfs.sh first" >&2; exit 1; }
command -v unmkinitramfs >/dev/null || { echo "unmkinitramfs is required" >&2; exit 1; }
for d in tmp proc sys dev; do
    path=$(realpath -e -- "$root/$d")
    [[ $path == "$root/"* && -d $path ]] || { echo "invalid ROOTFS/$d directory" >&2; exit 1; }
done

# Stage beside OUT so the final rename can replace it atomically.
work=$(mktemp -d -- "${out}.XXXXXX")
root_tmp=
declare -A mounted=()
unmount() {
    local d failed=0
    for d in dev sys proc; do
        [[ ${mounted[$d]:-} ]] || continue
        if umount "$root/$d"; then
            unset 'mounted[$d]'
        else
            failed=1
        fi
    done
    return "$failed"
}
cleanup() {
    local rc=$?
    unmount || rc=1
    if [[ -n $root_tmp ]]; then rm -rf -- "$root_tmp" || rc=1; fi
    rm -rf -- "$work" || rc=1
    exit "$rc"
}
trap cleanup EXIT
root_tmp=$(mktemp -d -- "$root/tmp/rp1nfs.XXXXXX")
for d in proc sys dev; do
    mount --bind "/$d" "$root/$d"
    mounted[$d]=1
done
tmp=${root_tmp#"$root"}
TMPDIR="$tmp" chroot "$root" /usr/sbin/mkinitramfs -d /etc/initramfs-tools-rp1nfs \
    -o "$tmp/initramfs" "$release"
unmount
mv -- "$root_tmp/initramfs" "$work/initramfs"

unmkinitramfs "$work/initramfs" "$work/unpacked"
image=$work/unpacked
# Older unmkinitramfs versions put the main archive under main/.
if [[ -d $image/main ]]; then image=$image/main; fi

# Run the archive's shell and resolve its paths inside the archive.
modules=/lib/modules/$release
updates=$modules/updates/rp1-hal
chroot "$image" /bin/sh -ec '
    for entry in /init /bin/sh; do
        if ! { [ -f "$entry" ] && [ -x "$entry" ]; }; then
            echo "initramfs entry point is missing or not executable: $entry" >&2
            exit 1
        fi
    done
    for required do
        if [ ! -f "$required" ]; then
            echo "missing initramfs file: $required" >&2
            exit 1
        fi
    done
' sh /scripts/nfs "$modules/modules.dep" \
    "$updates"/{scmi-core,scmi-module,scmi_transport_mailbox,clk-scmi}.ko

mv -fT -- "$work/initramfs" "$out"
ls -l -- "$out"
sha256sum -- "$out"
