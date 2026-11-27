# Getting started

**HW · 2026-11-27.** An earlier build of the IRQ template returned 1,000 matching `ping` replies
and generated 1,000 Linux interrupts on a CM5 Lite with Raspberry Pi OS Lite arm64 (trixie) and
stock Linux `6.12.75+rpt-rpi-2712` over TFTP/NFS. [Evidence runs](linux-compatibility.md#evidence-runs)
**HW · 2026-11-27.** A separate TFTP/NFS replay of the pinned
[example template](https://github.com/Natsu-B/rp1-hal-example) returned ten replies and ten
interrupts, then booted the previous configuration. The backup and restore commands below
were revised after that replay.

## 1. Before you start

- Use a CM5 Lite and IO board with a boot order that includes NETWORK for TFTP or SD for the manual
  path. Check the bootloader's UART output to see which mode it selected.
- Use the stock 2712 arm64 kernel, its CM5 Lite IO board DTB (`bcm2712-rpi-cm5l-cm5io.dtb`) and the
  overlays shipped with that kernel. The template uses `clock.authority = "rp1"` and owns RP1
  UART0 on GPIO14/15. Keep GPIO27 free for the software doorbell and check any extra overlays
  against the firmware's pin declarations.
- **Hands-on.** Connect a 3.3 V serial adapter RX to GPIO14 (RP1 UART0 TX) and connect ground.
  Leave GPIO15 free for this guide. Connect a second 3.3 V adapter to the IO board debug UART
  (BCM2712 UART10). Both ports use 115200 8N1.
- **Hands-on.** Isolate the CM5 Ethernet link from other networks before enabling DHCP or
  exporting the lab root.
- For TFTP/NFS, start with a working boot set you can restore. It can be stock or a known-good
  experiment. Get the stock DTB and overlays separately, from the target OS's kernel package.
- Prefer a writable, disposable copy of the arm64 trixie root for NFS. Before copying, measure
  the original with `sudo du -shx` and check the destination with `df -h`.
  Allow space for the full copy, the backups and builds. The replay root used 7 GiB. If space
  is tight, use an existing lab root only if you can take it offline as described in step 4.
  Prepare a spare, working SD card for step 5. You need sudo on the host for rootfs and boot
  staging, and Git read access to the template and its HAL dependency. If Cargo needs your Git
  credentials, set `CARGO_NET_GIT_FETCH_WITH_CLI=true`.

On the Debian/Ubuntu build host, install the equivalents of these packages and have rustup available.
On an arm64 host use `gcc-14` and `binutils`. On x86 also install `gcc-14-aarch64-linux-gnu`,
`binutils-aarch64-linux-gnu`, `qemu-user` and `qemu-user-binfmt`.

```sh
sudo apt-get install git build-essential bc bison flex libssl-dev kmod xz-utils \
  python3 llvm device-tree-compiler binutils-arm-none-eabi initramfs-tools-core \
  rsync nfs-common rpcbind picocom
```

Install `libelf-dev` if your kernel build asks for ELF development headers.
For TFTP/NFS, also install `nfs-kernel-server` on the host that exports `ROOTFS`.
Run host commands in Bash and keep the shell variables throughout. Stop a step on an error.
Start in a clean `rp1-hal` checkout at the same revision as this guide. Keep its lab helpers
available for backup and rollback. Use a separate worktree at the template's HAL pin for the builds.
If you need a checkout, run `git clone https://github.com/Natsu-B/rp1-hal` and enter it.

```bash
HAL=$(pwd -P)
HAL_REV=2e4fedc0a68c7aeed2cdeeb67b60ae5e0f125e25
WORK=$(mktemp -d "$(dirname "$HAL")/rp1-start.XXXXXX")
HAL_PINNED="$WORK/hal"
git -C "$HAL" worktree add --detach "$HAL_PINNED" "$HAL_REV"
export CARGO_BUILD_JOBS=2
```

**Hands-on (SD only).** Insert a spare, bootable card in a host reader. Identify the FAT boot and
ext4 root partitions with `lsblk -f` and mount them separately. Use the ext4 mount as `ROOTFS`.
For `STOCK_BOOT`, use the matching stock FAT mount or a host copy of `/boot/firmware` from a
running Pi. Check the Pi's kernel release and package version as in step 2 before copying.
If the offline root has no populated `/boot/firmware`, leave the prompt blank to assemble the
files from its firmware and kernel packages.

```bash
read -erp 'Path to the offline arm64 root filesystem: ' ROOTFS
ROOTFS=$(realpath -e "$ROOTFS")
test "$ROOTFS" != /
read -erp 'Path to matching stock boot files (Enter to assemble): ' STOCK_BOOT
KERNEL_RELEASE=6.12.75+rpt-rpi-2712
BOARD=bcm2712-rpi-cm5l-cm5io.dtb
if [[ -n $STOCK_BOOT ]]; then
  STOCK_BOOT=$(realpath -e "$STOCK_BOOT")
else
  STOCK_BOOT="$WORK/stock-boot"
  KERNEL_BOOT="$ROOTFS/usr/lib/linux-image-$KERNEL_RELEASE"
  mkdir -p "$STOCK_BOOT/overlays"
  cp -a "$ROOTFS/usr/lib/raspi-firmware/." "$STOCK_BOOT/"
  cp -a "$KERNEL_BOOT/broadcom/." "$STOCK_BOOT/"
  cp -a "$KERNEL_BOOT/overlays/." "$STOCK_BOOT/overlays/"
  (
    set -e -o pipefail
    cd "$KERNEL_BOOT/broadcom"
    sha256sum "$BOARD"
    cd "$KERNEL_BOOT"
    sha256sum overlays/overlay_map.dtb
    find overlays -name '*.dtbo' -print0 | LC_ALL=C sort -z | xargs -0r sha256sum --
  ) > "$WORK/stock-source.sha256"
  (cd "$STOCK_BOOT" && sha256sum -c "$WORK/stock-source.sha256")
fi
```

### SD only

Back up both partitions to a separate disk with room for the full root. Get the root partition's
PARTUUID and verify that the stock cmdline and fstab point to that partition.

```bash
read -erp 'Mounted SD FAT partition: ' SD_BOOT
read -erp 'New backup directory on a separate disk: ' SD_BACKUP
test "$(findmnt -n -o FSTYPE --mountpoint "$SD_BOOT")" = vfat
test "$(findmnt -n -o FSTYPE --mountpoint "$ROOTFS")" = ext4
mkdir "$SD_BACKUP"
SD_DEV=$(findmnt -n -o SOURCE --mountpoint "$ROOTFS")
PARTUUID=$(sudo blkid -s PARTUUID -o value "$SD_DEV")
test -n "$PARTUUID" && grep -Fw "root=PARTUUID=$PARTUUID" "$SD_BOOT/cmdline.txt"
sudo rsync -rt "$SD_BOOT/" "$SD_BACKUP/boot/"
sudo rsync -aHAX --numeric-ids --one-file-system "$ROOTFS/" "$SD_BACKUP/root/"
```

## 2. Build SCMI modules

The stock `6.12.75+rpt-rpi-2712` kernel has no SCMI client modules. The retained RP1 devices need
SCMI clocks, including Ethernet before DHCP. Use the same Debian version for the kernel, flavor
headers and source. On the target, check `uname -r` and `dpkg-query -W "linux-image-$(uname -r)"`.
In the offline root's config, expect `# CONFIG_ARM_SCMI_PROTOCOL is not set`,
`CONFIG_MODVERSIONS=y` and `CONFIG_ARM64_16K_PAGES=y`. `COMMON_CLK_SCMI` should be absent.

```bash
DEB_VERSION=1:6.12.75-1+rpt1
test "$(dpkg-query --admindir="$ROOTFS/var/lib/dpkg" -W -f='${Version}' \
  "linux-image-$KERNEL_RELEASE")" = "$DEB_VERSION"
test -r "$ROOTFS/boot/vmlinuz-$KERNEL_RELEASE"
grep -E 'ARM_SCMI_PROTOCOL|COMMON_CLK_SCMI|CONFIG_MODVERSIONS|CONFIG_ARM64_16K_PAGES' \
  "$ROOTFS/boot/config-$KERNEL_RELEASE"
```

If you need the inputs, download them from a Raspberry Pi OS arm64 APT environment with that
package version available. Copy the flavor headers and source package to the build host.

```bash
apt-get download "linux-headers-$KERNEL_RELEASE=$DEB_VERSION" "linux-source-6.12=$DEB_VERSION"
```

On the build host, enter a directory containing only those two DEBs. Verify their versions before
extracting them. If you already have matching inputs, set `KHEADERS` and `KSRC` and continue at
the release check. The flavor headers may already be in `$ROOTFS/usr/src/linux-headers-$KERNEL_RELEASE`.
Check `linux-headers-$KERNEL_RELEASE` with the same `dpkg-query` used for the image. Reuse source
extracted from a verified `linux-source-6.12` DEB of the same version.

```bash
mkdir "$WORK/packages"
(
  set -e
  for pkg in "linux-headers-$KERNEL_RELEASE" linux-source-6.12; do
    deb=$(realpath -e "$pkg"_*.deb)
    test "$(dpkg-deb -f "$deb" Version)" = "$DEB_VERSION"
    dpkg-deb -x "$deb" "$WORK/packages"
  done
)
KHEADERS="$WORK/packages/usr/src/linux-headers-$KERNEL_RELEASE"
tar -xJf "$WORK/packages/usr/src/linux-source-6.12.tar.xz" -C "$WORK"
KSRC="$WORK/linux-source-6.12"
```

Before building, check for the same stock config values in both files below.

```bash
test "$(cat "$KHEADERS/include/config/kernel.release")" = "$KERNEL_RELEASE"
test -s "$KHEADERS/Module.symvers"
grep -H -E '^(CONFIG_|# CONFIG_)(ARM_SCMI_PROTOCOL|COMMON_CLK_SCMI|MODVERSIONS|ARM64_16K_PAGES)' \
  "$KHEADERS/.config" "$ROOTFS/boot/config-$KERNEL_RELEASE"
```

The [SCMI build helper](../lab/realos/build-scmi-modules.sh) modifies the extracted source, builds
four modules against the stock `Module.symvers` and checks their vermagic. Keep the stock kernel
image. Use GCC 14 for arm64 (`aarch64-linux-gnu-gcc-14` by default, or set `KCC=gcc-14` on arm64).

```bash
SCMI_MODULES="$WORK/scmi"
KSRC="$KSRC" KHEADERS="$KHEADERS" OUT="$SCMI_MODULES" bash "$HAL_PINNED/lab/realos/build-scmi-modules.sh"
for module in "$SCMI_MODULES"/{scmi-core,scmi-module,scmi_transport_mailbox,clk-scmi}.ko; do
  modinfo -F vermagic "$module"
  sha256sum "$module"
done
```

### How Linux gets RP1 clocks

Install the four modules in `/lib/modules/<release>/updates/rp1-hal/` and run
`depmod -b "$ROOTFS" "$KERNEL_RELEASE"`. Step 4 does this with the rootfs helper. Step 5 does it
manually. The load order is `rp1-mailbox`, `scmi-core`, `scmi-module`,
`scmi_transport_mailbox`, then `clk-scmi`. Check built-in drivers in
`$ROOTFS/lib/modules/$KERNEL_RELEASE/modules.builtin` as well as loadable modules.

`rp1boot` applies the ELF's generated `.rp1.dtbo` to the stock device tree. It adds
`/firmware/scmi` with RP1 mailbox channel 0, shared SRAM and clock protocol `@14`, and switches
the retained clock consumers. Use the stock DTB directly. The generated overlay supplies the SCMI
entries. For userspace, load `uio_pdrv_genirq` with `of_id=generic-uio`. Its `uio` dependency is
loaded by modprobe when it is a module.

## 3. Build firmware and boot set inputs

Keep `cargo-rp1`, the template firmware, the Linux client and `rp1boot` on the template's HAL pin.
The first run needs no external peer. Linux wakes RP1 through the software doorbell.
Proc1 sleeps between requests and Linux waits for replies on the UIO interrupt.

```bash
TEMPLATE="$WORK/example"
git clone https://github.com/Natsu-B/rp1-hal-example "$TEMPLATE"
git -C "$TEMPLATE" checkout --detach 365e1f98a187a06d08b71b914e0284fddd71b3b2
cd "$TEMPLATE"
cargo install --locked --git https://github.com/Natsu-B/rp1-hal \
  --rev "$HAL_REV" --root "$WORK/tools" cargo-rp1
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
```

Check the ELF size and its ownership against the selected stock board tree. This host check uses
the D0 board revision overlay that the tested boot firmware selects and the `uart1-pi5` overlay
that the NFS helper always adds. Set `BOARD_REVISION_OVERLAY` empty if your board does
not select a revision overlay. Include any additional overlays or use a matching
firmware-merged base DTB. The loader checks the final tree again at boot.

```bash
BOARD_REVISION_OVERLAY=overlays/bcm2712d0.dtbo
STOCK_DT=("$BOARD" overlays/overlay_map.dtb overlays/uart1-pi5.dtbo)
[[ -z $BOARD_REVISION_OVERLAY ]] || STOCK_DT+=("$BOARD_REVISION_OVERLAY")
(cd "$STOCK_BOOT" && sha256sum -- "${STOCK_DT[@]}") > "$WORK/stock-dt.sha256"
BOARD_DTB="$STOCK_BOOT/$BOARD"
if [[ -n $BOARD_REVISION_OVERLAY ]]; then
  fdtoverlay -i "$BOARD_DTB" -o "$WORK/board.dtb" "$STOCK_BOOT/$BOARD_REVISION_OVERLAY"
  BOARD_DTB="$WORK/board.dtb"
fi
fdtoverlay -i "$BOARD_DTB" -o "$WORK/nfs-board.dtb" \
  "$STOCK_BOOT/overlays/uart1-pi5.dtbo"
cargo rp1 check "$RP1_ELF" --dtb "$WORK/nfs-board.dtb"
```

Both paths generate the loader, bundle and matching boot configuration described in
[Loader](loader.md).

## 4. Network boot with TFTP/NFS

On the isolated network, use a DHCP/TFTP service bound to the CM5 link and an NFSv3 server.
Configure the DHCP lease and TFTP server address for that link. Use the directory where the
bootloader actually requests `config.txt`, including any EEPROM `TFTP_PREFIX`. Export `ROOTFS`
with `rw,no_root_squash` to the CM5's lease address. An existing subnet export can be reused
on a dedicated, isolated link whose only client is the CM5. For a new export with
`ROOTFS=/srv/nfs/cm5` and a CM5 at `192.0.2.25`, use
`/srv/nfs/cm5 192.0.2.25(rw,sync,no_subtree_check,no_root_squash)` in `/etc/exports`.
Apply changes with `sudo exportfs -ra`. Check the path, client and options in `exportfs -v`.

```bash
ip -br addr
read -rp 'NFS server IPv4 address on the CM5 link: ' SERVER
read -erp 'Effective TFTP directory for this CM5: ' TFTP_DIR
TFTP_DIR=$(realpath -e "$TFTP_DIR")
test "$TFTP_DIR" != /
rpcinfo -t "$SERVER" nfs 3
sudo exportfs -v
```

Record the working boot's identity on the CM5 before changing its boot files.

```sh
cat /proc/cmdline
cat /proc/sys/kernel/random/boot_id
findmnt -n -o SOURCE,FSTYPE,OPTIONS /
```

Keep the root offline during staging and rollback so the helpers can change files and
build the initramfs without racing a running system. Unmount the export on all clients
and stop host writes. Keep the CM5 from starting a TFTP boot until staging is done.

If the CM5 uses this root, run `sudo systemctl halt` and wait for the halt on UART10.
Start a maintenance initramfs or fall back to SD. For SD fallback, check the boot order,
keep power on, disconnect Ethernet and reset.
On a powered bench with the `cm5ctl.py` helper, use `./cm5ctl.py force-boot` from its
directory. Use the normal start control if the board is off. In the SD or maintenance
boot, check `findmnt /` and `findmnt -t nfs,nfs4` for a local or RAM root with the export
unmounted.

Keep a backup so you can boot the previous set. The [rootfs helper](../lab/realos/prepare-rootfs.sh)
saves the TFTP tree and the root paths it manages before changing the root. It verifies
and reuses that backup on retries. Keep `WORK` outside both trees and use a new one for
each experiment. Root preparation configures NFS, UIO and root autologin on UART10.

The [initramfs helper](../lab/realos/build-initramfs.sh) runs the offline root's own
`mkinitramfs` in a chroot. On x86, verify the `qemu-aarch64` binfmt entry is enabled and its
flags include `F` in `/proc/sys/fs/binfmt_misc/qemu-aarch64`.

```bash
BACKUP_DIR="$WORK/before"
sudo env ROOTFS="$ROOTFS" TFTP_DIR="$TFTP_DIR" BACKUP_DIR="$BACKUP_DIR" \
  SCMI_MODULES="$SCMI_MODULES" CLIENT_BIN="$WORK/clients" bash "$HAL/lab/realos/prepare-rootfs.sh"
INITRAMFS="$WORK/initramfs-$KERNEL_RELEASE"
sudo env ROOTFS="$ROOTFS" KERNEL_RELEASE="$KERNEL_RELEASE" OUT="$INITRAMFS" \
  bash "$HAL/lab/realos/build-initramfs.sh"
lsinitramfs "$INITRAMFS" | grep -F "lib/modules/$KERNEL_RELEASE/updates/rp1-hal/"
```

Check for all four modules. The initramfs uses `MODULES=list`, `BOOT=nfs` and xz compression.
It loads SCMI before DHCP on `eth0`. Leave `ip=` out of the kernel cmdline so the kernel
does not start IP autoconfiguration before those drivers load. The template's `scripts/bundle.sh`
defaults are for a small standalone initramfs.

The NFS boot set helper builds `cargo-rp1` offline in `$HAL_PINNED/target`. Fetch its host dependencies
first. The HAL, template and loader build outputs used about 0.6 GiB in the replay, besides
the kernel source and build, initramfs and boot set. Check free space for `WORK`.

```bash
cargo fetch --locked --manifest-path "$HAL_PINNED/Cargo.toml" \
  --target "$(rustc -vV | sed -n 's/^host: //p')"
ROOTFS="$ROOTFS" KERNEL_RELEASE="$KERNEL_RELEASE" RP1_ELF="$RP1_ELF" \
  RP1BOOT_IMG="$RP1BOOT_IMG" INITRAMFS="$INITRAMFS" NFS_ROOT="$SERVER:$ROOTFS" \
  OUT="$WORK/bootset" RP1_BOARD_CHECK_DTB="$WORK/nfs-board.dtb" \
  bash "$HAL_PINNED/lab/realos/build-bootset.sh"
(cd "$WORK/bootset" && sha256sum -c SHA256SUMS)
```

The [boot set helper](../lab/realos/build-bootset.sh) sets
`root=/dev/nfs nfsroot=<server>:<export>,nfsvers=3 rw` and sets the UART10 and UIO options.
Stage the checked stock DTB, overlay map and selected overlays first, then the generated set.
Verify the served files before booting.

```bash
(cd "$WORK/bootset/boot" && sha256sum config.txt cmdline.txt rp1boot.img rp1-bundle.img rp1.elf manifest.txt) \
  > "$WORK/staged.sha256"
stock_files=()
for f in "${STOCK_DT[@]}"; do stock_files+=("$STOCK_BOOT/./$f"); done
sudo rsync -rtcLO --relative -- "${stock_files[@]}" "$TFTP_DIR/"
sudo rsync -rtcO --delay-updates "$WORK/bootset/boot/" "$TFTP_DIR/"
(cd "$TFTP_DIR" && sha256sum -c "$WORK/stock-dt.sha256")
(cd "$TFTP_DIR" && sha256sum -c "$WORK/staged.sha256")
```

**Hands-on.** Open both serial consoles before starting or resetting the CM5. Use a separate
host terminal for each adapter, or keep an existing capture open on both ports.

```bash
read -erp 'Serial device for this console: ' UART
picocom -b 115200 "$UART"
```

Reconnect the isolated Ethernet link if you disconnected it. On a running Linux console,
use `sudo systemctl reboot`. For a powered bench reset, use the `force-boot` command above.
Check for `Boot mode: NETWORK` and DHCP/TFTP requests for the staged files, then go to step 6.

## 5. Manual SD boot

**HW · 2026-11-17.** An earlier firmware build booted the stock 2712 kernel from SD, mounted ext4
without a Linux initramfs and loaded SCMI from the SD root. [Evidence runs](linux-compatibility.md#evidence-runs)
**HYP · 2026-11-27.** The manual SD replay is pending for this release.
If your kernel needs an initramfs to find the SD root, use a matching local-root initramfs with
`--initramfs`. That variant also needs a replay. Use the NFS rootfs helper only for step 4.

For the no-initramfs path, the MMC block, board SDHCI and ext4 drivers must show `=y`.

```bash
grep -E '^CONFIG_(MMC|MMC_BLOCK|MMC_SDHCI|MMC_SDHCI_PLTFM|MMC_SDHCI_BRCMSTB|EXT4_FS)=' \
  "$ROOTFS/boot/config-$KERNEL_RELEASE"
```

The module list loads the clock provider from ext4 and makes the UIO nodes available at login.
Keep the card's `fstab` and normal login. Staging replaces its `config.txt` and `cmdline.txt`.
To keep extra `dtoverlay` or `dtparam` lines, pass them as `--config-extra` and run
`cargo rp1 check --dtb` with the added settings applied to the board tree.

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
timeout. Use the card's normal login and check for ext4 at step 6. To restore the spare, power
off, return it to the reader and remount the same partitions. Verify their device names and
PARTUUIDs again before copying the backups back with `--delete`. Reconnect Ethernet if you
disconnected it.

```bash
sudo rsync -rt --delete "$SD_BACKUP/boot/" "$SD_BOOT/"
sudo rsync -aHAX --numeric-ids --one-file-system --delete "$SD_BACKUP/root/" "$ROOTFS/"
sync
sudo umount "$SD_BOOT" "$ROOTFS"
```

## 6. First run and troubleshooting

On UART10 look for `bundle ok` with the ID in `cmdline.txt`, then
`jumping to Linux at EL2`. On UART0 expect `proc0: platform ready`,
`proc1: hello`, then `proc0: from proc1: 42` once the first Linux client enables
shared memory. The manifest sets
`log = { sink = "uart0", baudrate = 115200 }`. Proc0 calls
`log!("proc0: platform ready {}\n", sys.report)` after successful platform init.

On the CM5 Linux console, run these commands as root (use sudo from a normal login).

```sh
uname -r
findmnt /
dmesg | grep -Ei 'scmi_mailbox_transport|SCMI Protocol'
ls -l /sys/bus/scmi_protocol/drivers/scmi-clocks/
for d in /sys/class/uio/uio*; do
  [ -r "$d/name" ] || continue
  name=$(cat "$d/name")
  case "$name" in rp1:*) printf '%s ' "$name"; ls -l "/dev/${d##*/}" ;; esac
done
grep -E '[[:space:]]rp1:irq$' /proc/interrupts
/usr/local/bin/linux-app 10
printf 'linux-app exit=%s\n' "$?"
grep -E '[[:space:]]rp1:irq$' /proc/interrupts
```

Expect the release above and an NFSv3 root for step 4 or ext4 for step 5. Look for
`Using scmi_mailbox_transport` and `SCMI Protocol v2.0 'Natsu-B:rp1-hal'` in dmesg.
The `scmi-clocks` driver directory should contain a bound device. The six UIO names used
by the client are `rp1:link`, `rp1:header`,
`rp1:ping-dn` (requests), `rp1:ping-up` (replies), `rp1:doorbell` (Linux to RP1) and
`rp1:irq` (RP1 to Linux). The account running the client needs read/write access to all six
matching `/dev/uioN` nodes. Find them by name after each boot.

Expect ten per-call timings, a `ping rtt n=10` summary and exit 0. The client checks that each
reply equals the value it sent. The total across CPU columns for `rp1:irq` should rise during the run.

| Symptom | Check |
|---|---|
| No `rp1boot` banner | Check the selected boot mode, TFTP prefix or FAT root, and the staged `config.txt`, loader and bundle. Read the UART10 bootloader messages. |
| Loader rejects DT or bundle placement | Check the UART10 reason. Match config, cmdline, loader and bundle from one build. Rerun `cargo rp1 check` against the board DTB with the selected overlays. Resolve pin/console conflicts before staging again. |
| `invalid module format` or unknown symbol | Read dmesg and compare `uname -r`, module vermagic, package versions and the stock `Module.symvers`. Rebuild the modules and initramfs together after a kernel update. |
| Clock probe defers before DHCP | Check for all four SCMI modules in the NFS initramfs and a usable RP1 mailbox driver. Verify `BOOT=nfs`, the interface name, the SCMI node and the absence of `ip=` in the cmdline. |
| UIO names missing | Check `uio_pdrv_genirq.of_id=generic-uio` in `/proc/cmdline` and the `device/driver` links under `/sys/class/uio/uio*`. If the driver is an unloaded module, run `modprobe uio_pdrv_genirq of_id=generic-uio` as root and recheck the names. |
| Permission denied | Match the sysfs names to the device nodes above. Run the first check as root. For regular use, grant the intended user or group read/write access to those six nodes. |
| Waiting for shared memory times out | Check `proc0: platform ready` and panic output on UART0, and the `rp1:link` and `rp1:doorbell` UIO nodes. |
| Ping times out or `rp1:irq` stays flat | Check `proc1: hello` and panic output on UART0. Verify that `rp1:doorbell` and `rp1:irq` are bound to `uio_pdrv_genirq`, GPIO27 is reserved and the ELF and Linux client come from the same pins. |
| Linux console present, RP1 messages missing | Check the UART0 adapter on GPIO14 and ground at 115200. Linux and loader messages come from UART10. |

To recover the previous NFS boot, take the test root offline as in step 4. Restore to
the same directories. The helper checks the backup first, then verifies the restored
TFTP tree and the saved root paths. It removes files it added, such as a new `linux-app`.

```bash
sudo env ROOTFS="$ROOTFS" TFTP_DIR="$TFTP_DIR" BACKUP_DIR="$BACKUP_DIR" \
  bash "$HAL/lab/realos/prepare-rootfs.sh" --restore
```

Boot the previous set using the reset procedure in step 4. Run the three identity commands
again on UART10. Check for a new boot ID and the original root and command line, including
the original `rp1.bundle=` value when the baseline uses a bundle.

See the [peripheral examples](../examples/peripherals/README.md),
[IRQ RPC and channel example](../examples/linux-ping/README.md) and
[What Linux loses](linux-compatibility.md). Build and declaration details are in
[Build](build.md), [Loader](loader.md) and [Declarations and notes](declaration-and-note.md).
