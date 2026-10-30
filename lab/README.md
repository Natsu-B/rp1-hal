# Linux shared-memory proof

STATIC: `shm-firmware` retains the dual-core UART0, SCMI server and GPIO27 doorbell, and adds
reader-local rings, a 1 MiB block and a command block. `shm-linux` checks 1,000,000 messages
in each direction in reject mode, every word of the block in both directions, overwrite
accounting, M3 scalar DDR read latency/read bandwidth/posted-write bandwidth, and the isolated
M3 per-message push/pop cost (commands 7/8, Linux idle on the link page).
Rates include API/loop overhead; M3 writes include a same-window read to drain PCIe writes.

Build with `bash lab/build-shm.sh` and `python3 lab/pack-shm.py`. Locations come from the
environment: `SHM_ARTIFACT_ROOT`, `RP1_LOADER_ROOT`, `RP1_BASE_DTB` (build), and
`SHM_ARTIFACT_ROOT`, `RP1_INITRAMFS`, `RP1_INITRAMFS_HELPER` (pack); see each script header.
The patch is for the PoC chainboot loader (its rp1-hal dev branch): it publishes the fixed
board-table PA and generated DT reservation size over I2C before starting RP1. ELF and merged
DTB remain SHA-256 pinned. This is a dev-loader integration, not the full rp1boot bundle loader.

STATIC memory contract: PA `0x22000000`, minimum 1 MiB (lab: 2 MiB), maximum 16 MiB,
`reserved-memory/no-map`, 64 KiB per-region alignment. The earlier scratch/openamp reservations
remain intact. Loader maximum placements are kernel `0x06000000..0x0e000000`, initramfs
`0x10000000..0x18000000`, scratch `0x1ffe0000..0x20000000`, original/copy DTB below `0x20400000`.
No range intersects even the maximum shared reservation `0x22000000..0x23000000`.
M3 `0x81000000+x` translates through ATU region 0 to PCIe `0x1022000000+x`.

The link-page first 64 bytes hold handoff magic/PA/size (offsets 0/4/8/12), existing doorbell
request/reply (16/20), published generation/status (24/28), and Linux enable request (32).
SCMI occupies `0x17c0..0x18c0` relative to the link page; allocations skip it and the ABI tail.
Linux loads `uio.ko`, then `uio_pdrv_genirq.ko of_id=generic-uio`, and runs `lab/check-shm.sh`.
UIO is polled: hwirq58 stays with SCMI's mailbox. No cached mapping, kernel patch or private
kernel API is used. Device mappings never go through libc memcpy/memset.

RP1 calls unsafe `shm::enable` once after the final Linux PERSTn, then one unsafe `Area::attach`
per owning core. `ring_tx`, `ring_rx` and `block` check the note's type/core/direction and reject
duplicate endpoint acquisition. Returned low-level block writers require the declared writer
ownership. Linux `Firmware::open` checks UIO names, physical ranges, note hash and generation;
handles retain mappings and advisory exclusive UIO locks. Other processes must respect those
locks and must not mmap aliases. Only cold boot is supported; a live PCIe reset invalidates ATU.
For writer=both the lab command handshake transfers writer ownership; the SeqLock alone does
not arbitrate two writers. Blocks support word sections, so RP1 needs no 1 MiB local copy.

Pass/fail is strict (`lab/check-shm.sh` + `lab/verify-shm.py`): each Linux step has a time
limit, any non-zero exit or wrong I2C reply prints `CHECK_FAIL`, the script ends with
`CHECK_SHM_RESULT=PASS|FAIL` then `DDR_INSPECT_END` and exits non-zero on failure. The host
runs `python3 lab/verify-shm.py --uart0 U0 --uart10 U10 --result OBSERVER_JSON`, which needs
the capture's `"complete": true`, the UART10 markers in order (`SHM_PASS`, `SHM_RC=0`, both
doorbells, `I2C_REPLY=0x00000001`, `ADC_REPLY=0x00000ADC`, `CHECK_SHM_RESULT=PASS`, `DDR_INSPECT_END`) and on UART0
`PROF_DONE`, `proc1 shm cmd=9 ... errors=0`, the three `I2C_PASS` lines and
`I2C_RESULT request=1 Ok`. The ADC is declared on proc1 in `shm-firmware`: proc1 gets its
`Clocks` with `Clocks::wait_from_proc0`, runs the boot burst (`proc1 adc init Ok`,
`ADC_P1BOOT ... PASS`) and answers Linux's 0xadc request (`ADC_P1REQ ... PASS`).
`check-shm.sh` issues that request itself, after the doorbell, shm and I2C steps (the burst
blocks proc1's doorbell and command handling for tens of ms, so it must not overlap them), and
fails unless the reply is 0xadc. `verify-shm.py` requires `ADC_REPLY=0x00000ADC` on UART10 (after
`I2C_REPLY`) and on UART0 `ADC_P1BOOT ... PASS`, `ADC_RESULT boot Ok`, `ADC_P1REQ ... PASS` and
`ADC_RESULT request=0xadc Ok`; any `ADC_RESULT ... Err` fails the run. The UART0 boot markers are
in the boot capture, so pass the boot capture and the observer's UART0 log together.
A boot-marker report (`full_success`) is not a pass for this regression. Host tests: `python3 lab/test_verify_shm.py`.

HW results belong in the run notes kept outside this repository; build success alone is STATIC evidence.
