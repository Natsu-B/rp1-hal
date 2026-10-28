//! RP1 boot over I2C (ported from the HW-proven PoC loader `bcm2712_i2c.rs`,
//! `bcm2712_aon.rs`, `rp1_bootstrap.rs`): RUN pin reset into the bootrom, chip id, image
//! write in 64-byte chunks, shared-memory handoff, scratch registers, start.
use super::{cpu, log};
use alloc::{format, string::String};
use rp1_proto::{link, note::ShmArea};

type Result<T> = core::result::Result<T, String>;

const I2C3: usize = 0x10_7d00_5600;
const AON_PINCTRL: usize = 0x10_7d51_0700;
const AON_GPIO: usize = 0x10_7d51_7c00;
const RUN_PIN: u32 = 1 << 2;

const C: usize = 0x00;
const S: usize = 0x04;
const DLEN: usize = 0x08;
const A: usize = 0x0c;
const FIFO: usize = 0x10;
const DIV: usize = 0x14;
const CLKT: usize = 0x1c;
const C_I2CEN: u32 = 1 << 15;
const C_ST: u32 = 1 << 7;
const C_CLEAR: u32 = 0b11 << 4;
const C_READ: u32 = 1;
const S_CLKT: u32 = 1 << 9;
const S_ERR: u32 = 1 << 8;
const S_RXD: u32 = 1 << 5;
const S_TXD: u32 = 1 << 4;
const S_DONE: u32 = 1 << 1;
const S_TA: u32 = 1;
const POLL: usize = 100_000;

const RP1_ADDR: u32 = 0x43;
const CHUNK: usize = 0x40;
const BOOT_MAGIC: u32 = 0xb007_c0de;
const SCRATCH_MAGIC: u32 = 0x4015_400c;
const SCRATCH_ENTRY: u32 = 0x4015_4010;
const SCRATCH_STACK: u32 = 0x4015_4018;
const BOOT_CTRL_A: u32 = 0x4001_0008;
const BOOT_COMMAND: u32 = 0x4015_4000;
const CHIP_ID: u32 = 0x4000_0000;
const RESETS_CLR: u32 = 0x4001_7004;
const EXPECTED_CHIP_ID: u32 = 0x2000_1927;

fn rd(addr: usize) -> u32 {
    // SAFETY: fixed BCM2712 MMIO (I2C3, AON pinctrl/GPIO), device-mapped.
    unsafe { core::ptr::read_volatile(addr as *const u32) }
}
fn wr(addr: usize, v: u32) {
    // SAFETY: as above.
    unsafe { core::ptr::write_volatile(addr as *mut u32, v) }
}

fn wait(mask: u32) -> Result<()> {
    for _ in 0..POLL {
        if rd(I2C3 + S) & mask != 0 {
            return Ok(());
        }
    }
    Err(format!("I2C timeout S={:#x}", rd(I2C3 + S)))
}
fn check() -> Result<()> {
    let s = rd(I2C3 + S);
    if s & (S_ERR | S_CLKT) != 0 {
        return Err(format!("I2C NACK/clock stretch S={s:#x}"));
    }
    Ok(())
}
fn done() -> Result<()> {
    for _ in 0..POLL {
        let s = rd(I2C3 + S);
        if s & (S_ERR | S_CLKT) != 0 {
            return Err(format!("I2C NACK S={s:#x}"));
        }
        if s & S_DONE != 0 && s & S_TA == 0 {
            wr(I2C3 + S, u32::MAX);
            return Ok(());
        }
    }
    Err(format!("I2C done timeout S={:#x}", rd(I2C3 + S)))
}

fn i2c_write(bytes: &[u8]) -> Result<()> {
    wr(I2C3 + A, RP1_ADDR);
    wr(I2C3 + DLEN, bytes.len() as u32);
    wr(I2C3 + S, u32::MAX);
    wr(I2C3 + C, C_I2CEN | C_CLEAR);
    wr(I2C3 + C, C_I2CEN | C_ST);
    for b in bytes {
        wait(S_TXD | S_ERR | S_CLKT)?;
        check()?;
        wr(I2C3 + FIFO, u32::from(*b));
    }
    done()
}
fn i2c_read(buf: &mut [u8]) -> Result<()> {
    wr(I2C3 + A, RP1_ADDR);
    wr(I2C3 + DLEN, buf.len() as u32);
    wr(I2C3 + S, u32::MAX);
    wr(I2C3 + C, C_I2CEN | C_CLEAR | C_READ | C_ST);
    let mut pos = 0;
    while pos < buf.len() {
        wait(S_RXD | S_DONE | S_ERR | S_CLKT)?;
        check()?;
        while pos < buf.len() && rd(I2C3 + S) & S_RXD != 0 {
            buf[pos] = rd(I2C3 + FIFO) as u8;
            pos += 1;
        }
    }
    done()
}

