//! Uncached UIO mappings. Only aligned volatile words ever touch Device memory.
use rp1_proto::{
    self as proto, Message, barrier,
    note::{LINK, Link, Note, SHM_AREA, SHM_REGION, ShmArea, ShmKind, ShmRegion, Writer},
    shm, uio,
};
use sha2::{Digest, Sha256};
use std::{
    cell::RefCell,
    fs::{self, File, OpenOptions},
    os::fd::AsRawFd,
    path::{Path, PathBuf},
    ptr::NonNull,
    rc::Rc,
    time::{Duration, Instant},
};
mod error;
mod notify;
mod status;
pub use error::{Error, Result};
use notify::{Doorbell, Irq};
pub use status::{CoreState, CoreStatus, Health, Monitor, PcieState, ScmiState, ShmState, Status};

/// The firmware's note as the loader put it in the boot DT.
const NOTE: &str = "/sys/firmware/devicetree/base/rp1-firmware/rp1,note";

struct Mapping {
    base: NonNull<u8>,
    len: usize,
    _file: File,
}
/// How a UIO node is opened.
#[derive(Clone, Copy)]
enum Access {
    /// Read-write, holding the node's exclusive `flock`: an endpoint has one owner.
    Exclusive,
    /// Read-write without a lock (`rp1:link`, `rp1:header`).
    Shared,
    /// Read-only without a lock.
    ReadOnly,
}
impl Mapping {
    /// Map the UIO node `name` after checking its address and size.
    fn open(name: &str, phys: u64, len: usize, access: Access) -> Result<Rc<Self>> {
        let dev = dev_path(&find_mapping(name, phys, len)?)?;
        let (file, prot) = match access {
            Access::Exclusive => (open_exclusive(&dev)?, libc::PROT_READ | libc::PROT_WRITE),
            Access::Shared => (open_read_write(&dev)?, libc::PROT_READ | libc::PROT_WRITE),
            Access::ReadOnly => (File::open(dev)?, libc::PROT_READ),
        };
        let pointer = unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                len,
                prot,
                libc::MAP_SHARED,
                file.as_raw_fd(),
                0,
            )
        };
        if pointer == libc::MAP_FAILED {
            return Err(std::io::Error::last_os_error().into());
        }
        Ok(Rc::new(Self {
            base: NonNull::new(pointer.cast()).ok_or_else(|| Error::setup("mmap returned null"))?,
            len,
            _file: file,
        }))
    }
    /// `rp1:link`, the link page.
    fn link(access: Access) -> Result<Rc<Self>> {
        Self::open(
            &format!("rp1:{}", uio::LINK),
            proto::LINK_PHYS,
            LINK_PAGE,
            access,
        )
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
/// Bytes of the `rp1:link` mapping.
const LINK_PAGE: usize = 0x4000;
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
        return Err(Error::setup(format!(
            "expected one UIO named {name}, found {}",
            matches.len()
        )));
    }
    Ok(matches.remove(0))
}
/// The UIO named `name`, checked against the note's / board table's address and size.
fn find_mapping(name: &str, phys: u64, len: usize) -> Result<PathBuf> {
    let path = find_uio(name)?;
    let hex = |field: &str| -> Result<u64> {
        let text = fs::read_to_string(path.join("maps/map0").join(field))?;
        u64::from_str_radix(text.trim().trim_start_matches("0x"), 16)
            .map_err(|_| Error::setup(format!("UIO {name} {field} is not hex: {text:?}")))
    };
    if hex("addr")? != phys || hex("size")? != len as u64 || hex("offset")? != 0 {
        return Err(Error::setup(format!(
            "UIO {name} mapping differs from note/board table"
        )));
    }
    Ok(path)
}
/// `/dev/uioN` for a `/sys/class/uio/uioN` entry.
fn dev_path(sys: &Path) -> Result<PathBuf> {
    let name = sys
        .file_name()
        .ok_or_else(|| Error::setup("invalid UIO path"))?;
    Ok(Path::new("/dev").join(name))
}
fn open_read_write(dev: &Path) -> Result<File> {
    Ok(OpenOptions::new().read(true).write(true).open(dev)?)
}
/// [`open_read_write`] with an exclusive non-blocking `flock`. `Err(Busy)` while another open
/// file holds it, also one of this process.
fn open_exclusive(dev: &Path) -> Result<File> {
    let file = open_read_write(dev)?;
    if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
        let e = std::io::Error::last_os_error();
        return Err(if e.kind() == std::io::ErrorKind::WouldBlock {
            Error::Busy
        } else {
            e.into()
        });
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

/// One enabled shared memory: its header, checked against the note.
struct Session {
    header: shm::Header,
    _map: Rc<Mapping>,
    link: Rc<Mapping>,
}
impl Session {
    /// Map `rp1:header` and check it against the note and the link page.
    fn open(note: &[u8], area: &ShmArea, link: &Rc<Mapping>) -> Result<Self> {
        let map = Mapping::open(
            &format!("rp1:{}", uio::HEADER),
            area.phys,
            65536,
            Access::Shared,
        )?;
        let digest: [u8; 32] = Sha256::digest(note).into();
        let header = unsafe { shm::Header::open(map.ptr(), &digest) }?;
        let regions = Note::parse(note)?
            .records()
            .filter_map(std::result::Result::ok)
            .filter(|r| r.kind == SHM_REGION)
            .count() as u32;
        if header.generation() != link.read(proto::link::SHM_GENERATION)
            || header.regions() != regions
        {
            return Err(proto::Error::Mismatch.into());
        }
        Ok(Self {
            header,
            _map: map,
            link: link.clone(),
        })
    }
    /// The shared memory is still the one this session opened.
    fn check(&self) -> Result<()> {
        Ok(self.header.check()?)
    }
}

/// The firmware the boot DT's note describes, reached through `rp1:link`.
///
/// Shared: any number of processes may open it at once. Each endpoint has one owner, the
/// holder of its UIO node's `flock`; opening a held endpoint is `Err(Busy)`, also within one
/// process. Endpoints keep their mappings after the `Firmware` is dropped.
pub struct Firmware {
    note: Vec<u8>,
    area: ShmArea,
    link: Rc<Mapping>,
    monitor: Monitor,
    session: RefCell<Option<Rc<Session>>>,
}
impl Firmware {
    /// Map `rp1:link` and check the loader's handoff against the boot DT's note. Does not
    /// enable shared memory and does not wait: [`Firmware::status`] works right away,
    /// endpoints after [`Firmware::wait_ready`].
    pub fn open() -> Result<Self> {
        let note = fs::read(NOTE)?;
        let area = ShmArea::parse(
            Note::parse(&note)?
                .get(SHM_AREA)
                .ok_or(proto::Error::Invalid)?
                .value,
        )?;
        let link = Mapping::link(Access::Shared)?;
        let magic = link.read(proto::link::HANDOFF_MAGIC);
        if magic == u32::MAX {
            return Err(Error::LinkDown);
        }
        if magic != proto::link::MAGIC
            || link.read(proto::link::HANDOFF_PHYS_LOW) != area.phys as u32
            || link.read(proto::link::HANDOFF_PHYS_HIGH) != (area.phys >> 32) as u32
            || link.read(proto::link::HANDOFF_SIZE) != area.size
        {
            return Err(Error::setup("loader handoff differs from note"));
        }
        Ok(Self {
            monitor: Monitor::new(link.clone(), &note)?,
            note,
            area,
            link,
            session: RefCell::new(None),
        })
    }
    /// Ask RP1 to enable shared memory, wait up to `timeout` until it is ready, and return
    /// its generation (`Err(Timeout)` if it is not ready by then). Cheap once ready: later
    /// calls only check that the shared memory is still the same, and open it again after
    /// the firmware re-initialized it.
    pub fn wait_ready(&self, timeout: Duration) -> Result<u32> {
        if let Some(session) = self.session.borrow().as_ref()
            && session.check().is_ok()
        {
            return Ok(session.header.generation());
        }
        self.link.write(proto::link::SHM_REQUEST, 1);
        let deadline = Instant::now() + timeout;
        loop {
            match self.link.read(proto::link::SHM_STATUS) {
                proto::link::SHM_READY => break,
                u32::MAX => return Err(Error::LinkDown),
                _ if Instant::now() >= deadline => return Err(Error::Timeout),
                _ => std::thread::sleep(Duration::from_millis(1)),
            }
        }
        let session = Session::open(&self.note, &self.area, &self.link)?;
        let generation = session.header.generation();
        *self.session.borrow_mut() = Some(Rc::new(session));
        Ok(generation)
    }
    /// The session [`Firmware::wait_ready`] opened; `Err(Proto(NotReady))` before.
    fn session(&self) -> Result<Rc<Session>> {
        let session = self
            .session
            .borrow()
            .clone()
            .ok_or(proto::Error::NotReady)?;
        session.check()?;
        Ok(session)
    }
    /// The note's region `name`, checked against the message type.
    fn region<T: Message>(&self, name: &str) -> Result<ShmRegion<'_>> {
        let r = Note::parse(&self.note)?
            .records()
            .filter_map(std::result::Result::ok)
            .filter(|r| r.kind == SHM_REGION)
            // Note::parse validated every region.
            .filter_map(|r| ShmRegion::parse(r.value).ok())
            .find(|r| r.name == name)
            .ok_or(Error::Unknown)?;
        if r.type_id != T::TYPE_ID || r.elem_size as usize != std::mem::size_of::<T>() {
            return Err(proto::Error::Mismatch.into());
        }
        Ok(r)
    }
    /// [`Firmware::region`] for a ring of `kind`. A name with `-` is unknown here:
    /// `<name>-dn` and `<name>-up` are the two rings of a channel or rpc, and only that
    /// endpoint opens them.
    fn ring<T: Message>(&self, name: &str, kind: ShmKind) -> Result<ShmRegion<'_>> {
        if name.contains('-') {
            return Err(Error::Unknown);
        }
        let r = self.region::<T>(name)?;
        if r.kind != kind {
            return Err(proto::Error::Mismatch.into());
        }
        Ok(r)
    }
    /// The DDR part of region `r`, as its exclusive owner.
    fn map(r: &ShmRegion<'_>) -> Result<Rc<Mapping>> {
        Mapping::open(
            &format!("rp1:{}", r.name),
            proto::SHM_PHYS + u64::from(r.ddr_offset),
            r.ddr_len as usize,
            Access::Exclusive,
        )
    }
    /// The RP1 -> Linux ring `name`.
    pub fn ring_rx<T: Message>(&self, name: &str) -> Result<RingRx<T>> {
        let session = self.session()?;
        let r = self.ring::<T>(name, ShmKind::RingToLinux)?;
        let map = Self::map(&r)?;
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
        Ok(RingRx {
            inner,
            irq: r.notify_irq.then(Irq::open).transpose()?,
            peer: Peer {
                link: session.link.clone(),
                core: r.core - 1,
            },
            spin: Duration::ZERO,
            stats: WaitStats::default(),
            _map: map,
            session,
        })
    }
    /// The Linux -> RP1 ring `name`.
    pub fn ring_tx<T: Message>(&self, name: &str) -> Result<RingTx<T>> {
        let session = self.session()?;
        let r = self.ring::<T>(name, ShmKind::RingToRp1)?;
        let map = Self::map(&r)?;
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
        let doorbell = r
            .notify_irq
            .then(|| self.doorbell(r.core - 1))
            .transpose()?;
        Ok(RingTx::open(inner, doorbell, map, session))
    }
    /// The doorbell of `core` (0 = proc0) that the note's LINK record declares.
    fn doorbell(&self, core: u8) -> Result<Doorbell> {
        let record = Note::parse(&self.note)?.get(LINK).ok_or(Error::Unknown)?;
        let pin = Link::parse(record.value)?.cores[core as usize]
            .doorbell
            .ok_or(proto::Error::Mismatch)?;
        Doorbell::open(core.into(), pin)
    }
    /// The block `name`.
    pub fn block<T: Message>(&self, name: &str) -> Result<BlockMap<T>> {
        let session = self.session()?;
        let r = self.region::<T>(name)?;
        let ShmKind::Block { writer } = r.kind else {
            return Err(proto::Error::Mismatch.into());
        };
        let map = Self::map(&r)?;
        let inner = unsafe { shm::Block::attach(map.ptr(), map.len) }?;
        Ok(BlockMap {
            inner,
            writer,
            _map: map,
            session,
        })
    }
    /// Sleep until one of `endpoints` has messages or `timeout` passes (`None`: no deadline);
    /// returns whether one has. Each round arms every ring and re-checks them all, then sleeps
    /// once: on `rp1:irq` up to [`RECHECK`] when all are irq rings, at most [`POLL`] when one
    /// is a poll ring. The caller then receives from every endpoint. No spin, and
    /// [`RingRx::wait_stats`] is not updated. `Err(PeerDown)` once a serving core is down.
    pub fn wait_any(
        &self,
        endpoints: &mut [&mut dyn Waitable],
        timeout: Option<Duration>,
    ) -> Result<bool> {
        let deadline = deadline(timeout);
        loop {
            for endpoint in endpoints.iter() {
                endpoint.check()?;
            }
            let (mut waiters, irqs): (Vec<_>, Vec<_>) =
                endpoints.iter_mut().map(|e| e.parts()).unzip();
            if deadline.is_some_and(|d| Instant::now() >= d) {
                return Ok(waiters.iter().any(|w| w.available() != 0));
            }
            let polled = irqs.iter().any(Option::is_none);
            let mut irq = irqs.into_iter().flatten().next();
            let bound = remaining(deadline, if polled { POLL } else { RECHECK });
            let mut io = Ok(false);
            let outcome = shm::wait_any_with(&mut waiters, || match &mut irq {
                Some(irq) => {
                    io = irq.wait(bound);
                    matches!(io, Ok(true))
                }
                None => {
                    std::thread::sleep(bound);
                    false
                }
            });
            io?;
            if outcome == shm::Wait::Ready {
                return Ok(true);
            }
        }
    }
    /// [`Monitor::status`] through this firmware's link mapping.
    pub fn status(&self) -> Result<Status> {
        self.monitor.status()
    }
    /// [`Monitor::health`] through this firmware's link mapping.
    pub fn health(&self, window: Duration) -> Result<Health> {
        self.monitor.health(window)
    }
}
fn overflow(overwrite: bool) -> shm::Overflow {
    if overwrite {
        shm::Overflow::Overwrite
    } else {
        shm::Overflow::Reject
    }
}

