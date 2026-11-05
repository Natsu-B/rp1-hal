//! Lab ring DMA proof, command 17: mode 0 CPU, 1 DMA, 2 auto threshold,
//! 3 early-head positive control, 4 bit30=0 comparison. Experiments stay in lab.
use crate::dma_proof;
use rp1_hal::{
    dma::Dma,
    host_irq::HostIrq,
    shm::{RingDma, RingTx},
};

pub type Wide64 = [u32; 16];
pub type Wide256 = [u32; 64];

/// Writes message `i` of a `W`-word stream in place: word 0 = `i`, the last word = `!i` (so the
/// Linux sees a stale slot), the others `i ^ k * 0x9e37_79b9` (cheap:
/// the generation is part of the measured loop; ISRAM, since code in shared SRAM runs several
/// times slower). Same as lab/shm-linux `wide`.
#[rp1_rt::isram]
pub fn wide<const W: usize>(m: &mut [u32; W], i: u32) {
    for (k, w) in m.iter_mut().enumerate() {
        *w = i ^ (k as u32).wrapping_mul(0x9e37_79b9);
    }
    m[W - 1] = !i;
}
/// [`crate::sample`] in place (the `events` stream).
#[rp1_rt::isram]
pub fn sample_into(m: &mut [u32; 4], i: u32) {
    *m = crate::sample(i);
}

fn now() -> u32 {
    unsafe { (0x400a_c028 as *const u32).read_volatile() }
}

pub struct Params {
    pub mode: u32,
    pub notify: bool,
    pub batch: u32,
    pub gap_us: u32,
}

#[derive(Default)]
pub struct Out {
    /// RP1 µs generating messages, in the ring API (calls that published), notifying.
    pub gen_us: u32,
    pub push_us: u32,
    pub notify_us: u32,
    pub errors: u32,
    pub pulses: u32,
    pub ctl_l: u32,
    pub wraps: u32,
    pub dma_batches: u32,
    pub cpu_batches: u32,
}

/// Streams `count` messages `message(i)` in batches of `p.batch` (at most `B`). Not in ISRAM
/// (no room for three instances); the ring API it calls is (`push_slice`, the staged push,
/// the DMA copy, `wake_needed`), and the generation time is reported separately.
#[allow(clippy::too_many_arguments)]
pub fn stream<const W: usize, const B: usize>(
    tx: &mut RingTx<[u32; W]>,
    dma: &mut Dma,
    mut irq: Option<&mut HostIrq>,
    message: fn(&mut [u32; W], u32),
    count: u32,
    p: &Params,
    start: u32,
) -> Out {
    let mut out = Out::default();
    // SAFETY: command 16 is not running; this is the only borrow.
    let stage_buf = unsafe { dma_proof::ring_stage() };
    let (production, experiment) = if p.mode >= 3 {
        stage_buf.split_at_mut(0)
    } else {
        stage_buf.split_at_mut(stage_buf.len())
    };
    // Empty production stage in experiment mode is unused; retain an optional helper.
    let mut stage = if p.mode >= 3 {
        None
    } else {
        Some(RingDma::new(production).unwrap())
    };
    let threshold = match p.mode {
        0 => usize::MAX,
        2 => RingDma::DEFAULT_THRESHOLD,
        _ => 0,
    };
    if let Some(stage) = stage.as_mut() {
        stage.set_threshold(threshold);
    }
    let slot = rp1_proto::shm::slot_size(4 * W);
    let cap = RingTx::<[u32; W]>::staged_capacity(4096).min(B);
    let batch = (p.batch as usize).clamp(1, cap);
    let depth = if W == 64 { 128 } else { 256 };
    let mut buf = [[0u32; W]; B];
    let mut i = 0;
    while i < count {
        let k = (count - i).min(batch as u32) as usize;
        let t = now();
        for (j, b) in buf[..k].iter_mut().enumerate() {
            message(b, i + j as u32);
        }
        out.gen_us = out.gen_us.wrapping_add(now().wrapping_sub(t));
        let mut done = 0;
        while done < k {
            let values = &buf[done..k];
            let writable = tx.writable() as usize;
            let n = values.len().min(writable);
            let by_dma = n * slot >= threshold;
            let head = tx.head();
            let t = now();
            let r = if p.mode >= 3 {
                // SAFETY: lab-only deliberate contract violation; sole stage/channel owner.
                let mut copy = ExperimentCopy {
                    early: p.mode == 3,
                    nonposted: p.mode == 3,
                    head,
                    n: n as u32,
                    depth,
                    slot,
                };
                tx.push_slice_staged(&values[..n], experiment, &mut copy)
            } else {
                stage.as_mut().unwrap().push_slice(tx, &mut dma.ch1, values)
            };
            match r {
                Ok(n) => {
                    if n > 0 {
                        out.push_us = out.push_us.wrapping_add(now().wrapping_sub(t));
                        if (head & (depth - 1)) + n as u32 >= depth {
                            out.wraps += 1;
                        }
                        if by_dma {
                            out.ctl_l = unsafe { (CTL as *const u32).read_volatile() };
                            out.dma_batches += 1;
                        } else {
                            out.cpu_batches += 1;
                        }
                    }
                    done += n;
                }
                Err(_) => {
                    out.errors += 1;
                    return out;
                }
            }
            if let Some(irq) = irq.as_deref_mut() {
                let t = now();
                if irq.notify_ring(tx) {
                    out.pulses += 1;
                }
                out.notify_us = out.notify_us.wrapping_add(now().wrapping_sub(t));
            }
            if now().wrapping_sub(start) > 180_000_000 {
                out.errors += 1;
                return out;
            }
        }
        i += k as u32;
        let t = now();
        while now().wrapping_sub(t) < p.gap_us {}
    }
    out
}

