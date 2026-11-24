//! Firmware status words in the link page (offsets from `LINK_START`, all `u32` LE).
//!
//! rp1-rt writes them, Linux reads them through BAR2. Each word has one writer: `EPOCH` and
//! `ABI` are proc0's, a core's block is that core's own.

/// Odd while proc0 clears the blocks at startup, even once they are valid; +2 per boot.
pub const EPOCH: usize = 0x40;
/// `ABI_VALUE` once proc0 has initialized the words of this layout.
pub const ABI: usize = 0x44;
/// v3 added `PLAT_PCIE` and `PLAT_SCMI`.
pub const ABI_VERSION: u16 = 3;
pub const ABI_VALUE: u32 = abi_value(ABI_VERSION);
const ABI_TAG: u16 = u16::from_le_bytes(*b"RL");
/// The `ABI` word of `version` of this layout: `"RL"`, then the version as `u16` LE.
pub const fn abi_value(version: u16) -> u32 {
    ABI_TAG as u32 | (version as u32) << 16
}
/// The version in an `ABI` word that has the `"RL"` tag, of this or another version.
pub const fn abi_version(word: u32) -> Option<u16> {
    if word as u16 == ABI_TAG {
        Some((word >> 16) as u16)
    } else {
        None
    }
}
/// The platform's PCIe state: bits 0–7 one of the `PLAT_PCIE_*` states, bits 8–15 the
/// `PLAT_PCIE_OFF_*` reason of `DETECTOR_OFF`, bits 16–31 PCIE_MONITOR2[31:16] when the
/// detector saw `LOST`. proc0's platform init writes it until it arms IRQ53, IRQ53 after that.
/// [`PcieState::from_word`] decodes it.
pub const PLAT_PCIE: usize = 0x48;
pub const PLAT_PCIE_NOT_STARTED: u32 = 0;
/// The platform init brings the endpoint up.
pub const PLAT_PCIE_INITIALIZING: u32 = 1;
/// The link is up and the PERSTn detector is armed.
pub const PLAT_PCIE_UP: u32 = 2;
/// The host reset the endpoint after init: the outbound window is gone until RP1 is reloaded.
pub const PLAT_PCIE_LOST: u32 = 3;
/// The detector is off, so a host reset would go unseen.
pub const PLAT_PCIE_DETECTOR_OFF: u32 = 4;
/// `DETECTOR_OFF` reason: PCIE_INTF was set when init armed the detector.
pub const PLAT_PCIE_OFF_ARM_FAULT: u8 = 1;
/// `DETECTOR_OFF` reason: 16 IRQ53 entries found no interrupt source.
pub const PLAT_PCIE_OFF_SPURIOUS_STORM: u8 = 2;
/// SCMI state in bits 0–7 (one of the `PLAT_SCMI_*` states), with the saturated request count
/// in bits 16–31 while serving. proc0's platform init writes it until it enables IRQ57, IRQ57
/// after that. [`ScmiState::from_word`] decodes it.
pub const PLAT_SCMI: usize = 0x4c;
pub const PLAT_SCMI_NOT_STARTED: u32 = 0;
pub const PLAT_SCMI_SERVING: u32 = 1;
/// The server stopped answering (it saw too many empty entries).
pub const PLAT_SCMI_QUARANTINED: u32 = 2;
/// Status block of proc0 and proc1.
pub const BLOCKS: [usize; 2] = [0x50, 0x80];
pub const BLOCK_LEN: usize = 0x30;
/// First byte after the status words; declared shm regions start here.
pub const END: usize = 0xc0;

/// Word offsets inside a status block.
pub const STATE: usize = 0x00;
/// Stop code of `Stopped`, exception number of `Fault`, 0 otherwise.
pub const REASON: usize = 0x04;
/// Line of `Panicked`, stacked PC of `Fault`.
pub const LINE: usize = 0x08;
/// CFSR of `Fault`.
pub const AUX: usize = 0x0c;
/// TIMER µs (low 32 bits) of the last heartbeat; only a change means something.
pub const BEAT: usize = 0x10;
/// Last `FILE_LEN` bytes of the panic location's file name, NUL padded.
pub const FILE: usize = 0x18;
pub const FILE_LEN: usize = 20;

const _: () = assert!(BLOCKS[1] + BLOCK_LEN <= END && FILE + FILE_LEN <= BLOCK_LEN);
// The fixed words end where the lab's words begin (0x24..0x2f; lab/check-shm.sh writes
// link + 0x28 and 0x2c).
const _: () = assert!(crate::link::SHM_REQUEST + 4 <= 0x24);

