//! `shm-linux ring-irq [count]`: the `events` ring (notify = "irq") against a polling consumer.
//! Each run streams `count` messages from RP1 (command 14 notifies after every batch with
//! `HostIrq::push_slice_notify`, 15 does not) and checks every one. The consumer either
//! busy-polls or pops until empty and then sleeps in `RingRx::wait` on `rp1:irq`. Prints one
//! `RINGIRQ` line per run (measured missing/corrupt, throughput, consumer CPU from getrusage,
//! wakes, `check=`) and `RINGIRQ_PASS` only if every run's check holds:
//!
//! - every run: no missing or corrupt message;
//! - notified irq runs: the sleeps are ended by interrupts, not by the 10 ms re-check
//!   (`woken > 0`, `late` at most 1 % of the sleeps), so a broken notification path fails
//!   even though the re-check alone would still deliver every message;
//! - `suppressed-irq` (command 15 with an irq consumer, a short paced stream): no pulses, the
//!   data is found by the re-check (`late > 0`), and no single wait exceeds
//!   `RECHECK + 40 ms` (bounded recovery; 40 ms is scheduling slack).
use crate::{Control, request, response, sample};
use rp1_linux::{Firmware, Irq, RECHECK, Result};
use std::time::{Duration, Instant};

fn cpu_seconds() -> f64 {
    let mut u: libc::rusage = unsafe { std::mem::zeroed() };
    unsafe { libc::getrusage(libc::RUSAGE_SELF, &mut u) };
    let t = |v: libc::timeval| v.tv_sec as f64 + v.tv_usec as f64 / 1e6;
    t(u.ru_utime) + t(u.ru_stime)
}

struct Run {
    name: &'static str,
    command: u32,
    batch: u32,
    gap_us: u32,
    irq: bool,
    /// `RingRx::set_spin` before arming (irq runs).
    spin_us: u64,
    /// Messages in this run (`None`: the `count` argument).
    count: Option<u32>,
}

/// Suppressed-notification run: 100 batches 1 ms apart, each found by the re-check.
const SUPPRESSED_COUNT: u32 = 3200;

pub fn main(count: u32) -> Result<()> {
    let mut fw = Firmware::open()?;
    let mut events = fw.ring_rx::<[u32; 4]>("events")?;
    let mut ctrl = fw.block::<Control>("control")?;
    drop(fw);
    let mut irq = Irq::open()?;
    // (name, command, batch, gap_us, irq, spin_us)
    let run = |name, command, batch, gap_us, irq, spin_us| Run {
        name,
        command,
        batch,
        gap_us,
        irq,
        spin_us,
        count: None,
    };
    let runs = [
        run("burst-poll", 15, 32, 0, false, 0),
        run("burst-irq", 14, 32, 0, true, 0),
        run("paced-poll", 15, 32, 100, false, 0),
        run("paced-irq", 14, 32, 100, true, 0),
        run("paced-irq-spin20", 14, 32, 100, true, 20),
        run("single-poll", 15, 1, 0, false, 0),
        run("single-irq", 14, 1, 0, true, 0),
        run("single-irq-spin20", 14, 1, 0, true, 20),
        Run {
            count: Some(SUPPRESSED_COUNT.min(count)),
            ..run("suppressed-irq", 15, 32, 1000, true, 0)
        },
    ];
    let mut failed = Vec::new();
    let mut buf = [[0u32; 4]; 32];
    for (k, run) in runs.iter().enumerate() {
        // Ids above the regression's (1..=13): proc1 ignores a repeated id.
        let id = 100 + k as u32;
        events.set_spin(Duration::from_micros(run.spin_us));
        let stats0 = events.wait_stats();
        let wakes0 = irq.wakes();
        let cpu0 = cpu_seconds();
        let count = run.count.unwrap_or(count);
        let start = request(
            &mut ctrl,
            id,
            run.command,
            count,
            run.batch << 16 | run.gap_us,
        )?;
        let (mut got, mut corrupt) = (0u32, 0u32);
        let mut polls_empty = 0u64;
        let mut max_wait = Duration::ZERO;
        while got < count {
            let n = events.pop_into(&mut buf)?;
            for s in &buf[..n] {
                if *s != sample(got) {
                    if corrupt == 0 {
                        eprintln!("RINGIRQ_CORRUPT name={} at={got} got={s:?}", run.name);
                    }
                    corrupt += 1;
                }
                got += 1;
            }
            if n == 0 {
                if run.irq {
                    let t = Instant::now();
                    events.wait(&mut irq, Some(Duration::from_secs(1)))?;
                    max_wait = max_wait.max(t.elapsed());
                } else {
                    polls_empty += 1;
                }
            }
            if start.elapsed() > Duration::from_secs(180) {
                break;
            }
        }
        let missing = count - got;
        let seconds = start.elapsed().as_secs_f64();
        let cpu = cpu_seconds() - cpu0;
        let report = response(&ctrl, id, start)?;
        let s = events.wait_stats();
        let (sleeps, woken, late) = (
            s.sleeps - stats0.sleeps,
            s.woken - stats0.woken,
            s.late - stats0.late,
        );
        let wakes = irq.wakes() - wakes0;
        let pulses = report[7];
        let check = if missing != 0 || corrupt != 0 {
            "FAIL(messages)"
        } else if run.irq && run.command == 14 && sleeps > 0 && (woken == 0 || late * 100 > sleeps)
        {
            "FAIL(not-notified)"
        } else if run.command == 15
            && run.irq
            && (pulses != 0 || late == 0 || max_wait > RECHECK + Duration::from_millis(40))
        {
            "FAIL(recovery)"
        } else {
            "ok"
        };
        if check != "ok" {
            failed.push(run.name);
        }
        println!(
            "RINGIRQ name={} n={got} missing={missing} corrupt={corrupt} batch={} gap_us={} spin_us={} \
             seconds={seconds:.6} MBps={:.3} cpu_s={cpu:.3} cpu_pct={:.1} rp1_us={} pulses={pulses} \
             wakes={wakes} wakes_per_msg={:.6} sleeps={sleeps} woken={woken} rechecks={} late={late} \
             max_wait_us={} empty_polls={polls_empty} uio_events={} check={check}",
            run.name,
            run.batch,
            run.gap_us,
            run.spin_us,
            count as f64 * 16.0 / seconds / 1e6,
            100.0 * cpu / seconds,
            report[6],
            wakes as f64 / count as f64,
            s.rechecks - stats0.rechecks,
            max_wait.as_micros(),
            irq.event_count(),
        );
        // Let a trailing pulse land before the next run's counters start.
        std::thread::sleep(Duration::from_millis(50));
    }
    if !failed.is_empty() {
        println!("RINGIRQ_FAIL runs={failed:?}");
        return Err("ring-irq check failed".into());
    }
    println!("RINGIRQ_PASS count={count}");
    Ok(())
}