/// RP1 bootrom I2C protocol: BE32 address followed by data.
fn write_mem(addr: u32, data: &[u8]) -> Result<()> {
    let mut p = [0u8; 4 + CHUNK];
    p[..4].copy_from_slice(&addr.to_be_bytes());
    p[4..4 + data.len()].copy_from_slice(data);
    i2c_write(&p[..4 + data.len()])
}
fn write32(addr: u32, v: u32) -> Result<()> {
    write_mem(addr, &v.to_le_bytes())
}
fn read32(addr: u32) -> Result<u32> {
    i2c_write(&addr.to_be_bytes())?;
    let mut b = [0u8; 4];
    i2c_read(&mut b)?;
    Ok(u32::from_le_bytes(b))
}

fn setup_i2c() {
    // AON pins for I2C3 (function 4, pull-up), as the PoC.
    let mux = rd(AON_PINCTRL + 0x0c);
    wr(AON_PINCTRL + 0x0c, (mux & !0xff) | 4 | (4 << 4));
    let p14 = rd(AON_PINCTRL + 0x14);
    wr(
        AON_PINCTRL + 0x14,
        (p14 & !(0xf << 18)) | (2 << 18) | (2 << 20),
    );
    let p18 = rd(AON_PINCTRL + 0x18);
    wr(
        AON_PINCTRL + 0x18,
        (p18 & !(0xf << 20)) | (2 << 20) | (2 << 22),
    );
    wr(I2C3 + C, C_CLEAR);
    wr(I2C3 + DIV, 2500);
    wr(I2C3 + CLKT, 0x40);
    wr(I2C3 + S, u32::MAX);
    wr(I2C3 + C, C_I2CEN);
}

fn run_pin(high: bool) {
    // AON GPIO2 drives RP1 RUN (BCM2712-internal, not a CM5 edge signal).
    wr(AON_GPIO + 0x08, rd(AON_GPIO + 0x08) & !RUN_PIN);
    let d = rd(AON_GPIO + 0x04);
    wr(
        AON_GPIO + 0x04,
        if high { d | RUN_PIN } else { d & !RUN_PIN },
    );
}

pub fn boot(image: &[u8], entry: u32, stack: u32, area: Option<&ShmArea>) -> Result<()> {
    setup_i2c();
    run_pin(false);
    cpu::delay_us(50_000);
    run_pin(true);
    cpu::delay_us(10_000);
    let mut ok = false;
    for _ in 0..100 {
        if write32(RESETS_CLR, 0x0080_0000).is_ok() {
            ok = true;
            break;
        }
        cpu::delay_us(1_000);
    }
    if !ok {
        return Err("RP1 did not answer on I2C 0x43 after reset (RESETS_CLR write)".into());
    }
    cpu::delay_us(1_000);
    let mut id = Err(String::new());
    for _ in 0..50 {
        id = read32(CHIP_ID);
        if id.is_ok() {
            break;
        }
        cpu::delay_us(1_000);
    }
    let id = id.map_err(|e| format!("RP1 chip id read failed: {e}"))?;
    log!("RP1 bootrom: chip id {id:#010x}");
    if id != EXPECTED_CHIP_ID {
        return Err(format!(
            "unexpected RP1 chip id {id:#x} (want {EXPECTED_CHIP_ID:#x})"
        ));
    }
    for (i, chunk) in image.chunks(CHUNK).enumerate() {
        write_mem(rp1_proto::SRAM_START + (i * CHUNK) as u32, chunk)
            .map_err(|e| format!("RP1 image write at +{:#x}: {e}", i * CHUNK))?;
    }
    if let Some(a) = area {
        // Handoff words (rp1_proto::link): clear the first 64 bytes, then PA/size, magic last.
        let base = a.handoff;
        for off in (0..64).step_by(4) {
            write32(base + off, 0)?;
        }
        write32(base + link::HANDOFF_PHYS_LOW as u32, a.phys as u32)?;
        write32(base + link::HANDOFF_PHYS_HIGH as u32, (a.phys >> 32) as u32)?;
        write32(base + link::HANDOFF_SIZE as u32, a.size)?;
        write32(base + link::HANDOFF_MAGIC as u32, link::MAGIC)?;
        log!(
            "handoff at {base:#x}: shm phys={:#x} size={:#x}",
            a.phys,
            a.size
        );
    }
    let entry = entry | 1;
    write32(SCRATCH_MAGIC, BOOT_MAGIC)?;
    write32(SCRATCH_ENTRY, entry ^ BOOT_MAGIC)?;
    write32(SCRATCH_STACK, stack)?;
    write32(BOOT_CTRL_A, 0x100)?;
    write32(BOOT_COMMAND, 0x8000_0000)?;
    log!("RP1 proc0 started: entry={entry:#x} sp={stack:#x}");
    Ok(())
}
