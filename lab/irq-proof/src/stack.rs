//! Stack high-water mark of this core's main stack, which its handlers share: paint the unused
//! words at entry, scan later (the method of lab/shm-firmware's stack_watermark, for either core).
const PAINT: u32 = 0xd3ad_beef;
const TOP: usize = 0x1000_4000;

pub struct Watermark {
    bottom: *mut u32,
    painted_end: *mut u32,
}
impl Watermark {
    /// Bytes at the bottom of the declared stack that nothing has written since [`paint`].
    /// The lowest word is the sentinel: 0 means the stack reached or passed its declared size.
    pub fn free_bytes(&self) -> u32 {
        let mut p = self.bottom;
        while p < self.painted_end && unsafe { p.read_volatile() } == PAINT {
            p = unsafe { p.add(1) };
        }
        (p as usize - self.bottom as usize) as u32
    }
}

/// The stack size the declaration gives `core` (linker symbols from rp1-build).
pub fn declared(core: u32) -> usize {
    unsafe extern "C" {
        static __stack_size0: u8;
        static __stack_size1: u8;
    }
    if core == 0 {
        core::ptr::addr_of!(__stack_size0) as usize
    } else {
        core::ptr::addr_of!(__stack_size1) as usize
    }
}

/// Paint once at the start of a core's main. The caller's frame and everything above MSP stay
/// untouched, and count as used.
/// # Safety
/// On `core`, before any of its NVIC lines is enabled.
#[inline(always)]
pub unsafe fn paint(core: u32) -> Watermark {
    let bottom = (TOP - declared(core)) as *mut u32;
    let sp: usize;
    unsafe { core::arch::asm!("mrs {}, MSP", out(reg) sp, options(nostack, preserves_flags)) };
    let mut p = bottom;
    while (p as usize) < sp {
        unsafe { p.write_volatile(PAINT) };
        p = unsafe { p.add(1) };
    }
    Watermark {
        bottom,
        painted_end: sp as *mut u32,
    }
}
