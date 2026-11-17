//! Firmware status words in the link page (offsets from `LINK_START`, all `u32` LE).
//!
//! rp1-rt writes them, Linux reads them through BAR2. Each word has one writer: `EPOCH` and
//! `ABI` are proc0's, a core's block is that core's own.

/// Odd while proc0 clears the blocks at startup, even once they are valid; +2 per boot.
pub const EPOCH: usize = 0x40;
/// `ABI_VALUE` once proc0 has initialized the words of this layout.
pub const ABI: usize = 0x44;
pub const ABI_VALUE: u32 = u32::from_le_bytes([b'R', b'L', 2, 0]);
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
    use super::*;

    #[test]
    fn states_round_trip() {
        for w in 0..8 {
            assert_eq!(CoreState::from_word(w).word(), w);
        }
        assert_eq!(CoreState::from_word(1), CoreState::Unknown(1));
        assert!(CoreState::Panicked.is_terminal() && !CoreState::Running.is_terminal());
        assert_eq!(ABI_VALUE, 0x0002_4c52);
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