use rp1_hal::dma::{BusAddr, Error};
use rp1_hal::pac::dw_axi_dmac::{
    BASE, CHANNEL_BASE, CHANNEL_STRIDE, cfg2, ch, common, ctl_h, ctl_l, int,
};
const CH: usize = BASE + CHANNEL_BASE + CHANNEL_STRIDE;
const CTL: usize = CH + ch::CTL_L;
struct ExperimentCopy {
    early: bool,
    nonposted: bool,
    head: u32,
    n: u32,
    depth: u32,
    slot: usize,
}
// Deliberately violates SlotCopy ordering for controls; only used by this lab command.
unsafe impl rp1_proto::shm::SlotCopy for ExperimentCopy {
    type Error = Error;
    unsafe fn copy_to_remote(&mut self, src: &[u8], dst: *mut u8) -> Result<(), Error> {
        use core::ptr::{read_volatile as rd, write_volatile as wr};
        let s = BusAddr::from_proc(src.as_ptr() as usize)?.0;
        let d = BusAddr::from_proc(dst as usize)?.0;
        let w = rp1_hal::dma::width_log2(s, d, src.len() as u32);
        unsafe {
            if rd((BASE + common::CHEN) as *const u32) & 2 != 0 {
                return Err(Error::Busy);
            }
            // Publish before even enabling the first DMA block; 100us exposes the race.
            if self.early {
                let base = dst.sub(
                    (self.head & (self.depth - 1)) as usize * self.slot
                        + rp1_proto::shm::CONTROL_SIZE,
                );
                wr(base.cast::<u32>(), self.head.wrapping_add(self.n));
                rp1_proto::barrier();
                let t = now();
                while now().wrapping_sub(t) < 100 {
                    core::hint::spin_loop();
                }
                self.early = false;
            }
            for (off, v) in [
                (ch::SAR, s),
                (ch::DAR, d),
                (ch::BLOCK_TS, ((src.len() as u64) >> w) - 1),
            ] {
                wr((CH + off) as *mut u32, v as u32);
                wr((CH + off + 4) as *mut u32, (v >> 32) as u32);
            }
            // Mode 4 differs from production only in bit30 (AW_CACHE remains zero).
            wr(
                CTL as *mut u32,
                if self.nonposted {
                    ctl_l::NONPOSTED_LAST_WRITE_EN
                } else {
                    0
                } | (ctl_l::MSIZE_4 << ctl_l::DST_MSIZE_SHIFT)
                    | (ctl_l::MSIZE_4 << ctl_l::SRC_MSIZE_SHIFT)
                    | (w << ctl_l::DST_TR_WIDTH_SHIFT)
                    | (w << ctl_l::SRC_TR_WIDTH_SHIFT)
                    | (ctl_l::AW_CACHE_NONBUFFERABLE << ctl_l::AW_CACHE_SHIFT),
            );
            wr(
                (CH + ch::CTL_H) as *mut u32,
                ctl_h::ARLEN_EN
                    | (7 << ctl_h::ARLEN_SHIFT)
                    | ctl_h::AWLEN_EN
                    | (7 << ctl_h::AWLEN_SHIFT),
            );
            wr((CH + ch::CFG_L) as *mut u32, 0);
            wr(
                (CH + ch::CFG_H) as *mut u32,
                (1 << cfg2::PRIORITY_SHIFT) | cfg2::HS_SEL_SRC | cfg2::HS_SEL_DST,
            );
            wr((CH + ch::INTCLEAR) as *mut u32, int::ALL);
            wr((CH + ch::INTSIGNAL_ENA) as *mut u32, 0);
            wr(
                (CH + ch::INTSTATUS_ENA) as *mut u32,
                int::DMA_TFR_DONE | int::ALL_ERR,
            );
            rp1_proto::barrier();
            wr(
                (BASE + common::CHEN) as *mut u32,
                2 | (2 << common::CHEN_WE_SHIFT),
            );
            let t = now();
            loop {
                let status = rd((CH + ch::INTSTATUS) as *const u32);
                let enabled = rd((BASE + common::CHEN) as *const u32) & 2 != 0;
                if status & int::ALL_ERR != 0 || now().wrapping_sub(t) > 10_000 {
                    wr(
                        (BASE + common::CHEN) as *mut u32,
                        2 << common::CHEN_WE_SHIFT,
                    );
                    // Never release stage while the engine can read it.
                    while rd((BASE + common::CHEN) as *const u32) & 2 != 0 {
                        core::hint::spin_loop();
                    }
                    rp1_proto::barrier();
                    return Err(if status & int::ALL_ERR != 0 {
                        Error::Bus(status)
                    } else {
                        Error::Timeout
                    });
                }
                if status & int::DMA_TFR_DONE != 0 && !enabled {
                    break;
                }
            }
            rp1_proto::barrier();
        }
        Ok(())
    }
}
