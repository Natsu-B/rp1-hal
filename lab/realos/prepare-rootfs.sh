#!/usr/bin/env bash
# Adjust a Raspberry Pi OS (trixie) root file system copy so it boots as an NFS root under
# rp1-hal. Installs the SCMI modules and target checks, enables serial root autologin,
# and configures the NFS initramfs. Idempotent; run as root on the host.
#
# Selected configuration files and module indexes are saved as <file>.orig-realos (once).
# Use a disposable copy of the root file system for a complete rollback.
#
# Required: ROOTFS (the root file system copy), SCMI_MODULES (directory with the .ko files of
#           build-scmi-modules.sh).
# Optional: CONSOLE_TTY (default ttyAMA10), NET_IF (default eth0), CLIENT_BIN (directory of
#           Linux-side rp1-hal programs). linux-status goes to usr/local/libexec/rp1-lab;
#           other clients go to usr/local/bin.
set -euo pipefail
root=$(realpath -e -- "${ROOTFS:?set ROOTFS to the root file system copy}")
if [[ $root -ef / ]]; then
    echo "ROOTFS must be an offline copy, not the running root file system" >&2
    exit 1
fi
mods=${SCMI_MODULES:?set SCMI_MODULES to the SCMI module directory}
tty=${CONSOLE_TTY:-ttyAMA10}
netif=${NET_IF:-eth0}
[[ -f $root/etc/os-release ]] || { echo "$root is not a root file system" >&2; exit 1; }
release=$(modinfo -F vermagic "$mods/scmi-core.ko" | cut -d' ' -f1)
[[ -d $root/lib/modules/$release ]] || { echo "no modules for $release in $root" >&2; exit 1; }

save() { # save <path relative to root>, once
    local f=$root/$1
    if [[ ( -e $f || -L $f ) && ! ( -e $f.orig-realos || -L $f.orig-realos ) ]]; then
        cp -a "$f" "$f.orig-realos"
        echo "saved $1 -> $1.orig-realos"
    fi
}
put() { # put <path relative to root> <content>
    local f=$root/$1
    mkdir -p "$(dirname "$f")"
    if [[ ! -f $f ]] || [[ $(cat "$f") != "$2" ]]; then
        save "$1"
        printf '%s\n' "$2" > "$f"
        echo "wrote $1"
    fi
}
link() { # link <path relative to root> <target>
    local f=$root/$1
    if [[ $(readlink "$f" 2>/dev/null) != "$2" ]]; then
        save "$1"
        mkdir -p "$(dirname "$f")"
        ln -sfn "$2" "$f"
        echo "linked $1 -> $2"
    fi
}

# SCMI modules (the stock kernel has none) + module index.
dst=lib/modules/$release/updates/rp1-hal
mkdir -p "$root/$dst"
for m in "$mods"/*.ko; do
    if ! cmp -s "$m" "$root/$dst/$(basename "$m")"; then
        install -m 0644 "$m" "$root/$dst/"
        echo "installed $dst/$(basename "$m")"
    fi
done
for f in modules.dep modules.dep.bin modules.alias modules.alias.bin modules.symbols modules.symbols.bin; do
    save "lib/modules/$release/$f"
done
depmod -b "$root" "$release"
echo "depmod $release"

# fstab: the SD card is not this system's root.
if grep -qE '^[[:space:]]*(PARTUUID=|UUID=|LABEL=|/dev/mmcblk)' "$root/etc/fstab"; then
    save etc/fstab
    sed -i -E 's@^([[:space:]]*(PARTUUID=|UUID=|LABEL=|/dev/mmcblk))@#realos# \1@' "$root/etc/fstab"
    echo "commented SD entries in etc/fstab"
fi

link etc/systemd/system/default.target /usr/lib/systemd/system/multi-user.target
link etc/systemd/system/rpi-eeprom-update.service /dev/null
put etc/cloud/cloud-init.disabled "disabled for the rp1-hal NFS root (seed is on the SD card)"

put etc/NetworkManager/conf.d/90-rp1-nfsroot.conf "# rp1-hal NFS root: the initramfs configured $netif and the root file system uses it.
[keyfile]
unmanaged-devices=interface-name:$netif"

put "etc/systemd/system/serial-getty@$tty.service.d/autologin.conf" "[Service]
ExecStart=
ExecStart=-/sbin/agetty --autologin root --keep-baud 115200,57600,38400,9600 - \$TERM"

put etc/modules-load.d/rp1-hal.conf "# rp1-hal shared memory / doorbell UIO nodes (compatible \"generic-uio\")
uio_pdrv_genirq"
put etc/modprobe.d/rp1-hal.conf "options uio_pdrv_genirq of_id=generic-uio"

# initramfs-tools configuration for the NFS root initramfs (build-initramfs.sh).
conf=etc/initramfs-tools-rp1nfs
if [[ ! -d $root/$conf ]]; then
    cp -a "$root/etc/initramfs-tools" "$root/$conf"
    echo "copied etc/initramfs-tools -> $conf"
fi
put "$conf/initramfs.conf" "# rp1-hal NFS root initramfs (see build-initramfs.sh)
MODULES=list
BUSYBOX=auto
KEYMAP=n
COMPRESS=xz
DEVICE=$netif
NFSROOT=auto
BOOT=nfs
RUNSIZE=20%
FSTYPE=auto"
put "$conf/modules" "# RP1 mailbox + SCMI clocks: RP1 devices (Ethernet included) take their clocks from the
# rp1-hal firmware over SCMI. Loaded in this order by the initramfs.
rp1-mailbox
scmi-core
scmi-module
scmi_transport_mailbox
clk-scmi"
here=$(cd "$(dirname "$0")" && pwd)
install -D -m 0755 "$here/check-realos.sh" "$root/usr/local/sbin/rp1-realos-check"
echo "installed usr/local/sbin/rp1-realos-check"
install -m 0755 "$here/check-linux-devices.sh" "$root/usr/local/sbin/rp1-device-check"
echo "installed usr/local/sbin/rp1-device-check"
if [[ -n ${CLIENT_BIN:-} ]]; then
    shopt -s nullglob
    clients=("$CLIENT_BIN"/*)
    shopt -u nullglob
    (( ${#clients[@]} )) || { echo "CLIENT_BIN $CLIENT_BIN has no files" >&2; exit 1; }
    for f in "${clients[@]}"; do
        name=${f##*/}
        case "$name" in
            linux-status) dest=usr/local/libexec/rp1-lab/$name ;;
            *) dest=usr/local/bin/$name ;;
        esac
        install -D -m 0755 "$f" "$root/$dest"
        echo "installed $dest"
    done
fi
echo "prepare-rootfs done ($release)"
