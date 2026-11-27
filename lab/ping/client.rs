use super::{CALL_TIMEOUT, Result, echo, rtt};
use rp1_linux::{CoreState, Error, Firmware, PcieState, PeerDown, ScmiState, ShmState};
use rp1_proto::note::{LINK, Link, Note, SHM_REGION, ShmRegion};
use std::time::{Duration, Instant};

#[path = "commands.rs"]
mod commands;

const ROUNDS: u32 = 5_000;

pub fn run(args: &[String]) -> Result<()> {
    let [mode, step] = args else {
        return Err(
            "usage: linux-ping --test irq|poll|wake ready|rtt|late|panic|stop|idle|backpressure"
                .into(),
        );
    };
    let strict = mode == "wake";
    let irq = match mode.as_str() {
        "irq" | "wake" => true,
        "poll" => false,
        _ => return Err("expected irq, poll or wake".into()),
    };
    check_note(irq, strict)?;
    let fw = Firmware::open()?;
    let generation = fw.wait_ready(Duration::from_secs(10))?;
    if step == "ready" {
        let status = fw.status()?;
        if status.pcie != PcieState::Up
            || !matches!(status.scmi, ScmiState::Serving { .. })
            || status.shm != ShmState::Ready
            || status.shm_generation != generation
            || status.cores.iter().any(|c| c.state != CoreState::Running)
        {
            return Err(format!("unexpected firmware status {status:?}").into());
        }
        println!("PING_READY mode={mode} generation={generation}");
        return Ok(());
    }
    if strict && step == "idle" {
        return idle(&fw);
    }
    if strict && step == "backpressure" {
        return backpressure(&fw);
    }
    let mut client = fw.client("ping")?;
    match step.as_str() {
        "rtt" if strict => {
            let mut nanos = Vec::with_capacity(1_000);
            for value in 0..1_000u32 {
                let start = Instant::now();
                let reply = client.call_until(&value, start + CALL_TIMEOUT)?;
                nanos.push(start.elapsed().as_nanos());
                assert_eq!(reply, value);
            }
            for (index, ns) in nanos.iter().enumerate() {
                println!("PING_SAMPLE seq={index} ns={ns}");
            }
            nanos.sort_unstable();
            println!(
                "PING_RTT mode=wake n=1000 failed=0 p50_ns={} p99_ns={} max_ns={}",
                nanos[499], nanos[989], nanos[999]
            );
        }
        "rtt" => rtt(&mut client, ROUNDS, &format!(" mode={mode}"))?,
        "late" => {
            let before = client.call(&commands::COUNT, CALL_TIMEOUT)?;
            let mut injected = 0;
            for value in 0..ROUNDS {
                if value % 100 == 0 {
                    if !matches!(
                        client.call(&(commands::DELAY | value), Duration::ZERO),
                        Err(Error::Timeout)
                    ) {
                        return Err("delayed ping did not time out".into());
                    }
                    injected += 1;
                }
                echo(&mut client, value)?;
            }
            let replied = client
                .call(&commands::COUNT, CALL_TIMEOUT)?
                .wrapping_sub(before);
            if replied != injected {
                return Err(format!("expected {injected} late replies, got {replied}").into());
            }
            println!(
                "PING_LATE mode={mode} n={ROUNDS} injected={injected} timed_out={injected} replied={replied} wrong=0 failed=0"
            );
        }
        "panic" | "stop" if strict => {
            let panic = step == "panic";
            let (request, expected) = if panic {
                (commands::PANIC, CoreState::Panicked)
            } else {
                (commands::STOP, CoreState::Stopped)
            };
            let start = Instant::now();
            let result = client.call_until(&request, start + Duration::from_millis(10));
            let us = start.elapsed().as_micros();
            let Err(Error::PeerDown(PeerDown::Terminal {
                core: 1,
                state,
                reason,
            })) = result
            else {
                return Err(format!("expected proc1 terminal after {us} us: {result:?}").into());
            };
            assert_eq!(state, expected);
            assert!(if panic {
                (10_000..=15_000).contains(&us)
            } else {
                (1..10_000).contains(&us)
            });
            println!("PING_PEER_DOWN core=1 state={state:?} reason={reason} elapsed_us={us}");
        }
        "panic" if irq => {
            let start = Instant::now();
            let result = client.call(&commands::PANIC, CALL_TIMEOUT);
            let elapsed = start.elapsed();
            if !matches!(
                result,
                Err(Error::PeerDown(PeerDown::Terminal {
                    core: 1,
                    state: CoreState::Panicked,
                    ..
                }))
            ) || elapsed > Duration::from_millis(15)
            {
                return Err(format!("panic detection failed after {elapsed:?}: {result:?}").into());
            }
            println!(
                "PING_PEER_DOWN core=1 state=Panicked elapsed_us={}",
                elapsed.as_micros()
            );
        }
        _ => return Err("invalid test step".into()),
    }
    Ok(())
}

