//! Frame for the sweep of lines whose IRQ number is only estimated (I2C0/2/3, SPI1/3/5, DMA,
//! PWM1, ADC): raise a block's interrupt and read this core's NVIC pending bit. The line is
//! never enabled, so no handler runs.
//!
//! `TABLE` is empty on purpose. Its entries are register procedures that run on hardware; they
//! go in only after review, together with a declaration that gives each swept block to RP1
//! (Linux's node disabled). DMA must not be swept in an image that also runs the shm rings.
use crate::proof::{mark, rd, settle, wr};

#[allow(dead_code)]
pub enum Op {
    Write(usize, u32),
    /// Read-modify-write: set / clear bits where the register has no SET/CLR alias.
    SetBits(usize, u32),
    ClearBits(usize, u32),
    /// A read with a side effect (read-to-clear).
    Read(usize),
    /// Wait until the bits are set, at most 10 ms (a reset or a clock coming up).
    WaitSet(usize, u32),
}

pub struct Swept {
    pub irq: u8,
    /// Once: bring the block out of reset far enough for `raise`.
    pub prepare: &'static [Op],
    pub raise: &'static [Op],
    /// Stops the source; also run before `raise`, so that the line starts not pending.
    pub quiesce: &'static [Op],
}

const TABLE: &[Swept] = &[];

const NVIC_ISPR: usize = 0xe000_e200;
const NVIC_ICPR: usize = 0xe000_e280;

fn apply(ops: &[Op], idle: &mut dyn FnMut()) -> bool {
    let mut ok = true;
    for op in ops {
        match *op {
            Op::Write(address, value) => wr(address, value),
            Op::SetBits(address, bits) => wr(address, rd(address) | bits),
            Op::ClearBits(address, bits) => wr(address, rd(address) & !bits),
            Op::Read(address) => {
                rd(address);
            }
            Op::WaitSet(address, bits) => {
                ok &= crate::proof::wait(|| rd(address) & bits == bits, 10_000, idle)
            }
        }
    }
    ok
}

/// One `IRQ_SWEEP` line per entry: whether the line was pending before, with the source
/// raised, and after it was stopped. A finding either way, never a pass condition.
pub fn run(idle: &mut dyn FnMut()) {
    for line in TABLE {
        let (word, bit) = (usize::from(line.irq / 32) * 4, 1 << (line.irq % 32));
        let pending = || (rd(NVIC_ISPR + word) & bit != 0) as u32;
        let mut ready = apply(line.prepare, idle);
        ready &= apply(line.quiesce, idle);
        settle(100, idle);
        wr(NVIC_ICPR + word, bit);
        let pre = pending();
        ready &= apply(line.raise, idle);
        settle(100, idle);
        let raised = pending();
        ready &= apply(line.quiesce, idle);
        settle(100, idle);
        wr(NVIC_ICPR + word, bit);
        mark!(
            "IRQ_SWEEP irq={} pre_pending={} raised={} post_pending={} ready={}",
            line.irq,
            pre,
            raised,
            pending(),
            ready as u32
        );
    }
    mark!("IRQ_SWEEP_DONE lines={}", TABLE.len());
}
