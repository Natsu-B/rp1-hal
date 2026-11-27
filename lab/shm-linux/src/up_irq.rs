//! RP1 -> Linux notification from proc0 of lab/irq-proof (the `host_irq` owner there, while
//! it also serves SCMI and PCIe): `events0` is sent by proc0's main, `tick0` by its TIMER_0
//! handler, both with `RingTx::send*`, and Linux sleeps on `rp1:irq` between messages.
//!
//! - `host0-irq [count]`: `events0` alone, a burst run and a paced run (where Linux must
//!   sleep). `RINGIRQ0 name=..` per run, `RINGIRQ0_PASS` if every run has no missing or
//!   corrupt message and, in the paced run, slept, was woken and found data by the re-check
//!   in at most 1 % of its sleeps.
//! - `mix-irq [count]`: both rings at once, Linux waiting on both with `Firmware::wait_any`.
//!   `MIXIRQ ... late_pct=.. missing=..` and `MIXIRQ_PASS` (late at most 1 % of the waits).
//! - `irq-pair`: two processes, one per ring, each with its own `rp1:irq` handle, at once.
//!   `IRQPAIR ...` and `IRQPAIR_PASS` when both got every message (late is recorded).
//!
//! "Late": a wait that ended with data only after [`RECHECK`], i.e. found by the re-check,
//! not by an interrupt.
use crate::irq_common::{Checker, EVENTS, Ids, TICK_REPORT, TICKS, command, parse_args};
use crate::irq_message::Message;
use crate::{Control, READY, Result, request_with, response};
use rp1_linux::{Firmware, RECHECK, RingRx};
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// Ticks of the mixed runs: one every 500 µs for a second.
const TICK_COUNT: u32 = 2000;
const TICK_PERIOD_US: u32 = 500;
/// How long a run may take.
const LIMIT: Duration = Duration::from_secs(180);

/// Receive what is there into `checker`; returns how many.
fn take(rx: &mut RingRx<Message>, checker: &mut Checker) -> Result<usize> {
    let mut buf = [[0u32; 4]; 64];
    let n = rx.try_recv_into(&mut buf)?;
    for m in &buf[..n] {
        checker.check(m);
    }
    Ok(n)
}

/// `host0-irq`.
pub fn host0(args: impl Iterator<Item = String>) -> Result<()> {
    let (_, count) = parse_args(args, &[], 100_000)?;
    let fw = Firmware::open()?;
    fw.wait_ready(READY)?;
    let mut events = fw.ring_rx::<Message>("events0")?;
    let mut ctrl = fw.block::<Control>("control0")?;
    drop(fw);
    let mut ids = Ids::new();
    let mut failed = Vec::new();
    // (name, batch, gap_us, must sleep)
    for (name, batch, gap_us, paced) in [("burst-irq", 32, 0, false), ("paced-irq", 32, 100, true)]
    {
        let generation = ids.next();
        let id = ids.next();
        let stats0 = events.wait_stats();
        let start = request_with(&mut ctrl, id, EVENTS, count, generation, &[batch, gap_us])?;
        let mut checker = Checker::new(generation);
        let mut buf = [[0u32; 4]; 32];
        while checker.received + checker.corrupt < count && start.elapsed() < LIMIT {
            let n = events.recv_into(&mut buf, Some(Duration::from_secs(1)))?;
            for m in &buf[..n] {
                checker.check(m);
            }
        }
        let seconds = start.elapsed().as_secs_f64();
        response(&ctrl, id, start)?;
        let s = events.wait_stats();
        let (sleeps, woken, late) = (
            s.sleeps - stats0.sleeps,
            s.woken - stats0.woken,
            s.late - stats0.late,
        );
        let missing = count - checker.received.min(count);
        let check = if missing != 0 || checker.corrupt != 0 {
            "FAIL(messages)"
        } else if paced && sleeps == 0 {
            "INCONCLUSIVE(never-slept)"
        } else if sleeps > 0 && (woken == 0 || late * 100 > sleeps) {
            "FAIL(not-notified)"
        } else {
            "ok"
        };
        if check != "ok" {
            failed.push(name);
        }
        println!(
            "RINGIRQ0 name={name} n={} missing={missing} corrupt={} stale={} batch={batch} \
             gap_us={gap_us} seconds={seconds:.6} sleeps={sleeps} woken={woken} late={late} \
             check={check}",
            checker.received, checker.corrupt, checker.stale
        );
        // Let a trailing pulse land before the next run's counters start.
        std::thread::sleep(Duration::from_millis(50));
    }
    if !failed.is_empty() {
        println!("RINGIRQ0_FAIL runs={failed:?}");
        return Err("host0-irq failed".into());
    }
    println!("RINGIRQ0_PASS count={count}");
    Ok(())
}

