//! The staged (DMA) producer: copy blocks, fencing, generated payloads.
use super::mem;
use crate::shm::access::{ld, st};
use crate::shm::*;
use crate::{Error, Message};
use core::ptr;
extern crate std;
use std::vec;

/// Synchronous copy satisfying the ordering contract; can fail after a copied block.
struct MemCopy {
    blocks: vec::Vec<(usize, usize)>,
    fail_at: Option<usize>,
}

unsafe impl SlotCopy for MemCopy {
    type Error = ();
    unsafe fn copy_to_remote(&mut self, src: &[u8], dst: *mut u8) -> Result<(), ()> {
        if self.fail_at == Some(self.blocks.len()) {
            return Err(());
        }
        self.blocks.push((dst as usize, src.len()));
        for (i, b) in src.iter().enumerate() {
            unsafe { dst.add(i).write_volatile(*b) };
        }
        Ok(())
    }
}

fn mem_copy() -> MemCopy {
    MemCopy {
        blocks: vec::Vec::new(),
        fail_at: None,
    }
}

#[repr(C, align(8))]
struct Stage([u8; 256]);

#[test]
fn staged_batch_wraps_in_two_blocks_and_publishes_after_copy() {
    // [u32; 3]: 12 B payload, 24 B slots (4 B of padding).
    let mut data = mem::<{ CONTROL_SIZE + 8 * 24 }>();
    let mut ctrl = mem::<CONTROL_SIZE>();
    let (d, c) = (data.0.as_mut_ptr(), ctrl.0.as_mut_ptr());
    unsafe {
        st(d, HEAD, u32::MAX - 2);
        st(c, TAIL, u32::MAX - 2);
    }
    type M = [u32; 3];
    let len = CONTROL_SIZE + 8 * 24;
    let mut tx = unsafe { RingTx::<M>::attach(d, len, c, 128, 8, Overflow::Reject) }.unwrap();
    let mut rx = unsafe { RingRx::<M>::attach(d, len, c, 128, 8, Overflow::Reject) }.unwrap();
    // Garbage in the padding of the remote slots is overwritten with zeros.
    for i in 0..8 {
        unsafe { st(d, CONTROL_SIZE + i * 24 + 20, 0xdead_beef) };
    }
    let mut stage = Stage([0xa5; 256]);
    let values: vec::Vec<M> = (0..20u32).map(|i| [i, !i, i * 7]).collect();
    let mut copy = mem_copy();
    // head & 7 = 5: three slots up to the end of the ring, then two from slot 0.
    assert_eq!(RingTx::<M>::staged_capacity(256), 10);
    assert_eq!(
        tx.push_slice_staged(&values[..5], &mut stage.0, &mut copy),
        Ok(5)
    );
    assert_eq!(
        copy.blocks,
        [
            (d as usize + CONTROL_SIZE + 5 * 24, 3 * 24),
            (d as usize + CONTROL_SIZE, 2 * 24)
        ]
    );
    assert_eq!(unsafe { ld(d, HEAD) }, (u32::MAX - 2).wrapping_add(5));
    assert_eq!(unsafe { ld(d, CONTROL_SIZE + 5 * 24 + 20) }, 0);
    assert_eq!(unsafe { ld(d, CONTROL_SIZE + 5 * 24 + 4) }, 12);
    let mut out = [[0u32; 3]; 8];
    assert_eq!(rx.pop_into(&mut out), Ok(5));
    assert_eq!(out[..5], values[..5]);
    // Bounded by the free slots (8) and the stage (10 slots); wraps again (6 + 2).
    copy.blocks.clear();
    assert_eq!(
        tx.push_slice_staged(&values[5..], &mut stage.0, &mut copy),
        Ok(8)
    );
    assert_eq!(copy.blocks.len(), 2);
    assert_eq!(
        tx.push_slice_staged(&values[13..], &mut stage.0, &mut copy),
        Ok(0)
    );
    assert_eq!(rx.pop_into(&mut out), Ok(8));
    assert_eq!(out[..8], values[5..13]);
    // Mixed with CPU pushes on the same ring.
    assert_eq!(tx.push_slice(&values[13..15]), 2);
    assert_eq!(
        tx.push_slice_staged(&values[15..], &mut stage.0, &mut copy),
        Ok(5)
    );
    assert_eq!(rx.pop_into(&mut out), Ok(7));
    assert_eq!(out[..7], values[13..20]);
}

