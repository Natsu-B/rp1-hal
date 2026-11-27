//! The one error type of rp1-linux.
use rp1_proto::status::{ABI_VERSION, CoreState};
use rp1_proto::wake::SessionError;
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
    /// A commit was visible at the deadline without a delivered wake.
    NotifyLost,
    /// The requested duration cannot be represented as a monotonic deadline.
    InvalidDeadline,
    /// The firmware or endpoint does not support this wait or capability.
    Unsupported,
    /// Another owner holds the endpoint, or all RPC slots have outstanding requests.
    Busy,
    /// The firmware's note has no such endpoint (or, for status, no LINK record).
    Unknown,
    /// The current session lost its serving core or its connection.
    PeerDown(PeerDown),
    /// BAR2 reads 0xffff_ffff: the PCIe link to RP1 is down.
    LinkDown,
    /// The firmware's status words are layout version `firmware`, not this crate's.
    Abi { firmware: u16 },
}

/// Why a peer is no longer usable.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PeerDown {
    Terminal {
        core: u8,
        state: CoreState,
        reason: u32,
    },
    Restarted,
    LinkDown,
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

impl From<SessionError> for Error {
    fn from(error: SessionError) -> Self {
        match error {
            SessionError::Restarted | SessionError::NotReady => Self::PeerDown(PeerDown::Restarted),
            SessionError::Unsupported => Self::Unsupported,
            SessionError::Exhausted => Self::Proto(rp1_proto::Error::Invalid),
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
            Self::NotifyLost => write!(f, "work was published without a wake before the deadline"),
            Self::InvalidDeadline => write!(f, "invalid monotonic deadline"),
            Self::Unsupported => write!(f, "unsupported link capability or wait mode"),
            Self::Busy => write!(f, "the endpoint is held or all RPC slots are outstanding"),
            Self::Unknown => write!(f, "the firmware's note has no such endpoint"),
            Self::PeerDown(PeerDown::Terminal {
                core,
                state,
                reason,
            }) => write!(f, "RP1 proc{core} is {state:?} (reason {reason:#x})"),
            Self::PeerDown(PeerDown::Restarted) => write!(f, "RP1 session restarted"),
            Self::PeerDown(PeerDown::LinkDown) => write!(f, "RP1 session lost its PCIe link"),
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
