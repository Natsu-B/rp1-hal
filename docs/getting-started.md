# Getting started

Boot the [rp1-hal example template](https://github.com/Natsu-B/rp1-hal-example) on a Raspberry Pi CM5 Lite
and call its polling `ping` RPC from Linux. Start with TFTP/NFS so that you can replace the boot files
without writing an SD card.

**HW · 2026-11-27.** On a CM5 Lite with an IO board, P1-9 boot B ran the template's polling RPC
for 1,000 replies on Raspberry Pi OS Lite arm64 (trixie) over TFTP/NFS. That run used HAL candidate
`e961f78`. V7 boots B and C used the same stock kernel release and NFS setup to run the peripheral
and IRQ RPC/channel examples at `aa6fe43` and `c5ec941`, with EEPROM bootloader `9cd61c53`
(2026-04-14). The steps below use the template at
`0ba84ed`, pinned to HAL `17fe413`. The full replay with this pin is pending.

## 1. Before you start

- Use a CM5 Lite and IO board with a boot order that includes NETWORK for TFTP or SD for the manual
  path. Check the bootloader's UART output to see which mode it selected.
- Use the stock 2712 arm64 kernel, its CM5 Lite IO board DTB (`bcm2712-rpi-cm5l-cm5io.dtb`) and the
  overlays shipped with that kernel. The template uses `clock.authority = "rp1"` and owns RP1
  UART0 on GPIO14/15. Check any extra overlays against your firmware's pin declarations.
- **Hands-on.** Connect a 3.3 V serial adapter RX to GPIO14 (RP1 UART0 TX) and connect ground.
  Leave GPIO15 free for this guide. Connect a second 3.3 V adapter to the IO board debug UART
  (BCM2712 UART10). Both ports use 115200 8N1. **Hands-on.** Isolate the CM5 Ethernet link from
  other networks before enabling DHCP or exporting the lab root.
- Use a writable, disposable copy of the arm64 trixie root filesystem for NFS. Prepare a spare,
  working SD card only if following step 6. Keep a backup of the working boot files and enough
  disk space to restore them. You need sudo on the host for rootfs and boot staging, and Git read
  access to the template and its HAL dependency. If Cargo needs your Git credentials, set
  `CARGO_NET_GIT_FETCH_WITH_CLI=true`.

On the Debian/Ubuntu build host, install the equivalents of these packages and have rustup available.
On an arm64 host use `gcc-14` and `binutils`. On x86 also install `gcc-14-aarch64-linux-gnu`,
`binutils-aarch64-linux-gnu`, `qemu-user` and `qemu-user-binfmt`.

```sh
sudo apt-get install git build-essential bc bison flex libssl-dev libelf-dev kmod xz-utils \
  python3 llvm device-tree-compiler binutils-arm-none-eabi initramfs-tools-core \
  rsync nfs-common rpcbind picocom
```

Run host commands in Bash. Start at the root of this `rp1-hal` checkout and keep the shell variables
throughout. Stop a step on an error. For SD, use the mounted ext4 partition as `ROOTFS` in steps 2–4.

```bash
HAL=$PWD
WORK=$(mktemp -d "$HAL/../rp1-start.XXXXXX")
read -erp 'Path to the offline arm64 root filesystem: ' ROOTFS
ROOTFS=$(realpath -e "$ROOTFS")
test "$ROOTFS" != /
read -erp 'Path to the matching stock boot files: ' STOCK_BOOT
STOCK_BOOT=$(realpath -e "$STOCK_BOOT")
export CARGO_BUILD_JOBS=2
```

## 2. Build SCMI modules

The stock kernel in this run lacks the SCMI client modules. The retained RP1 devices need SCMI
clocks, including Ethernet before DHCP. Use the same Debian version for the kernel, flavor
headers, common headers, kbuild and source. On the target, check `uname -r` and
`dpkg-query -W "linux-image-$(uname -r)"`. For the offline root, check the package database and
kernel files on the host.

```bash
KERNEL_RELEASE=6.12.75+rpt-rpi-2712
DEB_VERSION=1:6.12.75-1+rpt1
test "$(dpkg-query --admindir="$ROOTFS/var/lib/dpkg" -W -f='${Version}' \
  "linux-image-$KERNEL_RELEASE")" = "$DEB_VERSION"
test -r "$ROOTFS/boot/vmlinuz-$KERNEL_RELEASE"
grep -E 'ARM_SCMI_PROTOCOL|COMMON_CLK_SCMI|CONFIG_MODVERSIONS' "$ROOTFS/boot/config-$KERNEL_RELEASE"
```

If you need the inputs, download them from a Raspberry Pi OS arm64 APT environment with that
package version available. Copy the header packages and the source package to the build host.
Keep the release's DTB and overlays with the stock boot files when staging the kernel.

```bash
apt-get download "linux-headers-$KERNEL_RELEASE=$DEB_VERSION" \
  "linux-headers-6.12.75+rpt-common-rpi=$DEB_VERSION" \
  "linux-kbuild-6.12.75+rpt=$DEB_VERSION" "linux-source-6.12=$DEB_VERSION"
```

On the build host, enter a directory containing only those four DEBs. Verify their versions before
extracting them. If you already have the matching headers and extracted source, set `KHEADERS`
and `KSRC` to those paths and continue at the release check.

```bash
mkdir "$WORK/packages"
(
  set -e
  for pkg in "linux-headers-$KERNEL_RELEASE" linux-headers-6.12.75+rpt-common-rpi \
    linux-kbuild-6.12.75+rpt linux-source-6.12; do
    deb=$(realpath -e "$pkg"_*.deb)
    test "$(dpkg-deb -f "$deb" Version)" = "$DEB_VERSION"
    dpkg-deb -x "$deb" "$WORK/packages"
  done
)
KHEADERS="$WORK/packages/usr/src/linux-headers-$KERNEL_RELEASE"
tar -xJf "$WORK/packages/usr/src/linux-source-6.12.tar.xz" -C "$WORK"
KSRC="$WORK/linux-source-6.12"
```

The [SCMI build helper](../lab/realos/build-scmi-modules.sh) modifies the extracted source tree.
It starts from the headers' `.config`, selects the SCMI protocol, mailbox transport and clock
driver as modules, then runs `olddefconfig` and `modules_prepare`. It builds the SCMI modules with
the stock `Module.symvers`, then builds `clk-scmi` with the SCMI symbol CRCs. Keep the stock
kernel image. Use GCC 14 for arm64 (`aarch64-linux-gnu-gcc-14` by default, or set `KCC=gcc-14` on
arm64).

```bash
test "$(cat "$KHEADERS/include/config/kernel.release")" = "$KERNEL_RELEASE"
test -s "$KHEADERS/Module.symvers"
grep -H -E '^(CONFIG_|# CONFIG_)(ARM_SCMI_PROTOCOL|COMMON_CLK_SCMI|MODVERSIONS|ARM64_16K_PAGES)' \
  "$KHEADERS/.config" "$ROOTFS/boot/config-$KERNEL_RELEASE"
SCMI_MODULES="$WORK/scmi"
KSRC="$KSRC" KHEADERS="$KHEADERS" OUT="$SCMI_MODULES" bash "$HAL/lab/realos/build-scmi-modules.sh"
for module in "$SCMI_MODULES"/{scmi-core,scmi-module,scmi_transport_mailbox,clk-scmi}.ko; do
  modinfo -F vermagic "$module"
  sha256sum "$module"
done
```

## 3. Install and prepare Linux

Install the four modules in `/lib/modules/<release>/updates/rp1-hal/` and run
`depmod -b "$ROOTFS" "$KERNEL_RELEASE"`. Step 5 does this with the rootfs helper. Step 6 does it
manually. The load order is `rp1-mailbox`, `scmi-core`, `scmi-module`,
`scmi_transport_mailbox`, then `clk-scmi`. Check built-in drivers in
`$ROOTFS/lib/modules/$KERNEL_RELEASE/modules.builtin` as well as loadable modules.

`rp1boot` applies the ELF's generated `.rp1.dtbo` to the stock device tree. It adds
`/firmware/scmi` with RP1 mailbox channel 0, shared SRAM and clock protocol `@14`, and switches
the retained clock consumers. Use the stock DTB directly. The generated overlay supplies the SCMI
entries. For userspace, load `uio_pdrv_genirq` with `of_id=generic-uio`. Its `uio` dependency is
loaded by modprobe when it is a module. The client needs access to the matching `/dev/uio*` nodes.

## 4. Build firmware and boot set inputs

Keep `cargo-rp1`, the template firmware, the Linux client and `rp1boot` on the template's HAL pin.
The template uses polling RPC, so the first run needs no external peer.

```bash
TEMPLATE="$WORK/example"
git clone https://github.com/Natsu-B/rp1-hal-example "$TEMPLATE"
git -C "$TEMPLATE" checkout --detach 0ba84ed9d380dded2f6f437d669a62b13aa2ff56
cd "$TEMPLATE"
cargo install --locked --git https://github.com/Natsu-B/rp1-hal \
  --rev 17fe4133871795299a22969a92b274919952f5a2 --root "$WORK/tools" cargo-rp1
export PATH="$WORK/tools/bin:$PATH"
cargo rp1 build --locked -p firmware --target thumbv7m-none-eabi --release
cargo build-linux
bash scripts/build-rp1boot.sh "$WORK/loader"
RP1_ELF="$TEMPLATE/target/thumbv7m-none-eabi/release/firmware"
RP1BOOT_IMG="$WORK/loader/rp1boot.img"
LINUX_APP="$TEMPLATE/target/aarch64-unknown-linux-musl/release/linux-app"
mkdir "$WORK/clients"
install -m 0755 "$LINUX_APP" "$WORK/clients/linux-app"
sha256sum "$RP1_ELF" "$RP1BOOT_IMG" "$LINUX_APP"
cargo fetch --locked --manifest-path "$HAL/Cargo.toml" \
  --target "$(rustc -vV | sed -n 's/^host: //p')"
```

Check the ELF size and its ownership against the selected stock board tree. This is the host
check uses the D0 board revision overlay selected by the tested boot firmware and the NFS
helper's explicit `uart1-pi5` overlay. Set `BOARD_REVISION_OVERLAY` empty if your board does
not select a revision overlay. Include any additional overlays or use a matching
firmware-merged base DTB. The loader checks the final tree again at boot.

```bash
BOARD=bcm2712-rpi-cm5l-cm5io.dtb
BOARD_REVISION_OVERLAY=overlays/bcm2712d0.dtbo
BOARD_DTB="$STOCK_BOOT/$BOARD"
if [[ -n $BOARD_REVISION_OVERLAY ]]; then
  fdtoverlay -i "$BOARD_DTB" -o "$WORK/board.dtb" "$STOCK_BOOT/$BOARD_REVISION_OVERLAY"
  BOARD_DTB="$WORK/board.dtb"
fi
fdtoverlay -i "$BOARD_DTB" -o "$WORK/nfs-board.dtb" \
  "$STOCK_BOOT/overlays/uart1-pi5.dtbo"
cargo rp1 check "$RP1_ELF" --dtb "$WORK/nfs-board.dtb"
```

Both paths generate `config.txt`, `cmdline.txt`, `rp1boot.img` and `rp1-bundle.img`. The boot
firmware reads the loader as `kernel=rp1boot.img` and the bundle as
`initramfs rp1-bundle.img followkernel`. The Linux initramfs, when used, is a separate payload
inside the bundle. Keep `rp1.elf` and `manifest.txt` to identify the build. The bundle accepts a
raw or gzipped arm64 Image and a single cmdline of at most 880 bytes. It adds `rp1.bundle=` itself.

## 5. Network boot with TFTP/NFS

On the isolated network, use a DHCP/TFTP service bound to the CM5 link and an NFSv3 server.
Configure the DHCP lease and TFTP server address for that link. Use the directory where the
bootloader actually requests `config.txt`, including any EEPROM `TFTP_PREFIX`. Use a writable
export with `rw,no_root_squash` restricted to the CM5's lease address. For example, an exports
entry for a CM5 at `192.168.50.25` is
`/srv/nfs/cm5 192.168.50.25(rw,sync,no_subtree_check,no_root_squash)`.
Apply the exports with `sudo exportfs -ra` if you changed the configuration.

```bash
ip -br addr
read -rp 'NFS server IPv4 address on the CM5 link: ' SERVER
read -erp 'Effective TFTP directory for this CM5: ' TFTP_DIR
TFTP_DIR=$(realpath -e "$TFTP_DIR")
test "$TFTP_DIR" != /
rpcinfo -t "$SERVER" nfs 3
sudo exportfs -v
```

Use the [rootfs helper](../lab/realos/prepare-rootfs.sh) on the offline copy. It adjusts fstab and
network setup for NFS, sets up UIO and enables serial root autologin on UART10 for this isolated
lab root. The [initramfs helper](../lab/realos/build-initramfs.sh) runs the copy's own
`mkinitramfs` in a chroot. On x86, verify the `qemu-aarch64` binfmt entry is enabled and its
flags include `F` in `/proc/sys/fs/binfmt_misc/qemu-aarch64`.

```bash
sudo env ROOTFS="$ROOTFS" SCMI_MODULES="$SCMI_MODULES" CLIENT_BIN="$WORK/clients" \
  bash "$HAL/lab/realos/prepare-rootfs.sh"
INITRAMFS="$WORK/initramfs-$KERNEL_RELEASE"
sudo env ROOTFS="$ROOTFS" KERNEL_RELEASE="$KERNEL_RELEASE" OUT="$INITRAMFS" \
  bash "$HAL/lab/realos/build-initramfs.sh"
lsinitramfs "$INITRAMFS" | grep -F "lib/modules/$KERNEL_RELEASE/updates/rp1-hal/"
```

Check for all four modules. The initramfs uses `MODULES=list`, `BOOT=nfs` and xz compression.
It loads SCMI before DHCP on `eth0`. Leave `ip=` out of the kernel cmdline so the kernel
does not start IP autoconfiguration before those drivers load. The template's `scripts/bundle.sh`
defaults are for a small standalone initramfs.

```bash
ROOTFS="$ROOTFS" KERNEL_RELEASE="$KERNEL_RELEASE" RP1_ELF="$RP1_ELF" \
  RP1BOOT_IMG="$RP1BOOT_IMG" INITRAMFS="$INITRAMFS" NFS_ROOT="$SERVER:$ROOTFS" \
  OUT="$WORK/bootset" RP1_BOARD_CHECK_DTB="$WORK/nfs-board.dtb" \
  bash "$HAL/lab/realos/build-bootset.sh"
(cd "$WORK/bootset" && sha256sum -c SHA256SUMS)
```

The [boot set helper](../lab/realos/build-bootset.sh) sets
`root=/dev/nfs nfsroot=<server>:<export>,nfsvers=3 rw` and sets the UART10 and UIO options.
Verify the served stock DTB and overlay match the checked files. Check any board-revision overlay
the boot firmware selects as well. With the CM5 idle, back up the whole effective TFTP directory
before replacing the boot set.

```bash
cmp "$STOCK_BOOT/$BOARD" "$TFTP_DIR/$BOARD"
if [[ -n $BOARD_REVISION_OVERLAY ]]; then
  cmp "$STOCK_BOOT/$BOARD_REVISION_OVERLAY" "$TFTP_DIR/$BOARD_REVISION_OVERLAY"
fi
cmp "$STOCK_BOOT/overlays/uart1-pi5.dtbo" "$TFTP_DIR/overlays/uart1-pi5.dtbo"
hash_tree() { (cd "$1" && find . -type f -print0 | sort -z | xargs -0r sha256sum); }
hash_tree "$TFTP_DIR" > "$WORK/tftp-before.sha256"
mkdir "$WORK/tftp-save"
sudo rsync -a "$TFTP_DIR/" "$WORK/tftp-save/"
(cd "$WORK/tftp-save" && sha256sum -c "$WORK/tftp-before.sha256")
(cd "$WORK/bootset/boot" && sha256sum config.txt cmdline.txt rp1boot.img rp1-bundle.img rp1.elf manifest.txt) \
  > "$WORK/staged.sha256"
sudo rsync -a --delay-updates "$WORK/bootset/boot/" "$TFTP_DIR/"
(cd "$TFTP_DIR" && sha256sum -c "$WORK/staged.sha256")
```

**Hands-on.** Open both serial consoles before starting or resetting the CM5. Run this in
a separate host terminal for each adapter. Check for `Boot mode: NETWORK` and DHCP/TFTP
requests for the staged files, then go to step 7.

```bash
read -erp 'Serial device for this console: ' UART
picocom -b 115200 "$UART"
```

## 6. Manual SD boot

**HW · 2026-11-17.** P0-4 booted the older `91292ee` firmware from SD on the stock 2712 kernel
above, mounted ext4 without a Linux initramfs and loaded SCMI from the SD root.
**HYP · v0.2.** This manual path has not been replayed with the template or the V7 boot sets.
If your kernel needs an initramfs to find the SD root, use a matching local-root initramfs with
`--initramfs`. That variant also needs a replay. Use the NFS rootfs helper only for step 5.

**Hands-on.** Insert a spare, bootable card in a host reader. Identify the FAT boot and ext4 root
partitions with `lsblk -f` and mount them separately. Back up both partitions to a separate disk
with room for the full root. Use the card's ext4 mount as `ROOTFS` in steps 2–4. Leave its
`fstab` and normal login intact. Get the PARTUUID from the mounted partition and compare it
with the stock cmdline and fstab before staging.

```bash
read -erp 'Mounted SD FAT partition: ' SD_BOOT
read -erp 'New backup directory on a separate disk: ' SD_BACKUP
test "$(findmnt -n -o FSTYPE --mountpoint "$SD_BOOT")" = vfat
test "$(findmnt -n -o FSTYPE --mountpoint "$ROOTFS")" = ext4
mkdir "$SD_BACKUP"
SD_DEV=$(findmnt -n -o SOURCE --mountpoint "$ROOTFS")
PARTUUID=$(sudo blkid -s PARTUUID -o value "$SD_DEV")
test -n "$PARTUUID" && grep -Fw "root=PARTUUID=$PARTUUID" "$SD_BOOT/cmdline.txt"
grep -E '^CONFIG_(MMC|MMC_BLOCK|MMC_SDHCI|MMC_SDHCI_PLTFM|MMC_SDHCI_BRCMSTB|EXT4_FS)=' \
  "$ROOTFS/boot/config-$KERNEL_RELEASE"
sudo rsync -rt "$SD_BOOT/" "$SD_BACKUP/boot/"
sudo rsync -aHAX --numeric-ids --one-file-system "$ROOTFS/" "$SD_BACKUP/root/"
```

For the no-initramfs path, the MMC block, board SDHCI and ext4 drivers must show `=y`.
The module list loads the clock provider from ext4 and makes the UIO nodes available at login.

```bash
m="$ROOTFS/lib/modules/$KERNEL_RELEASE/updates/rp1-hal"
sudo install -d "$m"
sudo install -m 0644 "$SCMI_MODULES"/{scmi-core,scmi-module,scmi_transport_mailbox,clk-scmi}.ko "$m/"
sudo depmod -b "$ROOTFS" "$KERNEL_RELEASE"
printf '%s\n' rp1-mailbox scmi-core scmi-module scmi_transport_mailbox clk-scmi uio_pdrv_genirq |
  sudo tee "$ROOTFS/etc/modules-load.d/rp1-hal.conf" >/dev/null
printf '%s\n' 'options uio_pdrv_genirq of_id=generic-uio' |
  sudo tee "$ROOTFS/etc/modprobe.d/rp1-hal.conf" >/dev/null
sudo install -D -m 0755 "$LINUX_APP" "$ROOTFS/usr/local/bin/linux-app"
SD_CMDLINE="earlycon=pl011,0x107d001000,115200n8 console=ttyAMA10,115200 root=PARTUUID=$PARTUUID rootfstype=ext4 rootwait rw boot=local uio_pdrv_genirq.of_id=generic-uio systemd.mask=rpi-eeprom-update.service"
cmp "$STOCK_BOOT/$BOARD" "$SD_BOOT/$BOARD"
if [[ -n $BOARD_REVISION_OVERLAY ]]; then
  cmp "$STOCK_BOOT/$BOARD_REVISION_OVERLAY" "$SD_BOOT/$BOARD_REVISION_OVERLAY"
fi
cargo rp1 check "$RP1_ELF" --dtb "$BOARD_DTB"
test ! -e "$WORK/sd-bootset"
cargo rp1 bundle "$RP1_ELF" --kernel "$ROOTFS/boot/vmlinuz-$KERNEL_RELEASE" \
  --loader "$RP1BOOT_IMG" --cmdline "$SD_CMDLINE" \
  --config-extra device_tree_address=0x20000000 --out "$WORK/sd-bootset"
sudo rsync -rt "$WORK/sd-bootset/" "$SD_BOOT/"
sync
sudo umount "$SD_BOOT" "$ROOTFS"
```

**Hands-on.** With the CM5 off, insert the card. Disconnect the Ethernet link if NETWORK comes
before SD. Start the board and look for `Boot mode: SD` on UART10 after any network link
timeout. Use the card's normal login
and check for ext4 at step 7. To restore the spare, power off, return it to the reader and remount
the same partitions. Verify their device names and PARTUUIDs again before copying the backups
back with `--delete`. Reconnect Ethernet if you disconnected it.

```bash
sudo rsync -rt --delete "$SD_BACKUP/boot/" "$SD_BOOT/"
sudo rsync -aHAX --numeric-ids --one-file-system --delete "$SD_BACKUP/root/" "$ROOTFS/"
sync
sudo umount "$SD_BOOT" "$ROOTFS"
```

## 7. First run and troubleshooting

On UART10 look for `bundle ok` with the ID in `cmdline.txt`, then
`jumping to Linux at EL2`. On UART0 expect `proc0: platform ready`,
`proc1: hello`, then `proc0: from proc1: 42`. The manifest sets
`log = { sink = "uart0", baudrate = 115200 }`. Proc0 calls
`log!("proc0: platform ready {}\n", sys.report)` after successful platform init.

On the CM5 Linux console, run these commands as root (use sudo from a normal login).

```sh
uname -r
findmnt /
dmesg | grep -Ei 'scmi_mailbox_transport|SCMI Protocol|scmi.*clk'
ls -l /sys/bus/scmi_protocol/drivers/scmi-clocks/
for d in /sys/class/uio/uio*; do
  if [ -r "$d/name" ]; then printf '%s ' "${d##*/}"; cat "$d/name"; fi
done
/usr/local/bin/linux-app 10
printf 'linux-app exit=%s\n' "$?"
```

Expect the release above and an NFSv3 root for step 5 or ext4 for step 6. The Linux SCMI clock
driver should bind and the UIO names should include `rp1:link`, `rp1:header`,
`rp1:ping-dn` and `rp1:ping-up`. Expect ten per-call timings, a `ping rtt n=10` summary
and exit 0. The client checks that each reply equals the value it sent.

| Symptom | Check |
|---|---|
| No `rp1boot` banner | Check the selected boot mode, TFTP prefix or FAT root, and the staged `config.txt`, loader and bundle. Read the UART10 bootloader messages. |
| Loader rejects DT or bundle placement | Check the UART10 reason. Match config, cmdline, loader and bundle from one build. Rerun `cargo rp1 check` against the board DTB with the selected overlays. Resolve pin/console conflicts before staging again. |
| `invalid module format` or unknown symbol | Read dmesg and compare `uname -r`, module vermagic, package versions and the stock `Module.symvers`. Rebuild the modules and initramfs together after a kernel update. |
| Clock probe defers before DHCP | Check for all four SCMI modules in the NFS initramfs and a usable RP1 mailbox driver. Verify `BOOT=nfs`, the interface name, the SCMI node and the absence of `ip=` in the cmdline. |
| UIO missing or permission denied | Verify `modprobe uio_pdrv_genirq` and `of_id=generic-uio`, then compare sysfs names with the corresponding `/dev/uio*` permissions. Run the first check as root. |
| Linux console present, RP1 messages missing | Check the UART0 adapter on GPIO14 and ground at 115200. Linux and loader messages come from UART10. |

For NFS rollback, stop the test boot and restore the verified TFTP backup. Use the same confirmed
directory as step 5. Then boot the previous configuration and check its bundle ID and root.

```bash
(cd "$WORK/tftp-save" && sha256sum -c "$WORK/tftp-before.sha256")
sudo rsync -a --delete "$WORK/tftp-save/" "$TFTP_DIR/"
hash_tree "$TFTP_DIR" > "$WORK/tftp-restored.sha256"
cmp "$WORK/tftp-before.sha256" "$WORK/tftp-restored.sha256"
```

See the [peripheral examples](../examples/peripherals/README.md),
[IRQ RPC and channel example](../examples/linux-ping/README.md) and
[What Linux loses](linux-compatibility.md). Build and declaration details are in
[Build](build.md), [Loader](loader.md) and [Declarations and notes](declaration-and-note.md).
