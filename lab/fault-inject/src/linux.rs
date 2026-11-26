//! Target-side collection. The host verifier judges the UART evidence.
mod kmsg;
mod protocol;

use protocol::*;
use rp1_linux::{CoreState, Monitor, ScmiState, Status};
use std::{
    fs::{self, File, OpenOptions},
    io,
    net::Ipv4Addr,
    os::fd::AsRawFd,
    path::Path,
    process::{Command, Output},
    ptr::NonNull,
    thread::sleep,
    time::{Duration, Instant},
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const MAP_LEN: usize = 0x4000;
const CLK_RATE: &str = "/sys/kernel/debug/clk/uart/clk_rate";
const STATUS_WAIT: Duration = Duration::from_secs(5);

struct LinkPage {
    base: NonNull<u8>,
    _file: File,
}

impl LinkPage {
    fn open() -> Result<Self> {
        // Monitor validates the physical address and size on the same named UIO node.
        let mut matches = Vec::new();
        for entry in fs::read_dir("/sys/class/uio")? {
            let path = entry?.path();
            if fs::read_to_string(path.join("name"))?.trim() == "rp1:link" {
                matches.push(path);
            }
        }
        let [path] = matches.as_slice() else {
            return Err("expected exactly one rp1:link UIO node".into());
        };
        let dev = Path::new("/dev").join(path.file_name().ok_or("invalid UIO name")?);
        let file = OpenOptions::new().read(true).write(true).open(dev)?;
        if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
            return Err(io::Error::last_os_error().into());
        }
        let base = unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                MAP_LEN,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_SHARED,
                file.as_raw_fd(),
                0,
            )
        };
        if base == libc::MAP_FAILED {
            return Err(io::Error::last_os_error().into());
        }
        Ok(Self {
            base: NonNull::new(base.cast()).ok_or("null UIO mapping")?,
            _file: file,
        })
    }

    fn word(&self, offset: usize) -> *mut u32 {
        assert!(offset.is_multiple_of(4) && offset + 4 <= MAP_LEN);
        unsafe { self.base.as_ptr().add(offset).cast() }
    }

    fn read(&self, offset: usize) -> u32 {
        unsafe { self.word(offset).read_volatile() }
    }

    fn request(&self, kind: Kind) -> Result<()> {
        unsafe { self.word(REQUEST).write_volatile(kind as u32) };
        rp1_proto::barrier();
        if self.read(REQUEST) != kind as u32 {
            return Err("fault request posted-write readback failed".into());
        }
        Ok(())
    }

    fn check_boot(&self, nonce: u32) -> Result<()> {
        if self.read(READY) != MAGIC || self.read(NONCE) != nonce {
            return Err("missing or changed fault lab nonce".into());
        }
        Ok(())
    }

    fn ticks(&self) -> Result<[u32; 2]> {
        let ticks = TICKS.map(|offset| self.read(offset));
        if ticks.contains(&u32::MAX) {
            return Err("unreadable timer counts".into());
        }
        Ok(ticks)
    }
}

impl Drop for LinkPage {
    fn drop(&mut self) {
        unsafe { libc::munmap(self.base.as_ptr().cast(), MAP_LEN) };
    }
}

fn same_epoch(monitor: &Monitor, epoch: u32) -> Result<Status> {
    let status = monitor.status()?;
    if status.epoch != epoch {
        return Err("RP1 restarted during collection".into());
    }
    Ok(status)
}

fn show(output: &Output) {
    print!("{}", String::from_utf8_lossy(&output.stdout));
    print!("{}", String::from_utf8_lossy(&output.stderr));
}

