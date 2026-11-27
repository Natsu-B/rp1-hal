//! The DMA ring producer: RP1 -> Linux batches copied into Linux DDR by a DMA channel.
use crate::{
    RingTx,
    notify::{Notify, sealed::Token},
    ring::window_ready,
};
use rp1_hal::dma;
pub use rp1_proto::shm::{SlotCopy, Staged};
use rp1_proto::{Message, shm};

/// Moves RP1 → Linux ring batches into Linux DDR by DMA ([`RingTx::push_slice_dma`]):
/// the slot images are staged in a shared-SRAM buffer and copied by a channel of the
/// declared DMA controller (`dma = {}`; the caller lends the channel per call, so the same
/// channel can serve several rings). `head` is published only after the copy finished and
/// satisfies SlotCopy ordering (docs/dma.md, "DMA ring producer").
///
/// Batches whose slot images are smaller than the threshold use the CPU path
/// ([`RingTx::push_slice`], posted word stores), which is faster for them.
pub struct RingDma<'s> {
    stage: &'s mut [u8],
    threshold: usize,
}
impl<'s> RingDma<'s> {
    /// Provisional CPU/DMA crossover in staged bytes; tune per workload.
    pub const DEFAULT_THRESHOLD: usize = 256;
    /// `stage` must be shared SRAM (`#[unsafe(link_section = ".shared")]`) and 8-byte aligned;
    /// its size bounds one DMA batch ([`shm::RingTx::staged_capacity`]).
    pub fn new(stage: &'s mut [u8]) -> Result<Self, dma::Error> {
        use dma::{BusAddr, Error};
        let start = stage.as_ptr() as usize;
        let sram = BusAddr::SRAM..BusAddr::SRAM + 0x1_0000;
        if stage.is_empty() || !start.is_multiple_of(8) {
            return Err(Error::Invalid);
        }
        let first = BusAddr::from_proc(start)?.0;
        let last = BusAddr::from_proc(start + stage.len() - 1)?.0;
        if !sram.contains(&first) || !sram.contains(&last) {
            return Err(Error::NotDmaVisible);
        }
        Ok(Self {
            stage,
            threshold: Self::DEFAULT_THRESHOLD,
        })
    }
    /// Batches of at least `bytes` staged bytes go by DMA (`0` = always when a slot fits the
    /// stage, `usize::MAX` = never).
    pub fn set_threshold(&mut self, bytes: usize) {
        self.threshold = bytes;
    }
    pub fn threshold(&self) -> usize {
        self.threshold
    }
    /// Whether up to `requested` messages go by DMA: the batch that fits the stage and the
    /// ring's free slots reaches the threshold.
    #[rp1_rt::isram]
    fn use_dma<T: Message>(&self, ring: &mut shm::RingTx<T>, requested: usize) -> bool {
        let slot = rp1_proto::shm::slot_size(size_of::<T>());
        let min_msgs = self.threshold.div_ceil(slot).max(1);
        let n = requested.min(shm::RingTx::<T>::staged_capacity(self.stage.len()));
        // CPU-only candidates need no tail read before the push checks the free slots.
        n >= min_msgs && ring.writable() as usize >= min_msgs
    }
    #[rp1_rt::isram]
    fn push_slice<T: Message, C: SlotCopy>(
        &mut self,
        ring: &mut shm::RingTx<T>,
        copy: &mut C,
        values: &[T],
    ) -> Result<usize, Staged<C::Error>> {
        if ring.is_fenced() {
            return Err(Staged::Fenced);
        }
        if !self.use_dma(ring, values.len()) {
            return Ok(ring.push_slice(values));
        }
        ring.push_slice_staged(values, self.stage, copy)
    }
    #[rp1_rt::isram]
    fn push_with<T: Message, C: SlotCopy>(
        &mut self,
        ring: &mut shm::RingTx<T>,
        copy: &mut C,
        count: usize,
        fill: impl FnMut(usize, &mut T),
    ) -> Result<usize, Staged<C::Error>> {
        if ring.is_fenced() {
            return Err(Staged::Fenced);
        }
        if !self.use_dma(ring, count) {
            return Ok(ring.push_with(count, fill));
        }
        ring.push_with_staged(count, self.stage, copy, fill)
    }
}

