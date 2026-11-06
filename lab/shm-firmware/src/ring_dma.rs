//! Command 17: CPU/DMA slice (0/1/2), ordering controls (3/4), and generated
//! DMA/CPU/auto (5/6/7). Optional timing stays in the lab, not in the HAL.
use crate::dma_proof;
use rp1_hal::{
    dma::Dma,
    host_irq::HostIrq,
    shm::{RingDma, RingTx, SlotCopy},
};

pub type Wide64 = [u32; 16];
pub type Wide256 = [u32; 64];

/// All words are verified by Linux, including the sequence and inverted last word.
#[rp1_rt::isram]
pub fn wide<const W: usize>(m: &mut [u32; W], i: u32) {
    for (k, w) in m.iter_mut().enumerate() {
        *w = i ^ (k as u32).wrapping_mul(0x9e37_79b9);
    }
    m[W - 1] = !i;
}
#[rp1_rt::isram]
pub fn sample_into(m: &mut [u32; 4], i: u32) {
    *m = crate::sample(i);
}

#[inline(always)]
fn now() -> u32 {
    unsafe { (0x400a_c028 as *const u32).read_volatile() }
}

pub struct Params {
    pub mode: u32,
    pub notify: bool,
    pub batch: u32,
    pub gap_us: u32,
    pub timed: bool,
}

#[derive(Default)]
pub struct Out {
    pub gen_us: u32,
    pub push_us: u32,
    pub notify_us: u32,
    pub errors: u32,
    pub pulses: u32,
    pub ctl_l: u32,
    pub wraps: u32,
    pub dma_batches: u32,
    pub cpu_batches: u32,
    /// copy/prepare/publish/blocked µs, blocks/two-block batches/8 B/16 B blocks.
    pub perf: [u32; 8],
}

/// Keep the slice comparison's large stack buffer out of the generated path.
#[allow(clippy::too_many_arguments)]
pub fn stream<const W: usize, const B: usize>(
    tx: &mut RingTx<[u32; W]>,
    dma: &mut Dma,
    irq: Option<&mut HostIrq>,
    message: fn(&mut [u32; W], u32),
    count: u32,
    p: &Params,
    start: u32,
) -> Out {
    if p.mode >= 5 {
        run(tx, dma, irq, message, count, p, start, &mut [])
    } else {
        slice_stream::<W, B>(tx, dma, irq, message, count, p, start)
    }
}
#[inline(never)]
#[allow(clippy::too_many_arguments)]
fn slice_stream<const W: usize, const B: usize>(
    tx: &mut RingTx<[u32; W]>,
    dma: &mut Dma,
    irq: Option<&mut HostIrq>,
    message: fn(&mut [u32; W], u32),
    count: u32,
    p: &Params,
    start: u32,
) -> Out {
    let mut buf = [[0u32; W]; B];
    run(tx, dma, irq, message, count, p, start, &mut buf)
}

#[allow(clippy::too_many_arguments)]
fn run<const W: usize>(
    tx: &mut RingTx<[u32; W]>,
    dma: &mut Dma,
    irq: Option<&mut HostIrq>,
    message: fn(&mut [u32; W], u32),
    count: u32,
    p: &Params,
    start: u32,
    buf: &mut [[u32; W]],
) -> Out {
    let mut out = Out::default();
    if p.mode > 7 {
        out.errors = 1;
        return out;
    }
    // SAFETY: command 16 is not running; this command owns the sole borrow.
    let stage_buf = unsafe { dma_proof::ring_stage() };
    let cap = RingTx::<[u32; W]>::staged_capacity(stage_buf.len());
    let generated = p.mode >= 5;
    let batch = (p.batch as usize).clamp(1, if generated { cap } else { cap.min(buf.len()) });
    let mut stage = RingDma::new(stage_buf).unwrap();
    let threshold = match p.mode {
        0 | 6 => usize::MAX,
        2 | 7 => RingDma::DEFAULT_THRESHOLD,
        _ => 0,
    };
    stage.set_threshold(threshold);
    let slot = rp1_proto::shm::slot_size(4 * W);
    let depth = if W == 64 { 128 } else { 256 };
    let mut producer = TypedProducer {
        tx,
        dma,
        message,
        buf,
        stage,
        threshold,
        slot,
        depth,
    };
    run_batches(&mut producer, irq, count, p, start, batch, depth)
}

