//! What the irq-proof notification subcommands share: request ids, the commands of irq-proof's
//! `rings` module and the per-message check of lab/irq_message.rs.
use crate::irq_message::{self, Message};
use crate::{Control, Result, request_with, response};
use rp1_linux::BlockMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub const DOWN_START: u32 = 20;
pub const DOWN_REPORT: u32 = 21;
pub const EVENTS: u32 = 22;
pub const TICKS: u32 = 23;
pub const TICK_REPORT: u32 = 24;
pub const DOWN_START_WITHOUT_READBACK: u32 = 25;

/// How long a ring may hold messages without its reader taking any before the run is a stall.
pub const STALL: Duration = Duration::from_secs(1);

/// Request ids and run generations. The firmware ignores a request with the id it served
/// last, and runs of earlier processes left their own ids, so every process starts from the
/// clock.
pub struct Ids(u32);
impl Ids {
    pub fn new() -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |t| t.subsec_nanos() ^ t.as_secs() as u32);
        // Above the default suite's ids (1..13, 100..108).
        Self(nanos | 0x8000_0000)
    }
    pub fn next(&mut self) -> u32 {
        self.0 = self.0.wrapping_add(1) | 0x8000_0000;
        self.0
    }
}

/// One command to irq-proof and its answer (`response` fails on RP1-reported errors).
pub fn command(
    ctrl: &mut BlockMap<Control>,
    ids: &mut Ids,
    command: u32,
    count: u32,
    seed: u32,
    extra: &[u32],
) -> Result<Control> {
    let id = ids.next();
    let start = request_with(ctrl, id, command, count, seed, extra)?;
    response(ctrl, id, start)
}

/// The Linux side's check of an RP1 -> Linux run, as the firmware's `Tally` for the other
/// direction.
pub struct Checker {
    generation: u32,
    next: u32,
    /// Whole messages of this run, in order.
    pub received: u32,
    /// Not whole, or not the next in order.
    pub corrupt: u32,
    /// Whole messages of another run.
    pub stale: u32,
}
impl Checker {
    pub fn new(generation: u32) -> Self {
        Self {
            generation,
            next: 0,
            received: 0,
            corrupt: 0,
            stale: 0,
        }
    }
    pub fn check(&mut self, m: &Message) {
        if !irq_message::is_whole(m) {
            self.corrupt += 1;
        } else if m[2] != self.generation {
            self.stale += 1;
        } else if m[0] != self.next {
            self.corrupt += 1;
            self.next = m[0].wrapping_add(1);
        } else {
            self.received += 1;
            self.next = m[0].wrapping_add(1);
        }
    }
}

/// Parses `[--name value]... [count]`: the values of the flags in `names` (in that order) and
/// the count, with defaults.
pub fn parse_args(
    mut args: impl Iterator<Item = String>,
    names: &[(&str, u32)],
    count: u32,
) -> Result<(Vec<u32>, u32)> {
    let mut values: Vec<u32> = names.iter().map(|(_, v)| *v).collect();
    let mut count = count;
    while let Some(arg) = args.next() {
        if let Some(i) = names.iter().position(|(n, _)| *n == arg) {
            let value = args.next().ok_or(format!("{arg} needs a value"))?;
            values[i] = value.parse()?;
        } else {
            count = arg.parse()?;
        }
    }
    Ok((values, count))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::irq_message::{crc32, message};

    #[test]
    fn crc_is_zlibs() {
        // zlib.crc32 of the same eight little-endian bytes.
        assert_eq!(crc32(0, 0), 0x6522_df69);
        assert_eq!(crc32(1, 0x1234_5678), 0x27a1_8739);
    }

    #[test]
    fn checker_tells_stale_corrupt_and_missing_apart() {
        let mut c = Checker::new(7);
        c.check(&message(0, 7));
        c.check(&message(0, 6)); // an earlier run's
        c.check(&message(2, 7)); // 1 is missing: out of order
        let mut torn = message(3, 7);
        torn[3] ^= 1;
        c.check(&torn);
        c.check(&message(3, 7));
        assert_eq!((c.received, c.corrupt, c.stale), (2, 2, 1));
    }

    #[test]
    fn flags_and_count() {
        let args = |s: &str| s.split_whitespace().map(String::from).collect::<Vec<_>>();
        let names = [("--max", 16), ("--burst", 1)];
        let (v, n) = parse_args(args("--max 1 --burst 128 100000").into_iter(), &names, 5).unwrap();
        assert_eq!((v, n), (vec![1, 128], 100_000));
        let (v, n) = parse_args(args("").into_iter(), &names, 5).unwrap();
        assert_eq!((v, n), (vec![16, 1], 5));
        assert!(parse_args(args("--max").into_iter(), &names, 5).is_err());
    }
}