fn check_note(irq: bool, strict: bool) -> Result<()> {
    let bytes = std::fs::read("/sys/firmware/devicetree/base/rp1-firmware/rp1,note")?;
    let note = Note::parse(&bytes).map_err(Error::from)?;
    if (note.version() == rp1_proto::note::LINK_WAKE_VERSION) != strict {
        return Err("NOTE version differs from the test mode".into());
    }

    let link =
        Link::parse(note.get(LINK).ok_or("missing LINK record")?.value).map_err(Error::from)?;
    let regions = || {
        note.records()
            .filter_map(std::result::Result::ok)
            .filter(|r| r.kind == SHM_REGION)
            .map(|r| ShmRegion::parse(r.value).expect("validated region"))
    };
    if !["ping-dn", "ping-up"]
        .iter()
        .all(|name| regions().any(|r| r.name == *name && r.core == 2 && r.notify_irq == irq))
        || (irq
            && (link.cores[1].host_irq_vector != Some(rp1_proto::irq::VECTOR as u8)
                || link.cores[1].doorbell != Some(27)))
    {
        return Err("ping notification or owner differs from the test mode".into());
    }
    Ok(())
}

// The idle and Full recovery probes use the same observations as the original
// channel probe: WaitStats, /proc CPU ticks, and reply ordering.
fn cpu_ticks() -> Result<u64> {
    let stat = std::fs::read_to_string("/proc/self/stat")?;
    let (_, fields) = stat.rsplit_once(") ").ok_or("missing stat fields")?;
    let fields: Vec<_> = fields.split_whitespace().collect();
    Ok(fields[11].parse::<u64>()? + fields[12].parse::<u64>()?)
}

fn irq_count() -> Result<u64> {
    let interrupts = std::fs::read_to_string("/proc/interrupts")?;
    let line = interrupts
        .lines()
        .find(|line| line.split_whitespace().last() == Some("rp1:irq"))
        .ok_or("rp1:irq absent from /proc/interrupts")?;
    Ok(line
        .split_whitespace()
        .skip(1)
        .map_while(|field| field.parse::<u64>().ok())
        .sum())
}

fn idle(fw: &Firmware) -> Result<()> {
    let (mut tx, mut rx) = fw.channel::<u32, u32>("channel")?.split();
    assert!(rx.try_recv()?.is_none());
    let ticks = cpu_ticks()?;
    let irq_before = irq_count()?;
    let now = Instant::now();
    let result = rx.wait(now + Duration::from_secs(10));
    let wall_us = now.elapsed().as_micros();
    let irq_after = irq_count()?;
    println!("IDLE_IRQ before={irq_before} after={irq_after}");
    assert_eq!(irq_before, irq_after);
    let ticks = cpu_ticks()? - ticks;
    let stats = rx.wait_stats();
    println!("IDLE_CPU wall_us={wall_us} ticks={ticks} wait_stats={stats:?}");
    assert!(matches!(result, Err(Error::Timeout)));
    assert!((10_000_000..=10_050_000).contains(&wall_us));
    assert_eq!(
        (
            stats.sleeps,
            stats.woken,
            stats.rechecks,
            stats.late,
            stats.deadline_checks
        ),
        (1, 0, 0, 0, 1)
    );
    let nonce = 0xc5b2_0001;
    tx.send(&nonce)?;
    rx.wait(Instant::now() + CALL_TIMEOUT)?;
    assert_eq!(rx.try_recv()?, Some(nonce));
    println!("IDLE_NONCE ok=1");
    Ok(())
}

fn backpressure(fw: &Firmware) -> Result<()> {
    let (mut tx, mut rx) = fw.channel::<u32, u32>("channel")?.split();
    for trial in 1..=12u32 {
        let begin = trial * 16;
        let values = std::array::from_fn::<_, 8, _>(|i| begin + i as u32);
        assert_eq!(tx.send_slice(&values)?, values.len());
        let until = Instant::now() + CALL_TIMEOUT;
        // This is an explicit diagnostic sample of ring occupancy.
        while rx.available() < 8 && Instant::now() < until {
            std::thread::sleep(Duration::from_millis(1));
        }
        assert_eq!(rx.available(), 8);
        tx.send(&(begin + 8))?;
        std::thread::sleep(Duration::from_millis(30));
        assert_eq!(rx.available(), 8);
        let released = Instant::now();
        for expected in begin..=begin + 8 {
            rx.wait(released + Duration::from_millis(100))?;
            assert_eq!(rx.try_recv()?, Some(expected));
        }
        let resumed_us = released.elapsed().as_micros();
        println!("CHANNEL_BACKPRESSURE trial={trial} full=8 compared=9 resumed_us={resumed_us}");
        assert!((1..100_000).contains(&resumed_us));
        assert!(rx.try_recv()?.is_none());
    }
    Ok(())
}
