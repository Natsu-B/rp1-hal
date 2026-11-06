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
fn request(ctrl: &mut BlockMap<Control>, id: u32, count: u32, ext: [u32; 6]) -> Result<Instant> {
    let mut words = [0; 16];
    words[..5].copy_from_slice(&[id, 17, count, 0, 1]);
    words[8..14].copy_from_slice(&ext);
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
    timed: bool,
    odd_head: bool,
}
const MODES: [&str; 8] = [
    "cpu",
    "dma",
    "auto",
    "early-head",
    "bit30-zero",
    "generate-dma",
    "generate-cpu",
    "generate-auto",
];
struct Res {
    got: u32,
    stale: u64,
    corrupt: u32,
    seconds: f64,
    report: Control,
    perf: [u32; 8],
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
        [
            ring,
            run.mode,
            run.irq as u32,
            run.batch,
            run.gap_us,
            run.timed as u32,
        ],
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
    let mut perf = [0; 8];
    if run.timed {
        let perf_id = id.wrapping_add(1).max(1);
        let mut words = [0; 16];
        words[..5].copy_from_slice(&[perf_id, 18, 0, 0, 1]);
        let mut writer = unsafe { ctrl.begin_write() }?;
        writer.write_words(0, &words)?;
        writer.commit();
        perf.copy_from_slice(&response(ctrl, perf_id, Instant::now())?[8..16]);
    }
    Ok(Res {
        perf,
        got,
        stale,
        corrupt,
        seconds,
        report,
    })
}

