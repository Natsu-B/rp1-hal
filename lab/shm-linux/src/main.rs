use rp1_linux::{BlockMap, Error, Firmware};
mod dma;
mod down_irq;
mod irq_common;
#[path = "../../irq_message.rs"]
mod irq_message;
mod ring_dma;
mod ring_irq;
mod up_irq;
use rp1_proto as proto;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
/// How long [`Firmware::wait_ready`] may take: RP1 enables shared memory on request.
const READY: Duration = Duration::from_secs(10);
use std::time::{Duration, Instant};
type Control = [u32; 16];
type Bulk = [u32; 262144];
fn sample(i: u32) -> [u32; 4] {
    [i, !i, i ^ 0x5a96_c33c, 0x5250_3148]
}
fn expired(start: Instant) -> Result<()> {
    if start.elapsed() > Duration::from_secs(180) {
        Err("shm test timed out".into())
    } else {
        Ok(())
    }
}
fn request(
    ctrl: &mut BlockMap<Control>,
    id: u32,
    command: u32,
    count: u32,
    seed: u32,
) -> Result<Instant> {
    request_with(ctrl, id, command, count, seed, &[])
}
/// [`request`] with arguments in words 8.. (at most 8).
fn request_with(
    ctrl: &mut BlockMap<Control>,
    id: u32,
    command: u32,
    count: u32,
    seed: u32,
    extra: &[u32],
) -> Result<Instant> {
    let mut words = [0; 16];
    words[..5].copy_from_slice(&[id, command, count, seed, 1]);
    words[8..8 + extra.len()].copy_from_slice(extra);
    // Only Linux writes until the command is published; it then yields the writer role.
    let mut writer = unsafe { ctrl.begin_write() }?;
    writer.write_words(0, &words)?;
    writer.commit();
    Ok(Instant::now())
}
fn response(ctrl: &BlockMap<Control>, id: u32, start: Instant) -> Result<Control> {
    loop {
        match ctrl.read() {
            Ok(words) if words[0] == id && words[4] == 2 => {
                if words[5] != 0 {
                    return Err(format!("RP1 reported {} errors", words[5]).into());
                }
                return Ok(words);
            }
            Ok(_) | Err(Error::Proto(proto::Error::NotReady)) => {}
            Err(e) => return Err(e.into()),
        }
        expired(start)?;
    }
}
fn verify_bulk(bulk: &BlockMap<Bulk>, seed: u32) -> Result<()> {
    bulk.read_with(|reader| {
        let mut words = [0; 1024];
        for at in (0..262144).step_by(words.len()) {
            reader.read_words(at, &mut words)?;
            for (i, v) in words.iter().enumerate() {
                if *v != (at + i) as u32 ^ seed {
                    return Err(proto::Error::Corrupt);
                }
            }
        }
        Ok::<_, proto::Error>(())
    })??;
    Ok(())
}
fn probe_window() -> Result<()> {
    use std::{
        fs::{self, OpenOptions},
        os::fd::AsRawFd,
    };
    let mut found = None;
    for entry in fs::read_dir("/sys/class/uio")? {
        let path = entry?.path();
        if fs::read_to_string(path.join("name"))?.trim() == "rp1:link" {
            if found.is_some() {
                return Err("duplicate link UIO".into());
            }
            found = Some(path);
        }
    }
    let path = found.ok_or("missing link UIO")?;
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(std::path::Path::new("/dev").join(path.file_name().unwrap()))?;
    if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    let pointer = unsafe {
        libc::mmap(
            std::ptr::null_mut(),
            0x4000,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_SHARED,
            file.as_raw_fd(),
            0,
        )
    };
    if pointer == libc::MAP_FAILED {
        return Err(std::io::Error::last_os_error().into());
    }
    let result = (|| {
        let p = pointer.cast::<u32>();
        if unsafe { p.read_volatile() } != rp1_proto::link::MAGIC {
            return Err("probe needs loader handoff".into());
        }
        if unsafe { p.add(7).read_volatile() } != 0 || unsafe { p.add(9).read_volatile() } != 0 {
            return Err("whole-window probe requires a fresh cold boot".into());
        }
        // Lab-only pre-init protocol: request 2, result at link+36. All access is aligned UIO.
        unsafe {
            p.add(8).write_volatile(2);
        }
        rp1_proto::barrier();
        let start = Instant::now();
        loop {
            match unsafe { p.add(9).read_volatile() } {
                1 => {
                    println!("WINDOW_PASS bytes={} all_words errors=0", unsafe {
                        p.add(3).read_volatile()
                    });
                    return Ok(());
                }
                2 => return Err("whole-window pattern mismatch".into()),
                _ => {}
            }
            expired(start)?;
            std::thread::sleep(Duration::from_millis(1));
        }
    })();
    unsafe {
        libc::munmap(pointer, 0x4000);
    }
    result
}
fn main() -> Result<()> {
    let command = std::env::args().nth(1);
    if command.as_deref() == Some("probe-only") {
        probe_window()?;
        let generation = Firmware::open()?.wait_ready(READY)?;
        println!("SHM_READY generation={generation}");
        return Ok(());
    }
    if command.as_deref() == Some("workload") {
        return workload(count_arg(2)?);
    }
    if command.as_deref() == Some("ping") {
        return ping();
    }
    if std::env::args().nth(1).as_deref() == Some("dma") {
        return dma::main();
    }
    if std::env::args().nth(1).as_deref() == Some("ring-dma-perf") {
        return ring_dma::perf(std::env::args().skip(2));
    }
    if matches!(
        std::env::args().nth(1).as_deref(),
        Some("ring-dma" | "ring-dma-order" | "ring-dma-wide256-cpu" | "ring-dma-wide256-dma")
    ) {
        let count = std::env::args()
            .nth(2)
            .map(|v| v.parse())
            .transpose()?
            .unwrap_or(1_000_000);
        return ring_dma::main(
            count,
            std::env::args().nth(1).as_deref() == Some("ring-dma-order"),
            match std::env::args().nth(1).as_deref() {
                Some("ring-dma-wide256-cpu") => Some(0),
                Some("ring-dma-wide256-dma") => Some(1),
                _ => None,
            },
        );
    }
    // The notification proofs of lab/irq-proof (down_irq.rs, up_irq.rs).
    match std::env::args().nth(1).as_deref() {
        Some("down-irq") => return down_irq::main(&down_irq::DOWN1, std::env::args().skip(2)),
        Some("down0-irq") => return down_irq::main(&down_irq::DOWN0, std::env::args().skip(2)),
        Some("controls") => return down_irq::controls(),
        Some("host0-irq") => return up_irq::host0(std::env::args().skip(2)),
        Some("mix-irq") => return up_irq::mix(std::env::args().skip(2)),
        Some("irq-pair") => return up_irq::pair(std::env::args().skip(2)),
        Some("pair-member") => return up_irq::pair_member(std::env::args().skip(2)),
        _ => {}
    }
    if std::env::args().nth(1).as_deref() == Some("ring-irq") {
        let count = std::env::args()
            .nth(2)
            .map(|v| v.parse())
            .transpose()?
            .unwrap_or(1_000_000);
        return ring_irq::main(count);
    }
    let count = count_arg(1)?;
    probe_window()?;
    workload(count)
}

