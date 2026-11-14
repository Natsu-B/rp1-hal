//! Raw register access for the sweep. The procedures need register states the drivers do not
//! offer (a source raised with its wrapper closed, a forced status bit, a block left in reset),
//! so they are written here against addresses and stay inside this lab image.
pub const SET: usize = 0x2000;
pub const CLR: usize = 0x3000;
const TIMER_RAWL: usize = 0x400a_c028;

pub fn rd(address: usize) -> u32 {
    // SAFETY: callers pass register addresses of blocks this image owns or only reads.
    unsafe { (address as *const u32).read_volatile() }
}
pub fn wr(address: usize, value: u32) {
    // SAFETY: as for `rd`; what may be written is the sweep contract in sweep.rs.
    unsafe { (address as *mut u32).write_volatile(value) }
}
/// Read-modify-write, for registers without SET/CLR aliases.
pub fn set(address: usize, bits: u32) {
    wr(address, rd(address) | bits);
}
pub fn clear(address: usize, bits: u32) {
    wr(address, rd(address) & !bits);
}
/// Microseconds, free-running.
pub fn now() -> u32 {
    rd(TIMER_RAWL)
}
/// Polls `done` for at most `us`; returns whether it came true.
pub fn poll(us: u32, mut done: impl FnMut() -> bool) -> bool {
    let start = now();
    loop {
        if done() {
            return true;
        }
        if now().wrapping_sub(start) >= us {
            return false;
        }
    }
}

const RESETS: usize = 0x4001_4000;
const RESETS_DONE: usize = RESETS + 0x18;
/// One block's bit in RESETS (`bank` 0 or 1).
pub struct Reset {
    pub bank: usize,
    pub bit: u32,
}
impl Reset {
    fn mask(&self) -> u32 {
        1 << self.bit
    }
    pub fn held(&self) -> bool {
        rd(RESETS + 4 * self.bank) & self.mask() != 0
    }
    /// Holds the block in reset; returns whether the bit reads back.
    pub fn hold(&self) -> bool {
        wr(RESETS + SET + 4 * self.bank, self.mask());
        self.held()
    }
    /// Through reset and out again, so that the block starts from its reset state whatever
    /// ran before.
    pub fn cycle(&self) -> bool {
        self.hold() && {
            wr(RESETS + CLR + 4 * self.bank, self.mask());
            poll(crate::sweep::STEP_US, || {
                rd(RESETS_DONE + 4 * self.bank) & self.mask() != 0
            })
        }
    }
}
