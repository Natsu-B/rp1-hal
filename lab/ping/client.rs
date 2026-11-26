use super::{CALL_TIMEOUT, Result, echo, rtt};
use rp1_linux::{CoreState, Error, Firmware, PcieState, ScmiState, ShmState};
use rp1_proto::note::{LINK, Link, Note, SHM_REGION, ShmRegion};
use std::time::{Duration, Instant};

#[path = "commands.rs"]
mod commands;

const ROUNDS: u32 = 5_000;

pub fn run(args: &[String]) -> Result<()> {
    let [mode, step] = args else {
        return Err("usage: linux-ping --test irq|poll ready|rtt|late|panic".into());
    };
    let irq = match mode.as_str() {
        "irq" => true,
        "poll" => false,
        _ => return Err("expected irq or poll".into()),
    };
    let fw = Firmware::open()?;
    let generation = fw.wait_ready(Duration::from_secs(10))?;
    if step == "ready" {
        check_note(irq)?;
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
    let mut client = fw.client("ping")?;
    match step.as_str() {
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
        "panic" if irq => {
            let start = Instant::now();
            let result = client.call(&commands::PANIC, CALL_TIMEOUT);
            let elapsed = start.elapsed();
            if !matches!(
                result,
                Err(Error::PeerDown {
                    core: 1,
                    state: CoreState::Panicked,
                    ..
                })
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

fn check_note(irq: bool) -> Result<()> {
    let bytes = std::fs::read("/sys/firmware/devicetree/base/rp1-firmware/rp1,note")?;
    let note = Note::parse(&bytes).map_err(Error::from)?;
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
