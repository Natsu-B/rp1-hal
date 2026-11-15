//! The sweep: for each line of [`TABLE`], raise the block's interrupt output and read which
//! NVIC pending bit of proc0 follows. No line is ever enabled, so no handler runs and a wrong
//! guess only leaves a pending bit. "Not routed" is a finding, not a failure.
//!
//! What the sweep may write: the registers of the declared blocks, their own reset bits, one
//! force bit of IO_BANK1/2, and the target bit of the NVIC's clear-pending register. Never a
//! pin (GPIO control, pads), an NVIC enable or priority, PCIe, or another block.
use crate::blocks::{Adc, Bank, I2c, Pwm1, Spi};
use crate::mark;
use crate::reg::{Reset, now, poll, rd, wr};
use core::fmt;

/// Longest wait of any step (resets, a controller switching off, an ADC conversion).
pub const STEP_US: u32 = 1_000;
/// How long a source, the block's output or the pending bit gets to follow a write.
pub const SETTLE_US: u32 = 100;
const TRIALS: usize = 3;
const MAX_STAGES: usize = 2;
const MAX_PINS: usize = 7;
pub const FUNCSEL_NULL: u8 = 0x1f;

/// The value or values a block wants recorded from its preparation.
pub type Found = [u32; 2];

/// Why a line did not run, or stopped.
#[derive(Clone, Copy)]
pub struct Why {
    reason: &'static str,
    detail: u32,
}
impl Why {
    pub const fn new(reason: &'static str) -> Self {
        Self::with(reason, 0)
    }
    pub const fn with(reason: &'static str, detail: u32) -> Self {
        Self { reason, detail }
    }
}

/// Why `prepare` refused a line, and whether it had written the block by then: that decides
/// how the block is left.
pub enum Refusal {
    /// Only reads happened. The block is left exactly as it was found, without another access.
    Untouched(Why),
    /// The block was written. It leaves like a swept block: the ordered deassert, then `cleanup`.
    Touched(Why),
}

/// A register of the block that must read `expect` under `mask` before a stimulus.
pub struct Check {
    offset: usize,
    mask: u32,
    expect: u32,
}
impl Check {
    pub const fn new(offset: usize, mask: u32, expect: u32) -> Self {
        Self {
            offset,
            mask,
            expect,
        }
    }
}

pub enum Rule {
    /// The line is skipped if the pin selects this function: the block would reach the pin.
    Not(u8),
    /// The line is skipped unless the pin selects this function.
    Only(u8),
}
/// A pin a block could drive, and the function select that decides whether the line may run.
pub struct Pin {
    pub gpio: u8,
    pub rule: Rule,
}
impl Pin {
    /// The pin's function select, read from its control register (never written).
    pub fn funcsel(&self) -> u8 {
        let (bank, index) = match self.gpio {
            0..=27 => (0x400d_0000, self.gpio),
            28..=33 => (0x400d_4000, self.gpio - 28),
            _ => (0x400d_8000, self.gpio - 34),
        };
        (rd(bank + 8 * usize::from(index) + 4) & 0x1f) as u8
    }
    pub fn allows(&self, funcsel: u8) -> bool {
        match self.rule {
            Rule::Not(function) => funcsel != function,
            Rule::Only(function) => funcsel == function,
        }
    }
}
const fn not<const N: usize>(gpios: [u8; N], function: u8) -> [Pin; N] {
    let mut pins = [const {
        Pin {
            gpio: 0,
            rule: Rule::Not(0),
        }
    }; N];
    let mut i = 0;
    while i < N {
        pins[i] = Pin {
            gpio: gpios[i],
            rule: Rule::Not(function),
        };
        i += 1;
    }
    pins
}