impl<T: Message, N: Notify> RingTx<T, N> {
    /// Publish up to `values.len()` messages like [`RingTx::push_slice`] (returns how many):
    /// by `copy` when the batch reaches `dma`'s threshold (then at most
    /// [`shm::RingTx::staged_capacity`] of the stage), otherwise by the CPU. On error nothing
    /// was published. A channel implements [`SlotCopy`]; a wrapper may also implement that
    /// same blocking ordering contract. Does not wake the reader; follow with
    /// [`RingTx::flush`], or use [`RingTx::send_slice_dma`].
    #[inline(always)]
    pub fn push_slice_dma<C: SlotCopy>(
        &mut self,
        dma: &mut RingDma<'_>,
        copy: &mut C,
        values: &[T],
    ) -> Result<usize, Staged<C::Error>> {
        window_ready().map_err(|_| Staged::Reset)?;
        dma.push_slice(&mut self.ring, copy, values)
    }
    /// Generate accepted payloads directly in the stage when the writable batch reaches
    /// the DMA threshold. Otherwise use [`RingTx::push_with`] (including overwrite rings).
    /// Follows the [generation tradeoff and callback contract](shm::RingTx::push_with).
    /// See [`shm::RingTx::push_with_staged`] for alignment, panic and failure semantics.
    /// No callback is invoked when fenced, full, or when stage validation fails.
    #[inline(always)]
    pub fn push_with_dma<C: SlotCopy>(
        &mut self,
        dma: &mut RingDma<'_>,
        copy: &mut C,
        count: usize,
        fill: impl FnMut(usize, &mut T),
    ) -> Result<usize, Staged<C::Error>> {
        window_ready().map_err(|_| Staged::Reset)?;
        dma.push_with(&mut self.ring, copy, count, fill)
    }
    /// [`RingTx::push_slice_dma`], then wake the reader if it sleeps (also after an error or a
    /// full ring: the reader may have armed meanwhile). The wake reads `head` back; it does not
    /// verify the payload or establish Channel's AXI-to-PCIe ordering chain (HYP, docs/dma.md).
    #[inline(always)]
    pub fn send_slice_dma<C: SlotCopy>(
        &mut self,
        dma: &mut RingDma<'_>,
        copy: &mut C,
        values: &[T],
    ) -> Result<usize, Staged<C::Error>> {
        window_ready().map_err(|_| Staged::Reset)?;
        let sent = dma.push_slice(&mut self.ring, copy, values);
        self.notify.wake(Token(()), &mut self.ring);
        sent
    }
    /// [`RingTx::push_with_dma`], then wake the reader as [`RingTx::send_slice_dma`] does.
    #[inline(always)]
    pub fn send_with_dma<C: SlotCopy>(
        &mut self,
        dma: &mut RingDma<'_>,
        copy: &mut C,
        count: usize,
        fill: impl FnMut(usize, &mut T),
    ) -> Result<usize, Staged<C::Error>> {
        window_ready().map_err(|_| Staged::Reset)?;
        let sent = dma.push_with(&mut self.ring, copy, count, fill);
        self.notify.wake(Token(()), &mut self.ring);
        sent
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rp1_proto::shm::{Overflow, RingTx};

    #[repr(align(8))]
    struct Mem([u8; 1024]);
    impl Mem {
        fn zeroed() -> Self {
            Self([0; 1024])
        }
    }
    fn ring_and_stage<'a>(
        data: &mut Mem,
        ctrl: &Mem,
        stage: &'a mut Mem,
    ) -> (RingTx<[u32; 4]>, RingDma<'a>) {
        let ring = unsafe {
            RingTx::attach(
                data.0.as_mut_ptr(),
                1024,
                ctrl.0.as_ptr(),
                1024,
                32,
                Overflow::Reject,
            )
        }
        .unwrap();
        (
            ring,
            RingDma {
                stage: &mut stage.0,
                threshold: RingDma::DEFAULT_THRESHOLD,
            },
        )
    }
    struct Fail;
    // SAFETY: performs no accesses and returns failure synchronously.
    unsafe impl SlotCopy for Fail {
        type Error = ();
        unsafe fn copy_to_remote(&mut self, _: &[u8], _: *mut u8) -> Result<(), ()> {
            Err(())
        }
    }

