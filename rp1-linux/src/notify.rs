//! How a `notify = "irq"` endpoint and RP1 notify each other: the `rp1:irq` interrupt
//! (RP1 -> Linux) and the `rp1:doorbell` force bit (Linux -> RP1). Both nodes are shared:
//! v2 endpoints open individual handles; v3 uses one exclusive listener.
use super::{Access, Mapping, Result, dev_path, find_uio, open_read_write};
use rp1_proto::uio;
use std::{fs::File, os::fd::AsRawFd, rc::Rc, time::Duration};

/// The `rp1:irq` UIO node: the RP1 -> Linux interrupt (MSI-X vector `rp1_proto::irq::VECTOR`,
/// raised by the core that declares `host_irq = true`). One line for every `notify = "irq"`
/// ring: a wake for one ring is a spurious wake for another, and every open handle sees it.
pub struct Irq {
    file: File,
}
impl Irq {
    /// Open `rp1:irq` and enable the interrupt (`uio_pdrv_genirq` starts it enabled, and masks
    /// it again in its handler; pulses before the first enable are lost).
    pub(crate) fn open() -> Result<Self> {
        let dev = dev_path(&find_uio(&format!("rp1:{}", uio::IRQ))?)?;
        let irq = Self {
            file: open_read_write(&dev)?,
        };
        enable(&irq.file)?;
        Ok(irq)
    }
    /// An `Irq` on any file, e.g. a socket standing in for the UIO node.
    #[cfg(test)]
    pub(crate) fn from_file(file: File) -> Self {
        Self { file }
    }
    /// Sleep up to `timeout` (rounded up to whole ms) for an interrupt. Returns whether one
    /// arrived; it is consumed and the line re-enabled before returning, so an edge from then
    /// on wakes the next call (edges while masked merge into one pending wake). A signal
    /// (`EINTR`) ends the wait early as a spurious wake (`Ok(false)`).
    pub(crate) fn wait(&mut self, timeout: Duration) -> std::io::Result<bool> {
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
        if n < 0 {
            let e = std::io::Error::last_os_error();
            return if e.kind() == std::io::ErrorKind::Interrupted {
                Ok(false)
            } else {
                Err(e)
            };
        }
        if n != 4 {
            // UIO reads are all-or-nothing; anything else is not a UIO event count.
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("rp1:irq read returned {n} bytes"),
            ));
        }
        enable(&self.file)?;
        Ok(true)
    }
}

pub(crate) mod listener;

fn enable(file: &File) -> std::io::Result<()> {
    let one = 1u32.to_ne_bytes();
    loop {
        let n = unsafe { libc::write(file.as_raw_fd(), one.as_ptr().cast(), 4) };
        match n {
            4 => return Ok(()),
            0..=3 => return Err(std::io::Error::from(std::io::ErrorKind::WriteZero)),
            _ => {
                let e = std::io::Error::last_os_error();
                if e.kind() != std::io::ErrorKind::Interrupted {
                    return Err(e);
                }
            }
        }
    }
}
/// Physical address of `rp1:doorbell`: RP1 `IO_BANK0`, one 16 KiB page.
const DOORBELL_PHYS: u64 = 0x1f_000d_0000;
const DOORBELL_LEN: usize = 0x4000;
/// `PROC0_INTF` through the page's SET alias (+0x2000); `PROC1_INTF` is 0xc further.
const INTF_SET: usize = 0x2108;