/// One block's way of raising its interrupt output. Every method is a few register accesses
/// with waits of at most [`STEP_US`].
pub trait Source: Sync {
    fn base(&self) -> usize;
    /// Every pin the block's signals can reach.
    fn pins(&self) -> &'static [Pin];
    /// The stimuli, each run from the baseline.
    fn stages(&self) -> &'static [&'static str];
    /// Checks the prerequisites and brings the block to its baseline, the source lowered.
    fn prepare(&self) -> Result<Found, Refusal>;
    fn baseline(&self) -> &'static [Check];
    /// Raises the source; returns whether the block reports what this stage intends.
    fn raise(&self, stage: usize) -> bool;
    /// The block's interrupt status, recorded with each observation.
    fn status(&self) -> u32;
    fn lower(&self);
    /// Whether the block reports its interrupt output low.
    fn lowered(&self) -> bool;
    /// Back to the baseline after `lower`, for the next stimulus.
    fn rearm(&self) -> bool;
    /// Lowers the source and leaves the block with every interrupt enable reading 0, in reset
    /// where the block has a reset of its own. Returns false, without holding the reset, if an
    /// enable does not read back 0.
    fn cleanup(&self) -> bool;
    /// The state `cleanup` must leave.
    fn settled(&self) -> bool;
}

pub struct Line {
    pub irq: u8,
    pub name: &'static str,
    pub source: &'static dyn Source,
}

const I2C: u8 = 3;
const SPI: u8 = 8;
/// SPI1 uses function 0 on its main pins and the usual 8 on its second chip select.
const SPI1_PINS: [Pin; 7] = {
    let [a, b, c, d, e, f] = not([16, 17, 18, 19, 20, 21], 0);
    let [g] = not([27], SPI);
    [a, b, c, d, e, f, g]
};

/// The lines whose number is an estimate and whose block can be raised without a pin, a
/// transfer or another owner's state. The others (DMA 40, SPI5 24, PIO 50/51, CLOCKS 59,
/// VBUS 60) have no safe stimulus yet and must stay out.
pub const TABLE: &[Line] = &[
    Line {
        irq: 7,
        name: "i2c0",
        source: &I2c {
            base: 0x4007_0000,
            reset: Reset { bank: 0, bit: 7 },
            pins: &not([0, 1, 8, 9], I2C),
        },
    },
    Line {
        irq: 9,
        name: "i2c2",
        source: &I2c {
            base: 0x4007_8000,
            reset: Reset { bank: 0, bit: 9 },
            pins: &not([4, 5, 12, 13], I2C),
        },
    },
    Line {
        irq: 10,
        name: "i2c3",
        source: &I2c {
            base: 0x4007_c000,
            reset: Reset { bank: 0, bit: 10 },
            pins: &not([6, 7, 14, 15, 22, 23], I2C),
        },
    },
    Line {
        irq: 20,
        name: "spi1",
        source: &Spi {
            base: 0x4005_4000,
            reset: Reset { bank: 1, bit: 11 },
            pins: &SPI1_PINS,
        },
    },
    Line {
        irq: 22,
        name: "spi3",
        source: &Spi {
            base: 0x4005_c000,
            reset: Reset { bank: 1, bit: 13 },
            pins: &not([4, 5, 6, 7, 25], SPI),
        },
    },
    Line {
        irq: 52,
        name: "adc",
        source: &Adc,
    },
    Line {
        irq: 41,
        name: "pwm1",
        source: &Pwm1,
    },
    Line {
        irq: 1,
        name: "io_bank1",
        source: &Bank { base: 0x400d_4000 },
    },
    Line {
        irq: 2,
        name: "io_bank2",
        source: &Bank { base: 0x400d_8000 },
    },
];
const _: () = {
    let mut i = 0;
    while i < TABLE.len() {
        assert!(!matches!(TABLE[i].irq, 24 | 40 | 50 | 51 | 59 | 60));
        assert!(TABLE[i].irq < 64);
        i += 1;
    }
};

const NVIC_ISER: usize = 0xe000_e100;
const NVIC_ISPR: usize = 0xe000_e200;
const NVIC_ICPR: usize = 0xe000_e280;
/// The host's view of RP1's interrupts (PCIe MSI-X status). Recorded only: it is not known to
/// show the same signals as the NVIC.
const INTSTAT: usize = 0x4010_8108;
fn words(address: usize) -> [u32; 2] {
    [rd(address), rd(address + 4)]
}

