import runpy
import unittest
from pathlib import Path

verify = runpy.run_path(str(Path(__file__).with_name("verify-dma.py")))
NONCE = "1234abcd"
INJECTION = (
    "mask_before=0xe03f7fe2 mask_idle=0xa03f7fe0 "
    "int_status_completed=0x00000002 int_status_armed=0x00000000 mask_after=0xa03f7fe0"
)


def parity_capture():
    yield f"PARITY_DMA_BOOT nonce={NONCE} PLAT_INIT ok=1 sys_hz=200000000"
    for n in range(8):
        for size in (4, 64, 4096):
            yield (
                f"PARITY_DMA_COPY nonce={NONCE} ch={n} bytes={size} count=1000 "
                "total_us=9000 max_us=12 crc_diff=0 range_diff=0"
            )
        count = 12 + int(n < 4)
        yield (
            f"PARITY_DMA_CANCEL nonce={NONCE} ch={n} count={count} stopped={count} "
            "quarantined=0 rechecked=1 first_us=6 decision_us=905 ch_status_before=0x00000014 "
            "terminal_status=0x40000000 ch_status_after=0x00000021 int_status_after=0x00000000 "
            "partial=1 early=0 range_diff=0 late_diff=0 payload_diff=0"
        )
    yield (
        f"PARITY_DMA_DROP nonce={NONCE} ch=0 injected=0 active=1 drop_us=7 "
        "quarantined=0 chen_after=0 int_status_after=0x00000000 late_diff=0 pair_busy=0 ok=1"
    )
    yield f"PARITY_DMA_RECOVERY nonce={NONCE} forgotten=1 leases_claimed=2 reinit=1"
    yield f"PARITY_DMA_BUS_ERROR nonce={NONCE} ch=6 status=0x00000020 stopped=0 recovered=0 chen_after=0 ok=1"
    yield (
        f"PARITY_DMA_QUARANTINE nonce={NONCE} ch=7 active=1 first_us=6 decision_us=1000 rechecked=1 "
        "ch_status_before=0x00000000 ch_status_after=0x00000010 int_status_after=0x00000000 "
        f"chen_after=0 quarantined=1 leases_claimed=2 {INJECTION} ok=1"
    )
    yield (
        f"PARITY_DMA_DROP nonce={NONCE} ch=1 injected=1 active=1 drop_us=8 "
        f"quarantined=1 chen_after=0 int_status_after=0x00000000 late_diff=0 pair_busy=1 {INJECTION} ok=1"
    )
    yield f"PARITY_DMA_SINGLE_END nonce={NONCE} channels=8 copies=24000 cancels=100 ok=1"


def shm_capture():
    yield "p1 dma init rc=0x0"
    for size in (4, 12, 16, 64, 1000, 1024):
        yield f"DMA_SRAM bytes={size} rc=0x0 bad_bytes=0"
    yield "DMA_SRAM_RATE bytes=1024 n=1000 ch0_us=12345 ch2_us=12456 cpu_us=23456"
    yield "DMA_SRAM_PASS"
    for size in (768, 2304, 2112, 3072, 4032, 3960, 4080):
        for width in ((16, 8) if size % 16 == 0 else (8, 8)):
            yield f"RINGDMA_RAW bytes={size} width={width} count=1000 total_us=13242 bad=0"
    yield "DMA_ABORT_FULL bytes=4094 width=1 rc=0x0 us=300 copied=4094 ok=1"
    for quarter in range(1, 4):
        yield (
            f"DMA_ABORT q={quarter} delay_us=50 how=1 chen_after=0 copied=380 len=4094 "
            "tail_bad=0 guard_bad=0 late_change=0 elapsed_us=70 ok=1"
        )
        yield f"DMA_ABORT_REUSE q={quarter} rc=0x0 us=300 copied=4094 ok=1"
    yield (
        "DMA_ABORT_RAW reg=0x01c chen_after=0 clear_us=1 intstatus=0x80000000 aborted=1 "
        "copied=1233 tail_bad=2 guard_bad=0 reuse_rc=0x0 reuse_ok=1 ok=1"
    )
    yield "DMA_ABORT_PASS midway=3"
    yield (
        "DMA_DDR small16_n1000_us=12 small512_n1000_us=34 cpu_512k_us=44 atu_512k_us=55 "
        "direct_512k_us=66 readback_bad=0 atu_64k_x8_us=77 direct_64k_x8_us=88 errors=0"
    )


