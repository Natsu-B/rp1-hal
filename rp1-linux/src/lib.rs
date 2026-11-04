//! Uncached UIO mappings. Only aligned volatile words ever touch Device memory.
use rp1_proto::{
    Error, Message, barrier,
    note::{Note, SHM_AREA, SHM_REGION, ShmArea, ShmKind, ShmRegion, Writer},
    shm, uio,
};
use sha2::{Digest, Sha256};
use std::{
    collections::HashSet,
    fs::{self, File, OpenOptions},
    os::fd::AsRawFd,
    path::{Path, PathBuf},
    ptr::NonNull,
    rc::Rc,
    time::{Duration, Instant},
};
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

struct Mapping {
    base: NonNull<u8>,
    len: usize,
    _file: File,
}
impl Mapping {
    fn open(name: &str, phys: u64, len: usize) -> Result<Rc<Self>> {
        let path = &find_uio(name)?;
        let hex = |field: &str| -> Result<u64> {
            let text = fs::read_to_string(path.join("maps/map0").join(field))?;
            Ok(u64::from_str_radix(
                text.trim().trim_start_matches("0x"),
                16,
            )?)
        };
        if hex("addr")? != phys || hex("size")? != len as u64 || hex("offset")? != 0 {
            return Err(format!("UIO {name} mapping differs from note/board table").into());
        }
        let file = open_exclusive(path)?;
        let pointer = unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                len,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_SHARED,
                file.as_raw_fd(),
                0,
            )
        };
        if pointer == libc::MAP_FAILED {
            return Err(std::io::Error::last_os_error().into());
        }
        Ok(Rc::new(Self {
            base: NonNull::new(pointer.cast()).ok_or("null mmap")?,
            len,
            _file: file,
        }))
    }
    fn ptr(&self) -> *mut u8 {
        self.base.as_ptr()
    }
    fn read(&self, offset: usize) -> u32 {
        assert!(offset.is_multiple_of(4) && offset + 4 <= self.len);
        unsafe { self.ptr().add(offset).cast::<u32>().read_volatile() }
    }
    fn write(&self, offset: usize, value: u32) {
        assert!(offset.is_multiple_of(4) && offset + 4 <= self.len);
        unsafe { self.ptr().add(offset).cast::<u32>().write_volatile(value) };
        barrier();
    }
}
/// The `/sys/class/uio` entry with this name; exactly one must exist.
fn find_uio(name: &str) -> Result<PathBuf> {
    let mut matches = Vec::new();
    for entry in fs::read_dir("/sys/class/uio")? {
        let path = entry?.path();
        if fs::read_to_string(path.join("name"))?.trim() == name {
            matches.push(path);
        }
    }
    if matches.len() != 1 {
        return Err(format!("expected one UIO named {name}, found {}", matches.len()).into());
    }
    Ok(matches.remove(0))
}
/// Open `/dev/uioN` for a sysfs entry, with an exclusive non-blocking `flock` (one owner).
fn open_exclusive(sys: &Path) -> Result<File> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(Path::new("/dev").join(sys.file_name().ok_or("invalid UIO path")?))?;
    if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    Ok(file)
}
impl Drop for Mapping {
    fn drop(&mut self) {
        unsafe {
            libc::munmap(self.ptr().cast(), self.len);
        }
    }
}

struct Session {
    header: shm::Header,
    _map: Rc<Mapping>,
    _link: Rc<Mapping>,
}
impl Session {
    fn check(&self) -> std::result::Result<(), Error> {
        self.header.check()
    }
}