#[test]
fn failed_copy_fences_all_publish_paths() {
    let mut data = mem::<{ CONTROL_SIZE + 4 * 24 }>();
    let mut ctrl = mem::<CONTROL_SIZE>();
    let (d, c) = (data.0.as_mut_ptr(), ctrl.0.as_mut_ptr());
    let mut tx =
        unsafe { RingTx::<[u32; 3]>::attach(d, data.0.len(), c, 128, 4, Overflow::Reject) }
            .unwrap();
    let mut stage = Stage([0; 256]);
    let mut copy = mem_copy();
    copy.fail_at = Some(0);
    assert_eq!(
        tx.push_slice_staged(&[[1, 2, 0]], &mut stage.0, &mut copy),
        Err(Staged::Copy(()))
    );
    assert!(tx.is_fenced());
    assert_eq!(tx.writable(), 0);
    assert_eq!(tx.free(), 0);
    copy.fail_at = None;
    assert_eq!(
        tx.push_slice_staged(&[[9, 8, 0]], &mut stage.0, &mut copy),
        Err(Staged::Fenced)
    );
    assert_eq!(tx.push(&[9, 8, 0]), Err(Error::NotReady));
    assert_eq!(tx.push_slice(&[[9, 8, 0]]), 0);
    assert_eq!(unsafe { ld(d, HEAD) }, 0);
    assert!(copy.blocks.is_empty());
}

#[test]
fn second_wrap_block_failure_fences_even_with_equal_tails_and_padding() {
    let mut data = mem::<{ CONTROL_SIZE + 4 * 24 }>();
    let mut ctrl = mem::<CONTROL_SIZE>();
    let (d, c) = (data.0.as_mut_ptr(), ctrl.0.as_mut_ptr());
    unsafe {
        st(d, HEAD, 3);
        st(c, TAIL, 3);
    }
    let mut tx =
        unsafe { RingTx::<[u32; 3]>::attach(d, data.0.len(), c, 128, 4, Overflow::Reject) }
            .unwrap();
    let mut stage = Stage([0; 256]);
    let mut copy = mem_copy();
    copy.fail_at = Some(1);
    let values = [[1, 2, 0], [3, 4, 0]];
    assert_eq!(
        tx.push_slice_staged(&values, &mut stage.0, &mut copy),
        Err(Staged::Copy(()))
    );
    assert_eq!(copy.blocks.len(), 1);
    assert_eq!(unsafe { ld(d, HEAD) }, 3);
    // A late interior write can still land, but no retry/CPU push reuses its slot.
    unsafe {
        st(d, CONTROL_SIZE + 8, 99);
    }
    assert_eq!(tx.push_slice(&[[8, 9, 0]]), 0);
    assert_eq!(
        tx.push_slice_staged(&[[8, 9, 0]], &mut stage.0, &mut copy),
        Err(Staged::Fenced)
    );
    assert_eq!(unsafe { ld(d, CONTROL_SIZE + 8) }, 99);
}

#[test]
fn staged_rejects_bad_stages_and_overwrite_uses_the_cpu_path() {
    let mut data = mem::<{ CONTROL_SIZE + 4 * 16 }>();
    let mut ctrl = mem::<CONTROL_SIZE>();
    let (d, c) = (data.0.as_mut_ptr(), ctrl.0.as_mut_ptr());
    let len = CONTROL_SIZE + 4 * 16;
    let mut tx = unsafe { RingTx::<u64>::attach(d, len, c, 128, 4, Overflow::Reject) }.unwrap();
    let mut stage = Stage([0; 256]);
    let mut copy = mem_copy();
    assert_eq!(
        tx.push_slice_staged(&[1], &mut stage.0[4..], &mut copy),
        Err(Staged::Invalid)
    );
    assert_eq!(
        tx.push_slice_staged(&[1], &mut stage.0[..15], &mut copy),
        Err(Staged::Invalid)
    );
    assert!(copy.blocks.is_empty());
    let mut ow = unsafe { RingTx::<u64>::attach(d, len, c, 128, 4, Overflow::Overwrite) }.unwrap();
    assert_eq!(
        ow.push_slice_staged(&[1, 2], &mut stage.0, &mut copy),
        Ok(2)
    );
    assert!(copy.blocks.is_empty());
}

