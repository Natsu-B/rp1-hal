//! Command 17: CPU/DMA slice (0/1/2), ordering controls (3/4), and generated
//! DMA/CPU/auto (5/6/7). Optional timing stays in the lab, not in the HAL.
use crate::{
    copy_stats::{CopyStats, TimedCopy},
    dma_proof,
};
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
impl Out {
    /// `--timed` only: split one publishing push into copy, publication and the rest.
    #[rp1_rt::cold]
    fn account_timed(&mut self, elapsed: u32, copy_us: u32, publish: u32, blocks: u32) {
        self.perf[1] =
            self.perf[1].wrapping_add(elapsed.saturating_sub(copy_us).saturating_sub(publish));
        self.perf[2] = self.perf[2].wrapping_add(publish);
        self.perf[5] += u32::from(blocks == 2);
    }
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
    if p.mode > 7 {
        return Out {
            errors: 1,
            ..Out::default()
        };
    }
    // SAFETY: command 16 is not running; this command owns the sole borrow.
    let stage_buf = unsafe { dma_proof::ring_stage() };
    let cap = RingTx::<[u32; W]>::staged_capacity(stage_buf.len());
    let generated = p.mode >= 5;
    let batch = (p.batch as usize).clamp(1, if generated { cap } else { cap.min(buf.len()) });
    let mut stage = RingDma::new(stage_buf).unwrap();
    stage.set_threshold(match p.mode {
        0 | 6 => usize::MAX,
        2 | 7 => RingDma::DEFAULT_THRESHOLD,
        _ => 0,
    });
    let depth = if W == 64 { 128 } else { 256 };
    let engine = if matches!(p.mode, 3 | 4) {
        LabCopy::Experiment(ExperimentCopy {
            early: false,
            nonposted: p.mode == 3,
            head: 0,
            n: 0,
            depth,
            slot: rp1_proto::shm::slot_size(4 * W),
        })
    } else {
        LabCopy::Production(&mut dma.ch1)
    };
    let mut producer = TypedProducer {
        tx,
        message,
        buf,
        stage,
        engine,
    };
    run_batches(&mut producer, irq, count, p, start, batch, depth)
}

/// The per-ring operations, type-erased so that the batch loop exists once and fits in
/// ISRAM (code in shared SRAM runs several times slower, which a small batch pays in full).
trait Producer {
    fn head(&self) -> u32;
    fn notify(&mut self, irq: &mut HostIrq) -> bool;
    /// Slice modes: write messages `first..first + count` to the batch buffer.
    fn generate(&mut self, count: usize, first: u32);
    /// Publish `messages` (indices within the batch that starts at sequence `first`).
    fn push(
        &mut self,
        messages: core::ops::Range<usize>,
        first: u32,
        p: &Params,
        stats: &mut CopyStats,
    ) -> Result<usize, rp1_proto::shm::Staged<Error>>;
}
struct TypedProducer<'a, const W: usize> {
    tx: &'a mut RingTx<[u32; W]>,
    message: fn(&mut [u32; W], u32),
    buf: &'a mut [[u32; W]],
    stage: RingDma<'a>,
    engine: LabCopy<'a>,
}
impl<const W: usize> Producer for TypedProducer<'_, W> {
    fn head(&self) -> u32 {
        self.tx.head()
    }
    fn notify(&mut self, irq: &mut HostIrq) -> bool {
        irq.notify_ring(self.tx)
    }
    #[rp1_rt::isram]
    fn generate(&mut self, count: usize, first: u32) {
        for (j, b) in self.buf[..count].iter_mut().enumerate() {
            (self.message)(b, first + j as u32);
        }
    }
    fn push(
        &mut self,
        messages: core::ops::Range<usize>,
        first: u32,
        p: &Params,
        stats: &mut CopyStats,
    ) -> Result<usize, rp1_proto::shm::Staged<Error>> {
        let Self {
            tx,
            message,
            buf,
            stage,
            engine,
        } = self;
        if let LabCopy::Experiment(copy) = engine {
            copy.early = p.mode == 3;
            copy.head = tx.head();
            copy.n = messages.len().min(tx.writable() as usize) as u32;
        }
        let mut copy = TimedCopy {
            inner: engine,
            stats,
            timed: p.timed,
        };
        if p.mode < 5 {
            return stage.push_slice(tx, &mut copy, &buf[messages]);
        }
        let (message, first) = (*message, first + messages.start as u32);
        stage.push_with(tx, &mut copy, messages.len(), move |j, value| {
            message(value, first + j as u32)
        })
    }
}

#[rp1_rt::isram]
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
    // Cumulative over the run; a batch is judged by what its push added.
    let mut stats = CopyStats::default();
    let mut head = producer.head();
    let mut i = 0;
    'run: while i < count {
        let k = (count - i).min(batch as u32) as usize;
        if p.mode < 5 {
            let t = now();
            producer.generate(k, i);
            out.gen_us = out.gen_us.wrapping_add(now().wrapping_sub(t));
        }
        let mut done = 0;
        while done < k {
            let (blocks, copy_us) = (stats.blocks, stats.us);
            let t = now();
            let result = producer.push(done..k, i, p, &mut stats);
            let end = now();
            let elapsed = end.wrapping_sub(t);
            let Ok(n) = result else {
                out.errors += 1;
                break 'run;
            };
            if n == 0 {
                out.perf[3] = out.perf[3].wrapping_add(elapsed);
            } else {
                out.push_us = out.push_us.wrapping_add(elapsed);
                if (head & (depth - 1)) + n as u32 >= depth {
                    out.wraps += 1;
                }
                head = head.wrapping_add(n as u32);
                done += n;
                // The copier ran: the batch went by DMA whatever was predicted.
                let blocks = stats.blocks - blocks;
                if blocks > 0 {
                    out.ctl_l = unsafe { (CTL as *const u32).read_volatile() };
                    out.dma_batches += 1;
                } else {
                    out.cpu_batches += 1;
                }
                if p.timed {
                    let copy_us = stats.us.wrapping_sub(copy_us);
                    let publish = if blocks > 0 {
                        end.wrapping_sub(stats.last_end)
                    } else {
                        0
                    };
                    out.account_timed(elapsed, copy_us, publish, blocks);
                }
            }
            if let Some(irq) = irq.as_deref_mut() {
                let t = now();
                if producer.notify(irq) {
                    out.pulses += 1;
                }
                out.notify_us = out.notify_us.wrapping_add(now().wrapping_sub(t));
            }
            if end.wrapping_sub(start) > 180_000_000 {
                out.errors += 1;
                break 'run;
            }
        }
        i += k as u32;
        if p.gap_us != 0 {
            let t = now();
            while now().wrapping_sub(t) < p.gap_us {
                core::hint::spin_loop();
            }
        }
    }
    if p.timed {
        out.perf[0] = stats.us;
        out.perf[4] = stats.blocks;
        out.perf[6] = stats.width8;
        out.perf[7] = stats.width16;
    }
    out
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
    // Not a throughput path: keep it out of the ISRAM that the measured loop needs.
    #[rp1_rt::cold]
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