pub fn main(count: u32, ordering_only: bool, isolate_mode: Option<u32>) -> Result<()> {
    let r = |name, mode, irq, batch, kind| Run {
        name,
        mode,
        irq,
        batch,
        gap_us: 0,
        count: None,
        must_sleep: false,
        kind,
        timed: false,
        odd_head: false,
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
    for (ring, old, large) in [(0, 32, 128), (1, 32, 56), (2, 8, 15)] {
        for b in [old, large] {
            for mode in [0, 1, 5, 6] {
                plan.push((ring, r("speed-poll", mode, false, b, Judged)));
            }
        }
        plan.push((
            ring,
            Run {
                gap_us: 100,
                must_sleep: true,
                ..r("generated-paced-irq", 5, true, large, Judged)
            },
        ));
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
        for (ring, b) in [(0, 31), (1, 31), (2, 7), (0, 127), (1, 55), (2, 15)] {
            plan.push((ring, r("production-order", 5, false, b, Judged)));
        }
    }
    if let Some(mode) = isolate_mode {
        plan.clear();
        plan.push((2, r("isolate-wide256", mode, false, 7, Judged)));
    }
    execute(count, plan)
}

/// Reproducible throughput plan; flags allow the same client to measure baseline firmware.
pub fn perf(args: impl Iterator<Item = String>) -> Result<()> {
    let options = PerfOptions::parse(args)?;
    execute(options.count, options.plan())
}

struct PerfOptions {
    count: u32,
    warmup: u32,
    repeat: u32,
    timed: bool,
    baseline: bool,
    ring: Option<u32>,
    batch: Option<u32>,
    mode: Option<u32>,
    irq: bool,
    gap_us: u32,
    odd_head: bool,
}
impl PerfOptions {
    fn parse(mut args: impl Iterator<Item = String>) -> Result<Self> {
        let mut o = Self {
            count: 1_000_000,
            warmup: 100_000,
            repeat: 5,
            timed: false,
            baseline: false,
            ring: None,
            batch: None,
            mode: None,
            irq: false,
            gap_us: 0,
            odd_head: false,
        };
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--timed" => o.timed = true,
                "--baseline" => o.baseline = true,
                "--irq" => o.irq = true,
                "--odd-head" => o.odd_head = true,
                "--count" | "--warmup" | "--repeat" | "--ring" | "--batch" | "--mode"
                | "--gap-us" => {
                    let n = args.next().ok_or("missing flag value")?.parse()?;
                    match arg.as_str() {
                        "--count" => o.count = n,
                        "--warmup" => o.warmup = n,
                        "--repeat" => o.repeat = n,
                        "--ring" => o.ring = Some(n),
                        "--batch" => o.batch = Some(n),
                        "--mode" => o.mode = Some(n),
                        _ => o.gap_us = n,
                    }
                }
                _ => return Err(format!("unknown ring-dma-perf option: {arg}").into()),
            }
        }
        if o.count == 0
            || o.repeat == 0
            || o.ring.is_some_and(|r| r > 2)
            || o.mode.is_some_and(|m| ![0, 1, 2, 5, 6, 7].contains(&m))
            || o.baseline && (o.timed || o.mode.is_some_and(|m| m > 2))
        {
            return Err(
                "invalid ring-dma-perf options (baseline supports modes 0/1/2 without timing)"
                    .into(),
            );
        }
        for ring in 0..3 {
            if o.ring.is_some_and(|r| r != ring) {
                continue;
            }
            let cap = if o.baseline {
                [32, 32, 8]
            } else {
                [170, 56, 15]
            }[ring as usize];
            if o.batch.is_some_and(|b| b == 0 || b > cap) {
                return Err(
                    format!("batch exceeds firmware capacity {cap} for ring {ring}").into(),
                );
            }
        }
        Ok(o)
    }
    fn plan(&self) -> Vec<(u32, Run)> {
        let mut plan = Vec::new();
        for (ring, batches) in [
            (0, &[1, 2, 4, 8, 12, 16, 31, 32, 64, 127, 128, 170][..]),
            (1, &[1, 2, 4, 8, 12, 16, 31, 32, 48, 55, 56][..]),
            (2, &[1, 2, 4, 7, 8, 12, 14, 15][..]),
        ] {
            if self.ring.is_some_and(|r| r != ring) {
                continue;
            }
            let candidates = if self.batch.is_some() {
                self.batch.as_slice()
            } else {
                batches
            };
            for &batch in candidates {
                if self.baseline && batch > [32, 32, 8][ring as usize] {
                    continue;
                }
                let modes = self.mode.as_slice();
                let modes = if modes.is_empty() {
                    if self.baseline {
                        &[0, 1][..]
                    } else {
                        &[0, 1, 5, 6][..]
                    }
                } else {
                    modes
                };
                for &mode in modes {
                    let run = Run {
                        name: "measure",
                        mode,
                        irq: self.irq,
                        batch,
                        gap_us: self.gap_us,
                        count: Some(self.count),
                        must_sleep: self.irq && self.gap_us > 0,
                        kind: Kind::Judged,
                        timed: self.timed,
                        odd_head: self.odd_head,
                    };
                    if self.warmup > 0 {
                        plan.push((
                            ring,
                            Run {
                                name: "warmup",
                                count: Some(self.warmup),
                                ..run
                            },
                        ));
                    }
                    for _ in 0..self.repeat {
                        plan.push((ring, run));
                    }
                }
            }
        }
        plan
    }
}

