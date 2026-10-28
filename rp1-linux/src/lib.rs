//! Uncached UIO mappings. Only aligned volatile words ever touch Device memory.
use rp1_proto::{
    Error, Message, barrier,
    note::{Note, SHM_AREA, SHM_REGION, ShmArea, ShmKind, ShmRegion, Writer},
    shm,
};
use sha2::{Digest, Sha256};
use std::{
    collections::HashSet,
    fs::{self, File, OpenOptions},
    os::fd::AsRawFd,
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
        let path = &matches[0];
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
        let link = Mapping::open("rp1:link", rp1_proto::LINK_PHYS, 0x4000)?;
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
        let map = Mapping::open("rp1:header", area.phys, 65536)?;
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
        if r.type_id != T::TYPE_ID
            || r.elem_size as usize != std::mem::size_of::<T>()
            || r.notify_irq
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
        self.taken.insert(name.into());
        Ok(RingRx {
            inner,
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

pub struct RingRx<T> {
    inner: shm::RingRx<T>,
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
    pub fn dropped(&self) -> u32 {
        self.inner.dropped()
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