/// One line's pending bit.
struct Target {
    word: usize,
    bit: u32,
}
impl Target {
    fn of(irq: u8) -> Self {
        Self {
            word: usize::from(irq / 32),
            bit: 1 << (irq % 32),
        }
    }
    fn is_in(&self, pending: [u32; 2]) -> bool {
        pending[self.word] & self.bit != 0
    }
    /// Clears this bit only, and returns the pending words afterwards.
    fn clear(&self) -> [u32; 2] {
        wr(NVIC_ICPR + 4 * self.word, self.bit);
        rp1_proto::barrier();
        words(NVIC_ISPR)
    }
    /// The bits of `after` that are neither in `before` nor this line's.
    fn foreign(&self, before: [u32; 2], after: [u32; 2]) -> [u32; 2] {
        let mut new = [after[0] & !before[0], after[1] & !before[1]];
        new[self.word] &= !self.bit;
        new
    }
}

/// One stimulus and what followed. Kept in memory until the block is quiet again, so that the
/// log's delay is never inside a measurement.
#[derive(Clone, Copy, Default)]
struct Observation {
    stage: u8,
    trial: u8,
    /// Pending words as found, and after the target bit was cleared.
    pre: [u32; 2],
    base: [u32; 2],
    /// With the source raised: pending words, the block's status, the host's view, and how
    /// long the target bit took (or the whole settle time).
    post: [u32; 2],
    status: u32,
    intstat: [u32; 2],
    us: u32,
    /// Pending words after the source was lowered and the target bit cleared.
    cleared: [u32; 2],
}

/// How an observation ended early.
enum End {
    /// Nothing was raised: the baseline did not hold.
    Unmet(Why),
    /// The source did not come up.
    NoSource,
    /// A pending bit appeared that is not the target's. It is left pending: it has no owner
    /// here, and with every line disabled it cannot run a handler.
    Foreign,
    /// The block or the NVIC did not return to the baseline.
    Broken(Why),
}

fn observe(source: &dyn Source, target: &Target, o: &mut Observation) -> Result<bool, End> {
    for check in source.baseline() {
        let value = rd(source.base() + check.offset);
        if value & check.mask != check.expect {
            return Err(End::Unmet(Why::with("baseline", check.offset as u32)));
        }
    }
    o.pre = words(NVIC_ISPR);
    o.base = target.clear();
    if target.is_in(o.base) {
        return Err(End::Unmet(Why::new("pending_before")));
    }

    let raised = source.raise(usize::from(o.stage));
    rp1_proto::barrier();
    let start = now();
    poll(SETTLE_US, || target.is_in(words(NVIC_ISPR)));
    o.us = now().wrapping_sub(start);
    o.post = words(NVIC_ISPR);
    o.status = source.status();
    o.intstat = words(INTSTAT);

    // Ordered: source off, the block's output read back low, and only then the pending bit.
    source.lower();
    rp1_proto::barrier();
    if !poll(SETTLE_US, || source.lowered()) {
        return Err(End::Broken(Why::new("output_stuck")));
    }
    target.clear();
    // A level that is still high would set the bit again.
    poll(SETTLE_US, || target.is_in(words(NVIC_ISPR)));
    o.cleared = words(NVIC_ISPR);
    if target.is_in(o.cleared) {
        return Err(End::Broken(Why::new("pending_after")));
    }
    if !source.rearm() {
        return Err(End::Broken(Why::new("rearm")));
    }
    if !raised {
        return Err(End::NoSource);
    }
    if target.foreign(o.base, o.post) != [0; 2] {
        return Err(End::Foreign);
    }
    Ok(target.is_in(o.post))
}

enum Verdict {
    /// The stage whose every trial set the target bit.
    Routed(usize),
    NotRouted,
    /// A foreign bit, without or with the target bit.
    OtherLine,
    NewPending,
    Skip(Why),
    /// The procedure broke; the lines after this one are not run.
    Abort(Why),
}