/// Longest single sleep of a wait on a `notify = "irq"` ring. A wake is a hint: an edge can be
/// lost (before the UIO is first enabled, or while the vector's hardware source is held) or
/// merged, so the ring is re-checked at least this often even without one (protocol.md §3).
pub const RECHECK: Duration = Duration::from_millis(10);
/// Longest single sleep of a wait on a `notify = "poll"` ring: its writer never notifies, so
/// every sleep ends with a re-check.
pub const POLL: Duration = Duration::from_millis(1);

/// The core that serves an endpoint, seen through its status words.
struct Peer {
    link: Rc<Mapping>,
    /// 0 = proc0.
    core: u8,
}
impl Peer {
    /// `Err(PeerDown)` once the core stopped, panicked or faulted (one link-page read while it
    /// runs), `Err(LinkDown)` when its state reads all ones.
    fn check(&self) -> Result<()> {
        let block = proto::status::BLOCKS[self.core as usize];
        let word = self.link.read(block + proto::status::STATE);
        if word == u32::MAX {
            return Err(Error::LinkDown);
        }
        let state = CoreState::from_word(word);
        if !state.is_terminal() {
            return Ok(());
        }
        // The core wrote the reason before the terminal state.
        barrier();
        Err(Error::PeerDown {
            core: self.core,
            state,
            reason: self.link.read(block + proto::status::REASON),
        })
    }
}