/// A decoded `PLAT_PCIE` word.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PcieState {
    NotStarted,
    Initializing,
    /// The link is up and the detector is armed.
    Up,
    /// The host reset the endpoint after init. `monitor2_hi` is PCIE_MONITOR2[31:16] at the
    /// detection.
    Lost {
        monitor2_hi: u16,
    },
    /// A later host reset would not be seen. `reason` is one of the `PLAT_PCIE_OFF_*`.
    DetectorOff {
        reason: u8,
    },
    /// Bits outside the state's fields are set, or the state is not one of the above.
    Unknown(u32),
}
impl PcieState {
    pub const fn from_word(word: u32) -> Self {
        let (state, reason, high) = (word & 0xff, (word >> 8) & 0xff, word >> 16);
        match (state, reason, high) {
            (PLAT_PCIE_NOT_STARTED, 0, 0) => Self::NotStarted,
            (PLAT_PCIE_INITIALIZING, 0, 0) => Self::Initializing,
            (PLAT_PCIE_UP, 0, 0) => Self::Up,
            (PLAT_PCIE_LOST, 0, _) => Self::Lost {
                monitor2_hi: high as u16,
            },
            (PLAT_PCIE_DETECTOR_OFF, _, 0) => Self::DetectorOff {
                reason: reason as u8,
            },
            _ => Self::Unknown(word),
        }
    }
}
/// `Up`, `Lost(monitor2_hi=0x001b)`, `DetectorOff(spurious_storm)`, `Unknown(0x00000105)`.
impl core::fmt::Display for PcieState {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match *self {
            Self::NotStarted => f.write_str("NotStarted"),
            Self::Initializing => f.write_str("Initializing"),
            Self::Up => f.write_str("Up"),
            Self::Lost { monitor2_hi } => write!(f, "Lost(monitor2_hi=0x{monitor2_hi:04x})"),
            Self::DetectorOff {
                reason: PLAT_PCIE_OFF_ARM_FAULT,
            } => f.write_str("DetectorOff(arm_fault)"),
            Self::DetectorOff {
                reason: PLAT_PCIE_OFF_SPURIOUS_STORM,
            } => f.write_str("DetectorOff(spurious_storm)"),
            Self::DetectorOff { reason } => write!(f, "DetectorOff(reason={reason})"),
            Self::Unknown(word) => write!(f, "Unknown(0x{word:08x})"),
        }
    }
}

/// A decoded `PLAT_SCMI` word.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScmiState {
    NotStarted,
    Serving { requests: u16 },
    Quarantined,
    Unknown(u32),
}
impl ScmiState {
    pub const fn from_word(word: u32) -> Self {
        match word {
            PLAT_SCMI_NOT_STARTED => Self::NotStarted,
            PLAT_SCMI_QUARANTINED => Self::Quarantined,
            _ if word & 0xffff == PLAT_SCMI_SERVING => Self::Serving {
                requests: (word >> 16) as u16,
            },
            _ => Self::Unknown(word),
        }
    }
}
/// `NotStarted`, `Serving(requests=12)`, `Quarantined`, `Unknown(0x00000003)`.
impl core::fmt::Display for ScmiState {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match *self {
            Self::NotStarted => f.write_str("NotStarted"),
            Self::Serving { requests } => write!(f, "Serving(requests={requests})"),
            Self::Quarantined => f.write_str("Quarantined"),
            Self::Unknown(word) => write!(f, "Unknown(0x{word:08x})"),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum CoreState {
    #[default]
    NotStarted,
    Running,
    /// The firmware called `rp1_rt::status::stop`.
    Stopped,
    Panicked,
    Fault,
    Unknown(u32),
}
impl CoreState {
    pub const fn from_word(word: u32) -> Self {
        match word {
            0 => Self::NotStarted,
            2 => Self::Running,
            3 => Self::Stopped,
            4 => Self::Panicked,
            5 => Self::Fault,
            w => Self::Unknown(w),
        }
    }
    pub const fn word(self) -> u32 {
        match self {
            Self::NotStarted => 0,
            Self::Running => 2,
            Self::Stopped => 3,
            Self::Panicked => 4,
            Self::Fault => 5,
            Self::Unknown(w) => w,
        }
    }
    /// The core wrote its details before this state and never leaves it.
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Stopped | Self::Panicked | Self::Fault)
    }
}

