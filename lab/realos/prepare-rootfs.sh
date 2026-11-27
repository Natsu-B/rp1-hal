#!/usr/bin/env bash
# Adjust a Raspberry Pi OS (trixie) root file system copy so it boots as an NFS root under
# rp1-hal. Installs the SCMI modules and target checks, enables serial root autologin,
# and configures the NFS initramfs. Idempotent; run as root on the host.
#
# BACKUP_DIR must be outside ROOTFS and TFTP_DIR, with an existing parent directory.
# Run with --restore and the same ROOTFS, BACKUP_DIR and optional TFTP_DIR to roll back.
#
# Required: ROOTFS (the root file system copy), SCMI_MODULES (directory with the .ko files of
#           build-scmi-modules.sh).
# Optional: TFTP_DIR (save the whole TFTP tree), CONSOLE_TTY (default ttyAMA10),
#           NET_IF (default eth0), CLIENT_BIN (directory of Linux-side rp1-hal programs).
#           linux-status goes to usr/local/libexec/rp1-lab, other clients to usr/local/bin.
set -euo pipefail
root=$(realpath -e -- "${ROOTFS:?set ROOTFS to the root file system copy}")
if [[ $root -ef / ]]; then
    echo "ROOTFS must be an offline copy, not the running root file system" >&2
    exit 1
fi
backup=${BACKUP_DIR:?set BACKUP_DIR to the backup directory}
tftp=${TFTP_DIR:-}
here=$(cd "$(dirname "$0")" && pwd)

# Serialize prepare and restore on the same offline trees.
exec {root_lock}<"$root"
flock -n "$root_lock"
if [[ -n $tftp ]]; then
    tftp=$(realpath -e -- "$tftp")
    if [[ $root == "$tftp" ]]; then
        echo "ROOTFS and TFTP_DIR must be separate trees" >&2
        exit 1
    fi
    exec {tftp_lock}<"$tftp"
    flock -n "$tftp_lock"
