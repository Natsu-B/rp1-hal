use super::{CALL_TIMEOUT, Result, echo, rtt};
use rp1_linux::{
    Client, CoreState, Error, Firmware, PcieState, PeerDown, ScmiState, ShmState, WaitState,
    Waitable,
};
use rp1_proto::note::{LINK, Link, Note, SHM_REGION, ShmRegion};
use std::{
    io::Read,
    time::{Duration, Instant},
};

#[path = "commands.rs"]
mod commands;
#[path = "rtt_trace.rs"]
mod rtt_trace;

const ROUNDS: u32 = 5_000;

pub fn run(args: &[String]) -> Result<()> {
    let [mode, step] = args else {
        return Err(
            "usage: linux-ping --test irq|poll|wake ready|rtt|late|panic|stop|fault|proc0-panic|idle|idle60|burst-idle|drop|backpressure|reopen|status"
                .into(),
        );
    };
    let strict = mode == "wake";
    let irq = match mode.as_str() {
        "irq" | "wake" => true,
        "poll" => false,
        _ => return Err("expected irq, poll or wake".into()),
    };
    if std::env::var_os("RP1_PING_MLOCK").as_deref() == Some("1".as_ref()) {
        rtt_trace::lock_memory()?;
    }
    check_note(irq, strict)?;
    let opened = Instant::now();
    let fw = Firmware::open()?;
    if step == "status" {
        let status = fw.status()?;
        println!("PING_STATUS pcie={:?} shm={:?}", status.pcie, status.shm);
        return Ok(());
    }
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
    if strict && matches!(step.as_str(), "idle" | "idle60") {
        return idle(&fw, if step == "idle" { 11 } else { 62 });
    }
    if strict && step == "burst-idle" {
        return burst_idle(&fw);
    }
    if strict && matches!(step.as_str(), "drop" | "proc0-panic") {
        return dropped(&fw, step == "proc0-panic");
    }
    if strict && step == "backpressure" {
        return backpressure(&fw);
    }
    let mut client = fw.client("ping")?;
    match step.as_str() {
        "reopen" if strict => {
            let mut bytes = [0; 4];
            std::fs::File::open("/dev/urandom")?.read_exact(&mut bytes)?;
            let nonce = u32::from_ne_bytes(bytes) & !commands::DELAY;
            echo(&mut client, nonce)?;
            let elapsed_ns = opened.elapsed().as_nanos();
            println!(
                "PING_REOPEN generation={generation} nonce={nonce:#010x} elapsed_ns={elapsed_ns} ok=1"
            );
            if elapsed_ns > 1_000_000_000 {
                return Err(format!("reopened nonce took {elapsed_ns} ns").into());
            }
        }
        "rtt" if strict => strict_rtt(&mut client, 1_000)?,
        "rtt-10k" if strict => strict_rtt(&mut client, 10_000)?,
        "reference" if !strict => strict_rtt(&mut client, 1_000)?,
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
        "panic" | "stop" | "fault" if strict => {
            let (request, expected) = match step.as_str() {
                "panic" => (commands::PANIC, CoreState::Panicked),
                "fault" => (commands::FAULT, CoreState::Fault),
                _ => (commands::STOP, CoreState::Stopped),
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
            println!("PING_PEER_DOWN core=1 state={state:?} reason={reason} elapsed_us={us}");
            if state != expected || us > 15_000 {
                return Err(format!(
                    "expected {expected:?} within 15000 us, got {state:?} at {us} us"
                )
                .into());
            }
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

fn strict_rtt(client: &mut Client<u32, u32>, rounds: u32) -> Result<()> {
    let mut nanos = Vec::with_capacity(rounds as usize);
    let mut trace = rtt_trace::Trace::from_env(rounds)?;
    let mut failed = 0;
    for value in 0..rounds {
        let observation = trace.as_ref().map(|trace| trace.begin(client));
        let start = Instant::now();
        let reply = client.call_until(&value, start + CALL_TIMEOUT);
        let ns = start.elapsed().as_nanos();
        if let (Some(trace), Some(observation)) = (&mut trace, observation) {
            trace.end(client, observation, value, start, ns);
        }
        let ok = matches!(reply, Ok(echo) if echo == value);
        println!("PING_SAMPLE seq={value} ns={ns} ok={}", u8::from(ok));
        if ok {
            nanos.push(ns);
        } else {
            failed += 1;
            eprintln!("PING_SAMPLE_ERROR seq={value} result={reply:?}");
            if !matches!(reply, Ok(_) | Err(Error::Timeout)) {
                break;
            }
        }
    }
    if let Some(trace) = trace {
        trace.report()?;
    }
    nanos.sort_unstable();
    let rank = |percent: usize| {
        nanos
            .len()
            .checked_mul(percent)
            .unwrap()
            .div_ceil(100)
            .checked_sub(1)
            .map_or(0, |index| nanos[index])
    };
    println!(
        "PING_RTT n={} failed={failed} p50_ns={} p99_ns={} max_ns={} wait_stats={:?}",
        nanos.len() + failed,
        rank(50),
        rank(99),
        rank(100),
        client.wait_stats(),
    );
    if failed == 0 {
        Ok(())
    } else {
        Err(format!("{failed} incorrect or failed replies").into())
    }
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

fn idle(fw: &Firmware, seconds: u64) -> Result<()> {
    let (mut tx, mut rx) = fw.channel::<u32, u32>("channel")?.split();
    assert!(rx.try_recv()?.is_none());
    let mut client = fw.client("ping")?;
    echo(&mut client, commands::IDLE_BEGIN)?;
    let irq_before = irq_count()?;
    let now = Instant::now();
    println!("IDLE_BEGIN seconds={seconds} pid={}", std::process::id());
    let result = rx.wait(now + Duration::from_secs(seconds));
    let wall_us = now.elapsed().as_micros();
    let irq_after = irq_count()?;
    println!("IDLE_IRQ before={irq_before} after={irq_after}");
    let stats = rx.wait_stats();
    println!("IDLE_WAIT wall_us={wall_us} result={result:?} wait_stats={stats:?}");
    let nonce = 0xc5b2_0001;
    tx.send(&nonce)?;
    rx.wait(Instant::now() + CALL_TIMEOUT)?;
    assert_eq!(rx.try_recv()?, Some(nonce));
    echo(&mut client, commands::IDLE_END)?;
    println!("IDLE_NONCE value={nonce:#x} ok=1");
    assert_eq!(irq_before, irq_after);
    assert!(matches!(result, Err(Error::Timeout)));
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
    Ok(())
}

fn burst_idle(fw: &Firmware) -> Result<()> {
    let mut client = fw.client("ping")?;
    let (mut tx, mut rx) = fw.channel::<u32, u32>("channel")?.split();
    let mut shortest_us = u128::MAX;
    let mut failed = 0;
    for trial in 0..1_000 {
        let calls = [1, 2, 10, 100][trial % 4];
        for seq in 0..calls {
            echo(&mut client, seq)?;
        }
        let irq = irq_count()?;
        let stats = rx.wait_stats();
        let mut endpoints: [&mut dyn Waitable; 1] = [&mut rx];
        let mut wait = fw.wait_set(&mut endpoints)?;
        let start = Instant::now();
        let result = wait.wait(start + Duration::from_millis(100));
        let idle_us = start.elapsed().as_micros();
        drop(wait);
        let after = rx.wait_stats();
        let idle_irqs = irq_count()? - irq;
        let idle_wakes = after.woken - stats.woken;
        shortest_us = shortest_us.min(idle_us);
        let quiet = matches!(result, Err(Error::Timeout))
            && idle_us >= 100_000
            && idle_irqs == 0
            && idle_wakes == 0
            && after.deadline_checks - stats.deadline_checks == 1;
        let irq = irq_count()?;
        let nonce = 0xc5b2_1000 + trial as u32;
        let mut endpoints: [&mut dyn Waitable; 1] = [&mut rx];
        let mut wait = fw.wait_set(&mut endpoints)?;
        let end = Instant::now() + CALL_TIMEOUT;
        assert_eq!(wait.prepare(end)?, WaitState::Armed);
        tx.send(&nonce)?;
        let mut pfd = libc::pollfd {
            fd: std::os::fd::AsRawFd::as_raw_fd(&wait.as_fd()),
            events: libc::POLLIN,
            revents: 0,
        };
        loop {
            let ms = end.saturating_duration_since(Instant::now()).as_millis();
            let polled = unsafe { libc::poll(&mut pfd, 1, (ms as i32).max(1)) };
            if polled >= 0 {
                break;
            }
            let error = std::io::Error::last_os_error();
            if error.kind() != std::io::ErrorKind::Interrupted {
                return Err(error.into());
            }
        }
        let ready = matches!(wait.dispatch(pfd.revents), Ok(WaitState::Ready(0)));
        drop(wait);
        let next_irqs = irq_count()? - irq;
        let next_wakes = rx.wait_stats().woken - after.woken;
        let received = rx.try_recv()? == Some(nonce) && rx.try_recv()?.is_none();
        let ok = quiet && ready && received && next_irqs == 1 && next_wakes == 1;
        failed += u32::from(!ok);
        println!(
            "BURST_IDLE trial={trial} calls={calls} idle_us={idle_us} idle_irqs={idle_irqs} idle_wakes={idle_wakes} next_irqs={next_irqs} next_wakes={next_wakes} ok={}",
            u32::from(ok)
        );
    }
    println!("BURST_IDLE_RESULT n=1000 failed={failed} min_idle_us={shortest_us}");
    if failed != 0 {
        return Err(format!("{failed} burst/idle trials failed").into());
    }
    Ok(())
}

fn dropped(fw: &Firmware, proc0_panic: bool) -> Result<()> {
    let (mut tx, mut rx) = fw.channel::<u32, u32>("dropped")?.split();
    let mut endpoints = [&mut rx as &mut dyn Waitable];
    let mut wait = fw.wait_set(&mut endpoints)?;
    let request = if proc0_panic {
        commands::PROC0_PANIC
    } else {
        0x8751_0929
    };
    let start = Instant::now();
    let end = start + Duration::from_millis(10);
    assert_eq!(wait.prepare(end)?, WaitState::Armed);
    tx.send(&request)?;
    // Poll the fd without rearming, then dispatch including at the deadline.
    let mut pfd = libc::pollfd {
        fd: std::os::fd::AsRawFd::as_raw_fd(&wait.as_fd()),
        events: libc::POLLIN,
        revents: 0,
    };
    let result = loop {
        let remaining = end.saturating_duration_since(Instant::now());
        let timeout = libc::timespec {
            tv_sec: 0,
            tv_nsec: remaining.as_nanos() as _,
        };
        let polled = unsafe { libc::ppoll(&mut pfd, 1, &timeout, std::ptr::null()) };
        if polled < 0 {
            let error = std::io::Error::last_os_error();
            if error.kind() == std::io::ErrorKind::Interrupted {
                continue;
            }
            return Err(error.into());
        }
        match wait.dispatch(pfd.revents) {
            Ok(WaitState::Armed) => continue,
            result => break result,
        }
    };
    let elapsed = start.elapsed();
    println!(
        "PING_DROP_OR_PROC0_PANIC proc0_panic={proc0_panic} result={result:?} elapsed_ns={}",
        elapsed.as_nanos()
    );
    drop(wait);
    assert!(elapsed <= Duration::from_millis(15), "elapsed={elapsed:?}");
    if proc0_panic {
        assert!(matches!(
            result,
            Err(Error::PeerDown(PeerDown::Terminal {
                core: 0,
                state: CoreState::Panicked,
                ..
            }))
        ));
    } else {
        assert!(matches!(result, Err(Error::NotifyLost)));
        assert_eq!(rx.try_recv()?, Some(request));
        println!("PING_DROP_DATA value={request:#x} ok=1");
    }
    Ok(())
}

fn backpressure(fw: &Firmware) -> Result<()> {
    let (mut tx, mut rx) = fw.channel::<u32, u32>("channel")?.split();
    let mut failed = 0;
    for trial in 1..=1000u32 {
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
        assert_eq!(rx.available(), 8);
        let released = Instant::now();
        for expected in begin..=begin + 8 {
            rx.wait(released + Duration::from_millis(100))?;
            assert_eq!(rx.try_recv()?, Some(expected));
        }
        let resumed_ns = released.elapsed().as_nanos();
        println!("CHANNEL_BACKPRESSURE trial={trial} full=8 compared=9 resumed_ns={resumed_ns}");
        failed += usize::from(resumed_ns > 250_000);
        assert!(rx.try_recv()?.is_none());
    }
    println!("CHANNEL_BACKPRESSURE_SUMMARY n=1000 failed={failed}");
    if failed == 0 {
        Ok(())
    } else {
        Err(format!("{failed} Full recoveries exceeded 250 µs").into())
    }
}
