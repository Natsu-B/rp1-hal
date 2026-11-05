//! Lab-only DMA ring proof (command 17, `dma` feature): an RP1 -> Linux ring stream whose
//! batches are written by the CPU (`RingTx::push_slice`) or by DMA (`shm::RingDma`, channel 1,
//! the cancellation buffer as stage), on the `events` (16 B), `wide64` or `wide256` ring, with
//! or without `HostIrq` notification. Parameters in the request words 8..13 (lab/shm-linux
//! `ring-dma`): ring, mode (0 CPU, 1 DMA, 2 default threshold, 3 DMA without the read-back
//! guard: the ordering experiment), notify, batch, gap_us. Returns (errors, pulses, guard
//! retries, guard failures, DMA batches, CPU batches).
use crate::dma_proof;
use rp1_hal::{
    dma::Dma,
    host_irq::HostIrq,
    shm::{RingDma, RingTx, Staged},
};

pub type Wide64 = [u32; 16];
pub type Wide256 = [u32; 64];

/// Message `i` of a `W`-word stream (the last word changes with every message, so the
/// read-back guard and Linux both see a stale slot). Same as lab/shm-linux `wide`.
#[inline(always)]
pub fn wide<const W: usize>(i: u32) -> [u32; W] {
    let mut m = [0u32; W];
    for (k, w) in m.iter_mut().enumerate() {
        *w = i.wrapping_mul(2 * k as u32 + 1) ^ (k as u32) << 24;
    }
    m[W - 1] = !i;
    m
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
    pub errors: u32,
    pub pulses: u32,
    pub retries: u32,
    pub not_visible: u32,
    pub dma_batches: u32,
    pub cpu_batches: u32,
}

/// Streams `count` messages `message(i)` in batches of `p.batch` (at most `B`).
#[rp1_rt::isram]
#[allow(clippy::too_many_arguments)]
pub fn stream<const W: usize, const B: usize>(
    tx: &mut RingTx<[u32; W]>,
    dma: &mut Dma,
    mut irq: Option<&mut HostIrq>,
    message: fn(u32) -> [u32; W],
    count: u32,
    p: &Params,
    start: u32,
) -> Out {
    let mut out = Out::default();
    // SAFETY: command 16 is not running; this is the only borrow.
    let Ok(mut stage) = RingDma::new(unsafe { dma_proof::ring_stage() }) else {
        out.errors = 1;
        return out;
    };
    stage.set_threshold(match p.mode {
        0 => usize::MAX,
        2 => RingDma::DEFAULT_THRESHOLD,
        _ => 0,
    });
    let slot = rp1_proto::shm::slot_size(4 * W);
    let cap = RingTx::<[u32; W]>::staged_capacity(4096).min(B);
    let batch = (p.batch as usize).clamp(1, cap);
    let retries0 = tx.guard_retries();
    let mut buf = [[0u32; W]; B];
    let mut i = 0;
    while i < count {
        let k = (count - i).min(batch as u32) as usize;
        for (j, b) in buf[..k].iter_mut().enumerate() {
            *b = message(i + j as u32);
        }
        let mut done = 0;
        while done < k {
            let values = &buf[done..k];
            let by_dma = values.len() * slot >= stage.threshold();
            let r = if p.mode == 3 {
                // SAFETY: lab ordering experiment; Linux checks every message.
                unsafe { stage.push_slice_unguarded(tx, &mut dma.ch1, values) }
            } else {
                stage.push_slice(tx, &mut dma.ch1, values)
            };
            match r {
                Ok(n) => {
                    if n > 0 {
                        if by_dma {
                            out.dma_batches += 1;
                        } else {
                            out.cpu_batches += 1;
                        }
                    }
                    done += n;
                }
                Err(Staged::NotVisible) => out.not_visible += 1,
                Err(_) => {
                    out.errors += 1;
                    return out;
                }
            }
            if let Some(irq) = irq.as_deref_mut()
                && irq.notify_ring(tx)
            {
                out.pulses += 1;
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
    out.retries = tx.guard_retries().wrapping_sub(retries0);
    out
}
