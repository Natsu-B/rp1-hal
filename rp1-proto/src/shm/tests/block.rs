//! The DDR header and SeqLock blocks.
use super::mem;
use crate::Error;
use crate::shm::access::ld;
use crate::shm::*;

#[test]
fn header_and_block() {
    let mut h = mem::<64>();
    let hp = h.0.as_mut_ptr();
    let mut hdr = unsafe { HeaderInit::begin(hp, 0) };
    hdr.write(&[5; 32], 3, 0);
    assert_eq!(
        unsafe { Header::open(hp, &[5; 32]) }.err(),
        Some(Error::NotReady)
    );
    hdr.publish();
    let l = unsafe { Header::open(hp, &[5; 32]) }.unwrap();
    assert_eq!((l.generation(), l.regions()), (2, 3));
    assert_eq!(
        unsafe { Header::open(hp, &[6; 32]) }.err(),
        Some(Error::Mismatch)
    );
    let again = unsafe { HeaderInit::begin(hp, 2) };
    assert_eq!(l.check(), Err(Error::Reset));
    again.publish();
    assert_eq!(
        unsafe { Header::open(hp, &[5; 32]) }.unwrap().generation(),
        4
    );

    let mut b = mem::<{ 8 + 24 }>();
    let bp = b.0.as_mut_ptr();
    let mut w = unsafe { Block::<[u32; 5]>::attach(bp, 32) }.unwrap();
    let r = unsafe { Block::<[u32; 5]>::attach(bp, 32) }.unwrap();
    unsafe { w.write(&[1, 2, 3, 4, 5]) };
    assert_eq!(r.read(), Ok([1, 2, 3, 4, 5]));
    {
        let mut s = unsafe { w.begin_write() };
        s.fill(0, 5, |i| i as u32 * 10).unwrap();
        assert!(s.write_words(4, &[1, 2]).is_err());
        assert_eq!(r.read(), Err(Error::NotReady));
    }
    let mut out = [0; 3];
    r.read_with(|x| x.read_words(2, &mut out)).unwrap().unwrap();
    assert_eq!(out, [20, 30, 40]);
    r.read_with(|x| x.read_words(1, &mut out)).unwrap().unwrap();
    assert_eq!(out, [10, 20, 30]);
    assert_eq!(unsafe { ld(bp, 0) }, 4);
    let seen = r.sequence();
    assert_eq!(seen & 1, 0);
    let mut other = unsafe { Block::<[u32; 5]>::attach(bp, 32) }.unwrap();
    unsafe {
        other.write(&[5; 5]);
        w.write(&[6; 5]);
    }
    assert_eq!(unsafe { ld(bp, 0) }, 8);
    assert_ne!(r.sequence(), seen);
}
