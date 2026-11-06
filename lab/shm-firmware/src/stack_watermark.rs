//! Proc1's descending private-DSRAM stack: paint only unused words, then scan cumulatively.
const PAINT: u32 = 0xd3ad_beef;

pub struct Watermark {
    bottom: *mut u32,
    painted_end: *mut u32,
}
impl Watermark {
    pub fn free_bytes(&self) -> usize {
        let mut p = self.bottom;
        while p < self.painted_end && unsafe { p.read_volatile() } == PAINT {
            p = unsafe { p.add(1) };
        }
        p as usize - self.bottom as usize
    }
}

/// Paint once at proc1 entry. The active frame and all stack above MSP remain untouched.
/// # Safety
/// Proc1 only, interrupts masked; the linker reserves `__stack_size1` below the reset MSP.
#[cfg(target_arch = "arm")]
#[inline(always)]
pub unsafe fn paint() -> Watermark {
    unsafe extern "C" {
        static __stack_size1: u8;
    }
    let top = 0x1000_4000 as *mut u32;
    let bottom = (top as usize - core::ptr::addr_of!(__stack_size1) as usize) as *mut u32;
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scan_reports_unused_prefix_and_includes_active_frame() {
        let mut words = [PAINT; 16];
        let bottom = words.as_mut_ptr();
        // Only the first 12 words were painted; the active frame must count as used
        // even if its bytes coincidentally match the paint value.
        let mark = Watermark {
            bottom,
            painted_end: unsafe { bottom.add(12) },
        };
        assert_eq!(mark.free_bytes(), 48);
        // A deeper call left its lowest touched word.
        unsafe { bottom.add(5).write_volatile(0) };
        assert_eq!(mark.free_bytes(), 20);
        unsafe { bottom.write_volatile(0) };
        assert_eq!(mark.free_bytes(), 0);
    }
}