/// Starts the tick run, then the event stream (answered when it is sent): returns the event
/// request's id and time.
fn start_both(
    ctrl: &mut rp1_linux::BlockMap<Control>,
    ids: &mut Ids,
    count: u32,
    events: u32,
    ticks: u32,
) -> Result<(u32, Instant)> {
    command(ctrl, ids, TICKS, TICK_COUNT, ticks, &[TICK_PERIOD_US])?;
    let id = ids.next();
    let start = request_with(ctrl, id, EVENTS, count, events, &[32, 100])?;
    Ok((id, start))
}

/// `mix-irq`.
pub fn mix(args: impl Iterator<Item = String>) -> Result<()> {
    let (_, count) = parse_args(args, &[], 100_000)?;
    let fw = Firmware::open()?;
    fw.wait_ready(READY)?;
    let mut events = fw.ring_rx::<Message>("events0")?;
    let mut ticks = fw.ring_rx::<Message>("tick0")?;
    let mut ctrl = fw.block::<Control>("control0")?;
    let mut ids = Ids::new();
    let (events_gen, ticks_gen) = (ids.next(), ids.next());
    let (mut got_events, mut got_ticks) = (Checker::new(events_gen), Checker::new(ticks_gen));
    let (id, start) = start_both(&mut ctrl, &mut ids, count, events_gen, ticks_gen)?;
    let (mut waits, mut late) = (0u32, 0u32);
    let mut idle_since = Instant::now();
    while start.elapsed() < LIMIT {
        let n = take(&mut events, &mut got_events)? + take(&mut ticks, &mut got_ticks)?;
        if n != 0 {
            idle_since = Instant::now();
            continue;
        }
        let all_events = got_events.received + got_events.corrupt >= count;
        let all_ticks = got_ticks.received + got_ticks.corrupt >= TICK_COUNT;
        // Ticks found the ring full are not sent: stop once nothing came for a while.
        if all_events && (all_ticks || idle_since.elapsed() > Duration::from_secs(2)) {
            break;
        }
        let t = Instant::now();
        let ready =
            fw.wait_any_for(&mut [&mut events, &mut ticks], Some(Duration::from_secs(1)))?;
        waits += 1;
        late += u32::from(ready && t.elapsed() >= RECHECK);
    }
    let seconds = start.elapsed().as_secs_f64();
    response(&ctrl, id, start)?;
    let r = command(&mut ctrl, &mut ids, TICK_REPORT, 0, 0, &[])?;
    let (sent_ticks, full, tick_errors, tick_late_us) = (r[8], r[9], r[10], r[11]);
    let missing = (count - got_events.received.min(count))
        + (sent_ticks - got_ticks.received.min(sent_ticks));
    let corrupt = got_events.corrupt + got_ticks.corrupt;
    let late_pct = 100.0 * f64::from(late) / f64::from(waits.max(1));
    println!(
        "MIXIRQ events={} ticks={} ticks_sent={sent_ticks} ticks_full={full} \
         tick_errors={tick_errors} tick_late_max_us={tick_late_us} missing={missing} \
         corrupt={corrupt} waits={waits} late={late} late_pct={late_pct:.2} seconds={seconds:.3}",
        got_events.received, got_ticks.received
    );
    if missing == 0 && corrupt == 0 && tick_errors == 0 && late_pct <= 1.0 {
        println!("MIXIRQ_PASS");
        Ok(())
    } else {
        println!("MIXIRQ_FAIL");
        Err("mix-irq failed".into())
    }
}