struct Funcsels<'a>(&'a [Pin], [u8; MAX_PINS]);
impl fmt::Display for Funcsels<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.is_empty() {
            return f.write_str("-");
        }
        for (i, pin) in self.0.iter().enumerate() {
            let separator = if i == 0 { "" } else { "," };
            write!(f, "{}{}:{:x}", separator, pin.gpio, self.1[i])?;
        }
        Ok(())
    }
}
struct Hits<'a>(&'a [&'static str], [u8; MAX_STAGES]);
impl fmt::Display for Hits<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, stage) in self.0.iter().enumerate() {
            let separator = if i == 0 { "" } else { "," };
            write!(f, "{}{}:{}/{}", separator, stage, self.1[i], TRIALS)?;
        }
        Ok(())
    }
}

fn sweep_line(line: &Line) -> Verdict {
    let (source, target) = (line.source, Target::of(line.irq));
    let stages = source.stages();
    let mut observations = [Observation::default(); TRIALS * MAX_STAGES];
    let mut taken = 0;
    let mut hits = [0; MAX_STAGES];
    let mut funcsels = [0; MAX_PINS];
    let mut found = [0; 2];
    let mut end = None;
    // Whether anything was written to the block: only then does it need leaving.
    let mut written = false;

    for (pin, funcsel) in source.pins().iter().zip(&mut funcsels) {
        *funcsel = pin.funcsel();
        if !pin.allows(*funcsel) && end.is_none() {
            end = Some(End::Unmet(Why::with("pin_routed", u32::from(pin.gpio))));
        }
    }
    if end.is_none() {
        match source.prepare() {
            Ok(values) => (found, written) = (values, true),
            Err(Refusal::Untouched(why)) => end = Some(End::Unmet(why)),
            Err(Refusal::Touched(why)) => (end, written) = (Some(End::Unmet(why)), true),
        }
    }
    'trials: for trial in 0..TRIALS {
        for (stage, hit) in hits.iter_mut().enumerate().take(stages.len()) {
            if end.is_some() {
                break 'trials;
            }
            let o = &mut observations[taken];
            (o.stage, o.trial) = (stage as u8, trial as u8);
            taken += 1;
            match observe(source, &target, o) {
                Ok(pending) => *hit += u8::from(pending),
                Err(stop) => end = Some(stop),
            }
        }
    }

    let last = observations[taken.saturating_sub(1)];
    let mut verdict = match end {
        // Before anything was raised the line is merely not run; later, the block drifted.
        Some(End::Unmet(why)) if taken <= 1 => Verdict::Skip(why),
        Some(End::Unmet(why) | End::Broken(why)) => Verdict::Abort(why),
        Some(End::NoSource) => Verdict::Skip(Why::with("source_fail", last.status)),
        Some(End::Foreign) if target.is_in(last.post) => Verdict::NewPending,
        Some(End::Foreign) => Verdict::OtherLine,
        None => match hits.iter().position(|&n| usize::from(n) == TRIALS) {
            Some(stage) => Verdict::Routed(stage),
            None if hits == [0; MAX_STAGES] => Verdict::NotRouted,
            None => Verdict::Abort(Why::new("unstable")),
        },
    };
    // A block that was only read is left exactly as it was found.
    if written {
        let quiet = source.cleanup();
        let after = if !quiet {
            Some("cleanup")
        } else if !source.settled() || target.is_in(words(NVIC_ISPR)) {
            Some("postcondition")
        } else if words(NVIC_ISER) != [0; 2] {
            Some("iser")
        } else {
            None
        };
        if let (Some(reason), false) = (after, matches!(verdict, Verdict::Abort(_))) {
            verdict = Verdict::Abort(Why::new(reason));
        }
    }

    mark!(
        "IRQ_SWEEP_PRE irq={} fs={} found={:x},{:x}",
        line.irq,
        Funcsels(source.pins(), funcsels),
        found[0],
        found[1]
    );
    for o in &observations[..taken] {
        mark!(
            "IRQ_SWEEP_TRIAL irq={} stage={} trial={} pre={:x},{:x} base={:x},{:x} post={:x},{:x} status={:x} intstat={:x},{:x} us={} clr={:x},{:x}",
            line.irq,
            stages[usize::from(o.stage)],
            o.trial,
            o.pre[0],
            o.pre[1],
            o.base[0],
            o.base[1],
            o.post[0],
            o.post[1],
            o.status,
            o.intstat[0],
            o.intstat[1],
            o.us,
            o.cleared[0],
            o.cleared[1]
        );
    }
    // The observation the result rests on: the routed stage's last trial, else the last one.
    let shown = match verdict {
        Verdict::Routed(stage) => observations[..taken]
            .iter()
            .rfind(|o| usize::from(o.stage) == stage)
            .copied()
            .unwrap_or(last),
        _ if taken > 0 => last,
        _ => {
            let pending = words(NVIC_ISPR);
            Observation {
                base: pending,
                post: pending,
                ..last
            }
        }
    };
    report(line, &verdict, Hits(stages, hits), &shown, &target);
    verdict
}