/// One exclusive Linux owner. Handles retain their mappings even when Firmware is dropped.
pub struct Firmware {
    note: Vec<u8>,
    link: Rc<Mapping>,
    session: Rc<Session>,
    taken: HashSet<String>,
}
impl Firmware {
    /// Match the boot DT's note to UIO names/addresses and the published DDR note hash.
    /// If necessary, request cold-boot initialization after Linux PCIe probe is complete.
    pub fn open() -> Result<Self> {
        let bytes = fs::read("/sys/firmware/devicetree/base/rp1-firmware/rp1,note")?;
        let note = Note::parse(&bytes)?;
        let area = ShmArea::parse(note.get(SHM_AREA).ok_or(Error::Invalid)?.value)?;
        let link = Mapping::open(&format!("rp1:{}", uio::LINK), rp1_proto::LINK_PHYS, 0x4000)?;
        if link.read(rp1_proto::link::HANDOFF_MAGIC) != rp1_proto::link::MAGIC
            || link.read(rp1_proto::link::HANDOFF_PHYS_LOW) != area.phys as u32
            || link.read(rp1_proto::link::HANDOFF_PHYS_HIGH) != (area.phys >> 32) as u32
            || link.read(rp1_proto::link::HANDOFF_SIZE) != area.size
        {
            return Err("loader handoff differs from note".into());
        }
        link.write(rp1_proto::link::SHM_REQUEST, 1);
        let start = Instant::now();
        while link.read(rp1_proto::link::SHM_STATUS) != 1 {
            if start.elapsed() > Duration::from_secs(10) {
                return Err("shm initialization timed out".into());
            }
            std::thread::sleep(Duration::from_millis(1));
        }
        let map = Mapping::open(&format!("rp1:{}", uio::HEADER), area.phys, 65536)?;
        let digest: [u8; 32] = Sha256::digest(&bytes).into();
        let header = unsafe { shm::Header::open(map.ptr(), &digest) }?;
        if header.generation() != link.read(rp1_proto::link::SHM_GENERATION)
            || header.regions()
                != note
                    .records()
                    .filter_map(std::result::Result::ok)
                    .filter(|r| r.kind == SHM_REGION)
                    .count() as u32
        {
            return Err(Error::Mismatch.into());
        }
        let session = Rc::new(Session {
            header,
            _map: map,
            _link: link.clone(),
        });
        Ok(Self {
            note: bytes,
            link,
            session,
            taken: HashSet::new(),
        })
    }
    fn region<T: Message>(&self, name: &str) -> Result<ShmRegion<'_>> {
        self.session.check()?;
        if self.taken.contains(name) {
            return Err("endpoint already taken".into());
        }
        let note = Note::parse(&self.note)?;
        let r = note
            .records()
            .filter_map(std::result::Result::ok)
            .filter(|r| r.kind == SHM_REGION)
            .map(|r| ShmRegion::parse(r.value))
            .collect::<std::result::Result<Vec<_>, _>>()?
            .into_iter()
            .find(|r| r.name == name)
            .ok_or("unknown shm name")?;
        // notify=irq is valid only on RP1 -> Linux rings (rp1-build checks it; refuse others).
        if r.type_id != T::TYPE_ID
            || r.elem_size as usize != std::mem::size_of::<T>()
            || (r.notify_irq && r.kind != ShmKind::RingToLinux)
        {
            return Err(Error::Mismatch.into());
        }
        Ok(r)
    }
    pub fn ring_rx<T: Message>(&mut self, name: &str) -> Result<RingRx<T>> {
        let r = self.region::<T>(name)?;
        if r.kind != ShmKind::RingToLinux {
            return Err(Error::Mismatch.into());
        }
        let map = Mapping::open(
            &format!("rp1:{name}"),
            rp1_proto::SHM_PHYS + u64::from(r.ddr_offset),
            r.ddr_len as usize,
        )?;
        let inner = unsafe {
            shm::RingRx::attach(
                map.ptr(),
                map.len,
                self.link.ptr().add(r.sram_offset as usize),
                r.sram_len as usize,
                r.depth,
                overflow(r.overwrite),
            )
        }?;
        let notify_irq = r.notify_irq;
        self.taken.insert(name.into());
        Ok(RingRx {
            inner,
            notify_irq,
            spin: Duration::ZERO,
            stats: WaitStats::default(),
            _map: map,
            session: self.session.clone(),
        })
    }
    pub fn ring_tx<T: Message>(&mut self, name: &str) -> Result<RingTx<T>> {
        let r = self.region::<T>(name)?;
        if r.kind != ShmKind::RingToRp1 {
            return Err(Error::Mismatch.into());
        }
        let map = Mapping::open(
            &format!("rp1:{name}"),
            rp1_proto::SHM_PHYS + u64::from(r.ddr_offset),
            r.ddr_len as usize,
        )?;
        let inner = unsafe {
            shm::RingTx::attach(
                self.link.ptr().add(r.sram_offset as usize),
                r.sram_len as usize,
                map.ptr(),
                map.len,
                r.depth,
                overflow(r.overwrite),
            )
        }?;
        self.taken.insert(name.into());
        Ok(RingTx {
            inner,
            _map: map,
            session: self.session.clone(),
        })
    }
    pub fn block<T: Message>(&mut self, name: &str) -> Result<BlockMap<T>> {
        let r = self.region::<T>(name)?;
        let ShmKind::Block { writer } = r.kind else {
            return Err(Error::Mismatch.into());
        };
        let map = Mapping::open(
            &format!("rp1:{name}"),
            rp1_proto::SHM_PHYS + u64::from(r.ddr_offset),
            r.ddr_len as usize,
        )?;
        let inner = unsafe { shm::Block::attach(map.ptr(), map.len) }?;
        self.taken.insert(name.into());
        Ok(BlockMap {
            inner,
            writer,
            _map: map,
            session: self.session.clone(),
        })
    }
    pub fn generation(&self) -> u32 {
        self.session.header.generation()
    }
}
fn overflow(overwrite: bool) -> shm::Overflow {
    if overwrite {
        shm::Overflow::Overwrite
    } else {
        shm::Overflow::Reject
    }
}