/// When a wait that may take `timeout` ends (`None`: never).
fn deadline(timeout: Option<Duration>) -> Option<Instant> {
    timeout.and_then(|t| Instant::now().checked_add(t))
}
/// One sleep's bound: `longest`, or less when the deadline is closer.
fn remaining(deadline: Option<Instant>, longest: Duration) -> Duration {
    deadline
        .map_or(longest, |d| d.saturating_duration_since(Instant::now()))
        .min(longest)
}

/// Counters of the waits of a [`RingRx`] (observational).
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WaitStats {
    /// Sleeps (the ring was empty after arming).
    pub sleeps: u64,
    /// Sleeps ended by an interrupt.
    pub woken: u64,
    /// Sleeps that ran to their bound ([`RECHECK`], [`POLL`] or the deadline) without one.
    pub rechecks: u64,
    /// Of `rechecks`, those after which messages were available: data found by the re-check
    /// rather than a wake (a lost, merged-away or slower-than-`RECHECK` notification, or any
    /// message of a poll ring).
    pub late: u64,
}

/// The reading end of an RP1 -> Linux ring. A `notify = "irq"` ring sleeps on `rp1:irq`
/// between messages; a `notify = "poll"` ring sleeps [`POLL`] at a time.
pub struct RingRx<T> {
    inner: shm::RingRx<T>,
    /// `notify = "irq"`: the interrupt its writer raises.
    irq: Option<Irq>,
    peer: Peer,
    spin: Duration,
    stats: WaitStats,
    _map: Rc<Mapping>,
    session: Rc<Session>,
}
impl<T: Message> RingRx<T> {
    /// The oldest message, without waiting.
    pub fn try_recv(&mut self) -> Result<Option<T>> {
        self.session.check()?;
        let r = self.inner.pop()?;
        self.session.check()?;
        Ok(r)
    }
    /// The oldest message, waiting up to `timeout` for one (`None`: no deadline). `Ok(None)`
    /// when the deadline passed first.
    pub fn recv(&mut self, timeout: Option<Duration>) -> Result<Option<T>> {
        let deadline = deadline(timeout);
        loop {
            if let Some(value) = self.try_recv()? {
                return Ok(Some(value));
            }
            if !self.wait_until(deadline)? {
                return Ok(None);
            }
        }
    }
    /// As many messages as fit in `out`, without waiting: one `tail` store per call
    /// (`rp1_proto::shm::RingRx::pop_into`).
    pub fn try_recv_into(&mut self, out: &mut [T]) -> Result<usize> {
        self.session.check()?;
        let n = self.inner.pop_into(out)?;
        self.session.check()?;
        Ok(n)
    }
    /// [`RingRx::try_recv_into`], waiting up to `timeout` for the first message (`None`: no
    /// deadline). `Ok(0)` when the deadline passed first.
    pub fn recv_into(&mut self, out: &mut [T], timeout: Option<Duration>) -> Result<usize> {
        let deadline = deadline(timeout);
        loop {
            let n = self.try_recv_into(out)?;
            if n != 0 || out.is_empty() || !self.wait_until(deadline)? {
                return Ok(n);
            }
        }
    }
    /// [`RingRx::try_recv_into`] that keeps the messages read before an error.
    pub fn try_recv_into_report(&mut self, out: &mut [T]) -> (usize, Option<Error>) {
        if let Err(e) = self.session.check() {
            return (0, Some(e));
        }
        let (n, error) = self.inner.pop_into_report(out);
        let error = error.map(Error::from);
        (n, error.or_else(|| self.session.check().err()))
    }
    /// Newest published slot and its index, without consuming it (reject rings only).
    pub fn newest(&self) -> Result<Option<(u32, Result<T>)>> {
        self.session.check()?;
        let r = self.inner.newest()?;
        self.session.check()?;
        Ok(r.map(|(index, value)| (index, value.map_err(Error::from))))
    }
    pub fn tail(&self) -> u32 {
        self.inner.tail()
    }
    pub fn dropped(&self) -> u32 {
        self.inner.dropped()
    }
    /// Messages available now.
    pub fn available(&self) -> u32 {
        self.inner.available()
    }
    /// Wait until messages are available or `timeout` passes (`None`: no deadline), without
    /// taking any; returns whether some are.
    ///
    /// Spins first (see [`RingRx::set_spin`]), then sleeps. An irq ring arms its sleep
    /// counter, reads it back and re-checks `head` before it sleeps on `rp1:irq`, and RP1
    /// drains `head` before it samples the counter (protocol.md §2.3); each sleep is bounded
    /// by [`RECHECK`] because a wake is only a hint. Before each sleep it reads the serving
    /// core's state: `Err(PeerDown)` once that core is down.
    pub fn wait(&mut self, timeout: Option<Duration>) -> Result<bool> {
        self.wait_until(deadline(timeout))
    }
    fn wait_until(&mut self, deadline: Option<Instant>) -> Result<bool> {
        self.session.check()?;
        if self.spin_until(deadline)? {
            return Ok(true);
        }
        loop {
            self.session.check()?;
            if deadline.is_some_and(|d| Instant::now() >= d) {
                return Ok(self.inner.available() != 0);
            }
            self.peer.check()?;
            let outcome = match &mut self.irq {
                Some(irq) => {
                    let bound = remaining(deadline, RECHECK);
                    let mut io = Ok(false);
                    let outcome = self.inner.wait_with(|| {
                        io = irq.wait(bound);
                        matches!(io, Ok(true))
                    });
                    io?;
                    outcome
                }
                // Its writer never looks at the sleep counter: just sleep.
                None if self.inner.available() != 0 => shm::Wait::Ready,
                None => {
                    std::thread::sleep(remaining(deadline, POLL));
                    shm::Wait::Timeout
                }
            };
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
        }
    }
    /// Poll for up to the spin, never past `deadline`; whether messages arrived.
    fn spin_until(&self, deadline: Option<Instant>) -> Result<bool> {
        if self.spin.is_zero() {
            return Ok(false);
        }
        // `None` (spin too long to represent, no deadline): until data.
        let end = match (Instant::now().checked_add(self.spin), deadline) {
            (Some(s), Some(d)) => Some(s.min(d)),
            (s, d) => s.or(d),
        };
        while end.is_none_or(|e| Instant::now() < e) {
            self.session.check()?;
            if self.inner.available() != 0 {
                return Ok(true);
            }
            std::hint::spin_loop();
        }
        Ok(false)
    }
    /// Poll for up to `spin` (never past the wait's deadline) before each wait sleeps
    /// (default zero).
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

mod sealed {
    use rp1_proto::shm::Waiter;

