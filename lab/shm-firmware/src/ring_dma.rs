//! Lab-only DMA ring proof (command 17, `dma` feature): an RP1 -> Linux ring stream whose
//! batches are written by the CPU (`RingTx::push_slice`) or by DMA (`shm::RingDma`, channel 1,
//! the cancellation buffer as stage), on the `events` (16 B), `wide64` or `wide256` ring, with
//! or without `HostIrq` notification. Parameters in the request words 8..13 (lab/shm-linux
//! `ring-dma`): ring, mode (0 CPU, 1 DMA, 2 default threshold, 3 DMA without the read-back
//! guard: the ordering experiment), notify, batch, gap_us. Returns errors, pulses, guard
//! retries and failures, DMA and CPU batches, and the RP1 time spent generating messages, in
//! the ring API and notifying.
use crate::dma_proof;
use rp1_hal::{
    dma::Dma,
    host_irq::HostIrq,
    shm::{RingDma, RingTx, Staged},
};

pub type Wide64 = [u32; 16];
pub type Wide256 = [u32; 64];

/// Writes message `i` of a `W`-word stream in place: word 0 = `i`, the last word = `!i` (so the
/// read-back guard and Linux both see a stale slot), the others `i ^ k * 0x9e37_79b9` (cheap:
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
    pub retries: u32,
    pub not_visible: u32,
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
        let t = now();
        for (j, b) in buf[..k].iter_mut().enumerate() {
            message(b, i + j as u32);
        }
        out.gen_us = out.gen_us.wrapping_add(now().wrapping_sub(t));
        let mut done = 0;
        while done < k {
            let values = &buf[done..k];
            let by_dma = values.len() * slot >= stage.threshold();
            let t = now();
            let r = if p.mode == 3 {
                // SAFETY: lab ordering experiment; Linux checks every message.
                unsafe { stage.push_slice_unguarded(tx, &mut dma.ch1, values) }
            } else {
                stage.push_slice(tx, &mut dma.ch1, values)
            };
            match r {
                Ok(n) => {
                    if n > 0 {
                        out.push_us = out.push_us.wrapping_add(now().wrapping_sub(t));
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
    out.retries = tx.guard_retries().wrapping_sub(retries0);
    out
}