/// Longest single sleep in [`RingRx::wait`]. A wake is a hint: an edge can be lost (before
/// the UIO is first enabled, or while the vector's hardware source is held) or merged, so the
/// ring is re-checked at least this often even without one (protocol.md §3).
pub const RECHECK: Duration = Duration::from_millis(10);

/// The `rp1:irq` UIO node: the RP1 -> Linux interrupt (MSI-X vector `rp1_proto::irq::VECTOR`,
/// raised by the core that declares `host_irq = true`). One owner (exclusive `flock`); one
/// `Irq` serves every `notify = "irq"` ring, so a wake for one ring is a spurious wake for
/// another, which [`RingRx::wait`] tolerates.
pub struct Irq {
    file: File,
    wakes: u64,
    count: u32,
}
impl Irq {
    /// Open `rp1:irq` and enable the interrupt (`uio_pdrv_genirq` starts it enabled, and masks
    /// it again in its handler; pulses before the first enable are lost).
    pub fn open() -> Result<Self> {
        let file = open_exclusive(&find_uio(&format!("rp1:{}", uio::IRQ))?)?;
        let mut irq = Self {
            file,
            wakes: 0,
            count: 0,
        };
        irq.enable()?;
        Ok(irq)
    }
    fn enable(&mut self) -> std::io::Result<()> {
        let one = 1u32.to_ne_bytes();
        let n = unsafe { libc::write(self.file.as_raw_fd(), one.as_ptr().cast(), 4) };
        if n != 4 {
            return Err(std::io::Error::last_os_error());
        }
        Ok(())
    }
    /// Sleep up to `timeout` (rounded up to whole ms) for an interrupt. Returns whether one
    /// arrived; it is consumed and the line re-enabled before returning, so an edge from then
    /// on wakes the next call (edges while masked merge into one pending wake).
    pub fn wait(&mut self, timeout: Duration) -> std::io::Result<bool> {
        let mut pfd = libc::pollfd {
            fd: self.file.as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        };
        let ms = timeout.as_micros().div_ceil(1000).min(i32::MAX as u128) as i32;
        let r = unsafe { libc::poll(&mut pfd, 1, ms) };
        if r < 0 {
            let e = std::io::Error::last_os_error();
            return if e.kind() == std::io::ErrorKind::Interrupted {
                Ok(false)
            } else {
                Err(e)
            };
        }
        if r == 0 {
            return Ok(false);
        }
        let mut count = [0u8; 4];
        let n = unsafe { libc::read(self.file.as_raw_fd(), count.as_mut_ptr().cast(), 4) };
        if n != 4 {
            return Err(std::io::Error::last_os_error());
        }
        self.count = u32::from_ne_bytes(count);
        self.wakes += 1;
        self.enable()?;
        Ok(true)
    }
    /// Interrupts consumed by [`Irq::wait`] so far.
    pub fn wakes(&self) -> u64 {
        self.wakes
    }
    /// The UIO's total event count at the last wake (the kernel's count, including events
    /// that merged into one wake).
    pub fn event_count(&self) -> u32 {
        self.count
    }
}

/// Counters of [`RingRx::wait`] (observational).
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WaitStats {
    /// Sleeps (the ring was empty after arming).
    pub sleeps: u64,
    /// Sleeps ended by an interrupt.
    pub woken: u64,
    /// Sleeps that ran to their bound ([`RECHECK`] or the deadline) without one.
    pub rechecks: u64,
    /// Of `rechecks`, those after which messages were available: data found by the re-check
    /// rather than a wake (a lost, merged-away or slower-than-`RECHECK` notification).
    pub late: u64,
}

