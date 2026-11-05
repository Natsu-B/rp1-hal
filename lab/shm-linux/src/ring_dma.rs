//! Ring DMA production and ordering controls. Checks newest published slot first, then
//! consumes with an explicit error report. Counts each offending logical slot once.
//! Stale/corrupt records only its first observed violation kind, not a reliable breakdown;
//! only violation presence is meaningful for the ordering check.
use crate::{Control, response, sample};
use rp1_linux::{BlockMap, Firmware, Irq, Result, RingRx};
use rp1_proto::Error;
use std::time::{Duration, Instant};

/// Same as lab/shm-firmware `ring_dma::wide`.
fn wide<const W: usize>(i: u32) -> [u32; W] {
    let mut m = [0u32; W];
    for (k, w) in m.iter_mut().enumerate() {
        *w = i ^ (k as u32).wrapping_mul(0x9e37_79b9);
    }
    m[W - 1] = !i;
    m
}
fn message<const W: usize>(i: u32) -> [u32; W] {
    if W == 4 {
        let s = sample(i);
        let mut m = [0u32; W];
        m.copy_from_slice(&s[..W]);
        m
    } else {
        wide::<W>(i)
    }
}
fn cpu_seconds() -> f64 {
    let mut u: libc::rusage = unsafe { std::mem::zeroed() };
    unsafe { libc::getrusage(libc::RUSAGE_SELF, &mut u) };
    let t = |v: libc::timeval| v.tv_sec as f64 + v.tv_usec as f64 / 1e6;
    t(u.ru_utime) + t(u.ru_stime)
}
fn request(ctrl: &mut BlockMap<Control>, id: u32, count: u32, ext: [u32; 5]) -> Result<Instant> {
    let mut words = [0; 16];
    words[..5].copy_from_slice(&[id, 17, count, 0, 1]);
    words[8..13].copy_from_slice(&ext);
    let mut writer = unsafe { ctrl.begin_write() }?;
    writer.write_words(0, &words)?;
    writer.commit();
    Ok(Instant::now())
}

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Judged,
    Thresh,
    Order,
}
#[derive(Clone, Copy)]
struct Run {
    name: &'static str,
    mode: u32,
    irq: bool,
    batch: u32,
    gap_us: u32,
    count: Option<u32>,
    must_sleep: bool,
    kind: Kind,
}
const MODES: [&str; 5] = ["cpu", "dma", "auto", "early-head", "bit30-zero"];
struct Res {
    got: u32,
    stale: u64,
    corrupt: u32,
    seconds: f64,
    report: Control,
}

#[allow(clippy::too_many_arguments)]
fn one<const W: usize>(
    rx: &mut RingRx<[u32; W]>,
    ctrl: &mut BlockMap<Control>,
    irq: &mut Irq,
    ring: u32,
    id: u32,
    run: &Run,
    count: u32,
) -> Result<Res> {
    let first = rx.tail();
    let start = request(
        ctrl,
        id,
        count,
        [ring, run.mode, run.irq as u32, run.batch, run.gap_us],
    )?;
    let mut buf = vec![[0u32; W]; 64];
    let (mut got, mut corrupt, mut stale) = (0u32, 0u32, 0u64);
    let mut seen = Seen::default();
    while got < count {
        // Inspect the newest committed slot before traversing the older prefix.
        if let Some((index, value)) = rx.newest()? {
            match value {
                Ok(m) if m != message::<W>(index.wrapping_sub(first)) => {
                    if seen.first(index) {
                        corrupt += 1;
                    }
                }
                Err(Error::Corrupt | Error::Invalid) => {
                    if seen.first(index) {
                        stale += 1;
                    }
                }
                Err(e) => return Err(e.into()),
                _ => {}
            }
        }
        let (n, error) = rx.pop_into_report(&mut buf);
        match error {
            Some(Error::Corrupt | Error::Invalid) => {
                if seen.first(rx.tail()) {
                    stale += 1;
                }
            }
            Some(e) => return Err(e.into()),
            None => {}
        }
        for m in &buf[..n] {
            if *m != message::<W>(got) {
                if corrupt == 0 {
                    eprintln!(
                        "RINGDMA_CORRUPT name={} at={got} got={:x?}",
                        run.name,
                        &m[..]
                    );
                }
                if seen.first(first.wrapping_add(got)) {
                    corrupt += 1;
                }
            }
            got += 1;
        }
        if n == 0 && run.irq && stale == 0 {
            rx.wait(irq, Some(Duration::from_secs(1)))?;
        }
        if start.elapsed() > Duration::from_secs(180) {
            break;
        }
    }
    let seconds = start.elapsed().as_secs_f64();
    let report = response(ctrl, id, start)?;
    Ok(Res {
        got,
        stale,
        corrupt,
        seconds,
        report,
    })
}

