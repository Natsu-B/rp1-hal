//! Lab-only words in the link page's 0x24..0x3f scratch range.
pub const READY: usize = 0x24;
pub const NONCE: usize = 0x28;
pub const REQUEST: usize = 0x2c;
pub const ACK: usize = 0x30;
pub const TICKS: [usize; 2] = [0x34, 0x38];
pub const MAGIC: u32 = u32::from_le_bytes(*b"FLT1");
pub const ALLOWED: &str = "p0-thread-panic,p0-isr-panic,p0-thread-udf,p0-isr-udf,p1-panic,stkerr";

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum Kind {
    P0ThreadPanic = 1,
    P0IsrPanic,
    P0ThreadUdf,
    P0IsrUdf,
    P1Panic,
    Stkerr,
}

impl Kind {
    const KINDS: [Self; 6] = [
        Self::P0ThreadPanic,
        Self::P0IsrPanic,
        Self::P0ThreadUdf,
        Self::P0IsrUdf,
        Self::P1Panic,
        Self::Stkerr,
    ];

    #[cfg(target_os = "none")]
    pub fn from_word(word: u32) -> Option<Self> {
        Self::KINDS.into_iter().find(|kind| *kind as u32 == word)
    }

    #[cfg(target_os = "linux")]
    pub fn parse(name: &str) -> Option<Self> {
        Self::KINDS
            .into_iter()
            .zip(ALLOWED.split(','))
            .find_map(|(kind, label)| (label == name).then_some(kind))
    }

    #[cfg(target_os = "linux")]
    pub fn core(self) -> usize {
        usize::from(self == Self::P1Panic)
    }
}
