use std::{
    fs::{File, OpenOptions},
    io,
    os::fd::AsRawFd,
    path::Path,
    time::{Duration, Instant},
};

struct Mapping {
    ptr: *mut u8,
    len: usize,
    _file: File,
}
impl Mapping {
    fn open(path: &Path, expected: &str) -> io::Result<Self> {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| io::Error::other("invalid UIO path"))?;
        if !name.starts_with("uio")
            || !name[3..].bytes().all(|b| b.is_ascii_digit())
            || name.len() == 3
        {
            return Err(io::Error::other("expected /dev/uioN"));
        }
        let sys = Path::new("/sys/class/uio").join(name);
        if std::fs::read_to_string(sys.join("name"))?.trim() != expected {
            return Err(io::Error::other(format!("UIO must be named {expected}")));
        }
        let size = std::fs::read_to_string(sys.join("maps/map0/size"))?;
        let size = usize::from_str_radix(size.trim().trim_start_matches("0x"), 16)
            .map_err(io::Error::other)?;
        let offset = std::fs::read_to_string(sys.join("maps/map0/offset"))?;
        if usize::from_str_radix(offset.trim().trim_start_matches("0x"), 16)
            .map_err(io::Error::other)?
            != 0
            || size < 0x4000
        {
            return Err(io::Error::other(
                "expected an aligned mapping of at least 16 KiB",
            ));
        }
        let file = OpenOptions::new().read(true).write(true).open(path)?;
        let ptr = unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                0x4000,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_SHARED,
                file.as_raw_fd(),
                0,
            )
        };
        if ptr == libc::MAP_FAILED {
            return Err(io::Error::last_os_error());
        }
        Ok(Self {
            ptr: ptr.cast(),
            len: 0x4000,
            _file: file,
        })
    }
    fn read(&self, offset: usize) -> u32 {
        assert!(offset.is_multiple_of(4) && offset + 4 <= self.len);
        unsafe { self.ptr.add(offset).cast::<u32>().read_volatile() }
    }
    fn write(&mut self, offset: usize, value: u32) {
        assert!(offset.is_multiple_of(4) && offset + 4 <= self.len);
        unsafe {
            self.ptr.add(offset).cast::<u32>().write_volatile(value);
        }
        rp1_proto::barrier();
    }
}
impl Drop for Mapping {
    fn drop(&mut self) {
        unsafe {
            libc::munmap(self.ptr.cast(), self.len);
        }
    }
}

/// Interrupt mode re-checks the reply this often when no wake comes (a lost edge costs at most
/// this much latency, never a hang).
const RECHECK_MS: i32 = 10;

/// The `rp1:irq` UIO node (RP1 MSI-X vector `rp1_proto::irq::VECTOR`, no mapping used).
struct Irq {
    file: File,
}
impl Irq {
    fn open(path: &Path) -> io::Result<Self> {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| io::Error::other("invalid UIO path"))?;
        let sys = Path::new("/sys/class/uio").join(name);
        if std::fs::read_to_string(sys.join("name"))?.trim() != "rp1:irq" {
            return Err(io::Error::other("UIO must be named rp1:irq"));
        }
        let file = OpenOptions::new().read(true).write(true).open(path)?;
        Ok(Self { file })
    }
    /// Re-enable the interrupt (uio_pdrv_genirq masks it in its handler).
    fn enable(&mut self) -> io::Result<()> {
        let one = 1u32.to_ne_bytes();
        let n = unsafe { libc::write(self.file.as_raw_fd(), one.as_ptr().cast(), 4) };
        if n != 4 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }
    /// Wait up to `timeout_ms` for an interrupt; returns the UIO event count, or None.
    fn wait(&mut self, timeout_ms: i32) -> io::Result<Option<u32>> {
        let mut pfd = libc::pollfd {
            fd: self.file.as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        };
        let r = unsafe { libc::poll(&mut pfd, 1, timeout_ms) };
        if r < 0 {
            return Err(io::Error::last_os_error());
        }
        if r == 0 {
            return Ok(None);
        }
        let mut count = [0u8; 4];
        let n = unsafe { libc::read(self.file.as_raw_fd(), count.as_mut_ptr().cast(), 4) };
        if n != 4 {
            return Err(io::Error::last_os_error());
        }
        Ok(Some(u32::from_ne_bytes(count)))
    }
}

