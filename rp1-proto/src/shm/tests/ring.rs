//! The CPU ring paths: full, wrap, resume, batches, generated payloads, invalid attachments.
use super::mem;
use crate::Error;
use crate::shm::access::{ld, st};
use crate::shm::*;
extern crate std;
use std::vec;

#[test]
fn reject_ring_full_wrap_and_resume() {
    let mut data = mem::<{ CONTROL_SIZE + 4 * 16 }>();
    let mut ctrl = mem::<CONTROL_SIZE>();
    let (d, c) = (data.0.as_mut_ptr(), ctrl.0.as_mut_ptr());
    let mut tx =
        unsafe { RingTx::<[u32; 2]>::attach(d, 192, c, 128, 4, Overflow::Reject) }.unwrap();
    let mut rx =
        unsafe { RingRx::<[u32; 2]>::attach(d, 192, c, 128, 4, Overflow::Reject) }.unwrap();
    assert_eq!(rx.pop(), Ok(None));
    for i in 0..4 {
        tx.push(&[i, !i]).unwrap();
    }
    assert_eq!(tx.push(&[9, 9]), Err(Error::Full));
    assert_eq!(tx.push(&[9, 9]), Err(Error::Full));
    // Spinning on Full stores nothing remotely; the count travels with the next push.
    assert_eq!((tx.dropped(), rx.dropped()), (2, 0));
    for round in 0..1000u32 {
        assert_eq!(rx.pop(), Ok(Some([round, !round])));
        tx.push(&[round + 4, !(round + 4)]).unwrap();
        assert_eq!(rx.dropped(), 2);
    }
    // A restarted reader resumes from the shared tail.
    let mut again =
        unsafe { RingRx::<[u32; 2]>::attach(d, 192, c, 128, 4, Overflow::Reject) }.unwrap();
    assert_eq!(again.pop(), Ok(Some([1000, !1000])));
    // Corruption of a slot sequence is detected.
    unsafe { st(d, CONTROL_SIZE + 16, 7) };
    assert_eq!(again.pop(), Err(Error::Corrupt));
}

#[test]
fn batch_push_and_pop_publish_once() {
    let mut data = mem::<{ CONTROL_SIZE + 8 * 16 }>();
    let mut ctrl = mem::<CONTROL_SIZE>();
    let (d, c) = (data.0.as_mut_ptr(), ctrl.0.as_mut_ptr());
    unsafe {
        st(d, HEAD, u32::MAX - 2);
        st(c, TAIL, u32::MAX - 2);
    }
    let mut tx = unsafe { RingTx::<u64>::attach(d, 256, c, 128, 8, Overflow::Reject) }.unwrap();
    let mut rx = unsafe { RingRx::<u64>::attach(d, 256, c, 128, 8, Overflow::Reject) }.unwrap();
    // Reject-mode slot lengths are written once by the writer's attach.
    for i in 0..8 {
        assert_eq!(unsafe { ld(d, CONTROL_SIZE + i * 16 + 4) }, 8);
    }
    let values: vec::Vec<u64> = (0..20).collect();
    // Only the free slots are written; one head store covers the batch.
    assert_eq!(tx.push_slice(&values[..5]), 5);
    assert_eq!(unsafe { ld(d, HEAD) }, (u32::MAX - 2).wrapping_add(5));
    assert_eq!(tx.push_slice(&values[5..]), 3);
    assert_eq!(tx.push_slice(&values[8..]), 0);
    assert_eq!(tx.push(&99), Err(Error::Full));
    let mut out = [0u64; 3];
    assert_eq!(rx.pop_into(&mut out), Ok(3));
    assert_eq!(out, [0, 1, 2]);
    assert_eq!(unsafe { ld(c, TAIL) }, (u32::MAX - 2).wrapping_add(3));
    // The writer sees the released slots only through the stored tail.
    assert_eq!(tx.push_slice(&values[8..]), 3);
    let mut out = [0u64; 16];
    assert_eq!(rx.pop_into(&mut out), Ok(8));
    assert_eq!(out[..8], values[3..11]);
    assert_eq!(rx.pop_into(&mut out), Ok(0));
    assert_eq!(rx.pop(), Ok(None));
    // Single push/pop interleave with batches.
    tx.push(&50).unwrap();
    assert_eq!(tx.push_slice(&[51, 52]), 2);
    assert_eq!(rx.pop(), Ok(Some(50)));
    assert_eq!(rx.pop_into(&mut out[..1]), Ok(1));
    assert_eq!(out[0], 51);
    // A corrupt slot after a good one: the good one is returned, the error follows.
    assert_eq!(tx.push_slice(&[53, 54]), 2);
    let bad = rx.tail().wrapping_add(2);
    unsafe { st(d, CONTROL_SIZE + (bad & 7) as usize * 16, 7) };
    assert_eq!(rx.newest(), Ok(Some((bad, Err(Error::Corrupt)))));
    assert_eq!(rx.pop_into_report(&mut out), (2, Some(Error::Corrupt)));
    assert_eq!(out[..2], [52, 53]);
    // The slot becomes valid before another call: the first report retained the error.
    unsafe { st(d, CONTROL_SIZE + (bad & 7) as usize * 16, bad) };
    assert_eq!(rx.pop_into_report(&mut out), (1, None));
    assert_eq!(out[0], 54);
}