LINUX = """V7_DMA_BEGIN
WINDOW_PASS bytes=8388608 all_words errors=0
SHM_READY generation=2
V7_DMA_PROBE_RC=0
PING_RTT n=20
V7_DMA_PING_BEFORE_RC=0
DMA_DDR_LINUX rp1_errors=0 small16_us_per=123 rp1_readback_bad=0
DMA_DDR_VERIFY words=262144 bad=0 canary_left=0 first_bad=None
DMA_DDR_PASS
V7_DMA_COMMAND_RC=0
SHM_PASS
V7_DMA_WORKLOAD_RC=0
PING_RTT n=20
V7_DMA_PING_AFTER_RC=0
V7_DMA_END
"""


class DmaCaptureTest(unittest.TestCase):
    def test_parity_enforces_counts_and_bounds(self):
        log = "\n".join(parity_capture())
        self.assertEqual(
            verify["parity"](log),
            f"nonce={NONCE} stopped=100 rechecked=8 first_us=6 decision_us=905 "
            "drop_us=7 quarantine_us=1000 drop_quarantine_us=8",
        )
        verify["parity"](log.replace("stopped=0 recovered=0", "stopped=1 recovered=1"))
        for bad in (
            log.replace("count=1000", "count=999", 1),
            log.replace("crc_diff=0", "crc_diff=1", 1),
            log.replace("range_diff=0", "range_diff=1", 1),
            log.replace("total_us=9000", "total_us=12001", 1),
            log.replace("decision_us=905", "decision_us=1001", 1),
            log.replace("decision_us=1000", "decision_us=1001", 1),
            log.replace("terminal_status=0x40000000", "terminal_status=0x00000000", 1),
            log.replace("drop_us=7", "drop_us=1001", 1),
            log.replace("drop_us=8", "drop_us=1001", 1),
            log.replace("injected=1 active=1", "injected=1 active=0", 1),
            log.replace("pair_busy=1", "pair_busy=0", 1),
            log.replace("mask_before=0xe03f7fe2", "mask_before=0xa03f7fe0", 1),
            log.replace("mask_idle=0xa03f7fe0", "mask_idle=0xe03f7fe2", 1),
            log.replace("mask_idle=0xa03f7fe0", "mask_idle=0xa0000000"),
            log.replace("mask_after=0xa03f7fe0", "mask_after=0xe03f7fe2"),
            log.replace("int_status_completed=0x00000002", "int_status_completed=0x00000000"),
            log.replace("int_status_armed=0x00000000", "int_status_armed=0x40000000"),
            log.replace(f" {INJECTION}", "", 1),
            log.replace("status=0x00000020", "status=0x00000000", 1),
            log.replace("stopped=0 recovered=0", "stopped=1 recovered=0", 1),
            log.replace("early=0", "early=1", 1),
            log.replace("late_diff=0", "late_diff=1", 1),
            log.replace("partial=1", "partial=0"),
            log.replace("quarantined=1", "quarantined=0", 1),
            log.replace("leases_claimed=2", "leases_claimed=1"),
            log.replace(f"nonce={NONCE}", "nonce=12345678", 1),
            log + "\n" + log,
            log.rsplit("\n", 1)[0],
        ):
            with self.subTest(bad=bad[-150:]), self.assertRaises(ValueError):
                verify["parity"](bad)

    def test_shm_enforces_linux_and_firmware_markers(self):
        uart0 = "\n".join(shm_capture())
        verify["shm"](uart0, LINUX)
        for bad in (
            uart0.replace("DMA_ABORT_PASS", "DMA_ABORT_FAIL"),
            uart0.replace("late_change=0", "late_change=1", 1),
            uart0.replace("copied=380", "copied=4094", 1),
            uart0.replace("bytes=12 rc=0x0", "bytes=12 rc=0x4", 1),
            uart0 + "\nDMA_SRAM_PASS",
        ):
            with self.assertRaises(ValueError):
                verify["shm"](bad, LINUX)
        for bad in (
            LINUX.replace("canary_left=0", "canary_left=1"),
            LINUX.replace("V7_DMA_COMMAND_RC=0", "V7_DMA_COMMAND_RC=1"),
            LINUX.replace("V7_DMA_END", ""),
            LINUX + "V7_DMA_END\n",
        ):
            with self.assertRaises(ValueError):
                verify["shm"](uart0, bad)


if __name__ == "__main__":
    unittest.main()
