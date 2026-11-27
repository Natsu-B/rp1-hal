//! The exclusive v3 UIO listener. The device count carries no endpoint identity.
use super::{File, dev_path, find_uio, uio};
use crate::{Result, lock_exclusive};
use std::{
    fs::OpenOptions,
    io,
    os::{
        fd::{AsFd, AsRawFd, BorrowedFd},
        unix::fs::OpenOptionsExt,
    },
    time::Instant,
};

pub(crate) struct Listener(File);

impl Listener {
    pub(crate) fn open() -> Result<Self> {
        let dev = dev_path(&find_uio(&format!("rp1:{}", uio::IRQ))?)?;
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .custom_flags(libc::O_NONBLOCK)
            .open(dev)?;
        lock_exclusive(&file)?;
        Self::enable(file)
    }

    pub(crate) fn enable(file: File) -> Result<Self> {
        // Open baselines the count. The kernel may already have masked the vector,
        // even if this listener has never read an interrupt.
        super::enable(&file)?;
        Ok(Self(file))
    }

    /// Read at most one count, then immediately unmask. EAGAIN may mean the UIO
    /// thread has yet to publish a count for an IRQ that already masked the vector.
    pub(crate) fn intake(&self) -> io::Result<bool> {
        let mut count = [0u8; 4];
        loop {
            let n =
                unsafe { libc::read(self.0.as_raw_fd(), count.as_mut_ptr().cast(), count.len()) };
            match n {
                4 => {
                    super::enable(&self.0)?;
                    return Ok(true);
                }
                -1 => match io::Error::last_os_error() {
                    e if e.kind() == io::ErrorKind::Interrupted => continue,
                    e if e.kind() == io::ErrorKind::WouldBlock => return Ok(false),
                    e => return Err(e),
                },
                _ => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "short rp1:irq count",
                    ));
                }
            }
        }
    }
}

impl AsFd for Listener {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.0.as_fd()
    }
}

/// Keep the same monotonic end across signals. A zero-length poll can still
/// report a pending event.
pub(crate) fn poll_until(fds: &mut [libc::pollfd], end: Option<Instant>) -> io::Result<()> {
    poll_with(end, |remaining| {
        let timeout = remaining.map(|remaining| libc::timespec {
            tv_sec: remaining.as_secs().min(i32::MAX as u64) as _,
            tv_nsec: remaining.subsec_nanos().into(),
        });
        let timeout = timeout
            .as_ref()
            .map_or(std::ptr::null(), std::ptr::from_ref);
        let n = unsafe { libc::ppoll(fds.as_mut_ptr(), fds.len() as _, timeout, std::ptr::null()) };
        if n >= 0 {
            Ok(())
        } else {
            Err(io::Error::last_os_error())
        }
    })
}

fn poll_with(
    end: Option<Instant>,
    mut poll: impl FnMut(Option<std::time::Duration>) -> io::Result<()>,
) -> io::Result<()> {
    loop {
        let remaining = end.map(|end| end.saturating_duration_since(Instant::now()));
        match poll(remaining) {
            Err(e)
                if e.kind() == io::ErrorKind::Interrupted
                    && !remaining.is_some_and(|d| d.is_zero()) => {}
            // The caller still does nonblocking count intake at the deadline.
            Err(e) if e.kind() == io::ErrorKind::Interrupted => return Ok(()),
            result => return result,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        io::{Read, Write},
        os::{fd::OwnedFd, unix::net::UnixStream},
        time::Duration,
    };

    #[test]
    fn initial_enable_eagain_count_wrap_reenable_and_short_read() {
        let (socket, mut peer) = UnixStream::pair().unwrap();
        socket.set_nonblocking(true).unwrap();
        peer.set_nonblocking(true).unwrap();
        let listener = Listener::enable(File::from(OwnedFd::from(socket))).unwrap();
        let mut command = [0; 4];
        peer.read_exact(&mut command).unwrap();
        assert_eq!(u32::from_ne_bytes(command), 1);
        assert!(!listener.intake().unwrap());

        // A wrapped device count is just as readable as any other count.
        peer.write_all(&0u32.to_ne_bytes()).unwrap();
        assert!(listener.intake().unwrap());
        peer.read_exact(&mut command).unwrap();
        assert_eq!(u32::from_ne_bytes(command), 1);
        peer.write_all(&[1, 2]).unwrap();
        assert_eq!(
            listener.intake().unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
        assert_eq!(
            peer.read(&mut command).unwrap_err().kind(),
            io::ErrorKind::WouldBlock
        );
        drop(listener);
        assert_eq!(peer.read(&mut command).unwrap(), 0);
    }

    #[test]
    fn ppoll_can_report_link_and_another_fd_at_the_deadline() {
        let (a, mut b) = UnixStream::pair().unwrap();
        let (c, mut d) = UnixStream::pair().unwrap();
        b.write_all(&[1]).unwrap();
        d.write_all(&[1]).unwrap();
        let mut fds = [&a, &c].map(|fd| libc::pollfd {
            fd: fd.as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        });
        poll_until(&mut fds, Some(Instant::now())).unwrap();
        assert!(fds.iter().all(|fd| fd.revents == libc::POLLIN));
    }

    #[test]
    fn signals_keep_the_original_deadline() {
        let end = Instant::now() + Duration::from_millis(10);
        let mut calls = 0;
        poll_with(Some(end), |remaining| {
            let remaining = remaining.unwrap();
            calls += 1;
            if calls == 1 {
                std::thread::sleep(remaining);
            } else {
                assert!(remaining.is_zero());
            }
            Err(io::ErrorKind::Interrupted.into())
        })
        .unwrap();
        assert!(calls <= 2);
        assert!(Instant::now() >= end);
    }
}