/// The `FILE` words for `path`: its last `FILE_LEN` bytes, NUL padded.
pub fn file_words(path: &str) -> [u32; FILE_LEN / 4] {
    let tail = &path.as_bytes()[path.len().saturating_sub(FILE_LEN)..];
    let mut bytes = [0; FILE_LEN];
    bytes[..tail.len()].copy_from_slice(tail);
    let mut words = [0; FILE_LEN / 4];
    for (w, b) in words.iter_mut().zip(bytes.as_chunks::<4>().0) {
        *w = u32::from_le_bytes(*b);
    }
    words
}
/// The bytes of the `FILE` words up to the first NUL.
pub fn file_bytes(words: &[u32; FILE_LEN / 4], out: &mut [u8; FILE_LEN]) -> usize {
    for (b, w) in out.as_chunks_mut::<4>().0.iter_mut().zip(words) {
        *b = w.to_le_bytes();
    }
    out.iter().position(|b| *b == 0).unwrap_or(FILE_LEN)
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use std::string::ToString;

    #[test]
    fn states_round_trip() {
        for w in 0..8 {
            assert_eq!(CoreState::from_word(w).word(), w);
        }
        assert_eq!(CoreState::from_word(1), CoreState::Unknown(1));
        assert!(CoreState::Panicked.is_terminal() && !CoreState::Running.is_terminal());
        assert_eq!(ABI_VALUE, 0x0003_4c52);
        assert_eq!(abi_version(ABI_VALUE), Some(3));
        assert_eq!(abi_version(abi_value(2)), Some(2));
        assert_eq!(abi_version(0), None);
        assert_eq!(abi_version(u32::MAX), None);
    }

    #[test]
    fn plat_pcie_decodes_strictly() {
        let cases = [
            (PLAT_PCIE_NOT_STARTED, PcieState::NotStarted, "NotStarted"),
            (
                PLAT_PCIE_INITIALIZING,
                PcieState::Initializing,
                "Initializing",
            ),
            (PLAT_PCIE_UP, PcieState::Up, "Up"),
            (
                PLAT_PCIE_LOST | 0x001b_0000,
                PcieState::Lost {
                    monitor2_hi: 0x001b,
                },
                "Lost(monitor2_hi=0x001b)",
            ),
            (
                PLAT_PCIE_DETECTOR_OFF | (PLAT_PCIE_OFF_ARM_FAULT as u32) << 8,
                PcieState::DetectorOff {
                    reason: PLAT_PCIE_OFF_ARM_FAULT,
                },
                "DetectorOff(arm_fault)",
            ),
            (
                PLAT_PCIE_DETECTOR_OFF | (PLAT_PCIE_OFF_SPURIOUS_STORM as u32) << 8,
                PcieState::DetectorOff {
                    reason: PLAT_PCIE_OFF_SPURIOUS_STORM,
                },
                "DetectorOff(spurious_storm)",
            ),
            (
                0x0904,
                PcieState::DetectorOff { reason: 9 },
                "DetectorOff(reason=9)",
            ),
        ];
        for (word, state, text) in cases {
            assert_eq!(PcieState::from_word(word), state, "{word:#x}");
            assert_eq!(state.to_string(), text);
        }
        for word in [5, 0x102, 0x1_0002, 0x1_0004, 0x103, u32::MAX] {
            assert_eq!(PcieState::from_word(word), PcieState::Unknown(word));
        }
        assert_eq!(PcieState::Unknown(0x105).to_string(), "Unknown(0x00000105)");
    }

    #[test]
    fn plat_scmi_decodes_strictly() {
        let cases = [
            (0, ScmiState::NotStarted, "NotStarted"),
            (
                PLAT_SCMI_SERVING,
                ScmiState::Serving { requests: 0 },
                "Serving(requests=0)",
            ),
            (
                PLAT_SCMI_SERVING | 12 << 16,
                ScmiState::Serving { requests: 12 },
                "Serving(requests=12)",
            ),
            (PLAT_SCMI_QUARANTINED, ScmiState::Quarantined, "Quarantined"),
            (3, ScmiState::Unknown(3), "Unknown(0x00000003)"),
            (0x101, ScmiState::Unknown(0x101), "Unknown(0x00000101)"),
        ];
        for (word, state, text) in cases {
            assert_eq!(ScmiState::from_word(word), state, "{word:#x}");
            assert_eq!(state.to_string(), text);
        }
    }

    #[test]
    fn file_keeps_the_tail() {
        let mut out = [0; FILE_LEN];
        for (path, tail) in [
            ("", ""),
            ("src/main.rs", "src/main.rs"),
            ("examples/dual-core/src/main.rs", "ual-core/src/main.rs"),
        ] {
            let n = file_bytes(&file_words(path), &mut out);
            assert_eq!(&out[..n], tail.as_bytes(), "{path}");
        }
    }
}