#[test]
fn generated_slots_match_slice_across_wrap_and_cpu_batches() {
    for start in [0, 5, u32::MAX - 2] {
        let mut a = mem::<{ CONTROL_SIZE + 8 * 24 }>();
        let mut b = mem::<{ CONTROL_SIZE + 8 * 24 }>();
        let mut ctrl = mem::<CONTROL_SIZE>();
        let c = ctrl.0.as_mut_ptr();
        unsafe {
            st(a.0.as_mut_ptr(), HEAD, start);
            st(b.0.as_mut_ptr(), HEAD, start);
            st(c, TAIL, start);
        }
        let mut slice = unsafe {
            RingTx::<[u32; 3]>::attach(a.0.as_mut_ptr(), a.0.len(), c, 128, 8, Overflow::Reject)
        }
        .unwrap();
        let mut generated = unsafe {
            RingTx::<[u32; 3]>::attach(b.0.as_mut_ptr(), b.0.len(), c, 128, 8, Overflow::Reject)
        }
        .unwrap();
        let values = [[1, 2, 3], [4, 5, 6], [7, 8, 9], [10, 11, 12], [13, 14, 15]];
        let (mut sa, mut sb) = (Stage([0xa5; 256]), Stage([0x5a; 256]));
        let (mut ca, mut cb) = (mem_copy(), mem_copy());
        assert_eq!(slice.push_slice_staged(&values, &mut sa.0, &mut ca), Ok(5));
        let mut calls = 0;
        let stage_start = sb.0.as_ptr() as usize;
        assert_eq!(
            generated.push_with_staged(5, &mut sb.0, &mut cb, |k, v| {
                assert_eq!(k, calls);
                assert_eq!(v as *mut [u32; 3] as usize, stage_start + k * 24 + 8);
                *v = values[k];
                calls += 1;
            }),
            Ok(5)
        );
        assert_eq!(calls, 5);
        assert_eq!(sa.0[..120], sb.0[..120]);
        assert_eq!(a.0, b.0);
        assert_eq!(slice.push_slice(&values[..2]), 2);
        assert_eq!(generated.push_with(2, |k, v| *v = values[k]), 2);
        assert_eq!(a.0, b.0);
        // Only one free slot, then full; callbacks must follow actual accepted count.
        assert_eq!(
            generated.push_with_staged(20, &mut sb.0, &mut cb, |k, v| {
                assert_eq!(k, 0);
                *v = values[0];
            }),
            Ok(1)
        );
        assert_eq!(
            generated.push_with_staged(20, &mut sb.0, &mut cb, |_, _| panic!("full")),
            Ok(0)
        );
        assert_eq!(generated.push_with(20, |_, _| panic!("full")), 0);
    }
}

#[test]
fn over_aligned_generator_uses_one_temporary_on_every_path() {
    #[repr(C, align(32))]
    #[derive(Clone, Copy)]
    struct Aligned([u32; 8]);
    // SAFETY: words only, all bit patterns valid, no padding.
    unsafe impl Message for Aligned {
        const TYPE_ID: u64 = 0x70617274;
    }
    #[repr(align(32))]
    struct AlignedStage([u8; 256]);
    // Offset 24 would put the payload at a 32-byte boundary: alignment above the stage's
    // 8 bytes still never borrows the stage.
    for offset in [0, 24] {
        for staged in [false, true] {
            let (mut data, mut ctrl) = (mem::<1024>(), mem::<CONTROL_SIZE>());
            let (d, c) = (data.0.as_mut_ptr(), ctrl.0.as_mut_ptr());
            let mut tx =
                unsafe { RingTx::<Aligned>::attach(d, 1024, c, 128, 8, Overflow::Reject) }.unwrap();
            let mut rx =
                unsafe { RingRx::<Aligned>::attach(d, 1024, c, 128, 8, Overflow::Reject) }.unwrap();
            let mut stage = AlignedStage([0xa5; 256]);
            let range = stage.0.as_ptr_range();
            let mut copy = mem_copy();
            let mut seen = [0usize; 2];
            let mut fill = |k: usize, v: &mut Aligned| {
                seen[k] = v as *mut Aligned as usize;
                assert!(!range.contains(&(seen[k] as *const u8)));
                v.0 = [k as u32 + 1; 8];
            };
            if staged {
                assert_eq!(
                    tx.push_with_staged(8, &mut stage.0[offset..offset + 80], &mut copy, &mut fill),
                    Ok(2)
                );
            } else {
                assert_eq!(tx.push_with(2, &mut fill), 2);
            }
            assert_eq!(seen[0], seen[1]);
            assert_eq!(seen[0] % 32, 0);
            for k in 0..2 {
                assert_eq!(rx.pop().unwrap().unwrap().0, [k + 1; 8]);
            }
        }
    }
}

