//! SeqLock blocks: one value shared by a declared writer, read word by word.
use super::access::{aligned, copy_to_mapping, ld, read_value, st};
use super::block_size;
use crate::{Error, Message, barrier};
use core::{marker::PhantomData, mem::size_of, ptr};

/// A SeqLock-protected block (protocol.md §5): `seq` u32, `flags` u32, then `T`.
///
/// Large `T` (tables, bulk buffers) are accessed word by word through [`BlockWriter`] and
/// [`BlockReader`] so that no copy of `T` is needed in RP1 memory.
pub struct Block<T> {
    base: *mut u8,
    seq: u32,
    _t: PhantomData<T>,
}
unsafe impl<T> Send for Block<T> {}
impl<T: Message> Block<T> {
    /// # Safety
    /// `base` is an 8-byte aligned mapping of `len >= block_size(size_of::<T>())` bytes that
    /// stays valid and is accessed only through volatile operations.
    pub unsafe fn attach(base: *mut u8, len: usize) -> Result<Self, Error> {
        if !aligned(base) || len < block_size(size_of::<T>()) {
            return Err(Error::Invalid);
        }
        Ok(Self {
            base,
            seq: unsafe { ld(base, 0) } & !1,
            _t: PhantomData,
        })
    }
    /// Words of `T` (rounded up).
    pub const WORDS: usize = size_of::<T>().div_ceil(4);
    /// Writer: replace the whole value.
    ///
    /// # Safety
    /// This handle is the declared sole writer (`writer = "rp1"` / `"linux"`).
    pub unsafe fn write(&mut self, value: &T) {
        let mut w = unsafe { self.begin_write() };
        unsafe { copy_to_mapping(w.data(), ptr::from_ref(value).cast(), size_of::<T>()) };
        w.commit();
    }
    /// Writer: open a write section (odd `seq`). Readers retry until [`BlockWriter::commit`].
    ///
    /// # Safety
    /// As [`Block::write`].
    pub unsafe fn begin_write(&mut self) -> BlockWriter<'_, T> {
        // writer=both transfers ownership between peers; the previous writer advanced seq.
        self.seq = unsafe { ld(self.base, 0) }.wrapping_add(1) | 1;
        unsafe { st(self.base, 0, self.seq) };
        barrier();
        BlockWriter { block: self }
    }
    /// Current SeqLock sequence (one aligned load). Cheap change detection for pollers: an
    /// unchanged even value means no write section ran since it was last seen. Over the RP1
    /// outbound window this is one PCIe round trip instead of a whole-value read.
    pub fn sequence(&self) -> u32 {
        unsafe { ld(self.base, 0) }
    }
    /// Reader: copy the whole value; `Err(NotReady)` if a write was in progress or overlapped.
    pub fn read(&self) -> Result<T, Error> {
        self.read_with(|r| unsafe { read_value::<T>(r.block.base.add(8)) })
    }
    /// Reader: run `f` inside one SeqLock read section.
    pub fn read_with<R>(&self, f: impl FnOnce(&BlockReader<'_, T>) -> R) -> Result<R, Error> {
        let seq = unsafe { ld(self.base, 0) };
        if seq & 1 != 0 {
            return Err(Error::NotReady);
        }
        barrier();
        let r = f(&BlockReader { block: self });
        barrier();
        if unsafe { ld(self.base, 0) } != seq {
            return Err(Error::NotReady);
        }
        Ok(r)
    }
    /// Raw word access without SeqLock (`writer = "both"`: caller synchronizes).
    ///
    /// # Safety
    /// The caller provides the synchronization the SeqLock would.
    pub unsafe fn raw(&self) -> BlockReader<'_, T> {
        BlockReader { block: self }
    }
}
pub struct BlockWriter<'a, T> {
    block: &'a mut Block<T>,
}
impl<T: Message> BlockWriter<'_, T> {
    fn data(&mut self) -> *mut u8 {
        unsafe { self.block.base.add(8) }
    }
    /// Store `words` at word index `at` of `T`.
    pub fn write_words(&mut self, at: usize, words: &[u32]) -> Result<(), Error> {
        if at
            .checked_add(words.len())
            .is_none_or(|end| end > Block::<T>::WORDS)
        {
            return Err(Error::Invalid);
        }
        let d = self.data();
        for (i, w) in words.iter().enumerate() {
            unsafe { st(d, (at + i) * 4, *w) };
        }
        Ok(())
    }
    /// Fill words `at..at+count` with `f(index)` without a staging buffer.
    pub fn fill(
        &mut self,
        at: usize,
        count: usize,
        mut f: impl FnMut(usize) -> u32,
    ) -> Result<(), Error> {
        if at
            .checked_add(count)
            .is_none_or(|end| end > Block::<T>::WORDS)
        {
            return Err(Error::Invalid);
        }
        let d = self.data();
        for i in at..at + count {
            unsafe { st(d, i * 4, f(i)) };
        }
        Ok(())
    }
    /// Base of `T` in the mapping, for specialised (e.g. burst) writers.
    ///
    /// # Safety
    /// Stay within `Block::<T>::WORDS` words and use only aligned volatile stores.
    pub unsafe fn as_ptr(&mut self) -> *mut u32 {
        self.data().cast()
    }
    /// End the write section (even `seq`).
    pub fn commit(self) {}
}
impl<T> Drop for BlockWriter<'_, T> {
    fn drop(&mut self) {
        barrier();
        self.block.seq = self.block.seq.wrapping_add(1);
        unsafe { st(self.block.base, 0, self.block.seq) };
        barrier();
    }
}
pub struct BlockReader<'a, T> {
    block: &'a Block<T>,
}
impl<T: Message> BlockReader<'_, T> {
    pub fn read_word(&self, at: usize) -> Result<u32, Error> {
        if at >= Block::<T>::WORDS {
            return Err(Error::Invalid);
        }
        Ok(unsafe { ld(self.block.base, 8 + at * 4) })
    }
    /// Load words `at..at+out.len()` of `T`.
    pub fn read_words(&self, at: usize, out: &mut [u32]) -> Result<(), Error> {
        if at
            .checked_add(out.len())
            .is_none_or(|end| end > Block::<T>::WORDS)
        {
            return Err(Error::Invalid);
        }
        for (i, word) in out.iter_mut().enumerate() {
            *word = unsafe { ld(self.block.base, 8 + (at + i) * 4) };
        }
        Ok(())
    }
    /// Base of `T` in the mapping, for specialised (e.g. burst) readers.
    ///
    /// # Safety
    /// Stay within `Block::<T>::WORDS` words and use only aligned volatile loads.
    pub unsafe fn as_ptr(&self) -> *const u32 {
        unsafe { self.block.base.add(8).cast() }
    }
}