#[test]
fn reject_ring_index_wraps_u32() {
    let mut data = mem::<{ CONTROL_SIZE + 2 * 16 }>();
    let mut ctrl = mem::<CONTROL_SIZE>();
    let (d, c) = (data.0.as_mut_ptr(), ctrl.0.as_mut_ptr());
    unsafe {
        st(d, HEAD, u32::MAX - 1);
        st(c, TAIL, u32::MAX - 1);
    }
    let mut tx = unsafe { RingTx::<u32>::attach(d, 160, c, 128, 2, Overflow::Reject) }.unwrap();
    let mut rx = unsafe { RingRx::<u32>::attach(d, 160, c, 128, 2, Overflow::Reject) }.unwrap();
    for i in 0..10 {
        tx.push(&i).unwrap();
        assert_eq!(rx.pop(), Ok(Some(i)));
    }
}

#[test]
fn rpc_notify_uses_the_request_sequence_store_across_wrap() {
    let mut data = mem::<160>();
    let mut ctrl = mem::<128>();
    let (d, c) = (data.0.as_mut_ptr(), ctrl.0.as_mut_ptr());
    unsafe {
        st(d, HEAD, u32::MAX);
        st(c, TAIL, u32::MAX);
    }
    let mut tx = unsafe { RingTx::<u32>::attach(d, 160, c, 128, 2, Overflow::Reject) }.unwrap();
    let mut rx = unsafe { RingRx::<u32>::attach(d, 160, c, 128, 2, Overflow::Reject) }.unwrap();
    tx.push_request(&17, true).unwrap();
    assert_eq!(unsafe { ld(d, CONTROL_SIZE + 16) }, 0x7fff_ffff);
    assert_eq!(rx.pop(), Err(Error::Corrupt));
    assert_eq!(rx.pop_request(), Ok(Some((17, true))));
    tx.push_request(&23, false).unwrap();
    assert_eq!(rx.pop_request(), Ok(Some((23, false))));
    tx.push_request(&39, true).unwrap();
    assert_eq!(unsafe { ld(d, CONTROL_SIZE + 16) }, 0x8000_0001);
    assert_eq!(rx.pop_request(), Ok(Some((39, true))));
}

#[test]
fn invalid_mapping_and_odd_message_size() {
    let mut data = mem::<160>();
    let mut ctrl = mem::<128>();
    let d = data.0.as_mut_ptr();
    let c = ctrl.0.as_mut_ptr();
    assert!(unsafe { RingTx::<u32>::attach(d.add(4), 156, c, 128, 2, Overflow::Reject) }.is_err());
    assert!(unsafe { RingTx::<u32>::attach(d, 160, c, 128, 3, Overflow::Reject) }.is_err());
    assert!(unsafe { RingTx::<u32>::attach(d, 159, c, 128, 2, Overflow::Reject) }.is_err());
    let mut tx = unsafe { RingTx::<[u8; 3]>::attach(d, 160, c, 128, 2, Overflow::Reject) }.unwrap();
    let mut rx = unsafe { RingRx::<[u8; 3]>::attach(d, 160, c, 128, 2, Overflow::Reject) }.unwrap();
    tx.push(&[11, 22, 33]).unwrap();
    assert_eq!(rx.pop(), Ok(Some([11, 22, 33])));
    let header = unsafe { HeaderInit::begin(d, u32::MAX) };
    assert_eq!(header.generation(), 2);
}