/// `irq-pair`: starts one `pair-member` process per ring, then the runs.
pub fn pair(args: impl Iterator<Item = String>) -> Result<()> {
    let (_, count) = parse_args(args, &[], 100_000)?;
    let fw = Firmware::open()?;
    fw.wait_ready(READY)?;
    let mut ctrl = fw.block::<Control>("control0")?;
    let mut ids = Ids::new();
    let (events_gen, ticks_gen) = (ids.next(), ids.next());
    let me = std::env::current_exe()?;
    let mut members = Vec::new();
    for (ring, generation, expect) in [
        ("events0", events_gen, count),
        ("tick0", ticks_gen, TICK_COUNT),
    ] {
        let mut child = Command::new(&me)
            .args([
                "pair-member",
                ring,
                &generation.to_string(),
                &expect.to_string(),
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .spawn()?;
        let mut out = BufReader::new(child.stdout.take().ok_or("no member stdout")?);
        let mut line = String::new();
        out.read_line(&mut line)?;
        if line.trim() != "PAIR_READY" {
            return Err(format!("pair member {ring}: {line:?}").into());
        }
        members.push((ring, child, out));
    }
    let (id, start) = start_both(&mut ctrl, &mut ids, count, events_gen, ticks_gen)?;
    response(&ctrl, id, start)?;
    let mut results = Vec::new();
    for (ring, mut child, mut out) in members {
        let mut rest = String::new();
        std::io::Read::read_to_string(&mut out, &mut rest)?;
        print!("{rest}");
        let status = child.wait()?;
        results.push((ring, status.success(), rest));
    }
    let r = command(&mut ctrl, &mut ids, TICK_REPORT, 0, 0, &[])?;
    let field = |text: &str, name: &str| -> Option<u32> {
        text.split_whitespace()
            .find_map(|w| w.strip_prefix(name)?.strip_prefix('=')?.parse().ok())
    };
    let events_got = field(&results[0].2, "n").unwrap_or(0);
    let ticks_got = field(&results[1].2, "n").unwrap_or(0);
    let missing = (count - events_got.min(count)) + (r[8] - ticks_got.min(r[8]));
    let ok = results.iter().all(|(_, ok, _)| *ok) && missing == 0 && r[10] == 0;
    println!(
        "IRQPAIR events={events_got} ticks={ticks_got} ticks_sent={} ticks_full={} missing={missing} \
         seconds={:.3}",
        r[8],
        r[9],
        start.elapsed().as_secs_f64()
    );
    std::io::stdout().flush()?;
    if ok {
        println!("IRQPAIR_PASS");
        Ok(())
    } else {
        println!("IRQPAIR_FAIL");
        Err("irq-pair failed".into())
    }
}

/// `pair-member RING GENERATION COUNT`: one side of `irq-pair`. Opens the ring (its own
/// `rp1:irq` handle), says `PAIR_READY`, receives until COUNT messages or 2 s without one.
pub fn pair_member(mut args: impl Iterator<Item = String>) -> Result<()> {
    let ring = args.next().ok_or("ring name")?;
    let generation: u32 = args.next().ok_or("generation")?.parse()?;
    let count: u32 = args.next().ok_or("count")?.parse()?;
    let fw = Firmware::open()?;
    fw.wait_ready(READY)?;
    let mut rx = fw.ring_rx::<Message>(&ring)?;
    drop(fw);
    println!("PAIR_READY");
    std::io::stdout().flush()?;
    let mut checker = Checker::new(generation);
    let mut buf = [[0u32; 4]; 32];
    let mut last = Instant::now();
    let start = Instant::now();
    while checker.received + checker.corrupt < count
        && last.elapsed() < Duration::from_secs(2)
        && start.elapsed() < LIMIT
    {
        let n = rx.recv_into(&mut buf, Some(Duration::from_millis(500)))?;
        for m in &buf[..n] {
            checker.check(m);
        }
        if n != 0 {
            last = Instant::now();
        }
    }
    let s = rx.wait_stats();
    println!(
        "PAIR_MEMBER ring={ring} n={} corrupt={} stale={} sleeps={} woken={} late={}",
        checker.received, checker.corrupt, checker.stale, s.sleeps, s.woken, s.late
    );
    if checker.corrupt == 0 {
        Ok(())
    } else {
        Err("pair member: corrupt messages".into())
    }
}