fn ping(tag: &str, host: Ipv4Addr) {
    let output = Command::new("ping")
        .args(["-n", "-I", "eth0", "-c", "20", "-w", "30"])
        .arg(host.to_string())
        .env("LC_ALL", "C")
        .output();
    match output {
        Ok(out) => {
            show(&out);
            let text = String::from_utf8_lossy(&out.stdout);
            let counts = text.lines().find_map(|line| {
                let (sent, rest) = line.split_once(" packets transmitted, ")?;
                let (received, _) = rest.split_once(" received")?;
                Some((sent.parse::<u32>().ok()?, received.parse::<u32>().ok()?))
            });
            let (sent, received) = match counts {
                Some((sent, received)) => (sent.to_string(), received.to_string()),
                None => ("unknown".into(), "unknown".into()),
            };
            println!(
                "FAULT_PING {tag} sent={sent} received={received} rc={}",
                out.status.code().unwrap_or(128)
            );
        }
        Err(error) => println!("FAULT_ERROR {tag} step=ping error={error}"),
    }
}

fn usb(tag: &str, device: &str) {
    let output = Command::new("rp1-device-check")
        .arg(device)
        .env("LABEL", "fault")
        .env("LC_ALL", "C")
        .output();
    match output {
        Ok(out) => {
            show(&out);
            let text = String::from_utf8_lossy(&out.stdout);
            let prefix = format!("DEVCHECK dev={device} op=control ");
            if let Some(row) = text.lines().find_map(|line| line.strip_prefix(&prefix)) {
                // Carry the actual control probe's predicates into the nonce-bound record.
                let fields = row.split_whitespace().take(4).collect::<Vec<_>>().join(" ");
                println!("FAULT_USB {tag} {fields}");
            }
        }
        Err(error) => println!("FAULT_ERROR {tag} step=usb error={error}"),
    }
}

fn requests(state: ScmiState) -> String {
    match state {
        ScmiState::Serving { requests } => requests.to_string(),
        _ => "unknown".into(),
    }
}

fn scmi(tag: &str, monitor: &Monitor, epoch: u32, n: u16) -> Result<()> {
    let before = same_epoch(monitor, epoch)?.scmi;
    let mut replies = 0;
    for _ in 0..n {
        match fs::read_to_string(CLK_RATE) {
            Ok(rate) => {
                let rate = rate.trim();
                println!("FAULT_RATE {tag} hz={rate}");
                if rate.parse::<u64>().is_ok_and(|hz| hz > 0) {
                    replies += 1;
                }
            }
            Err(error) => println!("FAULT_ERROR {tag} step=clk_rate error={error}"),
        }
    }
    let after = same_epoch(monitor, epoch)?.scmi;
    println!(
        "FAULT_SCMI {tag} n={n} replies={replies} before={} after={} scmi_before={before} scmi_after={after}",
        requests(before),
        requests(after)
    );
    Ok(())
}

struct Args {
    kind: Kind,
    label: String,
    nonce: u32,
    host: Ipv4Addr,
    port: String,
    n: u16,
}

fn parse_args() -> Result<Args> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let [label, nonce, host, port, n] = args.as_slice() else {
        return Err(format!(
            "usage: fault-linux KIND NONCE HOST_IPV4 USB_PORT SCMI_N (kinds: {ALLOWED})"
        )
        .into());
    };
    let kind = Kind::parse(label).ok_or("unknown fault kind")?;
    if nonce.len() != 8 || !nonce.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("expected the eight-digit FAULT_BOOT nonce from UART0".into());
    }
    let nonce = u32::from_str_radix(nonce, 16)?;
    let host: Ipv4Addr = host.parse()?;
    let n: u16 = n.parse()?;
    if n == 0 || n > 100 {
        return Err("SCMI_N must be in 1..=100".into());
    }
    let (bus, ports) = port
        .split_once('-')
        .ok_or("expected USB port such as 3-1")?;
    if std::iter::once(bus)
        .chain(ports.split('.'))
        .any(|part| part.is_empty() || !part.bytes().all(|b| b.is_ascii_digit()))
    {
        return Err("invalid USB port".into());
    }

    Ok(Args {
        kind,
        label: label.clone(),
        nonce,
        host,
        port: port.clone(),
        n,
    })
}

fn client_sha256() -> Result<String> {
    // Use this process's exe symlink, including when the deployed file was replaced.
    let output = Command::new("sha256sum")
        .arg(format!("/proc/{}/exe", std::process::id()))
        .output()?;
    let text = std::str::from_utf8(&output.stdout)?;
    let digest = text.split_whitespace().next().unwrap_or("");
    if !output.status.success()
        || digest.len() != 64
        || !digest.bytes().all(|b| b.is_ascii_hexdigit())
    {
        return Err("cannot hash fault-linux executable".into());
    }
    Ok(digest.to_owned())
}

