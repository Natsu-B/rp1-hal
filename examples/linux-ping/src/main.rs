use rp1_linux::{Client, Firmware};
use std::time::{Duration, Instant};

#[cfg(feature = "ping-test")]
#[path = "../../../lab/ping/client.rs"]
mod ping_test;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const CALL_TIMEOUT: Duration = Duration::from_secs(1);

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    #[cfg(feature = "ping-test")]
    if args.first().is_some_and(|arg| arg == "--test") {
        return ping_test::run(&args[1..]);
    }
    let rounds = match args.as_slice() {
        [] => 1000,
        [rounds] => rounds.parse()?,
        _ => return Err("usage: linux-ping [rounds]".into()),
    };
    if rounds == 0 {
        return Err("rounds must be positive".into());
    }
    let fw = Firmware::open()?;
    fw.wait_ready(Duration::from_secs(10))?;
    let mut client = fw.client("ping")?;
    rtt(&mut client, rounds, "")
}

fn echo(client: &mut Client<u32, u32>, value: u32) -> Result<u128> {
    let start = Instant::now();
    let reply = client.call(&value, CALL_TIMEOUT)?;
    let elapsed = start.elapsed().as_micros();
    if reply != value {
        return Err(format!("ping {value}: received {reply}").into());
    }
    Ok(elapsed)
}

fn rtt(client: &mut Client<u32, u32>, rounds: u32, mode: &str) -> Result<()> {
    let mut samples = (0..rounds)
        .map(|value| echo(client, value))
        .collect::<Result<Vec<_>>>()?;
    samples.sort_unstable();
    let percentile = |p: usize| samples[(samples.len() * p).div_ceil(100) - 1];
    println!(
        "PING_RTT{mode} n={rounds} failed=0 p50_us={} p99_us={} max_us={}",
        percentile(50),
        percentile(99),
        percentile(100)
    );
    Ok(())
}
