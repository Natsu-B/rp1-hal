//! Uncached device-memory access. No Rust references to mapped payloads escape.
use crate::{Error, Message, barrier};
use core::{
    marker::PhantomData,
    mem::{MaybeUninit, size_of},
    ptr,
};

pub const HEADER_SIZE: usize = 64;
pub const CONTROL_SIZE: usize = 128;
/// First link cache line. The loader writes DDR physical address (low/high) before boot.
#[repr(C, align(64))]
pub struct LinkHeader {
    pub ddr_address_low: core::sync::atomic::AtomicU32,
    pub ddr_address_high: core::sync::atomic::AtomicU32,
    pub status: core::sync::atomic::AtomicU32,
    pub generation: core::sync::atomic::AtomicU32,
    pub request: core::sync::atomic::AtomicU32,
    pub reply: core::sync::atomic::AtomicU32,
    reserved: [u32; 10],
}
unsafe impl crate::SharedSafe for LinkHeader {}
impl Default for LinkHeader {
    fn default() -> Self {
        Self::new()
    }
}
impl LinkHeader {
    pub const fn new() -> Self {
        use core::sync::atomic::AtomicU32;
        Self {
            ddr_address_low: AtomicU32::new(0),
            ddr_address_high: AtomicU32::new(0),
            status: AtomicU32::new(0),
            generation: AtomicU32::new(0),
            request: AtomicU32::new(0),
            reply: AtomicU32::new(0),
            reserved: [0; 10],
        }
    }
}
const _: () = assert!(size_of::<LinkHeader>() == 64);
pub const fn slot_size<T>() -> usize {
    8 + ((size_of::<T>() + 7) & !7)
}
pub const fn ring_body_size<T>(depth: usize) -> usize {
    CONTROL_SIZE + depth * slot_size::<T>()
}

#[inline]
unsafe fn load(base: *const u8, offset: usize) -> u32 {
    unsafe { ptr::read_volatile(base.add(offset).cast()) }
}
#[inline]
unsafe fn store(base: *mut u8, offset: usize, value: u32) {
    unsafe {
        ptr::write_volatile(base.add(offset).cast(), value);
    }
}

unsafe fn write_value<T: Message>(dst: *mut u8, value: &T) {
    let src = ptr::from_ref(value).cast::<u8>();
    for offset in (0..size_of::<T>()).step_by(4) {
        let mut word = [0u8; 4];
        for (i, b) in word.iter_mut().enumerate() {
            if offset + i < size_of::<T>() {
                *b = unsafe { src.add(offset + i).read() };
            }
        }
        unsafe {
            store(dst, offset, u32::from_le_bytes(word));
        }
    }
}
unsafe fn read_value<T: Message>(src: *const u8) -> T {
    let mut value = MaybeUninit::<T>::uninit();
    let dst = value.as_mut_ptr().cast::<u8>();
    for offset in (0..size_of::<T>()).step_by(4) {
        let word = unsafe { load(src, offset) }.to_le_bytes();
        for (i, b) in word.iter().enumerate() {
            if offset + i < size_of::<T>() {
                unsafe {
                    dst.add(offset + i).write(*b);
                }
            }
        }
    }
    unsafe { value.assume_init() }
}

#[derive(Clone, Copy)]
pub struct Generation {
    header: *mut u8,
    value: u32,
}
impl Generation {
    /// Initialize the DDR header while both peers are quiescent. Returns the new even generation.
    ///
    /// # Safety
    /// `header` is an aligned, uncached, writable 64-byte device mapping. No peer may
    /// access the data until initialization of all rings/blocks is complete.
    pub unsafe fn initialize(
        header: *mut u8,
        note_hash: [u8; 32],
        regions: u32,
    ) -> Result<Self, Error> {
        if header.is_null() || header as usize & 3 != 0 {
            return Err(Error::Invalid);
        }
        let old = unsafe { load(header, 12) };
        let value = (old & !1).wrapping_add(2);
        unsafe {
            store(header, 12, value.wrapping_sub(1));
        }
        barrier();
        for (i, word) in b"RP1SHM\0\0".as_chunks::<4>().0.iter().enumerate() {
            unsafe {
                store(header, i * 4, u32::from_le_bytes(*word));
            }
        }
        unsafe {
            store(header, 8, (64 << 16) | 1);
        }
        for (i, word) in note_hash.as_chunks::<4>().0.iter().enumerate() {
            unsafe {
                store(header, 16 + i * 4, u32::from_le_bytes(*word));
            }
        }
        unsafe {
            store(header, 48, regions);
            store(header, 52, 0);
            store(header, 56, 0);
            store(header, 60, 0);
        }
        // Publication is separate so ring initialization cannot race Linux open.
        Ok(Self { header, value })
    }
    /// # Safety
    /// All ring/block state for this generation has been initialized, and this is the only initializer.
    pub unsafe fn publish(&self) {
        barrier();
        unsafe {
            store(self.header, 12, self.value);
        }
    }

