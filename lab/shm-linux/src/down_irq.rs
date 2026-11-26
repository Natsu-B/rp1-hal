//! Linux -> RP1 notification against lab/irq-proof: Linux sends, the doorbell handler of the
//! ring's core drains (its budget per entry, raising the doorbell again on `More`) and checks
//! every message.
//!
//! - `down-irq [--max N] [--burst N] [count]`: ring `down1`, proc1, doorbell GPIO27. Prints
//!   `DOWNIRQ ...` and `DOWNIRQ_PASS`; with a budget below the burst (the handler must run out
//!   of budget) also `DOWNIRQ_MORE raises=.. missing=..` and `DOWNIRQ_MORE_PASS` instead.
//! - `down-irq --stream-only [count]`: send only, no commands (killed by check-irq.sh at a
//!   random moment; the next run must still go through).
//! - `down0-irq [--max N] [--burst N] [count]`: ring `down0`, proc0, doorbell GPIO26, as above
//!   with `DOWN0IRQ`.
//! - `controls`: the two record-only control runs on `down1`, `CONTROL_J1` (this writer skips
//!   the readback before it looks at the reader) and `CONTROL_J2` (the handler skips the
//!   readback of its arm). A stall is expected to be possible there; each is counted and
//!   recovered by the next send with the readback.
//!
//! A stall: the ring holds messages and the reader's `tail` has not moved for [`STALL`]
//! (1 s). The reader has no timed re-check, so a lost doorbell stops it for good; a stall
//! prints `<NAME>_STALL` and fails the run.
use crate::irq_common::{
    DOWN_REPORT, DOWN_START, DOWN_START_WITHOUT_READBACK, Ids, STALL, command, parse_args,
};
use crate::irq_message::{Message, message};
use crate::{Control, READY, Result};
use rp1_linux::{Error, Firmware, RingTx};
use rp1_proto as proto;
use std::time::{Duration, Instant};

/// One Linux -> RP1 ring of irq-proof.
pub struct Ring {
    name: &'static str,
    /// The block that takes the ring core's commands.
    control: &'static str,
    /// Its depth in the declaration.
    depth: u32,
    label: &'static str,
}
pub const DOWN1: Ring = Ring {
    name: "down1",
    control: "control",
    depth: 128,
    label: "DOWNIRQ",
};
pub const DOWN0: Ring = Ring {
    name: "down0",
    control: "control0",
    depth: 64,
    label: "DOWN0IRQ",
};

/// Watches the reader's progress for the stall rule.
struct Watch {
    depth: u32,
    free: u32,
    since: Instant,
}
impl Watch {
    fn new(depth: u32) -> Self {
        Self {
            depth,
            free: u32::MAX,
            since: Instant::now(),
        }
    }
    /// Whether the ring has held messages with the same free count for [`STALL`].
    fn stalled(&mut self, free: u32) -> bool {
        if free != self.free || free == self.depth {
            self.free = free;
            self.since = Instant::now();
            return false;
        }
        self.since.elapsed() >= STALL
    }
}

/// How a wait for the reader ended.
enum Drained {
    Empty,
    /// Messages still in the ring and `tail` stuck for [`STALL`].
    Stall {
        pending: u32,
    },
}

/// Wait until the reader has taken everything.
fn wait_drained(tx: &RingTx<Message>, depth: u32) -> Drained {
    let mut watch = Watch::new(depth);
    loop {
        let free = tx.free();
        if free == depth {
            return Drained::Empty;
        }
        if watch.stalled(free) {
            return Drained::Stall {
                pending: depth - free,
            };
        }
        std::thread::sleep(Duration::from_micros(50));
    }
}

