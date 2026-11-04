//! `shm-linux ring-irq [count]`: the `events` ring (notify = "irq") against a polling consumer.
//! Each run streams `count` messages from RP1 (command 14 notifies after every batch with
//! `HostIrq::notify_ring`, 15 does not) and checks every one. The consumer either busy-polls
//! or pops until empty and then sleeps in `RingRx::wait` on `rp1:irq`. Prints one `RINGIRQ`
//! line per run (throughput, consumer CPU from getrusage, wakes) and `RINGIRQ_PASS` at the end.
use crate::{Control, expired, request, response, sample};
use rp1_linux::{Firmware, Irq, Result};
use std::time::Duration;

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
}

pub fn main(count: u32) -> Result<()> {
    let mut fw = Firmware::open()?;
    let mut events = fw.ring_rx::<[u32; 4]>("events")?;
    let mut ctrl = fw.block::<Control>("control")?;
    drop(fw);
    let mut irq = Irq::open()?;
    let runs = [
        Run {
            name: "burst-poll",
            command: 15,
            batch: 32,
            gap_us: 0,
            irq: false,
        },
        Run {
            name: "burst-irq",
            command: 14,
            batch: 32,
            gap_us: 0,
            irq: true,
        },
        Run {
            name: "paced-poll",
            command: 15,
            batch: 32,
            gap_us: 100,
            irq: false,
        },
        Run {
            name: "paced-irq",
            command: 14,
            batch: 32,
            gap_us: 100,
            irq: true,
        },
        Run {
            name: "single-poll",
            command: 15,
            batch: 1,
            gap_us: 0,
            irq: false,
        },
        Run {
            name: "single-irq",
            command: 14,
            batch: 1,
            gap_us: 0,
            irq: true,
        },
    ];
    let mut buf = [[0u32; 4]; 32];
    for (k, run) in runs.iter().enumerate() {
        // Ids above the regression's (1..=13): proc1 ignores a repeated id.
        let id = 100 + k as u32;
        let stats0 = events.wait_stats();
        let wakes0 = irq.wakes();
        let cpu0 = cpu_seconds();
        let start = request(
            &mut ctrl,
            id,
            run.command,
            count,
            run.batch << 16 | run.gap_us,
        )?;
        let mut got = 0;
        let mut polls_empty = 0u64;
        while got < count {
            let n = events.pop_into(&mut buf)?;
            for s in &buf[..n] {
                if *s != sample(got) {
                    return Err(format!("{} corrupt at {got}: {s:?}", run.name).into());
                }
                got += 1;
            }
            if n == 0 {
                if run.irq {
                    events.wait(&mut irq, Some(Duration::from_secs(1)))?;
                } else {
                    polls_empty += 1;
                }
            }
            expired(start)?;
        }
        let seconds = start.elapsed().as_secs_f64();
        let cpu = cpu_seconds() - cpu0;
        let report = response(&ctrl, id, start)?;
        let s = events.wait_stats();
        let wakes = irq.wakes() - wakes0;
        println!(
            "RINGIRQ name={} n={got} missing=0 corrupt=0 batch={} gap_us={} seconds={seconds:.6} \
             MBps={:.3} cpu_s={cpu:.3} cpu_pct={:.1} rp1_us={} pulses={} wakes={wakes} \
             wakes_per_msg={:.6} sleeps={} woken={} rechecks={} late={} empty_polls={polls_empty} \
             uio_events={}",
            run.name,
            run.batch,
            run.gap_us,
            count as f64 * 16.0 / seconds / 1e6,
            100.0 * cpu / seconds,
            report[6],
            report[7],
            wakes as f64 / count as f64,
            s.sleeps - stats0.sleeps,
            s.woken - stats0.woken,
            s.rechecks - stats0.rechecks,
            s.late - stats0.late,
            irq.event_count(),
        );
        // Let a trailing pulse land before the next run's counters start.
        std::thread::sleep(Duration::from_millis(50));
    }
    println!("RINGIRQ_PASS count={count}");
    Ok(())
}