/// The CPU generator: a batch across the end of the ring and the index wrap, a full ring that
/// never calls `fill`, and an overwrite ring given more messages than its depth.
#[test]
fn push_with_wraps_stops_when_full_and_overwrites() {
    let mut data = mem::<{ CONTROL_SIZE + 4 * 16 }>();
    let mut ctrl = mem::<CONTROL_SIZE>();
    let (d, c) = (data.0.as_mut_ptr(), ctrl.0.as_mut_ptr());
    let len = CONTROL_SIZE + 4 * 16;
    unsafe {
        st(d, HEAD, u32::MAX - 1);
        st(c, TAIL, u32::MAX - 1);
    }
    let mut tx = unsafe { RingTx::<u64>::attach(d, len, c, 128, 4, Overflow::Reject) }.unwrap();
    let mut rx = unsafe { RingRx::<u64>::attach(d, len, c, 128, 4, Overflow::Reject) }.unwrap();
    // Slots 2, 3, 0: one head store for the batch.
    assert_eq!(tx.push_with(3, |k, v| *v = 10 + k as u64), 3);
    assert_eq!(unsafe { ld(d, HEAD) }, 1);
    let mut calls = 0;
    assert_eq!(
        tx.push_with(5, |_, v| {
            calls += 1;
            *v = 13;
        }),
        1
    );
    assert_eq!(calls, 1);
    assert_eq!(tx.push_with(5, |_, _| panic!("full")), 0);
    let mut out = [0u64; 4];
    assert_eq!(rx.pop_into(&mut out), Ok(4));
    assert_eq!(out, [10, 11, 12, 13]);

    data.0.fill(0);
    ctrl.0.fill(0);
    let mut tx = unsafe { RingTx::<u64>::attach(d, len, c, 128, 4, Overflow::Overwrite) }.unwrap();
    let mut rx = unsafe { RingRx::<u64>::attach(d, len, c, 128, 4, Overflow::Overwrite) }.unwrap();
    assert_eq!(tx.push_with(6, |k, v| *v = k as u64), 6);
    assert_eq!(rx.pop(), Err(Error::Lost(2)));
    let got: vec::Vec<_> = core::iter::from_fn(|| rx.pop().unwrap()).collect();
    assert_eq!(got, [2, 3, 4, 5]);
}

/// A panicking generator publishes nothing on a reject ring: `head` and the remote `dropped`
/// count stay as they were until the next successful batch.
#[test]
fn push_with_panic_publishes_nothing() {
    let mut data = mem::<{ CONTROL_SIZE + 4 * 16 }>();
    let mut ctrl = mem::<CONTROL_SIZE>();
    let (d, c) = (data.0.as_mut_ptr(), ctrl.0.as_mut_ptr());
    let len = CONTROL_SIZE + 4 * 16;
    let mut tx = unsafe { RingTx::<u64>::attach(d, len, c, 128, 4, Overflow::Reject) }.unwrap();
    assert_eq!(tx.push_with(4, |_, v| *v = 1), 4);
    assert_eq!(tx.push(&2), Err(Error::Full));
    unsafe { st(c, TAIL, 4) };
    let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        tx.push_with(2, |k, v| {
            if k == 1 {
                panic!("generator");
            }
            *v = 3;
        });
    }));
    assert!(panic.is_err());
    assert_eq!(tx.head(), 4);
    assert_eq!(unsafe { (ld(d, HEAD), ld(d, DROPPED)) }, (4, 0));
    assert_eq!(tx.push_with(1, |_, v| *v = 4), 1);
    assert_eq!(unsafe { ld(d, DROPPED) }, 1);
}