    struct MustNotCopy;
    // SAFETY: performs no reads or writes. The test only exercises paths that bypass DMA.
    unsafe impl SlotCopy for MustNotCopy {
        type Error = ();
        unsafe fn copy_to_remote(&mut self, _: &[u8], _: *mut u8) -> Result<(), ()> {
            panic!("CPU fallback invoked DMA")
        }
    }

    #[test]
    fn dma_threshold_uses_only_writable_slots() {
        let (mut data, mut ctrl, mut stage) = (Mem::zeroed(), Mem::zeroed(), Mem::zeroed());
        let (mut ring, mut helper) = ring_and_stage(&mut data, &ctrl, &mut stage);
        assert_eq!(ring.push_slice(&[[0; 4]; 31]), 31);
        assert!(!helper.use_dma(&mut ring, 32));
        assert_eq!(
            helper.push_with(&mut ring, &mut Fail, 32, |k, v| {
                assert_eq!(k, 0);
                *v = [42; 4];
            }),
            Ok(1)
        );
        // A fresh tail read must observe consumer progress after the previous decision.
        unsafe { ctrl.0.as_mut_ptr().cast::<u32>().write_volatile(32) };
        assert!(helper.use_dma(&mut ring, 32));
    }

    #[test]
    fn generated_copy_failure_fences_retries() {
        let (mut data, ctrl, mut stage) = (Mem::zeroed(), Mem::zeroed(), Mem::zeroed());
        let (mut ring, mut helper) = ring_and_stage(&mut data, &ctrl, &mut stage);
        assert_eq!(
            helper.push_with(&mut ring, &mut Fail, 32, |_, v| *v = [43; 4]),
            Err(Staged::Copy(()))
        );
        assert_eq!(ring.head(), 0);
        assert_eq!(
            helper.push_with(&mut ring, &mut Fail, 32, |_, _| panic!("fenced")),
            Err(Staged::Fenced)
        );
        assert_eq!(
            helper.push_slice(&mut ring, &mut Fail, &[[0; 4]]),
            Err(Staged::Fenced)
        );
    }

    #[test]
    fn cpu_fallback_progresses_without_staging_capacity() {
        #[repr(align(8))]
        struct Mem([u8; 2048]);
        for (stage_len, threshold) in [
            (256, RingDma::DEFAULT_THRESHOLD), // smaller than one 264-byte slot
            (256, 0),                          // even "always DMA" must make progress
            (256, usize::MAX),
            (264, usize::MAX), // one staged slot must not cap the CPU batch
        ] {
            for overflow in [Overflow::Reject, Overflow::Overwrite] {
                let (mut data, ctrl, mut stage) = (Mem([0; 2048]), Mem([0; 2048]), Mem([0; 2048]));
                let mut ring = unsafe {
                    RingTx::<[u32; 64]>::attach(
                        data.0.as_mut_ptr(),
                        data.0.len(),
                        ctrl.0.as_ptr(),
                        ctrl.0.len(),
                        4,
                        overflow,
                    )
                }
                .unwrap();
                let mut helper = RingDma {
                    stage: &mut stage.0[..stage_len],
                    threshold,
                };
                let values = [[42; 64]; 3];
                assert_eq!(
                    helper.push_slice(&mut ring, &mut MustNotCopy, &values),
                    Ok(3)
                );
            }
        }
    }

