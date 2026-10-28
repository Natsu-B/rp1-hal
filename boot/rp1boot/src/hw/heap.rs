//! Bump allocator over the linker's `.heap` (single core, no atomics; the last block can be
//! freed or grown in place, which keeps `Vec` growth and short-lived buffers cheap).
use core::alloc::{GlobalAlloc, Layout};
use core::cell::Cell;

unsafe extern "C" {
    static __heap_start: u8;
    static __heap_end: u8;
}

struct Bump {
    next: Cell<usize>,
    last: Cell<usize>,
}
// SAFETY: the loader runs on one core with interrupts masked.
unsafe impl Sync for Bump {}

#[global_allocator]
static HEAP: Bump = Bump {
    next: Cell::new(0),
    last: Cell::new(0),
};

impl Bump {
    fn bounds(&self) -> (usize, usize) {
        let (s, e) = (
            (&raw const __heap_start) as usize,
            (&raw const __heap_end) as usize,
        );
        if self.next.get() == 0 {
            self.next.set(s);
        }
        (s, e)
    }
}

unsafe impl GlobalAlloc for Bump {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let (_, end) = self.bounds();
        let at = self.next.get().next_multiple_of(layout.align());
        match at.checked_add(layout.size()) {
            Some(n) if n <= end => {
                self.next.set(n);
                self.last.set(at);
                at as *mut u8
            }
            _ => core::ptr::null_mut(),
        }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        if ptr as usize == self.last.get() {
            self.next.set(self.last.get());
            self.last.set(0);
        }
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let (_, end) = self.bounds();
        if ptr as usize == self.last.get() && ptr as usize + new_size <= end {
            self.next.set(ptr as usize + new_size);
            return ptr;
        }
        // SAFETY: GlobalAlloc contract; copy the old contents into a fresh block.
        unsafe {
            let new = self.alloc(Layout::from_size_align_unchecked(new_size, layout.align()));
            if !new.is_null() {
                core::ptr::copy_nonoverlapping(ptr, new, layout.size().min(new_size));
            }
            new
        }
    }
}
