//! Core-to-core SPSC, with no exclusive accesses in shared SRAM.
use crate::{Core, Proc0, Proc1};
use core::{
    cell::UnsafeCell,
    marker::PhantomData,
    mem::MaybeUninit,
    sync::atomic::{AtomicU32, Ordering},
};
use rp1_proto::{Message, SharedSafe};

pub struct Spsc<T: Message, const N: usize, P: Core = Proc1, C: Core = Proc0> {
    head: AtomicU32,
    tail: AtomicU32,
    slots: [UnsafeCell<MaybeUninit<T>>; N],
    _cores: PhantomData<fn(P, C)>,
}
unsafe impl<T: Message, const N: usize, P: Core, C: Core> Sync for Spsc<T, N, P, C> {}
unsafe impl<T: Message, const N: usize, P: Core, C: Core> SharedSafe for Spsc<T, N, P, C> {}
impl<T: Message, const N: usize, P: Core, C: Core> Default for Spsc<T, N, P, C> {
    fn default() -> Self {
        Self::new()
    }
}
impl<T: Message, const N: usize, P: Core, C: Core> Spsc<T, N, P, C> {
    pub const fn new() -> Self {
        assert!(N.is_power_of_two() && N <= (1 << 30));
        Self {
            head: AtomicU32::new(0),
            tail: AtomicU32::new(0),
            slots: [const { UnsafeCell::new(MaybeUninit::uninit()) }; N],
            _cores: PhantomData,
        }
    }
    pub fn producer<'a>(&'a self, _core: &'a mut P) -> Producer<'a, T, N, P, C> {
        Producer {
            queue: self,
            _core: PhantomData,
        }
    }
    pub fn consumer<'a>(&'a self, _core: &'a mut C) -> Consumer<'a, T, N, P, C> {
        Consumer {
            queue: self,
            _core: PhantomData,
        }
    }
}
pub struct Producer<'a, T: Message, const N: usize, P: Core, C: Core> {
    queue: &'a Spsc<T, N, P, C>,
    _core: PhantomData<&'a mut P>,
}
pub struct Consumer<'a, T: Message, const N: usize, P: Core, C: Core> {
    queue: &'a Spsc<T, N, P, C>,
    _core: PhantomData<&'a mut C>,
}
impl<T: Message, const N: usize, P: Core, C: Core> Producer<'_, T, N, P, C> {
    pub fn push(&mut self, value: T) -> Result<(), T> {
        let q = self.queue;
        let head = q.head.load(Ordering::Relaxed);
        if head.wrapping_sub(q.tail.load(Ordering::Acquire)) >= N as u32 {
            return Err(value);
        }
        unsafe {
            (*q.slots[head as usize & (N - 1)].get()).write(value);
        }
        q.head.store(head.wrapping_add(1), Ordering::Release);
        #[cfg(target_arch = "arm")]
        unsafe {
            core::arch::asm!("dsb sy", "sev", options(nostack, preserves_flags));
        }
        Ok(())
    }
}
impl<T: Message, const N: usize, P: Core, C: Core> Consumer<'_, T, N, P, C> {
    pub fn pop(&mut self) -> Option<T> {
        let q = self.queue;
        let tail = q.tail.load(Ordering::Relaxed);
        if tail == q.head.load(Ordering::Acquire) {
            return None;
        }
        let value = unsafe { (*q.slots[tail as usize & (N - 1)].get()).assume_init_read() };
        q.tail.store(tail.wrapping_add(1), Ordering::Release);
        Some(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn full_empty_and_wrap() {
        let q = Spsc::<u32, 2>::new();
        q.head.store(u32::MAX, Ordering::Relaxed);
        q.tail.store(u32::MAX, Ordering::Relaxed);
        let mut p = unsafe { Proc1::new() };
        let mut c = unsafe { Proc0::new() };
        let mut tx = q.producer(&mut p);
        let mut rx = q.consumer(&mut c);
        assert_eq!(rx.pop(), None);
        assert_eq!(tx.push(1), Ok(()));
        assert_eq!(tx.push(2), Ok(()));
        assert_eq!(tx.push(3), Err(3));
        assert_eq!(rx.pop(), Some(1));
        assert_eq!(rx.pop(), Some(2));
        assert_eq!(rx.pop(), None);
    }
}