/// `down-irq` / `down0-irq`.
pub fn main(ring: &Ring, mut args: impl Iterator<Item = String>) -> Result<()> {
    let mut rest: Vec<String> = Vec::new();
    let mut stream_only = false;
    for arg in args.by_ref() {
        if arg == "--stream-only" {
            stream_only = true;
        } else {
            rest.push(arg);
        }
    }
    let (flags, count) = parse_args(
        rest.into_iter(),
        &[("--max", 16), ("--burst", 1)],
        1_000_000,
    )?;
    let (budget, burst) = (flags[0], flags[1].clamp(1, ring.depth));
    let fw = Firmware::open()?;
    fw.wait_ready(READY)?;
    let mut tx = fw.ring_tx::<Message>(ring.name)?;
    let mut ids = Ids::new();
    let generation = ids.next();
    if stream_only {
        let mut seq = 0;
        while seq < count {
            match tx.send(&message(seq, generation)) {
                Ok(()) => seq += 1,
                Err(Error::Proto(proto::Error::Full)) => {}
                Err(e) => return Err(e.into()),
            }
        }
        return Ok(());
    }
    let mut ctrl = fw.block::<Control>(ring.control)?;
    drop(fw);
    command(&mut ctrl, &mut ids, DOWN_START, budget, generation, &[])?;

    let start = Instant::now();
    let mut stalls = 0;
    let (mut seq, mut watch) = (0u32, Watch::new(ring.depth));
    let mut batch = Vec::with_capacity(burst as usize);
    'send: while seq < count {
        let k = (count - seq).min(burst);
        batch.clear();
        batch.extend((seq..seq + k).map(|s| message(s, generation)));
        let mut done = 0;
        while done < batch.len() {
            let sent = if burst == 1 {
                match tx.send(&batch[0]) {
                    Ok(()) => 1,
                    Err(Error::Proto(proto::Error::Full)) => 0,
                    Err(e) => return Err(e.into()),
                }
            } else {
                tx.send_slice(&batch[done..])?
            };
            done += sent;
            if sent == 0 && watch.stalled(tx.free()) {
                println!(
                    "{}_STALL ring={} at={} pending={}",
                    ring.label,
                    ring.name,
                    seq as usize + done,
                    ring.depth - tx.free()
                );
                stalls += 1;
                break 'send;
            }
        }
        seq += k;
    }
    if stalls == 0
        && let Drained::Stall { pending } = wait_drained(&tx, ring.depth)
    {
        println!(
            "{}_STALL ring={} at={count} pending={pending}",
            ring.label, ring.name
        );
        stalls += 1;
    }
    let seconds = start.elapsed().as_secs_f64();
    let r = command(&mut ctrl, &mut ids, DOWN_REPORT, 0, 0, &[])?;
    let (received, corrupt, stale, entries, raises, errors) =
        (r[8], r[9], r[10], r[11], r[12], r[13]);
    let missing = count - received.min(count);
    println!(
        "{} ring={} n={count} missing={missing} corrupt={corrupt} stalls={stalls} max={budget} \
         burst={burst} entries={entries} raises={raises} stale={stale} errors={errors} \
         seconds={seconds:.6} msgs_per_s={:.0}",
        ring.label,
        ring.name,
        count as f64 / seconds
    );
    let ok = missing == 0 && corrupt == 0 && stalls == 0 && errors == 0;
    if budget < burst {
        println!(
            "{}_MORE raises={raises} missing={missing} corrupt={corrupt} stalls={stalls}",
            ring.label
        );
        if ok && raises > 0 {
            println!("{}_MORE_PASS", ring.label);
            return Ok(());
        }
    } else if ok {
        println!("{}_PASS", ring.label);
        return Ok(());
    }
    println!("{}_FAIL", ring.label);
    Err(format!("{} failed", ring.label).into())
}

/// Messages per control run, and the time it may take (check-irq.sh gives `controls` 30 s).
const CONTROL_COUNT: u32 = 20_000;
const CONTROL_TIME: Duration = Duration::from_secs(12);

/// `controls`: (j1) and (j2), record only. Exit status 0 unless a run could not be made.
pub fn controls() -> Result<()> {
    let fw = Firmware::open()?;
    fw.wait_ready(READY)?;
    let mut tx = fw.ring_tx::<Message>(DOWN1.name)?;
    let mut ctrl = fw.block::<Control>(DOWN1.control)?;
    drop(fw);
    let mut ids = Ids::new();
    for (label, linux_readback, start_command) in [
        ("CONTROL_J1", false, DOWN_START),
        ("CONTROL_J2", true, DOWN_START_WITHOUT_READBACK),
    ] {
        let generation = ids.next();
        command(&mut ctrl, &mut ids, start_command, 16, generation, &[])?;
        let start = Instant::now();
        let (mut sent, mut stalls, mut late) = (0u32, 0u32, 0u32);
        // After a stall the next message goes with the readback: it rings for the stuck one.
        let mut recover = false;
        while sent < CONTROL_COUNT && start.elapsed() < CONTROL_TIME {
            let m = message(sent, generation);
            if linux_readback || recover {
                tx.send(&m)?;
            } else {
                tx.__send_without_readback(&m)?;
            }
            sent += 1;
            let sent_at = Instant::now();
            recover = matches!(wait_drained(&tx, DOWN1.depth), Drained::Stall { .. });
            stalls += u32::from(recover);
            late += u32::from(!recover && sent_at.elapsed() > Duration::from_millis(1));
        }
        if recover {
            // Ring the doorbell for the last stuck message before the next run.
            tx.send(&message(sent, generation))?;
            sent += 1;
            let _ = wait_drained(&tx, DOWN1.depth);
        }
        let r = command(&mut ctrl, &mut ids, DOWN_REPORT, 0, 0, &[])?;
        println!(
            "{label} n={sent} stalls={stalls} late={late} received={} corrupt={} \
             seconds={:.3} (record only)",
            r[8],
            r[9],
            start.elapsed().as_secs_f64()
        );
    }
    // Leave the handler draining with the readback.
    command(&mut ctrl, &mut ids, DOWN_START, 16, 0, &[])?;
    Ok(())
}