fn ping() -> Result<()> {
    const ROUNDS: usize = 1000;
    let fw = Firmware::open()?;
    fw.wait_ready(READY)?;
    let mut client = fw.client::<[u32; 4], [u32; 4]>("ping")?;
    let mut samples = [0; ROUNDS];
    for (i, us) in samples.iter_mut().enumerate() {
        let value = sample(i as u32);
        let start = Instant::now();
        let reply = client.call(&value, Duration::from_secs(1))?;
        *us = start.elapsed().as_micros();
        if reply != value {
            return Err(format!("ping {i}: received {reply:?}").into());
        }
    }
    samples.sort_unstable();
    println!(
        "PING_RTT n={ROUNDS} failed=0 p50_us={} p99_us={} max_us={}",
        samples[499], samples[989], samples[999]
    );
    Ok(())
}

fn count_arg(index: usize) -> Result<u32> {
    let count = std::env::args()
        .nth(index)
        .map(|v| v.parse())
        .transpose()?
        .unwrap_or(1_000_000);
    if count < 1_000_000 {
        return Err("proof needs at least 1e6 messages per direction".into());
    }
    Ok(count)
}

fn workload(count: u32) -> Result<()> {
    let fw = Firmware::open()?;
    let generation = fw.wait_ready(READY)?;
    let mut up = fw.ring_rx::<[u32; 4]>("up")?;
    let mut down = fw.ring_tx::<[u32; 4]>("down")?;
    let mut latest = fw.ring_rx::<[u32; 4]>("latest")?;
    let mut ctrl = fw.block::<Control>("control")?;
    let mut bulk = fw.block::<Bulk>("bulk")?;
    println!("SHM generation={generation} count={count}");
    // Keep the same mapping ownership while showing that handles outlive Firmware.
    drop(fw);
    let start = request(&mut ctrl, 1, 1, count, 0)?;
    let mut got = 0;
    while got < count {
        if let Some(s) = up.try_recv()? {
            if s != sample(got) {
                return Err(format!("up corrupt at {got}: {s:?}").into());
            }
            got += 1;
        }
        expired(start)?;
    }
    let report = response(&ctrl, 1, start)?;
    println!(
        "RING up n={got} missing=0 corrupt=0 seconds={:.6} MBps={:.3} rp1_us={} full_retries={}",
        start.elapsed().as_secs_f64(),
        count as f64 * 16.0 / start.elapsed().as_secs_f64() / 1e6,
        report[6],
        report[7]
    );
    let start = request(&mut ctrl, 2, 2, count, 0)?;
    let mut sent = 0;
    while sent < count {
        match down.send(&sample(sent)) {
            Ok(()) => sent += 1,
            Err(Error::Proto(proto::Error::Full)) => {}
            Err(e) => return Err(e.into()),
        }
        expired(start)?;
    }
    let report = response(&ctrl, 2, start)?;
    println!(
        "RING down n={sent} missing=0 corrupt=0 seconds={:.6} MBps={:.3} rp1_us={} full_retries={}",
        start.elapsed().as_secs_f64(),
        count as f64 * 16.0 / start.elapsed().as_secs_f64() / 1e6,
        report[6],
        down.dropped()
    );
    let seed = 0x96c3_5aa5;
    let start = request(&mut ctrl, 3, 3, 0, seed)?;
    response(&ctrl, 3, start)?;
    let read = Instant::now();
    verify_bulk(&bulk, seed)?;
    println!(
        "BLOCK rp1-to-linux bytes=1048576 all_words=262144 errors=0 linux_read_MBps={:.3}",
        1.048576 / read.elapsed().as_secs_f64()
    );
    let seed = !seed;
    let write = Instant::now();
    let mut writer = unsafe { bulk.begin_write() }?;
    writer.fill(0, 262144, |i| i as u32 ^ seed)?;
    writer.commit();
    let seconds = write.elapsed().as_secs_f64();
    let start = request(&mut ctrl, 4, 4, 0, seed)?;
    response(&ctrl, 4, start)?;
    println!(
        "BLOCK linux-to-rp1 bytes=1048576 all_words=262144 errors=0 linux_write_MBps={:.3}",
        1.048576 / seconds
    );
    let start = request(&mut ctrl, 5, 5, 0, seed)?;
    let report = response(&ctrl, 5, start)?;
    verify_bulk(&bulk, seed)?;
    println!(
        "DDR read_latency_n=4096 us={} ns_per_u32={:.2} local_baseline_us={} read_bytes=1048576 read_us={} read_MBps={:.3} write_bytes=1048576 write_us={} write_MBps={:.3}",
        report[8],
        report[8] as f64 * 1000.0 / 4096.0,
        report[11],
        report[9],
        1.048576 / (report[9] as f64 / 1e6),
        report[10],
        1.048576 / (report[10] as f64 / 1e6)
    );
    let n = 100_000;
    let start = request(&mut ctrl, 6, 6, n, 0)?;
    // Deliberately overrun the ring before consuming, then check every survivor.
    response(&ctrl, 6, start)?;
    let (mut lost, mut received, mut expected) = (0, 0, 0);
    while lost + received < n {
        match latest.try_recv() {
            Ok(Some(s)) => {
                if s != sample(expected) {
                    return Err("overwrite corrupt".into());
                }
                expected += 1;
                received += 1;
            }
            Err(Error::Lost(k)) => {
                lost += k;
                expected += k;
            }
            Ok(None) | Err(Error::Proto(proto::Error::NotReady)) => {}
            Err(e) => return Err(e.into()),
        }
        expired(start)?;
    }
    println!("OVERWRITE n={n} received={received} lost={lost} corrupt=0");
    // Isolated per-message M3 cost, with no concurrent Linux traffic to the link page.
    for i in 0..128 {
        down.send(&sample(i))?;
    }
    let start = request(&mut ctrl, 7, 7, 128, 0)?;
    let report = response(&ctrl, 7, start)?;
    println!(
        "ISOLATED down_pop n=128 rp1_us={} us_per_msg={:.3}",
        report[6],
        report[6] as f64 / 128.0
    );
    let start = request(&mut ctrl, 8, 8, 256, 0)?;
    let report = response(&ctrl, 8, start)?;
    for i in 0..256 {
        if up.try_recv()? != Some(sample(i)) {
            return Err(format!("isolated up corrupt at {i}").into());
        }
    }
    println!(
        "ISOLATED up_push n=256 rp1_us={} us_per_msg={:.3}",
        report[6],
        report[6] as f64 / 256.0
    );
    // Batched streams: RP1 uses push_slice/pop_into from ISRAM, Linux uses the batch API too.
    let mut batch = [[0u32; 4]; 64];
    let start = request(&mut ctrl, 10, 10, count, 0)?;
    let mut got = 0;
    while got < count {
        let n = up.try_recv_into(&mut batch)?;
        for s in &batch[..n] {
            if *s != sample(got) {
                return Err(format!("up batch corrupt at {got}: {s:?}").into());
            }
            got += 1;
        }
        expired(start)?;
    }
    let report = response(&ctrl, 10, start)?;
    println!(
        "RING up_batch n={got} missing=0 corrupt=0 seconds={:.6} MBps={:.3} rp1_us={} us_per_msg={:.3}",
        start.elapsed().as_secs_f64(),
        count as f64 * 16.0 / start.elapsed().as_secs_f64() / 1e6,
        report[6],
        report[6] as f64 / count as f64
    );
    let start = request(&mut ctrl, 11, 11, count, 0)?;
    let mut sent = 0;
    while sent < count {
        let k = (count - sent).min(batch.len() as u32) as usize;
        for (j, b) in batch[..k].iter_mut().enumerate() {
            *b = sample(sent + j as u32);
        }
        sent += down.send_slice(&batch[..k])? as u32;
        expired(start)?;
    }
    let report = response(&ctrl, 11, start)?;
    println!(
        "RING down_batch n={sent} missing=0 corrupt=0 seconds={:.6} MBps={:.3} rp1_us={} us_per_msg={:.3}",
        start.elapsed().as_secs_f64(),
        count as f64 * 16.0 / start.elapsed().as_secs_f64() / 1e6,
        report[6],
        report[6] as f64 / count as f64
    );
    // Isolated batch costs (Linux idle on the link page), as ISOLATED above.
    for i in 0..128 {
        down.send(&sample(i))?;
    }
    let start = request(&mut ctrl, 13, 13, 128, 0)?;
    let report = response(&ctrl, 13, start)?;
    println!(
        "ISOLATED down_pop_batch n=128 rp1_us={} us_per_msg={:.3}",
        report[6],
        report[6] as f64 / 128.0
    );
    let start = request(&mut ctrl, 12, 12, 256, 0)?;
    let report = response(&ctrl, 12, start)?;
    for i in 0..256 {
        if up.try_recv()? != Some(sample(i)) {
            return Err(format!("isolated batch up corrupt at {i}").into());
        }
    }
    println!(
        "ISOLATED up_push_batch n=256 rp1_us={} us_per_msg={:.3}",
        report[6],
        report[6] as f64 / 256.0
    );
    #[cfg(feature = "sram-profile")]
    {
        let start = request(&mut ctrl, 9, 9, 0, 0)?;
        let report = response(&ctrl, 9, start)?;
        println!("PROFILE rp1_us={} (results on UART0)", report[6]);
    }
    #[cfg(not(feature = "sram-profile"))]
    println!("PROFILE skipped");
    println!("SHM_PASS");
    Ok(())
}
