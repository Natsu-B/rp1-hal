#!/bin/sh
# Linux-side UART1 check against the host peer (`lab/uart1-peer.py`): Linux keeps UART1 on GPIO 0/1
# (`dtoverlay=uart1-pi5`, ttyAMA*) while the RP1 firmware owns UART0, I2C1 and SPI0.
#
# Set UART1_NONCE (hex, at least 8 digits; the same as the peer's --nonce) before this script, e.g. by
# prepending `UART1_NONCE=...` to it when sending it over the console.
#   Linux -> host: writes `RP1SHARE_TX_<nonce>_<i>` lines; the peer records what arrives.
#   host -> Linux: the peer repeats `RP1SHARE_RX_<nonce>`; this prints what it read as
#                  `UART1_RX_GOT=<token>` (the token comes from the wire, not from UART1_NONCE).
# The verdict is the host's: `lab/verify-shm.py --uart1-host ... --uart1-nonce ...` prints
# UART1_TX_PASS / UART1_RX_PASS. Diagnostics (pinmux, pads, PL011 registers, driver counters) are
# printed as UART1_DIAG lines. Ends with `UART1_CHECK_DONE` then `DDR_INSPECT_END`.
export PATH=/bin
busybox --install -s /bin
echo UART1_CHECK_BEGIN
# RP1 register at its 0x40xxxxxx address, read through the PCIe BAR (0x1f00000000 + offset).
reg() { busybox devmem $(printf 0x1f%08x $(( $1 - 0x40000000 ))) 32; }
diag() {
    for g in 0 1; do
        echo "UART1_DIAG $1 GPIO$g status=$(reg $((0x400d0000 + g * 8))) ctrl=$(reg $((0x400d0004 + g * 8))) pad=$(reg $((0x400f0004 + g * 4)))"
    done
    echo "UART1_DIAG $1 pl011 fr=$(reg 0x40034018) ibrd=$(reg 0x40034024) fbrd=$(reg 0x40034028) lcr_h=$(reg 0x4003402c) cr=$(reg 0x40034030)"
    grep -E "^[0-9]+: .*mmio:0x1F00034000" /proc/tty/driver/ttyAMA | sed "s/^/UART1_DIAG $1 driver /"
}
U=
for t in /sys/class/tty/ttyAMA*; do
    readlink -f "$t/device" | grep -q '/1f00034000\.serial' && U=/dev/${t##*/}
done
echo "UART1_DEV=${U:-missing}"
dmesg | grep -E '1f00034000|ttyAMA' | sed 's/^/UART1_DIAG dmesg /'
for path in $(find /sys/firmware/devicetree/base -name serial@34000); do
    echo "UART1_DIAG dt $path status=$(cat "$path/status" 2>/dev/null)"
done
case "$UART1_NONCE" in
    '' | *[!0-9a-f]*) echo "UART1_DIAG bad UART1_NONCE '$UART1_NONCE'"; U= ;;
esac
[ ${#UART1_NONCE} -ge 8 ] || U=
if [ -n "$U" ]; then
    # Hold the port open: closing it would shut the PL011 down and drop what the peer sends.
    exec 3<>"$U"
    stty -F "$U" 115200 cs8 -cstopb -parenb -crtscts clocal raw -echo
    echo "UART1_DIAG stty $(stty -F "$U" | tr '\n' ' ')"
    diag before
    rm -f /tmp/uart1-rx
    # The redirection must be on the background command itself: a non-interactive sh gives a
    # background command without one /dev/null as stdin.
    cat <&3 > /tmp/uart1-rx &
    reader=$!
    i=0
    while [ $i -lt 10 ]; do
        printf 'RP1SHARE_TX_%s_%d\n' "$UART1_NONCE" $i >&3
        i=$((i + 1))
        sleep 0.2
    done
    echo "UART1_TX_SENT nonce=$UART1_NONCE lines=$i"
    sleep 4
    kill $reader 2>/dev/null
    wait $reader 2>/dev/null
    diag after
    exec 3>&-
    echo "UART1_RX_BYTES=$(wc -c < /tmp/uart1-rx)"
    echo "UART1_RX_GOT=$(grep -a -o 'RP1SHARE_RX_[0-9a-f]*' /tmp/uart1-rx | head -n 1)"
    od -A d -c -N 96 /tmp/uart1-rx | sed 's/^/UART1_DIAG rx /'
else
    echo UART1_RX_GOT=
fi
echo UART1_CHECK_DONE
echo DDR_INSPECT_END
