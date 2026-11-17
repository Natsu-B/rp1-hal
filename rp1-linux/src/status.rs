//! The firmware's status words on the link page (`rp1_proto::status`), written by rp1-rt.
use super::{Mapping, NOTE, Result};
pub use rp1_proto::status::CoreState;
use rp1_proto::{
    Error, barrier, link,
    note::{LINK, Link, Note},
    status as s,
};
use std::{fs, rc::Rc, time::Duration};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Status {
    /// Even; +2 per firmware start.
    pub epoch: u32,
    pub shm: ShmState,
    pub shm_generation: u32,
    /// proc0, proc1.
    pub cores: [CoreStatus; 2],
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShmState {
    NotEnabled,
    Ready,
    WindowLost,
    Error(u32),
}
/// One core's block. `reason`, `line`, `aux` and `file` are only read in a terminal state
/// (0 / empty otherwise); `beat` is always read.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CoreStatus {
    pub state: CoreState,
    pub reason: u32,
    pub line: u32,
    pub aux: u32,
    pub beat: u32,
    /// Tail of the panic location's file name.
    pub file: String,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Health {
    Ok,
    /// A core that declares `heartbeat` did not beat.
    Stalled {
        core: u8,
    },
    /// A core is in a terminal (or unknown) state.
    Down {
        core: u8,
        state: CoreState,
    },
}

/// Reads the status words. Works with any rp1-hal firmware, before shared memory is enabled, and
/// next to the process that holds [`super::Firmware`].
pub struct Monitor {
    link: Rc<Mapping>,
    heartbeat_ms: [Option<u16>; 2],
}
impl Monitor {
    /// Map `rp1:link` read-only and without a lock, and read the declared heartbeats from the
    /// boot DT's note.
    pub fn open() -> Result<Self> {
        let note = fs::read(NOTE)?;
        let link = Mapping::open_read_only(
            &format!("rp1:{}", rp1_proto::uio::LINK),
            rp1_proto::LINK_PHYS,
            0x4000,
        )?;
        Self::new(link, &note)
    }
    pub(crate) fn new(link: Rc<Mapping>, note: &[u8]) -> Result<Self> {
        let declared = Note::parse(note)?
            .get(LINK)
            .map(|r| Link::parse(r.value))
            .transpose()?
            .unwrap_or_default();
        Ok(Self {
            link,
            heartbeat_ms: declared.cores.map(|c| c.heartbeat_ms),
        })
    }

    /// One read of the status words.
    ///
    /// `Err(NotReady)` unless the ABI word matches and `EPOCH` is even and unchanged across the
    /// read (no firmware restart in between; never started looks the same). The details of a
    /// core are final when its state is terminal. The two cores, their beats and the shm words
    /// are not one snapshot.
    pub fn status(&self) -> Result<Status> {
        let read = |offset| self.link.read(offset);
        let epoch = read(s::EPOCH);
        let abi = read(s::ABI);
        if abi == u32::MAX {
            return Err("BAR2 reads 0xffffffff: PCIe link down".into());
        }
        if abi != s::ABI_VALUE || !epoch.is_multiple_of(2) {
            return Err(Error::NotReady.into());
        }
        let cores = [0, 1].map(|core| self.core(s::BLOCKS[core]));
        let shm = match read(link::SHM_STATUS) {
            0 => ShmState::NotEnabled,
            1 => ShmState::Ready,
            link::SHM_WINDOW_LOST => ShmState::WindowLost,
            code => ShmState::Error(code),
        };
        let shm_generation = read(link::SHM_GENERATION);
        barrier();
        if read(s::EPOCH) != epoch {
            return Err(Error::NotReady.into());
        }
        Ok(Status {
            epoch,
            shm,
            shm_generation,
            cores,
        })
    }
    fn core(&self, block: usize) -> CoreStatus {
        let read = |offset| self.link.read(block + offset);
        let state = CoreState::from_word(read(s::STATE));
        let beat = read(s::BEAT);
        if !state.is_terminal() {
            return CoreStatus {
                state,
                beat,
                ..CoreStatus::default()
            };
        }
        // The core wrote the details before the terminal state.
        barrier();
        let words = std::array::from_fn(|i| read(s::FILE + 4 * i));
        let mut bytes = [0; s::FILE_LEN];
        let n = s::file_bytes(&words, &mut bytes);
        CoreStatus {
            state,
            reason: read(s::REASON),
            line: read(s::LINE),
            aux: read(s::AUX),
            beat,
            file: String::from_utf8_lossy(&bytes[..n]).into_owned(),
        }
    }

    /// `Down` for a core in a terminal or unknown state, else `Stalled` for a core whose
    /// declared heartbeat did not change between two reads, else `Ok`.
    ///
    /// The reads are `max(window, 2 × the largest declared period)` apart, so a core that beats
    /// at its declared period is never reported stalled. Without a declared heartbeat it
    /// returns after one read. `Err(Reset)`: the firmware restarted in between.
    pub fn health(&self, window: Duration) -> Result<Health> {
        let first = self.status()?;
        if let Some(down) = down(&first) {
            return Ok(down);
        }
        let Some(period) = self.heartbeat_ms.iter().flatten().max() else {
            return Ok(Health::Ok);
        };
        std::thread::sleep(window.max(2 * Duration::from_millis((*period).into())));
        let second = self.status()?;
        if second.epoch != first.epoch {
            return Err(Error::Reset.into());
        }
        if let Some(down) = down(&second) {
            return Ok(down);
        }
        for core in 0..2 {
            if self.heartbeat_ms[core].is_some()
                && first.cores[core].beat == second.cores[core].beat
            {
                return Ok(Health::Stalled { core: core as u8 });
            }
        }
        Ok(Health::Ok)
    }
}
fn down(status: &Status) -> Option<Health> {
    status.cores.iter().enumerate().find_map(|(core, c)| {
        (c.state.is_terminal() || matches!(c.state, CoreState::Unknown(_))).then_some(
            Health::Down {
                core: core as u8,
                state: c.state,
            },
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::anon;
    use std::{
        sync::atomic::{AtomicBool, AtomicU32, Ordering::Relaxed},
        thread,
        time::Instant,
    };

    /// A link page with valid status words, both cores running.
    fn monitor(heartbeat_ms: [Option<u16>; 2]) -> Monitor {
        let link = anon(0x4000);
        link.write(s::EPOCH, 6);
        link.write(s::ABI, s::ABI_VALUE);
        for block in s::BLOCKS {
            link.write(block + s::STATE, CoreState::Running.word());
        }
        Monitor { link, heartbeat_ms }
    }

    #[test]
    fn status_needs_the_abi_and_an_even_epoch() {
        let m = monitor([None; 2]);
        let not_ready =
            |m: &Monitor| m.status().unwrap_err().downcast_ref() == Some(&Error::NotReady);
        assert_eq!(m.status().unwrap().epoch, 6);
        m.link.write(s::EPOCH, 7);
        assert!(not_ready(&m));
        m.link.write(s::EPOCH, 8);
        m.link.write(s::ABI, 0);
        assert!(not_ready(&m));
        m.link.write(s::ABI, u32::MAX);
        assert!(m.status().unwrap_err().to_string().contains("link down"));
    }

    #[test]
    fn details_only_in_a_terminal_state() {
        let m = monitor([None; 2]);
        let p1 = s::BLOCKS[1];
        m.link.write(p1 + s::LINE, 77);
        m.link.write(p1 + s::BEAT, 1234);
        let status = m.status().unwrap();
        assert_eq!(status.shm, ShmState::NotEnabled);
        assert_eq!(
            status.cores[1],
            CoreStatus {
                state: CoreState::Running,
                beat: 1234,
                ..CoreStatus::default()
            }
        );
        for (i, w) in s::file_words("examples/dual-core/src/main.rs")
            .iter()
            .enumerate()
        {
            m.link.write(p1 + s::FILE + 4 * i, *w);
        }
        m.link.write(p1 + s::STATE, CoreState::Panicked.word());
        let core = m.status().unwrap().cores[1].clone();
        assert_eq!((core.state, core.line), (CoreState::Panicked, 77));
        assert_eq!(core.file, "ual-core/src/main.rs");
        assert_eq!(
            m.health(Duration::ZERO).unwrap(),
            Health::Down {
                core: 1,
                state: CoreState::Panicked
            }
        );
    }

    #[test]
    fn short_window_does_not_stall_a_healthy_core() {
        // proc0 declares 20 ms and beats every 10 ms; proc1 declares nothing and never beats.
        let m = monitor([Some(20), None]);
        let beat = unsafe { &*m.link.ptr().add(s::BLOCKS[0] + s::BEAT).cast::<AtomicU32>() };
        let stop = AtomicBool::new(false);
        thread::scope(|scope| {
            scope.spawn(|| {
                while !stop.load(Relaxed) {
                    beat.store(beat.load(Relaxed) + 1, Relaxed);
                    thread::sleep(Duration::from_millis(10));
                }
            });
            let start = Instant::now();
            assert_eq!(m.health(Duration::from_millis(1)).unwrap(), Health::Ok);
            assert!(start.elapsed() >= Duration::from_millis(40));
            stop.store(true, Relaxed);
        });
        assert_eq!(
            m.health(Duration::from_millis(1)).unwrap(),
            Health::Stalled { core: 0 }
        );
        // No declared heartbeat: one read, no wait.
        let start = Instant::now();
        let m = monitor([None; 2]);
        assert_eq!(m.health(Duration::from_secs(60)).unwrap(), Health::Ok);
        assert!(start.elapsed() < Duration::from_secs(5));
    }
}