pub fn main(count: u32, ordering_only: bool, isolate_mode: Option<u32>) -> Result<()> {
    let mut fw = Firmware::open()?;
    let mut events = fw.ring_rx::<[u32; 4]>("events")?;
    let mut w64 = fw.ring_rx::<[u32; 16]>("wide64")?;
    let mut w256 = fw.ring_rx::<[u32; 64]>("wide256")?;
    let mut ctrl = fw.block::<Control>("control")?;
    drop(fw);
    let mut irq = Irq::open()?;
    let r = |name, mode, irq, batch, kind| Run {
        name,
        mode,
        irq,
        batch,
        gap_us: 0,
        count: None,
        must_sleep: false,
        kind,
    };
    use Kind::*;
    // (ring, max batch, runs)
    let mut plan: Vec<(u32, Run)> = Vec::new();
    for (ring, b) in [(0u32, 32u32), (1, 32), (2, 8)] {
        plan.push((ring, r("cpu-poll", 0, false, b, Judged)));
        plan.push((ring, r("dma-poll", 1, false, b, Judged)));
        plan.push((ring, r("cpu-irq", 0, true, b, Judged)));
        plan.push((ring, r("dma-irq", 1, true, b, Judged)));
    }
    plan.push((
        0,
        Run {
            gap_us: 100,
            must_sleep: true,
            ..r("dma-paced-irq", 1, true, 32, Judged)
        },
    ));
    // Default threshold: a single 16 B message stays on the CPU, a 32-message batch uses DMA.
    plan.push((0, r("auto-single", 2, true, 1, Judged)));
    plan.push((0, r("auto-burst", 2, true, 32, Judged)));
    for (ring, batches) in [
        (0u32, &[1u32, 2, 3, 4, 6, 8, 12, 16, 32][..]),
        (1, &[1, 2, 4][..]),
        (2, &[1, 2][..]),
    ] {
        for &b in batches {
            for mode in [0, 1] {
                plan.push((
                    ring,
                    Run {
                        count: Some(32_000.min(count)),
                        ..r("thresh", mode, false, b, Thresh)
                    },
                ));
            }
        }
    }
    for (ring, b) in [(0u32, 32u32), (1, 32), (2, 8)] {
        for mode in [3, 4] {
            plan.push((
                ring,
                Run {
                    count: Some(32_000.min(count)),
                    ..r(
                        if mode == 3 {
                            "positive-control"
                        } else {
                            "bit30-comparison"
                        },
                        mode,
                        false,
                        b,
                        Order,
                    )
                },
            ));
        }
    }
    if ordering_only {
        plan.retain(|(_, run)| run.kind == Order);
        for (ring, b) in [(0, 31), (1, 31), (2, 7)] {
            plan.insert(
                ring as usize,
                (ring, r("production-order", 1, false, b, Judged)),
            );
        }
    }
    if let Some(mode) = isolate_mode {
        plan.clear();
        plan.push((2, r("isolate-wide256", mode, false, 7, Judged)));
    }
    let names = ["events", "wide64", "wide256"];
    let sizes = [16.0, 64.0, 256.0];
    let mut failed = Vec::new();
    let mut thresh: Vec<(u32, u32, u32, f64)> = Vec::new();
    for (ring, run) in &plan {
        // Firmware ignores a repeated ID, including across separate client invocations.
        let id = ctrl.read()?[0].wrapping_add(1).max(1);
        let n = run.count.unwrap_or(count);
        let ring = *ring;
        println!(
            "RINGDMA_BEGIN name={} ring={} mode={} n={n}",
            run.name, names[ring as usize], MODES[run.mode as usize]
        );
        let (stats0, wakes0, cpu0) = (
            match ring {
                0 => events.wait_stats(),
                1 => w64.wait_stats(),
                _ => w256.wait_stats(),
            },
            irq.wakes(),
            cpu_seconds(),
        );
        let res = match ring {
            0 => one(&mut events, &mut ctrl, &mut irq, ring, id, run, n)?,
            1 => one(&mut w64, &mut ctrl, &mut irq, ring, id, run, n)?,
            _ => one(&mut w256, &mut ctrl, &mut irq, ring, id, run, n)?,
        };
        let s = match ring {
            0 => events.wait_stats(),
            1 => w64.wait_stats(),
            _ => w256.wait_stats(),
        };
        let (sleeps, woken, late) = (
            s.sleeps - stats0.sleeps,
            s.woken - stats0.woken,
            s.late - stats0.late,
        );
        let cpu = cpu_seconds() - cpu0;
        let rep = &res.report;
        let (pulses, ctl_l, wraps, dma_b, cpu_b) = (rep[7], rep[8], rep[9], rep[10], rep[11]);
        let (gen_us, push_us, notify_us) = (rep[12], rep[13], rep[14]);
        // Payload rate of the ring API alone (RP1 time inside push_slice / RingDma::push_slice).
        let ring_mbps = n as f64 * sizes[ring as usize] / push_us.max(1) as f64;
        let missing = n - res.got;
        let mbps = n as f64 * sizes[ring as usize] / res.seconds / 1e6;
        let path_ok = match run.mode {
            0 => dma_b == 0 && cpu_b > 0,
            1 | 3 | 4 => cpu_b == 0 && dma_b > 0,
            _ if run.batch == 1 => dma_b == 0,
            _ => dma_b > 0,
        };
        let check = if missing != 0 {
            "FAIL(missing)"
        } else if run.kind != Order && (res.corrupt != 0 || res.stale != 0) {
            "FAIL(messages)"
        } else if rep[5] != 0 {
            "FAIL(firmware)"
        } else if !path_ok {
            "FAIL(path)"
        } else if run.must_sleep && sleeps == 0 {
            "INCONCLUSIVE(never-slept)"
        } else if run.irq && sleeps > 0 && (woken == 0 || late * 100 > sleeps) {
            "FAIL(not-notified)"
        } else {
            "ok"
        };
        let line = format!(
            "name={} ring={} mode={} irq={} batch={} gap_us={} n={} missing={missing} \
             corrupt={} stale={} seconds={:.6} MBps={mbps:.3} cpu_pct={:.1} rp1_us={} \
             pulses={pulses} wakes={} sleeps={sleeps} woken={woken} late={late} \
             gen_us={gen_us} push_us={push_us} notify_us={notify_us} ring_MBps={ring_mbps:.3} \
             ctl_l=0x{ctl_l:08x} wraps={wraps} dma_batches={dma_b} cpu_batches={cpu_b}",
            run.name,
            names[ring as usize],
            MODES[run.mode as usize],
            run.irq,
            run.batch,
            run.gap_us,
            res.got,
            res.corrupt,
            res.stale,
            res.seconds,
            100.0 * cpu / res.seconds,
            rep[6],
            irq.wakes() - wakes0,
        );
        match run.kind {
            Judged | Thresh => println!("RINGDMA {line} check={check}"),
            Order => println!(
                "RINGDMA_ORDER {line} result={}",
                if missing == 0 && res.corrupt == 0 && res.stale == 0 {
                    "no-violation"
                } else {
                    "VIOLATION"
                }
            ),
        }
        if check != "ok" || run.mode == 3 && res.corrupt == 0 && res.stale == 0 {
            failed.push(format!("{}/{}", names[ring as usize], run.name));
        }
        if run.kind == Thresh {
            thresh.push((
                ring,
                run.batch,
                run.mode,
                // RP1 µs per batch inside the ring API (generation is the same on both paths).
                push_us as f64 / (cpu_b + dma_b).max(1) as f64,
            ));
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    for t in thresh.chunks(2) {
        if let [c, d] = t {
            println!(
                "RINGDMA_THRESH ring={} batch={} bytes={} cpu_us_per_batch={:.3} dma_us_per_batch={:.3} faster={}",
                names[c.0 as usize],
                c.1,
                c.1 as usize * rp1_proto::shm::slot_size(sizes[c.0 as usize] as usize),
                c.3,
                d.3,
                if d.3 < c.3 { "dma" } else { "cpu" }
            );
        }
    }
    if !failed.is_empty() {
        println!("RINGDMA_FAIL runs={failed:?}");
        return Err("ring-dma check failed".into());
    }
    println!("RINGDMA_PASS count={count}");
    Ok(())
}

// Requires ring depths <=256: lab/shm-firmware declares events/wide64=256, wide256=128.
// Revisit this bound before adding deeper rings. A logical index occupies one physical slot
// until consumed; remembering its last violation deduplicates newest/prefix/retries across
// kinds. The first observed kind wins; stale/corrupt counts are not a reliable breakdown.
struct Seen([Option<u32>; 256]);
impl Default for Seen {
    fn default() -> Self {
        Self([None; 256])
    }
}
impl Seen {
    fn first(&mut self, index: u32) -> bool {
        let entry = &mut self.0[(index & 255) as usize];
        if *entry == Some(index) {
            false
        } else {
            *entry = Some(index);
            true
        }
    }
}
#[cfg(test)]
mod tests {
    use super::Seen;
    #[test]
    fn repeated_observations_count_once_and_reused_slot_counts_again() {
        let mut s = Seen::default();
        assert!(s.first(1));
        assert!(s.first(0)); // newest is observed before prefix
        assert!(!s.first(1)); // prefix/retry does not lose or double count it
        assert!(s.first(257));
        assert!(!s.first(257));
    }
}