    /// # Safety
    /// `header` remains a readable aligned 64-byte uncached device mapping for this handle's lifetime.
    pub unsafe fn open(header: *mut u8, note_hash: &[u8; 32]) -> Result<Self, Error> {
        if header.is_null() || header as usize & 3 != 0 {
            return Err(Error::Invalid);
        }
        let value = unsafe { load(header, 12) };
        if value & 1 != 0 {
            return Err(Error::NotReady);
        }
        if unsafe { load(header, 0) } != u32::from_le_bytes(*b"RP1S")
            || unsafe { load(header, 4) } != u32::from_le_bytes(*b"HM\0\0")
            || unsafe { load(header, 8) } != (64 << 16) | 1
        {
            return Err(Error::NotReady);
        }
        for (i, word) in note_hash.as_chunks::<4>().0.iter().enumerate() {
            if unsafe { load(header, 16 + i * 4) }.to_le_bytes() != *word {
                return Err(Error::Mismatch);
            }
        }
        barrier();
        let this = Self { header, value };
        this.check()?;
        Ok(this)
    }
    pub fn check(&self) -> Result<(), Error> {
        barrier();
        if unsafe { load(self.header, 12) } != self.value {
            Err(Error::Reset)
        } else {
            Ok(())
        }
    }
    pub fn number(&self) -> u32 {
        self.value
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Overflow {
    Reject,
    Overwrite,
}

struct Ring<T> {
    body: *mut u8,
    credits: *mut u8,
    depth: u32,
    generation: Generation,
    mode: Overflow,
    _type: PhantomData<T>,
}
pub struct RingTx<T> {
    ring: Ring<T>,
    head: u32,
}
pub struct RingRx<T> {
    ring: Ring<T>,
    tail: u32,
}

/// Attach exactly one writer and one reader, preserving shared tail on process restart.
///
/// # Safety
/// Mappings must remain valid, uncached and aligned to 4 bytes. `body` and `credits`
/// must be disjoint from each other and the header. Only these two endpoints may
/// access the ring; initialize the controls to zero before publishing the generation.
pub unsafe fn ring<T: Message>(
    body: *mut u8,
    body_len: usize,
    credits: *mut u8,
    credits_len: usize,
    depth: u32,
    mode: Overflow,
    generation: Generation,
) -> Result<(RingTx<T>, RingRx<T>), Error> {
    if body.is_null()
        || credits.is_null()
        || (body as usize | credits as usize) & 3 != 0
        || !depth.is_power_of_two()
        || depth > (1 << 30)
        || size_of::<T>() == 0
        || size_of::<T>() > u16::MAX as usize
        || credits_len < CONTROL_SIZE
        || (depth as usize)
            .checked_mul(slot_size::<T>())
            .and_then(|n| n.checked_add(CONTROL_SIZE))
            .is_none_or(|n| n > body_len)
    {
        return Err(Error::Invalid);
    }
    generation.check()?;
    let make = || Ring {
        body,
        credits,
        depth,
        generation,
        mode,
        _type: PhantomData,
    };
    let head = unsafe { load(body, 0) };
    let tail = unsafe { load(credits, 0) };
    Ok((RingTx { ring: make(), head }, RingRx { ring: make(), tail }))
}

impl<T: Message> RingTx<T> {
    /// Returns whether the reader requested a doorbell. Flush the same outbound window before notifying.
    pub fn push(&mut self, value: T) -> Result<bool, Error> {
        let r = &self.ring;
        r.generation.check()?;
        let tail = unsafe { load(r.credits, 0) };
        if r.mode == Overflow::Reject && self.head.wrapping_sub(tail) >= r.depth {
            let dropped = unsafe { load(r.body, 64) };
            unsafe {
                store(r.body, 64, dropped.wrapping_add(1));
            }
            r.generation.check()?;
            return Err(Error::Full);
        }
        let slot = unsafe {
            r.body
                .add(CONTROL_SIZE + (self.head & (r.depth - 1)) as usize * slot_size::<T>())
        };
        // flags bit 0 marks a write in progress, avoiding a reserved sequence number at wrap.
        unsafe {
            store(slot, 4, 1 << 16);
        }
        barrier();
        unsafe {
            write_value(slot.add(8), &value);
            store(slot, 0, self.head);
        }
        barrier();
        unsafe {
            store(slot, 4, size_of::<T>() as u32);
        }
        barrier();
        self.head = self.head.wrapping_add(1);
        unsafe {
            store(r.body, 0, self.head);
        }
        barrier();
        let notify = unsafe { load(r.credits, 64) } != 0;
        r.generation.check()?;
        Ok(notify)
    }
}
impl<T: Message> RingRx<T> {
    pub fn pop(&mut self) -> Result<Option<T>, Error> {
        let r = &self.ring;
        r.generation.check()?;
        let head = unsafe { load(r.body, 0) };
        let available = head.wrapping_sub(self.tail);
        if available == 0 {
            r.generation.check()?;
            return Ok(None);
        }
        if available > r.depth {
            if r.mode == Overflow::Reject {
                return Err(Error::Corrupt);
            }
            let lost = available - r.depth;
            self.tail = head.wrapping_sub(r.depth);
            unsafe {
                store(r.credits, 0, self.tail);
            }
            r.generation.check()?;
            return Err(Error::Lost(lost));
        }
        let slot = unsafe {
            r.body
                .add(CONTROL_SIZE + (self.tail & (r.depth - 1)) as usize * slot_size::<T>())
        };
        let seq = unsafe { load(slot, 0) };
        let flags = unsafe { load(slot, 4) };
        if flags >> 16 != 0 {
            return Err(Error::NotReady);
        }
        if seq != self.tail {
            return Err(if r.mode == Overflow::Overwrite {
                Error::NotReady
            } else {
                Error::Corrupt
            });
        }
        if flags as usize != size_of::<T>() {
            return Err(Error::Invalid);
        }
        barrier();
        let value = unsafe { read_value(slot.add(8)) };
        barrier();
        if unsafe { load(slot, 0) } != seq || unsafe { load(slot, 4) } != flags {
            return Err(Error::NotReady);
        }
        self.tail = self.tail.wrapping_add(1);
        unsafe {
            store(r.credits, 0, self.tail);
        }
        r.generation.check()?;
        Ok(Some(value))
    }
    /// Arm notification, then recheck head to avoid sleeping past a posted write.
    pub fn prepare_wait(&mut self) -> Result<bool, Error> {
        let r = &self.ring;
        r.generation.check()?;
        unsafe {
            store(r.credits, 64, 1);
        }
        barrier();
        let empty = unsafe { load(r.body, 0) } == self.tail;
        if !empty {
            unsafe {
                store(r.credits, 64, 0);
            }
        }
        r.generation.check()?;
        Ok(empty)
    }
    pub fn finish_wait(&mut self) -> Result<(), Error> {
        unsafe {
            store(self.ring.credits, 64, 0);
        }
        self.ring.generation.check()
    }
}

pub struct Block<T> {
    base: *mut u8,
    generation: Generation,
    _type: PhantomData<T>,
}
impl<T: Message> Block<T> {
    /// # Safety
    /// `base` is an aligned uncached mapping of at least `8 + round_up(size_of::<T>(), 8)`
    /// bytes. Exactly one writer may use `write`; readers use separate attached handles.
    pub unsafe fn attach(base: *mut u8, len: usize, generation: Generation) -> Result<Self, Error> {
        if base.is_null() || base as usize & 3 != 0 || len < slot_size::<T>() {
            return Err(Error::Invalid);
        }
        Ok(Self {
            base,
            generation,
            _type: PhantomData,
        })
    }
    /// # Safety
    /// This handle is the declared sole writer. For writer=both, synchronize externally.
    pub unsafe fn write(&mut self, value: T) -> Result<(), Error> {
        self.generation.check()?;
        let seq = unsafe { load(self.base, 0) } & !1;
        unsafe {
            store(self.base, 0, seq.wrapping_add(1));
        }
        barrier();
        unsafe {
            write_value(self.base.add(8), &value);
        }
        barrier();
        unsafe {
            store(self.base, 0, seq.wrapping_add(2));
        }
        self.generation.check()
    }
    pub fn read(&self) -> Result<T, Error> {
        self.generation.check()?;
        let seq = unsafe { load(self.base, 0) };
        if seq & 1 != 0 {
            return Err(Error::NotReady);
        }
        barrier();
        let value = unsafe { read_value(self.base.add(8)) };
        barrier();
        if unsafe { load(self.base, 0) } != seq {
            return Err(Error::NotReady);
        }
        self.generation.check()?;
        Ok(value)
    }
    /// # Safety
    /// Caller implements synchronization for writer=both and uses only volatile access.
    pub unsafe fn raw(&self) -> *mut u8 {
        self.base
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reject_overwrite_wrap_wait_and_restart() {
        for mode in [Overflow::Reject, Overflow::Overwrite] {
            let mut header = [0u32; 16];
            let mut body = [0u32; 48];
            let mut credits = [0u32; 32];
            let g =
                unsafe { Generation::initialize(header.as_mut_ptr().cast(), [7; 32], 1).unwrap() };
            unsafe {
                g.publish();
            }
            let (mut tx, mut rx) = unsafe {
                ring::<u32>(
                    body.as_mut_ptr().cast(),
                    192,
                    credits.as_mut_ptr().cast(),
                    128,
                    2,
                    mode,
                    g,
                )
                .unwrap()
            };
            assert!(rx.prepare_wait().unwrap());
            assert!(tx.push(10).unwrap());
            rx.finish_wait().unwrap();
            tx.push(11).unwrap();
            if mode == Overflow::Reject {
                assert_eq!(tx.push(12), Err(Error::Full));
                assert_eq!(rx.pop(), Ok(Some(10)));
            } else {
                tx.push(12).unwrap();
                assert_eq!(rx.pop(), Err(Error::Lost(1)));
                assert_eq!(rx.pop(), Ok(Some(11)));
            }
            while rx.pop().unwrap().is_some() {}
            tx.head = u32::MAX;
            rx.tail = u32::MAX;
            unsafe {
                store(tx.ring.body, 0, u32::MAX);
                store(tx.ring.credits, 0, u32::MAX);
            }
            tx.push(99).unwrap();
            assert_eq!(rx.pop(), Ok(Some(99)));
            let (_, mut reopened) = unsafe {
                ring::<u32>(
                    body.as_mut_ptr().cast(),
                    192,
                    credits.as_mut_ptr().cast(),
                    128,
                    2,
                    mode,
                    g,
                )
                .unwrap()
            };
            assert_eq!(reopened.pop(), Ok(None));
            unsafe {
                store(g.header, 12, g.value + 1);
            }
            assert_eq!(rx.pop(), Err(Error::Reset));
        }
    }
    #[test]
    fn generation_and_block() {
        let mut header = [0u32; 16];
        let mut data = [0u32; 8];
        let g = unsafe { Generation::initialize(header.as_mut_ptr().cast(), [3; 32], 1).unwrap() };
        assert!(matches!(
            unsafe { Generation::open(g.header, &[3; 32]) },
            Err(Error::NotReady)
        ));
        unsafe {
            g.publish();
        }
        assert!(unsafe { Generation::open(g.header, &[3; 32]) }.is_ok());
        assert!(matches!(
            unsafe { Generation::open(g.header, &[4; 32]) },
            Err(Error::Mismatch)
        ));
        let mut b = unsafe { Block::<[u32; 2]>::attach(data.as_mut_ptr().cast(), 32, g).unwrap() };
        unsafe {
            b.write([11, 12]).unwrap();
        }
        assert_eq!(b.read(), Ok([11, 12]));
        unsafe {
            store(b.base, 0, 3);
        }
        assert_eq!(b.read(), Err(Error::NotReady));
    }
}
