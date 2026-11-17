//! Overwrite-mode rings: lost-message accounting across index laps.
use super::mem;
use crate::Error;
use crate::shm::access::{ld, st};
use crate::shm::*;
extern crate std;
use std::vec;

#[test]
fn overwrite_pop_into_reports_lost() {
    let mut data = mem::<{ CONTROL_SIZE + 4 * 16 }>();
    let mut ctrl = mem::<CONTROL_SIZE>();
    let (d, c) = (data.0.as_mut_ptr(), ctrl.0.as_mut_ptr());
    let mut tx = unsafe { RingTx::<u64>::attach(d, 192, c, 128, 4, Overflow::Overwrite) }.unwrap();
    let mut rx = unsafe { RingRx::<u64>::attach(d, 192, c, 128, 4, Overflow::Overwrite) }.unwrap();
    assert_eq!(tx.push_slice(&[0, 1, 2, 3, 4, 5]), 6);
    let mut out = [0u64; 8];
    assert_eq!(rx.pop_into(&mut out), Err(Error::Lost(2)));
    assert_eq!(rx.pop_into(&mut out), Ok(1));
    assert_eq!(out[0], 2);
}

#[test]
fn overwrite_ring_reports_lost_and_follows() {
    let mut data = mem::<{ CONTROL_SIZE + 4 * 16 }>();
    let mut ctrl = mem::<CONTROL_SIZE>();
    let (d, c) = (data.0.as_mut_ptr(), ctrl.0.as_mut_ptr());
    let mut tx = unsafe { RingTx::<u64>::attach(d, 192, c, 128, 4, Overflow::Overwrite) }.unwrap();
    let mut rx = unsafe { RingRx::<u64>::attach(d, 192, c, 128, 4, Overflow::Overwrite) }.unwrap();
    for i in 0..10u64 {
        tx.push(&i).unwrap();
    }
    assert_eq!(rx.pop(), Err(Error::Lost(6)));
    let got: vec::Vec<_> = core::iter::from_fn(|| rx.pop().unwrap()).collect();
    assert_eq!(got, [6, 7, 8, 9]);
    // A slot marked busy is not returned.
    tx.push(&10).unwrap();
    unsafe { st(d, CONTROL_SIZE + (10 & 3) * 16 + 4, BUSY | 8) };
    assert_eq!(rx.pop(), Err(Error::NotReady));
}

#[test]
fn overwrite_stopped_reader_survives_full_index_lap() {
    // REVIEW2 #4: a reader stopped at tail=0 while the writer runs 2^32 pushes.
    let mut data = mem::<{ CONTROL_SIZE + 4 * 16 }>();
    let mut ctrl = mem::<CONTROL_SIZE>();
    let (d, c) = (data.0.as_mut_ptr(), ctrl.0.as_mut_ptr());
    unsafe { st(d, HEAD, u32::MAX - 3) };
    let mut tx = unsafe { RingTx::<u64>::attach(d, 192, c, 128, 4, Overflow::Overwrite) }.unwrap();
    let mut rx = unsafe { RingRx::<u64>::attach(d, 192, c, 128, 4, Overflow::Overwrite) }.unwrap();
    for i in 0..4u64 {
        tx.push(&i).unwrap();
    }
    assert_eq!(unsafe { (ld(d, HEAD), ld(d, HEAD_HI)) }, (0, 1));
    assert_eq!(rx.available(), u32::MAX);
    assert_eq!(rx.pop(), Err(Error::Lost(u32::MAX - 3)));
    let got: vec::Vec<_> = core::iter::from_fn(|| rx.pop().unwrap()).collect();
    assert_eq!(got, [0, 1, 2, 3]);
    assert_eq!(unsafe { (ld(c, TAIL), ld(c, TAIL_HI)) }, (0, 1));
    // The reader's lap survives a restart through the shared `tail_hi`.
    tx.push(&4).unwrap();
    let mut again =
        unsafe { RingRx::<u64>::attach(d, 192, c, 128, 4, Overflow::Overwrite) }.unwrap();
    assert_eq!(again.pop(), Ok(Some(4)));
    assert_eq!(again.pop(), Ok(None));
}

#[test]
fn overwrite_lost_beyond_u32_is_reported_in_parts() {
    let mut data = mem::<{ CONTROL_SIZE + 4 * 16 }>();
    let mut ctrl = mem::<CONTROL_SIZE>();
    let (d, c) = (data.0.as_mut_ptr(), ctrl.0.as_mut_ptr());
    unsafe { st(d, HEAD, u32::MAX - 3) };
    unsafe { st(d, HEAD_HI, 1) };
    let mut tx = unsafe { RingTx::<u64>::attach(d, 192, c, 128, 4, Overflow::Overwrite) }.unwrap();
    let mut rx = unsafe { RingRx::<u64>::attach(d, 192, c, 128, 4, Overflow::Overwrite) }.unwrap();
    for i in 0..4u64 {
        tx.push(&i).unwrap();
    }
    // 2^33 pushed, depth 4 kept: 2^33 - 4 lost in total.
    let mut lost = 0u64;
    loop {
        match rx.pop() {
            Err(Error::Lost(n)) => lost += u64::from(n),
            Ok(Some(v)) => {
                assert_eq!(v, 0);
                break;
            }
            other => panic!("{other:?}"),
        }
    }
    assert_eq!(lost, (1u64 << 33) - 4);
}

#[test]
fn overwrite_torn_head_reads_as_empty() {
    // `head` wrapped but `head_hi` not yet visible: the reader must not see 2^32 lost.
    let mut data = mem::<{ CONTROL_SIZE + 4 * 16 }>();
    let mut ctrl = mem::<CONTROL_SIZE>();
    let (d, c) = (data.0.as_mut_ptr(), ctrl.0.as_mut_ptr());
    unsafe {
        st(c, TAIL, u32::MAX);
        st(d, HEAD, 0);
    }
    let mut rx = unsafe { RingRx::<u64>::attach(d, 192, c, 128, 4, Overflow::Overwrite) }.unwrap();
    assert_eq!(rx.pop(), Ok(None));
    assert_eq!(rx.available(), 0);
}
