//! Volatile word access to a mapping and copies between a mapping and ordinary memory.
use crate::Message;
use core::{
    mem::{MaybeUninit, size_of},
    ptr,
};

#[inline(always)]
pub(super) unsafe fn ld(base: *const u8, offset: usize) -> u32 {
    #[cfg(test)]
    super::tests::model::before_load(unsafe { base.add(offset) });
    unsafe { ptr::read_volatile(base.add(offset).cast()) }
}
#[inline(always)]
pub(super) unsafe fn st(base: *mut u8, offset: usize, value: u32) {
    unsafe { ptr::write_volatile(base.add(offset).cast(), value) }
}

/// Copy `bytes` bytes (padded up to 4/8) from a mapping into ordinary memory.
///
/// # Safety
/// `src` is 8-byte aligned (4 on 32-bit targets) and readable for `bytes` rounded up to 8.
/// `dst` is writable ordinary memory for `bytes`.
#[inline(always)]
pub unsafe fn copy_from_mapping(dst: *mut u8, src: *const u8, bytes: usize) {
    #[cfg(target_pointer_width = "64")]
    {
        let mut i = 0;
        while i < bytes {
            let w = unsafe { ptr::read_volatile(src.add(i).cast::<u64>()) }.to_le_bytes();
            let n = if bytes - i < 8 { bytes - i } else { 8 };
            unsafe { ptr::copy_nonoverlapping(w.as_ptr(), dst.add(i), n) };
            i += 8;
        }
    }
    #[cfg(not(target_pointer_width = "64"))]
    {
        let mut i = 0;
        while i < bytes {
            let w = unsafe { ld(src, i) }.to_le_bytes();
            let n = if bytes - i < 4 { bytes - i } else { 4 };
            unsafe { ptr::copy_nonoverlapping(w.as_ptr(), dst.add(i), n) };
            i += 4;
        }
    }
}
/// Copy `bytes` bytes into a mapping; the tail of the last word is zero-filled.
///
/// # Safety
/// As [`copy_from_mapping`] with the roles swapped; `dst` is writable up to `bytes` rounded to 8.
#[inline(always)]
pub unsafe fn copy_to_mapping(dst: *mut u8, src: *const u8, bytes: usize) {
    #[cfg(target_pointer_width = "64")]
    {
        let mut i = 0;
        while i < bytes {
            let mut w = [0u8; 8];
            let n = if bytes - i < 8 { bytes - i } else { 8 };
            unsafe { ptr::copy_nonoverlapping(src.add(i), w.as_mut_ptr(), n) };
            unsafe { ptr::write_volatile(dst.add(i).cast::<u64>(), u64::from_le_bytes(w)) };
            i += 8;
        }
    }
    #[cfg(not(target_pointer_width = "64"))]
    {
        let mut i = 0;
        while i < bytes {
            let mut w = [0u8; 4];
            let n = if bytes - i < 4 { bytes - i } else { 4 };
            unsafe { ptr::copy_nonoverlapping(src.add(i), w.as_mut_ptr(), n) };
            unsafe { st(dst, i, u32::from_le_bytes(w)) };
            i += 4;
        }
    }
}
pub(super) unsafe fn read_value<T: Message>(src: *const u8) -> T {
    let mut v = MaybeUninit::<T>::uninit();
    unsafe {
        copy_from_mapping(v.as_mut_ptr().cast(), src, size_of::<T>());
        // Message: every bit pattern is valid and there is no padding.
        v.assume_init()
    }
}

pub(super) fn aligned(p: *const u8) -> bool {
    !p.is_null() && p as usize & 7 == 0
}