#[test]
fn generated_head_is_hidden_until_both_blocks_complete() {
    struct Checked {
        head: *const u8,
        calls: usize,
    }
    // SAFETY: synchronous host copy; the observation does not change ordering.
    unsafe impl SlotCopy for Checked {
        type Error = ();
        unsafe fn copy_to_remote(&mut self, src: &[u8], dst: *mut u8) -> Result<(), ()> {
            assert_eq!(unsafe { ld(self.head, HEAD) }, u32::MAX);
            self.calls += 1;
            unsafe {
                ptr::copy_nonoverlapping(src.as_ptr(), dst, src.len());
            }
            Ok(())
        }
    }
    let mut data = mem::<256>();
    let mut ctrl = mem::<128>();
    let (d, c) = (data.0.as_mut_ptr(), ctrl.0.as_mut_ptr());
    unsafe {
        st(d, HEAD, u32::MAX);
        st(c, TAIL, u32::MAX);
    }
    let mut tx = unsafe { RingTx::<u8>::attach(d, 256, c, 128, 4, Overflow::Reject) }.unwrap();
    let mut copy = Checked { head: d, calls: 0 };
    let mut stage = Stage([0xa5; 256]);
    assert_eq!(
        tx.push_with_staged(2, &mut stage.0, &mut copy, |k, v| *v = k as u8),
        Ok(2)
    );
    assert_eq!(copy.calls, 2);
    assert_eq!(unsafe { ld(d, HEAD) }, 1);
    assert_eq!(&stage.0[9..16], &[0; 7]);
    assert_eq!(&stage.0[25..32], &[0; 7]);
    assert_eq!(tx.push_with(2, |_, v| *v = 8), 2);
    assert_eq!(tx.push(&9), Err(Error::Full));
    unsafe { st(c, TAIL, tx.head()) };
    let head = tx.head();
    let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        tx.push_with(2, |k, v| {
            if k == 1 {
                panic!("cpu generator");
            }
            *v = 7;
        });
    }));
    assert!(panic.is_err());
    assert_eq!(tx.head(), head);
    assert_eq!(unsafe { ld(d, HEAD) }, head);
    assert_eq!(unsafe { ld(d, DROPPED) }, 0);
    assert_eq!(tx.push_with(1, |_, v| *v = 9), 1);
    assert_eq!(unsafe { ld(d, DROPPED) }, 1);
}

#[test]
fn generated_stage_validation_and_overwrite() {
    let mut data = mem::<1024>();
    let mut ctrl = mem::<CONTROL_SIZE>();
    let (d, c) = (data.0.as_mut_ptr(), ctrl.0.as_mut_ptr());
    let mut stage = Stage([0xa5; 256]);
    let mut copy = mem_copy();
    for overflow in [Overflow::Reject, Overflow::Overwrite] {
        data.0.fill(0);
        ctrl.0.fill(0);
        let mut tx = unsafe { RingTx::<u8>::attach(d, 1024, c, 128, 4, overflow) }.unwrap();
        assert_eq!(
            tx.push_with_staged(1, &mut stage.0[1..], &mut copy, |_, _| panic!("invalid")),
            Err(Staged::Invalid)
        );
        assert_eq!(
            tx.push_with_staged(1, &mut stage.0[..8], &mut copy, |_, _| panic!("invalid")),
            Err(Staged::Invalid)
        );
        let before = copy.blocks.len();
        let mut calls = 0;
        let n = tx
            .push_with_staged(7, &mut stage.0, &mut copy, |k, v| {
                calls += 1;
                *v = k as u8;
            })
            .unwrap();
        assert_eq!(n, if overflow == Overflow::Reject { 4 } else { 7 });
        assert_eq!(calls, n);
        if overflow == Overflow::Overwrite {
            assert_eq!(copy.blocks.len(), before);
        }
    }
}

#[test]
fn generated_panic_never_copies_or_publishes_and_copy_errors_fence() {
    for fail_at in [0, 1] {
        let mut data = mem::<{ CONTROL_SIZE + 4 * 16 }>();
        let mut ctrl = mem::<CONTROL_SIZE>();
        let (d, c) = (data.0.as_mut_ptr(), ctrl.0.as_mut_ptr());
        unsafe {
            st(d, HEAD, 3);
            st(c, TAIL, 3);
        }
        let mut tx =
            unsafe { RingTx::<u8>::attach(d, data.0.len(), c, 128, 4, Overflow::Reject) }.unwrap();
        let mut stage = Stage([0; 256]);
        let mut copy = mem_copy();
        let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = tx.push_with_staged(2, &mut stage.0, &mut copy, |k, v| {
                if k == 1 {
                    panic!("generator");
                }
                *v = 7;
            });
        }));
        assert!(panic.is_err());
        assert!(copy.blocks.is_empty());
        assert_eq!(tx.head(), 3);
        assert_eq!(unsafe { ld(d, HEAD) }, 3);
        copy.fail_at = Some(fail_at);
        assert_eq!(
            tx.push_with_staged(2, &mut stage.0, &mut copy, |_, v| *v = 9),
            Err(Staged::Copy(()))
        );
        assert_eq!(copy.blocks.len(), fail_at);
        assert_eq!(tx.head(), 3);
        assert_eq!(unsafe { ld(d, HEAD) }, 3);
        assert_eq!(tx.push_with(2, |_, _| panic!("fenced")), 0);
        assert_eq!(
            tx.push_with_staged(2, &mut stage.0, &mut copy, |_, _| panic!("fenced")),
            Err(Staged::Fenced)
        );
        assert_eq!(tx.push(&1), Err(Error::NotReady));
        assert_eq!(tx.push_slice(&[1]), 0);
        assert_eq!(
            tx.push_slice_staged(&[1], &mut stage.0, &mut copy),
            Err(Staged::Fenced)
        );
    }
}
