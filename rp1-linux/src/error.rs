//! The one error type of rp1-linux.
use rp1_proto::status::{ABI_VERSION, CoreState};
use std::{fmt, io};

/// Why an rp1-linux call failed.
#[derive(Debug)]
pub enum Error {
    /// A system call failed, or a UIO node or the note in the boot DT is missing or does not
    /// match the firmware.
    Io(io::Error),
    /// The shared-memory protocol refused: `Full`, `Corrupt`, `Invalid`, `Reset`, `NotReady`
    /// or `Mismatch` (never `Lost`, which is [`Error::Lost`]).
    Proto(rp1_proto::Error),
    /// An overwrite ring dropped this many messages; the next read continues after them.
    Lost(u32),
    /// The deadline passed first.
    Timeout,
    /// Another owner holds the endpoint.
    Busy,
    /// The firmware's note has no such endpoint (or, for status, no LINK record).
    Unknown,
    /// The core that serves the endpoint (0 = proc0) stopped, panicked or faulted.
    PeerDown {
        core: u8,
        state: CoreState,
        reason: u32,
    },
    /// BAR2 reads 0xffff_ffff: the PCIe link to RP1 is down.
    LinkDown,
    /// The firmware's status words are layout version `firmware`, not this crate's.
    Abi { firmware: u16 },
}

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    /// [`Error::Io`] for a UIO node or note that is not what the firmware describes.
    pub(crate) fn setup(message: impl Into<String>) -> Self {
        Self::Io(io::Error::new(io::ErrorKind::InvalidData, message.into()))
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}
impl From<rp1_proto::Error> for Error {
    fn from(e: rp1_proto::Error) -> Self {
        match e {
            rp1_proto::Error::Lost(n) => Self::Lost(n),
            e => Self::Proto(e),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "{e}"),
            Self::Proto(e) => write!(f, "shared memory: {e}"),
            Self::Lost(n) => write!(f, "{n} messages were overwritten"),
            Self::Timeout => write!(f, "timed out"),
            Self::Busy => write!(f, "the endpoint is held by another owner"),
            Self::Unknown => write!(f, "the firmware's note has no such endpoint"),
            Self::PeerDown {
                core,
                state,
                reason,
            } => write!(f, "RP1 proc{core} is {state:?} (reason {reason:#x})"),
            Self::LinkDown => write!(f, "BAR2 reads 0xffffffff: PCIe link down"),
            Self::Abi { firmware } => write!(
                f,
                "the firmware's status words are RL v{firmware}, this rp1-linux reads \
                 v{ABI_VERSION}: build the firmware and rp1-linux from the same tree"
            ),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::Proto(e) => Some(e),
            _ => None,
        }
    }
}