fn execute(count: u32, plan: Vec<(u32, Run)>) -> Result<()> {
    use Kind::*;
    let mut fw = Firmware::open()?;
    let mut events = fw.ring_rx::<[u32; 4]>("events")?;
    let mut w64 = fw.ring_rx::<[u32; 16]>("wide64")?;
    let mut w256 = fw.ring_rx::<[u32; 64]>("wide256")?;
    let mut ctrl = fw.block::<Control>("control")?;
    drop(fw);
    let mut irq = Irq::open()?;
    let names = ["events", "wide64", "wide256"];
    let sizes = [16.0, 64.0, 256.0];
    let mut failed = Vec::new();
    let mut thresh: Vec<(u32, u32, u32, f64)> = Vec::new();
    for (ring, run) in &plan {
        let tail = match *ring {
            0 => events.tail(),
            1 => w64.tail(),
            _ => w256.tail(),
        };
        if run.odd_head && tail.is_multiple_of(2) {
            let id = ctrl.read()?[0].wrapping_add(1).max(1);
            let prime = Run {
                mode: 0,
                irq: false,
                batch: 1,
                gap_us: 0,
                timed: false,
                ..*run
            };
            let res = match *ring {
                0 => one(&mut events, &mut ctrl, &mut irq, 0, id, &prime, 1)?,
                1 => one(&mut w64, &mut ctrl, &mut irq, 1, id, &prime, 1)?,
                _ => one(&mut w256, &mut ctrl, &mut irq, 2, id, &prime, 1)?,
            };
            if res.got != 1 || res.corrupt != 0 || res.stale != 0 || res.report[5] != 0 {
                return Err("odd-head priming failed".into());
            }
        }
        // Firmware ignores a repeated ID, including across separate client invocations.
        let id = ctrl.read()?[0].wrapping_add(1).max(1);
        let n = run.count.unwrap_or(count);
        let ring = *ring;
        println!(
            "RINGDMA_PERF_BEGIN run_id={id} name={} ring={} mode={} size={} batch={} timed={} count={n} start_head={}",
            run.name,
            names[ring as usize],
            MODES[run.mode as usize],
            sizes[ring as usize],
            run.batch,
            run.timed,
            match ring {
                0 => events.tail(),
                1 => w64.tail(),
                _ => w256.tail(),
            }
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
            0 | 6 => dma_b == 0 && cpu_b > 0,
            1 | 3 | 4 | 5 => cpu_b == 0 && dma_b > 0,
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
        } else if run.must_sleep && late != 0 {
            "FAIL(late-wake)"
        } else if run.must_sleep && sleeps == 0 {
            "INCONCLUSIVE(never-slept)"
        } else if run.irq && sleeps > 0 && (woken == 0 || late * 100 > sleeps) {
            "FAIL(not-notified)"
        } else {
            "ok"
        };
        let [
            copy_us,
            prepare_us,
            publish_us,
            blocked_us,
            blocks,
            two_block_batches,
            width8,
            width16,
        ] = res.perf;
        let total_us = rep[6];
        let rp1_us_per_message = total_us as f64 / n.max(1) as f64;
        let line = format!(
            "run_id={id} name={} ring={} mode={} irq={} batch={} gap_us={} got={} missing={missing} \
             corrupt={} stale={} seconds={:.6} MBps={mbps:.3} cpu_pct={:.1} rp1_us={} \
             pulses={pulses} wakes={} sleeps={sleeps} woken={woken} late={late} \
             gen_us={gen_us} push_us={push_us} notify_us={notify_us} ring_MBps={ring_mbps:.3} \
             ctl_l=0x{ctl_l:08x} wraps={wraps} dma_batches={dma_b} cpu_batches={cpu_b} \
             timed={} total_us={total_us} rp1_us_per_message={rp1_us_per_message:.4} \
             copy_us={copy_us} prepare_us={prepare_us} publish_us={publish_us} blocked_us={blocked_us} \
             blocks={blocks} two_block_batches={two_block_batches} width8={width8} width16={width16} errors={}",
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
            run.timed,
            rep[5],
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
        let passed = check == "ok" && (run.mode != 3 || res.corrupt != 0 || res.stale != 0);
        println!(
            "RINGDMA_PERF_END run_id={id} status={}",
            if passed { "PASS" } else { "FAIL" }
        );
        if !passed {
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
    fn perf_plan_preserves_baseline_and_repeats_identical_conditions() {
        let parse = |args: &[&str]| super::PerfOptions::parse(args.iter().map(|s| s.to_string()));
        let o = parse(&["--ring", "0", "--batch", "128", "--mode", "5", "--odd-head"]).unwrap();
        let plan = o.plan();
        assert_eq!(plan.len(), 6);
        assert_eq!(plan[0].1.count, Some(100_000));
        for (ring, run) in &plan[1..] {
            assert_eq!(
                (*ring, run.mode, run.batch, run.count),
                (0, 5, 128, Some(1_000_000))
            );
            assert!(run.odd_head);
        }
        let base = parse(&["--baseline"]).unwrap().plan();
        assert!(
            base.iter().all(|(r, run)| run.mode <= 1
                && run.batch <= [32, 32, 8][*r as usize]
                && !run.timed)
        );
        for args in [
            &["--count", "0"][..],
            &["--repeat", "0"],
            &["--mode", "3"],
            &["--baseline", "--timed"],
            &["--ring", "2", "--batch", "16"],
            &["--wat"],
        ] {
            assert!(parse(args).is_err());
        }
    }

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