/// Linux -> RP1: GPIO27 PROC1_INTF force bit. RP1 -> Linux: either polling the reply word in
/// the link page, or (4th argument) waiting on the `rp1:irq` UIO node (MSI-X vector 29, which
/// proc1 pulses after the reply; hwirq58 stays with rp1-mailbox / SCMI).
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if !(2..=4).contains(&args.len()) {
        return Err(
            "usage: linux-doorbell /dev/uio<rp1:link> /dev/uio<rp1:doorbell> [iterations] [/dev/uio<rp1:irq>]".into(),
        );
    }
    let iterations: u32 = match args.get(2) {
        Some(n) => n
            .to_str()
            .and_then(|n| n.parse().ok())
            .filter(|n| *n > 0)
            .ok_or("iterations must be a positive integer")?,
        None => 10,
    };
    let mut rtt = Vec::with_capacity(iterations as usize);
    let mut link = Mapping::open(Path::new(&args[0]), "rp1:link")?;
    let mut gpio = Mapping::open(Path::new(&args[1]), "rp1:doorbell")?;
    let mut irq = match args.get(3) {
        Some(p) => Some(Irq::open(Path::new(p))?),
        None => None,
    };
    // Interrupt mode: drain anything pending from before (`stale`), count wakes that came before
    // the reply (`early`, the receiver must tolerate them) and the UIO event counter span.
    // `stale=1` is expected after a polling run with 2 or more replies: the first pulse is
    // delivered and uio_pdrv_genirq masks the line, the next ones are held as one pending edge
    // (lazy disable) and replayed by the first `enable()` here (HW 2026-11-05).
    // `missed`: the reply was found by a timed re-check, not by a wake.
    let (mut early, mut missed, mut first_count, mut last_count) = (0u32, 0u32, None, 0u32);
    let mut stale = 0u32;
    if let Some(irq) = irq.as_mut() {
        irq.enable()?;
        while let Some(c) = irq.wait(20)? {
            stale += 1;
            last_count = c;
            irq.enable()?;
        }
    }
    for sequence in 1..=iterations {
        link.write(0x14, 0);
        link.write(0x10, sequence);
        // Same-window read completes the posted SRAM writes before a BAR1 doorbell.
        if link.read(0x10) != sequence || link.read(0x14) != 0 {
            return Err("posted write readback mismatch".into());
        }
        gpio.write(0x2114, 1 << 27);
        let start = Instant::now();
        if let Some(irq) = irq.as_mut() {
            // A wake is a hint (edges can be lost or merged, rp1_hal::host_irq): re-check the
            // reply after every wake and after every bounded wait, never sleep on one edge.
            loop {
                let woke = irq.wait(RECHECK_MS)?;
                if let Some(c) = woke {
                    first_count.get_or_insert(c);
                    last_count = c;
                }
                match link.read(0x14) {
                    v if v == sequence => {
                        missed += u32::from(woke.is_none());
                        break;
                    }
                    u32::MAX => return Err("BAR2 reads 0xffffffff: PCIe link down".into()),
                    // Check the deadline first so a stream of early wakes cannot spin forever.
                    _ if start.elapsed() > Duration::from_secs(1) => {
                        return Err(format!("RP1 reply timed out (sequence {sequence})").into());
                    }
                    _ if woke.is_some() => {
                        early += 1;
                        irq.enable()?;
                    }
                    _ => {}
                }
            }
            let elapsed = start.elapsed();
            irq.enable()?;
            rtt.push(elapsed);
            if iterations <= 10 {
                println!("irq {sequence}: {elapsed:?}");
            }
            continue;
        }
        loop {
            match link.read(0x14) {
                v if v == sequence => break,
                // Link down reads as all-ones (design §3.4): fail rather than spin on it.
                u32::MAX => return Err("BAR2 reads 0xffffffff: PCIe link down".into()),
                _ => {}
            }
            if start.elapsed() > Duration::from_secs(1) {
                return Err("RP1 doorbell reply timed out".into());
            }
            std::hint::spin_loop();
        }
        let elapsed = start.elapsed();
        rp1_proto::barrier();
        if iterations <= 10 {
            println!("doorbell {sequence}: {elapsed:?}");
        }
        rtt.push(elapsed);
    }
    // A wake still pending after the last reply would be a duplicate delivery.
    let mut trailing = 0u32;
    if let Some(irq) = irq.as_mut() {
        while let Some(c) = irq.wait(50)? {
            trailing += 1;
            last_count = c;
            irq.enable()?;
        }
    }
    rtt.sort_unstable();
    let pct = |p: usize| rtt[((rtt.len() - 1) * p).div_ceil(100)];
    let p999 = rtt[((rtt.len() - 1) * 999).div_ceil(1000)];
    println!(
        "{} rtt n={} min={:?} p50={:?} p99={:?} p99.9={:?} max={:?}",
        if irq.is_some() { "irq" } else { "doorbell" },
        rtt.len(),
        rtt[0],
        pct(50),
        pct(99),
        p999,
        rtt[rtt.len() - 1]
    );
    if irq.is_some() {
        let span = first_count.map_or(0, |f| last_count.wrapping_sub(f) + 1);
        println!(
            "irq events stale={stale} early={early} missed={missed} trailing={trailing} uio_count_span={span}"
        );
    }
    Ok(())
}