    pub trait Waitable {
        /// The session and the serving core are still up.
        fn check(&self) -> crate::Result<()>;
        /// The ring's wait handle and, for an irq ring, its interrupt.
        fn parts(&mut self) -> (&mut dyn Waiter, Option<&mut crate::notify::Irq>);
    }
}
/// An endpoint [`Firmware::wait_any`] waits on: a [`RingRx`] of any message type.
pub trait Waitable: sealed::Waitable {}
impl<T: Message> Waitable for RingRx<T> {}
impl<T: Message> sealed::Waitable for RingRx<T> {
    fn check(&self) -> Result<()> {
        self.session.check()?;
        self.peer.check()
    }
    fn parts(&mut self) -> (&mut dyn shm::Waiter, Option<&mut Irq>) {
        (&mut self.inner, self.irq.as_mut())
    }
}

/// The writing end of a Linux -> RP1 ring. On a `notify = "irq"` ring every send that finds
/// the reader asleep rings its core's doorbell (protocol.md §3).
pub struct RingTx<T> {
    inner: shm::RingTx<T>,
    /// `notify = "irq"`: the reader's doorbell.
    doorbell: Option<Doorbell>,
    _map: Rc<Mapping>,
    session: Rc<Session>,
}
impl<T: Message> RingTx<T> {
    /// The writer over an attached ring. Rings the doorbell once if the reader already
    /// sleeps: a previous writer may have published `head` and ended before it rang.
    fn open(
        inner: shm::RingTx<T>,
        doorbell: Option<Doorbell>,
        map: Rc<Mapping>,
        session: Rc<Session>,
    ) -> Self {
        let tx = Self {
            inner,
            doorbell,
            _map: map,
            session,
        };
        tx.wake();
        tx
    }
    /// Ring the doorbell if the reader sleeps. `reader_asleep` first reads `head` back, so
    /// what was sent has reached RP1 before the doorbell does.
    fn wake(&self) {
        if let Some(doorbell) = &self.doorbell
            && self.inner.reader_asleep()
        {
            doorbell.ring();
        }
    }
    /// Publish one message and wake the reader if it sleeps. `Err(Proto(Full))`: no room
    /// (counted in [`RingTx::dropped`]).
    pub fn send(&mut self, value: &T) -> Result<()> {
        self.session.check()?;
        self.inner.push(value)?;
        self.wake();
        Ok(())
    }
    /// Publish as many of `values` as fit (one `head` store,
    /// `rp1_proto::shm::RingTx::push_slice`) and wake the reader if it sleeps.
    pub fn send_slice(&mut self, values: &[T]) -> Result<usize> {
        self.session.check()?;
        let n = self.inner.push_slice(values);
        if n != 0 {
            self.wake();
        }
        Ok(n)
    }
    pub fn dropped(&self) -> u32 {
        self.inner.dropped()
    }
    /// Free slots, from the reader's `tail` read now (one read of RP1 memory).
    pub fn free(&self) -> u32 {
        self.inner.free()
    }
    /// Lab control only: [`RingTx::send`] whose check of the reader skips the readback of
    /// `head` that the protocol needs (`rp1_proto::shm::RingTx::__reader_asleep_without_readback`).
    #[doc(hidden)]
    pub fn __send_without_readback(&mut self, value: &T) -> Result<()> {
        self.session.check()?;
        self.inner.push(value)?;
        if let Some(doorbell) = &self.doorbell
            && self.inner.__reader_asleep_without_readback()
        {
            doorbell.ring();
        }
        Ok(())
    }
    /// Lab control only: publish without waking the reader, as a writer that ends between its
    /// publish and its doorbell. Only the next writer's opening notify can wake the reader.
    #[doc(hidden)]
    pub fn __push_without_wake(&mut self, value: &T) -> Result<()> {
        self.session.check()?;
        self.inner.push(value)?;
        Ok(())
    }
}
pub struct BlockMap<T> {
    inner: shm::Block<T>,
    writer: Writer,
    _map: Rc<Mapping>,
    session: Rc<Session>,
}
impl<T: Message> BlockMap<T> {
    pub fn read(&self) -> Result<T> {
        self.session.check()?;
        let r = self.inner.read()?;
        self.session.check()?;
        Ok(r)
    }
    pub fn read_with<R>(&self, f: impl FnOnce(&shm::BlockReader<'_, T>) -> R) -> Result<R> {
        self.session.check()?;
        let r = self.inner.read_with(f)?;
        self.session.check()?;
        Ok(r)
    }
    pub fn write(&mut self, value: &T) -> Result<()> {
        self.session.check()?;
        if self.writer != Writer::Linux {
            return Err(proto::Error::Invalid.into());
        }
        unsafe {
            self.inner.write(value);
        }
        Ok(())
    }
    /// # Safety
    /// For writer=both, the caller must exclude the RP1 writer for the entire section.
    pub unsafe fn begin_write(&mut self) -> Result<shm::BlockWriter<'_, T>> {
        self.session.check()?;
        if self.writer == Writer::Rp1 {
            return Err(proto::Error::Invalid.into());
        }
        Ok(unsafe { self.inner.begin_write() })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::{fd::OwnedFd, unix::net::UnixStream};

    /// Anonymous memory standing in for a UIO mapping.
    pub(crate) fn anon(len: usize) -> Rc<Mapping> {
        let p = unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                len,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
                -1,
                0,
            )
        };
        assert_ne!(p, libc::MAP_FAILED);
        Rc::new(Mapping {
            base: NonNull::new(p.cast()).unwrap(),
            len,
            _file: File::open("/dev/null").unwrap(),
        })
    }
    /// Where the rig's rings keep their link-page side (after the status words).
    const UP_SRAM: usize = 0x100;
    const DOWN_SRAM: usize = 0x200;
    struct Rig {
        /// Linux's end of an RP1 -> Linux ring of four `u32`, served by proc1.
        rx: RingRx<u32>,
        /// RP1's end of it.
        tx: shm::RingTx<u32>,
        header: Rc<Mapping>,
        /// The link page: status words and the rings' SRAM side.
        link: Rc<Mapping>,
        session: Rc<Session>,
        /// The other end of the irq ring's `rp1:irq` stand-in (`None` for a poll ring).
        _irq: Option<File>,
    }
    /// A published header and one depth-4 ring, irq or poll; both ends in this process.
    fn rig(irq: bool) -> Rig {
        let header = anon(4096);
        let sha = [7u8; 32];
        let mut init = unsafe { shm::HeaderInit::begin(header.ptr(), 0) };
        init.write(&sha, 1, 0);
        init.publish();
        let link = anon(LINK_PAGE);
        let session = Rc::new(Session {
            header: unsafe { shm::Header::open(header.ptr(), &sha) }.unwrap(),
            _map: header.clone(),
            link: link.clone(),
        });
        let data = anon(4096);
        let (len, ctrl) = (shm::ring_body_size(4, 4).unwrap(), unsafe {
            link.ptr().add(UP_SRAM)
        });
        let ow = shm::Overflow::Reject;
        let inner = unsafe { shm::RingRx::attach(data.ptr(), len, ctrl, 128, 4, ow) };
        let tx = unsafe { shm::RingTx::attach(data.ptr(), len, ctrl, 128, 4, ow) };
        let (irq, peer) = if irq {
            let (irq, peer) = irq_pair();
            (Some(irq), Some(peer))
        } else {
            (None, None)
        };
        let rx = RingRx {
            inner: inner.unwrap(),
            irq,
            peer: Peer {
                link: link.clone(),
                core: 1,
            },
            spin: Duration::ZERO,
            stats: WaitStats::default(),
            _map: data,
            session: session.clone(),
        };
        Rig {
            rx,
            tx: tx.unwrap(),
            header,
            link,
            session,
            _irq: peer,
        }
    }
    /// An `Irq` on one end of a socket pair; writing 4 bytes to the other end is an event.
    fn irq_pair() -> (Irq, File) {
        let (a, b) = UnixStream::pair().unwrap();
        (
            Irq::from_file(File::from(OwnedFd::from(a))),
            File::from(OwnedFd::from(b)),
        )
    }
    /// RP1's end of a depth-4 Linux -> RP1 ring on the rig's link page, and the ring's DDR
    /// side.
    fn down(r: &Rig) -> (shm::RingRx<u32>, Rc<Mapping>) {
        let ddr = anon(4096);
        let sram = unsafe { r.link.ptr().add(DOWN_SRAM) };
        let len = shm::ring_body_size(4, 4).unwrap();
        let ow = shm::Overflow::Reject;
        let rp1 = unsafe { shm::RingRx::attach(sram, len, ddr.ptr(), 128, 4, ow) }.unwrap();
        (rp1, ddr)
    }
    /// Linux's end of [`down`]'s ring, with proc1's doorbell 27 on `page` (`None`: a poll
    /// ring).
    fn writer(r: &Rig, ddr: &Rc<Mapping>, page: Option<&Rc<Mapping>>) -> RingTx<u32> {
        let sram = unsafe { r.link.ptr().add(DOWN_SRAM) };
        let len = shm::ring_body_size(4, 4).unwrap();
        let ow = shm::Overflow::Reject;
        let inner = unsafe { shm::RingTx::attach(sram, len, ddr.ptr(), 128, 4, ow) }.unwrap();
        let doorbell = page.map(|page| Doorbell::new(page.clone(), 1, 27));
        RingTx::open(inner, doorbell, ddr.clone(), r.session.clone())
    }

