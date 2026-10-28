# Linux shared-memory proof

STATIC: `shm-firmware` retains the dual-core UART0, SCMI server and GPIO27 doorbell, and adds
reader-local rings, a 1 MiB block and a command block. `shm-linux` checks 1,000,000 messages
in each direction in reject mode, every word of the block in both directions, overwrite
accounting, M3 scalar DDR read latency/read bandwidth/posted-write bandwidth, and the isolated
M3 per-message push/pop cost (commands 7/8, Linux idle on the link page).
Rates include API/loop overhead; M3 writes include a same-window read to drain PCIe writes.

Build with `bash lab/build-shm.sh` and `python3 lab/pack-shm.py`. Paths can be overridden with
`CM5_HACK_ROOT`, `SHM_ARTIFACT_ROOT`, `RP1_LOADER_ROOT`, and `RP1_BASE_DTB`.
The patch is for the existing `claude/rp1-hal-dev-20261028` PoC loader: it publishes the fixed
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

HW results belong in the external `hwrun3/RESULT.md`; build success alone is STATIC evidence.
