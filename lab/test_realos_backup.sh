#!/usr/bin/env bash
# Exercise the public prepare/restore entry points on unprivileged fixtures.
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
test_dir=$(mktemp -d)
trap 'rm -rf -- "$test_dir"' EXIT
mkdir "$test_dir/bin"
cat > "$test_dir/bin/modinfo" <<'EOF'
#!/usr/bin/env bash
echo '6.12-fixture SMP'
EOF
cat > "$test_dir/bin/depmod" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
[[ $1 == -b && $3 == -o ]]
dir=$4/lib/modules/$5
printf 'regenerated\n' > "$dir/modules.dep"
printf 'new index\n' > "$dir/modules.softdep"
if [[ -f $2/lib/modules/$5/updates/rp1-hal/scmi-core.ko ]]; then
    cat "$2/lib/modules/$5/updates/rp1-hal/scmi-core.ko" >> "$dir/modules.dep"
fi
EOF
chmod +x "$test_dir/bin/"*
export PATH="$test_dir/bin:$PATH"

prepare() { bash "$here/realos/prepare-rootfs.sh" "$@"; }
reject() {
    if "$@" > "$test_dir/error.log" 2>&1; then
        printf 'expected failure from %s\n' "$*" >&2
        exit 1
    fi
}

for layout in traditional merged; do
    work=$test_dir/$layout
    export ROOTFS=$work/root BACKUP_DIR=$work/backup
    export SCMI_MODULES=$work/modules CLIENT_BIN=$work/clients CONSOLE_TTY=ttyAMA42
    export TFTP_DIR=$work/tftp
    mkdir -p "$ROOTFS"/{etc/initramfs-tools/{conf.d,hooks},etc/systemd/system,usr/local/sbin} \
        "$TFTP_DIR/empty" "$SCMI_MODULES" "$CLIENT_BIN"
    if [[ $layout == merged ]]; then
        mkdir -p "$ROOTFS/usr/lib"
        ln -s usr/lib "$ROOTFS/lib"
    fi
    release=$ROOTFS/lib/modules/6.12-fixture
    mkdir -p "$release"
    fstab=$ROOTFS/etc/fstab
    client=$ROOTFS/usr/local/bin/linux-app
    check=$ROOTFS/usr/local/sbin/rp1-realos-check
    tty=$ROOTFS/etc/systemd/system/serial-getty@ttyAMA42.service.d
    target=$ROOTFS/etc/systemd/system/default.target
    initramfs=$ROOTFS/etc/initramfs-tools-rp1nfs
    printf 'ID=fixture\n' > "$ROOTFS/etc/os-release"
    printf 'PARTUUID=fixture / ext4 defaults 0 1\n' > "$fstab"
    ln "$fstab" "$fstab.hardlink"
    ln -s /missing/original "$target"
    printf 'old index\n' > "$release/modules.dep"
    printf 'untouched\n' > "$release/modules.custom"
    printf 'old check\n' > "$check"
    chmod 0711 "$check"
    printf 'new module\n' > "$SCMI_MODULES/scmi-core.ko"
    printf 'new app\n' > "$CLIENT_BIN/linux-app"
    printf 'new status client\n' > "$CLIENT_BIN/linux-status"
    printf 'original TFTP\n' > "$TFTP_DIR/Å config"
    ln "$TFTP_DIR/Å config" "$TFTP_DIR/hardlink"
    ln -s 'Å config' "$TFTP_DIR/alias"
    python3 - "$TFTP_DIR/Å config" <<'PY'