fn main() -> Result<()> {
    let Args {
        kind,
        label,
        nonce,
        host,
        port,
        n,
    } = parse_args()?;
    let monitor = Monitor::open()?;
    let initial = monitor.status()?;
    let epoch = initial.epoch;
    let link = LinkPage::open()?;
    link.check_boot(nonce)?;
    if link.read(REQUEST) != 0
        || link.read(ACK) != 0
        || link.read(SITE) != 0
        || initial.cores.iter().any(|c| c.state != CoreState::Running)
    {
        return Err("need a fresh boot with both cores Running".into());
    }
    let boot_id = fs::read_to_string("/proc/sys/kernel/random/boot_id")?;
    let tag = format!("nonce={nonce:08x} kind={label}");
    let sha256 = client_sha256()?;
    let kernel_log = kmsg::start();
    println!(
        "FAULT_BEGIN {tag} boot_id={} epoch={epoch} client_sha256={sha256}",
        boot_id.trim()
    );

    let core = kind.core();
    if kind != Kind::Stkerr {
        if !Path::new(CLK_RATE).is_file() {
            return Err(format!("{CLK_RATE} is missing, check debugfs and the SCMI clock").into());
        }
        let [p0_before, p1_before] = link.ticks()?;
        sleep(Duration::from_millis(250));
        let [p0_after, p1_after] = link.ticks()?;
        println!(
            "FAULT_PRE {tag} p0_before={p0_before} p0_after={p0_after} p1_before={p1_before} p1_after={p1_after}"
        );
    }
    link.check_boot(nonce)?;
    let before = same_epoch(&monitor, epoch)?;
    if before.cores.iter().any(|c| c.state != CoreState::Running) {
        return Err("core entered a terminal state before the request".into());
    }
    link.request(kind)?;
    println!("FAULT_REQUEST {tag} code={}", kind as u32);

    let start = Instant::now();
    let status = loop {
        let status = same_epoch(&monitor, epoch)?;
        if status.cores[core].state.is_terminal() || start.elapsed() >= STATUS_WAIT {
            break status;
        }
        sleep(Duration::from_millis(20));
    };
    link.check_boot(nonce)?;
    let site = link.read(SITE);
    if site != 0 && site != u32::MAX {
        println!(
            "FAULT_SITE {tag} code={} core={} ipsr={}",
            site >> SITE_KIND_SHIFT,
            (site >> SITE_CORE_BIT) & 1,
            site & SITE_IPSR_MASK
        );
    }
    let c = &status.cores[core];
    println!(
        "FAULT_STATUS {tag} core={core} ack={} state={:?} reason={} aux=0x{:x} pc_or_line=0x{:x} peer={:?} pcie={} scmi={}",
        link.read(ACK),
        c.state,
        c.reason,
        c.aux,
        c.line,
        status.cores[1 - core].state,
        status.pcie,
        status.scmi
    );
    if kind == Kind::Stkerr || !c.state.is_terminal() {
        return Ok(());
    }

    let [p0_before, p1_before] = link.ticks()?;
    let now = Instant::now();
    sleep(Duration::from_secs(1));
    let [p0_after, p1_after] = link.ticks()?;
    println!(
        "FAULT_TIMER {tag} elapsed_ms={} p0_before={p0_before} p0_after={p0_after} p1_before={p1_before} p1_after={p1_after}",
        now.elapsed().as_millis()
    );

    ping(&tag, host);
    usb(&tag, &format!("usb:{port}"));
    scmi(&tag, &monitor, epoch, n)?;
    let final_status = same_epoch(&monitor, epoch)?;
    link.check_boot(nonce)?;
    kmsg::report(&tag, kernel_log);
    println!(
        "FAULT_END {tag} pcie={} peer={:?}",
        final_status.pcie,
        final_status.cores[1 - core].state
    );
    Ok(())
}