    /// A valid note with an SHM area, a LINK record (doorbells as given) and these regions,
    /// each a Linux -> RP1 ring of four `u32` on `core` (1 = proc0, 2 = proc1).
    pub(crate) fn note(doorbells: [Option<u8>; 2], rings: &[(&str, u8, bool)]) -> Vec<u8> {
        use proto::note::{self, Link, LinkCore};
        let mut records = Vec::new();
        let mut record = |kind: u16, flags: u16, value: &[u8]| {
            records.extend(kind.to_le_bytes());
            records.extend(flags.to_le_bytes());
            records.extend((value.len() as u32).to_le_bytes());
            records.extend(value);
            records.resize(records.len().next_multiple_of(4), 0);
        };
        record(
            note::FIRMWARE,
            1,
            &[[1, b'f', 1, b'v'].as_slice(), &[0; 52]].concat(),
        );
        record(note::ENTRY, 1, &[0; 20]);
        record(note::OWNERSHIP, 1, &[0; 4]);
        record(note::CLOCK, 1, &[1, 0, 0, 0, 0]);
        record(note::BOOT, 1, &[0; 65]);
        let area = ShmArea {
            size: 0x10_0000,
            handoff: proto::HANDOFF,
            sram_end: proto::LINK_LEN as u16,
            phys: proto::SHM_PHYS,
        };
        record(SHM_AREA, 1, &area.encode());
        let cores = doorbells.map(|doorbell| LinkCore {
            doorbell,
            ..LinkCore::default()
        });
        record(note::LINK, 0, &Link { cores }.encode());
        let body = shm::ring_body_size(4, 4).unwrap() as u16;
        for (i, &(name, core, irq)) in rings.iter().enumerate() {
            let mut v = vec![1, 2, core, irq.into(), 1, 0, 1, 1];
            v.extend((65536 * (i as u32 + 1)).to_le_bytes());
            v.extend(65536u32.to_le_bytes());
            v.extend((0x100 + 0x100 * i as u16).to_le_bytes());
            v.extend(body.to_le_bytes());
            v.extend(4u32.to_le_bytes());
            v.extend(4u32.to_le_bytes());
            v.extend(<u32 as Message>::TYPE_ID.to_le_bytes());
            v.extend([0; 8]);
            v.extend([&[name.len() as u8], name.as_bytes(), &[3], b"u32"].concat());
            record(SHM_REGION, 1, &v);
        }
        let mut bytes = proto::note::MAGIC.to_vec();
        bytes.extend(2u16.to_le_bytes());
        bytes.extend(32u16.to_le_bytes());
        bytes.extend(((32 + records.len()) as u32).to_le_bytes());
        bytes.extend(0u32.to_le_bytes());
        bytes.extend(proto::note::crc32(&records).to_le_bytes());
        bytes.extend(1u32.to_le_bytes());
        bytes.extend(0u32.to_le_bytes());
        bytes.extend(records);
        Note::parse(&bytes).expect("fixture note");
        bytes
    }
    /// A [`Firmware`] over `note` and an anonymous link page, with this session.
    pub(crate) fn firmware(note: Vec<u8>, session: Option<Rc<Session>>) -> Firmware {
        let link = anon(LINK_PAGE);
        let area = ShmArea::parse(Note::parse(&note).unwrap().get(SHM_AREA).unwrap().value);
        Firmware {
            monitor: Monitor::new(link.clone(), &note).unwrap(),
            area: area.unwrap(),
            note,
            link,
            session: RefCell::new(session),
        }
    }