fn report(line: &Line, verdict: &Verdict, hits: Hits<'_>, o: &Observation, target: &Target) {
    let (result, reason, detail) = match verdict {
        Verdict::Routed(stage) => ("ROUTED", hits.0[*stage], 0),
        Verdict::NotRouted => ("NOT_ROUTED", "-", 0),
        Verdict::OtherLine => ("OTHER_LINE", "-", 0),
        Verdict::NewPending => ("NEW_PENDING", "-", 0),
        Verdict::Skip(why) => ("SKIP", why.reason, why.detail),
        Verdict::Abort(why) => ("ABORT", why.reason, why.detail),
    };
    let new = target.foreign(o.base, o.post);
    mark!(
        "IRQ_SWEEP irq={} block={} result={} reason={} detail={:x} hits={} ispr_before={:08x},{:08x} ispr_after={:08x},{:08x} new={:08x},{:08x} status={:x}",
        line.irq,
        line.name,
        result,
        reason,
        detail,
        hits,
        o.base[0],
        o.base[1],
        o.post[0],
        o.post[1],
        new[0],
        new[1],
        o.status
    );
}

/// Runs the whole table and prints one `IRQ_SWEEP` line per entry between `IRQ_SWEEP_BEGIN`
/// and `IRQ_SWEEP_END`. Nothing runs unless every NVIC line of this core is disabled: that is
/// what makes an unexpected route harmless.
pub fn run() {
    let start = now();
    let enabled = words(NVIC_ISER);
    mark!(
        "IRQ_SWEEP_BEGIN lines={} iser={:08x},{:08x} ispr={:08x},{:08x}",
        TABLE.len(),
        enabled[0],
        enabled[1],
        rd(NVIC_ISPR),
        rd(NVIC_ISPR + 4)
    );
    let mut blocked = (enabled != [0; 2]).then_some(Why::new("irq_enabled"));
    for line in TABLE {
        match blocked {
            None => {
                if let Verdict::Abort(_) = sweep_line(line) {
                    blocked = Some(Why::with("after_abort", u32::from(line.irq)));
                }
            }
            Some(why) => {
                let pending = words(NVIC_ISPR);
                let idle = Observation {
                    base: pending,
                    post: pending,
                    ..Observation::default()
                };
                let hits = Hits(line.source.stages(), [0; MAX_STAGES]);
                report(
                    line,
                    &Verdict::Skip(why),
                    hits,
                    &idle,
                    &Target::of(line.irq),
                );
            }
        }
    }
    let enabled = words(NVIC_ISER);
    mark!(
        "IRQ_SWEEP_END lines={} iser={:08x},{:08x} us={}",
        TABLE.len(),
        enabled[0],
        enabled[1],
        now().wrapping_sub(start)
    );
}