/// One core's doorbell: Linux interrupts it by setting the force bit of its doorbell pin in
/// `PROCn_INTF` (pin and core from the note's LINK record). The force is a level that stays
/// until RP1 acks it, so setting it again while set costs nothing and adds no interrupt.
#[derive(Clone)]
pub(crate) struct Doorbell {
    page: Rc<Mapping>,
    offset: usize,
    bit: u32,
}
impl Doorbell {
    /// Map `rp1:doorbell` for `pin` of `core` (0 = proc0).
    pub(crate) fn open(core: usize, pin: u8) -> Result<Self> {
        // Tests substitute only the UIO mapping and observe the completed force.
        #[cfg(test)]
        if let Some(page) = fake_doorbell::PAGE.with(|page| page.borrow().clone()) {
            return Ok(Self::new(page, core, pin));
        }
        let page = Mapping::open(
            &format!("rp1:{}", uio::DOORBELL),
            DOORBELL_PHYS,
            DOORBELL_LEN,
            Access::Shared,
        )?;
        Ok(Self::new(page, core, pin))
    }
    pub(crate) fn new(page: Rc<Mapping>, core: usize, pin: u8) -> Self {
        Self {
            page,
            offset: INTF_SET + 0xc * core,
            bit: 1 << pin,
        }
    }
    /// Set the force bit (one posted write).
    pub(crate) fn ring(&self) {
        self.page.write(self.offset, self.bit);
    }
    /// Complete the SET through the same bank. The ISR may have cleared it by
    /// the time this returns, so the readback value need not contain the bit.
    pub(crate) fn ring_flushed(&self) {
        self.ring();
        let _ = self.page.read(self.offset - 0x2000);
        rp1_proto::barrier();
        #[cfg(test)]
        fake_doorbell::on_force();
    }
}

#[cfg(test)]
pub(crate) mod fake_doorbell {
    use super::*;
    use std::cell::RefCell;

    type ForceHook = Box<dyn FnMut()>;
    thread_local! {
        pub(super) static PAGE: RefCell<Option<Rc<Mapping>>> = const { RefCell::new(None) };
        static FORCE: RefCell<Option<ForceHook>> = const { RefCell::new(None) };
    }

    pub(crate) fn with_page<T>(page: Rc<Mapping>, open: impl FnOnce() -> T) -> T {
        PAGE.with(|slot| {
            assert!(slot.replace(Some(page)).is_none());
            let result = open();
            slot.replace(None);
            result
        })
    }

    pub(crate) fn hook(force: impl FnMut() + 'static) {
        FORCE.with(|slot| assert!(slot.replace(Some(Box::new(force))).is_none()));
    }

    pub(crate) fn clear_hook() {
        FORCE.with(|slot| slot.replace(None));
    }

    pub(super) fn on_force() {
        FORCE.with(|slot| {
            if let Some(force) = slot.borrow_mut().as_mut() {
                force();
            }
        });
    }
}

#[cfg(test)]
mod wake_model;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::anon;
    use std::io::{Read, Write};
    use std::os::{fd::OwnedFd, unix::net::UnixStream};

    #[test]
    fn irq_wait_wakes_and_rejects_short_reads() {
        let (a, b) = UnixStream::pair().unwrap();
        let mut irq = Irq::from_file(File::from(OwnedFd::from(a)));
        let mut peer = File::from(OwnedFd::from(b));
        assert!(!irq.wait(Duration::from_millis(1)).unwrap());
        peer.write_all(&5u32.to_ne_bytes()).unwrap();
        assert!(irq.wait(Duration::from_secs(5)).unwrap());
        peer.write_all(&[1, 2]).unwrap();
        let e = irq.wait(Duration::from_secs(5)).unwrap_err();
        assert_eq!(e.kind(), std::io::ErrorKind::InvalidData);
    }

    #[test]
    fn zero_timeout_drains_an_already_readable_irq_and_reenables() {
        let (a, mut peer) = UnixStream::pair().unwrap();
        peer.set_nonblocking(true).unwrap();
        let mut irq = Irq::from_file(File::from(OwnedFd::from(a)));
        assert!(!irq.wait(Duration::ZERO).unwrap());
        peer.write_all(&7u32.to_ne_bytes()).unwrap();
        assert!(irq.wait(Duration::ZERO).unwrap());
        let mut reenable = [0; 4];
        peer.read_exact(&mut reenable).unwrap();
        assert_eq!(u32::from_ne_bytes(reenable), 1);
        assert!(!irq.wait(Duration::ZERO).unwrap());
    }

    /// The SET aliases of `PROC0_INTF` / `PROC1_INTF` (IO_BANK0 0x108 / 0x114, +0x2000).
    #[test]
    fn a_doorbell_sets_its_pin_in_its_cores_intf() {
        let page = anon(DOORBELL_LEN);
        Doorbell::new(page.clone(), 0, 26).ring();
        Doorbell::new(page.clone(), 1, 27).ring();
        assert_eq!(page.read(0x2108), 1 << 26);
        assert_eq!(page.read(0x2114), 1 << 27);
    }
}