trait Producer {
    fn notify(&mut self, irq: &mut HostIrq) -> bool;
    fn generate(&mut self, count: usize, first: u32) -> u32;
    fn attempt(&mut self, k: usize, done: usize, i: u32, p: &Params) -> Attempt;
}
struct Attempt {
    head: u32,
    by_dma: bool,
    elapsed: u32,
    gen_us: u32,
    stats: CopyStats,
    result: Result<usize, rp1_proto::shm::Staged<Error>>,
}
struct TypedProducer<'a, const W: usize> {
    tx: &'a mut RingTx<[u32; W]>,
    dma: &'a mut Dma,
    message: fn(&mut [u32; W], u32),
    buf: &'a mut [[u32; W]],
    stage: RingDma<'a>,
    threshold: usize,
    slot: usize,
    depth: u32,
}
impl<const W: usize> Producer for TypedProducer<'_, W> {
    fn notify(&mut self, irq: &mut HostIrq) -> bool {
        irq.notify_ring(self.tx)
    }
    fn generate(&mut self, count: usize, first: u32) -> u32 {
        let t = now();
        for (j, b) in self.buf[..count].iter_mut().enumerate() {
            (self.message)(b, first + j as u32);
        }
        now().wrapping_sub(t)
    }
    fn attempt(&mut self, k: usize, done: usize, i: u32, p: &Params) -> Attempt {
        let Self {
            tx,
            dma,
            message,
            buf,
            stage,
            threshold,
            slot,
            depth,
        } = self;
        let (threshold, slot, depth) = (*threshold, *slot, *depth);
        let generated = p.mode >= 5;
        let experiment_mode = matches!(p.mode, 3 | 4);
        let head = tx.head();
        let t = now();
        let n = (k - done).min(tx.writable() as usize);
        let by_dma = n > 0 && n * slot >= threshold;
        let mut gen_us = 0u32;
        let mut fill = |j: usize, value: &mut [u32; W]| {
            if p.timed {
                let t = now();
                message(value, i + done as u32 + j as u32);
                gen_us = gen_us.wrapping_add(now().wrapping_sub(t));
            } else {
                message(value, i + done as u32 + j as u32);
            }
        };
        let mut stats = CopyStats::default();
        let mut engine = if experiment_mode {
            LabCopy::Experiment(ExperimentCopy {
                early: p.mode == 3,
                nonposted: p.mode == 3,
                head,
                n: n as u32,
                depth,
                slot,
            })
        } else {
            LabCopy::Production(&mut dma.ch1)
        };
        let mut copy = TimedCopy {
            inner: &mut engine,
            stats: &mut stats,
            timed: p.timed,
        };
        let r = if generated {
            stage.push_with(tx, &mut copy, k - done, &mut fill)
        } else {
            stage.push_slice(tx, &mut copy, &buf[done..k])
        };

        let end = now();
        stats.return_time = end;
        Attempt {
            head,
            by_dma,
            elapsed: end.wrapping_sub(t),
            gen_us,
            stats,
            result: r,
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn run_batches(
    producer: &mut dyn Producer,
    mut irq: Option<&mut HostIrq>,
    count: u32,
    p: &Params,
    start: u32,
    batch: usize,
    depth: u32,
) -> Out {
    let mut out = Out::default();
    let generated = p.mode >= 5;
    let mut i = 0;
    while i < count {
        let k = (count - i).min(batch as u32) as usize;
        if !generated {
            out.gen_us = out.gen_us.wrapping_add(producer.generate(k, i));
        }
        let mut done = 0;
        while done < k {
            let Attempt {
                head,
                by_dma,
                elapsed,
                gen_us,
                stats,
                result,
            } = producer.attempt(k, done, i, p);
            out.gen_us = out.gen_us.wrapping_add(gen_us);
            let end = stats.return_time;
            match result {
                Ok(n) => {
                    if n > 0 {
                        out.push_us = out.push_us.wrapping_add(elapsed);
                        if (head & (depth - 1)) + n as u32 >= depth {
                            out.wraps += 1;
                        }
                        if by_dma {
                            out.ctl_l = unsafe { (CTL as *const u32).read_volatile() };
                            out.dma_batches += 1;
                        } else {
                            out.cpu_batches += 1;
                        }
                        if p.timed {
                            let publish = if stats.blocks > 0 {
                                end.wrapping_sub(stats.last_end)
                            } else {
                                0
                            };
                            out.perf[0] = out.perf[0].wrapping_add(stats.us);
                            out.perf[1] = out.perf[1].wrapping_add(
                                elapsed
                                    .saturating_sub(stats.us)
                                    .saturating_sub(publish)
                                    .saturating_sub(gen_us),
                            );
                            out.perf[2] = out.perf[2].wrapping_add(publish);
                            out.perf[4] += stats.blocks;
                            out.perf[5] += u32::from(stats.blocks == 2);
                            out.perf[6] += stats.width8;
                            out.perf[7] += stats.width16;
                        }
                    } else {
                        out.perf[3] = out.perf[3].wrapping_add(elapsed);
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
                if producer.notify(irq) {
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
        while now().wrapping_sub(t) < p.gap_us {
            core::hint::spin_loop();
        }
    }
    out
}

#[derive(Default)]
struct CopyStats {
    return_time: u32,
    us: u32,
    last_end: u32,
    blocks: u32,
    width8: u32,
    width16: u32,
}
enum LabCopy<'a> {
    Production(&'a mut rp1_hal::dma::Channel<1>),
    Experiment(ExperimentCopy),
}
// SAFETY: production delegates the blocking contract; controls deliberately violate ordering
// only in this lab image, under the same sole stage/channel ownership as ExperimentCopy.
unsafe impl SlotCopy for LabCopy<'_> {
    type Error = Error;
    #[inline(always)]
    unsafe fn copy_to_remote(&mut self, src: &[u8], dst: *mut u8) -> Result<(), Error> {
        match self {
            Self::Production(channel) => unsafe { channel.copy_to_remote(src, dst) },
            Self::Experiment(copy) => unsafe { copy.copy_to_remote(src, dst) },
        }
    }
}
struct TimedCopy<'a, C> {
    inner: &'a mut C,
    stats: &'a mut CopyStats,
    timed: bool,
}
// SAFETY: delegates unchanged to the same blocking copier, preserving its contract.
unsafe impl<C: SlotCopy> SlotCopy for TimedCopy<'_, C> {
    type Error = C::Error;
    #[inline(always)]
    unsafe fn copy_to_remote(&mut self, src: &[u8], dst: *mut u8) -> Result<(), Self::Error> {
        if !self.timed {
            return unsafe { self.inner.copy_to_remote(src, dst) };
        }
        let t = now();
        let r = unsafe { self.inner.copy_to_remote(src, dst) };
        let end = now();
        self.stats.us = self.stats.us.wrapping_add(end.wrapping_sub(t));
        self.stats.last_end = end;
        self.stats.blocks += 1;
        let w = rp1_hal::dma::width_log2(src.as_ptr() as u64, dst as u64, src.len() as u32);
        self.stats.width8 += u32::from(w == 3);
        self.stats.width16 += u32::from(w == 4);
        r
    }
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