pub struct RingRx<T> {
    inner: shm::RingRx<T>,
    notify_irq: bool,
    spin: Duration,
    stats: WaitStats,
    _map: Rc<Mapping>,
    session: Rc<Session>,
}
impl<T: Message> RingRx<T> {
    pub fn pop(&mut self) -> std::result::Result<Option<T>, Error> {
        self.session.check()?;
        let r = self.inner.pop()?;
        self.session.check()?;
        Ok(r)
    }
    /// Batch form of [`RingRx::pop`]: one `tail` store per call (`rp1_proto::shm::RingRx::pop_into`).
    pub fn pop_into(&mut self, out: &mut [T]) -> std::result::Result<usize, Error> {
        self.session.check()?;
        let n = self.inner.pop_into(out)?;
        self.session.check()?;
        Ok(n)
    }
    pub fn dropped(&self) -> u32 {
        self.inner.dropped()
    }
    /// Messages available now.
    pub fn available(&self) -> u32 {
        self.inner.available()
    }
    /// Block until messages are available or `timeout` passes (`None`: no deadline); returns
    /// whether some are. For rings declared `notify = "irq"` (otherwise `Error::Invalid`).
    ///
    /// Handshake (protocol.md §2.3): arm the sleep counter, read it back, re-check `head`, and
    /// only then sleep on `irq`; RP1 drains `head` before it samples the counter. Each sleep
    /// is bounded by [`RECHECK`], because a wake is only a hint.
    pub fn wait(&mut self, irq: &mut Irq, timeout: Option<Duration>) -> Result<bool> {
        if !self.notify_irq {
            return Err(Error::Invalid.into());
        }
        let deadline = timeout.and_then(|t| Instant::now().checked_add(t));
        if !self.spin.is_zero() {
            let start = Instant::now();
            while start.elapsed() < self.spin {
                if self.inner.available() != 0 {
                    return Ok(true);
                }
                std::hint::spin_loop();
            }
        }
        loop {
            self.session.check()?;
            let bound = deadline
                .map_or(RECHECK, |d| d.saturating_duration_since(Instant::now()))
                .min(RECHECK);
            let mut io = Ok(false);
            let outcome = self.inner.wait_with(|| {
                io = irq.wait(bound);
                matches!(io, Ok(true))
            });
            io?;
            match outcome {
                shm::Wait::Ready => return Ok(true),
                shm::Wait::Woken => self.stats.woken += 1,
                shm::Wait::Timeout => {
                    self.stats.rechecks += 1;
                    if self.inner.available() != 0 {
                        self.stats.late += 1;
                    }
                }
            }
            self.stats.sleeps += 1;
            if deadline.is_some_and(|d| Instant::now() >= d) {
                return Ok(self.inner.available() != 0);
            }
        }
    }
    /// Poll for up to `spin` before the first arm of each [`RingRx::wait`] (default zero).
    /// With a producer that publishes a message every few µs, arming right away is mostly
    /// cancelled by the re-check, yet RP1 may still see the odd counter and pulse (one PCIe
    /// round trip each way per arm); a short spin trades consumer CPU for fewer arms.
    pub fn set_spin(&mut self, spin: Duration) {
        self.spin = spin;
    }
    pub fn wait_stats(&self) -> WaitStats {
        self.stats
    }
}
pub struct RingTx<T> {
    inner: shm::RingTx<T>,
    _map: Rc<Mapping>,
    session: Rc<Session>,
}
impl<T: Message> RingTx<T> {
    pub fn push(&mut self, value: &T) -> std::result::Result<(), Error> {
        self.session.check()?;
        self.inner.push(value)
    }
    /// Batch form of [`RingTx::push`]: one `head` store per call (`rp1_proto::shm::RingTx::push_slice`).
    pub fn push_slice(&mut self, values: &[T]) -> std::result::Result<usize, Error> {
        self.session.check()?;
        Ok(self.inner.push_slice(values))
    }
    pub fn dropped(&self) -> u32 {
        self.inner.dropped()
    }
}
pub struct BlockMap<T> {
    inner: shm::Block<T>,
    writer: Writer,
    _map: Rc<Mapping>,
    session: Rc<Session>,
}
impl<T: Message> BlockMap<T> {
    pub fn read(&self) -> std::result::Result<T, Error> {
        self.session.check()?;
        let r = self.inner.read()?;
        self.session.check()?;
        Ok(r)
    }
    pub fn read_with<R>(
        &self,
        f: impl FnOnce(&shm::BlockReader<'_, T>) -> R,
    ) -> std::result::Result<R, Error> {
        self.session.check()?;
        let r = self.inner.read_with(f)?;
        self.session.check()?;
        Ok(r)
    }
    pub fn write(&mut self, value: &T) -> std::result::Result<(), Error> {
        self.session.check()?;
        if self.writer != Writer::Linux {
            return Err(Error::Invalid);
        }
        unsafe {
            self.inner.write(value);
        }
        Ok(())
    }
    /// # Safety
    /// For writer=both, the caller must exclude the RP1 writer for the entire section.
    pub unsafe fn begin_write(&mut self) -> std::result::Result<shm::BlockWriter<'_, T>, Error> {
        self.session.check()?;
        if self.writer == Writer::Rp1 {
            return Err(Error::Invalid);
        }
        Ok(unsafe { self.inner.begin_write() })
    }
}