    #[test]
    fn rings_refuse_channel_halves_by_name() {
        let r = rig(false);
        let fw = firmware(
            note([None; 2], &[("x", 2, false), ("x-dn", 2, false)]),
            Some(r.session.clone()),
        );
        // `x-dn` is in the note, yet only a channel or rpc may open it.
        assert!(matches!(fw.ring_tx::<u32>("x-dn"), Err(Error::Unknown)));
        assert!(matches!(fw.ring_rx::<u32>("x-dn"), Err(Error::Unknown)));
        assert!(matches!(fw.ring_tx::<u32>("y"), Err(Error::Unknown)));
        // `x` passes the note and fails only for want of its UIO node on this host.
        assert!(matches!(fw.ring_tx::<u32>("x"), Err(Error::Io(_))));
        assert!(matches!(
            fw.ring_rx::<u32>("x"),
            Err(Error::Proto(proto::Error::Mismatch))
        ));
    }

    #[test]
    fn an_endpoint_has_one_owner() {
        let dir = std::env::temp_dir().join(format!("rp1-linux-flock-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let (a, b) = (dir.join("a"), dir.join("b"));
        fs::write(&a, []).unwrap();
        fs::write(&b, []).unwrap();
        let held = open_exclusive(&a).unwrap();
        // A second open is refused, also in the owner's own process.
        assert!(matches!(open_exclusive(&a), Err(Error::Busy)));
        // Another endpoint opens.
        let other = open_exclusive(&b).unwrap();
        drop(held);
        let again = open_exclusive(&a).unwrap();
        drop((other, again));
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn wait_ready_requests_and_times_out() {
        let fw = firmware(note([None; 2], &[("x", 2, false)]), None);
        assert!(matches!(
            fw.ring_tx::<u32>("x"),
            Err(Error::Proto(proto::Error::NotReady))
        ));
        let t = Instant::now();
        let e = fw.wait_ready(Duration::from_millis(20)).unwrap_err();
        assert!(matches!(e, Error::Timeout), "{e:?}");
        assert!(t.elapsed() >= Duration::from_millis(20));
        assert_eq!(fw.link.read(proto::link::SHM_REQUEST), 1);
        // Status needs no shared memory.
        assert!(matches!(
            fw.status(),
            Err(Error::Proto(proto::Error::NotReady))
        ));
        fw.link.write(proto::link::SHM_STATUS, u32::MAX);
        assert!(matches!(
            fw.wait_ready(Duration::from_secs(5)),
            Err(Error::LinkDown)
        ));
    }

    #[test]
    fn spin_stops_at_the_deadline() {
        let mut r = rig(true);
        r.rx.set_spin(Duration::from_secs(60));
        let t = Instant::now();
        assert!(!r.rx.wait(Some(Duration::from_millis(1))).unwrap());
        assert!(t.elapsed() < Duration::from_secs(5), "{:?}", t.elapsed());
        // Zero timeout: no spin, no sleep.
        assert!(!r.rx.wait(Some(Duration::ZERO)).unwrap());
        assert_eq!(r.rx.wait_stats().sleeps, 0);
    }

    #[test]
    fn wait_checks_the_session_before_spinning() {
        let mut r = rig(true);
        r.rx.set_spin(Duration::from_secs(60));
        r.header.write(12, 1); // RP1 re-initializes the region
        let t = Instant::now();
        let e = r.rx.recv(None).unwrap_err();
        assert!(matches!(e, Error::Proto(proto::Error::Reset)), "{e:?}");
        assert!(t.elapsed() < Duration::from_secs(5));
    }

    #[test]
    fn recv_keeps_its_deadline() {
        for irq in [true, false] {
            let mut r = rig(irq);
            let timeout = Duration::from_millis(30);
            let t = Instant::now();
            assert_eq!(r.rx.recv(Some(timeout)).unwrap(), None);
            let elapsed = t.elapsed();
            // Each sleep ends by RECHECK; 40 ms is scheduling slack.
            assert!(
                elapsed >= timeout && elapsed < timeout + RECHECK + Duration::from_millis(40),
                "irq={irq} {elapsed:?}"
            );
            let s = r.rx.wait_stats();
            assert!(
                s.sleeps > 0 && s.rechecks == s.sleeps && s.late == 0,
                "{s:?}"
            );
            r.tx.push(&5).unwrap();
            let mut out = [0; 4];
            assert_eq!(r.rx.recv_into(&mut out, Some(timeout)).unwrap(), 1);
            assert_eq!(out[0], 5);
            // Zero timeout: no sleep.
            assert_eq!(r.rx.recv(Some(Duration::ZERO)).unwrap(), None);
            assert_eq!(r.rx.wait_stats(), s);
        }
    }

    #[test]
    fn a_wait_ends_when_the_serving_core_is_down() {
        let mut r = rig(true);
        let block = proto::status::BLOCKS[1];
        r.link
            .write(block + proto::status::STATE, CoreState::Running.word());
        assert_eq!(r.rx.recv(Some(Duration::from_millis(1))).unwrap(), None);
        r.link.write(block + proto::status::REASON, 7);
        r.link
            .write(block + proto::status::STATE, CoreState::Panicked.word());
        let t = Instant::now();
        let e = r.rx.recv(None).unwrap_err();
        assert!(t.elapsed() < Duration::from_secs(5));
        assert!(
            matches!(
                e,
                Error::PeerDown {
                    core: 1,
                    state: CoreState::Panicked,
                    reason: 7
                }
            ),
            "{e:?}"
        );
        // Messages already there are still received.
        r.tx.push(&1).unwrap();
        assert_eq!(r.rx.recv(None).unwrap(), Some(1));
        let fw = firmware(note([None; 2], &[]), None);
        let e = fw.wait_any(&mut [&mut r.rx], None).unwrap_err();
        assert!(matches!(e, Error::PeerDown { core: 1, .. }), "{e:?}");
        // BAR2 reads after the PCIe link went down.
        r.link.write(block + proto::status::STATE, u32::MAX);
        assert!(matches!(r.rx.recv(None), Err(Error::LinkDown)));
    }

    /// proc1's `PROCn_INTF` SET alias in `rp1:doorbell`: the address the raw GPIO27 doorbell
    /// uses (HW).
    const PROC1_BELL: usize = 0x2114;
    /// Whether the doorbell was rung since the last call.
    fn rung(page: &Mapping) -> bool {
        let value = page.read(PROC1_BELL);
        page.write(PROC1_BELL, 0);
        assert!(value == 0 || value == 1 << 27, "{value:#x}");
        value != 0
    }

    #[test]
    fn send_rings_the_doorbell_only_for_a_sleeping_reader() {
        let r = rig(false);
        let page = anon(0x4000);
        let (mut rp1, ddr) = down(&r);
        let mut tx = writer(&r, &ddr, Some(&page));
        // The reader is awake (even counter): neither the opening notify nor a send rings.
        assert!(!rung(&page));
        tx.send(&1).unwrap();
        assert!(!rung(&page));
        assert_eq!(rp1.pop().unwrap(), Some(1));
        // Asleep (odd): every send rings.
        assert!(rp1.prepare_wait());
        tx.send(&2).unwrap();
        assert!(rung(&page));
        assert_eq!(tx.send_slice(&[3, 4]).unwrap(), 2);
        assert!(rung(&page));
        // Awake again: no ring.
        rp1.finish_wait();
        tx.send(&5).unwrap();
        assert!(!rung(&page));
        // A poll ring never rings, also while its reader sleeps.
        let r = rig(false);
        let (mut rp1, ddr) = down(&r);
        let mut tx = writer(&r, &ddr, None);
        assert!(rp1.prepare_wait());
        tx.send(&6).unwrap();
        assert!(!rung(&page));
    }

    #[test]
    fn free_reads_the_tail_now_and_the_lab_control_still_rings() {
        let r = rig(false);
        let page = anon(0x4000);
        let (mut rp1, ddr) = down(&r);
        let mut tx = writer(&r, &ddr, Some(&page));
        assert_eq!(tx.free(), 4);
        tx.send(&1).unwrap();
        assert_eq!(tx.free(), 3);
        assert_eq!(rp1.pop().unwrap(), Some(1));
        assert_eq!(tx.free(), 4);
        // Without the readback a sleeping reader is still seen here (the readback matters
        // only for ordering on the bus).
        assert!(rp1.prepare_wait());
        tx.__send_without_readback(&2).unwrap();
        assert!(rung(&page));
        rp1.finish_wait();
        tx.__send_without_readback(&3).unwrap();
        assert!(!rung(&page));
    }

    #[test]
    fn a_new_writer_rings_once_for_a_reader_already_asleep() {
        let r = rig(false);
        let page = anon(0x4000);
        let (mut rp1, ddr) = down(&r);
        // The reader sleeps; a previous writer published and ended before it rang.
        assert!(rp1.prepare_wait());
        let mut inner = unsafe {
            shm::RingTx::<u32>::attach(
                r.link.ptr().add(DOWN_SRAM),
                shm::ring_body_size(4, 4).unwrap(),
                ddr.ptr(),
                128,
                4,
                shm::Overflow::Reject,
            )
        }
        .unwrap();
        inner.push(&1).unwrap();
        let _tx = writer(&r, &ddr, Some(&page));
        assert!(rung(&page));
        // Reader awake when the next writer opens: no ring.
        assert_eq!(rp1.drain(4, |_| {}).unwrap(), shm::Progress::Idle);
        rp1.finish_wait();
        let _tx = writer(&r, &ddr, Some(&page));
        assert!(!rung(&page));
    }

    /// The lab's dying writer: its message leaves the sleeping reader unrung, and the next
    /// writer rings for it at open.
    #[test]
    fn a_push_without_wake_is_rung_for_by_the_next_writer() {
        let r = rig(false);
        let page = anon(0x4000);
        let (mut rp1, ddr) = down(&r);
        let mut tx = writer(&r, &ddr, Some(&page));
        assert!(rp1.prepare_wait());
        tx.__push_without_wake(&1).unwrap();
        assert!(!rung(&page));
        drop(tx);
        let _tx = writer(&r, &ddr, Some(&page));
        assert!(rung(&page));
    }

    #[test]
    fn wait_any_returns_on_any_ring() {
        let fw = firmware(note([None; 2], &[]), None);
        for irq in [true, false] {
            let mut a = rig(true);
            let mut b = rig(irq);
            let t = Instant::now();
            let timeout = Some(Duration::from_millis(30));
            assert!(!fw.wait_any(&mut [&mut a.rx, &mut b.rx], timeout).unwrap());
            assert!(t.elapsed() >= Duration::from_millis(30));
            b.tx.push(&9).unwrap();
            assert!(fw.wait_any(&mut [&mut a.rx, &mut b.rx], None).unwrap());
            assert_eq!(b.rx.try_recv().unwrap(), Some(9));
            // Both rings were disarmed again: no notification is due.
            assert!(!a.tx.wake_needed() && !b.tx.wake_needed());
        }
    }
}
