//! Print the RP1 firmware's status words (`rp1_linux::Monitor`). With `--health MS`, also judge
//! liveness over at least MS ms and exit 1 unless it is `Ok`. Needs only `rp1:link`, read-only,
//! so it also runs while another process owns the firmware's shared memory.
use rp1_linux::{Health, Monitor};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let window = match args.as_slice() {
        [] => None,
        [flag, ms] if flag == "--health" => Some(Duration::from_millis(ms.parse()?)),
        _ => return Err("usage: linux-status [--health MS]".into()),
    };
    let monitor = Monitor::open()?;
    let status = monitor.status()?;
    println!(
        "RP1_STATUS epoch={} shm={:?} shm_generation={} pcie={} scmi={}",
        status.epoch, status.shm, status.shm_generation, status.pcie, status.scmi
    );
    for (core, c) in status.cores.iter().enumerate() {
        println!(
            "RP1_CORE core={core} state={:?} reason={} line={} aux=0x{:x} beat={} file={:?}",
            c.state, c.reason, c.line, c.aux, c.beat, c.file
        );
    }
    if let Some(window) = window {
        let health = monitor.health(window)?;
        println!("RP1_HEALTH {health:?}");
        if health != Health::Ok {
            std::process::exit(1);
        }
    }
    Ok(())
}