    #[test]
    fn overwrite_cpu_batch_ignores_staging_capacity() {
        #[repr(align(8))]
        struct Mem([u8; 2048]);
        let (mut data, ctrl, mut stage) = (Mem([0; 2048]), Mem([0; 2048]), Mem([0; 2048]));
        let mut ring = unsafe {
            RingTx::<[u32; 64]>::attach(
                data.0.as_mut_ptr(),
                data.0.len(),
                ctrl.0.as_ptr(),
                ctrl.0.len(),
                4,
                Overflow::Overwrite,
            )
        }
        .unwrap();
        let mut helper = RingDma {
            stage: &mut stage.0[..264],
            threshold: RingDma::DEFAULT_THRESHOLD,
        };
        // Overwrite must use the CPU even when one staged slot reaches the DMA threshold.
        assert_eq!(
            helper.push_slice(&mut ring, &mut MustNotCopy, &[[42; 64]; 3]),
            Ok(3)
        );
        assert_eq!(ring.head(), 3);
        // An overwrite CPU batch may also exceed the ring depth.
        assert_eq!(
            helper.push_slice(&mut ring, &mut MustNotCopy, &[[43; 64]; 7]),
            Ok(7)
        );
        assert_eq!(ring.head(), 10);
    }

    #[test]
    fn ring_dma_stage_must_be_aligned_shared_sram() {
        use super::RingDma;
        use dma::Error;
        // Host memory is not DMAC-visible; a stage is checked before any use.
        #[repr(align(8))]
        struct S([u8; 64]);
        let mut s = S([0; 64]);
        assert!(matches!(RingDma::new(&mut s.0), Err(Error::NotDmaVisible)));
        assert!(matches!(RingDma::new(&mut s.0[1..]), Err(Error::Invalid)));
        assert!(matches!(RingDma::new(&mut []), Err(Error::Invalid)));
    }

    /// The endpoint methods: the DMA path (here its CPU fallback) wakes a sleeping reader once,
    /// and a lost window is reported before any ring access.
    #[test]
    fn send_slice_dma_wakes_and_a_lost_window_is_untouched() {
        use crate::mmio::fake::{self, Access};
        use crate::notify::{HostIrq, sealed::Notify};
        use rp1_proto::{LINK_START, link};
        const STATUS: usize = LINK_START as usize + link::SHM_STATUS;
        const MSIX_SET: usize = 0x4010_8008 + 4 * rp1_proto::irq::VECTOR as usize + 0x800;

        let (mut data, mut ctrl, mut stage) = (Mem::zeroed(), Mem::zeroed(), Mem::zeroed());
        let (ring, mut helper) = ring_and_stage(&mut data, &ctrl, &mut stage);
        helper.set_threshold(usize::MAX);
        let mut tx = crate::RingTx::new(ring, HostIrq::claim(Token(())));
        // The reader sleeps (odd `waiting`).
        let waiting = unsafe { ctrl.0.as_mut_ptr().add(rp1_proto::shm::WAITING) };
        unsafe { waiting.cast::<u32>().write_volatile(1) };
        fake::set(STATUS, link::SHM_READY);
        fake::take();
        assert_eq!(
            tx.send_slice_dma(&mut helper, &mut Fail, &[[1; 4]; 2]),
            Ok(2)
        );
        assert_eq!(
            tx.send_with_dma(&mut helper, &mut Fail, 1, |_, v| *v = [2; 4]),
            Ok(1)
        );
        let pulses = fake::take();
        assert_eq!(
            pulses
                .iter()
                .filter(|a| **a == Access::Write(MSIX_SET, 2))
                .count(),
            1
        );
        assert_eq!(tx.push_slice_dma(&mut helper, &mut Fail, &[[3; 4]]), Ok(1));
        assert_eq!(fake::take(), []);

        fake::set(STATUS, link::SHM_WINDOW_LOST);
        let before = data.0;
        assert_eq!(
            tx.send_slice_dma(&mut helper, &mut Fail, &[[4; 4]]),
            Err(Staged::Reset)
        );
        assert_eq!(
            tx.push_with_dma(&mut helper, &mut Fail, 1, |_, _| panic!("filled")),
            Err(Staged::Reset)
        );
        assert_eq!((data.0, tx.head()), (before, 4));
        assert_eq!(fake::take(), []);
    }
}