import os
import sys
os.setxattr(sys.argv[1], "user.z", b"one")
os.setxattr(sys.argv[1], "user.a", b"two")
PY
    if [[ $layout == merged ]]; then
        mkdir -p "$(dirname "$client")" "$initramfs/hooks"
        printf 'old client\n' > "$client"
        printf 'prior initramfs configuration\n' > "$initramfs/initramfs.conf"
        printf 'prior hook\n' > "$initramfs/hooks/local"
        printf 'older legacy backup\n' > "$fstab.orig-realos"
    fi

    reject env -u BACKUP_DIR bash "$here/realos/prepare-rootfs.sh"
    reject env BACKUP_DIR="$ROOTFS/backup" bash "$here/realos/prepare-rootfs.sh"
    [[ ! -e $ROOTFS/backup && ! -e $release/modules.softdep && -L $target && ! -e $tty ]]
    cmp "$fstab" "$fstab.hardlink"

    LC_ALL=C prepare
    [[ -f $BACKUP_DIR/manifest.json && -f $BACKUP_DIR/manifest.sha256 ]]
    cp "$BACKUP_DIR/manifest.json" "$work/manifest"
    [[ $(readlink "$target") == /usr/lib/systemd/system/multi-user.target ]]
    [[ -f $tty/autologin.conf && $(cat "$client") == 'new app' ]]
    [[ $(cat "$ROOTFS/usr/local/libexec/rp1-lab/linux-status") == 'new status client' ]]
    [[ $(cat "$release/modules.softdep") == 'new index' ]]
    grep -Fxq 'new module' "$release/modules.dep"
    grep -q '^PARTUUID=' "$fstab.hardlink"

    rm "$TFTP_DIR/alias" "$TFTP_DIR/Å config" "$TFTP_DIR/hardlink"
    rmdir "$TFTP_DIR/empty"
    mkdir "$TFTP_DIR/alias"
    printf 'staged\n' > "$TFTP_DIR/alias/new"
    printf 'new boot file\n' > "$TFTP_DIR/new"
    printf 'unrelated runtime file\n' > "$ROOTFS/usr/local/bin/runtime"
    printf 'changed outside scope\n' > "$release/modules.custom"
    if [[ $layout == merged ]]; then
        printf 'changed hook\n' > "$initramfs/hooks/local"
    fi

    LC_ALL=C.UTF-8 prepare
    cmp "$work/manifest" "$BACKUP_DIR/manifest.json"
    reject env CONSOLE_TTY=ttyAMA43 bash "$here/realos/prepare-rootfs.sh"
    [[ ! -e $ROOTFS/etc/systemd/system/serial-getty@ttyAMA43.service.d ]]
    mv "$BACKUP_DIR/tftp/alias" "$BACKUP_DIR/tftp/saved-link"
    ln -s wrong "$BACKUP_DIR/tftp/alias"
    reject prepare --restore
    [[ -f $tty/autologin.conf && -f $TFTP_DIR/new ]]
    mv -fT "$BACKUP_DIR/tftp/saved-link" "$BACKUP_DIR/tftp/alias"

    saved_config="$BACKUP_DIR/tftp/Å config"
    python3 -c 'import os, sys; os.setxattr(sys.argv[1], "user.z", b"bad")' "$saved_config"
    reject prepare --restore
    [[ -f $tty/autologin.conf && -f $TFTP_DIR/new ]]
    python3 - "$saved_config" <<'PYTHON'
import os
import sys

path = sys.argv[1]
# Recreate the attributes in the opposite order from the source.
os.removexattr(path, "user.z")
os.removexattr(path, "user.a")
os.setxattr(path, "user.a", b"two")
os.setxattr(path, "user.z", b"one")
PYTHON

    reject env TFTP_DIR="$work" bash "$here/realos/prepare-rootfs.sh" --restore
    [[ -f $tty/autologin.conf && -f $TFTP_DIR/new ]]
    LC_ALL=C.UTF-8 prepare --restore
    LC_ALL=C prepare --restore
    cmp "$work/manifest" "$BACKUP_DIR/manifest.json"
    [[ $(readlink "$target") == /missing/original ]]
    [[ $(cat "$check") == 'old check' && $(stat -c %a "$check") == 711 ]]
    [[ ! -e $tty && ! -e $release/modules.softdep && ! -e $release/updates ]]
    [[ $(cat "$release/modules.dep") == 'old index' ]]
    [[ $(cat "$release/modules.custom") == 'changed outside scope' ]]
    [[ ! -e $ROOTFS/usr/local/libexec && ! -e $ROOTFS/usr/local/sbin/rp1-device-check ]]
    [[ -f $ROOTFS/usr/local/bin/runtime ]]
    cmp "$fstab" "$fstab.hardlink"
    [[ $(readlink "$TFTP_DIR/alias") == 'Å config' && -d $TFTP_DIR/empty ]]
    [[ "$TFTP_DIR/Å config" -ef "$TFTP_DIR/hardlink" && ! -e $TFTP_DIR/new ]]
    if [[ $layout == traditional ]]; then
        [[ ! -e $client && ! -e $initramfs && ! -e $fstab.orig-realos ]]
    else
        [[ $(cat "$client") == 'old client' && $(cat "$initramfs/hooks/local") == 'changed hook' ]]
        [[ $(cat "$initramfs/initramfs.conf") == 'prior initramfs configuration' ]]
        [[ $(cat "$fstab.orig-realos") == 'older legacy backup' ]]
    fi

    # The initramfs builder must reject a persistent output inside the root
    # before it can mount anything or invoke the chroot.
    reject env OUT="$ROOTFS/etc/initramfs" bash "$here/realos/build-initramfs.sh"
    grep -q 'OUT must be outside ROOTFS' "$test_dir/error.log"
done

# Root-only callers use the same transaction without a TFTP directory.
unset TFTP_DIR
export BACKUP_DIR=$work/root-only
prepare
[[ ! -e $BACKUP_DIR/tftp && -f $tty/autologin.conf ]]
prepare --restore
[[ ! -e $tty ]]
