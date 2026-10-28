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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 3 {
        return Err(
            "usage: linux-doorbell /dev/uio<link> /dev/uio<doorbell> /dev/uio<events>".into(),
        );
    }
    let mut link = Mapping::open(Path::new(&args[0]), "rp1:link")?;
    let mut gpio = Mapping::open(Path::new(&args[1]), "rp1:doorbell")?;
    let mut events = Mapping::open(Path::new(&args[2]), "rp1:events")?;
    for sequence in 1..=10u32 {
        events.write(0x300c, 1 << 1);
        link.write(0x10, sequence);
        // Same-window read completes the posted SRAM write before a BAR1 doorbell.
        if link.read(0x10) != sequence {
            return Err("posted write readback mismatch".into());
        }
        gpio.write(0x2114, 1 << 27);
        let start = Instant::now();
        while events.read(0x0c) & (1 << 1) == 0 {
            if start.elapsed() > Duration::from_secs(1) {
                return Err("RP1 doorbell reply timed out".into());
            }
            std::thread::yield_now();
        }
        rp1_proto::barrier();
        if link.read(0x14) != sequence {
            return Err("RP1 echo mismatch".into());
        }
        events.write(0x300c, 1 << 1);
        println!("doorbell {sequence}: {:?}", start.elapsed());
    }
    Ok(())
}