fi
if [[ ${1:-} == --restore && $# == 1 ]]; then
    python3 "$here/rootfs-backup.py" restore "$root" "$tftp" "$backup"
    exit
fi
if (( $# )); then
    echo "usage: prepare-rootfs.sh [--restore]" >&2
    exit 1
fi

backup=$(python3 "$here/rootfs-backup.py" validate "$root" "$tftp" "$backup")
mods=${SCMI_MODULES:?set SCMI_MODULES to the SCMI module directory}
tty=${CONSOLE_TTY:-ttyAMA10}
netif=${NET_IF:-eth0}
[[ $tty != */* ]] || { echo "CONSOLE_TTY must be a device name" >&2; exit 1; }
[[ -f $root/etc/os-release ]] || { echo "$root is not a root file system" >&2; exit 1; }
release=$(modinfo -F vermagic "$mods/scmi-core.ko" | cut -d' ' -f1)
[[ -n $release && $release != */* ]] || { echo "invalid module release" >&2; exit 1; }
module_dir=$(realpath -e -- "$root/lib/modules/$release")
[[ $module_dir == "$root/"* && -d $module_dir ]] || { echo "invalid module directory" >&2; exit 1; }
module_dir=${module_dir#"$root"/}
dst=$module_dir/updates/rp1-hal
conf=etc/initramfs-tools-rp1nfs
if [[ -L $root/$conf || ( -e $root/$conf && ! -d $root/$conf ) ]]; then
    echo "$conf must be a directory" >&2
    exit 1
fi

if [[ ! -d $root/$conf ]] && [[ ! -d $root/etc/initramfs-tools || -L $root/etc/initramfs-tools ]]; then
    echo "etc/initramfs-tools must be a directory" >&2
    exit 1
fi

shopt -s nullglob
modules=("$mods"/*.ko)
clients=()
if [[ -n ${CLIENT_BIN:-} ]]; then
    clients=("$CLIENT_BIN"/*)
    (( ${#clients[@]} )) || { echo "CLIENT_BIN $CLIENT_BIN has no files" >&2; exit 1; }
fi
for f in "${modules[@]}" "${clients[@]}"; do
    [[ -f $f ]] || { echo "not a file: $f" >&2; exit 1; }
done

# Discover depmod's output names in scratch space before changing ROOTFS.
# The next pass builds the actual indexes against the newly installed modules.
scratch=$(mktemp -d -- "$(dirname -- "$backup")/.rp1-depmod.XXXXXX")
trap 'rm -rf -- "$scratch"' EXIT
mkdir -p -- "$scratch/usr/lib/modules/$release"
ln -s usr/lib "$scratch/lib"
depmod -b "$root" -o "$scratch" "$release"
index_path=$scratch/lib/modules/$release
indexes=("$index_path"/*)
(( ${#indexes[@]} )) || { echo "depmod produced no indexes" >&2; exit 1; }
index_names=("${indexes[@]##*/}")
rm -- "${indexes[@]}"

# Each operation registers its own targets in the first pass.
planning=1
scope=()
save() { # save <path relative to root>, once
    if (( planning )); then scope+=("$1.orig-realos"); return; fi
    local f=$root/$1
    if [[ ( -e $f || -L $f ) && ! ( -e $f.orig-realos || -L $f.orig-realos ) ]]; then
        cp -aT -- "$f" "$f.orig-realos"
        echo "saved $1 -> $1.orig-realos"
    fi
}
put() { # put <path relative to root> <content>
    if (( planning )); then scope+=("$1"); save "$1"; return; fi
    local f=$root/$1
    mkdir -p "$(dirname "$f")"
    if [[ -L $f || ! -f $f || $(cat "$f") != "$2" ]]; then
        save "$1"
        if [[ -L $f || ! -e $f ]]; then
            install -m 0644 /dev/null "$scratch/config"
        else
            cp --attributes-only --preserve=all -- "$f" "$scratch/config"
        fi
        printf '%s\n' "$2" > "$scratch/config"
        mv -fT -- "$scratch/config" "$f"
        echo "wrote $1"
    fi
}
link() { # link <path relative to root> <target>
    if (( planning )); then scope+=("$1"); save "$1"; return; fi
    local f=$root/$1
    if [[ $(readlink "$f" 2>/dev/null) != "$2" ]]; then
        save "$1"
        mkdir -p "$(dirname "$f")"
        ln -sfnT -- "$2" "$f"
        echo "linked $1 -> $2"
    fi
}

install_file() { # install_file <source> <path relative to root> <mode>
    if (( planning )); then scope+=("$2"); return; fi
    install -DT -m "$3" -- "$1" "$root/$2"
    echo "installed $2"
}

prepare_modules() {
    local f
    for f in "${modules[@]}"; do
        install_file "$f" "$dst/${f##*/}" 0644
    done
    for f in modules.dep modules.dep.bin modules.alias modules.alias.bin modules.symbols modules.symbols.bin; do
        save "$module_dir/$f"
    done
    if (( ! planning )); then
        depmod -b "$root" -o "$scratch" "$release"
        local outputs=("$index_path"/*)
        local names=("${outputs[@]##*/}")
        if [[ $(printf '%s\n' "${names[@]}") != "$(printf '%s\n' "${index_names[@]}")" ]]; then
            echo "depmod output paths changed during preparation" >&2
            exit 1
        fi
    fi
    for f in "${index_names[@]}"; do
        install_file "$index_path/$f" "$module_dir/$f" 0644
    done
}

prepare() {
    local f name dest
    prepare_modules

    # fstab: the SD card is not this system's root.
    if (( planning )); then
        scope+=(etc/fstab)
        save etc/fstab
    elif grep -qE '^[[:space:]]*(PARTUUID=|UUID=|LABEL=|/dev/mmcblk)' "$root/etc/fstab"; then
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
    # The first run can copy this whole directory, including hooks and scripts.
    if (( ! planning )) && [[ ! -d $root/$conf ]]; then
        cp -aT -- "$root/etc/initramfs-tools" "$root/$conf"
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
    install_file "$here/check-realos.sh" usr/local/sbin/rp1-realos-check 0755
    install_file "$here/check-linux-devices.sh" usr/local/sbin/rp1-device-check 0755
    for f in "${clients[@]}"; do
        name=${f##*/}
        case "$name" in
            linux-status) dest=usr/local/libexec/rp1-lab/$name ;;
            *) dest=usr/local/bin/$name ;;
        esac
        install_file "$f" "$dest" 0755
    done
}

prepare
python3 "$here/rootfs-backup.py" save "$root" "$tftp" "$backup" "$conf" "${scope[@]}"
planning=0
prepare
echo "prepare-rootfs done ($release)"
